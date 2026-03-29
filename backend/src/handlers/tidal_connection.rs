//! Tidal Connection Handlers
//!
//! Handles Tidal OAuth flow for provider connection.
//! This allows users to connect their Tidal account for DNP list enforcement.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use base64::{
    engine::general_purpose::{self, URL_SAFE_NO_PAD},
    Engine as _,
};
use chrono::{Duration, Utc};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
use crate::services::tidal::{TidalConfig, TidalService};
use crate::services::OAuthTokenEncryption;
use crate::services::OffenseService;
use crate::services::PlaylistRepository;
use crate::AppState;

const TIDAL_SYNC_STATUS_KEY: &str = "tidal";
const TIDAL_PROVIDER_LABEL: &str = "Tidal";

/// Query parameters for the authorize endpoint
#[derive(Debug, Deserialize)]
pub struct TidalAuthorizeQuery {
    /// Optional redirect URI override
    pub redirect_uri: Option<String>,
}

/// Response from the authorize endpoint
#[derive(Debug, Serialize)]
pub struct TidalAuthorizeResponse {
    pub authorization_url: String,
    pub state: String,
    pub scopes: Vec<String>,
    pub already_connected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Request body for the callback endpoint
#[derive(Debug, Deserialize)]
pub struct TidalCallbackRequest {
    pub code: String,
    pub state: String,
    #[serde(default)]
    pub redirect_uri: Option<String>,
}

/// Response from the callback endpoint
#[derive(Debug, Serialize)]
pub struct TidalCallbackResponse {
    pub success: bool,
    pub connection_id: Uuid,
    pub provider_user_id: String,
    pub status: String,
    pub message: String,
    pub sync_summary: Option<TidalLibrarySyncSummary>,
    pub sync_warning: Option<String>,
}

/// Connection status response
#[derive(Debug, Serialize)]
pub struct TidalConnectionStatus {
    pub connected: bool,
    pub connection_id: Option<Uuid>,
    pub provider_user_id: Option<String>,
    pub status: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub expires_at: Option<String>,
    pub last_health_check: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TidalLibrarySyncSummary {
    pub imported_tracks: i32,
    pub favorite_tracks_synced: usize,
    pub favorite_artists_synced: usize,
    pub favorite_albums_synced: usize,
    pub playlists_synced: usize,
}

#[derive(Debug, Serialize)]
pub struct TidalLibrarySyncResponse {
    pub success: bool,
    pub summary: TidalLibrarySyncSummary,
    pub message: String,
}

/// OAuth state stored in Redis for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OAuthStateData {
    pub user_id: Uuid,
    #[serde(default)]
    pub redirect_uri: Option<String>,
    #[serde(default)]
    pub code_verifier: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
}

/// GET /api/v1/connections/tidal/authorize
///
/// Initiates Tidal OAuth flow for provider connection.
/// Returns an authorization URL that the user should be redirected to.
pub async fn tidal_authorize_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Query(query): Query<TidalAuthorizeQuery>,
) -> Result<(StatusCode, Json<TidalAuthorizeResponse>)> {
    tracing::info!(
        user_id = %authenticated_user.id,
        "Initiating Tidal connection OAuth flow"
    );

    // If a connection exists, allow reconnect to refresh stale/expired tokens.
    let existing_connection =
        get_user_tidal_connection(&state.db_pool, authenticated_user.id).await?;
    if let Some(conn) = existing_connection {
        if conn.status == "active" {
            tracing::info!(
                user_id = %authenticated_user.id,
                connection_id = %conn.id,
                "Existing active Tidal connection found; proceeding with reconnect flow"
            );
        }
    }

    // Create Tidal service
    let tidal_config = TidalConfig::from_env().map_err(|e| {
        tracing::error!(error = %e, "Failed to create Tidal config");
        AppError::ConfigurationError {
            message: "Tidal OAuth is not properly configured".to_string(),
        }
    })?;

    // Generate state for CSRF protection
    let oauth_state = Uuid::new_v4().to_string();

    // Determine redirect URI: validate against the configured callback URL.
    let default_redirect_uri = tidal_config.redirect_uri.clone();
    let redirect_uri = match query.redirect_uri {
        Some(ref uri) if uri != &default_redirect_uri => {
            tracing::warn!(
                user_id = %authenticated_user.id,
                requested_uri = %uri,
                expected_uri = %default_redirect_uri,
                "Rejected non-allowlisted Tidal redirect_uri"
            );
            return Err(AppError::InvalidFieldValue {
                field: "redirect_uri".to_string(),
                message: "Provided redirect_uri does not match the configured callback URL"
                    .to_string(),
            });
        }
        _ => default_redirect_uri,
    };
    let requested_scopes = TidalService::configured_oauth_scopes();

    let tidal_service = TidalService::new(TidalConfig {
        client_id: tidal_config.client_id,
        client_secret: tidal_config.client_secret,
        redirect_uri: redirect_uri.clone(),
        client_unique_key: tidal_config.client_unique_key,
    });
    let (code_verifier, code_challenge) = if tidal_service.uses_pkce() {
        let (verifier, challenge) = generate_pkce_verifier_and_challenge();
        (Some(verifier), Some(challenge))
    } else {
        (None, None)
    };

    // Get authorization URL
    let authorization_url = tidal_service.get_auth_url(&oauth_state, code_challenge.as_deref());

    tracing::info!(
        user_id = %authenticated_user.id,
        authorization_url = %authorization_url,
        "Tidal authorize URL constructed (full URL for debugging)"
    );

    // Store state in Redis for validation during callback
    let state_data = OAuthStateData {
        user_id: authenticated_user.id,
        redirect_uri: Some(redirect_uri.clone()),
        code_verifier,
        created_at: Utc::now(),
    };

    store_oauth_state(&state.redis_pool, &oauth_state, &state_data).await?;

    tracing::info!(
        user_id = %authenticated_user.id,
        state = %oauth_state,
        redirect_uri = %redirect_uri,
        oauth_mode = %tidal_service.oauth_mode_name(),
        scopes = ?requested_scopes,
        "Tidal OAuth flow initiated successfully"
    );

    Ok((
        StatusCode::OK,
        Json(TidalAuthorizeResponse {
            authorization_url,
            state: oauth_state,
            scopes: requested_scopes,
            already_connected: false,
            message: None,
        }),
    ))
}

