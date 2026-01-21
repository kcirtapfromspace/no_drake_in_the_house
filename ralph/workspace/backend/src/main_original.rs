mod models;
mod services;
mod middleware;
mod handlers;

use std::sync::Arc;
use chrono::Utc;
use models::*;
use services::*;
use middleware::*;
use handlers::*;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::{from_fn, from_fn_with_state},
    response::Json,
    routing::{delete, get, post},
    Extension,
    Router,
};
use serde_json::json;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// Application state
#[derive(Clone)]
struct AppState {
    auth_service: Arc<AuthService>,
    entity_service: Arc<EntityResolutionService>,
    token_vault_service: Arc<TokenVaultService>,
    spotify_service: Arc<SpotifyService>,
    spotify_library_service: Arc<SpotifyLibraryService>,
    spotify_enforcement_service: Arc<SpotifyEnforcementService>,
    apple_music_service: Arc<AppleMusicService>,
    apple_music_library_service: Arc<AppleMusicLibraryService>,
    dnp_list_service: Arc<DnpListService>,
    community_list_service: Arc<CommunityListService>,
    rate_limiting_service: Arc<RateLimitingService>,
    job_queue_service: Arc<JobQueueService>,
    audit_service: Arc<AuditService>,
    security_monitor: Arc<SecurityMonitor>,
    vulnerability_scanner: Arc<VulnerabilityScanner>,
    content_moderation_service: Arc<ContentModerationService>,
    monitoring_service: Arc<MonitoringService>,
    health_check_service: Arc<HealthCheckService>,
    analytics_service: Arc<AnalyticsService>,
    db_pool: sqlx::PgPool,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "music_streaming_blocklist_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    println!("Starting Music Streaming Blocklist Manager Backend with Authentication");
    
    // Initialize services
    let entity_service = Arc::new({
        let service = EntityResolutionService::new().with_confidence_threshold(0.7);
        
        // Add some test artists
        let artist1 = Artist::with_external_ids(
            "The Beatles".to_string(),
            ExternalIds::new().with_spotify("4V8Sr092TqfHkfAA5fXXqG".to_string()),
        );
        
        let mut artist2 = Artist::new("Drake".to_string());
        artist2.external_ids.spotify = Some("3TVXtAsR1Inumwj472S9r4".to_string());
        artist2.add_alias(ArtistAlias::new("Aubrey Graham".to_string(), "real_name".to_string(), 0.9));
        
        service.add_artist(artist1).await.unwrap();
        service.add_artist(artist2).await.unwrap();
        
        service
    });

    // Initialize authentication service
    let auth_service = Arc::new(AuthService::new());
    
    // Initialize token vault service
    let token_vault_service = Arc::new(TokenVaultService::new());

    // Initialize Spotify service
    let spotify_config = SpotifyConfig::default();
    let spotify_service = Arc::new(SpotifyService::new(spotify_config, token_vault_service.clone()).unwrap());

    // Initialize Apple Music service
    let apple_music_config = AppleMusicConfig::default();
    let apple_music_service = Arc::new(AppleMusicService::new(apple_music_config, token_vault_service.clone()).unwrap());

    // Initialize database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/music_streaming_blocklist".to_string());
    let db_pool = sqlx::PgPool::connect(&database_url).await
        .expect("Failed to connect to database");

    // Initialize Spotify library service
    let spotify_library_service = Arc::new(SpotifyLibraryService::new(spotify_service.clone()));

    // Initialize Spotify enforcement service
    let spotify_enforcement_service = Arc::new(SpotifyEnforcementService::new(
        spotify_service.clone(),
        spotify_library_service.clone(),
        db_pool.clone(),
    ));

    // Initialize Apple Music library service
    let apple_music_library_service = Arc::new(AppleMusicLibraryService::new(
        apple_music_service.clone(),
        entity_service.clone(),
    ));

    // Initialize DNP list service
    let dnp_list_service = Arc::new(DnpListService::new(
        db_pool.clone(),
        entity_service.clone(),
    ));

    // Initialize community list service
    let community_list_service = Arc::new(CommunityListService::new(
        db_pool.clone(),
        entity_service.clone(),
    ));

    // Initialize rate limiting service
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let rate_limiting_service = Arc::new(RateLimitingService::new(&redis_url)
        .expect("Failed to initialize rate limiting service"));

    // Initialize job queue service
    let job_queue_service = Arc::new(JobQueueService::new(&redis_url, rate_limiting_service.clone())
        .expect("Failed to initialize job queue service"));

    // Register job handlers
    let enforcement_handler = EnforcementJobHandler::new(
        spotify_service.clone(),
        spotify_enforcement_service.clone(),
        rate_limiting_service.clone(),
    );
    job_queue_service.register_handler(enforcement_handler).await
        .expect("Failed to register enforcement job handler");

    let rollback_handler = RollbackJobHandler::new(
        spotify_enforcement_service.clone(),
        rate_limiting_service.clone(),
    );
    job_queue_service.register_handler(rollback_handler).await
        .expect("Failed to register rollback job handler");

    let token_refresh_handler = TokenRefreshJobHandler::new(spotify_service.clone());
    job_queue_service.register_handler(token_refresh_handler).await
        .expect("Failed to register token refresh job handler");

    // Start background workers
    let worker_config = WorkerConfig {
        worker_id: "main_worker".to_string(),
        concurrency: 2,
        job_types: vec![JobType::EnforcementExecution, JobType::BatchRollback, JobType::TokenRefresh],
        poll_interval_ms: 1000,
        max_execution_time_ms: 600000, // 10 minutes
        heartbeat_interval_ms: 30000,  // 30 seconds
    };
    job_queue_service.start_worker(worker_config).await
        .expect("Failed to start background worker");

    // Initialize audit service
    let audit_service = Arc::new(AuditService::new(db_pool.clone()));

    // Initialize security monitor
    let security_monitor = Arc::new(SecurityMonitor::new());

    // Initialize vulnerability scanner
    let vulnerability_scanner = Arc::new(VulnerabilityScanner::new());

    // Initialize content moderation service
    let content_moderation_service = Arc::new(ContentModerationService::new(
        db_pool.clone(),
        audit_service.clone(),
    ));

    // Initialize monitoring service
    let monitoring_service = Arc::new(MonitoringService::new()
        .expect("Failed to initialize monitoring service"));

    // Initialize health check service
    let health_check_service = Arc::new(HealthCheckService::new(monitoring_service.clone()));

    // Initialize analytics service
    let analytics_service = Arc::new(
        AnalyticsService::new(db_pool.clone()).with_monitoring_service(monitoring_service.clone()),
    );

    // Start background monitoring tasks
    monitoring_service.start_background_tasks().await;

    // Initialize default SLOs
    monitoring_service.update_slo(SLO {
        name: "api_availability".to_string(),
        target_percentage: 99.9,
        measurement_window_hours: 24,
        current_percentage: 100.0,
        error_budget_remaining: 100.0,
    }).await;

    monitoring_service.update_slo(SLO {
        name: "enforcement_success_rate".to_string(),
        target_percentage: 99.0,
        measurement_window_hours: 24,
        current_percentage: 100.0,
        error_budget_remaining: 100.0,
    }).await;

    monitoring_service.update_slo(SLO {
        name: "api_latency_p95".to_string(),
        target_percentage: 95.0,
        measurement_window_hours: 1,
        current_percentage: 100.0,
        error_budget_remaining: 100.0,
    }).await;

    let app_state = AppState {
        auth_service: auth_service.clone(),
        entity_service: entity_service.clone(),
        token_vault_service: token_vault_service.clone(),
        spotify_service: spotify_service.clone(),
        spotify_library_service: spotify_library_service.clone(),
        spotify_enforcement_service: spotify_enforcement_service.clone(),
        apple_music_service: apple_music_service.clone(),
        apple_music_library_service: apple_music_library_service.clone(),
        dnp_list_service: dnp_list_service.clone(),
        community_list_service: community_list_service.clone(),
        rate_limiting_service: rate_limiting_service.clone(),
        job_queue_service: job_queue_service.clone(),
        audit_service: audit_service.clone(),
        security_monitor: security_monitor.clone(),
        vulnerability_scanner: vulnerability_scanner.clone(),
        content_moderation_service: content_moderation_service.clone(),
        monitoring_service: monitoring_service.clone(),
        health_check_service: health_check_service.clone(),
        analytics_service: analytics_service.clone(),
        db_pool,
    };

