use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

pub const PROVIDER_LIBRARY_SYNC_RUNNING_TTL_SECONDS: u64 = 60 * 60;
pub const PROVIDER_LIBRARY_SYNC_STATUS_TTL_SECONDS: u64 = 60 * 60 * 24;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderLibrarySyncCounts {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracks_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub albums_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artists_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playlists_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imported_items_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderLibrarySyncStatus {
    pub state: String,
    pub message: String,
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(flatten)]
    pub counts: ProviderLibrarySyncCounts,
}

impl ProviderLibrarySyncStatus {
    pub fn idle(provider_name: &str) -> Self {
        Self {
            state: "idle".to_string(),
            message: format!("No {} library sync has been started yet.", provider_name),
            started_at: Utc::now().to_rfc3339(),
            completed_at: None,
            counts: ProviderLibrarySyncCounts::default(),
        }
    }

    pub fn running(message: impl Into<String>, started_at: DateTime<Utc>) -> Self {
        Self {
            state: "running".to_string(),
            message: message.into(),
            started_at: started_at.to_rfc3339(),
            completed_at: None,
            counts: ProviderLibrarySyncCounts::default(),
        }
    }

    pub fn completed(
        message: impl Into<String>,
        started_at: DateTime<Utc>,
        completed_at: DateTime<Utc>,
        counts: ProviderLibrarySyncCounts,
    ) -> Self {
        Self {
            state: "completed".to_string(),
            message: message.into(),
            started_at: started_at.to_rfc3339(),
            completed_at: Some(completed_at.to_rfc3339()),
            counts,
        }
    }

    pub fn failed(
        message: impl Into<String>,
        started_at: DateTime<Utc>,
        completed_at: DateTime<Utc>,
    ) -> Self {
        Self {
            state: "failed".to_string(),
            message: message.into(),
            started_at: started_at.to_rfc3339(),
            completed_at: Some(completed_at.to_rfc3339()),
            counts: ProviderLibrarySyncCounts::default(),
        }
    }
}

pub fn imported_items_count(value: i32) -> Option<usize> {
    usize::try_from(value).ok()
}

fn provider_library_sync_status_key(provider_key: &str, user_id: Uuid) -> String {
    format!(
        "provider_library_sync_status:{}:{}",
        provider_key.to_ascii_lowercase(),
        user_id
    )
}

pub async fn store_provider_library_sync_status(
    redis_pool: &deadpool_redis::Pool,
    provider_key: &str,
    user_id: Uuid,
    status: &ProviderLibrarySyncStatus,
    ttl_seconds: u64,
) -> Result<(), AppError> {
    use deadpool_redis::redis::AsyncCommands;

    let key = provider_library_sync_status_key(provider_key, user_id);
    let value = serde_json::to_string(status).map_err(|e| AppError::Internal {
        message: Some(format!(
            "Failed to serialize {provider_key} sync status: {e}"
        )),
    })?;

    let mut conn = redis_pool.get().await.map_err(|e| AppError::Internal {
        message: Some(format!("Failed to acquire Redis connection: {e}")),
    })?;

    let _: () = conn
        .set_ex(key, value, ttl_seconds)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to store {provider_key} sync status: {e}")),
        })?;

    Ok(())
}

pub async fn get_provider_library_sync_status(
    redis_pool: &deadpool_redis::Pool,
    provider_key: &str,
    user_id: Uuid,
) -> Result<Option<ProviderLibrarySyncStatus>, AppError> {
    use deadpool_redis::redis::AsyncCommands;

    let key = provider_library_sync_status_key(provider_key, user_id);
    let mut conn = redis_pool.get().await.map_err(|e| AppError::Internal {
        message: Some(format!("Failed to acquire Redis connection: {e}")),
    })?;

    let value: Option<String> = conn.get(key).await.map_err(|e| AppError::Internal {
        message: Some(format!("Failed to fetch {provider_key} sync status: {e}")),
    })?;

    value
        .map(|raw| {
            serde_json::from_str(&raw).map_err(|e| AppError::Internal {
                message: Some(format!("Failed to parse {provider_key} sync status: {e}")),
            })
        })
        .transpose()
}
