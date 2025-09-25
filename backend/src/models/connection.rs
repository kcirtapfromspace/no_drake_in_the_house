use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Connection {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: Option<String>,
    pub scopes: Vec<String>,
    pub access_token_encrypted: Option<String>,
    pub refresh_token_encrypted: Option<String>,
    pub token_version: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: String,
    pub last_health_check: Option<DateTime<Utc>>,
    pub error_code: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ConnectionResponse {
    pub id: Uuid,
    pub provider: String,
    pub status: String,
    pub scopes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_health_check: Option<DateTime<Utc>>,
}

impl From<Connection> for ConnectionResponse {
    fn from(connection: Connection) -> Self {
        ConnectionResponse {
            id: connection.id,
            provider: connection.provider,
            status: connection.status,
            scopes: connection.scopes,
            created_at: connection.created_at,
            last_health_check: connection.last_health_check,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProviderRateState {
    pub provider: String,
    pub remaining: i32,
    pub reset_at: Option<DateTime<Utc>>,
    pub window_size: i32,
    pub updated_at: DateTime<Utc>,
}