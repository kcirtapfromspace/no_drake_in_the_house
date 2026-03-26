use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A first-class playlist entity stored in the `playlists` table.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Playlist {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_playlist_id: String,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub owner_name: Option<String>,
    pub owner_id: Option<String>,
    pub is_public: Option<bool>,
    pub is_collaborative: bool,
    pub source_type: String,
    pub provider_track_count: Option<i32>,
    pub snapshot_id: Option<String>,
    pub last_synced: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// A track within a playlist, stored in `playlist_tracks`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlaylistTrack {
    pub id: Uuid,
    pub playlist_id: Uuid,
    pub provider_track_id: String,
    pub track_name: Option<String>,
    pub album_name: Option<String>,
    pub artist_id: Option<Uuid>,
    pub artist_name: Option<String>,
    pub position: i32,
    pub added_at: Option<DateTime<Utc>>,
    pub last_synced: DateTime<Utc>,
}

/// Input for upserting a playlist during provider sync.
#[derive(Debug, Clone)]
pub struct UpsertPlaylist {
    pub provider_playlist_id: String,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub owner_name: Option<String>,
    pub owner_id: Option<String>,
    pub is_public: Option<bool>,
    pub is_collaborative: bool,
    pub source_type: String,
    pub provider_track_count: Option<i32>,
    pub snapshot_id: Option<String>,
}

/// Input for inserting a track into a playlist.
#[derive(Debug, Clone)]
pub struct UpsertPlaylistTrack {
    pub provider_track_id: String,
    pub track_name: String,
    pub album_name: Option<String>,
    pub artist_name: String,
    pub position: i32,
    pub added_at: Option<DateTime<Utc>>,
}

/// Playlist summary with computed flagging data, returned by list queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistSummary {
    pub id: Uuid,
    pub provider: String,
    pub provider_playlist_id: String,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub owner_name: Option<String>,
    pub is_public: Option<bool>,
    pub source_type: String,
    pub total_tracks: i64,
    pub provider_track_count: Option<i32>,
    pub tracks_out_of_sync: bool,
    pub flagged_tracks: i64,
    pub clean_ratio: f64,
    pub grade: String,
    pub unique_artists: i64,
    pub flagged_artists: Vec<String>,
    pub last_synced: Option<DateTime<Utc>>,
    pub cover_images: Vec<String>,
}

/// Track within a playlist, enriched with offense status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistTrackWithStatus {
    pub id: Uuid,
    pub position: i32,
    pub provider_track_id: String,
    pub track_name: String,
    pub album_name: Option<String>,
    pub artist_id: Option<Uuid>,
    pub artist_name: String,
    pub artist_image_url: Option<String>,
    pub added_at: Option<DateTime<Utc>>,
    pub status: String,
}
