use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Tidal track information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalTrack {
    pub id: u64,
    pub title: String,
    pub artists: Vec<TidalArtist>,
    pub album: TidalAlbum,
    pub duration: u32, // in seconds
    pub explicit: bool,
    pub popularity: Option<u32>,
    pub audio_quality: String,
    pub url: String,
    pub isrc: Option<String>,
}

/// Tidal artist information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalArtist {
    pub id: u64,
    pub name: String,
    pub url: Option<String>,
    pub picture: Option<String>,
    #[serde(rename = "type")]
    pub artist_type: Option<String>,
}

/// Tidal album information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalAlbum {
    pub id: u64,
    pub title: String,
    pub artists: Vec<TidalArtist>,
    pub cover: Option<String>,
    pub release_date: Option<String>,
    pub duration: Option<u32>,
    pub number_of_tracks: Option<u32>,
    pub url: Option<String>,
}

/// Tidal playlist information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalPlaylist {
    pub uuid: String,
    pub title: String,
    pub description: Option<String>,
    pub creator: Option<TidalUser>,
    pub duration: u32,
    pub number_of_tracks: u32,
    pub public_playlist: bool,
    pub url: Option<String>,
    pub image: Option<String>,
    pub created: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

/// Tidal user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalUser {
    pub id: u64,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub country_code: Option<String>,
    pub picture: Option<String>,
}

/// Tidal favorite track item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalFavoriteTrack {
    pub created: DateTime<Utc>,
    pub item: TidalTrack,
}

/// Tidal favorite artist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalFavoriteArtist {
    pub created: DateTime<Utc>,
    pub item: TidalArtist,
}

/// Tidal favorite album
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalFavoriteAlbum {
    pub created: DateTime<Utc>,
    pub item: TidalAlbum,
}

/// Tidal playlist track item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalPlaylistTrack {
    pub item: TidalTrack,
    pub date_added: DateTime<Utc>,
}

/// User's complete Tidal library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalLibrary {
    pub user_id: Uuid,
    pub tidal_user_id: u64,
    pub favorite_tracks: Vec<TidalFavoriteTrack>,
    pub favorite_artists: Vec<TidalFavoriteArtist>,
    pub favorite_albums: Vec<TidalFavoriteAlbum>,
    pub playlists: Vec<TidalPlaylist>,
    pub scanned_at: DateTime<Utc>,
}

/// Tidal API response wrapper for paginated results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalPaginatedResponse<T> {
    pub limit: u32,
    pub offset: u32,
    pub total_number_of_items: u32,
    pub items: Vec<T>,
}

/// Tidal library scan result with detailed counts and items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalLibraryScanResult {
    /// The full library data
    pub library: TidalLibrary,
    /// Summary counts for the scan
    pub counts: TidalLibraryScanCounts,
    /// Scan metadata
    pub metadata: TidalLibraryScanMetadata,
}

/// Summary counts from a Tidal library scan
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TidalLibraryScanCounts {
    /// Total favorite tracks
    pub favorite_tracks_count: u32,
    /// Total favorite artists
    pub favorite_artists_count: u32,
    /// Total favorite albums
    pub favorite_albums_count: u32,
    /// Total playlists
    pub playlists_count: u32,
    /// Total tracks across all playlists
    pub playlist_tracks_count: u32,
    /// Total unique tracks (deduplicated)
    pub total_unique_tracks: u32,
    /// Total unique artists found
    pub total_unique_artists: u32,
}

/// Metadata about the Tidal library scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalLibraryScanMetadata {
    /// When the scan started
    pub started_at: DateTime<Utc>,
    /// When the scan completed
    pub completed_at: DateTime<Utc>,
    /// Total duration in milliseconds
    pub duration_ms: u64,
    /// Number of API requests made
    pub api_requests_count: u32,
    /// Number of rate limit retries encountered
    pub rate_limit_retries: u32,
    /// Whether the scan was complete or partial
    pub is_complete: bool,
    /// Any errors encountered during scanning (non-fatal)
    pub warnings: Vec<String>,
}

impl TidalLibraryScanResult {
    /// Create a new scan result from library data and timing info
    pub fn new(
        library: TidalLibrary,
        started_at: DateTime<Utc>,
        api_requests_count: u32,
        rate_limit_retries: u32,
        warnings: Vec<String>,
    ) -> Self {
        let completed_at = Utc::now();
        let duration_ms = (completed_at - started_at).num_milliseconds() as u64;

        // Calculate counts
        let favorite_tracks_count = library.favorite_tracks.len() as u32;
        let favorite_artists_count = library.favorite_artists.len() as u32;
        let favorite_albums_count = library.favorite_albums.len() as u32;
        let playlists_count = library.playlists.len() as u32;
        let playlist_tracks_count: u32 = library.playlists.iter().map(|p| p.number_of_tracks).sum();

        // Calculate unique tracks and artists
        let mut unique_track_ids = std::collections::HashSet::new();
        let mut unique_artist_ids = std::collections::HashSet::new();

        // From favorite tracks
        for favorite in &library.favorite_tracks {
            unique_track_ids.insert(favorite.item.id);
            for artist in &favorite.item.artists {
                unique_artist_ids.insert(artist.id);
            }
        }

        // From favorite artists
        for favorite in &library.favorite_artists {
            unique_artist_ids.insert(favorite.item.id);
        }

        // From favorite albums
        for favorite in &library.favorite_albums {
            for artist in &favorite.item.artists {
                unique_artist_ids.insert(artist.id);
            }
        }

        let counts = TidalLibraryScanCounts {
            favorite_tracks_count,
            favorite_artists_count,
            favorite_albums_count,
            playlists_count,
            playlist_tracks_count,
            total_unique_tracks: unique_track_ids.len() as u32,
            total_unique_artists: unique_artist_ids.len() as u32,
        };

        let metadata = TidalLibraryScanMetadata {
            started_at,
            completed_at,
            duration_ms,
            api_requests_count,
            rate_limit_retries,
            is_complete: warnings.is_empty(),
            warnings,
        };

        Self {
            library,
            counts,
            metadata,
        }
    }
}

