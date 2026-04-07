//! Music Streaming Blocklist Manager Backend
//!
//! A modular backend service for managing Do-Not-Play lists across music streaming platforms.
//!
//! ## Workspace Crate Structure
//!
//! - `ndith-core`: Shared types (error, config, models, validation)
//! - `ndith-db`: PostgreSQL + Redis connection management, health checks, recovery
//! - `ndith-analytics`: DuckDB + graph analytics (heavyweight native deps isolated)
//! - `ndith-news`: LanceDB + fastembed news pipeline (heavyweight deps isolated)
//! - `ndith-services`: Business logic (auth, OAuth, token vault, catalog sync, etc.)
//! - Root binary: Handlers, middleware, router, main entry point
#![allow(clippy::result_large_err)]

use axum::{
    extract::{Path, RawQuery, State},
    response::{Json, Redirect},
    routing::{any, delete, get, put},
    Router,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use axum::routing::post;
use axum::{http::StatusCode, response::IntoResponse};

// ---- Root-local modules (handlers, middleware, metrics, monitoring stay here) ----
pub mod backfill_orchestrator;
pub mod handlers;
pub mod middleware;
pub mod monitoring;
pub mod runtime;

// metrics module from ndith-services (not a root-local module)
pub use ndith_services::metrics;

#[cfg(test)]
pub mod test_database;

// ---- Re-export from workspace crates for backward compatibility ----

// ndith-core: error, config, models, validation
pub use ndith_core::config;
pub use ndith_core::error;
pub use ndith_core::models;
pub use ndith_core::validation;

pub use ndith_core::config::{
    AppConfig, AppleMusicCredentials, AuthConfig, ConfigError, DatabaseSettings, DeezerConfig,
    Environment, OAuthSettings, PlatformSyncConfig, RedisSettings, ServerConfig,
    SpotifyCredentials, TidalCredentials, TokenRefreshConfig, YouTubeCredentials,
};
pub use ndith_core::error::{AppError, ErrorResponse, Result};
pub use ndith_core::models::*;
pub use ndith_core::validation::{
    validate_email, validate_password, validate_totp_code, ValidatedJson,
};

// ndith-db: database, health, recovery
pub use ndith_db::database;
pub use ndith_db::health;
pub use ndith_db::recovery;

pub use ndith_db::{
    create_pool, create_redis_pool, db_health_check, redis_health_check, run_migrations,
    seed_test_data, DatabaseConfig, RedisConfiguration,
};
pub use ndith_db::{
    liveness_check, readiness_check, HealthCheckResponse, HealthStatus, SystemInfo,
};
pub use ndith_db::{retry_database_operation, retry_redis_operation, CircuitBreaker, RetryConfig};

// ndith-services: all business logic services
pub use ndith_services as services;

pub use backfill_orchestrator::{BackfillOrchestrator, BackfillProgress, BackfillResult};
#[cfg(feature = "news")]
pub use ndith_news::{
    NewsPipelineConfig, NewsPipelineOrchestrator, ScheduledPipelineHandle, ScheduledPipelineRunner,
};
pub use ndith_services::catalog_sync::{
    CatalogSyncOrchestrator, CreditsSyncService, OrchestratorBuilder,
};
pub use ndith_services::DnpListService;
pub use ndith_services::{
    AuditLoggingService, AuthService, NotificationService, RateLimitService,
    TokenRefreshBackgroundJob, UserService,
};
pub use ndith_services::{CircuitBreakerConfig, CircuitBreakerService};
pub use ndith_services::{TokenVaultBackgroundService, TokenVaultService, TokenVaultStatistics};

// Re-export metrics and monitoring from root
pub use metrics::{metrics_handler, DatabaseMetrics, MetricsCollector, RedisMetrics, RequestTimer};
pub use middleware::{create_cors_layer, validate_cors_config};
pub use monitoring::{
    AlertManager, AlertThresholds, DatabaseServiceMetrics, HttpServiceMetrics, MonitoringConfig,
    MonitoringResponse, MonitoringSystem, PerformanceProfiler, RedisServiceMetrics, ServiceMetrics,
    SystemMetrics,
};
pub use runtime::{run_service, ServiceMode};

// Re-export stub services for testing
#[cfg(test)]
pub use ndith_services::stubs::*;

// Application state - will be expanded as services are added
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub redis_pool: deadpool_redis::Pool,
    pub auth_service: Arc<AuthService>,
    pub rate_limiter: Arc<RateLimitService>,
    pub audit_logger: Arc<AuditLoggingService>,
    pub dnp_service: Arc<DnpListService>,
    pub user_service: Arc<UserService>,
    pub monitoring: Arc<MonitoringSystem>,
    pub metrics: Arc<MetricsCollector>,
    pub catalog_sync: Arc<CatalogSyncOrchestrator>,
    pub platform_config: PlatformSyncConfig,
    pub credits_sync: Option<Arc<CreditsSyncService>>,
    pub backfill_orchestrator: Option<Arc<BackfillOrchestrator>>,
    pub apple_music_service: Arc<ndith_services::AppleMusicService>,
    /// Circuit breaker for provider API calls (US-026)
    pub circuit_breaker: Arc<CircuitBreakerService>,
    /// Test user ID for development (will be removed in production auth)
    pub test_user_id: Option<uuid::Uuid>,
}

