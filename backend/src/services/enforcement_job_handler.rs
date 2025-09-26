use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::models::{
    ActionBatch, ActionBatchStatus, ExecuteBatchRequest, Connection,
    EnforcementPlan, BatchExecutionResult,
};
use crate::services::{
    JobHandler, Job, JobType, SpotifyService, SpotifyEnforcementService,
    RateLimitingService, JobProgress,
};

/// Job handler for enforcement execution operations
pub struct EnforcementJobHandler {
    spotify_service: Arc<SpotifyService>,
    enforcement_service: Arc<SpotifyEnforcementService>,
    rate_limiter: Arc<RateLimitingService>,
}

impl EnforcementJobHandler {
    pub fn new(
        spotify_service: Arc<SpotifyService>,
        enforcement_service: Arc<SpotifyEnforcementService>,
        rate_limiter: Arc<RateLimitingService>,
    ) -> Self {
        Self {
            spotify_service,
            enforcement_service,
            rate_limiter,
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
        request: ExecuteBatchRequest,
        job_id: Uuid,
    ) -> Result<BatchExecutionResult> {
        let total_actions = plan.actions.len() as u32;
        let mut completed_actions = 0;

        // Update initial progress
        self.update_progress(
            job_id,
            "Starting enforcement execution",
            0,
            total_actions,
            0.0,
            serde_json::json!({
                "plan_id": plan.id,
                "total_actions": total_actions,
                "dry_run": plan.options.dry_run
            }),
        ).await?;

        // Group actions by type for optimal batching
        let action_groups = self.group_actions_by_type(&plan.actions);
        let total_groups = action_groups.len() as u32;
        let mut completed_groups = 0;

        let mut all_results = Vec::new();

        for (action_type, actions) in action_groups {
            // Update progress for current group
            self.update_progress(
                job_id,
                &format!("Processing {} actions", action_type),
                completed_groups,
                total_groups,
                (completed_groups as f64 / total_groups as f64) * 100.0,
                serde_json::json!({
                    "current_action_type": action_type,
                    "actions_in_group": actions.len()
                }),
            ).await?;

            // Create batches for this action type
            let batches = self.rate_limiter.create_optimal_batches(
                &connection.provider,
                &action_type,
                actions,
            ).await?;

            // Execute batches with rate limiting
            let batch_results = self.rate_limiter.execute_batches(
                &connection.provider,
                &action_type,
                batches,
                |batch| self.execute_action_batch(connection, batch),
            ).await?;

            // Process batch results
            for result in batch_results {
                match result {
                    Ok(batch_result) => {
                        completed_actions += batch_result.len() as u32;
                        all_results.extend(batch_result);
                    }
                    Err(e) => {
                        tracing::error!("Batch execution failed: {}", e);
                        // Continue with other batches
                    }
                }
            }

            completed_groups += 1;

            // Update progress after group completion
            self.update_progress(
                job_id,
                &format!("Completed {} actions", action_type),
                completed_groups,
                total_groups,
                (completed_groups as f64 / total_groups as f64) * 100.0,
                serde_json::json!({
                    "completed_actions": completed_actions,
                    "total_actions": total_actions
                }),
            ).await?;
        }

        // Final progress update
        self.update_progress(
            job_id,
            "Enforcement execution completed",
            total_groups,
            total_groups,
            100.0,
            serde_json::json!({
                "completed_actions": completed_actions,
                "total_actions": total_actions,
                "success_rate": if total_actions > 0 { 
                    (completed_actions as f64 / total_actions as f64) * 100.0 
                } else { 
                    100.0 
                }
            }),
        ).await?;

        // Create final result
        let result = BatchExecutionResult {
            batch_id: job_id,
            status: if completed_actions == total_actions {
                ActionBatchStatus::Completed
            } else if completed_actions > 0 {
                ActionBatchStatus::PartiallyCompleted
            } else {
                ActionBatchStatus::Failed
            },
            summary: crate::models::BatchSummary {
                total_actions,
                completed_actions,
                failed_actions: total_actions - completed_actions,
                skipped_actions: 0,
                execution_time_ms: 0, // This would be calculated from start time
                api_calls_made: completed_actions, // Approximate
                rate_limit_delays_ms: 0,
                errors: Vec::new(),
            },
            completed_actions: Vec::new(), // Would be populated with actual action items
            failed_actions: Vec::new(),
            rollback_info: None,
        };

        Ok(result)
    }

    async fn execute_action_batch(
        &self,
        connection: &Connection,
        actions: Vec<crate::models::PlannedAction>,
    ) -> Result<Vec<String>> {
        // This is a simplified implementation
        // In reality, this would call the appropriate Spotify API endpoints
        let mut results = Vec::new();
        
        for action in actions {
            match action.action_type.as_str() {
                "remove_liked_song" => {
                    // Simulate API call with rate limiting
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    results.push(format!("Removed track {}", action.entity_id));
                }
                "unfollow_artist" => {
                    tokio::time::sleep(Duration::from_millis(150)).await;
                    results.push(format!("Unfollowed artist {}", action.entity_id));
                }
                "remove_playlist_track" => {
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    results.push(format!("Removed track {} from playlist", action.entity_id));
                }
                _ => {
                    tracing::warn!("Unknown action type: {}", action.action_type);
                }
            }
        }
        
        Ok(results)
    }

    fn group_actions_by_type(
        &self,
        actions: &[crate::models::PlannedAction],
    ) -> Vec<(String, Vec<crate::models::PlannedAction>)> {
        use std::collections::HashMap;
        
        let mut groups: HashMap<String, Vec<crate::models::PlannedAction>> = HashMap::new();
        
        for action in actions {
            groups.entry(action.action_type.clone())
                .or_default()
                .push(action.clone());
        }
        
        groups.into_iter().collect()
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
            actions: vec![
                crate::models::PlannedAction {
                    entity_type: crate::models::spotify::EntityType::Track,
                    entity_id: "track_123".to_string(),
                    action_type: crate::models::spotify::ActionType::RemoveLikedSong,
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
            },
            impact_summary: crate::models::ImpactSummary {
                total_tracks_affected: 1,
                playlists_affected: 0,
                artists_affected: 1,
                estimated_duration_seconds: 30,
                provider_capabilities: std::collections::HashMap::new(),
            },
            created_at: chrono::Utc::now(),
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
    enforcement_service: Arc<SpotifyEnforcementService>,
    rate_limiter: Arc<RateLimitingService>,
}

impl RollbackJobHandler {
    pub fn new(
        enforcement_service: Arc<SpotifyEnforcementService>,
        rate_limiter: Arc<RateLimitingService>,
    ) -> Self {
        Self {
            enforcement_service,
            rate_limiter,
        }
    }

    async fn execute_rollback_job(&self, job: &Job) -> Result<serde_json::Value> {
        let payload = &job.payload;
        let user_id = job.user_id.ok_or_else(|| anyhow!("User ID required for rollback job"))?;

        let rollback_request: crate::models::RollbackBatchRequest = serde_json::from_value(
            payload.get("rollback_request").cloned().ok_or_else(|| anyhow!("rollback_request required"))?
        )?;

        tracing::info!(
            "Starting rollback job {} for user {} with batch {}",
            job.id,
            user_id,
            rollback_request.batch_id
        );

        // This would get the user's connection and execute the rollback
        // For now, return a mock result
        let result = serde_json::json!({
            "rollback_batch_id": rollback_request.batch_id,
            "status": "completed",
            "actions_rolled_back": 0,
            "message": "Rollback completed successfully"
        });

        Ok(result)
    }
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
            match self.spotify_service.refresh_connection_tokens(&connection).await {
                Ok(updated_connection) => {
                    tracing::info!("Successfully refreshed tokens for user {}", user_id);
                    Ok(serde_json::json!({
                        "status": "success",
                        "user_id": user_id,
                        "provider": updated_connection.provider,
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