//! Tidal Enforcement API Handlers
//!
//! Endpoints for running, monitoring, and rolling back Tidal enforcement operations.

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::tidal::{
    TidalCapabilities, TidalEnforcementPreview, TidalEnforcementRequest, TidalEnforcementResult,
};
use crate::services::tidal_enforcement::{
    TidalEnforcementHistoryItem, TidalEnforcementService, TidalRollbackResult,
};
use crate::AppState;

/// Request to run Tidal enforcement
#[derive(Debug, Serialize, Deserialize)]
pub struct RunTidalEnforcementRequest {
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

impl Default for RunTidalEnforcementRequest {
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

impl From<RunTidalEnforcementRequest> for TidalEnforcementRequest {
    fn from(req: RunTidalEnforcementRequest) -> Self {
        Self {
            block_featuring: req.block_featuring,
            block_collaborations: req.block_collaborations,
            preserve_user_playlists: req.preserve_user_playlists,
            dry_run: req.dry_run,
            idempotency_key: req.idempotency_key,
        }
    }
}

/// Response from enforcement run
#[derive(Debug, Serialize)]
pub struct TidalEnforcementRunResponse {
    pub batch_id: Uuid,
    pub status: String,
    pub tracks_removed: u32,
    pub albums_removed: u32,
    pub artists_unfollowed: u32,
    pub playlist_tracks_removed: u32,
    pub errors_count: u32,
    pub execution_time_ms: u64,
    pub dry_run: bool,
    pub message: String,
}

impl From<TidalEnforcementResult> for TidalEnforcementRunResponse {
    fn from(result: TidalEnforcementResult) -> Self {
        let message = if result.dry_run {
            format!(
                "Dry run complete. Would remove {} tracks, {} albums, unfollow {} artists, and remove {} playlist tracks.",
                result.tracks_removed, result.albums_removed, result.artists_unfollowed, result.playlist_tracks_removed
            )
        } else {
            format!(
                "Enforcement complete. Removed {} tracks, {} albums, unfollowed {} artists, and removed {} playlist tracks.",
                result.tracks_removed, result.albums_removed, result.artists_unfollowed, result.playlist_tracks_removed
            )
        };

        Self {
            batch_id: result.batch_id,
            status: result.status,
            tracks_removed: result.tracks_removed,
            albums_removed: result.albums_removed,
            artists_unfollowed: result.artists_unfollowed,
            playlist_tracks_removed: result.playlist_tracks_removed,
            errors_count: result.errors_count,
            execution_time_ms: result.execution_time_ms,
            dry_run: result.dry_run,
            message,
        }
    }
}

/// Response from enforcement preview
#[derive(Debug, Serialize)]
pub struct TidalEnforcementPreviewResponse {
    pub tracks_to_remove: u32,
    pub albums_to_remove: u32,
    pub artists_to_unfollow: u32,
    pub playlist_tracks_to_remove: u32,
    pub total_favorite_tracks: u32,
    pub total_favorite_albums: u32,
    pub total_favorite_artists: u32,
    pub total_playlists: u32,
    pub estimated_duration_seconds: u32,
    pub blocked_content: TidalEnforcementPreview,
}

impl From<TidalEnforcementPreview> for TidalEnforcementPreviewResponse {
    fn from(preview: TidalEnforcementPreview) -> Self {
        Self {
            tracks_to_remove: preview.tracks_to_remove,
            albums_to_remove: preview.albums_to_remove,
            artists_to_unfollow: preview.artists_to_unfollow,
            playlist_tracks_to_remove: preview.playlist_tracks_to_remove,
            total_favorite_tracks: preview.total_favorite_tracks,
            total_favorite_albums: preview.total_favorite_albums,
            total_favorite_artists: preview.total_favorite_artists,
            total_playlists: preview.total_playlists,
            estimated_duration_seconds: preview.estimated_duration_seconds,
            blocked_content: preview,
        }
    }
}

/// Response from enforcement history
#[derive(Debug, Serialize)]
pub struct TidalEnforcementHistoryResponse {
    pub runs: Vec<TidalEnforcementHistoryItem>,
}

/// Run Tidal enforcement
///
/// POST /api/v1/enforcement/tidal/run
pub async fn run_tidal_enforcement(
    State(state): State<AppState>,
    Json(request): Json<RunTidalEnforcementRequest>,
) -> Result<Json<TidalEnforcementRunResponse>, AppError> {
    // For now, use a hardcoded user_id - in production this would come from auth middleware
    let user_id = state.test_user_id.unwrap_or_else(Uuid::new_v4);

    // Get blocked artists from DNP list
    let (blocked_names, blocked_ids) = get_blocked_artists(&state, user_id).await?;

    if blocked_names.is_empty() && blocked_ids.is_empty() {
        return Ok(Json(TidalEnforcementRunResponse {
            batch_id: Uuid::nil(),
            status: "skipped".to_string(),
            tracks_removed: 0,
            albums_removed: 0,
            artists_unfollowed: 0,
            playlist_tracks_removed: 0,
            errors_count: 0,
            execution_time_ms: 0,
            dry_run: request.dry_run,
            message: "No blocked artists found in DNP list".to_string(),
        }));
    }

    let options: TidalEnforcementRequest = request.into();

    // Create enforcement service
    let enforcement_service = TidalEnforcementService::new(
        state.tidal_service.clone(),
        state.token_vault.clone(),
        state.db_pool.clone(),
    );

    // Run enforcement
    let result = enforcement_service
        .enforce_dnp_list(user_id, blocked_names, blocked_ids, options, |_progress| {
            // Progress callback - could be used for websocket updates
        })
        .await
        .map_err(|e| AppError::Internal {
            message: Some(e.to_string()),
        })?;

    Ok(Json(result.into()))
}

/// Preview enforcement (dry run)
///
/// POST /api/v1/enforcement/tidal/preview
pub async fn preview_tidal_enforcement(
    State(state): State<AppState>,
) -> Result<Json<TidalEnforcementPreviewResponse>, AppError> {
    let user_id = state.test_user_id.unwrap_or_else(Uuid::new_v4);

    // Get blocked artists
    let (blocked_names, blocked_ids) = get_blocked_artists(&state, user_id).await?;

    if blocked_names.is_empty() && blocked_ids.is_empty() {
        return Ok(Json(TidalEnforcementPreviewResponse {
            tracks_to_remove: 0,
            albums_to_remove: 0,
            artists_to_unfollow: 0,
            playlist_tracks_to_remove: 0,
            total_favorite_tracks: 0,
            total_favorite_albums: 0,
            total_favorite_artists: 0,
            total_playlists: 0,
            estimated_duration_seconds: 0,
            blocked_content: TidalEnforcementPreview {
                tracks_to_remove: 0,
                albums_to_remove: 0,
                artists_to_unfollow: 0,
                playlist_tracks_to_remove: 0,
                total_favorite_tracks: 0,
                total_favorite_albums: 0,
                total_favorite_artists: 0,
                total_playlists: 0,
                estimated_duration_seconds: 0,
                blocked_content: Default::default(),
            },
        }));
    }

    let enforcement_service = TidalEnforcementService::new(
        state.tidal_service.clone(),
        state.token_vault.clone(),
        state.db_pool.clone(),
    );

    let preview = enforcement_service
        .preview_enforcement(user_id, blocked_names, blocked_ids)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(e.to_string()),
        })?;