/// Blocked content found in Tidal library during enforcement scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalBlockedContent {
    pub blocked_tracks: Vec<BlockedTidalTrack>,
    pub blocked_artists: Vec<BlockedTidalArtist>,
    pub blocked_albums: Vec<BlockedTidalAlbum>,
    pub blocked_playlist_tracks: Vec<BlockedTidalPlaylistTrack>,
}

/// A blocked track in Tidal library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedTidalTrack {
    pub track_id: u64,
    pub track_name: String,
    pub artist_name: String,
    pub album_name: String,
    pub blocked_artist_ids: Vec<u64>,
    pub block_reason: String,
}

/// A blocked artist in Tidal library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedTidalArtist {
    pub artist_id: u64,
    pub artist_name: String,
    pub block_reason: String,
}

/// A blocked album in Tidal library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedTidalAlbum {
    pub album_id: u64,
    pub album_name: String,
    pub artist_name: String,
    pub blocked_artist_ids: Vec<u64>,
    pub block_reason: String,
}

/// A blocked track in a Tidal playlist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedTidalPlaylistTrack {
    pub playlist_uuid: String,
    pub playlist_name: String,
    pub track_id: u64,
    pub track_name: String,
    pub artist_name: String,
    pub blocked_artist_ids: Vec<u64>,
    pub block_reason: String,
}

impl Default for TidalBlockedContent {
    fn default() -> Self {
        Self {
            blocked_tracks: Vec::new(),
            blocked_artists: Vec::new(),
            blocked_albums: Vec::new(),
            blocked_playlist_tracks: Vec::new(),
        }
    }
}

/// Tidal enforcement run request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalEnforcementRequest {
    /// Block tracks featuring blocked artists
    #[serde(default = "default_true")]
    pub block_featuring: bool,
    /// Block collaborative tracks with blocked artists
    #[serde(default = "default_true")]
    pub block_collaborations: bool,
    /// Preserve user-created playlists (don't modify them)
    #[serde(default = "default_true")]
    pub preserve_user_playlists: bool,
    /// Dry run mode (preview only, no changes)
    #[serde(default)]
    pub dry_run: bool,
    /// Optional idempotency key
    pub idempotency_key: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for TidalEnforcementRequest {
    fn default() -> Self {
        Self {
            block_featuring: true,
            block_collaborations: true,
            preserve_user_playlists: true,
            dry_run: false,
            idempotency_key: None,
        }
    }
}

/// Tidal enforcement result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalEnforcementResult {
    pub batch_id: Uuid,
    pub status: String,
    pub tracks_removed: u32,
    pub albums_removed: u32,
    pub artists_unfollowed: u32,
    pub playlist_tracks_removed: u32,
    pub errors_count: u32,
    pub errors: Vec<TidalEnforcementError>,
    pub execution_time_ms: u64,
    pub dry_run: bool,
}

/// Error that occurred during Tidal enforcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalEnforcementError {
    pub entity_type: String,
    pub entity_id: String,
    pub error_code: String,
    pub error_message: String,
    pub is_recoverable: bool,
}

/// Tidal enforcement preview response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalEnforcementPreview {
    pub tracks_to_remove: u32,
    pub albums_to_remove: u32,
    pub artists_to_unfollow: u32,
    pub playlist_tracks_to_remove: u32,
    pub total_favorite_tracks: u32,
    pub total_favorite_albums: u32,
    pub total_favorite_artists: u32,
    pub total_playlists: u32,
    pub estimated_duration_seconds: u32,
    pub blocked_content: TidalBlockedContent,
}

/// Tidal enforcement history item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalEnforcementHistoryItem {
    pub batch_id: Uuid,
    pub status: String,
    pub dry_run: bool,
    pub tracks_removed: u32,
    pub albums_removed: u32,
    pub artists_unfollowed: u32,
    pub playlist_tracks_removed: u32,
    pub errors_count: u32,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub can_rollback: bool,
}

/// Tidal capabilities response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalCapabilities {
    pub library_modification: bool,
    pub playlist_modification: bool,
    pub unfollow_artists: bool,
    pub remove_favorite_albums: bool,
    pub batch_operations: bool,
    pub rollback_support: bool,
    pub enforcement_effects: Vec<String>,
    pub limitations: Vec<String>,
}

impl Default for TidalCapabilities {
    fn default() -> Self {
        Self {
            library_modification: true,
            playlist_modification: true,
            unfollow_artists: true,
            remove_favorite_albums: true,
            batch_operations: false, // Tidal API doesn't support batch operations like Spotify
            rollback_support: true,
            enforcement_effects: vec![
                "Remove favorite tracks from blocked artists".to_string(),
                "Remove tracks from playlists (user-owned only by default)".to_string(),
                "Unfollow blocked artists".to_string(),
                "Remove favorite albums from blocked artists".to_string(),
            ],
            limitations: vec![
                "Cannot prevent playback of tracks".to_string(),
                "Individual API calls required (no batch operations)".to_string(),
                "Rate limited to ~100 requests per minute".to_string(),
                "Rollback only available for recent operations with saved state".to_string(),
            ],
        }
    }
}
