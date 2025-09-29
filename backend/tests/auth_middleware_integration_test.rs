use music_streaming_blocklist_backend::models::{CreateUserRequest, LoginRequest, User, Claims};
use music_streaming_blocklist_backend::services::auth::AuthService;
use music_streaming_blocklist_backend::middleware::auth::auth_middleware;
use axum::{
    body::Body,
    extract::Request,
    http::{header::AUTHORIZATION, Method, StatusCode},
    middleware,
    response::Response,
    routing::get,
    Router,
};
use sqlx::PgPool;
use std::{env, sync::Arc};
use tower::ServiceExt;

async fn get_test_db_pool() -> PgPool {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://kiro:kiro_dev_password@localhost:5432/kiro_dev".to_string());
    
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

async fn protected_handler() -> &'static str {
    "Protected content"
}

async fn user_handler(user: User) -> String {
    format!("Hello, {}!", user.email)
}

#[tokio::test]
async fn test_auth_middleware_with_valid_token() {
    let db_pool = get_test_db_pool().await;
    let auth_service = Arc::new(AuthService::new(db_pool));
    
    // Create test user and get token
    let register_request = CreateUserRequest {
        email: format!("middleware_test_{}@example.com", uuid::Uuid::new_v4()),
        password: "secure_password123".to_string(),
    };
    
    auth_service.register_user(register_request.clone()).await.unwrap();
    
    let login_request = LoginRequest {
        email: register_request.email,
        password: register_request.password,
        totp_code: None,
    };
    
    let token_pair = auth_service.login_user(login_request).await.unwrap();
    
    // Create app with auth middleware
    let app = Router::new()
        .route("/protected", get(protected_handler))
        .route_layer(middleware::from_fn_with_state(
            auth_service.clone(),
            auth_middleware,
        ))
        .with_state(auth_service);

    // Test with valid token
    let request = Request::builder()
        .method(Method::GET)
        .uri("/protected")
        .header(AUTHORIZATION, format!("Bearer {}", token_pair.access_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_auth_middleware_without_token() {
    let db_pool = get_test_db_pool().await;
    let auth_service = Arc::new(AuthService::new(db_pool));
    
    let app = Router::new()
        .route("/protected", get(protected_handler))
        .route_layer(middleware::from_fn_with_state(
            auth_service.clone(),
            auth_middleware,
        ))
        .with_state(auth_service);

    let request = Request::builder()
        .method(Method::GET)
        .uri("/protected")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_middleware_with_invalid_token() {
    let db_pool = get_test_db_pool().await;
    let auth_service = Arc::new(AuthService::new(db_pool));
    
    let app = Router::new()
        .route("/protected", get(protected_handler))
        .route_layer(middleware::from_fn_with_state(
            auth_service.clone(),
            auth_middleware,
        ))
        .with_state(auth_service);

    let request = Request::builder()
        .method(Method::GET)
        .uri("/protected")
        .header(AUTHORIZATION, "Bearer invalid_token")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}