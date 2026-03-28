//! Spotify Enforcement API Handlers
//!
//! Endpoints for running, monitoring, and rolling back Spotify enforcement operations.
//! All enforcement and rollback actions call the Spotify Web API through `SpotifyService`.

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{
    AggressivenessLevel, AuthenticatedUser, BatchError, BatchProgress, BatchSummary,
    EnforcementOptions,
};
use crate::AppState;
use ndith_core::models::token_vault::{Connection, ConnectionStatus, StreamingProvider};
use ndith_services::{SpotifyConfig, SpotifyService, TokenVaultService};

/// Request to run Spotify enforcement
#[derive(Debug, Serialize, Deserialize)]
pub struct SpotifyRunEnforcementRequest {
    /// Aggressiveness level for enforcement
    #[serde(default = "default_aggressiveness")]
    pub aggressiveness: AggressivenessLevel,
    /// Block tracks featuring blocked artists
    #[serde(default = "default_true")]
    pub block_featuring: bool,
    /// Block collaborative tracks with blocked artists
    #[serde(default = "default_true")]
    pub block_collaborations: bool,
    /// Block tracks where blocked artist is only songwriter
    #[serde(default)]
    pub block_songwriter_only: bool,
    /// Preserve user-created playlists (don't modify them)
    #[serde(default = "default_true")]
    pub preserve_user_playlists: bool,
    /// Execute immediately or queue for background processing
    #[serde(default = "default_true")]
    pub execute_immediately: bool,
    /// Batch size for API calls
    #[serde(default = "default_batch_size")]
    pub batch_size: u32,
    /// Dry run mode (preview only, no changes)
    #[serde(default)]
    pub dry_run: bool,
    /// Optional idempotency key
    pub idempotency_key: Option<String>,
}

fn default_true() -> bool {
    true
}
fn default_batch_size() -> u32 {
    50
}
fn default_aggressiveness() -> AggressivenessLevel {
    AggressivenessLevel::Moderate
}

impl Default for SpotifyRunEnforcementRequest {
    fn default() -> Self {
        Self {
            aggressiveness: AggressivenessLevel::Moderate,
            block_featuring: true,
            block_collaborations: true,
            block_songwriter_only: false,
            preserve_user_playlists: true,
            execute_immediately: true,
            batch_size: 50,
            dry_run: false,
            idempotency_key: None,
        }
    }
}

impl From<SpotifyRunEnforcementRequest> for EnforcementOptions {
    fn from(req: SpotifyRunEnforcementRequest) -> Self {
        Self {
            aggressiveness: req.aggressiveness,
            block_collaborations: req.block_collaborations,
            block_featuring: req.block_featuring,
            block_songwriter_only: req.block_songwriter_only,
            preserve_user_playlists: req.preserve_user_playlists,
            dry_run: req.dry_run,
            providers: vec!["spotify".to_string()],
        }
    }
}

/// Response from Spotify enforcement run
#[derive(Debug, Serialize)]
pub struct SpotifyEnforcementRunResponse {
    pub batch_id: Uuid,
    pub status: String,
    pub summary: BatchSummary,
    pub songs_removed: usize,
    pub albums_removed: usize,
    pub artists_unfollowed: usize,
    pub playlist_tracks_removed: usize,
    pub errors_count: usize,
    pub message: String,
}

/// Response from Spotify enforcement preview
#[derive(Debug, Serialize)]
pub struct SpotifyEnforcementPreviewResponse {
    pub songs_to_remove: usize,
    pub albums_to_remove: usize,
    pub artists_to_unfollow: usize,
    pub playlist_tracks_to_remove: usize,
    pub total_library_songs: usize,
    pub total_library_albums: usize,
    pub total_followed_artists: usize,
    pub total_playlists: usize,
    pub estimated_duration_seconds: u64,
    pub blocked_content: SpotifyBlockedContentPreview,
}

/// Preview of blocked content in Spotify library
#[derive(Debug, Serialize)]
pub struct SpotifyBlockedContentPreview {
    pub songs: Vec<BlockedSongPreview>,
    pub albums: Vec<BlockedAlbumPreview>,
    pub artists: Vec<BlockedArtistPreview>,
    pub playlist_tracks: Vec<BlockedPlaylistTrackPreview>,
}

/// Preview info for a blocked song
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedSongPreview {
    pub track_id: String,
    pub name: String,
    pub artist_name: String,
    pub album_name: String,
    pub blocked_reason: String,
}

/// Preview info for a blocked album
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedAlbumPreview {
    pub album_id: String,
    pub name: String,
    pub artist_name: String,
    pub blocked_reason: String,
}

/// Preview info for a blocked artist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedArtistPreview {
    pub artist_id: String,
    pub name: String,
    pub blocked_reason: String,
}

/// Preview info for a blocked playlist track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedPlaylistTrackPreview {
    pub playlist_id: String,
    pub playlist_name: String,
    pub track_id: String,
    pub track_name: String,
    pub artist_name: String,
    pub blocked_reason: String,
}

/// Request for rollback operation
#[derive(Debug, Serialize, Deserialize)]
pub struct SpotifyRollbackRequest {
    /// Optional list of specific action IDs to rollback (if None, rollback entire batch)
    pub action_ids: Option<Vec<Uuid>>,
    /// Reason for rollback
    #[serde(default = "default_rollback_reason")]
    pub reason: String,
}

fn default_rollback_reason() -> String {
    "User requested rollback".to_string()
}

/// Response from enforcement history
#[derive(Debug, Serialize)]
pub struct SpotifyEnforcementHistoryResponse {
    pub batches: Vec<SpotifyEnforcementHistoryItem>,
    pub total_count: usize,
}

/// History item for Spotify enforcement
#[derive(Debug, Serialize)]
pub struct SpotifyEnforcementHistoryItem {
    pub batch_id: Uuid,
    pub status: String,
    pub dry_run: bool,
    pub songs_removed: u32,
    pub albums_removed: u32,
    pub artists_unfollowed: u32,
    pub playlist_tracks_removed: u32,
    pub errors_count: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub can_rollback: bool,
}

/// Spotify enforcement capabilities
#[derive(Debug, Serialize)]
pub struct SpotifyCapabilitiesResponse {
    pub library_modification: bool,
    pub playlist_modification: bool,
    pub unfollow_artists: bool,
    pub remove_saved_albums: bool,
    pub batch_operations: bool,
    pub rollback_support: bool,
    pub enforcement_effects: Vec<String>,
    pub limitations: Vec<String>,
}

/// Create a `SpotifyService` from app state.
fn create_spotify_service(
    state: &AppState,
) -> Result<(SpotifyService, TokenVaultService), AppError> {
    let spotify_config = SpotifyConfig::default();
    let token_vault = TokenVaultService::with_pool(state.db_pool.clone());
    let spotify_service = SpotifyService::new(
        spotify_config,
        Arc::new(TokenVaultService::with_pool(state.db_pool.clone())),
    )
    .map_err(|e| AppError::Internal {
        message: Some(format!("Failed to initialize Spotify service: {}", e)),
    })?;
    Ok((spotify_service, token_vault))
}

/// Fetch the user's active Spotify connection from TokenVault.
async fn get_spotify_connection(
    token_vault: &TokenVaultService,
    user_id: Uuid,
) -> Result<Connection, AppError> {
    let connections = token_vault.get_user_connections(user_id).await;
    connections
        .into_iter()
        .find(|c| c.provider == StreamingProvider::Spotify && c.status == ConnectionStatus::Active)
        .ok_or_else(|| {
            AppError::InvalidRequestFormat(
                "No active Spotify connection. Please connect your Spotify account first."
                    .to_string(),
            )
        })
}

/// Get Spotify artist IDs on the user's blocklist from the `external_ids` JSONB column.
async fn get_blocked_spotify_ids(
    state: &AppState,
    user_id: Uuid,
) -> Result<HashSet<String>, AppError> {
    let rows: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT DISTINCT a.external_ids->>'spotify' AS spotify_id
        FROM user_artist_blocks uab
        JOIN artists a ON uab.artist_id = a.id
        WHERE uab.user_id = $1
          AND a.external_ids->>'spotify' IS NOT NULL

        UNION

        SELECT DISTINCT a.external_ids->>'spotify' AS spotify_id
        FROM category_subscriptions cs
        JOIN artist_offenses ao ON ao.category = cs.category
        JOIN artists a ON ao.artist_id = a.id
        WHERE cs.user_id = $1
          AND a.external_ids->>'spotify' IS NOT NULL
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal {
        message: Some(e.to_string()),
    })?;

    Ok(rows.into_iter().map(|r| r.0).collect())
}

/// Get the canonical name set for blocked artists (used for matching by name).
async fn get_blocked_artist_names(
    state: &AppState,
    user_id: Uuid,
) -> Result<HashSet<String>, AppError> {
    let rows: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT DISTINCT LOWER(a.canonical_name)
        FROM user_artist_blocks uab
        JOIN artists a ON uab.artist_id = a.id
        WHERE uab.user_id = $1

        UNION

        SELECT DISTINCT LOWER(a.canonical_name)
        FROM category_subscriptions cs
        JOIN artist_offenses ao ON ao.category = cs.category
        JOIN artists a ON ao.artist_id = a.id
        WHERE cs.user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal {
        message: Some(e.to_string()),
    })?;

    Ok(rows.into_iter().map(|r| r.0).collect())
}

/// Check if a track is by a blocked artist (by Spotify ID or canonical name).
fn is_blocked_artist(
    artist_id: &str,
    artist_name: &str,
    blocked_ids: &HashSet<String>,
    blocked_names: &HashSet<String>,
) -> bool {
    blocked_ids.contains(artist_id) || blocked_names.contains(&artist_name.to_lowercase())
}

