//! YouTube Music OAuth provider implementation
//!
//! YouTube Music uses Google OAuth 2.0 with YouTube-specific scopes.
//! The YouTube Data API v3 is used for library access and modifications.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::error::oauth::{parse_provider_error, OAuthError};
use crate::error::{AppError, Result};
use crate::models::oauth::{
    OAuthConfig, OAuthFlowResponse, OAuthProviderType, OAuthTokens, OAuthUserInfo,
};
use crate::services::oauth::{BaseOAuthProvider, OAuthProvider};

/// YouTube Music OAuth provider implementation
/// Uses Google OAuth 2.0 with YouTube-specific scopes
pub struct YouTubeMusicOAuthProvider {
    base: BaseOAuthProvider,
}

impl YouTubeMusicOAuthProvider {
    /// Create a new YouTube Music OAuth provider with config
    pub fn from_config(config: OAuthConfig) -> Result<Self> {
        let base = BaseOAuthProvider::new(
            config,
            OAuthProviderType::YouTubeMusic,
            "https://oauth2.googleapis.com/token".to_string(),
            "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
            Some("https://oauth2.googleapis.com/revoke".to_string()),
        );

        let provider = Self { base };
        provider.validate_config()?;
        Ok(provider)
    }

    /// Create a YouTube Music OAuth provider with environment variables
    pub fn new() -> Result<Self> {
        let client_id = std::env::var("YOUTUBE_MUSIC_CLIENT_ID")
            .or_else(|_| std::env::var("GOOGLE_CLIENT_ID"))
            .map_err(|_| AppError::ConfigurationError {
                message:
                    "YOUTUBE_MUSIC_CLIENT_ID or GOOGLE_CLIENT_ID environment variable is required"
                        .to_string(),
            })?;
        let client_secret = std::env::var("YOUTUBE_MUSIC_CLIENT_SECRET")
            .or_else(|_| std::env::var("GOOGLE_CLIENT_SECRET"))
            .map_err(|_| AppError::ConfigurationError {
                message: "YOUTUBE_MUSIC_CLIENT_SECRET or GOOGLE_CLIENT_SECRET environment variable is required".to_string(),
            })?;
        let redirect_uri = std::env::var("YOUTUBE_MUSIC_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:3000/auth/callback/youtube_music".to_string());

        Self::with_credentials(client_id, client_secret, redirect_uri)
    }

    /// Create a YouTube Music OAuth provider with credentials
    pub fn with_credentials(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
    ) -> Result<Self> {
        let mut additional_params = HashMap::new();
        additional_params.insert("access_type".to_string(), "offline".to_string());
        additional_params.insert("prompt".to_string(), "consent".to_string());

        let config = OAuthConfig {
            client_id,
            client_secret,
            redirect_uri,
            scopes: vec![
                // Required for user identification
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
                // YouTube-specific scopes for library access
                "https://www.googleapis.com/auth/youtube".to_string(),
                "https://www.googleapis.com/auth/youtube.force-ssl".to_string(),
                "https://www.googleapis.com/auth/youtube.readonly".to_string(),
            ],
            additional_params,
        };

        Self::from_config(config)
    }

    /// Parse YouTube/Google user info response into standardized format
    fn parse_user_info(&self, user_data: Value) -> Result<OAuthUserInfo> {
        let provider_user_id = user_data["id"]
            .as_str()
            .ok_or_else(|| AppError::OAuthProviderError {
                provider: "YouTubeMusic".to_string(),
                message: "Missing user ID in Google response".to_string(),
            })?
            .to_string();

        let email = user_data["email"].as_str().map(|s| s.to_string());
        let email_verified = user_data["verified_email"].as_bool();
        let display_name = user_data["name"].as_str().map(|s| s.to_string());
        let first_name = user_data["given_name"].as_str().map(|s| s.to_string());
        let last_name = user_data["family_name"].as_str().map(|s| s.to_string());
        let avatar_url = user_data["picture"].as_str().map(|s| s.to_string());
        let locale = user_data["locale"].as_str().map(|s| s.to_string());

        // Store additional Google-specific data
        let mut provider_data = HashMap::new();
        if let Some(hd) = user_data["hd"].as_str() {
            provider_data.insert(
                "hosted_domain".to_string(),
                serde_json::Value::String(hd.to_string()),
            );
        }

        Ok(OAuthUserInfo {
            provider_user_id,
            email,
            email_verified,
            display_name,
            first_name,
            last_name,
            avatar_url,
            locale,
            provider_data,
        })
    }

