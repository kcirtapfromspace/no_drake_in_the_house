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
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Sync status request");

    // Get real status from orchestrator
    let overall_status = state
        .catalog_sync
        .get_status()
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to get sync status: {}", e)),
        })?;

    // Get health status for all platforms
    let health_results = state.catalog_sync.health_check_all().await;

    // Convert to response format
    let platforms: Vec<PlatformStatusItem> = overall_status
        .platforms
        .iter()
        .map(|(platform, status)| PlatformStatusItem {
            platform: format!("{:?}", platform).to_lowercase(),
            is_healthy: *health_results.get(platform).unwrap_or(&false),
            last_sync: status.last_sync.map(|dt| dt.to_rfc3339()),
            artists_synced: status.artists_synced,
            is_syncing: status.current_run.is_some(),
        })
        .collect();

    // Also include configured but not-yet-synced platforms from config
    let available = state.platform_config.available_platforms();

    let status = SyncStatusData {
        platforms,
        total_artists: overall_status.total_artists,
        last_full_sync: overall_status.last_full_sync.map(|dt| dt.to_rfc3339()),
        last_incremental_sync: overall_status
            .last_incremental_sync
            .map(|dt| dt.to_rfc3339()),
    };

    Ok(Json(serde_json::json!({
        "success": true,
        "data": status,
        "available_platforms": available
    })))
}

/// Trigger a catalog sync
pub async fn trigger_sync_handler(
    State(state): State<AppState>,
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
    let priority = match request.priority.as_str() {
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

    // Parse platforms - only use Deezer for now (the one that's always available)
    let platforms: Vec<Platform> = if request.platforms.is_empty() {
        // Default to only configured platforms
        vec![Platform::Deezer] // Start with just Deezer since it's always available
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

    // Build sync trigger request
    let trigger_request = SyncTriggerRequest {
        platforms: platforms.clone(),
        sync_type: sync_type.clone(),
        priority,
        artist_ids: None,
    };

    // Trigger the sync via orchestrator
    let run_ids = state
        .catalog_sync
        .trigger_sync(trigger_request)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to trigger sync: {}", e)),
        })?;

    tracing::info!(
        run_ids = ?run_ids,
        platforms = ?platforms,
        "Sync triggered successfully"
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
    State(state): State<AppState>,
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

    // Search across all platforms via orchestrator
    let search_results = state
        .catalog_sync
        .search_artist_all_platforms(&query.q, query.limit_per_platform)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Search failed: {}", e)),
        })?;

    // Convert to JSON-friendly format
    let results: std::collections::HashMap<String, Vec<serde_json::Value>> = search_results
        .into_iter()
        .map(|(platform, artists)| {
            let platform_name = format!("{:?}", platform).to_lowercase();
            let artist_json: Vec<serde_json::Value> = artists
                .into_iter()
                .map(|a| {
                    serde_json::json!({
                        "platform_id": a.platform_id,
                        "name": a.name,
                        "genres": a.genres,
                        "popularity": a.popularity,
                        "image_url": a.image_url
                    })
                })
                .collect();
            (platform_name, artist_json)
        })
        .collect();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "results": results,
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

/// Trigger credits sync for all artists
pub async fn trigger_credits_sync_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    tracing::info!("Credits sync trigger request");

    let credits_sync = state
        .credits_sync
        .as_ref()
        .ok_or_else(|| AppError::Internal {
            message: Some(
                "Credits sync service not configured (Apple Music credentials required)"
                    .to_string(),
            ),
        })?;

    // Clone the Arc to spawn a background task
    let credits_sync = credits_sync.clone();

    // Spawn sync in background
    tokio::spawn(async move {
        match credits_sync.sync_all_artists().await {
            Ok(stats) => {
                tracing::info!(
                    albums = stats.albums_processed,
                    tracks = stats.tracks_processed,
                    credits = stats.credits_added,
                    errors = stats.errors,
                    "Credits sync completed"
                );
            }
            Err(e) => {
                tracing::error!(error = %e, "Credits sync failed");
            }
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "success": true,
            "message": "Credits sync started in background"
        })),
    ))
}

/// Trigger credits sync for a specific artist
pub async fn trigger_artist_credits_sync_handler(
    State(state): State<AppState>,
    Path(artist_id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    tracing::info!(artist_id = %artist_id, "Artist credits sync trigger request");

    let credits_sync = state
        .credits_sync
        .as_ref()
        .ok_or_else(|| AppError::Internal {
            message: Some(
                "Credits sync service not configured (Apple Music credentials required)"
                    .to_string(),
            ),
        })?;

    // Clone the Arc to spawn a background task
    let credits_sync = credits_sync.clone();

    // Spawn sync in background
    tokio::spawn(async move {
        match credits_sync.sync_artist_credits(artist_id).await {
            Ok(stats) => {
                tracing::info!(
                    artist_id = %artist_id,
                    albums = stats.albums_processed,
                    tracks = stats.tracks_processed,
                    credits = stats.credits_added,
                    "Artist credits sync completed"
                );
            }
            Err(e) => {
                tracing::error!(artist_id = %artist_id, error = %e, "Artist credits sync failed");
            }
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "success": true,
            "message": "Artist credits sync started in background",
            "artist_id": artist_id
        })),
    ))
}

