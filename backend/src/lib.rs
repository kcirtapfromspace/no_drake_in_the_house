//! Music Streaming Blocklist Manager Backend
//!
//! A modular backend service for managing Do-Not-Play lists across music streaming platforms.

use axum::{
    extract::State,
    response::Json,
    routing::{delete, get, put},
    Router,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

pub mod config;
pub mod error;
pub mod health;
pub mod metrics;
pub mod models;
pub mod monitoring;
pub mod recovery;
pub mod services;
pub mod validation;

pub mod database;
pub mod handlers;
pub mod middleware;
#[cfg(test)]
pub mod test_database;

// Re-export commonly used types
pub use config::{
    AppConfig, AuthConfig, ConfigError, DatabaseSettings, Environment, OAuthSettings,
    RedisSettings, ServerConfig,
};
pub use database::{
    create_pool, create_redis_pool, health_check as db_health_check, redis_health_check,
    run_migrations, seed_test_data, DatabaseConfig, RedisConfiguration,
};
pub use error::{AppError, ErrorResponse, Result};
pub use health::{
    liveness_check, readiness_check, HealthCheckConfig, HealthCheckResponse, HealthChecker,
    HealthStatus, SystemInfo,
};
pub use metrics::{metrics_handler, DatabaseMetrics, MetricsCollector, RedisMetrics, RequestTimer};
pub use middleware::{create_cors_layer, validate_cors_config};
pub use models::*;
pub use monitoring::{
    AlertManager, AlertThresholds, DatabaseServiceMetrics, HttpServiceMetrics, MonitoringConfig,
    MonitoringResponse, MonitoringSystem, PerformanceProfiler, RedisServiceMetrics, ServiceMetrics,
    SystemMetrics,
};
pub use recovery::{retry_database_operation, retry_redis_operation, CircuitBreaker, RetryConfig};
pub use services::dnp_list::DnpListService;
pub use services::{AuditLoggingService, AuthService, RateLimitService, UserService};
pub use validation::{validate_email, validate_password, validate_totp_code, ValidatedJson};

// Re-export stub services for testing
#[cfg(test)]
pub use services::stubs::*;

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
                crate::services::registration_rate_limit_middleware,
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
        // Library routes
        .route("/library/import", post(handlers::offense::import_library))
        .route("/library/scan", get(handlers::offense::scan_library))
        .route("/library/tracks", get(handlers::offense::get_library))
        // Offense submission routes (protected)
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
        // Artist search route (alias for /dnp/search)
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
        // Graph routes (artist networks)
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
        // Analytics routes (dashboard, trends, reports)
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
        // Health check endpoints
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check_endpoint))
        .route("/health/live", get(liveness_check_endpoint))
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
        .route(
            "/metrics/prometheus",
            get(handlers::analytics_v2::get_metrics_handler),
        )
        .route("/monitoring", get(comprehensive_monitoring_endpoint))
        // Public API routes
        .nest("/api/v1/auth", auth_routes)
        // Public offense browsing routes
        .nest("/api/v1/offenses", offense_public_routes)
        // Protected API routes
        .nest("/api/v1", protected_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(create_cors_layer()),
        )
        .with_state(state)
}

/// Health check endpoint with comprehensive error handling
async fn health_check(State(state): State<AppState>) -> Result<Json<HealthCheckResponse>> {
    use crate::health::{HealthCheckConfig, HealthChecker};

    let config = HealthCheckConfig::default();
    let checker = HealthChecker::new(config);

    let health_response = checker
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

/// Comprehensive monitoring endpoint with health checks and performance metrics
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
