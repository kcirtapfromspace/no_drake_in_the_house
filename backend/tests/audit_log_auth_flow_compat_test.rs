use chrono::{Duration, Utc};
use music_streaming_blocklist_backend::{
    models::{LoginRequest, RegisterRequest},
    services::AuthService,
};
use sqlx::PgPool;
use std::env;
use uuid::Uuid;

#[tokio::test]
async fn auth_flow_writes_audit_log_rows_on_migrated_schema() {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://postgres:password@localhost:5432/nod195_migrated_check".to_string()
    });

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");
    let auth_service = AuthService::new(pool.clone());

    let has_migrated_shape: bool = sqlx::query_scalar(
        r#"
        SELECT
          EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'audit_log'
              AND column_name = 'user_id'
          )
          AND EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'audit_log'
              AND column_name = 'old_subject_type'
          )
          AND EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'audit_log'
              AND column_name = 'timestamp'
          )
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to inspect audit_log schema");
    assert!(
        has_migrated_shape,
        "This test requires migrated audit_log columns (user_id/old_subject_type/timestamp)"
    );

    let email = format!("nod195_auth_flow_{}@example.com", Uuid::new_v4());
    let password = "Nod195_StrongPass_123!".to_string();

    let before_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
        .fetch_one(&pool)
        .await
        .expect("Failed to read initial audit_log count");
    let start = Utc::now();

    let register_response = auth_service
        .register(RegisterRequest {
            email: email.clone(),
            password: password.clone(),
            confirm_password: password.clone(),
            terms_accepted: true,
        })
        .await
        .expect("Registration flow should succeed");

    let user_id = register_response.user.id;

    let login_response = auth_service
        .login(LoginRequest {
            email: email.clone(),
            password: password.clone(),
            totp_code: None,
        })
        .await
        .expect("Login flow should succeed");

    let _refreshed = auth_service
        .refresh_token(&login_response.refresh_token)
        .await
        .expect("Refresh flow should succeed");

    let end = Utc::now();
    let after_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
        .fetch_one(&pool)
        .await
        .expect("Failed to read final audit_log count");
    assert!(
        after_count > before_count,
        "Expected audit_log count to increase after auth flow"
    );

    let auth_action_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM audit_log
        WHERE user_id = $1
          AND action IN ('user_registered', 'user_login', 'token_refresh')
        "#,
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to read auth action count");
    assert!(
        auth_action_count >= 3,
        "Expected at least 3 audit rows for user_registered + user_login + token_refresh"
    );

    let latest_timestamp: Option<chrono::DateTime<Utc>> = sqlx::query_scalar(
        r#"
        SELECT MAX(timestamp)
        FROM audit_log
        WHERE user_id = $1
          AND action IN ('user_registered', 'user_login', 'token_refresh')
        "#,
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to read latest audit timestamp");

    let latest_timestamp = latest_timestamp.expect("Expected a latest audit timestamp");
    assert!(
        latest_timestamp >= start - Duration::seconds(5)
            && latest_timestamp <= end + Duration::seconds(5),
        "Expected latest audit timestamp to be in execution window"
    );

    // Cleanup test artifacts in FK-safe order.
    let _ = sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
        .bind(user_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM audit_log WHERE user_id = $1")
        .bind(user_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await;
}
