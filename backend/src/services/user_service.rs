use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::models::{CreateUserRequest, User};

pub async fn create_user(pool: &PgPool, request: CreateUserRequest) -> anyhow::Result<User> {
    let user_id = Uuid::new_v4();
    let now = Utc::now();
    
    // Hash password if provided (simplified for now)
    let password_hash = request.password.map(|_| "hashed_password".to_string());
    
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (id, email, password_hash, created_at, updated_at, settings)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, email, password_hash, created_at, updated_at, settings
        "#,
        user_id,
        request.email,
        password_hash,
        now,
        now,
        serde_json::json!({})
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Option<User>> {
    let user = sqlx::query_as!(
        User,
        "SELECT id, email, password_hash, created_at, updated_at, settings FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}