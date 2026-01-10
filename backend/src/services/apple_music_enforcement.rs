//! Apple Music Enforcement Service
//!
//! Implements enforcement via the Apple Music Ratings API.
//! Since Apple Music API doesn't support library modifications,
//! we use the ratings system to "dislike" songs/albums from blocked artists.
//! This influences Apple Music's recommendation algorithm.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

use crate::models::{
    Connection, AppleMusicRatingEnforcementOptions, EnforcementProgress,
    BlockedContentScan, BlockedSongInfo, BlockedAlbumInfo, EnforcementPreview,
    RatingEnforcementResult, RollbackResult, RatingError, EnforcementRunStatus,
    AppleMusicResourceType, EnforcementActionType,
};
use crate::services::AppleMusicService;

/// Service for executing Apple Music enforcement operations via ratings
pub struct AppleMusicEnforcementService {
    apple_music: Arc<AppleMusicService>,
    db_pool: PgPool,
}

impl AppleMusicEnforcementService {
    pub fn new(apple_music: Arc<AppleMusicService>, db_pool: PgPool) -> Self {
        Self { apple_music, db_pool }
    }

    /// Run full enforcement for a user's Apple Music library
    pub async fn enforce_dnp_list<F>(
        &self,
        user_id: Uuid,
        blocked_artist_names: Vec<String>,
        options: AppleMusicRatingEnforcementOptions,
        mut progress_callback: F,
    ) -> Result<RatingEnforcementResult>
    where
        F: FnMut(EnforcementProgress) + Send,
    {
        let start_time = Instant::now();
        let mut progress = EnforcementProgress::new();

        // Get user's Apple Music connection
        let connection = self.apple_music
            .get_user_connection(user_id)
            .await?
            .ok_or_else(|| anyhow!("No Apple Music connection found for user"))?;

        // Create enforcement run record
        let run_id = self.create_enforcement_run(
            user_id,
            connection.id,
            &options,
        ).await?;

        progress.phase = "scanning".to_string();
        progress_callback(progress.clone());

        // Scan library for blocked content
        let blocked_content = self.scan_for_blocked_content(
            &connection,
            &blocked_artist_names,
        ).await?;

        progress.total_items = blocked_content.blocked_songs.len() + blocked_content.blocked_albums.len();
        progress.phase = "enforcing".to_string();
        progress_callback(progress.clone());

        // If dry run, just return preview
        if options.dry_run {
            let result = RatingEnforcementResult {
                run_id,
                status: EnforcementRunStatus::Completed,
                songs_disliked: 0,
                albums_disliked: 0,
                errors: Vec::new(),
                duration_seconds: start_time.elapsed().as_secs(),
                started_at: Utc::now(),
                completed_at: Some(Utc::now()),
            };

            self.update_enforcement_run(
                &run_id,
                EnforcementRunStatus::Completed,
                blocked_content.total_songs_scanned as u32,
                blocked_content.total_albums_scanned as u32,
                0, 0, 0, None,
            ).await?;

            return Ok(result);
        }

        let mut songs_disliked = 0;
        let mut albums_disliked = 0;
        let mut errors = Vec::new();

        // Dislike songs
        if options.dislike_songs {
            for (idx, song) in blocked_content.blocked_songs.iter().enumerate() {
                progress.current_item = Some(format!("{} - {}", song.artist_name, song.name));
                progress.processed_items = idx;
                progress_callback(progress.clone());

                match self.apple_music
                    .rate_library_song(&connection, &song.library_song_id, -1)
                    .await
                {
                    Ok(()) => {
                        songs_disliked += 1;
                        progress.songs_disliked = songs_disliked;

                        // Record action for rollback
                        self.record_enforcement_action(
                            &run_id,
                            user_id,
                            AppleMusicResourceType::LibrarySong,
                            &song.library_song_id,
                            Some(&song.name),
                            Some(&song.artist_name),
                            EnforcementActionType::Dislike,
                            None,
                        ).await.ok();
                    }
                    Err(e) => {
                        errors.push(RatingError {
                            resource_id: song.library_song_id.clone(),
                            resource_type: "library_song".to_string(),
                            error_message: e.to_string(),
                        });
                        progress.errors += 1;
                    }
                }
                progress_callback(progress.clone());
            }
        }

        // Dislike albums
        if options.dislike_albums {
            let song_count = blocked_content.blocked_songs.len();
            for (idx, album) in blocked_content.blocked_albums.iter().enumerate() {
                progress.current_item = Some(format!("{} - {}", album.artist_name, album.name));
                progress.processed_items = song_count + idx;
                progress_callback(progress.clone());

                match self.apple_music
                    .rate_library_album(&connection, &album.library_album_id, -1)
                    .await
                {
                    Ok(()) => {
                        albums_disliked += 1;
                        progress.albums_disliked = albums_disliked;

                        // Record action for rollback
                        self.record_enforcement_action(
                            &run_id,
                            user_id,
                            AppleMusicResourceType::LibraryAlbum,
                            &album.library_album_id,
                            Some(&album.name),
                            Some(&album.artist_name),
                            EnforcementActionType::Dislike,
                            None,
                        ).await.ok();
                    }
                    Err(e) => {
                        errors.push(RatingError {
                            resource_id: album.library_album_id.clone(),
                            resource_type: "library_album".to_string(),
                            error_message: e.to_string(),
                        });
                        progress.errors += 1;
                    }
                }
                progress_callback(progress.clone());
            }
        }

        progress.phase = "completed".to_string();
        progress.processed_items = progress.total_items;
        progress_callback(progress.clone());

        let duration_seconds = start_time.elapsed().as_secs();
        let status = if errors.is_empty() {
            EnforcementRunStatus::Completed
        } else if songs_disliked + albums_disliked > 0 {
            EnforcementRunStatus::Completed // Partial success
        } else {
            EnforcementRunStatus::Failed
        };

        // Update enforcement run
        self.update_enforcement_run(
            &run_id,
            status.clone(),
            blocked_content.total_songs_scanned as u32,
            blocked_content.total_albums_scanned as u32,
            songs_disliked as u32,
            albums_disliked as u32,
            errors.len() as u32,
            if errors.is_empty() { None } else { Some(serde_json::to_value(&errors)?) },
        ).await?;

        Ok(RatingEnforcementResult {
            run_id,
            status,
            songs_disliked,
            albums_disliked,
            errors,
            duration_seconds,
            started_at: Utc::now() - chrono::Duration::seconds(duration_seconds as i64),
            completed_at: Some(Utc::now()),
        })
    }