    // Build protected routes
    let protected_routes = Router::new()
        .route("/auth/profile", get(profile_handler))
        .route("/auth/totp/setup", post(totp_setup_handler))
        .route("/auth/totp/enable", post(totp_enable_handler))
        .route("/auth/logout", post(logout_handler))
        .route("/api/v1/artists/resolve", get(resolve_artists_handler))
        .route("/api/v1/artists/search", get(search_artists_handler))
        .route("/api/v1/connections", get(get_connections_handler))
        .route("/api/v1/connections", post(store_connection_handler))
        .route("/api/v1/spotify/auth", get(spotify_auth_handler))
        .route("/api/v1/spotify/callback", post(spotify_callback_handler))
        .route("/api/v1/spotify/connection", get(spotify_connection_handler))
        .route("/api/v1/spotify/connection", delete(spotify_disconnect_handler))
        .route("/api/v1/spotify/health", get(spotify_health_handler))
        .route("/api/v1/spotify/library/scan", post(spotify_scan_library_handler))
        .route("/api/v1/spotify/library/plan", post(spotify_create_plan_handler))
        .route("/api/v1/spotify/enforcement/execute", post(spotify_execute_enforcement_handler))
        .route("/api/v1/spotify/enforcement/progress/:batch_id", get(spotify_enforcement_progress_handler))
        .route("/api/v1/spotify/enforcement/rollback", post(spotify_rollback_enforcement_handler))
        // Apple Music endpoints
        .route("/api/v1/apple-music/connection", post(apple_music_connect_handler))
        .route("/api/v1/apple-music/connection", get(apple_music_connection_handler))
        .route("/api/v1/apple-music/connection", delete(apple_music_disconnect_handler))
        .route("/api/v1/apple-music/health", get(apple_music_health_handler))
        .route("/api/v1/apple-music/capabilities", get(apple_music_capabilities_handler))
        .route("/api/v1/apple-music/library/scan", post(apple_music_scan_library_handler))
        .route("/api/v1/apple-music/library/analyze", post(apple_music_analyze_library_handler))
        .route("/api/v1/apple-music/library/statistics", get(apple_music_library_statistics_handler))
        .route("/api/v1/apple-music/search", get(apple_music_search_handler))
        // DNP list management endpoints
        .route("/api/v1/dnp/list", get(get_dnp_list_handler))
        .route("/api/v1/dnp/artists", post(add_artist_to_dnp_handler))
        .route("/api/v1/dnp/artists/:artist_id", delete(remove_artist_from_dnp_handler))
        .route("/api/v1/dnp/artists/:artist_id", put(update_dnp_entry_handler))
        .route("/api/v1/dnp/search", get(search_artists_for_dnp_handler))
        .route("/api/v1/dnp/import", post(bulk_import_dnp_handler))
        .route("/api/v1/dnp/export", get(export_dnp_list_handler))
        // Community list endpoints
        .route("/api/v1/community/lists", get(browse_community_lists_handler))
        .route("/api/v1/community/lists", post(create_community_list_handler))
        .route("/api/v1/community/lists/:list_id", get(get_community_list_handler))
        .route("/api/v1/community/lists/:list_id/artists", get(get_community_list_with_artists_handler))
        .route("/api/v1/community/lists/:list_id/artists", post(add_artist_to_community_list_handler))
        .route("/api/v1/community/lists/:list_id/artists/:artist_id", delete(remove_artist_from_community_list_handler))
        .route("/api/v1/community/lists/:list_id/subscribe", post(subscribe_to_community_list_handler))
        .route("/api/v1/community/lists/:list_id/unsubscribe", post(unsubscribe_from_community_list_handler))
        .route("/api/v1/community/lists/:list_id/subscription", put(update_subscription_handler))
        .route("/api/v1/community/lists/:list_id/impact", get(get_subscription_impact_preview_handler))
        .route("/api/v1/community/subscriptions", get(get_user_subscriptions_handler))
        // Job management endpoints
        .route("/api/v1/jobs", get(get_user_jobs_handler))
        .route("/api/v1/jobs/:job_id", get(get_job_status_handler))
        .route("/api/v1/jobs/:job_id/retry", post(retry_job_handler))
        .route("/api/v1/jobs/queue", post(enqueue_job_handler))
        .route("/api/v1/jobs/workers/stats", get(get_worker_stats_handler))
        // Rate limiting endpoints
        .route("/api/v1/rate-limits/:provider/status", get(get_rate_limit_status_handler))
        // Audit and compliance endpoints
        .route("/api/v1/audit/logs", get(get_audit_logs_handler))
        .route("/api/v1/audit/security-events", get(get_security_events_handler))
        .route("/api/v1/audit/security-stats", get(get_security_stats_handler))
        .route("/api/v1/compliance/data-export", post(request_data_export_handler))
        .route("/api/v1/compliance/data-export/:request_id", get(get_data_export_status_handler))
        .route("/api/v1/compliance/data-export/:request_id/download", get(download_data_export_handler))
        .route("/api/v1/compliance/data-deletion", post(request_data_deletion_handler))
        .route("/api/v1/security/vulnerability-scan", post(start_vulnerability_scan_handler))
        .route("/api/v1/security/vulnerability-scan/results", get(get_vulnerability_scan_results_handler))
        .route("/api/v1/security/vulnerability-summary", get(get_vulnerability_summary_handler))
        // Content moderation endpoints
        .route("/api/v1/moderation/queue", get(get_moderation_queue_handler))
        .route("/api/v1/moderation/queue/:entry_id/review", post(review_content_handler))
        .route("/api/v1/moderation/appeals", post(submit_appeal_handler))
        .route("/api/v1/moderation/appeals/:appeal_id/review", post(review_appeal_handler))
        .route("/api/v1/moderation/stats", get(get_moderation_stats_handler))
        .route("/api/v1/moderation/content/validate", post(validate_content_handler))
        // Monitoring and observability endpoints
        .route("/api/v1/monitoring/health", get(health_handler))
        .route("/api/v1/monitoring/alerts", get(alerts_handler))
        .route("/api/v1/monitoring/alerts", post(create_alert_handler))
        .route("/api/v1/monitoring/slos", get(slos_handler))
        .route("/api/v1/monitoring/system", get(system_metrics_handler))
        .route("/api/v1/monitoring/dashboard", get(dashboard_handler))
        .route("/api/v1/monitoring/test-alert/:severity", post(test_alert_handler))
        // Analytics and reporting endpoints
        .route("/api/v1/analytics/enforcement/success", get(get_enforcement_success_report))
        .route("/api/v1/analytics/dnp/effectiveness", get(get_dnp_effectiveness_report))
        .route("/api/v1/analytics/community-lists/:list_id/impact", get(get_community_list_impact_report))
        .route("/api/v1/analytics/system/performance", get(get_system_performance_dashboard))
        .route("/api/v1/analytics/summary", get(get_user_analytics_summary))
        .route("/api/v1/analytics/time-series", get(get_analytics_time_series))
        .route("/api/v1/analytics/comparative", get(get_comparative_analytics))
        .route("/api/v1/analytics/export", get(export_analytics_data))
        .route_layer(from_fn_with_state(
            auth_service.clone(),
            auth_middleware,
        ));

