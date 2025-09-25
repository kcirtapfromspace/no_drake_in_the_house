use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::models::{Artist, SearchArtistRequest};

pub async fn search_artists(pool: &PgPool, request: SearchArtistRequest) -> anyhow::Result<Vec<Artist>> {
    let limit = request.limit.unwrap_or(20).min(100);
    let search_term = format!("%{}%", request.query);
    
    let artists = sqlx::query_as!(
        Artist,
        r#"
        SELECT id, canonical_name, canonical_artist_id, external_ids, metadata, aliases, created_at
        FROM artists 
        WHERE canonical_name ILIKE $1
        ORDER BY canonical_name
        LIMIT $2
        "#,
        search_term,
        limit as i64
    )
    .fetch_all(pool)
    .await?;

    Ok(artists)
}

pub async fn create_artist(
    pool: &PgPool,
    canonical_name: String,
    external_ids: serde_json::Value,
    metadata: serde_json::Value,
) -> anyhow::Result<Artist> {
    let artist_id = Uuid::new_v4();
    let now = Utc::now();
    
    let artist = sqlx::query_as!(
        Artist,
        r#"
        INSERT INTO artists (id, canonical_name, canonical_artist_id, external_ids, metadata, aliases, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, canonical_name, canonical_artist_id, external_ids, metadata, aliases, created_at
        "#,
        artist_id,
        canonical_name,
        None::<Uuid>,
        external_ids,
        metadata,
        serde_json::json!({}),
        now
    )
    .fetch_one(pool)
    .await?;

    Ok(artist)
}

pub async fn get_artist_by_id(pool: &PgPool, artist_id: Uuid) -> anyhow::Result<Option<Artist>> {
    let artist = sqlx::query_as!(
        Artist,
        r#"
        SELECT id, canonical_name, canonical_artist_id, external_ids, metadata, aliases, created_at
        FROM artists 
        WHERE id = $1
        "#,
        artist_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(artist)
}