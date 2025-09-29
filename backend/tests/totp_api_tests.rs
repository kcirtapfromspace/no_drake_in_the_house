use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use hyper::Method;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use music_streaming_blocklist_backend::{
    create_router, AppState, AuthService,
};
use std::sync::Arc;

// Helper function to create test app state
fn create_test_app_state() -> AppState {
    let database_url = "postgres://mock:mock@localhost:5432/mock";
    let pool = sqlx::PgPool::connect_lazy(&database_url).expect("Failed to create pool");
    let auth_service = Arc::new(AuthService::new(pool.clone()));
    
    AppState {
        db_pool: pool,
        auth_service,
    }
}

// Simple test to verify the 2FA endpoints are properly configured
#[tokio::test]
async fn test_totp_endpoints_exist() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    // Test TOTP setup endpoint exists
    let setup_request = Request::builder()
        .method(Method::POST)
        .uri("/auth/totp/setup")
        .header("content-type", "application/json")
        .body(Body::from(json!({"user_id": Uuid::new_v4().to_string()}).to_string()))
        .unwrap();
    
    let response = app.clone().oneshot(setup_request).await.unwrap();
    // We expect a 400 (bad request) because we don't have a real database connection,
    // but this proves the endpoint exists and is routed correctly
    assert!(response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::INTERNAL_SERVER_ERROR);
    
    // Test TOTP enable endpoint exists
    let enable_request = Request::builder()
        .method(Method::POST)
        .uri("/auth/totp/enable")
        .header("content-type", "application/json")
        .body(Body::from(json!({"user_id": Uuid::new_v4().to_string(), "totp_code": "123456"}).to_string()))
        .unwrap();
    
    let response = app.clone().oneshot(enable_request).await.unwrap();
    assert!(response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::INTERNAL_SERVER_ERROR);
    
    // Test TOTP disable endpoint exists
    let disable_request = Request::builder()
        .method(Method::POST)
        .uri("/auth/totp/disable")
        .header("content-type", "application/json")
        .body(Body::from(json!({"user_id": Uuid::new_v4().to_string(), "totp_code": "123456"}).to_string()))
        .unwrap();
    
    let response = app.clone().oneshot(disable_request).await.unwrap();
    assert!(response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::INTERNAL_SERVER_ERROR);
    
    // Test TOTP status endpoint exists
    let status_request = Request::builder()
        .method(Method::GET)
        .uri("/auth/totp/status")
        .header("content-type", "application/json")
        .body(Body::from(json!({"user_id": Uuid::new_v4().to_string()}).to_string()))
        .unwrap();
    
    let response = app.clone().oneshot(status_request).await.unwrap();
    assert!(response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_totp_setup_invalid_request() {
    let app_state = create_test_app_state();
    
    let app = create_router(app_state);
    
    // Test with invalid user_id format
    let setup_request = Request::builder()
        .method(Method::POST)
        .uri("/auth/totp/setup")
        .header("content-type", "application/json")
        .body(Body::from(json!({"user_id": "invalid-uuid"}).to_string()))
        .unwrap();
    
    let response = app.oneshot(setup_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_totp_enable_missing_fields() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    // Test with missing totp_code
    let enable_request = Request::builder()
        .method(Method::POST)
        .uri("/auth/totp/enable")
        .header("content-type", "application/json")
        .body(Body::from(json!({"user_id": Uuid::new_v4().to_string()}).to_string()))
        .unwrap();
    
    let response = app.oneshot(enable_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// Helper function to generate a valid TOTP code for testing
fn generate_test_totp_code(secret: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha1::Sha1;
    use chrono::Utc;
    
    let secret_bytes = base32::decode(base32::Alphabet::Rfc4648 { padding: true }, secret)
        .expect("Invalid test secret");
    
    let current_time = Utc::now().timestamp() as u64;
    let time_step = current_time / 30;
    let time_bytes = time_step.to_be_bytes();
    
    type HmacSha1 = Hmac<Sha1>;
    let mut mac = HmacSha1::new_from_slice(&secret_bytes).expect("Invalid secret");
    mac.update(&time_bytes);
    let result = mac.finalize().into_bytes();
    
    let offset = (result[result.len() - 1] & 0xf) as usize;
    let code = ((result[offset] as u32 & 0x7f) << 24)
        | ((result[offset + 1] as u32 & 0xff) << 16)
        | ((result[offset + 2] as u32 & 0xff) << 8)
        | (result[offset + 3] as u32 & 0xff);
    
    format!("{:06}", code % 1000000)
}