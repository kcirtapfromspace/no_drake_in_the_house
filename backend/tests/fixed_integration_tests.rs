use music_streaming_blocklist_backend::{
    models::*,
    services::AuthService,
    initialize_database,
    DatabaseConfig,
    DnpListService,
};
use uuid::Uuid;


/// Fixed integration tests that work with the current implementation
/// These tests verify the core functionality that is currently available

#[tokio::test]
async fn test_database_and_auth_integration_fixed() {
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
async fn test_dnp_service_integration_fixed() {
    let config = DatabaseConfig::default();
    let pool = initialize_database(config).await.expect("Failed to initialize database");
    let auth_service = AuthService::new(pool.clone());
    let dnp_service = DnpListService::new(pool.clone());
    
    // Create a test user
    let unique_email = format!("dnp_test_{}@example.com", Uuid::new_v4());
    let registration_request = CreateUserRequest {
        email: unique_email.clone(),
        password: "password123".to_string(),
    };
    
    let user = auth_service.register_user(registration_request).await.unwrap();
    
    // Test creating an artist and adding to DNP list
    let artist_id = dnp_service.create_or_find_artist("Test Artist", None).await.unwrap();
    
    let dnp_entry = dnp_service.add_artist_to_dnp_list(
        user.id,
        artist_id,
        Some(vec!["test".to_string()]),
        Some("Test note".to_string())
    ).await.unwrap();
    
    assert_eq!(dnp_entry.artist_id, artist_id);
    
    // Test getting DNP list
    let dnp_list = dnp_service.get_user_dnp_list(user.id).await.unwrap();
    assert_eq!(dnp_list.total, 1);
    assert_eq!(dnp_list.entries.len(), 1);
    assert_eq!(dnp_list.entries[0].artist_id, artist_id);
    
    println!("✅ DNP service integration test passed");
}

#[tokio::test]
async fn test_auth_with_2fa_integration_fixed() {
    let config = DatabaseConfig::default();
    let pool = initialize_database(config).await.expect("Failed to initialize database");
    let auth_service = AuthService::new(pool);
    
    // Create a test user
    let unique_email = format!("2fa_test_{}@example.com", Uuid::new_v4());
    let registration_request = CreateUserRequest {
        email: unique_email.clone(),
        password: "password123".to_string(),
    };
    
    let user = auth_service.register_user(registration_request).await.unwrap();
    
    // Test TOTP setup
    let totp_setup = auth_service.setup_totp(user.id).await.unwrap();
    assert!(!totp_setup.secret.is_empty());
    assert!(totp_setup.qr_code_url.contains("otpauth://totp/"));
    assert_eq!(totp_setup.backup_codes.len(), 8);
    
    // Generate a valid TOTP code for testing
    let totp_code = generate_test_totp_code(&totp_setup.secret);
    
    // Test TOTP enable
    let enable_result = auth_service.enable_totp(user.id, &totp_code).await;
    assert!(enable_result.is_ok());
    
    // Test TOTP status
    let status = auth_service.get_totp_status(user.id).await.unwrap();
    assert!(status);
    
    println!("✅ Auth with 2FA integration test passed");
}

#[tokio::test]
async fn test_complete_user_workflow_simplified() {
    let config = DatabaseConfig::default();
    let pool = initialize_database(config).await.expect("Failed to initialize database");
    let auth_service = AuthService::new(pool.clone());
    let dnp_service = DnpListService::new(pool.clone());
    
    // Step 1: User Registration
    let unique_email = format!("workflow_test_{}@example.com", Uuid::new_v4());
    let registration_request = CreateUserRequest {
        email: unique_email.clone(),
        password: "secure_password123".to_string(),
    };
    
    let user = auth_service.register_user(registration_request).await.unwrap();
    println!("✅ User registered: {}", user.email);
    
    // Step 2: User Login
    let login_request = LoginRequest {
        email: unique_email.clone(),
        password: "secure_password123".to_string(),
        totp_code: None,
    };
    
    let _token_pair = auth_service.login_user(login_request).await.unwrap();
    println!("✅ User logged in successfully");
    
    // Step 3: Create DNP List Entry
    let artist_id = dnp_service.create_or_find_artist("Drake", None).await.unwrap();
    let _dnp_entry = dnp_service.add_artist_to_dnp_list(
        user.id,
        artist_id,
        Some(vec!["hip-hop".to_string(), "blocked".to_string()]),
        Some("Added for testing".to_string())
    ).await.unwrap();
    println!("✅ Artist added to DNP list: {}", artist_id);
    
    // Step 4: Verify DNP List
    let dnp_list = dnp_service.get_user_dnp_list(user.id).await.unwrap();
    assert_eq!(dnp_list.total, 1);
    assert_eq!(dnp_list.entries[0].artist_id, artist_id);
    println!("✅ DNP list verified: {} entries", dnp_list.total);
    
    // Step 5: Export DNP List
    let exported_json = dnp_service.export_dnp_list(user.id, ImportFormat::Json).await.unwrap();
    assert!(!exported_json.is_empty());
    println!("✅ DNP list exported successfully");
    
    // Step 6: Update DNP Entry
    let update_request = UpdateDnpEntryRequest {
        tags: Some(vec!["updated".to_string()]),
        note: Some("Updated note".to_string()),
    };
    
    let updated_entry = dnp_service.update_dnp_entry(user.id, artist_id, update_request).await.unwrap();
    assert!(updated_entry.tags.contains(&"updated".to_string()));
    println!("✅ DNP entry updated successfully");
    
    // Step 7: Remove from DNP List
    dnp_service.remove_artist_from_dnp_list(user.id, artist_id).await.unwrap();
    let empty_list = dnp_service.get_user_dnp_list(user.id).await.unwrap();
    assert_eq!(empty_list.total, 0);
    println!("✅ Artist removed from DNP list");
    
    println!("✅ Complete user workflow test passed");
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