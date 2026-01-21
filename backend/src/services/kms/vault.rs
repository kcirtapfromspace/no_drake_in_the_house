//! HashiCorp Vault KMS provider for production encryption.
//!
//! This provider integrates with HashiCorp Vault's Transit secrets engine
//! for envelope encryption of sensitive data like OAuth tokens.
//!
//! # Configuration
//!
//! Environment variables:
//! - `VAULT_ADDR`: Vault server address (e.g., "http://127.0.0.1:8200")
//! - `VAULT_TOKEN`: Vault authentication token (for token auth)
//! - `VAULT_ROLE_ID`: AppRole role ID (for AppRole auth)
//! - `VAULT_SECRET_ID`: AppRole secret ID (for AppRole auth)
//! - `VAULT_TRANSIT_KEY`: Transit key name (default: "token-vault")
//! - `VAULT_NAMESPACE`: Vault namespace (optional, for enterprise)

use super::KmsProvider;
use crate::models::DataKey;
use anyhow::{anyhow, Context, Result};
use base64::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Vault Transit API response wrapper
#[derive(Debug, Deserialize)]
struct VaultResponse<T> {
    data: Option<T>,
    errors: Option<Vec<String>>,
}

/// Response from Transit generate-data-key endpoint
#[derive(Debug, Deserialize)]
struct GenerateDataKeyResponse {
    plaintext: String,
    ciphertext: String,
}

/// Response from Transit decrypt endpoint
#[derive(Debug, Deserialize)]
struct DecryptResponse {
    plaintext: String,
}

/// Response from Transit rewrap endpoint
#[derive(Debug, Deserialize)]
struct RewrapResponse {
    ciphertext: String,
}

/// Response from AppRole login
#[derive(Debug, Deserialize)]
struct AppRoleLoginResponse {
    client_token: String,
    lease_duration: u64,
}

/// Request body for Transit encrypt endpoint
#[derive(Debug, Serialize)]
struct EncryptRequest {
    plaintext: String,
}

/// Request body for Transit decrypt endpoint
#[derive(Debug, Serialize)]
struct DecryptRequest {
    ciphertext: String,
}

/// Request body for Transit rewrap endpoint
#[derive(Debug, Serialize)]
struct RewrapRequest {
    ciphertext: String,
}

/// Request body for AppRole login
#[derive(Debug, Serialize)]
struct AppRoleLoginRequest {
    role_id: String,
    secret_id: String,
}

/// Authentication method for Vault
#[derive(Debug, Clone)]
pub enum VaultAuthMethod {
    /// Direct token authentication
    Token(String),
    /// AppRole authentication with role_id and secret_id
    AppRole { role_id: String, secret_id: String },
}

/// Configuration for Vault KMS provider
#[derive(Debug, Clone)]
pub struct VaultConfig {
    /// Vault server address (e.g., "http://127.0.0.1:8200")
    pub addr: String,
    /// Authentication method
    pub auth: VaultAuthMethod,
    /// Transit secrets engine mount path (default: "transit")
    pub transit_mount: String,
    /// Transit key name (default: "token-vault")
    pub transit_key: String,
    /// Vault namespace (optional, for enterprise)
    pub namespace: Option<String>,
}

impl VaultConfig {
    /// Create configuration from environment variables.
    ///
    /// Required:
    /// - `VAULT_ADDR`: Vault server address
    ///
    /// Authentication (one of):
    /// - `VAULT_TOKEN`: Direct token authentication
    /// - `VAULT_ROLE_ID` + `VAULT_SECRET_ID`: AppRole authentication
    ///
    /// Optional:
    /// - `VAULT_TRANSIT_MOUNT`: Transit mount path (default: "transit")
    /// - `VAULT_TRANSIT_KEY`: Transit key name (default: "token-vault")
    /// - `VAULT_NAMESPACE`: Vault namespace
    pub fn from_env() -> Result<Self> {
        let addr =
            std::env::var("VAULT_ADDR").context("VAULT_ADDR environment variable is required")?;

        let auth = if let Ok(token) = std::env::var("VAULT_TOKEN") {
            VaultAuthMethod::Token(token)
        } else {
            let role_id = std::env::var("VAULT_ROLE_ID")
                .context("Either VAULT_TOKEN or VAULT_ROLE_ID+VAULT_SECRET_ID is required")?;
            let secret_id = std::env::var("VAULT_SECRET_ID")
                .context("VAULT_SECRET_ID is required when using AppRole auth")?;
            VaultAuthMethod::AppRole { role_id, secret_id }
        };

        let transit_mount =
            std::env::var("VAULT_TRANSIT_MOUNT").unwrap_or_else(|_| "transit".to_string());
        let transit_key =
            std::env::var("VAULT_TRANSIT_KEY").unwrap_or_else(|_| "token-vault".to_string());
        let namespace = std::env::var("VAULT_NAMESPACE").ok();

        Ok(Self {
            addr,
            auth,
            transit_mount,
            transit_key,
            namespace,
        })
    }

