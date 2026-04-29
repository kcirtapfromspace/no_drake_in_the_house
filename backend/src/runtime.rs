use crate::config::TokenRefreshConfig;
use crate::services::catalog_sync::{
    AppleMusicSyncWorker, CrossPlatformIdentityResolver, DeezerSyncWorker, SpotifySyncWorker,
};
use crate::services::{
    AppleMusicConfig, AppleMusicService, NotificationService, TokenRefreshBackgroundJob,
};
use crate::{
    create_pool, create_redis_pool, create_router, run_migrations, validate_cors_config, AppState,
    AuditLoggingService, AuthService, BackfillOrchestrator, CircuitBreakerConfig,
    CircuitBreakerService, CreditsSyncService, DatabaseConfig, DnpListService, MonitoringConfig,
    MonitoringSystem, OrchestratorBuilder, PlatformSyncConfig, RateLimitService,
    RedisConfiguration, TokenVaultService, UserService,
};
#[cfg(feature = "news")]
use crate::{NewsPipelineConfig, NewsPipelineOrchestrator, ScheduledPipelineRunner};
use axum::Router;
#[cfg(feature = "news")]
use chrono::Duration;
use std::{env, sync::Arc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceMode {
    Monolith,
    Api,
    Analytics,
    Graph,
    News,
}

impl ServiceMode {
    pub fn from_env_or_default() -> Self {
        match env::var("NDITH_SERVICE_MODE")
            .ok()
            .as_deref()
            .map(str::trim)
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("monolith") => Self::Monolith,
            Some("api") => Self::Api,
            Some("analytics") => Self::Analytics,
            Some("graph") => Self::Graph,
            Some("news") => Self::News,
            _ => {
                if cfg!(feature = "full-platform") {
                    Self::Monolith
                } else {
                    Self::Api
                }
            }
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Monolith => "monolith",
            Self::Api => "api",
            Self::Analytics => "analytics",
            Self::Graph => "graph",
            Self::News => "news",
        }
    }

    fn router(self, state: AppState) -> Router {
        match self {
            Self::Monolith | Self::Api => create_router(state),
            #[cfg(feature = "analytics")]
            Self::Analytics => crate::create_analytics_router(state),
            #[cfg(not(feature = "analytics"))]
            Self::Analytics => create_router(state),
            #[cfg(feature = "analytics")]
            Self::Graph => crate::create_graph_router(state),
            #[cfg(not(feature = "analytics"))]
            Self::Graph => create_router(state),
            #[cfg(feature = "news")]
            Self::News => crate::create_news_router(state),
            #[cfg(not(feature = "news"))]
            Self::News => create_router(state),
        }
    }

    fn should_run_migrations(self) -> bool {
        matches!(self, Self::Monolith | Self::Api)
    }

    fn should_start_token_refresh(self) -> bool {
        matches!(self, Self::Monolith | Self::Api)
    }

    fn should_start_news_pipeline(self) -> bool {
        matches!(self, Self::Monolith | Self::News)
    }

    /// Whether the backend running in this mode is expected to write
    /// to the DuckDB analytics store on disk. Used to gate the
    /// fail-fast `DUCKDB_PATH` validation on startup.
    fn requires_duckdb_storage(self) -> bool {
        matches!(self, Self::Monolith | Self::Analytics)
    }
}

/// Fail-fast validation that the canonical DuckDB analytics path is
/// configured and writable when the service is expected to run
/// analytics writes. Returns `Err` with an explicit log so the pod
/// crashlooops instead of silently writing nowhere (NOD-196).
fn ensure_duckdb_storage_ready(mode: ServiceMode) -> Result<(), String> {
    let raw = env::var("DUCKDB_PATH").ok();
    let required = mode.requires_duckdb_storage();

    let path = match raw.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(p) => p.to_string(),
        None => {
            if required {
                let msg = format!(
                    "DUCKDB_PATH is not set, but service_mode={} requires a writable DuckDB store. \
                     Set DUCKDB_PATH and mount a writable volume at its parent directory.",
                    mode.as_str()
                );
                tracing::error!(service_mode = mode.as_str(), "{}", msg);
                return Err(msg);
            }
            tracing::warn!(
                service_mode = mode.as_str(),
                "DUCKDB_PATH not set; skipping analytics storage probe (mode does not require it)"
            );
            return Ok(());
        }
    };

    let db_path = std::path::PathBuf::from(&path);
    let parent = db_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    if let Err(e) = std::fs::create_dir_all(&parent) {
        let msg = format!(
            "DUCKDB_PATH parent directory {} is not creatable: {}. \
             Mount a writable volume at this path.",
            parent.display(),
            e
        );
        if required {
            tracing::error!(service_mode = mode.as_str(), duckdb_path = %path, "{}", msg);
            return Err(msg);
        }
        tracing::warn!(service_mode = mode.as_str(), duckdb_path = %path, "{}", msg);
        return Ok(());
    }

    let probe = parent.join(".duckdb_writability_probe");
    let write_result = std::fs::write(&probe, b"ok").and_then(|_| std::fs::remove_file(&probe));
    if let Err(e) = write_result {
        let msg = format!(
            "DUCKDB_PATH parent directory {} is not writable by runtime UID/GID: {}. \
             Check the mounted volume's fsGroup/permissions.",
            parent.display(),
            e
        );
        if required {
            tracing::error!(service_mode = mode.as_str(), duckdb_path = %path, "{}", msg);
            return Err(msg);
        }
        tracing::warn!(service_mode = mode.as_str(), duckdb_path = %path, "{}", msg);
        return Ok(());
    }

    tracing::info!(
        service_mode = mode.as_str(),
        duckdb_path = %path,
        duckdb_dir = %parent.display(),
        "DuckDB analytics storage probe passed"
    );
    Ok(())
}

