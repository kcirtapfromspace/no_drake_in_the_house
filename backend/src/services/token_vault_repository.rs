//! Token Vault Repository - PostgreSQL persistence layer for connections
//!
//! Provides database CRUD operations for the token vault service,
//! replacing the in-memory DashMap storage with persistent PostgreSQL storage.

use crate::models::{Connection, ConnectionStatus, StreamingProvider};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::{info, instrument, warn};
use uuid::Uuid;

/// Repository for token vault database operations
#[derive(Clone)]
pub struct TokenVaultRepository {
    pool: PgPool,
}

impl TokenVaultRepository {
    /// Create a new repository with the given database pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get a reference to the database pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Insert a new connection into the database
    #[instrument(skip(self, connection), fields(connection_id = %connection.id, user_id = %connection.user_id))]
    pub async fn insert_connection(&self, connection: &Connection) -> Result<()> {
        let scopes: Vec<&str> = connection.scopes.iter().map(|s| s.as_str()).collect();
        let status_str = connection_status_to_str(&connection.status);

        sqlx::query(
            r#"
            INSERT INTO connections (
                id, user_id, provider, provider_user_id, scopes,
                access_token_encrypted, refresh_token_encrypted, token_version,
                expires_at, status, last_health_check, error_code, data_key_id,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(connection.id)
        .bind(connection.user_id)
        .bind(connection.provider.as_str())
        .bind(&connection.provider_user_id)
        .bind(&scopes)
        .bind(&connection.access_token_encrypted)
        .bind(&connection.refresh_token_encrypted)
        .bind(connection.token_version)
        .bind(connection.expires_at)
        .bind(status_str)
        .bind(connection.last_health_check)
        .bind(&connection.error_code)
        .bind(&connection.data_key_id)
        .bind(connection.created_at)
        .bind(connection.updated_at)
        .execute(&self.pool)
        .await?;

        info!(connection_id = %connection.id, "Connection inserted");
        Ok(())
    }

    /// Update an existing connection in the database
    #[instrument(skip(self, connection), fields(connection_id = %connection.id))]
    pub async fn update_connection(&self, connection: &Connection) -> Result<()> {
        let scopes: Vec<&str> = connection.scopes.iter().map(|s| s.as_str()).collect();
        let status_str = connection_status_to_str(&connection.status);

        let result = sqlx::query(
            r#"
            UPDATE connections SET
                provider_user_id = $2,
                scopes = $3,
                access_token_encrypted = $4,
                refresh_token_encrypted = $5,
                token_version = $6,
                expires_at = $7,
                status = $8,
                last_health_check = $9,
                error_code = $10,
                data_key_id = $11
            WHERE id = $1
            "#,
        )
        .bind(connection.id)
        .bind(&connection.provider_user_id)
        .bind(&scopes)
        .bind(&connection.access_token_encrypted)
        .bind(&connection.refresh_token_encrypted)
        .bind(connection.token_version)
        .bind(connection.expires_at)
        .bind(status_str)
        .bind(connection.last_health_check)
        .bind(&connection.error_code)
        .bind(&connection.data_key_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("Connection not found: {}", connection.id));
        }

        info!(connection_id = %connection.id, "Connection updated");
        Ok(())
    }

    /// Upsert a connection (insert or update based on user_id + provider)
    #[instrument(skip(self, connection), fields(user_id = %connection.user_id, provider = %connection.provider))]
    pub async fn upsert_connection(&self, connection: &Connection) -> Result<Connection> {
        let scopes: Vec<&str> = connection.scopes.iter().map(|s| s.as_str()).collect();
        let status_str = connection_status_to_str(&connection.status);

        let row = sqlx::query_as::<_, ConnectionRow>(
            r#"
            INSERT INTO connections (
                id, user_id, provider, provider_user_id, scopes,
                access_token_encrypted, refresh_token_encrypted, token_version,
                expires_at, status, last_health_check, error_code, data_key_id,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            ON CONFLICT (user_id, provider) DO UPDATE SET
                provider_user_id = EXCLUDED.provider_user_id,
                scopes = EXCLUDED.scopes,
                access_token_encrypted = EXCLUDED.access_token_encrypted,
                refresh_token_encrypted = EXCLUDED.refresh_token_encrypted,
                token_version = connections.token_version + 1,
                expires_at = EXCLUDED.expires_at,
                status = EXCLUDED.status,
                last_health_check = EXCLUDED.last_health_check,
                error_code = EXCLUDED.error_code,
                data_key_id = EXCLUDED.data_key_id
            RETURNING id, user_id, provider, provider_user_id, scopes,
                      access_token_encrypted, refresh_token_encrypted, token_version,
                      expires_at, status, last_health_check, error_code, data_key_id,
                      created_at, updated_at
            "#,
        )
        .bind(connection.id)
        .bind(connection.user_id)
        .bind(connection.provider.as_str())
        .bind(&connection.provider_user_id)
        .bind(&scopes)
        .bind(&connection.access_token_encrypted)
        .bind(&connection.refresh_token_encrypted)
        .bind(connection.token_version)
        .bind(connection.expires_at)
        .bind(status_str)
        .bind(connection.last_health_check)
        .bind(&connection.error_code)
        .bind(&connection.data_key_id)
        .bind(connection.created_at)
        .bind(connection.updated_at)
        .fetch_one(&self.pool)
        .await?;

        let result = row_to_connection(row)?;
        info!(connection_id = %result.id, "Connection upserted");
        Ok(result)
    }

    /// Get a connection by ID
    #[instrument(skip(self))]
    pub async fn get_connection(&self, connection_id: Uuid) -> Result<Option<Connection>> {
        let row = sqlx::query_as::<_, ConnectionRow>(
            r#"
            SELECT id, user_id, provider, provider_user_id, scopes,
                   access_token_encrypted, refresh_token_encrypted, token_version,
                   expires_at, status, last_health_check, error_code, data_key_id,
                   created_at, updated_at
            FROM connections
            WHERE id = $1
            "#,
        )
        .bind(connection_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(row_to_connection(r)?)),
            None => Ok(None),
        }
    }

    /// Get a connection by user ID and provider
    #[instrument(skip(self))]
    pub async fn get_connection_by_user_provider(
        &self,
        user_id: Uuid,
        provider: &StreamingProvider,
    ) -> Result<Option<Connection>> {
        let row = sqlx::query_as::<_, ConnectionRow>(
            r#"
            SELECT id, user_id, provider, provider_user_id, scopes,
                   access_token_encrypted, refresh_token_encrypted, token_version,
                   expires_at, status, last_health_check, error_code, data_key_id,
                   created_at, updated_at
            FROM connections
            WHERE user_id = $1 AND provider = $2
            "#,
        )
        .bind(user_id)
        .bind(provider.as_str())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(row_to_connection(r)?)),
            None => Ok(None),
        }
    }

    /// Get all connections for a user
    #[instrument(skip(self))]
    pub async fn get_user_connections(&self, user_id: Uuid) -> Result<Vec<Connection>> {
        let rows = sqlx::query_as::<_, ConnectionRow>(
            r#"
            SELECT id, user_id, provider, provider_user_id, scopes,
                   access_token_encrypted, refresh_token_encrypted, token_version,
                   expires_at, status, last_health_check, error_code, data_key_id,
                   created_at, updated_at
            FROM connections
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_connection).collect()
    }

    /// Get all connections (for statistics and background tasks)
    #[instrument(skip(self))]
    pub async fn get_all_connections(&self) -> Result<Vec<Connection>> {
        let rows = sqlx::query_as::<_, ConnectionRow>(
            r#"
            SELECT id, user_id, provider, provider_user_id, scopes,
                   access_token_encrypted, refresh_token_encrypted, token_version,
                   expires_at, status, last_health_check, error_code, data_key_id,
                   created_at, updated_at
            FROM connections
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_connection).collect()
    }

    /// Get connections by status
    #[instrument(skip(self))]
    pub async fn get_connections_by_status(
        &self,
        status: &ConnectionStatus,
    ) -> Result<Vec<Connection>> {
        let status_str = connection_status_to_str(status);

        let rows = sqlx::query_as::<_, ConnectionRow>(
            r#"
            SELECT id, user_id, provider, provider_user_id, scopes,
                   access_token_encrypted, refresh_token_encrypted, token_version,
                   expires_at, status, last_health_check, error_code, data_key_id,
                   created_at, updated_at
            FROM connections
            WHERE status = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(status_str)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_connection).collect()
    }

    /// Get connections that need a health check (not checked within interval)
    #[instrument(skip(self))]
    pub async fn get_connections_needing_health_check(
        &self,
        interval_hours: i64,
    ) -> Result<Vec<Connection>> {
        let threshold = Utc::now() - chrono::Duration::hours(interval_hours);

        let rows = sqlx::query_as::<_, ConnectionRow>(
            r#"
            SELECT id, user_id, provider, provider_user_id, scopes,
                   access_token_encrypted, refresh_token_encrypted, token_version,
                   expires_at, status, last_health_check, error_code, data_key_id,
                   created_at, updated_at
            FROM connections
            WHERE status = 'active'
              AND (last_health_check IS NULL OR last_health_check < $1)
            ORDER BY last_health_check ASC NULLS FIRST
            "#,
        )
        .bind(threshold)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_connection).collect()
    }

    /// Get connections that need token refresh (expiring within threshold in minutes)
    #[instrument(skip(self))]
    pub async fn get_connections_needing_refresh(
        &self,
        threshold_minutes: i64,
    ) -> Result<Vec<Connection>> {
        let threshold = Utc::now() + chrono::Duration::minutes(threshold_minutes);

        let rows = sqlx::query_as::<_, ConnectionRow>(
            r#"
            SELECT id, user_id, provider, provider_user_id, scopes,
                   access_token_encrypted, refresh_token_encrypted, token_version,
                   expires_at, status, last_health_check, error_code, data_key_id,
                   created_at, updated_at
            FROM connections
            WHERE status = 'active'
              AND expires_at IS NOT NULL
              AND expires_at < $1
              AND refresh_token_encrypted IS NOT NULL
            ORDER BY expires_at ASC
            "#,
        )
        .bind(threshold)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_connection).collect()
    }

    /// Get connections expiring within the specified number of hours
    /// Used by the proactive token refresh background job (US-011)
    ///
    /// Returns active connections that:
    /// - Have not yet expired (expires_at > NOW())
    /// - Will expire within the threshold hours
    /// - Have a refresh token available
    #[instrument(skip(self))]
    pub async fn get_connections_expiring_within_hours(
        &self,
        threshold_hours: i64,
        limit: Option<i64>,
    ) -> Result<Vec<Connection>> {
        let threshold = Utc::now() + chrono::Duration::hours(threshold_hours);

        let rows = sqlx::query_as::<_, ConnectionRow>(
            r#"
            SELECT id, user_id, provider, provider_user_id, scopes,
                   access_token_encrypted, refresh_token_encrypted, token_version,
                   expires_at, status, last_health_check, error_code, data_key_id,
                   created_at, updated_at
            FROM connections
            WHERE status = 'active'
              AND expires_at IS NOT NULL
              AND expires_at > NOW()
              AND expires_at < $1
              AND refresh_token_encrypted IS NOT NULL
            ORDER BY expires_at ASC
            LIMIT $2
            "#,
        )
        .bind(threshold)
        .bind(limit.unwrap_or(1000))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_connection).collect()
    }

    /// Get connections by data key ID (for key rotation)
    #[instrument(skip(self))]
    pub async fn get_connections_by_data_key_id(
        &self,
        data_key_id: &str,
    ) -> Result<Vec<Connection>> {
        let rows = sqlx::query_as::<_, ConnectionRow>(
            r#"
            SELECT id, user_id, provider, provider_user_id, scopes,
                   access_token_encrypted, refresh_token_encrypted, token_version,
                   expires_at, status, last_health_check, error_code, data_key_id,
                   created_at, updated_at
            FROM connections
            WHERE data_key_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(data_key_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_connection).collect()
    }

    /// Update connection status
    #[instrument(skip(self))]
    pub async fn update_connection_status(
        &self,
        connection_id: Uuid,
        status: &ConnectionStatus,
        error_code: Option<&str>,
    ) -> Result<()> {
        let status_str = connection_status_to_str(status);

        let result = sqlx::query(
            r#"
            UPDATE connections
            SET status = $2, error_code = $3
            WHERE id = $1
            "#,
        )
        .bind(connection_id)
        .bind(status_str)
        .bind(error_code)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("Connection not found: {}", connection_id));
        }

        info!(connection_id = %connection_id, status = %status_str, "Connection status updated");
        Ok(())
    }

    /// Update last health check timestamp (uses current time)
    #[instrument(skip(self))]
    pub async fn update_health_check(&self, connection_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE connections
            SET last_health_check = NOW(), updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(connection_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("Connection not found: {}", connection_id));
        }

        Ok(())
    }

    /// Update last health check timestamp with a specific time
    #[instrument(skip(self))]
    pub async fn update_connection_health_check(
        &self,
        connection_id: Uuid,
        checked_at: DateTime<Utc>,
    ) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE connections
            SET last_health_check = $2, updated_at = $2
            WHERE id = $1
            "#,
        )
        .bind(connection_id)
        .bind(checked_at)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("Connection not found: {}", connection_id));
        }

        Ok(())
    }

    /// Delete a connection
    #[instrument(skip(self))]
    pub async fn delete_connection(&self, connection_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM connections
            WHERE id = $1
            "#,
        )
        .bind(connection_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            warn!(connection_id = %connection_id, "Connection not found for deletion");
        } else {
            info!(connection_id = %connection_id, "Connection deleted");
        }

        Ok(())
    }

    /// Delete all connections for a user
    #[instrument(skip(self))]
    pub async fn delete_user_connections(&self, user_id: Uuid) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM connections
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        let count = result.rows_affected();
        info!(user_id = %user_id, count = count, "User connections deleted");
        Ok(count)
    }

    /// Get connection statistics
    #[instrument(skip(self))]
    pub async fn get_statistics(&self) -> Result<ConnectionStatistics> {
        let row = sqlx::query_as::<_, ConnectionStatsRow>(
            r#"
            SELECT
                COUNT(*) as total_connections,
                COUNT(*) FILTER (WHERE status = 'active') as active_connections,
                COUNT(*) FILTER (WHERE status = 'expired') as expired_connections,
                COUNT(*) FILTER (WHERE status = 'revoked') as revoked_connections,
                COUNT(*) FILTER (WHERE status = 'error') as error_connections,
                COUNT(*) FILTER (WHERE status = 'needs_reauth') as needs_reauth_connections,
                COUNT(*) FILTER (
                    WHERE status = 'active'
                    AND expires_at IS NOT NULL
                    AND expires_at < NOW() + INTERVAL '5 minutes'
                ) as connections_needing_refresh
            FROM connections
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(ConnectionStatistics {
            total_connections: row.total_connections.unwrap_or(0) as usize,
            active_connections: row.active_connections.unwrap_or(0) as usize,
            expired_connections: row.expired_connections.unwrap_or(0) as usize,
            revoked_connections: row.revoked_connections.unwrap_or(0) as usize,
            error_connections: row.error_connections.unwrap_or(0) as usize,
            needs_reauth_connections: row.needs_reauth_connections.unwrap_or(0) as usize,
            connections_needing_refresh: row.connections_needing_refresh.unwrap_or(0) as usize,
        })
    }
}

