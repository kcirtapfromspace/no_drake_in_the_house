use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

use backend::error::{AppError, Result};
use backend::models::oauth::{
    AccountLinkRequest, OAuthFlowResponse, OAuthProviderType, OAuthTokens, OAuthUserInfo,
};
use backend::models::user::{User, UserSettings};
use backend::services::auth::AuthService;
use backend::services::oauth::{OAuthProvider, OAuthStateManager};
use backend::services::oauth_encryption::OAuthTokenEncryption;

// Mock OAuth provider for testing
struct MockOAuthProvider {
    provider_type: OAuthProviderType,
    should_fail: bool,
}

impl MockOAuthProvider {
    fn new(provider_type: OAuthProviderType) -> Self {
        Self {
            provider_type,
            should_fail: false,
        }
    }

    fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

#[async_trait::async_trait]
impl OAuthProvider for MockOAuthProvider {
    fn provider_type(&self) -> OAuthProviderType {
        self.provider_type.clone()
    }

    async fn initiate_flow(&self, redirect_uri: &str) -> Result<OAuthFlowResponse> {
        if self.should_fail {
            return Err(AppError::OAuthProviderError {
                provider: self.provider_type.to_string(),
                message: "Mock provider failure".to_string(),
            });
        }

        Ok(OAuthFlowResponse {
            authorization_url: format!(
                "https://mock-{}.com/oauth/authorize?redirect_uri={}",
                self.provider_type.to_string(),
                redirect_uri
            ),
            state: "mock_state".to_string(),
            code_verifier: None,
        })
    }

    async fn exchange_code(
        &self,
        _code: &str,
        _state: &str,
        _redirect_uri: &str,
    ) -> Result<OAuthTokens> {
        if self.should_fail {
            return Err(AppError::OAuthProviderError {
                provider: self.provider_type.to_string(),
                message: "Mock token exchange failure".to_string(),
            });
        }

        Ok(OAuthTokens {
            access_token: "mock_access_token".to_string(),
            refresh_token: Some("mock_refresh_token".to_string()),
            expires_in: Some(3600),
            token_type: "Bearer".to_string(),
            scope: Some("email profile".to_string()),
            id_token: None,
        })
    }

    async fn get_user_info(&self, _access_token: &str) -> Result<OAuthUserInfo> {
        if self.should_fail {
            return Err(AppError::OAuthProviderError {
                provider: self.provider_type.to_string(),
                message: "Mock user info failure".to_string(),
            });
        }

        Ok(OAuthUserInfo {
            provider_user_id: "mock_user_123".to_string(),
            email: Some("test@example.com".to_string()),
            email_verified: Some(true),
            display_name: Some("Test User".to_string()),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
            locale: Some("en".to_string()),
            provider_data: HashMap::new(),
        })
    }

    async fn refresh_token(&self, _refresh_token: &str) -> Result<OAuthTokens> {
        if self.should_fail {
            return Err(AppError::OAuthProviderError {
                provider: self.provider_type.to_string(),
                message: "Mock token refresh failure".to_string(),
            });
        }

        Ok(OAuthTokens {
            access_token: "new_mock_access_token".to_string(),
            refresh_token: Some("new_mock_refresh_token".to_string()),
            expires_in: Some(3600),
            token_type: "Bearer".to_string(),
            scope: Some("email profile".to_string()),
            id_token: None,
        })
    }

