//! Apple Music Authentication Handlers
//!
//! These handlers support the Apple Music MusicKit JS flow:
//! 1. Frontend requests developer token from /api/v1/apple-music/auth/developer-token
//! 2. Frontend initializes MusicKit JS with the developer token
//! 3. User authorizes in the MusicKit JS popup
//! 4. Frontend sends the Music User Token to /api/v1/apple-music/auth/connect
//! 5. Backend stores the token and creates a connection

use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::apple_music::AppleMusicResponse;
use crate::models::offense::ImportTrack;
use crate::models::user::AuthenticatedUser;
use crate::AppState;

fn clean_optional_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[derive(Debug, Serialize)]
pub struct SyncDiffStats {
    pub added: i64,
    pub removed: i64,
    pub unchanged: i64,
    pub total: i64,
    pub is_first_sync: bool,
}

async fn upsert_user_library_cache(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    provider: &str,
    tracks: &[ImportTrack],
) -> Result<SyncDiffStats, AppError> {
    let sync_ts = chrono::Utc::now();

    // Count existing rows to detect first-sync vs subsequent
    let existing_count: i64 = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT COUNT(*) FROM user_library_tracks WHERE user_id = $1 AND provider = $2",
    )
    .bind(user_id)
    .bind(provider)
    .fetch_one(pool)
    .await
    .map_err(AppError::DatabaseQueryFailed)?
    .unwrap_or(0);

    let is_first_sync = existing_count == 0;

    let mut tx = pool.begin().await.map_err(AppError::DatabaseQueryFailed)?;

    if tracks.is_empty() {
        // Delete everything if the provider returned an empty library
        let removed =
            sqlx::query("DELETE FROM user_library_tracks WHERE user_id = $1 AND provider = $2")
                .bind(user_id)
                .bind(provider)
                .execute(&mut *tx)
                .await
                .map_err(AppError::DatabaseQueryFailed)?
                .rows_affected() as i64;

        tx.commit().await.map_err(AppError::DatabaseQueryFailed)?;
        return Ok(SyncDiffStats {
            added: 0,
            removed,
            unchanged: 0,
            total: 0,
            is_first_sync,
        });
    }

    const CHUNK_SIZE: usize = 500;
    let mut upserted: i64 = 0;

    for chunk in tracks.chunks(CHUNK_SIZE) {
        let mut qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
            "INSERT INTO user_library_tracks (user_id, provider, provider_track_id, track_name, album_name, artist_id, artist_name, source_type, playlist_name, added_at, last_synced) ",
        );

        qb.push_values(chunk, |mut b, track| {
            b.push_bind(user_id)
                .push_bind(provider)
                .push_bind(&track.provider_track_id)
                .push_bind(&track.track_name)
                .push_bind(&track.album_name)
                .push_bind(Option::<Uuid>::None)
                .push_bind(&track.artist_name)
                .push_bind(&track.source_type)
                .push_bind(&track.playlist_name)
                .push_bind(track.added_at)
                .push_bind(sync_ts);
        });

        qb.push(
            " ON CONFLICT (user_id, provider, provider_track_id) DO UPDATE SET \
              track_name = EXCLUDED.track_name, \
              album_name = EXCLUDED.album_name, \
              artist_name = EXCLUDED.artist_name, \
              source_type = EXCLUDED.source_type, \
              playlist_name = EXCLUDED.playlist_name, \
              added_at = COALESCE(user_library_tracks.added_at, EXCLUDED.added_at), \
              last_synced = EXCLUDED.last_synced",
        );

        qb.build()
            .execute(&mut *tx)
            .await
            .map_err(AppError::DatabaseQueryFailed)?;

        upserted += chunk.len() as i64;
    }

    // Delete stale items that were not seen in this sync batch
    let removed = sqlx::query(
        "DELETE FROM user_library_tracks WHERE user_id = $1 AND provider = $2 AND last_synced < $3",
    )
    .bind(user_id)
    .bind(provider)
    .bind(sync_ts)
    .execute(&mut *tx)
    .await
    .map_err(AppError::DatabaseQueryFailed)?
    .rows_affected() as i64;

    tx.commit().await.map_err(AppError::DatabaseQueryFailed)?;

    let total = upserted - removed;
    let added = if is_first_sync {
        upserted
    } else {
        (upserted - existing_count + removed).max(0)
    };
    let unchanged = if is_first_sync {
        0
    } else {
        (existing_count - removed).max(0)
    };

    Ok(SyncDiffStats {
        added,
        removed,
        unchanged,
        total,
        is_first_sync,
    })
}

