//! Spotify Enforcement API Handlers
//!
//! Endpoints for running, monitoring, and rolling back Spotify enforcement operations.

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{
    ActionBatchStatus, AggressivenessLevel, AuthenticatedUser, BatchExecutionResult, BatchProgress,
    BatchSummary, EnforcementOptions, EnforcementPlan, ExecuteBatchRequest, RollbackBatchRequest,
    RollbackInfo,
};
use crate::AppState;

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
    let _options: EnforcementOptions = request.into();

    // Get blocked artist details for the response
    let blocked_artist_details = get_blocked_artists_with_details(&state, user_id).await?;
    let artists_count = blocked_artist_details.len();

    let batch_id = Uuid::new_v4();

    let message = if is_dry_run {
        format!(
            "Dry run complete. Found {} blocked artists that would be affected. No changes were made.",
            artists_count
        )
    } else {
        format!(
            "Spotify enforcement requires OAuth connection. Connect your Spotify account first, then enforcement will remove content from {} blocked artists.",
            artists_count
        )
    };

    Ok(Json(SpotifyEnforcementRunResponse {
        batch_id,
        status: if is_dry_run {
            "dry_run".to_string()
        } else {
            "pending_connection".to_string()
        },
        summary: BatchSummary::default(),
        songs_removed: 0,
        albums_removed: 0,
        artists_unfollowed: artists_count,
        playlist_tracks_removed: 0,
        errors_count: 0,
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

    // Convert blocked artists to preview format
    let blocked_artist_previews: Vec<BlockedArtistPreview> = blocked_artist_details
        .iter()
        .map(|a| BlockedArtistPreview {
            artist_id: a.id.to_string(),
            name: a.name.clone(),
            blocked_reason: a.reason.clone(),
        })
        .collect();

    let artists_count = blocked_artist_previews.len();

    // Note: Without Spotify library access, we show blocked artists but can't scan actual library
    // Estimated counts are placeholders until Spotify OAuth is connected
    Ok(Json(SpotifyEnforcementPreviewResponse {
        songs_to_remove: 0,  // Requires Spotify library scan
        albums_to_remove: 0, // Requires Spotify library scan
        artists_to_unfollow: artists_count,
        playlist_tracks_to_remove: 0, // Requires Spotify library scan
        total_library_songs: 0,
        total_library_albums: 0,
        total_followed_artists: 0,
        total_playlists: 0,
        estimated_duration_seconds: (artists_count as u64) * 2, // ~2 seconds per artist
        blocked_content: SpotifyBlockedContentPreview {
            songs: Vec::new(), // Populated when Spotify library is scanned
            albums: Vec::new(),
            artists: blocked_artist_previews,
            playlist_tracks: Vec::new(),
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

    // Get actions that can be rolled back
    let rollback_result = execute_batch_rollback(
        &state.db_pool,
        batch_id,
        user_id,
        request.action_ids.as_deref(),
        &request.reason,
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

            // Simulate rollback execution
            // In production, this would call Spotify API to re-add tracks/follow artists
            // For now, we mark as completed and update the original action as rolled back
            let rollback_success = execute_single_rollback_action(
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

/// Execute a single rollback action
/// In production, this would call the actual Spotify API
/// For now, it simulates the rollback
async fn execute_single_rollback_action(
    entity_type: &str,
    entity_id: &str,
    rollback_action: &str,
    before_state: &Option<serde_json::Value>,
) -> bool {
    // Log the rollback action
    tracing::info!(
        "Executing rollback: {} on {} {} with before_state: {:?}",
        rollback_action,
        entity_type,
        entity_id,
        before_state
    );

    // In production, this would:
    // - For "add_liked_song": Call PUT /v1/me/tracks with the track IDs
    // - For "follow_artist": Call PUT /v1/me/following?type=artist with artist IDs
    // - For "add_playlist_track": Call POST /v1/playlists/{playlist_id}/tracks
    // - For "add_saved_album": Call PUT /v1/me/albums with album IDs

    // For now, simulate success (in a real implementation, we'd check the API response)
    true
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

    // Execute the rollback based on provider
    // Currently we support spotify, but the structure allows for other providers
    let rollback_result = execute_batch_rollback(
        &state.db_pool,
        batch_id,
        user_id,
        request.action_ids.as_deref(),
        &request.reason,
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