    fn validate_config(&self) -> Result<()> {
        Ok(())
    }
}

// Test helper functions
async fn create_test_auth_service() -> AuthService {
    let db_pool = backend::tests::common::setup_test_db().await;
    let mut auth_service = AuthService::new(db_pool);

    // Add mock OAuth providers
    auth_service.add_oauth_provider(
        OAuthProviderType::Google,
        Box::new(MockOAuthProvider::new(OAuthProviderType::Google)),
    );
    auth_service.add_oauth_provider(
        OAuthProviderType::GitHub,
        Box::new(MockOAuthProvider::new(OAuthProviderType::GitHub)),
    );

    auth_service
}

async fn create_test_user(auth_service: &AuthService) -> User {
    use backend::models::user::CreateUserRequest;

    let request = CreateUserRequest {
        email: "test@example.com".to_string(),
        password: "test_password_123".to_string(),
    };

    auth_service.register_user(request).await.unwrap()
}

#[tokio::test]
async fn test_oauth_state_manager() {
    let state_manager = OAuthStateManager::new();

    let oauth_state = backend::models::oauth::OAuthState::new(
        OAuthProviderType::Google,
        "http://localhost:3000/callback".to_string(),
        None,
        300,
    );

    let state_token = oauth_state.state_token.clone();
    let stored_token = state_manager.store_state(oauth_state);

    assert_eq!(state_token, stored_token);

    // Valid state should be retrievable
    let retrieved_state =
        state_manager.validate_and_consume_state(&state_token, &OAuthProviderType::Google);
    assert!(retrieved_state.is_ok());

    // State should be consumed (not retrievable again)
    let second_attempt =
        state_manager.validate_and_consume_state(&state_token, &OAuthProviderType::Google);
    assert!(second_attempt.is_err());
}

#[tokio::test]
async fn test_oauth_state_validation_wrong_provider() {
    let state_manager = OAuthStateManager::new();

    let oauth_state = backend::models::oauth::OAuthState::new(
        OAuthProviderType::Google,
        "http://localhost:3000/callback".to_string(),
        None,
        300,
    );

    let state_token = state_manager.store_state(oauth_state);

    // Wrong provider should fail
    let result = state_manager.validate_and_consume_state(&state_token, &OAuthProviderType::GitHub);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_oauth_token_encryption() {
    let key = OAuthTokenEncryption::generate_key();
    let encryption = OAuthTokenEncryption::with_key(&key).unwrap();

    let access_token = "test_access_token";
    let refresh_token = Some("test_refresh_token");

    // Test token pair encryption
    let (encrypted_access, encrypted_refresh) = encryption
        .encrypt_token_pair(access_token, refresh_token.as_deref())
        .unwrap();

    let (decrypted_access, decrypted_refresh) = encryption
        .decrypt_token_pair(&encrypted_access, encrypted_refresh.as_deref())
        .unwrap();

    assert_eq!(access_token, decrypted_access);
    assert_eq!(refresh_token, decrypted_refresh.as_deref());
}

#[tokio::test]
async fn test_initiate_oauth_flow() {
    let auth_service = create_test_auth_service().await;

    let result = auth_service
        .initiate_oauth_flow(
            OAuthProviderType::Google,
            "http://localhost:3000/callback".to_string(),
        )
        .await;

    assert!(result.is_ok());
    let flow_response = result.unwrap();
    assert!(flow_response.authorization_url.contains("mock-google.com"));
    assert!(!flow_response.state.is_empty());
}

#[tokio::test]
async fn test_initiate_oauth_flow_unsupported_provider() {
    let auth_service = create_test_auth_service().await;

    let result = auth_service
        .initiate_oauth_flow(
            OAuthProviderType::Apple, // Not configured in test setup
            "http://localhost:3000/callback".to_string(),
        )
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::OAuthProviderError { provider, .. } => {
            assert_eq!(provider, "apple");
        }
        _ => panic!("Expected OAuthProviderError"),
    }
}

#[tokio::test]
async fn test_complete_oauth_flow_new_user() {
    let auth_service = create_test_auth_service().await;

    // First initiate flow to get valid state
    let flow_response = auth_service
        .initiate_oauth_flow(
            OAuthProviderType::Google,
            "http://localhost:3000/callback".to_string(),
        )
        .await
        .unwrap();

    let result = auth_service
        .complete_oauth_flow(
            OAuthProviderType::Google,
            "mock_auth_code".to_string(),
            flow_response.state,
            "http://localhost:3000/callback".to_string(),
        )
        .await;

    assert!(result.is_ok());
    let token_pair = result.unwrap();
    assert!(!token_pair.access_token.is_empty());
    assert!(!token_pair.refresh_token.is_empty());
}

#[tokio::test]
async fn test_complete_oauth_flow_existing_user() {
    let auth_service = create_test_auth_service().await;
    let _user = create_test_user(&auth_service).await;

    // First initiate flow to get valid state
    let flow_response = auth_service
        .initiate_oauth_flow(
            OAuthProviderType::Google,
            "http://localhost:3000/callback".to_string(),
        )
        .await
        .unwrap();

    let result = auth_service
        .complete_oauth_flow(
            OAuthProviderType::Google,
            "mock_auth_code".to_string(),
            flow_response.state,
            "http://localhost:3000/callback".to_string(),
        )
        .await;

    assert!(result.is_ok());
    let token_pair = result.unwrap();
    assert!(!token_pair.access_token.is_empty());
}

#[tokio::test]
async fn test_complete_oauth_flow_invalid_state() {
    let auth_service = create_test_auth_service().await;

    let result = auth_service
        .complete_oauth_flow(
            OAuthProviderType::Google,
            "mock_auth_code".to_string(),
            "invalid_state".to_string(),
            "http://localhost:3000/callback".to_string(),
        )
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::OAuthStateValidationFailed => {}
        _ => panic!("Expected OAuthStateValidationFailed"),
    }
}

