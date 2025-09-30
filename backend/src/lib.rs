//! Music Streaming Blocklist Manager Backend
//! 
//! A modular backend service for managing Do-Not-Play lists across music streaming platforms.

use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;
use axum::{
    extract::State,
    response::Json,
    routing::{get, put, delete},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use sqlx::PgPool;

pub mod error;
pub mod validation;
pub mod recovery;
pub mod health;
pub mod metrics;
pub mod monitoring;
pub mod models;
pub mod services;
pub mod middleware;
pub mod database;
pub mod handlers;

// Re-export commonly used types
pub use error::{AppError, Result, ErrorResponse};
pub use validation::{ValidatedJson, validate_email, validate_password, validate_totp_code};
pub use recovery::{RetryConfig, CircuitBreaker, retry_database_operation, retry_redis_operation};
pub use health::{HealthChecker, HealthCheckConfig, HealthCheckResponse, readiness_check, liveness_check, HealthStatus, SystemInfo};
pub use metrics::{MetricsCollector, DatabaseMetrics, RedisMetrics, metrics_handler, RequestTimer};
pub use monitoring::{
    MonitoringSystem, MonitoringConfig, MonitoringResponse, AlertManager, AlertThresholds,
    SystemMetrics, ServiceMetrics, DatabaseServiceMetrics, RedisServiceMetrics, HttpServiceMetrics,
    PerformanceProfiler
};
pub use models::*;
pub use services::{AuthService, RateLimitService, AuditLoggingService, UserService};
pub use services::dnp_list::DnpListService;
pub use database::{
    DatabaseConfig, create_pool, run_migrations, health_check as db_health_check, seed_test_data,
    RedisConfiguration, create_redis_pool, redis_health_check
};
pub use middleware::{create_cors_layer, validate_cors_config};

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
        .route("/register", post(handlers::auth::register_handler)
            .layer(axum::middleware::from_fn_with_state(
                state.rate_limiter.clone(),
                crate::services::registration_rate_limit_middleware,
            )))
        .route("/login", post(handlers::auth::login_handler))
        .route("/refresh", post(handlers::auth::refresh_token_handler));

    // Create protected routes that require authentication
    let protected_routes = Router::new()
        // User routes
        .route("/users/profile", get(handlers::user::get_profile_handler))
        .route("/users/profile", put(handlers::user::update_profile_handler))
        .route("/users/export", get(handlers::user::export_data_handler))
        .route("/users/account", delete(handlers::user::delete_account_handler))
        
        // 2FA routes
        .route("/auth/2fa/setup", post(handlers::auth::setup_2fa_handler))
        .route("/auth/2fa/verify", post(handlers::auth::verify_2fa_handler))
        .route("/auth/2fa/disable", post(handlers::auth::disable_2fa_handler))
        
        // DNP routes
        .route("/dnp/search", get(handlers::dnp::search_artists_handler))
        .route("/dnp/list", get(handlers::dnp::get_dnp_list_handler))
        .route("/dnp/list", post(handlers::dnp::add_to_dnp_handler))
        .route("/dnp/list/:artist_id", delete(handlers::dnp::remove_from_dnp_handler))
        .route("/dnp/list/:artist_id", put(handlers::dnp::update_dnp_entry_handler))
        
        .layer(axum::middleware::from_fn_with_state(
            state.auth_service.clone(),
            crate::middleware::auth::auth_middleware,
        ));

    Router::new()
        // Health check endpoints
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check_endpoint))
        .route("/health/live", get(liveness_check_endpoint))
        
        // Monitoring endpoints
        .route("/metrics", get(metrics_endpoint))
        .route("/monitoring", get(comprehensive_monitoring_endpoint))
        
        // Public API routes
        .nest("/api/v1/auth", auth_routes)
        
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
    use crate::health::{HealthChecker, HealthCheckConfig};
    
    let config = HealthCheckConfig::default();
    let checker = HealthChecker::new(config);
    
    let health_response = checker.check_health(&state.db_pool, &state.redis_pool).await;
    
    tracing::info!(
        status = ?health_response.status,
        correlation_id = %health_response.correlation_id,
        "Health check completed"
    );
    
    Ok(Json(health_response))
}

/// Readiness check endpoint for Kubernetes
async fn readiness_check_endpoint(State(state): State<AppState>) -> Result<Json<serde_json::Value>> {
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
async fn comprehensive_monitoring_endpoint(State(state): State<AppState>) -> Result<Json<MonitoringResponse>> {
    let monitoring_response = state.monitoring
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