fn parse_retry_after_seconds(error_message: &str) -> Option<u64> {
    let tokens: Vec<&str> = error_message.split_whitespace().collect();

    for (idx, token) in tokens.iter().enumerate() {
        let numeric = token.trim_matches(|c: char| !c.is_ascii_digit());
        if numeric.is_empty() {
            continue;
        }

        let Ok(seconds) = numeric.parse::<u64>() else {
            continue;
        };

        let next = tokens.get(idx + 1).map(|value| {
            value
                .trim_matches(|c: char| !c.is_ascii_alphabetic())
                .to_ascii_lowercase()
        });

        if matches!(next.as_deref(), Some("second") | Some("seconds")) {
            return Some(seconds);
        }
    }

    None
}

fn map_library_error(error_message: &str, context: &str) -> AppError {
    let lowered = error_message.to_ascii_lowercase();
    if lowered.contains("data key not found")
        || lowered.contains("failed to decrypt data key")
        || lowered.contains("failed to decrypt token")
    {
        AppError::ExternalServiceError(
            "Apple Music credentials need re-authorization. Disconnect and reconnect Apple Music, then try again."
                .to_string(),
        )
    } else if lowered.contains("rate limited") || lowered.contains("too many requests") {
        AppError::RateLimitExceeded {
            retry_after: parse_retry_after_seconds(error_message),
        }
    } else if lowered.contains("cloudlibrary")
        || lowered.contains("insufficient privileges")
        || lowered.contains("\"code\":\"40015\"")
    {
        AppError::OperationNotAllowed {
            reason: "Apple Music library access is unavailable for this account. Enable Sync Library in Apple Music settings and verify the account has an active Apple Music subscription."
                .to_string(),
        }
    } else if lowered.contains("401")
        || lowered.contains("403")
        || lowered.contains("unauthorized")
        || lowered.contains("forbidden")
    {
        AppError::ExternalServiceError(
            "Apple Music authorization failed. Disconnect and reconnect Apple Music, then try again."
                .to_string(),
        )
    } else if lowered.contains("timed out") {
        AppError::ExternalServiceUnavailable {
            service: "Apple Music".to_string(),
        }
    } else {
        AppError::ExternalServiceError(format!(
            "Apple Music library {} failed: {}",
            context, error_message
        ))
    }
}

fn map_connect_error(error_message: &str) -> AppError {
    let lowered = error_message.to_ascii_lowercase();
    if lowered.contains("rate limited") || lowered.contains("too many requests") {
        AppError::RateLimitExceeded {
            retry_after: parse_retry_after_seconds(error_message),
        }
    } else if lowered.contains("cloudlibrary")
        || lowered.contains("insufficient privileges")
        || lowered.contains("\"code\":\"40015\"")
    {
        AppError::OperationNotAllowed {
            reason: "Apple Music library access is unavailable for this account. Enable Sync Library in Apple Music settings and verify the account has an active Apple Music subscription."
                .to_string(),
        }
    } else if lowered.contains("invalid apple music user token")
        || lowered.contains("401")
        || lowered.contains("403")
        || lowered.contains("unauthorized")
        || lowered.contains("forbidden")
    {
        AppError::ExternalServiceError(
            "Apple Music authorization failed. Complete the Apple Music sign-in prompt and try again."
                .to_string(),
        )
    } else if lowered.contains("timed out") {
        AppError::ExternalServiceUnavailable {
            service: "Apple Music".to_string(),
        }
    } else {
        AppError::ExternalServiceError(format!("Failed to connect Apple Music: {}", error_message))
    }
}

