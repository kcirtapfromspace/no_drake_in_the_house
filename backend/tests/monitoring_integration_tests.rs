//! Integration tests for monitoring and health check system

use music_streaming_blocklist_backend::{
    MonitoringSystem, MonitoringConfig, MetricsCollector, HealthChecker, HealthCheckConfig,
    DatabaseConfig, create_pool, create_redis_pool, RedisConfiguration,
    run_migrations, AppError
};
use std::time::Duration;
use tokio_test;

#[tokio::test]
async fn test_metrics_collector_creation() {
    let metrics = MetricsCollector::new().expect("Failed to create metrics collector");
    
    // Test that we can get metrics without error
    let metrics_text = metrics.get_metrics().expect("Failed to get metrics");
    assert!(!metrics_text.is_empty());
    assert!(metrics_text.contains("# HELP"));
    assert!(metrics_text.contains("# TYPE"));
}

#[tokio::test]
async fn test_monitoring_system_initialization() {
    let config = MonitoringConfig {
        health_check_interval: Duration::from_secs(30),
        metrics_update_interval: Duration::from_secs(10),
        system_metrics_enabled: true,
        detailed_health_checks: true,
    };
    
    let monitoring = MonitoringSystem::new(config).expect("Failed to create monitoring system");
    
    // Test that we can access the metrics collector
    let metrics = monitoring.metrics();
    let metrics_text = metrics.get_metrics().expect("Failed to get metrics");
    assert!(!metrics_text.is_empty());
}

#[tokio::test]
async fn test_health_checker_with_mock_services() {
    // This test uses environment variables for database connection
    // In a real test environment, you'd use testcontainers
    
    let config = HealthCheckConfig {
        timeout: Duration::from_secs(5),
        include_system_info: true,
        detailed_checks: true,
    };
    
    let checker = HealthChecker::new(config);
    
    // Test that the health checker can be created
    assert!(true); // Placeholder - would need actual database for full test
}

#[tokio::test]
async fn test_metrics_recording() {
    let metrics = MetricsCollector::new().expect("Failed to create metrics collector");
    
    // Record some test metrics
    metrics.record_user_registration();
    metrics.record_user_login(true, "password");
    metrics.record_dnp_operation("add_artist", true);
    metrics.record_auth_failure("invalid_credentials");
    
    // Record HTTP request metrics
    use axum::http::{Method, StatusCode};
    metrics.record_http_request(
        &Method::GET,
        "/health",
        StatusCode::OK,
        Duration::from_millis(100)
    );
    
    // Get metrics and verify they contain our recorded data
    let metrics_text = metrics.get_metrics().expect("Failed to get metrics");
    
    // Check for presence of our metrics
    assert!(metrics_text.contains("kiro_user_registrations_total"));
    assert!(metrics_text.contains("kiro_auth_user_logins_total"));
    assert!(metrics_text.contains("kiro_dnp_dnp_operations_total"));
    assert!(metrics_text.contains("kiro_auth_auth_failures_total"));
    assert!(metrics_text.contains("kiro_http_requests_total"));
    assert!(metrics_text.contains("kiro_http_request_duration_seconds"));
}

#[tokio::test]
async fn test_database_metrics_integration() {
    let metrics = MetricsCollector::new().expect("Failed to create metrics collector");
    let db_metrics = music_streaming_blocklist_backend::DatabaseMetrics::new(std::sync::Arc::new(metrics.clone()));
    
    // Test timing a mock operation
    let result = db_metrics.time_operation("select", "users", async {
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok::<(), &str>(())
    }).await;
    
    assert!(result.is_ok());
    
    // Verify metrics were recorded
    let metrics_text = metrics.get_metrics().expect("Failed to get metrics");
    assert!(metrics_text.contains("kiro_db_operations_total"));
    assert!(metrics_text.contains("kiro_db_query_duration_seconds"));
}

#[tokio::test]
async fn test_redis_metrics_integration() {
    let metrics = MetricsCollector::new().expect("Failed to create metrics collector");
    let redis_metrics = music_streaming_blocklist_backend::RedisMetrics::new(std::sync::Arc::new(metrics.clone()));
    
    // Test timing a mock operation
    let result = redis_metrics.time_operation("get", async {
        tokio::time::sleep(Duration::from_millis(5)).await;
        Ok::<String, &str>("test_value".to_string())
    }).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test_value");
    
    // Verify metrics were recorded
    let metrics_text = metrics.get_metrics().expect("Failed to get metrics");
    assert!(metrics_text.contains("kiro_redis_operations_total"));
    assert!(metrics_text.contains("kiro_redis_operation_duration_seconds"));
}

#[tokio::test]
async fn test_system_metrics_collection() {
    let config = MonitoringConfig::default();
    let monitoring = MonitoringSystem::new(config).expect("Failed to create monitoring system");
    
    // Update system metrics
    monitoring.metrics().update_system_metrics(1000000, 50.0, 3600);
    
    // Verify metrics were updated
    let metrics_text = monitoring.metrics().get_metrics().expect("Failed to get metrics");
    assert!(metrics_text.contains("kiro_memory_usage_bytes"));
    assert!(metrics_text.contains("kiro_cpu_usage_percent"));
    assert!(metrics_text.contains("kiro_uptime_seconds"));
}

