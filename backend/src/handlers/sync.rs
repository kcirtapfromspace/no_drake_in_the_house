//! Catalog Sync API Handlers
//!
//! Endpoints for managing multi-platform artist catalog synchronization.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::AuthenticatedUser;
use crate::services::catalog_sync::{
    CanonicalArtist, IdentityMatch, OverallSyncStatus, Platform, PlatformArtist, SyncPriority,
    SyncStatus, SyncTriggerRequest, SyncType,
};
use crate::{AppError, AppState, Result};

/// Query parameters for sync history
#[derive(Debug, Deserialize)]
pub struct SyncHistoryQuery {
    /// Filter by platform
    pub platform: Option<String>,
    /// Limit results
    #[serde(default = "default_limit")]
    pub limit: i32,
    /// Offset for pagination
    #[serde(default)]
    pub offset: i32,
}

fn default_limit() -> i32 {
    20
}

/// Request to trigger a sync
#[derive(Debug, Deserialize)]
pub struct TriggerSyncRequest {
    /// Platforms to sync (empty = all)
    #[serde(default)]
    pub platforms: Vec<String>,
    /// Sync type (full or incremental)
    #[serde(default = "default_sync_type")]
    pub sync_type: String,
    /// Priority (low, normal, high, critical)
    #[serde(default = "default_priority")]
    pub priority: String,
}

fn default_sync_type() -> String {
    "incremental".to_string()
}

fn default_priority() -> String {
    "normal".to_string()
}

/// Request to resolve an artist identity
#[derive(Debug, Deserialize)]
pub struct ResolveIdentityRequest {
    /// Platform the artist is from
    pub platform: String,
    /// Platform-specific artist ID
    pub platform_id: String,
    /// Artist name
    pub name: String,
    /// Genres (optional)
    #[serde(default)]
    pub genres: Vec<String>,
}

/// Request to merge two artists
#[derive(Debug, Deserialize)]
pub struct MergeArtistsRequest {
    /// Primary artist ID (will be kept)
    pub primary_id: Uuid,
    /// Secondary artist ID (will be merged into primary)
    pub secondary_id: Uuid,
}

/// Search query for cross-platform artist search
#[derive(Debug, Deserialize)]
pub struct CrossPlatformSearchQuery {
    /// Search query
    pub q: String,
    /// Limit per platform
    #[serde(default = "default_per_platform_limit")]
    pub limit_per_platform: u32,
}

fn default_per_platform_limit() -> u32 {
    5
}

/// Response for sync status
#[derive(Debug, Serialize)]
pub struct SyncStatusResponse {
    pub success: bool,
    pub data: SyncStatusData,
}

#[derive(Debug, Serialize)]
pub struct SyncStatusData {
    pub platforms: Vec<PlatformStatusItem>,
    pub total_artists: u32,
    pub last_full_sync: Option<String>,
    pub last_incremental_sync: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PlatformStatusItem {
    pub platform: String,
    pub is_healthy: bool,
    pub last_sync: Option<String>,
    pub artists_synced: u32,
    pub is_syncing: bool,
}

/// Get overall sync status across all platforms
pub async fn get_sync_status_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Sync status request");

    // For now, return mock status since orchestrator isn't integrated into AppState yet
    let status = SyncStatusData {
        platforms: vec![
            PlatformStatusItem {
                platform: "spotify".to_string(),
                is_healthy: true,
                last_sync: None,
                artists_synced: 0,
                is_syncing: false,
            },
            PlatformStatusItem {
                platform: "apple_music".to_string(),
                is_healthy: true,
                last_sync: None,
                artists_synced: 0,
                is_syncing: false,
            },
            PlatformStatusItem {
                platform: "tidal".to_string(),
                is_healthy: true,
                last_sync: None,
                artists_synced: 0,
                is_syncing: false,
            },
            PlatformStatusItem {
                platform: "youtube_music".to_string(),
                is_healthy: true,
                last_sync: None,
                artists_synced: 0,
                is_syncing: false,
            },
            PlatformStatusItem {
                platform: "deezer".to_string(),
                is_healthy: true,
                last_sync: None,
                artists_synced: 0,
                is_syncing: false,
            },
        ],
        total_artists: 0,
        last_full_sync: None,
        last_incremental_sync: None,
    };

    Ok(Json(serde_json::json!({
        "success": true,
        "data": status
    })))
}

/// Trigger a catalog sync
pub async fn trigger_sync_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Json(request): Json<TriggerSyncRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    tracing::info!(
        platforms = ?request.platforms,
        sync_type = %request.sync_type,
        priority = %request.priority,
        "Sync trigger request"
    );

    // Validate sync type
    let sync_type = match request.sync_type.as_str() {
        "full" => SyncType::Full,
        "incremental" => SyncType::Incremental,
        _ => {
            return Err(AppError::InvalidFieldValue {
                field: "sync_type".to_string(),
                message: "Must be 'full' or 'incremental'".to_string(),
            });
        }
    };

    // Validate priority
    let _priority = match request.priority.as_str() {
        "low" => SyncPriority::Low,
        "normal" => SyncPriority::Normal,
        "high" => SyncPriority::High,
        "critical" => SyncPriority::Critical,
        _ => {
            return Err(AppError::InvalidFieldValue {
                field: "priority".to_string(),
                message: "Must be 'low', 'normal', 'high', or 'critical'".to_string(),
            });
        }
    };

    // Parse platforms
    let platforms: Vec<Platform> = if request.platforms.is_empty() {
        vec![
            Platform::Spotify,
            Platform::AppleMusic,
            Platform::Tidal,
            Platform::YouTubeMusic,
            Platform::Deezer,
        ]
    } else {
        request
            .platforms
            .iter()
            .filter_map(|p| parse_platform(p))
            .collect()
    };

    if platforms.is_empty() {
        return Err(AppError::InvalidFieldValue {
            field: "platforms".to_string(),
            message: "No valid platforms specified".to_string(),
        });
    }

    // For now, return placeholder run IDs since orchestrator isn't integrated
    let run_ids: Vec<Uuid> = platforms.iter().map(|_| Uuid::new_v4()).collect();

    tracing::info!(
        run_ids = ?run_ids,
        platforms = ?platforms,
        "Sync triggered"
    );

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "success": true,
            "data": {
                "run_ids": run_ids,
                "platforms": platforms.iter().map(|p| format!("{:?}", p).to_lowercase()).collect::<Vec<_>>(),
                "sync_type": format!("{:?}", sync_type).to_lowercase()
            },
            "message": "Sync triggered successfully"
        })),
    ))
}

