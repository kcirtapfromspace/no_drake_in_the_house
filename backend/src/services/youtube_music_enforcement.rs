//! YouTube Music Enforcement Service
//!
//! Implements enforcement operations for YouTube Music via the YouTube Data API v3.
//! Supports removing liked videos, playlist items, and unsubscribing from channels.

use std::sync::Arc;
use std::time::Instant;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::youtube_music::{
    BlockedPlaylistItemInfo, BlockedSubscriptionInfo, BlockedVideoInfo, YouTubeRating,
    YouTubeMusicBlockedContentScan, YouTubeMusicEnforcementAction,
    YouTubeMusicEnforcementActionType, YouTubeMusicEnforcementError,
    YouTubeMusicEnforcementOptions, YouTubeMusicEnforcementPreview,
    YouTubeMusicEnforcementProgress, YouTubeMusicEnforcementResult,
    YouTubeMusicEnforcementRun, YouTubeMusicEnforcementRunStatus,
    YouTubeMusicResourceType, YouTubeMusicRollbackResult,
};
use crate::services::youtube_music_library::YouTubeMusicLibraryService;

/// YouTube Music enforcement service
pub struct YouTubeMusicEnforcementService {
    db_pool: PgPool,
    library_service: Arc<YouTubeMusicLibraryService>,
    client: reqwest::Client,
}

impl YouTubeMusicEnforcementService {
    const YOUTUBE_API_BASE: &'static str = "https://www.googleapis.com/youtube/v3";
    const BATCH_DELAY_MS: u64 = 200; // Delay between API calls for rate limiting

