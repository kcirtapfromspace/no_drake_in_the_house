//! YouTube Music models for library scanning and enforcement
//!
//! YouTube Music uses the YouTube Data API v3 for library access.
//! Unlike Spotify, YouTube Music's API has limited support for library modifications,
//! so enforcement is implemented via likes/dislikes and playlist removal.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================
// YouTube Music Library Types
// ============================================

/// YouTube Music video/song information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMusicVideo {
    pub id: String,
    pub snippet: YouTubeMusicSnippet,
    pub content_details: Option<YouTubeMusicContentDetails>,
    pub status: Option<YouTubeMusicVideoStatus>,
}

/// YouTube Music video snippet (metadata)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YouTubeMusicSnippet {
    pub title: String,
    pub description: Option<String>,
    pub channel_id: String,
    pub channel_title: String,
    pub thumbnails: Option<YouTubeMusicThumbnails>,
    pub published_at: Option<String>,
    pub category_id: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// YouTube Music content details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YouTubeMusicContentDetails {
    pub duration: Option<String>, // ISO 8601 duration format (e.g., "PT4M30S")
    pub dimension: Option<String>,
    pub definition: Option<String>,
    pub licensed_content: Option<bool>,
}

/// YouTube Music video status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YouTubeMusicVideoStatus {
    pub upload_status: Option<String>,
    pub privacy_status: Option<String>,
    pub license: Option<String>,
    pub embeddable: Option<bool>,
}

/// YouTube Music thumbnails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMusicThumbnails {
    pub default: Option<YouTubeMusicThumbnail>,
    pub medium: Option<YouTubeMusicThumbnail>,
    pub high: Option<YouTubeMusicThumbnail>,
    pub standard: Option<YouTubeMusicThumbnail>,
    pub maxres: Option<YouTubeMusicThumbnail>,
}

/// Single thumbnail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMusicThumbnail {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// YouTube Music playlist information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMusicPlaylist {
    pub id: String,
    pub snippet: YouTubeMusicPlaylistSnippet,
    pub content_details: Option<YouTubeMusicPlaylistContentDetails>,
    pub status: Option<YouTubeMusicPlaylistStatus>,
}

/// YouTube Music playlist snippet
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YouTubeMusicPlaylistSnippet {
    pub title: String,
    pub description: Option<String>,
    pub channel_id: String,
    pub channel_title: String,
    pub thumbnails: Option<YouTubeMusicThumbnails>,
    pub published_at: Option<String>,
}

/// YouTube Music playlist content details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YouTubeMusicPlaylistContentDetails {
    pub item_count: Option<u32>,
}

/// YouTube Music playlist status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YouTubeMusicPlaylistStatus {
    pub privacy_status: Option<String>,
}

/// YouTube Music playlist item (video in a playlist)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMusicPlaylistItem {
    pub id: String,
    pub snippet: YouTubeMusicPlaylistItemSnippet,
    pub content_details: Option<YouTubeMusicPlaylistItemContentDetails>,
    pub status: Option<YouTubeMusicPlaylistItemStatus>,
}

/// YouTube Music playlist item snippet
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YouTubeMusicPlaylistItemSnippet {
    pub playlist_id: String,
    pub title: String,
    pub description: Option<String>,
    pub channel_id: String,
    pub channel_title: String,
    pub thumbnails: Option<YouTubeMusicThumbnails>,
    pub position: Option<u32>,
    pub resource_id: YouTubeMusicResourceId,
    pub video_owner_channel_title: Option<String>,
    pub video_owner_channel_id: Option<String>,
}

/// YouTube Music resource ID
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YouTubeMusicResourceId {
    pub kind: String, // e.g., "youtube#video"
    pub video_id: Option<String>,
    pub channel_id: Option<String>,
    pub playlist_id: Option<String>,
}

/// YouTube Music playlist item content details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YouTubeMusicPlaylistItemContentDetails {
    pub video_id: String,
    pub video_published_at: Option<String>,
}

/// YouTube Music playlist item status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YouTubeMusicPlaylistItemStatus {
    pub privacy_status: Option<String>,
}

/// YouTube Music channel (artist) information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMusicChannel {
    pub id: String,
    pub snippet: YouTubeMusicChannelSnippet,
    pub statistics: Option<YouTubeMusicChannelStatistics>,
}

/// YouTube Music channel snippet
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YouTubeMusicChannelSnippet {
    pub title: String,
    pub description: Option<String>,
    pub custom_url: Option<String>,
    pub thumbnails: Option<YouTubeMusicThumbnails>,
    pub published_at: Option<String>,
    pub country: Option<String>,
}

/// YouTube Music channel statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YouTubeMusicChannelStatistics {
    pub view_count: Option<String>,
    pub subscriber_count: Option<String>,
    pub video_count: Option<String>,
    pub hidden_subscriber_count: Option<bool>,
}

