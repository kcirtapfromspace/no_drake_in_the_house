use std::collections::HashMap;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::User;
use crate::services::monitoring::{
    MonitoringService, HealthCheckService, Alert, AlertSeverity, SLO, CorrelationId,
};

/// Response for metrics endpoint
#[derive(Serialize)]
pub struct MetricsResponse {
    pub metrics: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Response for health check endpoint
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub services: HashMap<String, serde_json::Value>,
    pub overall_status: String,
}

/// Response for alerts endpoint
#[derive(Serialize)]
pub struct AlertsResponse {
    pub alerts: Vec<Alert>,
    pub total_count: usize,
}

/// Response for SLOs endpoint
#[derive(Serialize)]
pub struct SLOsResponse {
    pub slos: HashMap<String, SLO>,
    pub violations: Vec<String>,
}

/// Query parameters for alerts
#[derive(Deserialize)]
pub struct AlertsQuery {
    pub limit: Option<usize>,
    pub severity: Option<String>,
}

/// Request to create a manual alert
#[derive(Deserialize)]
pub struct CreateAlertRequest {
    pub name: String,
    pub severity: String,
    pub message: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Export Prometheus metrics
pub async fn metrics_handler(
    State(monitoring_service): State<Arc<MonitoringService>>,
) -> Result<String, StatusCode> {
    match monitoring_service.export_metrics() {
        Ok(metrics) => Ok(metrics),
        Err(e) => {
            tracing::error!("Failed to export metrics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get comprehensive health status
pub async fn health_handler(
    State(monitoring_service): State<Arc<MonitoringService>>,
    State(health_check_service): State<Arc<HealthCheckService>>,
    State(db_pool): State<sqlx::PgPool>,
) -> Result<Json<HealthResponse>, StatusCode> {
    // Run health checks
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    health_check_service.run_all_health_checks(&db_pool, &redis_url).await;
    
    // Get all health checks
    let health_checks = monitoring_service.get_health_checks().await;
    let overall_status = monitoring_service.get_overall_health().await;
    
    let services: HashMap<String, serde_json::Value> = health_checks
        .into_iter()
        .map(|(service, check)| {
            (service, serde_json::to_value(check).unwrap_or(serde_json::Value::Null))
        })
        .collect();
    
    let response = HealthResponse {
        status: "ok".to_string(),
        timestamp: chrono::Utc::now(),
        services,
        overall_status: format!("{:?}", overall_status),
    };
    
    Ok(Json(response))
}

/// Get recent alerts
pub async fn alerts_handler(
    State(monitoring_service): State<Arc<MonitoringService>>,
    Query(params): Query<AlertsQuery>,
    Extension(_user): Extension<User>,
) -> Result<Json<AlertsResponse>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let alerts = monitoring_service.get_recent_alerts(limit).await;
    
    // Filter by severity if specified
    let filtered_alerts = if let Some(severity_filter) = params.severity {
        alerts
            .into_iter()
            .filter(|alert| {
                match (severity_filter.as_str(), &alert.severity) {
                    ("critical", AlertSeverity::Critical) => true,
                    ("warning", AlertSeverity::Warning) => true,
                    ("info", AlertSeverity::Info) => true,
                    _ => false,
                }
            })
            .collect()
    } else {
        alerts
    };
    
    let response = AlertsResponse {
        total_count: filtered_alerts.len(),
        alerts: filtered_alerts,
    };
    
    Ok(Json(response))
}

/// Create a manual alert
pub async fn create_alert_handler(
    State(monitoring_service): State<Arc<MonitoringService>>,
    Extension(user): Extension<User>,
    Json(request): Json<CreateAlertRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let severity = match request.severity.as_str() {
        "critical" => AlertSeverity::Critical,
        "warning" => AlertSeverity::Warning,
        "info" => AlertSeverity::Info,
        _ => return Err(StatusCode::BAD_REQUEST),
    };
    
    let alert = Alert {
        id: Uuid::new_v4().to_string(),
        name: request.name,
        severity,
        message: request.message,
        timestamp: chrono::Utc::now(),
        correlation_id: Some(CorrelationId::new().0),
        metadata: request.metadata.unwrap_or_default(),
    };
    
    monitoring_service.trigger_alert(alert.clone()).await;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "alert_id": alert.id,
        "message": "Alert created successfully"
    })))
}

/// Get SLO status
pub async fn slos_handler(
    State(monitoring_service): State<Arc<MonitoringService>>,
    Extension(_user): Extension<User>,
) -> Result<Json<SLOsResponse>, StatusCode> {
    let slos = monitoring_service.get_slos().await;
    
    // Find SLO violations
    let violations: Vec<String> = slos
        .iter()
        .filter(|(_, slo)| slo.current_percentage < slo.target_percentage)
        .map(|(name, _)| name.clone())
        .collect();
    
    let response = SLOsResponse {
        slos,
        violations,
    };
    
    Ok(Json(response))
}

/// Get system performance metrics
pub async fn system_metrics_handler(
    State(monitoring_service): State<Arc<MonitoringService>>,
    Extension(_user): Extension<User>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Update system metrics
    monitoring_service.update_system_metrics().await;
    
    // Return current system status
    Ok(Json(serde_json::json!({
        "timestamp": chrono::Utc::now(),
        "message": "System metrics updated successfully",
        "note": "Use /metrics endpoint for detailed Prometheus metrics"
    })))
}

/// Get monitoring dashboard data
pub async fn dashboard_handler(
    State(monitoring_service): State<Arc<MonitoringService>>,
    Extension(_user): Extension<User>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let health_checks = monitoring_service.get_health_checks().await;
    let recent_alerts = monitoring_service.get_recent_alerts(10).await;
    let slos = monitoring_service.get_slos().await;
    let overall_health = monitoring_service.get_overall_health().await;
    
    Ok(Json(serde_json::json!({
        "timestamp": chrono::Utc::now(),
        "overall_health": format!("{:?}", overall_health),
        "services": health_checks,
        "recent_alerts": recent_alerts,
        "slos": slos,
        "metrics_endpoint": "/api/v1/monitoring/metrics"
    })))
}

/// Test alert endpoint for development/testing
pub async fn test_alert_handler(
    State(monitoring_service): State<Arc<MonitoringService>>,
    Path(severity): Path<String>,
    Extension(_user): Extension<User>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let alert_severity = match severity.as_str() {
        "critical" => AlertSeverity::Critical,
        "warning" => AlertSeverity::Warning,
        "info" => AlertSeverity::Info,
        _ => return Err(StatusCode::BAD_REQUEST),
    };
    
    let alert = Alert {
        id: Uuid::new_v4().to_string(),
        name: format!("Test {} Alert", severity.to_uppercase()),
        severity: alert_severity,
        message: format!("This is a test {} alert triggered manually", severity),
        timestamp: chrono::Utc::now(),
        correlation_id: Some(CorrelationId::new().0),
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("test".to_string(), serde_json::Value::Bool(true));
            metadata.insert("triggered_by".to_string(), serde_json::Value::String("manual".to_string()));
            metadata
        },
    };
    
    monitoring_service.trigger_alert(alert.clone()).await;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "alert_id": alert.id,
        "message": format!("Test {} alert triggered", severity)
    })))
}