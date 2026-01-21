use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::error::oauth::{parse_provider_error, OAuthError};
use crate::error::{AppError, Result};
use crate::models::oauth::{
    OAuthConfig, OAuthFlowResponse, OAuthProviderType, OAuthTokens, OAuthUserInfo,
};
use crate::services::oauth::{BaseOAuthProvider, OAuthProvider};

/// Google OAuth provider implementation
pub struct GoogleOAuthProvider {
    base: BaseOAuthProvider,
}

impl GoogleOAuthProvider {
    /// Create a new Google OAuth provider with config
    pub fn from_config(config: OAuthConfig) -> Result<Self> {
        let base = BaseOAuthProvider::new(
            config,
            OAuthProviderType::Google,
            "https://oauth2.googleapis.com/token".to_string(),
            "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
            Some("https://oauth2.googleapis.com/revoke".to_string()),
        );

        let provider = Self { base };

        // Validate configuration on creation
        provider.validate_config()?;

        Ok(provider)
    }

    /// Test Google OAuth configuration by verifying client credentials
    pub async fn test_configuration(&self) -> Result<()> {
        // Test by making a request to Google's OAuth2 discovery document
        let discovery_url = "https://accounts.google.com/.well-known/openid_configuration";

        let response = self
            .base
            .client
            .get(discovery_url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                AppError::OAuth(OAuthError::NetworkError {
                    provider: OAuthProviderType::Google,
                    reason: format!("Failed to connect to Google OAuth API: {}", e),
                    is_transient: true,
                    retry_count: 0,
                })
            })?;

        if !response.status().is_success() {
            return Err(AppError::OAuth(OAuthError::ProviderUnavailable {
                provider: OAuthProviderType::Google,
                reason: "Google OAuth API is not accessible".to_string(),
                estimated_recovery: None,
                retry_after: Some(60),
            }));
        }

        let discovery_doc: serde_json::Value = response.json().await.map_err(|e| {
            AppError::OAuth(OAuthError::ProviderError {
                provider: OAuthProviderType::Google,
                error_code: "discovery_parse_error".to_string(),
                message: format!("Failed to parse Google discovery document: {}", e),
                details: None,
            })
        })?;

        // Verify that the endpoints we're using match Google's current endpoints
        let expected_auth_endpoint = discovery_doc["authorization_endpoint"]
            .as_str()
            .ok_or_else(|| {
                AppError::ExternalServiceError(
                    "Missing authorization_endpoint in Google discovery document".to_string(),
                )
            })?;
        let expected_token_endpoint =
            discovery_doc["token_endpoint"].as_str().ok_or_else(|| {
                AppError::ExternalServiceError(
                    "Missing token_endpoint in Google discovery document".to_string(),
                )
            })?;
        let expected_userinfo_endpoint =
            discovery_doc["userinfo_endpoint"].as_str().ok_or_else(|| {
                AppError::ExternalServiceError(
                    "Missing userinfo_endpoint in Google discovery document".to_string(),
                )
            })?;

        if self.base.auth_endpoint != expected_auth_endpoint {
            tracing::warn!(
                "Google auth endpoint mismatch: using {} but Google reports {}",
                self.base.auth_endpoint,
                expected_auth_endpoint
            );
        }

        if self.base.token_endpoint != expected_token_endpoint {
            tracing::warn!(
                "Google token endpoint mismatch: using {} but Google reports {}",
                self.base.token_endpoint,
                expected_token_endpoint
            );
        }

        if self.base.user_info_endpoint != expected_userinfo_endpoint {
            tracing::warn!(
                "Google userinfo endpoint mismatch: using {} but Google reports {}",
                self.base.user_info_endpoint,
                expected_userinfo_endpoint
            );
        }