    // Build the application router
    let app = Router::new()
        // Public routes
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        .route("/auth/register", post(register_handler))
        .route("/auth/login", post(login_handler))
        .route("/auth/refresh", post(refresh_token_handler))
        // Merge protected routes
        .merge(protected_routes)
        // Add global middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(from_fn_with_state(
                    monitoring_service.clone(),
                    crate::middleware::tracing::tracing_middleware,
                ))
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                )
                .layer(from_fn(security_headers_middleware))
                .layer(from_fn(auth_rate_limit_middleware)),
        )
        .with_state(app_state);

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://0.0.0.0:3000");
    println!("Health check: http://0.0.0.0:3000/health");
    println!("Authentication endpoints:");
    println!("  POST /auth/register");
    println!("  POST /auth/login");
    println!("  POST /auth/refresh");
    println!("  GET  /auth/profile (requires auth)");
    println!("  POST /auth/totp/setup (requires auth)");
    println!("  POST /auth/totp/enable (requires auth)");
    println!("  POST /auth/logout (requires auth)");
    println!("API endpoints (require auth):");
    println!("  GET  /api/v1/artists/resolve");
    println!("  GET  /api/v1/artists/search");
    println!("  GET  /api/v1/connections");
    println!("  POST /api/v1/connections");
    println!("Spotify endpoints (require auth):");
    println!("  GET    /api/v1/spotify/auth");
    println!("  POST   /api/v1/spotify/callback");
    println!("  GET    /api/v1/spotify/connection");
    println!("  DELETE /api/v1/spotify/connection");
    println!("  GET    /api/v1/spotify/health");
    println!("  POST   /api/v1/spotify/library/scan");
    println!("  POST   /api/v1/spotify/library/plan");
    println!("  POST   /api/v1/spotify/enforcement/execute");
    println!("  GET    /api/v1/spotify/enforcement/progress/:batch_id");
    println!("  POST   /api/v1/spotify/enforcement/rollback");
    println!("DNP List endpoints (require auth):");
    println!("  GET    /api/v1/dnp/list");
    println!("  POST   /api/v1/dnp/artists");
    println!("  DELETE /api/v1/dnp/artists/:artist_id");
    println!("  PUT    /api/v1/dnp/artists/:artist_id");
    println!("  GET    /api/v1/dnp/search");
    println!("  POST   /api/v1/dnp/import");
    println!("  GET    /api/v1/dnp/export");
    println!("Community List endpoints (require auth):");
    println!("  GET    /api/v1/community/lists");
    println!("  POST   /api/v1/community/lists");
    println!("  GET    /api/v1/community/lists/:list_id");
    println!("  GET    /api/v1/community/lists/:list_id/artists");
    println!("  POST   /api/v1/community/lists/:list_id/artists");
    println!("  DELETE /api/v1/community/lists/:list_id/artists/:artist_id");
    println!("  POST   /api/v1/community/lists/:list_id/subscribe");
    println!("  POST   /api/v1/community/lists/:list_id/unsubscribe");
    println!("  PUT    /api/v1/community/lists/:list_id/subscription");
    println!("  GET    /api/v1/community/lists/:list_id/impact");
    println!("  GET    /api/v1/community/subscriptions");
    println!("Job Management endpoints (require auth):");
    println!("  GET    /api/v1/jobs");
    println!("  GET    /api/v1/jobs/:job_id");
    println!("  POST   /api/v1/jobs/:job_id/retry");
    println!("  POST   /api/v1/jobs/queue");
    println!("  GET    /api/v1/jobs/workers/stats");
    println!("Rate Limiting endpoints (require auth):");
    println!("  GET    /api/v1/rate-limits/:provider/status");
    println!("Apple Music endpoints (require auth):");
    println!("  POST   /api/v1/apple-music/connection");
    println!("  GET    /api/v1/apple-music/connection");
    println!("  DELETE /api/v1/apple-music/connection");
    println!("  GET    /api/v1/apple-music/health");
    println!("  GET    /api/v1/apple-music/capabilities");
    println!("  POST   /api/v1/apple-music/library/scan");
    println!("  POST   /api/v1/apple-music/library/analyze");
    println!("  GET    /api/v1/apple-music/library/statistics");
    println!("  GET    /api/v1/apple-music/search");
    println!("Monitoring and Observability endpoints:");
    println!("  GET    /metrics (public - Prometheus metrics)");
    println!("  GET    /api/v1/monitoring/health (requires auth)");
    println!("  GET    /api/v1/monitoring/alerts (requires auth)");
    println!("  POST   /api/v1/monitoring/alerts (requires auth)");
    println!("  GET    /api/v1/monitoring/slos (requires auth)");
    println!("  GET    /api/v1/monitoring/system (requires auth)");
    println!("  GET    /api/v1/monitoring/dashboard (requires auth)");
    println!("  POST   /api/v1/monitoring/test-alert/:severity (requires auth)");
    println!("Analytics and Reporting endpoints (require auth):");
    println!("  GET    /api/v1/analytics/enforcement/success");
    println!("  GET    /api/v1/analytics/dnp/effectiveness");
    println!("  GET    /api/v1/analytics/community-lists/:list_id/impact");
    println!("  GET    /api/v1/analytics/system/performance (admin only)");
    println!("  GET    /api/v1/analytics/summary");
    println!("  GET    /api/v1/analytics/time-series");
    println!("  GET    /api/v1/analytics/comparative");
    println!("  GET    /api/v1/analytics/export");

    axum::serve(listener, app).await.unwrap();
}

// Apple Music handlers