    /// Create a new YouTube Music enforcement service
    pub fn new(db_pool: PgPool, library_service: Arc<YouTubeMusicLibraryService>) -> Self {
        Self {
            db_pool,
            library_service,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Preview enforcement without making any changes
    pub async fn preview_enforcement(
        &self,
        user_id: Uuid,
        access_token: &str,
        blocked_artists: Vec<String>,
    ) -> Result<YouTubeMusicEnforcementPreview> {
        let scan = self
            .library_service
            .scan_for_blocked_content(access_token, &blocked_artists)
            .await?;

        // Estimate duration based on number of items (200ms per item + overhead)
        let total_items = scan.blocked_videos.len()
            + scan.blocked_playlist_items.len()
            + scan.blocked_subscriptions.len();
        let estimated_duration_seconds = ((total_items as u64) * 250 / 1000) + 5;

        Ok(YouTubeMusicEnforcementPreview {
            videos_to_remove: scan.blocked_videos,
            playlist_items_to_remove: scan.blocked_playlist_items,
            subscriptions_to_remove: scan.blocked_subscriptions,
            total_videos: scan.total_videos_scanned,
            total_playlist_items: scan.total_playlist_items_scanned,
            total_subscriptions: scan.total_subscriptions_scanned,
            estimated_duration_seconds,
        })
    }

    /// Execute enforcement on a user's YouTube Music library
    pub async fn execute_enforcement<F>(
        &self,
        user_id: Uuid,
        connection_id: Uuid,
        access_token: &str,
        blocked_artists: Vec<String>,
        options: YouTubeMusicEnforcementOptions,
        mut progress_callback: F,
    ) -> Result<YouTubeMusicEnforcementResult>
    where
        F: FnMut(YouTubeMusicEnforcementProgress),
    {
        let start_time = Instant::now();
        let run_id = Uuid::new_v4();

        // Create enforcement run record
        let run = self
            .create_enforcement_run(run_id, user_id, connection_id, &options)
            .await?;

        // Scan for blocked content
        let scan = self
            .library_service
            .scan_for_blocked_content(access_token, &blocked_artists)
            .await?;

        let total_items = scan.blocked_videos.len()
            + scan.blocked_playlist_items.len()
            + scan.blocked_subscriptions.len();

        let mut progress = YouTubeMusicEnforcementProgress {
            phase: "scanning".to_string(),
            total_items,
            processed_items: 0,
            videos_removed: 0,
            playlist_items_removed: 0,
            subscriptions_removed: 0,
            errors: 0,
            current_item: None,
        };
        progress_callback(progress.clone());

        // If dry run, just return preview results
        if options.dry_run {
            return Ok(YouTubeMusicEnforcementResult {
                run_id,
                status: YouTubeMusicEnforcementRunStatus::Completed,
                videos_removed: scan.blocked_videos.len(),
                playlist_items_removed: scan.blocked_playlist_items.len(),
                subscriptions_removed: scan.blocked_subscriptions.len(),
                errors: Vec::new(),
                duration_seconds: start_time.elapsed().as_secs(),
                started_at: run.started_at,
                completed_at: Some(Utc::now()),
            });
        }

        let mut errors = Vec::new();

        // Phase 1: Remove from likes
        if options.remove_from_likes {
            progress.phase = "removing_likes".to_string();
            progress_callback(progress.clone());

            for video in &scan.blocked_videos {
                progress.current_item = Some(video.title.clone());
                progress_callback(progress.clone());

                match self.remove_like(access_token, &video.video_id).await {
                    Ok(_) => {
                        progress.videos_removed += 1;
                        // Record action for rollback
                        self.record_enforcement_action(
                            run_id,
                            user_id,
                            YouTubeMusicResourceType::LikedVideo,
                            &video.video_id,
                            Some(&video.title),
                            Some(&video.channel_title),
                            YouTubeMusicEnforcementActionType::RemoveFromLikes,
                            None,
                        )
                        .await?;
                    }
                    Err(e) => {
                        progress.errors += 1;
                        errors.push(YouTubeMusicEnforcementError {
                            resource_id: video.video_id.clone(),
                            resource_type: "liked_video".to_string(),
                            error_message: e.to_string(),
                        });
                    }
                }

                progress.processed_items += 1;
                progress_callback(progress.clone());
                tokio::time::sleep(std::time::Duration::from_millis(Self::BATCH_DELAY_MS)).await;
            }
        }

        // Phase 2: Dislike blocked content (optional)
        if options.dislike_blocked_content {
            progress.phase = "disliking".to_string();
            progress_callback(progress.clone());

            for video in &scan.blocked_videos {
                progress.current_item = Some(video.title.clone());
                progress_callback(progress.clone());

                match self.rate_video(access_token, &video.video_id, YouTubeRating::Dislike).await {
                    Ok(_) => {
                        self.record_enforcement_action(
                            run_id,
                            user_id,
                            YouTubeMusicResourceType::Video,
                            &video.video_id,
                            Some(&video.title),
                            Some(&video.channel_title),
                            YouTubeMusicEnforcementActionType::Dislike,
                            None,
                        )
                        .await?;
                    }
                    Err(e) => {
                        // Don't count dislike failures as critical
                        tracing::warn!(
                            video_id = %video.video_id,
                            error = %e,
                            "Failed to dislike video"
                        );
                    }
                }

                tokio::time::sleep(std::time::Duration::from_millis(Self::BATCH_DELAY_MS)).await;
            }
        }

        // Phase 3: Remove from playlists
        if options.remove_from_playlists {
            progress.phase = "removing_from_playlists".to_string();
            progress_callback(progress.clone());

            for item in &scan.blocked_playlist_items {
                progress.current_item = Some(item.title.clone());
                progress_callback(progress.clone());

                match self.remove_playlist_item(access_token, &item.playlist_item_id).await {
                    Ok(_) => {
                        progress.playlist_items_removed += 1;
                        self.record_enforcement_action(
                            run_id,
                            user_id,
                            YouTubeMusicResourceType::PlaylistItem,
                            &item.playlist_item_id,
                            Some(&item.title),
                            Some(&item.channel_title),
                            YouTubeMusicEnforcementActionType::RemoveFromPlaylist,
                            Some(&item.playlist_id),
                        )
                        .await?;
                    }
                    Err(e) => {
                        progress.errors += 1;
                        errors.push(YouTubeMusicEnforcementError {
                            resource_id: item.playlist_item_id.clone(),
                            resource_type: "playlist_item".to_string(),
                            error_message: e.to_string(),
                        });
                    }
                }

                progress.processed_items += 1;
                progress_callback(progress.clone());
                tokio::time::sleep(std::time::Duration::from_millis(Self::BATCH_DELAY_MS)).await;
            }
        }

        // Phase 4: Unsubscribe from artist channels
        if options.unsubscribe_from_artists {
            progress.phase = "unsubscribing".to_string();
            progress_callback(progress.clone());

            for subscription in &scan.blocked_subscriptions {
                progress.current_item = Some(subscription.channel_title.clone());
                progress_callback(progress.clone());

                // First, we need to find the subscription ID for this channel
                match self
                    .find_and_delete_subscription(access_token, &subscription.channel_id)
                    .await
                {
                    Ok(_) => {
                        progress.subscriptions_removed += 1;
                        self.record_enforcement_action(
                            run_id,
                            user_id,
                            YouTubeMusicResourceType::Subscription,
                            &subscription.channel_id,
                            None,
                            Some(&subscription.channel_title),
                            YouTubeMusicEnforcementActionType::Unsubscribe,
                            None,
                        )
                        .await?;
                    }
                    Err(e) => {
                        progress.errors += 1;
                        errors.push(YouTubeMusicEnforcementError {
                            resource_id: subscription.channel_id.clone(),
                            resource_type: "subscription".to_string(),
                            error_message: e.to_string(),
                        });
                    }
                }

                progress.processed_items += 1;
                progress_callback(progress.clone());
                tokio::time::sleep(std::time::Duration::from_millis(Self::BATCH_DELAY_MS)).await;
            }
        }

        // Update run record
        let status = if errors.is_empty() {
            YouTubeMusicEnforcementRunStatus::Completed
        } else if progress.videos_removed + progress.playlist_items_removed + progress.subscriptions_removed > 0 {
            YouTubeMusicEnforcementRunStatus::Completed // Partial success
        } else {
            YouTubeMusicEnforcementRunStatus::Failed
        };

        self.update_enforcement_run(
            run_id,
            &status,
            progress.videos_removed as u32,
            progress.playlist_items_removed as u32,
            progress.subscriptions_removed as u32,
            progress.errors as u32,
        )
        .await?;

        progress.phase = "completed".to_string();
        progress_callback(progress.clone());

        Ok(YouTubeMusicEnforcementResult {
            run_id,
            status,
            videos_removed: progress.videos_removed,
            playlist_items_removed: progress.playlist_items_removed,
            subscriptions_removed: progress.subscriptions_removed,
            errors,
            duration_seconds: start_time.elapsed().as_secs(),
            started_at: run.started_at,
            completed_at: Some(Utc::now()),
        })
    }

    /// Remove a video from liked videos
    async fn remove_like(&self, access_token: &str, video_id: &str) -> Result<()> {
        // Use videos.rate endpoint with "none" to remove like
        let url = format!(
            "{}/videos/rate?id={}&rating=none",
            Self::YOUTUBE_API_BASE,
            video_id
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Length", "0")
            .send()
            .await
            .map_err(|e| {
                AppError::ExternalServiceError(format!("YouTube API request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalServiceError(format!(
                "Failed to remove like ({}): {}",
                status, error_text
            )));
        }

        Ok(())
    }

    /// Rate a video (like, dislike, or none)
    async fn rate_video(
        &self,
        access_token: &str,
        video_id: &str,
        rating: YouTubeRating,
    ) -> Result<()> {
        let url = format!(
            "{}/videos/rate?id={}&rating={}",
            Self::YOUTUBE_API_BASE,
            video_id,
            rating
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Length", "0")
            .send()
            .await
            .map_err(|e| {
                AppError::ExternalServiceError(format!("YouTube API request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalServiceError(format!(
                "Failed to rate video ({}): {}",
                status, error_text
            )));
        }

        Ok(())
    }

    /// Remove an item from a playlist
    async fn remove_playlist_item(&self, access_token: &str, playlist_item_id: &str) -> Result<()> {
        let url = format!(
            "{}/playlistItems?id={}",
            Self::YOUTUBE_API_BASE,
            playlist_item_id
        );

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| {
                AppError::ExternalServiceError(format!("YouTube API request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalServiceError(format!(
                "Failed to remove playlist item ({}): {}",
                status, error_text
            )));
        }

        Ok(())
    }

    /// Find and delete a subscription by channel ID
    async fn find_and_delete_subscription(
        &self,
        access_token: &str,
        channel_id: &str,
    ) -> Result<()> {
        // First, find the subscription ID
        let url = format!(
            "{}/subscriptions?part=id&mine=true&forChannelId={}",
            Self::YOUTUBE_API_BASE,
            channel_id
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                AppError::ExternalServiceError(format!("YouTube API request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalServiceError(format!(
                "Failed to find subscription ({}): {}",
                status, error_text
            )));
        }

        let data: serde_json::Value = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!("Failed to parse response: {}", e))
        })?;

