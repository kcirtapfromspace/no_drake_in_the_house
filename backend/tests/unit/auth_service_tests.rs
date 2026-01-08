use crate::common::*;
use music_streaming_blocklist_backend::{models::*, services::AuthService};
use rstest::*;
use serial_test::serial;
use uuid::Uuid;

#[fixture]
async fn test_db() -> TestDatabase {
    TestDatabase::new().await
}

#[fixture]
async fn auth_service(#[future] test_db: TestDatabase) -> AuthService {
    let db = test_db.await;
    AuthService::new(db.pool)
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_user_registration_success(#[future] test_db: TestDatabase) {
    let db = test_db.await;
    let auth_service = AuthService::new(db.pool.clone());

    let request = TestDataFactory::create_user_request();
    let user = auth_service.register_user(request.clone()).await.unwrap();

    assert_eq!(user.email, request.email);
    TestAssertions::assert_valid_uuid(&user.id.to_string());
    TestAssertions::assert_bcrypt_hash(&user.password_hash.unwrap());
    assert!(!user.email_verified);
    assert!(!user.totp_enabled);
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_user_registration_duplicate_email(#[future] test_db: TestDatabase) {
    let db = test_db.await;
    let auth_service = AuthService::new(db.pool.clone());

    let request = TestDataFactory::create_user_request();

    // First registration should succeed
    auth_service.register_user(request.clone()).await.unwrap();

    // Second registration with same email should fail
    let result = auth_service.register_user(request).await;
    assert!(result.is_err());
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_user_login_success(#[future] test_db: TestDatabase) {
    let db = test_db.await;
    let auth_service = AuthService::new(db.pool.clone());

    // Register user first
    let register_request = TestDataFactory::create_user_request();
    auth_service
        .register_user(register_request.clone())
        .await
        .unwrap();

    // Test login
    let login_request = TestDataFactory::create_login_request(register_request.email);
    let token_pair = auth_service.login_user(login_request).await.unwrap();

    TestAssertions::assert_valid_jwt(&token_pair.access_token);
    TestAssertions::assert_valid_jwt(&token_pair.refresh_token);
    assert_eq!(token_pair.token_type, "Bearer");
    assert_eq!(token_pair.expires_in, 24 * 60 * 60); // 24 hours
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_user_login_invalid_credentials(#[future] test_db: TestDatabase) {
    let db = test_db.await;
    let auth_service = AuthService::new(db.pool.clone());

    // Test login with non-existent user
    let login_request = LoginRequest {
        email: "nonexistent@example.com".to_string(),
        password: "password123".to_string(),
        totp_code: None,
    };

    let result = auth_service.login_user(login_request).await;
    assert!(result.is_err());
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_token_validation(#[future] test_db: TestDatabase) {
    let db = test_db.await;
    let auth_service = AuthService::new(db.pool.clone());

    // Register and login user
    let register_request = TestDataFactory::create_user_request();
    let user = auth_service
        .register_user(register_request.clone())
        .await
        .unwrap();

    let login_request = TestDataFactory::create_login_request(register_request.email.clone());
    let token_pair = auth_service.login_user(login_request).await.unwrap();

    // Test token validation
    let claims = auth_service
        .validate_access_token(&token_pair.access_token)
        .await
        .unwrap();
    assert_eq!(claims.email, register_request.email);
    assert_eq!(claims.sub, user.id.to_string());
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_totp_setup_and_enable(#[future] test_db: TestDatabase) {
    let db = test_db.await;
    let auth_service = AuthService::new(db.pool.clone());

    // Create test user
    let user = db.create_test_user().await;

    // Test TOTP setup
    let totp_setup = auth_service.setup_totp(user.id).await.unwrap();

    assert!(!totp_setup.secret.is_empty());
    assert!(totp_setup.qr_code_url.contains("otpauth://totp/"));
    assert_eq!(totp_setup.backup_codes.len(), 8);

    // Verify backup codes format
    for code in &totp_setup.backup_codes {
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    // Generate valid TOTP code and enable
    let totp_code = generate_test_totp_code(&totp_setup.secret);
    let enable_result = auth_service.enable_totp(user.id, &totp_code).await;
    assert!(enable_result.is_ok());

    // Verify 2FA is enabled
    let status = auth_service.get_totp_status(user.id).await.unwrap();
    assert!(status);
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_password_strength_requirements(#[future] test_db: TestDatabase) {
    let db = test_db.await;
    let auth_service = AuthService::new(db.pool.clone());

    // Test with weak password (should still work as validation is on frontend)
    let request = CreateUserRequest {
        email: format!("weak_password_{}@example.com", Uuid::new_v4()),
        password: "123".to_string(),
    };

    // The service should still hash even weak passwords
    let user = auth_service.register_user(request).await.unwrap();
    TestAssertions::assert_bcrypt_hash(&user.password_hash.unwrap());
}

// Helper function to generate valid TOTP codes for testing
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

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Duration;

    #[rstest]
    #[tokio::test]
    #[serial]
    async fn test_registration_performance(#[future] test_db: TestDatabase) {
        let db = test_db.await;
        let auth_service = AuthService::new(db.pool.clone());

        let request = TestDataFactory::create_user_request();

        let (result, duration) =
            PerformanceTestHelper::measure_async(|| auth_service.register_user(request.clone()))
                .await;

        assert!(result.is_ok());
        PerformanceTestHelper::assert_performance_threshold(duration, 1000); // 1 second max
    }

    #[rstest]
    #[tokio::test]
    #[serial]
    async fn test_login_performance(#[future] test_db: TestDatabase) {
        let db = test_db.await;
        let auth_service = AuthService::new(db.pool.clone());

        // Setup user
        let register_request = TestDataFactory::create_user_request();
        auth_service
            .register_user(register_request.clone())
            .await
            .unwrap();

        let login_request = TestDataFactory::create_login_request(register_request.email);

        let (result, duration) =
            PerformanceTestHelper::measure_async(|| auth_service.login_user(login_request.clone()))
                .await;

        assert!(result.is_ok());
        PerformanceTestHelper::assert_performance_threshold(duration, 500); // 500ms max
    }
}
