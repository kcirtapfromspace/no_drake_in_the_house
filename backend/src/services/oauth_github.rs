use async_trait::async_trait;
use std::collections::HashMap;
use serde_json::Value;

use crate::models::oauth::{
    OAuthProviderType, OAuthTokens, OAuthUserInfo, OAuthFlowResponse, OAuthConfig
};
use crate::services::oauth::{OAuthProvider, BaseOAuthProvider};
use crate::error::{AppError, Result};

/// GitHub OAuth provider implementation
pub struct GitHubOAuthProvider {
    base: BaseOAuthProvider,
}

impl GitHubOAuthProvider {
    /// Test GitHub OAuth configuration by making a test API call
    pub async fn test_configuration(&self) -> Result<()> {
        // Test connection to GitHub API
        let response = self.base.client
            .get("https://api.github.com/meta")
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "OAuth-App")
            .send()
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("Failed to connect to GitHub API: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalServiceError(
                "GitHub API is not accessible".to_string()
            ));
        }

        let meta_info: serde_json::Value = response.json().await
            .map_err(|e| AppError::ExternalServiceError(format!("Failed to parse GitHub meta response: {}", e)))?;

        // Verify that GitHub API is responding correctly
        if meta_info["verifiable_password_authentication"].is_null() {
            tracing::warn!("GitHub API response format may have changed");
        }

        Ok(())
    }

    /// Create a new GitHub OAuth provider with environment variables
    pub fn new() -> Result<Self> {
        let client_id = std::env::var("GITHUB_CLIENT_ID")
            .map_err(|_| AppError::ConfigurationError {
                message: "GITHUB_CLIENT_ID environment variable is required".to_string(),
            })?;
        let client_secret = std::env::var("GITHUB_CLIENT_SECRET")
            .map_err(|_| AppError::ConfigurationError {
                message: "GITHUB_CLIENT_SECRET environment variable is required".to_string(),
            })?;
        let redirect_uri = std::env::var("GITHUB_REDIRECT_URI")
            .map_err(|_| AppError::ConfigurationError {
                message: "GITHUB_REDIRECT_URI environment variable is required".to_string(),
            })?;

        Self::with_credentials(client_id, client_secret, redirect_uri)
    }

    /// Create a new GitHub OAuth provider with config
    pub fn new_with_config(config: OAuthConfig) -> Result<Self> {
        let base = BaseOAuthProvider::new(
            config,
            OAuthProviderType::GitHub,
            "https://github.com/login/oauth/access_token".to_string(),
            "https://github.com/login/oauth/authorize".to_string(),
            "https://api.github.com/user".to_string(),
            None, // GitHub doesn't have a standard revocation endpoint
        );

        Ok(Self { base })
    }

    /// Create a GitHub OAuth provider with default configuration
    pub fn with_credentials(client_id: String, client_secret: String, redirect_uri: String) -> Result<Self> {
        let config = OAuthConfig {
            client_id,
            client_secret,
            redirect_uri,
            scopes: vec![
                "user:email".to_string(),
                "read:user".to_string(),
            ],
            additional_params: HashMap::new(),
        };

        let provider = Self::new_with_config(config)?;
        
        // Validate configuration on creation
        provider.validate_config()?;
        
        Ok(provider)
    }

    /// Parse GitHub user info response into standardized format
    fn parse_user_info(&self, user_data: Value) -> Result<OAuthUserInfo> {
        let provider_user_id = user_data["id"]
            .as_i64()
            .ok_or_else(|| AppError::OAuthProviderError {
                provider: "GitHub".to_string(),
                message: "Missing user ID in GitHub response".to_string(),
            })?
            .to_string();

        let email = user_data["email"].as_str().map(|s| s.to_string());
        let display_name = user_data["name"].as_str().map(|s| s.to_string());
        let avatar_url = user_data["avatar_url"].as_str().map(|s| s.to_string());

        // GitHub doesn't provide separate first/last names
        let (first_name, last_name) = if let Some(name) = &display_name {
            let parts: Vec<&str> = name.splitn(2, ' ').collect();
            match parts.len() {
                1 => (Some(parts[0].to_string()), None),
                2 => (Some(parts[0].to_string()), Some(parts[1].to_string())),
                _ => (None, None),
            }
        } else {
            (None, None)
        };

        // Store additional GitHub-specific data
        let mut provider_data = HashMap::new();
        if let Some(login) = user_data["login"].as_str() {
            provider_data.insert("username".to_string(), serde_json::Value::String(login.to_string()));
        }
        if let Some(html_url) = user_data["html_url"].as_str() {
            provider_data.insert("profile_url".to_string(), serde_json::Value::String(html_url.to_string()));
        }
        if let Some(bio) = user_data["bio"].as_str() {
            provider_data.insert("bio".to_string(), serde_json::Value::String(bio.to_string()));
        }
        if let Some(company) = user_data["company"].as_str() {
            provider_data.insert("company".to_string(), serde_json::Value::String(company.to_string()));
        }
        if let Some(location) = user_data["location"].as_str() {
            provider_data.insert("location".to_string(), serde_json::Value::String(location.to_string()));
        }
        if let Some(blog) = user_data["blog"].as_str() {
            if !blog.is_empty() {
                provider_data.insert("blog".to_string(), serde_json::Value::String(blog.to_string()));
            }
        }
        if let Some(public_repos) = user_data["public_repos"].as_i64() {
            provider_data.insert("public_repos".to_string(), serde_json::Value::Number(public_repos.into()));
        }
        if let Some(followers) = user_data["followers"].as_i64() {
            provider_data.insert("followers".to_string(), serde_json::Value::Number(followers.into()));
        }
        if let Some(following) = user_data["following"].as_i64() {
            provider_data.insert("following".to_string(), serde_json::Value::Number(following.into()));
        }

        Ok(OAuthUserInfo {
            provider_user_id,
            email,
            email_verified: None, // GitHub doesn't provide email verification status in user endpoint
            display_name,
            first_name,
            last_name,
            avatar_url,
            locale: None, // GitHub doesn't provide locale
            provider_data,
        })
    }

    /// Get user's primary email from GitHub API
    async fn get_user_emails(&self, access_token: &str) -> Result<Vec<Value>> {
        let response = self.base.client
            .get("https://api.github.com/user/emails")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "OAuth-App") // GitHub requires User-Agent header
            .send()
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("GitHub emails request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            // Handle specific GitHub API error responses
            if status == reqwest::StatusCode::UNAUTHORIZED {
                return Err(AppError::OAuthProviderError {
                    provider: "GitHub".to_string(),
                    message: "GitHub access token is invalid or expired".to_string(),
                });
            }
            
            if status == reqwest::StatusCode::FORBIDDEN {
                // This might happen if the user:email scope wasn't granted
                return Err(AppError::OAuthProviderError {
                    provider: "GitHub".to_string(),
                    message: "GitHub API access denied. Check if 'user:email' scope was granted.".to_string(),
                });
            }
            
            // Parse GitHub-specific error response
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                if let Some(message) = error_json["message"].as_str() {
                    return Err(AppError::OAuthProviderError {
                        provider: "GitHub".to_string(),
                        message: format!("GitHub emails API error: {}", message),
                    });
                }
            }
            
            return Err(AppError::ExternalServiceError(format!(
                "GitHub emails request failed with status {}: {}", 
                status, 
                error_text
            )));
        }

        response.json().await
            .map_err(|e| AppError::ExternalServiceError(format!("Failed to parse GitHub emails response: {}", e)))
    }
}