// Health check response structure
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub services: HashMap<String, ServiceHealth>,
}

#[derive(Serialize)]
pub struct ServiceHealth {
    pub status: String,
    pub response_time_ms: Option<u64>,
}

/// Create the main application router
pub fn create_router(state: AppState) -> Router {
    use axum::routing::post;

    // Create public auth routes (no authentication required)
    let auth_routes = Router::new()
        .route(
            "/register",
            post(handlers::auth::register_handler).layer(axum::middleware::from_fn_with_state(
                state.rate_limiter.clone(),
                ndith_services::registration_rate_limit_middleware,
            )),
        )
        .route("/login", post(handlers::auth::login_handler))
        .route("/refresh", post(handlers::auth::refresh_token_handler))
        // OAuth routes
        .route(
            "/oauth/:provider/initiate",
            post(handlers::oauth::initiate_oauth_handler),
        )
        .route(
            "/oauth/:provider/callback",
            post(handlers::oauth::oauth_callback_handler),
        )
        // Apple OAuth routes (explicit endpoints per acceptance criteria US-003)
        .route(
            "/oauth/apple/authorize",
            get(handlers::oauth::apple_authorize_handler),
        )
        .route(
            "/oauth/apple/callback",
            post(handlers::oauth::apple_oauth_callback_handler),
        )
        .route(
            "/oauth/apple/callback-form",
            post(handlers::oauth::apple_oauth_callback_handler),
        )
        // GitHub OAuth routes
        .route(
            "/oauth/github/authorize",
            get(handlers::oauth::github_authorize_handler),
        )
        .route(
            "/oauth/github/callback",
            post(handlers::oauth::github_oauth_callback_handler),
        )
        // Google OAuth routes
        .route(
            "/oauth/google/authorize",
            get(handlers::oauth::google_authorize_handler),
        )
        .route(
            "/oauth/google/callback",
            post(handlers::oauth::google_oauth_callback_handler),
        );

    // Create protected routes that require authentication
    let protected_routes = Router::new()
        // User routes
        .route("/users/profile", get(handlers::user::get_profile_handler))
        .route(
            "/users/profile",
            put(handlers::user::update_profile_handler),
        )
        .route("/users/export", get(handlers::user::export_data_handler))
        .route(
            "/users/account",
            delete(handlers::user::delete_account_handler),
        )
        // Auth routes (protected)
        .route("/auth/logout", post(handlers::auth::logout_handler))
        // 2FA routes
        .route("/auth/2fa/setup", post(handlers::auth::setup_2fa_handler))
        .route("/auth/2fa/verify", post(handlers::auth::verify_2fa_handler))
        .route(
            "/auth/2fa/disable",
            post(handlers::auth::disable_2fa_handler),
        )
        // OAuth account management (protected)
        .route(
            "/auth/oauth/:provider/link",
            post(handlers::oauth::link_oauth_account_handler),
        )
        .route(
            "/auth/oauth/:provider/link-callback",
            post(handlers::oauth::oauth_link_callback_handler),
        )
        .route(
            "/auth/oauth/:provider/unlink",
            delete(handlers::oauth::unlink_oauth_account_handler),
        )
        .route(
            "/auth/oauth/accounts",
            get(handlers::oauth::get_linked_accounts_handler),
        )
        // DNP routes
        .route("/dnp/search", get(handlers::dnp::search_artists_handler))
        .route("/dnp/list", get(handlers::dnp::get_dnp_list_handler))
        .route("/dnp/list", post(handlers::dnp::add_to_dnp_handler))
        .route(
            "/dnp/list/:artist_id",
            delete(handlers::dnp::remove_from_dnp_handler),
        )
        .route(
            "/dnp/list/:artist_id",
            put(handlers::dnp::update_dnp_entry_handler),
        )
        // Track-level blocking routes
        .route("/dnp/tracks", post(handlers::dnp::add_track_block_handler))
        .route("/dnp/tracks", get(handlers::dnp::get_track_blocks_handler))
        .route(
            "/dnp/tracks/:track_id",
            delete(handlers::dnp::remove_track_block_handler),
        )
        .route(
            "/dnp/tracks/batch",
            post(handlers::dnp::batch_track_blocks_handler),
        )
        // Deep blocking routes
        .route(
            "/dnp/blocked-tracks",
            get(handlers::dnp::get_blocked_tracks_handler),
        )
        .route(
            "/dnp/blocked-albums",
            get(handlers::dnp::get_blocked_albums_handler),
        )
        .route(
            "/dnp/block-summary",
            get(handlers::dnp::get_block_summary_handler),
        )
        .route(
            "/dnp/revenue-impact",
            get(handlers::dnp::get_revenue_impact_handler),
        )
        .route(
            "/dnp/revenue-by-category",
            get(handlers::dnp::get_revenue_by_category_handler),
        )
        // Artist analytics
        .route(
            "/artists/:artist_id/analytics",
            get(handlers::dnp::get_artist_analytics_handler),
        )
        // Library routes
        .route("/library/import", post(handlers::offense::import_library))
        .route("/library/scan", get(handlers::offense::scan_library))
        .route(
            "/library/scan/cached",
            get(handlers::offense::get_cached_scan),
        )
        .route("/library/tracks", get(handlers::offense::get_library))
        .route("/library/items", get(handlers::offense::get_library_items))
        .route(
            "/library/groups",
            get(handlers::offense::get_library_groups),
        )
        .route(
            "/library/offenders",
            get(handlers::offense::get_library_offenders),
        )
        .route(
            "/library/taste-grade",
            get(handlers::offense::get_taste_grade),
        )
        .route(
            "/library/playlists/tracks",
            get(handlers::offense::get_playlist_tracks),
        )
        .route(
            "/library/playlists/:playlist_id/tracks",
            get(handlers::offense::get_playlist_tracks_by_id),
        )
        .route("/library/playlists", get(handlers::offense::list_playlists))
        // Offense submission routes
        .route("/offenses/submit", post(handlers::offense::create_offense))
        .route(
            "/offenses/:offense_id/evidence",
            post(handlers::offense::add_evidence),
        )
        .route(
            "/offenses/:offense_id/verify",
            post(handlers::offense::verify_offense),
        )
        // Category subscription routes
        .route("/categories", get(handlers::category::get_categories))
        .route(
            "/categories/:category_id/subscribe",
            post(handlers::category::subscribe_category),
        )
        .route(
            "/categories/:category_id/subscribe",
            delete(handlers::category::unsubscribe_category),
        )
        .route(
            "/categories/blocked-artists",
            get(handlers::category::get_blocked_artists),
        )
        .route(
            "/artists/search",
            get(handlers::dnp::search_artists_handler),
        )
        // Catalog sync routes
        .route("/sync/status", get(handlers::sync::get_sync_status_handler))
        .route("/sync/trigger", post(handlers::sync::trigger_sync_handler))
        .route("/sync/runs", get(handlers::sync::get_sync_runs_handler))
        .route(
            "/sync/runs/:run_id",
            get(handlers::sync::get_sync_run_handler),
        )
        .route(
            "/sync/runs/:run_id/cancel",
            post(handlers::sync::cancel_sync_run_handler),
        )
        .route(
            "/sync/resolve-identity",
            post(handlers::sync::resolve_identity_handler),
        )
        .route(
            "/sync/merge-artists",
            post(handlers::sync::merge_artists_handler),
        )
        .route(
            "/sync/search",
            get(handlers::sync::cross_platform_search_handler),
        )
        .route(
            "/sync/artists",
            get(handlers::sync::get_canonical_artists_handler),
        )
        .route(
            "/sync/artists/:artist_id",
            get(handlers::sync::get_canonical_artist_handler),
        )
        .route("/sync/health", get(handlers::sync::platform_health_handler))
        // Credits sync routes
        .route(
            "/sync/credits",
            post(handlers::sync::trigger_credits_sync_handler),
        )
        .route(
            "/sync/credits/status",
            get(handlers::sync::get_credits_sync_status_handler),
        )
        .route(
            "/sync/credits/:artist_id",
            post(handlers::sync::trigger_artist_credits_sync_handler),
        )
        // Bulk import routes
        .route(
            "/sync/import-charts",
            post(handlers::sync::import_charts_handler),
        )
        .route(
            "/sync/import-musicbrainz",
            post(handlers::sync::import_musicbrainz_handler),
        )
        // Offense backfill routes
        .route(
            "/sync/backfill-offenses",
            post(handlers::sync::backfill_offenses_handler),
        )
        .route(
            "/sync/backfill-status",
            get(handlers::sync::backfill_status_handler),
        );
    let protected_routes = add_full_platform_routes(protected_routes);
    let protected_routes = protected_routes
        // Apple Music enforcement routes
        .route(
            "/enforcement/apple-music/run",
            post(handlers::enforcement::run_apple_music_enforcement),
        )
        .route(
            "/enforcement/apple-music/preview",
            post(handlers::enforcement::preview_apple_music_enforcement),
        )
        .route(
            "/enforcement/apple-music/rollback/:run_id",
            post(handlers::enforcement::rollback_apple_music_enforcement),
        )
        .route(
            "/enforcement/apple-music/history",
            get(handlers::enforcement::get_apple_music_enforcement_history),
        )
        .route(
            "/enforcement/apple-music/capabilities",
            get(handlers::enforcement::get_apple_music_capabilities),
        )
        // Playlist sanitizer routes
        .route(
            "/sanitizer/grade",
            post(handlers::playlist_sanitizer::grade_playlist),
        )
        .route(
            "/sanitizer/suggest",
            post(handlers::playlist_sanitizer::suggest_replacements),
        )
        .route(
            "/sanitizer/plan/:plan_id",
            put(handlers::playlist_sanitizer::confirm_plan),
        )
        .route(
            "/sanitizer/publish/:plan_id",
            post(handlers::playlist_sanitizer::publish_playlist),
        )
        // Spotify enforcement routes
        .route(
            "/enforcement/spotify/run",
            post(handlers::spotify_enforcement::run_spotify_enforcement),
        )
        .route(
            "/enforcement/spotify/preview",
            post(handlers::spotify_enforcement::preview_spotify_enforcement),
        )
        .route(
            "/enforcement/spotify/rollback/:batch_id",
            post(handlers::spotify_enforcement::rollback_spotify_enforcement),
        )
        .route(
            "/enforcement/spotify/history",
            get(handlers::spotify_enforcement::get_spotify_enforcement_history),
        )
        .route(
            "/enforcement/spotify/capabilities",
            get(handlers::spotify_enforcement::get_spotify_capabilities),
        )
        .route(
            "/enforcement/spotify/progress/:batch_id",
            get(handlers::spotify_enforcement::get_spotify_enforcement_progress),
        )
        .route(
            "/enforcement/batches/:batch_id/rollback",
            post(handlers::spotify_enforcement::rollback_enforcement_batch),
        )
        // Connection health check routes
        .route(
            "/connections",
            get(handlers::connections::get_connections_health_handler),
        )
        // Apple Music auth routes
        .route(
            "/apple-music/auth/connect",
            post(handlers::apple_music_auth::connect_apple_music),
        )
        .route(
            "/apple-music/auth/status",
            get(handlers::apple_music_auth::get_connection_status),
        )
        .route(
            "/apple-music/auth/disconnect",
            delete(handlers::apple_music_auth::disconnect_apple_music),
        )
        .route(
            "/apple-music/auth/verify",
            post(handlers::apple_music_auth::verify_connection),
        )
        .route(
            "/apple-music/library/sync",
            post(handlers::apple_music_auth::sync_library),
        )
        .route(
            "/apple-music/library/sync-status",
            get(handlers::apple_music_auth::get_library_sync_status),
        )
        .route(
            "/apple-music/library",
            get(handlers::apple_music_auth::get_library),
        )
        // Spotify connection routes
        .route(
            "/connections/spotify/authorize",
            get(handlers::spotify_connection::spotify_authorize_handler),
        )
        .route(
            "/connections/spotify/callback",
            post(handlers::spotify_connection::spotify_callback_handler),
        )
        .route(
            "/connections/spotify/status",
            get(handlers::spotify_connection::spotify_connection_status_handler),
        )
        .route(
            "/connections/spotify/refresh",
            post(handlers::spotify_connection::spotify_refresh_token_handler),
        )
        .route(
            "/connections/spotify/library/sync",
            post(handlers::spotify_connection::spotify_library_sync_handler),
        )
        .route(
            "/connections/spotify/library/sync-status",
            get(handlers::spotify_connection::spotify_library_sync_status_handler),
        )
        .route(
            "/connections/spotify",
            delete(handlers::spotify_connection::spotify_disconnect_handler),
        )
        // Tidal connection routes
        .route(
            "/connections/tidal/authorize",
            get(handlers::tidal_connection::tidal_authorize_handler),
        )
        .route(
            "/connections/tidal/callback",
            post(handlers::tidal_connection::tidal_callback_handler),
        )
        .route(
            "/connections/tidal/status",
            get(handlers::tidal_connection::tidal_connection_status_handler),
        )
        .route(
            "/connections/tidal/library/sync",
            post(handlers::tidal_connection::tidal_library_sync_handler),
        )
        .route(
            "/connections/tidal/library/sync-status",
            get(handlers::tidal_connection::tidal_library_sync_status_handler),
        )
        .route(
            "/connections/tidal",
            delete(handlers::tidal_connection::tidal_disconnect_handler),
        )
        // YouTube Music connection routes
        .route(
            "/connections/youtube/authorize",
            get(handlers::youtube_connection::youtube_authorize_handler),
        )
        .route(
            "/connections/youtube/callback",
            post(handlers::youtube_connection::youtube_callback_handler),
        )
        .route(
            "/connections/youtube/status",
            get(handlers::youtube_connection::youtube_status_handler),
        )
        .route(
            "/connections/youtube/library/sync",
            post(handlers::youtube_connection::youtube_library_sync_handler),
        )
        .route(
            "/connections/youtube/library/sync-status",
            get(handlers::youtube_connection::youtube_library_sync_status_handler),
        )
        .route(
            "/connections/youtube",
            delete(handlers::youtube_connection::youtube_disconnect_handler),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.auth_service.clone(),
            crate::middleware::auth::auth_middleware,
        ));

    // Public offense database routes (no auth required to browse)
    let offense_public_routes = Router::new()
        .route("/", get(handlers::offense::get_flagged_artists))
        .route("/query", get(handlers::offense::get_category_artists))
        .route("/:offense_id", get(handlers::offense::get_offense));

    Router::new()
        // OIDC discovery + JWKS (must be top-level, public, for Convex JWT verification)
        .route(
            "/.well-known/openid-configuration",
            get(handlers::oidc::openid_configuration),
        )
        .route("/.well-known/jwks.json", get(handlers::oidc::jwks))
        // Health check endpoints
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check_endpoint))
        .route("/health/live", get(liveness_check_endpoint))
        // OAuth callback bounce route:
        // providers redirect to backend (:3000), then we forward to frontend SPA callback route.
        .route("/auth/callback/:provider", get(oauth_callback_redirect))
        // OAuth health and configuration endpoints
        .route("/oauth/health", get(handlers::oauth::oauth_health_handler))
        .route(
            "/oauth/health/:provider",
            get(handlers::oauth::oauth_provider_health_handler),
        )
        .route(
            "/oauth/health/check",
            post(handlers::oauth::force_oauth_health_check_handler),
        )
        .route(
            "/oauth/config",
            get(handlers::oauth::oauth_config_status_handler),
        )
        .route(
            "/oauth/config/:provider/guidance",
            get(handlers::oauth::oauth_config_guidance_handler),
        )
        // Monitoring endpoints
        .route("/metrics", get(metrics_endpoint))
        .route("/metrics/prometheus", get(prometheus_metrics_endpoint))
        .route("/monitoring", get(comprehensive_monitoring_endpoint))
        // Public API routes
        .nest("/api/v1/auth", auth_routes)
        // Public offense browsing routes
        .nest("/api/v1/offenses", offense_public_routes)
        // Public Apple Music auth route
        .route(
            "/api/v1/apple-music/auth/developer-token",
            get(handlers::apple_music_auth::get_developer_token),
        )
        // Protected API routes
        .nest("/api/v1", protected_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(create_cors_layer()),
        )
        .layer(axum::middleware::from_fn(
            crate::middleware::security::security_headers_middleware,
        ))
        // Add latency metrics middleware
        .layer(axum::middleware::from_fn_with_state(
            state.metrics.clone(),
            crate::middleware::latency::latency_middleware,
        ))
        .with_state(state)
}

