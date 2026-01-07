//! Category subscription integration tests
//! Tests the full flow of subscribing to categories and blocking artists

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

/// Test setup helper that creates a user and returns auth token
async fn setup_authenticated_user(pool: &PgPool) -> (Uuid, String) {
    let user_id = Uuid::new_v4();
    let email = format!("test_{}@example.com", user_id);
    let password_hash = "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.wQrqQjA.H.zMXG"; // "password123"

    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, created_at)
        VALUES ($1, $2, $3, NOW())
        "#
    )
    .bind(user_id)
    .bind(&email)
    .bind(password_hash)
    .execute(pool)
    .await
    .expect("Failed to create test user");

    // Create a mock JWT token (in real tests, call auth endpoint)
    let token = format!("test_token_{}", user_id);

    (user_id, token)
}

/// Seed test artists with offenses for testing
async fn seed_test_artists(pool: &PgPool) -> Vec<Uuid> {
    let mut artist_ids = Vec::new();

    // Create domestic violence artist
    let dv_artist = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO artists (id, canonical_name, external_ids, metadata)
        VALUES ($1, 'Test DV Artist', '{"spotify": "test_dv"}'::jsonb, '{"genres": ["test"]}'::jsonb)
        "#
    )
    .bind(dv_artist)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO artist_offenses (artist_id, category, severity, title, description, status)
        VALUES ($1, 'domestic_violence'::offense_category, 'severe'::offense_severity,
                'Test DV Offense', 'Test description', 'verified'::evidence_status)
        "#
    )
    .bind(dv_artist)
    .execute(pool)
    .await
    .unwrap();

    artist_ids.push(dv_artist);

    // Create hate speech artist
    let hs_artist = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO artists (id, canonical_name, external_ids, metadata)
        VALUES ($1, 'Test HS Artist', '{"spotify": "test_hs"}'::jsonb, '{"genres": ["test"]}'::jsonb)
        "#
    )
    .bind(hs_artist)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO artist_offenses (artist_id, category, severity, title, description, status)
        VALUES ($1, 'hate_speech'::offense_category, 'moderate'::offense_severity,
                'Test HS Offense', 'Test description', 'verified'::evidence_status)
        "#
    )
    .bind(hs_artist)
    .execute(pool)
    .await
    .unwrap();

    artist_ids.push(hs_artist);

    // Create artist with multiple offenses
    let multi_artist = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO artists (id, canonical_name, external_ids, metadata)
        VALUES ($1, 'Test Multi Artist', '{"spotify": "test_multi"}'::jsonb, '{"genres": ["test"]}'::jsonb)
        "#
    )
    .bind(multi_artist)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO artist_offenses (artist_id, category, severity, title, description, status)
        VALUES ($1, 'domestic_violence'::offense_category, 'severe'::offense_severity,
                'Test Multi DV', 'Test description', 'verified'::evidence_status)
        "#
    )
    .bind(multi_artist)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO artist_offenses (artist_id, category, severity, title, description, status)
        VALUES ($1, 'racism'::offense_category, 'moderate'::offense_severity,
                'Test Multi Racism', 'Test description', 'verified'::evidence_status)
        "#
    )
    .bind(multi_artist)
    .execute(pool)
    .await
    .unwrap();

    artist_ids.push(multi_artist);

    artist_ids
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test getting categories returns correct structure
    #[tokio::test]
    #[ignore = "Requires database connection"]
    async fn test_get_categories_structure() {
        // This would be run with a real database
        let pool = get_test_pool().await;
        let (user_id, _) = setup_authenticated_user(&pool).await;
        seed_test_artists(&pool).await;

        let rows = sqlx::query(
            r#"
            SELECT category::text as category, COUNT(DISTINCT artist_id) as count
            FROM artist_offenses
            WHERE status IN ('pending', 'verified')
            GROUP BY category
            "#
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        // Verify we have categories with counts
        assert!(!rows.is_empty(), "Should have offense categories");

        // Verify domestic_violence category exists
        let dv_count: Option<i64> = rows.iter()
            .find(|r| r.get::<String, _>("category") == "domestic_violence")
            .map(|r| r.get("count"));

        assert!(dv_count.is_some(), "Should have domestic_violence category");
        assert!(dv_count.unwrap() >= 2, "Should have at least 2 DV artists");
    }

    /// Test subscribing to a category
    #[tokio::test]
    #[ignore = "Requires database connection"]
    async fn test_subscribe_to_category() {
        let pool = get_test_pool().await;
        let (user_id, _) = setup_authenticated_user(&pool).await;

        // Subscribe to domestic_violence
        sqlx::query(
            "INSERT INTO category_subscriptions (user_id, category) VALUES ($1, 'domestic_violence'::offense_category)"
        )
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

        // Verify subscription exists
        let sub_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM category_subscriptions WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(sub_count, 1, "Should have one subscription");
    }

    /// Test that subscribed categories block correct artists
    #[tokio::test]
    #[ignore = "Requires database connection"]
    async fn test_category_subscription_blocks_artists() {
        let pool = get_test_pool().await;
        let (user_id, _) = setup_authenticated_user(&pool).await;
        let artist_ids = seed_test_artists(&pool).await;

        // Subscribe to domestic_violence
        sqlx::query(
            "INSERT INTO category_subscriptions (user_id, category) VALUES ($1, 'domestic_violence'::offense_category)"
        )
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

        // Get blocked artists
        let blocked_rows = sqlx::query(
            r#"
            SELECT DISTINCT
                a.id,
                a.canonical_name as name,
                ao.category::text as category
            FROM category_subscriptions cs
            JOIN artist_offenses ao ON ao.category = cs.category
            JOIN artists a ON a.id = ao.artist_id
            WHERE cs.user_id = $1
            AND ao.status IN ('pending', 'verified')
            "#
        )
        .bind(user_id)
        .fetch_all(&pool)
        .await
        .unwrap();

        // Should block DV artist and Multi artist (both have DV offenses)
        assert_eq!(blocked_rows.len(), 2, "Should block 2 artists");

        let blocked_names: Vec<String> = blocked_rows.iter()
            .map(|r| r.get("name"))
            .collect();

        assert!(blocked_names.contains(&"Test DV Artist".to_string()));
        assert!(blocked_names.contains(&"Test Multi Artist".to_string()));
        assert!(!blocked_names.contains(&"Test HS Artist".to_string()));
    }

    /// Test unsubscribing removes blocked artists
    #[tokio::test]
    #[ignore = "Requires database connection"]
    async fn test_unsubscribe_removes_blocks() {
        let pool = get_test_pool().await;
        let (user_id, _) = setup_authenticated_user(&pool).await;
        seed_test_artists(&pool).await;

        // Subscribe to domestic_violence
        sqlx::query(
            "INSERT INTO category_subscriptions (user_id, category) VALUES ($1, 'domestic_violence'::offense_category)"
        )
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

        // Verify blocks exist
        let initial_blocked: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(DISTINCT a.id)
            FROM category_subscriptions cs
            JOIN artist_offenses ao ON ao.category = cs.category
            JOIN artists a ON a.id = ao.artist_id
            WHERE cs.user_id = $1
            "#
        )
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert!(initial_blocked > 0, "Should have blocked artists initially");

        // Unsubscribe
        sqlx::query(
            "DELETE FROM category_subscriptions WHERE user_id = $1 AND category = 'domestic_violence'::offense_category"
        )
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

        // Verify no more blocks
        let final_blocked: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(DISTINCT a.id)
            FROM category_subscriptions cs
            JOIN artist_offenses ao ON ao.category = cs.category
            JOIN artists a ON a.id = ao.artist_id
            WHERE cs.user_id = $1
            "#
        )
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(final_blocked, 0, "Should have no blocked artists after unsubscribe");
    }

    /// Test subscribing to multiple categories
    #[tokio::test]
    #[ignore = "Requires database connection"]
    async fn test_multiple_category_subscriptions() {
        let pool = get_test_pool().await;
        let (user_id, _) = setup_authenticated_user(&pool).await;
        seed_test_artists(&pool).await;

        // Subscribe to domestic_violence and hate_speech
        sqlx::query(
            "INSERT INTO category_subscriptions (user_id, category) VALUES ($1, 'domestic_violence'::offense_category)"
        )
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO category_subscriptions (user_id, category) VALUES ($1, 'hate_speech'::offense_category)"
        )
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

        // Get blocked artists
        let blocked_rows = sqlx::query(
            r#"
            SELECT DISTINCT a.canonical_name as name
            FROM category_subscriptions cs
            JOIN artist_offenses ao ON ao.category = cs.category
            JOIN artists a ON a.id = ao.artist_id
            WHERE cs.user_id = $1
            AND ao.status IN ('pending', 'verified')
            "#
        )
        .bind(user_id)
        .fetch_all(&pool)
        .await
        .unwrap();

        // Should block all 3 test artists
        assert_eq!(blocked_rows.len(), 3, "Should block 3 artists with both categories");
    }

    /// Test artist with multiple offense categories is only listed once
    #[tokio::test]
    #[ignore = "Requires database connection"]
    async fn test_distinct_blocked_artists() {
        let pool = get_test_pool().await;
        let (user_id, _) = setup_authenticated_user(&pool).await;
        seed_test_artists(&pool).await;

        // Subscribe to domestic_violence and racism (Multi artist has both)
        sqlx::query(
            "INSERT INTO category_subscriptions (user_id, category) VALUES ($1, 'domestic_violence'::offense_category)"
        )
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO category_subscriptions (user_id, category) VALUES ($1, 'racism'::offense_category)"
        )
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

        // Get blocked artists with DISTINCT
        let blocked_rows = sqlx::query(
            r#"
            SELECT DISTINCT a.id, a.canonical_name as name
            FROM category_subscriptions cs
            JOIN artist_offenses ao ON ao.category = cs.category
            JOIN artists a ON a.id = ao.artist_id
            WHERE cs.user_id = $1
            AND ao.status IN ('pending', 'verified')
            "#
        )
        .bind(user_id)
        .fetch_all(&pool)
        .await
        .unwrap();

        // Multi artist should appear only once despite having both DV and racism offenses
        let multi_count = blocked_rows.iter()
            .filter(|r| r.get::<String, _>("name") == "Test Multi Artist")
            .count();

        assert_eq!(multi_count, 1, "Multi-offense artist should appear exactly once");
    }

    /// Test idempotent subscription (subscribe twice doesn't error)
    #[tokio::test]
    #[ignore = "Requires database connection"]
    async fn test_idempotent_subscription() {
        let pool = get_test_pool().await;
        let (user_id, _) = setup_authenticated_user(&pool).await;

        // Subscribe twice
        for _ in 0..2 {
            sqlx::query(
                "INSERT INTO category_subscriptions (user_id, category) VALUES ($1, 'domestic_violence'::offense_category) ON CONFLICT DO NOTHING"
            )
            .bind(user_id)
            .execute(&pool)
            .await
            .unwrap();
        }

        // Should only have one subscription
        let sub_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM category_subscriptions WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(sub_count, 1, "Should have exactly one subscription");
    }
}

/// Get test database pool
async fn get_test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://kiro:kiro_dev_password@localhost:5432/kiro_dev".to_string());

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database")
}