#[tokio::test]
async fn test_link_oauth_account() {
    let auth_service = create_test_auth_service().await;
    let user = create_test_user(&auth_service).await;

    // First initiate flow to get valid state
    let flow_response = auth_service
        .initiate_oauth_flow(
            OAuthProviderType::GitHub,
            "http://localhost:3000/callback".to_string(),
        )
        .await
        .unwrap();

    let link_request = AccountLinkRequest {
        provider: OAuthProviderType::GitHub,
        code: "mock_auth_code".to_string(),
        state: flow_response.state,
    };

    let result = auth_service.link_oauth_account(user.id, link_request).await;
    assert!(result.is_ok());

    // Verify the account was linked
    let updated_user = auth_service.get_user_by_id(user.id).await.unwrap();
    assert!(updated_user.has_oauth_account(&OAuthProviderType::GitHub));
}

#[tokio::test]
async fn test_link_oauth_account_already_linked() {
    let auth_service = create_test_auth_service().await;
    let user = create_test_user(&auth_service).await;

    // Link account first time
    let flow_response = auth_service
        .initiate_oauth_flow(
            OAuthProviderType::GitHub,
            "http://localhost:3000/callback".to_string(),
        )
        .await
        .unwrap();

    let link_request = AccountLinkRequest {
        provider: OAuthProviderType::GitHub,
        code: "mock_auth_code".to_string(),
        state: flow_response.state,
    };

    auth_service
        .link_oauth_account(user.id, link_request)
        .await
        .unwrap();

    // Try to link again
    let flow_response2 = auth_service
        .initiate_oauth_flow(
            OAuthProviderType::GitHub,
            "http://localhost:3000/callback".to_string(),
        )
        .await
        .unwrap();

    let link_request2 = AccountLinkRequest {
        provider: OAuthProviderType::GitHub,
        code: "mock_auth_code".to_string(),
        state: flow_response2.state,
    };

    let result = auth_service
        .link_oauth_account(user.id, link_request2)
        .await;
    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::Conflict { .. } => {}
        _ => panic!("Expected Conflict error"),
    }
}

#[tokio::test]
async fn test_unlink_oauth_account() {
    let auth_service = create_test_auth_service().await;
    let user = create_test_user(&auth_service).await;

    // Link account first
    let flow_response = auth_service
        .initiate_oauth_flow(
            OAuthProviderType::GitHub,
            "http://localhost:3000/callback".to_string(),
        )
        .await
        .unwrap();

    let link_request = AccountLinkRequest {
        provider: OAuthProviderType::GitHub,
        code: "mock_auth_code".to_string(),
        state: flow_response.state,
    };

    auth_service
        .link_oauth_account(user.id, link_request)
        .await
        .unwrap();

    // Now unlink it
    let result = auth_service
        .unlink_oauth_account(user.id, OAuthProviderType::GitHub)
        .await;
    assert!(result.is_ok());

    // Verify the account was unlinked
    let updated_user = auth_service.get_user_by_id(user.id).await.unwrap();
    assert!(!updated_user.has_oauth_account(&OAuthProviderType::GitHub));
}

#[tokio::test]
async fn test_unlink_oauth_account_not_linked() {
    let auth_service = create_test_auth_service().await;
    let user = create_test_user(&auth_service).await;

    let result = auth_service
        .unlink_oauth_account(user.id, OAuthProviderType::GitHub)
        .await;
    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::NotFound { .. } => {}
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_unlink_last_auth_method() {
    let auth_service = create_test_auth_service().await;

    // Create OAuth-only user (no password)
    let flow_response = auth_service
        .initiate_oauth_flow(
            OAuthProviderType::Google,
            "http://localhost:3000/callback".to_string(),
        )
        .await
        .unwrap();

    let token_pair = auth_service
        .complete_oauth_flow(
            OAuthProviderType::Google,
            "mock_auth_code".to_string(),
            flow_response.state,
            "http://localhost:3000/callback".to_string(),
        )
        .await
        .unwrap();

    // Get the created user
    let claims = auth_service.verify_token(&token_pair.access_token).unwrap();
    let user = auth_service.get_user_by_id(claims.user_id).await.unwrap();

    // Try to unlink the only authentication method
    let result = auth_service
        .unlink_oauth_account(user.id, OAuthProviderType::Google)
        .await;
    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::Conflict { .. } => {}
        _ => panic!("Expected Conflict error"),
    }
}

