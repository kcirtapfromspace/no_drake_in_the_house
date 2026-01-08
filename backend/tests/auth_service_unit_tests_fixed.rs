use music_streaming_blocklist_backend::{
    initialize_database, models::*, services::AuthService, DatabaseConfig,
};
use uuid::Uuid;

async fn create_test_auth_service() -> AuthService {
    let config = DatabaseConfig::default();
    let pool = initialize_database(config)
        .await
        .expect("Failed to initialize database");
    AuthService::new(pool)
}

#[tokio::test]
async fn test_password_hashing_strength_fixed() {
    let auth_service = create_test_auth_service().await;

    let register_request = CreateUserRequest {
        email: format!("hash_test_{}@example.com", Uuid::new_v4()),
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
async fn test_user_registration_fixed() {
    let auth_service = create_test_auth_service().await;

    let register_request = CreateUserRequest {
        email: format!("test_{}@example.com", Uuid::new_v4()),
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
}

#[tokio::test]
async fn test_user_login_fixed() {
    let auth_service = create_test_auth_service().await;

    // Register user first
    let unique_email = format!("login_test_{}@example.com", Uuid::new_v4());
    let register_request = CreateUserRequest {
        email: unique_email.clone(),
        password: "secure_password123".to_string(),
    };

    auth_service.register_user(register_request).await.unwrap();

    // Test login
    let login_request = LoginRequest {
        email: unique_email,
        password: "secure_password123".to_string(),
        totp_code: None,
    };

    let token_pair = auth_service.login_user(login_request).await.unwrap();

    assert!(!token_pair.access_token.is_empty());
    assert!(!token_pair.refresh_token.is_empty());
    assert_eq!(token_pair.token_type, "Bearer");
    assert_eq!(token_pair.expires_in, 24 * 60 * 60); // 24 hours
}

#[tokio::test]
async fn test_totp_setup_fixed() {
    let auth_service = create_test_auth_service().await;

    // Create test user
    let unique_email = format!("totp_test_{}@example.com", Uuid::new_v4());
    let register_request = CreateUserRequest {
        email: unique_email,
        password: "password123".to_string(),
    };

    let user = auth_service.register_user(register_request).await.unwrap();

    // Test TOTP setup
    let totp_setup = auth_service.setup_totp(user.id).await.unwrap();

    assert!(!totp_setup.secret.is_empty());
    assert!(totp_setup.qr_code_url.contains("otpauth://totp/"));
    assert_eq!(totp_setup.backup_codes.len(), 8);

    // Verify backup codes are 6 digits
    for code in &totp_setup.backup_codes {
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }
}

#[tokio::test]
async fn test_totp_enable_disable_fixed() {
    let auth_service = create_test_auth_service().await;

    // Create test user and setup TOTP
    let unique_email = format!("totp_enable_test_{}@example.com", Uuid::new_v4());
    let register_request = CreateUserRequest {
        email: unique_email,
        password: "password123".to_string(),
    };

    let user = auth_service.register_user(register_request).await.unwrap();
    let setup_response = auth_service.setup_totp(user.id).await.unwrap();

    // Generate a valid TOTP code
    let totp_code = generate_test_totp_code(&setup_response.secret);

    // Enable TOTP
    let enable_result = auth_service.enable_totp(user.id, &totp_code).await;
    assert!(enable_result.is_ok());

    // Verify 2FA is now enabled
    let status = auth_service.get_totp_status(user.id).await.unwrap();
    assert!(status);

    // Generate a new valid TOTP code for disabling
    let disable_code = generate_test_totp_code(&setup_response.secret);

    // Disable TOTP
    let disable_result = auth_service.disable_totp(user.id, &disable_code).await;
    assert!(disable_result.is_ok());

    // Verify 2FA is now disabled
    let status = auth_service.get_totp_status(user.id).await.unwrap();
    assert!(!status);
}

// Helper function to generate a valid TOTP code for testing
fn generate_test_totp_code(secret: &str) -> String {
    use chrono::Utc;
    use hmac::{Hmac, Mac};
    use sha1::Sha1;

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
