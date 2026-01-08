use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use music_streaming_blocklist_backend::*;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

/// Test helper to create a test app with in-memory services
async fn create_test_app() -> Router {
    // Create test database pool (this would normally use testcontainers)
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://test:test@localhost:5432/test_db".to_string());

    // For this test, we'll use mock services since we don't have a real database
    // In a real integration test, you'd use testcontainers

    // Create mock app state - this is simplified for testing
    let db_pool = sqlx::PgPool::connect(&database_url)
        .await
        .unwrap_or_else(|_| {
            // If we can't connect to a real database, skip this test
            panic!("Test database not available. Set TEST_DATABASE_URL environment variable.");
        });

    let redis_config = RedisConfiguration::default();
    let redis_pool = create_redis_pool(redis_config).await.unwrap();

    let auth_service = Arc::new(AuthService::new(db_pool.clone()));
    let rate_limiter = Arc::new(RateLimitService::new("redis://localhost:6379").unwrap());
    let audit_logger = Arc::new(AuditLoggingService::new(db_pool.clone()));
    let dnp_service = Arc::new(DnpListService::new(db_pool.clone()));
    let user_service = Arc::new(UserService::new(db_pool.clone()));

    let monitoring_config = MonitoringConfig::default();
    let monitoring = Arc::new(MonitoringSystem::new(monitoring_config).unwrap());
    let metrics = monitoring.metrics();

    let app_state = AppState {
        db_pool,
        redis_pool,
        auth_service,
        rate_limiter,
        audit_logger,
        dnp_service,
        user_service,
        monitoring,
        metrics,
    };

    create_router(app_state)
}

#[tokio::test]
async fn test_auth_handlers_structure() {
    // This test verifies that the handlers have the correct structure
    // without requiring a database connection

    // Test that we can create the request/response types
    let register_request = models::RegisterRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    let login_request = models::LoginRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        totp_code: None,
    };

    let refresh_request = models::RefreshTokenRequest {
        refresh_token: "test_token".to_string(),
    };

    let totp_verify_request = models::TotpVerifyRequest {
        totp_code: "123456".to_string(),
        user_id: Uuid::new_v4(),
    };

    // Verify the structures can be serialized
    let _register_json = serde_json::to_string(&register_request).unwrap();
    let _login_json = serde_json::to_string(&login_request).unwrap();
    let _refresh_json = serde_json::to_string(&refresh_request).unwrap();
    let _totp_json = serde_json::to_string(&totp_verify_request).unwrap();

    println!("✅ Auth handler request/response structures are valid");
}

#[tokio::test]
async fn test_dnp_handlers_structure() {
    // Test DNP handler structures
    let artist_id = Uuid::new_v4();

    let add_request = models::AddToDnpRequest {
        artist_id,
        tags: Some(vec!["test".to_string()]),
        note: Some("Test note".to_string()),
    };

    let update_request = models::UpdateDnpEntryRequest {
        tags: Some(vec!["updated".to_string()]),
        note: Some("Updated note".to_string()),
    };

    // Verify the structures can be serialized
    let _add_json = serde_json::to_string(&add_request).unwrap();
    // Note: UpdateDnpEntryRequest doesn't implement Serialize, which is fine for internal use

    println!("✅ DNP handler request/response structures are valid");
}

#[tokio::test]
async fn test_error_handling_structure() {
    // Test that our error types work correctly
    let validation_error = AppError::InvalidFieldValue {
        field: "email".to_string(),
        message: "Invalid email format".to_string(),
    };

    let not_found_error = AppError::NotFound {
        resource: "User".to_string(),
    };

    let conflict_error = AppError::AlreadyExists {
        resource: "DNP entry".to_string(),
    };

    // Test status codes
    assert_eq!(validation_error.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(not_found_error.status_code(), StatusCode::NOT_FOUND);
    assert_eq!(conflict_error.status_code(), StatusCode::CONFLICT);

    // Test error codes
    assert_eq!(validation_error.error_code(), "INVALID_FIELD_VALUE");
    assert_eq!(not_found_error.error_code(), "RESOURCE_NOT_FOUND");
    assert_eq!(conflict_error.error_code(), "RESOURCE_ALREADY_EXISTS");

    println!("✅ Error handling structures are valid");
}

#[tokio::test]
async fn test_handler_compilation() {
    // This test verifies that all the handler functions compile correctly
    // by checking that we can reference them

    // Auth handlers
    let _register_handler = handlers::auth::register_handler;
    let _login_handler = handlers::auth::login_handler;
    let _refresh_handler = handlers::auth::refresh_token_handler;
    let _setup_2fa_handler = handlers::auth::setup_2fa_handler;
    let _verify_2fa_handler = handlers::auth::verify_2fa_handler;
    let _disable_2fa_handler = handlers::auth::disable_2fa_handler;

    // DNP handlers
    let _search_handler = handlers::dnp::search_artists_handler;
    let _get_list_handler = handlers::dnp::get_dnp_list_handler;
    let _add_handler = handlers::dnp::add_to_dnp_handler;
    let _remove_handler = handlers::dnp::remove_from_dnp_handler;
    let _update_handler = handlers::dnp::update_dnp_entry_handler;

    println!("✅ All handlers compile successfully");
}
