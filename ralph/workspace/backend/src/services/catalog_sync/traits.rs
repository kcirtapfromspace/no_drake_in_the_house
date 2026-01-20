//! Catalog Sync Traits
//!
//! Defines the interface for platform-specific sync workers

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Supported streaming platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Spotify,
    AppleMusic,
    Tidal,
    YouTubeMusic,
    Deezer,
}

impl Platform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::Spotify => "spotify",
            Platform::AppleMusic => "apple_music",
            Platform::Tidal => "tidal",
            Platform::YouTubeMusic => "youtube_music",
            Platform::Deezer => "deezer",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Platform::Spotify => "Spotify",
            Platform::AppleMusic => "Apple Music",
            Platform::Tidal => "Tidal",
            Platform::YouTubeMusic => "YouTube Music",
            Platform::Deezer => "Deezer",
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Sync run type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncType {
    /// Full catalog sync
    Full,
    /// Incremental sync since last checkpoint
    Incremental,
    /// Sync specific artists only
    Targeted,
}

/// Sync run status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Paused,
    Cancelled,
}

/// Rate limit configuration per platform
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Requests per time window
    pub requests_per_window: u32,
    /// Time window in seconds
    pub window_seconds: u32,
    /// Burst limit (if supported)
    pub burst_limit: Option<u32>,
    /// Daily quota (if applicable)
    pub daily_quota: Option<u32>,
}

impl RateLimitConfig {
    /// Spotify: 100 requests/minute
    pub fn spotify() -> Self {
        Self {
            requests_per_window: 100,
            window_seconds: 60,
            burst_limit: Some(30),
            daily_quota: None,
        }
    }

    /// Apple Music: 1000 requests/hour
    pub fn apple_music() -> Self {
        Self {
            requests_per_window: 1000,
            window_seconds: 3600,
            burst_limit: Some(50),
            daily_quota: None,
        }
    }

    /// Tidal: 500 requests/5 minutes
    pub fn tidal() -> Self {
        Self {
            requests_per_window: 500,
            window_seconds: 300,
            burst_limit: Some(20),
            daily_quota: None,
        }
    }

    /// YouTube Music: 10,000 requests/day
    pub fn youtube_music() -> Self {
        Self {
            requests_per_window: 100,
            window_seconds: 60,
            burst_limit: Some(10),
            daily_quota: Some(10000),
        }
    }

    /// Deezer: 50 requests/5 seconds
    pub fn deezer() -> Self {
        Self {
            requests_per_window: 50,
            window_seconds: 5,
            burst_limit: None,
            daily_quota: None,
        }
    }
}

/// Artist data from a platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformArtist {
    /// Platform-specific ID
    pub platform_id: String,
    /// Platform this artist is from
    pub platform: Platform,
    /// Artist name as shown on the platform
    pub name: String,
    /// Genres (if available)
    pub genres: Vec<String>,
    /// Follower/listener count (if available)
    pub popularity: Option<u64>,
    /// Profile image URL
    pub image_url: Option<String>,
    /// External URLs (artist page, etc.)
    pub external_urls: HashMap<String, String>,
    /// Additional platform-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Track data from a platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformTrack {
    /// Platform-specific ID
    pub platform_id: String,
    /// Platform this track is from
    pub platform: Platform,
    /// Track title
    pub title: String,
    /// ISRC (International Standard Recording Code)
    pub isrc: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Artist IDs on this platform
    pub artist_ids: Vec<String>,
    /// Album ID (if part of an album)
    pub album_id: Option<String>,
    /// Release date
    pub release_date: Option<String>,
    /// Preview URL
    pub preview_url: Option<String>,
    /// Explicit content flag
    pub explicit: bool,
}

/// Album data from a platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformAlbum {
    /// Platform-specific ID
    pub platform_id: String,
    /// Platform this album is from
    pub platform: Platform,
    /// Album title
    pub title: String,
    /// UPC (Universal Product Code)
    pub upc: Option<String>,
    /// Artist IDs on this platform
    pub artist_ids: Vec<String>,
    /// Release date
    pub release_date: Option<String>,
    /// Album type (album, single, compilation, etc.)
    pub album_type: String,
    /// Total tracks
    pub total_tracks: u32,
    /// Cover image URL
    pub image_url: Option<String>,
}

/// Sync checkpoint for resumable syncs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCheckpoint {
    /// Platform being synced
    pub platform: Platform,
    /// Sync run ID
    pub sync_run_id: Uuid,
    /// Last processed offset/cursor
    pub offset: String,
    /// Number of items processed so far
    pub items_processed: u64,
    /// Timestamp of last update
    pub updated_at: DateTime<Utc>,
    /// Additional checkpoint data
    pub data: HashMap<String, serde_json::Value>,
}