/// Connection statistics
#[derive(Debug, Clone)]
pub struct ConnectionStatistics {
    pub total_connections: usize,
    pub active_connections: usize,
    pub expired_connections: usize,
    pub revoked_connections: usize,
    pub error_connections: usize,
    pub needs_reauth_connections: usize,
    pub connections_needing_refresh: usize,
}

// Database row type for connections
#[derive(sqlx::FromRow)]
struct ConnectionRow {
    id: Uuid,
    user_id: Uuid,
    provider: String,
    provider_user_id: Option<String>,
    scopes: Option<Vec<String>>,
    access_token_encrypted: Option<String>,
    refresh_token_encrypted: Option<String>,
    token_version: Option<i32>,
    expires_at: Option<DateTime<Utc>>,
    status: Option<String>,
    last_health_check: Option<DateTime<Utc>>,
    error_code: Option<String>,
    data_key_id: Option<String>,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
}

// Database row type for statistics
#[derive(sqlx::FromRow)]
struct ConnectionStatsRow {
    total_connections: Option<i64>,
    active_connections: Option<i64>,
    expired_connections: Option<i64>,
    revoked_connections: Option<i64>,
    error_connections: Option<i64>,
    needs_reauth_connections: Option<i64>,
    connections_needing_refresh: Option<i64>,
}

