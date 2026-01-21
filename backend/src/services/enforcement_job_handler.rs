use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{self, json};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use uuid::Uuid;

use crate::models::{
    ActionBatch, ActionBatchStatus, ActionItem, ActionItemStatus, BatchError, BatchSummary,
    Connection, ConnectionStatus, EnforcementPlan, BatchExecutionResult, ExecuteBatchRequest,
    StreamingProvider,
};
use crate::models::spotify::{ActionType, EntityType};
use crate::services::{
    JobHandler, Job, JobType, SpotifyService, SpotifyEnforcementService,
    RateLimitingService, JobProgress, NotificationService,
};

/// Result of a single action execution
#[derive(Debug, Clone)]
pub struct ActionExecutionResult {
    pub action_id: Uuid,
    pub entity_type: String,
    pub entity_id: String,
    pub action_type: String,
    pub success: bool,
    pub before_state: Option<serde_json::Value>,
    pub after_state: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub error_code: Option<String>,
    pub is_recoverable: bool,
    pub execution_time_ms: u64,
}

/// Backoff configuration for rate limiting
#[derive(Debug, Clone)]
pub struct BackoffConfig {
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub multiplier: f64,
    pub max_retries: u32,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            initial_delay_ms: 100,
            max_delay_ms: 30000, // 30 seconds max
            multiplier: 2.0,
            max_retries: 5,
        }
    }
}

/// Job handler for enforcement execution operations
pub struct EnforcementJobHandler {
    spotify_service: Arc<SpotifyService>,
    enforcement_service: Arc<SpotifyEnforcementService>,
    rate_limiter: Arc<RateLimitingService>,
    notification_service: Arc<NotificationService>,
    backoff_config: BackoffConfig,
    db_pool: sqlx::PgPool,
}

impl EnforcementJobHandler {
    pub fn new(
        spotify_service: Arc<SpotifyService>,
        enforcement_service: Arc<SpotifyEnforcementService>,
        rate_limiter: Arc<RateLimitingService>,
        notification_service: Arc<NotificationService>,
        db_pool: sqlx::PgPool,
    ) -> Self {
        Self {
            spotify_service,
            enforcement_service,
            rate_limiter,
            notification_service,
            backoff_config: BackoffConfig::default(),
            db_pool,
        }
    }

    pub fn with_backoff_config(mut self, config: BackoffConfig) -> Self {
        self.backoff_config = config;
        self
    }

    /// Check if a connection is healthy enough for enforcement (US-027)
    ///
    /// Returns Some(result) with a skip explanation if enforcement should be skipped,
    /// or None if enforcement can proceed.
    async fn check_connection_health_for_enforcement(
        &self,
        connection: &Connection,
        user_id: Uuid,
        job_id: Uuid,
    ) -> Result<Option<serde_json::Value>> {
        match connection.status {
            ConnectionStatus::NeedsReauth => {
                let reason = connection.error_code.clone().unwrap_or_else(|| {
                    "Connection requires re-authentication".to_string()
                });

                tracing::warn!(
                    job_id = %job_id,
                    user_id = %user_id,
                    provider = %connection.provider,
                    reason = %reason,
                    "Skipping enforcement: connection needs re-authentication"
                );

                // Send notification to user about skipped enforcement
                let _ = self.notification_service
                    .notify_enforcement_skipped(
                        user_id,
                        &connection.provider,
                        &reason,
                    )
                    .await;

                Ok(Some(json!({
                    "status": "skipped",
                    "reason": "connection_needs_reauth",
                    "message": format!(
                        "Enforcement skipped: Your {} connection needs to be re-authenticated. {}",
                        connection.provider, reason
                    ),
                    "provider": connection.provider.to_string(),
                    "action_required": "reconnect",
                    "reconnect_url": format!("/settings/connections/{}/reconnect", connection.provider),
                })))
            }
            ConnectionStatus::Expired => {
                let reason = "Token has expired and could not be refreshed";

                tracing::warn!(
                    job_id = %job_id,
                    user_id = %user_id,
                    provider = %connection.provider,
                    "Skipping enforcement: connection token expired"
                );

                let _ = self.notification_service
                    .notify_enforcement_skipped(
                        user_id,
                        &connection.provider,
                        reason,
                    )
                    .await;

                Ok(Some(json!({
                    "status": "skipped",
                    "reason": "connection_expired",
                    "message": format!(
                        "Enforcement skipped: Your {} token has expired. Please reconnect your account.",
                        connection.provider
                    ),
                    "provider": connection.provider.to_string(),
                    "action_required": "reconnect",
                    "reconnect_url": format!("/settings/connections/{}/reconnect", connection.provider),
                })))
            }
            ConnectionStatus::Revoked => {
                let reason = "Access was revoked by the provider";

                tracing::warn!(
                    job_id = %job_id,
                    user_id = %user_id,
                    provider = %connection.provider,
                    "Skipping enforcement: connection was revoked"
                );

                let _ = self.notification_service
                    .notify_enforcement_skipped(
                        user_id,
                        &connection.provider,
                        reason,
                    )
                    .await;

                Ok(Some(json!({
                    "status": "skipped",
                    "reason": "connection_revoked",
                    "message": format!(
                        "Enforcement skipped: Your {} access was revoked. Please reconnect your account.",
                        connection.provider
                    ),
                    "provider": connection.provider.to_string(),
                    "action_required": "reconnect",
                    "reconnect_url": format!("/settings/connections/{}/reconnect", connection.provider),
                })))
            }
            ConnectionStatus::Error => {
                let reason = connection.error_code.clone().unwrap_or_else(|| {
                    "Connection has an error".to_string()
                });

                tracing::warn!(
                    job_id = %job_id,
                    user_id = %user_id,
                    provider = %connection.provider,
                    reason = %reason,
                    "Skipping enforcement: connection has error"
                );

                let _ = self.notification_service
                    .notify_enforcement_skipped(
                        user_id,
                        &connection.provider,
                        &reason,
                    )
                    .await;

                Ok(Some(json!({
                    "status": "skipped",
                    "reason": "connection_error",
                    "message": format!(
                        "Enforcement skipped: Your {} connection has an error: {}",
                        connection.provider, reason
                    ),
                    "provider": connection.provider.to_string(),
                    "action_required": "check_connection",
                    "reconnect_url": format!("/settings/connections/{}/reconnect", connection.provider),
                })))
            }
            ConnectionStatus::Active => {
                // Connection is healthy, enforcement can proceed
                Ok(None)
            }
        }
    }