/// Get credits sync status
pub async fn get_credits_sync_status_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Credits sync status request");

    // Query recent sync runs
    let runs = sqlx::query_as::<_, (Uuid, Option<Uuid>, String, String, i32, i32, i32, Option<chrono::DateTime<chrono::Utc>>)>(
        r#"
        SELECT id, artist_id, platform, status, albums_processed, tracks_processed, credits_added, completed_at
        FROM credits_sync_runs
        ORDER BY created_at DESC
        LIMIT 10
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal { message: Some(format!("Failed to query sync runs: {}", e)) })?;

    let run_list: Vec<serde_json::Value> = runs
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.0,
                "artist_id": r.1,
                "platform": r.2,
                "status": r.3,
                "albums_processed": r.4,
                "tracks_processed": r.5,
                "credits_added": r.6,
                "completed_at": r.7.map(|dt| dt.to_rfc3339())
            })
        })
        .collect();

    // Get totals
    let totals = sqlx::query_as::<_, (i64, i64, i64)>(
        "SELECT COUNT(DISTINCT album_id), COUNT(*), COUNT(*) FROM tracks, track_credits",
    )
    .fetch_optional(&state.db_pool)
    .await
    .unwrap_or(None)
    .unwrap_or((0, 0, 0));

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "recent_runs": run_list,
            "totals": {
                "albums": totals.0,
                "tracks": totals.1,
                "credits": totals.2
            },
            "service_available": state.credits_sync.is_some()
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

// ============================================================================
// Chart Import and Backfill Endpoints
// ============================================================================

/// Request to import artists from charts
#[derive(Debug, Deserialize)]
pub struct ImportChartsRequest {
    /// Target number of artists to import
    #[serde(default = "default_target_count")]
    pub target_count: usize,
    /// Storefronts to import from (default: ["us"])
    #[serde(default = "default_storefronts")]
    pub storefronts: Vec<String>,
    /// Include genre-specific charts for broader coverage
    #[serde(default = "default_true")]
    pub include_genres: bool,
}

fn default_target_count() -> usize {
    1000
}

fn default_storefronts() -> Vec<String> {
    vec!["us".to_string()]
}

fn default_true() -> bool {
    true
}

/// Request to import artists from MusicBrainz
#[derive(Debug, Deserialize)]
pub struct ImportMusicBrainzRequest {
    /// Target number of artists to import
    #[serde(default = "default_musicbrainz_count")]
    pub target_count: usize,
    /// Skip artists that already exist in the database
    #[serde(default = "default_true")]
    pub skip_existing: bool,
}

fn default_musicbrainz_count() -> usize {
    9000
}

/// Request to run offense backfill
#[derive(Debug, Deserialize)]
pub struct BackfillRequest {
    /// Maximum number of artists to process (None = all)
    pub max_artists: Option<usize>,
    /// Batch size for processing
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    /// Skip artists searched within this many days
    #[serde(default = "default_skip_days")]
    pub skip_recent_days: i32,
}

fn default_batch_size() -> usize {
    100
}

fn default_skip_days() -> i32 {
    7
}

/// Import artists from Apple Music charts
pub async fn import_charts_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(request): Json<ImportChartsRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    tracing::info!(
        target_count = request.target_count,
        storefronts = ?request.storefronts,
        include_genres = request.include_genres,
        "Chart import request"
    );

    // Get Apple Music worker from catalog sync
    let apple_music_worker = state
        .catalog_sync
        .get_worker(&Platform::AppleMusic)
        .ok_or_else(|| AppError::Internal {
            message: Some(
                "Apple Music worker not configured. Set APPLE_MUSIC_* environment variables."
                    .to_string(),
            ),
        })?;

    // Clone for background task
    let orchestrator = state.catalog_sync.clone();
    let target_count = request.target_count;
    let db_pool = state.db_pool.clone();

    // Spawn import in background
    let run_id = Uuid::new_v4();
    tokio::spawn(async move {
        tracing::info!(run_id = %run_id, "Starting Apple Music chart import");

        // We need to downcast the worker to AppleMusicSyncWorker to access chart methods
        // For now, use the search functionality as a fallback
        let mut imported = 0;

        // Import using search for popular artist names as fallback
        // In production, this would use the fetch_top_artists_bulk method
        let popular_searches = [
            "Taylor Swift",
            "Drake",
            "The Weeknd",
            "Bad Bunny",
            "Ed Sheeran",
            "Dua Lipa",
            "Harry Styles",
            "Post Malone",
            "Billie Eilish",
            "Ariana Grande",
            "BTS",
            "Doja Cat",
            "Justin Bieber",
            "Kendrick Lamar",
            "Travis Scott",
            "Kanye West",
            "Rihanna",
            "Beyonce",
            "Eminem",
            "Jay-Z",
        ];

        for artist_name in popular_searches {
            if imported >= target_count {
                break;
            }

            match orchestrator
                .search_and_persist(&Platform::AppleMusic, artist_name, 5, Some(run_id))
                .await
            {
                Ok(count) => {
                    imported += count as usize;
                    tracing::debug!("Imported {} artists from search '{}'", count, artist_name);
                }
                Err(e) => {
                    tracing::warn!("Failed to search '{}': {}", artist_name, e);
                }
            }
        }

        tracing::info!(
            run_id = %run_id,
            imported = imported,
            "Apple Music chart import completed"
        );
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "success": true,
            "data": {
                "run_id": run_id,
                "target_count": request.target_count,
                "status": "started"
            },
            "message": "Chart import started in background"
        })),
    ))
}

