use music_streaming_blocklist_backend::services::auth_simple::AuthService;
use music_streaming_blocklist_backend::models::{CreateUserRequest, LoginRequest};
use sqlx::PgPool;
use std::env;

async fn get_test_db_pool() -> PgPool {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://kiro:kiro_dev_password@localhost:5432/kiro_dev".to_string());
    
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

#[tokio::test]
async fn test_auth_service_integration() {
    let db_pool = get_test_db_pool().await;
    let auth_service = AuthService::new().with_database(db_pool);

    // Test user registration
    let register_request = CreateUserRequest {
        email: "test_auth@example.com".to_string(),
        password: "testpassword123".to_string(),
    };

    let user = auth_service.register_user(register_request).await;
    assert!(user.is_ok(), "User registration should succeed");
    
    let user = user.unwrap();
    assert_eq!(user.email, "test_auth@example.com");
    assert!(user.password_hash.is_some());

    // Test user login
    let login_request = LoginRequest {
        email: "test_auth@example.com".to_string(),
        password: "testpassword123".to_string(),
        totp_code: None,
    };

    let token_pair = auth_service.login_user(login_request).await;
    assert!(token_pair.is_ok(), "User login should succeed");
    
    let token_pair = token_pair.unwrap();
    assert!(!token_pair.access_token.is_empty());
    assert!(!token_pair.refresh_token.is_empty());
    assert_eq!(token_pair.token_type, "Bearer");

    // Test token verification
    let claims = auth_service.verify_token(&token_pair.access_token);
    assert!(claims.is_ok(), "Token verification should succeed");
    
    let claims = claims.unwrap();
    assert_eq!(claims.sub, user.id.to_string());
    assert_eq!(claims.email, "test_auth@example.com");

    // Test get user
    let retrieved_user = auth_service.get_user(user.id).await;
    assert!(retrieved_user.is_ok(), "Get user should succeed");
    
    let retrieved_user = retrieved_user.unwrap();
    assert_eq!(retrieved_user.id, user.id);
    assert_eq!(retrieved_user.email, "test_auth@example.com");

    // Clean up - delete test user
    let _ = sqlx::query!("DELETE FROM users WHERE email = $1", "test_auth@example.com")
        .execute(auth_service.db_pool.as_ref().unwrap())
        .await;
}

#[tokio::test]
async fn test_auth_service_duplicate_email() {
    let db_pool = get_test_db_pool().await;
    let auth_service = AuthService::new().with_database(db_pool);

    // Register first user
    let register_request = CreateUserRequest {
        email: "duplicate_test@example.com".to_string(),
        password: "testpassword123".to_string(),
    };

    let result1 = auth_service.register_user(register_request.clone()).await;
    assert!(result1.is_ok(), "First registration should succeed");

    // Try to register same email again
    let result2 = auth_service.register_user(register_request).await;
    assert!(result2.is_err(), "Duplicate email registration should fail");

    // Clean up
    let _ = sqlx::query!("DELETE FROM users WHERE email = $1", "duplicate_test@example.com")
        .execute(auth_service.db_pool.as_ref().unwrap())
        .await;
}

#[tokio::test]
async fn test_auth_service_invalid_credentials() {
    let db_pool = get_test_db_pool().await;
    let auth_service = AuthService::new().with_database(db_pool);

    // Try to login with non-existent user
    let login_request = LoginRequest {
        email: "nonexistent@example.com".to_string(),
        password: "wrongpassword".to_string(),
        totp_code: None,
    };

    let result = auth_service.login_user(login_request).await;
    assert!(result.is_err(), "Login with invalid credentials should fail");
}