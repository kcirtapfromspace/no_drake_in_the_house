use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Action batch for tracking enforcement operations
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ActionBatch {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub idempotency_key: String,
    pub dry_run: bool,
    pub status: ActionBatchStatus,
    pub options: serde_json::Value, // EnforcementOptions as JSON
    pub summary: serde_json::Value, // BatchSummary as JSON
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Status of an action batch
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum ActionBatchStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    PartiallyCompleted,
    Cancelled,
}

/// Individual action item within a batch
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
    pub status: ActionItemStatus,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Status of an individual action item
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum ActionItemStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
    Rolled_back,
}

/// Summary of batch execution results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSummary {
    pub total_actions: u32,
    pub completed_actions: u32,
    pub failed_actions: u32,
    pub skipped_actions: u32,
    pub execution_time_ms: u64,
    pub api_calls_made: u32,
    pub rate_limit_delays_ms: u64,
    pub errors: Vec<BatchError>,
}

/// Error information for batch execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    pub action_id: Uuid,
    pub entity_type: String,
    pub entity_id: String,
    pub error_code: String,
    pub error_message: String,
    pub retry_count: u32,
    pub is_recoverable: bool,
}

/// Execution result for a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecutionResult {
    pub batch_id: Uuid,
    pub status: ActionBatchStatus,
    pub summary: BatchSummary,
    pub completed_actions: Vec<ActionItem>,
    pub failed_actions: Vec<ActionItem>,
    pub rollback_info: Option<RollbackInfo>,
}

/// Information about rollback operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackInfo {
    pub rollback_batch_id: Uuid,
    pub rollback_actions: Vec<ActionItem>,
    pub rollback_summary: BatchSummary,
    pub partial_rollback: bool,
    pub rollback_errors: Vec<BatchError>,
}

/// Request to execute an enforcement batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteBatchRequest {
    pub plan_id: Uuid,
    pub idempotency_key: Option<String>,
    pub execute_immediately: bool,
    pub batch_size: Option<u32>,
    pub rate_limit_buffer_ms: Option<u64>,
}

/// Request to rollback a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackBatchRequest {
    pub batch_id: Uuid,
    pub action_ids: Option<Vec<Uuid>>, // If None, rollback entire batch
    pub reason: String,
}

/// Progress update for batch execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProgress {
    pub batch_id: Uuid,
    pub total_actions: u32,
    pub completed_actions: u32,
    pub failed_actions: u32,
    pub current_action: Option<String>,
    pub estimated_remaining_ms: u64,
    pub rate_limit_status: RateLimitStatus,
}

/// Rate limiting status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStatus {
    pub requests_remaining: u32,
    pub reset_time: DateTime<Utc>,
    pub current_delay_ms: u64,
}

impl ActionBatch {
    pub fn new(
        user_id: Uuid,
        provider: String,
        idempotency_key: String,
        dry_run: bool,
        options: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            provider,
            idempotency_key,
            dry_run,
            status: ActionBatchStatus::Pending,
            options,
            summary: serde_json::json!({}),
            created_at: Utc::now(),
            completed_at: None,
        }
    }

    pub fn mark_completed(&mut self, summary: BatchSummary) {
        self.status = if summary.failed_actions > 0 {
            ActionBatchStatus::PartiallyCompleted
        } else {
            ActionBatchStatus::Completed
        };
        self.summary = serde_json::to_value(summary).unwrap_or_default();
        self.completed_at = Some(Utc::now());
    }

    pub fn mark_failed(&mut self, error: String) {
        self.status = ActionBatchStatus::Failed;
        self.completed_at = Some(Utc::now());
        
        // Add error to summary
        if let Ok(mut summary) = serde_json::from_value::<BatchSummary>(self.summary.clone()) {
            summary.errors.push(BatchError {
                action_id: Uuid::new_v4(),
                entity_type: "batch".to_string(),
                entity_id: self.id.to_string(),
                error_code: "BATCH_FAILED".to_string(),
                error_message: error,
                retry_count: 0,
                is_recoverable: false,
            });
            self.summary = serde_json::to_value(summary).unwrap_or_default();
        }
    }
}

impl ActionItem {
    pub fn new(
        batch_id: Uuid,
        entity_type: String,
        entity_id: String,
        action: String,
        before_state: Option<serde_json::Value>,
    ) -> Self {
        let idempotency_key = format!("{}_{}_{}_{}", batch_id, entity_type, entity_id, action);
        
        Self {
            id: Uuid::new_v4(),
            batch_id,
            entity_type,
            entity_id,
            action,
            idempotency_key: Some(idempotency_key),
            before_state,
            after_state: None,
            status: ActionItemStatus::Pending,
            error_message: None,
            created_at: Utc::now(),
        }
    }

    pub fn mark_completed(&mut self, after_state: serde_json::Value) {
        self.status = ActionItemStatus::Completed;
        self.after_state = Some(after_state);
    }

    pub fn mark_failed(&mut self, error: String) {
        self.status = ActionItemStatus::Failed;
        self.error_message = Some(error);
    }

    pub fn mark_skipped(&mut self, reason: String) {
        self.status = ActionItemStatus::Skipped;
        self.error_message = Some(reason);
    }

    pub fn can_rollback(&self) -> bool {
        matches!(self.status, ActionItemStatus::Completed) && self.before_state.is_some()
    }
}

impl Default for BatchSummary {
    fn default() -> Self {
        Self {
            total_actions: 0,
            completed_actions: 0,
            failed_actions: 0,
            skipped_actions: 0,
            execution_time_ms: 0,
            api_calls_made: 0,
            rate_limit_delays_ms: 0,
            errors: Vec::new(),
        }
    }
}

impl std::fmt::Display for ActionBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionBatchStatus::Pending => write!(f, "pending"),
            ActionBatchStatus::InProgress => write!(f, "in_progress"),
            ActionBatchStatus::Completed => write!(f, "completed"),
            ActionBatchStatus::Failed => write!(f, "failed"),
            ActionBatchStatus::PartiallyCompleted => write!(f, "partially_completed"),
            ActionBatchStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::fmt::Display for ActionItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionItemStatus::Pending => write!(f, "pending"),
            ActionItemStatus::InProgress => write!(f, "in_progress"),
            ActionItemStatus::Completed => write!(f, "completed"),
            ActionItemStatus::Failed => write!(f, "failed"),
            ActionItemStatus::Skipped => write!(f, "skipped"),
            ActionItemStatus::Rolled_back => write!(f, "rolled_back"),
        }
    }
}