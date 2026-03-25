use chrono::{DateTime, Utc};
use ndith_core::models::{
    PlaylistSummary, PlaylistTrackWithStatus, UpsertPlaylist, UpsertPlaylistTrack,
};
use sqlx::PgPool;
use uuid::Uuid;

type Result<T> = std::result::Result<T, ndith_core::error::AppError>;

pub struct PlaylistRepository<'a> {
    db: &'a PgPool,
}

impl<'a> PlaylistRepository<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self { db }
    }

    /// Upsert a playlist entity and return its DB id.
    pub async fn upsert_playlist(
        &self,
        user_id: Uuid,
        provider: &str,
        playlist: &UpsertPlaylist,
    ) -> Result<Uuid> {
        let row: (Uuid,) = sqlx::query_as(
            r#"
            INSERT INTO playlists (
                user_id, provider, provider_playlist_id, name, description,
                image_url, owner_name, owner_id, is_public, is_collaborative,
                source_type, provider_track_count, snapshot_id, last_synced
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW())
            ON CONFLICT (user_id, provider, provider_playlist_id)
            DO UPDATE SET
                name = EXCLUDED.name,
                description = EXCLUDED.description,
                image_url = EXCLUDED.image_url,
                owner_name = EXCLUDED.owner_name,
                owner_id = EXCLUDED.owner_id,
                is_public = EXCLUDED.is_public,
                is_collaborative = EXCLUDED.is_collaborative,
                source_type = EXCLUDED.source_type,
                provider_track_count = EXCLUDED.provider_track_count,
                snapshot_id = EXCLUDED.snapshot_id,
                last_synced = NOW()
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(provider)
        .bind(&playlist.provider_playlist_id)
        .bind(&playlist.name)
        .bind(&playlist.description)
        .bind(&playlist.image_url)
        .bind(&playlist.owner_name)
        .bind(&playlist.owner_id)
        .bind(playlist.is_public)
        .bind(playlist.is_collaborative)
        .bind(&playlist.source_type)
        .bind(playlist.provider_track_count)
        .bind(&playlist.snapshot_id)
        .fetch_one(self.db)
        .await
        .map_err(ndith_core::error::AppError::DatabaseQueryFailed)?;

        Ok(row.0)
    }

    /// Replace all tracks in a playlist (delete-then-insert in a transaction).
    pub async fn replace_playlist_tracks(
        &self,
        playlist_id: Uuid,
        tracks: &[UpsertPlaylistTrack],
    ) -> Result<usize> {
        let mut tx = self
            .db
            .begin()
            .await
            .map_err(ndith_core::error::AppError::DatabaseQueryFailed)?;

        // Delete existing tracks
        sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = $1")
            .bind(playlist_id)
            .execute(&mut *tx)
            .await
            .map_err(ndith_core::error::AppError::DatabaseQueryFailed)?;

        if tracks.is_empty() {
            tx.commit()
                .await
                .map_err(ndith_core::error::AppError::DatabaseQueryFailed)?;
            return Ok(0);
        }

        // Try to resolve artist_id for each track
        for track in tracks {
            let artist_id: Option<Uuid> = sqlx::query_scalar(
                "SELECT id FROM artists WHERE LOWER(canonical_name) = LOWER($1) LIMIT 1",
            )
            .bind(&track.artist_name)
            .fetch_optional(&mut *tx)
            .await
            .map_err(ndith_core::error::AppError::DatabaseQueryFailed)?;

            sqlx::query(
                r#"
                INSERT INTO playlist_tracks (
                    playlist_id, provider_track_id, track_name, album_name,
                    artist_id, artist_name, position, added_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(playlist_id)
            .bind(&track.provider_track_id)
            .bind(&track.track_name)
            .bind(&track.album_name)
            .bind(artist_id)
            .bind(&track.artist_name)
            .bind(track.position)
            .bind(track.added_at)
            .execute(&mut *tx)
            .await
            .map_err(ndith_core::error::AppError::DatabaseQueryFailed)?;
        }

        tx.commit()
            .await
            .map_err(ndith_core::error::AppError::DatabaseQueryFailed)?;

        Ok(tracks.len())
    }

    /// Delete playlists not refreshed since `sync_ts` for a given user+provider.
    pub async fn delete_stale_playlists(
        &self,
        user_id: Uuid,
        provider: &str,
        sync_ts: DateTime<Utc>,
    ) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM playlists
            WHERE user_id = $1
              AND provider = $2
              AND last_synced < $3
            "#,
        )
        .bind(user_id)
        .bind(provider)
        .bind(sync_ts)
        .execute(self.db)
        .await
        .map_err(ndith_core::error::AppError::DatabaseQueryFailed)?;

        Ok(result.rows_affected())
    }

    /// List all playlists for a user with computed flagging grades.
    pub async fn list_playlists_with_grades(
        &self,
        user_id: Uuid,
        provider: Option<&str>,
    ) -> Result<Vec<PlaylistSummary>> {
        #[derive(sqlx::FromRow)]
        struct RawRow {
            id: Uuid,
            provider: String,
            provider_playlist_id: String,
            name: String,
            description: Option<String>,
            image_url: Option<String>,
            owner_name: Option<String>,
            is_public: Option<bool>,
            source_type: String,
            last_synced: Option<DateTime<Utc>>,
            total_tracks: i64,
            unique_artists: i64,
            flagged_tracks: i64,
        }

        let rows = if let Some(prov) = provider {
            sqlx::query_as::<_, RawRow>(
                r#"
                SELECT
                    p.id,
                    p.provider,
                    p.provider_playlist_id,
                    p.name,
                    p.description,
                    p.image_url,
                    p.owner_name,
                    p.is_public,
                    p.source_type,
                    p.last_synced,
                    COUNT(pt.id)::bigint AS total_tracks,
                    COUNT(DISTINCT pt.artist_name) FILTER (
                        WHERE pt.artist_name IS NOT NULL AND TRIM(pt.artist_name) <> ''
                    )::bigint AS unique_artists,
                    COUNT(pt.id) FILTER (
                        WHERE EXISTS (
                            SELECT 1 FROM artists a
                            JOIN artist_offenses ao ON ao.artist_id = a.id AND ao.status = 'verified'
                            WHERE a.id = pt.artist_id
                               OR (pt.artist_name IS NOT NULL AND LOWER(pt.artist_name) = LOWER(a.canonical_name))
                        )
                        OR EXISTS (
                            SELECT 1 FROM user_artist_blocks uab
                            WHERE uab.user_id = p.user_id AND uab.artist_id = pt.artist_id
                        )
                    )::bigint AS flagged_tracks
                FROM playlists p
                LEFT JOIN playlist_tracks pt ON pt.playlist_id = p.id
                WHERE p.user_id = $1
                  AND p.provider = $2
                GROUP BY p.id
                ORDER BY p.name
                "#,
            )
            .bind(user_id)
            .bind(prov)
            .fetch_all(self.db)
            .await
            .map_err(ndith_core::error::AppError::DatabaseQueryFailed)?
        } else {
            sqlx::query_as::<_, RawRow>(
                r#"
                SELECT
                    p.id,
                    p.provider,
                    p.provider_playlist_id,
                    p.name,
                    p.description,
                    p.image_url,
                    p.owner_name,
                    p.is_public,
                    p.source_type,
                    p.last_synced,
                    COUNT(pt.id)::bigint AS total_tracks,
                    COUNT(DISTINCT pt.artist_name) FILTER (
                        WHERE pt.artist_name IS NOT NULL AND TRIM(pt.artist_name) <> ''
                    )::bigint AS unique_artists,
                    COUNT(pt.id) FILTER (
                        WHERE EXISTS (
                            SELECT 1 FROM artists a
                            JOIN artist_offenses ao ON ao.artist_id = a.id AND ao.status = 'verified'
                            WHERE a.id = pt.artist_id
                               OR (pt.artist_name IS NOT NULL AND LOWER(pt.artist_name) = LOWER(a.canonical_name))
                        )
                        OR EXISTS (
                            SELECT 1 FROM user_artist_blocks uab
                            WHERE uab.user_id = p.user_id AND uab.artist_id = pt.artist_id
                        )
                    )::bigint AS flagged_tracks
                FROM playlists p
                LEFT JOIN playlist_tracks pt ON pt.playlist_id = p.id
                WHERE p.user_id = $1
                GROUP BY p.id
                ORDER BY p.name
                "#,
            )
            .bind(user_id)
            .fetch_all(self.db)
            .await
            .map_err(ndith_core::error::AppError::DatabaseQueryFailed)?
        };

        // Build summaries with grades and flagged artist names
        let mut summaries = Vec::with_capacity(rows.len());
        for r in rows {
            let clean_ratio = if r.total_tracks > 0 {
                (r.total_tracks - r.flagged_tracks) as f64 / r.total_tracks as f64
            } else {
                1.0
            };

            // Fetch flagged artist names for this playlist
            let flagged_artists: Vec<String> = sqlx::query_scalar(
                r#"
                SELECT DISTINCT pt.artist_name
                FROM playlist_tracks pt
                WHERE pt.playlist_id = $1
                  AND pt.artist_name IS NOT NULL
                  AND TRIM(pt.artist_name) <> ''
                  AND (
                      EXISTS (
                          SELECT 1 FROM artists a
                          JOIN artist_offenses ao ON ao.artist_id = a.id AND ao.status = 'verified'
                          WHERE a.id = pt.artist_id
                             OR LOWER(pt.artist_name) = LOWER(a.canonical_name)
                      )
                      OR EXISTS (
                          SELECT 1 FROM user_artist_blocks uab
                          WHERE uab.user_id = $2 AND uab.artist_id = pt.artist_id
                      )
                  )
                "#,
            )
            .bind(r.id)
            .bind(user_id)
            .fetch_all(self.db)
            .await
            .map_err(ndith_core::error::AppError::DatabaseQueryFailed)?;

            // Fetch cover images from artist metadata
            let cover_images: Vec<String> = sqlx::query_scalar(
                r#"
                SELECT DISTINCT a.metadata->>'image_url'
                FROM playlist_tracks pt
                JOIN artists a ON a.id = pt.artist_id
                WHERE pt.playlist_id = $1
                  AND a.metadata->>'image_url' IS NOT NULL
                  AND a.metadata->>'image_url' <> ''
                LIMIT 4
                "#,
            )
            .bind(r.id)
            .fetch_all(self.db)
            .await
            .map_err(ndith_core::error::AppError::DatabaseQueryFailed)?;

            summaries.push(PlaylistSummary {
                id: r.id,
                provider: r.provider,
                provider_playlist_id: r.provider_playlist_id,
                name: r.name,
                description: r.description,
                image_url: r.image_url,
                owner_name: r.owner_name,
                is_public: r.is_public,
                source_type: r.source_type,
                total_tracks: r.total_tracks,
                flagged_tracks: r.flagged_tracks,
                clean_ratio,
                grade: grade_from_ratio(clean_ratio),
                unique_artists: r.unique_artists,
                flagged_artists,
                last_synced: r.last_synced,
                cover_images,
            });
        }

        // Sort: worst grade first, then alphabetically
        summaries.sort_by(|a, b| {
            grade_order(&a.grade)
                .cmp(&grade_order(&b.grade))
                .then(a.name.cmp(&b.name))
        });

        Ok(summaries)
    }

    /// Get tracks for a specific playlist with offense/block status.
    pub async fn get_playlist_tracks_with_status(
        &self,
        playlist_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<PlaylistTrackWithStatus>> {
        #[derive(sqlx::FromRow)]
        struct RawTrack {
            id: Uuid,
            position: i32,
            provider_track_id: String,
            track_name: Option<String>,
            album_name: Option<String>,
            artist_id: Option<Uuid>,
            artist_name: Option<String>,
            artist_image_url: Option<String>,
            is_offending: bool,
            is_blocked: bool,
            added_at: Option<DateTime<Utc>>,
        }

        let rows = sqlx::query_as::<_, RawTrack>(
            r#"
            SELECT
                pt.id,
                pt.position,
                pt.provider_track_id,
                pt.track_name,
                pt.album_name,
                pt.artist_id,
                pt.artist_name,
                a_img.metadata->>'image_url' AS artist_image_url,
                EXISTS (
                    SELECT 1 FROM artists a
                    JOIN artist_offenses ao ON ao.artist_id = a.id AND ao.status = 'verified'
                    WHERE a.id = pt.artist_id
                       OR (pt.artist_name IS NOT NULL AND LOWER(pt.artist_name) = LOWER(a.canonical_name))
                ) AS is_offending,
                EXISTS (
                    SELECT 1 FROM user_artist_blocks uab
                    WHERE uab.user_id = $2 AND uab.artist_id = pt.artist_id
                ) AS is_blocked,
                pt.added_at
            FROM playlist_tracks pt
            LEFT JOIN artists a_img ON a_img.id = pt.artist_id
            WHERE pt.playlist_id = $1
            ORDER BY pt.position
            "#,
        )
        .bind(playlist_id)
        .bind(user_id)
        .fetch_all(self.db)
        .await
        .map_err(ndith_core::error::AppError::DatabaseQueryFailed)?;

        let tracks = rows
            .into_iter()
            .map(|r| {
                let status = if r.is_blocked {
                    "blocked"
                } else if r.is_offending {
                    "flagged"
                } else {
                    "clean"
                };
                PlaylistTrackWithStatus {
                    id: r.id,
                    position: r.position,
                    provider_track_id: r.provider_track_id,
                    track_name: r.track_name.unwrap_or_else(|| "Unknown".to_string()),
                    album_name: r.album_name,
                    artist_id: r.artist_id,
                    artist_name: r.artist_name.unwrap_or_else(|| "Unknown".to_string()),
                    artist_image_url: r.artist_image_url,
                    added_at: r.added_at,
                    status: status.to_string(),
                }
            })
            .collect();

        Ok(tracks)
    }
}

fn grade_from_ratio(clean_ratio: f64) -> String {
    if clean_ratio < 0.5 {
        "F"
    } else if clean_ratio < 0.6 {
        "D"
    } else if clean_ratio < 0.7 {
        "C"
    } else if clean_ratio < 0.8 {
        "B"
    } else if clean_ratio < 0.95 {
        "A"
    } else {
        "A+"
    }
    .to_string()
}

fn grade_order(grade: &str) -> u8 {
    match grade {
        "F" => 0,
        "D" => 1,
        "C" => 2,
        "B" => 3,
        "A" => 4,
        "A+" => 5,
        _ => 5,
    }
}