/// Convert a database row to a Connection model
fn row_to_connection(row: ConnectionRow) -> Result<Connection> {
    let provider = StreamingProvider::from_str(&row.provider)
        .ok_or_else(|| anyhow!("Invalid provider: {}", row.provider))?;

    let status = str_to_connection_status(row.status.as_deref().unwrap_or("active"));

    Ok(Connection {
        id: row.id,
        user_id: row.user_id,
        provider,
        provider_user_id: row.provider_user_id.unwrap_or_default(),
        scopes: row.scopes.unwrap_or_default(),
        access_token_encrypted: row.access_token_encrypted,
        refresh_token_encrypted: row.refresh_token_encrypted,
        token_version: row.token_version.unwrap_or(1),
        expires_at: row.expires_at,
        status,
        last_health_check: row.last_health_check,
        error_code: row.error_code,
        data_key_id: row.data_key_id,
        created_at: row.created_at.unwrap_or_else(Utc::now),
        updated_at: row.updated_at.unwrap_or_else(Utc::now),
    })
}

/// Convert ConnectionStatus to database string
fn connection_status_to_str(status: &ConnectionStatus) -> &'static str {
    match status {
        ConnectionStatus::Active => "active",
        ConnectionStatus::Expired => "expired",
        ConnectionStatus::Revoked => "revoked",
        ConnectionStatus::Error => "error",
        ConnectionStatus::NeedsReauth => "needs_reauth",
    }
}