#[async_trait]
impl OAuthProvider for GitHubOAuthProvider {
    fn provider_type(&self) -> OAuthProviderType {
        OAuthProviderType::GitHub
    }

    async fn initiate_flow(&self, redirect_uri: &str) -> Result<OAuthFlowResponse> {
        let state = self.base.generate_state();
        
        // GitHub-specific parameters
        let mut additional_params = HashMap::new();
        additional_params.insert("allow_signup".to_string(), "true".to_string());

        let authorization_url = self.base.build_auth_url(redirect_uri, &state, Some(additional_params));

        Ok(OAuthFlowResponse {
            authorization_url,
            state,
            code_verifier: None, // GitHub doesn't require PKCE for server-side apps
        })
    }

    async fn exchange_code(&self, code: &str, _state: &str, redirect_uri: &str) -> Result<OAuthTokens> {
        let params = [
            ("grant_type", "authorization_code"),
            ("client_id", &self.base.config.client_id),
            ("client_secret", &self.base.config.client_secret),
            ("code", code),
            ("redirect_uri", redirect_uri),
        ];

        let response = self.base.client
            .post(&self.base.token_endpoint)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json") // GitHub returns JSON when this header is set
            .header("User-Agent", "OAuth-App") // GitHub requires User-Agent header
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("GitHub token exchange request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            // Parse GitHub-specific error response
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                let error_code = error_json["error"].as_str().unwrap_or("unknown_error");
                let error_description = error_json["error_description"].as_str().unwrap_or(&error_text);
                
                // Handle specific GitHub errors
                match error_code {
                    "bad_verification_code" => {
                        return Err(AppError::OAuthProviderError {
                            provider: "GitHub".to_string(),
                            message: "GitHub authorization code is invalid or expired".to_string(),
                        });
                    }
                    "incorrect_client_credentials" => {
                        return Err(AppError::ConfigurationError {
                            message: "GitHub OAuth client credentials are invalid".to_string(),
                        });
                    }
                    "redirect_uri_mismatch" => {
                        return Err(AppError::OAuthProviderError {
                            provider: "GitHub".to_string(),
                            message: "GitHub OAuth redirect URI mismatch".to_string(),
                        });
                    }
                    _ => {
                        return Err(AppError::OAuthProviderError {
                            provider: "GitHub".to_string(),
                            message: format!("Token exchange failed ({}): {}", error_code, error_description),
                        });
                    }
                }
            }
            
            return Err(AppError::ExternalServiceError(format!(
                "GitHub token exchange failed with status {}: {}", 
                status, 
                error_text
            )));
        }

