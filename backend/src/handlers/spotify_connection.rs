//! Spotify Connection Handlers
//!
//! Handles Spotify OAuth flow for provider connection (not authentication).
//! This allows users to connect their Spotify account for DNP list enforcement.
//! All external Spotify API calls are wrapped with a circuit breaker (US-026).

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use base64::{engine::general_purpose, Engine as _};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::handlers::provider_library_sync_status::{
    get_provider_library_sync_status, imported_items_count, store_provider_library_sync_status,
    ProviderLibrarySyncCounts, ProviderLibrarySyncStatus,
    PROVIDER_LIBRARY_SYNC_RUNNING_TTL_SECONDS, PROVIDER_LIBRARY_SYNC_STATUS_TTL_SECONDS,
};
use crate::models::oauth::{OAuthConfig, OAuthProviderType};
use crate::models::offense::{ImportLibraryRequest, ImportTrack};
use crate::models::playlist::{UpsertPlaylist, UpsertPlaylistTrack};
use crate::models::user::AuthenticatedUser;
use crate::services::oauth::OAuthProvider;
use crate::services::oauth_spotify::SpotifyOAuthProvider;
use crate::services::OAuthTokenEncryption;
use crate::services::OffenseService;
use crate::services::PlaylistRepository;
use crate::AppState;
use ndith_core::config::provider_callback_uri_with_override;
use std::collections::HashMap;

const SPOTIFY_SYNC_STATUS_KEY: &str = "spotify";
const SPOTIFY_PROVIDER_LABEL: &str = "Spotify";

/// Required Spotify scopes for DNP enforcement
pub const SPOTIFY_CONNECTION_SCOPES: &[&str] = &[
    "user-library-read",
    "user-library-modify",
    "playlist-read-private",
    "playlist-read-collaborative",
    "playlist-modify-private",
    "user-follow-read",
    "user-follow-modify",
];

/// Create a Spotify OAuth provider configured with connection-specific scopes
fn create_connection_provider() -> Result<SpotifyOAuthProvider> {
    let client_id =
        std::env::var("SPOTIFY_CLIENT_ID").map_err(|_| AppError::ConfigurationError {
            message: "SPOTIFY_CLIENT_ID environment variable is required".to_string(),
        })?;
    let client_secret =
        std::env::var("SPOTIFY_CLIENT_SECRET").map_err(|_| AppError::ConfigurationError {
            message: "SPOTIFY_CLIENT_SECRET environment variable is required".to_string(),
        })?;
    let redirect_uri =
        provider_callback_uri_with_override("spotify", &["SPOTIFY_REDIRECT_URI"]);

    let config = OAuthConfig {
        client_id,
        client_secret,
        redirect_uri,
        scopes: SPOTIFY_CONNECTION_SCOPES
            .iter()
            .map(|s| s.to_string())
            .collect(),
        additional_params: HashMap::new(),
    };

    Ok(SpotifyOAuthProvider::from_config(config))
}

/// Query parameters for the authorize endpoint
#[derive(Debug, Deserialize)]
pub struct SpotifyAuthorizeQuery {
    /// Optional redirect URI override
    pub redirect_uri: Option<String>,
}

/// Response from the authorize endpoint
#[derive(Debug, Serialize)]
pub struct SpotifyAuthorizeResponse {
    pub authorization_url: String,
    pub state: String,
    pub scopes: Vec<String>,
}

/// Request body for the callback endpoint
#[derive(Debug, Deserialize)]
pub struct SpotifyCallbackRequest {
    pub code: String,
    pub state: String,
}

/// Response from the callback endpoint
#[derive(Debug, Serialize)]
pub struct SpotifyCallbackResponse {
    pub success: bool,
    pub connection_id: Uuid,
    pub provider_user_id: String,
    pub status: String,
    pub message: String,
    pub sync_summary: Option<SpotifyLibrarySyncSummary>,
    pub sync_warning: Option<String>,
}

/// Connection status response
#[derive(Debug, Serialize)]
pub struct SpotifyConnectionStatus {
    pub connected: bool,
    pub connection_id: Option<Uuid>,
    pub provider_user_id: Option<String>,
    pub status: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub expires_at: Option<String>,
    pub last_health_check: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpotifyLibrarySyncSummary {
    pub imported_tracks: i32,
    pub liked_tracks_synced: usize,
    pub playlist_tracks_synced: usize,
    pub playlists_synced: usize,
    pub saved_albums_synced: usize,
    pub followed_artists_synced: usize,
}

#[derive(Debug, Serialize)]
pub struct SpotifyLibrarySyncResponse {
    pub success: bool,
    pub summary: SpotifyLibrarySyncSummary,
    pub message: String,
}

/// OAuth state stored in Redis/memory for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OAuthStateData {
    pub user_id: Uuid,
    pub redirect_uri: String,
    pub created_at: chrono::DateTime<Utc>,
}