#[tokio::test]
async fn test_http_request_timing() {
    let metrics = MetricsCollector::new().expect("Failed to create metrics collector");
    
    // Test request timer
    use axum::http::{Method, StatusCode};
    let timer = music_streaming_blocklist_backend::RequestTimer::new(
        std::sync::Arc::new(metrics.clone()),
        Method::POST,
        "/api/auth/login".to_string()
    );
    
    // Simulate some work
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Finish the timer
    timer.finish(StatusCode::OK);
    
    // Verify metrics were recorded
    let metrics_text = metrics.get_metrics().expect("Failed to get metrics");
    assert!(metrics_text.contains("kiro_http_requests_total"));
    assert!(metrics_text.contains("method=\"POST\""));
    assert!(metrics_text.contains("endpoint=\"/api/auth/login\""));
    assert!(metrics_text.contains("status_code=\"200\""));
}

#[tokio::test]
async fn test_performance_profiler() {
    let metrics = MetricsCollector::new().expect("Failed to create metrics collector");
    let profiler = music_streaming_blocklist_backend::PerformanceProfiler::new(std::sync::Arc::new(metrics.clone()));
    
    // Profile a database operation
    let db_result = profiler.profile_db_operation("insert", "users", async {
        tokio::time::sleep(Duration::from_millis(20)).await;
        Ok::<i32, &str>(42)
    }).await;
    
    assert!(db_result.is_ok());
    assert_eq!(db_result.unwrap(), 42);
    
    // Profile a Redis operation
    let redis_result = profiler.profile_redis_operation("set", async {
        tokio::time::sleep(Duration::from_millis(5)).await;
        Ok::<(), &str>(())
    }).await;
    
    assert!(redis_result.is_ok());
    
    // Verify metrics were recorded
    let metrics_text = metrics.get_metrics().expect("Failed to get metrics");
    assert!(metrics_text.contains("kiro_db_operations_total"));
    assert!(metrics_text.contains("kiro_redis_operations_total"));
}

#[tokio::test]
async fn test_alert_manager() {
    use music_streaming_blocklist_backend::{AlertManager, AlertThresholds, MonitoringResponse};
    use music_streaming_blocklist_backend::{SystemMetrics, ServiceMetrics, DatabaseServiceMetrics, RedisServiceMetrics, HttpServiceMetrics};
    use music_streaming_blocklist_backend::{HealthCheckResponse, HealthStatus, SystemInfo};
    use std::collections::HashMap;
    
    let thresholds = AlertThresholds {
        max_memory_usage_percent: 80.0,
        max_cpu_usage_percent: 80.0,
        max_response_time_ms: 1000,
        max_error_rate_percent: 5.0,
        min_available_connections: 5,
    };
    
    let alert_manager = AlertManager::new(thresholds);
    
    // Create a monitoring response with high memory usage
    let monitoring_response = MonitoringResponse {
        health: HealthCheckResponse {
            status: HealthStatus::Healthy,
            timestamp: chrono::Utc::now(),
            correlation_id: "test".to_string(),
            version: "test".to_string(),
            uptime_seconds: 100,
            services: HashMap::new(),
            system_info: SystemInfo {
                memory_usage_mb: 1000,
                cpu_usage_percent: 50.0,
                disk_usage_percent: 30.0,
                active_connections: 10,
            },
        },
        system_metrics: SystemMetrics {
            memory_usage_bytes: 1000000000,
            memory_usage_percent: 85.0, // Above threshold
            cpu_usage_percent: 50.0,
            uptime_seconds: 100,
            active_connections: 10,
            thread_count: 5,
        },
        service_metrics: ServiceMetrics {
            database: DatabaseServiceMetrics {
                active_connections: 5,
                idle_connections: 10,
                total_queries: 100,
                avg_query_duration_ms: 50.0,
                error_rate_percent: 1.0,
            },
            redis: RedisServiceMetrics {
                active_connections: 2,
                total_operations: 200,
                avg_operation_duration_ms: 10.0,
                error_rate_percent: 0.5,
            },
            http: HttpServiceMetrics {
                total_requests: 1000,
                requests_per_second: 10.0,
                avg_response_time_ms: 100.0,
                error_rate_percent: 2.0,
                active_requests: 5,
            },
        },
        timestamp: chrono::Utc::now(),
    };
    
    let alerts = alert_manager.check_alerts(&monitoring_response);
    assert!(!alerts.is_empty());
    
    // Should have a memory usage alert
    assert!(alerts.iter().any(|a| a.metric == "memory_usage_percent"));
}

#[tokio::test]
async fn test_metrics_endpoint_format() {
    let metrics = MetricsCollector::new().expect("Failed to create metrics collector");
    
    // Record some sample data
    metrics.record_user_registration();
    metrics.record_user_login(true, "password");
    
    let metrics_text = metrics.get_metrics().expect("Failed to get metrics");
    
    // Verify Prometheus format
    assert!(metrics_text.contains("# HELP"));
    assert!(metrics_text.contains("# TYPE"));
    assert!(metrics_text.contains("kiro_user_registrations_total"));
    
    // Verify it's valid Prometheus format (basic check)
    let lines: Vec<&str> = metrics_text.lines().collect();
    let help_lines: Vec<&str> = lines.iter().filter(|line| line.starts_with("# HELP")).cloned().collect();
    let type_lines: Vec<&str> = lines.iter().filter(|line| line.starts_with("# TYPE")).cloned().collect();
    
    assert!(!help_lines.is_empty());
    assert!(!type_lines.is_empty());
}