use music_streaming_blocklist_backend::{
    models::*,
    services::AuthService,
    initialize_database,
    DatabaseConfig,
};
use uuid::Uuid;

/// Basic integration tests that work with the current implementation
/// These tests verify the core functionality that is currently available

#[tokio::test]
async fn test_database_and_auth_integration() {
    // Initialize test database
    let config = DatabaseConfig::default();
    let pool = initialize_database(config).await.expect("Failed to initialize database");
    
    // Initialize auth service
    let auth_service = AuthService::new(pool.clone());
    
    // Test user registration
    let unique_email = format!("integration_test_{}@example.com", Uuid::new_v4());
    let registration_request = CreateUserRequest {
        email: unique_email.clone(),
        password: "secure_password123".to_string(),
    };
    
    let user = auth_service.register_user(registration_request).await.unwrap();
    assert_eq!(user.email, unique_email);
    assert!(user.id != Uuid::nil());
    
    // Test user login
    let login_request = LoginRequest {
        email: unique_email,
        password: "secure_password123".to_string(),
        totp_code: None,
    };
    
    let token_pair = auth_service.login_user(login_request).await.unwrap();
    assert!(!token_pair.access_token.is_empty());
    assert!(!token_pair.refresh_token.is_empty());
    
    println!("✅ Database and auth integration test passed");
}

#[tokio::test]
async fn test_auth_service_with_database() {
    let config = DatabaseConfig::default();
    let pool = initialize_database(config).await.expect("Failed to initialize database");
    let auth_service = AuthService::new(pool);
    
    // Test duplicate email registration
    let unique_email = format!("duplicate_test_{}@example.com", Uuid::new_v4());
    let registration_request = CreateUserRequest {
        email: unique_email.clone(),
        password: "password123".to_string(),
    };
    
    // First registration should succeed
    let user1 = auth_service.register_user(registration_request.clone()).await.unwrap();
    assert_eq!(user1.email, unique_email);
    
    // Second registration with same email should fail
    let result = auth_service.register_user(registration_request).await;
    assert!(result.is_err());
    
    println!("✅ Auth service duplicate email test passed");
}

#[tokio::test]
async fn test_invalid_login_credentials() {
    let config = DatabaseConfig::default();
    let pool = initialize_database(config).await.expect("Failed to initialize database");
    let auth_service = AuthService::new(pool);
    
    // Test login with non-existent user
    let login_request = LoginRequest {
        email: "nonexistent@example.com".to_string(),
        password: "password123".to_string(),
        totp_code: None,
    };
    
    let result = auth_service.login_user(login_request).await;
    assert!(result.is_err());
    
    // Register a user first
    let unique_email = format!("valid_user_{}@example.com", Uuid::new_v4());
    let registration_request = CreateUserRequest {
        email: unique_email.clone(),
        password: "correct_password".to_string(),
    };
    
    auth_service.register_user(registration_request).await.unwrap();
    
    // Test login with wrong password
    let wrong_password_login = LoginRequest {
        email: unique_email,
        password: "wrong_password".to_string(),
        totp_code: None,
    };
    
    let result = auth_service.login_user(wrong_password_login).await;
    assert!(result.is_err());
    
    println!("✅ Invalid login credentials test passed");
}

#[tokio::test]
async fn test_token_validation() {
    let config = DatabaseConfig::default();
    let pool = initialize_database(config).await.expect("Failed to initialize database");
    let auth_service = AuthService::new(pool);
    
    // Register and login user
    let unique_email = format!("token_test_{}@example.com", Uuid::new_v4());
    let registration_request = CreateUserRequest {
        email: unique_email.clone(),
        password: "password123".to_string(),
    };
    
    let user = auth_service.register_user(registration_request).await.unwrap();
    
    let login_request = LoginRequest {
        email: unique_email.clone(),
        password: "password123".to_string(),
        totp_code: None,
    };
    
    let token_pair = auth_service.login_user(login_request).await.unwrap();
    
    // Test token validation
    let claims = auth_service.validate_access_token(&token_pair.access_token).await.unwrap();
    assert_eq!(claims.email, unique_email);
    assert_eq!(claims.sub, user.id.to_string());
    
    // Test invalid token
    let invalid_token = "invalid.token.here";
    let result = auth_service.validate_access_token(invalid_token).await;
    assert!(result.is_err());
    
    println!("✅ Token validation test passed");
}