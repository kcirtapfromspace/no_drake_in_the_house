use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Apple Music track information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicTrack {
    pub id: String,
    pub attributes: AppleMusicTrackAttributes,
    pub relationships: Option<AppleMusicTrackRelationships>,
}

/// Apple Music track attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicTrackAttributes {
    pub name: String,
    pub artist_name: String,
    pub album_name: String,
    pub duration_in_millis: Option<u32>,
    pub genre_names: Vec<String>,
    pub release_date: Option<String>,
    pub isrc: Option<String>,
    pub artwork: Option<AppleMusicArtwork>,
    pub play_params: Option<AppleMusicPlayParams>,
    pub preview_url: Option<String>,
    pub content_rating: Option<String>,
}

/// Apple Music track relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicTrackRelationships {
    pub artists: Option<AppleMusicRelationshipData<AppleMusicArtist>>,
    pub albums: Option<AppleMusicRelationshipData<AppleMusicAlbum>>,
}

/// Apple Music artist information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicArtist {
    pub id: String,
    pub attributes: AppleMusicArtistAttributes,
}

/// Apple Music artist attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicArtistAttributes {
    pub name: String,
    pub genre_names: Vec<String>,
    pub artwork: Option<AppleMusicArtwork>,
    pub url: Option<String>,
}

/// Apple Music album information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicAlbum {
    pub id: String,
    pub attributes: AppleMusicAlbumAttributes,
    pub relationships: Option<AppleMusicAlbumRelationships>,
}

/// Apple Music album attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicAlbumAttributes {
    pub name: String,
    pub artist_name: String,
    pub artwork: Option<AppleMusicArtwork>,
    pub genre_names: Vec<String>,
    pub release_date: Option<String>,
    pub track_count: Option<u32>,
    pub upc: Option<String>,
    pub content_rating: Option<String>,
}

/// Apple Music album relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicAlbumRelationships {
    pub artists: Option<AppleMusicRelationshipData<AppleMusicArtist>>,
    pub tracks: Option<AppleMusicRelationshipData<AppleMusicTrack>>,
}

/// Apple Music playlist information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicPlaylist {
    pub id: String,
    pub attributes: AppleMusicPlaylistAttributes,
    pub relationships: Option<AppleMusicPlaylistRelationships>,
}

/// Apple Music playlist attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicPlaylistAttributes {
    pub name: String,
    pub description: Option<AppleMusicEditorialNotes>,
    pub artwork: Option<AppleMusicArtwork>,
    pub curator_name: Option<String>,
    pub last_modified_date: Option<DateTime<Utc>>,
    pub play_params: Option<AppleMusicPlayParams>,
    pub is_chart: Option<bool>,
    pub track_count: Option<u32>,
}

/// Apple Music playlist relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicPlaylistRelationships {
    pub tracks: Option<AppleMusicRelationshipData<AppleMusicTrack>>,
    pub curator: Option<AppleMusicRelationshipData<AppleMusicCurator>>,
}

/// Apple Music curator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicCurator {
    pub id: String,
    pub attributes: AppleMusicCuratorAttributes,
}

/// Apple Music curator attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicCuratorAttributes {
    pub name: String,
    pub artwork: Option<AppleMusicArtwork>,
    pub url: Option<String>,
}

/// Apple Music artwork information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicArtwork {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub url: String,
    pub bg_color: Option<String>,
    pub text_color1: Option<String>,
    pub text_color2: Option<String>,
    pub text_color3: Option<String>,
    pub text_color4: Option<String>,
}

/// Apple Music play parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicPlayParams {
    pub id: String,
    pub kind: String,
}

/// Apple Music editorial notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicEditorialNotes {
    pub short: Option<String>,
    pub standard: Option<String>,
}

/// Apple Music relationship data wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicRelationshipData<T> {
    pub data: Vec<T>,
    pub href: Option<String>,
    pub next: Option<String>,
}

/// Apple Music API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicResponse<T> {
    pub data: Vec<T>,
    pub href: Option<String>,
    pub next: Option<String>,
    pub meta: Option<AppleMusicMeta>,
}

/// Apple Music response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicMeta {
    pub total: Option<u32>,
}

/// Apple Music library resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicLibraryResource<T> {
    pub id: String,
    pub attributes: T,
    pub relationships: Option<serde_json::Value>,
}

/// Apple Music library track
pub type AppleMusicLibraryTrack = AppleMusicLibraryResource<AppleMusicTrackAttributes>;

/// Apple Music library album
pub type AppleMusicLibraryAlbum = AppleMusicLibraryResource<AppleMusicAlbumAttributes>;

/// Apple Music library playlist
pub type AppleMusicLibraryPlaylist = AppleMusicLibraryResource<AppleMusicPlaylistAttributes>;

