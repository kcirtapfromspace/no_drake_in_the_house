//! YouTube Music Catalog Sync Worker
//!
//! Implements catalog synchronization for YouTube Music using the YouTube Data API.
//! Rate limit: 10,000 requests/day (quota-based)

use super::traits::*;
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

/// YouTube Data API base URL
const YOUTUBE_API_BASE: &str = "https://www.googleapis.com/youtube/v3";

/// YouTube Music sync worker
///
/// Note: YouTube Music doesn't have a dedicated API. We use the YouTube Data API
/// to search for music channels and videos.
pub struct YouTubeMusicSyncWorker {
    client: Client,
    /// API key for YouTube Data API
    api_key: String,
    /// Daily quota tracking
    daily_quota: Arc<RwLock<DailyQuotaState>>,
    /// Rate limiter state
    rate_limiter: Arc<RwLock<RateLimiterState>>,
}

struct RateLimiterState {
    requests_this_window: u32,
    window_start: std::time::Instant,
}

struct DailyQuotaState {
    quota_used: u32,
    day_start: chrono::NaiveDate,
}

impl YouTubeMusicSyncWorker {
    /// Create a new YouTube Music sync worker
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            daily_quota: Arc::new(RwLock::new(DailyQuotaState {
                quota_used: 0,
                day_start: Utc::now().date_naive(),
            })),
            rate_limiter: Arc::new(RwLock::new(RateLimiterState {
                requests_this_window: 0,
                window_start: std::time::Instant::now(),
            })),
        }
    }

    /// Check and update quota
    async fn check_quota(&self, cost: u32) -> Result<()> {
        let config = self.rate_limit_config();
        let mut state = self.daily_quota.write().await;

        // Reset if new day
        let today = Utc::now().date_naive();
        if state.day_start != today {
            state.quota_used = 0;
            state.day_start = today;
        }

        // Check if we'd exceed daily quota
        if let Some(daily_limit) = config.daily_quota {
            if state.quota_used + cost > daily_limit {
                return Err(anyhow::anyhow!(
                    "YouTube API daily quota exceeded ({}/{})",
                    state.quota_used,
                    daily_limit
                ));
            }
        }

        state.quota_used += cost;
        Ok(())
    }

    /// Wait for rate limit if needed
    async fn wait_for_rate_limit(&self) {
        let config = self.rate_limit_config();
        let mut state = self.rate_limiter.write().await;

        let elapsed = state.window_start.elapsed();
        if elapsed.as_secs() >= config.window_seconds as u64 {
            state.requests_this_window = 0;
            state.window_start = std::time::Instant::now();
        }

        if state.requests_this_window >= config.requests_per_window {
            let wait_time = Duration::from_secs(config.window_seconds as u64) - elapsed;
            tracing::debug!("YouTube rate limit hit, waiting {:?}", wait_time);
            drop(state);
            sleep(wait_time).await;

            let mut state = self.rate_limiter.write().await;
            state.requests_this_window = 0;
            state.window_start = std::time::Instant::now();
        } else {
            state.requests_this_window += 1;
        }
    }

    /// Make an API request
    async fn api_request<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        quota_cost: u32,
    ) -> Result<T> {
        self.check_quota(quota_cost).await?;
        self.wait_for_rate_limit().await;

        let url = format!(
            "{}{}{}key={}",
            YOUTUBE_API_BASE,
            endpoint,
            if endpoint.contains('?') { "&" } else { "?" },
            self.api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("YouTube API request failed")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("YouTube API error: {} - {}", status, body));
        }

        response
            .json()
            .await
            .context("Failed to parse YouTube response")
    }
}