/// GET /api/v1/connections/spotify/authorize
///
/// Initiates Spotify OAuth flow for provider connection.
/// Returns an authorization URL that the user should be redirected to.
pub async fn spotify_authorize_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Query(query): Query<SpotifyAuthorizeQuery>,
) -> Result<(StatusCode, Json<SpotifyAuthorizeResponse>)> {
    tracing::info!(
        user_id = %authenticated_user.id,
        "Initiating Spotify connection OAuth flow"
    );

    // If a connection exists, allow reconnect to refresh stale/expired tokens.
    let existing_connection =
        get_user_spotify_connection(&state.db_pool, authenticated_user.id).await?;
    if let Some(conn) = existing_connection {
        if conn.status == "active" {
            tracing::info!(
                user_id = %authenticated_user.id,
                connection_id = %conn.id,
                "Existing active Spotify connection found; proceeding with reconnect flow"
            );
        }
    }

    // Create Spotify OAuth provider with connection-specific scopes
    let spotify_provider = create_connection_provider().map_err(|e| {
        tracing::error!(error = %e, "Failed to create Spotify OAuth provider");
        AppError::ConfigurationError {
            message: "Spotify OAuth is not properly configured".to_string(),
        }
    })?;

    // Determine redirect URI (override if provided in query)
    let default_redirect_uri =
        provider_callback_uri_with_override("spotify", &["SPOTIFY_REDIRECT_URI"]);
    let redirect_uri = match query.redirect_uri {
        Some(ref uri) if uri != &default_redirect_uri => {
            tracing::warn!(
                user_id = %authenticated_user.id,
                requested_uri = %uri,
                expected_uri = %default_redirect_uri,
                "Rejected non-allowlisted Spotify redirect_uri"
            );
            return Err(AppError::InvalidFieldValue {
                field: "redirect_uri".to_string(),
                message: "Provided redirect_uri does not match the configured callback URL"
                    .to_string(),
            });
        }
        _ => default_redirect_uri,
    };

    // Initiate OAuth flow
    let flow_response = spotify_provider
        .initiate_flow(&redirect_uri)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to initiate Spotify OAuth flow");
            AppError::ExternalServiceError(format!("Failed to initiate Spotify OAuth: {}", e))
        })?;

    // Store state in Redis for validation during callback
    let state_data = OAuthStateData {
        user_id: authenticated_user.id,
        redirect_uri: redirect_uri.clone(),
        created_at: Utc::now(),
    };

    // Store state data in Redis with 10 minute expiration
    store_oauth_state(&state.redis_pool, &flow_response.state, &state_data).await?;

    tracing::info!(
        user_id = %authenticated_user.id,
        state = %flow_response.state,
        "Spotify OAuth flow initiated successfully"
    );

    Ok((
        StatusCode::OK,
        Json(SpotifyAuthorizeResponse {
            authorization_url: flow_response.authorization_url,
            state: flow_response.state,
            scopes: SPOTIFY_CONNECTION_SCOPES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }),
    ))
}

/// POST /api/v1/connections/spotify/callback
///
/// Handles the OAuth callback from Spotify.
/// Exchanges the authorization code for tokens and stores the connection.
pub async fn spotify_callback_handler(
    State(state): State<AppState>,
    Json(request): Json<SpotifyCallbackRequest>,
) -> Result<(StatusCode, Json<SpotifyCallbackResponse>)> {
    tracing::info!(
        state = %request.state,
        code_length = request.code.len(),
        "Processing Spotify connection callback"
    );

    // Validate inputs
    if request.code.is_empty() {
        return Err(AppError::InvalidFieldValue {
            field: "code".to_string(),
            message: "Authorization code is required".to_string(),
        });
    }

    if request.state.is_empty() {
        return Err(AppError::InvalidFieldValue {
            field: "state".to_string(),
            message: "State parameter is required for security".to_string(),
        });
    }

    // Retrieve and validate state from Redis
    let state_data = get_oauth_state(&state.redis_pool, &request.state).await?;

    // Check if state has expired (10 minute validity)
    let state_age = Utc::now() - state_data.created_at;
    if state_age > Duration::minutes(10) {
        return Err(AppError::InvalidFieldValue {
            field: "state".to_string(),
            message: "OAuth state has expired. Please try again.".to_string(),
        });
    }

    // NOTE: State is deleted from Redis after successful connection storage (below)
    // to allow retries if the token exchange or user info fetch fails transiently.

    // Create Spotify OAuth provider with connection-specific scopes
    let spotify_provider = create_connection_provider().map_err(|e| {
        tracing::error!(error = %e, "Failed to create Spotify OAuth provider");
        AppError::ConfigurationError {
            message: "Spotify OAuth is not properly configured".to_string(),
        }
    })?;

    // Exchange code for tokens through circuit breaker (US-026)
    let code = request.code.clone();
    let oauth_state = request.state.clone();
    let redirect_uri = state_data.redirect_uri.clone();
    let tokens = state
        .circuit_breaker
        .execute_typed(OAuthProviderType::Spotify, || async {
            spotify_provider
                .exchange_code(&code, &oauth_state, &redirect_uri)
                .await
        })
        .await?;

    // Get user info from Spotify — non-fatal if rate-limited, since we already
    // have valid tokens and can fetch the profile later.
    let access_token = tokens.access_token.clone();
    let user_info = match state
        .circuit_breaker
        .execute_typed(OAuthProviderType::Spotify, || async {
            spotify_provider.get_user_info(&access_token).await
        })
        .await
    {
        Ok(info) => Some(info),
        Err(e) => {
            tracing::warn!(error = %e, "Failed to fetch Spotify user info (storing None); tokens are valid");
            None
        }
    };

    // Encrypt tokens using OAuthTokenEncryption
    let encryption = OAuthTokenEncryption::new().map_err(|e| {
        tracing::error!(error = %e, "Failed to initialize token encryption");
        AppError::Internal {
            message: Some("Token encryption not available".to_string()),
        }
    })?;

    let access_token_encrypted = encryption
        .encrypt_token(&tokens.access_token)
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to encrypt access token");
            AppError::Internal {
                message: Some("Failed to secure tokens".to_string()),
            }
        })?;

    let refresh_token_encrypted = if let Some(ref refresh_token) = tokens.refresh_token {
        Some(encryption.encrypt_token(refresh_token).map_err(|e| {
            tracing::error!(error = %e, "Failed to encrypt refresh token");
            AppError::Internal {
                message: Some("Failed to secure tokens".to_string()),
            }
        })?)
    } else {
        None
    };

    // Calculate token expiration
    let expires_at = tokens
        .expires_in
        .map(|secs| Utc::now() + Duration::seconds(secs));

    // Store connection in database
    let provider_user_id = user_info
        .as_ref()
        .map(|info| info.provider_user_id.as_str());
    let connection_id = store_spotify_connection(
        &state.db_pool,
        state_data.user_id,
        provider_user_id,
        &access_token_encrypted,
        refresh_token_encrypted.as_deref(),
        expires_at,
    )
    .await?;

    // Now that the connection is stored, delete the OAuth state to prevent replay attacks.
    // Non-fatal: the connection is already saved, so a Redis failure shouldn't abort the flow.
    if let Err(e) = delete_oauth_state(&state.redis_pool, &request.state).await {
        tracing::warn!("Failed to delete OAuth state: {}", e);
    }

    // Spawn library sync as a background task to avoid Cloudflare 524 timeouts.
    // The sync involves many paginated Spotify API calls and can take minutes.
    let db_pool = state.db_pool.clone();
    let user_id = state_data.user_id;
    let access_token = tokens.access_token.clone();
    tokio::spawn(async move {
        match sync_spotify_library_to_user_library(&db_pool, user_id, &access_token).await {
            Ok(summary) => {
                tracing::info!(
                    user_id = %user_id,
                    liked = summary.liked_tracks_synced,
                    playlists = summary.playlist_tracks_synced,
                    albums = summary.saved_albums_synced,
                    artists = summary.followed_artists_synced,
                    imported = summary.imported_tracks,
                    "Background Spotify library sync completed"
                );
            }
            Err(e) => {
                tracing::warn!(
                    user_id = %user_id,
                    error = %e,
                    "Background Spotify library sync failed"
                );
            }
        }
    });
    let sync_summary = None;
    let sync_warning =
        Some("Library sync started in background. Refresh to see progress.".to_string());

    let provider_user_id_display = provider_user_id.unwrap_or("unknown").to_string();

    tracing::info!(
        user_id = %state_data.user_id,
        connection_id = %connection_id,
        provider_user_id = %provider_user_id_display,
        "Spotify connection created successfully"
    );

    Ok((
        StatusCode::OK,
        Json(SpotifyCallbackResponse {
            success: true,
            connection_id,
            provider_user_id: provider_user_id_display,
            status: "active".to_string(),
            message: "Spotify account connected successfully".to_string(),
            sync_summary,
            sync_warning,
        }),
    ))
}

