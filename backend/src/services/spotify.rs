use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, RefreshToken, Scope, TokenResponse, TokenUrl,
};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::models::{Connection, ConnectionStatus, StreamingProvider, TokenHealthCheck, DecryptedToken};
use crate::services::TokenVaultService;

/// Spotify OAuth configuration
#[derive(Debug, Clone)]
pub struct SpotifyConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub auth_url: String,
    pub token_url: String,
}

impl Default for SpotifyConfig {
    fn default() -> Self {
        Self {
            client_id: std::env::var("SPOTIFY_CLIENT_ID")
                .unwrap_or_else(|_| "your_spotify_client_id".to_string()),
            client_secret: std::env::var("SPOTIFY_CLIENT_SECRET")
                .unwrap_or_else(|_| "your_spotify_client_secret".to_string()),
            redirect_uri: std::env::var("SPOTIFY_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:3000/auth/spotify/callback".to_string()),
            auth_url: "https://accounts.spotify.com/authorize".to_string(),
            token_url: "https://accounts.spotify.com/api/token".to_string(),
        }
    }
}

/// Spotify OAuth authorization URL response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyAuthUrl {
    pub auth_url: String,
    pub state: String,
    pub code_verifier: String,
}

/// Spotify OAuth callback request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyCallbackRequest {
    pub code: String,
    pub state: String,
    pub code_verifier: String,
}

/// Spotify user profile from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyUserProfile {
    pub id: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub country: Option<String>,
    pub product: Option<String>,
}

/// Rate limiting state for Spotify API
#[derive(Debug, Clone)]
pub struct RateLimit {
    pub remaining: u32,
    pub reset_at: DateTime<Utc>,
    pub retry_after: Option<u32>,
}

/// Spotify API client with OAuth and rate limiting
pub struct SpotifyService {
    config: SpotifyConfig,
    oauth_client: BasicClient,
    http_client: Client,
    token_vault: Arc<TokenVaultService>,
    rate_limits: Arc<RwLock<HashMap<String, RateLimit>>>,
    pending_auth_sessions: Arc<RwLock<HashMap<String, PkceCodeVerifier>>>,
}