pub fn create_graph_router(state: AppState) -> Router {
    create_scoped_service_router(state, add_graph_routes(Router::new()))
}

pub fn create_analytics_router(state: AppState) -> Router {
    create_scoped_service_router(state, add_analytics_routes(Router::new()))
}

pub fn create_news_router(state: AppState) -> Router {
    let auth_service = state.auth_service.clone();
    let metrics = state.metrics.clone();

    let protected_routes = add_news_routes(Router::new());

    Router::new()
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check_endpoint))
        .route("/health/live", get(liveness_check_endpoint))
        .route("/metrics", get(metrics_endpoint))
        .route("/metrics/prometheus", get(prometheus_metrics_endpoint))
        .route("/monitoring", get(comprehensive_monitoring_endpoint))
        // Service-key-authenticated route — bypasses JWT middleware
        .route(
            "/api/v1/news/research/trigger",
            post(handlers::news::trigger_research_handler),
        )
        .nest(
            "/api/v1",
            protected_routes.layer(axum::middleware::from_fn_with_state(
                auth_service,
                crate::middleware::auth::auth_middleware,
            )),
        )
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(create_cors_layer()),
        )
        .layer(axum::middleware::from_fn(
            crate::middleware::security::security_headers_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            crate::middleware::latency::latency_middleware,
        ))
        .with_state(state)
}