    /// Create configuration for testing with a dev server
    #[cfg(test)]
    pub fn for_dev_server(addr: &str, token: &str) -> Self {
        Self {
            addr: addr.to_string(),
            auth: VaultAuthMethod::Token(token.to_string()),
            transit_mount: "transit".to_string(),
            transit_key: "token-vault".to_string(),
            namespace: None,
        }
    }
}

/// HashiCorp Vault KMS provider using Transit secrets engine.
///
/// This provider implements envelope encryption using Vault's Transit engine:
/// - Data keys are generated and encrypted by Vault
/// - Plaintext data keys are used locally for encryption
/// - Encrypted data keys are stored with the data
/// - Decryption requires calling Vault to unwrap the data key
///
/// # Features
/// - Supports both Token and AppRole authentication
/// - Automatic token refresh for AppRole
/// - Graceful error handling (returns errors, doesn't panic)
/// - Thread-safe with internal token caching
pub struct VaultKmsProvider {
    config: VaultConfig,
    client: Client,
    /// Cached token for API calls (refreshed on AppRole auth)
    token: Arc<RwLock<Option<String>>>,
}

impl VaultKmsProvider {
    /// Create a new Vault KMS provider with the given configuration.
    pub fn new(config: VaultConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        let initial_token = match &config.auth {
            VaultAuthMethod::Token(token) => Some(token.clone()),
            VaultAuthMethod::AppRole { .. } => None,
        };

        Self {
            config,
            client,
            token: Arc::new(RwLock::new(initial_token)),
        }
    }

    /// Create a Vault KMS provider from environment variables.
    pub fn from_env() -> Result<Self> {
        let config = VaultConfig::from_env()?;
        Ok(Self::new(config))
    }

    /// Get an authenticated token, performing AppRole login if needed.
    async fn get_token(&self) -> Result<String> {
        // Check for cached token
        {
            let token_guard = self.token.read().await;
            if let Some(ref token) = *token_guard {
                return Ok(token.clone());
            }
        }

        // Need to authenticate via AppRole
        match &self.config.auth {
            VaultAuthMethod::Token(token) => Ok(token.clone()),
            VaultAuthMethod::AppRole { role_id, secret_id } => {
                let token = self.approle_login(role_id, secret_id).await?;

                // Cache the token
                let mut token_guard = self.token.write().await;
                *token_guard = Some(token.clone());

                Ok(token)
            }
        }
    }

    /// Perform AppRole login to get a token.
    async fn approle_login(&self, role_id: &str, secret_id: &str) -> Result<String> {
        let url = format!("{}/v1/auth/approle/login", self.config.addr);

        let request = AppRoleLoginRequest {
            role_id: role_id.to_string(),
            secret_id: secret_id.to_string(),
        };

        let mut req_builder = self.client.post(&url).json(&request);

        if let Some(ref ns) = self.config.namespace {
            req_builder = req_builder.header("X-Vault-Namespace", ns);
        }

        let response = req_builder
            .send()
            .await
            .context("Failed to connect to Vault for AppRole login")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Vault AppRole login failed ({}): {}", status, body));
        }

        let vault_response: VaultResponse<AppRoleLoginResponse> = response
            .json()
            .await
            .context("Failed to parse Vault AppRole login response")?;

        if let Some(errors) = vault_response.errors {
            return Err(anyhow!("Vault AppRole login errors: {:?}", errors));
        }

