use music_streaming_blocklist_backend::services::auth::AuthService;
use music_streaming_blocklist_backend::models::{CreateUserRequest, LoginRequest, TotpEnableRequest, TotpDisableRequest};
use sqlx::PgPool;
use uuid::Uuid;
use anyhow::Result;

// Test helper to create a test database pool
async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/test_db".to_string());
    
    PgPool::connect(&database_url).await.expect("Failed to connect to test database")
}

// Test helper to create a test user
async fn create_test_user(auth_service: &AuthService) -> Result<Uuid> {
    let request = CreateUserRequest {
        email: format!("test_{}@example.com", Uuid::new_v4()),
        password: "test_password123".to_string(),
    };
    
    let user = auth_service.register_user(request).await?;
    Ok(user.id)
}

#[tokio::test]
async fn test_totp_setup_success() {
    let pool = create_test_pool().await;
    let auth_service = AuthService::new(pool);
    
    // Create test user
    let user_id = create_test_user(&auth_service).await.unwrap();
    
    // Setup TOTP
    let setup_response = auth_service.setup_totp(user_id).await.unwrap();
    
    // Verify response structure
    assert!(!setup_response.secret.is_empty());
    assert!(setup_response.qr_code_url.contains("otpauth://totp/"));
    assert!(setup_response.qr_code_url.contains(&setup_response.secret));
    assert_eq!(setup_response.backup_codes.len(), 8);
    
    // Verify backup codes are 6 digits
    for code in &setup_response.backup_codes {
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }
    
    // Verify 2FA is not yet enabled
    let status = auth_service.get_totp_status(user_id).await.unwrap();
    assert!(!status);
}

#[tokio::test]
async fn test_totp_setup_already_enabled() {
    let pool = create_test_pool().await;
    let auth_service = AuthService::new(pool);
    
    // Create test user and setup TOTP
    let user_id = create_test_user(&auth_service).await.unwrap();
    let setup_response = auth_service.setup_totp(user_id).await.unwrap();
    
    // Generate a valid TOTP code
    let totp_code = generate_test_totp_code(&setup_response.secret);
    
    // Enable TOTP
    auth_service.enable_totp(user_id, &totp_code).await.unwrap();
    
    // Try to setup TOTP again - should fail
    let result = auth_service.setup_totp(user_id).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already enabled"));
}

#[tokio::test]
async fn test_totp_enable_success() {
    let pool = create_test_pool().await;
    let auth_service = AuthService::new(pool);
    
    // Create test user and setup TOTP
    let user_id = create_test_user(&auth_service).await.unwrap();
    let setup_response = auth_service.setup_totp(user_id).await.unwrap();
    
    // Generate a valid TOTP code
    let totp_code = generate_test_totp_code(&setup_response.secret);
    
    // Enable TOTP
    let result = auth_service.enable_totp(user_id, &totp_code).await;
    assert!(result.is_ok());
    
    // Verify 2FA is now enabled
    let status = auth_service.get_totp_status(user_id).await.unwrap();
    assert!(status);
}

#[tokio::test]
async fn test_totp_enable_invalid_code() {
    let pool = create_test_pool().await;
    let auth_service = AuthService::new(pool);
    
    // Create test user and setup TOTP
    let user_id = create_test_user(&auth_service).await.unwrap();
    auth_service.setup_totp(user_id).await.unwrap();
    
    // Try to enable with invalid code
    let result = auth_service.enable_totp(user_id, "123456").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid TOTP code"));
    
    // Verify 2FA is still not enabled
    let status = auth_service.get_totp_status(user_id).await.unwrap();
    assert!(!status);
}

#[tokio::test]
async fn test_totp_enable_already_enabled() {
    let pool = create_test_pool().await;
    let auth_service = AuthService::new(pool);
    
    // Create test user, setup and enable TOTP
    let user_id = create_test_user(&auth_service).await.unwrap();
    let setup_response = auth_service.setup_totp(user_id).await.unwrap();
    let totp_code = generate_test_totp_code(&setup_response.secret);
    auth_service.enable_totp(user_id, &totp_code).await.unwrap();
    
    // Try to enable again
    let result = auth_service.enable_totp(user_id, &totp_code).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already enabled"));
}