        let subscription_id = data["items"]
            .as_array()
            .and_then(|items| items.first())
            .and_then(|item| item["id"].as_str())
            .ok_or_else(|| {
                AppError::ExternalServiceError(format!(
                    "No subscription found for channel {}",
                    channel_id
                ))
            })?;

        // Now delete the subscription
        self.delete_subscription(access_token, subscription_id).await
    }

    /// Delete a subscription by its ID
    async fn delete_subscription(&self, access_token: &str, subscription_id: &str) -> Result<()> {
        let url = format!(
            "{}/subscriptions?id={}",
            Self::YOUTUBE_API_BASE,
            subscription_id
        );

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| {
                AppError::ExternalServiceError(format!("YouTube API request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalServiceError(format!(
                "Failed to delete subscription ({}): {}",
                status, error_text
            )));
        }

        Ok(())
    }

    /// Rollback an enforcement run
    pub async fn rollback_enforcement(
        &self,
        user_id: Uuid,
        run_id: Uuid,
        access_token: &str,
    ) -> Result<YouTubeMusicRollbackResult> {
        let start_time = Instant::now();
        let mut actions_restored = 0;
        let mut errors = Vec::new();

        // Get all actions from this run
        let actions = self.get_enforcement_actions(run_id).await?;

        for action in actions {
            let result = match action.action {
                YouTubeMusicEnforcementActionType::RemoveFromLikes => {
                    // Re-like the video
                    self.rate_video(access_token, &action.resource_id, YouTubeRating::Like)
                        .await
                }
                YouTubeMusicEnforcementActionType::Dislike => {
                    // Remove dislike
                    self.rate_video(access_token, &action.resource_id, YouTubeRating::None)
                        .await
                }
                YouTubeMusicEnforcementActionType::RemoveFromPlaylist => {
                    // Cannot easily restore playlist items without the video ID
                    // Would need to store more state
                    Err(AppError::ExternalServiceError(
                        "Playlist item restoration not supported".to_string(),
                    ))
                }
                YouTubeMusicEnforcementActionType::Unsubscribe => {
                    // Re-subscribe
                    self.subscribe_to_channel(access_token, &action.resource_id)
                        .await
                }
                _ => Ok(()), // Skip rollback actions
            };

            match result {
                Ok(_) => actions_restored += 1,
                Err(e) => {
                    errors.push(YouTubeMusicEnforcementError {
                        resource_id: action.resource_id.clone(),
                        resource_type: action.resource_type.to_string(),
                        error_message: e.to_string(),
                    });
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(Self::BATCH_DELAY_MS)).await;
        }

        // Update run status
        self.update_enforcement_run_status(run_id, &YouTubeMusicEnforcementRunStatus::RolledBack)
            .await?;

        Ok(YouTubeMusicRollbackResult {
            run_id,
            actions_restored,
            errors,
            duration_seconds: start_time.elapsed().as_secs(),
        })
    }

    /// Subscribe to a channel
    async fn subscribe_to_channel(&self, access_token: &str, channel_id: &str) -> Result<()> {
        let url = format!("{}/subscriptions?part=snippet", Self::YOUTUBE_API_BASE);

        let body = serde_json::json!({
            "snippet": {
                "resourceId": {
                    "kind": "youtube#channel",
                    "channelId": channel_id
                }
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                AppError::ExternalServiceError(format!("YouTube API request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalServiceError(format!(
                "Failed to subscribe to channel ({}): {}",
                status, error_text
            )));
        }

        Ok(())
    }

    /// Create an enforcement run record in the database
    async fn create_enforcement_run(
        &self,
        run_id: Uuid,
        user_id: Uuid,
        connection_id: Uuid,
        options: &YouTubeMusicEnforcementOptions,
    ) -> Result<YouTubeMusicEnforcementRun> {
        let started_at = Utc::now();
        let options_json = serde_json::to_value(options).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO youtube_music_enforcement_runs (
                id, user_id, connection_id, status, options, started_at,
                videos_scanned, playlists_scanned, subscriptions_scanned,
                videos_removed, playlist_items_removed, subscriptions_removed, errors
            ) VALUES ($1, $2, $3, $4, $5, $6, 0, 0, 0, 0, 0, 0, 0)
            "#,
        )
        .bind(run_id)
        .bind(user_id)
        .bind(connection_id)
        .bind("running")
        .bind(&options_json)
        .bind(started_at)
        .execute(&self.db_pool)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to create enforcement run: {}", e)),
        })?;

        Ok(YouTubeMusicEnforcementRun {
            id: run_id,
            user_id,
            connection_id,
            status: YouTubeMusicEnforcementRunStatus::Running,
            options: options.clone(),
            started_at,
            completed_at: None,
            videos_scanned: 0,
            playlists_scanned: 0,
            subscriptions_scanned: 0,
            videos_removed: 0,
            playlist_items_removed: 0,
            subscriptions_removed: 0,
            errors: 0,
            error_details: None,
        })
    }

    /// Update an enforcement run record
    async fn update_enforcement_run(
        &self,
        run_id: Uuid,
        status: &YouTubeMusicEnforcementRunStatus,
        videos_removed: u32,
        playlist_items_removed: u32,
        subscriptions_removed: u32,
        errors: u32,
    ) -> Result<()> {
        let completed_at = Utc::now();

        sqlx::query(
            r#"
            UPDATE youtube_music_enforcement_runs
            SET status = $2, completed_at = $3, videos_removed = $4,
                playlist_items_removed = $5, subscriptions_removed = $6, errors = $7
            WHERE id = $1
            "#,
        )
        .bind(run_id)
        .bind(status.to_string())
        .bind(completed_at)
        .bind(videos_removed as i32)
        .bind(playlist_items_removed as i32)
        .bind(subscriptions_removed as i32)
        .bind(errors as i32)
        .execute(&self.db_pool)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to update enforcement run: {}", e)),
        })?;

        Ok(())
    }

    /// Update enforcement run status only
    async fn update_enforcement_run_status(
        &self,
        run_id: Uuid,
        status: &YouTubeMusicEnforcementRunStatus,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE youtube_music_enforcement_runs
            SET status = $2, completed_at = $3
            WHERE id = $1
            "#,
        )
        .bind(run_id)
        .bind(status.to_string())
        .bind(Utc::now())
        .execute(&self.db_pool)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to update enforcement run status: {}", e)),
        })?;

        Ok(())
    }

    /// Record an enforcement action for rollback support
    async fn record_enforcement_action(
        &self,
        run_id: Uuid,
        user_id: Uuid,
        resource_type: YouTubeMusicResourceType,
        resource_id: &str,
        resource_name: Option<&str>,
        artist_name: Option<&str>,
        action: YouTubeMusicEnforcementActionType,
        playlist_id: Option<&str>,
    ) -> Result<()> {
        let action_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO youtube_music_enforcement_actions (
                id, run_id, user_id, resource_type, resource_id,
                resource_name, artist_name, action, playlist_id, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(action_id)
        .bind(run_id)
        .bind(user_id)
        .bind(resource_type.to_string())
        .bind(resource_id)
        .bind(resource_name)
        .bind(artist_name)
        .bind(action.to_string())
        .bind(playlist_id)
        .bind(Utc::now())
        .execute(&self.db_pool)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to record enforcement action: {}", e)),
        })?;

        Ok(())
    }

    /// Get enforcement actions for a run
    async fn get_enforcement_actions(
        &self,
        run_id: Uuid,
    ) -> Result<Vec<YouTubeMusicEnforcementAction>> {
        let rows = sqlx::query_as::<_, EnforcementActionRow>(
            r#"
            SELECT id, run_id, user_id, resource_type, resource_id,
                   resource_name, artist_name, action, playlist_id, created_at
            FROM youtube_music_enforcement_actions
            WHERE run_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(run_id)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to get enforcement actions: {}", e)),
        })?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Get enforcement history for a user
    pub async fn get_enforcement_history(
        &self,
        user_id: Uuid,
        limit: i32,
    ) -> Result<Vec<YouTubeMusicEnforcementHistoryItem>> {
        let rows = sqlx::query_as::<_, EnforcementRunRow>(
            r#"
            SELECT id, user_id, connection_id, status, options, started_at, completed_at,
                   videos_scanned, playlists_scanned, subscriptions_scanned,
                   videos_removed, playlist_items_removed, subscriptions_removed, errors
            FROM youtube_music_enforcement_runs
            WHERE user_id = $1
            ORDER BY started_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to get enforcement history: {}", e)),
        })?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}

