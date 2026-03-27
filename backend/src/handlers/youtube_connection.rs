//! YouTube Music Connection Handlers
//!
//! Handles YouTube Music OAuth flow for provider connection.
//! This allows users to connect their YouTube Music account for DNP list enforcement.

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
use crate::models::offense::{ImportLibraryRequest, ImportTrack};
use crate::models::playlist::{UpsertPlaylist, UpsertPlaylistTrack};
use crate::models::user::AuthenticatedUser;
use crate::services::oauth::OAuthProvider;
use crate::services::oauth_youtube_music::YouTubeMusicOAuthProvider;
use crate::services::OAuthTokenEncryption;
use crate::services::OffenseService;
use crate::services::PlaylistRepository;
use crate::AppState;
use ndith_core::config::provider_callback_uri_with_override;

const YOUTUBE_SYNC_STATUS_KEY: &str = "youtube";
const YOUTUBE_PROVIDER_LABEL: &str = "YouTube Music";

/// Query parameters for the authorize endpoint
#[derive(Debug, Deserialize)]
pub struct YouTubeAuthorizeQuery {
    /// Optional redirect URI override
    pub redirect_uri: Option<String>,
}

/// Response from the authorize endpoint
#[derive(Debug, Serialize)]
pub struct YouTubeAuthorizeResponse {
    pub authorization_url: String,
    pub state: String,
    pub scopes: Vec<String>,
}

/// Request body for the callback endpoint
#[derive(Debug, Deserialize)]
pub struct YouTubeCallbackRequest {
    pub code: String,
    pub state: String,
}

/// Response from the callback endpoint
#[derive(Debug, Serialize)]
pub struct YouTubeCallbackResponse {
    pub success: bool,
    pub connection_id: Uuid,
    pub provider_user_id: String,
    pub status: String,
    pub message: String,
    pub sync_summary: Option<YouTubeLibrarySyncSummary>,
    pub sync_warning: Option<String>,
}

/// Connection status response
#[derive(Debug, Serialize)]
pub struct YouTubeConnectionStatus {
    pub connected: bool,
    pub connection_id: Option<Uuid>,
    pub provider_user_id: Option<String>,
    pub status: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub expires_at: Option<String>,
    pub last_health_check: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct YouTubeLibrarySyncSummary {
    pub imported_tracks: i32,
    pub liked_videos_synced: usize,
    pub playlist_items_synced: usize,
    pub subscriptions_synced: usize,
    pub playlists_scanned: usize,
}

#[derive(Debug, Serialize)]
pub struct YouTubeLibrarySyncResponse {
    pub success: bool,
    pub summary: YouTubeLibrarySyncSummary,
    pub message: String,
}

/// OAuth state stored in Redis for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OAuthStateData {
    pub user_id: Uuid,
    pub redirect_uri: String,
    pub created_at: chrono::DateTime<Utc>,
}

/// YouTube Music scopes for library access
const YOUTUBE_SCOPES: &[&str] = &[
    "openid",
    "email",
    "profile",
    "https://www.googleapis.com/auth/youtube",
    "https://www.googleapis.com/auth/youtube.force-ssl",
    "https://www.googleapis.com/auth/youtube.readonly",
];