/// Connect Apple Music account
async fn apple_music_connect_handler(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(request): Json<AppleMusicConnectionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state
        .apple_music_service
        .create_connection(user.id, request.user_token, request.music_user_token)
        .await
    {
        Ok(connection) => Ok(Json(json!({
            "success": true,
            "connection": {
                "id": connection.id,
                "provider": connection.provider,
                "status": connection.status,
                "created_at": connection.created_at
            }
        }))),
        Err(e) => {
            tracing::error!("Failed to connect Apple Music: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Get Apple Music connection status
async fn apple_music_connection_handler(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.apple_music_service.get_user_connection(user.id).await {
        Ok(Some(connection)) => Ok(Json(json!({
            "connected": true,
            "connection": {
                "id": connection.id,
                "provider": connection.provider,
                "status": connection.status,
                "created_at": connection.created_at,
                "last_health_check": connection.last_health_check
            }
        }))),
        Ok(None) => Ok(Json(json!({
            "connected": false
        }))),
        Err(e) => {
            tracing::error!("Failed to get Apple Music connection: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Disconnect Apple Music account
async fn apple_music_disconnect_handler(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.apple_music_service.disconnect_user(user.id).await {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "message": "Apple Music account disconnected successfully"
        }))),
        Err(e) => {
            tracing::error!("Failed to disconnect Apple Music: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Check Apple Music connection health
async fn apple_music_health_handler(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.apple_music_service.get_user_connection(user.id).await {
        Ok(Some(connection)) => {
            match state.apple_music_service.check_token_health(&connection).await {
                Ok(health_check) => Ok(Json(json!({
                    "healthy": health_check.is_valid,
                    "checked_at": health_check.checked_at,
                    "error_message": health_check.error_message
                }))),
                Err(e) => {
                    tracing::error!("Failed to check Apple Music health: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Ok(None) => Ok(Json(json!({
            "healthy": false,
            "error_message": "No Apple Music connection found"
        }))),
        Err(e) => {
            tracing::error!("Failed to get Apple Music connection for health check: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get Apple Music capabilities
async fn apple_music_capabilities_handler(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let capabilities = state.apple_music_service.get_capabilities();
    Ok(Json(json!({
        "capabilities": capabilities,
        "limitations": [
            "Apple Music API has limited write capabilities",
            "Library modifications must be done manually",
            "Playlist modifications are not supported via API",
            "Content analysis and reporting only"
        ]
    })))
}

/// Scan Apple Music library
async fn apple_music_scan_library_handler(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.apple_music_service.get_user_connection(user.id).await {
        Ok(Some(connection)) => {
            match state.apple_music_service.scan_library(&connection).await {
                Ok(library) => Ok(Json(json!({
                    "success": true,
                    "library": {
                        "total_tracks": library.library_tracks.len(),
                        "total_albums": library.library_albums.len(),
                        "total_playlists": library.library_playlists.len(),
                        "scanned_at": library.scanned_at
                    }
                }))),
                Err(e) => {
                    tracing::error!("Failed to scan Apple Music library: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Ok(None) => Err(StatusCode::BAD_REQUEST),
        Err(e) => {
            tracing::error!("Failed to get Apple Music connection: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Analyze Apple Music library for blocked content
async fn apple_music_analyze_library_handler(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(request): Json<AppleMusicAnalysisRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.apple_music_service.get_user_connection(user.id).await {
        Ok(Some(connection)) => {
            // Get user's DNP list
            match state.dnp_list_service.get_user_dnp_list(user.id).await {
                Ok(dnp_list) => {
                    let options = request.options.unwrap_or_default();
                    match state
                        .apple_music_library_service
                        .analyze_library(&connection, &dnp_list.artists, options)
                        .await
                    {
                        Ok(analysis) => {
                            let instructions = state
                                .apple_music_library_service
                                .generate_manual_action_instructions(&analysis);
                            
                            Ok(Json(json!({
                                "success": true,
                                "analysis": analysis,
                                "manual_instructions": instructions
                            })))
                        }
                        Err(e) => {
                            tracing::error!("Failed to analyze Apple Music library: {}", e);
                            Err(StatusCode::INTERNAL_SERVER_ERROR)
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to get user DNP list: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Ok(None) => Err(StatusCode::BAD_REQUEST),
        Err(e) => {
            tracing::error!("Failed to get Apple Music connection: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get Apple Music library statistics
async fn apple_music_library_statistics_handler(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.apple_music_service.get_user_connection(user.id).await {
        Ok(Some(connection)) => {
            match state
                .apple_music_library_service
                .get_library_statistics(&connection)
                .await
            {
                Ok(stats) => Ok(Json(json!({
                    "success": true,
                    "statistics": stats
                }))),
                Err(e) => {
                    tracing::error!("Failed to get Apple Music library statistics: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Ok(None) => Err(StatusCode::BAD_REQUEST),
        Err(e) => {
            tracing::error!("Failed to get Apple Music connection: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Search Apple Music catalog
async fn apple_music_search_handler(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.apple_music_service.get_user_connection(user.id).await {
        Ok(Some(connection)) => {
            let term = params.get("term").cloned().unwrap_or_default();
            let types = params
                .get("types")
                .map(|t| t.split(',').map(|s| s.to_string()).collect())
                .unwrap_or_else(|| vec!["songs".to_string(), "artists".to_string()]);
            let limit = params
                .get("limit")
                .and_then(|l| l.parse().ok());
            let offset = params
                .get("offset")
                .and_then(|o| o.parse().ok());

            let search_request = AppleMusicSearchRequest {
                term,
                types,
                limit,
                offset,
            };

            match state
                .apple_music_service
                .search(&connection, search_request)
                .await
            {
                Ok(search_response) => Ok(Json(json!({
                    "success": true,
                    "results": search_response
                }))),
                Err(e) => {
                    tracing::error!("Failed to search Apple Music: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Ok(None) => Err(StatusCode::BAD_REQUEST),
        Err(e) => {
            tracing::error!("Failed to get Apple Music connection: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Request/Response types for Apple Music handlers

#[derive(serde::Deserialize)]
struct AppleMusicConnectionRequest {
    user_token: String,
    music_user_token: Option<String>,
}

#[derive(serde::Deserialize)]
struct AppleMusicAnalysisRequest {
    options: Option<AppleMusicEnforcementOptions>,
}

// DNP List handler functions

async fn get_dnp_list_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.dnp_list_service.get_user_dnp_list(user.id).await {
        Ok(dnp_list) => Ok(Json(json!({
            "success": true,
            "data": dnp_list,
            "message": "DNP list retrieved successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get DNP list: {}", e)
            })),
        )),
    }
}

async fn add_artist_to_dnp_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    Json(request): Json<AddArtistToDnpRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.dnp_list_service.add_artist_to_dnp_list(user.id, request).await {
        Ok(entry) => Ok(Json(json!({
            "success": true,
            "data": entry,
            "message": "Artist added to DNP list successfully"
        }))),
        Err(e) => {
            let status = if e.to_string().contains("already in your DNP list") {
                StatusCode::CONFLICT
            } else if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to add artist to DNP list: {}", e)
                })),
            ))
        }
    }
}

async fn remove_artist_from_dnp_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    axum::extract::Path(artist_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.dnp_list_service.remove_artist_from_dnp_list(user.id, artist_id).await {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "data": null,
            "message": "Artist removed from DNP list successfully"
        }))),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to remove artist from DNP list: {}", e)
                })),
            ))
        }
    }
}

async fn update_dnp_entry_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    axum::extract::Path(artist_id): axum::extract::Path<Uuid>,
    Json(request): Json<UpdateDnpEntryRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.dnp_list_service.update_dnp_entry(user.id, artist_id, request).await {
        Ok(entry) => Ok(Json(json!({
            "success": true,
            "data": entry,
            "message": "DNP entry updated successfully"
        }))),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to update DNP entry: {}", e)
                })),
            ))
        }
    }
}

#[derive(serde::Deserialize)]
struct SearchArtistsQuery {
    q: String,
    limit: Option<usize>,
}

async fn search_artists_for_dnp_handler(
    State(app_state): State<AppState>,
    Extension(_user): Extension<User>,
    axum::extract::Query(query): axum::extract::Query<SearchArtistsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.dnp_list_service.search_artists(&query.q, query.limit).await {
        Ok(results) => Ok(Json(json!({
            "success": true,
            "data": results,
            "message": "Artist search completed successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Artist search failed: {}", e)
            })),
        )),
    }
}

async fn bulk_import_dnp_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    Json(request): Json<BulkImportRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.dnp_list_service.bulk_import(user.id, request).await {
        Ok(result) => Ok(Json(json!({
            "success": true,
            "data": result,
            "message": "Bulk import completed"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Bulk import failed: {}", e)
            })),
        )),
    }
}

#[derive(serde::Deserialize)]
struct ExportQuery {
    format: Option<String>,
}

async fn export_dnp_list_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    axum::extract::Query(query): axum::extract::Query<ExportQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let format = match query.format.as_deref() {
        Some("csv") => ImportFormat::Csv,
        Some("json") | None => ImportFormat::Json,
        Some(other) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Unsupported export format: {}", other)
                })),
            ));
        }
    };

    match app_state.dnp_list_service.export_dnp_list(user.id, format).await {
        Ok(data) => Ok(Json(json!({
            "success": true,
            "data": {
                "content": data,
                "format": match format {
                    ImportFormat::Csv => "csv",
                    ImportFormat::Json => "json",
                }
            },
            "message": "DNP list exported successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Export failed: {}", e)
            })),
        )),
    }
}

// Handler functions

async fn health_handler() -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "data": {
            "status": "healthy",
            "version": env!("CARGO_PKG_VERSION"),
            "timestamp": Utc::now().to_rfc3339()
        },
        "message": "Service is healthy"
    }))
}

async fn register_handler(
    State(app_state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.auth_service.register_user(request).await {
        Ok(user) => Ok(Json(json!({
            "success": true,
            "data": {
                "id": user.id.to_string(),
                "email": user.email,
                "email_verified": user.email_verified,
                "totp_enabled": user.totp_enabled,
                "created_at": user.created_at.to_rfc3339()
            },
            "message": "User registered successfully"
        }))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Registration failed: {}", e)
            })),
        )),
    }
}

async fn login_handler(
    State(app_state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.auth_service.login_user(request).await {
        Ok(token_pair) => Ok(Json(json!({
            "success": true,
            "data": token_pair,
            "message": "Login successful"
        }))),
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Login failed: {}", e)
            })),
        )),
    }
}

async fn refresh_token_handler(
    State(app_state): State<AppState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.auth_service.refresh_token(&request.refresh_token).await {
        Ok(token_pair) => Ok(Json(json!({
            "success": true,
            "data": token_pair,
            "message": "Token refreshed successfully"
        }))),
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Token refresh failed: {}", e)
            })),
        )),
    }
}

async fn profile_handler(
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {

    Ok(Json(json!({
        "success": true,
        "data": {
            "id": user.id.to_string(),
            "email": user.email,
            "email_verified": user.email_verified,
            "totp_enabled": user.totp_enabled,
            "created_at": user.created_at.to_rfc3339(),
            "last_login": user.last_login.map(|dt| dt.to_rfc3339())
        },
        "message": "Profile retrieved successfully"
    })))
}

async fn totp_setup_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {

    match app_state.auth_service.setup_totp(user.id).await {
        Ok(setup_response) => Ok(Json(json!({
            "success": true,
            "data": setup_response,
            "message": "TOTP setup initiated"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("TOTP setup failed: {}", e)
            })),
        )),
    }
}

async fn totp_enable_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    Json(totp_request): Json<TotpSetupRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {

    match app_state.auth_service.enable_totp(user.id, &totp_request.totp_code).await {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "data": null,
            "message": "TOTP enabled successfully"
        }))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("TOTP enable failed: {}", e)
            })),
        )),
    }
}

async fn logout_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {

    match app_state.auth_service.revoke_all_sessions(user.id).await {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "data": null,
            "message": "Logged out successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Logout failed: {}", e)
            })),
        )),
    }
}