        Ok(())
    }

    /// Create a Google OAuth provider with environment variables
    pub fn new() -> Result<Self> {
        let client_id =
            std::env::var("GOOGLE_CLIENT_ID").map_err(|_| AppError::ConfigurationError {
                message: "GOOGLE_CLIENT_ID environment variable is required".to_string(),
            })?;
        let client_secret =
            std::env::var("GOOGLE_CLIENT_SECRET").map_err(|_| AppError::ConfigurationError {
                message: "GOOGLE_CLIENT_SECRET environment variable is required".to_string(),
            })?;
        let redirect_uri =
            std::env::var("GOOGLE_REDIRECT_URI").map_err(|_| AppError::ConfigurationError {
                message: "GOOGLE_REDIRECT_URI environment variable is required".to_string(),
            })?;

        Self::with_credentials(client_id, client_secret, redirect_uri)
    }

    /// Create a Google OAuth provider with default configuration
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
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
            additional_params,
        };

        Self::from_config(config)
    }

    /// Parse Google user info response into standardized format
    fn parse_user_info(&self, user_data: Value) -> Result<OAuthUserInfo> {
        let provider_user_id = user_data["id"]
            .as_str()
            .ok_or_else(|| AppError::OAuthProviderError {
                provider: "Google".to_string(),
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
        if let Some(link) = user_data["link"].as_str() {
            provider_data.insert(
                "profile_link".to_string(),
                serde_json::Value::String(link.to_string()),
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
}

#[async_trait]
impl OAuthProvider for GoogleOAuthProvider {
    fn provider_type(&self) -> OAuthProviderType {
        OAuthProviderType::Google
    }

    async fn initiate_flow(&self, redirect_uri: &str) -> Result<OAuthFlowResponse> {
        let state = self.base.generate_state();

        // Google-specific parameters for better user experience
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
            code_verifier: None, // Google doesn't require PKCE for server-side apps
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
                    provider: OAuthProviderType::Google,
                    reason: format!("Google token exchange request failed: {}", e),
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

            // Use the new error parsing function
            return Err(AppError::OAuth(parse_provider_error(
                OAuthProviderType::Google,
                status.as_u16(),
                &error_text,
            )));
        }

        let token_response: serde_json::Value = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!("Failed to parse Google token response: {}", e))
        })?;

        // Parse the Google OAuth2 token response
        let access_token = token_response["access_token"]
            .as_str()
            .ok_or_else(|| AppError::OAuthProviderError {
                provider: "Google".to_string(),
                message: "Missing access_token in Google response".to_string(),
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

        // Validate that we received the expected token type
        if token_type.to_lowercase() != "bearer" {
            return Err(AppError::OAuthProviderError {
                provider: "Google".to_string(),
                message: format!("Unexpected token type from Google: {}", token_type),
            });
        }

        // Validate that we received an ID token (required for OpenID Connect)
        if id_token.is_none() {
            tracing::warn!(
                "Google did not return an ID token, OpenID Connect features may not work"
            );
        }

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
                    provider: OAuthProviderType::Google,
                    reason: format!("Google user info request failed: {}", e),
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

            // Handle specific Google API error responses
            if status == reqwest::StatusCode::UNAUTHORIZED {
                return Err(AppError::OAuth(OAuthError::InvalidToken {
                    provider: OAuthProviderType::Google,
                    token_type: crate::error::oauth::TokenType::AccessToken,
                    reason: "Google access token is invalid or expired".to_string(),
                }));
            }

            return Err(AppError::OAuth(OAuthError::UserInfoRetrievalFailed {
                provider: OAuthProviderType::Google,
                reason: format!(
                    "Google user info request failed with status {}: {}",
                    status, error_text
                ),
                missing_scopes: vec![],
            }));
        }

        let user_data: serde_json::Value = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!(
                "Failed to parse Google user info response: {}",
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
                    provider: OAuthProviderType::Google,
                    reason: format!("Google token refresh request failed: {}", e),
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

                // Handle specific Google refresh token errors
                match error_code {
                    "invalid_grant" => {
                        return Err(AppError::OAuth(OAuthError::TokenRefreshFailed {
                            provider: OAuthProviderType::Google,
                            reason: "Google refresh token is invalid or expired. User needs to re-authenticate.".to_string(),
                            requires_reauth: true,
                        }));
                    }
                    "invalid_client" => {
                        return Err(AppError::OAuth(OAuthError::InvalidConfiguration {
                            provider: OAuthProviderType::Google,
                            reason: "Google OAuth client credentials are invalid".to_string(),
                            validation_errors: vec![error_description.to_string()],
                        }));
                    }
                    _ => {
                        return Err(AppError::OAuth(OAuthError::TokenRefreshFailed {
                            provider: OAuthProviderType::Google,
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
                provider: OAuthProviderType::Google,
                reason: format!(
                    "Google token refresh failed with status {}: {}",
                    status, error_text
                ),
                requires_reauth: false,
            }));
        }

        let token_response: serde_json::Value = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!(
                "Failed to parse Google refresh response: {}",
                e
            ))
        })?;

        // Parse the token response
        let access_token = token_response["access_token"]
            .as_str()
            .ok_or_else(|| AppError::OAuthProviderError {
                provider: "Google".to_string(),
                message: "Missing access_token in Google refresh response".to_string(),
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
            id_token: None, // Refresh responses don't include ID tokens
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
                        "Google token revocation request failed: {}",
                        e
                    ))
                })?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());

                // Google revocation endpoint returns 200 for success, 400 for invalid token
                if status == reqwest::StatusCode::BAD_REQUEST {
                    // Token was already invalid/revoked, which is fine
                    tracing::debug!("Google token was already revoked or invalid");
                    return Ok(());
                }

                return Err(AppError::ExternalServiceError(format!(
                    "Google token revocation failed with status {}: {}",
                    status, error_text
                )));
            }
        }
        Ok(())
    }

    fn validate_config(&self) -> Result<()> {
        if self.base.config.client_id.is_empty() {
            return Err(AppError::OAuth(OAuthError::InvalidConfiguration {
                provider: OAuthProviderType::Google,
                reason: "Google OAuth client_id is required".to_string(),
                validation_errors: vec!["client_id is empty".to_string()],
            }));
        }

        if self.base.config.client_secret.is_empty() {
            return Err(AppError::OAuth(OAuthError::InvalidConfiguration {
                provider: OAuthProviderType::Google,
                reason: "Google OAuth client_secret is required".to_string(),
                validation_errors: vec!["client_secret is empty".to_string()],
            }));
        }

        if self.base.config.redirect_uri.is_empty() {
            return Err(AppError::OAuth(OAuthError::InvalidConfiguration {
                provider: OAuthProviderType::Google,
                reason: "Google OAuth redirect_uri is required".to_string(),
                validation_errors: vec!["redirect_uri is empty".to_string()],
            }));
        }

        // Validate redirect URI format
        if let Err(_) = reqwest::Url::parse(&self.base.config.redirect_uri) {
            return Err(AppError::ConfigurationError {
                message: "Google OAuth redirect_uri must be a valid URL".to_string(),
            });
        }

        // Validate that required scopes are present
        let required_scopes = ["openid", "email"];
        for required_scope in &required_scopes {
            if !self.base.config.scopes.iter().any(|s| s == required_scope) {
                return Err(AppError::ConfigurationError {
                    message: format!("Google OAuth requires '{}' scope", required_scope),
                });
            }
        }

        // Validate client_id format (Google client IDs have a specific format)
        // Skip validation in development mode
        let is_dev_mode =
            std::env::var("OAUTH_DEV_MODE").unwrap_or_else(|_| "false".to_string()) == "true";
        if !is_dev_mode
            && !self
                .base
                .config
                .client_id
                .ends_with(".apps.googleusercontent.com")
        {
            return Err(AppError::ConfigurationError {
                message: "Google OAuth client_id must end with '.apps.googleusercontent.com'"
                    .to_string(),
            });
        }

        Ok(())
    }
}

