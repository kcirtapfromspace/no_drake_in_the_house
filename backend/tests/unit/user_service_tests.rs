use crate::common::*;
use music_streaming_blocklist_backend::{models::*, services::UserService};
use rstest::*;
use serde_json::json;
use serial_test::serial;

#[fixture]
async fn test_db() -> TestDatabase {
    TestDatabase::new().await
}

#[fixture]
async fn user_service(#[future] test_db: TestDatabase) -> (UserService, TestDatabase) {
    let db = test_db.await;
    let service = UserService::new(db.pool.clone());
    (service, db)
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_get_user_profile(#[future] user_service: (UserService, TestDatabase)) {
    let (service, db) = user_service.await;

    let user = db.create_test_user().await;

    let profile = service.get_profile(user.id).await.unwrap();

    assert_eq!(profile.id, user.id);
    assert_eq!(profile.email, user.email);
    TestAssertions::assert_email_format(&profile.email);
    assert!(!profile.email_verified);
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_get_nonexistent_user_profile(#[future] user_service: (UserService, TestDatabase)) {
    let (service, _db) = user_service.await;

    let nonexistent_id = uuid::Uuid::new_v4();
    let result = service.get_profile(nonexistent_id).await;

    assert!(result.is_err());
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_update_user_settings(#[future] user_service: (UserService, TestDatabase)) {
    let (service, db) = user_service.await;

    let user = db.create_test_user().await;

    let new_settings = UserSettings {
        theme: Some("dark".to_string()),
        notifications_enabled: Some(true),
        auto_enforcement: Some(false),
        preferred_platforms: Some(vec!["spotify".to_string(), "apple_music".to_string()]),
        privacy_settings: Some(json!({
            "profile_visibility": "private",
            "share_dnp_stats": false
        })),
    };

    let updated_profile = service
        .update_settings(user.id, new_settings.clone())
        .await
        .unwrap();

    assert_eq!(updated_profile.id, user.id);

    // Verify settings were updated
    let profile = service.get_profile(user.id).await.unwrap();
    if let Some(settings) = profile.settings {
        assert_eq!(settings.theme, new_settings.theme);
        assert_eq!(
            settings.notifications_enabled,
            new_settings.notifications_enabled
        );
        assert_eq!(settings.auto_enforcement, new_settings.auto_enforcement);
        assert_eq!(
            settings.preferred_platforms,
            new_settings.preferred_platforms
        );
    } else {
        panic!("Settings should be present after update");
    }
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_update_partial_settings(#[future] user_service: (UserService, TestDatabase)) {
    let (service, db) = user_service.await;

    let user = db.create_test_user().await;

    // First, set some initial settings
    let initial_settings = UserSettings {
        theme: Some("light".to_string()),
        notifications_enabled: Some(true),
        auto_enforcement: Some(true),
        preferred_platforms: Some(vec!["spotify".to_string()]),
        privacy_settings: None,
    };

    service
        .update_settings(user.id, initial_settings)
        .await
        .unwrap();

    // Then update only some fields
    let partial_settings = UserSettings {
        theme: Some("dark".to_string()),
        notifications_enabled: None, // Don't change this
        auto_enforcement: Some(false),
        preferred_platforms: None, // Don't change this
        privacy_settings: Some(json!({"new_setting": "value"})),
    };

    service
        .update_settings(user.id, partial_settings)
        .await
        .unwrap();

    // Verify the merge worked correctly
    let profile = service.get_profile(user.id).await.unwrap();
    let settings = profile.settings.unwrap();

    assert_eq!(settings.theme, Some("dark".to_string())); // Updated
    assert_eq!(settings.notifications_enabled, Some(true)); // Preserved
    assert_eq!(settings.auto_enforcement, Some(false)); // Updated
    assert_eq!(
        settings.preferred_platforms,
        Some(vec!["spotify".to_string()])
    ); // Preserved
    assert!(settings.privacy_settings.is_some()); // Updated
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_export_user_data(#[future] user_service: (UserService, TestDatabase)) {
    let (service, db) = user_service.await;

    let user = db.create_test_user().await;

    // Add some data to export
    let settings = UserSettings {
        theme: Some("dark".to_string()),
        notifications_enabled: Some(true),
        auto_enforcement: Some(false),
        preferred_platforms: Some(vec!["spotify".to_string()]),
        privacy_settings: None,
    };

    service.update_settings(user.id, settings).await.unwrap();

    // Export user data
    let export_data = service.export_user_data(user.id).await.unwrap();

    assert_eq!(export_data.user_id, user.id);
    assert_eq!(export_data.email, user.email);
    assert!(export_data.created_at.is_some());
    assert!(export_data.settings.is_some());

    // Verify sensitive data is not included
    assert!(export_data.password_hash.is_none());
    assert!(export_data.totp_secret.is_none());
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_delete_user_account(#[future] user_service: (UserService, TestDatabase)) {
    let (service, db) = user_service.await;

    let user = db.create_test_user().await;

    // Add some user data
    let settings = UserSettings {
        theme: Some("dark".to_string()),
        notifications_enabled: Some(true),
        auto_enforcement: Some(false),
        preferred_platforms: None,
        privacy_settings: None,
    };

    service.update_settings(user.id, settings).await.unwrap();

    // Delete the account
    let deleted = service.delete_account(user.id).await.unwrap();
    assert!(deleted);

    // Verify user no longer exists
    let result = service.get_profile(user.id).await;
    assert!(result.is_err());
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_delete_nonexistent_account(#[future] user_service: (UserService, TestDatabase)) {
    let (service, _db) = user_service.await;

    let nonexistent_id = uuid::Uuid::new_v4();
    let deleted = service.delete_account(nonexistent_id).await.unwrap();
    assert!(!deleted);
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_update_email_verification_status(
    #[future] user_service: (UserService, TestDatabase),
) {
    let (service, db) = user_service.await;

    let user = db.create_test_user().await;

    // Initially should not be verified
    let profile = service.get_profile(user.id).await.unwrap();
    assert!(!profile.email_verified);

    // Mark as verified
    let updated = service
        .update_email_verification(user.id, true)
        .await
        .unwrap();
    assert!(updated);

    // Verify the status changed
    let profile = service.get_profile(user.id).await.unwrap();
    assert!(profile.email_verified);

    // Mark as unverified
    let updated = service
        .update_email_verification(user.id, false)
        .await
        .unwrap();
    assert!(updated);

    // Verify the status changed back
    let profile = service.get_profile(user.id).await.unwrap();
    assert!(!profile.email_verified);
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_get_user_statistics(#[future] user_service: (UserService, TestDatabase)) {
    let (service, db) = user_service.await;

    let user = db.create_test_user().await;

    let stats = service.get_user_statistics(user.id).await.unwrap();

    assert_eq!(stats.user_id, user.id);
    assert_eq!(stats.total_dnp_entries, 0); // New user should have no entries
    assert_eq!(stats.total_enforcement_actions, 0);
    assert!(stats.account_created_at.is_some());
    assert!(stats.last_login_at.is_none()); // No login yet
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Duration;

    #[rstest]
    #[tokio::test]
    #[serial]
    async fn test_get_profile_performance(#[future] user_service: (UserService, TestDatabase)) {
        let (service, db) = user_service.await;

        let user = db.create_test_user().await;

        let (result, duration) =
            PerformanceTestHelper::measure_async(|| service.get_profile(user.id)).await;

        assert!(result.is_ok());
        PerformanceTestHelper::assert_performance_threshold(duration, 50); // 50ms max
    }

    #[rstest]
    #[tokio::test]
    #[serial]
    async fn test_update_settings_performance(#[future] user_service: (UserService, TestDatabase)) {
        let (service, db) = user_service.await;

        let user = db.create_test_user().await;

        let settings = UserSettings {
            theme: Some("dark".to_string()),
            notifications_enabled: Some(true),
            auto_enforcement: Some(false),
            preferred_platforms: Some(vec!["spotify".to_string()]),
            privacy_settings: Some(json!({"test": "value"})),
        };

        let (result, duration) = PerformanceTestHelper::measure_async(|| {
            service.update_settings(user.id, settings.clone())
        })
        .await;

        assert!(result.is_ok());
        PerformanceTestHelper::assert_performance_threshold(duration, 100); // 100ms max
    }
}