/// User's complete Apple Music library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicLibrary {
    pub user_id: Uuid,
    pub apple_music_user_id: String,
    pub library_tracks: Vec<AppleMusicLibraryTrack>,
    pub library_albums: Vec<AppleMusicLibraryAlbum>,
    pub library_playlists: Vec<AppleMusicLibraryPlaylist>,
    pub scanned_at: DateTime<Utc>,
}

/// Apple Music token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicTokenInfo {
    pub user_token: String,
    pub developer_token: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub music_user_token: Option<String>,
}

/// Apple Music developer token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicDeveloperToken {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

/// Apple Music user token request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicUserTokenRequest {
    pub developer_token: String,
}

/// Apple Music user token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicUserTokenResponse {
    pub music_user_token: String,
}

/// Apple Music capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicCapabilities {
    pub library_read: bool,
    pub library_modify: bool,
    pub playlist_read: bool,
    pub playlist_modify: bool,
    pub ratings_read: bool,
    pub ratings_modify: bool,
    pub recommendations_read: bool,
    pub recently_played_read: bool,
}

impl Default for AppleMusicCapabilities {
    fn default() -> Self {
        Self {
            library_read: true,
            library_modify: false, // Limited by Apple Music API
            playlist_read: true,
            playlist_modify: false,      // Limited by Apple Music API
            ratings_read: true,          // Can read user ratings
            ratings_modify: true,        // Can set ratings (enforcement mechanism!)
            recommendations_read: false, // Not available in API
            recently_played_read: true,
        }
    }
}

/// Apple Music enforcement options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicEnforcementOptions {
    pub scan_library: bool,
    pub scan_playlists: bool,
    pub export_blocked_content: bool, // Since we can't modify, export for manual action
    pub generate_report: bool,
}

impl Default for AppleMusicEnforcementOptions {
    fn default() -> Self {
        Self {
            scan_library: true,
            scan_playlists: true,
            export_blocked_content: true,
            generate_report: true,
        }
    }
}

/// Apple Music enforcement result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicEnforcementResult {
    pub user_id: Uuid,
    pub scan_completed_at: DateTime<Utc>,
    pub library_tracks_scanned: u32,
    pub library_albums_scanned: u32,
    pub playlists_scanned: u32,
    pub blocked_tracks_found: u32,
    pub blocked_albums_found: u32,
    pub blocked_playlists_found: u32,
    pub export_file_path: Option<String>,
    pub report_generated: bool,
    pub limitations_encountered: Vec<String>,
}

/// Apple Music search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicSearchRequest {
    pub term: String,
    pub types: Vec<String>, // "songs", "albums", "artists", "playlists"
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Apple Music search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicSearchResponse {
    pub results: AppleMusicSearchResults,
}

/// Apple Music search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicSearchResults {
    pub songs: Option<AppleMusicResponse<AppleMusicTrack>>,
    pub albums: Option<AppleMusicResponse<AppleMusicAlbum>>,
    pub artists: Option<AppleMusicResponse<AppleMusicArtist>>,
    pub playlists: Option<AppleMusicResponse<AppleMusicPlaylist>>,
}

/// Apple Music error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicError {
    pub id: String,
    pub title: String,
    pub detail: Option<String>,
    pub status: String,
    pub code: String,
    pub source: Option<AppleMusicErrorSource>,
}

/// Apple Music error source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicErrorSource {
    pub parameter: Option<String>,
    pub pointer: Option<String>,
}

/// Apple Music error response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicErrorResponse {
    pub errors: Vec<AppleMusicError>,
}

impl AppleMusicLibrary {
    pub fn new(user_id: Uuid, apple_music_user_id: String) -> Self {
        Self {
            user_id,
            apple_music_user_id,
            library_tracks: Vec::new(),
            library_albums: Vec::new(),
            library_playlists: Vec::new(),
            scanned_at: Utc::now(),
        }
    }

    pub fn total_items(&self) -> u32 {
        (self.library_tracks.len() + self.library_albums.len() + self.library_playlists.len())
            as u32
    }
}

impl AppleMusicEnforcementResult {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            scan_completed_at: Utc::now(),
            library_tracks_scanned: 0,
            library_albums_scanned: 0,
            playlists_scanned: 0,
            blocked_tracks_found: 0,
            blocked_albums_found: 0,
            blocked_playlists_found: 0,
            export_file_path: None,
            report_generated: false,
            limitations_encountered: Vec::new(),
        }
    }

    pub fn add_limitation(&mut self, limitation: String) {
        self.limitations_encountered.push(limitation);
    }

    pub fn total_blocked_items(&self) -> u32 {
        self.blocked_tracks_found + self.blocked_albums_found + self.blocked_playlists_found
    }
}

// ============================================
// Rating/Enforcement Types
// ============================================

/// Result of a batch rating operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRatingResult {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub errors: Vec<RatingError>,
}

/// Individual rating error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingError {
    pub resource_id: String,
    pub resource_type: String,
    pub error_message: String,
}

