use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use music_streaming_blocklist_backend::{
    AppError, AuthService, Claims, OAuthProviderType, TokenType,
};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Once;
use uuid::Uuid;

const TEST_JWT_SECRET: &str = "ndith_ci_test_secret";
static TEST_ENV_INIT: Once = Once::new();

fn init_test_env() {
    TEST_ENV_INIT.call_once(|| unsafe {
        std::env::set_var("JWT_SECRET", TEST_JWT_SECRET);
    });
}

fn test_pool() -> PgPool {
    PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy("postgres://ndith:ndith@localhost:5432/ndith_test")
        .expect("lazy test pool should be created")
}

fn test_auth_service() -> AuthService {
    init_test_env();
    AuthService::new(test_pool())
}

#[tokio::test]
async fn test_auth_service_creation() {
    let auth_service = test_auth_service();
    assert!(auth_service.get_available_oauth_providers().is_empty());

    let oauth_service = test_auth_service().with_oauth_enabled();
    assert!(oauth_service.get_available_oauth_providers().is_empty());
}

#[tokio::test]
async fn test_validate_access_token_rejects_invalid_token() {
    let auth_service = test_auth_service();
    let err = auth_service
        .validate_access_token("invalid.jwt.token")
        .await
        .expect_err("invalid JWT should be rejected");

    assert!(matches!(err, AppError::TokenInvalid));
}

#[tokio::test]
async fn test_validate_access_token_accepts_valid_token() {
    let auth_service = test_auth_service();
    let user_id = Uuid::new_v4();
    let claims = Claims::new_access_token(user_id, "test@example.com".to_string(), 300);
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
    )
    .expect("token should encode");

    let validated = auth_service
        .validate_access_token(&token)
        .await
        .expect("token should validate");

    assert_eq!(validated.sub, user_id.to_string());
    assert_eq!(validated.email, "test@example.com");
    assert_eq!(validated.token_type, TokenType::Access);
    assert!(!validated.jti.is_empty());
}

#[tokio::test]
async fn test_validate_access_token_rejects_expired_token() {
    let auth_service = test_auth_service();
    let token = encode(
        &Header::default(),
        &Claims {
            sub: Uuid::new_v4().to_string(),
            email: "expired@example.com".to_string(),
            exp: Utc::now().timestamp() - 1,
            iat: Utc::now().timestamp() - 60,
            jti: Uuid::new_v4().to_string(),
            token_type: TokenType::Access,
            scopes: vec!["read".to_string()],
            role: Default::default(),
        },
        &EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
    )
    .expect("expired token should encode");

    let err = auth_service
        .validate_access_token(&token)
        .await
        .expect_err("expired JWT should be rejected");

    assert!(matches!(
        err,
        AppError::TokenInvalid | AppError::TokenExpired
    ));
}

#[tokio::test]
async fn test_initiate_oauth_flow_requires_configured_provider() {
    let auth_service = test_auth_service().with_oauth_enabled();
    let err = auth_service
        .initiate_oauth_flow(
            OAuthProviderType::Google,
            "https://nodrakeinthe.house/auth/callback/google".to_string(),
        )
        .await
        .expect_err("unconfigured OAuth provider should be rejected");

    let message = err.to_string();
    assert!(
        message.contains("not configured") || message.contains("not available"),
        "unexpected error: {message}"
    );
}
