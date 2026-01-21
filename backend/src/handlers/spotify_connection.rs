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
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::oauth::{
    OAuthConfig, OAuthFlowResponse, OAuthProviderType, OAuthTokens, OAuthUserInfo,
};
use crate::models::user::AuthenticatedUser;
use crate::services::oauth::OAuthProvider;
use crate::services::oauth_spotify::SpotifyOAuthProvider;
use crate::services::OAuthTokenEncryption;
use crate::AppState;
use std::collections::HashMap;

/// Required Spotify scopes for DNP enforcement
pub const SPOTIFY_CONNECTION_SCOPES: &[&str] = &[
    "user-library-read",
    "user-library-modify",
    "playlist-read-private",
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
    let redirect_uri = std::env::var("SPOTIFY_CONNECTION_REDIRECT_URI").unwrap_or_else(|_| {
        "http://localhost:3000/api/v1/connections/spotify/callback".to_string()
    });

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

    // Check if user already has an active Spotify connection
    let existing_connection =
        get_user_spotify_connection(&state.db_pool, authenticated_user.id).await?;
    if let Some(conn) = existing_connection {
        if conn.status == "active" {
            tracing::warn!(
                user_id = %authenticated_user.id,
                connection_id = %conn.id,
                "User already has an active Spotify connection"
            );
            return Err(AppError::InvalidFieldValue {
                field: "provider".to_string(),
                message:
                    "You already have an active Spotify connection. Disconnect first to reconnect."
                        .to_string(),
            });
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
    let redirect_uri = query.redirect_uri.unwrap_or_else(|| {
        std::env::var("SPOTIFY_CONNECTION_REDIRECT_URI").unwrap_or_else(|_| {
            "http://localhost:3000/api/v1/connections/spotify/callback".to_string()
        })
    });

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

    // Delete state from Redis to prevent replay attacks
    delete_oauth_state(&state.redis_pool, &request.state).await?;

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

    // Get user info from Spotify through circuit breaker (US-026)
    let access_token = tokens.access_token.clone();
    let user_info = state
        .circuit_breaker
        .execute_typed(OAuthProviderType::Spotify, || async {
            spotify_provider.get_user_info(&access_token).await
        })
        .await?;

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
    let connection_id = store_spotify_connection(
        &state.db_pool,
        state_data.user_id,
        &user_info.provider_user_id,
        &access_token_encrypted,
        refresh_token_encrypted.as_deref(),
        expires_at,
    )
    .await?;

    tracing::info!(
        user_id = %state_data.user_id,
        connection_id = %connection_id,
        provider_user_id = %user_info.provider_user_id,
        "Spotify connection created successfully"
    );

    Ok((
        StatusCode::OK,
        Json(SpotifyCallbackResponse {
            success: true,
            connection_id,
            provider_user_id: user_info.provider_user_id,
            status: "Active".to_string(),
            message: "Spotify account connected successfully".to_string(),
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

// Database helper functions

#[derive(Debug, sqlx::FromRow)]
struct ConnectionRecord {
    id: Uuid,
    provider_user_id: Option<String>,
    status: String,
    scopes: Option<Vec<String>>,
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
    provider_user_id: &str,
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