fn create_scoped_service_router(state: AppState, protected_routes: Router<AppState>) -> Router {
    let auth_service = state.auth_service.clone();
    let metrics = state.metrics.clone();

    Router::new()
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check_endpoint))
        .route("/health/live", get(liveness_check_endpoint))
        .route("/metrics", get(metrics_endpoint))
        .route("/metrics/prometheus", get(prometheus_metrics_endpoint))
        .route("/monitoring", get(comprehensive_monitoring_endpoint))
        .nest(
            "/api/v1",
            protected_routes.layer(axum::middleware::from_fn_with_state(
                auth_service,
                crate::middleware::auth::auth_middleware,
            )),
        )
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(create_cors_layer()),
        )
        .layer(axum::middleware::from_fn(
            crate::middleware::security::security_headers_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            crate::middleware::latency::latency_middleware,
        ))
        .with_state(state)
}

fn add_full_platform_routes(router: Router<AppState>) -> Router<AppState> {
    add_news_routes(add_analytics_routes(add_graph_routes(router)))
}

#[cfg(feature = "analytics")]
fn add_graph_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/graph/search",
            get(handlers::graph::search_artists_handler),
        )
        .route(
            "/graph/stats",
            get(handlers::graph::get_global_stats_handler),
        )
        .route(
            "/graph/artists/:artist_id/network",
            get(handlers::graph::get_artist_network_handler),
        )
        .route(
            "/graph/artists/:artist_id/collaborators",
            get(handlers::graph::get_collaborators_handler),
        )
        .route(
            "/graph/artists/:from_id/path-to/:to_id",
            get(handlers::graph::find_path_handler),
        )
        .route(
            "/graph/artists/:artist_id/stats",
            get(handlers::graph::get_network_stats_handler),
        )
        .route(
            "/graph/artists/:artist_id/collab-stats",
            get(handlers::graph::get_collaboration_stats_handler),
        )
        .route(
            "/graph/artists/:artist_id/proximity",
            get(handlers::graph::search_by_proximity_handler),
        )
        .route(
            "/graph/blocked-network",
            get(handlers::graph::analyze_blocked_network_handler),
        )
        .route(
            "/graph/blocked-network/artists",
            get(handlers::graph::get_blocked_artists_network_handler),
        )
        .route(
            "/graph/blocked-network/at-risk",
            get(handlers::graph::get_at_risk_artists_handler),
        )
        .route(
            "/graph/offense-radius",
            get(handlers::graph::get_offense_radius_handler),
        )
        .route(
            "/graph/sync/status",
            get(handlers::graph::get_sync_status_handler),
        )
        .route(
            "/graph/sync/trigger",
            post(handlers::graph::trigger_sync_handler),
        )
        .route(
            "/graph/health",
            get(handlers::graph::get_graph_health_handler),
        )
        .route(
            "/graph/offense-network",
            get(handlers::graph::get_offense_network_handler),
        )
        .route(
            "/graph/artists/:artist_id/offense-connections",
            get(handlers::graph::get_artist_offense_connections_handler),
        )
}

