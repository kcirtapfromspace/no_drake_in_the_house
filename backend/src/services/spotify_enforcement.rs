use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use uuid::Uuid;

use crate::models::{
    Connection, EnforcementPlan, PlannedAction, ActionType, ActionBatch, ActionItem,
    ActionBatchStatus, ActionItemStatus, BatchSummary, BatchError, BatchExecutionResult,
    ExecuteBatchRequest, RollbackBatchRequest, RollbackInfo, BatchProgress, RateLimitStatus,
};
use crate::services::{SpotifyService, SpotifyLibraryService};

/// Service for executing Spotify enforcement operations
pub struct SpotifyEnforcementService {
    spotify_service: Arc<SpotifyService>,
    library_service: Arc<SpotifyLibraryService>,
    db_pool: sqlx::PgPool,
}

impl SpotifyEnforcementService {
    pub fn new(
        spotify_service: Arc<SpotifyService>,
        library_service: Arc<SpotifyLibraryService>,
        db_pool: sqlx::PgPool,
    ) -> Self {
        Self {
            spotify_service,
            library_service,
            db_pool,
        }
    }

    /// Execute an enforcement plan as a batch operation
    pub async fn execute_enforcement_batch(
        &self,
        connection: &Connection,
        plan: &EnforcementPlan,
        request: ExecuteBatchRequest,
    ) -> Result<BatchExecutionResult> {
        // Create action batch record
        let idempotency_key = request.idempotency_key
            .unwrap_or_else(|| format!("{}_{}", plan.id, Utc::now().timestamp_millis()));

        // Check for existing batch with same idempotency key
        if let Some(existing_batch) = self.get_batch_by_idempotency_key(&idempotency_key).await? {
            return Ok(BatchExecutionResult {
                batch_id: existing_batch.id,
                status: existing_batch.status,
                summary: serde_json::from_value(existing_batch.summary).unwrap_or_default(),
                completed_actions: self.get_batch_actions(&existing_batch.id, Some(ActionItemStatus::Completed)).await?,
                failed_actions: self.get_batch_actions(&existing_batch.id, Some(ActionItemStatus::Failed)).await?,
                rollback_info: None,
            });
        }

        let mut batch = ActionBatch::new(
            connection.user_id,
            "spotify".to_string(),
            idempotency_key,
            plan.options.dry_run,
            serde_json::to_value(&plan.options)?,
        );

        // Save batch to database
        batch = self.create_batch(&batch).await?;

        // Create action items from planned actions
        let action_items = self.create_action_items(&batch.id, &plan.actions).await?;

        if plan.options.dry_run {
            // For dry run, just return the plan without executing
            let summary = BatchSummary {
                total_actions: action_items.len() as u32,
                completed_actions: 0,
                failed_actions: 0,
                skipped_actions: action_items.len() as u32,
                execution_time_ms: 0,
                api_calls_made: 0,
                rate_limit_delays_ms: 0,
                errors: Vec::new(),
            };

            batch.mark_completed(summary.clone());
            self.update_batch(&batch).await?;

            return Ok(BatchExecutionResult {
                batch_id: batch.id,
                status: batch.status,
                summary,
                completed_actions: Vec::new(),
                failed_actions: Vec::new(),
                rollback_info: None,
            });
        }

        // Execute the batch
        if request.execute_immediately {
            self.execute_batch_immediately(connection, &mut batch, action_items).await
        } else {
            // Queue for background processing
            self.queue_batch_for_execution(&batch).await?;
            Ok(BatchExecutionResult {
                batch_id: batch.id,
                status: ActionBatchStatus::Pending,
                summary: BatchSummary::default(),
                completed_actions: Vec::new(),
                failed_actions: Vec::new(),
                rollback_info: None,
            })
        }
    }