/// GET /api/v1/connections/spotify/status
///
/// Returns the status of the user's Spotify connection.
pub async fn spotify_connection_status_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<(StatusCode, Json<SpotifyConnectionStatus>)> {
    tracing::debug!(
        user_id = %authenticated_user.id,
        "Checking Spotify connection status"
    );

    let connection = get_user_spotify_connection(&state.db_pool, authenticated_user.id).await?;

    match connection {
        Some(conn) => Ok((
            StatusCode::OK,
            Json(SpotifyConnectionStatus {
                connected: conn.status == "active",
                connection_id: Some(conn.id),
                provider_user_id: conn.provider_user_id,
                status: Some(conn.status),
                scopes: conn.scopes,
                expires_at: conn.expires_at.map(|t| t.to_rfc3339()),
                last_health_check: conn.last_health_check.map(|t| t.to_rfc3339()),
            }),
        )),
        None => Ok((
            StatusCode::OK,
            Json(SpotifyConnectionStatus {
                connected: false,
                connection_id: None,
                provider_user_id: None,
                status: None,
                scopes: None,
                expires_at: None,
                last_health_check: None,
            }),
        )),
    }
}

/// DELETE /api/v1/connections/spotify
///
/// Disconnects the user's Spotify account.
pub async fn spotify_disconnect_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    tracing::info!(
        user_id = %authenticated_user.id,
        "Disconnecting Spotify account"
    );

    let connection = get_user_spotify_connection(&state.db_pool, authenticated_user.id).await?;

    match connection {
        Some(conn) => {
            // Delete the connection
            delete_spotify_connection(&state.db_pool, conn.id).await?;

            tracing::info!(
                user_id = %authenticated_user.id,
                connection_id = %conn.id,
                "Spotify connection deleted"
            );

            Ok((
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "message": "Spotify account disconnected successfully"
                })),
            ))
        }
        None => Err(AppError::NotFound {
            resource: "Spotify connection".to_string(),
        }),
    }
}

