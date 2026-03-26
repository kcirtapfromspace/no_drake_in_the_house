use chrono::Utc;
use ndith_core::error::{AppError, Result};
use ndith_core::models::offense::{
    AddEvidenceRequest, ArtistOffense, CreateOffenseRequest, FlaggedArtist, ImportLibraryRequest,
    LibraryScanResponse, OffenseEvidence, OffenseSeverity, OffenseSummary, OffenseWithEvidence,
    UserLibraryTrack,
};
use sqlx::PgPool;
use uuid::Uuid;

/// Service for managing artist offenses and evidence
pub struct OffenseService<'a> {
    db: &'a PgPool,
}

impl<'a> OffenseService<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self { db }
    }

    /// Get all verified offenses for an artist
    pub async fn get_artist_offenses(&self, artist_id: Uuid) -> Result<Vec<ArtistOffense>> {
        let offenses = sqlx::query_as::<_, ArtistOffense>(
            r#"
            SELECT id, artist_id, category, severity, title, description,
                   incident_date, incident_date_approximate, arrested, charged,
                   convicted, settled, status, verified_at, verified_by,
                   submitted_by, created_at, updated_at
            FROM artist_offenses
            WHERE artist_id = $1 AND status = 'verified'
            ORDER BY severity DESC, incident_date DESC NULLS LAST
            "#,
        )
        .bind(artist_id)
        .fetch_all(self.db)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

        Ok(offenses)
    }

    /// Get a single offense with all evidence
    pub async fn get_offense_with_evidence(&self, offense_id: Uuid) -> Result<OffenseWithEvidence> {
        let offense = sqlx::query_as::<_, ArtistOffense>(
            r#"
            SELECT id, artist_id, category, severity, title, description,
                   incident_date, incident_date_approximate, arrested, charged,
                   convicted, settled, status, verified_at, verified_by,
                   submitted_by, created_at, updated_at
            FROM artist_offenses
            WHERE id = $1
            "#,
        )
        .bind(offense_id)
        .fetch_optional(self.db)
        .await
        .map_err(AppError::DatabaseQueryFailed)?
        .ok_or_else(|| AppError::NotFound {
            resource: "Offense".to_string(),
        })?;

        let evidence = sqlx::query_as::<_, OffenseEvidence>(
            r#"
            SELECT id, offense_id, url, source_name, source_type, title, excerpt,
                   published_date, archived_url, is_primary_source, credibility_score,
                   submitted_by, created_at
            FROM offense_evidence
            WHERE offense_id = $1
            ORDER BY is_primary_source DESC, credibility_score DESC NULLS LAST
            "#,
        )
        .bind(offense_id)
        .fetch_all(self.db)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

        // Get artist name
        let artist_name: String =
            sqlx::query_scalar("SELECT canonical_name FROM artists WHERE id = $1")
                .bind(offense.artist_id)
                .fetch_optional(self.db)
                .await
                .map_err(AppError::DatabaseQueryFailed)?
                .unwrap_or_else(|| "Unknown Artist".to_string());

        Ok(OffenseWithEvidence {
            offense,
            evidence,
            artist_name,
        })
    }

    /// Create a new offense (pending verification)
    pub async fn create_offense(
        &self,
        request: CreateOffenseRequest,
        submitted_by: Option<Uuid>,
    ) -> Result<ArtistOffense> {
        let offense = sqlx::query_as::<_, ArtistOffense>(
            r#"
            INSERT INTO artist_offenses (
                artist_id, category, severity, title, description,
                incident_date, incident_date_approximate, arrested, charged,
                convicted, settled, submitted_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, artist_id, category, severity, title, description,
                      incident_date, incident_date_approximate, arrested, charged,
                      convicted, settled, status, verified_at, verified_by,
                      submitted_by, created_at, updated_at
            "#,
        )
        .bind(request.artist_id)
        .bind(request.category)
        .bind(request.severity)
        .bind(&request.title)
        .bind(&request.description)
        .bind(request.incident_date)
        .bind(request.incident_date_approximate.unwrap_or(false))
        .bind(request.arrested.unwrap_or(false))
        .bind(request.charged.unwrap_or(false))
        .bind(request.convicted.unwrap_or(false))
        .bind(request.settled.unwrap_or(false))
        .bind(submitted_by)
        .fetch_one(self.db)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

        Ok(offense)
    }

    /// Add evidence to an offense
    pub async fn add_evidence(
        &self,
        request: AddEvidenceRequest,
        submitted_by: Option<Uuid>,
    ) -> Result<OffenseEvidence> {
        let evidence = sqlx::query_as::<_, OffenseEvidence>(
            r#"
            INSERT INTO offense_evidence (
                offense_id, url, source_name, source_type, title, excerpt,
                published_date, is_primary_source, credibility_score, submitted_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, offense_id, url, source_name, source_type, title, excerpt,
                      published_date, archived_url, is_primary_source, credibility_score,
                      submitted_by, created_at
            "#,
        )
        .bind(request.offense_id)
        .bind(&request.url)
        .bind(&request.source_name)
        .bind(&request.source_type)
        .bind(&request.title)
        .bind(&request.excerpt)
        .bind(request.published_date)
        .bind(request.is_primary_source.unwrap_or(false))
        .bind(request.credibility_score)
        .bind(submitted_by)
        .fetch_one(self.db)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

        Ok(evidence)
    }

    /// Verify an offense (moderator action)
    pub async fn verify_offense(&self, offense_id: Uuid, verified_by: Uuid) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE artist_offenses
            SET status = 'verified', verified_at = NOW(), verified_by = $2
            WHERE id = $1
            "#,
        )
        .bind(offense_id)
        .bind(verified_by)
        .execute(self.db)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: "Offense".to_string(),
            });
        }

        Ok(())
    }

    /// Get all flagged artists (for browsing the database)
    pub async fn get_flagged_artists(
        &self,
        severity_filter: Option<OffenseSeverity>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<FlaggedArtist>> {
        // Get artists with verified offenses, aggregated by highest severity
        let rows = sqlx::query_as::<_, (Uuid, String, OffenseSeverity, i64)>(
            r#"
            SELECT a.id, a.canonical_name,
                   MAX(ao.severity) as highest_severity,
                   COUNT(DISTINCT ao.id) as offense_count
            FROM artists a
            JOIN artist_offenses ao ON a.id = ao.artist_id
            WHERE ao.status = 'verified'
              AND ($1::offense_severity IS NULL OR ao.severity = $1)
            GROUP BY a.id, a.canonical_name
            ORDER BY highest_severity DESC, offense_count DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(severity_filter)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.db)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

        // Batch-fetch all offense summaries in a single query
        let artist_ids: Vec<Uuid> = rows.iter().map(|(id, _, _, _)| *id).collect();
        let mut summaries_map = self.get_offense_summaries_batch(&artist_ids).await?;

        let flagged_artists = rows
            .into_iter()
            .map(|(artist_id, artist_name, severity, _)| FlaggedArtist {
                id: artist_id,
                name: artist_name,
                track_count: 0, // Will be filled by library scan
                severity,
                offenses: summaries_map.remove(&artist_id).unwrap_or_default(),
            })
            .collect();

        Ok(flagged_artists)
    }

    /// Get offense summaries for a single artist (convenience wrapper over batch)
    #[allow(dead_code)]
    async fn get_offense_summaries(&self, artist_id: Uuid) -> Result<Vec<OffenseSummary>> {
        let map = self.get_offense_summaries_batch(&[artist_id]).await?;
        Ok(map.into_values().next().unwrap_or_default())
    }

    /// Batch-fetch offense summaries for multiple artists in a single query.
    /// Returns a map from artist_id to their offense summaries.
    async fn get_offense_summaries_batch(
        &self,
        artist_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Vec<OffenseSummary>>> {
        if artist_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let rows = sqlx::query_as::<
            _,
            (
                Uuid,
                ndith_core::models::offense::OffenseCategory,
                String,
                Option<chrono::NaiveDate>,
                i64,
            ),
        >(
            r#"
            SELECT ao.artist_id, ao.category, ao.title, ao.incident_date,
                   COUNT(oe.id) as evidence_count
            FROM artist_offenses ao
            LEFT JOIN offense_evidence oe ON ao.id = oe.offense_id
            WHERE ao.artist_id = ANY($1) AND ao.status = 'verified'
            GROUP BY ao.id, ao.artist_id, ao.category, ao.title, ao.incident_date
            ORDER BY ao.severity DESC
            "#,
        )
        .bind(artist_ids)
        .fetch_all(self.db)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

        let mut map: std::collections::HashMap<Uuid, Vec<OffenseSummary>> =
            std::collections::HashMap::new();
        for (artist_id, category, title, date, evidence_count) in rows {
            map.entry(artist_id).or_default().push(OffenseSummary {
                category,
                title,
                date: date
                    .map(|d| d.format("%Y").to_string())
                    .unwrap_or_else(|| "Unknown".to_string()),
                evidence_count: evidence_count as i32,
            });
        }

        Ok(map)
    }

    /// Import user library tracks from streaming service export.
    ///
    /// Uses batch artist lookups and chunked bulk upserts (500 rows per chunk)
    /// wrapped in a transaction for atomicity.
    pub async fn import_library(
        &self,
        user_id: Uuid,
        request: ImportLibraryRequest,
    ) -> Result<i32> {
        if request.tracks.is_empty() {
            return Ok(0);
        }

        // Batch-resolve all unique artist names in ONE query
        let artist_names_lower: Vec<String> = request
            .tracks
            .iter()
            .map(|t| t.artist_name.to_lowercase())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let artist_map: std::collections::HashMap<String, Uuid> = sqlx::query_as::<_, (Uuid, String)>(
            "SELECT id, LOWER(canonical_name) AS lname FROM artists WHERE LOWER(canonical_name) = ANY($1)",
        )
        .bind(&artist_names_lower)
        .fetch_all(self.db)
        .await
        .map_err(AppError::DatabaseQueryFailed)?
        .into_iter()
        .map(|(id, lname)| (lname, id))
        .collect();

        // Chunked bulk upsert inside a transaction
        let mut tx = self
            .db
            .begin()
            .await
            .map_err(AppError::DatabaseQueryFailed)?;

        const CHUNK_SIZE: usize = 500;
        let total = request.tracks.len() as i32;

        for chunk in request.tracks.chunks(CHUNK_SIZE) {
            let mut qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
                "INSERT INTO user_library_tracks (user_id, provider, provider_track_id, track_name, album_name, artist_id, artist_name, source_type, playlist_name, added_at) ",
            );

            qb.push_values(chunk, |mut b, track| {
                let artist_id = artist_map.get(&track.artist_name.to_lowercase()).copied();
                b.push_bind(user_id)
                    .push_bind(&request.provider)
                    .push_bind(&track.provider_track_id)
                    .push_bind(&track.track_name)
                    .push_bind(&track.album_name)
                    .push_bind(artist_id)
                    .push_bind(&track.artist_name)
                    .push_bind(&track.source_type)
                    .push_bind(&track.playlist_name)
                    .push_bind(track.added_at);
            });

            qb.push(
                " ON CONFLICT (user_id, provider, provider_track_id) DO UPDATE SET \
                  track_name = EXCLUDED.track_name, \
                  album_name = EXCLUDED.album_name, \
                  artist_id = EXCLUDED.artist_id, \
                  artist_name = EXCLUDED.artist_name, \
                  last_synced = NOW()",
            );

            qb.build()
                .execute(&mut *tx)
                .await
                .map_err(AppError::DatabaseQueryFailed)?;
        }

        tx.commit().await.map_err(AppError::DatabaseQueryFailed)?;

        Ok(total)
    }

    /// Atomically delete-and-reimport a provider's library tracks in a single transaction.
    ///
    /// This prevents a race condition where a crash between DELETE and reimport
    /// would leave the user with an empty library.
    pub async fn delete_and_import_library(
        &self,
        user_id: Uuid,
        request: ImportLibraryRequest,
    ) -> Result<i32> {
        // Batch-resolve all unique artist names in ONE query (outside tx for read perf)
        let artist_map: std::collections::HashMap<String, Uuid> = if request.tracks.is_empty() {
            std::collections::HashMap::new()
        } else {
            let artist_names_lower: Vec<String> = request
                .tracks
                .iter()
                .map(|t| t.artist_name.to_lowercase())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            sqlx::query_as::<_, (Uuid, String)>(
                "SELECT id, LOWER(canonical_name) AS lname FROM artists WHERE LOWER(canonical_name) = ANY($1)",
            )
            .bind(&artist_names_lower)
            .fetch_all(self.db)
            .await
            .map_err(AppError::DatabaseQueryFailed)?
            .into_iter()
            .map(|(id, lname)| (lname, id))
            .collect()
        };

        let mut tx = self
            .db
            .begin()
            .await
            .map_err(AppError::DatabaseQueryFailed)?;

        // DELETE inside the transaction so rollback restores the old rows on failure
        sqlx::query("DELETE FROM user_library_tracks WHERE user_id = $1 AND provider = $2")
            .bind(user_id)
            .bind(&request.provider)
            .execute(&mut *tx)
            .await
            .map_err(AppError::DatabaseQueryFailed)?;

        if request.tracks.is_empty() {
            tx.commit().await.map_err(AppError::DatabaseQueryFailed)?;
            return Ok(0);
        }

        const CHUNK_SIZE: usize = 500;
        let total = request.tracks.len() as i32;

        for chunk in request.tracks.chunks(CHUNK_SIZE) {
            let mut qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
                "INSERT INTO user_library_tracks (user_id, provider, provider_track_id, track_name, album_name, artist_id, artist_name, source_type, playlist_name, added_at) ",
            );

            qb.push_values(chunk, |mut b, track| {
                let artist_id = artist_map.get(&track.artist_name.to_lowercase()).copied();
                b.push_bind(user_id)
                    .push_bind(&request.provider)
                    .push_bind(&track.provider_track_id)
                    .push_bind(&track.track_name)
                    .push_bind(&track.album_name)
                    .push_bind(artist_id)
                    .push_bind(&track.artist_name)
                    .push_bind(&track.source_type)
                    .push_bind(&track.playlist_name)
                    .push_bind(track.added_at);
            });

            qb.push(
                " ON CONFLICT (user_id, provider, provider_track_id) DO UPDATE SET \
                  track_name = EXCLUDED.track_name, \
                  album_name = EXCLUDED.album_name, \
                  artist_id = EXCLUDED.artist_id, \
                  artist_name = EXCLUDED.artist_name, \
                  last_synced = NOW()",
            );

            qb.build()
                .execute(&mut *tx)
                .await
                .map_err(AppError::DatabaseQueryFailed)?;
        }

        tx.commit().await.map_err(AppError::DatabaseQueryFailed)?;

        Ok(total)
    }

    /// Scan user's library against offense database
    pub async fn scan_library(&self, user_id: Uuid) -> Result<LibraryScanResponse> {
        // Get total track and artist counts
        let stats: (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total_tracks,
                COUNT(DISTINCT artist_name) as total_artists
            FROM user_library_tracks
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(self.db)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

        // Find flagged artists in user's library
        let flagged_rows = sqlx::query_as::<_, (Uuid, String, i64, OffenseSeverity)>(
            r#"
            SELECT a.id, a.canonical_name,
                   COUNT(DISTINCT ult.id) as track_count,
                   MAX(ao.severity) as highest_severity
            FROM user_library_tracks ult
            JOIN artists a ON (
                ult.artist_id = a.id
                OR LOWER(ult.artist_name) = LOWER(a.canonical_name)
            )
            JOIN artist_offenses ao ON a.id = ao.artist_id
            WHERE ult.user_id = $1 AND ao.status = 'verified'
            GROUP BY a.id, a.canonical_name
            ORDER BY highest_severity DESC, track_count DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(self.db)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

        // Batch-fetch all offense summaries in a single query
        let artist_ids: Vec<Uuid> = flagged_rows.iter().map(|(id, _, _, _)| *id).collect();
        let mut summaries_map = self.get_offense_summaries_batch(&artist_ids).await?;

        let mut flagged_artists = Vec::new();
        let mut flagged_tracks = 0i64;

        for (artist_id, artist_name, track_count, severity) in flagged_rows {
            flagged_tracks += track_count;
            flagged_artists.push(FlaggedArtist {
                id: artist_id,
                name: artist_name,
                track_count: track_count as i32,
                severity,
                offenses: summaries_map.remove(&artist_id).unwrap_or_default(),
            });
        }

        let now = Utc::now();

        let flagged_artists_json = serde_json::to_value(&flagged_artists)
            .unwrap_or_else(|_| serde_json::Value::Array(vec![]));

        // Persist scan results for later retrieval
        sqlx::query(
            r#"
            INSERT INTO library_scan_results (
                user_id, provider, total_tracks, total_artists,
                flagged_artists, flagged_tracks,
                egregious_count, severe_count, moderate_count, minor_count,
                scan_started_at, scan_completed_at, flagged_artists_json
            )
            VALUES ($1, 'all', $2, $3, $4, $5, $6, $7, $8, $9, $10, $10, $11)
            ON CONFLICT (user_id, provider) DO UPDATE SET
                total_tracks = EXCLUDED.total_tracks,
                total_artists = EXCLUDED.total_artists,
                flagged_artists = EXCLUDED.flagged_artists,
                flagged_tracks = EXCLUDED.flagged_tracks,
                egregious_count = EXCLUDED.egregious_count,
                severe_count = EXCLUDED.severe_count,
                moderate_count = EXCLUDED.moderate_count,
                minor_count = EXCLUDED.minor_count,
                scan_completed_at = EXCLUDED.scan_completed_at,
                flagged_artists_json = EXCLUDED.flagged_artists_json
            "#,
        )
        .bind(user_id)
        .bind(stats.0 as i32)
        .bind(stats.1 as i32)
        .bind(flagged_artists.len() as i32)
        .bind(flagged_tracks as i32)
        .bind(
            flagged_artists
                .iter()
                .filter(|a| a.severity == OffenseSeverity::Egregious)
                .count() as i32,
        )
        .bind(
            flagged_artists
                .iter()
                .filter(|a| a.severity == OffenseSeverity::Severe)
                .count() as i32,
        )
        .bind(
            flagged_artists
                .iter()
                .filter(|a| a.severity == OffenseSeverity::Moderate)
                .count() as i32,
        )
        .bind(
            flagged_artists
                .iter()
                .filter(|a| a.severity == OffenseSeverity::Minor)
                .count() as i32,
        )
        .bind(now)
        .bind(&flagged_artists_json)
        .execute(self.db)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

        Ok(LibraryScanResponse {
            total_tracks: stats.0 as i32,
            total_artists: stats.1 as i32,
            flagged_artists,
            flagged_tracks: flagged_tracks as i32,
            scanned_at: Some(now),
        })
    }

    /// Get cached scan results for a user (if any)
    pub async fn get_cached_scan(&self, user_id: Uuid) -> Result<Option<LibraryScanResponse>> {
        let row = sqlx::query_as::<_, (i32, i32, i32, serde_json::Value, chrono::DateTime<Utc>)>(
            r#"
            SELECT total_tracks, total_artists, flagged_tracks,
                   flagged_artists_json, scan_completed_at
            FROM library_scan_results
            WHERE user_id = $1 AND provider = 'all' AND scan_completed_at IS NOT NULL
            "#,
        )
        .bind(user_id)
        .fetch_optional(self.db)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

        match row {
            Some((total_tracks, total_artists, flagged_tracks, json, completed_at)) => {
                let flagged_artists: Vec<FlaggedArtist> =
                    serde_json::from_value(json).unwrap_or_default();
                Ok(Some(LibraryScanResponse {
                    total_tracks,
                    total_artists,
                    flagged_artists,
                    flagged_tracks,
                    scanned_at: Some(completed_at),
                }))
            }
            None => Ok(None),
        }
    }

    /// Get user's library tracks
    pub async fn get_user_library(
        &self,
        user_id: Uuid,
        provider: Option<String>,
    ) -> Result<Vec<UserLibraryTrack>> {
        let tracks = sqlx::query_as::<_, UserLibraryTrack>(
            r#"
            SELECT id, user_id, provider, provider_track_id, track_name, album_name,
                   artist_id, artist_name, source_type, playlist_name, added_at, last_synced
            FROM user_library_tracks
            WHERE user_id = $1 AND ($2::text IS NULL OR provider = $2)
            ORDER BY last_synced DESC
            "#,
        )
        .bind(user_id)
        .bind(provider)
        .fetch_all(self.db)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

        Ok(tracks)
    }

    /// Get artists by offense category
    pub async fn get_artists_by_category(
        &self,
        category: &str,
    ) -> Result<Vec<ndith_core::models::offense::CategoryArtist>> {
        let artists = sqlx::query_as::<_, ndith_core::models::offense::CategoryArtist>(
            r#"
            SELECT DISTINCT ON (a.id)
                a.id,
                a.canonical_name as name,
                ao.category::text as category,
                ao.severity::text as severity
            FROM artists a
            JOIN artist_offenses ao ON a.id = ao.artist_id
            WHERE ao.category::text = $1
            AND ao.status IN ('pending', 'verified')
            ORDER BY a.id, ao.severity DESC
            "#,
        )
        .bind(category)
        .fetch_all(self.db)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

        Ok(artists)
    }

    /// Get full artist details with all offenses and evidence
    pub async fn get_artist_details(
        &self,
        artist_id: Uuid,
    ) -> Result<ndith_core::models::offense::ArtistDetails> {
        // Get artist info
        let artist = sqlx::query_as::<_, (String, Option<serde_json::Value>)>(
            r#"
            SELECT canonical_name, metadata
            FROM artists
            WHERE id = $1
            "#,
        )
        .bind(artist_id)
        .fetch_optional(self.db)
        .await
        .map_err(AppError::DatabaseQueryFailed)?
        .ok_or_else(|| AppError::NotFound {
            resource: "Artist".to_string(),
        })?;

        let genres: Vec<String> = artist
            .1
            .and_then(|m| m.get("genres").cloned())
            .and_then(|g| serde_json::from_value(g).ok())
            .unwrap_or_default();

        // Get all offenses with evidence
        let offenses = self.get_artist_offenses(artist_id).await?;

        let mut offense_details = Vec::new();
        for offense in offenses {
            let evidence = sqlx::query_as::<_, OffenseEvidence>(
                r#"
                SELECT id, offense_id, url, source_name, source_type, title, excerpt,
                       published_date, archived_url, is_primary_source, credibility_score,
                       submitted_by, created_at
                FROM offense_evidence
                WHERE offense_id = $1
                ORDER BY is_primary_source DESC, credibility_score DESC NULLS LAST
                "#,
            )
            .bind(offense.id)
            .fetch_all(self.db)
            .await
            .map_err(AppError::DatabaseQueryFailed)?;

            offense_details.push(ndith_core::models::offense::OffenseDetail {
                id: offense.id,
                category: offense.category.to_string(),
                severity: offense.severity.to_string(),
                title: offense.title,
                description: offense.description,
                incident_date: offense.incident_date,
                status: offense.status.to_string(),
                evidence: evidence
                    .into_iter()
                    .map(|e| ndith_core::models::offense::EvidenceDetail {
                        id: e.id,
                        source_url: e.url,
                        source_name: e.source_name,
                        source_type: e.source_type,
                        title: e.title,
                        excerpt: e.excerpt,
                        published_date: e.published_date,
                        credibility_score: e.credibility_score,
                    })
                    .collect(),
            });
        }

        Ok(ndith_core::models::offense::ArtistDetails {
            id: artist_id,
            canonical_name: artist.0,
            genres: if genres.is_empty() {
                None
            } else {
                Some(genres)
            },
            image_url: None,
            offenses: offense_details,
        })
    }
}
