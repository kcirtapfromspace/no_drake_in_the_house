use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{Utc, Duration};
use serde_json::json;

use backend::models::oauth::{
    OAuthProviderType, OAuthUserInfo, OAuthTokens, OAuthFlowResponse, 
    OAuthAccount, OAuthState, TokenExpirationStatus, OAuthTokenStatus
};
use backend::services::oauth::{OAuthProvider, OAuthStateManager};
use backend::services::oauth_encryption::OAuthTokenEncryption;
use backend::error::{AppError, Result};

/// Mock OAuth provider for comprehensive testing
struct MockOAuthProvider {
    provider_type: OAuthProviderType,
    should_fail: bool,
    failure_type: MockFailureType,
    response_delay_ms: u64,
}

#[derive(Debug, Clone)]
enum MockFailureType {
    NetworkError,
    InvalidCredentials,
    TokenExpired,
    RateLimited,
    ProviderUnavailable,
}

impl MockOAuthProvider {
    fn new(provider_type: OAuthProviderType) -> Self {
        Self {
            provider_type,
            should_fail: false,
            failure_type: MockFailureType::NetworkError,
            response_delay_ms: 0,
        }
    }

    fn with_failure(mut self, failure_type: MockFailureType) -> Self {
        self.should_fail = true;
        self.failure_type = failure_type;
        self
    }

    fn with_delay(mut self, delay_ms: u64) -> Self {
        self.response_delay_ms = delay_ms;
        self
    }
}

#[async_trait::async_trait]
impl OAuthProvider for MockOAuthProvider {
    fn provider_type(&self) -> OAuthProviderType {
        self.provider_type
    }