pub async fn run_service(mode: ServiceMode) -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "migrate" {
        return run_migration_command().await;
    }

    init_tracing();

    tracing::info!(
        service_mode = mode.as_str(),
        "Music Streaming Blocklist backend starting"
    );

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .map_err(|e| format!("Invalid PORT value: {}", e))?;
    let bind_addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!(
        service_mode = mode.as_str(),
        bind_addr = %bind_addr,
        "Bound listener; continuing startup"
    );

    validate_cors_config().map_err(|e| format!("CORS configuration error: {}", e))?;
    tracing::info!("CORS configuration validated");

    ensure_duckdb_storage_ready(mode)
        .map_err(|e| format!("DuckDB analytics storage validation failed: {}", e))?;

    let db_config = DatabaseConfig::default();
    tracing::info!("Initializing database: {}", db_config.url);

    let db_pool = create_pool(db_config)
        .await
        .map_err(|e| format!("Failed to create database pool: {}", e))?;

    if mode.should_run_migrations() {
        run_migrations(&db_pool)
            .await
            .map_err(|e| format!("Database migration failed: {}", e))?;
        tracing::info!(
            service_mode = mode.as_str(),
            "Database migrations completed"
        );
    } else {
        tracing::info!(
            service_mode = mode.as_str(),
            "Skipping database migrations for scoped service"
        );
    }

    let redis_config = RedisConfiguration::default();
    tracing::info!("Using Redis URL: {}", redis_config.url);

    let redis_pool = create_redis_pool(redis_config)
        .await
        .map_err(|e| format!("Failed to initialize Redis pool: {}", e))?;
    tracing::info!("Redis connection pool initialized");

    let monitoring_config = MonitoringConfig::default();
    let monitoring = Arc::new(
        MonitoringSystem::new(monitoring_config.clone())
            .map_err(|e| format!("Failed to initialize monitoring system: {}", e))?,
    );
    let metrics = monitoring.metrics();

    monitoring
        .start_background_monitoring(monitoring_config, db_pool.clone(), redis_pool.clone())
        .await;
    tracing::info!("Monitoring system initialized");

    let circuit_breaker_config = CircuitBreakerConfig::default();
    let circuit_breaker = Arc::new(
        CircuitBreakerService::with_config(circuit_breaker_config.clone())
            .with_metrics(metrics.registry().as_ref())
            .map_err(|e| format!("Failed to initialize circuit breaker: {}", e))?,
    );
    tracing::info!(
        service_mode = mode.as_str(),
        failure_threshold = circuit_breaker_config.failure_threshold,
        "Circuit breaker initialized"
    );

    let auth_service = Arc::new(AuthService::new(db_pool.clone()));
    let rate_limiter = Arc::new(
        RateLimitService::new(
            &env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        )
        .map_err(|e| format!("Failed to initialize rate limiter: {}", e))?,
    );
    let audit_logger = Arc::new(AuditLoggingService::new(db_pool.clone()));
    let dnp_service = Arc::new(DnpListService::new(db_pool.clone()));
    let user_service = Arc::new(UserService::new(db_pool.clone()));
    tracing::info!("Core services initialized successfully");

    let platform_config = PlatformSyncConfig::from_env();
    tracing::info!(
        service_mode = mode.as_str(),
        "Platform sync configured. Available platforms: {:?}",
        platform_config.available_platforms()
    );

    let identity_resolver =
        CrossPlatformIdentityResolver::new("NoDrakeInTheHouse", "1.0", "admin@nodrake.example.com");

    let mut orchestrator_builder = OrchestratorBuilder::new()
        .with_identity_resolver(identity_resolver)
        .with_db_pool(db_pool.clone());

    orchestrator_builder = orchestrator_builder.with_worker(DeezerSyncWorker::new());
    tracing::info!("Deezer sync worker registered");

    let credits_sync = if let Some(apple_creds) = &platform_config.apple_music {
        orchestrator_builder = orchestrator_builder.with_worker(AppleMusicSyncWorker::new(
            apple_creds.team_id.clone(),
            apple_creds.key_id.clone(),
            apple_creds.private_key.clone(),
            "us".to_string(),
        ));
        tracing::info!("Apple Music sync worker registered");

        Some(Arc::new(CreditsSyncService::new(
            db_pool.clone(),
            apple_creds.team_id.clone(),
            apple_creds.key_id.clone(),
            apple_creds.private_key.clone(),
            "us".to_string(),
        )))
    } else {
        tracing::warn!("Credits sync service not available (no Apple Music credentials)");
        None
    };

    if let Some(spotify_creds) = &platform_config.spotify {
        orchestrator_builder = orchestrator_builder.with_worker(SpotifySyncWorker::new(
            spotify_creds.client_id.clone(),
            spotify_creds.client_secret.clone(),
        ));
        tracing::info!("Spotify sync worker registered");
    }

    let catalog_sync = Arc::new(
        orchestrator_builder
            .build()
            .expect("Failed to build catalog sync orchestrator"),
    );
    tracing::info!("Catalog sync orchestrator initialized");

    #[cfg(feature = "news")]
    let (backfill_orchestrator, news_pipeline) = if mode.should_start_news_pipeline() {
        initialize_full_platform_services(db_pool.clone()).await
    } else {
        tracing::info!(
            service_mode = mode.as_str(),
            "Skipping news pipeline bootstrap for scoped service"
        );
        (None, None)
    };
    #[cfg(not(feature = "news"))]
    let backfill_orchestrator = if mode.should_start_news_pipeline() {
        initialize_full_platform_services(db_pool.clone()).await
    } else {
        tracing::info!(
            service_mode = mode.as_str(),
            "Skipping news pipeline bootstrap for scoped service"
        );
        None
    };

    let token_vault = Arc::new(TokenVaultService::with_pool(db_pool.clone()));
    tracing::info!(
        service_mode = mode.as_str(),
        persistent = token_vault.is_persistent(),
        "Token vault service initialized"
    );

    let notification_service = Arc::new(NotificationService::new(db_pool.clone()));
    tracing::info!("Notification service initialized");

    if mode.should_start_token_refresh() {
        let token_refresh_config = TokenRefreshConfig::from_env();
        let token_refresh_job = TokenRefreshBackgroundJob::with_all_services(
            token_vault.clone(),
            token_refresh_config.clone(),
            Some(notification_service.clone()),
            Some(metrics.clone()),
        );

        tracing::info!(
            service_mode = mode.as_str(),
            interval_hours = token_refresh_config.interval_hours,
            "Starting proactive token refresh background job"
        );

        tokio::spawn(async move {
            if let Err(e) = token_refresh_job.start().await {
                tracing::error!(error = %e, "Token refresh background job failed");
            }
        });
    } else {
        tracing::info!(
            service_mode = mode.as_str(),
            "Skipping token refresh background job for scoped service"
        );
    }

    let apple_music_config = AppleMusicConfig::default();
    let apple_music_service = Arc::new(
        AppleMusicService::new(apple_music_config, token_vault)
            .expect("Failed to create Apple Music service"),
    );
    tracing::info!("Apple Music enforcement service initialized");

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
        #[cfg(feature = "news")]
        news_pipeline,
        apple_music_service,
        circuit_breaker,
        test_user_id: None,
    };

    let app = mode.router(app_state);

    tracing::info!(
        service_mode = mode.as_str(),
        bind_addr = %bind_addr,
        "Server running"
    );

    axum::serve(listener, app).await?;
    Ok(())
}