/// POST /api/v1/connections/spotify/library/sync
///
/// Fetches Spotify liked songs, playlist tracks, followed artists, and saved albums,
/// then imports them into `user_library_tracks` for offense scanning.
pub async fn spotify_library_sync_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<(StatusCode, Json<SpotifyLibrarySyncResponse>)> {
    tracing::info!(user_id = %authenticated_user.id, "Spotify library sync requested");

    if let Some(status) = get_provider_library_sync_status(
        &state.redis_pool,
        SPOTIFY_SYNC_STATUS_KEY,
        authenticated_user.id,
    )
    .await?
    {
        if status.state == "running" {
            return Ok((
                StatusCode::ACCEPTED,
                Json(SpotifyLibrarySyncResponse {
                    success: true,
                    message: "Spotify library sync is already running. The sync status endpoint will update when imports finish.".to_string(),
                    summary: zero_spotify_sync_summary(),
                }),
            ));
        }
    }

    let connection = get_user_spotify_connection(&state.db_pool, authenticated_user.id)
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: "Spotify connection".to_string(),
        })?;

    tracing::info!(connection_id = %connection.id, "Found Spotify connection");

    let encrypted_access_token = connection.access_token_encrypted.ok_or_else(|| {
        AppError::ExternalServiceError(
            "Spotify access token is unavailable. Reconnect Spotify and try again.".to_string(),
        )
    })?;

    tracing::info!("Decrypting Spotify access token");
    let access_token = decrypt_connection_access_token(&encrypted_access_token).await?;
    tracing::info!("Token decrypted, starting background library sync");

    let started_at = Utc::now();
    store_provider_library_sync_status(
        &state.redis_pool,
        SPOTIFY_SYNC_STATUS_KEY,
        authenticated_user.id,
        &ProviderLibrarySyncStatus::running("Spotify library sync is in progress.", started_at),
        PROVIDER_LIBRARY_SYNC_RUNNING_TTL_SECONDS,
    )
    .await?;

    // Run sync in background to avoid Cloudflare 524 timeouts.
    // The sync makes many paginated Spotify API calls that can take minutes.
    let db_pool = state.db_pool.clone();
    let redis_pool = state.redis_pool.clone();
    let user_id = authenticated_user.id;
    tokio::spawn(async move {
        match sync_spotify_library_to_user_library(&db_pool, user_id, &access_token).await {
            Ok(summary) => {
                let success_message = format!(
                    "Synced Spotify library: {} imported items ({} liked songs, {} playlist tracks across {} playlists, {} saved albums, {} followed artists)",
                    summary.imported_tracks,
                    summary.liked_tracks_synced,
                    summary.playlist_tracks_synced,
                    summary.playlists_synced,
                    summary.saved_albums_synced,
                    summary.followed_artists_synced
                );
                tracing::info!(
                    user_id = %user_id,
                    imported = summary.imported_tracks,
                    liked = summary.liked_tracks_synced,
                    playlists = summary.playlist_tracks_synced,
                    playlists_synced = summary.playlists_synced,
                    albums = summary.saved_albums_synced,
                    artists = summary.followed_artists_synced,
                    "Spotify library sync completed successfully"
                );
                // Update connection to reflect successful sync
                if let Err(e) = sqlx::query(
                    "UPDATE connections SET status = 'active', last_health_check = NOW(), error_code = NULL WHERE user_id = $1 AND provider = 'spotify'",
                )
                .bind(user_id)
                .execute(&db_pool)
                .await
                {
                    tracing::warn!(error = %e, "Failed to update connection after sync");
                }

                if let Err(error) = store_provider_library_sync_status(
                    &redis_pool,
                    SPOTIFY_SYNC_STATUS_KEY,
                    user_id,
                    &ProviderLibrarySyncStatus::completed(
                        success_message,
                        started_at,
                        Utc::now(),
                        spotify_sync_status_counts(&summary),
                    ),
                    PROVIDER_LIBRARY_SYNC_STATUS_TTL_SECONDS,
                )
                .await
                {
                    tracing::error!(
                        user_id = %user_id,
                        error = %error,
                        "Failed to persist Spotify sync completion status"
                    );
                }
            }
            Err(error) => {
                let (message, needs_reauth) = map_spotify_sync_error(&error);
                tracing::error!(
                    user_id = %user_id,
                    error = %message,
                    "Spotify library sync failed in background"
                );

                let failure_query = if needs_reauth {
                    "UPDATE connections SET status = 'needs_reauth', error_code = $1 WHERE user_id = $2 AND provider = 'spotify'"
                } else {
                    "UPDATE connections SET error_code = $1 WHERE user_id = $2 AND provider = 'spotify'"
                };

                if let Err(update_error) = sqlx::query(failure_query)
                    .bind(&message)
                    .bind(user_id)
                    .execute(&db_pool)
                    .await
                {
                    tracing::warn!(
                        user_id = %user_id,
                        error = %update_error,
                        "Failed to update Spotify connection after sync failure"
                    );
                }

                if let Err(status_error) = store_provider_library_sync_status(
                    &redis_pool,
                    SPOTIFY_SYNC_STATUS_KEY,
                    user_id,
                    &ProviderLibrarySyncStatus::failed(message, started_at, Utc::now()),
                    PROVIDER_LIBRARY_SYNC_STATUS_TTL_SECONDS,
                )
                .await
                {
                    tracing::error!(
                        user_id = %user_id,
                        error = %status_error,
                        "Failed to persist Spotify sync failure status"
                    );
                }
            }
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(SpotifyLibrarySyncResponse {
            success: true,
            message: "Spotify library sync started. Poll the sync status endpoint for completion before refreshing heavy library views."
                .to_string(),
            summary: zero_spotify_sync_summary(),
        }),
    ))
}

pub async fn spotify_library_sync_status_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<Json<ProviderLibrarySyncStatus>> {
    let status = get_provider_library_sync_status(
        &state.redis_pool,
        SPOTIFY_SYNC_STATUS_KEY,
        authenticated_user.id,
    )
    .await?
    .unwrap_or_else(|| ProviderLibrarySyncStatus::idle(SPOTIFY_PROVIDER_LABEL));

    Ok(Json(status))
}

// Database helper functions

fn zero_spotify_sync_summary() -> SpotifyLibrarySyncSummary {
    SpotifyLibrarySyncSummary {
        imported_tracks: 0,
        liked_tracks_synced: 0,
        playlist_tracks_synced: 0,
        playlists_synced: 0,
        saved_albums_synced: 0,
        followed_artists_synced: 0,
    }
}

fn spotify_sync_status_counts(summary: &SpotifyLibrarySyncSummary) -> ProviderLibrarySyncCounts {
    ProviderLibrarySyncCounts {
        tracks_count: Some(summary.liked_tracks_synced + summary.playlist_tracks_synced),
        albums_count: Some(summary.saved_albums_synced),
        artists_count: Some(summary.followed_artists_synced),
        playlists_count: Some(summary.playlists_synced),
        imported_items_count: imported_items_count(summary.imported_tracks),
    }
}