        let token_response: Value = response.json().await
            .map_err(|e| AppError::ExternalServiceError(format!("Failed to parse GitHub token response: {}", e)))?;

        let access_token = token_response["access_token"]
            .as_str()
            .ok_or_else(|| AppError::OAuthProviderError {
                provider: "GitHub".to_string(),
                message: "Missing access_token in GitHub response".to_string(),
            })?
            .to_string();

        let token_type = token_response["token_type"]
            .as_str()
            .unwrap_or("bearer")
            .to_string();

        let scope = token_response["scope"]
            .as_str()
            .map(|s| s.to_string());

        // Validate that we received the expected token type
        if token_type.to_lowercase() != "bearer" {
            return Err(AppError::OAuthProviderError {
                provider: "GitHub".to_string(),
                message: format!("Unexpected token type from GitHub: {}", token_type),
            });
        }

        // GitHub doesn't provide refresh tokens or expiration for OAuth apps
        Ok(OAuthTokens {
            access_token,
            refresh_token: None,
            expires_in: None,
            token_type,
            scope,
            id_token: None,
        })
    }

    async fn get_user_info(&self, access_token: &str) -> Result<OAuthUserInfo> {
        let response = self.base.client
            .get(&self.base.user_info_endpoint)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "OAuth-App") // GitHub requires User-Agent header
            .send()
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("GitHub user info request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            // Handle specific GitHub API error responses
            if status == reqwest::StatusCode::UNAUTHORIZED {
                return Err(AppError::OAuthProviderError {
                    provider: "GitHub".to_string(),
                    message: "GitHub access token is invalid or expired".to_string(),
                });
            }
            
            if status == reqwest::StatusCode::FORBIDDEN {
                return Err(AppError::OAuthProviderError {
                    provider: "GitHub".to_string(),
                    message: "GitHub API rate limit exceeded or insufficient permissions".to_string(),
                });
            }
            
            // Parse GitHub-specific error response
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                if let Some(message) = error_json["message"].as_str() {
                    return Err(AppError::OAuthProviderError {
                        provider: "GitHub".to_string(),
                        message: format!("GitHub API error: {}", message),
                    });
                }
            }
            
            return Err(AppError::ExternalServiceError(format!(
                "GitHub user info request failed with status {}: {}", 
                status, 
                error_text
            )));
        }

        let user_data: Value = response.json().await
            .map_err(|e| AppError::ExternalServiceError(format!("Failed to parse GitHub user info response: {}", e)))?;

        self.parse_user_info(user_data)
    }

    async fn refresh_token(&self, _refresh_token: &str) -> Result<OAuthTokens> {
        // GitHub OAuth apps don't support refresh tokens
        Err(AppError::OAuthProviderError {
            provider: "GitHub".to_string(),
            message: "GitHub OAuth apps don't support token refresh".to_string(),
        })
    }

    async fn revoke_token(&self, _token: &str) -> Result<()> {
        // GitHub doesn't have a standard token revocation endpoint for OAuth apps
        // Tokens can be revoked through the GitHub web interface
        Ok(())
    }

    fn validate_config(&self) -> Result<()> {
        if self.base.config.client_id.is_empty() {
            return Err(AppError::ConfigurationError {
                message: "GitHub OAuth client_id is required".to_string(),
            });
        }

        if self.base.config.client_secret.is_empty() {
            return Err(AppError::ConfigurationError {
                message: "GitHub OAuth client_secret is required".to_string(),
            });
        }

        if self.base.config.redirect_uri.is_empty() {
            return Err(AppError::ConfigurationError {
                message: "GitHub OAuth redirect_uri is required".to_string(),
            });
        }

        // Validate redirect URI format
        if let Err(_) = reqwest::Url::parse(&self.base.config.redirect_uri) {
            return Err(AppError::ConfigurationError {
                message: "GitHub OAuth redirect_uri must be a valid URL".to_string(),
            });
        }

        // Validate client_id format (GitHub client IDs are typically 20 characters)
        // Skip validation in development mode
        let is_dev_mode = std::env::var("OAUTH_DEV_MODE").unwrap_or_else(|_| "false".to_string()) == "true";
        if !is_dev_mode && self.base.config.client_id.len() != 20 {
            tracing::warn!("GitHub client_id length is not 20 characters, which is unusual");
        }

        // Validate that required scopes are present
        let has_email_scope = self.base.config.scopes.iter().any(|s| s.contains("email"));
        if !has_email_scope {
            return Err(AppError::ConfigurationError {
                message: "GitHub OAuth requires 'user:email' or similar email scope".to_string(),
            });
        }

        Ok(())
    }
}

