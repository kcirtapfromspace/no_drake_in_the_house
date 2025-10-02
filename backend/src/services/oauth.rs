use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::models::oauth::{
    OAuthProviderType, OAuthTokens, OAuthUserInfo, OAuthFlowResponse, 
    OAuthConfig, OAuthState
};
use crate::error::{AppError, Result};

/// Trait defining the interface for OAuth providers
#[async_trait]
pub trait OAuthProvider: Send + Sync {
    /// Get the provider type
    fn provider_type(&self) -> OAuthProviderType;
    
    /// Initiate OAuth flow and return authorization URL with state
    async fn initiate_flow(&self, redirect_uri: &str) -> Result<OAuthFlowResponse>;
    
    /// Exchange authorization code for tokens
    async fn exchange_code(&self, code: &str, state: &str, redirect_uri: &str) -> Result<OAuthTokens>;
    
    /// Get user information using access token
    async fn get_user_info(&self, access_token: &str) -> Result<OAuthUserInfo>;
    
    /// Refresh access token using refresh token
    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthTokens>;
    
    /// Revoke tokens (optional, not all providers support this)
    async fn revoke_token(&self, token: &str) -> Result<()> {
        // Default implementation - providers can override if they support revocation
        Ok(())
    }
    
    /// Validate provider-specific configuration
    fn validate_config(&self) -> Result<()>;
}

/// Base OAuth provider implementation with common functionality
pub struct BaseOAuthProvider {
    pub config: OAuthConfig,
    pub provider_type: OAuthProviderType,
    pub client: reqwest::Client,
    pub token_endpoint: String,
    pub auth_endpoint: String,
    pub user_info_endpoint: String,
    pub revoke_endpoint: Option<String>,
}