    /// Execute batch immediately with rate limiting and error handling
    async fn execute_batch_immediately(
        &self,
        connection: &Connection,
        batch: &mut ActionBatch,
        mut action_items: Vec<ActionItem>,
    ) -> Result<BatchExecutionResult> {
        let start_time = Instant::now();
        let mut summary = BatchSummary {
            total_actions: action_items.len() as u32,
            ..Default::default()
        };

        // Update batch status to in progress
        batch.status = ActionBatchStatus::InProgress;
        self.update_batch(batch).await?;

        let mut completed_actions = Vec::new();
        let mut failed_actions = Vec::new();

        // Group actions by type for optimal batching
        let grouped_actions = self.group_actions_for_batching(&action_items);

        for (action_type, actions) in grouped_actions {
            match action_type {
                ActionType::RemoveLikedSong => {
                    self.execute_liked_songs_batch(connection, actions, &mut summary, &mut completed_actions, &mut failed_actions).await?;
                }
                ActionType::RemovePlaylistTrack => {
                    self.execute_playlist_tracks_batch(connection, actions, &mut summary, &mut completed_actions, &mut failed_actions).await?;
                }
                ActionType::UnfollowArtist => {
                    self.execute_unfollow_artists_batch(connection, actions, &mut summary, &mut completed_actions, &mut failed_actions).await?;
                }
                ActionType::RemoveSavedAlbum => {
                    self.execute_remove_albums_batch(connection, actions, &mut summary, &mut completed_actions, &mut failed_actions).await?;
                }
                _ => {
                    // Handle other action types individually
                    for mut action in actions {
                        match self.execute_single_action(connection, &mut action).await {
                            Ok(_) => {
                                summary.completed_actions += 1;
                                completed_actions.push(action);
                            }
                            Err(e) => {
                                action.mark_failed(e.to_string());
                                summary.failed_actions += 1;
                                summary.errors.push(BatchError {
                                    action_id: action.id,
                                    entity_type: action.entity_type.clone(),
                                    entity_id: action.entity_id.clone(),
                                    error_code: "EXECUTION_FAILED".to_string(),
                                    error_message: e.to_string(),
                                    retry_count: 0,
                                    is_recoverable: true,
                                });
                                failed_actions.push(action);
                            }
                        }
                        
                        // Update action in database
                        self.update_action_item(&completed_actions.last().or(failed_actions.last()).unwrap()).await?;
                    }
                }
            }
        }

        summary.execution_time_ms = start_time.elapsed().as_millis() as u64;
        batch.mark_completed(summary.clone());
        self.update_batch(batch).await?;

        Ok(BatchExecutionResult {
            batch_id: batch.id,
            status: batch.status.clone(),
            summary,
            completed_actions,
            failed_actions,
            rollback_info: None,
        })
    }

    /// Execute batch of liked song removals with optimal API usage
    async fn execute_liked_songs_batch(
        &self,
        connection: &Connection,
        actions: Vec<ActionItem>,
        summary: &mut BatchSummary,
        completed_actions: &mut Vec<ActionItem>,
        failed_actions: &mut Vec<ActionItem>,
    ) -> Result<()> {
        // Spotify allows removing up to 50 tracks at once
        const BATCH_SIZE: usize = 50;
        
        for chunk in actions.chunks(BATCH_SIZE) {
            let track_ids: Vec<String> = chunk.iter().map(|a| a.entity_id.clone()).collect();
            
            // Wait for rate limit if needed
            self.wait_for_rate_limit().await?;
            
            match self.remove_liked_songs_batch(connection, &track_ids).await {
                Ok(_) => {
                    // Mark all actions in chunk as completed
                    for mut action in chunk.to_vec() {
                        action.mark_completed(json!({
                            "removed_at": Utc::now(),
                            "batch_size": track_ids.len()
                        }));
                        summary.completed_actions += 1;
                        summary.api_calls_made += 1;
                        completed_actions.push(action);
                    }
                }
                Err(e) => {
                    // Mark all actions in chunk as failed
                    for mut action in chunk.to_vec() {
                        action.mark_failed(e.to_string());
                        summary.failed_actions += 1;
                        summary.errors.push(BatchError {
                            action_id: action.id,
                            entity_type: action.entity_type.clone(),
                            entity_id: action.entity_id.clone(),
                            error_code: "BATCH_REMOVE_FAILED".to_string(),
                            error_message: e.to_string(),
                            retry_count: 0,
                            is_recoverable: true,
                        });
                        failed_actions.push(action);
                    }
                }
            }
            
            // Update all actions in database
            for action in chunk {
                self.update_action_item(action).await?;
            }
        }
        
        Ok(())
    }

