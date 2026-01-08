use music_streaming_blocklist_backend::{
    error::AppError,
    models::oauth::{OAuthConfig, OAuthProviderType, OAuthState, OAuthTokens, OAuthUserInfo},
    services::oauth::{BaseOAuthProvider, OAuthStateManager},
    services::oauth_encryption::OAuthTokenEncryption,
};
use serde_json::json;
use std::collections::HashMap;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn create_test_config() -> OAuthConfig {
    OAuthConfig {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        redirect_uri: "http://localhost:3000/auth/callback".to_string(),
        scopes: vec![
            "openid".to_string(),
            "email".to_string(),
            "profile".to_string(),
        ],
        additional_params: HashMap::new(),
    }
}

#[tokio::test]
async fn test_oauth_provider_type_conversions() {
    // Test Display trait
    assert_eq!(OAuthProviderType::Google.to_string(), "google");
    assert_eq!(OAuthProviderType::Apple.to_string(), "apple");
    assert_eq!(OAuthProviderType::GitHub.to_string(), "github");

    // Test FromStr trait
    assert_eq!(
        "google".parse::<OAuthProviderType>().unwrap(),
        OAuthProviderType::Google
    );
    assert_eq!(
        "apple".parse::<OAuthProviderType>().unwrap(),
        OAuthProviderType::Apple
    );
    assert_eq!(
        "github".parse::<OAuthProviderType>().unwrap(),
        OAuthProviderType::GitHub
    );

    // Test case insensitive parsing
    assert_eq!(
        "GOOGLE".parse::<OAuthProviderType>().unwrap(),
        OAuthProviderType::Google
    );
    assert_eq!(
        "Apple".parse::<OAuthProviderType>().unwrap(),
        OAuthProviderType::Apple
    );

    // Test invalid provider
    assert!("invalid".parse::<OAuthProviderType>().is_err());
}

#[test]
fn test_oauth_state_creation_and_validation() {
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

    // Check expiration
    assert!(!state.is_expired());
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
    let retrieved_state =
        manager.validate_and_consume_state(&state_token, &OAuthProviderType::Google);
    assert!(retrieved_state.is_ok());

    // State should be consumed (not retrievable again)
    let second_attempt =
        manager.validate_and_consume_state(&state_token, &OAuthProviderType::Google);
    assert!(second_attempt.is_err());
}

#[test]
fn test_oauth_state_manager_invalid_state() {
    let manager = OAuthStateManager::new();

    // Non-existent state
    let result = manager.validate_and_consume_state("non_existent", &OAuthProviderType::Google);
    assert!(result.is_err());

    // Wrong provider
    let state = OAuthState::new(
        OAuthProviderType::Google,
        "http://localhost:3000/auth/callback".to_string(),
        None,
        300,
    );

    let state_token = manager.store_state(state);
    let result = manager.validate_and_consume_state(&state_token, &OAuthProviderType::Apple);
    assert!(result.is_err());
}

#[test]
fn test_base_oauth_provider_auth_url_generation() {
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
    assert!(auth_url.starts_with("https://accounts.google.com/o/oauth2/v2/auth?"));
}

#[test]
fn test_base_oauth_provider_auth_url_with_additional_params() {
    let mut config = create_test_config();
    config
        .additional_params
        .insert("access_type".to_string(), "offline".to_string());

    let provider = BaseOAuthProvider::new(
        config,
        OAuthProviderType::Google,
        "https://oauth2.googleapis.com/token".to_string(),
        "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
        "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
        None,
    );

    let redirect_uri = "http://localhost:3000/auth/callback";
    let state = "test_state";

    let mut additional_params = HashMap::new();
    additional_params.insert("prompt".to_string(), "consent".to_string());

    let auth_url = provider.build_auth_url(redirect_uri, state, Some(additional_params));

    assert!(auth_url.contains("access_type=offline"));
    assert!(auth_url.contains("prompt=consent"));
}