/// POST /api/v1/connections/tidal/callback
///
/// Handles the OAuth callback from Tidal.
/// Exchanges the authorization code for tokens and stores the connection.
pub async fn tidal_callback_handler(
    State(state): State<AppState>,
    Json(request): Json<TidalCallbackRequest>,
) -> Result<(StatusCode, Json<TidalCallbackResponse>)> {
    tracing::info!(
        state = %request.state,
        code_length = request.code.len(),
        "Processing Tidal connection callback"
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

    // Create Tidal service
    let tidal_config = TidalConfig::from_env().map_err(|e| {
        tracing::error!(error = %e, "Failed to create Tidal config");
        AppError::ConfigurationError {
            message: "Tidal OAuth is not properly configured".to_string(),
        }
    })?;

    let redirect_uri = state_data
        .redirect_uri
        .as_deref()
        .map(str::trim)
        .filter(|uri| !uri.is_empty())
        .map(str::to_string)
        .or_else(|| {
            request
                .redirect_uri
                .as_deref()
                .map(str::trim)
                .filter(|uri| !uri.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| tidal_config.redirect_uri.clone());

    let tidal_service = TidalService::new(TidalConfig {
        client_id: tidal_config.client_id,
        client_secret: tidal_config.client_secret,
        redirect_uri,
        client_unique_key: tidal_config.client_unique_key,
    });
    tracing::info!(
        user_id = %state_data.user_id,
        oauth_mode = %tidal_service.oauth_mode_name(),
        "Exchanging Tidal OAuth callback code"
    );

    // Exchange code for tokens
    let token_response = tidal_service
        .exchange_code(&request.code, state_data.code_verifier.as_deref())
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to exchange Tidal code for tokens");
            AppError::ExternalServiceError(format!(
                "Failed to exchange Tidal authorization code: {}",
                e
            ))
        })?;

    // Extract user info from token response.
    // Some Tidal app configurations return token payload without embedded `user`,
    // so fall back to a profile lookup.
    let provider_user_id = if let Some(user_info) = token_response.user_info.clone() {
        user_info.user_id.to_string()
    } else {
        tracing::warn!(
            user_id = %state_data.user_id,
            "Tidal token response missing user info; falling back to profile lookup"
        );

        let profile = tidal_service
            .get_current_user(&token_response.access_token)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to fetch Tidal user profile after token exchange");
                AppError::ExternalServiceError(format!(
                    "Tidal token exchange succeeded, but user profile lookup failed: {}",
                    e
                ))
            })?;

        profile.id.to_string()
    };

    // Encrypt tokens
    let encryption = OAuthTokenEncryption::new().map_err(|e| {
        tracing::error!(error = %e, "Failed to initialize token encryption");
        AppError::Internal {
            message: Some("Token encryption not available".to_string()),
        }
    })?;

    let access_token_encrypted = encryption
        .encrypt_token(&token_response.access_token)
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to encrypt access token");
            AppError::Internal {
                message: Some("Failed to secure tokens".to_string()),
            }
        })?;

    let refresh_token_encrypted = if let Some(ref refresh_token) = token_response.refresh_token {
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
    let expires_at = Utc::now() + Duration::seconds(token_response.expires_in as i64);
    let granted_scopes = token_response
        .scope
        .as_deref()
        .map(|scope| {
            scope
                .split_whitespace()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect::<Vec<String>>()
        })
        .filter(|scopes| !scopes.is_empty())
        .unwrap_or_else(TidalService::configured_oauth_scopes);

    // Store connection in database
    let connection_id = store_tidal_connection(
        &state.db_pool,
        state_data.user_id,
        &provider_user_id,
        &granted_scopes,
        &access_token_encrypted,
        refresh_token_encrypted.as_deref(),
        Some(expires_at),
    )
    .await?;

    // Now that the connection is stored, delete the OAuth state to prevent replay attacks.
    if let Err(e) = delete_oauth_state(&state.redis_pool, &request.state).await {
        tracing::warn!("Failed to delete Tidal OAuth state: {}", e);
    }

    let (sync_summary, sync_warning) = match sync_tidal_library_to_user_library(
        &state.db_pool,
        state_data.user_id,
        &token_response.access_token,
    )
    .await
    {
        Ok(summary) => (Some(summary), None),
        Err(error) => {
            tracing::warn!(
                user_id = %state_data.user_id,
                error = %error,
                "Tidal connection succeeded but initial library sync failed"
            );
            (
                None,
                Some(
                    "Tidal connected, but automatic library sync failed. Try syncing again from the Music Library page."
                        .to_string(),
                ),
            )
        }
    };

    tracing::info!(
        user_id = %state_data.user_id,
        connection_id = %connection_id,
        provider_user_id = %provider_user_id,
        "Tidal connection created successfully"
    );

    Ok((
        StatusCode::OK,
        Json(TidalCallbackResponse {
            success: true,
            connection_id,
            provider_user_id,
            status: "active".to_string(),
            message: "Tidal account connected successfully".to_string(),
            sync_summary,
            sync_warning,
        }),
    ))
}