    /// Execute batch of playlist track removals with delta optimization
    async fn execute_playlist_tracks_batch(
        &self,
        connection: &Connection,
        actions: Vec<ActionItem>,
        summary: &mut BatchSummary,
        completed_actions: &mut Vec<ActionItem>,
        failed_actions: &mut Vec<ActionItem>,
    ) -> Result<()> {
        // Group by playlist for delta removal
        let mut playlist_groups: HashMap<String, Vec<ActionItem>> = HashMap::new();
        
        for action in actions {
            if let Some(playlist_id) = action.before_state
                .as_ref()
                .and_then(|s| s.get("playlist_id"))
                .and_then(|p| p.as_str()) {
                playlist_groups.entry(playlist_id.to_string()).or_default().push(action);
            }
        }

        for (playlist_id, playlist_actions) in playlist_groups {
            // Wait for rate limit
            self.wait_for_rate_limit().await?;
            
            // Create tracks array for removal
            let tracks_to_remove: Vec<Value> = playlist_actions
                .iter()
                .map(|action| json!({
                    "uri": format!("spotify:track:{}", action.entity_id)
                }))
                .collect();

            match self.remove_playlist_tracks_batch(connection, &playlist_id, &tracks_to_remove).await {
                Ok(snapshot_id) => {
                    for mut action in playlist_actions {
                        action.mark_completed(json!({
                            "removed_at": Utc::now(),
                            "playlist_id": playlist_id,
                            "new_snapshot_id": snapshot_id
                        }));
                        summary.completed_actions += 1;
                        completed_actions.push(action);
                    }
                    summary.api_calls_made += 1;
                }
                Err(e) => {
                    for mut action in playlist_actions {
                        action.mark_failed(e.to_string());
                        summary.failed_actions += 1;
                        summary.errors.push(BatchError {
                            action_id: action.id,
                            entity_type: action.entity_type.clone(),
                            entity_id: action.entity_id.clone(),
                            error_code: "PLAYLIST_REMOVE_FAILED".to_string(),
                            error_message: e.to_string(),
                            retry_count: 0,
                            is_recoverable: true,
                        });
                        failed_actions.push(action);
                    }
                }
            }
            
            // Update actions in database
            for action in &completed_actions[completed_actions.len() - playlist_actions.len()..] {
                self.update_action_item(action).await?;
            }
            for action in &failed_actions[failed_actions.len() - playlist_actions.len()..] {
                self.update_action_item(action).await?;
            }
        }
        
        Ok(())
    }

    /// Execute batch of artist unfollows
    async fn execute_unfollow_artists_batch(
        &self,
        connection: &Connection,
        actions: Vec<ActionItem>,
        summary: &mut BatchSummary,
        completed_actions: &mut Vec<ActionItem>,
        failed_actions: &mut Vec<ActionItem>,
    ) -> Result<()> {
        // Spotify allows unfollowing up to 50 artists at once
        const BATCH_SIZE: usize = 50;
        
        for chunk in actions.chunks(BATCH_SIZE) {
            let artist_ids: Vec<String> = chunk.iter().map(|a| a.entity_id.clone()).collect();
            
            self.wait_for_rate_limit().await?;
            
            match self.unfollow_artists_batch(connection, &artist_ids).await {
                Ok(_) => {
                    for mut action in chunk.to_vec() {
                        action.mark_completed(json!({
                            "unfollowed_at": Utc::now(),
                            "batch_size": artist_ids.len()
                        }));
                        summary.completed_actions += 1;
                        completed_actions.push(action);
                    }
                    summary.api_calls_made += 1;
                }
                Err(e) => {
                    for mut action in chunk.to_vec() {
                        action.mark_failed(e.to_string());
                        summary.failed_actions += 1;
                        summary.errors.push(BatchError {
                            action_id: action.id,
                            entity_type: action.entity_type.clone(),
                            entity_id: action.entity_id.clone(),
                            error_code: "UNFOLLOW_FAILED".to_string(),
                            error_message: e.to_string(),
                            retry_count: 0,
                            is_recoverable: true,
                        });
                        failed_actions.push(action);
                    }
                }
            }
            
            for action in chunk {
                self.update_action_item(action).await?;
            }
        }
        
        Ok(())
    }