fn map_spotify_sync_error(error: &AppError) -> (String, bool) {
    let raw = error.to_string();
    let lowered = raw.to_ascii_lowercase();

    if lowered.contains("insufficient client scope")
        || lowered.contains("missing required scope")
        || lowered.contains("playlist-read-private")
        || lowered.contains("playlist-read-collaborative")
        || lowered.contains("user-library-read")
        || lowered.contains("user-follow-read")
    {
        (
            "Spotify token is missing required library scopes. Disconnect and reconnect Spotify, then sync again.".to_string(),
            true,
        )
    } else if lowered.contains("401")
        || lowered.contains("403")
        || lowered.contains("unauthorized")
        || lowered.contains("forbidden")
        || lowered.contains("expired")
    {
        (
            "Spotify authorization failed. Disconnect and reconnect Spotify, then sync again."
                .to_string(),
            true,
        )
    } else {
        (raw, false)
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ConnectionRecord {
    id: Uuid,
    provider_user_id: Option<String>,
    status: String,
    scopes: Option<Vec<String>>,
    access_token_encrypted: Option<String>,
    expires_at: Option<chrono::DateTime<Utc>>,
    last_health_check: Option<chrono::DateTime<Utc>>,
}

async fn get_user_spotify_connection(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<ConnectionRecord>> {
    let row = sqlx::query_as::<_, ConnectionRecord>(
        r#"
        SELECT
            id,
            provider_user_id,
            status,
            scopes,
            access_token_encrypted,
            expires_at,
            last_health_check
        FROM connections
        WHERE user_id = $1 AND provider = 'spotify'
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to query Spotify connection");
        AppError::DatabaseQueryFailed(e)
    })?;

    Ok(row)
}

async fn store_spotify_connection(
    pool: &PgPool,
    user_id: Uuid,
    provider_user_id: Option<&str>,
    access_token_encrypted: &[u8],
    refresh_token_encrypted: Option<&[u8]>,
    expires_at: Option<chrono::DateTime<Utc>>,
) -> Result<Uuid> {
    use base64::{engine::general_purpose, Engine as _};

    let access_token_b64 = general_purpose::STANDARD.encode(access_token_encrypted);
    let refresh_token_b64 = refresh_token_encrypted.map(|t| general_purpose::STANDARD.encode(t));

    let scopes: Vec<String> = SPOTIFY_CONNECTION_SCOPES
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Upsert connection
    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO connections (
            user_id,
            provider,
            provider_user_id,
            scopes,
            access_token_encrypted,
            refresh_token_encrypted,
            token_version,
            expires_at,
            status,
            created_at
        )
        VALUES ($1, 'spotify', $2, $3, $4, $5, 1, $6, 'active', NOW())
        ON CONFLICT (user_id, provider)
        DO UPDATE SET
            provider_user_id = EXCLUDED.provider_user_id,
            scopes = EXCLUDED.scopes,
            access_token_encrypted = EXCLUDED.access_token_encrypted,
            refresh_token_encrypted = EXCLUDED.refresh_token_encrypted,
            token_version = connections.token_version + 1,
            expires_at = EXCLUDED.expires_at,
            status = 'active',
            error_code = NULL
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(provider_user_id)
    .bind(&scopes)
    .bind(&access_token_b64)
    .bind(&refresh_token_b64)
    .bind(expires_at)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to store Spotify connection");
        AppError::DatabaseQueryFailed(e)
    })?;

    Ok(row.0)
}

async fn delete_spotify_connection(pool: &PgPool, connection_id: Uuid) -> Result<()> {
    sqlx::query(
        r#"
        DELETE FROM connections
        WHERE id = $1
        "#,
    )
    .bind(connection_id)
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to delete Spotify connection");
        AppError::DatabaseQueryFailed(e)
    })?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct SpotifyPaging<T> {
    items: Vec<T>,
    next: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SpotifyArtistPayload {
    id: Option<String>,
    name: String,
}

#[derive(Debug, Deserialize)]
struct SpotifyAlbumPayload {
    id: Option<String>,
    name: String,
    artists: Option<Vec<SpotifyArtistPayload>>,
}

#[derive(Debug, Deserialize)]
struct SpotifyShowPayload {
    name: String,
}

#[derive(Debug, Deserialize)]
struct SpotifyTrackPayload {
    id: Option<String>,
    uri: Option<String>,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    item_type: Option<String>,
    is_local: Option<bool>,
    name: String,
    artists: Option<Vec<SpotifyArtistPayload>>,
    album: Option<SpotifyAlbumPayload>,
    show: Option<SpotifyShowPayload>,
}

#[derive(Debug, Deserialize)]
struct SpotifySavedTrackPayload {
    added_at: Option<String>,
    track: Option<SpotifyTrackPayload>,
}

#[derive(Debug, Deserialize)]
struct SpotifySavedAlbumPayload {
    added_at: Option<String>,
    album: Option<SpotifyAlbumPayload>,
}

#[derive(Debug, Deserialize)]
struct SpotifyPlaylistPayload {
    id: String,
    name: String,
    description: Option<String>,
    images: Option<Vec<SpotifyImagePayload>>,
    owner: Option<SpotifyPlaylistOwner>,
    public: Option<bool>,
    collaborative: Option<bool>,
    snapshot_id: Option<String>,
    tracks: Option<SpotifyPlaylistTracksRef>,
}

#[derive(Debug, Deserialize)]
struct SpotifyImagePayload {
    url: String,
}

#[derive(Debug, Deserialize)]
struct SpotifyPlaylistOwner {
    id: Option<String>,
    display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SpotifyPlaylistTracksRef {
    total: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct SpotifyPlaylistTrackPayload {
    added_at: Option<String>,
    track: Option<SpotifyTrackPayload>,
}

#[derive(Debug, Deserialize)]
struct SpotifyFollowedArtistsResponse {
    artists: SpotifyFollowedArtistsPayload,
}

#[derive(Debug, Deserialize)]
struct SpotifyFollowedArtistsPayload {
    items: Vec<SpotifyArtistPayload>,
    cursors: Option<SpotifyCursorPayload>,
}

#[derive(Debug, Deserialize)]
struct SpotifyCursorPayload {
    after: Option<String>,
}

async fn decrypt_connection_access_token(encoded_token: &str) -> Result<String> {
    let encrypted_bytes = general_purpose::STANDARD
        .decode(encoded_token)
        .map_err(|e| {
            AppError::ExternalServiceError(format!(
                "Stored Spotify token could not be decoded: {}",
                e
            ))
        })?;

    let encryption = OAuthTokenEncryption::new().map_err(|e| AppError::Internal {
        message: Some(format!("Failed to initialize token encryption: {}", e)),
    })?;

    encryption
        .decrypt_token(&encrypted_bytes)
        .await
        .map_err(|e| {
            AppError::ExternalServiceError(format!(
                "Stored Spotify token could not be decrypted: {}",
                e
            ))
        })
}

fn parse_spotify_timestamp(value: Option<&str>) -> Option<chrono::DateTime<Utc>> {
    value
        .and_then(|ts| chrono::DateTime::parse_from_rfc3339(ts).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

fn spotify_primary_artist(track: &SpotifyTrackPayload) -> String {
    track
        .artists
        .as_ref()
        .and_then(|artists| artists.first())
        .map(|artist| artist.name.clone())
        .or_else(|| track.show.as_ref().map(|show| show.name.clone()))
        .unwrap_or_else(|| "Unknown Artist".to_string())
}

fn spotify_album_name(track: &SpotifyTrackPayload) -> Option<String> {
    track
        .album
        .as_ref()
        .map(|album| album.name.clone())
        .or_else(|| track.show.as_ref().map(|show| show.name.clone()))
}

fn spotify_playlist_item_id(
    track: &SpotifyTrackPayload,
    playlist_id: &str,
    playlist_index: usize,
) -> Option<String> {
    track.id.clone().or_else(|| track.uri.clone()).or_else(|| {
        if track.is_local.unwrap_or(false) {
            Some(format!("spotify:local:{}:{}", playlist_id, playlist_index))
        } else {
            None
        }
    })
}

async fn spotify_get_json<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    access_token: &str,
    url: &str,
) -> Result<T> {
    const MAX_RETRIES: u32 = 5;

    for attempt in 0..=MAX_RETRIES {
        let response = client
            .get(url)
            .bearer_auth(access_token)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                AppError::ExternalServiceError(format!("Spotify request failed: {}", e))
            })?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            if attempt == MAX_RETRIES {
                let body = response.text().await.unwrap_or_default();
                return Err(AppError::ExternalServiceError(format!(
                    "Spotify rate limited after {} retries: {}",
                    MAX_RETRIES, body
                )));
            }

            // Respect Retry-After header, fall back to exponential backoff.
            // Cap at 60s — if Spotify says wait hours, fail fast instead of blocking.
            const MAX_WAIT_SECS: u64 = 60;
            let raw_retry = response
                .headers()
                .get("retry-after")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or_else(|| 2_u64.pow(attempt + 1));
            let retry_after_secs = raw_retry.min(MAX_WAIT_SECS);

            if raw_retry > MAX_WAIT_SECS {
                tracing::warn!(
                    url = url,
                    retry_after_raw = raw_retry,
                    "Spotify rate limit too long ({}s), failing fast instead of waiting",
                    raw_retry
                );
                return Err(AppError::ExternalServiceError(format!(
                    "Spotify rate limited for {}s ({}h) — try again later or use a fresh app",
                    raw_retry,
                    raw_retry / 3600
                )));
            }

            tracing::warn!(
                url = url,
                attempt = attempt,
                retry_after_secs = retry_after_secs,
                "Spotify rate limited, waiting before retry"
            );
            tokio::time::sleep(std::time::Duration::from_secs(retry_after_secs)).await;
            continue;
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalServiceError(format!(
                "Spotify request failed ({}): {}",
                status, body
            )));
        }

        return response.json::<T>().await.map_err(|e| {
            AppError::ExternalServiceError(format!("Failed to parse Spotify response: {}", e))
        });
    }

    Err(AppError::ExternalServiceError(
        "Spotify request failed: max retries exceeded".to_string(),
    ))
}