/// Database row for enforcement action
#[derive(sqlx::FromRow)]
struct EnforcementActionRow {
    id: Uuid,
    run_id: Uuid,
    user_id: Uuid,
    resource_type: String,
    resource_id: String,
    resource_name: Option<String>,
    artist_name: Option<String>,
    action: String,
    playlist_id: Option<String>,
    created_at: DateTime<Utc>,
}

impl From<EnforcementActionRow> for YouTubeMusicEnforcementAction {
    fn from(row: EnforcementActionRow) -> Self {
        Self {
            id: row.id,
            run_id: row.run_id,
            user_id: row.user_id,
            resource_type: match row.resource_type.as_str() {
                "video" => YouTubeMusicResourceType::Video,
                "liked_video" => YouTubeMusicResourceType::LikedVideo,
                "playlist_item" => YouTubeMusicResourceType::PlaylistItem,
                "playlist" => YouTubeMusicResourceType::Playlist,
                "subscription" => YouTubeMusicResourceType::Subscription,
                "channel" => YouTubeMusicResourceType::Channel,
                _ => YouTubeMusicResourceType::Video,
            },
            resource_id: row.resource_id,
            resource_name: row.resource_name,
            artist_name: row.artist_name,
            action: match row.action.as_str() {
                "remove_from_likes" => YouTubeMusicEnforcementActionType::RemoveFromLikes,
                "dislike" => YouTubeMusicEnforcementActionType::Dislike,
                "remove_from_playlist" => YouTubeMusicEnforcementActionType::RemoveFromPlaylist,
                "unsubscribe" => YouTubeMusicEnforcementActionType::Unsubscribe,
                "add_to_likes" => YouTubeMusicEnforcementActionType::AddToLikes,
                "remove_dislike" => YouTubeMusicEnforcementActionType::RemoveDislike,
                "add_to_playlist" => YouTubeMusicEnforcementActionType::AddToPlaylist,
                "subscribe" => YouTubeMusicEnforcementActionType::Subscribe,
                _ => YouTubeMusicEnforcementActionType::RemoveFromLikes,
            },
            playlist_id: row.playlist_id,
            previous_state: None,
            created_at: row.created_at,
        }
    }
}

