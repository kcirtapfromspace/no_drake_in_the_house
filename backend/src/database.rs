use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::{Duration, Instant};
use tracing::{info, warn, error};
use deadpool_redis::{Config as RedisConfig, Pool as RedisPool, Runtime};

/// Database configuration and connection management
#[derive(Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://kiro:kiro_dev_password@localhost:5432/kiro_dev".to_string()),
            max_connections: 10,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600), // 10 minutes
        }
    }
}

/// Initialize database connection pool with proper configuration
pub async fn create_pool(config: DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    info!("Initializing database connection pool...");
    
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(config.connection_timeout)
        .idle_timeout(config.idle_timeout)
        .test_before_acquire(true) // Test connections before use
        .connect(&config.url)
        .await?;
    
    // Verify the connection works
    let _test_row = sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            error!("Failed to verify database connection: {}", e);
            e
        })?;
    
    info!("Database connection pool created and verified successfully");
    Ok(pool)
}

/// Run database migrations on startup
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Running database migrations...");
    
    let start = Instant::now();
    sqlx::migrate!("./migrations").run(pool).await?;
    
    let duration = start.elapsed();
    info!("Database migrations completed in {:?}", duration);
    
    Ok(())
}

/// Comprehensive database health check
pub async fn health_check(pool: &PgPool) -> DatabaseHealthStatus {
    let start = Instant::now();
    
    // Test basic connectivity
    let connectivity_result = sqlx::query("SELECT 1 as health_check")
        .fetch_one(pool)
        .await;
    
    let response_time = start.elapsed();
    
    match connectivity_result {
        Ok(_) => {
            // Test write capability
            let write_test = sqlx::query(
                "INSERT INTO health_check (status) VALUES ($1) ON CONFLICT DO NOTHING"
            )
            .bind("health_check")
            .execute(pool)
            .await;
            
            match write_test {
                Ok(_) => DatabaseHealthStatus {
                    status: "healthy".to_string(),
                    response_time_ms: response_time.as_millis() as u64,
                    details: Some(serde_json::json!({
                        "connectivity": "ok",
                        "write_test": "ok",
                        "pool_size": pool.size(),
                        "idle_connections": pool.num_idle()
                    })),
                    error: None,
                },
                Err(e) => {
                    warn!("Database write test failed: {}", e);
                    DatabaseHealthStatus {
                        status: "degraded".to_string(),
                        response_time_ms: response_time.as_millis() as u64,
                        details: Some(serde_json::json!({
                            "connectivity": "ok",
                            "write_test": "failed"
                        })),
                        error: Some(e.to_string()),
                    }
                }
            }
        }
        Err(e) => {
            error!("Database connectivity test failed: {}", e);
            DatabaseHealthStatus {
                status: "unhealthy".to_string(),
                response_time_ms: response_time.as_millis() as u64,
                details: None,
                error: Some(e.to_string()),
            }
        }
    }
}

/// Database health status structure
#[derive(serde::Serialize, Clone, Debug)]
pub struct DatabaseHealthStatus {
    pub status: String,
    pub response_time_ms: u64,
    pub details: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Complete database initialization including migrations and seeding
pub async fn initialize_database(config: DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    info!("Starting complete database initialization...");
    
    // Create connection pool
    let pool = create_pool(config).await?;
    
    // Run migrations
    run_migrations(&pool).await?;
    
    // Seed test data in development
    if let Err(e) = seed_test_data(&pool).await {
        warn!("Failed to seed test data: {}", e);
    }
    
    // Verify database health
    let health = health_check(&pool).await;
    if health.status != "healthy" {
        error!("Database health check failed after initialization: {:?}", health);
        return Err(sqlx::Error::Configuration("Database unhealthy after initialization".into()));
    }
    
    info!("Database initialization completed successfully");
    Ok(pool)
}

/// Seed test data for development environment
pub async fn seed_test_data(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Seeding test data...");
    
    // Check if we're in development mode
    let env = std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());
    if env != "development" {
        info!("Skipping test data seeding in {} environment", env);
        return Ok(());
    }
    
    // Check if test data already exists
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    
    if user_count > 0 {
        info!("Test data already exists, skipping seeding");
        return Ok(());
    }
    
    // Begin transaction for atomic seeding
    let mut tx = pool.begin().await?;
    
    // Create test users
    let test_user_id = sqlx::query_scalar!(
        r#"
        INSERT INTO users (email, password_hash, settings)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        "test@example.com",
        "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/RK.s5uDfS", // "password123"
        serde_json::json!({
            "notifications": true,
            "theme": "dark"
        })
    )
    .fetch_one(&mut *tx)
    .await?;
    
    // Create test artists
    let drake_id = sqlx::query_scalar!(
        r#"
        INSERT INTO artists (canonical_name, external_ids, metadata)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        "Drake",
        serde_json::json!({
            "spotify": "3TVXtAsR1Inumwj472S9r4",
            "apple": "271256",
            "musicbrainz": "b49b81cc-d5b7-4bdd-aadb-385df8de69a6"
        }),
        serde_json::json!({
            "genres": ["hip hop", "rap", "pop rap"],
            "image": "https://example.com/drake.jpg"
        })
    )
    .fetch_one(&mut *tx)
    .await?;
    
