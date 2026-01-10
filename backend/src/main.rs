use music_streaming_blocklist_backend::{
    create_pool, create_redis_pool, create_router, run_migrations, validate_cors_config, AppState,
    AuditLoggingService, AuthService, DatabaseConfig, DnpListService, MonitoringConfig,
    MonitoringSystem, RateLimitService, RedisConfiguration, UserService,
    OrchestratorBuilder, PlatformSyncConfig, CreditsSyncService,
    NewsPipelineConfig, NewsPipelineOrchestrator, ScheduledPipelineRunner,
    BackfillOrchestrator,
};
use music_streaming_blocklist_backend::services::{
    AppleMusicService, AppleMusicConfig,
};
use music_streaming_blocklist_backend::services::catalog_sync::{
    AppleMusicSyncWorker, DeezerSyncWorker, CrossPlatformIdentityResolver,
};
use music_streaming_blocklist_backend::services::stubs::TokenVaultService;
use chrono::Duration;
use std::{env, sync::Arc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

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

    let db_pool = create_pool(db_config)
        .await
        .map_err(|e| format!("Failed to create database pool: {}", e))?;

    // Run migrations automatically on startup
    run_migrations(&db_pool)
        .await
        .map_err(|e| format!("Database migration failed: {}", e))?;
    tracing::info!("Database initialization completed");

    // Initialize Redis connection pool
    let redis_config = RedisConfiguration::default();
    tracing::info!("Using Redis URL: {}", redis_config.url);

    let redis_pool = create_redis_pool(redis_config)
        .await
        .map_err(|e| format!("Failed to initialize Redis pool: {}", e))?;
    tracing::info!("Redis connection pool initialized");

    // Initialize monitoring system
    let monitoring_config = MonitoringConfig::default();
    let monitoring = Arc::new(
        MonitoringSystem::new(monitoring_config.clone())
            .map_err(|e| format!("Failed to initialize monitoring system: {}", e))?,
    );
    let metrics = monitoring.metrics();

    tracing::info!("Monitoring system initialized");

    // Initialize services with error handling
    let auth_service = Arc::new(AuthService::new(db_pool.clone()));

    let rate_limiter = Arc::new(
        RateLimitService::new(
            &std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        )
        .map_err(|e| format!("Failed to initialize rate limiter: {}", e))?,
    );

    let audit_logger = Arc::new(AuditLoggingService::new(db_pool.clone()));
    let dnp_service = Arc::new(DnpListService::new(db_pool.clone()));
    let user_service = Arc::new(UserService::new(db_pool.clone()));

    tracing::info!("All services initialized successfully");

    // Start background monitoring tasks
    monitoring
        .start_background_monitoring(monitoring_config, db_pool.clone(), redis_pool.clone())
        .await;

    // Initialize platform sync configuration
    let platform_config = PlatformSyncConfig::from_env();
    tracing::info!(
        "Platform sync configured. Available platforms: {:?}",
        platform_config.available_platforms()
    );

    // Initialize catalog sync orchestrator with available workers
    let identity_resolver = CrossPlatformIdentityResolver::new(
        "NoDrakeInTheHouse",
        "1.0",
        "admin@nodrake.example.com",
    );

    let mut orchestrator_builder = OrchestratorBuilder::new()
        .with_identity_resolver(identity_resolver)
        .with_db_pool(db_pool.clone());

    // Always add Deezer (no auth required)
    orchestrator_builder = orchestrator_builder.with_worker(DeezerSyncWorker::new());
    tracing::info!("Deezer sync worker registered (public API, no auth needed)");
    tracing::info!("Catalog sync orchestrator configured with database persistence");

    // Add Apple Music worker if credentials are available
    let credits_sync = if let Some(apple_creds) = &platform_config.apple_music {
        orchestrator_builder = orchestrator_builder.with_worker(AppleMusicSyncWorker::new(
            apple_creds.team_id.clone(),
            apple_creds.key_id.clone(),
            apple_creds.private_key.clone(),
            "us".to_string(), // Default storefront
        ));
        tracing::info!("Apple Music sync worker registered");

        // Initialize credits sync service with Apple Music credentials
        let credits_service = CreditsSyncService::new(
            db_pool.clone(),
            apple_creds.team_id.clone(),
            apple_creds.key_id.clone(),
            apple_creds.private_key.clone(),
            "us".to_string(),
        );
        tracing::info!("Credits sync service initialized (Apple Music)");
        Some(Arc::new(credits_service))
    } else {
        tracing::warn!("Credits sync service not available (no Apple Music credentials)");
        None
    };

    // TODO: Add other platform workers when credentials are available
    // if let Some(spotify_creds) = &platform_config.spotify {
    //     orchestrator_builder = orchestrator_builder.with_worker(
    //         SpotifySyncWorker::new(spotify_creds.client_id.clone(), spotify_creds.client_secret.clone())
    //     );
    // }

    let catalog_sync = Arc::new(
        orchestrator_builder
            .build()
            .expect("Failed to build catalog sync orchestrator"),
    );
    tracing::info!("Catalog sync orchestrator initialized");

    // Initialize news pipeline with scheduled polling (10 minute RSS intervals)
    // Uses database persistence to auto-create artist_offenses from detected news
    let news_config = NewsPipelineConfig::default();
    let news_pipeline = Arc::new(NewsPipelineOrchestrator::with_database(
        news_config,
        db_pool.clone(),
    ));

    // Start scheduled news polling - RSS every 10 minutes for near real-time updates
    let rss_interval_minutes: i64 = std::env::var("NEWS_RSS_INTERVAL_MINUTES")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .unwrap_or(10);

    let _news_scheduler = ScheduledPipelineRunner::new(news_pipeline)
        .with_rss_interval(Duration::minutes(rss_interval_minutes))
        .with_social_interval(Duration::hours(1))
        .with_full_interval(Duration::hours(6))
        .start();

    tracing::info!(
        rss_interval_minutes = rss_interval_minutes,
        "News pipeline started: RSS polling every {} minutes",
        rss_interval_minutes
    );

    // Initialize backfill orchestrator for offense discovery
    // Note: For full news pipeline integration, we'd pass the news_pipeline Arc here
    let backfill_orchestrator = Some(Arc::new(
        BackfillOrchestrator::new(db_pool.clone())
    ));
    tracing::info!("Backfill orchestrator initialized");

    // Initialize Apple Music service for enforcement
    let apple_music_config = AppleMusicConfig::default();
    let token_vault = Arc::new(TokenVaultService::new());
    let apple_music_service = Arc::new(
        AppleMusicService::new(apple_music_config, token_vault)
            .expect("Failed to create Apple Music service")
    );
    tracing::info!("Apple Music enforcement service initialized");

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
        catalog_sync,
        platform_config,
        credits_sync,
        backfill_orchestrator,
        apple_music_service,
        test_user_id: None, // Will be populated from auth middleware in production
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
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🔄 Running database migrations...");

    // Initialize database connection pool
    let db_config = DatabaseConfig::default();
    tracing::info!("Database URL: {}", db_config.url);

    let db_pool = create_pool(db_config)
        .await
        .map_err(|e| format!("Failed to create database pool: {}", e))?;

    // Run migrations
    run_migrations(&db_pool)
        .await
        .map_err(|e| format!("Migration failed: {}", e))?;

    tracing::info!("✅ Database migrations completed successfully!");

    Ok(())
}

/// Initialize structured logging with JSON output for production
fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "music_streaming_blocklist_backend=debug,tower_http=debug".into()
            }),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .json() // JSON structured logging
                .with_current_span(false)
                .with_span_list(true),
        )
        .init();
}
