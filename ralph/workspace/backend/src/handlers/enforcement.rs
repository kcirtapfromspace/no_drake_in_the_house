//! Apple Music Enforcement API Handlers
//!
//! Endpoints for running, monitoring, and rolling back enforcement operations.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{
    AppleMusicRatingEnforcementOptions, EnforcementPreview, EnforcementProgress,
    RatingEnforcementResult, RollbackResult,
};
use crate::services::{AppleMusicEnforcementService, EnforcementHistoryItem};
use crate::AppState;

/// Request to run Apple Music enforcement
#[derive(Debug, Serialize, Deserialize)]
pub struct RunEnforcementRequest {
    #[serde(default = "default_true")]
    pub dislike_songs: bool,
    #[serde(default = "default_true")]
    pub dislike_albums: bool,
    #[serde(default = "default_true")]
    pub include_library: bool,
    #[serde(default)]
    pub include_catalog: bool,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default)]
    pub dry_run: bool,
}

fn default_true() -> bool {
    true
}
fn default_batch_size() -> usize {
    50
}

impl Default for RunEnforcementRequest {
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

impl From<RunEnforcementRequest> for AppleMusicRatingEnforcementOptions {
    fn from(req: RunEnforcementRequest) -> Self {
        Self {
            dislike_songs: req.dislike_songs,
            dislike_albums: req.dislike_albums,
            include_library: req.include_library,
            include_catalog: req.include_catalog,
            batch_size: req.batch_size,
            dry_run: req.dry_run,
        }
    }
}

/// Response from enforcement run
#[derive(Debug, Serialize)]
pub struct EnforcementRunResponse {
    pub run_id: Uuid,
    pub status: String,
    pub songs_disliked: usize,
    pub albums_disliked: usize,
    pub errors_count: usize,
    pub duration_seconds: u64,
    pub message: String,
}

/// Response from enforcement preview
#[derive(Debug, Serialize)]
pub struct EnforcementPreviewResponse {
    pub songs_to_dislike: usize,
    pub albums_to_dislike: usize,
    pub total_library_songs: usize,
    pub total_library_albums: usize,
    pub estimated_duration_seconds: u64,
    pub blocked_content: EnforcementPreview,
}

/// Response from enforcement history
#[derive(Debug, Serialize)]
pub struct EnforcementHistoryResponse {
    pub runs: Vec<EnforcementHistoryItem>,
}

/// Run Apple Music enforcement
///
/// POST /api/v1/enforcement/apple-music/run
pub async fn run_apple_music_enforcement(
    State(state): State<AppState>,
    Json(request): Json<RunEnforcementRequest>,
) -> Result<Json<EnforcementRunResponse>, AppError> {
    // For now, use a hardcoded user_id - in production this would come from auth middleware
    let user_id = state.test_user_id.unwrap_or_else(Uuid::new_v4);

    // Get blocked artist names from DNP list
    let blocked_artists = get_blocked_artist_names(&state, user_id).await?;

    if blocked_artists.is_empty() {
        return Ok(Json(EnforcementRunResponse {
            run_id: Uuid::nil(),
            status: "skipped".to_string(),
            songs_disliked: 0,
            albums_disliked: 0,
            errors_count: 0,
            duration_seconds: 0,
            message: "No blocked artists found in DNP list".to_string(),
        }));
    }

    let options: AppleMusicRatingEnforcementOptions = request.into();
    let is_dry_run = options.dry_run;

    // Create enforcement service
    let enforcement_service =
        AppleMusicEnforcementService::new(state.apple_music_service.clone(), state.db_pool.clone());

    // Run enforcement
    let result = enforcement_service
        .enforce_dnp_list(user_id, blocked_artists, options, |_progress| {
            // Progress callback - could be used for websocket updates
        })
        .await
        .map_err(|e| AppError::Internal {
            message: Some(e.to_string()),
        })?;

    let message = if is_dry_run {
        format!(
            "Dry run complete. Would dislike {} songs and {} albums.",
            result.songs_disliked, result.albums_disliked
        )
    } else {
        format!(
            "Enforcement complete. Disliked {} songs and {} albums.",
            result.songs_disliked, result.albums_disliked
        )
    };

    Ok(Json(EnforcementRunResponse {
        run_id: result.run_id,
        status: result.status.to_string(),
        songs_disliked: result.songs_disliked,
        albums_disliked: result.albums_disliked,
        errors_count: result.errors.len(),
        duration_seconds: result.duration_seconds,
        message,
    }))
}

/// Preview enforcement (dry run)
///
/// POST /api/v1/enforcement/apple-music/preview
pub async fn preview_apple_music_enforcement(
    State(state): State<AppState>,
) -> Result<Json<EnforcementPreviewResponse>, AppError> {
    let user_id = state.test_user_id.unwrap_or_else(Uuid::new_v4);

    // Get blocked artist names
    let blocked_artists = get_blocked_artist_names(&state, user_id).await?;

    if blocked_artists.is_empty() {
        return Ok(Json(EnforcementPreviewResponse {
            songs_to_dislike: 0,
            albums_to_dislike: 0,
            total_library_songs: 0,
            total_library_albums: 0,
            estimated_duration_seconds: 0,
            blocked_content: EnforcementPreview {
                songs_to_dislike: Vec::new(),
                albums_to_dislike: Vec::new(),
                total_songs: 0,
                total_albums: 0,
                estimated_duration_seconds: 0,
            },
        }));
    }

    let enforcement_service =
        AppleMusicEnforcementService::new(state.apple_music_service.clone(), state.db_pool.clone());

    let preview = enforcement_service
        .preview_enforcement(user_id, blocked_artists)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(e.to_string()),
        })?;

    Ok(Json(EnforcementPreviewResponse {
        songs_to_dislike: preview.songs_to_dislike.len(),
        albums_to_dislike: preview.albums_to_dislike.len(),
        total_library_songs: preview.total_songs,
        total_library_albums: preview.total_albums,
        estimated_duration_seconds: preview.estimated_duration_seconds,
        blocked_content: preview,
    }))
}