/// Resource type for enforcement actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AppleMusicResourceType {
    Song,
    LibrarySong,
    Album,
    LibraryAlbum,
    Playlist,
    LibraryPlaylist,
}

impl std::fmt::Display for AppleMusicResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Song => write!(f, "song"),
            Self::LibrarySong => write!(f, "library_song"),
            Self::Album => write!(f, "album"),
            Self::LibraryAlbum => write!(f, "library_album"),
            Self::Playlist => write!(f, "playlist"),
            Self::LibraryPlaylist => write!(f, "library_playlist"),
        }
    }
}

/// Enforcement action type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementActionType {
    Dislike,
    RemoveRating,
}

impl std::fmt::Display for EnforcementActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dislike => write!(f, "dislike"),
            Self::RemoveRating => write!(f, "remove_rating"),
        }
    }
}

/// Enforcement run status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementRunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    RolledBack,
}

impl std::fmt::Display for EnforcementRunStatus {
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

/// Apple Music enforcement run record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicEnforcementRun {
    pub id: Uuid,
    pub user_id: Uuid,
    pub connection_id: Uuid,
    pub status: EnforcementRunStatus,
    pub options: AppleMusicRatingEnforcementOptions,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub songs_scanned: u32,
    pub albums_scanned: u32,
    pub songs_disliked: u32,
    pub albums_disliked: u32,
    pub errors: u32,
    pub error_details: Option<serde_json::Value>,
}

/// Apple Music enforcement action record (for rollback support)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicEnforcementAction {
    pub id: Uuid,
    pub run_id: Uuid,
    pub user_id: Uuid,
    pub resource_type: AppleMusicResourceType,
    pub resource_id: String,
    pub resource_name: Option<String>,
    pub artist_name: Option<String>,
    pub action: EnforcementActionType,
    pub previous_rating: Option<i8>,
    pub created_at: DateTime<Utc>,
}

/// Options for rating-based enforcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMusicRatingEnforcementOptions {
    pub dislike_songs: bool,
    pub dislike_albums: bool,
    pub include_library: bool,
    pub include_catalog: bool,
    pub batch_size: usize,
    pub dry_run: bool,
}

impl Default for AppleMusicRatingEnforcementOptions {
    fn default() -> Self {
        Self {
            dislike_songs: true,
            dislike_albums: true,
            include_library: true,
            include_catalog: false,
            batch_size: 50,
            dry_run: false,
        }
    }
}

/// Progress update during enforcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementProgress {
    pub phase: String,
    pub total_items: usize,
    pub processed_items: usize,
    pub songs_disliked: usize,
    pub albums_disliked: usize,
    pub errors: usize,
    pub current_item: Option<String>,
}

impl EnforcementProgress {
    pub fn new() -> Self {
        Self {
            phase: "initializing".to_string(),
            total_items: 0,
            processed_items: 0,
            songs_disliked: 0,
            albums_disliked: 0,
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

impl Default for EnforcementProgress {
    fn default() -> Self {
        Self::new()
    }
}

/// Blocked content scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedContentScan {
    pub blocked_songs: Vec<BlockedSongInfo>,
    pub blocked_albums: Vec<BlockedAlbumInfo>,
    pub total_songs_scanned: usize,
    pub total_albums_scanned: usize,
}

impl BlockedContentScan {
    pub fn new() -> Self {
        Self {
            blocked_songs: Vec::new(),
            blocked_albums: Vec::new(),
            total_songs_scanned: 0,
            total_albums_scanned: 0,
        }
    }

    pub fn total_blocked(&self) -> usize {
        self.blocked_songs.len() + self.blocked_albums.len()
    }
}

impl Default for BlockedContentScan {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a blocked song
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedSongInfo {
    pub library_song_id: String,
    pub catalog_song_id: Option<String>,
    pub name: String,
    pub artist_name: String,
    pub album_name: String,
    pub blocked_artist_id: Option<Uuid>,
}

/// Information about a blocked album
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedAlbumInfo {
    pub library_album_id: String,
    pub catalog_album_id: Option<String>,
    pub name: String,
    pub artist_name: String,
    pub blocked_artist_id: Option<Uuid>,
}

/// Enforcement preview result (dry run)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementPreview {
    pub songs_to_dislike: Vec<BlockedSongInfo>,
    pub albums_to_dislike: Vec<BlockedAlbumInfo>,
    pub total_songs: usize,
    pub total_albums: usize,
    pub estimated_duration_seconds: u64,
}

/// Result of an enforcement run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingEnforcementResult {
    pub run_id: Uuid,
    pub status: EnforcementRunStatus,
    pub songs_disliked: usize,
    pub albums_disliked: usize,
    pub errors: Vec<RatingError>,
    pub duration_seconds: u64,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Result of a rollback operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    pub run_id: Uuid,
    pub ratings_removed: usize,
    pub errors: Vec<RatingError>,
    pub duration_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================
    // AppleMusicTrack Tests
    // ============================================

    #[test]
    fn test_apple_music_track_creation() {
        let track = AppleMusicTrack {
            id: "1234567890".to_string(),
            attributes: AppleMusicTrackAttributes {
                name: "Hotline Bling".to_string(),
                artist_name: "Drake".to_string(),
                album_name: "Views".to_string(),
                duration_in_millis: Some(267000),
                genre_names: vec!["Hip-Hop/Rap".to_string()],
                release_date: Some("2016-04-29".to_string()),
                isrc: Some("USCM51600001".to_string()),
                artwork: None,
                play_params: None,
                preview_url: Some("https://example.com/preview.m4a".to_string()),
                content_rating: None,
            },
            relationships: None,
        };

        assert_eq!(track.id, "1234567890");
        assert_eq!(track.attributes.name, "Hotline Bling");
        assert_eq!(track.attributes.artist_name, "Drake");
        assert_eq!(track.attributes.duration_in_millis, Some(267000));
    }

    #[test]
    fn test_apple_music_track_serialization() {
        let track = AppleMusicTrack {
            id: "123".to_string(),
            attributes: AppleMusicTrackAttributes {
                name: "Test Song".to_string(),
                artist_name: "Test Artist".to_string(),
                album_name: "Test Album".to_string(),
                duration_in_millis: Some(180000),
                genre_names: vec!["Pop".to_string()],
                release_date: None,
                isrc: None,
                artwork: None,
                play_params: None,
                preview_url: None,
                content_rating: None,
            },
            relationships: None,
        };

        let json = serde_json::to_string(&track).unwrap();
        assert!(json.contains("Test Song"));
        assert!(json.contains("Test Artist"));

        let deserialized: AppleMusicTrack = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, track.id);
        assert_eq!(deserialized.attributes.name, track.attributes.name);
    }