    /// Get YouTube channel ID for the authenticated user
    pub async fn get_youtube_channel_id(&self, access_token: &str) -> Result<String> {
        let response = self
            .base
            .client
            .get("https://www.googleapis.com/youtube/v3/channels")
            .query(&[("part", "id"), ("mine", "true")])
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                AppError::OAuth(OAuthError::NetworkError {
                    provider: OAuthProviderType::YouTubeMusic,
                    reason: format!("YouTube channel request failed: {}", e),
                    is_transient: true,
                    retry_count: 0,
                })
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::OAuth(OAuthError::UserInfoRetrievalFailed {
                provider: OAuthProviderType::YouTubeMusic,
                reason: format!(
                    "YouTube channel request failed with status {}: {}",
                    status, error_text
                ),
                missing_scopes: vec![],
            }));
        }

        let channel_data: serde_json::Value = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!(
                "Failed to parse YouTube channel response: {}",
                e
            ))
        })?;

        channel_data["items"]
            .as_array()
            .and_then(|items| items.first())
            .and_then(|item| item["id"].as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::OAuthProviderError {
                provider: "YouTubeMusic".to_string(),
                message: "No YouTube channel found for user".to_string(),
            })
    }
}

#[async_trait]
impl OAuthProvider for YouTubeMusicOAuthProvider {
    fn provider_type(&self) -> OAuthProviderType {
        OAuthProviderType::YouTubeMusic
    }

    async fn initiate_flow(&self, redirect_uri: &str) -> Result<OAuthFlowResponse> {
        let state = self.base.generate_state();

        // YouTube Music-specific parameters
        let mut additional_params = HashMap::new();
        additional_params.insert("access_type".to_string(), "offline".to_string());
        additional_params.insert("prompt".to_string(), "consent".to_string());
        additional_params.insert("include_granted_scopes".to_string(), "true".to_string());

        let authorization_url =
            self.base
                .build_auth_url(redirect_uri, &state, Some(additional_params));

        Ok(OAuthFlowResponse {
            authorization_url,
            state,
            code_verifier: None,
        })
    }

    async fn exchange_code(
        &self,
        code: &str,
        _state: &str,
        redirect_uri: &str,
    ) -> Result<OAuthTokens> {
        let params = [
            ("grant_type", "authorization_code"),
            ("client_id", &self.base.config.client_id),
            ("client_secret", &self.base.config.client_secret),
            ("code", code),
            ("redirect_uri", redirect_uri),
        ];

        let response = self
            .base
            .client
            .post(&self.base.token_endpoint)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                AppError::OAuth(OAuthError::NetworkError {
                    provider: OAuthProviderType::YouTubeMusic,
                    reason: format!("YouTube Music token exchange request failed: {}", e),
                    is_transient: true,
                    retry_count: 0,
                })
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            return Err(AppError::OAuth(parse_provider_error(
                OAuthProviderType::YouTubeMusic,
                status.as_u16(),
                &error_text,
            )));
        }

