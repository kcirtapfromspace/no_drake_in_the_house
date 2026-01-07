use crate::error::{AppError, Result};
use crate::models::offense::{
    AddEvidenceRequest, ArtistOffense, CreateOffenseRequest, FlaggedArtist,
    ImportLibraryRequest, LibraryScanResponse, OffenseEvidence, OffenseSeverity,
    OffenseSummary, OffenseWithEvidence, UserLibraryTrack,
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
        .map_err(|e| AppError::DatabaseQueryFailed(e))?;

        Ok(offenses)
    }

    /// Get a single offense with all evidence
    pub async fn get_offense_with_evidence(
        &self,
        offense_id: Uuid,
    ) -> Result<OffenseWithEvidence> {
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
        .map_err(|e| AppError::DatabaseQueryFailed(e))?
        .ok_or_else(|| AppError::NotFound { resource: "Offense".to_string() })?;

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
        .map_err(|e| AppError::DatabaseQueryFailed(e))?;

        // Get artist name
        let artist_name: String = sqlx::query_scalar(
            "SELECT canonical_name FROM artists WHERE id = $1",
        )
        .bind(offense.artist_id)
        .fetch_optional(self.db)
        .await
        .map_err(|e| AppError::DatabaseQueryFailed(e))?
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
        .map_err(|e| AppError::DatabaseQueryFailed(e))?;

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
        .map_err(|e| AppError::DatabaseQueryFailed(e))?;

        Ok(evidence)
    }

    /// Verify an offense (moderator action)
    pub async fn verify_offense(&self, offense_id: Uuid, verified_by: Uuid) -> Result<()> {
        sqlx::query(
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
        .map_err(|e| AppError::DatabaseQueryFailed(e))?;

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
        .map_err(|e| AppError::DatabaseQueryFailed(e))?;

        let mut flagged_artists = Vec::new();
        for (artist_id, artist_name, severity, _) in rows {
            let offenses = self.get_offense_summaries(artist_id).await?;
            flagged_artists.push(FlaggedArtist {
                id: artist_id,
                name: artist_name,
                track_count: 0, // Will be filled by library scan
                severity,
                offenses,
            });
        }

        Ok(flagged_artists)
    }

    /// Get offense summaries for an artist
    async fn get_offense_summaries(&self, artist_id: Uuid) -> Result<Vec<OffenseSummary>> {
        let rows = sqlx::query_as::<_, (crate::models::offense::OffenseCategory, String, Option<chrono::NaiveDate>, i64)>(
            r#"
            SELECT ao.category, ao.title, ao.incident_date,
                   COUNT(oe.id) as evidence_count
            FROM artist_offenses ao
            LEFT JOIN offense_evidence oe ON ao.id = oe.offense_id
            WHERE ao.artist_id = $1 AND ao.status = 'verified'
            GROUP BY ao.id, ao.category, ao.title, ao.incident_date
            ORDER BY ao.severity DESC
            "#,
        )
        .bind(artist_id)
        .fetch_all(self.db)
        .await
        .map_err(|e| AppError::DatabaseQueryFailed(e))?;

        Ok(rows
            .into_iter()
            .map(|(category, title, date, evidence_count)| OffenseSummary {
                category,
                title,
                date: date
                    .map(|d| d.format("%Y").to_string())
                    .unwrap_or_else(|| "Unknown".to_string()),
                evidence_count: evidence_count as i32,
            })
            .collect())
    }

    /// Import user library tracks from streaming service export
    pub async fn import_library(
        &self,
        user_id: Uuid,
        request: ImportLibraryRequest,
    ) -> Result<i32> {
        let mut imported = 0;

        for track in request.tracks {
            // Try to find artist in our database
            let artist_id: Option<Uuid> = sqlx::query_scalar(
                r#"
                SELECT id FROM artists
                WHERE LOWER(canonical_name) = LOWER($1)
                LIMIT 1
                "#,
            )
            .bind(&track.artist_name)
            .fetch_optional(self.db)
            .await
            .map_err(|e| AppError::DatabaseQueryFailed(e))?;

            // Insert or update the track
            sqlx::query(
                r#"
                INSERT INTO user_library_tracks (
                    user_id, provider, provider_track_id, track_name, album_name,
                    artist_id, artist_name, source_type, playlist_name, added_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                ON CONFLICT (user_id, provider, provider_track_id)
                DO UPDATE SET
                    track_name = EXCLUDED.track_name,
                    album_name = EXCLUDED.album_name,
                    artist_id = EXCLUDED.artist_id,
                    artist_name = EXCLUDED.artist_name,
                    last_synced = NOW()
                "#,
            )
            .bind(user_id)
            .bind(&request.provider)
            .bind(&track.provider_track_id)
            .bind(&track.track_name)
            .bind(&track.album_name)
            .bind(artist_id)
            .bind(&track.artist_name)
            .bind(&track.source_type)
            .bind(&track.playlist_name)
            .bind(track.added_at)
            .execute(self.db)
            .await
            .map_err(|e| AppError::DatabaseQueryFailed(e))?;

            imported += 1;
        }

        Ok(imported)
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
        .map_err(|e| AppError::DatabaseQueryFailed(e))?;

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
        .map_err(|e| AppError::DatabaseQueryFailed(e))?;

        let mut flagged_artists = Vec::new();
        let mut flagged_tracks = 0i64;

        for (artist_id, artist_name, track_count, severity) in flagged_rows {
            flagged_tracks += track_count;
            let offenses = self.get_offense_summaries(artist_id).await?;
            flagged_artists.push(FlaggedArtist {
                id: artist_id,
                name: artist_name,
                track_count: track_count as i32,
                severity,
                offenses,
            });
        }

        Ok(LibraryScanResponse {
            total_tracks: stats.0 as i32,
            total_artists: stats.1 as i32,
            flagged_artists,
            flagged_tracks: flagged_tracks as i32,
        })
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
        .map_err(|e| AppError::DatabaseQueryFailed(e))?;

        Ok(tracks)
    }
}