    async fn execute_enforcement_job(&self, job: &Job) -> Result<serde_json::Value> {
        // Parse job payload
        let payload = &job.payload;
        let user_id = job.user_id.ok_or_else(|| anyhow!("User ID required for enforcement job"))?;
        let provider = job.provider.as_ref().ok_or_else(|| anyhow!("Provider required for enforcement job"))?;

        // Extract enforcement parameters from payload
        let plan_id: Uuid = serde_json::from_value(
            payload.get("plan_id").cloned().ok_or_else(|| anyhow!("plan_id required"))?
        )?;
        
        let execute_request: ExecuteBatchRequest = serde_json::from_value(
            payload.get("execute_request").cloned().ok_or_else(|| anyhow!("execute_request required"))?
        )?;

        tracing::info!(
            "Starting enforcement job {} for user {} with plan {}",
            job.id,
            user_id,
            plan_id
        );

        // Get user connection for the provider
        let connection = match provider.as_str() {
            "spotify" => {
                self.spotify_service.get_user_connection(user_id).await?
                    .ok_or_else(|| anyhow!("No Spotify connection found for user"))?
            }
            _ => return Err(anyhow!("Unsupported provider: {}", provider)),
        };

        // US-027: Check if connection needs re-authentication
        // Skip enforcement with clear explanation if connection is not healthy
        if let Some(skip_result) = self.check_connection_health_for_enforcement(
            &connection,
            user_id,
            job.id,
        ).await? {
            return Ok(skip_result);
        }

        // Get the enforcement plan (this would typically come from a plan service)
        let plan = self.get_enforcement_plan(&plan_id).await?;

        // Create checkpoint for resumable processing
        let checkpoint = self.rate_limiter.create_checkpoint(
            job.id,
            provider.clone(),
            "enforcement_execution".to_string(),
            plan.actions.len() as u32,
        ).await?;

        // Execute enforcement with rate limiting and progress tracking
        let result = self.execute_with_progress_tracking(
            &connection,
            &plan,
            execute_request,
            job.id,
        ).await?;

        tracing::info!(
            "Completed enforcement job {} with status {:?}",
            job.id,
            result.status
        );

        Ok(serde_json::to_value(result)?)
    }

    async fn execute_with_progress_tracking(
        &self,
        connection: &Connection,
        plan: &EnforcementPlan,
        _request: ExecuteBatchRequest,
        job_id: Uuid,
    ) -> Result<BatchExecutionResult> {
        let start_time = Instant::now();
        let total_actions = plan.actions.len() as u32;
        let batch_id = job_id; // Use job_id as batch_id for consistency

        // Update initial progress
        self.update_progress(
            job_id,
            "Starting enforcement execution",
            0,
            total_actions,
            0.0,
            json!({
                "plan_id": plan.id,
                "total_actions": total_actions,
                "dry_run": plan.options.dry_run
            }),
        ).await?;

        // Execute all actions using the new batch execution method
        let execution_results = self.execute_action_batch(
            connection,
            batch_id,
            plan.actions.clone(),
        ).await?;

        // Aggregate results into summary
        let mut completed_count = 0u32;
        let mut failed_count = 0u32;
        let mut total_execution_time_ms = 0u64;
        let mut rate_limit_delays_ms = 0u64;
        let mut errors = Vec::new();
        let mut completed_actions = Vec::new();
        let mut failed_actions = Vec::new();

        for result in execution_results {
            total_execution_time_ms += result.execution_time_ms;

            if result.success {
                completed_count += 1;
                completed_actions.push(ActionItem {
                    id: result.action_id,
                    batch_id,
                    entity_type: result.entity_type.clone(),
                    entity_id: result.entity_id.clone(),
                    action: result.action_type.clone(),
                    idempotency_key: Some(format!("{}_{}_{}", batch_id, result.entity_id, result.action_type)),
                    before_state: result.before_state.clone(),
                    after_state: result.after_state.clone(),
                    status: ActionItemStatus::Completed,
                    error_message: None,
                    created_at: Utc::now(),
                });
            } else {
                failed_count += 1;

                // Track rate limit delays
                if result.error_code.as_deref() == Some("RATE_LIMITED") {
                    rate_limit_delays_ms += result.execution_time_ms;
                }

                errors.push(BatchError {
                    action_id: result.action_id,
                    entity_type: result.entity_type.clone(),
                    entity_id: result.entity_id.clone(),
                    error_code: result.error_code.clone().unwrap_or_else(|| "UNKNOWN".to_string()),
                    error_message: result.error_message.clone().unwrap_or_default(),
                    retry_count: 0,
                    is_recoverable: result.is_recoverable,
                });

                failed_actions.push(ActionItem {
                    id: result.action_id,
                    batch_id,
                    entity_type: result.entity_type.clone(),
                    entity_id: result.entity_id.clone(),
                    action: result.action_type.clone(),
                    idempotency_key: Some(format!("{}_{}_{}", batch_id, result.entity_id, result.action_type)),
                    before_state: result.before_state.clone(),
                    after_state: None,
                    status: ActionItemStatus::Failed,
                    error_message: result.error_message.clone(),
                    created_at: Utc::now(),
                });
            }
        }

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        // Final progress update
        self.update_progress(
            job_id,
            "Enforcement execution completed",
            total_actions,
            total_actions,
            100.0,
            json!({
                "completed_count": completed_count,
                "failed_count": failed_count,
                "total_actions": total_actions,
                "success_rate": if total_actions > 0 {
                    (completed_count as f64 / total_actions as f64) * 100.0
                } else {
                    100.0 
                }
            }),
        ).await?;

        // Build summary
        let summary = BatchSummary {
            total_actions,
            completed_actions: completed_count,
            failed_actions: failed_count,
            skipped_actions: 0,
            execution_time_ms,
            api_calls_made: completed_count + failed_count, // Each action is an API call
            rate_limit_delays_ms,
            errors,
        };

        // Determine batch status
        let status = if completed_count == total_actions {
            ActionBatchStatus::Completed
        } else if completed_count > 0 {
            ActionBatchStatus::PartiallyCompleted
        } else {
            ActionBatchStatus::Failed
        };

        // Store batch result in database
        self.store_batch_result(batch_id, &status, &summary).await?;

        Ok(BatchExecutionResult {
            batch_id,
            status,
            summary,
            completed_actions,
            failed_actions,
            rollback_info: None,
        })
    }