async fn run_migration_command() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Running database migrations");

    let db_config = DatabaseConfig::default();
    let db_pool = create_pool(db_config)
        .await
        .map_err(|e| format!("Failed to create database pool: {}", e))?;

    run_migrations(&db_pool)
        .await
        .map_err(|e| format!("Migration failed: {}", e))?;

    tracing::info!("Database migrations completed successfully");
    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "music_streaming_blocklist_backend=debug,tower_http=debug".into()
            }),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_current_span(false)
                .with_span_list(true),
        )
        .init();
}

#[cfg(feature = "news")]
async fn initialize_full_platform_services(
    db_pool: sqlx::PgPool,
) -> (
    Option<Arc<BackfillOrchestrator>>,
    Option<Arc<NewsPipelineOrchestrator>>,
) {
    let news_config = NewsPipelineConfig::default();
    let news_pipeline = Arc::new(NewsPipelineOrchestrator::with_database(
        news_config,
        db_pool.clone(),
    ));

    let rss_interval_minutes: i64 = std::env::var("NEWS_RSS_INTERVAL_MINUTES")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .unwrap_or(10);

    let _news_scheduler = ScheduledPipelineRunner::new(news_pipeline.clone())
        .with_rss_interval(Duration::minutes(rss_interval_minutes))
        .with_social_interval(Duration::hours(1))
        .with_full_interval(Duration::hours(6))
        .start();

    tracing::info!(
        rss_interval_minutes = rss_interval_minutes,
        "News pipeline started"
    );

    (
        Some(Arc::new(BackfillOrchestrator::with_news_pipeline(
            db_pool,
            news_pipeline.clone(),
        ))),
        Some(news_pipeline),
    )
}

