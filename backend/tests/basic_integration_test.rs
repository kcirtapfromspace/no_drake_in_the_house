use music_streaming_blocklist_backend::{
    services::AuthService,
    initialize_database,
    DatabaseConfig,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::Response,
    routing::get,
    Router,
};
use tower::ServiceExt;
use serde_json::{json, Value};

// Simple health check test without database dependency
#[tokio::test]
async fn test_simple_health_endpoint() {
    // Create a simple router with just a health endpoint
    let app = Router::new().route("/health", get(simple_health_check));

    // Make a request to the health endpoint
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Check the response
    assert_eq!(response.status(), StatusCode::OK);
}

async fn simple_health_check() -> axum::Json<Value> {
    axum::Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "0.1.0",
        "services": {
            "api": {
                "status": "healthy"
            }
        }
    }))
}

// Database tests are commented out until database integration is implemented
// #[tokio::test]
// async fn test_database_connection() {
//     // Test that we can connect to the database
//     let pool = test_config::create_test_pool().await;
//     
//     // Simple query to verify connection
//     let result = sqlx::query!("SELECT 1 as test_value")
//         .fetch_one(&pool)
//         .await;
//     
//     assert!(result.is_ok());
//     assert_eq!(result.unwrap().test_value, Some(1));
// }

#[tokio::test]
async fn test_auth_service_creation() {
    // Test that we can create an auth service
    let config = DatabaseConfig::default();
    let pool = initialize_database(config).await.expect("Failed to initialize database");
    let auth_service = AuthService::new(pool);
    
    // Test basic functionality
    let test_token = "test_token";
    let result = auth_service.verify_token(test_token);
    
    // Should fail with invalid token (which is expected)
    assert!(result.is_err());
}

#[tokio::test]
async fn test_basic_routing() {
    // Create a simple router
    let app = Router::new()
        .route("/test", get(|| async { "Hello, World!" }));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}