#[cfg(not(feature = "analytics"))]
fn add_graph_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/graph", any(full_platform_unavailable))
        .route("/graph/*path", any(full_platform_unavailable))
}

#[cfg(feature = "analytics")]
fn add_analytics_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/analytics/dashboard",
            get(handlers::analytics_v2::get_dashboard_handler),
        )
        .route(
            "/analytics/dashboard/user-stats",
            get(handlers::analytics_v2::get_user_quick_stats_handler),
        )
        .route(
            "/analytics/health",
            get(handlers::analytics_v2::get_system_health_handler),
        )
        .route(
            "/analytics/trends",
            get(handlers::analytics_v2::get_trend_summary_handler),
        )
        .route(
            "/analytics/trends/artists/:artist_id",
            get(handlers::analytics_v2::get_artist_trend_handler),
        )
        .route(
            "/analytics/trends/platforms",
            get(handlers::analytics_v2::get_platform_trends_handler),
        )
        .route(
            "/analytics/trends/rising",
            get(handlers::analytics_v2::get_rising_artists_handler),
        )
        .route(
            "/analytics/trends/falling",
            get(handlers::analytics_v2::get_falling_artists_handler),
        )
        .route(
            "/analytics/reports/types",
            get(handlers::analytics_v2::get_report_types_handler),
        )
        .route(
            "/analytics/reports",
            post(handlers::analytics_v2::generate_report_handler),
        )
        .route(
            "/analytics/reports/:report_id",
            get(handlers::analytics_v2::get_report_status_handler),
        )
        .route(
            "/analytics/reports/:report_id/download",
            get(handlers::analytics_v2::download_report_handler),
        )
        .route(
            "/analytics/export/parquet",
            post(handlers::analytics_v2::export_to_parquet_handler),
        )
        .route(
            "/analytics/trouble-scores/artist/:artist_id",
            get(handlers::analytics_v2::get_artist_trouble_score_handler),
        )
        .route(
            "/analytics/trouble-scores/artist/:artist_id/history",
            get(handlers::analytics_v2::get_artist_score_history_handler),
        )
        .route(
            "/analytics/trouble-scores/leaderboard",
            get(handlers::analytics_v2::get_trouble_leaderboard_handler),
        )
        .route(
            "/analytics/trouble-scores/distribution",
            get(handlers::analytics_v2::get_tier_distribution_handler),
        )
        .route(
            "/analytics/trouble-scores/recalculate",
            post(handlers::analytics_v2::recalculate_trouble_scores_handler),
        )
        .route(
            "/analytics/revenue/distribution",
            get(handlers::analytics_v2::get_user_revenue_distribution_handler),
        )
        .route(
            "/analytics/revenue/top-artists",
            get(handlers::analytics_v2::get_user_top_artists_revenue_handler),
        )
        .route(
            "/analytics/revenue/problematic",
            get(handlers::analytics_v2::get_user_problematic_revenue_handler),
        )
        .route(
            "/analytics/revenue/global-problematic",
            get(handlers::analytics_v2::get_global_problematic_revenue_handler),
        )
        .route(
            "/analytics/revenue/artist/:artist_id",
            get(handlers::analytics_v2::get_artist_revenue_breakdown_handler),
        )
        .route(
            "/analytics/payout-rates",
            get(handlers::analytics_v2::get_payout_rates_handler),
        )
        .route(
            "/analytics/category-revenue",
            get(handlers::analytics_v2::get_global_category_revenue_handler),
        )
        .route(
            "/analytics/category-revenue/categories",
            get(handlers::analytics_v2::get_offense_categories_handler),
        )
        .route(
            "/analytics/category-revenue/:category",
            get(handlers::analytics_v2::get_category_revenue_handler),
        )
        .route(
            "/analytics/category-revenue/artist/:artist_id/discography",
            get(handlers::analytics_v2::get_artist_discography_revenue_handler),
        )
        .route(
            "/analytics/category-revenue/user/exposure",
            get(handlers::analytics_v2::get_user_category_exposure_handler),
        )
        .route(
            "/analytics/enforcement",
            get(handlers::analytics_v2::get_enforcement_analytics_handler),
        )
        .route(
            "/analytics/summary",
            get(handlers::analytics_v2::get_user_activity_summary_handler),
        )
}

