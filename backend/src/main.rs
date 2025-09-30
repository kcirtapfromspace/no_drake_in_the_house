use music_streaming_blocklist_backend::{
    AppState, create_router, AuthService, RateLimitService, AuditLoggingService, DnpListService, UserService,
    DatabaseConfig, create_pool, validate_cors_config, RedisConfiguration, create_redis_pool,
    MonitoringSystem, MonitoringConfig, run_migrations
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::{sync::Arc, env};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for migration command
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "migrate" {
        return run_migration_command().await;
    }
    // Initialize structured logging with JSON output
    init_tracing();

    tracing::info!("🎵 Music Streaming Blocklist Manager API starting...");

    // Validate CORS configuration
    validate_cors_config().map_err(|e| format!("CORS configuration error: {}", e))?;
    tracing::info!("CORS configuration validated");

    // Initialize database connection pool
    let db_config = DatabaseConfig::default();
    tracing::info!("Initializing database: {}", db_config.url);
    
    let db_pool = create_pool(db_config).await
        .map_err(|e| format!("Failed to create database pool: {}", e))?;
    
    // Run migrations automatically on startup
    run_migrations(&db_pool).await
        .map_err(|e| format!("Database migration failed: {}", e))?;
    tracing::info!("Database initialization completed");

    // Initialize Redis connection pool
    let redis_config = RedisConfiguration::default();
    tracing::info!("Using Redis URL: {}", redis_config.url);
    
    let redis_pool = create_redis_pool(redis_config).await
        .map_err(|e| format!("Failed to initialize Redis pool: {}", e))?;
    tracing::info!("Redis connection pool initialized");

    // Initialize monitoring system
    let monitoring_config = MonitoringConfig::default();
    let monitoring = Arc::new(MonitoringSystem::new(monitoring_config.clone())
        .map_err(|e| format!("Failed to initialize monitoring system: {}", e))?);
    let metrics = monitoring.metrics();
    
    tracing::info!("Monitoring system initialized");

    // Initialize services with error handling
    let auth_service = Arc::new(AuthService::new(db_pool.clone()));
    
    let rate_limiter = Arc::new(RateLimitService::new(&std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string()))
        .map_err(|e| format!("Failed to initialize rate limiter: {}", e))?);
    
    let audit_logger = Arc::new(AuditLoggingService::new(db_pool.clone()));
    let dnp_service = Arc::new(DnpListService::new(db_pool.clone()));
    let user_service = Arc::new(UserService::new(db_pool.clone()));

    tracing::info!("All services initialized successfully");
    
    // Start background monitoring tasks
    monitoring.start_background_monitoring(
        monitoring_config,
        db_pool.clone(),
        redis_pool.clone(),
    ).await;

    // Initialize application state
    let app_state = AppState {
        db_pool,
        redis_pool,
        auth_service,
        rate_limiter,
        audit_logger,
        dnp_service,
        user_service,
        monitoring,
        metrics,
    };

    // Build application router
    let app = create_router(app_state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("🚀 Server running on http://0.0.0.0:3000");
    
    axum::serve(listener, app).await?;

    Ok(())
}

/// Run database migrations only
async fn run_migration_command() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize basic logging for migration
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🔄 Running database migrations...");

    // Initialize database connection pool
    let db_config = DatabaseConfig::default();
    tracing::info!("Database URL: {}", db_config.url);
    
    let db_pool = create_pool(db_config).await
        .map_err(|e| format!("Failed to create database pool: {}", e))?;
    
    // Run migrations
    run_migrations(&db_pool).await
        .map_err(|e| format!("Migration failed: {}", e))?;
    
    tracing::info!("✅ Database migrations completed successfully!");
    
    Ok(())
}

/// Initialize structured logging with JSON output for production
fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "music_streaming_blocklist_backend=debug,tower_http=debug".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .json() // JSON structured logging
                .with_current_span(false)
                .with_span_list(true)
        )
        .init();
}



