use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Spotify track information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyTrack {
    pub id: String,
    pub name: String,
    pub artists: Vec<SpotifyArtist>,
    pub album: SpotifyAlbum,
    pub duration_ms: u32,
    pub explicit: bool,
    pub popularity: Option<u32>,
    pub preview_url: Option<String>,
    pub external_urls: HashMap<String, String>,
    pub is_local: bool,
    pub is_playable: Option<bool>,
}

/// Spotify artist information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyArtist {
    pub id: String,
    pub name: String,
    pub external_urls: HashMap<String, String>,
    pub href: Option<String>,
    pub uri: String,
    pub genres: Option<Vec<String>>,
    pub images: Option<Vec<SpotifyImage>>,
    pub popularity: Option<u32>,
    pub followers: Option<SpotifyFollowers>,
}

/// Spotify album information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyAlbum {
    pub id: String,
    pub name: String,
    pub artists: Vec<SpotifyArtist>,
    pub album_type: String,
    pub total_tracks: u32,
    pub external_urls: HashMap<String, String>,
    pub images: Vec<SpotifyImage>,
    pub release_date: String,
    pub release_date_precision: String,
}

/// Spotify playlist information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyPlaylist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub owner: SpotifyUser,
    pub public: Option<bool>,
    pub collaborative: bool,
    pub tracks: SpotifyPlaylistTracks,
    pub external_urls: HashMap<String, String>,
    pub images: Vec<SpotifyImage>,
    pub snapshot_id: String,
}

/// Spotify playlist tracks container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyPlaylistTracks {
    pub href: String,
    pub total: u32,
    pub items: Option<Vec<SpotifyPlaylistTrack>>,
}

/// Spotify playlist track item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyPlaylistTrack {
    pub added_at: DateTime<Utc>,
    pub added_by: Option<SpotifyUser>,
    pub is_local: bool,
    pub track: Option<SpotifyTrack>,
}

/// Spotify user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyUser {
    pub id: String,
    pub display_name: Option<String>,
    pub external_urls: HashMap<String, String>,
    pub followers: Option<SpotifyFollowers>,
    pub images: Vec<SpotifyImage>,
}

/// Spotify image information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyImage {
    pub url: String,
    pub height: Option<u32>,
    pub width: Option<u32>,
}

/// Spotify followers information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyFollowers {
    pub href: Option<String>,
    pub total: u32,
}

/// Spotify saved track (liked song)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifySavedTrack {
    pub added_at: DateTime<Utc>,
    pub track: SpotifyTrack,
}

/// Spotify followed artist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyFollowedArtist {
    pub artist: SpotifyArtist,
    pub followed_at: Option<DateTime<Utc>>,
}

/// User's complete Spotify library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyLibrary {
    pub user_id: Uuid,
    pub spotify_user_id: String,
    pub liked_songs: Vec<SpotifySavedTrack>,
    pub playlists: Vec<SpotifyPlaylist>,
    pub followed_artists: Vec<SpotifyFollowedArtist>,
    pub saved_albums: Vec<SpotifyAlbum>,
    pub scanned_at: DateTime<Utc>,
}

/// Featured artist detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturedArtistDetection {
    pub track_id: String,
    pub track_name: String,
    pub primary_artists: Vec<String>, // Artist IDs
    pub featured_artists: Vec<String>, // Artist IDs detected as featured
    pub collaboration_artists: Vec<String>, // Artist IDs detected as collaborators
    pub detection_method: DetectionMethod,
    pub confidence: f64,
}

/// Method used to detect featured/collaboration artists
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DetectionMethod {
    TrackTitle, // Detected from track title (feat., ft., with, etc.)
    ArtistArray, // Multiple artists in the artists array
    AlbumArtist, // Different from track artists
    Metadata, // From additional metadata
}

/// Enforcement planning options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementOptions {
    pub aggressiveness: AggressivenessLevel,
    pub block_collaborations: bool,
    pub block_featuring: bool,
    pub block_songwriter_only: bool,
    pub preserve_user_playlists: bool, // Don't modify user-created playlists
    pub dry_run: bool,
}

/// Aggressiveness level for enforcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggressivenessLevel {
    Conservative, // Only exact matches
    Moderate,     // Include high-confidence featured/collab
    Aggressive,   // Include all detected associations
}

/// Enforcement plan for a user's library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementPlan {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub options: EnforcementOptions,
    pub dnp_artists: Vec<Uuid>, // Artist IDs from user's DNP list
    pub impact: EnforcementImpact,
    pub actions: Vec<PlannedAction>,
    pub estimated_duration_seconds: u32,
    pub created_at: DateTime<Utc>,
    pub idempotency_key: String,
}

/// Impact analysis of enforcement plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementImpact {
    pub liked_songs: LibraryImpact,
    pub playlists: PlaylistImpact,
    pub followed_artists: FollowingImpact,
    pub saved_albums: AlbumImpact,
    pub total_items_affected: u32,
    pub estimated_time_saved_hours: f64, // Estimated listening time avoided
}