async fn sync_spotify_library_to_user_library(
    pool: &PgPool,
    user_id: Uuid,
    access_token: &str,
) -> Result<SpotifyLibrarySyncSummary> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| {
            AppError::ExternalServiceError(format!("Failed to create Spotify client: {}", e))
        })?;

    // NOTE: This Vec grows unboundedly as we page through the user's entire Spotify library.
    // A very large library (100k+ tracks) could use significant memory. Consider streaming
    // to the DB in batches if this becomes a problem in production.
    let mut tracks: Vec<ImportTrack> = Vec::with_capacity(2048);
    let mut liked_count = 0usize;
    let mut playlist_track_count = 0usize;
    let mut playlists_synced = 0usize;
    let mut saved_album_count = 0usize;
    let mut followed_artist_count = 0usize;

    let mut liked_url = Some("https://api.spotify.com/v1/me/tracks?limit=50".to_string());
    while let Some(url) = liked_url {
        let page: SpotifyPaging<SpotifySavedTrackPayload> =
            spotify_get_json(&client, access_token, &url).await?;
        for item in page.items {
            let Some(track) = item.track else {
                continue;
            };
            let Some(track_id) = track.id.clone() else {
                continue;
            };

            tracks.push(ImportTrack {
                provider_track_id: format!("liked:{}", track_id),
                track_name: track.name.clone(),
                album_name: spotify_album_name(&track),
                artist_name: spotify_primary_artist(&track),
                source_type: Some("liked".to_string()),
                playlist_name: None,
                added_at: parse_spotify_timestamp(item.added_at.as_deref()),
            });
            liked_count += 1;
        }
        liked_url = page.next;
    }

    let mut saved_albums_url = Some("https://api.spotify.com/v1/me/albums?limit=50".to_string());
    while let Some(url) = saved_albums_url {
        let page: SpotifyPaging<SpotifySavedAlbumPayload> =
            spotify_get_json(&client, access_token, &url).await?;
        for item in page.items {
            let Some(album) = item.album else {
                continue;
            };
            let Some(album_id) = album.id.clone() else {
                continue;
            };
            let album_artist = album
                .artists
                .as_ref()
                .and_then(|artists| artists.first())
                .map(|artist| artist.name.clone())
                .unwrap_or_else(|| "Unknown Artist".to_string());

            tracks.push(ImportTrack {
                provider_track_id: format!("album:{}", album_id),
                track_name: format!("[Album] {}", album.name),
                album_name: Some(album.name.clone()),
                artist_name: album_artist,
                source_type: Some("saved_album".to_string()),
                playlist_name: None,
                added_at: parse_spotify_timestamp(item.added_at.as_deref()),
            });
            saved_album_count += 1;
        }
        saved_albums_url = page.next;
    }

    let mut artist_after: Option<String> = None;
    loop {
        let mut url = "https://api.spotify.com/v1/me/following?type=artist&limit=50".to_string();
        if let Some(after) = artist_after.as_deref() {
            url.push_str("&after=");
            url.push_str(after);
        }

        let response: SpotifyFollowedArtistsResponse =
            spotify_get_json(&client, access_token, &url).await?;

        for artist in response.artists.items {
            let Some(artist_id) = artist.id.clone() else {
                continue;
            };

            tracks.push(ImportTrack {
                provider_track_id: format!("artist:{}", artist_id),
                track_name: format!("[Artist] {}", artist.name),
                album_name: None,
                artist_name: artist.name,
                source_type: Some("followed_artist".to_string()),
                playlist_name: None,
                added_at: None,
            });
            followed_artist_count += 1;
        }

        artist_after = response.artists.cursors.and_then(|cursor| cursor.after);
        if artist_after.is_none() {
            break;
        }
    }

    // ── Normalized playlist dual-write ──────────────────────────────────
    let playlist_repo = PlaylistRepository::new(pool);
    let sync_ts = Utc::now();

    // ── Fetch playlists and their tracks, writing to both old and new tables ─
    let mut playlists_url = Some("https://api.spotify.com/v1/me/playlists?limit=50".to_string());
    while let Some(url) = playlists_url {
        let page: SpotifyPaging<SpotifyPlaylistPayload> =
            spotify_get_json(&client, access_token, &url).await?;

        for playlist in &page.items {
            // Fetch playlist tracks
            let estimated_count = playlist
                .tracks
                .as_ref()
                .and_then(|t| t.total)
                .unwrap_or(100) as usize;
            let mut normalized_tracks: Vec<UpsertPlaylistTrack> =
                Vec::with_capacity(estimated_count);
            let mut playlist_tracks_url = Some(format!(
                "https://api.spotify.com/v1/playlists/{}/items?limit=100&additional_types=track,episode&fields=next,items(added_at,track(id,uri,is_local,type,name,artists(id,name),album(id,name),show(name)))",
                playlist.id
            ));
            let mut playlist_index = 0usize;

            while let Some(playlist_url) = playlist_tracks_url {
                let track_page: SpotifyPaging<SpotifyPlaylistTrackPayload> =
                    spotify_get_json(&client, access_token, &playlist_url).await?;

                for playlist_item in track_page.items {
                    let Some(track) = playlist_item.track else {
                        continue;
                    };
                    let Some(track_id) =
                        spotify_playlist_item_id(&track, &playlist.id, playlist_index)
                    else {
                        continue;
                    };

                    // Old table (backward compat)
                    tracks.push(ImportTrack {
                        provider_track_id: format!(
                            "playlist:{}:{}:{}",
                            playlist.id, track_id, playlist_index
                        ),
                        track_name: track.name.clone(),
                        album_name: spotify_album_name(&track),
                        artist_name: spotify_primary_artist(&track),
                        source_type: Some("playlist_track".to_string()),
                        playlist_name: Some(playlist.name.clone()),
                        added_at: parse_spotify_timestamp(playlist_item.added_at.as_deref()),
                    });

                    // New normalized table
                    normalized_tracks.push(UpsertPlaylistTrack {
                        provider_track_id: track_id,
                        track_name: track.name.clone(),
                        album_name: spotify_album_name(&track),
                        artist_name: spotify_primary_artist(&track),
                        position: playlist_index as i32,
                        added_at: parse_spotify_timestamp(playlist_item.added_at.as_deref()),
                    });

                    playlist_track_count += 1;
                    playlist_index += 1;
                }

                playlist_tracks_url = track_page.next;
            }

            let playlist_upsert = UpsertPlaylist {
                provider_playlist_id: playlist.id.clone(),
                name: playlist.name.clone(),
                description: playlist.description.clone(),
                image_url: playlist
                    .images
                    .as_ref()
                    .and_then(|imgs| imgs.first())
                    .map(|img| img.url.clone()),
                owner_name: playlist.owner.as_ref().and_then(|o| o.display_name.clone()),
                owner_id: playlist.owner.as_ref().and_then(|o| o.id.clone()),
                is_public: playlist.public,
                is_collaborative: playlist.collaborative.unwrap_or(false),
                source_type: "playlist".to_string(),
                provider_track_count: playlist.tracks.as_ref().and_then(|t| t.total),
                snapshot_id: playlist.snapshot_id.clone(),
            };

            if playlist_upsert.provider_track_count.unwrap_or_default() > 0
                && normalized_tracks.is_empty()
            {
                let preserved = playlist_repo
                    .touch_playlist_last_synced(user_id, "spotify", &playlist.id)
                    .await?;
                tracing::warn!(
                    playlist_id = %playlist.id,
                    playlist_name = %playlist.name,
                    provider_track_count = playlist_upsert.provider_track_count,
                    preserved_existing_playlist = preserved,
                    "Spotify playlist returned no importable track rows despite a non-zero provider track count; preserving existing inventory"
                );
                continue;
            }

            playlist_repo
                .upsert_playlist_and_replace_tracks(
                    user_id,
                    "spotify",
                    &playlist_upsert,
                    &normalized_tracks,
                )
                .await?;
            playlists_synced += 1;
        }

        playlists_url = page.next;
    }

    // Write virtual playlist tracks to normalized tables
    // (liked songs, saved albums, followed artists collected earlier)
    {
        let mut liked_norm: Vec<UpsertPlaylistTrack> = Vec::with_capacity(liked_count);
        let mut album_norm: Vec<UpsertPlaylistTrack> = Vec::with_capacity(saved_album_count);
        let mut artist_norm: Vec<UpsertPlaylistTrack> = Vec::with_capacity(followed_artist_count);
        let mut liked_pos = 0i32;
        let mut album_pos = 0i32;
        let mut artist_pos = 0i32;

        for t in &tracks {
            match t.source_type.as_deref() {
                Some("liked") => {
                    let raw_id = t
                        .provider_track_id
                        .strip_prefix("liked:")
                        .unwrap_or(&t.provider_track_id);
                    liked_norm.push(UpsertPlaylistTrack {
                        provider_track_id: raw_id.to_string(),
                        track_name: t.track_name.clone(),
                        album_name: t.album_name.clone(),
                        artist_name: t.artist_name.clone(),
                        position: liked_pos,
                        added_at: t.added_at,
                    });
                    liked_pos += 1;
                }
                Some("saved_album") => {
                    let raw_id = t
                        .provider_track_id
                        .strip_prefix("album:")
                        .unwrap_or(&t.provider_track_id);
                    album_norm.push(UpsertPlaylistTrack {
                        provider_track_id: raw_id.to_string(),
                        track_name: t.track_name.clone(),
                        album_name: t.album_name.clone(),
                        artist_name: t.artist_name.clone(),
                        position: album_pos,
                        added_at: t.added_at,
                    });
                    album_pos += 1;
                }
                Some("followed_artist") => {
                    let raw_id = t
                        .provider_track_id
                        .strip_prefix("artist:")
                        .unwrap_or(&t.provider_track_id);
                    artist_norm.push(UpsertPlaylistTrack {
                        provider_track_id: raw_id.to_string(),
                        track_name: t.track_name.clone(),
                        album_name: t.album_name.clone(),
                        artist_name: t.artist_name.clone(),
                        position: artist_pos,
                        added_at: t.added_at,
                    });
                    artist_pos += 1;
                }
                _ => {} // playlist_track already handled above
            }
        }

        playlist_repo
            .upsert_playlist_and_replace_tracks(
                user_id,
                "spotify",
                &UpsertPlaylist {
                    provider_playlist_id: "__liked_songs__".to_string(),
                    name: "Liked Songs".to_string(),
                    description: None,
                    image_url: None,
                    owner_name: None,
                    owner_id: None,
                    is_public: Some(false),
                    is_collaborative: false,
                    source_type: "liked_songs".to_string(),
                    provider_track_count: Some(liked_count as i32),
                    snapshot_id: None,
                },
                &liked_norm,
            )
            .await?;
        playlist_repo
            .upsert_playlist_and_replace_tracks(
                user_id,
                "spotify",
                &UpsertPlaylist {
                    provider_playlist_id: "__saved_albums__".to_string(),
                    name: "Saved Albums".to_string(),
                    description: None,
                    image_url: None,
                    owner_name: None,
                    owner_id: None,
                    is_public: Some(false),
                    is_collaborative: false,
                    source_type: "saved_albums".to_string(),
                    provider_track_count: Some(saved_album_count as i32),
                    snapshot_id: None,
                },
                &album_norm,
            )
            .await?;
        playlist_repo
            .upsert_playlist_and_replace_tracks(
                user_id,
                "spotify",
                &UpsertPlaylist {
                    provider_playlist_id: "__followed_artists__".to_string(),
                    name: "Followed Artists".to_string(),
                    description: None,
                    image_url: None,
                    owner_name: None,
                    owner_id: None,
                    is_public: Some(false),
                    is_collaborative: false,
                    source_type: "followed_artists".to_string(),
                    provider_track_count: Some(followed_artist_count as i32),
                    snapshot_id: None,
                },
                &artist_norm,
            )
            .await?;
    }

    // Remove playlists that were deleted on Spotify since last sync
    playlist_repo
        .delete_stale_playlists(user_id, "spotify", sync_ts)
        .await?;

    // ── Legacy table: atomic delete-and-reimport in a single transaction ─
    let imported_tracks = {
        OffenseService::new(pool)
            .delete_and_import_library(
                user_id,
                ImportLibraryRequest {
                    provider: "spotify".to_string(),
                    tracks,
                },
            )
            .await?
    };

    Ok(SpotifyLibrarySyncSummary {
        imported_tracks,
        liked_tracks_synced: liked_count,
        playlist_tracks_synced: playlist_track_count,
        playlists_synced,
        saved_albums_synced: saved_album_count,
        followed_artists_synced: followed_artist_count,
    })
}