// ============================================
// YouTube Data API Response Types
// ============================================

/// YouTube Data API list response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YouTubeListResponse<T> {
    pub kind: String,
    pub etag: String,
    pub next_page_token: Option<String>,
    pub prev_page_token: Option<String>,
    pub page_info: Option<YouTubePageInfo>,
    pub items: Vec<T>,
}

/// YouTube Data API page info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YouTubePageInfo {
    pub total_results: Option<u32>,
    pub results_per_page: Option<u32>,
}

/// YouTube Data API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeErrorResponse {
    pub error: YouTubeError,
}

/// YouTube Data API error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeError {
    pub code: u16,
    pub message: String,
    pub errors: Vec<YouTubeErrorDetail>,
}

/// YouTube Data API error detail
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YouTubeErrorDetail {
    pub message: String,
    pub domain: String,
    pub reason: String,
    pub location: Option<String>,
    pub location_type: Option<String>,
}

// ============================================
// YouTube Music Library Types
// ============================================

/// User's YouTube Music library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMusicLibrary {
    pub user_id: Uuid,
    pub youtube_channel_id: String,
    pub liked_videos: Vec<YouTubeMusicVideo>,
    pub playlists: Vec<YouTubeMusicPlaylist>,
    pub subscriptions: Vec<YouTubeMusicChannel>,
    pub scanned_at: DateTime<Utc>,
}

impl YouTubeMusicLibrary {
    pub fn new(user_id: Uuid, youtube_channel_id: String) -> Self {
        Self {
            user_id,
            youtube_channel_id,
            liked_videos: Vec::new(),
            playlists: Vec::new(),
            subscriptions: Vec::new(),
            scanned_at: Utc::now(),
        }
    }

    pub fn total_items(&self) -> u32 {
        (self.liked_videos.len() + self.playlists.len() + self.subscriptions.len()) as u32
    }
}

// ============================================
// YouTube Music Enforcement Types
// ============================================

/// YouTube Music enforcement options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMusicEnforcementOptions {
    /// Remove blocked content from liked videos
    pub remove_from_likes: bool,
    /// Add "dislike" rating to blocked content
    pub dislike_blocked_content: bool,
    /// Remove blocked content from playlists
    pub remove_from_playlists: bool,
    /// Unsubscribe from blocked artist channels
    pub unsubscribe_from_artists: bool,
    /// Number of items to process per batch
    pub batch_size: usize,
    /// Perform dry run (preview only, no changes)
    pub dry_run: bool,
}

impl Default for YouTubeMusicEnforcementOptions {
    fn default() -> Self {
        Self {
            remove_from_likes: true,
            dislike_blocked_content: false, // Off by default, affects recommendations
            remove_from_playlists: true,
            unsubscribe_from_artists: true,
            batch_size: 50,
            dry_run: false,
        }
    }
}

/// Resource type for YouTube Music enforcement actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum YouTubeMusicResourceType {
    Video,
    LikedVideo,
    PlaylistItem,
    Playlist,
    Subscription,
    Channel,
}

impl std::fmt::Display for YouTubeMusicResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Video => write!(f, "video"),
            Self::LikedVideo => write!(f, "liked_video"),
            Self::PlaylistItem => write!(f, "playlist_item"),
            Self::Playlist => write!(f, "playlist"),
            Self::Subscription => write!(f, "subscription"),
            Self::Channel => write!(f, "channel"),
        }
    }
}

/// YouTube Music enforcement action type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum YouTubeMusicEnforcementActionType {
    RemoveFromLikes,
    Dislike,
    RemoveFromPlaylist,
    Unsubscribe,
    // Rollback actions
    AddToLikes,
    RemoveDislike,
    AddToPlaylist,
    Subscribe,
}

impl std::fmt::Display for YouTubeMusicEnforcementActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RemoveFromLikes => write!(f, "remove_from_likes"),
            Self::Dislike => write!(f, "dislike"),
            Self::RemoveFromPlaylist => write!(f, "remove_from_playlist"),
            Self::Unsubscribe => write!(f, "unsubscribe"),
            Self::AddToLikes => write!(f, "add_to_likes"),
            Self::RemoveDislike => write!(f, "remove_dislike"),
            Self::AddToPlaylist => write!(f, "add_to_playlist"),
            Self::Subscribe => write!(f, "subscribe"),
        }
    }
}

/// YouTube Music enforcement run status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum YouTubeMusicEnforcementRunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    RolledBack,
}

impl std::fmt::Display for YouTubeMusicEnforcementRunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::RolledBack => write!(f, "rolled_back"),
        }
    }
}