/// Sync progress report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProgress {
    /// Platform being synced
    pub platform: Platform,
    /// Sync run ID
    pub sync_run_id: Uuid,
    /// Current status
    pub status: SyncStatus,
    /// Total items to process (if known)
    pub total_items: Option<u64>,
    /// Items processed so far
    pub items_processed: u64,
    /// Errors encountered
    pub errors: u64,
    /// Start time
    pub started_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
    /// Estimated completion time
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Sync result summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// Platform synced
    pub platform: Platform,
    /// Sync run ID
    pub sync_run_id: Uuid,
    /// Final status
    pub status: SyncStatus,
    /// Artists processed
    pub artists_processed: u64,
    /// Tracks processed
    pub tracks_processed: u64,
    /// Albums processed
    pub albums_processed: u64,
    /// New artists discovered
    pub new_artists: u64,
    /// Updated artists
    pub updated_artists: u64,
    /// Errors encountered
    pub errors: u64,
    /// Total duration in milliseconds
    pub duration_ms: u64,
    /// API calls made
    pub api_calls: u64,
    /// Rate limit delays (total ms)
    pub rate_limit_delays_ms: u64,
}

/// Platform catalog worker trait
///
/// Implement this trait for each streaming platform to enable
/// artist catalog synchronization.
#[async_trait]
pub trait PlatformCatalogWorker: Send + Sync {
    /// Get the platform this worker handles
    fn platform(&self) -> Platform;

    /// Get rate limit configuration
    fn rate_limit_config(&self) -> RateLimitConfig;

    /// Check if the worker is properly configured and authenticated
    async fn health_check(&self) -> Result<bool>;

    /// Search for an artist by name
    async fn search_artist(&self, query: &str, limit: u32) -> Result<Vec<PlatformArtist>>;

    /// Get artist by platform ID
    async fn get_artist(&self, platform_id: &str) -> Result<Option<PlatformArtist>>;

    /// Get artist's top tracks
    async fn get_artist_top_tracks(
        &self,
        platform_id: &str,
        limit: u32,
    ) -> Result<Vec<PlatformTrack>>;

    /// Get artist's albums
    async fn get_artist_albums(
        &self,
        platform_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<PlatformAlbum>>;

    /// Get album tracks
    async fn get_album_tracks(&self, album_id: &str) -> Result<Vec<PlatformTrack>>;

    /// Get related/similar artists
    async fn get_related_artists(&self, platform_id: &str) -> Result<Vec<PlatformArtist>>;

    /// Perform incremental sync from checkpoint
    async fn sync_incremental(
        &self,
        checkpoint: Option<SyncCheckpoint>,
        progress_callback: Box<dyn Fn(SyncProgress) + Send + Sync>,
    ) -> Result<SyncResult>;

    /// Perform full catalog sync
    async fn sync_full(
        &self,
        progress_callback: Box<dyn Fn(SyncProgress) + Send + Sync>,
    ) -> Result<SyncResult>;

    /// Get current sync checkpoint
    async fn get_checkpoint(&self, sync_run_id: Uuid) -> Result<Option<SyncCheckpoint>>;

    /// Save sync checkpoint
    async fn save_checkpoint(&self, checkpoint: &SyncCheckpoint) -> Result<()>;
}

/// Collaboration detection from track credits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedCollaboration {
    /// Track that contains the collaboration
    pub track: PlatformTrack,
    /// Primary artist ID
    pub primary_artist_id: String,
    /// Featured artist IDs
    pub featured_artist_ids: Vec<String>,
    /// Collaboration type (feature, remix, producer, etc.)
    pub collaboration_type: CollaborationType,
}

/// Types of artist collaborations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollaborationType {
    /// Featured artist (ft., feat., featuring)
    Feature,
    /// Remix by another artist
    Remix,
    /// Producer credit
    Producer,
    /// Songwriter credit
    Songwriter,
    /// DJ/Mix (for DJ compilations)
    DjMix,
    /// Cover version
    Cover,
    /// Sample (uses sample from another artist)
    Sample,
    /// Unknown collaboration type
    Unknown,
}

impl CollaborationType {
    /// Detect collaboration type from track title
    pub fn from_title(title: &str) -> Option<Self> {
        let lower = title.to_lowercase();

        if lower.contains("remix") {
            Some(CollaborationType::Remix)
        } else if lower.contains("feat.")
            || lower.contains("ft.")
            || lower.contains("featuring")
            || lower.contains("with ")
        {
            Some(CollaborationType::Feature)
        } else if lower.contains("cover") {
            Some(CollaborationType::Cover)
        } else if lower.contains("prod.") || lower.contains("produced by") {
            Some(CollaborationType::Producer)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_display() {
        assert_eq!(Platform::Spotify.as_str(), "spotify");
        assert_eq!(Platform::AppleMusic.display_name(), "Apple Music");
    }

    #[test]
    fn test_collaboration_detection() {
        assert_eq!(
            CollaborationType::from_title("Song Title (feat. Other Artist)"),
            Some(CollaborationType::Feature)
        );
        assert_eq!(
            CollaborationType::from_title("Song Title (Remix)"),
            Some(CollaborationType::Remix)
        );
        assert_eq!(CollaborationType::from_title("Regular Song"), None);
    }
}
