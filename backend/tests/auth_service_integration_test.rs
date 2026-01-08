use music_streaming_blocklist_backend::models::{CreateUserRequest, LoginRequest};
use music_streaming_blocklist_backend::services::auth::AuthService;
use sqlx::PgPool;
use std::env;

async fn get_test_db_pool() -> PgPool {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://kiro:kiro_dev_password@localhost:5432/kiro_dev".to_string()
    });

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

#[tokio::test]
async fn test_user_registration_and_login() {
    let db_pool = get_test_db_pool().await;
    let auth_service = AuthService::new(db_pool);

    // Test user registration
    let register_request = CreateUserRequest {
        email: format!("test_{}@example.com", uuid::Uuid::new_v4()),
        password: "secure_password123".to_string(),
    };

    let user = auth_service
        .register_user(register_request.clone())
        .await
        .unwrap();

    assert_eq!(user.email, register_request.email);
    assert!(!user.email_verified);
    assert!(!user.totp_enabled);
    assert!(user.password_hash.is_some());

    // Test login
    let login_request = LoginRequest {
        email: register_request.email.clone(),
        password: register_request.password.clone(),
        totp_code: None,
    };

    let token_pair = auth_service.login_user(login_request).await.unwrap();

    assert!(!token_pair.access_token.is_empty());
    assert!(!token_pair.refresh_token.is_empty());
    assert_eq!(token_pair.token_type, "Bearer");
    assert_eq!(token_pair.expires_in, 24 * 60 * 60); // 24 hours

    // Test token verification
    let claims = auth_service.verify_token(&token_pair.access_token).unwrap();
    assert_eq!(claims.sub, user.id.to_string());
    assert_eq!(claims.email, user.email);
}

#[tokio::test]
async fn test_password_hashing_strength() {
    let db_pool = get_test_db_pool().await;
    let auth_service = AuthService::new(db_pool);

    let register_request = CreateUserRequest {
        email: format!("hash_test_{}@example.com", uuid::Uuid::new_v4()),
        password: "test_password_for_hashing".to_string(),
    };

    let user = auth_service.register_user(register_request).await.unwrap();
    let password_hash = user.password_hash.unwrap();

    // Verify bcrypt hash format (should start with $2b$ for bcrypt)
    assert!(password_hash.starts_with("$2b$"));

    // Verify the cost is at least 12 (requirement)
    let parts: Vec<&str> = password_hash.split('$').collect();
    if parts.len() >= 3 {
        let cost: u32 = parts[2].parse().unwrap_or(0);
        assert!(
            cost >= 12,
            "Password hash cost should be at least 12, got {}",
            cost
        );
    }
}

#[tokio::test]
async fn test_refresh_token_functionality() {
    let db_pool = get_test_db_pool().await;
    let auth_service = AuthService::new(db_pool);

    // Register and login
    let register_request = CreateUserRequest {
        email: format!("refresh_test_{}@example.com", uuid::Uuid::new_v4()),
        password: "secure_password123".to_string(),
    };

    auth_service
        .register_user(register_request.clone())
        .await
        .unwrap();

    let login_request = LoginRequest {
        email: register_request.email,
        password: register_request.password,
        totp_code: None,
    };

    let initial_tokens = auth_service.login_user(login_request).await.unwrap();

    // Test refresh token
    let new_tokens = auth_service
        .refresh_token(&initial_tokens.refresh_token)
        .await
        .unwrap();

    assert!(!new_tokens.access_token.is_empty());
    assert!(!new_tokens.refresh_token.is_empty());
    assert_ne!(new_tokens.access_token, initial_tokens.access_token);
    assert_ne!(new_tokens.refresh_token, initial_tokens.refresh_token);

    // Verify old refresh token is invalidated (token rotation)
    let result = auth_service
        .refresh_token(&initial_tokens.refresh_token)
        .await;
    assert!(result.is_err(), "Old refresh token should be invalidated");
}

#[tokio::test]
async fn test_invalid_credentials() {
    let db_pool = get_test_db_pool().await;
    let auth_service = AuthService::new(db_pool);

    // Test login with non-existent user
    let login_request = LoginRequest {
        email: "nonexistent@example.com".to_string(),
        password: "password123".to_string(),
        totp_code: None,
    };

    let result = auth_service.login_user(login_request).await;
    assert!(result.is_err());

    // Register a user
    let register_request = CreateUserRequest {
        email: format!("invalid_test_{}@example.com", uuid::Uuid::new_v4()),
        password: "correct_password".to_string(),
    };

    auth_service
        .register_user(register_request.clone())
        .await
        .unwrap();

    // Test login with wrong password
    let wrong_login = LoginRequest {
        email: register_request.email,
        password: "wrong_password".to_string(),
        totp_code: None,
    };

    let result = auth_service.login_user(wrong_login).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_duplicate_user_registration() {
    let db_pool = get_test_db_pool().await;
    let auth_service = AuthService::new(db_pool);

    let email = format!("duplicate_test_{}@example.com", uuid::Uuid::new_v4());

    let register_request = CreateUserRequest {
        email: email.clone(),
        password: "password123".to_string(),
    };

    // First registration should succeed
    auth_service
        .register_user(register_request.clone())
        .await
        .unwrap();

    // Second registration with same email should fail
    let result = auth_service.register_user(register_request).await;
    assert!(result.is_err());
}
