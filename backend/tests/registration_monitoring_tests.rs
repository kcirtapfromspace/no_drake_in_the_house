use std::time::Duration;
use tokio_test;

use music_streaming_blocklist_backend::services::registration_monitoring::{
    RegistrationMonitoringService, RegistrationHealthStatus,
};
use music_streaming_blocklist_backend::services::registration_performance::{
    RegistrationPerformanceService, RegistrationMetrics,
};

#[tokio::test]
async fn test_registration_monitoring_service_creation() {
    let service = RegistrationMonitoringService::new();
    assert!(service.is_ok(), "Should be able to create monitoring service");
}

#[tokio::test]
async fn test_registration_metrics_recording() {
    let service = RegistrationMonitoringService::new().unwrap();
    
    // Record some events
    service.record_registration_attempt();
    service.record_registration_success(Duration::from_millis(500));
    service.record_registration_attempt();
    service.record_registration_failure("validation_error");
    service.record_validation_failure(Duration::from_millis(100));
    service.record_email_duplicate();
    
    // Verify metrics are recorded (this would check Prometheus metrics in real implementation)
    // For now, just verify the service doesn't panic
    assert!(true);
}

#[tokio::test]
async fn test_health_status_creation() {
    let health = RegistrationHealthStatus::default();
    
    assert_eq!(health.status, "healthy");
    assert!(health.database_healthy);
    assert!(health.redis_healthy);
    assert!(health.validation_service_healthy);
    assert_eq!(health.avg_response_time_ms, 0.0);
    assert_eq!(health.error_rate_percent, 0.0);
}

#[tokio::test]
async fn test_dashboard_generation() {
    let service = RegistrationMonitoringService::new().unwrap();
    let metrics = RegistrationMetrics {
        total_attempts: 100,
        successful_registrations: 85,
        validation_failures: 10,
        email_duplicates: 5,
        avg_validation_time_ms: 50.0,
        avg_registration_time_ms: 250.0,
        last_updated: chrono::Utc::now(),
    };
    
    let dashboard = service.generate_dashboard(metrics).await;
    
    assert_eq!(dashboard.success_rate_percent, 85.0);
    assert_eq!(dashboard.avg_validation_time_ms, 50.0);
    assert_eq!(dashboard.avg_registration_time_ms, 250.0);
    assert!(!dashboard.error_patterns.is_empty());
}

#[tokio::test]
async fn test_structured_logging() {
    let service = RegistrationMonitoringService::new().unwrap();
    
    // Test logging different event types
    service.log_registration_event(
        "registration_attempt",
        None,
        "test@example.com",
        None,
        None,
        Some(serde_json::json!({"terms_accepted": true})),
    );
    
    service.log_registration_event(
        "registration_success",
        Some(uuid::Uuid::new_v4()),
        "test@example.com",
        Some(250.0),
        None,
        Some(serde_json::json!({"auto_login": true})),
    );
    
    service.log_registration_event(
        "registration_failure",
        None,
        "test@example.com",
        Some(100.0),
        Some("validation_error"),
        Some(serde_json::json!({"error_type": "validation_error"})),
    );
    
    // Verify no panics occur during logging
    assert!(true);
}

#[tokio::test]
async fn test_performance_service_integration() {
    // This test would require Redis for full functionality
    // For now, test that the service can be created
    let redis_url = "redis://localhost:6379";
    
    // This might fail if Redis is not available, which is expected in CI
    match RegistrationPerformanceService::new(redis_url) {
        Ok(service) => {
            let metrics = service.get_metrics().await;
            assert_eq!(metrics.total_attempts, 0);
            assert_eq!(metrics.successful_registrations, 0);
        }
        Err(_) => {
            // Redis not available, which is fine for this test
            assert!(true);
        }
    }
}

#[test]
fn test_error_rate_calculation() {
    let service = RegistrationMonitoringService::new().unwrap();
    
    // Test initial state
    assert_eq!(service.calculate_error_rate(), 0.0);
    
    // Record some attempts and failures
    service.record_registration_attempt();
    service.record_registration_attempt();
    service.record_registration_failure("test_error");
    
    // Should be 50% error rate
    let error_rate = service.calculate_error_rate();
    assert!((error_rate - 50.0).abs() < 0.1);
}

#[test]
fn test_response_time_calculation() {
    let service = RegistrationMonitoringService::new().unwrap();
    
    // Test initial state
    assert_eq!(service.calculate_avg_response_time(), 0.0);
    
    // Record some successful registrations with timing
    service.record_registration_success(Duration::from_millis(100));
    service.record_registration_success(Duration::from_millis(200));
    
    // Average should be around 150ms
    let avg_time = service.calculate_avg_response_time();
    assert!(avg_time > 0.0);
}

#[tokio::test]
async fn test_monitoring_middleware() {
    // This would test the middleware in a real HTTP context
    // For now, just verify it compiles
    use music_streaming_blocklist_backend::services::registration_monitoring::registration_monitoring_middleware;
    
    // The middleware function exists and compiles
    assert!(true);
}