    /// Scan user's library for blocked content
    pub async fn scan_for_blocked_content(
        &self,
        connection: &Connection,
        blocked_artist_names: &[String],
    ) -> Result<BlockedContentScan> {
        let mut scan = BlockedContentScan::new();

        // Get library tracks
        let library_tracks = self.apple_music.get_library_tracks(connection).await?;
        scan.total_songs_scanned = library_tracks.len();

        // Get library albums
        let library_albums = self.apple_music.get_library_albums(connection).await?;
        scan.total_albums_scanned = library_albums.len();

        // Normalize blocked artist names for comparison
        let blocked_names_lower: Vec<String> = blocked_artist_names
            .iter()
            .map(|n| n.to_lowercase())
            .collect();

        // Find blocked songs
        for track in library_tracks {
            let artist_lower = track.attributes.artist_name.to_lowercase();
            if blocked_names_lower.iter().any(|blocked| artist_lower.contains(blocked)) {
                scan.blocked_songs.push(BlockedSongInfo {
                    library_song_id: track.id.clone(),
                    catalog_song_id: None, // Library items don't always have catalog IDs
                    name: track.attributes.name.clone(),
                    artist_name: track.attributes.artist_name.clone(),
                    album_name: track.attributes.album_name.clone(),
                    blocked_artist_id: None, // Would need entity resolution to get this
                });
            }
        }

        // Find blocked albums
        for album in library_albums {
            let artist_lower = album.attributes.artist_name.to_lowercase();
            if blocked_names_lower.iter().any(|blocked| artist_lower.contains(blocked)) {
                scan.blocked_albums.push(BlockedAlbumInfo {
                    library_album_id: album.id.clone(),
                    catalog_album_id: None,
                    name: album.attributes.name.clone(),
                    artist_name: album.attributes.artist_name.clone(),
                    blocked_artist_id: None,
                });
            }
        }

        Ok(scan)
    }

    /// Preview what would be enforced (dry run)
    pub async fn preview_enforcement(
        &self,
        user_id: Uuid,
        blocked_artist_names: Vec<String>,
    ) -> Result<EnforcementPreview> {
        let connection = self.apple_music
            .get_user_connection(user_id)
            .await?
            .ok_or_else(|| anyhow!("No Apple Music connection found for user"))?;

        let blocked_content = self.scan_for_blocked_content(
            &connection,
            &blocked_artist_names,
        ).await?;

        // Estimate ~50ms per API call
        let total_items = blocked_content.blocked_songs.len() + blocked_content.blocked_albums.len();
        let estimated_duration = (total_items as u64) * 50 / 1000;

        Ok(EnforcementPreview {
            songs_to_dislike: blocked_content.blocked_songs,
            albums_to_dislike: blocked_content.blocked_albums,
            total_songs: blocked_content.total_songs_scanned,
            total_albums: blocked_content.total_albums_scanned,
            estimated_duration_seconds: estimated_duration,
        })
    }

