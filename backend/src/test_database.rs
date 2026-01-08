use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};
use std::sync::Once;
use tracing::info;

static INIT: Once = Once::new();

/// Initialize tracing for tests (only once)
pub fn init_test_tracing() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter("debug")
            .init();
    });
}

/// Create an in-memory SQLite database for testing
pub async fn create_test_database() -> SqlitePool {
    init_test_tracing();

    // Create in-memory SQLite database
    let pool = SqlitePoolOptions::new()
        .max_connections(1) // SQLite in-memory databases are single-connection
        .connect(":memory:")
        .await
        .expect("Failed to create in-memory SQLite database");

    // Create the necessary tables for testing
    setup_test_schema(&pool).await;

    info!("Test database created and initialized");
    pool
}

/// Set up the basic schema needed for OAuth tests
async fn setup_test_schema(pool: &SqlitePool) {
    // Create users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT,
            email_verified BOOLEAN DEFAULT FALSE,
            totp_enabled BOOLEAN DEFAULT FALSE,
            totp_secret TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            last_login TEXT,
            settings TEXT DEFAULT '{}'
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create users table");

    // Create oauth_accounts table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS oauth_accounts (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            provider TEXT NOT NULL,
            provider_user_id TEXT NOT NULL,
            email TEXT,
            display_name TEXT,
            avatar_url TEXT,
            access_token_encrypted BLOB NOT NULL,
            refresh_token_encrypted BLOB,
            token_expires_at TEXT,
            last_used_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users (id),
            UNIQUE (provider, provider_user_id)
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create oauth_accounts table");

    // Create user_sessions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT NOT NULL,
            refresh_token_hash TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users (id)
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create user_sessions table");

    // Create artists table (for DNP functionality)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS artists (
            id TEXT PRIMARY KEY,
            canonical_name TEXT NOT NULL,
            external_ids TEXT NOT NULL,
            metadata TEXT NOT NULL,
            aliases TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create artists table");

    // Create user_artist_blocks table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_artist_blocks (
            user_id TEXT NOT NULL,
            artist_id TEXT NOT NULL,
            tags TEXT,
            note TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (user_id, artist_id),
            FOREIGN KEY (user_id) REFERENCES users (id),
            FOREIGN KEY (artist_id) REFERENCES artists (id)
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create user_artist_blocks table");

    // Create community_lists table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS community_lists (
            id TEXT PRIMARY KEY,
            owner_user_id TEXT,
            name TEXT NOT NULL,
            description TEXT,
            criteria TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (owner_user_id) REFERENCES users (id)
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create community_lists table");

    // Create community_list_items table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS community_list_items (
            list_id TEXT NOT NULL,
            artist_id TEXT NOT NULL,
            rationale_link TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (list_id, artist_id),
            FOREIGN KEY (list_id) REFERENCES community_lists (id),
            FOREIGN KEY (artist_id) REFERENCES artists (id)
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create community_list_items table");

    // Create user_list_subscriptions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_list_subscriptions (
            user_id TEXT NOT NULL,
            list_id TEXT NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (user_id, list_id),
            FOREIGN KEY (user_id) REFERENCES users (id),
            FOREIGN KEY (list_id) REFERENCES community_lists (id)
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create user_list_subscriptions table");

    // Create action_batches table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS action_batches (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users (id)
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create action_batches table");

    // Create connections table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS connections (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            service TEXT NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users (id)
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create connections table");

    // Create audit_log table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS audit_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT,
            action TEXT NOT NULL,
            details TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users (id)
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create audit_log table");

    info!("Test database schema created successfully");
}

/// Insert test data for OAuth tests
pub async fn insert_test_data(pool: &SqlitePool) {
    // Insert a test user
    let user_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, email_verified, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&user_id)
    .bind("test@example.com")
    .bind("$2b$12$test_hash")
    .bind(true)
    .bind(chrono::Utc::now().to_rfc3339())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(pool)
    .await
    .expect("Failed to insert test user");

    // Insert a test artist
    let artist_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO artists (id, canonical_name, external_ids, metadata)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&artist_id)
    .bind("Test Artist")
    .bind(r#"{"spotify": "test_spotify_id"}"#)
    .bind(r#"{"genres": ["test"], "image": "https://example.com/image.jpg"}"#)
    .execute(pool)
    .await
    .expect("Failed to insert test artist");

    info!("Test data inserted successfully");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_test_database() {
        let pool = create_test_database().await;

        // Test that we can query the database
        let result = sqlx::query("SELECT COUNT(*) as count FROM users")
            .fetch_one(&pool)
            .await
            .unwrap();

        let count: i64 = result.get("count");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_insert_test_data() {
        let pool = create_test_database().await;
        insert_test_data(&pool).await;

        // Verify test data was inserted
        let result = sqlx::query("SELECT COUNT(*) as count FROM users")
            .fetch_one(&pool)
            .await
            .unwrap();

        let count: i64 = result.get("count");
        assert_eq!(count, 1);
    }
}