#[tokio::test]
async fn test_refresh_oauth_tokens() {
    let auth_service = create_test_auth_service().await;

    // Create user with OAuth account
    let flow_response = auth_service
        .initiate_oauth_flow(
            OAuthProviderType::Google,
            "http://localhost:3000/callback".to_string(),
        )
        .await
        .unwrap();

    let token_pair = auth_service
        .complete_oauth_flow(
            OAuthProviderType::Google,
            "mock_auth_code".to_string(),
            flow_response.state,
            "http://localhost:3000/callback".to_string(),
        )
        .await
        .unwrap();

    let claims = auth_service.verify_token(&token_pair.access_token).unwrap();

    // Refresh tokens
    let result = auth_service
        .refresh_oauth_tokens(claims.user_id, OAuthProviderType::Google)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_refresh_oauth_tokens_no_account() {
    let auth_service = create_test_auth_service().await;
    let user = create_test_user(&auth_service).await;

    let result = auth_service
        .refresh_oauth_tokens(user.id, OAuthProviderType::Google)
        .await;
    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::NotFound { .. } => {}
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_get_available_oauth_providers() {
    let auth_service = create_test_auth_service().await;

    let providers = auth_service.get_available_oauth_providers();
    assert!(providers.contains(&OAuthProviderType::Google));
    assert!(providers.contains(&OAuthProviderType::GitHub));
    assert!(!providers.contains(&OAuthProviderType::Apple)); // Not configured in test
}

#[tokio::test]
async fn test_is_oauth_provider_available() {
    let auth_service = create_test_auth_service().await;

    assert!(auth_service.is_oauth_provider_available(&OAuthProviderType::Google));
    assert!(auth_service.is_oauth_provider_available(&OAuthProviderType::GitHub));
    assert!(!auth_service.is_oauth_provider_available(&OAuthProviderType::Apple));
}

#[tokio::test]
async fn test_oauth_provider_failure() {
    let db_pool = backend::tests::common::setup_test_db().await;
    let mut auth_service = AuthService::new(db_pool);

    // Add failing mock provider
    auth_service.add_oauth_provider(
        OAuthProviderType::Google,
        Box::new(MockOAuthProvider::new(OAuthProviderType::Google).with_failure()),
    );

    let result = auth_service
        .initiate_oauth_flow(
            OAuthProviderType::Google,
            "http://localhost:3000/callback".to_string(),
        )
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::OAuthProviderError { .. } => {}
        _ => panic!("Expected OAuthProviderError"),
    }
}

#[tokio::test]
async fn test_extract_apple_user_info() {
    let auth_service = create_test_auth_service().await;

    // Create a mock Apple ID token (this is just for testing the parsing logic)
    let header =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(r#"{"alg":"RS256","kid":"test"}"#);
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(r#"{"sub":"123456","email":"test@example.com","email_verified":true,"is_private_email":false}"#);
    let signature = "mock_signature";
    let id_token = format!("{}.{}.{}", header, payload, signature);

    let result = auth_service.extract_apple_user_info(&id_token).await;
    assert!(result.is_ok());

    let user_info = result.unwrap();
    assert_eq!(user_info.provider_user_id, "123456");
    assert_eq!(user_info.email, Some("test@example.com".to_string()));
    assert_eq!(user_info.email_verified, Some(true));
}

#[tokio::test]
async fn test_extract_apple_user_info_invalid_token() {
    let auth_service = create_test_auth_service().await;

    let result = auth_service.extract_apple_user_info("invalid.token").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::OAuthProviderError { provider, .. } => {
            assert_eq!(provider, "Apple");
        }
        _ => panic!("Expected OAuthProviderError"),
    }
}
#[tokio::test]
async fn test_refresh_expired_oauth_tokens() {
    let auth_service = create_test_auth_service().await;

    // Create user with OAuth account
    let flow_response = auth_service
        .initiate_oauth_flow(
            OAuthProviderType::Google,
            "http://localhost:3000/callback".to_string(),
        )
        .await
        .unwrap();

    let token_pair = auth_service
        .complete_oauth_flow(
            OAuthProviderType::Google,
            "mock_auth_code".to_string(),
            flow_response.state,
            "http://localhost:3000/callback".to_string(),
        )
        .await
        .unwrap();

    let claims = auth_service.verify_token(&token_pair.access_token).unwrap();

    // Test refreshing tokens
    let refreshed_providers = auth_service
        .refresh_expired_oauth_tokens(claims.user_id)
        .await
        .unwrap();

    // Since our mock tokens don't expire immediately, this should be empty
    assert!(refreshed_providers.is_empty());
}

#[tokio::test]
async fn test_get_oauth_token_status() {
    let auth_service = create_test_auth_service().await;

    // Create user with OAuth account
    let flow_response = auth_service
        .initiate_oauth_flow(
            OAuthProviderType::Google,
            "http://localhost:3000/callback".to_string(),
        )
        .await
        .unwrap();

    let token_pair = auth_service
        .complete_oauth_flow(
            OAuthProviderType::Google,
            "mock_auth_code".to_string(),
            flow_response.state,
            "http://localhost:3000/callback".to_string(),
        )
        .await
        .unwrap();

    let claims = auth_service.verify_token(&token_pair.access_token).unwrap();

    // Get token status
    let statuses = auth_service
        .get_oauth_token_status(claims.user_id)
        .await
        .unwrap();

    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0].provider, OAuthProviderType::Google);
    assert!(statuses[0].has_refresh_token);
}