    /// Store batch execution result in the action_batches table
    async fn store_batch_result(
        &self,
        batch_id: Uuid,
        status: &ActionBatchStatus,
        summary: &BatchSummary,
    ) -> Result<()> {
        let summary_json = serde_json::to_value(summary)?;

        sqlx::query(
            r#"
            UPDATE action_batches
            SET status = $1, summary = $2, completed_at = $3
            WHERE id = $4
            "#
        )
        .bind(status.to_string())
        .bind(&summary_json)
        .bind(Utc::now())
        .bind(batch_id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Execute a batch of enforcement actions with real Spotify API calls
    ///
    /// This method:
    /// 1. Records before_state for each action (enables rollback)
    /// 2. Calls the appropriate Spotify API endpoint
    /// 3. Records after_state on success
    /// 4. Collects errors without stopping the batch
    /// 5. Respects rate limits with automatic backoff
    /// 6. Stores results in the action_items table
    async fn execute_action_batch(
        &self,
        connection: &Connection,
        batch_id: Uuid,
        actions: Vec<crate::models::PlannedAction>,
    ) -> Result<Vec<ActionExecutionResult>> {
        let mut results = Vec::new();
        let mut current_delay_ms = self.backoff_config.initial_delay_ms;
        let mut consecutive_failures = 0;

        // Group actions by type for batch optimization
        let grouped_actions = self.group_actions_for_batch_api(&actions);

        for (action_type, action_group) in grouped_actions {
            match action_type.as_str() {
                "remove_liked_song" => {
                    let batch_results = self.execute_remove_liked_songs_batch(
                        connection,
                        batch_id,
                        action_group,
                        &mut current_delay_ms,
                        &mut consecutive_failures,
                    ).await?;
                    results.extend(batch_results);
                }
                "unfollow_artist" => {
                    let batch_results = self.execute_unfollow_artists_batch(
                        connection,
                        batch_id,
                        action_group,
                        &mut current_delay_ms,
                        &mut consecutive_failures,
                    ).await?;
                    results.extend(batch_results);
                }
                "remove_playlist_track" => {
                    let batch_results = self.execute_remove_playlist_tracks_batch(
                        connection,
                        batch_id,
                        action_group,
                        &mut current_delay_ms,
                        &mut consecutive_failures,
                    ).await?;
                    results.extend(batch_results);
                }
                "remove_saved_album" => {
                    let batch_results = self.execute_remove_saved_albums_batch(
                        connection,
                        batch_id,
                        action_group,
                        &mut current_delay_ms,
                        &mut consecutive_failures,
                    ).await?;
                    results.extend(batch_results);
                }
                _ => {
                    tracing::warn!("Unknown action type: {}", action_type);
                    // Add failed results for unknown action types
                    for action in action_group {
                        results.push(ActionExecutionResult {
                            action_id: action.id,
                            entity_type: action.entity_type.to_string(),
                            entity_id: action.entity_id.clone(),
                            action_type: action_type.clone(),
                            success: false,
                            before_state: None,
                            after_state: None,
                            error_message: Some(format!("Unknown action type: {}", action_type)),
                            error_code: Some("UNKNOWN_ACTION_TYPE".to_string()),
                            is_recoverable: false,
                            execution_time_ms: 0,
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    /// Execute batch removal of liked songs via DELETE /v1/me/tracks
    async fn execute_remove_liked_songs_batch(
        &self,
        connection: &Connection,
        batch_id: Uuid,
        actions: Vec<crate::models::PlannedAction>,
        current_delay_ms: &mut u64,
        consecutive_failures: &mut u32,
    ) -> Result<Vec<ActionExecutionResult>> {
        const SPOTIFY_BATCH_SIZE: usize = 50; // Spotify allows up to 50 tracks per request
        let mut results = Vec::new();

        for chunk in actions.chunks(SPOTIFY_BATCH_SIZE) {
            // Apply rate limiting delay
            self.apply_rate_limit_delay(*current_delay_ms).await;

            let start_time = Instant::now();
            let track_ids: Vec<String> = chunk.iter().map(|a| a.entity_id.clone()).collect();

            // Record before_state for each action in the chunk
            let before_states: Vec<(Uuid, serde_json::Value)> = chunk
                .iter()
                .map(|a| {
                    (a.id, json!({
                        "track_id": a.entity_id,
                        "was_liked": true,
                        "recorded_at": Utc::now().to_rfc3339()
                    }))
                })
                .collect();

            // Call real Spotify API
            let api_result = self.spotify_service
                .remove_liked_songs_batch(connection, &track_ids)
                .await;

            let execution_time_ms = start_time.elapsed().as_millis() as u64;

            match api_result {
                Ok(()) => {
                    // Success - reset backoff and record after_state
                    *current_delay_ms = self.backoff_config.initial_delay_ms;
                    *consecutive_failures = 0;

                    for (action, (action_id, before_state)) in chunk.iter().zip(before_states) {
                        let after_state = json!({
                            "track_id": action.entity_id,
                            "removed": true,
                            "removed_at": Utc::now().to_rfc3339()
                        });

                        // Store result in database
                        self.store_action_result(
                            batch_id,
                            action_id,
                            &action.entity_type.to_string(),
                            &action.entity_id,
                            "remove_liked_song",
                            Some(&before_state),
                            Some(&after_state),
                            ActionItemStatus::Completed,
                            None,
                        ).await?;

                        results.push(ActionExecutionResult {
                            action_id,
                            entity_type: action.entity_type.to_string(),
                            entity_id: action.entity_id.clone(),
                            action_type: "remove_liked_song".to_string(),
                            success: true,
                            before_state: Some(before_state),
                            after_state: Some(after_state),
                            error_message: None,
                            error_code: None,
                            is_recoverable: true,
                            execution_time_ms,
                        });
                    }
                }
                Err(e) => {
                    // Handle rate limiting specifically
                    let (error_code, is_rate_limit) = self.classify_error(&e);

                    if is_rate_limit {
                        *current_delay_ms = self.calculate_backoff_delay(*current_delay_ms);
                        *consecutive_failures += 1;
                    }

                    let is_recoverable = is_rate_limit || *consecutive_failures < self.backoff_config.max_retries;

                    for (action, (action_id, before_state)) in chunk.iter().zip(before_states) {
                        // Store failed result in database
                        self.store_action_result(
                            batch_id,
                            action_id,
                            &action.entity_type.to_string(),
                            &action.entity_id,
                            "remove_liked_song",
                            Some(&before_state),
                            None,
                            ActionItemStatus::Failed,
                            Some(&e.to_string()),
                        ).await?;

                        results.push(ActionExecutionResult {
                            action_id,
                            entity_type: action.entity_type.to_string(),
                            entity_id: action.entity_id.clone(),
                            action_type: "remove_liked_song".to_string(),
                            success: false,
                            before_state: Some(before_state),
                            after_state: None,
                            error_message: Some(e.to_string()),
                            error_code: Some(error_code.clone()),
                            is_recoverable,
                            execution_time_ms,
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    /// Execute batch unfollowing of artists via DELETE /v1/me/following
    async fn execute_unfollow_artists_batch(
        &self,
        connection: &Connection,
        batch_id: Uuid,
        actions: Vec<crate::models::PlannedAction>,
        current_delay_ms: &mut u64,
        consecutive_failures: &mut u32,
    ) -> Result<Vec<ActionExecutionResult>> {
        const SPOTIFY_BATCH_SIZE: usize = 50;
        let mut results = Vec::new();

        for chunk in actions.chunks(SPOTIFY_BATCH_SIZE) {
            self.apply_rate_limit_delay(*current_delay_ms).await;

            let start_time = Instant::now();
            let artist_ids: Vec<String> = chunk.iter().map(|a| a.entity_id.clone()).collect();

            let before_states: Vec<(Uuid, serde_json::Value)> = chunk
                .iter()
                .map(|a| {
                    (a.id, json!({
                        "artist_id": a.entity_id,
                        "artist_name": a.entity_name,
                        "was_following": true,
                        "recorded_at": Utc::now().to_rfc3339()
                    }))
                })
                .collect();

            let api_result = self.spotify_service
                .unfollow_artists_batch(connection, &artist_ids)
                .await;

            let execution_time_ms = start_time.elapsed().as_millis() as u64;

            match api_result {
                Ok(()) => {
                    *current_delay_ms = self.backoff_config.initial_delay_ms;
                    *consecutive_failures = 0;

                    for (action, (action_id, before_state)) in chunk.iter().zip(before_states) {
                        let after_state = json!({
                            "artist_id": action.entity_id,
                            "unfollowed": true,
                            "unfollowed_at": Utc::now().to_rfc3339()
                        });

                        self.store_action_result(
                            batch_id,
                            action_id,
                            &action.entity_type.to_string(),
                            &action.entity_id,
                            "unfollow_artist",
                            Some(&before_state),
                            Some(&after_state),
                            ActionItemStatus::Completed,
                            None,
                        ).await?;

                        results.push(ActionExecutionResult {
                            action_id,
                            entity_type: action.entity_type.to_string(),
                            entity_id: action.entity_id.clone(),
                            action_type: "unfollow_artist".to_string(),
                            success: true,
                            before_state: Some(before_state),
                            after_state: Some(after_state),
                            error_message: None,
                            error_code: None,
                            is_recoverable: true,
                            execution_time_ms,
                        });
                    }
                }
                Err(e) => {
                    let (error_code, is_rate_limit) = self.classify_error(&e);

                    if is_rate_limit {
                        *current_delay_ms = self.calculate_backoff_delay(*current_delay_ms);
                        *consecutive_failures += 1;
                    }

                    let is_recoverable = is_rate_limit || *consecutive_failures < self.backoff_config.max_retries;

                    for (action, (action_id, before_state)) in chunk.iter().zip(before_states) {
                        self.store_action_result(
                            batch_id,
                            action_id,
                            &action.entity_type.to_string(),
                            &action.entity_id,
                            "unfollow_artist",
                            Some(&before_state),
                            None,
                            ActionItemStatus::Failed,
                            Some(&e.to_string()),
                        ).await?;

                        results.push(ActionExecutionResult {
                            action_id,
                            entity_type: action.entity_type.to_string(),
                            entity_id: action.entity_id.clone(),
                            action_type: "unfollow_artist".to_string(),
                            success: false,
                            before_state: Some(before_state),
                            after_state: None,
                            error_message: Some(e.to_string()),
                            error_code: Some(error_code.clone()),
                            is_recoverable,
                            execution_time_ms,
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    /// Execute batch removal of playlist tracks via DELETE /v1/playlists/{id}/tracks
    async fn execute_remove_playlist_tracks_batch(
        &self,
        connection: &Connection,
        batch_id: Uuid,
        actions: Vec<crate::models::PlannedAction>,
        current_delay_ms: &mut u64,
        consecutive_failures: &mut u32,
    ) -> Result<Vec<ActionExecutionResult>> {
        // Group tracks by playlist for efficient API calls
        let mut playlist_tracks: HashMap<String, Vec<crate::models::PlannedAction>> = HashMap::new();

        for action in actions {
            let playlist_id = action.metadata
                .get("playlist_id")
                .and_then(|p| p.as_str())
                .unwrap_or("unknown")
                .to_string();
            playlist_tracks.entry(playlist_id).or_default().push(action);
        }

        let mut results = Vec::new();

        for (playlist_id, playlist_actions) in playlist_tracks {
            // Spotify allows up to 100 tracks per playlist modification
            const SPOTIFY_PLAYLIST_BATCH_SIZE: usize = 100;

            for chunk in playlist_actions.chunks(SPOTIFY_PLAYLIST_BATCH_SIZE) {
                self.apply_rate_limit_delay(*current_delay_ms).await;

                let start_time = Instant::now();

                let tracks_to_remove: Vec<serde_json::Value> = chunk
                    .iter()
                    .map(|action| json!({
                        "uri": format!("spotify:track:{}", action.entity_id)
                    }))
                    .collect();

                let before_states: Vec<(Uuid, serde_json::Value)> = chunk
                    .iter()
                    .map(|a| {
                        (a.id, json!({
                            "track_id": a.entity_id,
                            "playlist_id": playlist_id,
                            "was_in_playlist": true,
                            "recorded_at": Utc::now().to_rfc3339()
                        }))
                    })
                    .collect();

                let api_result = self.spotify_service
                    .remove_playlist_tracks_batch(connection, &playlist_id, &tracks_to_remove)
                    .await;

                let execution_time_ms = start_time.elapsed().as_millis() as u64;

                match api_result {
                    Ok(snapshot_id) => {
                        *current_delay_ms = self.backoff_config.initial_delay_ms;
                        *consecutive_failures = 0;

                        for (action, (action_id, before_state)) in chunk.iter().zip(before_states) {
                            let after_state = json!({
                                "track_id": action.entity_id,
                                "playlist_id": playlist_id,
                                "removed": true,
                                "removed_at": Utc::now().to_rfc3339(),
                                "new_snapshot_id": snapshot_id
                            });

                            self.store_action_result(
                                batch_id,
                                action_id,
                                &action.entity_type.to_string(),
                                &action.entity_id,
                                "remove_playlist_track",
                                Some(&before_state),
                                Some(&after_state),
                                ActionItemStatus::Completed,
                                None,
                            ).await?;

                            results.push(ActionExecutionResult {
                                action_id,
                                entity_type: action.entity_type.to_string(),
                                entity_id: action.entity_id.clone(),
                                action_type: "remove_playlist_track".to_string(),
                                success: true,
                                before_state: Some(before_state),
                                after_state: Some(after_state),
                                error_message: None,
                                error_code: None,
                                is_recoverable: true,
                                execution_time_ms,
                            });
                        }
                    }
                    Err(e) => {
                        let (error_code, is_rate_limit) = self.classify_error(&e);

                        if is_rate_limit {
                            *current_delay_ms = self.calculate_backoff_delay(*current_delay_ms);
                            *consecutive_failures += 1;
                        }

                        let is_recoverable = is_rate_limit || *consecutive_failures < self.backoff_config.max_retries;

                        for (action, (action_id, before_state)) in chunk.iter().zip(before_states) {
                            self.store_action_result(
                                batch_id,
                                action_id,
                                &action.entity_type.to_string(),
                                &action.entity_id,
                                "remove_playlist_track",
                                Some(&before_state),
                                None,
                                ActionItemStatus::Failed,
                                Some(&e.to_string()),
                            ).await?;

                            results.push(ActionExecutionResult {
                                action_id,
                                entity_type: action.entity_type.to_string(),
                                entity_id: action.entity_id.clone(),
                                action_type: "remove_playlist_track".to_string(),
                                success: false,
                                before_state: Some(before_state),
                                after_state: None,
                                error_message: Some(e.to_string()),
                                error_code: Some(error_code.clone()),
                                is_recoverable,
                                execution_time_ms,
                            });
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Execute batch removal of saved albums via DELETE /v1/me/albums
    async fn execute_remove_saved_albums_batch(
        &self,
        connection: &Connection,
        batch_id: Uuid,
        actions: Vec<crate::models::PlannedAction>,
        current_delay_ms: &mut u64,
        consecutive_failures: &mut u32,
    ) -> Result<Vec<ActionExecutionResult>> {
        const SPOTIFY_BATCH_SIZE: usize = 50;
        let mut results = Vec::new();

        for chunk in actions.chunks(SPOTIFY_BATCH_SIZE) {
            self.apply_rate_limit_delay(*current_delay_ms).await;

            let start_time = Instant::now();
            let album_ids: Vec<String> = chunk.iter().map(|a| a.entity_id.clone()).collect();

            let before_states: Vec<(Uuid, serde_json::Value)> = chunk
                .iter()
                .map(|a| {
                    (a.id, json!({
                        "album_id": a.entity_id,
                        "album_name": a.entity_name,
                        "was_saved": true,
                        "recorded_at": Utc::now().to_rfc3339()
                    }))
                })
                .collect();

            let api_result = self.spotify_service
                .remove_saved_albums_batch(connection, &album_ids)
                .await;

            let execution_time_ms = start_time.elapsed().as_millis() as u64;

            match api_result {
                Ok(()) => {
                    *current_delay_ms = self.backoff_config.initial_delay_ms;
                    *consecutive_failures = 0;

                    for (action, (action_id, before_state)) in chunk.iter().zip(before_states) {
                        let after_state = json!({
                            "album_id": action.entity_id,
                            "removed": true,
                            "removed_at": Utc::now().to_rfc3339()
                        });

                        self.store_action_result(
                            batch_id,
                            action_id,
                            &action.entity_type.to_string(),
                            &action.entity_id,
                            "remove_saved_album",
                            Some(&before_state),
                            Some(&after_state),
                            ActionItemStatus::Completed,
                            None,
                        ).await?;

                        results.push(ActionExecutionResult {
                            action_id,
                            entity_type: action.entity_type.to_string(),
                            entity_id: action.entity_id.clone(),
                            action_type: "remove_saved_album".to_string(),
                            success: true,
                            before_state: Some(before_state),
                            after_state: Some(after_state),
                            error_message: None,
                            error_code: None,
                            is_recoverable: true,
                            execution_time_ms,
                        });
                    }
                }
                Err(e) => {
                    let (error_code, is_rate_limit) = self.classify_error(&e);

                    if is_rate_limit {
                        *current_delay_ms = self.calculate_backoff_delay(*current_delay_ms);
                        *consecutive_failures += 1;
                    }

                    let is_recoverable = is_rate_limit || *consecutive_failures < self.backoff_config.max_retries;

                    for (action, (action_id, before_state)) in chunk.iter().zip(before_states) {
                        self.store_action_result(
                            batch_id,
                            action_id,
                            &action.entity_type.to_string(),
                            &action.entity_id,
                            "remove_saved_album",
                            Some(&before_state),
                            None,
                            ActionItemStatus::Failed,
                            Some(&e.to_string()),
                        ).await?;

                        results.push(ActionExecutionResult {
                            action_id,
                            entity_type: action.entity_type.to_string(),
                            entity_id: action.entity_id.clone(),
                            action_type: "remove_saved_album".to_string(),
                            success: false,
                            before_state: Some(before_state),
                            after_state: None,
                            error_message: Some(e.to_string()),
                            error_code: Some(error_code.clone()),
                            is_recoverable,
                            execution_time_ms,
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    /// Group actions by type for batch API optimization
    fn group_actions_for_batch_api(
        &self,
        actions: &[crate::models::PlannedAction],
    ) -> HashMap<String, Vec<crate::models::PlannedAction>> {
        let mut groups: HashMap<String, Vec<crate::models::PlannedAction>> = HashMap::new();

        for action in actions {
            groups.entry(action.action_type.to_string())
                .or_default()
                .push(action.clone());
        }

        groups
    }

    /// Apply rate limit delay before API call
    async fn apply_rate_limit_delay(&self, delay_ms: u64) {
        if delay_ms > 0 {
            tracing::debug!("Applying rate limit delay of {}ms", delay_ms);
            sleep(Duration::from_millis(delay_ms)).await;
        }
    }

    /// Calculate exponential backoff delay
    fn calculate_backoff_delay(&self, current_delay_ms: u64) -> u64 {
        let new_delay = (current_delay_ms as f64 * self.backoff_config.multiplier) as u64;
        new_delay.min(self.backoff_config.max_delay_ms)
    }

    /// Classify an error to determine if it's rate limiting related
    fn classify_error(&self, error: &anyhow::Error) -> (String, bool) {
        let error_str = error.to_string().to_lowercase();

        if error_str.contains("429") || error_str.contains("rate limit") || error_str.contains("too many requests") {
            ("RATE_LIMITED".to_string(), true)
        } else if error_str.contains("401") || error_str.contains("unauthorized") {
            ("UNAUTHORIZED".to_string(), false)
        } else if error_str.contains("403") || error_str.contains("forbidden") {
            ("FORBIDDEN".to_string(), false)
        } else if error_str.contains("404") || error_str.contains("not found") {
            ("NOT_FOUND".to_string(), false)
        } else if error_str.contains("500") || error_str.contains("internal server error") {
            ("SERVER_ERROR".to_string(), true)
        } else if error_str.contains("timeout") || error_str.contains("timed out") {
            ("TIMEOUT".to_string(), true)
        } else {
            ("UNKNOWN_ERROR".to_string(), true)
        }
    }

    /// Store action result in the action_items table
    async fn store_action_result(
        &self,
        batch_id: Uuid,
        action_id: Uuid,
        entity_type: &str,
        entity_id: &str,
        action: &str,
        before_state: Option<&serde_json::Value>,
        after_state: Option<&serde_json::Value>,
        status: ActionItemStatus,
        error_message: Option<&str>,
    ) -> Result<()> {
        let idempotency_key = format!("{}_{}_{}", batch_id, entity_id, action);

        sqlx::query(
            r#"
            INSERT INTO action_items (
                id, batch_id, entity_type, entity_id, action,
                idempotency_key, before_state, after_state, status,
                error_message, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (idempotency_key)
            DO UPDATE SET
                after_state = EXCLUDED.after_state,
                status = EXCLUDED.status,
                error_message = EXCLUDED.error_message
            "#
        )
        .bind(action_id)
        .bind(batch_id)
        .bind(entity_type)
        .bind(entity_id)
        .bind(action)
        .bind(&idempotency_key)
        .bind(before_state)
        .bind(after_state)
        .bind(status.to_string())
        .bind(error_message)
        .bind(Utc::now())
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }


    async fn update_progress(
        &self,
        job_id: Uuid,
        current_step: &str,
        completed_steps: u32,
        total_steps: u32,
        percentage: f64,
        details: serde_json::Value,
    ) -> Result<()> {
        // This would typically update the job progress in the job queue service
        // For now, we'll just log the progress
        tracing::info!(
            "Job {} progress: {} ({}/{} - {:.1}%)",
            job_id,
            current_step,
            completed_steps,
            total_steps,
            percentage
        );
        Ok(())
    }

    async fn get_enforcement_plan(&self, plan_id: &Uuid) -> Result<EnforcementPlan> {
        // This is a mock implementation
        // In reality, this would fetch the plan from a database or cache
        Ok(EnforcementPlan {
            id: *plan_id,
            user_id: Uuid::new_v4(),
            provider: "spotify".to_string(),
            dnp_artists: vec![Uuid::new_v4()],
            actions: vec![
                crate::models::PlannedAction {
                    id: uuid::Uuid::new_v4(),
                    entity_type: crate::models::spotify::EntityType::Track,
                    entity_id: "track_123".to_string(),
                    entity_name: "Test Track".to_string(),
                    action_type: crate::models::spotify::ActionType::RemoveLikedSong,
                    reason: crate::models::spotify::BlockReason::DirectBlock,
                    confidence: 1.0,
                    estimated_duration_ms: 1000,
                    dependencies: vec![],
                    metadata: serde_json::json!({
                        "track_name": "Test Track",
                        "artist_name": "Test Artist"
                    }),
                },
            ],
            options: crate::models::EnforcementOptions {
                dry_run: false,
                aggressiveness: crate::models::AggressivenessLevel::Moderate,
                block_collaborations: true,
                block_featuring: true,
                block_songwriter_only: false,
                preserve_user_playlists: true,
            },
            impact: crate::models::EnforcementImpact {
                liked_songs: crate::models::LibraryImpact {
                    total_tracks: 100,
                    tracks_to_remove: 1,
                    collaborations_found: 0,
                    featuring_found: 0,
                    exact_matches: 1,
                },
                playlists: crate::models::PlaylistImpact {
                    total_playlists: 0,
                    playlists_to_modify: 0,
                    total_tracks: 0,
                    tracks_to_remove: 0,
                    user_playlists_affected: 0,
                    collaborative_playlists_affected: 0,
                    playlist_details: vec![],
                },
                followed_artists: crate::models::FollowingImpact {
                    total_followed: 50,
                    artists_to_unfollow: 1,
                    exact_matches: 1,
                },
                saved_albums: crate::models::AlbumImpact {
                    total_albums: 20,
                    albums_to_remove: 0,
                    exact_matches: 0,
                    collaboration_albums: 0,
                },
                total_items_affected: 1,
                estimated_time_saved_hours: 0.1,
            },
            estimated_duration_seconds: 30,
            created_at: chrono::Utc::now(),
            idempotency_key: "test_key".to_string(),
        })
    }
}

#[async_trait]
impl JobHandler for EnforcementJobHandler {
    async fn handle(&self, job: &Job) -> Result<serde_json::Value> {
        self.execute_enforcement_job(job).await
    }

    fn job_type(&self) -> JobType {
        JobType::EnforcementExecution
    }

    fn max_execution_time(&self) -> Duration {
        Duration::from_secs(600) // 10 minutes max for enforcement jobs
    }
}

/// Job handler for batch rollback operations
pub struct RollbackJobHandler {
    spotify_service: Arc<SpotifyService>,
    rate_limiter: Arc<RateLimitingService>,
    db_pool: sqlx::PgPool,
    backoff_config: BackoffConfig,
}

impl RollbackJobHandler {
    pub fn new(
        spotify_service: Arc<SpotifyService>,
        rate_limiter: Arc<RateLimitingService>,
        db_pool: sqlx::PgPool,
    ) -> Self {
        Self {
            spotify_service,
            rate_limiter,
            db_pool,
            backoff_config: BackoffConfig::default(),
        }
    }

    async fn execute_rollback_job(&self, job: &Job) -> Result<serde_json::Value> {
        let payload = &job.payload;
        let user_id = job.user_id.ok_or_else(|| anyhow!("User ID required for rollback job"))?;

        let original_batch_id: Uuid = serde_json::from_value(
            payload.get("original_batch_id").cloned().ok_or_else(|| anyhow!("original_batch_id required"))?
        )?;
        let rollback_batch_id: Uuid = serde_json::from_value(
            payload.get("rollback_batch_id").cloned().ok_or_else(|| anyhow!("rollback_batch_id required"))?
        )?;
        let reason: String = payload.get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("User requested rollback")
            .to_string();

        tracing::info!(
            "Starting rollback job {} for user {} with batch {} -> rollback batch {}",
            job.id,
            user_id,
            original_batch_id,
            rollback_batch_id
        );

        // Get user's Spotify connection
        let connection = self.spotify_service.get_user_connection(user_id).await?
            .ok_or_else(|| anyhow!("No Spotify connection found for user"))?;

        // Get all rollback action items for this rollback batch
        let rollback_actions: Vec<RollbackActionItem> = sqlx::query_as(
            r#"
            SELECT id, batch_id, entity_type, entity_id, action, before_state
            FROM action_items
            WHERE batch_id = $1 AND status = 'pending'
            ORDER BY created_at ASC
            "#,
        )
        .bind(rollback_batch_id)
        .fetch_all(&self.db_pool)
        .await?;

        if rollback_actions.is_empty() {
            return Ok(json!({
                "rollback_batch_id": rollback_batch_id,
                "original_batch_id": original_batch_id,
                "status": "skipped",
                "actions_rolled_back": 0,
                "actions_failed": 0,
                "message": "No pending rollback actions found"
            }));
        }

        let total_actions = rollback_actions.len() as u32;
        let mut actions_rolled_back: u32 = 0;
        let mut actions_failed: u32 = 0;
        let mut current_delay_ms = self.backoff_config.initial_delay_ms;

        // Group actions by type for batch optimization
        let mut add_tracks: Vec<RollbackActionItem> = Vec::new();
        let mut follow_artists: Vec<RollbackActionItem> = Vec::new();
        let mut add_playlist_tracks: Vec<RollbackActionItem> = Vec::new();
        let mut add_albums: Vec<RollbackActionItem> = Vec::new();

        for action in rollback_actions {
            match action.action.as_str() {
                "add_liked_song" => add_tracks.push(action),
                "follow_artist" => follow_artists.push(action),
                "add_playlist_track" => add_playlist_tracks.push(action),
                "add_saved_album" => add_albums.push(action),
                _ => {
                    tracing::warn!("Unknown rollback action type: {}", action.action);
                    actions_failed += 1;
                }
            }
        }

        // Execute rollback batches

        // 1. Re-add liked songs via PUT /v1/me/tracks
        let (added, failed) = self.execute_add_tracks_rollback(
            &connection,
            rollback_batch_id,
            add_tracks,
            &mut current_delay_ms,
        ).await?;
        actions_rolled_back += added;
        actions_failed += failed;

        // 2. Re-follow artists via PUT /v1/me/following
        let (added, failed) = self.execute_follow_artists_rollback(
            &connection,
            rollback_batch_id,
            follow_artists,
            &mut current_delay_ms,
        ).await?;
        actions_rolled_back += added;
        actions_failed += failed;

        // 3. Re-add playlist tracks (best effort) via POST /v1/playlists/{id}/tracks
        let (added, failed) = self.execute_add_playlist_tracks_rollback(
            &connection,
            rollback_batch_id,
            add_playlist_tracks,
            &mut current_delay_ms,
        ).await?;
        actions_rolled_back += added;
        actions_failed += failed;

        // 4. Re-add saved albums via PUT /v1/me/albums
        let (added, failed) = self.execute_add_albums_rollback(
            &connection,
            rollback_batch_id,
            add_albums,
            &mut current_delay_ms,
        ).await?;
        actions_rolled_back += added;
        actions_failed += failed;

        // Update rollback batch with final summary
        let batch_status = if actions_failed == 0 {
            "completed"
        } else if actions_rolled_back > 0 {
            "partially_completed"
        } else {
            "failed"
        };

        let summary = json!({
            "total_actions": total_actions,
            "completed_actions": actions_rolled_back,
            "failed_actions": actions_failed,
            "reason": reason
        });

        sqlx::query(
            r#"
            UPDATE action_batches
            SET status = $1, summary = $2, completed_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(batch_status)
        .bind(&summary)
        .bind(rollback_batch_id)
        .execute(&self.db_pool)
        .await?;

        let message = if actions_failed == 0 {
            format!(
                "Successfully rolled back {} actions from batch {}",
                actions_rolled_back, original_batch_id
            )
        } else {
            format!(
                "Partially rolled back batch {}. {} succeeded, {} failed",
                original_batch_id, actions_rolled_back, actions_failed
            )
        };

        tracing::info!(
            "Rollback job {} completed: {} actions rolled back, {} failed",
            job.id,
            actions_rolled_back,
            actions_failed
        );

        Ok(json!({
            "rollback_batch_id": rollback_batch_id,
            "original_batch_id": original_batch_id,
            "status": batch_status,
            "actions_rolled_back": actions_rolled_back,
            "actions_failed": actions_failed,
            "message": message
        }))
    }

    /// Re-add liked songs via PUT /v1/me/tracks
    async fn execute_add_tracks_rollback(
        &self,
        connection: &Connection,
        rollback_batch_id: Uuid,
        actions: Vec<RollbackActionItem>,
        current_delay_ms: &mut u64,
    ) -> Result<(u32, u32)> {
        const SPOTIFY_BATCH_SIZE: usize = 50;
        let mut success_count = 0u32;
        let mut fail_count = 0u32;

        for chunk in actions.chunks(SPOTIFY_BATCH_SIZE) {
            if *current_delay_ms > 0 {
                sleep(Duration::from_millis(*current_delay_ms)).await;
            }

            let track_ids: Vec<String> = chunk.iter().map(|a| a.entity_id.clone()).collect();

            match self.spotify_service.add_liked_songs_batch(connection, &track_ids).await {
                Ok(()) => {
                    *current_delay_ms = self.backoff_config.initial_delay_ms;

                    // Mark actions as completed
                    for action in chunk {
                        let after_state = json!({
                            "track_id": action.entity_id,
                            "re_added": true,
                            "re_added_at": Utc::now().to_rfc3339()
                        });

                        let _ = sqlx::query(
                            r#"
                            UPDATE action_items
                            SET status = 'completed', after_state = $1
                            WHERE id = $2
                            "#,
                        )
                        .bind(&after_state)
                        .bind(action.id)
                        .execute(&self.db_pool)
                        .await;

                        success_count += 1;
                    }
                }
                Err(e) => {
                    let is_rate_limit = e.to_string().to_lowercase().contains("rate limit")
                        || e.to_string().contains("429");

                    if is_rate_limit {
                        *current_delay_ms = (*current_delay_ms as f64 * self.backoff_config.multiplier) as u64;
                        *current_delay_ms = (*current_delay_ms).min(self.backoff_config.max_delay_ms);
                    }

                    for action in chunk {
                        let _ = sqlx::query(
                            r#"
                            UPDATE action_items
                            SET status = 'failed', error_message = $1
                            WHERE id = $2
                            "#,
                        )
                        .bind(e.to_string())
                        .bind(action.id)
                        .execute(&self.db_pool)
                        .await;

                        fail_count += 1;
                    }
                }
            }
        }

        Ok((success_count, fail_count))
    }

    /// Re-follow artists via PUT /v1/me/following
    async fn execute_follow_artists_rollback(
        &self,
        connection: &Connection,
        rollback_batch_id: Uuid,
        actions: Vec<RollbackActionItem>,
        current_delay_ms: &mut u64,
    ) -> Result<(u32, u32)> {
        const SPOTIFY_BATCH_SIZE: usize = 50;
        let mut success_count = 0u32;
        let mut fail_count = 0u32;

        for chunk in actions.chunks(SPOTIFY_BATCH_SIZE) {
            if *current_delay_ms > 0 {
                sleep(Duration::from_millis(*current_delay_ms)).await;
            }

            let artist_ids: Vec<String> = chunk.iter().map(|a| a.entity_id.clone()).collect();

            match self.spotify_service.follow_artists_batch(connection, &artist_ids).await {
                Ok(()) => {
                    *current_delay_ms = self.backoff_config.initial_delay_ms;

                    for action in chunk {
                        let after_state = json!({
                            "artist_id": action.entity_id,
                            "re_followed": true,
                            "re_followed_at": Utc::now().to_rfc3339()
                        });

                        let _ = sqlx::query(
                            r#"
                            UPDATE action_items
                            SET status = 'completed', after_state = $1
                            WHERE id = $2
                            "#,
                        )
                        .bind(&after_state)
                        .bind(action.id)
                        .execute(&self.db_pool)
                        .await;

                        success_count += 1;
                    }
                }
                Err(e) => {
                    let is_rate_limit = e.to_string().to_lowercase().contains("rate limit")
                        || e.to_string().contains("429");

                    if is_rate_limit {
                        *current_delay_ms = (*current_delay_ms as f64 * self.backoff_config.multiplier) as u64;
                        *current_delay_ms = (*current_delay_ms).min(self.backoff_config.max_delay_ms);
                    }

                    for action in chunk {
                        let _ = sqlx::query(
                            r#"
                            UPDATE action_items
                            SET status = 'failed', error_message = $1
                            WHERE id = $2
                            "#,
                        )
                        .bind(e.to_string())
                        .bind(action.id)
                        .execute(&self.db_pool)
                        .await;

                        fail_count += 1;
                    }
                }
            }
        }

        Ok((success_count, fail_count))
    }

    /// Re-add playlist tracks (best effort) via POST /v1/playlists/{id}/tracks
    async fn execute_add_playlist_tracks_rollback(
        &self,
        connection: &Connection,
        rollback_batch_id: Uuid,
        actions: Vec<RollbackActionItem>,
        current_delay_ms: &mut u64,
    ) -> Result<(u32, u32)> {
        use std::collections::HashMap;

        // Group tracks by playlist
        let mut playlist_tracks: HashMap<String, Vec<RollbackActionItem>> = HashMap::new();
        for action in actions {
            let playlist_id = action.before_state
                .as_ref()
                .and_then(|s| s.get("playlist_id"))
                .and_then(|p| p.as_str())
                .unwrap_or("unknown")
                .to_string();
            playlist_tracks.entry(playlist_id).or_default().push(action);
        }

        let mut success_count = 0u32;
        let mut fail_count = 0u32;

        for (playlist_id, playlist_actions) in playlist_tracks {
            if playlist_id == "unknown" {
                // Can't add tracks without playlist ID
                for action in playlist_actions {
                    let _ = sqlx::query(
                        r#"
                        UPDATE action_items
                        SET status = 'failed', error_message = 'Missing playlist_id in before_state'
                        WHERE id = $1
                        "#,
                    )
                    .bind(action.id)
                    .execute(&self.db_pool)
                    .await;

                    fail_count += 1;
                }
                continue;
            }

            const SPOTIFY_PLAYLIST_BATCH_SIZE: usize = 100;

            for chunk in playlist_actions.chunks(SPOTIFY_PLAYLIST_BATCH_SIZE) {
                if *current_delay_ms > 0 {
                    sleep(Duration::from_millis(*current_delay_ms)).await;
                }

                let track_uris: Vec<String> = chunk.iter()
                    .map(|a| format!("spotify:track:{}", a.entity_id))
                    .collect();

                match self.spotify_service.add_playlist_tracks_batch(
                    connection,
                    &playlist_id,
                    &track_uris,
                    None, // Add at end of playlist
                ).await {
                    Ok(snapshot_id) => {
                        *current_delay_ms = self.backoff_config.initial_delay_ms;

                        for action in chunk {
                            let after_state = json!({
                                "track_id": action.entity_id,
                                "playlist_id": playlist_id,
                                "re_added": true,
                                "re_added_at": Utc::now().to_rfc3339(),
                                "new_snapshot_id": snapshot_id
                            });

                            let _ = sqlx::query(
                                r#"
                                UPDATE action_items
                                SET status = 'completed', after_state = $1
                                WHERE id = $2
                                "#,
                            )
                            .bind(&after_state)
                            .bind(action.id)
                            .execute(&self.db_pool)
                            .await;

                            success_count += 1;
                        }
                    }
                    Err(e) => {
                        let is_rate_limit = e.to_string().to_lowercase().contains("rate limit")
                            || e.to_string().contains("429");

                        if is_rate_limit {
                            *current_delay_ms = (*current_delay_ms as f64 * self.backoff_config.multiplier) as u64;
                            *current_delay_ms = (*current_delay_ms).min(self.backoff_config.max_delay_ms);
                        }

                        for action in chunk {
                            let _ = sqlx::query(
                                r#"
                                UPDATE action_items
                                SET status = 'failed', error_message = $1
                                WHERE id = $2
                                "#,
                            )
                            .bind(e.to_string())
                            .bind(action.id)
                            .execute(&self.db_pool)
                            .await;

                            fail_count += 1;
                        }
                    }
                }
            }
        }

        Ok((success_count, fail_count))
    }

    /// Re-add saved albums via PUT /v1/me/albums
    async fn execute_add_albums_rollback(
        &self,
        connection: &Connection,
        rollback_batch_id: Uuid,
        actions: Vec<RollbackActionItem>,
        current_delay_ms: &mut u64,
    ) -> Result<(u32, u32)> {
        const SPOTIFY_BATCH_SIZE: usize = 50;
        let mut success_count = 0u32;
        let mut fail_count = 0u32;

        for chunk in actions.chunks(SPOTIFY_BATCH_SIZE) {
            if *current_delay_ms > 0 {
                sleep(Duration::from_millis(*current_delay_ms)).await;
            }

            let album_ids: Vec<String> = chunk.iter().map(|a| a.entity_id.clone()).collect();

            match self.spotify_service.add_saved_albums_batch(connection, &album_ids).await {
                Ok(()) => {
                    *current_delay_ms = self.backoff_config.initial_delay_ms;

                    for action in chunk {
                        let after_state = json!({
                            "album_id": action.entity_id,
                            "re_added": true,
                            "re_added_at": Utc::now().to_rfc3339()
                        });

                        let _ = sqlx::query(
                            r#"
                            UPDATE action_items
                            SET status = 'completed', after_state = $1
                            WHERE id = $2
                            "#,
                        )
                        .bind(&after_state)
                        .bind(action.id)
                        .execute(&self.db_pool)
                        .await;

                        success_count += 1;
                    }
                }
                Err(e) => {
                    let is_rate_limit = e.to_string().to_lowercase().contains("rate limit")
                        || e.to_string().contains("429");

                    if is_rate_limit {
                        *current_delay_ms = (*current_delay_ms as f64 * self.backoff_config.multiplier) as u64;
                        *current_delay_ms = (*current_delay_ms).min(self.backoff_config.max_delay_ms);
                    }

                    for action in chunk {
                        let _ = sqlx::query(
                            r#"
                            UPDATE action_items
                            SET status = 'failed', error_message = $1
                            WHERE id = $2
                            "#,
                        )
                        .bind(e.to_string())
                        .bind(action.id)
                        .execute(&self.db_pool)
                        .await;

                        fail_count += 1;
                    }
                }
            }
        }

        Ok((success_count, fail_count))
    }
}

/// Rollback action item from database
#[derive(sqlx::FromRow)]
struct RollbackActionItem {
    id: Uuid,
    batch_id: Uuid,
    entity_type: String,
    entity_id: String,
    action: String,
    before_state: Option<serde_json::Value>,
}

#[async_trait]
impl JobHandler for RollbackJobHandler {
    async fn handle(&self, job: &Job) -> Result<serde_json::Value> {
        self.execute_rollback_job(job).await
    }

    fn job_type(&self) -> JobType {
        JobType::BatchRollback
    }

    fn max_execution_time(&self) -> Duration {
        Duration::from_secs(300) // 5 minutes max for rollback jobs
    }
}

/// Job handler for token refresh operations
pub struct TokenRefreshJobHandler {
    spotify_service: Arc<SpotifyService>,
}

impl TokenRefreshJobHandler {
    pub fn new(spotify_service: Arc<SpotifyService>) -> Self {
        Self { spotify_service }
    }

    async fn execute_token_refresh_job(&self, job: &Job) -> Result<serde_json::Value> {
        let user_id = job.user_id.ok_or_else(|| anyhow!("User ID required for token refresh job"))?;

        tracing::info!("Starting token refresh job {} for user {}", job.id, user_id);

        // Get user connection and refresh tokens
        if let Some(connection) = self.spotify_service.get_user_connection(user_id).await? {
            let mut connection_mut = connection;
            match self.spotify_service.refresh_token(&mut connection_mut).await {
                Ok(()) => {
                    tracing::info!("Successfully refreshed tokens for user {}", user_id);
                    Ok(serde_json::json!({
                        "status": "success",
                        "user_id": user_id,
                        "provider": connection_mut.provider,
                        "refreshed_at": chrono::Utc::now()
                    }))
                }
                Err(e) => {
                    tracing::error!("Failed to refresh tokens for user {}: {}", user_id, e);
                    Err(e)
                }
            }
        } else {
            Err(anyhow!("No connection found for user {}", user_id))
        }
    }
}

#[async_trait]
impl JobHandler for TokenRefreshJobHandler {
    async fn handle(&self, job: &Job) -> Result<serde_json::Value> {
        self.execute_token_refresh_job(job).await
    }

    fn job_type(&self) -> JobType {
        JobType::TokenRefresh
    }

    fn max_execution_time(&self) -> Duration {
        Duration::from_secs(60) // 1 minute max for token refresh
    }
}