    /// Execute batch of album removals
    async fn execute_remove_albums_batch(
        &self,
        connection: &Connection,
        actions: Vec<ActionItem>,
        summary: &mut BatchSummary,
        completed_actions: &mut Vec<ActionItem>,
        failed_actions: &mut Vec<ActionItem>,
    ) -> Result<()> {
        const BATCH_SIZE: usize = 50;
        
        for chunk in actions.chunks(BATCH_SIZE) {
            let album_ids: Vec<String> = chunk.iter().map(|a| a.entity_id.clone()).collect();
            
            self.wait_for_rate_limit().await?;
            
            match self.remove_saved_albums_batch(connection, &album_ids).await {
                Ok(_) => {
                    for mut action in chunk.to_vec() {
                        action.mark_completed(json!({
                            "removed_at": Utc::now(),
                            "batch_size": album_ids.len()
                        }));
                        summary.completed_actions += 1;
                        completed_actions.push(action);
                    }
                    summary.api_calls_made += 1;
                }
                Err(e) => {
                    for mut action in chunk.to_vec() {
                        action.mark_failed(e.to_string());
                        summary.failed_actions += 1;
                        summary.errors.push(BatchError {
                            action_id: action.id,
                            entity_type: action.entity_type.clone(),
                            entity_id: action.entity_id.clone(),
                            error_code: "ALBUM_REMOVE_FAILED".to_string(),
                            error_message: e.to_string(),
                            retry_count: 0,
                            is_recoverable: true,
                        });
                        failed_actions.push(action);
                    }
                }
            }
            
            for action in chunk {
                self.update_action_item(action).await?;
            }
        }
        
        Ok(())
    }

    /// Rollback a completed batch
    pub async fn rollback_batch(
        &self,
        connection: &Connection,
        request: RollbackBatchRequest,
    ) -> Result<RollbackInfo> {
        let batch = self.get_batch(&request.batch_id).await?
            .ok_or_else(|| anyhow!("Batch not found"))?;

        if !matches!(batch.status, ActionBatchStatus::Completed | ActionBatchStatus::PartiallyCompleted) {
            return Err(anyhow!("Can only rollback completed batches"));
        }

        // Get actions to rollback
        let actions_to_rollback = if let Some(action_ids) = request.action_ids {
            self.get_batch_actions_by_ids(&action_ids).await?
        } else {
            self.get_batch_actions(&batch.id, Some(ActionItemStatus::Completed)).await?
        };

        // Create rollback batch
        let rollback_idempotency_key = format!("rollback_{}_{}", batch.id, Utc::now().timestamp_millis());
        let mut rollback_batch = ActionBatch::new(
            batch.user_id,
            batch.provider.clone(),
            rollback_idempotency_key,
            false, // Rollbacks are never dry runs
            json!({
                "rollback_of": batch.id,
                "reason": request.reason
            }),
        );

        rollback_batch = self.create_batch(&rollback_batch).await?;

        // Create rollback actions
        let mut rollback_actions = Vec::new();
        let mut rollback_summary = BatchSummary {
            total_actions: actions_to_rollback.len() as u32,
            ..Default::default()
        };

        for original_action in actions_to_rollback {
            if !original_action.can_rollback() {
                rollback_summary.skipped_actions += 1;
                continue;
            }

            let rollback_action = self.create_rollback_action(&rollback_batch.id, &original_action)?;
            
            match self.execute_rollback_action(connection, rollback_action).await {
                Ok(mut completed_action) => {
                    rollback_summary.completed_actions += 1;
                    rollback_summary.api_calls_made += 1;
                    
                    // Mark original action as rolled back
                    let mut original = original_action;
                    original.status = ActionItemStatus::Rolled_back;
                    self.update_action_item(&original).await?;
                    
                    rollback_actions.push(completed_action);
                }
                Err(e) => {
                    rollback_summary.failed_actions += 1;
                    rollback_summary.errors.push(BatchError {
                        action_id: rollback_action.id,
                        entity_type: rollback_action.entity_type.clone(),
                        entity_id: rollback_action.entity_id.clone(),
                        error_code: "ROLLBACK_FAILED".to_string(),
                        error_message: e.to_string(),
                        retry_count: 0,
                        is_recoverable: false,
                    });
                }
            }
        }

        rollback_batch.mark_completed(rollback_summary.clone());
        self.update_batch(&rollback_batch).await?;

        Ok(RollbackInfo {
            rollback_batch_id: rollback_batch.id,
            rollback_actions,
            rollback_summary,
            partial_rollback: request.action_ids.is_some(),
            rollback_errors: rollback_summary.errors.clone(),
        })
    }

