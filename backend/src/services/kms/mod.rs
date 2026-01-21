//! Key Management Service (KMS) abstraction layer
//!
//! This module provides a pluggable KMS interface for envelope encryption
//! of sensitive data like OAuth tokens. It supports multiple KMS backends
//! configured via the `KMS_PROVIDER` environment variable.

use crate::models::DataKey;
use anyhow::Result;
use std::sync::Arc;

mod mock;
mod vault;

pub use mock::MockKmsProvider;
pub use vault::{VaultAuthMethod, VaultConfig, VaultKmsProvider};

/// Trait defining the interface for Key Management Service providers.
///
/// Implementations of this trait handle the lifecycle of data encryption keys
/// used in envelope encryption schemes. The KMS manages master keys and uses
/// them to encrypt/decrypt data keys, which are then used to encrypt actual data.
pub trait KmsProvider: Send + Sync {
    /// Generate a new data encryption key.
    ///
    /// Creates a new 256-bit data key that can be used for envelope encryption.
    /// The returned `DataKey` contains both the plaintext key (for immediate use)
    /// and the encrypted key (for storage).
    ///
    /// # Arguments
    /// * `key_id` - A unique identifier for this key, typically derived from
    ///              the user ID and provider (e.g., "user-{uuid}-spotify")
    ///
    /// # Returns
    /// * `Ok(DataKey)` - A new data key with plaintext and encrypted versions
    /// * `Err` - If key generation or encryption fails
    fn generate_data_key(&self, key_id: &str) -> Result<DataKey>;

    /// Decrypt a data encryption key.
    ///
    /// Decrypts a previously encrypted data key using the KMS master key.
    /// This is typically called when retrieving stored encrypted data.
    ///
    /// # Arguments
    /// * `encrypted_key` - The encrypted data key bytes
    /// * `key_id` - The identifier for this key (for audit/logging purposes)
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - The decrypted plaintext key
    /// * `Err` - If decryption fails (e.g., invalid key, tampered data)
    fn decrypt_data_key(&self, encrypted_key: &[u8], key_id: &str) -> Result<Vec<u8>>;

    /// Rotate the master key and re-encrypt a data key.
    ///
    /// This is used during key rotation to re-encrypt existing data keys
    /// under a new master key version. The plaintext data key remains the same,
    /// but its encrypted form is updated.
    ///
    /// # Arguments
    /// * `encrypted_key` - The currently encrypted data key
    /// * `key_id` - The identifier for this key
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - The re-encrypted data key under the new master key
    /// * `Err` - If rotation fails
    fn rotate_key(&self, encrypted_key: &[u8], key_id: &str) -> Result<Vec<u8>>;
}

/// Supported KMS provider types
#[derive(Debug, Clone, PartialEq)]
pub enum KmsProviderType {
    /// Mock provider for development and testing (uses local master key)
    Mock,
    /// HashiCorp Vault Transit secrets engine
    Vault,
    // Future providers:
    // AwsKms,
    // GcpKms,
    // AzureKeyVault,
}

impl KmsProviderType {
    /// Parse from environment variable value
    pub fn from_env(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "mock" | "local" | "dev" | "" => Some(Self::Mock),
            "vault" | "hashicorp" | "hashicorp_vault" => Some(Self::Vault),
            // "aws" | "aws_kms" => Some(Self::AwsKms),
            // "gcp" | "gcp_kms" => Some(Self::GcpKms),
            // "azure" | "azure_key_vault" => Some(Self::AzureKeyVault),
            _ => None,
        }
    }
}

/// Create a KMS provider based on configuration.
///
/// Reads the `KMS_PROVIDER` environment variable to determine which
/// KMS backend to use. Defaults to `MockKmsProvider` for development.
///
/// # Environment Variables
/// * `KMS_PROVIDER` - The provider type: "mock", "local", "dev", "vault", "hashicorp" (default: "mock")
///
/// For Vault provider, also requires:
/// * `VAULT_ADDR` - Vault server address
/// * `VAULT_TOKEN` or `VAULT_ROLE_ID`+`VAULT_SECRET_ID` - Authentication
///
/// # Returns
/// An `Arc<dyn KmsProvider>` that can be shared across threads
///
/// # Panics
/// Panics if Vault provider is requested but configuration is invalid.
pub fn create_kms_provider() -> Arc<dyn KmsProvider> {
    let provider_type = std::env::var("KMS_PROVIDER")
        .ok()
        .and_then(|v| KmsProviderType::from_env(&v))
        .unwrap_or(KmsProviderType::Mock);

    match provider_type {
        KmsProviderType::Mock => {
            tracing::info!("Using MockKmsProvider for key management");
            Arc::new(MockKmsProvider::new())
        }
        KmsProviderType::Vault => {
            tracing::info!("Using VaultKmsProvider for key management");
            match VaultKmsProvider::from_env() {
                Ok(provider) => Arc::new(provider),
                Err(e) => {
                    tracing::error!("Failed to create VaultKmsProvider: {}", e);
                    panic!("Failed to initialize Vault KMS provider: {}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kms_provider_type_from_env() {
        // Mock provider aliases
        assert_eq!(
            KmsProviderType::from_env("mock"),
            Some(KmsProviderType::Mock)
        );
        assert_eq!(
            KmsProviderType::from_env("Mock"),
            Some(KmsProviderType::Mock)
        );
        assert_eq!(
            KmsProviderType::from_env("MOCK"),
            Some(KmsProviderType::Mock)
        );
        assert_eq!(
            KmsProviderType::from_env("local"),
            Some(KmsProviderType::Mock)
        );
        assert_eq!(
            KmsProviderType::from_env("dev"),
            Some(KmsProviderType::Mock)
        );
        assert_eq!(KmsProviderType::from_env(""), Some(KmsProviderType::Mock));

        // Vault provider aliases
        assert_eq!(
            KmsProviderType::from_env("vault"),
            Some(KmsProviderType::Vault)
        );
        assert_eq!(
            KmsProviderType::from_env("Vault"),
            Some(KmsProviderType::Vault)
        );
        assert_eq!(
            KmsProviderType::from_env("VAULT"),
            Some(KmsProviderType::Vault)
        );
        assert_eq!(
            KmsProviderType::from_env("hashicorp"),
            Some(KmsProviderType::Vault)
        );
        assert_eq!(
            KmsProviderType::from_env("hashicorp_vault"),
            Some(KmsProviderType::Vault)
        );

        // Unknown provider
        assert_eq!(KmsProviderType::from_env("unknown"), None);
    }

    #[test]
    fn test_create_kms_provider_default() {
        // Without KMS_PROVIDER set, should default to Mock
        std::env::remove_var("KMS_PROVIDER");
        let provider = create_kms_provider();

        // Verify it works by generating a key
        let result = provider.generate_data_key("test-key");
        assert!(result.is_ok());
    }
}