#[tokio::test]
async fn test_base_oauth_provider_token_exchange_success() {
    let mock_server = MockServer::start().await;

    // Mock successful token exchange
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(header("content-type", "application/x-www-form-urlencoded"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "mock_access_token",
            "refresh_token": "mock_refresh_token",
            "expires_in": 3600,
            "token_type": "Bearer",
            "scope": "openid email profile",
            "id_token": "mock_id_token"
        })))
        .mount(&mock_server)
        .await;

    let config = create_test_config();
    let provider = BaseOAuthProvider::new(
        config,
        OAuthProviderType::Google,
        format!("{}/token", mock_server.uri()),
        "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
        "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
        None,
    );

    let result = provider
        .exchange_code_standard("test_code", "http://localhost:3000/auth/callback")
        .await;

    assert!(result.is_ok());
    let tokens = result.unwrap();
    assert_eq!(tokens.access_token, "mock_access_token");
    assert_eq!(tokens.refresh_token, Some("mock_refresh_token".to_string()));
    assert_eq!(tokens.expires_in, Some(3600));
    assert_eq!(tokens.token_type, "Bearer");
    assert_eq!(tokens.scope, Some("openid email profile".to_string()));
    assert_eq!(tokens.id_token, Some("mock_id_token".to_string()));
}

#[tokio::test]
async fn test_base_oauth_provider_token_exchange_error() {
    let mock_server = MockServer::start().await;

    // Mock failed token exchange
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "invalid_grant",
            "error_description": "Invalid authorization code"
        })))
        .mount(&mock_server)
        .await;

    let config = create_test_config();
    let provider = BaseOAuthProvider::new(
        config,
        OAuthProviderType::Google,
        format!("{}/token", mock_server.uri()),
        "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
        "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
        None,
    );

    let result = provider
        .exchange_code_standard("invalid_code", "http://localhost:3000/auth/callback")
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::ExternalServiceError(msg) => {
            assert!(msg.contains("Token exchange failed"));
        }
        _ => panic!("Expected ExternalServiceError"),
    }
}

#[tokio::test]
async fn test_base_oauth_provider_user_info_success() {
    let mock_server = MockServer::start().await;

    // Mock successful user info request
    Mock::given(method("GET"))
        .and(path("/userinfo"))
        .and(header("authorization", "Bearer mock_access_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "123456789",
            "email": "test@example.com",
            "verified_email": true,
            "name": "Test User",
            "given_name": "Test",
            "family_name": "User",
            "picture": "https://example.com/avatar.jpg",
            "locale": "en"
        })))
        .mount(&mock_server)
        .await;

    let config = create_test_config();
    let provider = BaseOAuthProvider::new(
        config,
        OAuthProviderType::Google,
        "https://oauth2.googleapis.com/token".to_string(),
        "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
        format!("{}/userinfo", mock_server.uri()),
        None,
    );

    let result = provider.get_user_info_standard("mock_access_token").await;

    assert!(result.is_ok());
    let user_info = result.unwrap();
    assert_eq!(user_info["id"].as_str().unwrap(), "123456789");
    assert_eq!(user_info["email"].as_str().unwrap(), "test@example.com");
    assert_eq!(user_info["verified_email"].as_bool().unwrap(), true);
    assert_eq!(user_info["name"].as_str().unwrap(), "Test User");
}

#[tokio::test]
async fn test_base_oauth_provider_user_info_error() {
    let mock_server = MockServer::start().await;

    // Mock failed user info request
    Mock::given(method("GET"))
        .and(path("/userinfo"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "invalid_token",
            "error_description": "The access token is invalid"
        })))
        .mount(&mock_server)
        .await;

    let config = create_test_config();
    let provider = BaseOAuthProvider::new(
        config,
        OAuthProviderType::Google,
        "https://oauth2.googleapis.com/token".to_string(),
        "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
        format!("{}/userinfo", mock_server.uri()),
        None,
    );

    let result = provider.get_user_info_standard("invalid_token").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::ExternalServiceError(msg) => {
            assert!(msg.contains("User info request failed"));
        }
        _ => panic!("Expected ExternalServiceError"),
    }
}