    /// Rollback an enforcement run (remove dislikes)
    pub async fn rollback_enforcement(
        &self,
        user_id: Uuid,
        run_id: Uuid,
    ) -> Result<RollbackResult> {
        let start_time = Instant::now();

        let connection = self.apple_music
            .get_user_connection(user_id)
            .await?
            .ok_or_else(|| anyhow!("No Apple Music connection found for user"))?;

        // Get all actions from this run
        let actions = self.get_enforcement_actions(&run_id).await?;

        let mut ratings_removed = 0;
        let mut errors = Vec::new();

        for action in actions {
            let result = match action.resource_type.as_str() {
                "library_song" => {
                    self.apple_music
                        .delete_library_song_rating(&connection, &action.resource_id)
                        .await
                }
                "library_album" => {
                    self.apple_music
                        .delete_library_album_rating(&connection, &action.resource_id)
                        .await
                }
                "song" => {
                    self.apple_music
                        .delete_song_rating(&connection, &action.resource_id)
                        .await
                }
                "album" => {
                    self.apple_music
                        .delete_album_rating(&connection, &action.resource_id)
                        .await
                }
                _ => {
                    Err(anyhow!("Unknown resource type: {}", action.resource_type))
                }
            };

            match result {
                Ok(()) => ratings_removed += 1,
                Err(e) => {
                    errors.push(RatingError {
                        resource_id: action.resource_id,
                        resource_type: action.resource_type,
                        error_message: e.to_string(),
                    });
                }
            }
        }

        // Update run status to rolled back
        self.update_enforcement_run_status(&run_id, EnforcementRunStatus::RolledBack).await?;

        Ok(RollbackResult {
            run_id,
            ratings_removed,
            errors,
            duration_seconds: start_time.elapsed().as_secs(),
        })
    }

