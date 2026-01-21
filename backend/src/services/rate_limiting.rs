use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use deadpool_redis::{Config, Pool, Runtime};
use redis::AsyncCommands;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{sleep, Instant};
use uuid::Uuid;

use crate::models::{
    BatchCheckpoint, BatchConfig, CircuitBreakerState, CircuitState, RateLimitConfig,
    RateLimitResponse, RateLimitState, RateLimitedRequest, RequestPriority,
};

/// Rate limiting service with circuit breaker and batching support
pub struct RateLimitingService {
    redis_pool: Pool,
    configs: Arc<RwLock<HashMap<String, RateLimitConfig>>>,
    batch_configs: Arc<RwLock<HashMap<String, BatchConfig>>>,
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreakerState>>>,
}

impl RateLimitingService {
    pub fn new(redis_url: &str) -> Result<Self> {
        let config = Config::from_url(redis_url);
        let pool = config.create_pool(Some(Runtime::Tokio1))?;

        let mut configs = HashMap::new();
        configs.insert("spotify".to_string(), RateLimitConfig::spotify());
        configs.insert("apple_music".to_string(), RateLimitConfig::apple_music());

        let mut batch_configs = HashMap::new();
        batch_configs.insert(
            "spotify_remove_tracks".to_string(),
            BatchConfig::spotify_remove_tracks(),
        );
        batch_configs.insert(
            "spotify_unfollow_artists".to_string(),
            BatchConfig::spotify_unfollow_artists(),
        );
        batch_configs.insert(
            "spotify_playlist_operations".to_string(),
            BatchConfig::spotify_playlist_operations(),
        );

        Ok(Self {
            redis_pool: pool,
            configs: Arc::new(RwLock::new(configs)),
            batch_configs: Arc::new(RwLock::new(batch_configs)),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Check if a request can proceed based on rate limits and circuit breaker
    pub async fn can_proceed(&self, provider: &str) -> Result<bool> {
        // Check circuit breaker first
        if !self.is_circuit_closed(provider).await? {
            return Ok(false);
        }

        // Check rate limit
        let state = self.get_rate_limit_state(provider).await?;
        Ok(state.requests_remaining > 0)
    }

    /// Wait for rate limit window if needed
    pub async fn wait_for_rate_limit(&self, provider: &str) -> Result<Duration> {
        let state = self.get_rate_limit_state(provider).await?;

        if state.requests_remaining == 0 {
            let wait_duration = state
                .window_reset_at
                .signed_duration_since(Utc::now())
                .to_std()
                .unwrap_or(Duration::from_secs(0));

            if wait_duration > Duration::from_secs(0) {
                tracing::info!(
                    "Rate limit hit for {}, waiting {} seconds",
                    provider,
                    wait_duration.as_secs()
                );
                sleep(wait_duration).await;
                return Ok(wait_duration);
            }
        }

        // Add minimum delay between requests
        let min_delay = Duration::from_millis(100);
        if let Some(last_request) = state.last_request_at {
            let time_since_last = Utc::now()
                .signed_duration_since(last_request)
                .to_std()
                .unwrap_or(Duration::from_secs(0));

            if time_since_last < min_delay {
                let delay = min_delay - time_since_last;
                sleep(delay).await;
                return Ok(delay);
            }
        }

        Ok(Duration::from_secs(0))
    }

    /// Record a successful API request and update rate limit state
    pub async fn record_success(&self, provider: &str, response: RateLimitResponse) -> Result<()> {
        // Update circuit breaker
        self.record_circuit_success(provider).await?;

        // Update rate limit state from response headers
        let mut conn = self.redis_pool.get().await?;
        let key = format!("rate_limit:{}", provider);

        let mut state = self.get_rate_limit_state(provider).await?;

        if let Some(remaining) = response.requests_remaining {
            state.requests_remaining = remaining;
        } else if state.requests_remaining > 0 {
            state.requests_remaining -= 1;
        }

        if let Some(reset_at) = response.reset_at {
            state.window_reset_at = reset_at;
        }

        state.last_request_at = Some(Utc::now());
        state.current_backoff_seconds = 0;
        state.consecutive_failures = 0;

        let state_json = serde_json::to_string(&state)?;
        let _: () = conn.set_ex(&key, state_json, 3600).await?;

        Ok(())
    }

    /// Record a failed API request and update circuit breaker
    pub async fn record_failure(&self, provider: &str, error: &str) -> Result<()> {
        // Update circuit breaker
        self.record_circuit_failure(provider).await?;

        // Update rate limit state with backoff
        let mut conn = self.redis_pool.get().await?;
        let key = format!("rate_limit:{}", provider);

        let mut state = self.get_rate_limit_state(provider).await?;
        let config = self.get_config(provider).await?;

        state.consecutive_failures += 1;
        state.current_backoff_seconds = std::cmp::min(
            (config
                .backoff_multiplier
                .powi(state.consecutive_failures as i32) as u32),
            config.max_backoff_seconds,
        );

        // If it's a rate limit error, set remaining to 0
        if error.contains("rate limit") || error.contains("429") {
            state.requests_remaining = 0;
            if state.window_reset_at <= Utc::now() {
                state.window_reset_at = Utc::now() + chrono::Duration::seconds(60);
            }
        }

        let state_json = serde_json::to_string(&state)?;
        let _: () = conn.set_ex(&key, state_json, 3600).await?;

        Ok(())
    }

    /// Get optimal batch size for an operation
    pub async fn get_optimal_batch_size(&self, provider: &str, operation: &str) -> Result<u32> {
        let batch_configs = self.batch_configs.read().await;
        let key = format!("{}_{}", provider, operation);

        if let Some(config) = batch_configs.get(&key) {
            // Adjust batch size based on current rate limit state
            let state = self.get_rate_limit_state(provider).await?;
            let adjusted_size = if state.requests_remaining < config.optimal_batch_size {
                std::cmp::max(1, state.requests_remaining)
            } else {
                config.optimal_batch_size
            };

            Ok(adjusted_size)
        } else {
            Ok(20) // Default batch size
        }
    }

    /// Create optimal batches from a list of items
    pub async fn create_optimal_batches<T: Clone>(
        &self,
        provider: &str,
        operation: &str,
        items: Vec<T>,
    ) -> Result<Vec<Vec<T>>> {
        let batch_size = self.get_optimal_batch_size(provider, operation).await? as usize;
        let batches = items
            .chunks(batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();
        Ok(batches)
    }

    /// Execute batches with rate limiting and circuit breaker
    pub async fn execute_batches<T, F, Fut, R>(
        &self,
        provider: &str,
        operation: &str,
        batches: Vec<Vec<T>>,
        executor: F,
    ) -> Result<Vec<Result<R>>>
    where
        F: Fn(Vec<T>) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<R>> + Send,
        T: Send + Sync,
        R: Send,
    {
        let mut results = Vec::new();
        let batch_configs = self.batch_configs.read().await;
        let key = format!("{}_{}", provider, operation);
        let min_delay = batch_configs
            .get(&key)
            .map(|c| Duration::from_millis(c.min_delay_between_batches_ms))
            .unwrap_or(Duration::from_millis(100));

        for (i, batch) in batches.into_iter().enumerate() {
            // Check circuit breaker before each batch
            if !self.can_proceed(provider).await? {
                results.push(Err(anyhow!("Circuit breaker open for {}", provider)));
                continue;
            }

            // Wait for rate limit
            let wait_time = self.wait_for_rate_limit(provider).await?;

            // Add minimum delay between batches (except for first batch)
            if i > 0 {
                sleep(min_delay).await;
            }

            // Execute the batch
            let start_time = Instant::now();
            match executor(batch).await {
                Ok(result) => {
                    // Record success
                    self.record_success(
                        provider,
                        RateLimitResponse {
                            requests_remaining: None,
                            reset_at: None,
                            retry_after_seconds: None,
                            rate_limit_hit: false,
                        },
                    )
                    .await?;

                    results.push(Ok(result));

                    tracing::debug!(
                        "Batch {} completed in {}ms for {}",
                        i + 1,
                        start_time.elapsed().as_millis(),
                        provider
                    );
                }
                Err(e) => {
                    // Record failure
                    let error_msg = e.to_string();
                    self.record_failure(provider, &error_msg).await?;

                    tracing::warn!(
                        "Batch {} failed after {}ms for {}: {}",
                        i + 1,
                        start_time.elapsed().as_millis(),
                        provider,
                        error_msg
                    );

                    results.push(Err(e));
                }
            }
        }

        Ok(results)
    }

    /// Create and save a batch checkpoint
    pub async fn create_checkpoint(
        &self,
        batch_id: Uuid,
        provider: String,
        operation_type: String,
        total_items: u32,
    ) -> Result<BatchCheckpoint> {
        let checkpoint = BatchCheckpoint::new(batch_id, provider, operation_type, total_items);
        self.save_checkpoint(&checkpoint).await?;
        Ok(checkpoint)
    }

    /// Update batch checkpoint with progress
    pub async fn update_checkpoint(
        &self,
        checkpoint: &mut BatchCheckpoint,
        processed: u32,
        failed: u32,
        position: u32,
        last_item_id: Option<String>,
        data: serde_json::Value,
    ) -> Result<()> {
        checkpoint.update_progress(processed, failed, position, last_item_id, data);
        self.save_checkpoint(checkpoint).await?;
        Ok(())
    }

    /// Get existing checkpoint for resumable processing
    pub async fn get_checkpoint(&self, batch_id: &Uuid) -> Result<Option<BatchCheckpoint>> {
        let mut conn = self.redis_pool.get().await?;
        let key = format!("checkpoint:{}", batch_id);

        let checkpoint_json: Option<String> = conn.get(&key).await?;
        if let Some(json) = checkpoint_json {
            let checkpoint: BatchCheckpoint = serde_json::from_str(&json)?;
            Ok(Some(checkpoint))
        } else {
            Ok(None)
        }
    }

    /// Exponential backoff with jitter
    pub async fn exponential_backoff(&self, attempt: u32, base_delay_ms: u64) -> Duration {
        let max_delay = Duration::from_secs(300); // 5 minutes max
        let base_delay = Duration::from_millis(base_delay_ms);

        let exponential_delay = base_delay * (2_u32.pow(attempt.min(10))); // Cap at 2^10
        let jittered_delay =
            exponential_delay + Duration::from_millis((rand::random::<f64>() * 1000.0) as u64);

        let final_delay = std::cmp::min(jittered_delay, max_delay);

        tracing::debug!(
            "Exponential backoff: attempt {}, delay {}ms",
            attempt,
            final_delay.as_millis()
        );

        sleep(final_delay).await;
        final_delay
    }

    // Private helper methods

    async fn get_rate_limit_state(&self, provider: &str) -> Result<RateLimitState> {
        let mut conn = self.redis_pool.get().await?;
        let key = format!("rate_limit:{}", provider);

        let state_json: Option<String> = conn.get(&key).await?;
        if let Some(json) = state_json {
            let state: RateLimitState = serde_json::from_str(&json)?;
            Ok(state)
        } else {
            // Create default state
            let config = self.get_config(provider).await?;
            let state = RateLimitState {
                provider: provider.to_string(),
                requests_remaining: config.requests_per_window,
                window_reset_at: Utc::now()
                    + chrono::Duration::seconds(config.window_duration_seconds as i64),
                current_backoff_seconds: 0,
                consecutive_failures: 0,
                last_request_at: None,
            };

            let state_json = serde_json::to_string(&state)?;
            let _: () = conn
                .set_ex(&key, state_json, config.window_duration_seconds as u64)
                .await?;
            Ok(state)
        }
    }

    async fn get_config(&self, provider: &str) -> Result<RateLimitConfig> {
        let configs = self.configs.read().await;
        configs
            .get(provider)
            .cloned()
            .ok_or_else(|| anyhow!("No rate limit config for provider: {}", provider))
    }

    async fn is_circuit_closed(&self, provider: &str) -> Result<bool> {
        let circuit_breakers = self.circuit_breakers.read().await;
        if let Some(breaker) = circuit_breakers.get(provider) {
            Ok(breaker.should_allow_request())
        } else {
            Ok(true) // Default to closed (allow requests)
        }
    }

    async fn record_circuit_success(&self, provider: &str) -> Result<()> {
        let mut circuit_breakers = self.circuit_breakers.write().await;
        let breaker = circuit_breakers
            .entry(provider.to_string())
            .or_insert_with(|| CircuitBreakerState {
                provider: provider.to_string(),
                ..Default::default()
            });

        breaker.record_success();
        Ok(())
    }

    async fn record_circuit_failure(&self, provider: &str) -> Result<()> {
        let mut circuit_breakers = self.circuit_breakers.write().await;
        let breaker = circuit_breakers
            .entry(provider.to_string())
            .or_insert_with(|| CircuitBreakerState {
                provider: provider.to_string(),
                ..Default::default()
            });

        breaker.record_failure();

        tracing::warn!(
            "Circuit breaker for {} recorded failure. State: {:?}, Failures: {}",
            provider,
            breaker.state,
            breaker.failure_count
        );

        Ok(())
    }

    async fn save_checkpoint(&self, checkpoint: &BatchCheckpoint) -> Result<()> {
        let mut conn = self.redis_pool.get().await?;
        let key = format!("checkpoint:{}", checkpoint.batch_id);
        let checkpoint_json = serde_json::to_string(checkpoint)?;

        // Store checkpoint for 24 hours
        let _: () = conn.set_ex(&key, checkpoint_json, 86400).await?;
        Ok(())
    }
}

/// Builder for rate limiting configurations
pub struct RateLimitConfigBuilder {
    config: RateLimitConfig,
}

impl RateLimitConfigBuilder {
    pub fn new(provider: String) -> Self {
        Self {
            config: RateLimitConfig {
                provider,
                ..Default::default()
            },
        }
    }

    pub fn requests_per_window(mut self, requests: u32) -> Self {
        self.config.requests_per_window = requests;
        self
    }

    pub fn window_duration_seconds(mut self, seconds: u32) -> Self {
        self.config.window_duration_seconds = seconds;
        self
    }

    pub fn burst_allowance(mut self, burst: u32) -> Self {
        self.config.burst_allowance = burst;
        self
    }

    pub fn backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.config.backoff_multiplier = multiplier;
        self
    }

    pub fn max_backoff_seconds(mut self, seconds: u32) -> Self {
        self.config.max_backoff_seconds = seconds;
        self
    }

    pub fn build(self) -> RateLimitConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_circuit_breaker_transitions() {
        let mut breaker = CircuitBreakerState::default();

        // Initially closed
        assert_eq!(breaker.state, CircuitState::Closed);
        assert!(breaker.should_allow_request());

        // Record failures to open circuit
        for _ in 0..5 {
            breaker.record_failure();
        }
        assert_eq!(breaker.state, CircuitState::Open);

        // Should not allow requests when open
        assert!(!breaker.should_allow_request());

        // Transition to half-open
        breaker.transition_to_half_open();
        assert_eq!(breaker.state, CircuitState::HalfOpen);
        assert!(breaker.should_allow_request());

        // Record successes to close circuit
        for _ in 0..3 {
            breaker.record_success();
        }
        assert_eq!(breaker.state, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_batch_checkpoint_progress() {
        let batch_id = Uuid::new_v4();
        let mut checkpoint = BatchCheckpoint::new(
            batch_id,
            "spotify".to_string(),
            "remove_tracks".to_string(),
            100,
        );

        assert_eq!(checkpoint.progress_percentage(), 0.0);
        assert!(!checkpoint.is_complete());

        checkpoint.update_progress(
            50,
            5,
            55,
            Some("track_123".to_string()),
            serde_json::json!({"current_playlist": "playlist_456"}),
        );

        assert_eq!(checkpoint.progress_percentage(), 55.0);
        assert!(!checkpoint.is_complete());

        checkpoint.update_progress(
            90,
            10,
            100,
            Some("track_999".to_string()),
            serde_json::json!({"completed": true}),
        );

        assert_eq!(checkpoint.progress_percentage(), 100.0);
        assert!(checkpoint.is_complete());
    }
}
