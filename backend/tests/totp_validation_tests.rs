use music_streaming_blocklist_backend::services::auth::AuthService;
use sqlx::PgPool;

// Mock database pool for testing TOTP validation without database
fn create_mock_auth_service() -> AuthService {
    // Create a mock pool - we won't actually use it for TOTP validation
    let database_url = "postgres://mock:mock@localhost:5432/mock";
    let pool = PgPool::connect_lazy(&database_url).expect("Failed to create mock pool");
    AuthService::new(pool)
}

#[tokio::test]
async fn test_totp_code_validation_edge_cases() {
    let auth_service = create_mock_auth_service();

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
async fn test_totp_code_generation_consistency() {
    let auth_service = create_mock_auth_service();

    let secret = "JBSWY3DPEHPK3PXP"; // Base32 encoded test secret

    // Generate TOTP code for current time
    let current_code = generate_test_totp_code(secret);

    // The same code should be valid when verified immediately
    assert!(auth_service
        .verify_totp_code(secret, &current_code)
        .unwrap());
}

#[tokio::test]
async fn test_totp_invalid_secret_format() {
    let auth_service = create_mock_auth_service();

    // Test with invalid base32 secret
    let result = auth_service.verify_totp_code("invalid_secret", "123456");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_totp_short_secret() {
    let auth_service = create_mock_auth_service();

    // Test with too short secret (less than 10 bytes when decoded)
    let short_secret = "AAAA"; // Only 2.5 bytes when decoded
    let result = auth_service.verify_totp_code(short_secret, "123456");
    assert!(result.is_err());
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
