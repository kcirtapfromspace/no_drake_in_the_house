//! Apple Music Enforcement Database Integration Tests
//!
//! These tests verify the database operations for Apple Music enforcement functionality.
//! Tests require a running PostgreSQL database.

use chrono::Utc;
use music_streaming_blocklist_backend::{
    initialize_database,
    models::{
        AppleMusicRatingEnforcementOptions, BlockedAlbumInfo, BlockedContentScan, BlockedSongInfo,
        CreateUserRequest, EnforcementProgress, EnforcementRunStatus, RatingError,
    },
    services::AuthService,
    DatabaseConfig,
};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

/// Helper to initialize test database
async fn setup_test_db() -> PgPool {
    let config = DatabaseConfig::default();
    initialize_database(config)
        .await
        .expect("Failed to initialize database")
}

/// Helper to create a test user
async fn create_test_user(pool: &PgPool) -> Uuid {
    let auth_service = AuthService::new(pool.clone());
    let unique_email = format!("enforcement_test_{}@example.com", Uuid::new_v4());
    let registration_request = CreateUserRequest {
        email: unique_email,
        password: "test_password123".to_string(),
    };

    let user = auth_service
        .register_user(registration_request)
        .await
        .expect("Failed to create test user");

    user.id
}

// ============================================
// Enforcement Run Tests
// ============================================

#[tokio::test]
async fn test_create_enforcement_run() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let connection_id = Uuid::new_v4();

    let options = AppleMusicRatingEnforcementOptions::default();
    let options_json = serde_json::to_value(&options).unwrap();

    // Create enforcement run record
    let result = sqlx::query!(
        r#"
        INSERT INTO apple_music_enforcement_runs (
            user_id, connection_id, status, options, started_at
        )
        VALUES ($1, $2, 'running', $3, NOW())
        RETURNING id
        "#,
        user_id,
        connection_id,
        options_json
    )
    .fetch_one(&pool)
    .await;

    assert!(result.is_ok());
    let row = result.unwrap();
    assert!(!row.id.is_nil());

    println!("✅ Create enforcement run test passed");
}