#[cfg(not(feature = "news"))]
async fn initialize_full_platform_services(
    _db_pool: sqlx::PgPool,
) -> Option<Arc<BackfillOrchestrator>> {
    tracing::info!("News pipeline is disabled in this build");
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// `DUCKDB_PATH` is process-global; serialize tests that mutate it.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_duckdb_path<F: FnOnce()>(value: Option<&str>, f: F) {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = env::var("DUCKDB_PATH").ok();
        match value {
            Some(v) => env::set_var("DUCKDB_PATH", v),
            None => env::remove_var("DUCKDB_PATH"),
        }
        f();
        match prev {
            Some(v) => env::set_var("DUCKDB_PATH", v),
            None => env::remove_var("DUCKDB_PATH"),
        }
    }

    #[test]
    fn analytics_mode_requires_duckdb_path() {
        with_duckdb_path(None, || {
            let err = ensure_duckdb_storage_ready(ServiceMode::Analytics).unwrap_err();
            assert!(err.contains("DUCKDB_PATH is not set"));
        });
    }

    #[test]
    fn monolith_mode_requires_duckdb_path() {
        with_duckdb_path(None, || {
            assert!(ensure_duckdb_storage_ready(ServiceMode::Monolith).is_err());
        });
    }

    #[test]
    fn api_mode_tolerates_missing_duckdb_path() {
        with_duckdb_path(None, || {
            assert!(ensure_duckdb_storage_ready(ServiceMode::Api).is_ok());
        });
    }

    #[test]
    fn writable_path_passes_probe() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = dir.path().join("analytics.duckdb");
        with_duckdb_path(Some(db.to_str().unwrap()), || {
            assert!(ensure_duckdb_storage_ready(ServiceMode::Analytics).is_ok());
        });
    }

    /// Confirm that 0o555 actually blocks writes for this process. Returns
    /// `false` when the caller has DAC override (typically root in CI
    /// containers), so the unwritable-parent test can bail out cleanly.
    #[cfg(unix)]
    fn dac_enforces_readonly_dir(dir: &std::path::Path) -> bool {
        use std::os::unix::fs::PermissionsExt;
        let prev = std::fs::metadata(dir).expect("metadata").permissions();
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o555))
            .expect("chmod 0o555");
        let probe = dir.join(".dac_self_check");
        let blocked = std::fs::write(&probe, b"x").is_err();
        if !blocked {
            let _ = std::fs::remove_file(&probe);
        }
        std::fs::set_permissions(dir, prev).expect("restore permissions");
        blocked
    }

    /// NOD-248: cover the operationally important branch where
    /// `DUCKDB_PATH` is configured but the parent directory is not
    /// writable by the runtime UID. This is the failure mode that
    /// catches a misconfigured PVC mount in production.
    #[cfg(unix)]
    #[test]
    fn unwritable_parent_dir_fails_write_probe() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().expect("tempdir");
        if !dac_enforces_readonly_dir(dir.path()) {
            eprintln!(
                "skipping unwritable_parent_dir_fails_write_probe: \
                 process has DAC override (likely root)"
            );
            return;
        }
        let parent_display = dir.path().display().to_string();
        let db = dir.path().join("analytics.duckdb");
        let db_str = db.to_str().expect("utf8 path").to_string();

        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555))
            .expect("chmod 0o555");

        with_duckdb_path(Some(&db_str), || {
            let err = ensure_duckdb_storage_ready(ServiceMode::Analytics).unwrap_err();
            assert!(
                err.contains(&parent_display),
                "expected parent dir {parent_display} in error message, got: {err}"
            );
            assert!(
                err.contains("not writable"),
                "expected 'not writable' phrasing in error message, got: {err}"
            );
        });

        // Restore write+exec so TempDir's Drop can remove children.
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755))
            .expect("restore 0o755");
    }
}