        let token_response: serde_json::Value = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!(
                "Failed to parse YouTube Music token response: {}",
                e
            ))
        })?;

        let access_token = token_response["access_token"]
            .as_str()
            .ok_or_else(|| AppError::OAuthProviderError {
                provider: "YouTubeMusic".to_string(),
                message: "Missing access_token in YouTube Music response".to_string(),
            })?
            .to_string();

        let refresh_token = token_response["refresh_token"]
            .as_str()
            .map(|s| s.to_string());

        let expires_in = token_response["expires_in"].as_i64();

        let token_type = token_response["token_type"]
            .as_str()
            .unwrap_or("Bearer")
            .to_string();

        let scope = token_response["scope"].as_str().map(|s| s.to_string());

        let id_token = token_response["id_token"].as_str().map(|s| s.to_string());

        Ok(OAuthTokens {
            access_token,
            refresh_token,
            expires_in,
            token_type,
            scope,
            id_token,
        })
    }

    async fn get_user_info(&self, access_token: &str) -> Result<OAuthUserInfo> {
        let response = self
            .base
            .client
            .get(&self.base.user_info_endpoint)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                AppError::OAuth(OAuthError::NetworkError {
                    provider: OAuthProviderType::YouTubeMusic,
                    reason: format!("YouTube Music user info request failed: {}", e),
                    is_transient: true,
                    retry_count: 0,
                })
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            if status == reqwest::StatusCode::UNAUTHORIZED {
                return Err(AppError::OAuth(OAuthError::InvalidToken {
                    provider: OAuthProviderType::YouTubeMusic,
                    token_type: crate::error::oauth::TokenType::AccessToken,
                    reason: "YouTube Music access token is invalid or expired".to_string(),
                }));
            }

            return Err(AppError::OAuth(OAuthError::UserInfoRetrievalFailed {
                provider: OAuthProviderType::YouTubeMusic,
                reason: format!(
                    "YouTube Music user info request failed with status {}: {}",
                    status, error_text
                ),
                missing_scopes: vec![],
            }));
        }

        let user_data: serde_json::Value = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!(
                "Failed to parse YouTube Music user info response: {}",
                e
            ))
        })?;

        self.parse_user_info(user_data)
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthTokens> {
        let params = [
            ("grant_type", "refresh_token"),
            ("client_id", &self.base.config.client_id),
            ("client_secret", &self.base.config.client_secret),
            ("refresh_token", refresh_token),
        ];

        let response = self
            .base
            .client
            .post(&self.base.token_endpoint)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                AppError::OAuth(OAuthError::NetworkError {
                    provider: OAuthProviderType::YouTubeMusic,
                    reason: format!("YouTube Music token refresh request failed: {}", e),
                    is_transient: true,
                    retry_count: 0,
                })
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Parse Google-specific error response
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                let error_code = error_json["error"].as_str().unwrap_or("unknown_error");
                let error_description = error_json["error_description"]
                    .as_str()
                    .unwrap_or(&error_text);

                match error_code {
                    "invalid_grant" => {
                        return Err(AppError::OAuth(OAuthError::TokenRefreshFailed {
                            provider: OAuthProviderType::YouTubeMusic,
                            reason: "YouTube Music refresh token is invalid or expired. User needs to re-authenticate.".to_string(),
                            requires_reauth: true,
                        }));
                    }
                    "invalid_client" => {
                        return Err(AppError::OAuth(OAuthError::InvalidConfiguration {
                            provider: OAuthProviderType::YouTubeMusic,
                            reason: "YouTube Music OAuth client credentials are invalid"
                                .to_string(),
                            validation_errors: vec![error_description.to_string()],
                        }));
                    }
                    _ => {
                        return Err(AppError::OAuth(OAuthError::TokenRefreshFailed {
                            provider: OAuthProviderType::YouTubeMusic,
                            reason: format!(
                                "Token refresh failed ({}): {}",
                                error_code, error_description
                            ),
                            requires_reauth: false,
                        }));
                    }
                }
            }

            return Err(AppError::OAuth(OAuthError::TokenRefreshFailed {
                provider: OAuthProviderType::YouTubeMusic,
                reason: format!(
                    "YouTube Music token refresh failed with status {}: {}",
                    status, error_text
                ),
                requires_reauth: false,
            }));
        }

        let token_response: serde_json::Value = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!(
                "Failed to parse YouTube Music refresh response: {}",
                e
            ))
        })?;

        let access_token = token_response["access_token"]
            .as_str()
            .ok_or_else(|| AppError::OAuthProviderError {
                provider: "YouTubeMusic".to_string(),
                message: "Missing access_token in YouTube Music refresh response".to_string(),
            })?
            .to_string();

        let refresh_token = token_response["refresh_token"]
            .as_str()
            .map(|s| s.to_string());

        let expires_in = token_response["expires_in"].as_i64();

        let token_type = token_response["token_type"]
            .as_str()
            .unwrap_or("Bearer")
            .to_string();

        let scope = token_response["scope"].as_str().map(|s| s.to_string());

        Ok(OAuthTokens {
            access_token,
            refresh_token,
            expires_in,
            token_type,
            scope,
            id_token: None,
        })
    }

    async fn revoke_token(&self, token: &str) -> Result<()> {
        if let Some(revoke_endpoint) = &self.base.revoke_endpoint {
            let params = [("token", token)];

            let response = self
                .base
                .client
                .post(revoke_endpoint)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .form(&params)
                .send()
                .await
                .map_err(|e| {
                    AppError::ExternalServiceError(format!(
                        "YouTube Music token revocation request failed: {}",
                        e
                    ))
                })?;

            if !response.status().is_success() {
                let status = response.status();
                // Google revocation endpoint returns 200 for success, 400 for invalid token
                if status == reqwest::StatusCode::BAD_REQUEST {
                    // Token was already invalid/revoked, which is fine
                    tracing::debug!("YouTube Music token was already revoked or invalid");
                    return Ok(());
                }

                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                return Err(AppError::ExternalServiceError(format!(
                    "YouTube Music token revocation failed with status {}: {}",
                    status, error_text
                )));
            }
        }
        Ok(())
    }

    fn validate_config(&self) -> Result<()> {
        if self.base.config.client_id.is_empty() {
            return Err(AppError::OAuth(OAuthError::InvalidConfiguration {
                provider: OAuthProviderType::YouTubeMusic,
                reason: "YouTube Music OAuth client_id is required".to_string(),
                validation_errors: vec!["client_id is empty".to_string()],
            }));
        }

        if self.base.config.client_secret.is_empty() {
            return Err(AppError::OAuth(OAuthError::InvalidConfiguration {
                provider: OAuthProviderType::YouTubeMusic,
                reason: "YouTube Music OAuth client_secret is required".to_string(),
                validation_errors: vec!["client_secret is empty".to_string()],
            }));
        }

        if self.base.config.redirect_uri.is_empty() {
            return Err(AppError::OAuth(OAuthError::InvalidConfiguration {
                provider: OAuthProviderType::YouTubeMusic,
                reason: "YouTube Music OAuth redirect_uri is required".to_string(),
                validation_errors: vec!["redirect_uri is empty".to_string()],
            }));
        }

        // Validate redirect URI format
        if reqwest::Url::parse(&self.base.config.redirect_uri).is_err() {
            return Err(AppError::ConfigurationError {
                message: "YouTube Music OAuth redirect_uri must be a valid URL".to_string(),
            });
        }

        // Validate that required YouTube scopes are present
        let has_youtube_scope = self
            .base
            .config
            .scopes
            .iter()
            .any(|s| s.contains("youtube"));
        if !has_youtube_scope {
            return Err(AppError::ConfigurationError {
                message: "YouTube Music OAuth requires at least one YouTube scope".to_string(),
            });
        }

        Ok(())
    }
}