fn library_scan_timeout() -> Duration {
    let seconds = std::env::var("APPLE_MUSIC_LIBRARY_TIMEOUT_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(300);
    Duration::from_secs(seconds)
}

/// Response containing the developer token for MusicKit JS initialization
#[derive(Debug, Serialize)]
pub struct DeveloperTokenResponse {
    pub developer_token: String,
    pub expires_at: String,
}

/// Request to connect Apple Music account
#[derive(Debug, Deserialize)]
pub struct ConnectAppleMusicRequest {
    /// The Music User Token obtained from MusicKit JS after user authorization
    pub music_user_token: String,
}

/// Response after successfully connecting Apple Music
#[derive(Debug, Serialize)]
pub struct ConnectAppleMusicResponse {
    pub success: bool,
    pub connection_id: Uuid,
    pub message: String,
}

/// Response for Apple Music connection status
#[derive(Debug, Serialize)]
pub struct AppleMusicConnectionStatus {
    pub connected: bool,
    pub connection_id: Option<Uuid>,
    pub last_health_check: Option<String>,
    pub status: Option<String>,
}

/// Get developer token for MusicKit JS initialization
///
/// GET /api/v1/apple-music/auth/developer-token
///
/// Returns a developer token that the frontend uses to initialize MusicKit JS.
/// This token is signed with the Apple Music API key and allows the frontend
/// to request user authorization.
pub async fn get_developer_token(
    State(state): State<AppState>,
) -> Result<Json<DeveloperTokenResponse>, AppError> {
    let developer_token = state
        .apple_music_service
        .generate_developer_token()
        .await
        .map_err(|e| {
            let message = e.to_string();
            if message.contains("Apple Music is not configured") || message.contains("APPLE_MUSIC_")
            {
                AppError::ConfigurationError { message }
            } else {
                AppError::Internal {
                    message: Some(format!("Failed to generate developer token: {}", message)),
                }
            }
        })?;

    Ok(Json(DeveloperTokenResponse {
        developer_token: developer_token.token,
        expires_at: developer_token.expires_at.to_rfc3339(),
    }))
}

/// Connect user's Apple Music account
///
/// POST /api/v1/apple-music/auth/connect
///
/// After the user authorizes in MusicKit JS, the frontend sends the
/// Music User Token here. We validate and store it.
pub async fn connect_apple_music(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Json(request): Json<ConnectAppleMusicRequest>,
) -> Result<Json<ConnectAppleMusicResponse>, AppError> {
    let user_id = authenticated_user.id;
    if request.music_user_token.trim().is_empty() {
        return Err(AppError::InvalidFieldValue {
            field: "music_user_token".to_string(),
            message: "Music user token is required".to_string(),
        });
    }

    // Create the connection using the Music User Token
    let connection = state
        .apple_music_service
        .create_connection(
            user_id,
            request.music_user_token.clone(),
            Some(request.music_user_token), // Store as both access and refresh
        )
        .await
        .map_err(|e| map_connect_error(&e.to_string()))?;

    Ok(Json(ConnectAppleMusicResponse {
        success: true,
        connection_id: connection.id,
        message: "Apple Music account connected successfully".to_string(),
    }))
}

/// Get Apple Music connection status
///
/// GET /api/v1/apple-music/auth/status
pub async fn get_connection_status(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<Json<AppleMusicConnectionStatus>, AppError> {
    let user_id = authenticated_user.id;

    let connection = state
        .apple_music_service
        .get_user_connection(user_id)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to check connection status: {}", e)),
        })?;

    match connection {
        Some(conn) => Ok(Json(AppleMusicConnectionStatus {
            connected: true,
            connection_id: Some(conn.id),
            last_health_check: conn.last_health_check.map(|t| t.to_rfc3339()),
            status: Some(format!("{:?}", conn.status)),
        })),
        None => Ok(Json(AppleMusicConnectionStatus {
            connected: false,
            connection_id: None,
            last_health_check: None,
            status: None,
        })),
    }
}

/// Disconnect Apple Music account
///
/// DELETE /api/v1/apple-music/auth/disconnect
pub async fn disconnect_apple_music(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = authenticated_user.id;

    state
        .apple_music_service
        .disconnect_user(user_id)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to disconnect Apple Music: {}", e)),
        })?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Apple Music account disconnected"
    })))
}