impl BaseOAuthProvider {
    pub fn new(
        config: OAuthConfig,
        provider_type: OAuthProviderType,
        token_endpoint: String,
        auth_endpoint: String,
        user_info_endpoint: String,
        revoke_endpoint: Option<String>,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            provider_type,
            client,
            token_endpoint,
            auth_endpoint,
            user_info_endpoint,
            revoke_endpoint,
        }
    }

    /// Generate a secure random state token
    pub fn generate_state(&self) -> String {
        Uuid::new_v4().to_string()
    }

    /// Build authorization URL with parameters
    pub fn build_auth_url(&self, redirect_uri: &str, state: &str, additional_params: Option<HashMap<String, String>>) -> String {
        let scope_string = self.config.scopes.join(" ");
        let mut params = vec![
            ("client_id", self.config.client_id.as_str()),
            ("redirect_uri", redirect_uri),
            ("response_type", "code"),
            ("state", state),
            ("scope", scope_string.as_str()),
        ];

        // Add provider-specific additional parameters
        let mut additional_param_pairs = Vec::new();
        if let Some(extra_params) = additional_params {
            for (key, value) in extra_params {
                additional_param_pairs.push((key, value));
            }
        }
        
        // Add configured additional parameters
        for (key, value) in &self.config.additional_params {
            additional_param_pairs.push((key.clone(), value.clone()));
        }

        // Convert additional params to string references
        let additional_refs: Vec<(&str, &str)> = additional_param_pairs
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        params.extend(additional_refs);

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        format!("{}?{}", self.auth_endpoint, query_string)
    }

    /// Exchange authorization code for tokens using standard OAuth2 flow
    pub async fn exchange_code_standard(&self, code: &str, redirect_uri: &str) -> Result<OAuthTokens> {
        let params = [
            ("grant_type", "authorization_code"),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("code", code),
            ("redirect_uri", redirect_uri),
        ];

        let response = self.client
            .post(&self.token_endpoint)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("Token exchange request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::ExternalServiceError(format!(
                "Token exchange failed with status {}: {}", 
                status, 
                error_text
            )));
        }

        let token_response: Value = response.json().await
            .map_err(|e| AppError::ExternalServiceError(format!("Failed to parse token response: {}", e)))?;

        // Parse the standard OAuth2 token response
        let access_token = token_response["access_token"]
            .as_str()
            .ok_or_else(|| AppError::ExternalServiceError("Missing access_token in response".to_string()))?
            .to_string();

        let refresh_token = token_response["refresh_token"]
            .as_str()
            .map(|s| s.to_string());

        let expires_in = token_response["expires_in"]
            .as_i64();

        let token_type = token_response["token_type"]
            .as_str()
            .unwrap_or("Bearer")
            .to_string();

        let scope = token_response["scope"]
            .as_str()
            .map(|s| s.to_string());

        let id_token = token_response["id_token"]
            .as_str()
            .map(|s| s.to_string());

        Ok(OAuthTokens {
            access_token,
            refresh_token,
            expires_in,
            token_type,
            scope,
            id_token,
        })
    }

    /// Get user info using access token with standard Bearer authentication
    pub async fn get_user_info_standard(&self, access_token: &str) -> Result<Value> {
        let response = self.client
            .get(&self.user_info_endpoint)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("User info request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::ExternalServiceError(format!(
                "User info request failed with status {}: {}", 
                status, 
                error_text
            )));
        }

        response.json().await
            .map_err(|e| AppError::ExternalServiceError(format!("Failed to parse user info response: {}", e)))
    }

    /// Refresh access token using refresh token
    pub async fn refresh_token_standard(&self, refresh_token: &str) -> Result<OAuthTokens> {
        let params = [
            ("grant_type", "refresh_token"),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("refresh_token", refresh_token),
        ];

        let response = self.client
            .post(&self.token_endpoint)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("Token refresh request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::ExternalServiceError(format!(
                "Token refresh failed with status {}: {}", 
                status, 
                error_text
            )));
        }

        let token_response: Value = response.json().await
            .map_err(|e| AppError::ExternalServiceError(format!("Failed to parse refresh response: {}", e)))?;

        // Parse the token response
        let access_token = token_response["access_token"]
            .as_str()
            .ok_or_else(|| AppError::ExternalServiceError("Missing access_token in refresh response".to_string()))?
            .to_string();

        let refresh_token = token_response["refresh_token"]
            .as_str()
            .map(|s| s.to_string());

        let expires_in = token_response["expires_in"]
            .as_i64();

        let token_type = token_response["token_type"]
            .as_str()
            .unwrap_or("Bearer")
            .to_string();

        let scope = token_response["scope"]
            .as_str()
            .map(|s| s.to_string());

        Ok(OAuthTokens {
            access_token,
            refresh_token,
            expires_in,
            token_type,
            scope,
            id_token: None, // Refresh responses typically don't include ID tokens
        })
    }

    /// Revoke token if provider supports it
    pub async fn revoke_token_standard(&self, token: &str) -> Result<()> {
        if let Some(revoke_endpoint) = &self.revoke_endpoint {
            let params = [
                ("token", token),
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
            ];

            let response = self.client
                .post(revoke_endpoint)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .form(&params)
                .send()
                .await
                .map_err(|e| AppError::ExternalServiceError(format!("Token revocation request failed: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                return Err(AppError::ExternalServiceError(format!(
                    "Token revocation failed with status {}: {}", 
                    status, 
                    error_text
                )));
            }
        }
        Ok(())
    }
}

/// OAuth state manager for CSRF protection
pub struct OAuthStateManager {
    // In a real implementation, this would use Redis or database storage
    // For now, we'll use an in-memory store with expiration
    states: dashmap::DashMap<String, OAuthState>,
}

impl OAuthStateManager {
    pub fn new() -> Self {
        Self {
            states: dashmap::DashMap::new(),
        }
    }

    /// Store OAuth state for validation
    pub fn store_state(&self, state: OAuthState) -> String {
        let state_token = state.state_token.clone();
        self.states.insert(state_token.clone(), state);
        state_token
    }