/// Rollback an enforcement run
///
/// POST /api/v1/enforcement/apple-music/rollback/{run_id}
pub async fn rollback_apple_music_enforcement(
    State(state): State<AppState>,
    Path(run_id): Path<Uuid>,
) -> Result<Json<RollbackResult>, AppError> {
    let user_id = state.test_user_id.unwrap_or_else(Uuid::new_v4);

    let enforcement_service =
        AppleMusicEnforcementService::new(state.apple_music_service.clone(), state.db_pool.clone());

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
/// GET /api/v1/enforcement/apple-music/history
pub async fn get_apple_music_enforcement_history(
    State(state): State<AppState>,
) -> Result<Json<EnforcementHistoryResponse>, AppError> {
    let user_id = state.test_user_id.unwrap_or_else(Uuid::new_v4);

    let enforcement_service =
        AppleMusicEnforcementService::new(state.apple_music_service.clone(), state.db_pool.clone());

    let runs = enforcement_service
        .get_enforcement_history(user_id, 50)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(e.to_string()),
        })?;

    Ok(Json(EnforcementHistoryResponse { runs }))
}

/// Get capabilities info
///
/// GET /api/v1/enforcement/apple-music/capabilities
pub async fn get_apple_music_capabilities(
    State(_state): State<AppState>,
) -> Result<Json<AppleMusicCapabilitiesResponse>, AppError> {
    Ok(Json(AppleMusicCapabilitiesResponse {
        ratings_enforcement: true,
        library_modification: false,
        playlist_modification: false,
        enforcement_effects: vec![
            "Reduces recommendations for similar content".to_string(),
            "Influences 'For You' personalization".to_string(),
        ],
        limitations: vec![
            "Cannot remove songs from library".to_string(),
            "Cannot prevent playback".to_string(),
            "Cannot skip songs automatically".to_string(),
            "Must dislike individual songs/albums (no artist-level dislike)".to_string(),
        ],
    }))
}