async fn resolve_artists_handler(
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let query = ArtistSearchQuery::new("Beatles".to_string()).with_limit(5);
    
    match app_state.entity_service.resolve_artist(&query).await {
        Ok(results) => Ok(Json(json!({
            "success": true,
            "data": results,
            "message": "Artists resolved successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Resolution failed: {}", e)
            })),
        )),
    }
}

async fn search_artists_handler() -> Json<serde_json::Value> {
    let artists = vec!["Artist 1".to_string(), "Artist 2".to_string()];
    Json(json!({
        "success": true,
        "data": artists,
        "message": "Artists retrieved successfully"
    }))
}

async fn get_connections_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {

    let connections = app_state.token_vault_service.get_user_connections(user.id).await;
    
    Ok(Json(json!({
        "success": true,
        "data": connections,
        "message": "Connections retrieved successfully"
    })))
}

async fn store_connection_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    Json(store_request): Json<StoreTokenRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {

    // Ensure the request is for the authenticated user
    if store_request.user_id != user.id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "success": false,
                "data": null,
                "message": "Cannot store tokens for another user"
            })),
        ));
    }

    match app_state.token_vault_service.store_token(store_request).await {
        Ok(connection) => Ok(Json(json!({
            "success": true,
            "data": connection,
            "message": "Connection stored successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to store connection: {}", e)
            })),
        )),
    }
}

// Spotify handler functions

async fn spotify_auth_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.spotify_service.get_auth_url().await {
        Ok(auth_response) => Ok(Json(json!({
            "success": true,
            "data": auth_response,
            "message": "Spotify authorization URL generated"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to generate auth URL: {}", e)
            })),
        )),
    }
}

async fn spotify_callback_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    Json(callback_request): Json<SpotifyCallbackRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.spotify_service.handle_callback(user.id, callback_request).await {
        Ok(connection) => Ok(Json(json!({
            "success": true,
            "data": {
                "connection_id": connection.id,
                "provider": connection.provider,
                "provider_user_id": connection.provider_user_id,
                "scopes": connection.scopes,
                "status": connection.status,
                "created_at": connection.created_at.to_rfc3339()
            },
            "message": "Spotify connection established successfully"
        }))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Spotify connection failed: {}", e)
            })),
        )),
    }
}

async fn spotify_connection_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.spotify_service.get_user_connection(user.id).await {
        Ok(Some(connection)) => Ok(Json(json!({
            "success": true,
            "data": {
                "connection_id": connection.id,
                "provider": connection.provider,
                "provider_user_id": connection.provider_user_id,
                "scopes": connection.scopes,
                "status": connection.status,
                "expires_at": connection.expires_at.map(|dt| dt.to_rfc3339()),
                "last_health_check": connection.last_health_check.map(|dt| dt.to_rfc3339()),
                "created_at": connection.created_at.to_rfc3339()
            },
            "message": "Spotify connection retrieved"
        }))),
        Ok(None) => Ok(Json(json!({
            "success": true,
            "data": null,
            "message": "No Spotify connection found"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get connection: {}", e)
            })),
        )),
    }
}

async fn spotify_disconnect_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.spotify_service.disconnect_user(user.id).await {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "data": null,
            "message": "Spotify connection disconnected successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to disconnect: {}", e)
            })),
        )),
    }
}

async fn spotify_health_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.spotify_service.get_user_connection(user.id).await {
        Ok(Some(connection)) => {
            match app_state.spotify_service.check_token_health(&connection).await {
                Ok(health_check) => Ok(Json(json!({
                    "success": true,
                    "data": health_check,
                    "message": "Token health check completed"
                }))),
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "success": false,
                        "data": null,
                        "message": format!("Health check failed: {}", e)
                    })),
                )),
            }
        },
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "data": null,
                "message": "No Spotify connection found"
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get connection: {}", e)
            })),
        )),
    }
}

// Spotify library analysis handlers

async fn spotify_scan_library_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Get user's Spotify connection
    match app_state.spotify_service.get_user_connection(user.id).await {
        Ok(Some(connection)) => {
            match app_state.spotify_library_service.scan_library(&connection).await {
                Ok(library) => Ok(Json(json!({
                    "success": true,
                    "data": {
                        "user_id": library.user_id,
                        "spotify_user_id": library.spotify_user_id,
                        "liked_songs_count": library.liked_songs.len(),
                        "playlists_count": library.playlists.len(),
                        "followed_artists_count": library.followed_artists.len(),
                        "saved_albums_count": library.saved_albums.len(),
                        "scanned_at": library.scanned_at.to_rfc3339(),
                        "summary": {
                            "total_tracks_in_playlists": library.playlists.iter()
                                .map(|p| p.tracks.total)
                                .sum::<u32>(),
                            "total_library_items": library.liked_songs.len() + 
                                library.playlists.len() + 
                                library.followed_artists.len() + 
                                library.saved_albums.len()
                        }
                    },
                    "message": "Library scanned successfully"
                }))),
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "success": false,
                        "data": null,
                        "message": format!("Library scan failed: {}", e)
                    })),
                )),
            }
        },
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "data": null,
                "message": "No Spotify connection found. Please connect your Spotify account first."
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get connection: {}", e)
            })),
        )),
    }
}

#[derive(serde::Deserialize)]
struct CreateEnforcementPlanRequest {
    dnp_artist_ids: Vec<uuid::Uuid>,
    options: Option<EnforcementOptions>,
}

async fn spotify_create_plan_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    Json(request): Json<CreateEnforcementPlanRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Get user's Spotify connection
    match app_state.spotify_service.get_user_connection(user.id).await {
        Ok(Some(connection)) => {
            // Get DNP artists from entity service
            let mut dnp_artists = Vec::new();
            for artist_id in request.dnp_artist_ids {
                match app_state.entity_service.get_artist_by_id(artist_id).await {
                    Ok(Some(artist)) => dnp_artists.push(artist),
                    Ok(None) => {
                        return Err((
                            StatusCode::NOT_FOUND,
                            Json(json!({
                                "success": false,
                                "data": null,
                                "message": format!("Artist with ID {} not found", artist_id)
                            })),
                        ));
                    },
                    Err(e) => {
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({
                                "success": false,
                                "data": null,
                                "message": format!("Failed to get artist {}: {}", artist_id, e)
                            })),
                        ));
                    }
                }
            }

            let options = request.options.unwrap_or_default();

            match app_state.spotify_library_service.create_enforcement_plan(&connection, dnp_artists, options).await {
                Ok(plan) => Ok(Json(json!({
                    "success": true,
                    "data": {
                        "plan_id": plan.id,
                        "user_id": plan.user_id,
                        "provider": plan.provider,
                        "options": plan.options,
                        "dnp_artists_count": plan.dnp_artists.len(),
                        "impact": plan.impact,
                        "actions_count": plan.actions.len(),
                        "estimated_duration_seconds": plan.estimated_duration_seconds,
                        "created_at": plan.created_at.to_rfc3339(),
                        "idempotency_key": plan.idempotency_key,
                        "actions_summary": {
                            "remove_liked_songs": plan.get_actions_by_type(ActionType::RemoveLikedSong).len(),
                            "remove_playlist_tracks": plan.get_actions_by_type(ActionType::RemovePlaylistTrack).len(),
                            "unfollow_artists": plan.get_actions_by_type(ActionType::UnfollowArtist).len(),
                            "remove_saved_albums": plan.get_actions_by_type(ActionType::RemoveSavedAlbum).len(),
                        }
                    },
                    "message": "Enforcement plan created successfully"
                }))),
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "success": false,
                        "data": null,
                        "message": format!("Failed to create enforcement plan: {}", e)
                    })),
                )),
            }
        },
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "data": null,
                "message": "No Spotify connection found. Please connect your Spotify account first."
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get connection: {}", e)
            })),
        )),
    }
}

// Spotify enforcement handlers

#[derive(serde::Deserialize)]
struct ExecuteEnforcementRequest {
    plan_id: uuid::Uuid,
    idempotency_key: Option<String>,
    execute_immediately: Option<bool>,
    batch_size: Option<u32>,
    rate_limit_buffer_ms: Option<u64>,
}

