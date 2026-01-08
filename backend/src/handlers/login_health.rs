use axum::{extract::State, http::StatusCode, response::Json};
use serde_json;

use crate::{services::login_performance::LoginPerformanceService, AppState, Result};

/// Login performance metrics endpoint
pub async fn login_performance_metrics_handler(
    State(_state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    // Create login performance service
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let login_service =
        LoginPerformanceService::new(&redis_url).map_err(|e| crate::AppError::Internal {
            message: Some(format!("Failed to create login performance service: {}", e)),
        })?;

    let metrics = login_service.get_metrics().await;

    tracing::info!(
        total_logins = metrics.total_logins,
        successful_logins = metrics.successful_logins,
        avg_login_time_ms = metrics.avg_login_time_ms,
        cache_hit_rate = metrics.cache_hit_rate,
        "Login performance metrics retrieved"
    );

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "timestamp": metrics.last_updated,
            "performance": {
                "total_logins": metrics.total_logins,
                "successful_logins": metrics.successful_logins,
                "failed_logins": metrics.failed_logins,
                "success_rate_percent": if metrics.total_logins > 0 {
                    (metrics.successful_logins as f64 / metrics.total_logins as f64) * 100.0
                } else {
                    0.0
                },
                "avg_login_time_ms": metrics.avg_login_time_ms,
                "cache_hit_rate_percent": metrics.cache_hit_rate
            },
            "timing_breakdown": {
                "password_verification_ms": metrics.password_verification_time_ms,
                "database_query_ms": metrics.database_query_time_ms,
                "token_generation_ms": metrics.token_generation_time_ms
            },
            "recommendations": generate_performance_recommendations(&metrics)
        })),
    ))
}

/// Clear login caches endpoint (for admin use)
pub async fn clear_login_caches_handler(
    State(_state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let login_service =
        LoginPerformanceService::new(&redis_url).map_err(|e| crate::AppError::Internal {
            message: Some(format!("Failed to create login performance service: {}", e)),
        })?;

    login_service
        .clear_caches()
        .await
        .map_err(|e| crate::AppError::Internal {
            message: Some(format!("Failed to clear caches: {}", e)),
        })?;

    tracing::info!("Login caches cleared successfully");

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Login caches cleared successfully",
            "timestamp": chrono::Utc::now()
        })),
    ))
}

/// Preload frequent users endpoint (for warming up cache)
pub async fn preload_frequent_users_handler(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let login_service =
        LoginPerformanceService::new(&redis_url).map_err(|e| crate::AppError::Internal {
            message: Some(format!("Failed to create login performance service: {}", e)),
        })?;

    login_service
        .preload_frequent_users(&state.db_pool)
        .await
        .map_err(|e| crate::AppError::Internal {
            message: Some(format!("Failed to preload users: {}", e)),
        })?;

    tracing::info!("Frequent users preloaded successfully");

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Frequent users preloaded into cache",
            "timestamp": chrono::Utc::now()
        })),
    ))
}

/// Generate performance recommendations based on metrics
fn generate_performance_recommendations(
    metrics: &crate::services::login_performance::LoginPerformanceMetrics,
) -> Vec<serde_json::Value> {
    let mut recommendations = Vec::new();

    // Check average login time
    if metrics.avg_login_time_ms > 1000.0 {
        recommendations.push(serde_json::json!({
            "type": "performance",
            "severity": "high",
            "message": "Average login time is over 1 second. Consider optimizing password hashing or database queries.",
            "metric": "avg_login_time_ms",
            "current_value": metrics.avg_login_time_ms,
            "recommended_value": 500.0
        }));
    } else if metrics.avg_login_time_ms > 500.0 {
        recommendations.push(serde_json::json!({
            "type": "performance",
            "severity": "medium",
            "message": "Average login time is over 500ms. Consider enabling user caching.",
            "metric": "avg_login_time_ms",
            "current_value": metrics.avg_login_time_ms,
            "recommended_value": 300.0
        }));
    }

    // Check cache hit rate
    if metrics.cache_hit_rate < 50.0 && metrics.total_logins > 100 {
        recommendations.push(serde_json::json!({
            "type": "caching",
            "severity": "medium",
            "message": "Low cache hit rate. Consider preloading frequent users or increasing cache TTL.",
            "metric": "cache_hit_rate",
            "current_value": metrics.cache_hit_rate,
            "recommended_value": 80.0
        }));
    }

    // Check password verification time
    if metrics.password_verification_time_ms > 300.0 {
        recommendations.push(serde_json::json!({
            "type": "security",
            "severity": "low",
            "message": "Password verification is slow. This is expected for security, but consider bcrypt round optimization.",
            "metric": "password_verification_time_ms",
            "current_value": metrics.password_verification_time_ms,
            "recommended_action": "Review bcrypt rounds vs security requirements"
        }));
    }

    // Check database query time
    if metrics.database_query_time_ms > 200.0 {
        recommendations.push(serde_json::json!({
            "type": "database",
            "severity": "medium",
            "message": "Database queries are slow. Consider adding indexes or optimizing queries.",
            "metric": "database_query_time_ms",
            "current_value": metrics.database_query_time_ms,
            "recommended_value": 100.0
        }));
    }

    // Check success rate
    let success_rate = if metrics.total_logins > 0 {
        (metrics.successful_logins as f64 / metrics.total_logins as f64) * 100.0
    } else {
        100.0
    };

    if success_rate < 90.0 && metrics.total_logins > 50 {
        recommendations.push(serde_json::json!({
            "type": "reliability",
            "severity": "high",
            "message": "Low login success rate. Investigate authentication issues.",
            "metric": "success_rate",
            "current_value": success_rate,
            "recommended_value": 95.0
        }));
    }

    // If no issues, provide positive feedback
    if recommendations.is_empty() {
        recommendations.push(serde_json::json!({
            "type": "status",
            "severity": "info",
            "message": "Login performance is optimal. All metrics are within recommended ranges.",
            "status": "healthy"
        }));
    }

    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::login_performance::LoginPerformanceMetrics;

    #[test]
    fn test_performance_recommendations() {
        let mut metrics = LoginPerformanceMetrics::default();

        // Test slow login time
        metrics.avg_login_time_ms = 1500.0;
        metrics.total_logins = 100;
        metrics.successful_logins = 95;

        let recommendations = generate_performance_recommendations(&metrics);
        assert!(!recommendations.is_empty());

        // Should have a high severity recommendation for slow login
        let has_high_severity = recommendations
            .iter()
            .any(|r| r.get("severity").and_then(|s| s.as_str()) == Some("high"));
        assert!(has_high_severity);
    }

    #[test]
    fn test_healthy_recommendations() {
        let mut metrics = LoginPerformanceMetrics::default();

        // Set optimal values
        metrics.avg_login_time_ms = 200.0;
        metrics.cache_hit_rate = 85.0;
        metrics.password_verification_time_ms = 150.0;
        metrics.database_query_time_ms = 50.0;
        metrics.total_logins = 100;
        metrics.successful_logins = 98;

        let recommendations = generate_performance_recommendations(&metrics);

        // Should have a healthy status recommendation
        let has_healthy_status = recommendations.iter().any(|r| {
            r.get("type").and_then(|t| t.as_str()) == Some("status")
                && r.get("status").and_then(|s| s.as_str()) == Some("healthy")
        });
        assert!(has_healthy_status);
    }
}