    Ok(Json(preview.into()))
}

/// Rollback an enforcement run
///
/// POST /api/v1/enforcement/tidal/rollback/{run_id}
pub async fn rollback_tidal_enforcement(
    State(state): State<AppState>,
    Path(run_id): Path<Uuid>,
) -> Result<Json<TidalRollbackResult>, AppError> {
    let user_id = state.test_user_id.unwrap_or_else(Uuid::new_v4);

    let enforcement_service = TidalEnforcementService::new(
        state.tidal_service.clone(),
        state.token_vault.clone(),
        state.db_pool.clone(),
    );

    let result = enforcement_service
        .rollback_enforcement(user_id, run_id)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(e.to_string()),
        })?;

    Ok(Json(result))
}

/// Get enforcement history
///
/// GET /api/v1/enforcement/tidal/history
pub async fn get_tidal_enforcement_history(
    State(state): State<AppState>,
) -> Result<Json<TidalEnforcementHistoryResponse>, AppError> {
    let user_id = state.test_user_id.unwrap_or_else(Uuid::new_v4);

    let enforcement_service = TidalEnforcementService::new(
        state.tidal_service.clone(),
        state.token_vault.clone(),
        state.db_pool.clone(),
    );

    let runs = enforcement_service
        .get_enforcement_history(user_id, 50)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(e.to_string()),
        })?;

    Ok(Json(TidalEnforcementHistoryResponse { runs }))
}

/// Get Tidal capabilities info
///
/// GET /api/v1/enforcement/tidal/capabilities
pub async fn get_tidal_capabilities(
    State(_state): State<AppState>,
) -> Result<Json<TidalCapabilities>, AppError> {
    Ok(Json(TidalEnforcementService::get_capabilities()))
}

