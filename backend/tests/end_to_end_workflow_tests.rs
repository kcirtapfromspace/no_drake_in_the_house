use music_streaming_blocklist_backend::models::{CreateUserRequest, LoginRequest};
use music_streaming_blocklist_backend::services::auth_simple::AuthService as SimpleAuthService;
use sqlx::{postgres::PgPoolOptions, PgPool};

fn lazy_test_pool() -> PgPool {
    PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy("postgres://test:test@localhost:5432/test_db")
        .expect("lazy postgres pool should be constructible for smoke tests")
}

#[tokio::test]
async fn simple_auth_service_accepts_database_configuration() {
    let auth_service = SimpleAuthService::new().with_database(lazy_test_pool());

    let login = LoginRequest {
        email: "missing@example.com".to_string(),
        password: "password123".to_string(),
        totp_code: None,
    };

    let result = auth_service.login_user(login).await;
    assert!(result.is_err());
}

#[test]
fn workflow_request_models_construct_cleanly() {
    let register = CreateUserRequest {
        email: "user@example.com".to_string(),
        password: "password123".to_string(),
    };

    let login = LoginRequest {
        email: register.email.clone(),
        password: register.password.clone(),
        totp_code: None,
    };

    assert_eq!(register.email, login.email);
    assert_eq!(register.password, login.password);
}