/// Database row for enforcement run
#[derive(sqlx::FromRow)]
struct EnforcementRunRow {
    id: Uuid,
    user_id: Uuid,
    connection_id: Uuid,
    status: String,
    options: serde_json::Value,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    videos_scanned: i32,
    playlists_scanned: i32,
    subscriptions_scanned: i32,
    videos_removed: i32,
    playlist_items_removed: i32,
    subscriptions_removed: i32,
    errors: i32,
}

/// Enforcement history item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMusicEnforcementHistoryItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub connection_id: Uuid,
    pub status: String,
    pub options: YouTubeMusicEnforcementOptions,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub videos_scanned: i32,
    pub playlists_scanned: i32,
    pub subscriptions_scanned: i32,
    pub videos_removed: i32,
    pub playlist_items_removed: i32,
    pub subscriptions_removed: i32,
    pub errors: i32,
}

impl From<EnforcementRunRow> for YouTubeMusicEnforcementHistoryItem {
    fn from(row: EnforcementRunRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            connection_id: row.connection_id,
            status: row.status,
            options: serde_json::from_value(row.options).unwrap_or_default(),
            started_at: row.started_at,
            completed_at: row.completed_at,
            videos_scanned: row.videos_scanned,
            playlists_scanned: row.playlists_scanned,
            subscriptions_scanned: row.subscriptions_scanned,
            videos_removed: row.videos_removed,
            playlist_items_removed: row.playlist_items_removed,
            subscriptions_removed: row.subscriptions_removed,
            errors: row.errors,
        }
    }
}

/// Shared YouTube Music enforcement service
pub type SharedYouTubeMusicEnforcementService = Arc<YouTubeMusicEnforcementService>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enforcement_options_default() {
        let options = YouTubeMusicEnforcementOptions::default();
        assert!(options.remove_from_likes);
        assert!(!options.dislike_blocked_content);
        assert!(options.remove_from_playlists);
        assert!(options.unsubscribe_from_artists);
        assert_eq!(options.batch_size, 50);
        assert!(!options.dry_run);
    }

    #[test]
    fn test_youtube_rating_display() {
        assert_eq!(YouTubeRating::Like.to_string(), "like");
        assert_eq!(YouTubeRating::Dislike.to_string(), "dislike");
        assert_eq!(YouTubeRating::None.to_string(), "none");
    }
}
