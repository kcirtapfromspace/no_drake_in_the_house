//! Apple Music Authentication Handlers
//!
//! These handlers support the Apple Music MusicKit JS flow:
//! 1. Frontend requests developer token from /api/v1/apple-music/auth/developer-token
//! 2. Frontend initializes MusicKit JS with the developer token
//! 3. User authorizes in the MusicKit JS popup
//! 4. Frontend sends the Music User Token to /api/v1/apple-music/auth/connect
//! 5. Backend stores the token and creates a connection

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::AppState;

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
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to generate developer token: {}", e)),
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
    Json(request): Json<ConnectAppleMusicRequest>,
) -> Result<Json<ConnectAppleMusicResponse>, AppError> {
    // For now, use a hardcoded user_id - in production this would come from auth middleware
    let user_id = state.test_user_id.unwrap_or_else(Uuid::new_v4);

    // Create the connection using the Music User Token
    let connection = state
        .apple_music_service
        .create_connection(
            user_id,
            request.music_user_token.clone(),
            Some(request.music_user_token), // Store as both access and refresh
        )
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to connect Apple Music: {}", e)),
        })?;

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
) -> Result<Json<AppleMusicConnectionStatus>, AppError> {
    let user_id = state.test_user_id.unwrap_or_else(Uuid::new_v4);

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
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = state.test_user_id.unwrap_or_else(Uuid::new_v4);

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
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = state.test_user_id.unwrap_or_else(Uuid::new_v4);

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