/// YouTube Music enforcement run record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMusicEnforcementRun {
    pub id: Uuid,
    pub user_id: Uuid,
    pub connection_id: Uuid,
    pub status: YouTubeMusicEnforcementRunStatus,
    pub options: YouTubeMusicEnforcementOptions,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub videos_scanned: u32,
    pub playlists_scanned: u32,
    pub subscriptions_scanned: u32,
    pub videos_removed: u32,
    pub playlist_items_removed: u32,
    pub subscriptions_removed: u32,
    pub errors: u32,
    pub error_details: Option<serde_json::Value>,
}

/// YouTube Music enforcement action record (for rollback support)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMusicEnforcementAction {
    pub id: Uuid,
    pub run_id: Uuid,
    pub user_id: Uuid,
    pub resource_type: YouTubeMusicResourceType,
    pub resource_id: String,
    pub resource_name: Option<String>,
    pub artist_name: Option<String>,
    pub action: YouTubeMusicEnforcementActionType,
    pub playlist_id: Option<String>, // For playlist item actions
    pub previous_state: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Progress update during YouTube Music enforcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMusicEnforcementProgress {
    pub phase: String,
    pub total_items: usize,
    pub processed_items: usize,
    pub videos_removed: usize,
    pub playlist_items_removed: usize,
    pub subscriptions_removed: usize,
    pub errors: usize,
    pub current_item: Option<String>,
}

impl YouTubeMusicEnforcementProgress {
    pub fn new() -> Self {
        Self {
            phase: "initializing".to_string(),
            total_items: 0,
            processed_items: 0,
            videos_removed: 0,
            playlist_items_removed: 0,
            subscriptions_removed: 0,
            errors: 0,
            current_item: None,
        }
    }

    pub fn percent_complete(&self) -> f32 {
        if self.total_items == 0 {
            0.0
        } else {
            (self.processed_items as f32 / self.total_items as f32) * 100.0
        }
    }
}

impl Default for YouTubeMusicEnforcementProgress {
    fn default() -> Self {
        Self::new()
    }
}

/// Blocked content scan result for YouTube Music
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMusicBlockedContentScan {
    pub blocked_videos: Vec<BlockedVideoInfo>,
    pub blocked_playlist_items: Vec<BlockedPlaylistItemInfo>,
    pub blocked_subscriptions: Vec<BlockedSubscriptionInfo>,
    pub total_videos_scanned: usize,
    pub total_playlist_items_scanned: usize,
    pub total_subscriptions_scanned: usize,
}

impl YouTubeMusicBlockedContentScan {
    pub fn new() -> Self {
        Self {
            blocked_videos: Vec::new(),
            blocked_playlist_items: Vec::new(),
            blocked_subscriptions: Vec::new(),
            total_videos_scanned: 0,
            total_playlist_items_scanned: 0,
            total_subscriptions_scanned: 0,
        }
    }

    pub fn total_blocked(&self) -> usize {
        self.blocked_videos.len()
            + self.blocked_playlist_items.len()
            + self.blocked_subscriptions.len()
    }
}

impl Default for YouTubeMusicBlockedContentScan {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a blocked video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedVideoInfo {
    pub video_id: String,
    pub title: String,
    pub channel_id: String,
    pub channel_title: String,
    pub blocked_artist_id: Option<Uuid>,
}

/// Information about a blocked playlist item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedPlaylistItemInfo {
    pub playlist_item_id: String,
    pub playlist_id: String,
    pub video_id: String,
    pub title: String,
    pub channel_id: String,
    pub channel_title: String,
    pub blocked_artist_id: Option<Uuid>,
}

/// Information about a blocked subscription (artist channel)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedSubscriptionInfo {
    pub subscription_id: String,
    pub channel_id: String,
    pub channel_title: String,
    pub blocked_artist_id: Option<Uuid>,
}

/// Enforcement preview result (dry run)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMusicEnforcementPreview {
    pub videos_to_remove: Vec<BlockedVideoInfo>,
    pub playlist_items_to_remove: Vec<BlockedPlaylistItemInfo>,
    pub subscriptions_to_remove: Vec<BlockedSubscriptionInfo>,
    pub total_videos: usize,
    pub total_playlist_items: usize,
    pub total_subscriptions: usize,
    pub estimated_duration_seconds: u64,
}

/// Result of a YouTube Music enforcement run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMusicEnforcementResult {
    pub run_id: Uuid,
    pub status: YouTubeMusicEnforcementRunStatus,
    pub videos_removed: usize,
    pub playlist_items_removed: usize,
    pub subscriptions_removed: usize,
    pub errors: Vec<YouTubeMusicEnforcementError>,
    pub duration_seconds: u64,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Individual enforcement error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMusicEnforcementError {
    pub resource_id: String,
    pub resource_type: String,
    pub error_message: String,
}

/// Result of a rollback operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMusicRollbackResult {
    pub run_id: Uuid,
    pub actions_restored: usize,
    pub errors: Vec<YouTubeMusicEnforcementError>,
    pub duration_seconds: u64,
}