    // ============================================
    // AppleMusicAlbum Tests
    // ============================================

    #[test]
    fn test_apple_music_album_creation() {
        let album = AppleMusicAlbum {
            id: "album-123".to_string(),
            attributes: AppleMusicAlbumAttributes {
                name: "Views".to_string(),
                artist_name: "Drake".to_string(),
                artwork: Some(AppleMusicArtwork {
                    width: Some(1500),
                    height: Some(1500),
                    url: "https://example.com/artwork.jpg".to_string(),
                    bg_color: Some("000000".to_string()),
                    text_color1: Some("FFFFFF".to_string()),
                    text_color2: None,
                    text_color3: None,
                    text_color4: None,
                }),
                genre_names: vec!["Hip-Hop/Rap".to_string()],
                release_date: Some("2016-04-29".to_string()),
                track_count: Some(20),
                upc: Some("602547854769".to_string()),
                content_rating: None,
            },
            relationships: None,
        };

        assert_eq!(album.id, "album-123");
        assert_eq!(album.attributes.name, "Views");
        assert_eq!(album.attributes.track_count, Some(20));
        assert!(album.attributes.artwork.is_some());
    }

    #[test]
    fn test_apple_music_album_serialization() {
        let album = AppleMusicAlbum {
            id: "456".to_string(),
            attributes: AppleMusicAlbumAttributes {
                name: "Test Album".to_string(),
                artist_name: "Test Artist".to_string(),
                artwork: None,
                genre_names: vec![],
                release_date: None,
                track_count: None,
                upc: None,
                content_rating: None,
            },
            relationships: None,
        };

        let json = serde_json::to_string(&album).unwrap();
        let deserialized: AppleMusicAlbum = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, album.id);
    }

    // ============================================
    // AppleMusicArtist Tests
    // ============================================

    #[test]
    fn test_apple_music_artist_creation() {
        let artist = AppleMusicArtist {
            id: "artist-789".to_string(),
            attributes: AppleMusicArtistAttributes {
                name: "Drake".to_string(),
                genre_names: vec!["Hip-Hop/Rap".to_string(), "Pop".to_string()],
                artwork: None,
                url: Some("https://music.apple.com/artist/drake".to_string()),
            },
        };

        assert_eq!(artist.id, "artist-789");
        assert_eq!(artist.attributes.name, "Drake");
        assert_eq!(artist.attributes.genre_names.len(), 2);
    }

    // ============================================
    // AppleMusicArtwork Tests
    // ============================================

    #[test]
    fn test_apple_music_artwork_creation() {
        let artwork = AppleMusicArtwork {
            width: Some(1500),
            height: Some(1500),
            url: "https://example.com/artwork/{w}x{h}.jpg".to_string(),
            bg_color: Some("1a1a1a".to_string()),
            text_color1: Some("ffffff".to_string()),
            text_color2: Some("cccccc".to_string()),
            text_color3: Some("999999".to_string()),
            text_color4: Some("666666".to_string()),
        };

        assert_eq!(artwork.width, Some(1500));
        assert_eq!(artwork.height, Some(1500));
        assert!(artwork.url.contains("{w}"));
    }

    #[test]
    fn test_apple_music_artwork_minimal() {
        let artwork = AppleMusicArtwork {
            width: None,
            height: None,
            url: "https://example.com/artwork.jpg".to_string(),
            bg_color: None,
            text_color1: None,
            text_color2: None,
            text_color3: None,
            text_color4: None,
        };

        assert!(artwork.width.is_none());
        assert_eq!(artwork.url, "https://example.com/artwork.jpg");
    }

    // ============================================
    // AppleMusicLibrary Tests
    // ============================================

    #[test]
    fn test_apple_music_library_new() {
        let user_id = Uuid::new_v4();
        let library = AppleMusicLibrary::new(user_id, "apple-user-123".to_string());

        assert_eq!(library.user_id, user_id);
        assert_eq!(library.apple_music_user_id, "apple-user-123");
        assert!(library.library_tracks.is_empty());
        assert!(library.library_albums.is_empty());
        assert!(library.library_playlists.is_empty());
        assert_eq!(library.total_items(), 0);
    }

    #[test]
    fn test_apple_music_library_total_items() {
        let user_id = Uuid::new_v4();
        let mut library = AppleMusicLibrary::new(user_id, "user-123".to_string());

        // Add mock tracks
        library.library_tracks.push(AppleMusicLibraryTrack {
            id: "track1".to_string(),
            attributes: AppleMusicTrackAttributes {
                name: "Song 1".to_string(),
                artist_name: "Artist 1".to_string(),
                album_name: "Album 1".to_string(),
                duration_in_millis: None,
                genre_names: vec![],
                release_date: None,
                isrc: None,
                artwork: None,
                play_params: None,
                preview_url: None,
                content_rating: None,
            },
            relationships: None,
        });

        library.library_tracks.push(AppleMusicLibraryTrack {
            id: "track2".to_string(),
            attributes: AppleMusicTrackAttributes {
                name: "Song 2".to_string(),
                artist_name: "Artist 2".to_string(),
                album_name: "Album 2".to_string(),
                duration_in_millis: None,
                genre_names: vec![],
                release_date: None,
                isrc: None,
                artwork: None,
                play_params: None,
                preview_url: None,
                content_rating: None,
            },
            relationships: None,
        });

        assert_eq!(library.total_items(), 2);
    }

    // ============================================
    // AppleMusicCapabilities Tests
    // ============================================

    #[test]
    fn test_apple_music_capabilities_default() {
        let caps = AppleMusicCapabilities::default();

        assert!(caps.library_read);
        assert!(!caps.library_modify); // Limited by API
        assert!(caps.playlist_read);
        assert!(!caps.playlist_modify); // Limited by API
        assert!(caps.ratings_read);
        assert!(caps.ratings_modify); // Enforcement mechanism!
        assert!(!caps.recommendations_read); // Not available
        assert!(caps.recently_played_read);
    }

    #[test]
    fn test_apple_music_capabilities_serialization() {
        let caps = AppleMusicCapabilities::default();
        let json = serde_json::to_string(&caps).unwrap();

        assert!(json.contains("\"library_read\":true"));
        assert!(json.contains("\"ratings_modify\":true"));

        let deserialized: AppleMusicCapabilities = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.library_read, caps.library_read);
        assert_eq!(deserialized.ratings_modify, caps.ratings_modify);
    }

    // ============================================
    // AppleMusicEnforcementOptions Tests
    // ============================================

    #[test]
    fn test_enforcement_options_default() {
        let options = AppleMusicEnforcementOptions::default();

        assert!(options.scan_library);
        assert!(options.scan_playlists);
        assert!(options.export_blocked_content);
        assert!(options.generate_report);
    }

    // ============================================
    // AppleMusicEnforcementResult Tests
    // ============================================

    #[test]
    fn test_enforcement_result_new() {
        let user_id = Uuid::new_v4();
        let result = AppleMusicEnforcementResult::new(user_id);

        assert_eq!(result.user_id, user_id);
        assert_eq!(result.library_tracks_scanned, 0);
        assert_eq!(result.library_albums_scanned, 0);
        assert_eq!(result.blocked_tracks_found, 0);
        assert_eq!(result.total_blocked_items(), 0);
        assert!(!result.report_generated);
    }

    #[test]
    fn test_enforcement_result_add_limitation() {
        let user_id = Uuid::new_v4();
        let mut result = AppleMusicEnforcementResult::new(user_id);

        result.add_limitation("Cannot modify library".to_string());
        result.add_limitation("Rate limited".to_string());

        assert_eq!(result.limitations_encountered.len(), 2);
        assert!(result
            .limitations_encountered
            .contains(&"Cannot modify library".to_string()));
    }

    #[test]
    fn test_enforcement_result_total_blocked_items() {
        let user_id = Uuid::new_v4();
        let mut result = AppleMusicEnforcementResult::new(user_id);

        result.blocked_tracks_found = 10;
        result.blocked_albums_found = 5;
        result.blocked_playlists_found = 2;

        assert_eq!(result.total_blocked_items(), 17);
    }

    // ============================================
    // AppleMusicResourceType Tests
    // ============================================

    #[test]
    fn test_resource_type_display() {
        assert_eq!(AppleMusicResourceType::Song.to_string(), "song");
        assert_eq!(
            AppleMusicResourceType::LibrarySong.to_string(),
            "library_song"
        );
        assert_eq!(AppleMusicResourceType::Album.to_string(), "album");
        assert_eq!(
            AppleMusicResourceType::LibraryAlbum.to_string(),
            "library_album"
        );
        assert_eq!(AppleMusicResourceType::Playlist.to_string(), "playlist");
        assert_eq!(
            AppleMusicResourceType::LibraryPlaylist.to_string(),
            "library_playlist"
        );
    }

    #[test]
    fn test_resource_type_serialization() {
        let song = AppleMusicResourceType::Song;
        let json = serde_json::to_string(&song).unwrap();
        assert_eq!(json, "\"song\"");

        let library_song = AppleMusicResourceType::LibrarySong;
        let json = serde_json::to_string(&library_song).unwrap();
        assert_eq!(json, "\"library_song\"");

        let deserialized: AppleMusicResourceType = serde_json::from_str("\"album\"").unwrap();
        assert_eq!(deserialized, AppleMusicResourceType::Album);
    }

    // ============================================
    // EnforcementActionType Tests
    // ============================================

    #[test]
    fn test_action_type_display() {
        assert_eq!(EnforcementActionType::Dislike.to_string(), "dislike");
        assert_eq!(
            EnforcementActionType::RemoveRating.to_string(),
            "remove_rating"
        );
    }

    #[test]
    fn test_action_type_serialization() {
        let dislike = EnforcementActionType::Dislike;
        let json = serde_json::to_string(&dislike).unwrap();
        assert_eq!(json, "\"dislike\"");

        let deserialized: EnforcementActionType =
            serde_json::from_str("\"remove_rating\"").unwrap();
        assert_eq!(deserialized, EnforcementActionType::RemoveRating);
    }

    // ============================================
    // EnforcementRunStatus Tests
    // ============================================

    #[test]
    fn test_run_status_display() {
        assert_eq!(EnforcementRunStatus::Pending.to_string(), "pending");
        assert_eq!(EnforcementRunStatus::Running.to_string(), "running");
        assert_eq!(EnforcementRunStatus::Completed.to_string(), "completed");
        assert_eq!(EnforcementRunStatus::Failed.to_string(), "failed");
        assert_eq!(EnforcementRunStatus::RolledBack.to_string(), "rolled_back");
    }

    #[test]
    fn test_run_status_serialization() {
        let completed = EnforcementRunStatus::Completed;
        let json = serde_json::to_string(&completed).unwrap();
        assert_eq!(json, "\"completed\"");

        let deserialized: EnforcementRunStatus = serde_json::from_str("\"failed\"").unwrap();
        assert_eq!(deserialized, EnforcementRunStatus::Failed);
    }

    // ============================================
    // AppleMusicRatingEnforcementOptions Tests
    // ============================================

    #[test]
    fn test_rating_enforcement_options_default() {
        let options = AppleMusicRatingEnforcementOptions::default();

        assert!(options.dislike_songs);
        assert!(options.dislike_albums);
        assert!(options.include_library);
        assert!(!options.include_catalog);
        assert_eq!(options.batch_size, 50);
        assert!(!options.dry_run);
    }

    #[test]
    fn test_rating_enforcement_options_serialization() {
        let options = AppleMusicRatingEnforcementOptions {
            dislike_songs: true,
            dislike_albums: false,
            include_library: true,
            include_catalog: true,
            batch_size: 100,
            dry_run: true,
        };

        let json = serde_json::to_string(&options).unwrap();
        assert!(json.contains("\"dislike_songs\":true"));
        assert!(json.contains("\"dislike_albums\":false"));
        assert!(json.contains("\"dry_run\":true"));

        let deserialized: AppleMusicRatingEnforcementOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.dislike_songs, options.dislike_songs);
        assert_eq!(deserialized.batch_size, options.batch_size);
    }

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
    fn test_enforcement_progress_percent_complete() {
        let mut progress = EnforcementProgress::new();
        assert_eq!(progress.percent_complete(), 0.0);

        progress.total_items = 100;
        progress.processed_items = 25;
        assert_eq!(progress.percent_complete(), 25.0);

        progress.processed_items = 50;
        assert_eq!(progress.percent_complete(), 50.0);

        progress.processed_items = 100;
        assert_eq!(progress.percent_complete(), 100.0);
    }

    #[test]
    fn test_enforcement_progress_default() {
        let progress = EnforcementProgress::default();
        assert_eq!(progress.phase, "initializing");
    }

    // ============================================
    // BlockedContentScan Tests
    // ============================================

    #[test]
    fn test_blocked_content_scan_new() {
        let scan = BlockedContentScan::new();

        assert!(scan.blocked_songs.is_empty());
        assert!(scan.blocked_albums.is_empty());
        assert_eq!(scan.total_songs_scanned, 0);
        assert_eq!(scan.total_albums_scanned, 0);
        assert_eq!(scan.total_blocked(), 0);
    }

    #[test]
    fn test_blocked_content_scan_total_blocked() {
        let mut scan = BlockedContentScan::new();

        scan.blocked_songs.push(BlockedSongInfo {
            library_song_id: "song1".to_string(),
            catalog_song_id: None,
            name: "Song 1".to_string(),
            artist_name: "Artist".to_string(),
            album_name: "Album".to_string(),
            blocked_artist_id: None,
        });

        scan.blocked_albums.push(BlockedAlbumInfo {
            library_album_id: "album1".to_string(),
            catalog_album_id: None,
            name: "Album 1".to_string(),
            artist_name: "Artist".to_string(),
            blocked_artist_id: None,
        });

        assert_eq!(scan.total_blocked(), 2);
    }

    // ============================================
    // BlockedSongInfo Tests
    // ============================================

    #[test]
    fn test_blocked_song_info_creation() {
        let song = BlockedSongInfo {
            library_song_id: "lib-123".to_string(),
            catalog_song_id: Some("cat-456".to_string()),
            name: "Bad Song".to_string(),
            artist_name: "Bad Artist".to_string(),
            album_name: "Bad Album".to_string(),
            blocked_artist_id: Some(Uuid::new_v4()),
        };

        assert_eq!(song.library_song_id, "lib-123");
        assert!(song.catalog_song_id.is_some());
        assert!(song.blocked_artist_id.is_some());
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
    }

    // ============================================
    // BlockedAlbumInfo Tests
    // ============================================

    #[test]
    fn test_blocked_album_info_creation() {
        let album = BlockedAlbumInfo {
            library_album_id: "lib-album-123".to_string(),
            catalog_album_id: Some("cat-album-456".to_string()),
            name: "Bad Album".to_string(),
            artist_name: "Bad Artist".to_string(),
            blocked_artist_id: Some(Uuid::new_v4()),
        };

        assert_eq!(album.library_album_id, "lib-album-123");
        assert!(album.catalog_album_id.is_some());
    }

    // ============================================
    // EnforcementPreview Tests
    // ============================================

    #[test]
    fn test_enforcement_preview_creation() {
        let preview = EnforcementPreview {
            songs_to_dislike: vec![],
            albums_to_dislike: vec![],
            total_songs: 1000,
            total_albums: 200,
            estimated_duration_seconds: 5,
        };

        assert!(preview.songs_to_dislike.is_empty());
        assert_eq!(preview.total_songs, 1000);
        assert_eq!(preview.estimated_duration_seconds, 5);
    }

    // ============================================
    // RatingEnforcementResult Tests
    // ============================================

    #[test]
    fn test_rating_enforcement_result_creation() {
        let run_id = Uuid::new_v4();
        let result = RatingEnforcementResult {
            run_id,
            status: EnforcementRunStatus::Completed,
            songs_disliked: 15,
            albums_disliked: 5,
            errors: vec![],
            duration_seconds: 10,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };

        assert_eq!(result.run_id, run_id);
        assert_eq!(result.status, EnforcementRunStatus::Completed);
        assert_eq!(result.songs_disliked, 15);
        assert!(result.errors.is_empty());
    }

    // ============================================
    // RollbackResult Tests
    // ============================================

    #[test]
    fn test_rollback_result_creation() {
        let run_id = Uuid::new_v4();
        let result = RollbackResult {
            run_id,
            ratings_removed: 20,
            errors: vec![],
            duration_seconds: 8,
        };

        assert_eq!(result.run_id, run_id);
        assert_eq!(result.ratings_removed, 20);
        assert!(result.errors.is_empty());
    }

    // ============================================
    // BatchRatingResult Tests
    // ============================================

    #[test]
    fn test_batch_rating_result_creation() {
        let result = BatchRatingResult {
            total: 100,
            successful: 95,
            failed: 5,
            errors: vec![RatingError {
                resource_id: "song1".to_string(),
                resource_type: "library_song".to_string(),
                error_message: "Rate limited".to_string(),
            }],
        };

        assert_eq!(result.total, 100);
        assert_eq!(result.successful, 95);
        assert_eq!(result.failed, 5);
        assert_eq!(result.errors.len(), 1);
    }

    // ============================================
    // RatingError Tests
    // ============================================

    #[test]
    fn test_rating_error_creation() {
        let error = RatingError {
            resource_id: "song-123".to_string(),
            resource_type: "library_song".to_string(),
            error_message: "API Error: 429 Too Many Requests".to_string(),
        };

        assert_eq!(error.resource_id, "song-123");
        assert_eq!(error.resource_type, "library_song");
        assert!(error.error_message.contains("429"));
    }

    #[test]
    fn test_rating_error_serialization() {
        let error = RatingError {
            resource_id: "album-456".to_string(),
            resource_type: "library_album".to_string(),
            error_message: "Not found".to_string(),
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("album-456"));
        assert!(json.contains("Not found"));

        let deserialized: RatingError = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.resource_id, error.resource_id);
    }

    // ============================================
    // AppleMusicSearchRequest Tests
    // ============================================

    #[test]
    fn test_search_request_creation() {
        let request = AppleMusicSearchRequest {
            term: "Drake".to_string(),
            types: vec![
                "songs".to_string(),
                "albums".to_string(),
                "artists".to_string(),
            ],
            limit: Some(25),
            offset: Some(0),
        };

        assert_eq!(request.term, "Drake");
        assert_eq!(request.types.len(), 3);
        assert_eq!(request.limit, Some(25));
    }

    // ============================================
    // AppleMusicError Tests
    // ============================================

    #[test]
    fn test_apple_music_error_creation() {
        let error = AppleMusicError {
            id: "error-123".to_string(),
            title: "Not Found".to_string(),
            detail: Some("The requested resource was not found".to_string()),
            status: "404".to_string(),
            code: "NOT_FOUND".to_string(),
            source: Some(AppleMusicErrorSource {
                parameter: Some("id".to_string()),
                pointer: None,
            }),
        };

        assert_eq!(error.id, "error-123");
        assert_eq!(error.title, "Not Found");
        assert_eq!(error.status, "404");
        assert!(error.source.is_some());
    }

    #[test]
    fn test_apple_music_error_response() {
        let response = AppleMusicErrorResponse {
            errors: vec![AppleMusicError {
                id: "1".to_string(),
                title: "Unauthorized".to_string(),
                detail: None,
                status: "401".to_string(),
                code: "UNAUTHORIZED".to_string(),
                source: None,
            }],
        };

        assert_eq!(response.errors.len(), 1);
        assert_eq!(response.errors[0].status, "401");
    }

    // ============================================
    // AppleMusicTokenInfo Tests
    // ============================================

    #[test]
    fn test_token_info_creation() {
        let token_info = AppleMusicTokenInfo {
            user_token: "user-token-123".to_string(),
            developer_token: "dev-token-456".to_string(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
            music_user_token: Some("music-user-token-789".to_string()),
        };

        assert_eq!(token_info.user_token, "user-token-123");
        assert!(token_info.expires_at.is_some());
        assert!(token_info.music_user_token.is_some());
    }

    // ============================================
    // AppleMusicDeveloperToken Tests
    // ============================================

    #[test]
    fn test_developer_token_creation() {
        let expires_at = Utc::now() + chrono::Duration::hours(6);
        let token = AppleMusicDeveloperToken {
            token: "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9...".to_string(),
            expires_at,
        };

        assert!(token.token.starts_with("eyJ"));
        assert!(token.expires_at > Utc::now());
    }

    // ============================================
    // AppleMusicPlayParams Tests
    // ============================================

    #[test]
    fn test_play_params_creation() {
        let params = AppleMusicPlayParams {
            id: "1234567890".to_string(),
            kind: "song".to_string(),
        };

        assert_eq!(params.id, "1234567890");
        assert_eq!(params.kind, "song");
    }

    // ============================================
    // AppleMusicEditorialNotes Tests
    // ============================================

    #[test]
    fn test_editorial_notes_creation() {
        let notes = AppleMusicEditorialNotes {
            short: Some("A great album".to_string()),
            standard: Some("This album represents a major artistic achievement...".to_string()),
        };

        assert!(notes.short.is_some());
        assert!(notes.standard.is_some());
    }

    // ============================================
    // AppleMusicResponse Tests
    // ============================================

    #[test]
    fn test_response_wrapper() {
        let response: AppleMusicResponse<AppleMusicTrack> = AppleMusicResponse {
            data: vec![],
            href: Some("https://api.music.apple.com/v1/catalog/us/songs".to_string()),
            next: Some("https://api.music.apple.com/v1/catalog/us/songs?offset=25".to_string()),
            meta: Some(AppleMusicMeta { total: Some(100) }),
        };

        assert!(response.data.is_empty());
        assert!(response.href.is_some());
        assert!(response.next.is_some());
        assert_eq!(response.meta.as_ref().unwrap().total, Some(100));
    }
}