async fn spotify_execute_enforcement_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    Json(request): Json<ExecuteEnforcementRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Get user's Spotify connection
    match app_state.spotify_service.get_user_connection(user.id).await {
        Ok(Some(connection)) => {
            // For this implementation, we'll need to store and retrieve enforcement plans
            // For now, we'll create a mock plan based on the request
            // In a real implementation, you'd retrieve the plan from storage
            
            // Create a mock enforcement plan for demonstration
            let mock_plan = EnforcementPlan::new(
                user.id,
                "spotify".to_string(),
                EnforcementOptions::default(),
                vec![], // Empty DNP artists for now
            );

            let execute_request = ExecuteBatchRequest {
                plan_id: request.plan_id,
                idempotency_key: request.idempotency_key,
                execute_immediately: request.execute_immediately.unwrap_or(true),
                batch_size: request.batch_size,
                rate_limit_buffer_ms: request.rate_limit_buffer_ms,
            };

            match app_state.spotify_enforcement_service
                .execute_enforcement_batch(&connection, &mock_plan, execute_request).await {
                Ok(result) => Ok(Json(json!({
                    "success": true,
                    "data": {
                        "batch_id": result.batch_id,
                        "status": result.status,
                        "summary": result.summary,
                        "completed_actions_count": result.completed_actions.len(),
                        "failed_actions_count": result.failed_actions.len(),
                        "rollback_available": result.rollback_info.is_some()
                    },
                    "message": "Enforcement batch executed successfully"
                }))),
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "success": false,
                        "data": null,
                        "message": format!("Enforcement execution failed: {}", e)
                    })),
                )),
            }
        },
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "data": null,
                "message": "No Spotify connection found. Please connect your Spotify account first."
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get connection: {}", e)
            })),
        )),
    }
}

async fn spotify_enforcement_progress_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    axum::extract::Path(batch_id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.spotify_enforcement_service.get_batch_progress(&batch_id).await {
        Ok(progress) => Ok(Json(json!({
            "success": true,
            "data": progress,
            "message": "Batch progress retrieved successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get batch progress: {}", e)
            })),
        )),
    }
}

async fn spotify_rollback_enforcement_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    Json(request): Json<RollbackBatchRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Get user's Spotify connection
    match app_state.spotify_service.get_user_connection(user.id).await {
        Ok(Some(connection)) => {
            match app_state.spotify_enforcement_service.rollback_batch(&connection, request).await {
                Ok(rollback_info) => Ok(Json(json!({
                    "success": true,
                    "data": {
                        "rollback_batch_id": rollback_info.rollback_batch_id,
                        "rollback_actions_count": rollback_info.rollback_actions.len(),
                        "rollback_summary": rollback_info.rollback_summary,
                        "partial_rollback": rollback_info.partial_rollback,
                        "rollback_errors_count": rollback_info.rollback_errors.len()
                    },
                    "message": "Enforcement batch rolled back successfully"
                }))),
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "success": false,
                        "data": null,
                        "message": format!("Rollback failed: {}", e)
                    })),
                )),
            }
        },
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "data": null,
                "message": "No Spotify connection found. Please connect your Spotify account first."
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get connection: {}", e)
            })),
        )),
    }
}//
 Community List handler functions

async fn browse_community_lists_handler(
    State(app_state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<CommunityListQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.community_list_service.browse_community_lists(query).await {
        Ok(directory) => Ok(Json(json!({
            "success": true,
            "data": directory,
            "message": "Community lists retrieved successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to browse community lists: {}", e)
            })),
        )),
    }
}

async fn create_community_list_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    Json(request): Json<CreateCommunityListRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.community_list_service.create_community_list(user.id, request).await {
        Ok(list) => Ok(Json(json!({
            "success": true,
            "data": list,
            "message": "Community list created successfully"
        }))),
        Err(e) => {
            let status = if e.to_string().contains("neutral") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to create community list: {}", e)
                })),
            ))
        }
    }
}

async fn get_community_list_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    axum::extract::Path(list_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.community_list_service.get_community_list_by_id(list_id, Some(user.id)).await {
        Ok(list) => Ok(Json(json!({
            "success": true,
            "data": list,
            "message": "Community list retrieved successfully"
        }))),
        Err(e) => {
            let status = if e.to_string().contains("not found") || e.to_string().contains("not accessible") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to get community list: {}", e)
                })),
            ))
        }
    }
}

async fn get_community_list_with_artists_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    axum::extract::Path(list_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.community_list_service.get_community_list_with_artists(list_id, Some(user.id)).await {
        Ok(list_with_artists) => Ok(Json(json!({
            "success": true,
            "data": list_with_artists,
            "message": "Community list with artists retrieved successfully"
        }))),
        Err(e) => {
            let status = if e.to_string().contains("not found") || e.to_string().contains("not accessible") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to get community list with artists: {}", e)
                })),
            ))
        }
    }
}

async fn add_artist_to_community_list_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    axum::extract::Path(list_id): axum::extract::Path<Uuid>,
    Json(request): Json<AddArtistToCommunityListRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.community_list_service.add_artist_to_community_list(user.id, list_id, request).await {
        Ok(artist_entry) => Ok(Json(json!({
            "success": true,
            "data": artist_entry,
            "message": "Artist added to community list successfully"
        }))),
        Err(e) => {
            let status = if e.to_string().contains("not authorized") {
                StatusCode::FORBIDDEN
            } else if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("already in") {
                StatusCode::CONFLICT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to add artist to community list: {}", e)
                })),
            ))
        }
    }
}

async fn remove_artist_from_community_list_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    axum::extract::Path((list_id, artist_id)): axum::extract::Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.community_list_service.remove_artist_from_community_list(user.id, list_id, artist_id).await {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "data": null,
            "message": "Artist removed from community list successfully"
        }))),
        Err(e) => {
            let status = if e.to_string().contains("not authorized") {
                StatusCode::FORBIDDEN
            } else if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to remove artist from community list: {}", e)
                })),
            ))
        }
    }
}

async fn subscribe_to_community_list_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    axum::extract::Path(list_id): axum::extract::Path<Uuid>,
    Json(request): Json<SubscribeToCommunityListRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.community_list_service.subscribe_to_community_list(user.id, list_id, request).await {
        Ok(subscription_details) => Ok(Json(json!({
            "success": true,
            "data": subscription_details,
            "message": "Subscribed to community list successfully"
        }))),
        Err(e) => {
            let status = if e.to_string().contains("not found") || e.to_string().contains("not accessible") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("already subscribed") {
                StatusCode::CONFLICT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to subscribe to community list: {}", e)
                })),
            ))
        }
    }
}

async fn unsubscribe_from_community_list_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    axum::extract::Path(list_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.community_list_service.unsubscribe_from_community_list(user.id, list_id).await {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "data": null,
            "message": "Unsubscribed from community list successfully"
        }))),
        Err(e) => {
            let status = if e.to_string().contains("not subscribed") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to unsubscribe from community list: {}", e)
                })),
            ))
        }
    }
}

async fn update_subscription_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    axum::extract::Path(list_id): axum::extract::Path<Uuid>,
    Json(request): Json<UpdateSubscriptionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.community_list_service.update_subscription(user.id, list_id, request).await {
        Ok(subscription_details) => Ok(Json(json!({
            "success": true,
            "data": subscription_details,
            "message": "Subscription updated successfully"
        }))),
        Err(e) => {
            let status = if e.to_string().contains("not subscribed") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to update subscription: {}", e)
                })),
            ))
        }
    }
}

async fn get_subscription_impact_preview_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    axum::extract::Path(list_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.community_list_service.get_subscription_impact_preview(user.id, list_id).await {
        Ok(impact_preview) => Ok(Json(json!({
            "success": true,
            "data": impact_preview,
            "message": "Subscription impact preview generated successfully"
        }))),
        Err(e) => {
            let status = if e.to_string().contains("not found") || e.to_string().contains("not accessible") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to generate impact preview: {}", e)
                })),
            ))
        }
    }
}

async fn get_user_subscriptions_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.community_list_service.get_user_subscriptions(user.id).await {
        Ok(subscriptions) => Ok(Json(json!({
            "success": true,
            "data": {
                "subscriptions": subscriptions,
                "total": subscriptions.len()
            },
            "message": "User subscriptions retrieved successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get user subscriptions: {}", e)
            })),
        )),
    }
}// Job
 management handler functions