fn generate_pkce_verifier_and_challenge() -> (String, String) {
    let verifier: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    (verifier, challenge)
}

/// GET /api/v1/connections/tidal/status
///
/// Returns the status of the user's Tidal connection.
pub async fn tidal_connection_status_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<(StatusCode, Json<TidalConnectionStatus>)> {
    tracing::debug!(
        user_id = %authenticated_user.id,
        "Checking Tidal connection status"
    );

    let connection = get_user_tidal_connection(&state.db_pool, authenticated_user.id).await?;

    match connection {
        Some(conn) => Ok((
            StatusCode::OK,
            Json(TidalConnectionStatus {
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
            Json(TidalConnectionStatus {
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

/// DELETE /api/v1/connections/tidal
///
/// Disconnects the user's Tidal account.
pub async fn tidal_disconnect_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    tracing::info!(
        user_id = %authenticated_user.id,
        "Disconnecting Tidal account"
    );

    let connection = get_user_tidal_connection(&state.db_pool, authenticated_user.id).await?;

    match connection {
        Some(conn) => {
            delete_tidal_connection(&state.db_pool, conn.id).await?;

            tracing::info!(
                user_id = %authenticated_user.id,
                connection_id = %conn.id,
                "Tidal connection deleted"
            );

            Ok((
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "message": "Tidal account disconnected successfully"
                })),
            ))
        }
        None => Err(AppError::NotFound {
            resource: "Tidal connection".to_string(),
        }),
    }
}

/// POST /api/v1/connections/tidal/library/sync
///
/// Fetches Tidal favorites/playlists and imports them into `user_library_tracks`.
pub async fn tidal_library_sync_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<(StatusCode, Json<TidalLibrarySyncResponse>)> {
    if let Some(status) = get_provider_library_sync_status(
        &state.redis_pool,
        TIDAL_SYNC_STATUS_KEY,
        authenticated_user.id,
    )
    .await?
    {
        if status.state == "running" {
            return Ok((
                StatusCode::ACCEPTED,
                Json(TidalLibrarySyncResponse {
                    success: true,
                    message:
                        "Tidal library sync is already running. Check sync status for completion."
                            .to_string(),
                    summary: zero_tidal_sync_summary(),
                }),
            ));
        }
    }

    let connection = get_user_tidal_connection(&state.db_pool, authenticated_user.id)
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: "Tidal connection".to_string(),
        })?;

    let started_at = Utc::now();
    store_provider_library_sync_status(
        &state.redis_pool,
        TIDAL_SYNC_STATUS_KEY,
        authenticated_user.id,
        &ProviderLibrarySyncStatus::running("Tidal library sync is in progress.", started_at),
        PROVIDER_LIBRARY_SYNC_RUNNING_TTL_SECONDS,
    )
    .await?;

    let Some(encrypted_access_token) = connection.access_token_encrypted else {
        // Connection row exists but the token payload is missing. Mark it so the UI prompts a reconnect.
        if let Err(e) = sqlx::query(
            r#"
            UPDATE connections
            SET status = 'needs_reauth',
                error_code = $3
            WHERE user_id = $1 AND provider = $2
            "#,
        )
        .bind(authenticated_user.id)
        .bind("tidal")
        .bind("Tidal access token is unavailable. Disconnect and reconnect Tidal.")
        .execute(&state.db_pool)
        .await
        {
            tracing::warn!(
                user_id = %authenticated_user.id,
                error = %e,
                "Failed to mark Tidal connection as needs_reauth after missing token"
            );
        }

        let _ = store_provider_library_sync_status(
            &state.redis_pool,
            TIDAL_SYNC_STATUS_KEY,
            authenticated_user.id,
            &ProviderLibrarySyncStatus::failed(
                "Tidal access token is unavailable. Disconnect and reconnect Tidal.".to_string(),
                started_at,
                Utc::now(),
            ),
            PROVIDER_LIBRARY_SYNC_STATUS_TTL_SECONDS,
        )
        .await;

        return Err(AppError::ExternalServiceError(
            "Tidal access token is unavailable. Disconnect and reconnect Tidal, then try again."
                .to_string(),
        ));
    };

    let access_token = match decrypt_connection_access_token(&encrypted_access_token).await {
        Ok(token) => token,
        Err(err) => {
            if let Err(e) = sqlx::query(
                r#"
                UPDATE connections
                SET status = 'needs_reauth',
                    error_code = $3
                WHERE user_id = $1 AND provider = $2
                "#,
            )
            .bind(authenticated_user.id)
            .bind("tidal")
            .bind("Tidal access token could not be decrypted. Disconnect and reconnect Tidal.")
            .execute(&state.db_pool)
            .await
            {
                tracing::warn!(
                        user_id = %authenticated_user.id,
                        error = %e,
                    "Failed to mark Tidal connection as needs_reauth after decrypt failure"
                );
            }

            let _ = store_provider_library_sync_status(
                &state.redis_pool,
                TIDAL_SYNC_STATUS_KEY,
                authenticated_user.id,
                &ProviderLibrarySyncStatus::failed(err.to_string(), started_at, Utc::now()),
                PROVIDER_LIBRARY_SYNC_STATUS_TTL_SECONDS,
            )
            .await;

            return Err(err);
        }
    };
    if access_token.trim().is_empty() {
        // Stored token is corrupted/empty. Mark the connection so the UI prompts reconnect.
        if let Err(e) = sqlx::query(
            r#"
            UPDATE connections
            SET status = 'needs_reauth',
                error_code = $3
            WHERE user_id = $1 AND provider = $2
            "#,
        )
        .bind(authenticated_user.id)
        .bind("tidal")
        .bind("Tidal access token is missing or empty. Reconnect Tidal.")
        .execute(&state.db_pool)
        .await
        {
            tracing::warn!(
                user_id = %authenticated_user.id,
                error = %e,
                "Failed to mark Tidal connection as needs_reauth after empty token"
            );
        }

        let _ = store_provider_library_sync_status(
            &state.redis_pool,
            TIDAL_SYNC_STATUS_KEY,
            authenticated_user.id,
            &ProviderLibrarySyncStatus::failed(
                "Tidal access token is missing or empty. Reconnect Tidal.".to_string(),
                started_at,
                Utc::now(),
            ),
            PROVIDER_LIBRARY_SYNC_STATUS_TTL_SECONDS,
        )
        .await;

        return Err(AppError::ExternalServiceError(
            "Tidal access token is missing or empty. Disconnect and reconnect Tidal, then try again."
                .to_string(),
        ));
    }
    // Run sync in background to avoid Cloudflare 524 timeouts.
    let db_pool = state.db_pool.clone();
    let redis_pool = state.redis_pool.clone();
    let user_id = authenticated_user.id;
    tokio::spawn(async move {
        match sync_tidal_library_to_user_library(&db_pool, user_id, &access_token).await {
            Ok(summary) => {
                let success_message = format!(
                    "Synced Tidal library: {} imported items ({} favorite tracks, {} favorite artists, {} favorite albums, {} playlists)",
                    summary.imported_tracks,
                    summary.favorite_tracks_synced,
                    summary.favorite_artists_synced,
                    summary.favorite_albums_synced,
                    summary.playlists_synced
                );
                tracing::info!(
                    user_id = %user_id,
                    imported = summary.imported_tracks,
                    "Tidal library sync completed successfully"
                );

                if let Err(e) = sqlx::query(
                    "UPDATE connections SET status = 'active', last_health_check = NOW(), error_code = NULL WHERE user_id = $1 AND provider = 'tidal'",
                )
                .bind(user_id)
                .execute(&db_pool)
                .await
                {
                    tracing::warn!(error = %e, "Failed to update connection after Tidal sync");
                }

                if let Err(error) = store_provider_library_sync_status(
                    &redis_pool,
                    TIDAL_SYNC_STATUS_KEY,
                    user_id,
                    &ProviderLibrarySyncStatus::completed(
                        success_message,
                        started_at,
                        Utc::now(),
                        tidal_sync_status_counts(&summary),
                    ),
                    PROVIDER_LIBRARY_SYNC_STATUS_TTL_SECONDS,
                )
                .await
                {
                    tracing::error!(
                        user_id = %user_id,
                        error = %error,
                        "Failed to persist Tidal sync completion status"
                    );
                }
            }
            Err(error) => {
                let message = map_tidal_sync_error(&error);
                tracing::error!(
                    user_id = %user_id,
                    error = %message,
                    "Tidal library sync failed in background"
                );

                if let Err(status_error) = store_provider_library_sync_status(
                    &redis_pool,
                    TIDAL_SYNC_STATUS_KEY,
                    user_id,
                    &ProviderLibrarySyncStatus::failed(message, started_at, Utc::now()),
                    PROVIDER_LIBRARY_SYNC_STATUS_TTL_SECONDS,
                )
                .await
                {
                    tracing::error!(
                        user_id = %user_id,
                        error = %status_error,
                        "Failed to persist Tidal sync failure status"
                    );
                }
            }
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(TidalLibrarySyncResponse {
            success: true,
            message: "Tidal library sync started. Check sync status for progress.".to_string(),
            summary: zero_tidal_sync_summary(),
        }),
    ))
}

pub async fn tidal_library_sync_status_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<Json<ProviderLibrarySyncStatus>> {
    let status = get_provider_library_sync_status(
        &state.redis_pool,
        TIDAL_SYNC_STATUS_KEY,
        authenticated_user.id,
    )
    .await?
    .unwrap_or_else(|| ProviderLibrarySyncStatus::idle(TIDAL_PROVIDER_LABEL));

    Ok(Json(status))
}

fn map_tidal_sync_error(error: &AppError) -> String {
    let raw = error.to_string();
    let lowered = raw.to_ascii_lowercase();

    if lowered.contains("rate limited") || lowered.contains("429") || lowered.contains("too many requests") {
        "Tidal is temporarily rate-limiting requests. Please wait a few minutes and try again.".to_string()
    } else if lowered.contains("401") || lowered.contains("403") || lowered.contains("unauthorized") || lowered.contains("forbidden") || lowered.contains("expired") {
        "Tidal authorization failed. Disconnect and reconnect Tidal, then sync again.".to_string()
    } else if lowered.contains("406") || lowered.contains("not acceptable") {
        "Tidal API rejected the request. This is a known compatibility issue — the team is working on it.".to_string()
    } else {
        raw
    }
}

// ============================================================================
// Database helper functions
// ============================================================================

fn zero_tidal_sync_summary() -> TidalLibrarySyncSummary {
    TidalLibrarySyncSummary {
        imported_tracks: 0,
        favorite_tracks_synced: 0,
        favorite_artists_synced: 0,
        favorite_albums_synced: 0,
        playlists_synced: 0,
    }
}

fn tidal_sync_status_counts(summary: &TidalLibrarySyncSummary) -> ProviderLibrarySyncCounts {
    ProviderLibrarySyncCounts {
        tracks_count: Some(summary.favorite_tracks_synced),
        albums_count: Some(summary.favorite_albums_synced),
        artists_count: Some(summary.favorite_artists_synced),
        playlists_count: Some(summary.playlists_synced),
        imported_items_count: imported_items_count(summary.imported_tracks),
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

async fn get_user_tidal_connection(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<ConnectionRecord>> {
    let connection = sqlx::query_as::<_, ConnectionRecord>(
        r#"
        SELECT id, provider_user_id, status, scopes, access_token_encrypted, expires_at, last_health_check
        FROM connections
        WHERE user_id = $1 AND provider = 'tidal'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to query Tidal connection");
        AppError::DatabaseQueryFailed(e)
    })?;

    Ok(connection)
}

async fn store_tidal_connection(
    pool: &PgPool,
    user_id: Uuid,
    provider_user_id: &str,
    scopes: &[String],
    access_token_encrypted: &[u8],
    refresh_token_encrypted: Option<&[u8]>,
    expires_at: Option<chrono::DateTime<Utc>>,
) -> Result<Uuid> {
    use base64::{engine::general_purpose, Engine as _};

    let access_token_b64 = general_purpose::STANDARD.encode(access_token_encrypted);
    let refresh_token_b64 = refresh_token_encrypted.map(|t| general_purpose::STANDARD.encode(t));
    let scopes = scopes.to_vec();

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
        VALUES ($1, 'tidal', $2, $3, $4, $5, 1, $6, 'active', NOW())
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
        tracing::error!(error = %e, "Failed to store Tidal connection");
        AppError::DatabaseQueryFailed(e)
    })?;

    Ok(row.0)
}

async fn delete_tidal_connection(pool: &PgPool, connection_id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM connections WHERE id = $1")
        .bind(connection_id)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to delete Tidal connection");
            AppError::DatabaseQueryFailed(e)
        })?;

    Ok(())
}

async fn decrypt_connection_access_token(encoded_token: &str) -> Result<String> {
    let encrypted_bytes = general_purpose::STANDARD
        .decode(encoded_token)
        .map_err(|e| {
            AppError::ExternalServiceError(format!(
                "Stored Tidal token could not be decoded: {}",
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
                "Stored Tidal token could not be decrypted: {}",
                e
            ))
        })
}

async fn sync_tidal_library_to_user_library(
    pool: &PgPool,
    user_id: Uuid,
    access_token: &str,
) -> Result<TidalLibrarySyncSummary> {
    let tidal_service = TidalService::from_env().map_err(|e| AppError::ConfigurationError {
        message: format!("Tidal sync is not properly configured: {}", e),
    })?;

    let scan_result = tidal_service
        .scan_library(access_token, user_id)
        .await
        .map_err(|e| {
            AppError::ExternalServiceError(format!("Failed to scan Tidal library: {}", e))
        })?;

    for warning in &scan_result.metadata.warnings {
        tracing::warn!(
            user_id = %user_id,
            warning = %warning,
            "Tidal library scan warning"
        );
    }

    // If the scan reported missing OAuth scopes, fail fast with an actionable message.
    // Otherwise the sync would "succeed" with 0 imported items, which is misleading.
    let missing_scope = scan_result.metadata.warnings.iter().any(|warning| {
        warning
            .to_ascii_lowercase()
            .contains("missing required scope")
    });
    if missing_scope {
        // Mark the connection as needing re-auth so the UI prompts the user to reconnect.
        if let Err(e) = sqlx::query(
            r#"
            UPDATE connections
            SET status = 'needs_reauth',
                error_code = $3
            WHERE user_id = $1 AND provider = $2
            "#,
        )
        .bind(user_id)
        .bind("tidal")
        .bind("Token missing required scopes for library access. Reconnect Tidal.")
        .execute(pool)
        .await
        {
            tracing::warn!(
                user_id = %user_id,
                error = %e,
                "Failed to mark Tidal connection as needs_reauth after scope error"
            );
        }

        return Err(AppError::OperationNotAllowed {
            reason:
                "Tidal token is missing required scopes for library access. Disconnect and reconnect Tidal, then sync again."
                    .to_string(),
        });
    }

    let favorite_tracks_synced = scan_result.library.favorite_tracks.len();
    let favorite_artists_synced = scan_result.library.favorite_artists.len();
    let favorite_albums_synced = scan_result.library.favorite_albums.len();
    let playlists_synced = scan_result.library.playlists.len();

    // NOTE: This Vec grows as we process the user's entire Tidal library.
    // Consider streaming to the DB in batches if memory becomes a concern.
    let estimated_size = favorite_tracks_synced
        + favorite_artists_synced
        + favorite_albums_synced
        + playlists_synced;
    let mut tracks: Vec<ImportTrack> = Vec::with_capacity(estimated_size);

    for favorite in scan_result.library.favorite_tracks {
        let track = favorite.item;
        let artist_name = track
            .artists
            .first()
            .map(|artist| artist.name.clone())
            .unwrap_or_else(|| "Unknown Artist".to_string());

        tracks.push(ImportTrack {
            provider_track_id: format!("track:{}", track.id),
            track_name: track.title,
            album_name: Some(track.album.title),
            artist_name,
            source_type: Some("favorite_track".to_string()),
            playlist_name: None,
            added_at: Some(favorite.created),
        });
    }

    for favorite in scan_result.library.favorite_artists {
        let artist = favorite.item;
        tracks.push(ImportTrack {
            provider_track_id: format!("artist:{}", artist.id),
            track_name: format!("[Artist] {}", artist.name),
            album_name: None,
            artist_name: artist.name,
            source_type: Some("favorite_artist".to_string()),
            playlist_name: None,
            added_at: Some(favorite.created),
        });
    }

    for favorite in scan_result.library.favorite_albums {
        let album = favorite.item;
        let artist_name = album
            .artists
            .first()
            .map(|artist| artist.name.clone())
            .unwrap_or_else(|| "Unknown Artist".to_string());

        tracks.push(ImportTrack {
            provider_track_id: format!("album:{}", album.id),
            track_name: format!("[Album] {}", album.title),
            album_name: Some(album.title),
            artist_name,
            source_type: Some("favorite_album".to_string()),
            playlist_name: None,
            added_at: Some(favorite.created),
        });
    }

    // ── Normalized playlist dual-write ──────────────────────────────────
    let playlist_repo = PlaylistRepository::new(pool);
    let sync_ts = Utc::now();

    // Write favorites to normalized tables
    {
        let fav_track_norm: Vec<UpsertPlaylistTrack> = tracks
            .iter()
            .filter(|t| t.source_type.as_deref() == Some("favorite_track"))
            .enumerate()
            .map(|(i, t)| {
                let raw_id = t
                    .provider_track_id
                    .strip_prefix("track:")
                    .unwrap_or(&t.provider_track_id);
                UpsertPlaylistTrack {
                    provider_track_id: raw_id.to_string(),
                    track_name: t.track_name.clone(),
                    album_name: t.album_name.clone(),
                    artist_name: t.artist_name.clone(),
                    position: i as i32,
                    added_at: t.added_at,
                }
            })
            .collect();
        playlist_repo
            .upsert_playlist_and_replace_tracks(
                user_id,
                "tidal",
                &UpsertPlaylist {
                    provider_playlist_id: "__favorite_tracks__".to_string(),
                    name: "Favorite Tracks".to_string(),
                    description: None,
                    image_url: None,
                    owner_name: None,
                    owner_id: None,
                    is_public: Some(false),
                    is_collaborative: false,
                    source_type: "favorite_tracks".to_string(),
                    provider_track_count: Some(favorite_tracks_synced as i32),
                    snapshot_id: None,
                },
                &fav_track_norm,
            )
            .await?;

        let fav_album_norm: Vec<UpsertPlaylistTrack> = tracks
            .iter()
            .filter(|t| t.source_type.as_deref() == Some("favorite_album"))
            .enumerate()
            .map(|(i, t)| {
                let raw_id = t
                    .provider_track_id
                    .strip_prefix("album:")
                    .unwrap_or(&t.provider_track_id);
                UpsertPlaylistTrack {
                    provider_track_id: raw_id.to_string(),
                    track_name: t.track_name.clone(),
                    album_name: t.album_name.clone(),
                    artist_name: t.artist_name.clone(),
                    position: i as i32,
                    added_at: t.added_at,
                }
            })
            .collect();
        playlist_repo
            .upsert_playlist_and_replace_tracks(
                user_id,
                "tidal",
                &UpsertPlaylist {
                    provider_playlist_id: "__favorite_albums__".to_string(),
                    name: "Favorite Albums".to_string(),
                    description: None,
                    image_url: None,
                    owner_name: None,
                    owner_id: None,
                    is_public: Some(false),
                    is_collaborative: false,
                    source_type: "favorite_albums".to_string(),
                    provider_track_count: Some(favorite_albums_synced as i32),
                    snapshot_id: None,
                },
                &fav_album_norm,
            )
            .await?;

        let fav_artist_norm: Vec<UpsertPlaylistTrack> = tracks
            .iter()
            .filter(|t| t.source_type.as_deref() == Some("favorite_artist"))
            .enumerate()
            .map(|(i, t)| {
                let raw_id = t
                    .provider_track_id
                    .strip_prefix("artist:")
                    .unwrap_or(&t.provider_track_id);
                UpsertPlaylistTrack {
                    provider_track_id: raw_id.to_string(),
                    track_name: t.track_name.clone(),
                    album_name: t.album_name.clone(),
                    artist_name: t.artist_name.clone(),
                    position: i as i32,
                    added_at: t.added_at,
                }
            })
            .collect();
        playlist_repo
            .upsert_playlist_and_replace_tracks(
                user_id,
                "tidal",
                &UpsertPlaylist {
                    provider_playlist_id: "__favorite_artists__".to_string(),
                    name: "Favorite Artists".to_string(),
                    description: None,
                    image_url: None,
                    owner_name: None,
                    owner_id: None,
                    is_public: Some(false),
                    is_collaborative: false,
                    source_type: "favorite_artists".to_string(),
                    provider_track_count: Some(favorite_artists_synced as i32),
                    snapshot_id: None,
                },
                &fav_artist_norm,
            )
            .await?;
    }

    // Upsert actual Tidal playlists + fetch individual tracks per playlist
    for playlist in scan_result.library.playlists {
        let creator_name = playlist
            .creator
            .as_ref()
            .and_then(|c| c.username.clone())
            .unwrap_or_else(|| "Unknown Creator".to_string());
        let playlist_title = playlist.title.clone();
        let playlist_upsert = UpsertPlaylist {
            provider_playlist_id: playlist.uuid.clone(),
            name: playlist_title.clone(),
            description: playlist.description.clone(),
            image_url: playlist.image.clone(),
            owner_name: Some(creator_name.clone()),
            owner_id: None,
            is_public: Some(playlist.public_playlist),
            is_collaborative: false,
            source_type: "playlist".to_string(),
            provider_track_count: Some(playlist.number_of_tracks as i32),
            snapshot_id: None,
        };

        // Fetch individual playlist tracks from Tidal API
        match tidal_service
            .get_all_playlist_tracks(access_token, &playlist.uuid, "US")
            .await
        {
            Ok(playlist_tracks) => {
                let normalized: Vec<UpsertPlaylistTrack> = playlist_tracks
                    .iter()
                    .enumerate()
                    .map(|(i, pt)| {
                        let artist_name = pt
                            .item
                            .artists
                            .first()
                            .map(|a| a.name.clone())
                            .unwrap_or_else(|| "Unknown Artist".to_string());
                        UpsertPlaylistTrack {
                            provider_track_id: pt.item.id.to_string(),
                            track_name: pt.item.title.clone(),
                            album_name: Some(pt.item.album.title.clone()),
                            artist_name,
                            position: i as i32,
                            added_at: Some(pt.date_added),
                        }
                    })
                    .collect();

                if playlist_upsert.provider_track_count.unwrap_or_default() > 0
                    && normalized.is_empty()
                {
                    let preserved = playlist_repo
                        .touch_playlist_last_synced(user_id, "tidal", &playlist.uuid)
                        .await?;
                    tracing::warn!(
                        playlist_uuid = %playlist.uuid,
                        playlist_name = %playlist_title,
                        provider_track_count = playlist_upsert.provider_track_count,
                        preserved_existing_playlist = preserved,
                        "Tidal playlist returned no track rows despite a non-zero provider track count; preserving existing inventory"
                    );
                } else {
                    // Write to normalized table
                    playlist_repo
                        .upsert_playlist_and_replace_tracks(
                            user_id,
                            "tidal",
                            &playlist_upsert,
                            &normalized,
                        )
                        .await?;
                }

                // Also add to legacy tracks vec for backward compat
                for (i, pt) in playlist_tracks.iter().enumerate() {
                    let artist_name = pt
                        .item
                        .artists
                        .first()
                        .map(|a| a.name.clone())
                        .unwrap_or_else(|| "Unknown Artist".to_string());
                    tracks.push(ImportTrack {
                        provider_track_id: format!(
                            "playlist:{}:{}:{}",
                            playlist.uuid, pt.item.id, i
                        ),
                        track_name: pt.item.title.clone(),
                        album_name: Some(pt.item.album.title.clone()),
                        artist_name,
                        source_type: Some("playlist_track".to_string()),
                        playlist_name: Some(playlist_title.clone()),
                        added_at: Some(pt.date_added),
                    });
                }
            }
            Err(e) => {
                let preserved = playlist_repo
                    .touch_playlist_last_synced(user_id, "tidal", &playlist.uuid)
                    .await?;
                tracing::warn!(
                    playlist_uuid = %playlist.uuid,
                    error = %e,
                    preserved_existing_playlist = preserved,
                    "Failed to fetch Tidal playlist tracks; preserving previously imported inventory"
                );
            }
        }

        // Legacy table — store playlist header entry (always, for backward compat)
        tracks.push(ImportTrack {
            provider_track_id: format!("playlist:{}", playlist.uuid),
            track_name: format!("[Playlist] {}", playlist_title),
            album_name: None,
            artist_name: playlist
                .creator
                .and_then(|c| c.username)
                .unwrap_or_else(|| "Unknown Creator".to_string()),
            source_type: Some("playlist".to_string()),
            playlist_name: Some(playlist_title),
            added_at: Some(playlist.last_updated),
        });
    }

    // Remove playlists deleted since last sync
    playlist_repo
        .delete_stale_playlists(user_id, "tidal", sync_ts)
        .await?;

    // ── Legacy table: atomic delete-and-reimport in a single transaction ─
    let imported_tracks = OffenseService::new(pool)
        .delete_and_import_library(
            user_id,
            ImportLibraryRequest {
                provider: "tidal".to_string(),
                tracks,
            },
        )
        .await?;

    Ok(TidalLibrarySyncSummary {
        imported_tracks,
        favorite_tracks_synced,
        favorite_artists_synced,
        favorite_albums_synced,
        playlists_synced,
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

    let key = format!("tidal_oauth_state:{}", state);
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

    let key = format!("tidal_oauth_state:{}", state);
    let value: Option<String> = conn.get(&key).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get OAuth state from Redis");
        AppError::ExternalServiceError("Failed to retrieve OAuth state".to_string())
    })?;

    let value = value.ok_or_else(|| AppError::InvalidFieldValue {
        field: "state".to_string(),
        message: "Invalid or expired OAuth state".to_string(),
    })?;

    serde_json::from_str(&value).map_err(|e| {
        tracing::warn!(error = %e, "Invalid Tidal OAuth state payload");
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

    let key = format!("tidal_oauth_state:{}", state);
    let _: () = conn.del(&key).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to delete OAuth state from Redis");
        AppError::ExternalServiceError("Failed to delete OAuth state".to_string())
    })?;

    Ok(())
}