/// Run Spotify enforcement
///
/// POST /api/v1/enforcement/spotify/run
pub async fn run_spotify_enforcement(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<SpotifyRunEnforcementRequest>,
) -> Result<Json<SpotifyEnforcementRunResponse>, AppError> {
    let user_id = user.id;

    // Get blocked artist IDs from DNP list
    let blocked_artists = get_blocked_artist_ids(&state, user_id).await?;

    if blocked_artists.is_empty() {
        return Ok(Json(SpotifyEnforcementRunResponse {
            batch_id: Uuid::nil(),
            status: "skipped".to_string(),
            summary: BatchSummary::default(),
            songs_removed: 0,
            albums_removed: 0,
            artists_unfollowed: 0,
            playlist_tracks_removed: 0,
            errors_count: 0,
            message: "No blocked artists found in DNP list".to_string(),
        }));
    }

    let is_dry_run = request.dry_run;
    let preserve_user_playlists = request.preserve_user_playlists;
    let options: EnforcementOptions = request.into();

    // Get the user's Spotify connection and create the service
    let (spotify_service, token_vault) = create_spotify_service(&state)?;
    let connection = get_spotify_connection(&token_vault, user_id).await?;

    // Get blocked Spotify artist IDs and names for matching
    let blocked_spotify_ids = get_blocked_spotify_ids(&state, user_id).await?;
    let blocked_artist_names = get_blocked_artist_names(&state, user_id).await?;

    // Create action batch in database
    let batch_id = Uuid::new_v4();
    let idempotency_key = if options.dry_run {
        format!(
            "dryrun_{}_{}",
            user_id,
            chrono::Utc::now().timestamp_millis()
        )
    } else {
        format!(
            "enforce_{}_{}",
            user_id,
            chrono::Utc::now().timestamp_millis()
        )
    };

    let options_json = serde_json::to_value(&options).unwrap_or_default();

    sqlx::query(
        r#"
        INSERT INTO action_batches (id, user_id, provider, idempotency_key, dry_run, status, options, summary, created_at)
        VALUES ($1, $2, 'spotify', $3, $4, 'in_progress', $5, '{}', NOW())
        "#,
    )
    .bind(batch_id)
    .bind(user_id)
    .bind(&idempotency_key)
    .bind(is_dry_run)
    .bind(&options_json)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal {
        message: Some(e.to_string()),
    })?;

    // Scan the library
    let scan_result = spotify_service
        .scan_library(&connection)
        .await
        .map_err(|e| {
            AppError::ExternalServiceError(format!("Spotify library scan failed: {}", e))
        })?;

    let library = &scan_result.library;

    // ---- Collect items to remove ----
    let mut songs_to_remove: Vec<String> = Vec::new(); // track IDs
    let mut albums_to_remove: Vec<String> = Vec::new(); // album IDs
    let mut artists_to_unfollow: Vec<String> = Vec::new(); // artist IDs
                                                           // playlist_id -> Vec<(track_uri, track_id, track_name, artist_name)>
    let mut playlist_tracks_to_remove: std::collections::HashMap<
        String,
        Vec<(String, String, String, String)>,
    > = std::collections::HashMap::new();

    // Liked songs
    for saved_track in &library.liked_songs {
        let track = &saved_track.track;
        for artist in &track.artists {
            if is_blocked_artist(
                &artist.id,
                &artist.name,
                &blocked_spotify_ids,
                &blocked_artist_names,
            ) {
                songs_to_remove.push(track.id.clone());
                break;
            }
        }
    }

    // Saved albums
    for album in &library.saved_albums {
        for artist in &album.artists {
            if is_blocked_artist(
                &artist.id,
                &artist.name,
                &blocked_spotify_ids,
                &blocked_artist_names,
            ) {
                albums_to_remove.push(album.id.clone());
                break;
            }
        }
    }

    // Followed artists
    for followed in &library.followed_artists {
        let artist = &followed.artist;
        if is_blocked_artist(
            &artist.id,
            &artist.name,
            &blocked_spotify_ids,
            &blocked_artist_names,
        ) {
            artists_to_unfollow.push(artist.id.clone());
        }
    }

    // Playlist tracks (skip user-owned playlists if preserve_user_playlists is set)
    for playlist in &library.playlists {
        if preserve_user_playlists && playlist.owner.id == library.spotify_user_id {
            continue;
        }
        if let Some(ref items) = playlist.tracks.items {
            for playlist_track in items {
                if let Some(ref track) = playlist_track.track {
                    for artist in &track.artists {
                        if is_blocked_artist(
                            &artist.id,
                            &artist.name,
                            &blocked_spotify_ids,
                            &blocked_artist_names,
                        ) {
                            let uri = format!("spotify:track:{}", track.id);
                            let primary_artist = track
                                .artists
                                .first()
                                .map(|a| a.name.clone())
                                .unwrap_or_default();
                            playlist_tracks_to_remove
                                .entry(playlist.id.clone())
                                .or_default()
                                .push((uri, track.id.clone(), track.name.clone(), primary_artist));
                            break;
                        }
                    }
                }
            }
        }
    }

    let total_songs = songs_to_remove.len();
    let total_albums = albums_to_remove.len();
    let total_artists = artists_to_unfollow.len();
    let total_playlist_tracks: usize = playlist_tracks_to_remove.values().map(|v| v.len()).sum();
    let total_actions = total_songs + total_albums + total_artists + total_playlist_tracks;
    let mut completed_actions: u32 = 0;
    let mut failed_actions: u32 = 0;
    let mut errors: Vec<BatchError> = Vec::new();

    if is_dry_run {
        // Dry run -- record but don't execute
        let summary = serde_json::json!({
            "total_actions": total_actions,
            "completed_actions": 0,
            "failed_actions": 0,
            "skipped_actions": 0,
            "execution_time_ms": 0,
            "api_calls_made": 0,
            "rate_limit_delays_ms": 0,
            "errors": []
        });
        sqlx::query(
            "UPDATE action_batches SET status = 'completed', summary = $1, completed_at = NOW() WHERE id = $2",
        )
        .bind(&summary)
        .bind(batch_id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal { message: Some(e.to_string()) })?;

        return Ok(Json(SpotifyEnforcementRunResponse {
            batch_id,
            status: "dry_run".to_string(),
            summary: BatchSummary {
                total_actions: total_actions as u32,
                completed_actions: 0,
                failed_actions: 0,
                skipped_actions: 0,
                execution_time_ms: 0,
                api_calls_made: 0,
                rate_limit_delays_ms: 0,
                errors: Vec::new(),
            },
            songs_removed: total_songs,
            albums_removed: total_albums,
            artists_unfollowed: total_artists,
            playlist_tracks_removed: total_playlist_tracks,
            errors_count: 0,
            message: format!(
                "Dry run complete. Would remove {} liked songs, {} albums, unfollow {} artists, remove {} playlist tracks.",
                total_songs, total_albums, total_artists, total_playlist_tracks
            ),
        }));
    }

    // ---- Execute enforcement (non-dry-run) ----

    // 1. Remove liked songs in batches of 50
    for chunk in songs_to_remove.chunks(50) {
        let chunk_vec: Vec<String> = chunk.to_vec();
        // Record action items
        for track_id in &chunk_vec {
            let action_id = Uuid::new_v4();
            let before_state = serde_json::json!({ "track_id": track_id, "was_liked": true });
            let idempotency = format!("{}_{}_remove_liked_{}", batch_id, user_id, track_id);
            let _ = sqlx::query(
                r#"INSERT INTO action_items (id, batch_id, entity_type, entity_id, action, idempotency_key, before_state, status, created_at)
                   VALUES ($1, $2, 'track', $3, 'remove_liked_song', $4, $5, 'pending', NOW())"#,
            )
            .bind(action_id)
            .bind(batch_id)
            .bind(track_id)
            .bind(&idempotency)
            .bind(&before_state)
            .execute(&state.db_pool)
            .await;
        }

        match spotify_service
            .remove_liked_songs_batch(&connection, &chunk_vec)
            .await
        {
            Ok(()) => {
                completed_actions += chunk_vec.len() as u32;
                // Mark actions completed
                let _ = sqlx::query(
                    "UPDATE action_items SET status = 'completed', after_state = '{\"removed\": true}' WHERE batch_id = $1 AND action = 'remove_liked_song' AND status = 'pending' AND entity_id = ANY($2)",
                )
                .bind(batch_id)
                .bind(&chunk_vec)
                .execute(&state.db_pool)
                .await;
            }
            Err(e) => {
                let err_msg = e.to_string();
                tracing::error!("Failed to remove liked songs batch: {}", err_msg);
                failed_actions += chunk_vec.len() as u32;
                errors.push(BatchError {
                    action_id: batch_id,
                    entity_type: "track".to_string(),
                    entity_id: chunk_vec.first().cloned().unwrap_or_default(),
                    error_code: "REMOVE_LIKED_SONGS_FAILED".to_string(),
                    error_message: err_msg.clone(),
                    retry_count: 0,
                    is_recoverable: true,
                });
                let _ = sqlx::query(
                    "UPDATE action_items SET status = 'failed', error_message = $1 WHERE batch_id = $2 AND action = 'remove_liked_song' AND status = 'pending' AND entity_id = ANY($3)",
                )
                .bind(&err_msg)
                .bind(batch_id)
                .bind(&chunk_vec)
                .execute(&state.db_pool)
                .await;
            }
        }
    }

    // 2. Remove saved albums in batches of 50
    for chunk in albums_to_remove.chunks(50) {
        let chunk_vec: Vec<String> = chunk.to_vec();
        for album_id in &chunk_vec {
            let action_id = Uuid::new_v4();
            let before_state = serde_json::json!({ "album_id": album_id, "was_saved": true });
            let idempotency = format!("{}_{}_remove_album_{}", batch_id, user_id, album_id);
            let _ = sqlx::query(
                r#"INSERT INTO action_items (id, batch_id, entity_type, entity_id, action, idempotency_key, before_state, status, created_at)
                   VALUES ($1, $2, 'album', $3, 'remove_saved_album', $4, $5, 'pending', NOW())"#,
            )
            .bind(action_id)
            .bind(batch_id)
            .bind(album_id)
            .bind(&idempotency)
            .bind(&before_state)
            .execute(&state.db_pool)
            .await;
        }

        match spotify_service
            .remove_saved_albums_batch(&connection, &chunk_vec)
            .await
        {
            Ok(()) => {
                completed_actions += chunk_vec.len() as u32;
                let _ = sqlx::query(
                    "UPDATE action_items SET status = 'completed', after_state = '{\"removed\": true}' WHERE batch_id = $1 AND action = 'remove_saved_album' AND status = 'pending' AND entity_id = ANY($2)",
                )
                .bind(batch_id)
                .bind(&chunk_vec)
                .execute(&state.db_pool)
                .await;
            }
            Err(e) => {
                let err_msg = e.to_string();
                tracing::error!("Failed to remove saved albums batch: {}", err_msg);
                failed_actions += chunk_vec.len() as u32;
                errors.push(BatchError {
                    action_id: batch_id,
                    entity_type: "album".to_string(),
                    entity_id: chunk_vec.first().cloned().unwrap_or_default(),
                    error_code: "REMOVE_SAVED_ALBUMS_FAILED".to_string(),
                    error_message: err_msg.clone(),
                    retry_count: 0,
                    is_recoverable: true,
                });
                let _ = sqlx::query(
                    "UPDATE action_items SET status = 'failed', error_message = $1 WHERE batch_id = $2 AND action = 'remove_saved_album' AND status = 'pending' AND entity_id = ANY($3)",
                )
                .bind(&err_msg)
                .bind(batch_id)
                .bind(&chunk_vec)
                .execute(&state.db_pool)
                .await;
            }
        }
    }

    // 3. Unfollow blocked artists in batches of 50
    for chunk in artists_to_unfollow.chunks(50) {
        let chunk_vec: Vec<String> = chunk.to_vec();
        for artist_id in &chunk_vec {
            let action_id = Uuid::new_v4();
            let before_state = serde_json::json!({ "artist_id": artist_id, "was_followed": true });
            let idempotency = format!("{}_{}_unfollow_{}", batch_id, user_id, artist_id);
            let _ = sqlx::query(
                r#"INSERT INTO action_items (id, batch_id, entity_type, entity_id, action, idempotency_key, before_state, status, created_at)
                   VALUES ($1, $2, 'artist', $3, 'unfollow_artist', $4, $5, 'pending', NOW())"#,
            )
            .bind(action_id)
            .bind(batch_id)
            .bind(artist_id)
            .bind(&idempotency)
            .bind(&before_state)
            .execute(&state.db_pool)
            .await;
        }

        match spotify_service
            .unfollow_artists_batch(&connection, &chunk_vec)
            .await
        {
            Ok(()) => {
                completed_actions += chunk_vec.len() as u32;
                let _ = sqlx::query(
                    "UPDATE action_items SET status = 'completed', after_state = '{\"unfollowed\": true}' WHERE batch_id = $1 AND action = 'unfollow_artist' AND status = 'pending' AND entity_id = ANY($2)",
                )
                .bind(batch_id)
                .bind(&chunk_vec)
                .execute(&state.db_pool)
                .await;
            }
            Err(e) => {
                let err_msg = e.to_string();
                tracing::error!("Failed to unfollow artists batch: {}", err_msg);
                failed_actions += chunk_vec.len() as u32;
                errors.push(BatchError {
                    action_id: batch_id,
                    entity_type: "artist".to_string(),
                    entity_id: chunk_vec.first().cloned().unwrap_or_default(),
                    error_code: "UNFOLLOW_ARTISTS_FAILED".to_string(),
                    error_message: err_msg.clone(),
                    retry_count: 0,
                    is_recoverable: true,
                });
                let _ = sqlx::query(
                    "UPDATE action_items SET status = 'failed', error_message = $1 WHERE batch_id = $2 AND action = 'unfollow_artist' AND status = 'pending' AND entity_id = ANY($3)",
                )
                .bind(&err_msg)
                .bind(batch_id)
                .bind(&chunk_vec)
                .execute(&state.db_pool)
                .await;
            }
        }
    }

    // 4. Remove playlist tracks (batches of 100 per playlist)
    for (playlist_id, tracks) in &playlist_tracks_to_remove {
        // Build Spotify API track objects: [{"uri": "spotify:track:..."}]
        let track_objects: Vec<serde_json::Value> = tracks
            .iter()
            .map(|(uri, _id, _name, _artist)| serde_json::json!({ "uri": uri }))
            .collect();

        for (uri, track_id, track_name, artist_name) in tracks {
            let action_id = Uuid::new_v4();
            let before_state = serde_json::json!({
                "playlist_id": playlist_id,
                "track_id": track_id,
                "track_uri": uri,
                "track_name": track_name,
                "artist_name": artist_name,
            });
            let idempotency = format!(
                "{}_{}_{}_remove_playlist_track_{}",
                batch_id, user_id, playlist_id, track_id
            );
            let _ = sqlx::query(
                r#"INSERT INTO action_items (id, batch_id, entity_type, entity_id, action, idempotency_key, before_state, status, created_at)
                   VALUES ($1, $2, 'track', $3, 'remove_playlist_track', $4, $5, 'pending', NOW())"#,
            )
            .bind(action_id)
            .bind(batch_id)
            .bind(track_id)
            .bind(&idempotency)
            .bind(&before_state)
            .execute(&state.db_pool)
            .await;
        }

        // Spotify allows up to 100 tracks per playlist modification request
        for chunk in track_objects.chunks(100) {
            let chunk_vec: Vec<serde_json::Value> = chunk.to_vec();
            match spotify_service
                .remove_playlist_tracks_batch(&connection, playlist_id, &chunk_vec)
                .await
            {
                Ok(_snapshot_id) => {
                    completed_actions += chunk_vec.len() as u32;
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    tracing::error!(
                        "Failed to remove playlist tracks from {}: {}",
                        playlist_id,
                        err_msg
                    );
                    failed_actions += chunk_vec.len() as u32;
                    errors.push(BatchError {
                        action_id: batch_id,
                        entity_type: "playlist_track".to_string(),
                        entity_id: playlist_id.to_string(),
                        error_code: "REMOVE_PLAYLIST_TRACKS_FAILED".to_string(),
                        error_message: err_msg.clone(),
                        retry_count: 0,
                        is_recoverable: true,
                    });
                }
            }
        }

        // Mark all action items for this playlist
        let track_ids: Vec<String> = tracks.iter().map(|(_, id, _, _)| id.clone()).collect();
        if failed_actions == 0 {
            let _ = sqlx::query(
                "UPDATE action_items SET status = 'completed', after_state = '{\"removed\": true}' WHERE batch_id = $1 AND action = 'remove_playlist_track' AND status = 'pending' AND entity_id = ANY($2)",
            )
            .bind(batch_id)
            .bind(&track_ids)
            .execute(&state.db_pool)
            .await;
        }
    }

    // Update batch with final summary
    let batch_status = if failed_actions == 0 {
        "completed"
    } else if completed_actions > 0 {
        "partially_completed"
    } else {
        "failed"
    };

    let summary_json = serde_json::json!({
        "total_actions": total_actions,
        "completed_actions": completed_actions,
        "failed_actions": failed_actions,
        "skipped_actions": 0,
        "execution_time_ms": 0,
        "api_calls_made": 0,
        "rate_limit_delays_ms": 0,
        "errors": errors
    });

    sqlx::query(
        "UPDATE action_batches SET status = $1, summary = $2, completed_at = NOW() WHERE id = $3",
    )
    .bind(batch_status)
    .bind(&summary_json)
    .bind(batch_id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal {
        message: Some(e.to_string()),
    })?;

    let message = format!(
        "Spotify enforcement {}: removed {} liked songs, {} albums, unfollowed {} artists, removed {} playlist tracks. {} errors.",
        batch_status, total_songs, total_albums, total_artists, total_playlist_tracks, errors.len()
    );

    Ok(Json(SpotifyEnforcementRunResponse {
        batch_id,
        status: batch_status.to_string(),
        summary: BatchSummary {
            total_actions: total_actions as u32,
            completed_actions,
            failed_actions,
            skipped_actions: 0,
            execution_time_ms: 0,
            api_calls_made: 0,
            rate_limit_delays_ms: 0,
            errors,
        },
        songs_removed: total_songs,
        albums_removed: total_albums,
        artists_unfollowed: total_artists,
        playlist_tracks_removed: total_playlist_tracks,
        errors_count: failed_actions as usize,
        message,
    }))
}