async fn get_user_jobs_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let status_filter = params.get("status").and_then(|s| {
        match s.as_str() {
            "pending" => Some(JobStatus::Pending),
            "processing" => Some(JobStatus::Processing),
            "completed" => Some(JobStatus::Completed),
            "failed" => Some(JobStatus::Failed),
            "retrying" => Some(JobStatus::Retrying),
            _ => None,
        }
    });

    let limit = params.get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(50);

    match app_state.job_queue_service.get_user_jobs(&user.id, status_filter, Some(limit)).await {
        Ok(jobs) => Ok(Json(json!({
            "success": true,
            "data": jobs,
            "message": "User jobs retrieved successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get user jobs: {}", e)
            })),
        )),
    }
}

async fn get_job_status_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    axum::extract::Path(job_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.job_queue_service.get_job_status(&job_id).await {
        Ok(Some(job)) => {
            // Verify the job belongs to the user
            if job.user_id != Some(user.id) {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "success": false,
                        "data": null,
                        "message": "Access denied to this job"
                    })),
                ));
            }

            Ok(Json(json!({
                "success": true,
                "data": job,
                "message": "Job status retrieved successfully"
            })))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "data": null,
                "message": "Job not found"
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get job status: {}", e)
            })),
        )),
    }
}

async fn retry_job_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    axum::extract::Path(job_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // First verify the job belongs to the user
    match app_state.job_queue_service.get_job_status(&job_id).await {
        Ok(Some(job)) => {
            if job.user_id != Some(user.id) {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "success": false,
                        "data": null,
                        "message": "Access denied to this job"
                    })),
                ));
            }
        }
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": "Job not found"
                })),
            ));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to verify job: {}", e)
                })),
            ));
        }
    }

    match app_state.job_queue_service.retry_job(&job_id).await {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "data": null,
            "message": "Job retry initiated successfully"
        }))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to retry job: {}", e)
            })),
        )),
    }
}

#[derive(serde::Deserialize)]
struct EnqueueJobRequest {
    job_type: String,
    payload: serde_json::Value,
    priority: Option<String>,
    provider: Option<String>,
    scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
}

async fn enqueue_job_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    Json(request): Json<EnqueueJobRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let job_type = match request.job_type.as_str() {
        "enforcement_execution" => JobType::EnforcementExecution,
        "batch_rollback" => JobType::BatchRollback,
        "token_refresh" => JobType::TokenRefresh,
        "library_scan" => JobType::LibraryScan,
        "community_list_update" => JobType::CommunityListUpdate,
        "health_check" => JobType::HealthCheck,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Unknown job type: {}", request.job_type)
                })),
            ));
        }
    };

    let priority = match request.priority.as_deref() {
        Some("low") => JobPriority::Low,
        Some("normal") | None => JobPriority::Normal,
        Some("high") => JobPriority::High,
        Some("critical") => JobPriority::Critical,
        Some(other) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Unknown priority: {}", other)
                })),
            ));
        }
    };

    match app_state.job_queue_service.enqueue_job(
        job_type,
        request.payload,
        priority,
        Some(user.id),
        request.provider,
        request.scheduled_at,
    ).await {
        Ok(job_id) => Ok(Json(json!({
            "success": true,
            "data": {
                "job_id": job_id,
                "status": "queued"
            },
            "message": "Job enqueued successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to enqueue job: {}", e)
            })),
        )),
    }
}

async fn get_worker_stats_handler(
    State(app_state): State<AppState>,
    Extension(_user): Extension<User>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.job_queue_service.get_worker_stats().await {
        Ok(stats) => Ok(Json(json!({
            "success": true,
            "data": stats,
            "message": "Worker statistics retrieved successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get worker stats: {}", e)
            })),
        )),
    }
}

async fn get_rate_limit_status_handler(
    State(app_state): State<AppState>,
    Extension(_user): Extension<User>,
    axum::extract::Path(provider): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.rate_limiting_service.can_proceed(&provider).await {
        Ok(can_proceed) => {
            // Get additional rate limit information
            let status = json!({
                "provider": provider,
                "can_proceed": can_proceed,
                "timestamp": chrono::Utc::now()
            });

            Ok(Json(json!({
                "success": true,
                "data": status,
                "message": "Rate limit status retrieved successfully"
            })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get rate limit status: {}", e)
            })),
        )),
    }
}
// Audit an
d compliance handler functions

/// Get audit logs with filtering and pagination
async fn get_audit_logs_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Parse query parameters
    let query = AuditLogQuery {
        user_id: params.get("user_id").and_then(|id| uuid::Uuid::parse_str(id).ok()),
        action: params.get("action").cloned(),
        subject_type: params.get("subject_type").cloned(),
        subject_id: params.get("subject_id").cloned(),
        start_date: params.get("start_date").and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok().map(|dt| dt.with_timezone(&chrono::Utc))),
        end_date: params.get("end_date").and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok().map(|dt| dt.with_timezone(&chrono::Utc))),
        limit: params.get("limit").and_then(|l| l.parse().ok()),
        offset: params.get("offset").and_then(|o| o.parse().ok()),
    };

    match app_state.audit_service.query_audit_logs(query).await {
        Ok(response) => {
            // Log the audit log access
            let _ = app_state.audit_service.log_audit_event(CreateAuditLogRequest {
                actor_user_id: Some(user.id),
                action: "audit_logs_accessed".to_string(),
                subject_type: "audit_log".to_string(),
                subject_id: "query".to_string(),
                before_state: None,
                after_state: Some(json!({
                    "query_params": params,
                    "result_count": response.entries.len()
                })),
                ip_address: None,
                user_agent: None,
            }).await;

            Ok(Json(json!({
                "success": true,
                "data": response,
                "message": "Audit logs retrieved successfully"
            })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get audit logs: {}", e)
            })),
        )),
    }
}

/// Get security events for monitoring
async fn get_security_events_handler(
    State(app_state): State<AppState>,
    Extension(_user): Extension<User>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit = params.get("limit").and_then(|l| l.parse().ok());
    let severity_filter = params.get("severity").and_then(|s| {
        match s.as_str() {
            "critical" => Some(SecuritySeverity::Critical),
            "high" => Some(SecuritySeverity::High),
            "medium" => Some(SecuritySeverity::Medium),
            "low" => Some(SecuritySeverity::Low),
            _ => None,
        }
    });

    let events = app_state.audit_service.get_security_events(limit, severity_filter).await;

    Ok(Json(json!({
        "success": true,
        "data": {
            "events": events,
            "total_count": events.len()
        },
        "message": "Security events retrieved successfully"
    })))
}

/// Get security statistics
async fn get_security_stats_handler(
    State(app_state): State<AppState>,
    Extension(_user): Extension<User>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let stats = app_state.security_monitor.get_security_stats().await;

    Ok(Json(json!({
        "success": true,
        "data": stats,
        "message": "Security statistics retrieved successfully"
    })))
}

/// Request data export for GDPR/CCPA compliance
async fn request_data_export_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let request_type = match request.get("type").and_then(|t| t.as_str()) {
        Some("export") => DataRequestType::Export,
        Some("deletion") => DataRequestType::Deletion,
        _ => DataRequestType::Export,
    };

    match app_state.audit_service.create_data_export_request(user.id, request_type).await {
        Ok(export_request) => Ok(Json(json!({
            "success": true,
            "data": {
                "request_id": export_request.id,
                "status": export_request.status,
                "requested_at": export_request.requested_at,
                "expires_at": export_request.expires_at
            },
            "message": "Data export request created successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to create data export request: {}", e)
            })),
        )),
    }
}

/// Get data export request status
async fn get_data_export_status_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    axum::extract::Path(request_id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Get the request from database
    match sqlx::query_as(
        DataExportRequest,
        r#"
        SELECT id, user_id, request_type as "request_type: DataRequestType",
               status as "status: DataRequestStatus", requested_at,
               completed_at, export_url, expires_at, verification_token
        FROM data_export_requests
        WHERE id = $1 AND user_id = $2
        "#,
        request_id,
        user.id
    )
    .fetch_optional(&app_state.db_pool)
    .await
    {
        Ok(Some(request)) => Ok(Json(json!({
            "success": true,
            "data": {
                "request_id": request.id,
                "status": request.status,
                "requested_at": request.requested_at,
                "completed_at": request.completed_at,
                "export_url": request.export_url,
                "expires_at": request.expires_at
            },
            "message": "Data export status retrieved successfully"
        }))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "data": null,
                "message": "Data export request not found"
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get data export status: {}", e)
            })),
        )),
    }
}

