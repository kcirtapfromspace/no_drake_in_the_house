use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use music_streaming_blocklist_backend::{
    create_router, initialize_database, AppState, AuditLoggingService, AuthService, DatabaseConfig,
    RateLimitService,
};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
#[ignore] // Requires Redis and PostgreSQL
async fn test_rate_limiting_and_audit_integration() {
    // Initialize database
    let db_config = DatabaseConfig::default();
    let db_pool = initialize_database(db_config).await.unwrap();

    // Initialize Redis URL
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    // Initialize services
    let auth_service = Arc::new(AuthService::new(db_pool.clone()));
    let rate_limiter = Arc::new(RateLimitService::new(&redis_url).unwrap());
    let audit_logger = Arc::new(AuditLoggingService::new(db_pool.clone()));

    // Create app state
    let app_state = AppState {
        db_pool,
        auth_service,
        rate_limiter,
        audit_logger,
    };

    // Create router
    let app = create_router(app_state);

    // Test health endpoint (should not be rate limited heavily)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Test auth endpoint rate limiting by making multiple requests
    for i in 1..=5 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "email": "test@example.com",
                            "password": "testpassword"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // First few requests should get through (even if they fail auth)
        // but should have rate limit headers
        if i <= 3 {
            assert_ne!(response.status(), StatusCode::TOO_MANY_REQUESTS);
            // Check for rate limit headers
            assert!(response.headers().contains_key("x-ratelimit-limit"));
        }
    }

    println!("Rate limiting and audit integration test completed successfully");
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_rate_limit_service_basic() {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let rate_limiter = RateLimitService::new(&redis_url).unwrap();

    // Test basic rate limiting
    let result1 = rate_limiter
        .check_rate_limit("test_ip", "auth")
        .await
        .unwrap();
    assert!(result1.allowed);
    assert_eq!(result1.current_count, 1);

    let result2 = rate_limiter
        .check_rate_limit("test_ip", "auth")
        .await
        .unwrap();
    assert!(result2.allowed);
    assert_eq!(result2.current_count, 2);

    // Test different endpoint type
    let result3 = rate_limiter
        .check_rate_limit("test_ip", "api")
        .await
        .unwrap();
    assert!(result3.allowed);
    assert_eq!(result3.current_count, 1); // Different counter

    println!("Rate limit service basic test completed successfully");
}

#[tokio::test]
#[ignore] // Requires PostgreSQL
async fn test_audit_logging_service_basic() {
    use music_streaming_blocklist_backend::{AuditContext, AuditEventType, AuditSeverity};
    use std::net::IpAddr;

    // Initialize database
    let db_config = DatabaseConfig::default();
    let db_pool = initialize_database(db_config).await.unwrap();

    let audit_logger = AuditLoggingService::new(db_pool);

    // Test basic audit logging
    let context = AuditContext {
        user_id: None,
        session_id: Some("test_session".to_string()),
        ip_address: Some("127.0.0.1".parse::<IpAddr>().unwrap()),
        user_agent: Some("test_agent".to_string()),
        correlation_id: Some("test_correlation".to_string()),
    };

    let entry_id = audit_logger
        .log_security_event(
            AuditEventType::UserLogin,
            AuditSeverity::Info,
            "Test login event".to_string(),
            serde_json::json!({"test": "data"}),
            Some(context),
        )
        .await
        .unwrap();

    assert!(!entry_id.is_nil());

    // Test audit statistics
    let stats = audit_logger.get_audit_statistics(Some(1)).await.unwrap();
    assert!(stats.total_events >= 1);

    println!("Audit logging service basic test completed successfully");
}