/// Preview Spotify enforcement (dry run)
///
/// POST /api/v1/enforcement/spotify/preview
pub async fn preview_spotify_enforcement(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<SpotifyEnforcementPreviewResponse>, AppError> {
    let user_id = user.id;

    // Get blocked artists with their details
    let blocked_artist_details = get_blocked_artists_with_details(&state, user_id).await?;

    if blocked_artist_details.is_empty() {
        return Ok(Json(SpotifyEnforcementPreviewResponse {
            songs_to_remove: 0,
            albums_to_remove: 0,
            artists_to_unfollow: 0,
            playlist_tracks_to_remove: 0,
            total_library_songs: 0,
            total_library_albums: 0,
            total_followed_artists: 0,
            total_playlists: 0,
            estimated_duration_seconds: 0,
            blocked_content: SpotifyBlockedContentPreview {
                songs: Vec::new(),
                albums: Vec::new(),
                artists: Vec::new(),
                playlist_tracks: Vec::new(),
            },
        }));
    }

    // Get the user's Spotify connection and scan the library
    let (spotify_service, token_vault) = create_spotify_service(&state)?;
    let connection = get_spotify_connection(&token_vault, user_id).await?;

    let blocked_spotify_ids = get_blocked_spotify_ids(&state, user_id).await?;
    let blocked_artist_names = get_blocked_artist_names(&state, user_id).await?;

    let scan_result = spotify_service
        .scan_library(&connection)
        .await
        .map_err(|e| {
            AppError::ExternalServiceError(format!("Spotify library scan failed: {}", e))
        })?;

    let library = &scan_result.library;

    // Match blocked content against actual library
    let mut blocked_songs: Vec<BlockedSongPreview> = Vec::new();
    let mut blocked_albums: Vec<BlockedAlbumPreview> = Vec::new();
    let mut blocked_artists_preview: Vec<BlockedArtistPreview> = Vec::new();
    let mut blocked_playlist_tracks: Vec<BlockedPlaylistTrackPreview> = Vec::new();

    // Scan liked songs
    for saved_track in &library.liked_songs {
        let track = &saved_track.track;
        for artist in &track.artists {
            if is_blocked_artist(
                &artist.id,
                &artist.name,
                &blocked_spotify_ids,
                &blocked_artist_names,
            ) {
                blocked_songs.push(BlockedSongPreview {
                    track_id: track.id.clone(),
                    name: track.name.clone(),
                    artist_name: artist.name.clone(),
                    album_name: track.album.name.clone(),
                    blocked_reason: format!("Artist '{}' is on your DNP list", artist.name),
                });
                break;
            }
        }
    }

    // Scan saved albums
    for album in &library.saved_albums {
        for artist in &album.artists {
            if is_blocked_artist(
                &artist.id,
                &artist.name,
                &blocked_spotify_ids,
                &blocked_artist_names,
            ) {
                blocked_albums.push(BlockedAlbumPreview {
                    album_id: album.id.clone(),
                    name: album.name.clone(),
                    artist_name: artist.name.clone(),
                    blocked_reason: format!("Artist '{}' is on your DNP list", artist.name),
                });
                break;
            }
        }
    }

    // Scan followed artists
    for followed in &library.followed_artists {
        let artist = &followed.artist;
        if is_blocked_artist(
            &artist.id,
            &artist.name,
            &blocked_spotify_ids,
            &blocked_artist_names,
        ) {
            // Find the reason from blocked_artist_details
            let reason = blocked_artist_details
                .iter()
                .find(|a| a.name.to_lowercase() == artist.name.to_lowercase())
                .map(|a| a.reason.clone())
                .unwrap_or_else(|| "On your DNP list".to_string());
            blocked_artists_preview.push(BlockedArtistPreview {
                artist_id: artist.id.clone(),
                name: artist.name.clone(),
                blocked_reason: reason,
            });
        }
    }

    // Scan playlist tracks
    for playlist in &library.playlists {
        if let Some(ref items) = playlist.tracks.items {
            for playlist_track in items {
                if let Some(ref track) = playlist_track.track {
                    for artist in &track.artists {
                        if is_blocked_artist(
                            &artist.id,
                            &artist.name,
                            &blocked_spotify_ids,
                            &blocked_artist_names,
                        ) {
                            blocked_playlist_tracks.push(BlockedPlaylistTrackPreview {
                                playlist_id: playlist.id.clone(),
                                playlist_name: playlist.name.clone(),
                                track_id: track.id.clone(),
                                track_name: track.name.clone(),
                                artist_name: artist.name.clone(),
                                blocked_reason: format!(
                                    "Artist '{}' is on your DNP list",
                                    artist.name
                                ),
                            });
                            break;
                        }
                    }
                }
            }
        }
    }

    let total_items = blocked_songs.len()
        + blocked_albums.len()
        + blocked_artists_preview.len()
        + blocked_playlist_tracks.len();
    // Estimate ~0.75 seconds per item for API calls
    let estimated_duration = (total_items as u64 * 750) / 1000 + 1;

    Ok(Json(SpotifyEnforcementPreviewResponse {
        songs_to_remove: blocked_songs.len(),
        albums_to_remove: blocked_albums.len(),
        artists_to_unfollow: blocked_artists_preview.len(),
        playlist_tracks_to_remove: blocked_playlist_tracks.len(),
        total_library_songs: scan_result.counts.liked_songs_count as usize,
        total_library_albums: scan_result.counts.saved_albums_count as usize,
        total_followed_artists: scan_result.counts.followed_artists_count as usize,
        total_playlists: scan_result.counts.playlists_count as usize,
        estimated_duration_seconds: estimated_duration,
        blocked_content: SpotifyBlockedContentPreview {
            songs: blocked_songs,
            albums: blocked_albums,
            artists: blocked_artists_preview,
            playlist_tracks: blocked_playlist_tracks,
        },
    }))
}

/// Response from rollback operation
#[derive(Debug, Serialize)]
pub struct RollbackResponse {
    pub rollback_batch_id: Uuid,
    pub original_batch_id: Uuid,
    pub status: String,
    pub actions_rolled_back: u32,
    pub actions_failed: u32,
    pub actions_skipped: u32,
    pub job_id: Option<Uuid>,
    pub message: String,
}

/// Rollback a Spotify enforcement batch
///
/// POST /api/v1/enforcement/spotify/rollback/{batch_id}
pub async fn rollback_spotify_enforcement(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(batch_id): Path<Uuid>,
    Json(request): Json<SpotifyRollbackRequest>,
) -> Result<Json<RollbackResponse>, AppError> {
    let user_id = user.id;

    // Verify the batch exists and belongs to this user
    let batch_row: Option<BatchVerificationRow> = sqlx::query_as(
        r#"
        SELECT id, user_id, status, dry_run
        FROM action_batches
        WHERE id = $1 AND provider = 'spotify'
        "#,
    )
    .bind(batch_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal {
        message: Some(e.to_string()),
    })?;

    let batch = batch_row.ok_or_else(|| AppError::NotFound {
        resource: format!("Enforcement batch {}", batch_id),
    })?;

    // Verify ownership
    if batch.user_id != user_id {
        return Err(AppError::InsufficientPermissions);
    }

    // Check if batch can be rolled back (must be completed or partially_completed)
    if batch.status != "completed" && batch.status != "partially_completed" {
        return Err(AppError::InvalidFieldValue {
            field: "status".to_string(),
            message: format!(
                "Cannot rollback batch with status '{}'. Only 'completed' or 'partially_completed' batches can be rolled back.",
                batch.status
            ),
        });
    }

    // Cannot rollback a dry run batch
    if batch.dry_run {
        return Err(AppError::InvalidFieldValue {
            field: "dry_run".to_string(),
            message: "Cannot rollback a dry run batch - no actual changes were made".to_string(),
        });
    }

    // Get the user's Spotify connection for API calls
    let (spotify_service, token_vault) = create_spotify_service(&state)?;
    let connection = get_spotify_connection(&token_vault, user_id).await?;

    // Get actions that can be rolled back
    let rollback_result = execute_batch_rollback(
        &state.db_pool,
        batch_id,
        user_id,
        request.action_ids.as_deref(),
        &request.reason,
        &spotify_service,
        &connection,
    )
    .await?;

    Ok(Json(rollback_result))
}

/// Verification row for batch ownership check
#[derive(sqlx::FromRow)]
struct BatchVerificationRow {
    #[allow(dead_code)]
    id: Uuid,
    user_id: Uuid,
    status: String,
    dry_run: bool,
}

/// Execute the rollback operation
async fn execute_batch_rollback(
    db_pool: &sqlx::PgPool,
    original_batch_id: Uuid,
    user_id: Uuid,
    action_ids: Option<&[Uuid]>,
    reason: &str,
    spotify_service: &SpotifyService,
    connection: &Connection,
) -> Result<RollbackResponse, AppError> {
    // Get actions to rollback
    let actions_to_rollback: Vec<RollbackableAction> = if let Some(ids) = action_ids {
        sqlx::query_as(
            r#"
            SELECT id, entity_type, entity_id, action, before_state, after_state
            FROM action_items
            WHERE batch_id = $1
              AND status = 'completed'
              AND before_state IS NOT NULL
              AND id = ANY($2)
            ORDER BY created_at DESC
            "#,
        )
        .bind(original_batch_id)
        .bind(ids)
        .fetch_all(db_pool)
        .await
    } else {
        sqlx::query_as(
            r#"
            SELECT id, entity_type, entity_id, action, before_state, after_state
            FROM action_items
            WHERE batch_id = $1
              AND status = 'completed'
              AND before_state IS NOT NULL
            ORDER BY created_at DESC
            "#,
        )
        .bind(original_batch_id)
        .fetch_all(db_pool)
        .await
    }
    .map_err(|e| AppError::Internal {
        message: Some(e.to_string()),
    })?;

    if actions_to_rollback.is_empty() {
        return Ok(RollbackResponse {
            rollback_batch_id: Uuid::nil(),
            original_batch_id,
            status: "skipped".to_string(),
            actions_rolled_back: 0,
            actions_failed: 0,
            actions_skipped: 0,
            job_id: None,
            message: "No rollback-eligible actions found. Actions must be completed and have before_state saved.".to_string(),
        });
    }

    // Create rollback batch
    let rollback_batch_id = Uuid::new_v4();
    let rollback_idempotency_key = format!(
        "rollback_{}_{}",
        original_batch_id,
        chrono::Utc::now().timestamp_millis()
    );

    let options = serde_json::json!({
        "rollback_of": original_batch_id,
        "reason": reason,
        "is_rollback": true,
        "original_batch_id": original_batch_id
    });

    sqlx::query(
        r#"
        INSERT INTO action_batches (id, user_id, provider, idempotency_key, dry_run, status, options, summary, created_at)
        VALUES ($1, $2, 'spotify', $3, false, 'in_progress', $4, '{}', NOW())
        "#,
    )
    .bind(rollback_batch_id)
    .bind(user_id)
    .bind(&rollback_idempotency_key)
    .bind(&options)
    .execute(db_pool)
    .await
    .map_err(|e| AppError::Internal {
        message: Some(e.to_string()),
    })?;

    // Create rollback action items and execute rollback
    let mut actions_rolled_back: u32 = 0;
    let mut actions_failed: u32 = 0;
    let mut actions_skipped: u32 = 0;
    let mut errors: Vec<serde_json::Value> = Vec::new();

    for action in &actions_to_rollback {
        let rollback_action_type = get_rollback_action_type(&action.action);

        if let Some(rollback_type) = rollback_action_type {
            let rollback_action_id = Uuid::new_v4();
            let rollback_idempotency = format!(
                "{}_{}_{}_{}",
                rollback_batch_id, action.entity_type, action.entity_id, rollback_type
            );

            // Create rollback action item
            sqlx::query(
                r#"
                INSERT INTO action_items
                    (id, batch_id, entity_type, entity_id, action, idempotency_key, before_state, status, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending', NOW())
                "#,
            )
            .bind(rollback_action_id)
            .bind(rollback_batch_id)
            .bind(&action.entity_type)
            .bind(&action.entity_id)
            .bind(&rollback_type)
            .bind(&rollback_idempotency)
            .bind(&action.after_state) // The after_state of original becomes before_state of rollback
            .execute(db_pool)
            .await
            .map_err(|e| AppError::Internal {
                message: Some(e.to_string()),
            })?;

            // Execute rollback via Spotify API
            let rollback_success = execute_single_rollback_action(
                spotify_service,
                connection,
                &action.entity_type,
                &action.entity_id,
                &rollback_type,
                &action.before_state,
            )
            .await;

            if rollback_success {
                // Mark rollback action as completed
                let after_state = serde_json::json!({
                    "rolled_back_at": chrono::Utc::now().to_rfc3339(),
                    "original_action_id": action.id
                });

                sqlx::query(
                    r#"
                    UPDATE action_items
                    SET status = 'completed', after_state = $1
                    WHERE id = $2
                    "#,
                )
                .bind(&after_state)
                .bind(rollback_action_id)
                .execute(db_pool)
                .await
                .map_err(|e| AppError::Internal {
                    message: Some(e.to_string()),
                })?;

                // Mark original action as rolled back
                sqlx::query(
                    r#"
                    UPDATE action_items
                    SET status = 'rolled_back'
                    WHERE id = $1
                    "#,
                )
                .bind(action.id)
                .execute(db_pool)
                .await
                .map_err(|e| AppError::Internal {
                    message: Some(e.to_string()),
                })?;

                actions_rolled_back += 1;
            } else {
                // Mark rollback action as failed
                sqlx::query(
                    r#"
                    UPDATE action_items
                    SET status = 'failed', error_message = 'Rollback execution failed'
                    WHERE id = $1
                    "#,
                )
                .bind(rollback_action_id)
                .execute(db_pool)
                .await
                .map_err(|e| AppError::Internal {
                    message: Some(e.to_string()),
                })?;

                errors.push(serde_json::json!({
                    "action_id": action.id,
                    "entity_type": action.entity_type,
                    "entity_id": action.entity_id,
                    "error": "Rollback execution failed"
                }));
                actions_failed += 1;
            }
        } else {
            actions_skipped += 1;
        }
    }

    // Update rollback batch with final summary
    let batch_status = if actions_failed == 0 {
        "completed"
    } else if actions_rolled_back > 0 {
        "partially_completed"
    } else {
        "failed"
    };

    let summary = serde_json::json!({
        "total_actions": actions_to_rollback.len(),
        "completed_actions": actions_rolled_back,
        "failed_actions": actions_failed,
        "skipped_actions": actions_skipped,
        "errors": errors
    });

    sqlx::query(
        r#"
        UPDATE action_batches
        SET status = $1, summary = $2, completed_at = NOW()
        WHERE id = $3
        "#,
    )
    .bind(batch_status)
    .bind(&summary)
    .bind(rollback_batch_id)
    .execute(db_pool)
    .await
    .map_err(|e| AppError::Internal {
        message: Some(e.to_string()),
    })?;

    let message = if actions_failed == 0 {
        format!(
            "Successfully rolled back {} actions from batch {}",
            actions_rolled_back, original_batch_id
        )
    } else {
        format!(
            "Partially rolled back batch {}. {} succeeded, {} failed, {} skipped",
            original_batch_id, actions_rolled_back, actions_failed, actions_skipped
        )
    };

    Ok(RollbackResponse {
        rollback_batch_id,
        original_batch_id,
        status: batch_status.to_string(),
        actions_rolled_back,
        actions_failed,
        actions_skipped,
        job_id: None, // Could be used for background job tracking
        message,
    })
}

/// Action item that can be rolled back
#[derive(sqlx::FromRow)]
struct RollbackableAction {
    id: Uuid,
    entity_type: String,
    entity_id: String,
    action: String,
    before_state: Option<serde_json::Value>,
    after_state: Option<serde_json::Value>,
}

/// Get the reverse action type for rollback
fn get_rollback_action_type(original_action: &str) -> Option<String> {
    match original_action {
        "remove_liked_song" => Some("add_liked_song".to_string()),
        "unfollow_artist" => Some("follow_artist".to_string()),
        "remove_playlist_track" => Some("add_playlist_track".to_string()),
        "remove_saved_album" => Some("add_saved_album".to_string()),
        _ => None,
    }
}

/// Execute a single rollback action by calling the Spotify API.
///
/// The `SpotifyService` and `Connection` are passed from the outer rollback loop
/// so we create them once and reuse them.
async fn execute_single_rollback_action(
    spotify_service: &SpotifyService,
    connection: &Connection,
    entity_type: &str,
    entity_id: &str,
    rollback_action: &str,
    before_state: &Option<serde_json::Value>,
) -> bool {
    tracing::info!(
        "Executing rollback: {} on {} {} with before_state: {:?}",
        rollback_action,
        entity_type,
        entity_id,
        before_state
    );

    let result = match rollback_action {
        "add_liked_song" => {
            // Re-add a liked song: PUT /v1/me/tracks
            let track_id = before_state
                .as_ref()
                .and_then(|s| s.get("track_id"))
                .and_then(|v| v.as_str())
                .unwrap_or(entity_id);
            spotify_service
                .add_liked_songs_batch(connection, &[track_id.to_string()])
                .await
        }
        "follow_artist" => {
            // Re-follow an artist: PUT /v1/me/following?type=artist
            let artist_id = before_state
                .as_ref()
                .and_then(|s| s.get("artist_id"))
                .and_then(|v| v.as_str())
                .unwrap_or(entity_id);
            spotify_service
                .follow_artists_batch(connection, &[artist_id.to_string()])
                .await
        }
        "add_playlist_track" => {
            // Re-add a playlist track: POST /v1/playlists/{id}/tracks
            let playlist_id = before_state
                .as_ref()
                .and_then(|s| s.get("playlist_id"))
                .and_then(|v| v.as_str());
            let track_uri = before_state
                .as_ref()
                .and_then(|s| s.get("track_uri"))
                .and_then(|v| v.as_str())
                .or_else(|| {
                    before_state
                        .as_ref()
                        .and_then(|s| s.get("track_id"))
                        .and_then(|v| v.as_str())
                });

            match (playlist_id, track_uri) {
                (Some(pid), Some(uri)) => {
                    let full_uri = if uri.starts_with("spotify:track:") {
                        uri.to_string()
                    } else {
                        format!("spotify:track:{}", uri)
                    };
                    spotify_service
                        .add_playlist_tracks_batch(connection, pid, &[full_uri], None)
                        .await
                        .map(|_| ())
                }
                _ => {
                    tracing::error!(
                        "Missing playlist_id or track_uri for add_playlist_track rollback on entity {}",
                        entity_id
                    );
                    Err(anyhow::anyhow!(
                        "Missing playlist_id or track_uri in before_state"
                    ))
                }
            }
        }
        "add_saved_album" => {
            // Re-add a saved album: PUT /v1/me/albums
            let album_id = before_state
                .as_ref()
                .and_then(|s| s.get("album_id"))
                .and_then(|v| v.as_str())
                .unwrap_or(entity_id);
            spotify_service
                .add_saved_albums_batch(connection, &[album_id.to_string()])
                .await
        }
        _ => {
            tracing::warn!(
                "Unknown rollback action type: {} for entity {} {}",
                rollback_action,
                entity_type,
                entity_id
            );
            Err(anyhow::anyhow!(
                "Unknown rollback action: {}",
                rollback_action
            ))
        }
    };

    match result {
        Ok(()) => {
            tracing::info!(
                "Rollback {} on {} {} succeeded",
                rollback_action,
                entity_type,
                entity_id
            );
            true
        }
        Err(e) => {
            tracing::error!(
                "Rollback {} on {} {} failed: {}",
                rollback_action,
                entity_type,
                entity_id,
                e
            );
            false
        }
    }
}

/// Get Spotify enforcement history
///
/// GET /api/v1/enforcement/spotify/history
pub async fn get_spotify_enforcement_history(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<SpotifyEnforcementHistoryResponse>, AppError> {
    let user_id = user.id;

    // Query enforcement history from database using runtime query
    let rows: Vec<SpotifyEnforcementHistoryItemRow> = sqlx::query_as(
        r#"
        SELECT
            id,
            status,
            dry_run,
            summary,
            created_at,
            completed_at
        FROM action_batches
        WHERE user_id = $1 AND provider = 'spotify'
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal {
        message: Some(e.to_string()),
    })?;

    let batches: Vec<SpotifyEnforcementHistoryItem> = rows
        .into_iter()
        .map(|row| {
            let summary: BatchSummary = serde_json::from_value(row.summary).unwrap_or_default();
            let can_rollback = row.status == "completed" || row.status == "partially_completed";
            SpotifyEnforcementHistoryItem {
                batch_id: row.id,
                status: row.status,
                dry_run: row.dry_run,
                songs_removed: summary.completed_actions, // Approximation
                albums_removed: 0,
                artists_unfollowed: 0,
                playlist_tracks_removed: 0,
                errors_count: summary.failed_actions,
                created_at: row.created_at,
                completed_at: row.completed_at,
                can_rollback,
            }
        })
        .collect();

    let total_count = batches.len();

    Ok(Json(SpotifyEnforcementHistoryResponse {
        batches,
        total_count,
    }))
}

/// Internal row type for history query
#[derive(sqlx::FromRow)]
struct SpotifyEnforcementHistoryItemRow {
    id: Uuid,
    status: String,
    dry_run: bool,
    summary: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Get Spotify enforcement capabilities
///
/// GET /api/v1/enforcement/spotify/capabilities
pub async fn get_spotify_capabilities(
    State(_state): State<AppState>,
) -> Result<Json<SpotifyCapabilitiesResponse>, AppError> {
    Ok(Json(SpotifyCapabilitiesResponse {
        library_modification: true,
        playlist_modification: true,
        unfollow_artists: true,
        remove_saved_albums: true,
        batch_operations: true,
        rollback_support: true,
        enforcement_effects: vec![
            "Remove liked songs from blocked artists".to_string(),
            "Remove tracks from playlists (user-owned only by default)".to_string(),
            "Unfollow blocked artists".to_string(),
            "Remove saved albums from blocked artists".to_string(),
            "Supports batch operations for efficiency".to_string(),
        ],
        limitations: vec![
            "Cannot prevent playback of tracks".to_string(),
            "Cannot modify collaborative playlists owned by others".to_string(),
            "Rate limited to 50 items per batch API call".to_string(),
            "Rollback only available for recent operations with saved state".to_string(),
        ],
    }))
}

/// Generic rollback endpoint for any enforcement batch
///
/// POST /api/v1/enforcement/batches/{batch_id}/rollback
/// This is the generic endpoint that works with any provider's batches
pub async fn rollback_enforcement_batch(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(batch_id): Path<Uuid>,
    Json(request): Json<GenericRollbackRequest>,
) -> Result<Json<RollbackResponse>, AppError> {
    let user_id = user.id;

    // Get the batch to determine provider
    let batch_row: Option<BatchWithProvider> = sqlx::query_as(
        r#"
        SELECT id, user_id, provider, status, dry_run
        FROM action_batches
        WHERE id = $1
        "#,
    )
    .bind(batch_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal {
        message: Some(e.to_string()),
    })?;

    let batch = batch_row.ok_or_else(|| AppError::NotFound {
        resource: format!("Enforcement batch {}", batch_id),
    })?;

    // Verify ownership
    if batch.user_id != user_id {
        return Err(AppError::InsufficientPermissions);
    }

    // Check if batch can be rolled back
    if batch.status != "completed" && batch.status != "partially_completed" {
        return Err(AppError::InvalidFieldValue {
            field: "status".to_string(),
            message: format!(
                "Cannot rollback batch with status '{}'. Only 'completed' or 'partially_completed' batches can be rolled back.",
                batch.status
            ),
        });
    }

    // Cannot rollback a dry run batch
    if batch.dry_run {
        return Err(AppError::InvalidFieldValue {
            field: "dry_run".to_string(),
            message: "Cannot rollback a dry run batch - no actual changes were made".to_string(),
        });
    }

    // Get the user's Spotify connection for API calls
    let (spotify_service, token_vault) = create_spotify_service(&state)?;
    let connection = get_spotify_connection(&token_vault, user_id).await?;

    // Execute the rollback based on provider
    // Currently we support spotify, but the structure allows for other providers
    let rollback_result = execute_batch_rollback(
        &state.db_pool,
        batch_id,
        user_id,
        request.action_ids.as_deref(),
        &request.reason,
        &spotify_service,
        &connection,
    )
    .await?;

    Ok(Json(rollback_result))
}

/// Request for generic batch rollback
#[derive(Debug, Serialize, Deserialize)]
pub struct GenericRollbackRequest {
    /// Optional list of specific action IDs to rollback (if None, rollback entire batch)
    pub action_ids: Option<Vec<Uuid>>,
    /// Reason for rollback
    #[serde(default = "default_generic_rollback_reason")]
    pub reason: String,
}

fn default_generic_rollback_reason() -> String {
    "User requested rollback".to_string()
}

/// Batch info with provider
#[derive(sqlx::FromRow)]
struct BatchWithProvider {
    #[allow(dead_code)]
    id: Uuid,
    user_id: Uuid,
    #[allow(dead_code)]
    provider: String,
    status: String,
    dry_run: bool,
}

/// Get progress of a running enforcement batch
///
/// GET /api/v1/enforcement/spotify/progress/{batch_id}
pub async fn get_spotify_enforcement_progress(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<BatchProgress>, AppError> {
    // Query batch progress from database using runtime query
    let batch: Option<BatchProgressRow> = sqlx::query_as(
        r#"
        SELECT
            id,
            status,
            summary
        FROM action_batches
        WHERE id = $1
        "#,
    )
    .bind(batch_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal {
        message: Some(e.to_string()),
    })?;

    match batch {
        Some(batch) => {
            let summary: BatchSummary = serde_json::from_value(batch.summary).unwrap_or_default();

            Ok(Json(BatchProgress {
                batch_id,
                total_actions: summary.total_actions,
                completed_actions: summary.completed_actions,
                failed_actions: summary.failed_actions,
                current_action: None,
                estimated_remaining_ms: ((summary.total_actions
                    - summary.completed_actions
                    - summary.failed_actions) as u64)
                    * 750,
                rate_limit_status: crate::models::RateLimitStatus {
                    requests_remaining: 100,
                    reset_time: chrono::Utc::now() + chrono::Duration::hours(1),
                    current_delay_ms: 0,
                },
            }))
        }
        None => Err(AppError::NotFound {
            resource: format!("batch {}", batch_id),
        }),
    }
}

/// Internal row type for batch progress query
#[derive(sqlx::FromRow)]
struct BatchProgressRow {
    #[allow(dead_code)]
    id: Uuid,
    #[allow(dead_code)]
    status: String,
    summary: serde_json::Value,
}

// Helper function to get blocked artist IDs from DNP list
async fn get_blocked_artist_ids(state: &AppState, user_id: Uuid) -> Result<Vec<Uuid>, AppError> {
    let rows: Vec<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT DISTINCT a.id
        FROM user_artist_blocks uab
        JOIN artists a ON uab.artist_id = a.id
        WHERE uab.user_id = $1

        UNION

        SELECT DISTINCT a.id
        FROM category_subscriptions cs
        JOIN artist_offenses ao ON ao.category = cs.category
        JOIN artists a ON ao.artist_id = a.id
        WHERE cs.user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal {
        message: Some(e.to_string()),
    })?;

    Ok(rows.into_iter().map(|r| r.0).collect())
}

/// Blocked artist info with details
struct BlockedArtistInfo {
    #[allow(dead_code)]
    id: Uuid,
    name: String,
    reason: String,
}

async fn get_blocked_artists_with_details(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<BlockedArtistInfo>, AppError> {
    // Get directly blocked artists
    let direct_blocks: Vec<(Uuid, String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT a.id, a.canonical_name, uab.note
        FROM user_artist_blocks uab
        JOIN artists a ON uab.artist_id = a.id
        WHERE uab.user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal {
        message: Some(e.to_string()),
    })?;

    let mut artists: Vec<BlockedArtistInfo> = direct_blocks
        .into_iter()
        .map(|(id, name, note)| BlockedArtistInfo {
            id,
            name,
            reason: note.unwrap_or_else(|| "On your Do-Not-Play list".to_string()),
        })
        .collect();

    // Get category-blocked artists
    let category_blocks: Vec<(Uuid, String, String)> = sqlx::query_as(
        r#"
        SELECT DISTINCT a.id, a.canonical_name, cs.category
        FROM category_subscriptions cs
        JOIN artist_offenses ao ON ao.category = cs.category
        JOIN artists a ON ao.artist_id = a.id
        WHERE cs.user_id = $1
        AND a.id NOT IN (
            SELECT artist_id FROM user_artist_blocks WHERE user_id = $1
        )
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal {
        message: Some(e.to_string()),
    })?;

    artists.extend(
        category_blocks
            .into_iter()
            .map(|(id, name, category)| BlockedArtistInfo {
                id,
                name,
                reason: format!("Blocked via category subscription: {}", category),
            }),
    );

    Ok(artists)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================
    // SpotifyRunEnforcementRequest Tests
    // ============================================

    #[test]
    fn test_spotify_run_enforcement_request_default() {
        let request = SpotifyRunEnforcementRequest::default();

        assert!(matches!(
            request.aggressiveness,
            AggressivenessLevel::Moderate
        ));
        assert!(request.block_featuring);
        assert!(request.block_collaborations);
        assert!(!request.block_songwriter_only);
        assert!(request.preserve_user_playlists);
        assert!(request.execute_immediately);
        assert_eq!(request.batch_size, 50);
        assert!(!request.dry_run);
        assert!(request.idempotency_key.is_none());
    }

    #[test]
    fn test_spotify_run_enforcement_request_deserialization_defaults() {
        let json = r#"{}"#;
        let request: SpotifyRunEnforcementRequest = serde_json::from_str(json).unwrap();

        assert!(matches!(
            request.aggressiveness,
            AggressivenessLevel::Moderate
        ));
        assert!(request.block_featuring);
        assert!(request.block_collaborations);
        assert!(!request.dry_run);
    }

    #[test]
    fn test_spotify_run_enforcement_request_deserialization_custom() {
        let json = r#"{
            "aggressiveness": "Aggressive",
            "block_featuring": false,
            "block_collaborations": true,
            "block_songwriter_only": true,
            "preserve_user_playlists": false,
            "execute_immediately": false,
            "batch_size": 25,
            "dry_run": true,
            "idempotency_key": "test-key-123"
        }"#;
        let request: SpotifyRunEnforcementRequest = serde_json::from_str(json).unwrap();

        assert!(matches!(
            request.aggressiveness,
            AggressivenessLevel::Aggressive
        ));
        assert!(!request.block_featuring);
        assert!(request.block_collaborations);
        assert!(request.block_songwriter_only);
        assert!(!request.preserve_user_playlists);
        assert!(!request.execute_immediately);
        assert_eq!(request.batch_size, 25);
        assert!(request.dry_run);
        assert_eq!(request.idempotency_key, Some("test-key-123".to_string()));
    }

    #[test]
    fn test_spotify_run_enforcement_request_to_options() {
        let request = SpotifyRunEnforcementRequest {
            aggressiveness: AggressivenessLevel::Conservative,
            block_featuring: true,
            block_collaborations: false,
            block_songwriter_only: true,
            preserve_user_playlists: true,
            execute_immediately: true,
            batch_size: 100,
            dry_run: true,
            idempotency_key: None,
        };

        let options: EnforcementOptions = request.into();

        assert!(matches!(
            options.aggressiveness,
            AggressivenessLevel::Conservative
        ));
        assert!(options.block_featuring);
        assert!(!options.block_collaborations);
        assert!(options.block_songwriter_only);
        assert!(options.preserve_user_playlists);
        assert!(options.dry_run);
        assert_eq!(options.providers, vec!["spotify".to_string()]);
    }

    // ============================================
    // SpotifyEnforcementRunResponse Tests
    // ============================================

    #[test]
    fn test_spotify_enforcement_run_response_serialization() {
        let batch_id = Uuid::new_v4();
        let response = SpotifyEnforcementRunResponse {
            batch_id,
            status: "completed".to_string(),
            summary: BatchSummary::default(),
            songs_removed: 10,
            albums_removed: 5,
            artists_unfollowed: 2,
            playlist_tracks_removed: 15,
            errors_count: 1,
            message: "Enforcement complete".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains(&batch_id.to_string()));
        assert!(json.contains("completed"));
        assert!(json.contains("Enforcement complete"));
    }

    #[test]
    fn test_spotify_enforcement_run_response_skipped() {
        let response = SpotifyEnforcementRunResponse {
            batch_id: Uuid::nil(),
            status: "skipped".to_string(),
            summary: BatchSummary::default(),
            songs_removed: 0,
            albums_removed: 0,
            artists_unfollowed: 0,
            playlist_tracks_removed: 0,
            errors_count: 0,
            message: "No blocked artists found".to_string(),
        };

        assert_eq!(response.status, "skipped");
        assert!(response.batch_id.is_nil());
    }

    // ============================================
    // SpotifyEnforcementPreviewResponse Tests
    // ============================================

    #[test]
    fn test_spotify_enforcement_preview_response_empty() {
        let response = SpotifyEnforcementPreviewResponse {
            songs_to_remove: 0,
            albums_to_remove: 0,
            artists_to_unfollow: 0,
            playlist_tracks_to_remove: 0,
            total_library_songs: 500,
            total_library_albums: 100,
            total_followed_artists: 50,
            total_playlists: 20,
            estimated_duration_seconds: 0,
            blocked_content: SpotifyBlockedContentPreview {
                songs: Vec::new(),
                albums: Vec::new(),
                artists: Vec::new(),
                playlist_tracks: Vec::new(),
            },
        };

        assert_eq!(response.songs_to_remove, 0);
        assert_eq!(response.total_library_songs, 500);
    }

    #[test]
    fn test_spotify_enforcement_preview_response_with_content() {
        let songs = vec![BlockedSongPreview {
            track_id: "track-1".to_string(),
            name: "Bad Song".to_string(),
            artist_name: "Bad Artist".to_string(),
            album_name: "Bad Album".to_string(),
            blocked_reason: "Direct block".to_string(),
        }];

        let albums = vec![BlockedAlbumPreview {
            album_id: "album-1".to_string(),
            name: "Bad Album".to_string(),
            artist_name: "Bad Artist".to_string(),
            blocked_reason: "Direct block".to_string(),
        }];

        let artists = vec![BlockedArtistPreview {
            artist_id: "artist-1".to_string(),
            name: "Bad Artist".to_string(),
            blocked_reason: "In DNP list".to_string(),
        }];

        let response = SpotifyEnforcementPreviewResponse {
            songs_to_remove: 1,
            albums_to_remove: 1,
            artists_to_unfollow: 1,
            playlist_tracks_to_remove: 0,
            total_library_songs: 500,
            total_library_albums: 100,
            total_followed_artists: 50,
            total_playlists: 20,
            estimated_duration_seconds: 5,
            blocked_content: SpotifyBlockedContentPreview {
                songs,
                albums,
                artists,
                playlist_tracks: Vec::new(),
            },
        };

        assert_eq!(response.songs_to_remove, 1);
        assert_eq!(response.albums_to_remove, 1);
        assert_eq!(response.artists_to_unfollow, 1);
    }

    #[test]
    fn test_spotify_enforcement_preview_response_serialization() {
        let response = SpotifyEnforcementPreviewResponse {
            songs_to_remove: 5,
            albums_to_remove: 2,
            artists_to_unfollow: 1,
            playlist_tracks_to_remove: 10,
            total_library_songs: 500,
            total_library_albums: 100,
            total_followed_artists: 50,
            total_playlists: 20,
            estimated_duration_seconds: 15,
            blocked_content: SpotifyBlockedContentPreview {
                songs: Vec::new(),
                albums: Vec::new(),
                artists: Vec::new(),
                playlist_tracks: Vec::new(),
            },
        };

        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("songs_to_remove"));
        assert!(json.contains("albums_to_remove"));
        assert!(json.contains("artists_to_unfollow"));
        assert!(json.contains("playlist_tracks_to_remove"));
    }

    // ============================================
    // SpotifyRollbackRequest Tests
    // ============================================

    #[test]
    fn test_spotify_rollback_request_default_reason() {
        let json = r#"{}"#;
        let request: SpotifyRollbackRequest = serde_json::from_str(json).unwrap();

        assert!(request.action_ids.is_none());
        assert_eq!(request.reason, "User requested rollback");
    }

    #[test]
    fn test_spotify_rollback_request_with_actions() {
        let action_id = Uuid::new_v4();
        let json = format!(
            r#"{{
            "action_ids": ["{}"],
            "reason": "Rollback specific actions"
        }}"#,
            action_id
        );
        let request: SpotifyRollbackRequest = serde_json::from_str(&json).unwrap();

        assert!(request.action_ids.is_some());
        assert_eq!(request.action_ids.unwrap().len(), 1);
        assert_eq!(request.reason, "Rollback specific actions");
    }

    // ============================================
    // SpotifyEnforcementHistoryResponse Tests
    // ============================================

    #[test]
    fn test_spotify_enforcement_history_response_empty() {
        let response = SpotifyEnforcementHistoryResponse {
            batches: Vec::new(),
            total_count: 0,
        };

        assert!(response.batches.is_empty());
        assert_eq!(response.total_count, 0);
    }

    #[test]
    fn test_spotify_enforcement_history_response_with_batches() {
        let batches = vec![
            SpotifyEnforcementHistoryItem {
                batch_id: Uuid::new_v4(),
                status: "completed".to_string(),
                dry_run: false,
                songs_removed: 10,
                albums_removed: 5,
                artists_unfollowed: 2,
                playlist_tracks_removed: 15,
                errors_count: 0,
                created_at: chrono::Utc::now(),
                completed_at: Some(chrono::Utc::now()),
                can_rollback: true,
            },
            SpotifyEnforcementHistoryItem {
                batch_id: Uuid::new_v4(),
                status: "dry_run".to_string(),
                dry_run: true,
                songs_removed: 0,
                albums_removed: 0,
                artists_unfollowed: 0,
                playlist_tracks_removed: 0,
                errors_count: 0,
                created_at: chrono::Utc::now() - chrono::Duration::hours(1),
                completed_at: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
                can_rollback: false,
            },
        ];

        let response = SpotifyEnforcementHistoryResponse {
            batches,
            total_count: 2,
        };

        assert_eq!(response.batches.len(), 2);
        assert_eq!(response.total_count, 2);
        assert!(response.batches[0].can_rollback);
        assert!(!response.batches[1].can_rollback);
    }

    #[test]
    fn test_spotify_enforcement_history_item_serialization() {
        let item = SpotifyEnforcementHistoryItem {
            batch_id: Uuid::new_v4(),
            status: "completed".to_string(),
            dry_run: false,
            songs_removed: 10,
            albums_removed: 5,
            artists_unfollowed: 2,
            playlist_tracks_removed: 15,
            errors_count: 1,
            created_at: chrono::Utc::now(),
            completed_at: Some(chrono::Utc::now()),
            can_rollback: true,
        };

        let json = serde_json::to_string(&item).unwrap();

        assert!(json.contains("batch_id"));
        assert!(json.contains("completed"));
        assert!(json.contains("songs_removed"));
        assert!(json.contains("can_rollback"));
    }

    // ============================================
    // SpotifyCapabilitiesResponse Tests
    // ============================================

    #[test]
    fn test_spotify_capabilities_response_creation() {
        let response = SpotifyCapabilitiesResponse {
            library_modification: true,
            playlist_modification: true,
            unfollow_artists: true,
            remove_saved_albums: true,
            batch_operations: true,
            rollback_support: true,
            enforcement_effects: vec![
                "Remove liked songs".to_string(),
                "Remove playlist tracks".to_string(),
            ],
            limitations: vec!["Cannot prevent playback".to_string()],
        };

        assert!(response.library_modification);
        assert!(response.playlist_modification);
        assert!(response.rollback_support);
        assert_eq!(response.enforcement_effects.len(), 2);
        assert_eq!(response.limitations.len(), 1);
    }

    #[test]
    fn test_spotify_capabilities_response_serialization() {
        let response = SpotifyCapabilitiesResponse {
            library_modification: true,
            playlist_modification: true,
            unfollow_artists: true,
            remove_saved_albums: true,
            batch_operations: true,
            rollback_support: true,
            enforcement_effects: vec!["Effect 1".to_string()],
            limitations: vec!["Limitation 1".to_string()],
        };

        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("library_modification"));
        assert!(json.contains("true"));
        assert!(json.contains("enforcement_effects"));
        assert!(json.contains("limitations"));
    }

    // ============================================
    // BlockedSongPreview Tests
    // ============================================

    #[test]
    fn test_blocked_song_preview_serialization() {
        let preview = BlockedSongPreview {
            track_id: "spotify:track:123".to_string(),
            name: "Test Song".to_string(),
            artist_name: "Test Artist".to_string(),
            album_name: "Test Album".to_string(),
            blocked_reason: "Direct block".to_string(),
        };

        let json = serde_json::to_string(&preview).unwrap();
        let deserialized: BlockedSongPreview = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.track_id, preview.track_id);
        assert_eq!(deserialized.name, preview.name);
        assert_eq!(deserialized.artist_name, preview.artist_name);
    }

    // ============================================
    // BlockedAlbumPreview Tests
    // ============================================

    #[test]
    fn test_blocked_album_preview_serialization() {
        let preview = BlockedAlbumPreview {
            album_id: "spotify:album:456".to_string(),
            name: "Test Album".to_string(),
            artist_name: "Test Artist".to_string(),
            blocked_reason: "Collaboration".to_string(),
        };

        let json = serde_json::to_string(&preview).unwrap();
        let deserialized: BlockedAlbumPreview = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.album_id, preview.album_id);
        assert_eq!(deserialized.name, preview.name);
    }

    // ============================================
    // BlockedArtistPreview Tests
    // ============================================

    #[test]
    fn test_blocked_artist_preview_serialization() {
        let preview = BlockedArtistPreview {
            artist_id: "spotify:artist:789".to_string(),
            name: "Blocked Artist".to_string(),
            blocked_reason: "In DNP list".to_string(),
        };

        let json = serde_json::to_string(&preview).unwrap();
        let deserialized: BlockedArtistPreview = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.artist_id, preview.artist_id);
        assert_eq!(deserialized.blocked_reason, preview.blocked_reason);
    }

    // ============================================
    // BlockedPlaylistTrackPreview Tests
    // ============================================

    #[test]
    fn test_blocked_playlist_track_preview_serialization() {
        let preview = BlockedPlaylistTrackPreview {
            playlist_id: "playlist-123".to_string(),
            playlist_name: "My Playlist".to_string(),
            track_id: "track-456".to_string(),
            track_name: "Bad Song".to_string(),
            artist_name: "Blocked Artist".to_string(),
            blocked_reason: "Featuring blocked artist".to_string(),
        };

        let json = serde_json::to_string(&preview).unwrap();
        let deserialized: BlockedPlaylistTrackPreview = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.playlist_id, preview.playlist_id);
        assert_eq!(deserialized.playlist_name, preview.playlist_name);
        assert_eq!(deserialized.track_name, preview.track_name);
    }

    // ============================================
    // Default Function Tests
    // ============================================

    #[test]
    fn test_default_true_function() {
        assert!(default_true());
    }

    #[test]
    fn test_default_batch_size_function() {
        assert_eq!(default_batch_size(), 50);
    }

    #[test]
    fn test_default_rollback_reason_function() {
        assert_eq!(default_rollback_reason(), "User requested rollback");
    }

    // ============================================
    // Aggressiveness Level Tests
    // ============================================

    #[test]
    fn test_aggressiveness_level_serialization() {
        let conservative = AggressivenessLevel::Conservative;
        let moderate = AggressivenessLevel::Moderate;
        let aggressive = AggressivenessLevel::Aggressive;

        assert_eq!(
            serde_json::to_string(&conservative).unwrap(),
            "\"Conservative\""
        );
        assert_eq!(serde_json::to_string(&moderate).unwrap(), "\"Moderate\"");
        assert_eq!(
            serde_json::to_string(&aggressive).unwrap(),
            "\"Aggressive\""
        );
    }

    #[test]
    fn test_aggressiveness_level_deserialization() {
        let conservative: AggressivenessLevel = serde_json::from_str("\"Conservative\"").unwrap();
        let moderate: AggressivenessLevel = serde_json::from_str("\"Moderate\"").unwrap();
        let aggressive: AggressivenessLevel = serde_json::from_str("\"Aggressive\"").unwrap();

        assert!(matches!(conservative, AggressivenessLevel::Conservative));
        assert!(matches!(moderate, AggressivenessLevel::Moderate));
        assert!(matches!(aggressive, AggressivenessLevel::Aggressive));
    }

    // ============================================
    // JSON Roundtrip Tests
    // ============================================

    #[test]
    fn test_spotify_run_enforcement_request_json_roundtrip() {
        let original = SpotifyRunEnforcementRequest {
            aggressiveness: AggressivenessLevel::Aggressive,
            block_featuring: true,
            block_collaborations: false,
            block_songwriter_only: true,
            preserve_user_playlists: false,
            execute_immediately: false,
            batch_size: 25,
            dry_run: true,
            idempotency_key: Some("test-key".to_string()),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: SpotifyRunEnforcementRequest = serde_json::from_str(&json).unwrap();

        assert!(matches!(
            deserialized.aggressiveness,
            AggressivenessLevel::Aggressive
        ));
        assert_eq!(deserialized.block_featuring, original.block_featuring);
        assert_eq!(
            deserialized.block_collaborations,
            original.block_collaborations
        );
        assert_eq!(deserialized.batch_size, original.batch_size);
        assert_eq!(deserialized.dry_run, original.dry_run);
        assert_eq!(deserialized.idempotency_key, original.idempotency_key);
    }

    #[test]
    fn test_spotify_rollback_request_json_roundtrip() {
        let action_ids = vec![Uuid::new_v4(), Uuid::new_v4()];
        let original = SpotifyRollbackRequest {
            action_ids: Some(action_ids.clone()),
            reason: "Test rollback".to_string(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: SpotifyRollbackRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.action_ids.unwrap().len(), 2);
        assert_eq!(deserialized.reason, original.reason);
    }

    // ============================================
    // Edge Case Tests
    // ============================================

    #[test]
    fn test_response_with_max_values() {
        let response = SpotifyEnforcementRunResponse {
            batch_id: Uuid::new_v4(),
            status: "completed".to_string(),
            summary: BatchSummary {
                total_actions: u32::MAX,
                completed_actions: u32::MAX,
                failed_actions: 0,
                skipped_actions: 0,
                execution_time_ms: u64::MAX,
                api_calls_made: u32::MAX,
                rate_limit_delays_ms: u64::MAX,
                errors: Vec::new(),
            },
            songs_removed: usize::MAX,
            albums_removed: usize::MAX,
            artists_unfollowed: usize::MAX,
            playlist_tracks_removed: usize::MAX,
            errors_count: 0,
            message: "Test".to_string(),
        };

        assert_eq!(response.songs_removed, usize::MAX);
        assert_eq!(response.summary.execution_time_ms, u64::MAX);
    }

    #[test]
    fn test_response_with_nil_uuid() {
        let response = SpotifyEnforcementRunResponse {
            batch_id: Uuid::nil(),
            status: "skipped".to_string(),
            summary: BatchSummary::default(),
            songs_removed: 0,
            albums_removed: 0,
            artists_unfollowed: 0,
            playlist_tracks_removed: 0,
            errors_count: 0,
            message: "No blocked artists".to_string(),
        };

        assert!(response.batch_id.is_nil());
    }

    #[test]
    fn test_empty_blocked_content_preview() {
        let preview = SpotifyBlockedContentPreview {
            songs: Vec::new(),
            albums: Vec::new(),
            artists: Vec::new(),
            playlist_tracks: Vec::new(),
        };

        let json = serde_json::to_string(&preview).unwrap();
        assert!(json.contains("[]"));
    }

    // ============================================
    // Request Validation Tests
    // ============================================

    #[test]
    fn test_request_with_zero_batch_size() {
        let json = r#"{"batch_size": 0}"#;
        let request: SpotifyRunEnforcementRequest = serde_json::from_str(json).unwrap();

        // Zero batch size is allowed by deserialization, validation would be at service level
        assert_eq!(request.batch_size, 0);
    }

    #[test]
    fn test_request_with_large_batch_size() {
        let json = r#"{"batch_size": 1000}"#;
        let request: SpotifyRunEnforcementRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.batch_size, 1000);
    }
}