#[tokio::test]
async fn test_totp_enable_without_setup() {
    let pool = create_test_pool().await;
    let auth_service = AuthService::new(pool);
    
    // Create test user without TOTP setup
    let user_id = create_test_user(&auth_service).await.unwrap();
    
    // Try to enable TOTP without setup
    let result = auth_service.enable_totp(user_id, "123456").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("setup not initiated"));
}

#[tokio::test]
async fn test_totp_disable_success() {
    let pool = create_test_pool().await;
    let auth_service = AuthService::new(pool);
    
    // Create test user, setup and enable TOTP
    let user_id = create_test_user(&auth_service).await.unwrap();
    let setup_response = auth_service.setup_totp(user_id).await.unwrap();
    let totp_code = generate_test_totp_code(&setup_response.secret);
    auth_service.enable_totp(user_id, &totp_code).await.unwrap();
    
    // Generate a new valid TOTP code for disabling
    let disable_code = generate_test_totp_code(&setup_response.secret);
    
    // Disable TOTP
    let result = auth_service.disable_totp(user_id, &disable_code).await;
    assert!(result.is_ok());
    
    // Verify 2FA is now disabled
    let status = auth_service.get_totp_status(user_id).await.unwrap();
    assert!(!status);
}

#[tokio::test]
async fn test_totp_disable_invalid_code() {
    let pool = create_test_pool().await;
    let auth_service = AuthService::new(pool);
    
    // Create test user, setup and enable TOTP
    let user_id = create_test_user(&auth_service).await.unwrap();
    let setup_response = auth_service.setup_totp(user_id).await.unwrap();
    let totp_code = generate_test_totp_code(&setup_response.secret);
    auth_service.enable_totp(user_id, &totp_code).await.unwrap();
    
    // Try to disable with invalid code
    let result = auth_service.disable_totp(user_id, "123456").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid TOTP code"));
    
    // Verify 2FA is still enabled
    let status = auth_service.get_totp_status(user_id).await.unwrap();
    assert!(status);
}

#[tokio::test]
async fn test_totp_disable_not_enabled() {
    let pool = create_test_pool().await;
    let auth_service = AuthService::new(pool);
    
    // Create test user without enabling TOTP
    let user_id = create_test_user(&auth_service).await.unwrap();
    
    // Try to disable TOTP when not enabled
    let result = auth_service.disable_totp(user_id, "123456").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not enabled"));
}

#[tokio::test]
async fn test_login_with_totp_success() {
    let pool = create_test_pool().await;
    let auth_service = AuthService::new(pool);
    
    // Create test user, setup and enable TOTP
    let user_email = format!("test_{}@example.com", Uuid::new_v4());
    let user_password = "test_password123".to_string();
    
    let create_request = CreateUserRequest {
        email: user_email.clone(),
        password: user_password.clone(),
    };
    let user = auth_service.register_user(create_request).await.unwrap();
    
    let setup_response = auth_service.setup_totp(user.id).await.unwrap();
    let totp_code = generate_test_totp_code(&setup_response.secret);
    auth_service.enable_totp(user.id, &totp_code).await.unwrap();
    
    // Login with TOTP
    let login_totp_code = generate_test_totp_code(&setup_response.secret);
    let login_request = LoginRequest {
        email: user_email,
        password: user_password,
        totp_code: Some(login_totp_code),
    };
    
    let result = auth_service.login_user(login_request).await;
    assert!(result.is_ok());
    
    let token_pair = result.unwrap();
    assert!(!token_pair.access_token.is_empty());
    assert!(!token_pair.refresh_token.is_empty());
}