/// GitHub OAuth service for managing GitHub OAuth operations
pub struct GitHubOAuthService {
    provider: GitHubOAuthProvider,
}

impl GitHubOAuthService {
    /// Create a new GitHub OAuth service
    pub fn new(provider: GitHubOAuthProvider) -> Self {
        Self { provider }
    }

    /// Get the underlying OAuth provider
    pub fn provider(&self) -> &GitHubOAuthProvider {
        &self.provider
    }

    /// Initiate GitHub OAuth flow with additional options
    pub async fn initiate_flow_with_options(
        &self,
        redirect_uri: &str,
        allow_signup: Option<bool>,
        login: Option<&str>, // Suggest a specific account
    ) -> Result<OAuthFlowResponse> {
        let state = self.provider.base.generate_state();
        
        let mut additional_params = HashMap::new();
        additional_params.insert("allow_signup".to_string(), 
            allow_signup.unwrap_or(true).to_string());

        if let Some(suggested_login) = login {
            additional_params.insert("login".to_string(), suggested_login.to_string());
        }

        let authorization_url = self.provider.base.build_auth_url(redirect_uri, &state, Some(additional_params));

        Ok(OAuthFlowResponse {
            authorization_url,
            state,
            code_verifier: None,
        })
    }

    /// Get user's email addresses with verification status
    pub async fn get_user_emails(&self, access_token: &str) -> Result<Vec<GitHubEmail>> {
        let emails = self.provider.get_user_emails(access_token).await?;
        
        let mut github_emails = Vec::new();
        for email_data in emails {
            let email = GitHubEmail {
                email: email_data["email"].as_str().unwrap_or("").to_string(),
                primary: email_data["primary"].as_bool().unwrap_or(false),
                verified: email_data["verified"].as_bool().unwrap_or(false),
                visibility: email_data["visibility"].as_str().map(|s| s.to_string()),
            };
            github_emails.push(email);
        }

        Ok(github_emails)
    }