        vault_response
            .data
            .map(|d| d.client_token)
            .ok_or_else(|| anyhow!("Vault AppRole login response missing auth data"))
    }

    /// Make an authenticated request to Vault, with retry on auth failure.
    async fn vault_request<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&impl Serialize>,
    ) -> Result<T> {
        let url = format!("{}/v1/{}", self.config.addr, path);
        let token = self.get_token().await?;

        let mut req_builder = self
            .client
            .request(method.clone(), &url)
            .header("X-Vault-Token", &token);

        if let Some(ref ns) = self.config.namespace {
            req_builder = req_builder.header("X-Vault-Namespace", ns);
        }

        if let Some(b) = body {
            req_builder = req_builder.json(b);
        }

        let response = req_builder
            .send()
            .await
            .context("Failed to connect to Vault")?;

        // Handle 403 by clearing cached token and retrying with AppRole
        if response.status().as_u16() == 403 {
            if matches!(self.config.auth, VaultAuthMethod::AppRole { .. }) {
                // Clear cached token to force re-authentication
                {
                    let mut token_guard = self.token.write().await;
                    *token_guard = None;
                }

                // Retry the request
                let new_token = self.get_token().await?;
                let mut retry_builder = self
                    .client
                    .request(method, &url)
                    .header("X-Vault-Token", &new_token);

                if let Some(ref ns) = self.config.namespace {
                    retry_builder = retry_builder.header("X-Vault-Namespace", ns);
                }

                if let Some(b) = body {
                    retry_builder = retry_builder.json(b);
                }

                let retry_response = retry_builder
                    .send()
                    .await
                    .context("Failed to connect to Vault on retry")?;

                if !retry_response.status().is_success() {
                    let status = retry_response.status();
                    let body_text = retry_response.text().await.unwrap_or_default();
                    return Err(anyhow!(
                        "Vault request failed after retry ({}): {}",
                        status,
                        body_text
                    ));
                }

                return retry_response
                    .json()
                    .await
                    .context("Failed to parse Vault response");
            }
        }

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Vault request failed ({}): {}", status, body_text));
        }

        response
            .json()
            .await
            .context("Failed to parse Vault response")
    }

    /// Generate a data encryption key using Vault Transit.
    ///
    /// Uses the `generate-data-key` endpoint to create a new key that is:
    /// - Generated securely by Vault
    /// - Returned as plaintext for local encryption
    /// - Also returned encrypted (wrapped) for storage
    async fn generate_data_key_async(&self, key_id: &str) -> Result<DataKey> {
        let path = format!(
            "{}/datakey/plaintext/{}",
            self.config.transit_mount, self.config.transit_key
        );

        let vault_response: VaultResponse<GenerateDataKeyResponse> = self
            .vault_request(reqwest::Method::POST, &path, None::<&()>)
            .await
            .context("Failed to generate data key from Vault")?;

        if let Some(errors) = vault_response.errors {
            return Err(anyhow!("Vault generate-data-key errors: {:?}", errors));
        }

        let data = vault_response
            .data
            .ok_or_else(|| anyhow!("Vault generate-data-key response missing data"))?;

        // Decode the base64 plaintext key
        let plaintext_key = BASE64_STANDARD
            .decode(&data.plaintext)
            .context("Failed to decode plaintext key from Vault")?;

        // The ciphertext is already in Vault's format (vault:v1:...)
        let encrypted_key = data.ciphertext.into_bytes();

        tracing::debug!(key_id = %key_id, "Generated data key via Vault Transit");

        Ok(DataKey::new(
            key_id.to_string(),
            plaintext_key,
            encrypted_key,
            1, // Initial version
        ))
    }

    /// Decrypt a data encryption key using Vault Transit.
    async fn decrypt_data_key_async(&self, encrypted_key: &[u8], key_id: &str) -> Result<Vec<u8>> {
        let ciphertext = String::from_utf8(encrypted_key.to_vec())
            .context("Encrypted key is not valid UTF-8 (expected Vault ciphertext format)")?;

        let path = format!(
            "{}/decrypt/{}",
            self.config.transit_mount, self.config.transit_key
        );

        let request = DecryptRequest { ciphertext };

        let vault_response: VaultResponse<DecryptResponse> = self
            .vault_request(reqwest::Method::POST, &path, Some(&request))
            .await
            .context("Failed to decrypt data key via Vault")?;

        if let Some(errors) = vault_response.errors {
            return Err(anyhow!("Vault decrypt errors: {:?}", errors));
        }

        let data = vault_response
            .data
            .ok_or_else(|| anyhow!("Vault decrypt response missing data"))?;

        let plaintext = BASE64_STANDARD
            .decode(&data.plaintext)
            .context("Failed to decode decrypted key from Vault")?;

        tracing::debug!(key_id = %key_id, "Decrypted data key via Vault Transit");

        Ok(plaintext)
    }

    /// Rotate (rewrap) a data encryption key to the latest key version.
    async fn rotate_key_async(&self, encrypted_key: &[u8], key_id: &str) -> Result<Vec<u8>> {
        let ciphertext = String::from_utf8(encrypted_key.to_vec())
            .context("Encrypted key is not valid UTF-8 (expected Vault ciphertext format)")?;

        let path = format!(
            "{}/rewrap/{}",
            self.config.transit_mount, self.config.transit_key
        );

        let request = RewrapRequest { ciphertext };

        let vault_response: VaultResponse<RewrapResponse> = self
            .vault_request(reqwest::Method::POST, &path, Some(&request))
            .await
            .context("Failed to rewrap data key via Vault")?;

        if let Some(errors) = vault_response.errors {
            return Err(anyhow!("Vault rewrap errors: {:?}", errors));
        }

        let data = vault_response
            .data
            .ok_or_else(|| anyhow!("Vault rewrap response missing data"))?;

        tracing::debug!(key_id = %key_id, "Rewrapped data key via Vault Transit");

        Ok(data.ciphertext.into_bytes())
    }
}