#[derive(Debug, Serialize)]
pub struct AppleMusicCapabilitiesResponse {
    pub ratings_enforcement: bool,
    pub library_modification: bool,
    pub playlist_modification: bool,
    pub enforcement_effects: Vec<String>,
    pub limitations: Vec<String>,
}

// Helper function to get blocked artist names from DNP list
async fn get_blocked_artist_names(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<String>, AppError> {
    // Query DNP list for blocked artist names
    let rows: Vec<(Option<String>,)> = sqlx::query_as(
        r#"
        SELECT DISTINCT a.canonical_name as name
        FROM user_artist_blocks uab
        JOIN artists a ON uab.artist_id = a.id
        WHERE uab.user_id = $1

        UNION

        SELECT DISTINCT a.canonical_name as name
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

    Ok(rows.into_iter().filter_map(|r| r.0).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BlockedAlbumInfo, BlockedSongInfo, EnforcementRunStatus};

    // ============================================
    // RunEnforcementRequest Tests
    // ============================================

    #[test]
    fn test_run_enforcement_request_default() {
        let request = RunEnforcementRequest::default();

        assert!(request.dislike_songs);
        assert!(request.dislike_albums);
        assert!(request.include_library);
        assert!(!request.include_catalog);
        assert_eq!(request.batch_size, 50);
        assert!(!request.dry_run);
    }

    #[test]
    fn test_run_enforcement_request_deserialization_defaults() {
        let json = r#"{}"#;
        let request: RunEnforcementRequest = serde_json::from_str(json).unwrap();

        assert!(request.dislike_songs);
        assert!(request.dislike_albums);
        assert!(request.include_library);
        assert!(!request.include_catalog);
        assert_eq!(request.batch_size, 50);
        assert!(!request.dry_run);
    }

    #[test]
    fn test_run_enforcement_request_deserialization_custom() {
        let json = r#"{
            "dislike_songs": true,
            "dislike_albums": false,
            "include_library": true,
            "include_catalog": true,
            "batch_size": 100,
            "dry_run": true
        }"#;
        let request: RunEnforcementRequest = serde_json::from_str(json).unwrap();

        assert!(request.dislike_songs);
        assert!(!request.dislike_albums);
        assert!(request.include_library);
        assert!(request.include_catalog);
        assert_eq!(request.batch_size, 100);
        assert!(request.dry_run);
    }

    #[test]
    fn test_run_enforcement_request_deserialization_partial() {
        let json = r#"{"dry_run": true}"#;
        let request: RunEnforcementRequest = serde_json::from_str(json).unwrap();

        assert!(request.dislike_songs); // default
        assert!(request.dislike_albums); // default
        assert!(request.dry_run); // overridden
    }

    #[test]
    fn test_run_enforcement_request_to_options() {
        let request = RunEnforcementRequest {
            dislike_songs: true,
            dislike_albums: false,
            include_library: true,
            include_catalog: true,
            batch_size: 25,
            dry_run: true,
        };

        let options: AppleMusicRatingEnforcementOptions = request.into();

        assert!(options.dislike_songs);
        assert!(!options.dislike_albums);
        assert!(options.include_library);
        assert!(options.include_catalog);
        assert_eq!(options.batch_size, 25);
        assert!(options.dry_run);
    }

    // ============================================
    // EnforcementRunResponse Tests
    // ============================================

    #[test]
    fn test_enforcement_run_response_serialization() {
        let run_id = Uuid::new_v4();
        let response = EnforcementRunResponse {
            run_id,
            status: "completed".to_string(),
            songs_disliked: 10,
            albums_disliked: 5,
            errors_count: 2,
            duration_seconds: 30,
            message: "Enforcement complete".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains(&run_id.to_string()));
        assert!(json.contains("completed"));
        assert!(json.contains("10"));
        assert!(json.contains("Enforcement complete"));
    }

    #[test]
    fn test_enforcement_run_response_dry_run() {
        let response = EnforcementRunResponse {
            run_id: Uuid::new_v4(),
            status: "completed".to_string(),
            songs_disliked: 0,
            albums_disliked: 0,
            errors_count: 0,
            duration_seconds: 5,
            message: "Dry run complete. Would dislike 15 songs and 3 albums.".to_string(),
        };

        assert!(response.message.contains("Dry run"));
        assert!(response.message.contains("Would dislike"));
    }

    #[test]
    fn test_enforcement_run_response_skipped() {
        let response = EnforcementRunResponse {
            run_id: Uuid::nil(),
            status: "skipped".to_string(),
            songs_disliked: 0,
            albums_disliked: 0,
            errors_count: 0,
            duration_seconds: 0,
            message: "No blocked artists found in DNP list".to_string(),
        };

        assert_eq!(response.status, "skipped");
        assert!(response.run_id.is_nil());
    }

    // ============================================
    // EnforcementPreviewResponse Tests
    // ============================================

    #[test]
    fn test_enforcement_preview_response_empty() {
        let response = EnforcementPreviewResponse {
            songs_to_dislike: 0,
            albums_to_dislike: 0,
            total_library_songs: 500,
            total_library_albums: 100,
            estimated_duration_seconds: 0,
            blocked_content: EnforcementPreview {
                songs_to_dislike: Vec::new(),
                albums_to_dislike: Vec::new(),
                total_songs: 500,
                total_albums: 100,
                estimated_duration_seconds: 0,
            },
        };

        assert_eq!(response.songs_to_dislike, 0);
        assert_eq!(response.albums_to_dislike, 0);
        assert_eq!(response.total_library_songs, 500);
    }

    #[test]
    fn test_enforcement_preview_response_with_content() {
        let songs = vec![
            BlockedSongInfo {
                library_song_id: "song-1".to_string(),
                catalog_song_id: None,
                name: "Bad Song".to_string(),
                artist_name: "Bad Artist".to_string(),
                album_name: "Bad Album".to_string(),
                blocked_artist_id: None,
            },
            BlockedSongInfo {
                library_song_id: "song-2".to_string(),
                catalog_song_id: None,
                name: "Another Bad Song".to_string(),
                artist_name: "Bad Artist".to_string(),
                album_name: "Bad Album".to_string(),
                blocked_artist_id: None,
            },
        ];

        let albums = vec![BlockedAlbumInfo {
            library_album_id: "album-1".to_string(),
            catalog_album_id: None,
            name: "Bad Album".to_string(),
            artist_name: "Bad Artist".to_string(),
            blocked_artist_id: None,
        }];

        let response = EnforcementPreviewResponse {
            songs_to_dislike: songs.len(),
            albums_to_dislike: albums.len(),
            total_library_songs: 1000,
            total_library_albums: 200,
            estimated_duration_seconds: 1,
            blocked_content: EnforcementPreview {
                songs_to_dislike: songs,
                albums_to_dislike: albums,
                total_songs: 1000,
                total_albums: 200,
                estimated_duration_seconds: 1,
            },
        };

        assert_eq!(response.songs_to_dislike, 2);
        assert_eq!(response.albums_to_dislike, 1);
    }

    #[test]
    fn test_enforcement_preview_response_serialization() {
        let response = EnforcementPreviewResponse {
            songs_to_dislike: 5,
            albums_to_dislike: 2,
            total_library_songs: 500,
            total_library_albums: 100,
            estimated_duration_seconds: 3,
            blocked_content: EnforcementPreview {
                songs_to_dislike: Vec::new(),
                albums_to_dislike: Vec::new(),
                total_songs: 500,
                total_albums: 100,
                estimated_duration_seconds: 3,
            },
        };

        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("songs_to_dislike"));
        assert!(json.contains("albums_to_dislike"));
        assert!(json.contains("total_library_songs"));
        assert!(json.contains("estimated_duration_seconds"));
    }

    // ============================================
    // EnforcementHistoryResponse Tests
    // ============================================

    #[test]
    fn test_enforcement_history_response_empty() {
        let response = EnforcementHistoryResponse { runs: Vec::new() };

        assert!(response.runs.is_empty());
    }

    #[test]
    fn test_enforcement_history_response_with_runs() {
        let runs = vec![
            EnforcementHistoryItem {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                connection_id: Uuid::new_v4(),
                status: "completed".to_string(),
                options: AppleMusicRatingEnforcementOptions::default(),
                started_at: chrono::Utc::now(),
                completed_at: Some(chrono::Utc::now()),
                songs_scanned: 100,
                albums_scanned: 20,
                songs_disliked: 5,
                albums_disliked: 2,
                errors: 0,
            },
            EnforcementHistoryItem {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                connection_id: Uuid::new_v4(),
                status: "rolled_back".to_string(),
                options: AppleMusicRatingEnforcementOptions::default(),
                started_at: chrono::Utc::now() - chrono::Duration::hours(1),
                completed_at: Some(chrono::Utc::now() - chrono::Duration::minutes(55)),
                songs_scanned: 50,
                albums_scanned: 10,
                songs_disliked: 3,
                albums_disliked: 1,
                errors: 0,
            },
        ];

        let response = EnforcementHistoryResponse { runs };

        assert_eq!(response.runs.len(), 2);
        assert_eq!(response.runs[0].status, "completed");
        assert_eq!(response.runs[1].status, "rolled_back");
    }

    #[test]
    fn test_enforcement_history_response_serialization() {
        let response = EnforcementHistoryResponse {
            runs: vec![EnforcementHistoryItem {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                connection_id: Uuid::new_v4(),
                status: "completed".to_string(),
                options: AppleMusicRatingEnforcementOptions::default(),
                started_at: chrono::Utc::now(),
                completed_at: Some(chrono::Utc::now()),
                songs_scanned: 100,
                albums_scanned: 20,
                songs_disliked: 5,
                albums_disliked: 2,
                errors: 0,
            }],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("runs"));
        assert!(json.contains("completed"));
        assert!(json.contains("songs_scanned"));
    }

    // ============================================
    // AppleMusicCapabilitiesResponse Tests
    // ============================================

    #[test]
    fn test_capabilities_response_creation() {
        let response = AppleMusicCapabilitiesResponse {
            ratings_enforcement: true,
            library_modification: false,
            playlist_modification: false,
            enforcement_effects: vec![
                "Reduces recommendations".to_string(),
                "Influences For You".to_string(),
            ],
            limitations: vec![
                "Cannot remove from library".to_string(),
                "Cannot prevent playback".to_string(),
            ],
        };

        assert!(response.ratings_enforcement);
        assert!(!response.library_modification);
        assert!(!response.playlist_modification);
        assert_eq!(response.enforcement_effects.len(), 2);
        assert_eq!(response.limitations.len(), 2);
    }

    #[test]
    fn test_capabilities_response_serialization() {
        let response = AppleMusicCapabilitiesResponse {
            ratings_enforcement: true,
            library_modification: false,
            playlist_modification: false,
            enforcement_effects: vec!["Effect 1".to_string()],
            limitations: vec!["Limitation 1".to_string()],
        };

        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("ratings_enforcement"));
        assert!(json.contains("true"));
        assert!(json.contains("library_modification"));
        assert!(json.contains("false"));
        assert!(json.contains("enforcement_effects"));
        assert!(json.contains("limitations"));
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

    // ============================================
    // Message Generation Tests
    // ============================================

    #[test]
    fn test_dry_run_message_format() {
        let songs_disliked = 15;
        let albums_disliked = 3;
        let message = format!(
            "Dry run complete. Would dislike {} songs and {} albums.",
            songs_disliked, albums_disliked
        );

        assert!(message.contains("Dry run complete"));
        assert!(message.contains("15 songs"));
        assert!(message.contains("3 albums"));
    }

    #[test]
    fn test_enforcement_complete_message_format() {
        let songs_disliked = 10;
        let albums_disliked = 5;
        let message = format!(
            "Enforcement complete. Disliked {} songs and {} albums.",
            songs_disliked, albums_disliked
        );

        assert!(message.contains("Enforcement complete"));
        assert!(message.contains("10 songs"));
        assert!(message.contains("5 albums"));
    }

    // ============================================
    // Request Validation Tests
    // ============================================

    #[test]
    fn test_request_with_zero_batch_size() {
        let json = r#"{"batch_size": 0}"#;
        let request: RunEnforcementRequest = serde_json::from_str(json).unwrap();

        // Zero batch size is allowed by deserialization, validation would be at service level
        assert_eq!(request.batch_size, 0);
    }

    #[test]
    fn test_request_with_large_batch_size() {
        let json = r#"{"batch_size": 1000}"#;
        let request: RunEnforcementRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.batch_size, 1000);
    }

    #[test]
    fn test_request_songs_only() {
        let json = r#"{
            "dislike_songs": true,
            "dislike_albums": false
        }"#;
        let request: RunEnforcementRequest = serde_json::from_str(json).unwrap();

        assert!(request.dislike_songs);
        assert!(!request.dislike_albums);
    }

    #[test]
    fn test_request_albums_only() {
        let json = r#"{
            "dislike_songs": false,
            "dislike_albums": true
        }"#;
        let request: RunEnforcementRequest = serde_json::from_str(json).unwrap();

        assert!(!request.dislike_songs);
        assert!(request.dislike_albums);
    }

    #[test]
    fn test_request_catalog_included() {
        let json = r#"{"include_catalog": true}"#;
        let request: RunEnforcementRequest = serde_json::from_str(json).unwrap();

        assert!(request.include_library); // default
        assert!(request.include_catalog); // overridden
    }

    // ============================================
    // Response Status Tests
    // ============================================

    #[test]
    fn test_response_status_strings() {
        let statuses = vec![
            "pending",
            "running",
            "completed",
            "failed",
            "rolled_back",
            "skipped",
        ];

        for status in statuses {
            let response = EnforcementRunResponse {
                run_id: Uuid::new_v4(),
                status: status.to_string(),
                songs_disliked: 0,
                albums_disliked: 0,
                errors_count: 0,
                duration_seconds: 0,
                message: "Test".to_string(),
            };

            assert_eq!(response.status, status);
        }
    }

    // ============================================
    // Edge Case Tests
    // ============================================

    #[test]
    fn test_response_with_max_values() {
        let response = EnforcementRunResponse {
            run_id: Uuid::new_v4(),
            status: "completed".to_string(),
            songs_disliked: usize::MAX,
            albums_disliked: usize::MAX,
            errors_count: usize::MAX,
            duration_seconds: u64::MAX,
            message: "Test".to_string(),
        };

        assert_eq!(response.songs_disliked, usize::MAX);
        assert_eq!(response.duration_seconds, u64::MAX);
    }

    #[test]
    fn test_response_with_nil_uuid() {
        let response = EnforcementRunResponse {
            run_id: Uuid::nil(),
            status: "skipped".to_string(),
            songs_disliked: 0,
            albums_disliked: 0,
            errors_count: 0,
            duration_seconds: 0,
            message: "No blocked artists".to_string(),
        };

        assert!(response.run_id.is_nil());
    }

    #[test]
    fn test_preview_response_calculations() {
        let songs_count = 15;
        let albums_count = 5;
        let total_items = songs_count + albums_count;
        let estimated_ms_per_item = 50;
        let estimated_duration = (total_items * estimated_ms_per_item) / 1000;

        let response = EnforcementPreviewResponse {
            songs_to_dislike: songs_count,
            albums_to_dislike: albums_count,
            total_library_songs: 1000,
            total_library_albums: 200,
            estimated_duration_seconds: estimated_duration as u64,
            blocked_content: EnforcementPreview {
                songs_to_dislike: Vec::new(),
                albums_to_dislike: Vec::new(),
                total_songs: 1000,
                total_albums: 200,
                estimated_duration_seconds: estimated_duration as u64,
            },
        };

        assert_eq!(response.songs_to_dislike + response.albums_to_dislike, 20);
        assert_eq!(response.estimated_duration_seconds, 1); // (20 * 50) / 1000 = 1
    }

    // ============================================
    // Capabilities Endpoint Tests
    // ============================================

    #[test]
    fn test_capabilities_response_has_expected_effects() {
        let response = AppleMusicCapabilitiesResponse {
            ratings_enforcement: true,
            library_modification: false,
            playlist_modification: false,
            enforcement_effects: vec![
                "Reduces recommendations for similar content".to_string(),
                "Influences 'For You' personalization".to_string(),
            ],
            limitations: vec![
                "Cannot remove songs from library".to_string(),
                "Cannot prevent playback".to_string(),
                "Cannot skip songs automatically".to_string(),
                "Must dislike individual songs/albums (no artist-level dislike)".to_string(),
            ],
        };

        assert!(response
            .enforcement_effects
            .iter()
            .any(|e| e.contains("recommendations")));
        assert!(response
            .enforcement_effects
            .iter()
            .any(|e| e.contains("For You")));
    }

    #[test]
    fn test_capabilities_response_has_expected_limitations() {
        let response = AppleMusicCapabilitiesResponse {
            ratings_enforcement: true,
            library_modification: false,
            playlist_modification: false,
            enforcement_effects: vec![],
            limitations: vec![
                "Cannot remove songs from library".to_string(),
                "Cannot prevent playback".to_string(),
                "Cannot skip songs automatically".to_string(),
                "Must dislike individual songs/albums (no artist-level dislike)".to_string(),
            ],
        };

        assert!(response.limitations.iter().any(|l| l.contains("library")));
        assert!(response.limitations.iter().any(|l| l.contains("playback")));
        assert!(response.limitations.iter().any(|l| l.contains("skip")));
        assert!(response
            .limitations
            .iter()
            .any(|l| l.contains("artist-level")));
    }

    // ============================================
    // JSON Roundtrip Tests
    // ============================================

    #[test]
    fn test_run_enforcement_request_json_roundtrip() {
        let original = RunEnforcementRequest {
            dislike_songs: true,
            dislike_albums: false,
            include_library: true,
            include_catalog: true,
            batch_size: 75,
            dry_run: true,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: RunEnforcementRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.dislike_songs, original.dislike_songs);
        assert_eq!(deserialized.dislike_albums, original.dislike_albums);
        assert_eq!(deserialized.include_library, original.include_library);
        assert_eq!(deserialized.include_catalog, original.include_catalog);
        assert_eq!(deserialized.batch_size, original.batch_size);
        assert_eq!(deserialized.dry_run, original.dry_run);
    }

    #[test]
    fn test_enforcement_run_response_json_roundtrip() {
        let run_id = Uuid::new_v4();
        let original = EnforcementRunResponse {
            run_id,
            status: "completed".to_string(),
            songs_disliked: 10,
            albums_disliked: 5,
            errors_count: 1,
            duration_seconds: 30,
            message: "Test message".to_string(),
        };

        let json = serde_json::to_string(&original).unwrap();

        // Since EnforcementRunResponse only has Serialize, we parse as Value
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["run_id"], run_id.to_string());
        assert_eq!(parsed["status"], "completed");
        assert_eq!(parsed["songs_disliked"], 10);
        assert_eq!(parsed["albums_disliked"], 5);
        assert_eq!(parsed["errors_count"], 1);
        assert_eq!(parsed["duration_seconds"], 30);
        assert_eq!(parsed["message"], "Test message");
    }
}
