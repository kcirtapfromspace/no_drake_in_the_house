//! YouTube Music Library Scanning Service
//!
//! Implements library scanning for YouTube Music via the YouTube Data API v3.
//! Scans liked videos, playlists, and subscriptions for blocked artist content.

use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::youtube_music::{
    BlockedPlaylistItemInfo, BlockedSubscriptionInfo, BlockedVideoInfo, YouTubeListResponse,
    YouTubeMusicBlockedContentScan, YouTubeMusicChannel, YouTubeMusicLibrary,
    YouTubeMusicPlaylist, YouTubeMusicPlaylistItem, YouTubeMusicVideo,
};

/// YouTube Music library service for scanning user libraries
pub struct YouTubeMusicLibraryService {
    db_pool: PgPool,
    client: reqwest::Client,
}

impl YouTubeMusicLibraryService {
    const YOUTUBE_API_BASE: &'static str = "https://www.googleapis.com/youtube/v3";
    const MAX_RESULTS_PER_PAGE: u32 = 50;

    /// Create a new YouTube Music library service
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Scan a user's YouTube Music library
    pub async fn scan_library(
        &self,
        user_id: Uuid,
        access_token: &str,
    ) -> Result<YouTubeMusicLibrary> {
        // Get user's channel ID first
        let channel_id = self.get_user_channel_id(access_token).await?;

        // Scan library components concurrently
        let (liked_videos, playlists, subscriptions) = tokio::try_join!(
            self.scan_liked_videos(access_token),
            self.scan_playlists(access_token),
            self.scan_subscriptions(access_token),
        )?;

        let library = YouTubeMusicLibrary {
            user_id,
            youtube_channel_id: channel_id,
            liked_videos,
            playlists,
            subscriptions,
            scanned_at: chrono::Utc::now(),
        };

        tracing::info!(
            user_id = %user_id,
            liked_videos = library.liked_videos.len(),
            playlists = library.playlists.len(),
            subscriptions = library.subscriptions.len(),
            "YouTube Music library scan complete"
        );

        Ok(library)
    }