#[cfg(not(feature = "analytics"))]
fn add_analytics_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/analytics", any(full_platform_unavailable))
        .route("/analytics/*path", any(full_platform_unavailable))
}

#[cfg(feature = "news")]
fn add_news_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/news/articles", get(handlers::news::list_articles_handler))
        .route(
            "/news/articles/:article_id",
            get(handlers::news::get_article_handler),
        )
        .route(
            "/news/artists/:artist_id/mentions",
            get(handlers::news::get_artist_mentions_handler),
        )
        .route(
            "/news/search",
            post(handlers::news::semantic_search_handler),
        )
        .route("/news/offenses", get(handlers::news::get_offenses_handler))
        .route(
            "/news/offenses/:offense_id",
            get(handlers::news::get_offense_handler),
        )
        .route(
            "/news/offenses/:offense_id/verify",
            post(handlers::news::verify_offense_handler),
        )
        .route(
            "/news/pipeline/status",
            get(handlers::news::get_pipeline_status_handler),
        )
        .route(
            "/news/pipeline/trigger",
            post(handlers::news::trigger_pipeline_handler),
        )
        .route("/news/sources", get(handlers::news::get_sources_handler))
        .route("/news/trending", get(handlers::news::get_trending_handler))
        .route(
            "/news/categories",
            get(handlers::news::get_offense_categories_handler),
        )
        // Research endpoints
        .route(
            "/news/research/status",
            get(handlers::news::get_research_status_handler),
        )
        .route(
            "/news/research/artists/:id",
            get(handlers::news::get_artist_research_handler),
        )
        .route(
            "/news/research/artists/:id/trigger",
            post(handlers::news::trigger_artist_research_handler),
        )
        .route(
            "/news/research/queue",
            get(handlers::news::get_research_queue_handler),
        )
}