    /// Get batch execution progress
    pub async fn get_batch_progress(&self, batch_id: &Uuid) -> Result<BatchProgress> {
        let batch = self.get_batch(batch_id).await?
            .ok_or_else(|| anyhow!("Batch not found"))?;

        let completed_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM action_items WHERE batch_id = $1 AND status = 'completed'",
            batch_id
        )
        .fetch_one(&self.db_pool)
        .await? as u32;

        let failed_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM action_items WHERE batch_id = $1 AND status = 'failed'",
            batch_id
        )
        .fetch_one(&self.db_pool)
        .await? as u32;

        let total_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM action_items WHERE batch_id = $1",
            batch_id
        )
        .fetch_one(&self.db_pool)
        .await? as u32;

        // Get current action being processed
        let current_action = sqlx::query_scalar!(
            "SELECT entity_type || ':' || entity_id FROM action_items 
             WHERE batch_id = $1 AND status = 'in_progress' 
             ORDER BY created_at LIMIT 1",
            batch_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        // Estimate remaining time based on average action duration
        let remaining_actions = total_count - completed_count - failed_count;
        let estimated_remaining_ms = remaining_actions as u64 * 750; // 750ms average per action

        Ok(BatchProgress {
            batch_id: *batch_id,
            total_actions: total_count,
            completed_actions: completed_count,
            failed_actions: failed_count,
            current_action,
            estimated_remaining_ms,
            rate_limit_status: self.get_rate_limit_status().await?,
        })
    }

    // Private helper methods

    /// Group actions by type for optimal batching
    fn group_actions_for_batching(&self, actions: &[ActionItem]) -> HashMap<ActionType, Vec<ActionItem>> {
        let mut grouped = HashMap::new();
        
        for action in actions {
            let action_type = match action.action.as_str() {
                "remove_liked_song" => ActionType::RemoveLikedSong,
                "remove_playlist_track" => ActionType::RemovePlaylistTrack,
                "unfollow_artist" => ActionType::UnfollowArtist,
                "remove_saved_album" => ActionType::RemoveSavedAlbum,
                _ => continue,
            };
            
            grouped.entry(action_type).or_insert_with(Vec::new).push(action.clone());
        }
        
        grouped
    }

    /// Execute a single action (fallback for non-batchable actions)
    async fn execute_single_action(&self, connection: &Connection, action: &mut ActionItem) -> Result<()> {
        action.status = ActionItemStatus::InProgress;
        self.update_action_item(action).await?;

        let result = match action.action.as_str() {
            "remove_liked_song" => {
                self.remove_liked_songs_batch(connection, &[action.entity_id.clone()]).await?;
                json!({ "removed_at": Utc::now() })
            }
            "unfollow_artist" => {
                self.unfollow_artists_batch(connection, &[action.entity_id.clone()]).await?;
                json!({ "unfollowed_at": Utc::now() })
            }
            "remove_saved_album" => {
                self.remove_saved_albums_batch(connection, &[action.entity_id.clone()]).await?;
                json!({ "removed_at": Utc::now() })
            }
            _ => return Err(anyhow!("Unknown action type: {}", action.action)),
        };

        action.mark_completed(result);
        Ok(())
    }

    /// Wait for rate limit if needed
    async fn wait_for_rate_limit(&self) -> Result<()> {
        // Check current rate limit status
        let rate_limit = self.get_rate_limit_status().await?;
        
        if rate_limit.requests_remaining == 0 {
            let wait_time = rate_limit.reset_time.signed_duration_since(Utc::now());
            if wait_time.num_milliseconds() > 0 {
                sleep(Duration::from_millis(wait_time.num_milliseconds() as u64)).await;
            }
        }
        
        Ok(())
    }

    /// Get current rate limit status
    async fn get_rate_limit_status(&self) -> Result<RateLimitStatus> {
        // This would integrate with the rate limiting service
        // For now, return a mock status
        Ok(RateLimitStatus {
            requests_remaining: 100,
            reset_time: Utc::now() + chrono::Duration::seconds(3600),
            current_delay_ms: 0,
        })
    }

    // Database operations

    async fn create_batch(&self, batch: &ActionBatch) -> Result<ActionBatch> {
        let row = sqlx::query_as!(
            ActionBatch,
            r#"
            INSERT INTO action_batches (id, user_id, provider, idempotency_key, dry_run, status, options, summary, created_at, completed_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, user_id, provider, idempotency_key, dry_run, status as "status: ActionBatchStatus", options, summary, created_at, completed_at
            "#,
            batch.id,
            batch.user_id,
            batch.provider,
            batch.idempotency_key,
            batch.dry_run,
            batch.status.to_string(),
            batch.options,
            batch.summary,
            batch.created_at,
            batch.completed_at
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(row)
    }

    async fn update_batch(&self, batch: &ActionBatch) -> Result<()> {
        sqlx::query!(
            "UPDATE action_batches SET status = $1, summary = $2, completed_at = $3 WHERE id = $4",
            batch.status.to_string(),
            batch.summary,
            batch.completed_at,
            batch.id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn get_batch(&self, batch_id: &Uuid) -> Result<Option<ActionBatch>> {
        let row = sqlx::query_as!(
            ActionBatch,
            r#"
            SELECT id, user_id, provider, idempotency_key, dry_run, status as "status: ActionBatchStatus", options, summary, created_at, completed_at
            FROM action_batches WHERE id = $1
            "#,
            batch_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(row)
    }

    async fn get_batch_by_idempotency_key(&self, key: &str) -> Result<Option<ActionBatch>> {
        let row = sqlx::query_as!(
            ActionBatch,
            r#"
            SELECT id, user_id, provider, idempotency_key, dry_run, status as "status: ActionBatchStatus", options, summary, created_at, completed_at
            FROM action_batches WHERE idempotency_key = $1
            "#,
            key
        )
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(row)
    }

    async fn create_action_items(&self, batch_id: &Uuid, planned_actions: &[PlannedAction]) -> Result<Vec<ActionItem>> {
        let mut action_items = Vec::new();

        for planned_action in planned_actions {
            let action_item = ActionItem::new(
                *batch_id,
                planned_action.entity_type.to_string(),
                planned_action.entity_id.clone(),
                planned_action.action_type.to_string(),
                Some(planned_action.metadata.clone()),
            );

            let saved_item = sqlx::query_as!(
                ActionItem,
                r#"
                INSERT INTO action_items (id, batch_id, entity_type, entity_id, action, idempotency_key, before_state, after_state, status, error_message, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                RETURNING id, batch_id, entity_type, entity_id, action, idempotency_key, before_state, after_state, status as "status: ActionItemStatus", error_message, created_at
                "#,
                action_item.id,
                action_item.batch_id,
                action_item.entity_type,
                action_item.entity_id,
                action_item.action,
                action_item.idempotency_key,
                action_item.before_state,
                action_item.after_state,
                action_item.status.to_string(),
                action_item.error_message,
                action_item.created_at
            )
            .fetch_one(&self.db_pool)
            .await?;

            action_items.push(saved_item);
        }

        Ok(action_items)
    }

    async fn update_action_item(&self, action: &ActionItem) -> Result<()> {
        sqlx::query!(
            "UPDATE action_items SET status = $1, after_state = $2, error_message = $3 WHERE id = $4",
            action.status.to_string(),
            action.after_state,
            action.error_message,
            action.id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn get_batch_actions(&self, batch_id: &Uuid, status_filter: Option<ActionItemStatus>) -> Result<Vec<ActionItem>> {
        let query = if let Some(status) = status_filter {
            sqlx::query_as!(
                ActionItem,
                r#"
                SELECT id, batch_id, entity_type, entity_id, action, idempotency_key, before_state, after_state, status as "status: ActionItemStatus", error_message, created_at
                FROM action_items WHERE batch_id = $1 AND status = $2
                ORDER BY created_at
                "#,
                batch_id,
                status.to_string()
            )
        } else {
            sqlx::query_as!(
                ActionItem,
                r#"
                SELECT id, batch_id, entity_type, entity_id, action, idempotency_key, before_state, after_state, status as "status: ActionItemStatus", error_message, created_at
                FROM action_items WHERE batch_id = $1
                ORDER BY created_at
                "#,
                batch_id
            )
        };

        let rows = query.fetch_all(&self.db_pool).await?;
        Ok(rows)
    }

    async fn get_batch_actions_by_ids(&self, action_ids: &[Uuid]) -> Result<Vec<ActionItem>> {
        let rows = sqlx::query_as!(
            ActionItem,
            r#"
            SELECT id, batch_id, entity_type, entity_id, action, idempotency_key, before_state, after_state, status as "status: ActionItemStatus", error_message, created_at
            FROM action_items WHERE id = ANY($1)
            ORDER BY created_at
            "#,
            action_ids
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(rows)
    }

    async fn queue_batch_for_execution(&self, batch: &ActionBatch) -> Result<()> {
        // This would integrate with a job queue system like Redis/BullMQ
        // For now, just log that it would be queued
        tracing::info!("Batch {} queued for background execution", batch.id);
        Ok(())
    }

    fn create_rollback_action(&self, rollback_batch_id: &Uuid, original_action: &ActionItem) -> Result<ActionItem> {
        let rollback_action_type = match original_action.action.as_str() {
            "remove_liked_song" => "add_liked_song",
            "remove_playlist_track" => "add_playlist_track",
            "unfollow_artist" => "follow_artist",
            "remove_saved_album" => "add_saved_album",
            _ => return Err(anyhow!("Cannot create rollback for action: {}", original_action.action)),
        };

        Ok(ActionItem::new(
            *rollback_batch_id,
            original_action.entity_type.clone(),
            original_action.entity_id.clone(),
            rollback_action_type.to_string(),
            original_action.after_state.clone(),
        ))
    }

    async fn execute_rollback_action(&self, connection: &Connection, mut action: ActionItem) -> Result<ActionItem> {
        action.status = ActionItemStatus::InProgress;
        self.update_action_item(&action).await?;

        let result = match action.action.as_str() {
            "add_liked_song" => {
                self.add_liked_songs_batch(connection, &[action.entity_id.clone()]).await?;
                json!({ "added_at": Utc::now() })
            }
            "follow_artist" => {
                self.follow_artists_batch(connection, &[action.entity_id.clone()]).await?;
                json!({ "followed_at": Utc::now() })
            }
            "add_saved_album" => {
                self.add_saved_albums_batch(connection, &[action.entity_id.clone()]).await?;
                json!({ "added_at": Utc::now() })
            }
            "add_playlist_track" => {
                // This is more complex as we need playlist context
                if let Some(before_state) = &action.before_state {
                    if let Some(playlist_id) = before_state.get("playlist_id").and_then(|p| p.as_str()) {
                        self.add_playlist_tracks_batch(connection, playlist_id, &[action.entity_id.clone()]).await?;
                        json!({ "added_at": Utc::now(), "playlist_id": playlist_id })
                    } else {
                        return Err(anyhow!("Missing playlist_id for rollback"));
                    }
                } else {
                    return Err(anyhow!("Missing before_state for playlist track rollback"));
                }
            }
            _ => return Err(anyhow!("Unknown rollback action: {}", action.action)),
        };

        action.mark_completed(result);
        self.update_action_item(&action).await?;
        Ok(action)
    }

    // Spotify API operations

    async fn remove_liked_songs_batch(&self, connection: &Connection, track_ids: &[String]) -> Result<()> {
        let url = "https://api.spotify.com/v1/me/tracks";
        let body = json!({ "ids": track_ids });

        let response = self
            .spotify_service
            .make_api_request(connection, "DELETE", url, Some(body))
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to remove liked songs: {}", response.status()));
        }

        Ok(())
    }

    async fn add_liked_songs_batch(&self, connection: &Connection, track_ids: &[String]) -> Result<()> {
        let url = "https://api.spotify.com/v1/me/tracks";
        let body = json!({ "ids": track_ids });

        let response = self
            .spotify_service
            .make_api_request(connection, "PUT", url, Some(body))
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to add liked songs: {}", response.status()));
        }

        Ok(())
    }

    async fn remove_playlist_tracks_batch(&self, connection: &Connection, playlist_id: &str, tracks: &[Value]) -> Result<String> {
        let url = format!("https://api.spotify.com/v1/playlists/{}/tracks", playlist_id);
        let body = json!({ "tracks": tracks });

        let response = self
            .spotify_service
            .make_api_request(connection, "DELETE", &url, Some(body))
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to remove playlist tracks: {}", response.status()));
        }

        let result: Value = response.json().await?;
        let snapshot_id = result["snapshot_id"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing snapshot_id in response"))?;

        Ok(snapshot_id.to_string())
    }

    async fn add_playlist_tracks_batch(&self, connection: &Connection, playlist_id: &str, track_ids: &[String]) -> Result<String> {
        let url = format!("https://api.spotify.com/v1/playlists/{}/tracks", playlist_id);
        let uris: Vec<String> = track_ids.iter().map(|id| format!("spotify:track:{}", id)).collect();
        let body = json!({ "uris": uris });

        let response = self
            .spotify_service
            .make_api_request(connection, "POST", &url, Some(body))
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to add playlist tracks: {}", response.status()));
        }

        let result: Value = response.json().await?;
        let snapshot_id = result["snapshot_id"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing snapshot_id in response"))?;

        Ok(snapshot_id.to_string())
    }

    async fn unfollow_artists_batch(&self, connection: &Connection, artist_ids: &[String]) -> Result<()> {
        let url = "https://api.spotify.com/v1/me/following";
        let params = format!("type=artist&ids={}", artist_ids.join(","));
        let full_url = format!("{}?{}", url, params);

        let response = self
            .spotify_service
            .make_api_request(connection, "DELETE", &full_url, None)
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to unfollow artists: {}", response.status()));
        }

        Ok(())
    }

    async fn follow_artists_batch(&self, connection: &Connection, artist_ids: &[String]) -> Result<()> {
        let url = "https://api.spotify.com/v1/me/following";
        let params = format!("type=artist&ids={}", artist_ids.join(","));
        let full_url = format!("{}?{}", url, params);

        let response = self
            .spotify_service
            .make_api_request(connection, "PUT", &full_url, None)
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to follow artists: {}", response.status()));
        }

        Ok(())
    }

    async fn remove_saved_albums_batch(&self, connection: &Connection, album_ids: &[String]) -> Result<()> {
        let url = "https://api.spotify.com/v1/me/albums";
        let body = json!({ "ids": album_ids });

        let response = self
            .spotify_service
            .make_api_request(connection, "DELETE", url, Some(body))
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to remove saved albums: {}", response.status()));
        }

        Ok(())
    }

    async fn add_saved_albums_batch(&self, connection: &Connection, album_ids: &[String]) -> Result<()> {
        let url = "https://api.spotify.com/v1/me/albums";
        let body = json!({ "ids": album_ids });

        let response = self
            .spotify_service
            .make_api_request(connection, "PUT", url, Some(body))
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to add saved albums: {}", response.status()));
        }

        Ok(())
    }
}

// Helper trait implementations
impl std::fmt::Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionType::RemoveLikedSong => write!(f, "remove_liked_song"),
            ActionType::RemovePlaylistTrack => write!(f, "remove_playlist_track"),
            ActionType::UnfollowArtist => write!(f, "unfollow_artist"),
            ActionType::RemoveSavedAlbum => write!(f, "remove_saved_album"),
            ActionType::SkipTrack => write!(f, "skip_track"),
        }
    }
}

impl std::fmt::Display for crate::models::EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::models::EntityType::Track => write!(f, "track"),
            crate::models::EntityType::Artist => write!(f, "artist"),
            crate::models::EntityType::Album => write!(f, "album"),
            crate::models::EntityType::Playlist => write!(f, "playlist"),
        }
    }
}