use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

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
    pub recommendations_read: bool,
    pub recently_played_read: bool,
}

impl Default for AppleMusicCapabilities {
    fn default() -> Self {
        Self {
            library_read: true,
            library_modify: false, // Limited by Apple Music API
            playlist_read: true,
            playlist_modify: false, // Limited by Apple Music API
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
        (self.library_tracks.len() + self.library_albums.len() + self.library_playlists.len()) as u32
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