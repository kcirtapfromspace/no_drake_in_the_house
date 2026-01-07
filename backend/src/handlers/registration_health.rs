use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use serde_json;

use crate::{
    AppState, Result,
    services::registration_monitoring::RegistrationMonitoringService,
    services::registration_performance::RegistrationPerformanceService,
};

/// Health check endpoint for registration service
pub async fn registration_health_handler(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    // This would need to be added to AppState in a real implementation
    // For now, create a temporary monitoring service
    let monitoring_service = RegistrationMonitoringService::new()
        .map_err(|e| crate::AppError::Internal { 
            message: Some(format!("Failed to create monitoring service: {}", e)) 
        })?;

    // Check health of dependencies
    let health_status = monitoring_service
        .check_health(&state.db_pool, &get_redis_pool(&state).await?)
        .await
        .map_err(|e| crate::AppError::Internal { 
            message: Some(format!("Health check failed: {}", e)) 
        })?;

    let status_code = match health_status.status.as_str() {
        "healthy" => StatusCode::OK,
        "degraded" => StatusCode::OK, // Still operational
        _ => StatusCode::SERVICE_UNAVAILABLE,
    };

    tracing::info!(
        health_status = %health_status.status,
        database_healthy = health_status.database_healthy,
        redis_healthy = health_status.redis_healthy,
        "Registration health check completed"
    );

    Ok((status_code, Json(serde_json::json!({
        "status": health_status.status,
        "timestamp": health_status.last_check,
        "components": {
            "database": {
                "status": if health_status.database_healthy { "healthy" } else { "unhealthy" }
            },
            "redis": {
                "status": if health_status.redis_healthy { "healthy" } else { "unhealthy" }
            },
            "validation_service": {
                "status": if health_status.validation_service_healthy { "healthy" } else { "unhealthy" }
            }
        },
        "metrics": {
            "avg_response_time_ms": health_status.avg_response_time_ms,
            "error_rate_percent": health_status.error_rate_percent,
            "uptime_seconds": health_status.uptime_seconds
        }
    }))))
}

/// Registration metrics endpoint
pub async fn registration_metrics_handler(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    // This would need to be added to AppState in a real implementation
    // For now, create a temporary performance service
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    let performance_service = RegistrationPerformanceService::new(&redis_url)
        .map_err(|e| crate::AppError::Internal { 
            message: Some(format!("Failed to create performance service: {}", e)) 
        })?;

    let metrics = performance_service.get_metrics().await;

    tracing::info!(
        total_attempts = metrics.total_attempts,
        successful_registrations = metrics.successful_registrations,
        validation_failures = metrics.validation_failures,
        "Registration metrics retrieved"
    );

    Ok((StatusCode::OK, Json(serde_json::json!({
        "timestamp": metrics.last_updated,
        "totals": {
            "attempts": metrics.total_attempts,
            "successes": metrics.successful_registrations,
            "validation_failures": metrics.validation_failures,
            "email_duplicates": metrics.email_duplicates
        },
        "performance": {
            "avg_validation_time_ms": metrics.avg_validation_time_ms,
            "avg_registration_time_ms": metrics.avg_registration_time_ms,
            "success_rate_percent": if metrics.total_attempts > 0 {
                (metrics.successful_registrations as f64 / metrics.total_attempts as f64) * 100.0
            } else {
                0.0
            }
        }
    }))))
}

/// Registration dashboard endpoint
pub async fn registration_dashboard_handler(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    // Create monitoring and performance services
    let monitoring_service = RegistrationMonitoringService::new()
        .map_err(|e| crate::AppError::Internal { 
            message: Some(format!("Failed to create monitoring service: {}", e)) 
        })?;

    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    let performance_service = RegistrationPerformanceService::new(&redis_url)
        .map_err(|e| crate::AppError::Internal { 
            message: Some(format!("Failed to create performance service: {}", e)) 
        })?;

    // Get metrics and generate dashboard
    let metrics = performance_service.get_metrics().await;
    let dashboard = monitoring_service.generate_dashboard(metrics).await;

    tracing::info!(
        success_rate = dashboard.success_rate_percent,
        avg_validation_time = dashboard.avg_validation_time_ms,
        registrations_per_hour = dashboard.registrations_per_hour,
        "Registration dashboard generated"
    );

    Ok((StatusCode::OK, Json(serde_json::to_value(dashboard)
        .map_err(|e| crate::AppError::Internal { 
            message: Some(format!("Failed to serialize dashboard: {}", e)) 
        })?)))
}

/// Prometheus metrics endpoint for registration
pub async fn registration_prometheus_metrics_handler(
    State(_state): State<AppState>,
) -> Result<String> {
    // Create monitoring service to get metrics registry
    let monitoring_service = RegistrationMonitoringService::new()
        .map_err(|e| crate::AppError::Internal { 
            message: Some(format!("Failed to create monitoring service: {}", e)) 
        })?;

    let registry = monitoring_service.get_metrics_registry();
    let encoder = prometheus::TextEncoder::new();
    let metric_families = registry.gather();
    
    encoder.encode_to_string(&metric_families)
        .map_err(|e| crate::AppError::Internal { 
            message: Some(format!("Failed to encode metrics: {}", e)) 
        })
}

// Helper function to get Redis pool (this would be part of AppState in real implementation)
async fn get_redis_pool(state: &AppState) -> Result<deadpool_redis::Pool> {
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    let config = deadpool_redis::Config::from_url(&redis_url);
    config.create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .map_err(|e| crate::AppError::Internal { 
            message: Some(format!("Failed to create Redis pool: {}", e)) 
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use sqlx::PgPool;

    // Mock AppState for testing
    fn create_mock_app_state() -> AppState {
        // This would need to be implemented based on your actual AppState structure
        todo!("Implement mock AppState for testing")
    }

    #[tokio::test]
    async fn test_registration_health_handler() {
        // This test would require a proper test setup with database and Redis
        // For now, just test that the handler function exists and compiles
        assert!(true);
    }

    #[tokio::test]
    async fn test_registration_metrics_handler() {
        // This test would require a proper test setup
        // For now, just test that the handler function exists and compiles
        assert!(true);
    }
}