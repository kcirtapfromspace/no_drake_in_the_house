use music_streaming_blocklist_backend::services::user::{
    AccountDeletionRequest, UpdateUserProfileRequest, UserProfile,
};
use music_streaming_blocklist_backend::{
    initialize_database, DatabaseConfig, UserService, UserSettings,
};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn test_user_service_basic_functionality() {
    // Initialize test database
    let db_config = DatabaseConfig::default();
    let db_pool = initialize_database(db_config)
        .await
        .expect("Failed to initialize database");

    let user_service = UserService::new(db_pool.clone());

    // Create a test user directly in the database
    let user_id = Uuid::new_v4();
    let email = format!("test-{}@example.com", user_id);
    let settings = json!({
        "two_factor_enabled": false,
        "email_notifications": true,
        "privacy_mode": false
    });

    sqlx::query!(
        "INSERT INTO users (id, email, password_hash, settings) VALUES ($1, $2, $3, $4)",
        user_id,
        &email,
        "hashed_password",
        settings
    )
    .execute(&db_pool)
    .await
    .expect("Failed to create test user");

    // Test getting user profile
    let profile = user_service
        .get_profile(user_id)
        .await
        .expect("Failed to get profile");
    assert_eq!(profile.email, email);
    assert_eq!(profile.id, user_id);
    assert!(!profile.totp_enabled);

    // Test updating user profile
    let new_settings = UserSettings {
        two_factor_enabled: true,
        email_notifications: false,
        privacy_mode: true,
    };

    let update_request = UpdateUserProfileRequest {
        email: Some("newemail@example.com".to_string()),
        settings: Some(new_settings.clone()),
    };

    let updated_profile = user_service
        .update_profile(user_id, update_request)
        .await
        .expect("Failed to update profile");
    assert_eq!(updated_profile.email, "newemail@example.com");
    assert_eq!(updated_profile.settings.privacy_mode, true);

    // Test data export
    let export = user_service
        .export_user_data(user_id)
        .await
        .expect("Failed to export data");
    assert_eq!(export.profile.email, "newemail@example.com");

    // Test account deletion
    let deletion_request = AccountDeletionRequest {
        confirmation_email: "newemail@example.com".to_string(),
        reason: Some("Testing".to_string()),
    };

    let deletion_result = user_service
        .delete_account(user_id, deletion_request)
        .await
        .expect("Failed to delete account");
    assert!(deletion_result.cleanup_summary.contains_key("dnp_entries"));

    // Verify user is deleted
    let profile_result = user_service.get_profile(user_id).await;
    assert!(profile_result.is_err());

    println!("✅ All user service tests passed!");
}

#[tokio::test]
async fn test_user_profile_not_found() {
    let db_config = DatabaseConfig::default();
    let db_pool = initialize_database(db_config)
        .await
        .expect("Failed to initialize database");

    let user_service = UserService::new(db_pool);
    let non_existent_user_id = Uuid::new_v4();

    let result = user_service.get_profile(non_existent_user_id).await;
    assert!(result.is_err());

    println!("✅ User not found test passed!");
}

#[tokio::test]
async fn test_user_settings_update() {
    let db_config = DatabaseConfig::default();
    let db_pool = initialize_database(db_config)
        .await
        .expect("Failed to initialize database");

    let user_service = UserService::new(db_pool.clone());

    // Create a test user
    let user_id = Uuid::new_v4();
    let email = format!("settings-test-{}@example.com", user_id);
    sqlx::query!(
        "INSERT INTO users (id, email, password_hash, settings) VALUES ($1, $2, $3, $4)",
        user_id,
        &email,
        "hashed_password",
        json!({})
    )
    .execute(&db_pool)
    .await
    .expect("Failed to create test user");

    // Test settings-only update
    let new_settings = UserSettings {
        two_factor_enabled: true,
        email_notifications: false,
        privacy_mode: true,
    };

    let update_request = UpdateUserProfileRequest {
        email: None,
        settings: Some(new_settings.clone()),
    };

    let updated_profile = user_service
        .update_profile(user_id, update_request)
        .await
        .expect("Failed to update settings");
    assert_eq!(updated_profile.settings.two_factor_enabled, true);
    assert_eq!(updated_profile.settings.email_notifications, false);
    assert_eq!(updated_profile.settings.privacy_mode, true);

    println!("✅ Settings update test passed!");
}