impl KmsProvider for VaultKmsProvider {
    fn generate_data_key(&self, key_id: &str) -> Result<DataKey> {
        // Create a runtime for the sync interface
        // In production, this should be called from an async context
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.generate_data_key_async(key_id))
        })
    }

    fn decrypt_data_key(&self, encrypted_key: &[u8], key_id: &str) -> Result<Vec<u8>> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(self.decrypt_data_key_async(encrypted_key, key_id))
        })
    }

    fn rotate_key(&self, encrypted_key: &[u8], key_id: &str) -> Result<Vec<u8>> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.rotate_key_async(encrypted_key, key_id))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_config_for_dev_server() {
        let config = VaultConfig::for_dev_server("http://127.0.0.1:8200", "root");
        assert_eq!(config.addr, "http://127.0.0.1:8200");
        assert!(matches!(config.auth, VaultAuthMethod::Token(ref t) if t == "root"));
        assert_eq!(config.transit_mount, "transit");
        assert_eq!(config.transit_key, "token-vault");
        assert!(config.namespace.is_none());
    }

    #[test]
    fn test_vault_config_from_env_token_auth() {
        // Set up environment
        std::env::set_var("VAULT_ADDR", "http://localhost:8200");
        std::env::set_var("VAULT_TOKEN", "test-token");
        std::env::remove_var("VAULT_ROLE_ID");
        std::env::remove_var("VAULT_SECRET_ID");

        let config = VaultConfig::from_env().unwrap();
        assert_eq!(config.addr, "http://localhost:8200");
        assert!(matches!(config.auth, VaultAuthMethod::Token(ref t) if t == "test-token"));

        // Clean up
        std::env::remove_var("VAULT_ADDR");
        std::env::remove_var("VAULT_TOKEN");
    }

    #[test]
    fn test_vault_config_from_env_approle_auth() {
        // Set up environment
        std::env::set_var("VAULT_ADDR", "http://localhost:8200");
        std::env::remove_var("VAULT_TOKEN");
        std::env::set_var("VAULT_ROLE_ID", "test-role-id");
        std::env::set_var("VAULT_SECRET_ID", "test-secret-id");

        let config = VaultConfig::from_env().unwrap();
        assert!(matches!(
            config.auth,
            VaultAuthMethod::AppRole { ref role_id, ref secret_id }
            if role_id == "test-role-id" && secret_id == "test-secret-id"
        ));

        // Clean up
        std::env::remove_var("VAULT_ADDR");
        std::env::remove_var("VAULT_ROLE_ID");
        std::env::remove_var("VAULT_SECRET_ID");
    }

    #[test]
    fn test_vault_config_from_env_missing_addr() {
        std::env::remove_var("VAULT_ADDR");
        std::env::remove_var("VAULT_TOKEN");
        std::env::remove_var("VAULT_ROLE_ID");

        let result = VaultConfig::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("VAULT_ADDR"));
    }

    #[test]
    fn test_vault_config_from_env_missing_auth() {
        std::env::set_var("VAULT_ADDR", "http://localhost:8200");
        std::env::remove_var("VAULT_TOKEN");
        std::env::remove_var("VAULT_ROLE_ID");
        std::env::remove_var("VAULT_SECRET_ID");

        let result = VaultConfig::from_env();
        assert!(result.is_err());

        // Clean up
        std::env::remove_var("VAULT_ADDR");
    }

    #[test]
    fn test_vault_provider_creation() {
        let config = VaultConfig::for_dev_server("http://127.0.0.1:8200", "root");
        let provider = VaultKmsProvider::new(config);

        // Just verify it can be created without panicking
        assert!(provider.config.addr.contains("8200"));
    }
}