#[tokio::test]
async fn test_revoke_oauth_tokens() {
    let auth_service = create_test_auth_service().await;

    // Create user with OAuth account
    let flow_response = auth_service
        .initiate_oauth_flow(
            OAuthProviderType::Google,
            "http://localhost:3000/callback".to_string(),
        )
        .await
        .unwrap();

    let token_pair = auth_service
        .complete_oauth_flow(
            OAuthProviderType::Google,
            "mock_auth_code".to_string(),
            flow_response.state,
            "http://localhost:3000/callback".to_string(),
        )
        .await
        .unwrap();

    let claims = auth_service.verify_token(&token_pair.access_token).unwrap();

    // Revoke tokens
    let result = auth_service
        .revoke_oauth_tokens(claims.user_id, OAuthProviderType::Google)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_schedule_token_refresh() {
    let auth_service = create_test_auth_service().await;

    // Test scheduling (should be empty since no tokens are expiring soon)
    let schedules = auth_service.schedule_token_refresh().await.unwrap();
    assert!(schedules.is_empty());
}

#[tokio::test]
async fn test_cleanup_expired_tokens() {
    let auth_service = create_test_auth_service().await;

    // Test cleanup (should be 0 since no expired tokens exist)
    let cleaned_count = auth_service.cleanup_expired_tokens().await.unwrap();
    assert_eq!(cleaned_count, 0);
}

#[tokio::test]
async fn test_refresh_all_expired_tokens() {
    let auth_service = create_test_auth_service().await;

    // Test background refresh (should be 0 since no expired tokens exist)
    let refreshed_count = auth_service.refresh_all_expired_tokens().await.unwrap();
    assert_eq!(refreshed_count, 0);
}

#[tokio::test]
async fn test_token_expiration_status() {
    use backend::models::oauth::{OAuthTokenStatus, TokenExpirationStatus};
    use chrono::{Duration, Utc};

    // Test different expiration statuses
    let now = Utc::now();

    // Valid token
    let valid_status = TokenExpirationStatus::Valid {
        expires_at: now + Duration::days(7),
    };

    match valid_status {
        TokenExpirationStatus::Valid { expires_at } => {
            assert!(expires_at > now);
        }
        _ => panic!("Expected Valid status"),
    }

    // Expiring soon
    let expiring_status = TokenExpirationStatus::ExpiringSoon {
        hours_remaining: 12,
    };

    match expiring_status {
        TokenExpirationStatus::ExpiringSoon { hours_remaining } => {
            assert_eq!(hours_remaining, 12);
        }
        _ => panic!("Expected ExpiringSoon status"),
    }

    // Expired
    let expired_status = TokenExpirationStatus::Expired;
    assert!(matches!(expired_status, TokenExpirationStatus::Expired));

    // No expiration
    let no_expiration_status = TokenExpirationStatus::NoExpiration;
    assert!(matches!(
        no_expiration_status,
        TokenExpirationStatus::NoExpiration
    ));
}

#[tokio::test]
async fn test_refresh_priority() {
    use backend::models::oauth::RefreshPriority;

    let high_priority = RefreshPriority::High;
    let normal_priority = RefreshPriority::Normal;

    assert_eq!(high_priority, RefreshPriority::High);
    assert_eq!(normal_priority, RefreshPriority::Normal);
    assert_ne!(high_priority, normal_priority);
}