/// Google OAuth service for managing multiple Google OAuth operations
pub struct GoogleOAuthService {
    provider: GoogleOAuthProvider,
}

impl GoogleOAuthService {
    /// Create a new Google OAuth service
    pub fn new(provider: GoogleOAuthProvider) -> Self {
        Self { provider }
    }

    /// Get the underlying OAuth provider
    pub fn provider(&self) -> &GoogleOAuthProvider {
        &self.provider
    }

    /// Initiate Google OAuth flow with additional options
    pub async fn initiate_flow_with_options(
        &self,
        redirect_uri: &str,
        login_hint: Option<&str>,
        hd: Option<&str>, // Hosted domain for G Suite
    ) -> Result<OAuthFlowResponse> {
        let state = self.provider.base.generate_state();

        let mut additional_params = HashMap::new();
        additional_params.insert("access_type".to_string(), "offline".to_string());
        additional_params.insert("prompt".to_string(), "consent".to_string());
        additional_params.insert("include_granted_scopes".to_string(), "true".to_string());

        // Add optional parameters
        if let Some(hint) = login_hint {
            additional_params.insert("login_hint".to_string(), hint.to_string());
        }
        if let Some(domain) = hd {
            additional_params.insert("hd".to_string(), domain.to_string());
        }

        let authorization_url =
            self.provider
                .base
                .build_auth_url(redirect_uri, &state, Some(additional_params));

        Ok(OAuthFlowResponse {
            authorization_url,
            state,
            code_verifier: None,
        })
    }

