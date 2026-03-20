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
use crate::models::offense::{ImportLibraryRequest, ImportTrack};
use crate::models::user::AuthenticatedUser;
use crate::services::tidal::{TidalConfig, TidalService};
use crate::services::OAuthTokenEncryption;
use crate::services::OffenseService;
use crate::AppState;
use ndith_core::config::provider_callback_uri;

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

    // Determine redirect URI
    let redirect_uri = query
        .redirect_uri
        .unwrap_or_else(|| provider_callback_uri("tidal"));
    let requested_scopes = TidalService::configured_oauth_scopes();

    let tidal_service = TidalService::new(TidalConfig {
        client_id: tidal_config.client_id,
        client_secret: tidal_config.client_secret,
        redirect_uri: redirect_uri.clone(),
        client_unique_key: None,
    });
    let (code_verifier, code_challenge) = if tidal_service.uses_pkce() {
        let (verifier, challenge) = generate_pkce_verifier_and_challenge();
        (Some(verifier), Some(challenge))
    } else {
        (None, None)
    };

    // Get authorization URL
    let authorization_url = tidal_service.get_auth_url(&oauth_state, code_challenge.as_deref());

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

    // Delete state from Redis to prevent replay attacks
    delete_oauth_state(&state.redis_pool, &request.state).await?;

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
        .unwrap_or_else(|| provider_callback_uri("tidal"));

    let tidal_service = TidalService::new(TidalConfig {
        client_id: tidal_config.client_id,
        client_secret: tidal_config.client_secret,
        redirect_uri,
        client_unique_key: None,
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
    let connection = get_user_tidal_connection(&state.db_pool, authenticated_user.id)
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: "Tidal connection".to_string(),
        })?;

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

        return Err(AppError::ExternalServiceError(
            "Tidal access token is missing or empty. Disconnect and reconnect Tidal, then try again."
                .to_string(),
        ));
    }
    let summary =
        sync_tidal_library_to_user_library(&state.db_pool, authenticated_user.id, &access_token)
            .await?;

    Ok((
        StatusCode::OK,
        Json(TidalLibrarySyncResponse {
            success: true,
            message: format!(
                "Synced Tidal library: {} imported items ({} favorite tracks, {} favorite artists, {} favorite albums, {} playlists)",
                summary.imported_tracks,
                summary.favorite_tracks_synced,
                summary.favorite_artists_synced,
                summary.favorite_albums_synced,
                summary.playlists_synced
            ),
            summary,
        }),
    ))
}

// ============================================================================
// Database helper functions
// ============================================================================

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

    let mut tracks: Vec<ImportTrack> = Vec::new();

    let favorite_tracks_synced = scan_result.library.favorite_tracks.len();
    let favorite_artists_synced = scan_result.library.favorite_artists.len();
    let favorite_albums_synced = scan_result.library.favorite_albums.len();
    let playlists_synced = scan_result.library.playlists.len();

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

    for playlist in scan_result.library.playlists {
        let creator_name = playlist
            .creator
            .and_then(|creator| creator.username)
            .unwrap_or_else(|| "Unknown Creator".to_string());
        let playlist_title = playlist.title;

        tracks.push(ImportTrack {
            provider_track_id: format!("playlist:{}", playlist.uuid),
            track_name: format!("[Playlist] {}", playlist_title),
            album_name: None,
            artist_name: creator_name,
            source_type: Some("playlist".to_string()),
            playlist_name: Some(playlist_title),
            added_at: Some(playlist.last_updated),
        });
    }

    sqlx::query("DELETE FROM user_library_tracks WHERE user_id = $1 AND provider = $2")
        .bind(user_id)
        .bind("tidal")
        .execute(pool)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

    let imported_tracks = if tracks.is_empty() {
        0
    } else {
        OffenseService::new(pool)
            .import_library(
                user_id,
                ImportLibraryRequest {
                    provider: "tidal".to_string(),
                    tracks,
                },
            )
            .await?
    };

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