/// YouTube Music OAuth service for managing OAuth operations
pub struct YouTubeMusicOAuthService {
    provider: YouTubeMusicOAuthProvider,
}

impl YouTubeMusicOAuthService {
    /// Create a new YouTube Music OAuth service
    pub fn new(provider: YouTubeMusicOAuthProvider) -> Self {
        Self { provider }
    }

    /// Get the underlying OAuth provider
    pub fn provider(&self) -> &YouTubeMusicOAuthProvider {
        &self.provider
    }

    /// Get YouTube channel ID for the authenticated user
    pub async fn get_youtube_channel_id(&self, access_token: &str) -> Result<String> {
        self.provider.get_youtube_channel_id(access_token).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_youtube_music_provider() -> YouTubeMusicOAuthProvider {
        YouTubeMusicOAuthProvider::with_credentials(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            "http://localhost:3000/auth/youtube_music/callback".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn test_youtube_music_provider_creation() {
        let provider = create_test_youtube_music_provider();
        assert_eq!(provider.provider_type(), OAuthProviderType::YouTubeMusic);
    }

    #[test]
    fn test_youtube_music_provider_validation() {
        let provider = create_test_youtube_music_provider();
        assert!(provider.validate_config().is_ok());
    }

    #[test]
    fn test_youtube_music_provider_validation_missing_client_id() {
        let mut additional_params = HashMap::new();
        additional_params.insert("access_type".to_string(), "offline".to_string());

        let config = OAuthConfig {
            client_id: "".to_string(),
            client_secret: "test_secret".to_string(),
            redirect_uri: "http://localhost:3000/auth/callback".to_string(),
            scopes: vec!["https://www.googleapis.com/auth/youtube".to_string()],
            additional_params,
        };

        let result = YouTubeMusicOAuthProvider::from_config(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_youtube_music_provider_validation_missing_youtube_scope() {
        let config = OAuthConfig {
            client_id: "test_client_id".to_string(),
            client_secret: "test_secret".to_string(),
            redirect_uri: "http://localhost:3000/auth/callback".to_string(),
            scopes: vec!["openid".to_string(), "email".to_string()],
            additional_params: HashMap::new(),
        };

        let result = YouTubeMusicOAuthProvider::from_config(config);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_youtube_music_initiate_flow() {
        let provider = create_test_youtube_music_provider();
        let redirect_uri = "http://localhost:3000/auth/youtube_music/callback";

        let flow_response = provider.initiate_flow(redirect_uri).await.unwrap();

        assert!(flow_response
            .authorization_url
            .contains("accounts.google.com"));
        assert!(flow_response
            .authorization_url
            .contains("client_id=test_client_id"));
        assert!(flow_response.authorization_url.contains("redirect_uri="));
        assert!(flow_response
            .authorization_url
            .contains("access_type=offline"));
        assert!(flow_response.authorization_url.contains("prompt=consent"));
        assert!(!flow_response.state.is_empty());
        assert!(flow_response.code_verifier.is_none());
    }

    #[test]
    fn test_youtube_music_parse_user_info() {
        let provider = create_test_youtube_music_provider();

        let user_data = serde_json::json!({
            "id": "123456789",
            "email": "test@example.com",
            "verified_email": true,
            "name": "Test User",
            "given_name": "Test",
            "family_name": "User",
            "picture": "https://example.com/avatar.jpg",
            "locale": "en"
        });

        let user_info = provider.parse_user_info(user_data).unwrap();

        assert_eq!(user_info.provider_user_id, "123456789");
        assert_eq!(user_info.email, Some("test@example.com".to_string()));
        assert_eq!(user_info.email_verified, Some(true));
        assert_eq!(user_info.display_name, Some("Test User".to_string()));
        assert_eq!(user_info.first_name, Some("Test".to_string()));
        assert_eq!(user_info.last_name, Some("User".to_string()));
    }

    #[test]
    fn test_youtube_music_oauth_service() {
        let provider = create_test_youtube_music_provider();
        let service = YouTubeMusicOAuthService::new(provider);

        assert_eq!(
            service.provider().provider_type(),
            OAuthProviderType::YouTubeMusic
        );
    }
}