    /// Get user's primary verified email
    pub async fn get_primary_email(&self, access_token: &str) -> Result<Option<String>> {
        let emails = self.get_user_emails(access_token).await?;
        
        // Find primary verified email
        for email in &emails {
            if email.primary && email.verified {
                return Ok(Some(email.email.clone()));
            }
        }

        // Fallback to any verified email
        for email in &emails {
            if email.verified {
                return Ok(Some(email.email.clone()));
            }
        }

        Ok(None)
    }

    /// Get enhanced user info including verified email
    pub async fn get_enhanced_user_info(&self, access_token: &str) -> Result<OAuthUserInfo> {
        let mut user_info = self.provider.get_user_info(access_token).await?;
        
        // If email is not in user info or not verified, try to get verified email
        if user_info.email.is_none() {
            if let Ok(Some(primary_email)) = self.get_primary_email(access_token).await {
                user_info.email = Some(primary_email);
                user_info.email_verified = Some(true);
            }
        }

        Ok(user_info)
    }
}

/// GitHub email information
#[derive(Debug, Clone)]
pub struct GitHubEmail {
    pub email: String,
    pub primary: bool,
    pub verified: bool,
    pub visibility: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_github_provider() -> GitHubOAuthProvider {
        GitHubOAuthProvider::with_credentials(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            "http://localhost:3000/auth/github/callback".to_string(),
        ).unwrap()
    }

    #[test]
    fn test_github_provider_creation() {
        let provider = create_test_github_provider();
        assert_eq!(provider.provider_type(), OAuthProviderType::GitHub);
    }

    #[test]
    fn test_github_provider_validation() {
        let provider = create_test_github_provider();
        assert!(provider.validate_config().is_ok());
    }

