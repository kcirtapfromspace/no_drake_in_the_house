use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Rate limit configuration for a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub provider: String,
    pub requests_per_window: u32,
    pub window_duration_seconds: u32,
    pub burst_allowance: u32,
    pub backoff_multiplier: f64,
    pub max_backoff_seconds: u32,
}

/// Current rate limit state for a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitState {
    pub provider: String,
    pub requests_remaining: u32,
    pub window_reset_at: DateTime<Utc>,
    pub current_backoff_seconds: u32,
    pub consecutive_failures: u32,
    pub last_request_at: Option<DateTime<Utc>>,
}

/// Rate limit response from provider API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResponse {
    pub requests_remaining: Option<u32>,
    pub reset_at: Option<DateTime<Utc>>,
    pub retry_after_seconds: Option<u32>,
    pub rate_limit_hit: bool,
}

/// Batch configuration for optimal API usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    pub provider: String,
    pub operation_type: String,
    pub max_batch_size: u32,
    pub optimal_batch_size: u32,
    pub min_delay_between_batches_ms: u64,
    pub supports_parallel_batches: bool,
}

/// Circuit breaker state for a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerState {
    pub provider: String,
    pub state: CircuitState,
    pub failure_count: u32,
    pub last_failure_at: Option<DateTime<Utc>>,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub success_count_in_half_open: u32,
}

/// Circuit breaker states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Failing, rejecting requests
    HalfOpen, // Testing if service recovered
}

/// Request for rate-limited operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitedRequest {
    pub id: Uuid,
    pub provider: String,
    pub operation_type: String,
    pub priority: RequestPriority,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub max_retries: u32,
    pub retry_count: u32,
}

/// Priority levels for rate-limited requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Checkpoint for resumable batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCheckpoint {
    pub batch_id: Uuid,
    pub provider: String,
    pub operation_type: String,
    pub total_items: u32,
    pub processed_items: u32,
    pub failed_items: u32,
    pub current_position: u32,
    pub last_successful_item_id: Option<String>,
    pub checkpoint_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            provider: "default".to_string(),
            requests_per_window: 100,
            window_duration_seconds: 3600,
            burst_allowance: 10,
            backoff_multiplier: 2.0,
            max_backoff_seconds: 300,
        }
    }
}

impl RateLimitConfig {
    pub fn spotify() -> Self {
        Self {
            provider: "spotify".to_string(),
            requests_per_window: 100,
            window_duration_seconds: 60,
            burst_allowance: 20,
            backoff_multiplier: 1.5,
            max_backoff_seconds: 120,
        }
    }

    pub fn apple_music() -> Self {
        Self {
            provider: "apple_music".to_string(),
            requests_per_window: 1000,
            window_duration_seconds: 3600,
            burst_allowance: 50,
            backoff_multiplier: 2.0,
            max_backoff_seconds: 300,
        }
    }
}

impl Default for CircuitBreakerState {
    fn default() -> Self {
        Self {
            provider: "default".to_string(),
            state: CircuitState::Closed,
            failure_count: 0,
            last_failure_at: None,
            next_attempt_at: None,
            success_count_in_half_open: 0,
        }
    }
}

impl CircuitBreakerState {
    pub fn should_allow_request(&self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(next_attempt) = self.next_attempt_at {
                    Utc::now() >= next_attempt
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.success_count_in_half_open += 1;
                if self.success_count_in_half_open >= 3 {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count_in_half_open = 0;
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but reset to closed if it does
                self.state = CircuitState::Closed;
                self.failure_count = 0;
            }
        }
    }

    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_at = Some(Utc::now());

        match self.state {
            CircuitState::Closed => {
                if self.failure_count >= 5 {
                    self.state = CircuitState::Open;
                    self.next_attempt_at = Some(Utc::now() + chrono::Duration::seconds(30));
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                self.success_count_in_half_open = 0;
                self.next_attempt_at = Some(Utc::now() + chrono::Duration::seconds(60));
            }
            CircuitState::Open => {
                // Extend the wait time
                let backoff_seconds = std::cmp::min(60 * (self.failure_count - 4), 300);
                self.next_attempt_at = Some(Utc::now() + chrono::Duration::seconds(backoff_seconds as i64));
            }
        }
    }

    pub fn transition_to_half_open(&mut self) {
        if self.state == CircuitState::Open {
            self.state = CircuitState::HalfOpen;
            self.success_count_in_half_open = 0;
        }
    }
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            provider: "default".to_string(),
            operation_type: "default".to_string(),
            max_batch_size: 50,
            optimal_batch_size: 20,
            min_delay_between_batches_ms: 100,
            supports_parallel_batches: false,
        }
    }
}

impl BatchConfig {
    pub fn spotify_remove_tracks() -> Self {
        Self {
            provider: "spotify".to_string(),
            operation_type: "remove_tracks".to_string(),
            max_batch_size: 50,
            optimal_batch_size: 25,
            min_delay_between_batches_ms: 200,
            supports_parallel_batches: false,
        }
    }

    pub fn spotify_unfollow_artists() -> Self {
        Self {
            provider: "spotify".to_string(),
            operation_type: "unfollow_artists".to_string(),
            max_batch_size: 50,
            optimal_batch_size: 20,
            min_delay_between_batches_ms: 150,
            supports_parallel_batches: false,
        }
    }

    pub fn spotify_playlist_operations() -> Self {
        Self {
            provider: "spotify".to_string(),
            operation_type: "playlist_operations".to_string(),
            max_batch_size: 100,
            optimal_batch_size: 50,
            min_delay_between_batches_ms: 300,
            supports_parallel_batches: false,
        }
    }
}

impl RateLimitedRequest {
    pub fn new(
        provider: String,
        operation_type: String,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            provider,
            operation_type,
            priority: RequestPriority::Normal,
            payload,
            created_at: Utc::now(),
            max_retries: 3,
            retry_count: 0,
        }
    }

    pub fn with_priority(mut self, priority: RequestPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}

impl BatchCheckpoint {
    pub fn new(
        batch_id: Uuid,
        provider: String,
        operation_type: String,
        total_items: u32,
    ) -> Self {
        Self {
            batch_id,
            provider,
            operation_type,
            total_items,
            processed_items: 0,
            failed_items: 0,
            current_position: 0,
            last_successful_item_id: None,
            checkpoint_data: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn update_progress(
        &mut self,
        processed: u32,
        failed: u32,
        position: u32,
        last_item_id: Option<String>,
        data: serde_json::Value,
    ) {
        self.processed_items = processed;
        self.failed_items = failed;
        self.current_position = position;
        self.last_successful_item_id = last_item_id;
        self.checkpoint_data = data;
        self.updated_at = Utc::now();
    }

    pub fn is_complete(&self) -> bool {
        self.processed_items + self.failed_items >= self.total_items
    }

    pub fn progress_percentage(&self) -> f64 {
        if self.total_items == 0 {
            return 100.0;
        }
        ((self.processed_items + self.failed_items) as f64 / self.total_items as f64) * 100.0
    }
}