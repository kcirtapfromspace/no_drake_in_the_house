use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::error::Result;
use crate::models::AuthenticatedUser;
use crate::AppState;

/// Category info with subscription status
#[derive(Debug, Serialize)]
pub struct CategoryInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub artist_count: i64,
    pub subscribed: bool,
}

/// Get all categories with user's subscription status
pub async fn get_categories(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<CategoryInfo>>> {
    let categories = get_categories_with_status(&state.db_pool, user.id).await?;
    Ok(Json(categories))
}

/// Subscribe to a category
pub async fn subscribe_category(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(category_id): Path<String>,
) -> Result<impl IntoResponse> {
    let category = validate_category(&category_id)?;

    sqlx::query(
        "INSERT INTO category_subscriptions (user_id, category) VALUES ($1, $2::offense_category) ON CONFLICT (user_id, category) DO NOTHING"
    )
    .bind(user.id)
    .bind(&category)
    .execute(&state.db_pool)
    .await?;

    Ok(StatusCode::OK)
}

/// Unsubscribe from a category
pub async fn unsubscribe_category(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(category_id): Path<String>,
) -> Result<impl IntoResponse> {
    let category = validate_category(&category_id)?;

    sqlx::query(
        "DELETE FROM category_subscriptions WHERE user_id = $1 AND category = $2::offense_category",
    )
    .bind(user.id)
    .bind(&category)
    .execute(&state.db_pool)
    .await?;

    Ok(StatusCode::OK)
}

/// Get artists blocked by user's subscribed categories
#[derive(Debug, Serialize)]
pub struct BlockedArtist {
    pub id: Uuid,
    pub name: String,
    pub category: String,
    pub severity: String,
}

pub async fn get_blocked_artists(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<BlockedArtist>>> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT
            a.id,
            a.canonical_name as name,
            ao.category::text as category,
            ao.severity::text as severity
        FROM category_subscriptions cs
        JOIN artist_offenses ao ON ao.category = cs.category
        JOIN artists a ON a.id = ao.artist_id
        WHERE cs.user_id = $1
        AND ao.status IN ('pending', 'verified')
        ORDER BY a.canonical_name
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await?;

    let artists: Vec<BlockedArtist> = rows
        .iter()
        .map(|row| BlockedArtist {
            id: row.get("id"),
            name: row.get("name"),
            category: row.get("category"),
            severity: row.get("severity"),
        })
        .collect();

    Ok(Json(artists))
}

// Helper functions

fn validate_category(category_id: &str) -> Result<String> {
    let valid_categories = [
        "domestic_violence",
        "sexual_misconduct",
        "sexual_assault",
        "child_abuse",
        "hate_speech",
        "racism",
        "homophobia",
        "antisemitism",
        "violent_crime",
        "drug_trafficking",
        "fraud",
        "animal_abuse",
        "other",
    ];

    if valid_categories.contains(&category_id) {
        Ok(category_id.to_string())
    } else {
        Err(crate::error::AppError::InvalidFieldValue {
            field: "category".to_string(),
            message: format!("Invalid category: {}", category_id),
        })
    }
}

async fn get_categories_with_status(pool: &PgPool, user_id: Uuid) -> Result<Vec<CategoryInfo>> {
    // Define all categories with display info
    let category_defs = vec![
        (
            "sexual_misconduct",
            "Sexual Misconduct",
            "Artists with credible allegations or convictions",
        ),
        (
            "sexual_assault",
            "Sexual Assault",
            "Artists convicted or credibly accused",
        ),
        (
            "domestic_violence",
            "Domestic Violence",
            "Documented domestic violence incidents",
        ),
        (
            "child_abuse",
            "Child Abuse",
            "Artists convicted or accused of child abuse",
        ),
        (
            "violent_crime",
            "Violent Crime",
            "Artists convicted of violent crimes",
        ),
        (
            "drug_trafficking",
            "Drug Trafficking",
            "Artists convicted of drug trafficking",
        ),
        (
            "hate_speech",
            "Hate Speech",
            "Documented hate speech or extremism",
        ),
        (
            "racism",
            "Racism",
            "Documented racist statements or actions",
        ),
        (
            "homophobia",
            "Homophobia",
            "Documented homophobic statements or actions",
        ),
        (
            "antisemitism",
            "Antisemitism",
            "Documented antisemitic statements or actions",
        ),
        ("fraud", "Fraud", "Artists convicted of financial crimes"),
        (
            "animal_abuse",
            "Animal Abuse",
            "Artists convicted of animal abuse",
        ),
        ("other", "Other", "Other documented misconduct"),
    ];

    // Get user's subscriptions
    let sub_rows = sqlx::query(
        "SELECT category::text as category FROM category_subscriptions WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let subscriptions: Vec<String> = sub_rows
        .iter()
        .map(|row| row.get::<String, _>("category"))
        .collect();

    // Get artist counts per category
    let count_rows = sqlx::query(
        r#"
        SELECT category::text as category, COUNT(DISTINCT artist_id) as count
        FROM artist_offenses
        WHERE status IN ('pending', 'verified')
        GROUP BY category
        "#,
    )
    .fetch_all(pool)
    .await?;

    let count_map: std::collections::HashMap<String, i64> = count_rows
        .iter()
        .map(|row| {
            let cat: String = row.get("category");
            let count: i64 = row.get("count");
            (cat, count)
        })
        .collect();

    let categories = category_defs
        .into_iter()
        .map(|(id, name, desc)| CategoryInfo {
            id: id.to_string(),
            name: name.to_string(),
            description: desc.to_string(),
            artist_count: *count_map.get(id).unwrap_or(&0),
            subscribed: subscriptions.contains(&id.to_string()),
        })
        .collect();

    Ok(categories)
}
