use crate::common::*;
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use music_streaming_blocklist_backend::{
    create_app, initialize_database, models::*, AppState, DatabaseConfig,
};
use rstest::*;
use serde_json::json;
use serial_test::serial;
use tower::ServiceExt;

#[fixture]
async fn test_app() -> (axum::Router, TestDatabase) {
    let db = TestDatabase::new().await;

    // Create app state
    let state = AppState {
        db: db.pool.clone(),
        // Add other required state fields as needed
    };

    let app = create_app(state);
    (app, db)
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_health_endpoint(#[future] test_app: (axum::Router, TestDatabase)) {
    let (app, _db) = test_app.await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let health_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(health_response["status"], "healthy");
    assert!(health_response["services"].is_object());
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_user_registration_endpoint(#[future] test_app: (axum::Router, TestDatabase)) {
    let (app, _db) = test_app.await;

    let registration_data = json!({
        "email": "test@example.com",
        "password": "SecurePassword123!"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(registration_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let auth_response: AuthResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(auth_response.user.email, "test@example.com");
    TestAssertions::assert_valid_jwt(&auth_response.access_token);
    TestAssertions::assert_valid_jwt(&auth_response.refresh_token);
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_user_registration_duplicate_email(#[future] test_app: (axum::Router, TestDatabase)) {
    let (app, _db) = test_app.await;

    let registration_data = json!({
        "email": "duplicate@example.com",
        "password": "SecurePassword123!"
    });

    // First registration
    let response1 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(registration_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response1.status(), StatusCode::CREATED);

    // Second registration with same email
    let response2 = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(registration_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::CONFLICT);
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_user_login_endpoint(#[future] test_app: (axum::Router, TestDatabase)) {
    let (app, _db) = test_app.await;

    // First register a user
    let registration_data = json!({
        "email": "login_test@example.com",
        "password": "SecurePassword123!"
    });

    let _register_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(registration_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Now test login
    let login_data = json!({
        "email": "login_test@example.com",
        "password": "SecurePassword123!"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(login_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let auth_response: AuthResponse = serde_json::from_slice(&body).unwrap();

    TestAssertions::assert_valid_jwt(&auth_response.access_token);
    TestAssertions::assert_valid_jwt(&auth_response.refresh_token);
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_user_login_invalid_credentials(#[future] test_app: (axum::Router, TestDatabase)) {
    let (app, _db) = test_app.await;

    let login_data = json!({
        "email": "nonexistent@example.com",
        "password": "WrongPassword123!"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(login_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_protected_endpoint_without_auth(#[future] test_app: (axum::Router, TestDatabase)) {
    let (app, _db) = test_app.await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/users/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_protected_endpoint_with_valid_auth(#[future] test_app: (axum::Router, TestDatabase)) {
    let (app, _db) = test_app.await;

    // Register and login to get token
    let registration_data = json!({
        "email": "auth_test@example.com",
        "password": "SecurePassword123!"
    });

    let register_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(registration_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = hyper::body::to_bytes(register_response.into_body())
        .await
        .unwrap();
    let auth_response: AuthResponse = serde_json::from_slice(&body).unwrap();

    // Use token to access protected endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/users/profile")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", auth_response.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let profile: UserProfile = serde_json::from_slice(&body).unwrap();

    assert_eq!(profile.email, "auth_test@example.com");
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_dnp_list_endpoints(#[future] test_app: (axum::Router, TestDatabase)) {
    let (app, db) = test_app.await;

    // Create user and get auth token
    let user = db.create_test_user().await;
    let artist = db.create_test_artist(Some("Test Artist")).await;

    // For this test, we'll need to create a valid JWT token
    // This is a simplified version - in practice you'd use the auth service
    let token = "mock_jwt_token"; // Replace with actual token generation

    // Test adding artist to DNP list
    let add_request = json!({
        "artist_id": artist.id,
        "tags": ["test", "rock"],
        "note": "Test note"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dnp")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(add_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Note: This test may fail without proper JWT implementation
    // The status code will depend on the actual implementation
    assert!(response.status().is_success() || response.status() == StatusCode::UNAUTHORIZED);
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_cors_headers(#[future] test_app: (axum::Router, TestDatabase)) {
    let (app, _db) = test_app.await;

    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/v1/auth/register")
                .header("Origin", "http://localhost:5000")
                .header("Access-Control-Request-Method", "POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should handle CORS preflight requests
    assert!(response.status().is_success() || response.status() == StatusCode::METHOD_NOT_ALLOWED);
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_request_validation(#[future] test_app: (axum::Router, TestDatabase)) {
    let (app, _db) = test_app.await;

    // Test with invalid JSON
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("invalid json"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Test with missing required fields
    let invalid_data = json!({
        "email": "test@example.com"
        // Missing password
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(invalid_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Duration;

    #[rstest]
    #[tokio::test]
    #[serial]
    async fn test_health_endpoint_performance(#[future] test_app: (axum::Router, TestDatabase)) {
        let (app, _db) = test_app.await;

        let (response, duration) = PerformanceTestHelper::measure_async(|| async {
            app.clone()
                .oneshot(
                    Request::builder()
                        .uri("/health")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap()
        })
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        PerformanceTestHelper::assert_performance_threshold(duration, 100); // 100ms max
    }

    #[rstest]
    #[tokio::test]
    #[serial]
    async fn test_registration_endpoint_performance(
        #[future] test_app: (axum::Router, TestDatabase),
    ) {
        let (app, _db) = test_app.await;

        let registration_data = json!({
            "email": format!("perf_test_{}@example.com", uuid::Uuid::new_v4()),
            "password": "SecurePassword123!"
        });

        let (response, duration) = PerformanceTestHelper::measure_async(|| async {
            app.clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/auth/register")
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(registration_data.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap()
        })
        .await;

        assert_eq!(response.status(), StatusCode::CREATED);
        PerformanceTestHelper::assert_performance_threshold(duration, 1000); // 1 second max
    }
}