/// Get sync run history
pub async fn get_sync_runs_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Query(query): Query<SyncHistoryQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        platform = ?query.platform,
        limit = query.limit,
        offset = query.offset,
        "Sync history request"
    );

    // Return empty history for now
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "runs": [],
            "total": 0,
            "limit": query.limit,
            "offset": query.offset
        }
    })))
}

/// Get a specific sync run by ID
pub async fn get_sync_run_handler(
    State(_state): State<AppState>,
    Path(run_id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(run_id = %run_id, "Sync run detail request");

    // Return not found for now
    Err(AppError::NotFound {
        resource: "Sync run".to_string(),
    })
}

/// Cancel a sync run
pub async fn cancel_sync_run_handler(
    State(_state): State<AppState>,
    Path(run_id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(run_id = %run_id, "Cancel sync run request");

    // Return not found for now
    Err(AppError::NotFound {
        resource: "Sync run".to_string(),
    })
}

/// Resolve an artist identity across platforms
pub async fn resolve_identity_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Json(request): Json<ResolveIdentityRequest>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        platform = %request.platform,
        platform_id = %request.platform_id,
        name = %request.name,
        "Identity resolution request"
    );

    let platform =
        parse_platform(&request.platform).ok_or_else(|| AppError::InvalidFieldValue {
            field: "platform".to_string(),
            message: format!("Unknown platform: {}", request.platform),
        })?;

    // For now, return a new artist placeholder
    let canonical_id = Uuid::new_v4();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "canonical_artist": {
                "id": canonical_id,
                "name": request.name,
                "platform_ids": {
                    request.platform: request.platform_id
                },
                "genres": request.genres,
                "musicbrainz_id": null,
                "isni": null
            },
            "match": {
                "method": "new_artist",
                "confidence": 1.0,
                "needs_review": false
            }
        }
    })))
}

/// Merge two artists
pub async fn merge_artists_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Json(request): Json<MergeArtistsRequest>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        primary_id = %request.primary_id,
        secondary_id = %request.secondary_id,
        "Merge artists request"
    );

    if request.primary_id == request.secondary_id {
        return Err(AppError::InvalidFieldValue {
            field: "secondary_id".to_string(),
            message: "Cannot merge an artist with itself".to_string(),
        });
    }

    // Return not found for now since we don't have artists stored
    Err(AppError::NotFound {
        resource: "Artist".to_string(),
    })
}

/// Search for an artist across all platforms
pub async fn cross_platform_search_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Query(query): Query<CrossPlatformSearchQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        query = %query.q,
        limit_per_platform = query.limit_per_platform,
        "Cross-platform search request"
    );

    if query.q.trim().is_empty() {
        return Err(AppError::InvalidFieldValue {
            field: "q".to_string(),
            message: "Search query cannot be empty".to_string(),
        });
    }

    if query.q.len() > 100 {
        return Err(AppError::InvalidFieldValue {
            field: "q".to_string(),
            message: "Search query too long (max 100 characters)".to_string(),
        });
    }

    // Return empty results for now
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "results": {
                "spotify": [],
                "apple_music": [],
                "tidal": [],
                "youtube_music": [],
                "deezer": []
            },
            "query": query.q
        }
    })))
}

/// Get all canonical artists
pub async fn get_canonical_artists_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Query(query): Query<SyncHistoryQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        limit = query.limit,
        offset = query.offset,
        "Canonical artists request"
    );

    // Return empty list for now
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "artists": [],
            "total": 0,
            "limit": query.limit,
            "offset": query.offset
        }
    })))
}

/// Get a specific canonical artist
pub async fn get_canonical_artist_handler(
    State(_state): State<AppState>,
    Path(artist_id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(artist_id = %artist_id, "Canonical artist detail request");

    // Return not found for now
    Err(AppError::NotFound {
        resource: "Artist".to_string(),
    })
}

/// Platform health check
pub async fn platform_health_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Platform health check request");

    // Return placeholder health status
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "platforms": {
                "spotify": { "healthy": true, "latency_ms": null },
                "apple_music": { "healthy": true, "latency_ms": null },
                "tidal": { "healthy": true, "latency_ms": null },
                "youtube_music": { "healthy": true, "latency_ms": null },
                "deezer": { "healthy": true, "latency_ms": null }
            },
            "checked_at": chrono::Utc::now().to_rfc3339()
        }
    })))
}

/// Helper to parse platform string to enum
fn parse_platform(s: &str) -> Option<Platform> {
    match s.to_lowercase().as_str() {
        "spotify" => Some(Platform::Spotify),
        "apple_music" | "applemusic" | "apple" => Some(Platform::AppleMusic),
        "tidal" => Some(Platform::Tidal),
        "youtube_music" | "youtubemusic" | "youtube" => Some(Platform::YouTubeMusic),
        "deezer" => Some(Platform::Deezer),
        _ => None,
    }
}