// Helper function to get blocked artists from DNP list
async fn get_blocked_artists(
    state: &AppState,
    user_id: Uuid,
) -> Result<(Vec<String>, Vec<u64>), AppError> {
    // Query DNP list for blocked artist names and Tidal IDs
    let rows: Vec<(Option<String>, Option<String>)> = sqlx::query_as(
        r#"
        SELECT DISTINCT a.canonical_name as name, a.tidal_id
        FROM user_artist_blocks uab
        JOIN artists a ON uab.artist_id = a.id
        WHERE uab.user_id = $1

        UNION

        SELECT DISTINCT a.canonical_name as name, a.tidal_id
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

    let names: Vec<String> = rows.iter().filter_map(|r| r.0.clone()).collect();
    let ids: Vec<u64> = rows
        .iter()
        .filter_map(|r| r.1.as_ref().and_then(|id| id.parse::<u64>().ok()))
        .collect();

    Ok((names, ids))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_enforcement_request_default() {
        let request = RunTidalEnforcementRequest::default();

        assert!(request.block_featuring);
        assert!(request.block_collaborations);
        assert!(request.preserve_user_playlists);
        assert!(!request.dry_run);
        assert!(request.idempotency_key.is_none());
    }

    #[test]
    fn test_run_enforcement_request_deserialization_defaults() {
        let json = r#"{}"#;
        let request: RunTidalEnforcementRequest = serde_json::from_str(json).unwrap();

        assert!(request.block_featuring);
        assert!(request.block_collaborations);
        assert!(request.preserve_user_playlists);
        assert!(!request.dry_run);
    }

    #[test]
    fn test_run_enforcement_request_deserialization_custom() {
        let json = r#"{
            "block_featuring": false,
            "block_collaborations": true,
            "preserve_user_playlists": false,
            "dry_run": true,
            "idempotency_key": "test-key-123"
        }"#;
        let request: RunTidalEnforcementRequest = serde_json::from_str(json).unwrap();

        assert!(!request.block_featuring);
        assert!(request.block_collaborations);
        assert!(!request.preserve_user_playlists);
        assert!(request.dry_run);
        assert_eq!(request.idempotency_key, Some("test-key-123".to_string()));
    }

    #[test]
    fn test_run_enforcement_request_to_options() {
        let request = RunTidalEnforcementRequest {
            block_featuring: false,
            block_collaborations: true,
            preserve_user_playlists: false,
            dry_run: true,
            idempotency_key: Some("key".to_string()),
        };

        let options: TidalEnforcementRequest = request.into();

        assert!(!options.block_featuring);
        assert!(options.block_collaborations);
        assert!(!options.preserve_user_playlists);
        assert!(options.dry_run);
        assert_eq!(options.idempotency_key, Some("key".to_string()));
    }

    #[test]
    fn test_enforcement_run_response_dry_run_message() {
        let result = TidalEnforcementResult {
            batch_id: Uuid::new_v4(),
            status: "completed".to_string(),
            tracks_removed: 10,
            albums_removed: 5,
            artists_unfollowed: 3,
            playlist_tracks_removed: 2,
            errors_count: 0,
            errors: Vec::new(),
            execution_time_ms: 1000,
            dry_run: true,
        };

        let response: TidalEnforcementRunResponse = result.into();

        assert!(response.message.contains("Dry run complete"));
        assert!(response.message.contains("Would remove"));
    }

    #[test]
    fn test_enforcement_run_response_actual_run_message() {
        let result = TidalEnforcementResult {
            batch_id: Uuid::new_v4(),
            status: "completed".to_string(),
            tracks_removed: 10,
            albums_removed: 5,
            artists_unfollowed: 3,
            playlist_tracks_removed: 2,
            errors_count: 0,
            errors: Vec::new(),
            execution_time_ms: 1000,
            dry_run: false,
        };

        let response: TidalEnforcementRunResponse = result.into();

        assert!(response.message.contains("Enforcement complete"));
        assert!(response.message.contains("Removed"));
    }

    #[test]
    fn test_enforcement_run_response_skipped() {
        let response = TidalEnforcementRunResponse {
            batch_id: Uuid::nil(),
            status: "skipped".to_string(),
            tracks_removed: 0,
            albums_removed: 0,
            artists_unfollowed: 0,
            playlist_tracks_removed: 0,
            errors_count: 0,
            execution_time_ms: 0,
            dry_run: false,
            message: "No blocked artists found in DNP list".to_string(),
        };

        assert_eq!(response.status, "skipped");
        assert!(response.batch_id.is_nil());
    }
}
