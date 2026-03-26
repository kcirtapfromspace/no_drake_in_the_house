use chrono::{DateTime, Utc};
use ndith_core::models::{
    PlaylistSummary, PlaylistTrackWithStatus, UpsertPlaylist, UpsertPlaylistTrack,
};
use sqlx::PgPool;
use std::collections::HashMap;
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

        // Batch-resolve all artist names in ONE query
        let artist_names_lower: Vec<String> = tracks
            .iter()
            .map(|t| t.artist_name.to_lowercase())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let artist_map: HashMap<String, Uuid> = if artist_names_lower.is_empty() {
            HashMap::new()
        } else {
            sqlx::query_as::<_, (Uuid, String)>(
                "SELECT id, LOWER(canonical_name) AS lname FROM artists WHERE LOWER(canonical_name) = ANY($1)",
            )
            .bind(&artist_names_lower)
            .fetch_all(&mut *tx)
            .await
            .map_err(ndith_core::error::AppError::DatabaseQueryFailed)?
            .into_iter()
            .map(|(id, lname)| (lname, id))
            .collect()
        };

        // Bulk INSERT all tracks in chunks of 500
        const CHUNK_SIZE: usize = 500;
        for chunk in tracks.chunks(CHUNK_SIZE) {
            let mut qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
                "INSERT INTO playlist_tracks (playlist_id, provider_track_id, track_name, album_name, artist_id, artist_name, position, added_at) ",
            );

            qb.push_values(chunk, |mut b, track| {
                let artist_id = artist_map.get(&track.artist_name.to_lowercase()).copied();
                b.push_bind(playlist_id)
                    .push_bind(&track.provider_track_id)
                    .push_bind(&track.track_name)
                    .push_bind(&track.album_name)
                    .push_bind(artist_id)
                    .push_bind(&track.artist_name)
                    .push_bind(track.position)
                    .push_bind(track.added_at);
            });

            qb.push(
                " ON CONFLICT (playlist_id, provider_track_id, position) DO UPDATE SET \
                  track_name = EXCLUDED.track_name, \
                  album_name = EXCLUDED.album_name, \
                  artist_id = EXCLUDED.artist_id, \
                  artist_name = EXCLUDED.artist_name",
            );

            qb.build()
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
    ///
    /// Uses a two-pass approach for performance:
    /// 1. Cheap aggregation query for track/artist counts (no correlated subqueries)
    /// 2. Single batch query to compute flagged tracks across ALL playlists at once
    pub async fn list_playlists_with_grades(
        &self,
        user_id: Uuid,
        provider: Option<&str>,
    ) -> Result<Vec<PlaylistSummary>> {
        // ── Pass 1: lightweight playlist metadata + counts ────────────
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
        }

        let rows = if let Some(prov) = provider {
            sqlx::query_as::<_, RawRow>(
                r#"
                SELECT
                    p.id, p.provider, p.provider_playlist_id, p.name,
                    p.description, p.image_url, p.owner_name, p.is_public,
                    p.source_type, p.last_synced,
                    COUNT(pt.id)::bigint AS total_tracks,
                    COUNT(DISTINCT pt.artist_name) FILTER (
                        WHERE pt.artist_name IS NOT NULL AND TRIM(pt.artist_name) <> ''
                    )::bigint AS unique_artists
                FROM playlists p
                LEFT JOIN playlist_tracks pt ON pt.playlist_id = p.id
                WHERE p.user_id = $1 AND p.provider = $2
                GROUP BY p.id
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
                    p.id, p.provider, p.provider_playlist_id, p.name,
                    p.description, p.image_url, p.owner_name, p.is_public,
                    p.source_type, p.last_synced,
                    COUNT(pt.id)::bigint AS total_tracks,
                    COUNT(DISTINCT pt.artist_name) FILTER (
                        WHERE pt.artist_name IS NOT NULL AND TRIM(pt.artist_name) <> ''
                    )::bigint AS unique_artists
                FROM playlists p
                LEFT JOIN playlist_tracks pt ON pt.playlist_id = p.id
                WHERE p.user_id = $1
                GROUP BY p.id
                "#,
            )
            .bind(user_id)
            .fetch_all(self.db)
            .await
            .map_err(ndith_core::error::AppError::DatabaseQueryFailed)?
        };

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let playlist_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();

        // ── Pass 2: batch-compute flagged track counts AND flagged artist names
        //    in a single query (merged from two separate identical-CTE passes) ─
        #[derive(sqlx::FromRow)]
        struct FlaggedRow {
            playlist_id: Uuid,
            flagged_count: i64,
            flagged_artist_names: Vec<String>,
        }

        let flagged_rows: Vec<FlaggedRow> = sqlx::query_as(
            r#"
            WITH offending_ids AS (
                SELECT DISTINCT a.id
                FROM artists a
                JOIN artist_offenses ao ON ao.artist_id = a.id AND ao.status = 'verified'
            ),
            offending_names AS (
                SELECT DISTINCT LOWER(a.canonical_name) AS lname
                FROM artists a
                JOIN artist_offenses ao ON ao.artist_id = a.id AND ao.status = 'verified'
            ),
            blocked_ids AS (
                SELECT artist_id FROM user_artist_blocks WHERE user_id = $1
            )
            SELECT
                pt.playlist_id,
                COUNT(*)::bigint AS flagged_count,
                COALESCE(
                    ARRAY_AGG(DISTINCT pt.artist_name)
                    FILTER (WHERE pt.artist_name IS NOT NULL AND TRIM(pt.artist_name) <> ''),
                    ARRAY[]::text[]
                ) AS flagged_artist_names
            FROM playlist_tracks pt
            WHERE pt.playlist_id = ANY($2)
              AND (
                  pt.artist_id IN (SELECT id FROM offending_ids)
                  OR LOWER(pt.artist_name) IN (SELECT lname FROM offending_names)
                  OR pt.artist_id IN (SELECT artist_id FROM blocked_ids)
              )
            GROUP BY pt.playlist_id
            "#,
        )
        .bind(user_id)
        .bind(&playlist_ids)
        .fetch_all(self.db)
        .await
        .map_err(ndith_core::error::AppError::DatabaseQueryFailed)?;

        let mut flagged_map: HashMap<Uuid, i64> = HashMap::new();
        let mut flagged_artists_map: HashMap<Uuid, Vec<String>> = HashMap::new();
        for row in flagged_rows {
            flagged_map.insert(row.playlist_id, row.flagged_count);
            if !row.flagged_artist_names.is_empty() {
                flagged_artists_map.insert(row.playlist_id, row.flagged_artist_names);
            }
        }

        // ── Pass 4: batch-fetch cover images ──────────────────────────
        #[derive(sqlx::FromRow)]
        struct CoverRow {
            playlist_id: Uuid,
            image_url: String,
        }

        let cover_rows: Vec<CoverRow> = sqlx::query_as(
            r#"
            SELECT DISTINCT ON (sub.playlist_id, sub.image_url)
                sub.playlist_id, sub.image_url
            FROM (
                SELECT pt.playlist_id, a.metadata->>'image_url' AS image_url
                FROM playlist_tracks pt
                JOIN artists a ON a.id = pt.artist_id
                WHERE pt.playlist_id = ANY($1)
                  AND a.metadata->>'image_url' IS NOT NULL
                  AND a.metadata->>'image_url' <> ''
            ) sub
            "#,
        )
        .bind(&playlist_ids)
        .fetch_all(self.db)
        .await
        .map_err(ndith_core::error::AppError::DatabaseQueryFailed)?;

        let mut cover_map: HashMap<Uuid, Vec<String>> = HashMap::new();
        for row in cover_rows {
            let images = cover_map.entry(row.playlist_id).or_default();
            if images.len() < 4 {
                images.push(row.image_url);
            }
        }

        // ── Assemble summaries ────────────────────────────────────────
        let mut summaries = Vec::with_capacity(rows.len());
        for r in rows {
            let flagged_tracks = *flagged_map.get(&r.id).unwrap_or(&0);
            let clean_ratio = if r.total_tracks > 0 {
                (r.total_tracks - flagged_tracks) as f64 / r.total_tracks as f64
            } else {
                1.0
            };

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
                flagged_tracks,
                clean_ratio,
                grade: grade_from_ratio(clean_ratio),
                unique_artists: r.unique_artists,
                flagged_artists: flagged_artists_map.remove(&r.id).unwrap_or_default(),
                last_synced: r.last_synced,
                cover_images: cover_map.remove(&r.id).unwrap_or_default(),
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