// ============================================
// YouTube Music Rating Types
// ============================================

/// YouTube video rating (for enforcement via dislikes)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum YouTubeRating {
    Like,
    Dislike,
    None,
}

impl std::fmt::Display for YouTubeRating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Like => write!(f, "like"),
            Self::Dislike => write!(f, "dislike"),
            Self::None => write!(f, "none"),
        }
    }
}

// ============================================
// YouTube Music Token Types
// ============================================

/// YouTube Music token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMusicTokenInfo {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub scope: Option<String>,
}

/// YouTube Music capabilities (what the user has granted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMusicCapabilities {
    pub read_likes: bool,
    pub modify_likes: bool,
    pub read_playlists: bool,
    pub modify_playlists: bool,
    pub read_subscriptions: bool,
    pub modify_subscriptions: bool,
}

impl Default for YouTubeMusicCapabilities {
    fn default() -> Self {
        Self {
            read_likes: true,
            modify_likes: true,
            read_playlists: true,
            modify_playlists: true,
            read_subscriptions: true,
            modify_subscriptions: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_youtube_music_library_new() {
        let user_id = Uuid::new_v4();
        let library = YouTubeMusicLibrary::new(user_id, "UC123456".to_string());

        assert_eq!(library.user_id, user_id);
        assert_eq!(library.youtube_channel_id, "UC123456");
        assert!(library.liked_videos.is_empty());
        assert!(library.playlists.is_empty());
        assert!(library.subscriptions.is_empty());
        assert_eq!(library.total_items(), 0);
    }

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
    fn test_resource_type_display() {
        assert_eq!(YouTubeMusicResourceType::Video.to_string(), "video");
        assert_eq!(
            YouTubeMusicResourceType::LikedVideo.to_string(),
            "liked_video"
        );
        assert_eq!(
            YouTubeMusicResourceType::PlaylistItem.to_string(),
            "playlist_item"
        );
        assert_eq!(
            YouTubeMusicResourceType::Subscription.to_string(),
            "subscription"
        );
    }

    #[test]
    fn test_action_type_display() {
        assert_eq!(
            YouTubeMusicEnforcementActionType::RemoveFromLikes.to_string(),
            "remove_from_likes"
        );
        assert_eq!(
            YouTubeMusicEnforcementActionType::Dislike.to_string(),
            "dislike"
        );
        assert_eq!(
            YouTubeMusicEnforcementActionType::RemoveFromPlaylist.to_string(),
            "remove_from_playlist"
        );
        assert_eq!(
            YouTubeMusicEnforcementActionType::Unsubscribe.to_string(),
            "unsubscribe"
        );
    }

    #[test]
    fn test_run_status_display() {
        assert_eq!(
            YouTubeMusicEnforcementRunStatus::Pending.to_string(),
            "pending"
        );
        assert_eq!(
            YouTubeMusicEnforcementRunStatus::Running.to_string(),
            "running"
        );
        assert_eq!(
            YouTubeMusicEnforcementRunStatus::Completed.to_string(),
            "completed"
        );
        assert_eq!(
            YouTubeMusicEnforcementRunStatus::Failed.to_string(),
            "failed"
        );
    }

    #[test]
    fn test_enforcement_progress_new() {
        let progress = YouTubeMusicEnforcementProgress::new();

        assert_eq!(progress.phase, "initializing");
        assert_eq!(progress.total_items, 0);
        assert_eq!(progress.processed_items, 0);
        assert_eq!(progress.percent_complete(), 0.0);
    }

    #[test]
    fn test_enforcement_progress_percent() {
        let mut progress = YouTubeMusicEnforcementProgress::new();
        progress.total_items = 100;
        progress.processed_items = 50;

        assert_eq!(progress.percent_complete(), 50.0);
    }

    #[test]
    fn test_blocked_content_scan_new() {
        let scan = YouTubeMusicBlockedContentScan::new();

        assert!(scan.blocked_videos.is_empty());
        assert!(scan.blocked_playlist_items.is_empty());
        assert!(scan.blocked_subscriptions.is_empty());
        assert_eq!(scan.total_blocked(), 0);
    }

    #[test]
    fn test_youtube_rating_display() {
        assert_eq!(YouTubeRating::Like.to_string(), "like");
        assert_eq!(YouTubeRating::Dislike.to_string(), "dislike");
        assert_eq!(YouTubeRating::None.to_string(), "none");
    }

    #[test]
    fn test_capabilities_default() {
        let caps = YouTubeMusicCapabilities::default();

        assert!(caps.read_likes);
        assert!(caps.modify_likes);
        assert!(caps.read_playlists);
        assert!(caps.modify_playlists);
        assert!(caps.read_subscriptions);
        assert!(caps.modify_subscriptions);
    }
}