#[tokio::test]
async fn test_login_with_totp_missing_code() {
    let pool = create_test_pool().await;
    let auth_service = AuthService::new(pool);
    
    // Create test user, setup and enable TOTP
    let user_email = format!("test_{}@example.com", Uuid::new_v4());
    let user_password = "test_password123".to_string();
    
    let create_request = CreateUserRequest {
        email: user_email.clone(),
        password: user_password.clone(),
    };
    let user = auth_service.register_user(create_request).await.unwrap();
    
    let setup_response = auth_service.setup_totp(user.id).await.unwrap();
    let totp_code = generate_test_totp_code(&setup_response.secret);
    auth_service.enable_totp(user.id, &totp_code).await.unwrap();
    
    // Try to login without TOTP code
    let login_request = LoginRequest {
        email: user_email,
        password: user_password,
        totp_code: None,
    };
    
    let result = auth_service.login_user(login_request).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("2FA code required"));
}

#[tokio::test]
async fn test_login_with_totp_invalid_code() {
    let pool = create_test_pool().await;
    let auth_service = AuthService::new(pool);
    
    // Create test user, setup and enable TOTP
    let user_email = format!("test_{}@example.com", Uuid::new_v4());
    let user_password = "test_password123".to_string();
    
    let create_request = CreateUserRequest {
        email: user_email.clone(),
        password: user_password.clone(),
    };
    let user = auth_service.register_user(create_request).await.unwrap();
    
    let setup_response = auth_service.setup_totp(user.id).await.unwrap();
    let totp_code = generate_test_totp_code(&setup_response.secret);
    auth_service.enable_totp(user.id, &totp_code).await.unwrap();
    
    // Try to login with invalid TOTP code
    let login_request = LoginRequest {
        email: user_email,
        password: user_password,
        totp_code: Some("123456".to_string()),
    };
    
    let result = auth_service.login_user(login_request).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid 2FA code"));
}

#[tokio::test]
async fn test_totp_code_validation_edge_cases() {
    let pool = create_test_pool().await;
    let auth_service = AuthService::new(pool);
    
    let secret = "JBSWY3DPEHPK3PXP"; // Base32 encoded test secret
    
    // Test invalid code formats
    assert!(!auth_service.verify_totp_code(secret, "12345").unwrap()); // Too short
    assert!(!auth_service.verify_totp_code(secret, "1234567").unwrap()); // Too long
    assert!(!auth_service.verify_totp_code(secret, "12345a").unwrap()); // Contains letter
    assert!(!auth_service.verify_totp_code(secret, "").unwrap()); // Empty
    
    // Test with valid format but wrong code
    assert!(!auth_service.verify_totp_code(secret, "000000").unwrap());
}

#[tokio::test]
async fn test_totp_clock_skew_tolerance() {
    let pool = create_test_pool().await;
    let auth_service = AuthService::new(pool);
    
    // Create test user and setup TOTP
    let user_id = create_test_user(&auth_service).await.unwrap();
    let setup_response = auth_service.setup_totp(user_id).await.unwrap();
    
    // Generate TOTP code for current time
    let current_code = generate_test_totp_code(&setup_response.secret);
    
    // The code should be valid (within clock skew tolerance)
    assert!(auth_service.verify_totp_code(&setup_response.secret, &current_code).unwrap());
}

// Helper function to generate a valid TOTP code for testing
fn generate_test_totp_code(secret: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha1::Sha1;
    use chrono::Utc;
    
    let secret_bytes = base32::decode(base32::Alphabet::Rfc4648 { padding: true }, secret)
        .expect("Invalid test secret");
    
    let current_time = Utc::now().timestamp() as u64;
    let time_step = current_time / 30;
    let time_bytes = time_step.to_be_bytes();
    
    type HmacSha1 = Hmac<Sha1>;
    let mut mac = HmacSha1::new_from_slice(&secret_bytes).expect("Invalid secret");
    mac.update(&time_bytes);
    let result = mac.finalize().into_bytes();
    
    let offset = (result[result.len() - 1] & 0xf) as usize;
    let code = ((result[offset] as u32 & 0x7f) << 24)
        | ((result[offset + 1] as u32 & 0xff) << 16)
        | ((result[offset + 2] as u32 & 0xff) << 8)
        | (result[offset + 3] as u32 & 0xff);
    
    format!("{:06}", code % 1000000)
}