/// Download data export
async fn download_data_export_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    axum::extract::Path(request_id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.audit_service.process_data_export_request(request_id).await {
        Ok(user_data) => {
            // Log the data export download
            let _ = app_state.audit_service.log_audit_event(CreateAuditLogRequest {
                actor_user_id: Some(user.id),
                action: "data_export_downloaded".to_string(),
                subject_type: "user".to_string(),
                subject_id: user.id.to_string(),
                before_state: None,
                after_state: Some(json!({
                    "request_id": request_id,
                    "total_records": user_data.export_metadata.total_records
                })),
                ip_address: None,
                user_agent: None,
            }).await;

            Ok(Json(json!({
                "success": true,
                "data": user_data,
                "message": "Data export completed successfully"
            })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to process data export: {}", e)
            })),
        )),
    }
}

/// Request data deletion for GDPR/CCPA compliance
async fn request_data_deletion_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let confirmation = request.get("confirmation").and_then(|c| c.as_str()).unwrap_or("");
    
    if confirmation != "DELETE_ALL_MY_DATA" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "data": null,
                "message": "Data deletion requires explicit confirmation"
            })),
        ));
    }

    match app_state.audit_service.delete_user_data(user.id).await {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "data": null,
            "message": "User data deletion completed successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to delete user data: {}", e)
            })),
        )),
    }
}

/// Start vulnerability scan
async fn start_vulnerability_scan_handler(
    State(app_state): State<AppState>,
    Extension(_user): Extension<User>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let scan_type = request.get("scan_type").and_then(|t| t.as_str()).unwrap_or("dependency").to_string();
    let scan_tool = request.get("scan_tool").and_then(|t| t.as_str()).unwrap_or("cargo-audit").to_string();

    let scan_id = app_state.vulnerability_scanner.start_scan(scan_type, scan_tool).await;

    // Simulate vulnerability scan completion (in real implementation, this would be async)
    tokio::spawn({
        let scanner = app_state.vulnerability_scanner.clone();
        async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            // Mock vulnerabilities for demonstration
            let vulnerabilities = vec![
                Vulnerability {
                    id: "RUSTSEC-2023-0001".to_string(),
                    severity: VulnerabilitySeverity::Medium,
                    title: "Example vulnerability".to_string(),
                    description: "This is a mock vulnerability for demonstration".to_string(),
                    affected_component: "example-crate".to_string(),
                    fix_available: true,
                    fix_description: Some("Update to version 1.2.3".to_string()),
                },
            ];
            
            let _ = scanner.complete_scan(scan_id, vulnerabilities).await;
        }
    });

    Ok(Json(json!({
        "success": true,
        "data": {
            "scan_id": scan_id,
            "status": "running"
        },
        "message": "Vulnerability scan started successfully"
    })))
}

/// Get vulnerability scan results
async fn get_vulnerability_scan_results_handler(
    State(app_state): State<AppState>,
    Extension(_user): Extension<User>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit = params.get("limit").and_then(|l| l.parse().ok());
    let results = app_state.vulnerability_scanner.get_latest_scan_results(limit).await;

    Ok(Json(json!({
        "success": true,
        "data": {
            "scan_results": results,
            "total_count": results.len()
        },
        "message": "Vulnerability scan results retrieved successfully"
    })))
}

/// Get vulnerability summary
async fn get_vulnerability_summary_handler(
    State(app_state): State<AppState>,
    Extension(_user): Extension<User>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let summary = app_state.vulnerability_scanner.get_vulnerability_summary().await;

    Ok(Json(json!({
        "success": true,
        "data": summary,
        "message": "Vulnerability summary retrieved successfully"
    })))
}// 
Content moderation handler functions

/// Get moderation queue entries
async fn get_moderation_queue_handler(
    State(app_state): State<AppState>,
    Extension(_user): Extension<User>, // TODO: Check moderator permissions
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let status_filter = params.get("status").and_then(|s| {
        match s.as_str() {
            "pending" => Some(ModerationStatus::Pending),
            "under_review" => Some(ModerationStatus::UnderReview),
            "approved" => Some(ModerationStatus::Approved),
            "rejected" => Some(ModerationStatus::Rejected),
            "requires_changes" => Some(ModerationStatus::RequiresChanges),
            _ => None,
        }
    });

    let priority_filter = params.get("priority").and_then(|p| {
        match p.as_str() {
            "low" => Some(ModerationPriority::Low),
            "normal" => Some(ModerationPriority::Normal),
            "high" => Some(ModerationPriority::High),
            "urgent" => Some(ModerationPriority::Urgent),
            _ => None,
        }
    });

    let limit = params.get("limit").and_then(|l| l.parse().ok());
    let offset = params.get("offset").and_then(|o| o.parse().ok());

    match app_state.content_moderation_service
        .get_moderation_queue(status_filter, priority_filter, limit, offset)
        .await
    {
        Ok(entries) => Ok(Json(json!({
            "success": true,
            "data": {
                "entries": entries,
                "total_count": entries.len()
            },
            "message": "Moderation queue retrieved successfully"
        }))),
        Err(e) => {
            tracing::error!("Failed to get moderation queue: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Review content in moderation queue
async fn review_content_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>, // TODO: Check moderator permissions
    axum::extract::Path(entry_id): axum::extract::Path<uuid::Uuid>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let decision = match request.get("decision").and_then(|d| d.as_str()) {
        Some("approved") => ModerationStatus::Approved,
        Some("rejected") => ModerationStatus::Rejected,
        Some("requires_changes") => ModerationStatus::RequiresChanges,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let review_notes = request.get("review_notes").and_then(|n| n.as_str()).map(|s| s.to_string());

    match app_state.content_moderation_service
        .review_content(entry_id, user.id, decision, review_notes)
        .await
    {
        Ok(updated_entry) => Ok(Json(json!({
            "success": true,
            "data": updated_entry,
            "message": "Content review completed successfully"
        }))),
        Err(e) => {
            tracing::error!("Failed to review content: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Submit an appeal for moderation decision
async fn submit_appeal_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let moderation_entry_id = request.get("moderation_entry_id")
        .and_then(|id| id.as_str())
        .and_then(|id| uuid::Uuid::parse_str(id).ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let appeal_reason = request.get("appeal_reason")
        .and_then(|r| r.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?
        .to_string();

    let additional_evidence = request.get("additional_evidence").cloned();

    match app_state.content_moderation_service
        .submit_appeal(moderation_entry_id, user.id, appeal_reason, additional_evidence)
        .await
    {
        Ok(appeal) => Ok(Json(json!({
            "success": true,
            "data": appeal,
            "message": "Appeal submitted successfully"
        }))),
        Err(e) => {
            tracing::error!("Failed to submit appeal: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Review an appeal (placeholder - would need full implementation)
async fn review_appeal_handler(
    State(_app_state): State<AppState>,
    Extension(_user): Extension<User>, // TODO: Check reviewer permissions
    axum::extract::Path(_appeal_id): axum::extract::Path<uuid::Uuid>,
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Implement appeal review logic
    Ok(Json(json!({
        "success": true,
        "message": "Appeal review functionality not yet implemented"
    })))
}

/// Get moderation statistics
async fn get_moderation_stats_handler(
    State(app_state): State<AppState>,
    Extension(_user): Extension<User>, // TODO: Check moderator permissions
) -> Result<Json<serde_json::Value>, StatusCode> {
    match app_state.content_moderation_service.get_moderation_stats().await {
        Ok(stats) => Ok(Json(json!({
            "success": true,
            "data": stats,
            "message": "Moderation statistics retrieved successfully"
        }))),
        Err(e) => {
            tracing::error!("Failed to get moderation stats: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Validate content before submission
async fn validate_content_handler(
    State(app_state): State<AppState>,
    Extension(_user): Extension<User>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let content = request.get("content")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let context = request.get("context")
        .and_then(|c| c.as_str())
        .unwrap_or("general");

    match app_state.content_moderation_service
        .moderate_content(content, context)
        .await
    {
        Ok(result) => Ok(Json(json!({
            "success": true,
            "data": result,
            "message": "Content validation completed"
        }))),
        Err(e) => {
            tracing::error!("Failed to validate content: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
