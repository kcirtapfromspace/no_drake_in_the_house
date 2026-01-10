//! Artist Repository
//!
//! Database persistence layer for canonical artists and platform ID mappings.

use super::identity_resolver::CanonicalArtist;
use super::traits::Platform;
use anyhow::{Context, Result};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for persisting artists to the database
pub struct ArtistRepository {
    db_pool: PgPool,
}

impl ArtistRepository {
    /// Create a new artist repository
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Upsert a canonical artist
    /// Returns the artist UUID (existing or newly created)
    pub async fn upsert_artist(&self, artist: &CanonicalArtist) -> Result<Uuid> {
        // Build external_ids JSONB from platform_ids
        let external_ids: serde_json::Value = artist
            .platform_ids
            .iter()
            .map(|(platform, id)| {
                (
                    format!("{:?}", platform).to_lowercase(),
                    serde_json::Value::String(id.clone()),
                )
            })
            .collect::<serde_json::Map<String, serde_json::Value>>()
            .into();

        // Build metadata JSONB
        let metadata = serde_json::json!({
            "genres": artist.genres,
            "country": artist.country,
            "musicbrainz_id": artist.musicbrainz_id,
            "isni": artist.isni,
        });

        // Build aliases JSONB
        let aliases: serde_json::Value = artist
            .aliases
            .iter()
            .map(|alias| {
                serde_json::json!({
                    "name": alias,
                    "source": "sync",
                    "confidence": 1.0
                })
            })
            .collect::<Vec<_>>()
            .into();

        // Try to find existing artist by name or MusicBrainz ID
        let existing = if let Some(mb_id) = &artist.musicbrainz_id {
            sqlx::query_scalar!(
                r#"
                SELECT id FROM artists
                WHERE metadata->>'musicbrainz_id' = $1
                LIMIT 1
                "#,
                mb_id
            )
            .fetch_optional(&self.db_pool)
            .await?
        } else {
            sqlx::query_scalar!(
                r#"
                SELECT id FROM artists
                WHERE LOWER(canonical_name) = LOWER($1)
                LIMIT 1
                "#,
                &artist.name
            )
            .fetch_optional(&self.db_pool)
            .await?
        };

        if let Some(existing_id) = existing {
            // Update existing artist
            sqlx::query!(
                r#"
                UPDATE artists
                SET external_ids = external_ids || $2,
                    metadata = metadata || $3,
                    aliases = aliases || $4
                WHERE id = $1
                "#,
                existing_id,
                external_ids,
                metadata,
                aliases
            )
            .execute(&self.db_pool)
            .await
            .context("Failed to update artist")?;

            tracing::debug!(
                artist_id = %existing_id,
                name = %artist.name,
                "Updated existing artist"
            );

            Ok(existing_id)
        } else {
            // Insert new artist
            let new_id = sqlx::query_scalar!(
                r#"
                INSERT INTO artists (canonical_name, external_ids, metadata, aliases)
                VALUES ($1, $2, $3, $4)
                RETURNING id
                "#,
                &artist.name,
                external_ids,
                metadata,
                aliases
            )
            .fetch_one(&self.db_pool)
            .await
            .context("Failed to insert artist")?;

            tracing::info!(
                artist_id = %new_id,
                name = %artist.name,
                "Created new canonical artist"
            );

            Ok(new_id)
        }
    }

    /// Upsert a platform ID mapping for an artist
    pub async fn upsert_platform_id(
        &self,
        artist_id: Uuid,
        platform: &Platform,
        platform_id: &str,
        sync_run_id: Option<Uuid>,
        confidence: f64,
    ) -> Result<()> {
        let platform_str = format!("{:?}", platform).to_lowercase();

        sqlx::query!(
            r#"
            INSERT INTO artist_platform_ids (artist_id, platform, platform_id, confidence_score, sync_run_id, verification_status)
            VALUES ($1, $2, $3, $4, $5, 'verified')
            ON CONFLICT (platform, platform_id) DO UPDATE SET
                artist_id = EXCLUDED.artist_id,
                confidence_score = EXCLUDED.confidence_score,
                sync_run_id = EXCLUDED.sync_run_id,
                last_verified_at = NOW(),
                updated_at = NOW()
            "#,
            artist_id,
            platform_str,
            platform_id,
            confidence as f32,
            sync_run_id
        )
        .execute(&self.db_pool)
        .await
        .context("Failed to upsert platform ID")?;

        tracing::debug!(
            artist_id = %artist_id,
            platform = %platform_str,
            platform_id = %platform_id,
            "Upserted platform ID mapping"
        );

        Ok(())
    }

