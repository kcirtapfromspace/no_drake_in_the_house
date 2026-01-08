use crate::common::*;
use music_streaming_blocklist_backend::{
    initialize_database, models::*, services::*, DatabaseConfig,
};
use rstest::*;
use serial_test::serial;
use sqlx::Row;
use std::time::Duration;

#[fixture]
async fn test_db() -> TestDatabase {
    TestDatabase::new().await
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_database_connection_and_migrations(#[future] test_db: TestDatabase) {
    let db = test_db.await;

    // Test basic connectivity
    let result: i32 = sqlx::query_scalar("SELECT 1")
        .fetch_one(&db.pool)
        .await
        .expect("Failed to execute basic query");

    assert_eq!(result, 1);

    // Verify all required tables exist
    let tables = sqlx::query(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'",
    )
    .fetch_all(&db.pool)
    .await
    .expect("Failed to fetch table list");

    let table_names: Vec<String> = tables
        .iter()
        .map(|row| row.get::<String, _>("table_name"))
        .collect();

    // Check for core tables
    assert!(table_names.contains(&"users".to_string()));
    assert!(table_names.contains(&"artists".to_string()));
    assert!(table_names.contains(&"user_artist_blocks".to_string()));
    assert!(table_names.contains(&"audit_log".to_string()));
    assert!(table_names.contains(&"user_sessions".to_string()));
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_user_crud_operations(#[future] test_db: TestDatabase) {
    let db = test_db.await;

    // Create user
    let user_id = uuid::Uuid::new_v4();
    let email = format!("crud_test_{}@example.com", uuid::Uuid::new_v4());
    let password_hash = "$2b$12$test_hash";

    sqlx::query!(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)",
        user_id,
        email,
        password_hash
    )
    .execute(&db.pool)
    .await
    .expect("Failed to insert user");

    // Read user
    let user = sqlx::query_as!(
        User,
        "SELECT id, email, password_hash, totp_secret, totp_enabled, email_verified, created_at, updated_at, settings FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(&db.pool)
    .await
    .expect("Failed to fetch user");

    assert_eq!(user.id, user_id);
    assert_eq!(user.email, email);
    assert_eq!(user.password_hash, Some(password_hash.to_string()));

    // Update user
    let new_email = format!("updated_{}@example.com", uuid::Uuid::new_v4());
    sqlx::query!(
        "UPDATE users SET email = $1, email_verified = true WHERE id = $2",
        new_email,
        user_id
    )
    .execute(&db.pool)
    .await
    .expect("Failed to update user");

    // Verify update
    let updated_user = sqlx::query_as!(
        User,
        "SELECT id, email, password_hash, totp_secret, totp_enabled, email_verified, created_at, updated_at, settings FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(&db.pool)
    .await
    .expect("Failed to fetch updated user");

    assert_eq!(updated_user.email, new_email);
    assert!(updated_user.email_verified);

    // Delete user
    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&db.pool)
        .await
        .expect("Failed to delete user");

    // Verify deletion
    let deleted_user = sqlx::query_as!(
        User,
        "SELECT id, email, password_hash, totp_secret, totp_enabled, email_verified, created_at, updated_at, settings FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(&db.pool)
    .await
    .expect("Failed to check deleted user");

    assert!(deleted_user.is_none());
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_artist_crud_operations(#[future] test_db: TestDatabase) {
    let db = test_db.await;

    // Create artist
    let artist_id = uuid::Uuid::new_v4();
    let canonical_name = "Test Artist";
    let external_ids = serde_json::json!({"spotify": "spotify_123", "apple_music": "apple_456"});
    let metadata =
        serde_json::json!({"genres": ["rock", "pop"], "image": "https://example.com/image.jpg"});

    sqlx::query!(
        "INSERT INTO artists (id, canonical_name, external_ids, metadata) VALUES ($1, $2, $3, $4)",
        artist_id,
        canonical_name,
        external_ids,
        metadata
    )
    .execute(&db.pool)
    .await
    .expect("Failed to insert artist");

    // Read artist
    let artist = sqlx::query_as!(
        Artist,
        "SELECT id, canonical_name, external_ids, metadata, created_at FROM artists WHERE id = $1",
        artist_id
    )
    .fetch_one(&db.pool)
    .await
    .expect("Failed to fetch artist");

    assert_eq!(artist.id, artist_id);
    assert_eq!(artist.canonical_name, canonical_name);
    assert_eq!(artist.external_ids, external_ids);
    assert_eq!(artist.metadata, metadata);

    // Update artist
    let new_name = "Updated Artist Name";
    sqlx::query!(
        "UPDATE artists SET canonical_name = $1 WHERE id = $2",
        new_name,
        artist_id
    )
    .execute(&db.pool)
    .await
    .expect("Failed to update artist");

    // Verify update
    let updated_artist = sqlx::query_as!(
        Artist,
        "SELECT id, canonical_name, external_ids, metadata, created_at FROM artists WHERE id = $1",
        artist_id
    )
    .fetch_one(&db.pool)
    .await
    .expect("Failed to fetch updated artist");

    assert_eq!(updated_artist.canonical_name, new_name);
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_dnp_list_operations(#[future] test_db: TestDatabase) {
    let db = test_db.await;

    let user = db.create_test_user().await;
    let artist = db.create_test_artist(Some("DNP Test Artist")).await;

    // Add to DNP list
    let tags = vec!["test".to_string(), "rock".to_string()];
    let note = "Test note for DNP entry";

    sqlx::query!(
        "INSERT INTO user_artist_blocks (user_id, artist_id, tags, note) VALUES ($1, $2, $3, $4)",
        user.id,
        artist.id,
        &tags,
        note
    )
    .execute(&db.pool)
    .await
    .expect("Failed to insert DNP entry");

    // Fetch DNP list
    let dnp_entries = sqlx::query!(
        r#"
        SELECT 
            b.user_id,
            b.artist_id,
            b.tags,
            b.note,
            b.created_at,
            a.canonical_name,
            a.external_ids,
            a.metadata
        FROM user_artist_blocks b
        JOIN artists a ON b.artist_id = a.id
        WHERE b.user_id = $1
        ORDER BY b.created_at DESC
        "#,
        user.id
    )
    .fetch_all(&db.pool)
    .await
    .expect("Failed to fetch DNP entries");

    assert_eq!(dnp_entries.len(), 1);
    let entry = &dnp_entries[0];
    assert_eq!(entry.user_id, user.id);
    assert_eq!(entry.artist_id, artist.id);
    assert_eq!(entry.tags, Some(tags));
    assert_eq!(entry.note, Some(note.to_string()));
    assert_eq!(entry.canonical_name, artist.canonical_name);

    // Remove from DNP list
    let deleted_rows = sqlx::query!(
        "DELETE FROM user_artist_blocks WHERE user_id = $1 AND artist_id = $2",
        user.id,
        artist.id
    )
    .execute(&db.pool)
    .await
    .expect("Failed to delete DNP entry")
    .rows_affected();

    assert_eq!(deleted_rows, 1);

    // Verify removal
    let remaining_entries = sqlx::query!(
        "SELECT COUNT(*) as count FROM user_artist_blocks WHERE user_id = $1",
        user.id
    )
    .fetch_one(&db.pool)
    .await
    .expect("Failed to count remaining entries");

    assert_eq!(remaining_entries.count, Some(0));
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_audit_logging(#[future] test_db: TestDatabase) {
    let db = test_db.await;

    let user = db.create_test_user().await;

    // Insert audit log entry
    let action = "user_login";
    let resource_type = "user";
    let ip_address = std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1));
    let user_agent = "Test User Agent";

    sqlx::query!(
        r#"
        INSERT INTO audit_log (user_id, action, resource_type, resource_id, ip_address, user_agent)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        user.id,
        action,
        resource_type,
        user.id.to_string(),
        ip_address,
        user_agent
    )
    .execute(&db.pool)
    .await
    .expect("Failed to insert audit log entry");

    // Fetch audit log entries
    let audit_entries = sqlx::query!(
        "SELECT user_id, action, resource_type, resource_id, ip_address, user_agent, created_at FROM audit_log WHERE user_id = $1",
        user.id
    )
    .fetch_all(&db.pool)
    .await
    .expect("Failed to fetch audit log entries");

    assert_eq!(audit_entries.len(), 1);
    let entry = &audit_entries[0];
    assert_eq!(entry.user_id, Some(user.id));
    assert_eq!(entry.action, action);
    assert_eq!(entry.resource_type, resource_type);
    assert_eq!(entry.resource_id, Some(user.id.to_string()));
    assert_eq!(entry.ip_address, Some(ip_address));
    assert_eq!(entry.user_agent, Some(user_agent.to_string()));
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_database_constraints_and_indexes(#[future] test_db: TestDatabase) {
    let db = test_db.await;

    // Test unique constraint on user email
    let email = format!("constraint_test_{}@example.com", uuid::Uuid::new_v4());
    let password_hash = "$2b$12$test_hash";

    // First user should succeed
    sqlx::query!(
        "INSERT INTO users (email, password_hash) VALUES ($1, $2)",
        email,
        password_hash
    )
    .execute(&db.pool)
    .await
    .expect("Failed to insert first user");

    // Second user with same email should fail
    let result = sqlx::query!(
        "INSERT INTO users (email, password_hash) VALUES ($1, $2)",
        email,
        password_hash
    )
    .execute(&db.pool)
    .await;

    assert!(result.is_err(), "Duplicate email should be rejected");

    // Test foreign key constraint
    let nonexistent_user_id = uuid::Uuid::new_v4();
    let artist = db.create_test_artist(Some("FK Test Artist")).await;

    let result = sqlx::query!(
        "INSERT INTO user_artist_blocks (user_id, artist_id) VALUES ($1, $2)",
        nonexistent_user_id,
        artist.id
    )
    .execute(&db.pool)
    .await;

    assert!(result.is_err(), "Foreign key constraint should be enforced");
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_transaction_rollback(#[future] test_db: TestDatabase) {
    let db = test_db.await;

    let user = db.create_test_user().await;
    let artist = db.create_test_artist(Some("Transaction Test Artist")).await;

    // Start a transaction that will fail
    let mut tx = db.pool.begin().await.expect("Failed to start transaction");

    // Insert valid DNP entry
    sqlx::query!(
        "INSERT INTO user_artist_blocks (user_id, artist_id, note) VALUES ($1, $2, $3)",
        user.id,
        artist.id,
        "Valid entry"
    )
    .execute(&mut *tx)
    .await
    .expect("Failed to insert valid entry");

    // Try to insert invalid entry (duplicate)
    let result = sqlx::query!(
        "INSERT INTO user_artist_blocks (user_id, artist_id, note) VALUES ($1, $2, $3)",
        user.id,
        artist.id,
        "Duplicate entry"
    )
    .execute(&mut *tx)
    .await;

    assert!(result.is_err(), "Duplicate entry should fail");

    // Rollback transaction
    tx.rollback().await.expect("Failed to rollback transaction");

    // Verify no entries were inserted
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM user_artist_blocks WHERE user_id = $1",
        user.id
    )
    .fetch_one(&db.pool)
    .await
    .expect("Failed to count entries");

    assert_eq!(count, Some(0), "No entries should exist after rollback");
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[rstest]
    #[tokio::test]
    #[serial]
    async fn test_database_query_performance(#[future] test_db: TestDatabase) {
        let db = test_db.await;

        // Create test data
        let user = db.create_test_user().await;

        // Create multiple artists for performance testing
        for i in 0..100 {
            let artist = db
                .create_test_artist(Some(&format!("Performance Artist {}", i)))
                .await;
            sqlx::query!(
                "INSERT INTO user_artist_blocks (user_id, artist_id, tags, note) VALUES ($1, $2, $3, $4)",
                user.id,
                artist.id,
                &vec![format!("tag{}", i)],
                format!("Note {}", i)
            )
            .execute(&db.pool)
            .await
            .expect("Failed to insert DNP entry");
        }

        // Test query performance
        let (result, duration) = PerformanceTestHelper::measure_async(|| async {
            sqlx::query!(
                r#"
                SELECT 
                    b.user_id,
                    b.artist_id,
                    b.tags,
                    b.note,
                    b.created_at,
                    a.canonical_name,
                    a.external_ids,
                    a.metadata
                FROM user_artist_blocks b
                JOIN artists a ON b.artist_id = a.id
                WHERE b.user_id = $1
                ORDER BY b.created_at DESC
                LIMIT 50
                "#,
                user.id
            )
            .fetch_all(&db.pool)
            .await
        })
        .await;

        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 50); // Limited to 50

        PerformanceTestHelper::assert_performance_threshold(duration, 100); // 100ms max
    }

    #[rstest]
    #[tokio::test]
    #[serial]
    async fn test_concurrent_database_operations(#[future] test_db: TestDatabase) {
        let db = test_db.await;

        let user = db.create_test_user().await;

        // Create multiple artists concurrently
        let mut handles = vec![];

        for i in 0..10 {
            let pool = db.pool.clone();
            let user_id = user.id;

            let handle = tokio::spawn(async move {
                let artist_name = format!("Concurrent Artist {}", i);

                // Create artist
                let artist = sqlx::query_as!(
                    Artist,
                    r#"
                    INSERT INTO artists (canonical_name, external_ids, metadata)
                    VALUES ($1, $2, $3)
                    RETURNING id, canonical_name, external_ids, metadata, created_at
                    "#,
                    artist_name,
                    serde_json::json!({"spotify": format!("spotify_{}", i)}),
                    serde_json::json!({"genres": ["test"]})
                )
                .fetch_one(&pool)
                .await
                .expect("Failed to create artist");

                // Add to DNP list
                sqlx::query!(
                    "INSERT INTO user_artist_blocks (user_id, artist_id, note) VALUES ($1, $2, $3)",
                    user_id,
                    artist.id,
                    format!("Concurrent note {}", i)
                )
                .execute(&pool)
                .await
                .expect("Failed to add to DNP list");

                artist.id
            });

            handles.push(handle);
        }

        // Wait for all operations to complete
        let results = futures::future::join_all(handles).await;

        // Verify all operations succeeded
        for result in results {
            assert!(result.is_ok());
        }

        // Verify final count
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM user_artist_blocks WHERE user_id = $1",
            user.id
        )
        .fetch_one(&db.pool)
        .await
        .expect("Failed to count entries");

        assert_eq!(count, Some(10));
    }
}