#[cfg(not(feature = "news"))]
fn add_news_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/news", any(full_platform_unavailable))
        .route("/news/*path", any(full_platform_unavailable))
}

async fn full_platform_unavailable() -> impl IntoResponse {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({
            "error": "full_platform_unavailable",
            "message": "This Render API build does not include graph, analytics, or news services."
        })),
    )
}

async fn oauth_callback_redirect(
    Path(provider): Path<String>,
    RawQuery(raw_query): RawQuery,
) -> Redirect {
    let frontend_base = std::env::var("OAUTH_FRONTEND_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:5050".to_string());
    let frontend_base = frontend_base.trim_end_matches('/');

    let provider = match provider.as_str() {
        "youtube_music" => "youtube",
        other => other,
    };

    let mut redirect_to = format!("{}/auth/callback/{}", frontend_base, provider);
    if let Some(query) = raw_query.filter(|q| !q.is_empty()) {
        redirect_to.push('?');
        redirect_to.push_str(&query);
    }

    Redirect::temporary(&redirect_to)
}

/// Health check endpoint with comprehensive error handling
async fn health_check(State(state): State<AppState>) -> Result<Json<HealthCheckResponse>> {
    let health_response = state
        .monitoring
        .check_health(&state.db_pool, &state.redis_pool)
        .await;

    tracing::info!(
        status = ?health_response.status,
        correlation_id = %health_response.correlation_id,
        "Health check completed"
    );

    Ok(Json(health_response))
}

