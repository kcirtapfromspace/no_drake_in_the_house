use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::oauth::{
    OAuthConfig, OAuthFlowResponse, OAuthProviderType, OAuthTokens, OAuthUserInfo,
};
use crate::services::oauth::OAuthProvider;

/// Spotify OAuth provider implementation
pub struct SpotifyOAuthProvider {
    config: OAuthConfig,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct SpotifyTokenResponse {
    access_token: String,
    token_type: String,
    scope: String,
    expires_in: i64,
    refresh_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SpotifyUserProfile {
    id: String,
    display_name: Option<String>,
    email: Option<String>,
    country: Option<String>,
    images: Option<Vec<SpotifyImage>>,
}

#[derive(Debug, Deserialize)]
struct SpotifyImage {
    url: String,
    height: Option<u32>,
    width: Option<u32>,
}

impl SpotifyOAuthProvider {
    /// Create a Spotify OAuth provider from an OAuthConfig
    pub fn from_config(config: OAuthConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    /// Create a Spotify OAuth provider with environment variables
    pub fn new() -> Result<Self> {
        let client_id =
            std::env::var("SPOTIFY_CLIENT_ID").map_err(|_| AppError::ConfigurationError {
                message: "SPOTIFY_CLIENT_ID environment variable is required".to_string(),
            })?;
        let client_secret =
            std::env::var("SPOTIFY_CLIENT_SECRET").map_err(|_| AppError::ConfigurationError {
                message: "SPOTIFY_CLIENT_SECRET environment variable is required".to_string(),
            })?;
        let redirect_uri = std::env::var("SPOTIFY_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:3000/auth/callback/spotify".to_string());

        let config = OAuthConfig {
            client_id,
            client_secret,
            redirect_uri,
            scopes: vec![
                "user-read-private".to_string(),
                "user-read-email".to_string(),
                "user-library-read".to_string(),
                "user-library-modify".to_string(),
                "playlist-read-private".to_string(),
                "playlist-modify-public".to_string(),
                "playlist-modify-private".to_string(),
            ],
            additional_params: HashMap::new(),
        };

        Ok(Self::from_config(config))
    }

    /// Generate a secure random state token
    fn generate_state(&self) -> String {
        Uuid::new_v4().to_string()
    }

    /// Build Spotify authorization URL
    fn build_auth_url(&self, redirect_uri: &str, state: &str) -> String {
        let scope_string = self.config.scopes.join(" ");
        let mut params = vec![
            ("client_id", self.config.client_id.as_str()),
            ("response_type", "code"),
            ("redirect_uri", redirect_uri),
            ("state", state),
            ("scope", scope_string.as_str()),
        ];

        // Add show_dialog parameter to force user to approve the app each time (optional)
        params.push(("show_dialog", "false"));

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        format!("https://accounts.spotify.com/authorize?{}", query_string)
    }
}

#[async_trait]
impl OAuthProvider for SpotifyOAuthProvider {
    fn provider_type(&self) -> OAuthProviderType {
        OAuthProviderType::Spotify
    }

    async fn initiate_flow(&self, redirect_uri: &str) -> Result<OAuthFlowResponse> {
        let state = self.generate_state();
        let auth_url = self.build_auth_url(redirect_uri, &state);

        Ok(OAuthFlowResponse {
            authorization_url: auth_url,
            state,
            code_verifier: None, // Spotify doesn't use PKCE in this implementation
        })
    }

    async fn exchange_code(
        &self,
        code: &str,
        state: &str,
        redirect_uri: &str,
    ) -> Result<OAuthTokens> {
        let token_url = "https://accounts.spotify.com/api/token";

        // Prepare the request body
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
        ];

        // Create Basic Auth header
        let auth_string = format!("{}:{}", self.config.client_id, self.config.client_secret);
        let auth_header = format!("Basic {}", general_purpose::STANDARD.encode(auth_string));

        let response = self
            .client
            .post(token_url)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                AppError::ExternalServiceError(format!("Failed to exchange code: {}", e))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalServiceError(format!(
                "Spotify token exchange failed: {}",
                error_text
            )));
        }

        let token_response: SpotifyTokenResponse = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!("Failed to parse token response: {}", e))
        })?;

        Ok(OAuthTokens {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            expires_in: Some(token_response.expires_in),
            token_type: token_response.token_type,
            scope: Some(token_response.scope),
            id_token: None,
        })
    }

    async fn get_user_info(&self, access_token: &str) -> Result<OAuthUserInfo> {
        let user_url = "https://api.spotify.com/v1/me";

        let response = self
            .client
            .get(user_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| {
                AppError::ExternalServiceError(format!("Failed to get user info: {}", e))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalServiceError(format!(
                "Spotify user info request failed: {}",
                error_text
            )));
        }

        let user_profile: SpotifyUserProfile = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!("Failed to parse user profile: {}", e))
        })?;

        // Get avatar URL from images
        let avatar_url = user_profile
            .images
            .as_ref()
            .and_then(|images| images.first())
            .map(|image| image.url.clone());

        let mut provider_data = HashMap::new();
        if let Some(country) = user_profile.country {
            provider_data.insert("country".to_string(), serde_json::Value::String(country));
        }

        Ok(OAuthUserInfo {
            provider_user_id: user_profile.id,
            email: user_profile.email,
            email_verified: Some(true), // Spotify emails are generally verified
            display_name: user_profile.display_name,
            first_name: None,
            last_name: None,
            avatar_url,
            locale: None,
            provider_data,
        })
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthTokens> {
        let token_url = "https://accounts.spotify.com/api/token";

        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ];

        // Create Basic Auth header
        let auth_string = format!("{}:{}", self.config.client_id, self.config.client_secret);
        let auth_header = format!("Basic {}", general_purpose::STANDARD.encode(auth_string));

        let response = self
            .client
            .post(token_url)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                AppError::ExternalServiceError(format!("Failed to refresh token: {}", e))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalServiceError(format!(
                "Spotify token refresh failed: {}",
                error_text
            )));
        }

        let token_response: SpotifyTokenResponse = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!("Failed to parse refresh response: {}", e))
        })?;

        Ok(OAuthTokens {
            access_token: token_response.access_token,
            refresh_token: token_response
                .refresh_token
                .or_else(|| Some(refresh_token.to_string())), // Keep old refresh token if new one not provided
            expires_in: Some(token_response.expires_in),
            token_type: token_response.token_type,
            scope: Some(token_response.scope),
            id_token: None,
        })
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.client_id.is_empty() {
            return Err(AppError::InvalidFieldValue {
                field: "client_id".to_string(),
                message: "Spotify client ID is required".to_string(),
            });
        }

        if self.config.client_secret.is_empty() {
            return Err(AppError::InvalidFieldValue {
                field: "client_secret".to_string(),
                message: "Spotify client secret is required".to_string(),
            });
        }

        if self.config.scopes.is_empty() {
            return Err(AppError::InvalidFieldValue {
                field: "scopes".to_string(),
                message: "At least one Spotify scope is required".to_string(),
            });
        }

        Ok(())
    }
}

/// Create a Spotify OAuth provider with default configuration
pub fn create_spotify_oauth_provider() -> Result<SpotifyOAuthProvider> {
    let client_id =
        std::env::var("SPOTIFY_CLIENT_ID").map_err(|_| AppError::InvalidFieldValue {
            field: "SPOTIFY_CLIENT_ID".to_string(),
            message: "SPOTIFY_CLIENT_ID environment variable is required".to_string(),
        })?;

    let client_secret =
        std::env::var("SPOTIFY_CLIENT_SECRET").map_err(|_| AppError::InvalidFieldValue {
            field: "SPOTIFY_CLIENT_SECRET".to_string(),
            message: "SPOTIFY_CLIENT_SECRET environment variable is required".to_string(),
        })?;

    // Default Spotify scopes for music library management
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

    let config = OAuthConfig {
        client_id,
        client_secret,
        redirect_uri: std::env::var("SPOTIFY_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:3000/auth/callback/spotify".to_string()),
        scopes,
        additional_params: HashMap::new(),
    };

    let provider = SpotifyOAuthProvider::from_config(config);
    provider.validate_config()?;

    Ok(provider)
}