    let kanye_id = sqlx::query_scalar!(
        r#"
        INSERT INTO artists (canonical_name, external_ids, metadata)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        "Kanye West",
        serde_json::json!({
            "spotify": "5K4W6rqBFWDnAN6FQUkS6x",
            "apple": "2715720",
            "musicbrainz": "164f0d73-1234-4e2c-8743-d77bf2191051"
        }),
        serde_json::json!({
            "genres": ["hip hop", "rap", "experimental hip hop"],
            "image": "https://example.com/kanye.jpg"
        })
    )
    .fetch_one(&mut *tx)
    .await?;
    
    // Add artists to test user's DNP list
    sqlx::query!(
        r#"
        INSERT INTO user_artist_blocks (user_id, artist_id, tags, note)
        VALUES ($1, $2, $3, $4)
        "#,
        test_user_id,
        drake_id,
        &vec!["test".to_string(), "hip-hop".to_string()],
        "Test DNP entry for Drake"
    )
    .execute(&mut *tx)
    .await?;
    
    sqlx::query!(
        r#"
        INSERT INTO user_artist_blocks (user_id, artist_id, tags, note)
        VALUES ($1, $2, $3, $4)
        "#,
        test_user_id,
        kanye_id,
        &vec!["test".to_string(), "controversial".to_string()],
        "Test DNP entry for Kanye West"
    )
    .execute(&mut *tx)
    .await?;
    
    // Create a test community list
    let community_list_id = sqlx::query_scalar!(
        r#"
        INSERT INTO community_lists (owner_user_id, name, description, criteria)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        test_user_id,
        "Test Community List",
        "A test community list for development",
        "Artists with controversial public statements"
    )
    .fetch_one(&mut *tx)
    .await?;
    
    // Add artists to community list
    sqlx::query!(
        r#"
        INSERT INTO community_list_items (list_id, artist_id, rationale_link)
        VALUES ($1, $2, $3)
        "#,
        community_list_id,
        kanye_id,
        "https://example.com/rationale"
    )
    .execute(&mut *tx)
    .await?;
    
    // Commit transaction
    tx.commit().await?;
    
    info!("Test data seeded successfully");
    info!("Test user: test@example.com / password123");
    info!("Test user ID: {}", test_user_id);
    
    Ok(())
}

/// Redis configuration
#[derive(Clone)]
pub struct RedisConfiguration {
    pub url: String,
    pub max_size: usize,
    pub timeout: Duration,
}

impl Default for RedisConfiguration {
    fn default() -> Self {
        Self {
            url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            max_size: 10,
            timeout: Duration::from_secs(5),
        }
    }
}

/// Initialize Redis connection pool
pub async fn create_redis_pool(config: RedisConfiguration) -> Result<RedisPool, Box<dyn std::error::Error + Send + Sync>> {
    info!("Initializing Redis connection pool...");
    
    let redis_config = RedisConfig::from_url(&config.url);
    let pool = redis_config.create_pool(Some(Runtime::Tokio1))?;
    
    // Test the connection
    match pool.get().await {
        Ok(mut conn) => {
            match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
                Ok(_) => {
                    info!("Redis connection test successful");
                }
                Err(e) => {
                    error!("Redis PING failed: {}", e);
                    return Err(format!("Redis PING failed: {}", e).into());
                }
            }
        }
        Err(e) => {
            error!("Failed to get Redis connection: {}", e);
            return Err(format!("Connection test failed: {}", e).into());
        }
    }
    
    info!("Redis connection pool created and verified successfully");
    Ok(pool)
}

/// Redis health check
pub async fn redis_health_check(pool: &RedisPool) -> RedisHealthStatus {
    let start = Instant::now();
    
    match pool.get().await {
        Ok(mut conn) => {
            let ping_result: Result<String, redis::RedisError> = redis::cmd("PING")
                .query_async(&mut conn)
                .await;
            
            let response_time = start.elapsed();
            
            match ping_result {
                Ok(_) => {
                    // Test a simple operation
                    let set_result: Result<(), redis::RedisError> = redis::cmd("SET")
                        .arg("health_check")
                        .arg("ok")
                        .arg("EX")
                        .arg(60) // Expire in 60 seconds
                        .query_async(&mut conn)
                        .await;
                    
                    match set_result {
                        Ok(_) => RedisHealthStatus {
                            status: "healthy".to_string(),
                            response_time_ms: response_time.as_millis() as u64,
                            details: Some(serde_json::json!({
                                "ping": "ok",
                                "write_test": "ok",
                                "pool_status": {
                                    "size": pool.status().size,
                                    "available": pool.status().available
                                }
                            })),
                            error: None,
                        },
                        Err(e) => {
                            warn!("Redis write test failed: {}", e);
                            RedisHealthStatus {
                                status: "degraded".to_string(),
                                response_time_ms: response_time.as_millis() as u64,
                                details: Some(serde_json::json!({
                                    "ping": "ok",
                                    "write_test": "failed"
                                })),
                                error: Some(e.to_string()),
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Redis PING failed: {}", e);
                    RedisHealthStatus {
                        status: "unhealthy".to_string(),
                        response_time_ms: response_time.as_millis() as u64,
                        details: None,
                        error: Some(e.to_string()),
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to get Redis connection: {}", e);
            RedisHealthStatus {
                status: "unhealthy".to_string(),
                response_time_ms: start.elapsed().as_millis() as u64,
                details: None,
                error: Some(e.to_string()),
            }
        }
    }
}

/// Redis health status structure
#[derive(serde::Serialize, Clone, Debug)]
pub struct RedisHealthStatus {
    pub status: String,
    pub response_time_ms: u64,
    pub details: Option<serde_json::Value>,
    pub error: Option<String>,
}