/// Convert database string to ConnectionStatus
fn str_to_connection_status(s: &str) -> ConnectionStatus {
    match s.to_lowercase().as_str() {
        "active" => ConnectionStatus::Active,
        "expired" => ConnectionStatus::Expired,
        "revoked" => ConnectionStatus::Revoked,
        "error" => ConnectionStatus::Error,
        "needs_reauth" => ConnectionStatus::NeedsReauth,
        _ => ConnectionStatus::Error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_status_conversion() {
        assert_eq!(
            connection_status_to_str(&ConnectionStatus::Active),
            "active"
        );
        assert_eq!(
            connection_status_to_str(&ConnectionStatus::Expired),
            "expired"
        );
        assert_eq!(
            connection_status_to_str(&ConnectionStatus::Revoked),
            "revoked"
        );
        assert_eq!(connection_status_to_str(&ConnectionStatus::Error), "error");
        assert_eq!(
            connection_status_to_str(&ConnectionStatus::NeedsReauth),
            "needs_reauth"
        );

        assert_eq!(str_to_connection_status("active"), ConnectionStatus::Active);
        assert_eq!(
            str_to_connection_status("expired"),
            ConnectionStatus::Expired
        );
        assert_eq!(
            str_to_connection_status("revoked"),
            ConnectionStatus::Revoked
        );
        assert_eq!(str_to_connection_status("error"), ConnectionStatus::Error);
        assert_eq!(
            str_to_connection_status("needs_reauth"),
            ConnectionStatus::NeedsReauth
        );
        assert_eq!(str_to_connection_status("ACTIVE"), ConnectionStatus::Active);
        assert_eq!(str_to_connection_status("unknown"), ConnectionStatus::Error);
    }
}