/// Verify Apple Music connection health
///
/// POST /api/v1/apple-music/auth/verify
pub async fn verify_connection(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = authenticated_user.id;

    let connection = state
        .apple_music_service
        .get_user_connection(user_id)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to get connection: {}", e)),
        })?;

    match connection {
        Some(conn) => {
            let health = state
                .apple_music_service
                .check_token_health(&conn)
                .await
                .map_err(|e| AppError::Internal {
                    message: Some(format!("Failed to verify connection: {}", e)),
                })?;

            Ok(Json(serde_json::json!({
                "healthy": health.is_valid,
                "last_check": health.checked_at.to_rfc3339(),
                "needs_refresh": health.needs_refresh,
                "error": health.error_message
            })))
        }
        None => Err(AppError::NotFound {
            resource: format!("Apple Music connection for user {}", user_id),
        }),
    }
}

/// Response for library sync operation
#[derive(Debug, Serialize)]
pub struct LibrarySyncResponse {
    pub success: bool,
    pub tracks_count: usize,
    pub albums_count: usize,
    pub playlists_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imported_items_count: Option<i64>,
    pub is_first_sync: bool,
    pub added: i64,
    pub removed: i64,
    pub unchanged: i64,
    pub message: String,
}

/// Sync user's Apple Music library
///
/// POST /api/v1/apple-music/library/sync
///
/// Fetches the user's Apple Music library (tracks, albums, playlists)
/// and caches it for analysis against the DNP list.
pub async fn sync_library(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<Json<LibrarySyncResponse>, AppError> {
    let user_id = authenticated_user.id;
    let scan_timeout = library_scan_timeout();

    tracing::info!(user_id = %user_id, "Starting Apple Music library sync");

    // Get user's Apple Music connection
    let connection = state
        .apple_music_service
        .get_user_connection(user_id)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to get connection: {}", e)),
        })?
        .ok_or_else(|| AppError::NotFound {
            resource: "Apple Music connection".to_string(),
        })?;

    // Scan the library from Apple Music
    let library = tokio::time::timeout(
        scan_timeout,
        state.apple_music_service.scan_library(&connection),
    )
    .await
    .map_err(|_| {
        let error_message = format!(
            "Timed out waiting for Apple Music library sync after {} seconds",
            scan_timeout.as_secs()
        );
        tracing::warn!(
            user_id = %user_id,
            timeout_seconds = scan_timeout.as_secs(),
            "Apple Music library sync timed out"
        );
        map_library_error(&error_message, "sync")
    })?
    .map_err(|e| {
        let error_message = e.to_string();
        tracing::warn!(
            user_id = %user_id,
            error = %error_message,
            "Apple Music library sync failed"
        );

        map_library_error(&error_message, "sync")
    })?;

    let tracks_count = library.library_tracks.len();
    let albums_count = library.library_albums.len();
    let playlists_count = library.library_playlists.len();

    tracing::info!(
        user_id = %user_id,
        tracks = tracks_count,
        albums = albums_count,
        playlists = playlists_count,
        "Apple Music library scan complete"
    );

    // Cache the full library in `user_library_tracks` so the frontend can display items and
    // other services (scan/enforcement) can run without re-fetching the provider each time.
    let mut import_tracks: Vec<ImportTrack> =
        Vec::with_capacity(tracks_count + albums_count + playlists_count);

    for track in &library.library_tracks {
        let attrs = &track.attributes;
        import_tracks.push(ImportTrack {
            provider_track_id: format!("song:{}", track.id),
            track_name: attrs.name.clone(),
            album_name: clean_optional_string(&attrs.album_name),
            artist_name: attrs.artist_name.clone(),
            source_type: Some("library_song".to_string()),
            playlist_name: None,
            added_at: None,
        });
    }

    for album in &library.library_albums {
        let attrs = &album.attributes;
        import_tracks.push(ImportTrack {
            provider_track_id: format!("album:{}", album.id),
            track_name: attrs.name.clone(),
            album_name: None,
            artist_name: attrs.artist_name.clone(),
            source_type: Some("library_album".to_string()),
            playlist_name: None,
            added_at: None,
        });
    }

    for playlist in &library.library_playlists {
        let attrs = &playlist.attributes;
        import_tracks.push(ImportTrack {
            provider_track_id: format!("playlist:{}", playlist.id),
            track_name: attrs.name.clone(),
            album_name: attrs.track_count.map(|count| format!("{} tracks", count)),
            artist_name: attrs
                .curator_name
                .as_deref()
                .unwrap_or("Apple Music")
                .to_string(),
            source_type: Some("library_playlist".to_string()),
            playlist_name: clean_optional_string(&attrs.name),
            added_at: attrs.last_modified_date,
        });
    }

    let diff =
        upsert_user_library_cache(&state.db_pool, user_id, "apple_music", &import_tracks).await?;

    let message = if diff.is_first_sync {
        format!(
            "Library imported! {} items added ({} songs, {} albums, {} playlists).",
            diff.added, tracks_count, albums_count, playlists_count
        )
    } else if diff.added == 0 && diff.removed == 0 {
        "Library is up to date. No changes.".to_string()
    } else {
        let mut parts = Vec::new();
        if diff.added > 0 {
            parts.push(format!("{} added", diff.added));
        }
        if diff.removed > 0 {
            parts.push(format!("{} removed", diff.removed));
        }
        format!("Synced! {}", parts.join(", "))
    };

    tracing::info!(
        user_id = %user_id,
        is_first_sync = diff.is_first_sync,
        added = diff.added,
        removed = diff.removed,
        unchanged = diff.unchanged,
        total = diff.total,
        "Apple Music library sync complete"
    );

    Ok(Json(LibrarySyncResponse {
        success: true,
        tracks_count,
        albums_count,
        playlists_count,
        imported_items_count: Some(diff.total),
        is_first_sync: diff.is_first_sync,
        added: diff.added,
        removed: diff.removed,
        unchanged: diff.unchanged,
        message,
    }))
}