    /// Validate Google ID token (if using OpenID Connect)
    pub async fn validate_id_token(&self, id_token: &str) -> Result<Value> {
        // Google's token info endpoint for validation
        let validation_url = format!(
            "https://oauth2.googleapis.com/tokeninfo?id_token={}",
            id_token
        );

        let response = self
            .provider
            .base
            .client
            .get(&validation_url)
            .send()
            .await
            .map_err(|e| {
                AppError::ExternalServiceError(format!("ID token validation request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::ExternalServiceError(format!(
                "ID token validation failed with status {}: {}",
                status, error_text
            )));
        }

        let token_info: Value = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!(
                "Failed to parse token validation response: {}",
                e
            ))
        })?;

        // Verify the token is for our client
        if let Some(aud) = token_info["aud"].as_str() {
            if aud != self.provider.base.config.client_id {
                return Err(AppError::OAuthProviderError {
                    provider: "Google".to_string(),
                    message: "ID token audience mismatch".to_string(),
                });
            }
        }

        Ok(token_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_google_provider() -> GoogleOAuthProvider {
        GoogleOAuthProvider::with_credentials(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            "http://localhost:3000/auth/google/callback".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn test_google_provider_creation() {
        let provider = create_test_google_provider();
        assert_eq!(provider.provider_type(), OAuthProviderType::Google);
    }

    #[test]
    fn test_google_provider_validation() {
        let provider = create_test_google_provider();
        assert!(provider.validate_config().is_ok());
    }

    #[test]
    fn test_google_provider_validation_missing_client_id() {
        let config = OAuthConfig {
            client_id: "".to_string(),
            client_secret: "test_secret".to_string(),
            redirect_uri: "http://localhost:3000/auth/callback".to_string(),
            scopes: vec!["openid".to_string(), "email".to_string()],
            additional_params: HashMap::new(),
        };

        let provider = GoogleOAuthProvider::from_config(config).unwrap();
        let result = provider.validate_config();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("client_id is required"));
    }

    #[test]
    fn test_google_provider_validation_missing_scope() {
        let config = OAuthConfig {
            client_id: "test_client_id".to_string(),
            client_secret: "test_secret".to_string(),
            redirect_uri: "http://localhost:3000/auth/callback".to_string(),
            scopes: vec!["profile".to_string()], // Missing required scopes
            additional_params: HashMap::new(),
        };

        let provider = GoogleOAuthProvider::from_config(config).unwrap();
        let result = provider.validate_config();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires 'openid' scope"));
    }

    #[tokio::test]
    async fn test_google_initiate_flow() {
        let provider = create_test_google_provider();
        let redirect_uri = "http://localhost:3000/auth/google/callback";

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
    fn test_google_parse_user_info() {
        let provider = create_test_google_provider();

        let user_data = serde_json::json!({
            "id": "123456789",
            "email": "test@example.com",
            "verified_email": true,
            "name": "Test User",
            "given_name": "Test",
            "family_name": "User",
            "picture": "https://example.com/avatar.jpg",
            "locale": "en",
            "hd": "example.com",
            "link": "https://plus.google.com/123456789"
        });

        let user_info = provider.parse_user_info(user_data).unwrap();

        assert_eq!(user_info.provider_user_id, "123456789");
        assert_eq!(user_info.email, Some("test@example.com".to_string()));
        assert_eq!(user_info.email_verified, Some(true));
        assert_eq!(user_info.display_name, Some("Test User".to_string()));
        assert_eq!(user_info.first_name, Some("Test".to_string()));
        assert_eq!(user_info.last_name, Some("User".to_string()));
        assert_eq!(
            user_info.avatar_url,
            Some("https://example.com/avatar.jpg".to_string())
        );
        assert_eq!(user_info.locale, Some("en".to_string()));

        // Check provider-specific data
        assert_eq!(
            user_info.provider_data.get("hosted_domain"),
            Some(&serde_json::Value::String("example.com".to_string()))
        );
        assert_eq!(
            user_info.provider_data.get("profile_link"),
            Some(&serde_json::Value::String(
                "https://plus.google.com/123456789".to_string()
            ))
        );
    }

    #[test]
    fn test_google_parse_user_info_missing_id() {
        let provider = create_test_google_provider();

        let user_data = serde_json::json!({
            "email": "test@example.com",
            "name": "Test User"
        });

        let result = provider.parse_user_info(user_data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing user ID"));
    }

    #[test]
    fn test_google_oauth_service() {
        let provider = create_test_google_provider();
        let service = GoogleOAuthService::new(provider);

        assert_eq!(
            service.provider().provider_type(),
            OAuthProviderType::Google
        );
    }

    #[tokio::test]
    async fn test_google_oauth_service_with_options() {
        let provider = create_test_google_provider();
        let service = GoogleOAuthService::new(provider);

        let redirect_uri = "http://localhost:3000/auth/google/callback";
        let login_hint = Some("user@example.com");
        let hd = Some("example.com");

        let flow_response = service
            .initiate_flow_with_options(redirect_uri, login_hint, hd)
            .await
            .unwrap();

        assert!(flow_response
            .authorization_url
            .contains("login_hint=user%40example.com"));
        assert!(flow_response.authorization_url.contains("hd=example.com"));
    }
}