    async fn initiate_flow(&self, redirect_uri: &str) -> Result<OAuthFlowResponse> {
        if self.response_delay_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.response_delay_ms)).await;
        }

        if self.should_fail {
            return match self.failure_type {
                MockFailureType::NetworkError => Err(AppError::ExternalServiceError(
                    "Network connection failed".to_string()
                )),
                MockFailureType::ProviderUnavailable => Err(AppError::OAuthProviderError {
                    provider: self.provider_type.to_string(),
                    message: "Provider temporarily unavailable".to_string(),
                }),
                _ => Err(AppError::OAuthProviderError {
                    provider: self.provider_type.to_string(),
                    message: "Mock provider failure".to_string(),
                }),
            };
        }

        Ok(OAuthFlowResponse {
            authorization_url: format!(
                "https://mock-{}.com/oauth/authorize?redirect_uri={}&state=mock_state_{}",
                self.provider_type.to_string(),
                redirect_uri,
                Uuid::new_v4()
            ),
            state: format!("mock_state_{}", Uuid::new_v4()),
            code_verifier: Some(format!("mock_verifier_{}", Uuid::new_v4())),
        })
    }

    async fn exchange_code(&self, _code: &str, _state: &str, _redirect_uri: &str) -> Result<OAuthTokens> {
        if self.response_delay_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.response_delay_ms)).await;
        }

        if self.should_fail {
            return match self.failure_type {
                MockFailureType::InvalidCredentials => Err(AppError::OAuthProviderError {
                    provider: self.provider_type.to_string(),
                    message: "Invalid authorization code".to_string(),
                }),
                MockFailureType::TokenExpired => Err(AppError::OAuthProviderError {
                    provider: self.provider_type.to_string(),
                    message: "Authorization code expired".to_string(),
                }),
                MockFailureType::RateLimited => Err(AppError::OAuthProviderError {
                    provider: self.provider_type.to_string(),
                    message: "Rate limit exceeded".to_string(),
                }),
                _ => Err(AppError::OAuthProviderError {
                    provider: self.provider_type.to_string(),
                    message: "Token exchange failed".to_string(),
                }),
            };
        }

        Ok(OAuthTokens {
            access_token: format!("mock_access_token_{}", Uuid::new_v4()),
            refresh_token: Some(format!("mock_refresh_token_{}", Uuid::new_v4())),
            expires_in: Some(3600),
            token_type: "Bearer".to_string(),
            scope: Some("email profile".to_string()),
            id_token: match self.provider_type {
                OAuthProviderType::Apple => Some(format!("mock_id_token_{}", Uuid::new_v4())),
                _ => None,
            },
        })
    }

    async fn get_user_info(&self, _access_token: &str) -> Result<OAuthUserInfo> {
        if self.response_delay_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.response_delay_ms)).await;
        }

        if self.should_fail {
            return Err(AppError::OAuthProviderError {
                provider: self.provider_type.to_string(),
                message: "Failed to fetch user info".to_string(),
            });
        }

        let provider_user_id = format!("mock_user_{}_{}", self.provider_type, Uuid::new_v4());
        
        Ok(OAuthUserInfo {
            provider_user_id,
            email: Some(format!("test_{}@example.com", Uuid::new_v4())),
            email_verified: Some(true),
            display_name: Some(format!("Test User {}", self.provider_type)),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            avatar_url: Some(format!("https://example.com/avatar_{}.jpg", Uuid::new_v4())),
            locale: Some("en".to_string()),
            provider_data: HashMap::new(),
        })
    }

    async fn refresh_token(&self, _refresh_token: &str) -> Result<OAuthTokens> {
        if self.response_delay_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.response_delay_ms)).await;
        }

        if self.should_fail {
            return match self.failure_type {
                MockFailureType::TokenExpired => Err(AppError::OAuthProviderError {
                    provider: self.provider_type.to_string(),
                    message: "Refresh token expired".to_string(),
                }),
                _ => Err(AppError::OAuthProviderError {
                    provider: self.provider_type.to_string(),
                    message: "Token refresh failed".to_string(),
                }),
            };
        }

        Ok(OAuthTokens {
            access_token: format!("new_mock_access_token_{}", Uuid::new_v4()),
            refresh_token: Some(format!("new_mock_refresh_token_{}", Uuid::new_v4())),
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

/// Test helper functions
fn create_test_oauth_tokens() -> OAuthTokens {
    OAuthTokens {
        access_token: format!("mock_access_token_{}", Uuid::new_v4()),
        refresh_token: Some(format!("mock_refresh_token_{}", Uuid::new_v4())),
        expires_in: Some(3600),
        token_type: "Bearer".to_string(),
        scope: Some("email profile".to_string()),
        id_token: None,
    }
}

fn create_test_oauth_user_info(provider: OAuthProviderType) -> OAuthUserInfo {
    OAuthUserInfo {
        provider_user_id: format!("mock_user_{}_{}", provider, Uuid::new_v4()),
        email: Some(format!("test_{}@example.com", Uuid::new_v4())),
        email_verified: Some(true),
        display_name: Some(format!("Test User {}", provider)),
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
        avatar_url: Some(format!("https://example.com/avatar_{}.jpg", Uuid::new_v4())),
        locale: Some("en".to_string()),
        provider_data: HashMap::new(),
    }
}

// ===== OAuth State Management Tests =====

#[tokio::test]
async fn test_oauth_state_manager_basic_operations() {
    let state_manager = OAuthStateManager::new();
    
    let oauth_state = OAuthState::new(
        OAuthProviderType::Google,
        "http://localhost:3000/callback".to_string(),
        Some("code_verifier_123".to_string()),
        300, // 5 minutes
    );
    
    let state_token = oauth_state.state_token.clone();
    let stored_token = state_manager.store_state(oauth_state);
    
    assert_eq!(state_token, stored_token);
    
    // Valid state should be retrievable
    let retrieved_state = state_manager
        .validate_and_consume_state(&state_token, &OAuthProviderType::Google);
    assert!(retrieved_state.is_ok());
    
    let state = retrieved_state.unwrap();
    assert_eq!(state.provider, OAuthProviderType::Google);
    assert_eq!(state.redirect_uri, "http://localhost:3000/callback");
    assert_eq!(state.code_verifier, Some("code_verifier_123".to_string()));
}

#[tokio::test]
async fn test_oauth_state_manager_state_consumed_once() {
    let state_manager = OAuthStateManager::new();
    
    let oauth_state = OAuthState::new(
        OAuthProviderType::Google,
        "http://localhost:3000/callback".to_string(),
        None,
        300,
    );
    
    let state_token = state_manager.store_state(oauth_state);
    
    // First validation should succeed
    let first_attempt = state_manager
        .validate_and_consume_state(&state_token, &OAuthProviderType::Google);
    assert!(first_attempt.is_ok());
    
    // Second validation should fail (state consumed)
    let second_attempt = state_manager
        .validate_and_consume_state(&state_token, &OAuthProviderType::Google);
    assert!(second_attempt.is_err());
}

#[tokio::test]
async fn test_oauth_state_manager_wrong_provider() {
    let state_manager = OAuthStateManager::new();
    
    let oauth_state = OAuthState::new(
        OAuthProviderType::Google,
        "http://localhost:3000/callback".to_string(),
        None,
        300,
    );
    
    let state_token = state_manager.store_state(oauth_state);
    
    // Wrong provider should fail
    let result = state_manager
        .validate_and_consume_state(&state_token, &OAuthProviderType::GitHub);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_oauth_state_manager_expired_state() {
    let state_manager = OAuthStateManager::new();
    
    let oauth_state = OAuthState::new(
        OAuthProviderType::Google,
        "http://localhost:3000/callback".to_string(),
        None,
        -1, // Expired 1 second ago
    );
    
    let state_token = state_manager.store_state(oauth_state);
    
    // Expired state should fail
    let result = state_manager
        .validate_and_consume_state(&state_token, &OAuthProviderType::Google);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_oauth_state_manager_invalid_token() {
    let state_manager = OAuthStateManager::new();
    
    // Invalid state token should fail
    let result = state_manager
        .validate_and_consume_state("invalid_token", &OAuthProviderType::Google);
    assert!(result.is_err());
}

// ===== OAuth Token Encryption Tests =====

#[tokio::test]
async fn test_oauth_token_encryption_basic() {
    let key = OAuthTokenEncryption::generate_key();
    let encryption = OAuthTokenEncryption::with_key(&key).unwrap();
    
    let access_token = "test_access_token_12345";
    let refresh_token = Some("test_refresh_token_67890");
    
    // Test encryption
    let encrypted_access = encryption.encrypt(access_token).await.unwrap();
    let encrypted_refresh = if let Some(ref token) = refresh_token {
        Some(encryption.encrypt(token).await.unwrap())
    } else {
        None
    };
    
    // Test decryption
    let decrypted_access = encryption.decrypt(&encrypted_access).await.unwrap();
    let decrypted_refresh = if let Some(ref encrypted) = encrypted_refresh {
        Some(encryption.decrypt(encrypted).await.unwrap())
    } else {
        None
    };
    
    assert_eq!(access_token, decrypted_access);
    assert_eq!(refresh_token, decrypted_refresh);
}

#[tokio::test]
async fn test_oauth_token_encryption_token_pair() {
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
async fn test_oauth_token_encryption_performance() {
    let key = OAuthTokenEncryption::generate_key();
    let encryption = OAuthTokenEncryption::with_key(&key).unwrap();
    
    let access_token = "test_access_token_for_performance_testing";
    
    // Measure encryption performance
    let start_time = std::time::Instant::now();
    let encrypted = encryption.encrypt(access_token).await.unwrap();
    let encrypt_duration = start_time.elapsed();
    
    // Measure decryption performance
    let start_time = std::time::Instant::now();
    let decrypted = encryption.decrypt(&encrypted).await.unwrap();
    let decrypt_duration = start_time.elapsed();
    
    assert_eq!(access_token, decrypted);
    
    // Encryption/decryption should be fast (under 10ms each)
    assert!(encrypt_duration.as_millis() < 10, "Encryption took {}ms, should be under 10ms", encrypt_duration.as_millis());
    assert!(decrypt_duration.as_millis() < 10, "Decryption took {}ms, should be under 10ms", decrypt_duration.as_millis());
}

#[tokio::test]
async fn test_oauth_token_encryption_invalid_data() {
    let key = OAuthTokenEncryption::generate_key();
    let encryption = OAuthTokenEncryption::with_key(&key).unwrap();
    
    // Invalid encrypted data should fail
    let invalid_data = vec![1, 2, 3, 4, 5];
    let result = encryption.decrypt(&invalid_data).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_oauth_token_encryption_different_keys() {
    let key1 = OAuthTokenEncryption::generate_key();
    let key2 = OAuthTokenEncryption::generate_key();
    
    let encryption1 = OAuthTokenEncryption::with_key(&key1).unwrap();
    let encryption2 = OAuthTokenEncryption::with_key(&key2).unwrap();
    
    let token = "test_token";
    let encrypted = encryption1.encrypt(token).await.unwrap();
    
    // Decryption with different key should fail
    let result = encryption2.decrypt(&encrypted).await;
    assert!(result.is_err());
}

// ===== OAuth Provider Mock Tests =====

#[tokio::test]
async fn test_mock_oauth_provider_initiate_flow() {
    let provider = MockOAuthProvider::new(OAuthProviderType::Google);
    
    let result = provider
        .initiate_flow("http://localhost:3000/callback")
        .await;
    
    assert!(result.is_ok());
    let flow_response = result.unwrap();
    
    assert!(flow_response.authorization_url.contains("mock-google.com"));
    assert!(!flow_response.state.is_empty());
    assert!(flow_response.code_verifier.is_some());
}

#[tokio::test]
async fn test_mock_oauth_provider_exchange_code() {
    let provider = MockOAuthProvider::new(OAuthProviderType::Google);
    
    let result = provider
        .exchange_code("mock_code", "mock_state", "http://localhost:3000/callback")
        .await;
    
    assert!(result.is_ok());
    let tokens = result.unwrap();
    
    assert!(!tokens.access_token.is_empty());
    assert!(tokens.refresh_token.is_some());
    assert_eq!(tokens.token_type, "Bearer");
    assert_eq!(tokens.expires_in, Some(3600));
}

#[tokio::test]
async fn test_mock_oauth_provider_get_user_info() {
    let provider = MockOAuthProvider::new(OAuthProviderType::Google);
    
    let result = provider
        .get_user_info("mock_access_token")
        .await;
    
    assert!(result.is_ok());
    let user_info = result.unwrap();
    
    assert!(user_info.provider_user_id.contains("mock_user_Google"));
    assert!(user_info.email.is_some());
    assert_eq!(user_info.email_verified, Some(true));
    assert!(user_info.display_name.is_some());
}

#[tokio::test]
async fn test_mock_oauth_provider_failure_scenarios() {
    let provider = MockOAuthProvider::new(OAuthProviderType::Google)
        .with_failure(MockFailureType::NetworkError);
    
    // Test initiate flow failure
    let result = provider.initiate_flow("http://localhost:3000/callback").await;
    assert!(result.is_err());
    
    // Test exchange code failure
    let provider = MockOAuthProvider::new(OAuthProviderType::Google)
        .with_failure(MockFailureType::InvalidCredentials);
    
    let result = provider.exchange_code("invalid_code", "state", "redirect").await;
    assert!(result.is_err());
    
    // Test get user info failure
    let provider = MockOAuthProvider::new(OAuthProviderType::Google)
        .with_failure(MockFailureType::TokenExpired);
    
    let result = provider.get_user_info("expired_token").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mock_oauth_provider_refresh_token() {
    let provider = MockOAuthProvider::new(OAuthProviderType::Google);
    
    let result = provider
        .refresh_token("mock_refresh_token")
        .await;
    
    assert!(result.is_ok());
    let tokens = result.unwrap();
    
    assert!(tokens.access_token.starts_with("new_mock_access_token"));
    assert!(tokens.refresh_token.is_some());
    assert!(tokens.refresh_token.as_ref().unwrap().starts_with("new_mock_refresh_token"));
}

#[tokio::test]
async fn test_mock_oauth_provider_with_delay() {
    let provider = MockOAuthProvider::new(OAuthProviderType::Google)
        .with_delay(100); // 100ms delay
    
    let start_time = std::time::Instant::now();
    
    let result = provider
        .initiate_flow("http://localhost:3000/callback")
        .await;
    
    let duration = start_time.elapsed();
    
    assert!(result.is_ok());
    assert!(duration.as_millis() >= 100);
}

#[tokio::test]
async fn test_oauth_tokens_serialization() {
    let tokens = create_test_oauth_tokens();
    
    // Test serialization to JSON
    let json = serde_json::to_string(&tokens).unwrap();
    assert!(json.contains("access_token"));
    assert!(json.contains("refresh_token"));
    assert!(json.contains("Bearer"));
    
    // Test deserialization from JSON
    let deserialized: OAuthTokens = serde_json::from_str(&json).unwrap();
    assert_eq!(tokens.access_token, deserialized.access_token);
    assert_eq!(tokens.refresh_token, deserialized.refresh_token);
    assert_eq!(tokens.token_type, deserialized.token_type);
}

#[tokio::test]
async fn test_oauth_user_info_serialization() {
    let user_info = create_test_oauth_user_info(OAuthProviderType::Google);
    
    // Test serialization to JSON
    let json = serde_json::to_string(&user_info).unwrap();
    assert!(json.contains("provider_user_id"));
    assert!(json.contains("email"));
    assert!(json.contains("display_name"));
    
    // Test deserialization from JSON
    let deserialized: OAuthUserInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(user_info.provider_user_id, deserialized.provider_user_id);
    assert_eq!(user_info.email, deserialized.email);
    assert_eq!(user_info.display_name, deserialized.display_name);
}

#[tokio::test]
async fn test_oauth_provider_type_serialization() {
    // Test string conversion
    assert_eq!(OAuthProviderType::Google.to_string(), "google");
    assert_eq!(OAuthProviderType::Apple.to_string(), "apple");
    assert_eq!(OAuthProviderType::GitHub.to_string(), "github");
    
    // Test parsing from string
    assert_eq!("google".parse::<OAuthProviderType>().unwrap(), OAuthProviderType::Google);
    assert_eq!("apple".parse::<OAuthProviderType>().unwrap(), OAuthProviderType::Apple);
    assert_eq!("github".parse::<OAuthProviderType>().unwrap(), OAuthProviderType::GitHub);
    
    // Test invalid provider
    assert!("invalid".parse::<OAuthProviderType>().is_err());
}

#[tokio::test]
async fn test_oauth_flow_response_creation() {
    let flow_response = OAuthFlowResponse {
        authorization_url: "https://example.com/oauth/authorize".to_string(),
        state: "test_state_123".to_string(),
        code_verifier: Some("test_verifier_456".to_string()),
    };
    
    assert_eq!(flow_response.authorization_url, "https://example.com/oauth/authorize");
    assert_eq!(flow_response.state, "test_state_123");
    assert_eq!(flow_response.code_verifier, Some("test_verifier_456".to_string()));
}

// ===== Performance Tests =====

#[tokio::test]
async fn test_oauth_provider_performance() {
    let provider = MockOAuthProvider::new(OAuthProviderType::Google);
    
    // Measure initiate flow performance
    let start_time = std::time::Instant::now();
    let result = provider.initiate_flow("http://localhost:3000/callback").await;
    let duration = start_time.elapsed();
    
    assert!(result.is_ok());
    // Should be very fast for mock provider
    assert!(duration.as_millis() < 10);
}

#[tokio::test]
async fn test_oauth_provider_concurrent_operations() {
    let provider = Arc::new(MockOAuthProvider::new(OAuthProviderType::Google));
    
    // Test concurrent initiate flow calls
    let mut handles = vec![];
    
    for i in 0..10 {
        let provider_clone = Arc::clone(&provider);
        let handle = tokio::spawn(async move {
            provider_clone
                .initiate_flow(&format!("http://localhost:3000/callback/{}", i))
                .await
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}

// ===== Edge Case Tests =====

#[tokio::test]
async fn test_oauth_concurrent_state_validation() {
    let state_manager = Arc::new(OAuthStateManager::new());
    
    let oauth_state = OAuthState::new(
        OAuthProviderType::Google,
        "http://localhost:3000/callback".to_string(),
        None,
        300,
    );
    
    let state_token = state_manager.store_state(oauth_state);
    
    // Simulate concurrent validation attempts
    let state_manager1 = Arc::clone(&state_manager);
    let state_manager2 = Arc::clone(&state_manager);
    let token1 = state_token.clone();
    let token2 = state_token.clone();
    
    let task1 = tokio::spawn(async move {
        state_manager1.validate_and_consume_state(&token1, &OAuthProviderType::Google)
    });
    
    let task2 = tokio::spawn(async move {
        state_manager2.validate_and_consume_state(&token2, &OAuthProviderType::Google)
    });
    
    let (result1, result2) = tokio::join!(task1, task2);
    
    // Only one should succeed (state can only be consumed once)
    let success_count = [result1.unwrap(), result2.unwrap()]
        .iter()
        .filter(|r| r.is_ok())
        .count();
    
    assert_eq!(success_count, 1);
}

#[tokio::test]
async fn test_oauth_provider_validation() {
    let provider = MockOAuthProvider::new(OAuthProviderType::Google);
    
    // Test config validation
    let result = provider.validate_config();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_oauth_token_expiration_status() {
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
    assert!(matches!(no_expiration_status, TokenExpirationStatus::NoExpiration));
}