    /// Get enforcement history for a user
    pub async fn get_enforcement_history(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<EnforcementHistoryItem>> {
        let rows: Vec<EnforcementHistoryRow> = sqlx::query_as(
            r#"
            SELECT
                id, user_id, connection_id, status, options,
                started_at, completed_at,
                songs_scanned, albums_scanned, songs_disliked, albums_disliked,
                errors, error_details
            FROM apple_music_enforcement_runs
            WHERE user_id = $1
            ORDER BY started_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.db_pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    // ============================================
    // Database Operations
    // ============================================

    async fn create_enforcement_run(
        &self,
        user_id: Uuid,
        connection_id: Uuid,
        options: &AppleMusicRatingEnforcementOptions,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let options_json = serde_json::to_value(options)?;

        sqlx::query(
            r#"
            INSERT INTO apple_music_enforcement_runs
            (id, user_id, connection_id, status, options, started_at)
            VALUES ($1, $2, $3, 'running', $4, NOW())
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(connection_id)
        .bind(options_json)
        .execute(&self.db_pool)
        .await?;

        Ok(id)
    }

    async fn update_enforcement_run(
        &self,
        run_id: &Uuid,
        status: EnforcementRunStatus,
        songs_scanned: u32,
        albums_scanned: u32,
        songs_disliked: u32,
        albums_disliked: u32,
        errors: u32,
        error_details: Option<serde_json::Value>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE apple_music_enforcement_runs
            SET
                status = $2,
                completed_at = NOW(),
                songs_scanned = $3,
                albums_scanned = $4,
                songs_disliked = $5,
                albums_disliked = $6,
                errors = $7,
                error_details = $8
            WHERE id = $1
            "#,
        )
        .bind(run_id)
        .bind(status.to_string())
        .bind(songs_scanned as i32)
        .bind(albums_scanned as i32)
        .bind(songs_disliked as i32)
        .bind(albums_disliked as i32)
        .bind(errors as i32)
        .bind(error_details)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn update_enforcement_run_status(
        &self,
        run_id: &Uuid,
        status: EnforcementRunStatus,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE apple_music_enforcement_runs
            SET status = $2, completed_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(run_id)
        .bind(status.to_string())
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn record_enforcement_action(
        &self,
        run_id: &Uuid,
        user_id: Uuid,
        resource_type: AppleMusicResourceType,
        resource_id: &str,
        resource_name: Option<&str>,
        artist_name: Option<&str>,
        action: EnforcementActionType,
        previous_rating: Option<i8>,
    ) -> Result<()> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO apple_music_enforcement_actions
            (id, run_id, user_id, resource_type, resource_id, resource_name, artist_name, action, previous_rating, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
            "#,
        )
        .bind(id)
        .bind(run_id)
        .bind(user_id)
        .bind(resource_type.to_string())
        .bind(resource_id)
        .bind(resource_name)
        .bind(artist_name)
        .bind(action.to_string())
        .bind(previous_rating.map(|r| r as i16))
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn get_enforcement_actions(&self, run_id: &Uuid) -> Result<Vec<EnforcementActionRow>> {
        let rows: Vec<EnforcementActionRow> = sqlx::query_as(
            r#"
            SELECT id, run_id, user_id, resource_type, resource_id, resource_name, artist_name, action, previous_rating, created_at
            FROM apple_music_enforcement_actions
            WHERE run_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(run_id)
        .fetch_all(&self.db_pool)
        .await?;

        Ok(rows)
    }
}

// Database row types

#[derive(Debug, sqlx::FromRow)]
struct EnforcementHistoryRow {
    id: Uuid,
    user_id: Uuid,
    connection_id: Uuid,
    status: String,
    options: serde_json::Value,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    songs_scanned: i32,
    albums_scanned: i32,
    songs_disliked: i32,
    albums_disliked: i32,
    errors: i32,
    error_details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EnforcementHistoryItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub connection_id: Uuid,
    pub status: String,
    pub options: AppleMusicRatingEnforcementOptions,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub songs_scanned: u32,
    pub albums_scanned: u32,
    pub songs_disliked: u32,
    pub albums_disliked: u32,
    pub errors: u32,
}

impl From<EnforcementHistoryRow> for EnforcementHistoryItem {
    fn from(row: EnforcementHistoryRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            connection_id: row.connection_id,
            status: row.status,
            options: serde_json::from_value(row.options).unwrap_or_default(),
            started_at: row.started_at,
            completed_at: row.completed_at,
            songs_scanned: row.songs_scanned as u32,
            albums_scanned: row.albums_scanned as u32,
            songs_disliked: row.songs_disliked as u32,
            albums_disliked: row.albums_disliked as u32,
            errors: row.errors as u32,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct EnforcementActionRow {
    id: Uuid,
    run_id: Uuid,
    user_id: Uuid,
    resource_type: String,
    resource_id: String,
    resource_name: Option<String>,
    artist_name: Option<String>,
    action: String,
    previous_rating: Option<i16>,
    created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================
    // EnforcementProgress Tests
    // ============================================

    #[test]
    fn test_enforcement_progress_new() {
        let progress = EnforcementProgress::new();
        assert_eq!(progress.phase, "initializing");
        assert_eq!(progress.total_items, 0);
        assert_eq!(progress.processed_items, 0);
        assert_eq!(progress.songs_disliked, 0);
        assert_eq!(progress.albums_disliked, 0);
        assert_eq!(progress.errors, 0);
        assert!(progress.current_item.is_none());
    }

    #[test]
    fn test_enforcement_progress_percent_complete_zero_total() {
        let progress = EnforcementProgress::new();
        assert_eq!(progress.percent_complete(), 0.0);
    }

    #[test]
    fn test_enforcement_progress_percent_complete_partial() {
        let mut progress = EnforcementProgress::new();
        progress.total_items = 100;
        progress.processed_items = 50;
        assert_eq!(progress.percent_complete(), 50.0);
    }

    #[test]
    fn test_enforcement_progress_percent_complete_full() {
        let mut progress = EnforcementProgress::new();
        progress.total_items = 100;
        progress.processed_items = 100;
        assert_eq!(progress.percent_complete(), 100.0);
    }

    #[test]
    fn test_enforcement_progress_percent_complete_quarter() {
        let mut progress = EnforcementProgress::new();
        progress.total_items = 200;
        progress.processed_items = 50;
        assert_eq!(progress.percent_complete(), 25.0);
    }

    #[test]
    fn test_enforcement_progress_default() {
        let progress = EnforcementProgress::default();
        assert_eq!(progress.phase, "initializing");
        assert_eq!(progress.percent_complete(), 0.0);
    }

    #[test]
    fn test_enforcement_progress_clone() {
        let mut progress = EnforcementProgress::new();
        progress.phase = "scanning".to_string();
        progress.total_items = 50;
        progress.processed_items = 25;
        progress.songs_disliked = 10;
        progress.albums_disliked = 5;
        progress.errors = 2;
        progress.current_item = Some("Test Song".to_string());

        let cloned = progress.clone();
        assert_eq!(cloned.phase, "scanning");
        assert_eq!(cloned.total_items, 50);
        assert_eq!(cloned.processed_items, 25);
        assert_eq!(cloned.songs_disliked, 10);
        assert_eq!(cloned.albums_disliked, 5);
        assert_eq!(cloned.errors, 2);
        assert_eq!(cloned.current_item, Some("Test Song".to_string()));
    }

    // ============================================
    // BlockedContentScan Tests
    // ============================================

    #[test]
    fn test_blocked_content_scan_new() {
        let scan = BlockedContentScan::new();
        assert_eq!(scan.total_blocked(), 0);
        assert!(scan.blocked_songs.is_empty());
        assert!(scan.blocked_albums.is_empty());
        assert_eq!(scan.total_songs_scanned, 0);
        assert_eq!(scan.total_albums_scanned, 0);
    }

    #[test]
    fn test_blocked_content_scan_default() {
        let scan = BlockedContentScan::default();
        assert_eq!(scan.total_blocked(), 0);
    }

    #[test]
    fn test_blocked_content_scan_with_songs() {
        let mut scan = BlockedContentScan::new();
        scan.blocked_songs.push(BlockedSongInfo {
            library_song_id: "song1".to_string(),
            catalog_song_id: Some("cat1".to_string()),
            name: "Test Song".to_string(),
            artist_name: "Test Artist".to_string(),
            album_name: "Test Album".to_string(),
            blocked_artist_id: None,
        });
        scan.blocked_songs.push(BlockedSongInfo {
            library_song_id: "song2".to_string(),
            catalog_song_id: None,
            name: "Another Song".to_string(),
            artist_name: "Test Artist".to_string(),
            album_name: "Another Album".to_string(),
            blocked_artist_id: Some(Uuid::new_v4()),
        });

        assert_eq!(scan.total_blocked(), 2);
        assert_eq!(scan.blocked_songs.len(), 2);
    }

    #[test]
    fn test_blocked_content_scan_with_albums() {
        let mut scan = BlockedContentScan::new();
        scan.blocked_albums.push(BlockedAlbumInfo {
            library_album_id: "album1".to_string(),
            catalog_album_id: Some("cat1".to_string()),
            name: "Test Album".to_string(),
            artist_name: "Test Artist".to_string(),
            blocked_artist_id: None,
        });

        assert_eq!(scan.total_blocked(), 1);
        assert_eq!(scan.blocked_albums.len(), 1);
    }

    #[test]
    fn test_blocked_content_scan_with_both() {
        let mut scan = BlockedContentScan::new();
        scan.blocked_songs.push(BlockedSongInfo {
            library_song_id: "song1".to_string(),
            catalog_song_id: None,
            name: "Song".to_string(),
            artist_name: "Artist".to_string(),
            album_name: "Album".to_string(),
            blocked_artist_id: None,
        });
        scan.blocked_albums.push(BlockedAlbumInfo {
            library_album_id: "album1".to_string(),
            catalog_album_id: None,
            name: "Album".to_string(),
            artist_name: "Artist".to_string(),
            blocked_artist_id: None,
        });
        scan.total_songs_scanned = 100;
        scan.total_albums_scanned = 50;

        assert_eq!(scan.total_blocked(), 2);
        assert_eq!(scan.total_songs_scanned, 100);
        assert_eq!(scan.total_albums_scanned, 50);
    }

    // ============================================
    // EnforcementHistoryItem Tests
    // ============================================

    #[test]
    fn test_enforcement_history_item_from_row() {
        let user_id = Uuid::new_v4();
        let connection_id = Uuid::new_v4();
        let id = Uuid::new_v4();
        let started_at = Utc::now();
        let completed_at = Some(started_at + chrono::Duration::seconds(30));

        let row = EnforcementHistoryRow {
            id,
            user_id,
            connection_id,
            status: "completed".to_string(),
            options: serde_json::json!({
                "dislike_songs": true,
                "dislike_albums": true,
                "include_library": true,
                "include_catalog": false,
                "batch_size": 50,
                "dry_run": false
            }),
            started_at,
            completed_at,
            songs_scanned: 100,
            albums_scanned: 20,
            songs_disliked: 5,
            albums_disliked: 2,
            errors: 0,
            error_details: None,
        };

        let item: EnforcementHistoryItem = row.into();

        assert_eq!(item.id, id);
        assert_eq!(item.user_id, user_id);
        assert_eq!(item.connection_id, connection_id);
        assert_eq!(item.status, "completed");
        assert_eq!(item.songs_scanned, 100);
        assert_eq!(item.albums_scanned, 20);
        assert_eq!(item.songs_disliked, 5);
        assert_eq!(item.albums_disliked, 2);
        assert_eq!(item.errors, 0);
        assert!(item.options.dislike_songs);
        assert!(item.options.dislike_albums);
    }

    #[test]
    fn test_enforcement_history_item_with_errors() {
        let user_id = Uuid::new_v4();
        let connection_id = Uuid::new_v4();
        let id = Uuid::new_v4();

        let row = EnforcementHistoryRow {
            id,
            user_id,
            connection_id,
            status: "completed".to_string(),
            options: serde_json::json!({}),
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            songs_scanned: 50,
            albums_scanned: 10,
            songs_disliked: 3,
            albums_disliked: 1,
            errors: 2,
            error_details: Some(serde_json::json!([
                {"resource_id": "song1", "error": "Rate limited"},
                {"resource_id": "album1", "error": "Not found"}
            ])),
        };

        let item: EnforcementHistoryItem = row.into();
        assert_eq!(item.errors, 2);
    }

    // ============================================
    // BlockedSongInfo Tests
    // ============================================

    #[test]
    fn test_blocked_song_info_creation() {
        let artist_id = Uuid::new_v4();
        let song = BlockedSongInfo {
            library_song_id: "lib-123".to_string(),
            catalog_song_id: Some("cat-456".to_string()),
            name: "Problematic Song".to_string(),
            artist_name: "Blocked Artist".to_string(),
            album_name: "Controversial Album".to_string(),
            blocked_artist_id: Some(artist_id),
        };

        assert_eq!(song.library_song_id, "lib-123");
        assert_eq!(song.catalog_song_id, Some("cat-456".to_string()));
        assert_eq!(song.name, "Problematic Song");
        assert_eq!(song.artist_name, "Blocked Artist");
        assert_eq!(song.album_name, "Controversial Album");
        assert_eq!(song.blocked_artist_id, Some(artist_id));
    }

    #[test]
    fn test_blocked_song_info_serialization() {
        let song = BlockedSongInfo {
            library_song_id: "lib-123".to_string(),
            catalog_song_id: None,
            name: "Test".to_string(),
            artist_name: "Artist".to_string(),
            album_name: "Album".to_string(),
            blocked_artist_id: None,
        };

        let json = serde_json::to_string(&song).unwrap();
        let deserialized: BlockedSongInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.library_song_id, song.library_song_id);
        assert_eq!(deserialized.name, song.name);
    }

    // ============================================
    // BlockedAlbumInfo Tests
    // ============================================

    #[test]
    fn test_blocked_album_info_creation() {
        let artist_id = Uuid::new_v4();
        let album = BlockedAlbumInfo {
            library_album_id: "lib-album-123".to_string(),
            catalog_album_id: Some("cat-album-456".to_string()),
            name: "Problematic Album".to_string(),
            artist_name: "Blocked Artist".to_string(),
            blocked_artist_id: Some(artist_id),
        };

        assert_eq!(album.library_album_id, "lib-album-123");
        assert_eq!(album.catalog_album_id, Some("cat-album-456".to_string()));
        assert_eq!(album.name, "Problematic Album");
        assert_eq!(album.artist_name, "Blocked Artist");
        assert_eq!(album.blocked_artist_id, Some(artist_id));
    }

    #[test]
    fn test_blocked_album_info_serialization() {
        let album = BlockedAlbumInfo {
            library_album_id: "lib-123".to_string(),
            catalog_album_id: None,
            name: "Test Album".to_string(),
            artist_name: "Artist".to_string(),
            blocked_artist_id: None,
        };

        let json = serde_json::to_string(&album).unwrap();
        let deserialized: BlockedAlbumInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.library_album_id, album.library_album_id);
        assert_eq!(deserialized.name, album.name);
    }

    // ============================================
    // EnforcementActionRow Tests
    // ============================================

    #[test]
    fn test_enforcement_action_row_structure() {
        let run_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let id = Uuid::new_v4();

        let row = EnforcementActionRow {
            id,
            run_id,
            user_id,
            resource_type: "library_song".to_string(),
            resource_id: "song-123".to_string(),
            resource_name: Some("Test Song".to_string()),
            artist_name: Some("Test Artist".to_string()),
            action: "dislike".to_string(),
            previous_rating: None,
            created_at: Utc::now(),
        };

        assert_eq!(row.id, id);
        assert_eq!(row.run_id, run_id);
        assert_eq!(row.user_id, user_id);
        assert_eq!(row.resource_type, "library_song");
        assert_eq!(row.resource_id, "song-123");
        assert_eq!(row.action, "dislike");
        assert!(row.previous_rating.is_none());
    }

    #[test]
    fn test_enforcement_action_row_with_previous_rating() {
        let row = EnforcementActionRow {
            id: Uuid::new_v4(),
            run_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            resource_type: "library_song".to_string(),
            resource_id: "song-123".to_string(),
            resource_name: Some("Test Song".to_string()),
            artist_name: Some("Test Artist".to_string()),
            action: "dislike".to_string(),
            previous_rating: Some(1), // Was previously liked
            created_at: Utc::now(),
        };

        assert_eq!(row.previous_rating, Some(1));
    }

    // ============================================
    // RatingError Tests
    // ============================================

    #[test]
    fn test_rating_error_creation() {
        let error = RatingError {
            resource_id: "song-123".to_string(),
            resource_type: "library_song".to_string(),
            error_message: "Rate limit exceeded".to_string(),
        };

        assert_eq!(error.resource_id, "song-123");
        assert_eq!(error.resource_type, "library_song");
        assert_eq!(error.error_message, "Rate limit exceeded");
    }

    #[test]
    fn test_rating_error_serialization() {
        let error = RatingError {
            resource_id: "song-123".to_string(),
            resource_type: "library_song".to_string(),
            error_message: "API error: 429".to_string(),
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("song-123"));
        assert!(json.contains("library_song"));
        assert!(json.contains("API error: 429"));

        let deserialized: RatingError = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.resource_id, error.resource_id);
    }

    // ============================================
    // AppleMusicRatingEnforcementOptions Tests
    // ============================================

    #[test]
    fn test_enforcement_options_default() {
        let options = AppleMusicRatingEnforcementOptions::default();

        assert!(options.dislike_songs);
        assert!(options.dislike_albums);
        assert!(options.include_library);
        assert!(!options.include_catalog);
        assert_eq!(options.batch_size, 50);
        assert!(!options.dry_run);
    }

    #[test]
    fn test_enforcement_options_custom() {
        let options = AppleMusicRatingEnforcementOptions {
            dislike_songs: true,
            dislike_albums: false,
            include_library: true,
            include_catalog: true,
            batch_size: 100,
            dry_run: true,
        };

        assert!(options.dislike_songs);
        assert!(!options.dislike_albums);
        assert!(options.include_library);
        assert!(options.include_catalog);
        assert_eq!(options.batch_size, 100);
        assert!(options.dry_run);
    }

    #[test]
    fn test_enforcement_options_serialization() {
        let options = AppleMusicRatingEnforcementOptions {
            dislike_songs: true,
            dislike_albums: true,
            include_library: true,
            include_catalog: false,
            batch_size: 25,
            dry_run: false,
        };

        let json = serde_json::to_string(&options).unwrap();
        let deserialized: AppleMusicRatingEnforcementOptions = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.dislike_songs, options.dislike_songs);
        assert_eq!(deserialized.dislike_albums, options.dislike_albums);
        assert_eq!(deserialized.batch_size, options.batch_size);
    }

    // ============================================
    // EnforcementRunStatus Tests
    // ============================================

    #[test]
    fn test_enforcement_run_status_display() {
        assert_eq!(EnforcementRunStatus::Pending.to_string(), "pending");
        assert_eq!(EnforcementRunStatus::Running.to_string(), "running");
        assert_eq!(EnforcementRunStatus::Completed.to_string(), "completed");
        assert_eq!(EnforcementRunStatus::Failed.to_string(), "failed");
        assert_eq!(EnforcementRunStatus::RolledBack.to_string(), "rolled_back");
    }

    #[test]
    fn test_enforcement_run_status_equality() {
        assert_eq!(EnforcementRunStatus::Completed, EnforcementRunStatus::Completed);
        assert_ne!(EnforcementRunStatus::Completed, EnforcementRunStatus::Failed);
    }

    #[test]
    fn test_enforcement_run_status_clone() {
        let status = EnforcementRunStatus::Running;
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    // ============================================
    // AppleMusicResourceType Tests
    // ============================================

    #[test]
    fn test_resource_type_display() {
        assert_eq!(AppleMusicResourceType::Song.to_string(), "song");
        assert_eq!(AppleMusicResourceType::LibrarySong.to_string(), "library_song");
        assert_eq!(AppleMusicResourceType::Album.to_string(), "album");
        assert_eq!(AppleMusicResourceType::LibraryAlbum.to_string(), "library_album");
        assert_eq!(AppleMusicResourceType::Playlist.to_string(), "playlist");
        assert_eq!(AppleMusicResourceType::LibraryPlaylist.to_string(), "library_playlist");
    }

    #[test]
    fn test_resource_type_equality() {
        assert_eq!(AppleMusicResourceType::Song, AppleMusicResourceType::Song);
        assert_ne!(AppleMusicResourceType::Song, AppleMusicResourceType::LibrarySong);
    }

    // ============================================
    // EnforcementActionType Tests
    // ============================================

    #[test]
    fn test_action_type_display() {
        assert_eq!(EnforcementActionType::Dislike.to_string(), "dislike");
        assert_eq!(EnforcementActionType::RemoveRating.to_string(), "remove_rating");
    }

    #[test]
    fn test_action_type_equality() {
        assert_eq!(EnforcementActionType::Dislike, EnforcementActionType::Dislike);
        assert_ne!(EnforcementActionType::Dislike, EnforcementActionType::RemoveRating);
    }

    // ============================================
    // RatingEnforcementResult Tests
    // ============================================

    #[test]
    fn test_rating_enforcement_result_success() {
        let run_id = Uuid::new_v4();
        let result = RatingEnforcementResult {
            run_id,
            status: EnforcementRunStatus::Completed,
            songs_disliked: 10,
            albums_disliked: 5,
            errors: Vec::new(),
            duration_seconds: 30,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };

        assert_eq!(result.run_id, run_id);
        assert_eq!(result.status, EnforcementRunStatus::Completed);
        assert_eq!(result.songs_disliked, 10);
        assert_eq!(result.albums_disliked, 5);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_rating_enforcement_result_with_errors() {
        let run_id = Uuid::new_v4();
        let result = RatingEnforcementResult {
            run_id,
            status: EnforcementRunStatus::Completed,
            songs_disliked: 8,
            albums_disliked: 4,
            errors: vec![
                RatingError {
                    resource_id: "song-1".to_string(),
                    resource_type: "library_song".to_string(),
                    error_message: "Rate limited".to_string(),
                },
                RatingError {
                    resource_id: "album-1".to_string(),
                    resource_type: "library_album".to_string(),
                    error_message: "Not found".to_string(),
                },
            ],
            duration_seconds: 45,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };

        assert_eq!(result.errors.len(), 2);
        assert_eq!(result.songs_disliked, 8);
    }

    // ============================================
    // RollbackResult Tests
    // ============================================

    #[test]
    fn test_rollback_result_success() {
        let run_id = Uuid::new_v4();
        let result = RollbackResult {
            run_id,
            ratings_removed: 15,
            errors: Vec::new(),
            duration_seconds: 20,
        };

        assert_eq!(result.run_id, run_id);
        assert_eq!(result.ratings_removed, 15);
        assert!(result.errors.is_empty());
        assert_eq!(result.duration_seconds, 20);
    }

    #[test]
    fn test_rollback_result_with_errors() {
        let run_id = Uuid::new_v4();
        let result = RollbackResult {
            run_id,
            ratings_removed: 10,
            errors: vec![RatingError {
                resource_id: "song-1".to_string(),
                resource_type: "library_song".to_string(),
                error_message: "Failed to delete rating".to_string(),
            }],
            duration_seconds: 25,
        };

        assert_eq!(result.ratings_removed, 10);
        assert_eq!(result.errors.len(), 1);
    }

    // ============================================
    // EnforcementPreview Tests
    // ============================================

    #[test]
    fn test_enforcement_preview_empty() {
        let preview = EnforcementPreview {
            songs_to_dislike: Vec::new(),
            albums_to_dislike: Vec::new(),
            total_songs: 100,
            total_albums: 50,
            estimated_duration_seconds: 0,
        };

        assert!(preview.songs_to_dislike.is_empty());
        assert!(preview.albums_to_dislike.is_empty());
        assert_eq!(preview.total_songs, 100);
        assert_eq!(preview.total_albums, 50);
    }

    #[test]
    fn test_enforcement_preview_with_content() {
        let preview = EnforcementPreview {
            songs_to_dislike: vec![
                BlockedSongInfo {
                    library_song_id: "song-1".to_string(),
                    catalog_song_id: None,
                    name: "Song 1".to_string(),
                    artist_name: "Artist".to_string(),
                    album_name: "Album".to_string(),
                    blocked_artist_id: None,
                },
            ],
            albums_to_dislike: vec![
                BlockedAlbumInfo {
                    library_album_id: "album-1".to_string(),
                    catalog_album_id: None,
                    name: "Album 1".to_string(),
                    artist_name: "Artist".to_string(),
                    blocked_artist_id: None,
                },
            ],
            total_songs: 500,
            total_albums: 100,
            estimated_duration_seconds: 1, // (2 items * 50ms) / 1000
        };

        assert_eq!(preview.songs_to_dislike.len(), 1);
        assert_eq!(preview.albums_to_dislike.len(), 1);
        assert_eq!(preview.estimated_duration_seconds, 1);
    }

    // ============================================
    // Integration-style Tests (without DB)
    // ============================================

    #[test]
    fn test_artist_name_matching_case_insensitive() {
        // Test the matching logic used in scan_for_blocked_content
        let blocked_names = vec!["drake".to_string(), "r. kelly".to_string()];
        let blocked_names_lower: Vec<String> = blocked_names
            .iter()
            .map(|n| n.to_lowercase())
            .collect();

        // Test various artist names
        let test_cases = vec![
            ("Drake", true),
            ("DRAKE", true),
            ("drake", true),
            ("DrAkE", true),
            ("Drake & Future", true),
            ("R. Kelly", true),
            ("r. kelly", true),
            ("Kendrick Lamar", false),
            ("Taylor Swift", false),
        ];

        for (artist_name, should_match) in test_cases {
            let artist_lower = artist_name.to_lowercase();
            let matches = blocked_names_lower.iter().any(|blocked| artist_lower.contains(blocked));
            assert_eq!(
                matches, should_match,
                "Artist '{}' should {} be blocked",
                artist_name,
                if should_match { "" } else { "not" }
            );
        }
    }

    #[test]
    fn test_partial_artist_name_matching() {
        // Test that partial matches work (e.g., "Drake" matches "Drake & Future")
        let blocked_names = vec!["drake".to_string()];
        let blocked_names_lower: Vec<String> = blocked_names
            .iter()
            .map(|n| n.to_lowercase())
            .collect();

        let collaborations = vec![
            ("Drake & Future", true),
            ("Future & Drake", true),
            ("21 Savage, Drake, Metro Boomin", true),
            ("Drakeo the Ruler", true), // This is a false positive but expected
            ("Future", false),
            ("Kanye West", false),
        ];

        for (artist_name, should_match) in collaborations {
            let artist_lower = artist_name.to_lowercase();
            let matches = blocked_names_lower.iter().any(|blocked| artist_lower.contains(blocked));
            assert_eq!(
                matches, should_match,
                "Artist '{}' should {} match",
                artist_name,
                if should_match { "" } else { "not" }
            );
        }
    }

    #[test]
    fn test_estimated_duration_calculation() {
        // Test the estimation logic: ~50ms per API call
        let test_cases = vec![
            (0, 0),      // 0 items = 0 seconds
            (1, 0),      // 1 item = 0 seconds (50ms rounds down)
            (20, 1),     // 20 items = 1 second
            (40, 2),     // 40 items = 2 seconds
            (100, 5),    // 100 items = 5 seconds
            (1000, 50),  // 1000 items = 50 seconds
        ];

        for (total_items, expected_seconds) in test_cases {
            let estimated_duration = (total_items as u64) * 50 / 1000;
            assert_eq!(
                estimated_duration, expected_seconds,
                "Expected {} seconds for {} items, got {}",
                expected_seconds, total_items, estimated_duration
            );
        }
    }

    #[test]
    fn test_enforcement_status_determination() {
        // Test status determination logic based on results
        let test_cases = vec![
            // (songs_disliked, albums_disliked, error_count, expected_status)
            (10, 5, 0, EnforcementRunStatus::Completed),  // All successful
            (10, 5, 2, EnforcementRunStatus::Completed),  // Partial success
            (0, 0, 5, EnforcementRunStatus::Failed),      // All failed
            (1, 0, 10, EnforcementRunStatus::Completed),  // At least one success
        ];

        for (songs, albums, errors, expected) in test_cases {
            let status = if errors == 0 {
                EnforcementRunStatus::Completed
            } else if songs + albums > 0 {
                EnforcementRunStatus::Completed // Partial success
            } else {
                EnforcementRunStatus::Failed
            };

            assert_eq!(
                status, expected,
                "songs={}, albums={}, errors={} should result in {:?}",
                songs, albums, errors, expected
            );
        }
    }
}