    /// Validate and consume OAuth state
    pub fn validate_and_consume_state(&self, state_token: &str, provider: &OAuthProviderType) -> Result<OAuthState> {
        let state = self.states.remove(state_token)
            .ok_or_else(|| AppError::OAuthStateValidationFailed)?
            .1;

        if !state.is_valid(state_token, provider) {
            return Err(AppError::OAuthStateValidationFailed);
        }

        Ok(state)
    }

    /// Clean up expired states (should be called periodically)
    pub fn cleanup_expired_states(&self) {
        let now = Utc::now();
        self.states.retain(|_, state| state.expires_at > now);
    }
}

impl Default for OAuthStateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_config() -> OAuthConfig {
        OAuthConfig {
            client_id: "test_client_id".to_string(),
            client_secret: "test_client_secret".to_string(),
            redirect_uri: "http://localhost:3000/auth/callback".to_string(),
            scopes: vec!["openid".to_string(), "email".to_string(), "profile".to_string()],
            additional_params: HashMap::new(),
        }
    }

    #[test]
    fn test_oauth_provider_type_display() {
        assert_eq!(OAuthProviderType::Google.to_string(), "google");
        assert_eq!(OAuthProviderType::Apple.to_string(), "apple");
        assert_eq!(OAuthProviderType::GitHub.to_string(), "github");
    }

    #[test]
    fn test_oauth_provider_type_from_str() {
        assert_eq!("google".parse::<OAuthProviderType>().unwrap(), OAuthProviderType::Google);
        assert_eq!("apple".parse::<OAuthProviderType>().unwrap(), OAuthProviderType::Apple);
        assert_eq!("github".parse::<OAuthProviderType>().unwrap(), OAuthProviderType::GitHub);
        assert!("invalid".parse::<OAuthProviderType>().is_err());
    }

    #[test]
    fn test_base_oauth_provider_auth_url() {
        let config = create_test_config();
        let provider = BaseOAuthProvider::new(
            config,
            OAuthProviderType::Google,
            "https://oauth2.googleapis.com/token".to_string(),
            "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
            Some("https://oauth2.googleapis.com/revoke".to_string()),
        );

        let redirect_uri = "http://localhost:3000/auth/callback";
        let state = "test_state";
        let auth_url = provider.build_auth_url(redirect_uri, state, None);

        assert!(auth_url.contains("client_id=test_client_id"));
        assert!(auth_url.contains("redirect_uri=http%3A//localhost%3A3000/auth/callback"));
        assert!(auth_url.contains("response_type=code"));
        assert!(auth_url.contains("state=test_state"));
        assert!(auth_url.contains("scope=openid%20email%20profile"));
    }

    #[test]
    fn test_oauth_state_validation() {
        let state = OAuthState::new(
            OAuthProviderType::Google,
            "http://localhost:3000/auth/callback".to_string(),
            None,
            300, // 5 minutes
        );

        let state_token = state.state_token.clone();
        
        // Valid state
        assert!(state.is_valid(&state_token, &OAuthProviderType::Google));
        
        // Invalid token
        assert!(!state.is_valid("invalid_token", &OAuthProviderType::Google));
        
        // Invalid provider
        assert!(!state.is_valid(&state_token, &OAuthProviderType::Apple));
    }

    #[test]
    fn test_oauth_state_manager() {
        let manager = OAuthStateManager::new();
        
        let state = OAuthState::new(
            OAuthProviderType::Google,
            "http://localhost:3000/auth/callback".to_string(),
            None,
            300,
        );
        
        let state_token = manager.store_state(state);
        
        // Valid state should be retrievable
        let retrieved_state = manager.validate_and_consume_state(&state_token, &OAuthProviderType::Google);
        assert!(retrieved_state.is_ok());
        
        // State should be consumed (not retrievable again)
        let second_attempt = manager.validate_and_consume_state(&state_token, &OAuthProviderType::Google);
        assert!(second_attempt.is_err());
    }
}