#[tokio::test]
async fn test_update_enforcement_run_status() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let connection_id = Uuid::new_v4();

    let options = AppleMusicRatingEnforcementOptions::default();
    let options_json = serde_json::to_value(&options).unwrap();

    // Create run
    let row = sqlx::query!(
        r#"
        INSERT INTO apple_music_enforcement_runs (
            user_id, connection_id, status, options, started_at
        )
        VALUES ($1, $2, 'running', $3, NOW())
        RETURNING id
        "#,
        user_id,
        connection_id,
        options_json
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to create run");

    // Update to completed
    let update_result = sqlx::query!(
        r#"
        UPDATE apple_music_enforcement_runs
        SET status = 'completed',
            completed_at = NOW(),
            songs_scanned = 100,
            albums_scanned = 20,
            songs_disliked = 10,
            albums_disliked = 5,
            errors = 0
        WHERE id = $1
        "#,
        row.id
    )
    .execute(&pool)
    .await;

    assert!(update_result.is_ok());
    assert_eq!(update_result.unwrap().rows_affected(), 1);

    // Verify update
    let verified = sqlx::query!(
        r#"
        SELECT status, songs_disliked, albums_disliked
        FROM apple_music_enforcement_runs
        WHERE id = $1
        "#,
        row.id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch run");

    assert_eq!(verified.status, "completed");
    assert_eq!(verified.songs_disliked, 10);
    assert_eq!(verified.albums_disliked, 5);

    println!("✅ Update enforcement run status test passed");
}

#[tokio::test]
async fn test_record_enforcement_action() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let connection_id = Uuid::new_v4();

    let options = AppleMusicRatingEnforcementOptions::default();
    let options_json = serde_json::to_value(&options).unwrap();

    // Create run first
    let run = sqlx::query!(
        r#"
        INSERT INTO apple_music_enforcement_runs (
            user_id, connection_id, status, options, started_at
        )
        VALUES ($1, $2, 'running', $3, NOW())
        RETURNING id
        "#,
        user_id,
        connection_id,
        options_json
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to create run");

    // Record an enforcement action
    let action_result = sqlx::query!(
        r#"
        INSERT INTO apple_music_enforcement_actions (
            run_id, user_id, resource_type, resource_id, resource_name, artist_name, action
        )
        VALUES ($1, $2, 'library_song', 'song-123', 'Test Song', 'Test Artist', 'dislike')
        RETURNING id
        "#,
        run.id,
        user_id
    )
    .fetch_one(&pool)
    .await;

    assert!(action_result.is_ok());
    let action = action_result.unwrap();
    assert!(!action.id.is_nil());

    println!("✅ Record enforcement action test passed");
}

#[tokio::test]
async fn test_record_multiple_enforcement_actions() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let connection_id = Uuid::new_v4();

    let options = AppleMusicRatingEnforcementOptions::default();
    let options_json = serde_json::to_value(&options).unwrap();

    // Create run
    let run = sqlx::query!(
        r#"
        INSERT INTO apple_music_enforcement_runs (
            user_id, connection_id, status, options, started_at
        )
        VALUES ($1, $2, 'running', $3, NOW())
        RETURNING id
        "#,
        user_id,
        connection_id,
        options_json
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to create run");

    // Record multiple actions
    for i in 0..5 {
        sqlx::query!(
            r#"
            INSERT INTO apple_music_enforcement_actions (
                run_id, user_id, resource_type, resource_id, resource_name, artist_name, action
            )
            VALUES ($1, $2, 'library_song', $3, $4, 'Test Artist', 'dislike')
            "#,
            run.id,
            user_id,
            format!("song-{}", i),
            format!("Test Song {}", i)
        )
        .execute(&pool)
        .await
        .expect("Failed to insert action");
    }

    // Count actions
    let count = sqlx::query!(
        r#"
        SELECT COUNT(*) as count FROM apple_music_enforcement_actions WHERE run_id = $1
        "#,
        run.id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count actions");

    assert_eq!(count.count.unwrap(), 5);

    println!("✅ Record multiple enforcement actions test passed");
}

// ============================================
// Enforcement History Tests
// ============================================

#[tokio::test]
async fn test_get_enforcement_history() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let connection_id = Uuid::new_v4();

    let options = AppleMusicRatingEnforcementOptions::default();
    let options_json = serde_json::to_value(&options).unwrap();

    // Create multiple runs
    for _ in 0..3 {
        sqlx::query!(
            r#"
            INSERT INTO apple_music_enforcement_runs (
                user_id, connection_id, status, options, started_at, completed_at,
                songs_scanned, albums_scanned, songs_disliked, albums_disliked
            )
            VALUES ($1, $2, 'completed', $3, NOW() - INTERVAL '1 hour', NOW(),
                    100, 20, 10, 5)
            "#,
            user_id,
            connection_id,
            options_json
        )
        .execute(&pool)
        .await
        .expect("Failed to create run");
    }

    // Get history
    let history = sqlx::query!(
        r#"
        SELECT id, status, songs_disliked, albums_disliked
        FROM apple_music_enforcement_runs
        WHERE user_id = $1
        ORDER BY started_at DESC
        LIMIT 50
        "#,
        user_id
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch history");

    assert!(history.len() >= 3);
    assert!(history.iter().all(|r| r.status == "completed"));

    println!("✅ Get enforcement history test passed");
}

#[tokio::test]
async fn test_enforcement_history_ordered_by_date() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let connection_id = Uuid::new_v4();

    let options = AppleMusicRatingEnforcementOptions::default();
    let options_json = serde_json::to_value(&options).unwrap();

    // Create runs at different times
    for i in 0..3 {
        sqlx::query!(
            r#"
            INSERT INTO apple_music_enforcement_runs (
                user_id, connection_id, status, options,
                started_at, songs_disliked
            )
            VALUES ($1, $2, 'completed', $3,
                    NOW() - INTERVAL '1 hour' * $4, $5)
            "#,
            user_id,
            connection_id,
            options_json,
            i as i32,
            i as i32
        )
        .execute(&pool)
        .await
        .expect("Failed to create run");
    }

    // Get history (should be ordered newest first)
    let history = sqlx::query!(
        r#"
        SELECT songs_disliked
        FROM apple_music_enforcement_runs
        WHERE user_id = $1
        ORDER BY started_at DESC
        "#,
        user_id
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch history");

    // Most recent run (songs_disliked = 0) should be first
    assert_eq!(history[0].songs_disliked, 0);

    println!("✅ Enforcement history ordering test passed");
}

// ============================================
// Rollback Tests
// ============================================

#[tokio::test]
async fn test_mark_run_as_rolled_back() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let connection_id = Uuid::new_v4();

    let options = AppleMusicRatingEnforcementOptions::default();
    let options_json = serde_json::to_value(&options).unwrap();

    // Create completed run
    let run = sqlx::query!(
        r#"
        INSERT INTO apple_music_enforcement_runs (
            user_id, connection_id, status, options, started_at, completed_at
        )
        VALUES ($1, $2, 'completed', $3, NOW() - INTERVAL '1 hour', NOW())
        RETURNING id
        "#,
        user_id,
        connection_id,
        options_json
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to create run");

    // Mark as rolled back
    sqlx::query!(
        r#"
        UPDATE apple_music_enforcement_runs
        SET status = 'rolled_back'
        WHERE id = $1 AND user_id = $2
        "#,
        run.id,
        user_id
    )
    .execute(&pool)
    .await
    .expect("Failed to update status");

    // Verify
    let verified = sqlx::query!(
        r#"SELECT status FROM apple_music_enforcement_runs WHERE id = $1"#,
        run.id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch run");

    assert_eq!(verified.status, "rolled_back");

    println!("✅ Mark run as rolled back test passed");
}

#[tokio::test]
async fn test_get_actions_for_rollback() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let connection_id = Uuid::new_v4();

    let options = AppleMusicRatingEnforcementOptions::default();
    let options_json = serde_json::to_value(&options).unwrap();

    // Create run with actions
    let run = sqlx::query!(
        r#"
        INSERT INTO apple_music_enforcement_runs (
            user_id, connection_id, status, options, started_at
        )
        VALUES ($1, $2, 'completed', $3, NOW())
        RETURNING id
        "#,
        user_id,
        connection_id,
        options_json
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to create run");

    // Add actions
    for i in 0..3 {
        sqlx::query!(
            r#"
            INSERT INTO apple_music_enforcement_actions (
                run_id, user_id, resource_type, resource_id, action
            )
            VALUES ($1, $2, 'library_song', $3, 'dislike')
            "#,
            run.id,
            user_id,
            format!("song-{}", i)
        )
        .execute(&pool)
        .await
        .expect("Failed to insert action");
    }

    // Get actions for rollback
    let actions = sqlx::query!(
        r#"
        SELECT resource_type, resource_id, action
        FROM apple_music_enforcement_actions
        WHERE run_id = $1 AND user_id = $2
        "#,
        run.id,
        user_id
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch actions");

    assert_eq!(actions.len(), 3);
    assert!(actions.iter().all(|a| a.action == "dislike"));

    println!("✅ Get actions for rollback test passed");
}

// ============================================
// User Authorization Tests
// ============================================

#[tokio::test]
async fn test_user_can_only_see_own_runs() {
    let pool = setup_test_db().await;
    let user1_id = create_test_user(&pool).await;
    let user2_id = create_test_user(&pool).await;
    let connection_id = Uuid::new_v4();

    let options = AppleMusicRatingEnforcementOptions::default();
    let options_json = serde_json::to_value(&options).unwrap();

    // Create run for user1
    sqlx::query!(
        r#"
        INSERT INTO apple_music_enforcement_runs (
            user_id, connection_id, status, options, started_at
        )
        VALUES ($1, $2, 'completed', $3, NOW())
        "#,
        user1_id,
        connection_id,
        options_json
    )
    .execute(&pool)
    .await
    .expect("Failed to create run for user1");

    // Create run for user2
    sqlx::query!(
        r#"
        INSERT INTO apple_music_enforcement_runs (
            user_id, connection_id, status, options, started_at
        )
        VALUES ($1, $2, 'completed', $3, NOW())
        "#,
        user2_id,
        connection_id,
        options_json
    )
    .execute(&pool)
    .await
    .expect("Failed to create run for user2");

    // User1 should only see their run
    let user1_runs = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM apple_music_enforcement_runs
        WHERE user_id = $1
        "#,
        user1_id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count user1 runs");

    // User2 should only see their run
    let user2_runs = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM apple_music_enforcement_runs
        WHERE user_id = $1
        "#,
        user2_id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count user2 runs");

    assert_eq!(user1_runs.count.unwrap(), 1);
    assert_eq!(user2_runs.count.unwrap(), 1);

    println!("✅ User authorization test passed");
}

// ============================================
// Error Handling Tests
// ============================================

#[tokio::test]
async fn test_store_error_details() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let connection_id = Uuid::new_v4();

    let options = AppleMusicRatingEnforcementOptions::default();
    let options_json = serde_json::to_value(&options).unwrap();

    let error_details = json!([
        {"resource_id": "song-1", "error": "Rate limited"},
        {"resource_id": "song-2", "error": "Not found"}
    ]);

    // Create run with errors
    let run = sqlx::query!(
        r#"
        INSERT INTO apple_music_enforcement_runs (
            user_id, connection_id, status, options, started_at, errors, error_details
        )
        VALUES ($1, $2, 'completed', $3, NOW(), 2, $4)
        RETURNING id
        "#,
        user_id,
        connection_id,
        options_json,
        error_details
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to create run");

    // Verify error details
    let verified = sqlx::query!(
        r#"
        SELECT errors, error_details
        FROM apple_music_enforcement_runs
        WHERE id = $1
        "#,
        run.id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch run");

    assert_eq!(verified.errors, 2);
    assert!(verified.error_details.is_some());

    let details: serde_json::Value = verified.error_details.unwrap();
    assert!(details.as_array().unwrap().len() == 2);

    println!("✅ Store error details test passed");
}

// ============================================
// Statistics Tests
// ============================================

#[tokio::test]
async fn test_aggregate_enforcement_statistics() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let connection_id = Uuid::new_v4();

    let options = AppleMusicRatingEnforcementOptions::default();
    let options_json = serde_json::to_value(&options).unwrap();

    // Create multiple completed runs
    for i in 0..5 {
        sqlx::query!(
            r#"
            INSERT INTO apple_music_enforcement_runs (
                user_id, connection_id, status, options, started_at,
                songs_disliked, albums_disliked
            )
            VALUES ($1, $2, 'completed', $3, NOW(), $4, $5)
            "#,
            user_id,
            connection_id,
            options_json,
            (i + 1) * 10,  // 10, 20, 30, 40, 50
            (i + 1) * 2    // 2, 4, 6, 8, 10
        )
        .execute(&pool)
        .await
        .expect("Failed to create run");
    }

    // Get aggregate statistics
    let stats = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as total_runs,
            SUM(songs_disliked) as total_songs_disliked,
            SUM(albums_disliked) as total_albums_disliked
        FROM apple_music_enforcement_runs
        WHERE user_id = $1 AND status = 'completed'
        "#,
        user_id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to get stats");

    assert_eq!(stats.total_runs.unwrap(), 5);
    assert_eq!(stats.total_songs_disliked.unwrap(), 150); // 10+20+30+40+50
    assert_eq!(stats.total_albums_disliked.unwrap(), 30); // 2+4+6+8+10

    println!("✅ Aggregate statistics test passed");
}

// ============================================
// Model Tests
// ============================================

#[test]
fn test_enforcement_options_serialization() {
    let options = AppleMusicRatingEnforcementOptions {
        dislike_songs: true,
        dislike_albums: false,
        include_library: true,
        include_catalog: false,
        batch_size: 100,
        dry_run: true,
    };

    let json = serde_json::to_value(&options).expect("Failed to serialize");
    assert_eq!(json["dislike_songs"], true);
    assert_eq!(json["dislike_albums"], false);
    assert_eq!(json["batch_size"], 100);
    assert_eq!(json["dry_run"], true);

    let deserialized: AppleMusicRatingEnforcementOptions =
        serde_json::from_value(json).expect("Failed to deserialize");
    assert_eq!(deserialized.dislike_songs, options.dislike_songs);
    assert_eq!(deserialized.batch_size, options.batch_size);
}

#[test]
fn test_blocked_song_info_serialization() {
    let song = BlockedSongInfo {
        library_song_id: "lib-123".to_string(),
        catalog_song_id: Some("cat-456".to_string()),
        name: "Test Song".to_string(),
        artist_name: "Test Artist".to_string(),
        album_name: "Test Album".to_string(),
        blocked_artist_id: Some(Uuid::new_v4()),
    };

    let json = serde_json::to_value(&song).expect("Failed to serialize");
    assert_eq!(json["library_song_id"], "lib-123");
    assert_eq!(json["name"], "Test Song");

    let deserialized: BlockedSongInfo =
        serde_json::from_value(json).expect("Failed to deserialize");
    assert_eq!(deserialized.library_song_id, song.library_song_id);
}

#[test]
fn test_blocked_content_scan_operations() {
    let mut scan = BlockedContentScan::new();
    assert_eq!(scan.total_blocked(), 0);

    scan.blocked_songs.push(BlockedSongInfo {
        library_song_id: "song-1".to_string(),
        catalog_song_id: None,
        name: "Song 1".to_string(),
        artist_name: "Artist".to_string(),
        album_name: "Album".to_string(),
        blocked_artist_id: None,
    });

    scan.blocked_albums.push(BlockedAlbumInfo {
        library_album_id: "album-1".to_string(),
        catalog_album_id: None,
        name: "Album 1".to_string(),
        artist_name: "Artist".to_string(),
        blocked_artist_id: None,
    });

    assert_eq!(scan.total_blocked(), 2);
}

#[test]
fn test_enforcement_progress_calculation() {
    let mut progress = EnforcementProgress::new();
    assert_eq!(progress.percent_complete(), 0.0);

    progress.total_items = 100;
    progress.processed_items = 50;
    assert_eq!(progress.percent_complete(), 50.0);

    progress.processed_items = 100;
    assert_eq!(progress.percent_complete(), 100.0);
}

#[test]
fn test_enforcement_run_status_display() {
    assert_eq!(EnforcementRunStatus::Pending.to_string(), "pending");
    assert_eq!(EnforcementRunStatus::Running.to_string(), "running");
    assert_eq!(EnforcementRunStatus::Completed.to_string(), "completed");
    assert_eq!(EnforcementRunStatus::Failed.to_string(), "failed");
    assert_eq!(EnforcementRunStatus::RolledBack.to_string(), "rolled_back");
}

#[test]
fn test_rating_error_serialization() {
    let error = RatingError {
        resource_id: "song-123".to_string(),
        resource_type: "library_song".to_string(),
        error_message: "Rate limit exceeded".to_string(),
    };

    let json = serde_json::to_value(&error).expect("Failed to serialize");
    assert_eq!(json["resource_id"], "song-123");
    assert_eq!(json["error_message"], "Rate limit exceeded");
}