/// GET /api/v1/connections/youtube/authorize
///
/// Initiates YouTube Music OAuth flow for provider connection.
/// Returns an authorization URL that the user should be redirected to.
pub async fn youtube_authorize_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Query(query): Query<YouTubeAuthorizeQuery>,
) -> Result<(StatusCode, Json<YouTubeAuthorizeResponse>)> {
    tracing::info!(
        user_id = %authenticated_user.id,
        "Initiating YouTube Music connection OAuth flow"
    );

    // If a connection exists, allow reconnect to refresh stale/expired tokens.
    let existing_connection =
        get_user_youtube_connection(&state.db_pool, authenticated_user.id).await?;
    if let Some(conn) = existing_connection {
        if conn.status == "active" {
            tracing::info!(
                user_id = %authenticated_user.id,
                connection_id = %conn.id,
                "Existing active YouTube Music connection found; proceeding with reconnect flow"
            );
        }
    }

    // Create YouTube Music OAuth provider
    let youtube_provider = YouTubeMusicOAuthProvider::new().map_err(|e| {
        tracing::error!(error = %e, "Failed to create YouTube Music OAuth provider");
        AppError::ConfigurationError {
            message: "YouTube Music OAuth is not properly configured".to_string(),
        }
    })?;

    // Determine redirect URI: validate against the configured callback URL.
    let default_redirect_uri = provider_callback_uri_with_override(
        "youtube",
        &["YOUTUBE_MUSIC_REDIRECT_URI", "GOOGLE_REDIRECT_URI"],
    );
    let redirect_uri = match query.redirect_uri {
        Some(ref uri) if uri != &default_redirect_uri => {
            tracing::warn!(
                user_id = %authenticated_user.id,
                requested_uri = %uri,
                expected_uri = %default_redirect_uri,
                "Rejected non-allowlisted YouTube Music redirect_uri"
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
    let flow_response = youtube_provider.initiate_flow(&redirect_uri).await?;

    // Store state in Redis for validation during callback
    let state_data = OAuthStateData {
        user_id: authenticated_user.id,
        redirect_uri: redirect_uri.clone(),
        created_at: Utc::now(),
    };

    store_oauth_state(&state.redis_pool, &flow_response.state, &state_data).await?;

    tracing::info!(
        user_id = %authenticated_user.id,
        state = %flow_response.state,
        "YouTube Music OAuth flow initiated successfully"
    );

    Ok((
        StatusCode::OK,
        Json(YouTubeAuthorizeResponse {
            authorization_url: flow_response.authorization_url,
            state: flow_response.state,
            scopes: YOUTUBE_SCOPES.iter().map(|s| s.to_string()).collect(),
        }),
    ))
}

/// POST /api/v1/connections/youtube/callback
///
/// Handles the OAuth callback from YouTube/Google.
/// Exchanges the authorization code for tokens and creates the connection.
pub async fn youtube_callback_handler(
    State(state): State<AppState>,
    Json(request): Json<YouTubeCallbackRequest>,
) -> Result<(StatusCode, Json<YouTubeCallbackResponse>)> {
    tracing::info!(
        state = %request.state,
        code_length = request.code.len(),
        "Processing YouTube Music OAuth callback"
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

    // Delete state from Redis to prevent replay attacks
    delete_oauth_state(&state.redis_pool, &request.state).await?;

    // Create YouTube Music OAuth provider
    let youtube_provider = YouTubeMusicOAuthProvider::new().map_err(|e| {
        tracing::error!(error = %e, "Failed to create YouTube Music OAuth provider");
        AppError::ConfigurationError {
            message: "YouTube Music OAuth is not properly configured".to_string(),
        }
    })?;

    // Exchange code for tokens
    let tokens = youtube_provider
        .exchange_code(&request.code, &request.state, &state_data.redirect_uri)
        .await?;

    // Get user info from YouTube/Google
    let user_info = youtube_provider.get_user_info(&tokens.access_token).await?;

    // Encrypt tokens for storage
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

    // Calculate expiry
    let expires_at = tokens
        .expires_in
        .map(|secs| Utc::now() + Duration::seconds(secs));

    // Store connection in database
    let connection_id = store_youtube_connection(
        &state.db_pool,
        state_data.user_id,
        &user_info.provider_user_id,
        &access_token_encrypted,
        refresh_token_encrypted.as_deref(),
        expires_at,
    )
    .await?;

    let (sync_summary, sync_warning) = match sync_youtube_library_to_user_library(
        &state.db_pool,
        state_data.user_id,
        &tokens.access_token,
    )
    .await
    {
        Ok(summary) => (Some(summary), None),
        Err(error) => {
            tracing::warn!(
                user_id = %state_data.user_id,
                error = %error,
                "YouTube Music connection succeeded but initial library sync failed"
            );
            (
                None,
                Some(
                    "YouTube Music connected, but automatic library sync failed. Try syncing again from the Music Library page."
                        .to_string(),
                ),
            )
        }
    };

    tracing::info!(
        user_id = %state_data.user_id,
        connection_id = %connection_id,
        provider_user_id = %user_info.provider_user_id,
        "YouTube Music connection created successfully"
    );

    Ok((
        StatusCode::OK,
        Json(YouTubeCallbackResponse {
            success: true,
            connection_id,
            provider_user_id: user_info.provider_user_id,
            status: "active".to_string(),
            message: "YouTube Music connected successfully".to_string(),
            sync_summary,
            sync_warning,
        }),
    ))
}

/// GET /api/v1/connections/youtube/status
///
/// Returns the current YouTube Music connection status for the user.
pub async fn youtube_status_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<(StatusCode, Json<YouTubeConnectionStatus>)> {
    let connection = get_user_youtube_connection(&state.db_pool, authenticated_user.id).await?;

    match connection {
        Some(conn) => Ok((
            StatusCode::OK,
            Json(YouTubeConnectionStatus {
                connected: conn.status == "active",
                connection_id: Some(conn.id),
                provider_user_id: conn.provider_user_id,
                status: Some(conn.status),
                scopes: conn.scopes,
                expires_at: conn.expires_at.map(|dt| dt.to_rfc3339()),
                last_health_check: conn.last_health_check.map(|dt| dt.to_rfc3339()),
            }),
        )),
        None => Ok((
            StatusCode::OK,
            Json(YouTubeConnectionStatus {
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

/// DELETE /api/v1/connections/youtube
///
/// Disconnects the user's YouTube Music account.
pub async fn youtube_disconnect_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    tracing::info!(
        user_id = %authenticated_user.id,
        "Disconnecting YouTube Music account"
    );

    let connection = get_user_youtube_connection(&state.db_pool, authenticated_user.id).await?;

    match connection {
        Some(conn) => {
            delete_youtube_connection(&state.db_pool, conn.id).await?;

            tracing::info!(
                user_id = %authenticated_user.id,
                connection_id = %conn.id,
                "YouTube Music connection deleted"
            );

            Ok((
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "message": "YouTube Music account disconnected successfully"
                })),
            ))
        }
        None => Err(AppError::NotFound {
            resource: "YouTube Music connection".to_string(),
        }),
    }
}

/// POST /api/v1/connections/youtube/library/sync
///
/// Fetches YouTube Music liked videos, playlists, playlist items, and subscriptions,
/// then imports them into `user_library_tracks`.
pub async fn youtube_library_sync_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<(StatusCode, Json<YouTubeLibrarySyncResponse>)> {
    if let Some(status) = get_provider_library_sync_status(
        &state.redis_pool,
        YOUTUBE_SYNC_STATUS_KEY,
        authenticated_user.id,
    )
    .await?
    {
        if status.state == "running" {
            return Ok((
                StatusCode::ACCEPTED,
                Json(YouTubeLibrarySyncResponse {
                    success: true,
                    message: "YouTube Music library sync is already running. Check sync status for completion.".to_string(),
                    summary: zero_youtube_sync_summary(),
                }),
            ));
        }
    }

    let connection = get_user_youtube_connection(&state.db_pool, authenticated_user.id)
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: "YouTube Music connection".to_string(),
        })?;

    let encrypted_access_token = connection.access_token_encrypted.ok_or_else(|| {
        AppError::ExternalServiceError(
            "YouTube Music access token is unavailable. Reconnect YouTube Music and try again."
                .to_string(),
        )
    })?;

    let started_at = Utc::now();
    store_provider_library_sync_status(
        &state.redis_pool,
        YOUTUBE_SYNC_STATUS_KEY,
        authenticated_user.id,
        &ProviderLibrarySyncStatus::running(
            "YouTube Music library sync is in progress.",
            started_at,
        ),
        PROVIDER_LIBRARY_SYNC_RUNNING_TTL_SECONDS,
    )
    .await?;

    let access_token = decrypt_connection_access_token(&encrypted_access_token).await?;
    let summary = match sync_youtube_library_to_user_library(
        &state.db_pool,
        authenticated_user.id,
        &access_token,
    )
    .await
    {
        Ok(summary) => summary,
        Err(error) => {
            let (message, needs_reauth) = map_youtube_sync_error(&error);
            if let Err(update_error) = update_youtube_connection_sync_failure(
                &state.db_pool,
                authenticated_user.id,
                &message,
                needs_reauth,
            )
            .await
            {
                tracing::warn!(
                    user_id = %authenticated_user.id,
                    error = %update_error,
                    "Failed to update YouTube Music connection after sync failure"
                );
            }

            if let Err(status_error) = store_provider_library_sync_status(
                &state.redis_pool,
                YOUTUBE_SYNC_STATUS_KEY,
                authenticated_user.id,
                &ProviderLibrarySyncStatus::failed(message.clone(), started_at, Utc::now()),
                PROVIDER_LIBRARY_SYNC_STATUS_TTL_SECONDS,
            )
            .await
            {
                tracing::error!(
                    user_id = %authenticated_user.id,
                    error = %status_error,
                    "Failed to persist YouTube Music sync failure status"
                );
            }

            return Err(AppError::ExternalServiceError(message));
        }
    };

    update_youtube_connection_sync_success(&state.db_pool, authenticated_user.id).await?;
    let success_message = format!(
        "Synced YouTube Music library: {} imported items ({} liked videos, {} playlist items, {} subscriptions across {} playlists)",
        summary.imported_tracks,
        summary.liked_videos_synced,
        summary.playlist_items_synced,
        summary.subscriptions_synced,
        summary.playlists_scanned
    );
    store_provider_library_sync_status(
        &state.redis_pool,
        YOUTUBE_SYNC_STATUS_KEY,
        authenticated_user.id,
        &ProviderLibrarySyncStatus::completed(
            success_message.clone(),
            started_at,
            Utc::now(),
            youtube_sync_status_counts(&summary),
        ),
        PROVIDER_LIBRARY_SYNC_STATUS_TTL_SECONDS,
    )
    .await?;

    Ok((
        StatusCode::OK,
        Json(YouTubeLibrarySyncResponse {
            success: true,
            message: success_message,
            summary,
        }),
    ))
}

pub async fn youtube_library_sync_status_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<Json<ProviderLibrarySyncStatus>> {
    let status = get_provider_library_sync_status(
        &state.redis_pool,
        YOUTUBE_SYNC_STATUS_KEY,
        authenticated_user.id,
    )
    .await?
    .unwrap_or_else(|| ProviderLibrarySyncStatus::idle(YOUTUBE_PROVIDER_LABEL));

    Ok(Json(status))
}

// ============================================================================
// Database helper functions
// ============================================================================

fn zero_youtube_sync_summary() -> YouTubeLibrarySyncSummary {
    YouTubeLibrarySyncSummary {
        imported_tracks: 0,
        liked_videos_synced: 0,
        playlist_items_synced: 0,
        subscriptions_synced: 0,
        playlists_scanned: 0,
    }
}

fn youtube_sync_status_counts(summary: &YouTubeLibrarySyncSummary) -> ProviderLibrarySyncCounts {
    ProviderLibrarySyncCounts {
        tracks_count: Some(summary.liked_videos_synced + summary.playlist_items_synced),
        albums_count: None,
        artists_count: Some(summary.subscriptions_synced),
        playlists_count: Some(summary.playlists_scanned),
        imported_items_count: imported_items_count(summary.imported_tracks),
    }
}

fn map_youtube_sync_error(error: &AppError) -> (String, bool) {
    let raw = error.to_string();
    let lowered = raw.to_ascii_lowercase();

    if lowered.contains("401")
        || lowered.contains("403")
        || lowered.contains("unauthorized")
        || lowered.contains("forbidden")
        || lowered.contains("invalid_grant")
        || lowered.contains("expired")
    {
        (
            "YouTube Music authorization failed. Disconnect and reconnect YouTube Music, then sync again.".to_string(),
            true,
        )
    } else {
        (raw, false)
    }
}

async fn update_youtube_connection_sync_success(pool: &PgPool, user_id: Uuid) -> Result<()> {
    sqlx::query(
        "UPDATE connections SET status = 'active', last_health_check = NOW(), error_code = NULL WHERE user_id = $1 AND provider = 'youtube_music'",
    )
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(AppError::DatabaseQueryFailed)?;

    Ok(())
}

async fn update_youtube_connection_sync_failure(
    pool: &PgPool,
    user_id: Uuid,
    message: &str,
    needs_reauth: bool,
) -> Result<()> {
    let query = if needs_reauth {
        "UPDATE connections SET status = 'needs_reauth', error_code = $1 WHERE user_id = $2 AND provider = 'youtube_music'"
    } else {
        "UPDATE connections SET error_code = $1 WHERE user_id = $2 AND provider = 'youtube_music'"
    };

    sqlx::query(query)
        .bind(message)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

    Ok(())
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

async fn get_user_youtube_connection(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<ConnectionRecord>> {
    let connection = sqlx::query_as::<_, ConnectionRecord>(
        r#"
        SELECT id, provider_user_id, status, scopes, access_token_encrypted, expires_at, last_health_check
        FROM connections
        WHERE user_id = $1 AND provider = 'youtube_music'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to query YouTube Music connection");
        AppError::DatabaseQueryFailed(e)
    })?;

    Ok(connection)
}

async fn store_youtube_connection(
    pool: &PgPool,
    user_id: Uuid,
    provider_user_id: &str,
    access_token_encrypted: &[u8],
    refresh_token_encrypted: Option<&[u8]>,
    expires_at: Option<chrono::DateTime<Utc>>,
) -> Result<Uuid> {
    use base64::{engine::general_purpose, Engine as _};

    let access_token_b64 = general_purpose::STANDARD.encode(access_token_encrypted);
    let refresh_token_b64 = refresh_token_encrypted.map(|t| general_purpose::STANDARD.encode(t));

    let scopes: Vec<String> = YOUTUBE_SCOPES.iter().map(|s| s.to_string()).collect();

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
        VALUES ($1, 'youtube_music', $2, $3, $4, $5, 1, $6, 'active', NOW())
        ON CONFLICT (user_id, provider)
        DO UPDATE SET
            provider_user_id = EXCLUDED.provider_user_id,
            scopes = EXCLUDED.scopes,
            access_token_encrypted = EXCLUDED.access_token_encrypted,
            refresh_token_encrypted = EXCLUDED.refresh_token_encrypted,
            token_version = connections.token_version + 1,
            expires_at = EXCLUDED.expires_at,
            status = 'active',
            updated_at = NOW()
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
        tracing::error!(error = %e, "Failed to store YouTube Music connection");
        AppError::DatabaseQueryFailed(e)
    })?;

    Ok(row.0)
}

async fn delete_youtube_connection(pool: &PgPool, connection_id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM connections WHERE id = $1")
        .bind(connection_id)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to delete YouTube Music connection");
            AppError::DatabaseQueryFailed(e)
        })?;

    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubeListResponse<T> {
    items: Vec<T>,
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubeVideoPayload {
    id: String,
    snippet: YouTubeVideoSnippetPayload,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubeVideoSnippetPayload {
    title: String,
    channel_title: Option<String>,
    published_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubePlaylistPayload {
    id: String,
    snippet: YouTubePlaylistSnippetPayload,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubePlaylistSnippetPayload {
    title: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubePlaylistItemPayload {
    id: String,
    snippet: YouTubePlaylistItemSnippetPayload,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubePlaylistItemSnippetPayload {
    title: String,
    channel_title: Option<String>,
    published_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubeSubscriptionPayload {
    snippet: YouTubeSubscriptionSnippetPayload,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubeSubscriptionSnippetPayload {
    title: String,
    resource_id: Option<YouTubeSubscriptionResourceIdPayload>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubeSubscriptionResourceIdPayload {
    channel_id: Option<String>,
}

async fn decrypt_connection_access_token(encoded_token: &str) -> Result<String> {
    let encrypted_bytes = general_purpose::STANDARD
        .decode(encoded_token)
        .map_err(|e| {
            AppError::ExternalServiceError(format!(
                "Stored YouTube Music token could not be decoded: {}",
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
                "Stored YouTube Music token could not be decrypted: {}",
                e
            ))
        })
}

fn parse_youtube_timestamp(value: Option<&str>) -> Option<chrono::DateTime<Utc>> {
    value
        .and_then(|ts| chrono::DateTime::parse_from_rfc3339(ts).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

async fn youtube_get_json<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    access_token: &str,
    url: &str,
) -> Result<T> {
    let response = client
        .get(url)
        .bearer_auth(access_token)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| {
            AppError::ExternalServiceError(format!("YouTube API request failed: {}", e))
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::ExternalServiceError(format!(
            "YouTube API request failed ({}): {}",
            status, body
        )));
    }

    response.json::<T>().await.map_err(|e| {
        AppError::ExternalServiceError(format!("Failed to parse YouTube API response: {}", e))
    })
}

async fn sync_youtube_library_to_user_library(
    pool: &PgPool,
    user_id: Uuid,
    access_token: &str,
) -> Result<YouTubeLibrarySyncSummary> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| {
            AppError::ExternalServiceError(format!("Failed to create YouTube client: {}", e))
        })?;

    // NOTE: This Vec grows unboundedly as we page through the user's entire YouTube library.
    // Consider streaming to the DB in batches if memory becomes a concern.
    let mut tracks: Vec<ImportTrack> = Vec::with_capacity(1024);
    let mut liked_videos_count = 0usize;
    let mut playlist_items_count = 0usize;
    let mut subscriptions_count = 0usize;
    let mut playlists_count = 0usize;

    let mut liked_page_token: Option<String> = None;
    loop {
        let mut url =
            "https://www.googleapis.com/youtube/v3/videos?part=snippet&myRating=like&maxResults=50"
                .to_string();
        if let Some(token) = liked_page_token.as_deref() {
            url.push_str("&pageToken=");
            url.push_str(token);
        }

        let page: YouTubeListResponse<YouTubeVideoPayload> =
            youtube_get_json(&client, access_token, &url).await?;
        for video in page.items {
            tracks.push(ImportTrack {
                provider_track_id: format!("video:{}", video.id),
                track_name: video.snippet.title.clone(),
                album_name: None,
                artist_name: video
                    .snippet
                    .channel_title
                    .clone()
                    .unwrap_or_else(|| "Unknown Channel".to_string()),
                source_type: Some("liked_video".to_string()),
                playlist_name: None,
                added_at: parse_youtube_timestamp(video.snippet.published_at.as_deref()),
            });
            liked_videos_count += 1;
        }

        liked_page_token = page.next_page_token;
        if liked_page_token.is_none() {
            break;
        }
    }

    let mut playlists: Vec<YouTubePlaylistPayload> = Vec::new();
    let mut playlist_page_token: Option<String> = None;
    loop {
        let mut url =
            "https://www.googleapis.com/youtube/v3/playlists?part=snippet&mine=true&maxResults=50"
                .to_string();
        if let Some(token) = playlist_page_token.as_deref() {
            url.push_str("&pageToken=");
            url.push_str(token);
        }

        let page: YouTubeListResponse<YouTubePlaylistPayload> =
            youtube_get_json(&client, access_token, &url).await?;
        playlists_count += page.items.len();
        playlists.extend(page.items);
        playlist_page_token = page.next_page_token;
        if playlist_page_token.is_none() {
            break;
        }
    }

    // ── Normalized playlist dual-write ──────────────────────────────────
    let playlist_repo = PlaylistRepository::new(pool);
    let sync_ts = Utc::now();

    // Write liked videos to normalized table
    {
        let liked_norm: Vec<UpsertPlaylistTrack> = tracks
            .iter()
            .filter(|t| t.source_type.as_deref() == Some("liked_video"))
            .enumerate()
            .map(|(i, t)| {
                let raw_id = t
                    .provider_track_id
                    .strip_prefix("video:")
                    .unwrap_or(&t.provider_track_id);
                UpsertPlaylistTrack {
                    provider_track_id: raw_id.to_string(),
                    track_name: t.track_name.clone(),
                    album_name: None,
                    artist_name: t.artist_name.clone(),
                    position: i as i32,
                    added_at: t.added_at,
                }
            })
            .collect();
        playlist_repo
            .upsert_playlist_and_replace_tracks(
                user_id,
                "youtube_music",
                &UpsertPlaylist {
                    provider_playlist_id: "__liked_videos__".to_string(),
                    name: "Liked Videos".to_string(),
                    description: None,
                    image_url: None,
                    owner_name: None,
                    owner_id: None,
                    is_public: Some(false),
                    is_collaborative: false,
                    source_type: "liked_videos".to_string(),
                    provider_track_count: Some(liked_videos_count as i32),
                    snapshot_id: None,
                },
                &liked_norm,
            )
            .await?;
    }

    for playlist in playlists {
        let playlist_upsert = UpsertPlaylist {
            provider_playlist_id: playlist.id.clone(),
            name: playlist.snippet.title.clone(),
            description: None,
            image_url: None,
            owner_name: None,
            owner_id: None,
            is_public: None,
            is_collaborative: false,
            source_type: "playlist".to_string(),
            provider_track_count: None,
            snapshot_id: None,
        };

        let mut normalized_tracks: Vec<UpsertPlaylistTrack> = Vec::new();
        let mut playlist_item_token: Option<String> = None;
        let mut item_position = 0i32;
        loop {
            let mut url = format!(
                "https://www.googleapis.com/youtube/v3/playlistItems?part=snippet&playlistId={}&maxResults=50",
                playlist.id
            );
            if let Some(token) = playlist_item_token.as_deref() {
                url.push_str("&pageToken=");
                url.push_str(token);
            }

            let page: YouTubeListResponse<YouTubePlaylistItemPayload> =
                youtube_get_json(&client, access_token, &url).await?;

            for item in page.items {
                let artist = item
                    .snippet
                    .channel_title
                    .clone()
                    .unwrap_or_else(|| "Unknown Channel".to_string());

                // Legacy table
                tracks.push(ImportTrack {
                    provider_track_id: format!("playlist_item:{}", item.id),
                    track_name: item.snippet.title.clone(),
                    album_name: None,
                    artist_name: artist.clone(),
                    source_type: Some("playlist_item".to_string()),
                    playlist_name: Some(playlist.snippet.title.clone()),
                    added_at: parse_youtube_timestamp(item.snippet.published_at.as_deref()),
                });

                // Normalized table
                normalized_tracks.push(UpsertPlaylistTrack {
                    provider_track_id: item.id.clone(),
                    track_name: item.snippet.title.clone(),
                    album_name: None,
                    artist_name: artist,
                    position: item_position,
                    added_at: parse_youtube_timestamp(item.snippet.published_at.as_deref()),
                });

                playlist_items_count += 1;
                item_position += 1;
            }

            playlist_item_token = page.next_page_token;
            if playlist_item_token.is_none() {
                break;
            }
        }

        playlist_repo
            .upsert_playlist_and_replace_tracks(
                user_id,
                "youtube_music",
                &playlist_upsert,
                &normalized_tracks,
            )
            .await?;
    }

    let mut subscription_page_token: Option<String> = None;
    loop {
        let mut url = "https://www.googleapis.com/youtube/v3/subscriptions?part=snippet&mine=true&maxResults=50"
            .to_string();
        if let Some(token) = subscription_page_token.as_deref() {
            url.push_str("&pageToken=");
            url.push_str(token);
        }

        let page: YouTubeListResponse<YouTubeSubscriptionPayload> =
            youtube_get_json(&client, access_token, &url).await?;

        for subscription in page.items {
            let channel_title = subscription.snippet.title;
            let channel_id = subscription
                .snippet
                .resource_id
                .and_then(|resource| resource.channel_id)
                .unwrap_or_else(|| format!("unknown-{}", subscriptions_count));
            tracks.push(ImportTrack {
                provider_track_id: format!("subscription:{}", channel_id),
                track_name: format!("[Subscription] {}", channel_title),
                album_name: None,
                artist_name: channel_title,
                source_type: Some("subscription".to_string()),
                playlist_name: None,
                added_at: None,
            });
            subscriptions_count += 1;
        }

        subscription_page_token = page.next_page_token;
        if subscription_page_token.is_none() {
            break;
        }
    }

    {
        let sub_norm: Vec<UpsertPlaylistTrack> = tracks
            .iter()
            .filter(|t| t.source_type.as_deref() == Some("subscription"))
            .enumerate()
            .map(|(i, t)| {
                let raw_id = t
                    .provider_track_id
                    .strip_prefix("subscription:")
                    .unwrap_or(&t.provider_track_id);
                UpsertPlaylistTrack {
                    provider_track_id: raw_id.to_string(),
                    track_name: t.track_name.clone(),
                    album_name: None,
                    artist_name: t.artist_name.clone(),
                    position: i as i32,
                    added_at: t.added_at,
                }
            })
            .collect();
        playlist_repo
            .upsert_playlist_and_replace_tracks(
                user_id,
                "youtube_music",
                &UpsertPlaylist {
                    provider_playlist_id: "__subscriptions__".to_string(),
                    name: "Subscriptions".to_string(),
                    description: None,
                    image_url: None,
                    owner_name: None,
                    owner_id: None,
                    is_public: Some(false),
                    is_collaborative: false,
                    source_type: "subscriptions".to_string(),
                    provider_track_count: Some(subscriptions_count as i32),
                    snapshot_id: None,
                },
                &sub_norm,
            )
            .await?;
    }

    // Remove playlists deleted since last sync
    playlist_repo
        .delete_stale_playlists(user_id, "youtube_music", sync_ts)
        .await?;

    // ── Legacy table: atomic delete-and-reimport in a single transaction ─
    let imported_tracks = OffenseService::new(pool)
        .delete_and_import_library(
            user_id,
            ImportLibraryRequest {
                provider: "youtube_music".to_string(),
                tracks,
            },
        )
        .await?;

    Ok(YouTubeLibrarySyncSummary {
        imported_tracks,
        liked_videos_synced: liked_videos_count,
        playlist_items_synced: playlist_items_count,
        subscriptions_synced: subscriptions_count,
        playlists_scanned: playlists_count,
    })
}

// ============================================================================
// Redis helper functions for OAuth state
// ============================================================================

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

    let key = format!("youtube_oauth_state:{}", state);
    let value = serde_json::to_string(data).map_err(|e| {
        tracing::error!(error = %e, "Failed to serialize OAuth state");
        AppError::Internal {
            message: Some("Failed to store OAuth state".to_string()),
        }
    })?;

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

    let key = format!("youtube_oauth_state:{}", state);
    let value: Option<String> = conn.get(&key).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get OAuth state from Redis");
        AppError::ExternalServiceError("Failed to retrieve OAuth state".to_string())
    })?;

    let value = value.ok_or_else(|| AppError::InvalidFieldValue {
        field: "state".to_string(),
        message: "Invalid or expired OAuth state".to_string(),
    })?;

    serde_json::from_str(&value).map_err(|e| {
        tracing::warn!(error = %e, "Invalid YouTube OAuth state payload");
        AppError::InvalidFieldValue {
            field: "state".to_string(),
            message: "Invalid or expired OAuth state".to_string(),
        }
    })
}

async fn delete_oauth_state(redis_pool: &deadpool_redis::Pool, state: &str) -> Result<()> {
    use deadpool_redis::redis::AsyncCommands;

    let mut conn = redis_pool.get().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get Redis connection");
        AppError::ExternalServiceError("Failed to connect to session store".to_string())
    })?;

    let key = format!("youtube_oauth_state:{}", state);
    let _: () = conn.del(&key).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to delete OAuth state from Redis");
        AppError::ExternalServiceError("Failed to clean up OAuth state".to_string())
    })?;

    Ok(())
}