    /// Get the authenticated user's channel ID
    async fn get_user_channel_id(&self, access_token: &str) -> Result<String> {
        let url = format!(
            "{}/channels?part=id&mine=true",
            Self::YOUTUBE_API_BASE
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| AppError::ExternalServiceError(format!(
                "YouTube API request failed: {}", e
            )))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalServiceError(format!(
                "YouTube API error ({}): {}", status, error_text
            )));
        }

        let data: serde_json::Value = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!("Failed to parse YouTube response: {}", e))
        })?;

        data["items"]
            .as_array()
            .and_then(|items| items.first())
            .and_then(|item| item["id"].as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::ExternalServiceError(
                "No YouTube channel found for user".to_string()
            ))
    }

    /// Scan liked videos from the user's "Liked Music" playlist
    async fn scan_liked_videos(&self, access_token: &str) -> Result<Vec<YouTubeMusicVideo>> {
        let mut liked_videos = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let mut url = format!(
                "{}/videos?part=snippet,contentDetails,status&myRating=like&maxResults={}",
                Self::YOUTUBE_API_BASE,
                Self::MAX_RESULTS_PER_PAGE
            );

            if let Some(token) = &page_token {
                url.push_str(&format!("&pageToken={}", token));
            }

            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", access_token))
                .header("Accept", "application/json")
                .send()
                .await
                .map_err(|e| AppError::ExternalServiceError(format!(
                    "YouTube API request failed: {}", e
                )))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();

                // Handle rate limiting with retry-after
                if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    tracing::warn!("YouTube API rate limit hit during liked videos scan");
                    break;
                }

                return Err(AppError::ExternalServiceError(format!(
                    "YouTube API error ({}): {}", status, error_text
                )));
            }

            let list_response: YouTubeListResponse<YouTubeMusicVideo> =
                response.json().await.map_err(|e| {
                    AppError::ExternalServiceError(format!(
                        "Failed to parse YouTube videos response: {}", e
                    ))
                })?;

            liked_videos.extend(list_response.items);

            match list_response.next_page_token {
                Some(token) => page_token = Some(token),
                None => break,
            }

            // Rate limiting: small delay between pages
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        Ok(liked_videos)
    }

    /// Scan user's playlists
    async fn scan_playlists(&self, access_token: &str) -> Result<Vec<YouTubeMusicPlaylist>> {
        let mut playlists = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let mut url = format!(
                "{}/playlists?part=snippet,contentDetails,status&mine=true&maxResults={}",
                Self::YOUTUBE_API_BASE,
                Self::MAX_RESULTS_PER_PAGE
            );

            if let Some(token) = &page_token {
                url.push_str(&format!("&pageToken={}", token));
            }

            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", access_token))
                .header("Accept", "application/json")
                .send()
                .await
                .map_err(|e| AppError::ExternalServiceError(format!(
                    "YouTube API request failed: {}", e
                )))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();

                if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    tracing::warn!("YouTube API rate limit hit during playlists scan");
                    break;
                }

                return Err(AppError::ExternalServiceError(format!(
                    "YouTube API error ({}): {}", status, error_text
                )));
            }

            let list_response: YouTubeListResponse<YouTubeMusicPlaylist> =
                response.json().await.map_err(|e| {
                    AppError::ExternalServiceError(format!(
                        "Failed to parse YouTube playlists response: {}", e
                    ))
                })?;

            playlists.extend(list_response.items);

            match list_response.next_page_token {
                Some(token) => page_token = Some(token),
                None => break,
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        Ok(playlists)
    }

    /// Scan user's subscriptions (artist channels they follow)
    async fn scan_subscriptions(&self, access_token: &str) -> Result<Vec<YouTubeMusicChannel>> {
        let mut subscriptions = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let mut url = format!(
                "{}/subscriptions?part=snippet&mine=true&maxResults={}",
                Self::YOUTUBE_API_BASE,
                Self::MAX_RESULTS_PER_PAGE
            );

            if let Some(token) = &page_token {
                url.push_str(&format!("&pageToken={}", token));
            }

            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", access_token))
                .header("Accept", "application/json")
                .send()
                .await
                .map_err(|e| AppError::ExternalServiceError(format!(
                    "YouTube API request failed: {}", e
                )))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();

                if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    tracing::warn!("YouTube API rate limit hit during subscriptions scan");
                    break;
                }

                return Err(AppError::ExternalServiceError(format!(
                    "YouTube API error ({}): {}", status, error_text
                )));
            }

            // Subscriptions have a different structure - need to convert
            let data: serde_json::Value = response.json().await.map_err(|e| {
                AppError::ExternalServiceError(format!(
                    "Failed to parse YouTube subscriptions response: {}", e
                ))
            })?;

            let items = data["items"].as_array().cloned().unwrap_or_default();

            for item in items {
                // Convert subscription to channel info
                if let Some(snippet) = item["snippet"].as_object() {
                    let channel = YouTubeMusicChannel {
                        id: snippet["resourceId"]["channelId"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        snippet: crate::models::youtube_music::YouTubeMusicChannelSnippet {
                            title: snippet["title"].as_str().unwrap_or_default().to_string(),
                            description: snippet["description"].as_str().map(|s| s.to_string()),
                            custom_url: None,
                            thumbnails: None,
                            published_at: snippet["publishedAt"].as_str().map(|s| s.to_string()),
                            country: None,
                        },
                        statistics: None,
                    };
                    subscriptions.push(channel);
                }
            }

            match data["nextPageToken"].as_str() {
                Some(token) => page_token = Some(token.to_string()),
                None => break,
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        Ok(subscriptions)
    }

    /// Scan playlist items for a specific playlist
    pub async fn scan_playlist_items(
        &self,
        access_token: &str,
        playlist_id: &str,
    ) -> Result<Vec<YouTubeMusicPlaylistItem>> {
        let mut items = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let mut url = format!(
                "{}/playlistItems?part=snippet,contentDetails,status&playlistId={}&maxResults={}",
                Self::YOUTUBE_API_BASE,
                playlist_id,
                Self::MAX_RESULTS_PER_PAGE
            );

            if let Some(token) = &page_token {
                url.push_str(&format!("&pageToken={}", token));
            }

            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", access_token))
                .header("Accept", "application/json")
                .send()
                .await
                .map_err(|e| AppError::ExternalServiceError(format!(
                    "YouTube API request failed: {}", e
                )))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();

                if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    tracing::warn!("YouTube API rate limit hit during playlist items scan");
                    break;
                }

                return Err(AppError::ExternalServiceError(format!(
                    "YouTube API error ({}): {}", status, error_text
                )));
            }

            let list_response: YouTubeListResponse<YouTubeMusicPlaylistItem> =
                response.json().await.map_err(|e| {
                    AppError::ExternalServiceError(format!(
                        "Failed to parse YouTube playlist items response: {}", e
                    ))
                })?;

            items.extend(list_response.items);

            match list_response.next_page_token {
                Some(token) => page_token = Some(token),
                None => break,
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        Ok(items)
    }

    /// Scan library for blocked content based on blocked artist names
    pub async fn scan_for_blocked_content(
        &self,
        access_token: &str,
        blocked_artists: &[String],
    ) -> Result<YouTubeMusicBlockedContentScan> {
        let library = self.scan_library(Uuid::nil(), access_token).await?;

        let mut scan = YouTubeMusicBlockedContentScan::new();
        scan.total_videos_scanned = library.liked_videos.len();
        scan.total_subscriptions_scanned = library.subscriptions.len();

        // Normalize blocked artist names for matching
        let blocked_normalized: Vec<String> = blocked_artists
            .iter()
            .map(|n| n.to_lowercase())
            .collect();

        // Check liked videos
        for video in &library.liked_videos {
            let channel_title = video.snippet.channel_title.to_lowercase();
            let video_title = video.snippet.title.to_lowercase();

            // Check if channel title or video title contains blocked artist name
            let is_blocked = blocked_normalized.iter().any(|blocked| {
                channel_title.contains(blocked) || video_title.contains(blocked)
            });

            if is_blocked {
                scan.blocked_videos.push(BlockedVideoInfo {
                    video_id: video.id.clone(),
                    title: video.snippet.title.clone(),
                    channel_id: video.snippet.channel_id.clone(),
                    channel_title: video.snippet.channel_title.clone(),
                    blocked_artist_id: None, // Would need DB lookup for exact match
                });
            }
        }

        // Check subscriptions (artist channels)
        for channel in &library.subscriptions {
            let channel_title = channel.snippet.title.to_lowercase();

            let is_blocked = blocked_normalized.iter().any(|blocked| {
                channel_title.contains(blocked)
            });

            if is_blocked {
                scan.blocked_subscriptions.push(BlockedSubscriptionInfo {
                    subscription_id: channel.id.clone(), // Note: This is channel ID, not subscription resource ID
                    channel_id: channel.id.clone(),
                    channel_title: channel.snippet.title.clone(),
                    blocked_artist_id: None,
                });
            }
        }

        // Check playlist items
        for playlist in &library.playlists {
            let items = self.scan_playlist_items(access_token, &playlist.id).await?;
            scan.total_playlist_items_scanned += items.len();

            for item in items {
                let channel_title = item.snippet.channel_title.to_lowercase();
                let video_title = item.snippet.title.to_lowercase();

                let is_blocked = blocked_normalized.iter().any(|blocked| {
                    channel_title.contains(blocked) || video_title.contains(blocked)
                });

                if is_blocked {
                    let video_id = item.snippet.resource_id.video_id.clone().unwrap_or_default();
                    scan.blocked_playlist_items.push(BlockedPlaylistItemInfo {
                        playlist_item_id: item.id.clone(),
                        playlist_id: playlist.id.clone(),
                        video_id,
                        title: item.snippet.title.clone(),
                        channel_id: item.snippet.channel_id.clone(),
                        channel_title: item.snippet.channel_title.clone(),
                        blocked_artist_id: None,
                    });
                }
            }
        }

        tracing::info!(
            blocked_videos = scan.blocked_videos.len(),
            blocked_playlist_items = scan.blocked_playlist_items.len(),
            blocked_subscriptions = scan.blocked_subscriptions.len(),
            "YouTube Music blocked content scan complete"
        );

        Ok(scan)
    }

    /// Get blocked artist names from the database for a user
    pub async fn get_blocked_artist_names(&self, user_id: Uuid) -> Result<Vec<String>> {
        let rows: Vec<(Option<String>,)> = sqlx::query_as(
            r#"
            SELECT DISTINCT a.canonical_name as name
            FROM user_artist_blocks uab
            JOIN artists a ON uab.artist_id = a.id
            WHERE uab.user_id = $1

            UNION

            SELECT DISTINCT a.canonical_name as name
            FROM category_subscriptions cs
            JOIN artist_offenses ao ON ao.category = cs.category
            JOIN artists a ON ao.artist_id = a.id
            WHERE cs.user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(e.to_string()),
        })?;

        Ok(rows.into_iter().filter_map(|r| r.0).collect())
    }
}

/// Shared YouTube Music library service instance
pub type SharedYouTubeMusicLibraryService = Arc<YouTubeMusicLibraryService>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_service_creation() {
        // This test would require a mock database pool
        // For now, just test the constant values
        assert_eq!(YouTubeMusicLibraryService::MAX_RESULTS_PER_PAGE, 50);
        assert!(YouTubeMusicLibraryService::YOUTUBE_API_BASE.contains("googleapis.com"));
    }

    #[test]
    fn test_blocked_content_scan_new() {
        let scan = YouTubeMusicBlockedContentScan::new();
        assert!(scan.blocked_videos.is_empty());
        assert!(scan.blocked_playlist_items.is_empty());
        assert!(scan.blocked_subscriptions.is_empty());
        assert_eq!(scan.total_blocked(), 0);
    }
}
