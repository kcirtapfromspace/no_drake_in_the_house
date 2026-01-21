//! Connection Health Check Handlers
//!
//! Provides endpoints for checking the health status of provider connections.
//! US-012: Users can see if their provider connections are healthy.

use axum::{extract::State, http::StatusCode, Json};
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::user::AuthenticatedUser;
use crate::AppState;

/// Health status for a connection
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionHealthStatus {
    /// Connection is active and working
    Active,
    /// Connection will expire within 24 hours
    ExpiringSoon,
    /// Connection needs re-authentication
    NeedsReauth,
    /// Connection has an error
    Error,
}

/// Individual connection health information
#[derive(Debug, Serialize)]
pub struct ConnectionHealth {
    /// Unique connection identifier
    pub id: Uuid,
    /// Provider name (spotify, apple_music, etc.)
    pub provider: String,
    /// Provider's user ID for this connection
    pub provider_user_id: Option<String>,
    /// Current health status
    pub health_status: ConnectionHealthStatus,
    /// When the token expires (if applicable)
    pub expires_at: Option<DateTime<Utc>>,
    /// When the connection was last used
    pub last_used_at: Option<DateTime<Utc>>,
    /// Error message if status is error
    pub error_message: Option<String>,
    /// OAuth scopes granted
    pub scopes: Option<Vec<String>>,
}

/// Response for GET /api/v1/connections
#[derive(Debug, Serialize)]
pub struct ConnectionsHealthResponse {
    /// List of all user connections with health status
    pub connections: Vec<ConnectionHealth>,
    /// Total number of connections
    pub total: usize,
    /// Number of healthy (active) connections
    pub healthy_count: usize,
    /// Number of connections needing attention
    pub needs_attention_count: usize,
}

/// Database record for a connection
#[derive(Debug, FromRow)]
struct ConnectionRecord {
    id: Uuid,
    provider: String,
    provider_user_id: Option<String>,
    status: String,
    expires_at: Option<DateTime<Utc>>,
    last_health_check: Option<DateTime<Utc>>,
    error_code: Option<String>,
    scopes: Option<Vec<String>>,
}

/// GET /api/v1/connections
///
/// Returns a list of all provider connections for the authenticated user
/// with their current health status.
///
/// Health statuses:
/// - `active`: Connection is working normally
/// - `expiring_soon`: Token expires within 24 hours
/// - `needs_reauth`: Connection requires re-authentication
/// - `error`: Connection has an error
pub async fn get_connections_health_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<(StatusCode, Json<ConnectionsHealthResponse>)> {
    tracing::debug!(
        user_id = %authenticated_user.id,
        "Fetching connection health status"
    );

    let connections = get_user_connections(&state.db_pool, authenticated_user.id).await?;

    let connection_healths: Vec<ConnectionHealth> = connections
        .into_iter()
        .map(|conn| {
            let health_status = determine_health_status(&conn);
            ConnectionHealth {
                id: conn.id,
                provider: conn.provider,
                provider_user_id: conn.provider_user_id,
                health_status,
                expires_at: conn.expires_at,
                last_used_at: conn.last_health_check,
                error_message: conn.error_code,
                scopes: conn.scopes,
            }
        })
        .collect();

    let total = connection_healths.len();
    let healthy_count = connection_healths
        .iter()
        .filter(|c| c.health_status == ConnectionHealthStatus::Active)
        .count();
    let needs_attention_count = total - healthy_count;

    tracing::info!(
        user_id = %authenticated_user.id,
        total = total,
        healthy = healthy_count,
        needs_attention = needs_attention_count,
        "Connection health check completed"
    );

    Ok((
        StatusCode::OK,
        Json(ConnectionsHealthResponse {
            connections: connection_healths,
            total,
            healthy_count,
            needs_attention_count,
        }),
    ))
}

/// Determine the health status of a connection based on its current state
fn determine_health_status(conn: &ConnectionRecord) -> ConnectionHealthStatus {
    // Check for error status first
    if conn.status.to_lowercase() == "error" {
        return ConnectionHealthStatus::Error;
    }

    // Check for needs_reauth status
    if conn.status.to_lowercase() == "needs_reauth" || conn.status.to_lowercase() == "needsreauth" {
        return ConnectionHealthStatus::NeedsReauth;
    }

    // Check for revoked or expired status
    if conn.status.to_lowercase() == "revoked" || conn.status.to_lowercase() == "expired" {
        return ConnectionHealthStatus::NeedsReauth;
    }

    // Check if token is expiring soon (within 24 hours)
    if let Some(expires_at) = conn.expires_at {
        let now = Utc::now();

        // Already expired
        if expires_at <= now {
            return ConnectionHealthStatus::NeedsReauth;
        }

        // Expires within 24 hours
        let expiring_soon_threshold = now + Duration::hours(24);
        if expires_at <= expiring_soon_threshold {
            return ConnectionHealthStatus::ExpiringSoon;
        }
    }

    // Default to active if status is active and not expiring soon
    if conn.status.to_lowercase() == "active" {
        return ConnectionHealthStatus::Active;
    }

    // Unknown status, treat as needing attention
    ConnectionHealthStatus::NeedsReauth
}