impl SpotifyService {
    pub fn new(config: SpotifyConfig, token_vault: Arc<TokenVaultService>) -> Result<Self> {
        let oauth_client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new(config.auth_url.clone())?,
            Some(TokenUrl::new(config.token_url.clone())?),
        )
        .set_redirect_uri(RedirectUrl::new(config.redirect_uri.clone())?);

        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            config,
            oauth_client,
            http_client,
            token_vault,
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            pending_auth_sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Generate Spotify OAuth authorization URL with PKCE
    pub async fn get_auth_url(&self) -> Result<SpotifyAuthUrl> {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = self
            .oauth_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("user-read-private".to_string()))
            .add_scope(Scope::new("user-read-email".to_string()))
            .add_scope(Scope::new("user-library-read".to_string()))
            .add_scope(Scope::new("user-library-modify".to_string()))
            .add_scope(Scope::new("playlist-read-private".to_string()))
            .add_scope(Scope::new("playlist-modify-private".to_string()))
            .add_scope(Scope::new("playlist-modify-public".to_string()))
            .add_scope(Scope::new("user-follow-read".to_string()))
            .add_scope(Scope::new("user-follow-modify".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        let state = csrf_token.secret().clone();

        // Store the PKCE verifier for later use
        self.pending_auth_sessions
            .write()
            .await
            .insert(state.clone(), pkce_verifier);

        Ok(SpotifyAuthUrl {
            auth_url: auth_url.to_string(),
            state,
            code_verifier: "stored_internally".to_string(), // Don't expose the actual verifier
        })
    }

    /// Handle OAuth callback and exchange code for tokens
    pub async fn handle_callback(
        &self,
        user_id: Uuid,
        callback_request: SpotifyCallbackRequest,
    ) -> Result<Connection> {
        // Retrieve the stored PKCE verifier
        let pkce_verifier = {
            let mut sessions = self.pending_auth_sessions.write().await;
            sessions
                .remove(&callback_request.state)
                .ok_or_else(|| anyhow!("Invalid or expired OAuth state"))?
        };

        // Exchange authorization code for tokens
        let token_result = self
            .oauth_client
            .exchange_code(AuthorizationCode::new(callback_request.code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
            .map_err(|e| anyhow!("Token exchange failed: {}", e))?;

        let access_token = token_result.access_token().secret().clone();
        let refresh_token = token_result
            .refresh_token()
            .map(|rt| rt.secret().clone());
        let expires_at = token_result
            .expires_in()
            .map(|duration| Utc::now() + chrono::Duration::seconds(duration.as_secs() as i64));

        // Get user profile to obtain Spotify user ID
        let user_profile = self.get_user_profile(&access_token).await?;

        // Determine scopes (Spotify doesn't return scopes in token response)
        let scopes = vec![
            "user-read-private".to_string(),
            "user-read-email".to_string(),
            "user-library-read".to_string(),
            "user-library-modify".to_string(),
            "playlist-read-private".to_string(),
            "playlist-modify-private".to_string(),
            "playlist-modify-public".to_string(),
            "user-follow-read".to_string(),
            "user-follow-modify".to_string(),
        ];

        // Store the connection using the token vault
        let store_request = crate::models::StoreTokenRequest {
            user_id,
            provider: StreamingProvider::Spotify,
            provider_user_id: user_profile.id,
            access_token,
            refresh_token,
            scopes,
            expires_at,
        };

        self.token_vault.store_token(store_request).await
    }

    /// Get Spotify user profile
    async fn get_user_profile(&self, access_token: &str) -> Result<SpotifyUserProfile> {
        let response = self
            .http_client
            .get("https://api.spotify.com/v1/me")
            .bearer_auth(access_token)
            .send()
            .await?;

        if response.status().is_success() {
            let profile: SpotifyUserProfile = response.json().await?;
            Ok(profile)
        } else {
            Err(anyhow!(
                "Failed to get user profile: {}",
                response.status()
            ))
        }
    }

    /// Check token health and validity
    pub async fn check_token_health(&self, connection: &Connection) -> Result<TokenHealthCheck> {
        let decrypted_token = self
            .token_vault
            .get_decrypted_token(connection.id)
            .await?;

        let response = self
            .http_client
            .get("https://api.spotify.com/v1/me")
            .bearer_auth(&decrypted_token.access_token)
            .send()
            .await?;

        let is_valid = response.status().is_success();
        let needs_refresh = connection.needs_refresh();

        Ok(TokenHealthCheck {
            connection_id: connection.id,
            is_valid,
            expires_at: connection.expires_at,
            error_message: if !is_valid {
                Some(format!("HTTP {}", response.status()))
            } else {
                None
            },
            checked_at: Utc::now(),
            needs_refresh,
        })
    }

    /// Refresh access token using refresh token
    pub async fn refresh_token(&self, connection: &mut Connection) -> Result<()> {
        let decrypted_token = self
            .token_vault
            .get_decrypted_token(connection.id)
            .await?;

        let refresh_token = decrypted_token
            .refresh_token
            .clone()
            .ok_or_else(|| anyhow!("No refresh token available"))?;

        let token_result = self
            .oauth_client
            .exchange_refresh_token(&RefreshToken::new(refresh_token))
            .request_async(async_http_client)
            .await
            .map_err(|e| anyhow!("Token refresh failed: {}", e))?;

        let new_access_token = token_result.access_token().secret().clone();
        let new_refresh_token = token_result
            .refresh_token()
            .map(|rt| rt.secret().clone())
            .or(decrypted_token.refresh_token); // Keep old refresh token if new one not provided
        let new_expires_at = token_result
            .expires_in()
            .map(|duration| Utc::now() + chrono::Duration::seconds(duration.as_secs() as i64));

        // Update the connection with new tokens
        let store_request = crate::models::StoreTokenRequest {
            user_id: connection.user_id,
            provider: StreamingProvider::Spotify,
            provider_user_id: connection.provider_user_id.clone(),
            access_token: new_access_token,
            refresh_token: new_refresh_token,
            scopes: connection.scopes.clone(),
            expires_at: new_expires_at,
        };

        let updated_connection = self.token_vault.store_token(store_request).await?;
        
        // Update the connection object
        connection.access_token_encrypted = updated_connection.access_token_encrypted;
        connection.refresh_token_encrypted = updated_connection.refresh_token_encrypted;
        connection.expires_at = updated_connection.expires_at;
        connection.token_version = updated_connection.token_version;
        connection.status = ConnectionStatus::Active;
        connection.error_code = None;
        connection.updated_at = Utc::now();

        Ok(())
    }

    /// Make authenticated API request with rate limiting
    pub async fn make_api_request(
        &self,
        connection: &Connection,
        method: &str,
        url: &str,
        body: Option<serde_json::Value>,
    ) -> Result<reqwest::Response> {
        // Check and wait for rate limits
        self.wait_for_rate_limit().await?;

        // Get decrypted token
        let decrypted_token = self
            .token_vault
            .get_decrypted_token(connection.id)
            .await?;

        // Build request
        let mut request_builder = match method.to_uppercase().as_str() {
            "GET" => self.http_client.get(url),
            "POST" => self.http_client.post(url),
            "PUT" => self.http_client.put(url),
            "DELETE" => self.http_client.delete(url),
            _ => return Err(anyhow!("Unsupported HTTP method: {}", method)),
        };

        request_builder = request_builder.bearer_auth(&decrypted_token.access_token);

        if let Some(json_body) = body {
            request_builder = request_builder.json(&json_body);
        }

        let response = request_builder.send().await?;

        // Update rate limit information from response headers
        self.update_rate_limit_from_response(&response).await;

        // Handle rate limiting
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(60);

            self.set_rate_limit_retry_after(retry_after).await;
            return Err(anyhow!("Rate limited, retry after {} seconds", retry_after));
        }

        Ok(response)
    }

    /// Wait for rate limit to reset if necessary
    async fn wait_for_rate_limit(&self) -> Result<()> {
        let rate_limits = self.rate_limits.read().await;
        if let Some(rate_limit) = rate_limits.get("global") {
            if rate_limit.remaining == 0 && Utc::now() < rate_limit.reset_at {
                let wait_duration = (rate_limit.reset_at - Utc::now()).to_std()?;
                drop(rate_limits); // Release the lock before sleeping
                tokio::time::sleep(wait_duration).await;
            }
        }
        Ok(())
    }

    /// Update rate limit information from API response headers
    async fn update_rate_limit_from_response(&self, response: &reqwest::Response) {
        let mut rate_limits = self.rate_limits.write().await;
        
        // Spotify doesn't provide standard rate limit headers, so we use a simple approach
        // In a real implementation, you'd parse actual Spotify rate limit headers
        let remaining = response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(100); // Default assumption

        let reset_at = Utc::now() + chrono::Duration::seconds(60); // Default 1-minute window

        rate_limits.insert(
            "global".to_string(),
            RateLimit {
                remaining,
                reset_at,
                retry_after: None,
            },
        );
    }

    /// Set rate limit retry-after from 429 response
    async fn set_rate_limit_retry_after(&self, retry_after_seconds: u32) {
        let mut rate_limits = self.rate_limits.write().await;
        let reset_at = Utc::now() + chrono::Duration::seconds(retry_after_seconds as i64);
        
        rate_limits.insert(
            "global".to_string(),
            RateLimit {
                remaining: 0,
                reset_at,
                retry_after: Some(retry_after_seconds),
            },
        );
    }

    /// Get current connection for a user
    pub async fn get_user_connection(&self, user_id: Uuid) -> Result<Option<Connection>> {
        let connections = self.token_vault.get_user_connections(user_id).await;
        Ok(connections
            .into_iter()
            .find(|c| c.provider == StreamingProvider::Spotify))
    }

    /// Disconnect user's Spotify account
    pub async fn disconnect_user(&self, user_id: Uuid) -> Result<()> {
        if let Some(connection) = self.get_user_connection(user_id).await? {
            // Revoke the token with Spotify if possible
            let _ = self.revoke_token(&connection).await; // Don't fail if revocation fails
            
            // Remove from token vault
            self.token_vault.delete_connection(connection.id).await?;
        }
        Ok(())
    }

    /// Revoke token with Spotify
    async fn revoke_token(&self, connection: &Connection) -> Result<()> {
        let decrypted_token = self
            .token_vault
            .get_decrypted_token(connection.id)
            .await?;

        // Spotify doesn't have a standard revocation endpoint, but we can try to revoke
        // This is a placeholder - in practice, you might just delete the token locally
        tracing::info!(
            "Revoking Spotify token for connection {}",
            connection.id
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_spotify_config_default() {
        let config = SpotifyConfig::default();
        assert!(!config.client_id.is_empty());
        assert!(!config.redirect_uri.is_empty());
        assert_eq!(config.auth_url, "https://accounts.spotify.com/authorize");
        assert_eq!(config.token_url, "https://accounts.spotify.com/api/token");
    }

    #[tokio::test]
    async fn test_get_auth_url() {
        let config = SpotifyConfig::default();
        let token_vault = Arc::new(TokenVaultService::new());
        let service = SpotifyService::new(config, token_vault).unwrap();

        let auth_url_response = service.get_auth_url().await.unwrap();
        
        assert!(auth_url_response.auth_url.contains("accounts.spotify.com"));
        assert!(auth_url_response.auth_url.contains("client_id"));
        assert!(auth_url_response.auth_url.contains("code_challenge"));
        assert!(!auth_url_response.state.is_empty());
    }
}