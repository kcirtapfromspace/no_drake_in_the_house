//! Playlist Sanitizer API Handlers
//!
//! Endpoints for grading playlists, suggesting replacements, and publishing sanitized versions.

use axum::{
    extract::{Path, State},
    Json,
};
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{
    AuthenticatedUser, ConfirmPlanRequest, Connection, ConnectionStatus, GradePlaylistRequest,
    GradeResponse, PublishResponse, StreamingProvider, SuggestReplacementsRequest,
    SuggestResponse,
};
use crate::AppState;
use ndith_services::{PlaylistSanitizerService, SpotifyLibraryService, SpotifyService, TokenVaultService};

/// POST /api/v1/sanitizer/grade
///
/// Grade a playlist against the user's blocklist.
pub async fn grade_playlist(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<GradePlaylistRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = user.id;
    let playlist_id = parse_playlist_id(&request.playlist_id);

    let (sanitizer, connection) = setup_sanitizer(&state, user_id).await?;
    let blocked_ids = get_blocked_spotify_ids(&state, user_id).await?;

    if blocked_ids.is_empty() {
        return Err(AppError::InvalidRequestFormat(
            "No blocked artists found. Add artists to your blocklist first.".to_string(),
        ));
    }

    let grade = sanitizer
        .grade_playlist(&connection, &playlist_id, &blocked_ids)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(e.to_string()),
        })?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": GradeResponse { grade }
    })))
}

/// POST /api/v1/sanitizer/suggest
///
/// Grade a playlist and suggest replacements. Returns a draft plan.
pub async fn suggest_replacements(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<SuggestReplacementsRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = user.id;
    let playlist_id = parse_playlist_id(&request.playlist_id);

    let (sanitizer, connection) = setup_sanitizer(&state, user_id).await?;
    let blocked_ids = get_blocked_spotify_ids(&state, user_id).await?;

    if blocked_ids.is_empty() {
        return Err(AppError::InvalidRequestFormat(
            "No blocked artists found. Add artists to your blocklist first.".to_string(),
        ));
    }

    // Grade first
    let grade = sanitizer
        .grade_playlist(&connection, &playlist_id, &blocked_ids)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(e.to_string()),
        })?;

    if grade.blocked_tracks == 0 {
        return Ok(Json(serde_json::json!({
            "success": true,
            "data": {
                "plan_id": null,
                "grade": grade,
                "replacements": [],
                "message": "Playlist is already clean!"
            }
        })));
    }

    // Suggest replacements
    let replacements = sanitizer
        .suggest_replacements(&connection, &grade, &blocked_ids)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(e.to_string()),
        })?;

    // Save as draft plan
    let plan_id = sanitizer
        .save_plan(user_id, &grade, &replacements)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(e.to_string()),
        })?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": SuggestResponse {
            plan_id,
            grade,
            replacements,
        }
    })))
}

/// PUT /api/v1/sanitizer/plan/:plan_id
///
/// Confirm a plan: user selects replacements and target name.
pub async fn confirm_plan(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(plan_id): Path<Uuid>,
    Json(request): Json<ConfirmPlanRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let sanitizer = create_sanitizer_service(&state)?;

    let plan = sanitizer
        .confirm_plan(plan_id, user.id, &request)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(e.to_string()),
        })?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "plan_id": plan.id,
            "status": plan.status.to_string(),
            "target_playlist_name": plan.target_playlist_name,
        }
    })))
}

/// POST /api/v1/sanitizer/publish/:plan_id
///
/// Create the sanitized playlist on Spotify.
pub async fn publish_playlist(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(plan_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (sanitizer, connection) = setup_sanitizer(&state, user.id).await?;

    let result = sanitizer
        .publish_sanitized_playlist(&connection, plan_id, user.id)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(e.to_string()),
        })?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": PublishResponse {
            plan_id,
            result,
        }
    })))
}

// ---- Helpers ----

/// Parse a playlist ID from a URL or raw ID.
fn parse_playlist_id(input: &str) -> String {
    // Handle Spotify URLs like https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M?si=...
    if input.contains("spotify.com/playlist/") {
        if let Some(id_part) = input.split("playlist/").nth(1) {
            return id_part
                .split('?')
                .next()
                .unwrap_or(id_part)
                .to_string();
        }
    }
    // Handle spotify: URIs
    if input.starts_with("spotify:playlist:") {
        return input
            .strip_prefix("spotify:playlist:")
            .unwrap_or(input)
            .to_string();
    }
    input.to_string()
}

/// Create a PlaylistSanitizerService from AppState.
fn create_sanitizer_service(state: &AppState) -> Result<PlaylistSanitizerService, AppError> {
    let spotify_config = ndith_services::SpotifyConfig::default();
    let token_vault = Arc::new(TokenVaultService::with_pool(state.db_pool.clone()));
    let spotify_service = Arc::new(
        SpotifyService::new(spotify_config, token_vault).map_err(|e| AppError::Internal {
            message: Some(format!("Failed to initialize Spotify service: {}", e)),
        })?,
    );
    let library_service = Arc::new(SpotifyLibraryService::new(spotify_service.clone()));

    Ok(PlaylistSanitizerService::new(
        spotify_service,
        library_service,
        state.db_pool.clone(),
    ))
}

/// Create sanitizer service and fetch the user's Spotify connection.
async fn setup_sanitizer(
    state: &AppState,
    user_id: Uuid,
) -> Result<(PlaylistSanitizerService, Connection), AppError> {
    let sanitizer = create_sanitizer_service(state)?;

    // Get connection via TokenVaultService
    let token_vault = TokenVaultService::with_pool(state.db_pool.clone());
    let connections = token_vault.get_user_connections(user_id).await;

    let connection = connections
        .into_iter()
        .find(|c| c.provider == StreamingProvider::Spotify && c.status == ConnectionStatus::Active)
        .ok_or_else(|| AppError::InvalidRequestFormat(
            "No active Spotify connection. Please connect your Spotify account first.".to_string(),
        ))?;

    Ok((sanitizer, connection))
}

/// Get Spotify artist IDs on the user's blocklist.
///
/// Queries the user_artist_blocks and category_subscriptions tables,
/// extracting Spotify IDs from the artists.external_ids JSONB field.
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