/// Get user's cached Apple Music library
///
/// GET /api/v1/apple-music/library
pub async fn get_library(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Query(query): Query<AppleMusicLibraryQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = authenticated_user.id;
    let scan_timeout = library_scan_timeout();
    let preview_limit = query.limit.unwrap_or(25).clamp(1, 100);

    // Get user's Apple Music connection
    let connection = state
        .apple_music_service
        .get_user_connection(user_id)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to get connection: {}", e)),
        })?
        .ok_or_else(|| AppError::NotFound {
            resource: "Apple Music connection".to_string(),
        })?;

    // Fast preview: fetch only the first page of each library resource and rely on `meta.total`
    // for accurate counts. This keeps the UI responsive for large libraries.
    let preview_result = tokio::time::timeout(scan_timeout, async {
        use crate::models::apple_music::{
            AppleMusicArtistAttributes, AppleMusicLibraryAlbum, AppleMusicLibraryPlaylist,
            AppleMusicLibraryResource, AppleMusicLibraryTrack,
        };

        fn is_cloud_library_privilege_error(
            status: reqwest::StatusCode,
            response_body: &str,
        ) -> bool {
            if status != reqwest::StatusCode::BAD_REQUEST
                && status != reqwest::StatusCode::FORBIDDEN
            {
                return false;
            }

            let lowered = response_body.to_ascii_lowercase();
            lowered.contains("cloudlibrary")
                || lowered.contains("insufficient privileges")
                || lowered.contains("\"code\":\"40015\"")
        }

        async fn fetch_page<T: for<'de> serde::Deserialize<'de>>(
            service: &ndith_services::apple_music::AppleMusicService,
            connection: &ndith_core::models::token_vault::Connection,
            endpoint: &str,
            allow_cloud_library_empty: bool,
        ) -> Result<AppleMusicResponse<T>, anyhow::Error> {
            let response = service
                .make_api_request(connection, "GET", endpoint, None)
                .await?;
            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                if allow_cloud_library_empty
                    && is_cloud_library_privilege_error(status, &error_text)
                {
                    return Ok(AppleMusicResponse {
                        data: Vec::new(),
                        href: None,
                        next: None,
                        meta: None,
                    });
                }
                return Err(anyhow::anyhow!("{} - {}", status, error_text));
            }
            Ok(response.json::<AppleMusicResponse<T>>().await?)
        }

        // Songs sample + total
        let songs_endpoint = format!("/v1/me/library/songs?limit={}", preview_limit);
        let albums_endpoint = format!("/v1/me/library/albums?limit={}", preview_limit);
        let artists_endpoint = "/v1/me/library/artists?limit=1".to_string();
        let playlists_endpoint = format!("/v1/me/library/playlists?limit={}", preview_limit);

        let songs_fut = fetch_page::<AppleMusicLibraryTrack>(
            &state.apple_music_service,
            &connection,
            &songs_endpoint,
            false,
        );
        let albums_fut = fetch_page::<AppleMusicLibraryAlbum>(
            &state.apple_music_service,
            &connection,
            &albums_endpoint,
            false,
        );
        let artists_fut = fetch_page::<AppleMusicLibraryResource<AppleMusicArtistAttributes>>(
            &state.apple_music_service,
            &connection,
            &artists_endpoint,
            false,
        );
        let playlists_fut = fetch_page::<AppleMusicLibraryPlaylist>(
            &state.apple_music_service,
            &connection,
            &playlists_endpoint,
            true,
        );

        let (songs, albums, artists, playlists) =
            tokio::try_join!(songs_fut, albums_fut, artists_fut, playlists_fut)?;

        Ok::<_, anyhow::Error>((songs, albums, artists, playlists, chrono::Utc::now()))
    })
    .await
    .map_err(|_| {
        let error_message = format!(
            "Timed out waiting for Apple Music library fetch after {} seconds",
            scan_timeout.as_secs()
        );
        tracing::warn!(
            user_id = %user_id,
            timeout_seconds = scan_timeout.as_secs(),
            "Apple Music library fetch timed out"
        );
        map_library_error(&error_message, "fetch")
    })?
    .map_err(|e| {
        let error_message = e.to_string();
        tracing::warn!(
            user_id = %user_id,
            error = %error_message,
            "Apple Music library fetch failed"
        );

        map_library_error(&error_message, "fetch")
    })?;

    let (songs_response, albums_response, artists_response, playlists_response, scanned_at) =
        preview_result;

    let tracks_total = songs_response
        .meta
        .as_ref()
        .and_then(|meta| meta.total)
        .unwrap_or(songs_response.data.len() as u32);
    let albums_total = albums_response
        .meta
        .as_ref()
        .and_then(|meta| meta.total)
        .unwrap_or(albums_response.data.len() as u32);
    let artists_total = artists_response
        .meta
        .as_ref()
        .and_then(|meta| meta.total)
        .unwrap_or(artists_response.data.len() as u32);
    let playlists_total = playlists_response
        .meta
        .as_ref()
        .and_then(|meta| meta.total)
        .unwrap_or(playlists_response.data.len() as u32);

    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "tracks": songs_response.data.iter().map(|t| serde_json::json!({
            "id": t.id,
            "name": t.attributes.name,
            "artist": t.attributes.artist_name,
            "album": t.attributes.album_name,
        })).collect::<Vec<_>>(),
        "albums": albums_response.data.iter().map(|a| serde_json::json!({
            "id": a.id,
            "name": a.attributes.name,
            "artist": a.attributes.artist_name,
        })).collect::<Vec<_>>(),
        "playlists": playlists_response.data.iter().map(|p| serde_json::json!({
            "id": p.id,
            "name": p.attributes.name,
            "track_count": p.attributes.track_count,
        })).collect::<Vec<_>>(),
        "tracks_total": tracks_total,
        "albums_total": albums_total,
        "artists_total": artists_total,
        "playlists_total": playlists_total,
        "preview_limit": preview_limit,
        "scanned_at": scanned_at.to_rfc3339(),
    })))
}

#[derive(Debug, Deserialize)]
pub struct AppleMusicLibraryQuery {
    pub limit: Option<u32>,
}