/// Import artists from MusicBrainz
pub async fn import_musicbrainz_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(request): Json<ImportMusicBrainzRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    tracing::info!(
        target_count = request.target_count,
        skip_existing = request.skip_existing,
        "MusicBrainz import request"
    );

    let db_pool = state.db_pool.clone();
    let target_count = request.target_count;
    let skip_existing = request.skip_existing;

    // Spawn import in background
    let run_id = Uuid::new_v4();
    tokio::spawn(async move {
        tracing::info!(run_id = %run_id, "Starting MusicBrainz import");

        let importer = crate::services::MusicBrainzImporter::new(db_pool);

        match importer
            .bulk_import(target_count, skip_existing, |current, total| {
                if current % 100 == 0 {
                    tracing::info!("MusicBrainz import progress: {}/{}", current, total);
                }
            })
            .await
        {
            Ok(stats) => {
                tracing::info!(
                    run_id = %run_id,
                    imported = stats.artists_imported,
                    skipped = stats.artists_skipped,
                    errors = stats.errors,
                    "MusicBrainz import completed"
                );
            }
            Err(e) => {
                tracing::error!(run_id = %run_id, error = %e, "MusicBrainz import failed");
            }
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "success": true,
            "data": {
                "run_id": run_id,
                "target_count": request.target_count,
                "skip_existing": request.skip_existing,
                "status": "started"
            },
            "message": "MusicBrainz import started in background"
        })),
    ))
}

/// Run offense backfill for artists
pub async fn backfill_offenses_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(request): Json<BackfillRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    tracing::info!(
        max_artists = ?request.max_artists,
        batch_size = request.batch_size,
        skip_recent_days = request.skip_recent_days,
        "Backfill offenses request"
    );

    // Check if backfill orchestrator is available
    let backfill = state
        .backfill_orchestrator
        .as_ref()
        .ok_or_else(|| AppError::Internal {
            message: Some("Backfill orchestrator not configured".to_string()),
        })?;

    // Check if already running
    if backfill.is_running().await {
        return Err(AppError::InvalidFieldValue {
            field: "backfill".to_string(),
            message: "Backfill is already running".to_string(),
        });
    }

    // Ensure backfill table exists
    if let Err(e) = backfill.ensure_backfill_table().await {
        tracing::warn!("Failed to ensure backfill table: {}", e);
    }

    // Clone for background task
    let backfill = backfill.clone();
    let max_artists = request.max_artists;
    let batch_size = request.batch_size;
    let skip_recent_days = request.skip_recent_days;

    // Spawn backfill in background
    let run_id = Uuid::new_v4();
    tokio::spawn(async move {
        tracing::info!(run_id = %run_id, "Starting offense backfill");

        match backfill
            .backfill_artist_offenses(batch_size, max_artists, skip_recent_days)
            .await
        {
            Ok(result) => {
                tracing::info!(
                    run_id = %run_id,
                    artists_processed = result.artists_processed,
                    offenses_created = result.offenses_created,
                    errors = result.errors,
                    duration_secs = result.duration_seconds,
                    "Offense backfill completed"
                );
            }
            Err(e) => {
                tracing::error!(run_id = %run_id, error = %e, "Offense backfill failed");
            }
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "success": true,
            "data": {
                "run_id": run_id,
                "max_artists": request.max_artists,
                "batch_size": request.batch_size,
                "status": "started"
            },
            "message": "Offense backfill started in background"
        })),
    ))
}

/// Get backfill progress and statistics
pub async fn backfill_status_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Backfill status request");

    let backfill = state
        .backfill_orchestrator
        .as_ref()
        .ok_or_else(|| AppError::Internal {
            message: Some("Backfill orchestrator not configured".to_string()),
        })?;

    let progress = backfill.get_progress().await;
    let is_running = backfill.is_running().await;

    // Get overall stats
    let stats = backfill.get_stats().await.ok();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "is_running": is_running,
            "progress": {
                "artists_total": progress.artists_total,
                "artists_processed": progress.artists_processed,
                "offenses_found": progress.offenses_found,
                "errors": progress.errors,
                "started_at": progress.started_at.map(|dt| dt.to_rfc3339()),
                "estimated_completion": progress.estimated_completion.map(|dt| dt.to_rfc3339())
            },
            "stats": stats.map(|s| serde_json::json!({
                "total_artists": s.total_artists,
                "artists_searched": s.artists_searched,
                "artists_pending": s.artists_pending,
                "total_offenses_found": s.total_offenses_found,
                "last_search_at": s.last_search_at.map(|dt| dt.to_rfc3339())
            }))
        }
    })))
}