// YouTube API response types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubeSearchResponse {
    items: Vec<YouTubeSearchItem>,
    #[allow(dead_code)]
    next_page_token: Option<String>,
    page_info: YouTubePageInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubePageInfo {
    #[allow(dead_code)]
    total_results: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubeSearchItem {
    id: YouTubeItemId,
    snippet: YouTubeSnippet,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubeItemId {
    kind: String,
    channel_id: Option<String>,
    video_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubeSnippet {
    title: String,
    description: Option<String>,
    channel_id: String,
    channel_title: String,
    thumbnails: YouTubeThumbnails,
}

#[derive(Debug, Deserialize)]
struct YouTubeThumbnails {
    high: Option<YouTubeThumbnail>,
    medium: Option<YouTubeThumbnail>,
    default: Option<YouTubeThumbnail>,
}

#[derive(Debug, Deserialize)]
struct YouTubeThumbnail {
    url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubeChannelResponse {
    items: Vec<YouTubeChannel>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubeChannel {
    id: String,
    snippet: YouTubeChannelSnippet,
    statistics: Option<YouTubeChannelStatistics>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubeChannelSnippet {
    title: String,
    description: Option<String>,
    thumbnails: YouTubeThumbnails,
    custom_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubeChannelStatistics {
    subscriber_count: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubeVideoResponse {
    items: Vec<YouTubeVideo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubeVideo {
    id: String,
    snippet: YouTubeVideoSnippet,
    content_details: Option<YouTubeContentDetails>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubeVideoSnippet {
    title: String,
    channel_id: String,
    published_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct YouTubeContentDetails {
    duration: Option<String>,
}

impl YouTubeChannel {
    fn into_platform_artist(self) -> PlatformArtist {
        let mut external_urls = HashMap::new();
        if let Some(custom_url) = &self.snippet.custom_url {
            external_urls.insert(
                "youtube".to_string(),
                format!("https://www.youtube.com/{}", custom_url),
            );
        }

        let image_url = self
            .snippet
            .thumbnails
            .high
            .or(self.snippet.thumbnails.medium)
            .or(self.snippet.thumbnails.default)
            .map(|t| t.url);

        let popularity = self
            .statistics
            .and_then(|s| s.subscriber_count)
            .and_then(|s| s.parse().ok());

        PlatformArtist {
            platform_id: self.id,
            platform: Platform::YouTubeMusic,
            name: self.snippet.title,
            genres: vec![], // YouTube doesn't expose genres
            popularity,
            image_url,
            external_urls,
            metadata: HashMap::new(),
        }
    }
}

impl YouTubeVideo {
    fn into_platform_track(self) -> PlatformTrack {
        // Parse ISO 8601 duration (e.g., "PT4M30S")
        let duration_ms = self
            .content_details
            .and_then(|c| c.duration)
            .and_then(|d| parse_iso8601_duration(&d));

        PlatformTrack {
            platform_id: self.id,
            platform: Platform::YouTubeMusic,
            title: self.snippet.title,
            isrc: None, // YouTube doesn't provide ISRC
            duration_ms,
            artist_ids: vec![self.snippet.channel_id],
            album_id: None,
            release_date: self.snippet.published_at,
            preview_url: None,
            explicit: false, // Would need to check content rating
        }
    }
}

/// Parse ISO 8601 duration to milliseconds
fn parse_iso8601_duration(duration: &str) -> Option<u64> {
    // Format: PT#H#M#S or PT#M#S
    let duration = duration.strip_prefix("PT")?;

    let mut total_seconds = 0u64;
    let mut current_num = String::new();

    for c in duration.chars() {
        if c.is_ascii_digit() {
            current_num.push(c);
        } else {
            let num: u64 = current_num.parse().ok()?;
            current_num.clear();
            match c {
                'H' => total_seconds += num * 3600,
                'M' => total_seconds += num * 60,
                'S' => total_seconds += num,
                _ => {}
            }
        }
    }

    Some(total_seconds * 1000)
}

#[async_trait]
impl PlatformCatalogWorker for YouTubeMusicSyncWorker {
    fn platform(&self) -> Platform {
        Platform::YouTubeMusic
    }

    fn rate_limit_config(&self) -> RateLimitConfig {
        RateLimitConfig::youtube_music()
    }

    async fn health_check(&self) -> Result<bool> {
        // Make a simple search to verify API key works
        let result = self
            .api_request::<YouTubeSearchResponse>(
                "/search?part=snippet&type=channel&q=test&maxResults=1",
                100, // Search costs 100 quota units
            )
            .await;

        match result {
            Ok(_) => Ok(true),
            Err(e) => {
                tracing::warn!("YouTube health check failed: {}", e);
                Ok(false)
            }
        }
    }

    async fn search_artist(&self, query: &str, limit: u32) -> Result<Vec<PlatformArtist>> {
        let encoded_query = urlencoding::encode(query);
        let endpoint = format!(
            "/search?part=snippet&type=channel&q={}&maxResults={}&videoCategoryId=10",
            encoded_query,
            limit.min(50)
        );

        let response: YouTubeSearchResponse = self.api_request(&endpoint, 100).await?;

        // Get channel IDs to fetch full details
        let channel_ids: Vec<_> = response
            .items
            .iter()
            .filter_map(|item| item.id.channel_id.clone())
            .collect();

        if channel_ids.is_empty() {
            return Ok(vec![]);
        }

        // Fetch full channel details
        let ids = channel_ids.join(",");
        let details_endpoint = format!("/channels?part=snippet,statistics&id={}", ids);
        let channels: YouTubeChannelResponse = self.api_request(&details_endpoint, 1).await?;

        Ok(channels
            .items
            .into_iter()
            .map(|c| c.into_platform_artist())
            .collect())
    }

    async fn get_artist(&self, platform_id: &str) -> Result<Option<PlatformArtist>> {
        let endpoint = format!("/channels?part=snippet,statistics&id={}", platform_id);
        let response: YouTubeChannelResponse = self.api_request(&endpoint, 1).await?;
        Ok(response
            .items
            .into_iter()
            .next()
            .map(|c| c.into_platform_artist()))
    }

    async fn get_artist_top_tracks(
        &self,
        platform_id: &str,
        limit: u32,
    ) -> Result<Vec<PlatformTrack>> {
        // Search for videos by channel
        let endpoint = format!(
            "/search?part=snippet&channelId={}&type=video&order=viewCount&maxResults={}&videoCategoryId=10",
            platform_id,
            limit.min(50)
        );

        let response: YouTubeSearchResponse = self.api_request(&endpoint, 100).await?;

        let video_ids: Vec<_> = response
            .items
            .iter()
            .filter_map(|item| item.id.video_id.clone())
            .collect();

        if video_ids.is_empty() {
            return Ok(vec![]);
        }

        // Get video details for duration
        let ids = video_ids.join(",");
        let details_endpoint = format!("/videos?part=snippet,contentDetails&id={}", ids);
        let videos: YouTubeVideoResponse = self.api_request(&details_endpoint, 1).await?;

        Ok(videos
            .items
            .into_iter()
            .map(|v| v.into_platform_track())
            .collect())
    }

    async fn get_artist_albums(
        &self,
        _platform_id: &str,
        _limit: u32,
        _offset: u32,
    ) -> Result<Vec<PlatformAlbum>> {
        // YouTube doesn't have albums in the same way
        // Could potentially use playlists instead
        Ok(vec![])
    }

    async fn get_album_tracks(&self, _album_id: &str) -> Result<Vec<PlatformTrack>> {
        // Would need to fetch playlist items if treating playlists as albums
        Ok(vec![])
    }

    async fn get_related_artists(&self, platform_id: &str) -> Result<Vec<PlatformArtist>> {
        // YouTube doesn't have direct related channels for music
        // Could search for similar content
        let artist = self.get_artist(platform_id).await?;
        if let Some(artist) = artist {
            self.search_artist(&artist.name, 10).await
        } else {
            Ok(vec![])
        }
    }

    async fn sync_incremental(
        &self,
        checkpoint: Option<SyncCheckpoint>,
        progress_callback: Box<dyn Fn(SyncProgress) + Send + Sync>,
    ) -> Result<SyncResult> {
        let sync_run_id = checkpoint
            .as_ref()
            .map(|c| c.sync_run_id)
            .unwrap_or_else(Uuid::new_v4);
        let started_at = Utc::now();

        progress_callback(SyncProgress {
            platform: Platform::YouTubeMusic,
            sync_run_id,
            status: SyncStatus::Running,
            total_items: None,
            items_processed: checkpoint.as_ref().map(|c| c.items_processed).unwrap_or(0),
            errors: 0,
            started_at,
            updated_at: Utc::now(),
            estimated_completion: None,
        });

        let result = SyncResult {
            platform: Platform::YouTubeMusic,
            sync_run_id,
            status: SyncStatus::Completed,
            artists_processed: 0,
            tracks_processed: 0,
            albums_processed: 0,
            new_artists: 0,
            updated_artists: 0,
            errors: 0,
            duration_ms: (Utc::now() - started_at).num_milliseconds() as u64,
            api_calls: 0,
            rate_limit_delays_ms: 0,
        };

        progress_callback(SyncProgress {
            platform: Platform::YouTubeMusic,
            sync_run_id,
            status: SyncStatus::Completed,
            total_items: Some(result.artists_processed),
            items_processed: result.artists_processed,
            errors: result.errors,
            started_at,
            updated_at: Utc::now(),
            estimated_completion: None,
        });

        Ok(result)
    }

    async fn sync_full(
        &self,
        progress_callback: Box<dyn Fn(SyncProgress) + Send + Sync>,
    ) -> Result<SyncResult> {
        self.sync_incremental(None, progress_callback).await
    }

    async fn get_checkpoint(&self, _sync_run_id: Uuid) -> Result<Option<SyncCheckpoint>> {
        Ok(None)
    }

    async fn save_checkpoint(&self, _checkpoint: &SyncCheckpoint) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_iso8601_duration() {
        assert_eq!(parse_iso8601_duration("PT4M30S"), Some(270000));
        assert_eq!(parse_iso8601_duration("PT1H30M"), Some(5400000));
        assert_eq!(parse_iso8601_duration("PT30S"), Some(30000));
    }
}