    /// Find an artist by platform ID
    pub async fn find_by_platform_id(
        &self,
        platform: &Platform,
        platform_id: &str,
    ) -> Result<Option<Uuid>> {
        let platform_str = format!("{:?}", platform).to_lowercase();

        let result = sqlx::query_scalar!(
            r#"
            SELECT artist_id FROM artist_platform_ids
            WHERE platform = $1 AND platform_id = $2
            LIMIT 1
            "#,
            platform_str,
            platform_id
        )
        .fetch_optional(&self.db_pool)
        .await
        .context("Failed to query platform ID")?;

        Ok(result)
    }

    /// Create a sync run record and return its ID
    pub async fn create_sync_run(
        &self,
        platform: &Platform,
        sync_type: &str,
    ) -> Result<Uuid> {
        let platform_str = format!("{:?}", platform).to_lowercase();

        let run_id = sqlx::query_scalar!(
            r#"
            INSERT INTO platform_sync_runs (platform, sync_type, status, started_at)
            VALUES ($1, $2, 'running', NOW())
            RETURNING id
            "#,
            platform_str,
            sync_type
        )
        .fetch_one(&self.db_pool)
        .await
        .context("Failed to create sync run")?;

        tracing::info!(
            run_id = %run_id,
            platform = %platform_str,
            sync_type = %sync_type,
            "Created sync run"
        );

        Ok(run_id)
    }

    /// Complete a sync run with statistics
    pub async fn complete_sync_run(
        &self,
        run_id: Uuid,
        artists_processed: u64,
        artists_created: u64,
        artists_updated: u64,
        errors: u64,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE platform_sync_runs
            SET status = 'completed',
                completed_at = NOW(),
                artists_processed = $2,
                artists_created = $3,
                artists_updated = $4,
                errors_count = $5
            WHERE id = $1
            "#,
            run_id,
            artists_processed as i32,
            artists_created as i32,
            artists_updated as i32,
            errors as i32
        )
        .execute(&self.db_pool)
        .await
        .context("Failed to complete sync run")?;

        tracing::info!(
            run_id = %run_id,
            artists_processed = artists_processed,
            artists_created = artists_created,
            "Sync run completed"
        );

        Ok(())
    }

    /// Fail a sync run with error
    pub async fn fail_sync_run(&self, run_id: Uuid, error: &str) -> Result<()> {
        let error_log = serde_json::json!([{ "error": error, "timestamp": chrono::Utc::now() }]);

        sqlx::query!(
            r#"
            UPDATE platform_sync_runs
            SET status = 'failed',
                completed_at = NOW(),
                error_log = $2
            WHERE id = $1
            "#,
            run_id,
            error_log
        )
        .execute(&self.db_pool)
        .await
        .context("Failed to update sync run status")?;

        Ok(())
    }

    /// Get total artist count
    pub async fn get_artist_count(&self) -> Result<i64> {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!" FROM artists"#
        )
        .fetch_one(&self.db_pool)
        .await
        .context("Failed to count artists")?;

        Ok(count)
    }

    /// Get artist count by platform
    pub async fn get_artist_count_by_platform(&self, platform: &Platform) -> Result<i64> {
        let platform_str = format!("{:?}", platform).to_lowercase();

        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!" FROM artist_platform_ids
            WHERE platform = $1
            "#,
            platform_str
        )
        .fetch_one(&self.db_pool)
        .await
        .context("Failed to count artists by platform")?;

        Ok(count)
    }
}
