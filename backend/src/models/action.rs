use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ActionBatch {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub idempotency_key: Option<String>,
    pub dry_run: bool,
    pub status: String,
    pub options: serde_json::Value,
    pub summary: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ActionItem {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub entity_type: String,
    pub entity_id: String,
    pub action: String,
    pub idempotency_key: Option<String>,
    pub before_state: Option<serde_json::Value>,
    pub after_state: Option<serde_json::Value>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ActionBatchResponse {
    pub id: Uuid,
    pub provider: String,
    pub status: String,
    pub dry_run: bool,
    pub summary: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<ActionBatch> for ActionBatchResponse {
    fn from(batch: ActionBatch) -> Self {
        ActionBatchResponse {
            id: batch.id,
            provider: batch.provider,
            status: batch.status,
            dry_run: batch.dry_run,
            summary: batch.summary,
            created_at: batch.created_at,
            completed_at: batch.completed_at,
        }
    }
}