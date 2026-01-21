use crate::models::{User, CreateUserRequest, LoginRequest, TokenPair};
use anyhow::Result;
use uuid::Uuid;

/// Stub implementation of AuthService for tests
#[derive(Clone)]
pub struct AuthService {
    jwt_secret: String,
}

impl AuthService {
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }

    pub fn default() -> Self {
        Self::new("test_secret".to_string())
    }

    pub async fn register_user(&self, _request: CreateUserRequest) -> Result<User> {
        // Stub implementation
        Ok(User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            password_hash: Some("hashed_password".to_string()),
            email_verified: true,
            totp_secret: None,
            totp_enabled: false,
            oauth_providers: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_login: None,
            settings: crate::models::UserSettings {
                two_factor_enabled: false,
                email_notifications: true,
                privacy_mode: false,
            },
        })
    }

    pub async fn login_user(&self, _request: LoginRequest) -> Result<TokenPair> {
        // Stub implementation
        Ok(TokenPair {
            access_token: "stub_access_token".to_string(),
            refresh_token: "stub_refresh_token".to_string(),
            expires_in: 3600,
            token_type: "Bearer".to_string(),
        })
    }

    pub async fn validate_token(&self, _token: &str) -> Result<Uuid> {
        // Stub implementation - return a test user ID
        Ok(Uuid::new_v4())
    }

    pub async fn refresh_token(&self, _refresh_token: &str) -> Result<TokenPair> {
        // Stub implementation
        Ok(TokenPair {
            access_token: "new_stub_access_token".to_string(),
            refresh_token: "new_stub_refresh_token".to_string(),
            expires_in: 3600,
            token_type: "Bearer".to_string(),
        })
    }

    // Add missing methods that middleware expects
    pub fn verify_token(&self, _token: &str) -> Result<crate::models::Claims> {
        // Stub implementation
        Ok(crate::models::Claims {
            sub: Uuid::new_v4().to_string(),
            email: "test@example.com".to_string(),
            exp: (chrono::Utc::now().timestamp() + 3600),
            iat: chrono::Utc::now().timestamp(),
            jti: Uuid::new_v4().to_string(),
            token_type: crate::models::TokenType::Access,
            scopes: vec!["read".to_string(), "write".to_string()],
        })
    }

    pub async fn get_user(&self, _user_id: Uuid) -> Result<Option<User>> {
        // Stub implementation
        Ok(Some(User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            password_hash: Some("hashed_password".to_string()),
            email_verified: true,
            totp_secret: None,
            totp_enabled: false,
            oauth_providers: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_login: None,
            settings: crate::models::UserSettings {
                two_factor_enabled: false,
                email_notifications: true,
                privacy_mode: false,
            },
        }))
    }
}