/// Fetch all connections for a user from the database
async fn get_user_connections(pool: &PgPool, user_id: Uuid) -> Result<Vec<ConnectionRecord>> {
    let rows = sqlx::query_as::<_, ConnectionRecord>(
        r#"
        SELECT
            id,
            provider,
            provider_user_id,
            status,
            expires_at,
            last_health_check,
            error_code,
            scopes
        FROM connections
        WHERE user_id = $1
        ORDER BY provider ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, user_id = %user_id, "Failed to fetch user connections");
        AppError::DatabaseQueryFailed(e)
    })?;

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_health_status_active() {
        let conn = ConnectionRecord {
            id: Uuid::new_v4(),
            provider: "spotify".to_string(),
            provider_user_id: Some("user123".to_string()),
            status: "active".to_string(),
            expires_at: Some(Utc::now() + Duration::days(7)),
            last_health_check: Some(Utc::now()),
            error_code: None,
            scopes: Some(vec!["user-library-read".to_string()]),
        };

        assert_eq!(
            determine_health_status(&conn),
            ConnectionHealthStatus::Active
        );
    }

    #[test]
    fn test_determine_health_status_expiring_soon() {
        let conn = ConnectionRecord {
            id: Uuid::new_v4(),
            provider: "spotify".to_string(),
            provider_user_id: Some("user123".to_string()),
            status: "active".to_string(),
            expires_at: Some(Utc::now() + Duration::hours(12)), // 12 hours < 24 hours
            last_health_check: Some(Utc::now()),
            error_code: None,
            scopes: Some(vec!["user-library-read".to_string()]),
        };

        assert_eq!(
            determine_health_status(&conn),
            ConnectionHealthStatus::ExpiringSoon
        );
    }

    #[test]
    fn test_determine_health_status_expired() {
        let conn = ConnectionRecord {
            id: Uuid::new_v4(),
            provider: "spotify".to_string(),
            provider_user_id: Some("user123".to_string()),
            status: "active".to_string(),
            expires_at: Some(Utc::now() - Duration::hours(1)), // Already expired
            last_health_check: Some(Utc::now()),
            error_code: None,
            scopes: Some(vec!["user-library-read".to_string()]),
        };

        assert_eq!(
            determine_health_status(&conn),
            ConnectionHealthStatus::NeedsReauth
        );
    }

    #[test]
    fn test_determine_health_status_needs_reauth() {
        let conn = ConnectionRecord {
            id: Uuid::new_v4(),
            provider: "spotify".to_string(),
            provider_user_id: Some("user123".to_string()),
            status: "needs_reauth".to_string(),
            expires_at: Some(Utc::now() + Duration::days(7)),
            last_health_check: Some(Utc::now()),
            error_code: Some("refresh_token_revoked".to_string()),
            scopes: Some(vec!["user-library-read".to_string()]),
        };

        assert_eq!(
            determine_health_status(&conn),
            ConnectionHealthStatus::NeedsReauth
        );
    }

    #[test]
    fn test_determine_health_status_error() {
        let conn = ConnectionRecord {
            id: Uuid::new_v4(),
            provider: "spotify".to_string(),
            provider_user_id: Some("user123".to_string()),
            status: "error".to_string(),
            expires_at: Some(Utc::now() + Duration::days(7)),
            last_health_check: Some(Utc::now()),
            error_code: Some("provider_api_error".to_string()),
            scopes: Some(vec!["user-library-read".to_string()]),
        };

        assert_eq!(
            determine_health_status(&conn),
            ConnectionHealthStatus::Error
        );
    }

    #[test]
    fn test_determine_health_status_revoked() {
        let conn = ConnectionRecord {
            id: Uuid::new_v4(),
            provider: "spotify".to_string(),
            provider_user_id: Some("user123".to_string()),
            status: "revoked".to_string(),
            expires_at: Some(Utc::now() + Duration::days(7)),
            last_health_check: Some(Utc::now()),
            error_code: None,
            scopes: Some(vec!["user-library-read".to_string()]),
        };

        assert_eq!(
            determine_health_status(&conn),
            ConnectionHealthStatus::NeedsReauth
        );
    }

    #[test]
    fn test_determine_health_status_no_expiry() {
        // Some providers (like Apple Music) may not have expiry
        let conn = ConnectionRecord {
            id: Uuid::new_v4(),
            provider: "apple_music".to_string(),
            provider_user_id: Some("user123".to_string()),
            status: "active".to_string(),
            expires_at: None,
            last_health_check: Some(Utc::now()),
            error_code: None,
            scopes: None,
        };

        assert_eq!(
            determine_health_status(&conn),
            ConnectionHealthStatus::Active
        );
    }
}
