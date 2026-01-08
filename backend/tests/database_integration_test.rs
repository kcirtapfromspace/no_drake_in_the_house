use music_streaming_blocklist_backend::{db_health_check, initialize_database, DatabaseConfig};
use std::time::Duration;

#[tokio::test]
async fn test_database_initialization() {
    // Use test database configuration
    let config = DatabaseConfig {
        url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://kiro:kiro_dev_password@localhost:5432/kiro_dev".to_string()
        }),
        max_connections: 5,
        connection_timeout: Duration::from_secs(10),
        idle_timeout: Duration::from_secs(300),
    };

    // Test database initialization
    let pool = initialize_database(config)
        .await
        .expect("Failed to initialize database");

    // Test health check
    let health = db_health_check(&pool).await;
    assert_eq!(health.status, "healthy");
    assert!(health.response_time_ms >= 0); // Allow 0ms for very fast responses
    assert!(health.error.is_none());

    // Test basic query
    let result: i32 = sqlx::query_scalar("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("Failed to execute test query");
    assert_eq!(result, 1);

    // Test that required tables exist
    let table_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public'",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count tables");

    // Should have at least the core tables: users, artists, user_artist_blocks, audit_log, health_check
    assert!(
        table_count >= 5,
        "Expected at least 5 tables, found {}",
        table_count
    );

    // Test that test data was seeded (in development environment)
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .expect("Failed to count users");

    if std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()) == "development" {
        assert!(
            user_count > 0,
            "Expected test users to be seeded in development"
        );
    }

    println!("Database initialization test passed successfully!");
}

#[tokio::test]
async fn test_database_health_check_functionality() {
    let config = DatabaseConfig::default();
    let pool = initialize_database(config)
        .await
        .expect("Failed to initialize database");

    // Test health check multiple times to ensure consistency
    for i in 0..3 {
        let health = db_health_check(&pool).await;
        assert_eq!(health.status, "healthy", "Health check {} failed", i + 1);
        assert!(
            health.response_time_ms >= 0,
            "Response time should be non-negative"
        );

        if let Some(details) = &health.details {
            assert_eq!(details["connectivity"], "ok");
            assert_eq!(details["write_test"], "ok");
            assert!(details["pool_size"].as_u64().unwrap() > 0);
        }
    }

    println!("Database health check functionality test passed!");
}

#[tokio::test]
async fn test_migration_idempotency() {
    let config = DatabaseConfig::default();

    // Run initialization twice to ensure migrations are idempotent
    let pool1 = initialize_database(config.clone())
        .await
        .expect("First initialization failed");
    let pool2 = initialize_database(config)
        .await
        .expect("Second initialization failed");

    // Both should be healthy
    let health1 = db_health_check(&pool1).await;
    let health2 = db_health_check(&pool2).await;

    assert_eq!(health1.status, "healthy");
    assert_eq!(health2.status, "healthy");

    println!("Migration idempotency test passed!");
}
