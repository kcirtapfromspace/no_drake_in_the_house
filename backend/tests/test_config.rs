use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;

/// Get database URL for tests
pub fn get_test_database_url() -> String {
    env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://kiro:kiro_dev_password@localhost:5432/kiro_dev".to_string())
}

/// Create a test database pool
pub async fn create_test_pool() -> PgPool {
    let database_url = get_test_database_url();
    
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Setup test database with migrations
pub async fn setup_test_database() -> PgPool {
    let pool = create_test_pool().await;
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    pool
}

/// Clean up test data
pub async fn cleanup_test_data(pool: &PgPool) {
    // Clean up in reverse dependency order
    let _ = sqlx::query!("DELETE FROM user_artist_blocks").execute(pool).await;
    let _ = sqlx::query!("DELETE FROM community_list_items").execute(pool).await;
    let _ = sqlx::query!("DELETE FROM user_list_subscriptions").execute(pool).await;
    let _ = sqlx::query!("DELETE FROM community_lists").execute(pool).await;
    let _ = sqlx::query!("DELETE FROM artists").execute(pool).await;
    let _ = sqlx::query!("DELETE FROM users").execute(pool).await;
}