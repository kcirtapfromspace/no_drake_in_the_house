use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::offense::{
    AddEvidenceRequest, CreateOffenseRequest, FlaggedArtist, ImportLibraryRequest,
    LibraryScanResponse, OffenseSeverity, OffenseWithEvidence,
};
use crate::models::{AuthenticatedUser, Claims};
use crate::services::offense::OffenseService;
use crate::AppState;

/// Query parameters for listing flagged artists
#[derive(Debug, Deserialize)]
pub struct FlaggedArtistsQuery {
    pub severity: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Get all flagged artists in the database
pub async fn get_flagged_artists(
    State(state): State<AppState>,
    Query(query): Query<FlaggedArtistsQuery>,
) -> Result<Json<Vec<FlaggedArtist>>> {
    let offense_service = OffenseService::new(&state.db_pool);

    let severity = query.severity.as_ref().and_then(|s| match s.as_str() {
        "minor" => Some(OffenseSeverity::Minor),
        "moderate" => Some(OffenseSeverity::Moderate),
        "severe" => Some(OffenseSeverity::Severe),
        "egregious" => Some(OffenseSeverity::Egregious),
        _ => None,
    });

    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let artists = offense_service
        .get_flagged_artists(severity, limit, offset)
        .await?;

    Ok(Json(artists))
}

/// Get offense details with evidence
pub async fn get_offense(
    State(state): State<AppState>,
    Path(offense_id): Path<Uuid>,
) -> Result<Json<OffenseWithEvidence>> {
    let offense_service = OffenseService::new(&state.db_pool);
    let offense = offense_service
        .get_offense_with_evidence(offense_id)
        .await?;
    Ok(Json(offense))
}

/// Create a new offense (requires authentication)
pub async fn create_offense(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<CreateOffenseRequest>,
) -> Result<impl IntoResponse> {
    let offense_service = OffenseService::new(&state.db_pool);
    let offense = offense_service
        .create_offense(request, Some(user.id))
        .await?;
    Ok((StatusCode::CREATED, Json(offense)))
}

/// Add evidence to an offense (requires authentication)
pub async fn add_evidence(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<AddEvidenceRequest>,
) -> Result<impl IntoResponse> {
    let offense_service = OffenseService::new(&state.db_pool);
    let evidence = offense_service.add_evidence(request, Some(user.id)).await?;
    Ok((StatusCode::CREATED, Json(evidence)))
}

/// Verify an offense (moderator only)
/// Requires moderator or admin role to verify offenses
pub async fn verify_offense(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    claims: Claims,
    Path(offense_id): Path<Uuid>,
) -> Result<StatusCode> {
    // Check if user has moderator role
    if !claims.has_moderator_access() {
        tracing::warn!(
            user_id = %user.id,
            offense_id = %offense_id,
            role = ?claims.role,
            "Unauthorized attempt to verify offense - moderator role required"
        );
        return Err(AppError::InsufficientPermissions);
    }

    tracing::info!(
        user_id = %user.id,
        offense_id = %offense_id,
        role = ?claims.role,
        "Moderator verifying offense"
    );

    let offense_service = OffenseService::new(&state.db_pool);
    offense_service.verify_offense(offense_id, user.id).await?;
    Ok(StatusCode::OK)
}

/// Import library tracks from file upload
pub async fn import_library(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<ImportLibraryRequest>,
) -> Result<Json<serde_json::Value>> {
    let offense_service = OffenseService::new(&state.db_pool);
    let imported = offense_service.import_library(user.id, request).await?;
    Ok(Json(serde_json::json!({
        "imported": imported,
        "message": format!("Successfully imported {} tracks", imported)
    })))
}

/// Scan user's library against offense database
pub async fn scan_library(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<LibraryScanResponse>> {
    let offense_service = OffenseService::new(&state.db_pool);
    let result = offense_service.scan_library(user.id).await?;
    Ok(Json(result))
}

/// Get user's imported library tracks
pub async fn get_library(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<LibraryQuery>,
) -> Result<Json<Vec<crate::models::offense::UserLibraryTrack>>> {
    let offense_service = OffenseService::new(&state.db_pool);
    let tracks = offense_service
        .get_user_library(user.id, query.provider)
        .await?;
    Ok(Json(tracks))
}

#[derive(Debug, Deserialize)]
pub struct LibraryQuery {
    pub provider: Option<String>,
}

/// Query parameters for getting artists by category
#[derive(Debug, Deserialize)]
pub struct CategoryArtistsQuery {
    pub category: Option<String>,
    pub artist_id: Option<Uuid>,
}

/// Get artists by category or get artist details
pub async fn get_category_artists(
    State(state): State<AppState>,
    Query(query): Query<CategoryArtistsQuery>,
) -> Result<Json<serde_json::Value>> {
    let offense_service = OffenseService::new(&state.db_pool);

    // If artist_id is provided, return artist details
    if let Some(artist_id) = query.artist_id {
        let details = offense_service.get_artist_details(artist_id).await?;
        return Ok(Json(serde_json::json!({
            "success": true,
            "data": details
        })));
    }

    // If category is provided, return artists in that category
    if let Some(category) = &query.category {
        let artists = offense_service.get_artists_by_category(category).await?;
        return Ok(Json(serde_json::json!({
            "success": true,
            "data": {
                "artists": artists
            }
        })));
    }

    // Neither provided - return all flagged artists
    let artists = offense_service.get_flagged_artists(None, 100, 0).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "artists": artists
        }
    })))
}