/// Impact on liked songs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryImpact {
    pub total_tracks: u32,
    pub tracks_to_remove: u32,
    pub collaborations_found: u32,
    pub featuring_found: u32,
    pub exact_matches: u32,
}

/// Impact on playlists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistImpact {
    pub total_playlists: u32,
    pub playlists_to_modify: u32,
    pub total_tracks: u32,
    pub tracks_to_remove: u32,
    pub user_playlists_affected: u32,
    pub collaborative_playlists_affected: u32,
    pub playlist_details: Vec<PlaylistModification>,
}

/// Details of playlist modifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistModification {
    pub playlist_id: String,
    pub playlist_name: String,
    pub is_user_owned: bool,
    pub is_collaborative: bool,
    pub total_tracks: u32,
    pub tracks_to_remove: u32,
    pub affected_tracks: Vec<AffectedTrack>,
}

/// Impact on followed artists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowingImpact {
    pub total_followed: u32,
    pub artists_to_unfollow: u32,
    pub exact_matches: u32,
}

/// Impact on saved albums
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumImpact {
    pub total_albums: u32,
    pub albums_to_remove: u32,
    pub exact_matches: u32,
    pub collaboration_albums: u32,
}

/// Track affected by enforcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedTrack {
    pub track_id: String,
    pub track_name: String,
    pub artist_names: Vec<String>,
    pub blocked_artist_ids: Vec<String>,
    pub reason: BlockReason,
    pub confidence: f64,
}

/// Reason why a track is blocked
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockReason {
    ExactMatch,      // Artist is directly in DNP list
    Collaboration,   // Artist collaborates with DNP artist
    Featuring,       // Artist features DNP artist
    SongwriterOnly,  // DNP artist is songwriter/producer only
}

/// Planned action for enforcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedAction {
    pub id: Uuid,
    pub action_type: ActionType,
    pub entity_type: EntityType,
    pub entity_id: String,
    pub entity_name: String,
    pub reason: BlockReason,
    pub confidence: f64,
    pub estimated_duration_ms: u32,
    pub dependencies: Vec<Uuid>, // Other actions this depends on
    pub metadata: serde_json::Value,
}

/// Type of action to perform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    RemoveLikedSong,
    RemovePlaylistTrack,
    UnfollowArtist,
    RemoveSavedAlbum,
    SkipTrack, // For browser extension
}

/// Type of entity being acted upon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Track,
    Artist,
    Album,
    Playlist,
}

impl Default for EnforcementOptions {
    fn default() -> Self {
        Self {
            aggressiveness: AggressivenessLevel::Moderate,
            block_collaborations: true,
            block_featuring: true,
            block_songwriter_only: false,
            preserve_user_playlists: false,
            dry_run: true,
        }
    }
}

impl EnforcementPlan {
    pub fn new(
        user_id: Uuid,
        provider: String,
        options: EnforcementOptions,
        dnp_artists: Vec<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            provider,
            options,
            dnp_artists,
            impact: EnforcementImpact::default(),
            actions: Vec::new(),
            estimated_duration_seconds: 0,
            created_at: Utc::now(),
            idempotency_key: format!("{}_{}", user_id, Utc::now().timestamp_millis()),
        }
    }

    pub fn add_action(&mut self, action: PlannedAction) {
        self.estimated_duration_seconds += action.estimated_duration_ms / 1000;
        self.actions.push(action);
    }

    pub fn get_actions_by_type(&self, action_type: ActionType) -> Vec<&PlannedAction> {
        self.actions
            .iter()
            .filter(|action| std::mem::discriminant(&action.action_type) == std::mem::discriminant(&action_type))
            .collect()
    }
}

impl Default for EnforcementImpact {
    fn default() -> Self {
        Self {
            liked_songs: LibraryImpact::default(),
            playlists: PlaylistImpact::default(),
            followed_artists: FollowingImpact::default(),
            saved_albums: AlbumImpact::default(),
            total_items_affected: 0,
            estimated_time_saved_hours: 0.0,
        }
    }
}

impl Default for LibraryImpact {
    fn default() -> Self {
        Self {
            total_tracks: 0,
            tracks_to_remove: 0,
            collaborations_found: 0,
            featuring_found: 0,
            exact_matches: 0,
        }
    }
}

impl Default for PlaylistImpact {
    fn default() -> Self {
        Self {
            total_playlists: 0,
            playlists_to_modify: 0,
            total_tracks: 0,
            tracks_to_remove: 0,
            user_playlists_affected: 0,
            collaborative_playlists_affected: 0,
            playlist_details: Vec::new(),
        }
    }
}

impl Default for FollowingImpact {
    fn default() -> Self {
        Self {
            total_followed: 0,
            artists_to_unfollow: 0,
            exact_matches: 0,
        }
    }
}

impl Default for AlbumImpact {
    fn default() -> Self {
        Self {
            total_albums: 0,
            albums_to_remove: 0,
            exact_matches: 0,
            collaboration_albums: 0,
        }
    }
}