#[tokio::test]
async fn test_base_oauth_provider_refresh_token_success() {
    let mock_server = MockServer::start().await;

    // Mock successful token refresh
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(header("content-type", "application/x-www-form-urlencoded"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "new_access_token",
            "expires_in": 3600,
            "token_type": "Bearer",
            "scope": "openid email profile"
        })))
        .mount(&mock_server)
        .await;

    let config = create_test_config();
    let provider = BaseOAuthProvider::new(
        config,
        OAuthProviderType::Google,
        format!("{}/token", mock_server.uri()),
        "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
        "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
        None,
    );

    let result = provider.refresh_token_standard("mock_refresh_token").await;

    assert!(result.is_ok());
    let tokens = result.unwrap();
    assert_eq!(tokens.access_token, "new_access_token");
    assert_eq!(tokens.expires_in, Some(3600));
    assert_eq!(tokens.token_type, "Bearer");
    assert!(tokens.id_token.is_none()); // Refresh responses don't include ID tokens
}

#[tokio::test]
async fn test_base_oauth_provider_revoke_token_success() {
    let mock_server = MockServer::start().await;

    // Mock successful token revocation
    Mock::given(method("POST"))
        .and(path("/revoke"))
        .and(header("content-type", "application/x-www-form-urlencoded"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let config = create_test_config();
    let provider = BaseOAuthProvider::new(
        config,
        OAuthProviderType::Google,
        "https://oauth2.googleapis.com/token".to_string(),
        "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
        "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
        Some(format!("{}/revoke", mock_server.uri())),
    );

    let result = provider.revoke_token_standard("mock_token").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_base_oauth_provider_revoke_token_no_endpoint() {
    let config = create_test_config();
    let provider = BaseOAuthProvider::new(
        config,
        OAuthProviderType::Google,
        "https://oauth2.googleapis.com/token".to_string(),
        "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
        "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
        None, // No revoke endpoint
    );

    let result = provider.revoke_token_standard("mock_token").await;
    assert!(result.is_ok()); // Should succeed even without revoke endpoint
}

#[test]
fn test_oauth_token_encryption() {
    let key = OAuthTokenEncryption::generate_key();
    let encryption = OAuthTokenEncryption::with_key(&key).unwrap();

    let access_token = "test_access_token_12345";
    let refresh_token = Some("test_refresh_token_67890");

    // Test token pair encryption
    let (encrypted_access, encrypted_refresh) = encryption
        .encrypt_token_pair(access_token, refresh_token.as_deref())
        .unwrap();

    let (decrypted_access, decrypted_refresh) = encryption
        .decrypt_token_pair(&encrypted_access, encrypted_refresh.as_deref())
        .unwrap();

    assert_eq!(access_token, decrypted_access);
    assert_eq!(refresh_token, decrypted_refresh);
}

#[test]
fn test_oauth_token_encryption_different_nonces() {
    let key = OAuthTokenEncryption::generate_key();
    let encryption = OAuthTokenEncryption::with_key(&key).unwrap();

    let token = "same_token";
    let encrypted1 = encryption.encrypt_token(token).unwrap();
    let encrypted2 = encryption.encrypt_token(token).unwrap();

    // Same token should produce different ciphertext due to random nonces
    assert_ne!(encrypted1, encrypted2);

    // But both should decrypt to the same token
    let decrypted1 = encryption.decrypt_token(&encrypted1).unwrap();
    let decrypted2 = encryption.decrypt_token(&encrypted2).unwrap();

    assert_eq!(token, decrypted1);
    assert_eq!(token, decrypted2);
}