/// Readiness check endpoint for Kubernetes
async fn readiness_check_endpoint(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    readiness_check(&state.db_pool, &state.redis_pool).await?;

    Ok(Json(serde_json::json!({
        "status": "ready",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Liveness check endpoint for Kubernetes
async fn liveness_check_endpoint() -> Result<Json<serde_json::Value>> {
    liveness_check().await?;

    Ok(Json(serde_json::json!({
        "status": "alive",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Prometheus metrics endpoint
async fn metrics_endpoint(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    crate::metrics::metrics_handler(axum::extract::State(state.metrics)).await
}

#[cfg(feature = "full-platform")]
async fn prometheus_metrics_endpoint(State(state): State<AppState>) -> Result<String> {
    handlers::analytics_v2::get_metrics_handler(State(state)).await
}

#[cfg(not(feature = "full-platform"))]
async fn prometheus_metrics_endpoint(State(state): State<AppState>) -> impl IntoResponse {
    metrics_endpoint(State(state)).await
}

/// Comprehensive monitoring endpoint
async fn comprehensive_monitoring_endpoint(
    State(state): State<AppState>,
) -> Result<Json<MonitoringResponse>> {
    let monitoring_response = state
        .monitoring
        .comprehensive_check(&state.db_pool, &state.redis_pool)
        .await;

    tracing::info!(
        status = ?monitoring_response.health.status,
        memory_usage_percent = monitoring_response.system_metrics.memory_usage_percent,
        cpu_usage_percent = monitoring_response.system_metrics.cpu_usage_percent,
        "Comprehensive monitoring check completed"
    );

    Ok(Json(monitoring_response))
}

#[cfg(all(test, not(feature = "full-platform")))]
mod route_tests {
    use super::{add_full_platform_routes, AppState};
    use axum::Router;

    #[test]
    fn disabled_full_platform_routes_register_without_panicking() {
        let router = Router::<AppState>::new();
        let _ = add_full_platform_routes(router);
    }
}