// Redis helper functions for OAuth state management

async fn store_oauth_state(
    redis_pool: &deadpool_redis::Pool,
    state: &str,
    data: &OAuthStateData,
) -> Result<()> {
    use deadpool_redis::redis::AsyncCommands;

    let mut conn = redis_pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get Redis connection");
        AppError::ExternalServiceError("Failed to connect to session store".to_string())
    })?;

    let key = format!("spotify_oauth_state:{}", state);
    let value = serde_json::to_string(data).map_err(|e| {
        tracing::error!(error = %e, "Failed to serialize OAuth state");
        AppError::Internal {
            message: Some("Failed to store OAuth state".to_string()),
        }
    })?;

    // Store with 10 minute expiration
    let _: () = conn.set_ex(&key, &value, 600).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to store OAuth state in Redis");
        AppError::ExternalServiceError("Failed to store OAuth state".to_string())
    })?;

    Ok(())
}

async fn get_oauth_state(redis_pool: &deadpool_redis::Pool, state: &str) -> Result<OAuthStateData> {
    use deadpool_redis::redis::AsyncCommands;

    let mut conn = redis_pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get Redis connection");
        AppError::ExternalServiceError("Failed to connect to session store".to_string())
    })?;

    let key = format!("spotify_oauth_state:{}", state);
    let value: Option<String> = conn.get(&key).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get OAuth state from Redis");
        AppError::ExternalServiceError("Failed to retrieve OAuth state".to_string())
    })?;

    match value {
        Some(json) => serde_json::from_str(&json).map_err(|e| {
            tracing::error!(error = %e, "Failed to deserialize OAuth state");
            AppError::InvalidFieldValue {
                field: "state".to_string(),
                message: "Invalid OAuth state".to_string(),
            }
        }),
        None => Err(AppError::InvalidFieldValue {
            field: "state".to_string(),
            message: "Invalid or expired OAuth state. Please try again.".to_string(),
        }),
    }
}

async fn delete_oauth_state(redis_pool: &deadpool_redis::Pool, state: &str) -> Result<()> {
    use deadpool_redis::redis::AsyncCommands;

    let mut conn = redis_pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get Redis connection");
        AppError::ExternalServiceError("Failed to connect to session store".to_string())
    })?;

    let key = format!("spotify_oauth_state:{}", state);
    let _: () = conn.del(&key).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to delete OAuth state from Redis");
        AppError::ExternalServiceError("Failed to clear OAuth state".to_string())
    })?;

    Ok(())
}