    #[test]
    fn test_github_provider_validation_missing_client_id() {
        let config = OAuthConfig {
            client_id: "".to_string(),
            client_secret: "test_secret".to_string(),
            redirect_uri: "http://localhost:3000/auth/callback".to_string(),
            scopes: vec!["user:email".to_string(), "read:user".to_string()],
            additional_params: HashMap::new(),
        };

        let provider = GitHubOAuthProvider::new(config).unwrap();
        let result = provider.validate_config();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("client_id is required"));
    }

    #[tokio::test]
    async fn test_github_initiate_flow() {
        let provider = create_test_github_provider();
        let redirect_uri = "http://localhost:3000/auth/github/callback";
        
        let flow_response = provider.initiate_flow(redirect_uri).await.unwrap();
        
        assert!(flow_response.authorization_url.contains("github.com/login/oauth/authorize"));
        assert!(flow_response.authorization_url.contains("client_id=test_client_id"));
        assert!(flow_response.authorization_url.contains("redirect_uri="));
        assert!(flow_response.authorization_url.contains("allow_signup=true"));
        assert!(!flow_response.state.is_empty());
        assert!(flow_response.code_verifier.is_none());
    }

    #[test]
    fn test_github_parse_user_info() {
        let provider = create_test_github_provider();
        
        let user_data = serde_json::json!({
            "id": 123456789,
            "login": "testuser",
            "name": "Test User",
            "email": "test@example.com",
            "avatar_url": "https://avatars.githubusercontent.com/u/123456789",
            "html_url": "https://github.com/testuser",
            "bio": "Software Developer",
            "company": "Test Company",
            "location": "San Francisco, CA",
            "blog": "https://testuser.dev",
            "public_repos": 42,
            "followers": 100,
            "following": 50
        });

        let user_info = provider.parse_user_info(user_data).unwrap();
        
        assert_eq!(user_info.provider_user_id, "123456789");
        assert_eq!(user_info.email, Some("test@example.com".to_string()));
        assert_eq!(user_info.display_name, Some("Test User".to_string()));
        assert_eq!(user_info.first_name, Some("Test".to_string()));
        assert_eq!(user_info.last_name, Some("User".to_string()));
        assert_eq!(user_info.avatar_url, Some("https://avatars.githubusercontent.com/u/123456789".to_string()));
        
        // Check GitHub-specific data
        assert_eq!(
            user_info.provider_data.get("username"),
            Some(&serde_json::Value::String("testuser".to_string()))
        );
        assert_eq!(
            user_info.provider_data.get("profile_url"),
            Some(&serde_json::Value::String("https://github.com/testuser".to_string()))
        );
        assert_eq!(
            user_info.provider_data.get("bio"),
            Some(&serde_json::Value::String("Software Developer".to_string()))
        );
        assert_eq!(
            user_info.provider_data.get("company"),
            Some(&serde_json::Value::String("Test Company".to_string()))
        );
        assert_eq!(
            user_info.provider_data.get("public_repos"),
            Some(&serde_json::Value::Number(42.into()))
        );
    }

    #[test]
    fn test_github_parse_user_info_missing_id() {
        let provider = create_test_github_provider();
        
        let user_data = serde_json::json!({
            "login": "testuser",
            "name": "Test User"
        });

        let result = provider.parse_user_info(user_data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing user ID"));
    }

    #[test]
    fn test_github_parse_user_info_name_splitting() {
        let provider = create_test_github_provider();
        
        // Test single name
        let user_data = serde_json::json!({
            "id": 123456789,
            "name": "SingleName"
        });
        let user_info = provider.parse_user_info(user_data).unwrap();
        assert_eq!(user_info.first_name, Some("SingleName".to_string()));
        assert_eq!(user_info.last_name, None);
        
        // Test full name
        let user_data = serde_json::json!({
            "id": 123456789,
            "name": "First Last"
        });
        let user_info = provider.parse_user_info(user_data).unwrap();
        assert_eq!(user_info.first_name, Some("First".to_string()));
        assert_eq!(user_info.last_name, Some("Last".to_string()));
        
        // Test name with multiple spaces
        let user_data = serde_json::json!({
            "id": 123456789,
            "name": "First Middle Last"
        });
        let user_info = provider.parse_user_info(user_data).unwrap();
        assert_eq!(user_info.first_name, Some("First".to_string()));
        assert_eq!(user_info.last_name, Some("Middle Last".to_string()));
    }

    #[test]
    fn test_github_oauth_service() {
        let provider = create_test_github_provider();
        let service = GitHubOAuthService::new(provider);
        
        assert_eq!(service.provider().provider_type(), OAuthProviderType::GitHub);
    }

    #[tokio::test]
    async fn test_github_oauth_service_with_options() {
        let provider = create_test_github_provider();
        let service = GitHubOAuthService::new(provider);
        
        let redirect_uri = "http://localhost:3000/auth/github/callback";
        let allow_signup = Some(false);
        let login = Some("suggested_user");
        
        let flow_response = service
            .initiate_flow_with_options(redirect_uri, allow_signup, login)
            .await
            .unwrap();
        
        assert!(flow_response.authorization_url.contains("allow_signup=false"));
        assert!(flow_response.authorization_url.contains("login=suggested_user"));
    }

    #[test]
    fn test_github_email_structure() {
        let email = GitHubEmail {
            email: "test@example.com".to_string(),
            primary: true,
            verified: true,
            visibility: Some("public".to_string()),
        };
        
        assert_eq!(email.email, "test@example.com");
        assert!(email.primary);
        assert!(email.verified);
        assert_eq!(email.visibility, Some("public".to_string()));
    }
}