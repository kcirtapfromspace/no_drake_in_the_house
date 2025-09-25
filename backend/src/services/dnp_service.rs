use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::models::{AddArtistToDnpRequest, UserArtistBlock};

pub async fn get_user_dnp_list(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<UserArtistBlock>> {
    let blocks = sqlx::query_as!(
        UserArtistBlock,
        r#"
        SELECT user_id, artist_id, tags, note, created_at
        FROM user_artist_blocks 
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(blocks)
}

pub async fn add_artist_to_dnp(
    pool: &PgPool,
    user_id: Uuid,
    request: AddArtistToDnpRequest,
) -> anyhow::Result<UserArtistBlock> {
    let now = Utc::now();
    let tags = request.tags.unwrap_or_default();
    
    let block = sqlx::query_as!(
        UserArtistBlock,
        r#"
        INSERT INTO user_artist_blocks (user_id, artist_id, tags, note, created_at)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (user_id, artist_id) 
        DO UPDATE SET 
            tags = EXCLUDED.tags,
            note = EXCLUDED.note,
            created_at = EXCLUDED.created_at
        RETURNING user_id, artist_id, tags, note, created_at
        "#,
        user_id,
        request.artist_id,
        &tags,
        request.note,
        now
    )
    .fetch_one(pool)
    .await?;

    Ok(block)
}

pub async fn remove_artist_from_dnp(
    pool: &PgPool,
    user_id: Uuid,
    artist_id: Uuid,
) -> anyhow::Result<bool> {
    let result = sqlx::query!(
        "DELETE FROM user_artist_blocks WHERE user_id = $1 AND artist_id = $2",
        user_id,
        artist_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}