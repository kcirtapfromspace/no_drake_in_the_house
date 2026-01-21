//! Circuit Breaker for Provider APIs (US-026)
//!
//! Implements a circuit breaker pattern to prevent cascading failures when provider APIs are down.
//!
//! ## State Machine
//! - **Closed**: Normal operation, all requests pass through
//! - **Open**: Provider unavailable, requests fail immediately with ProviderUnavailable error
//! - **HalfOpen**: Testing recovery, allows one request every 30 seconds
//!
//! ## Transitions
//! - Closed → Open: After 5 consecutive failures within 1 minute
//! - Open → HalfOpen: After 30 seconds, allows 1 test request
//! - HalfOpen → Closed: After 3 successful requests
//! - HalfOpen → Open: On any failure

use std::collections::HashMap;
use thiserror::Error;

/// Error type for circuit breaker operations
#[derive(Debug, Error)]
pub enum CircuitBreakerError {
    /// Circuit is open, request was blocked
    #[error("Circuit breaker is open for provider '{provider}' - service temporarily unavailable")]
    CircuitOpen { provider: String },

    /// The inner operation failed
    #[error("Operation failed: {0}")]
    ExecutionFailed(#[from] anyhow::Error),
}
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};

use prometheus::{CounterVec, IntGaugeVec, Opts, Registry};
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::error::{AppError, OAuthError};
use crate::models::oauth::OAuthProviderType;

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures to open the circuit
    pub failure_threshold: u32,
    /// Time window for counting failures (in seconds)
    pub failure_window_seconds: u64,
    /// Time to wait before allowing a test request in open state (in seconds)
    pub open_timeout_seconds: u64,
    /// Number of successes needed in half-open state to close circuit
    pub half_open_success_threshold: u32,
    /// Interval between test requests in half-open state (in seconds)
    pub half_open_test_interval_seconds: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            failure_window_seconds: 60,
            open_timeout_seconds: 30,
            half_open_success_threshold: 3,
            half_open_test_interval_seconds: 30,
        }
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitBreakerStateEnum {
    Closed,
    Open,
    HalfOpen,
}

impl std::fmt::Display for CircuitBreakerStateEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Closed => write!(f, "closed"),
            Self::Open => write!(f, "open"),
            Self::HalfOpen => write!(f, "half_open"),
        }
    }
}

/// Internal state for a single circuit breaker
#[derive(Debug)]
struct CircuitState {
    state: CircuitBreakerStateEnum,
    /// Timestamps of recent failures (for counting within window)
    failure_timestamps: Vec<Instant>,
    /// When the circuit was opened
    opened_at: Option<Instant>,
    /// Last time a test request was allowed in half-open state
    last_half_open_test: Option<Instant>,
    /// Number of successes in half-open state
    half_open_successes: u32,
}

impl Default for CircuitState {
    fn default() -> Self {
        Self {
            state: CircuitBreakerStateEnum::Closed,
            failure_timestamps: Vec::new(),
            opened_at: None,
            last_half_open_test: None,
            half_open_successes: 0,
        }
    }
}

/// Circuit breaker metrics
#[derive(Clone)]
pub struct CircuitBreakerMetrics {
    /// Current state of each circuit (0=closed, 1=open, 2=half_open)
    state_gauge: IntGaugeVec,
    /// Total number of circuit trips (transitions to open)
    trips_total: CounterVec,
    /// Total number of requests blocked by open circuit
    requests_blocked: CounterVec,
    /// Total number of requests allowed through
    requests_allowed: CounterVec,
}

impl CircuitBreakerMetrics {
    /// Create and register metrics with the provided registry
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let state_gauge = IntGaugeVec::new(
            Opts::new(
                "circuit_breaker_state",
                "Current state of circuit breaker (0=closed, 1=open, 2=half_open)",
            )
            .namespace("kiro")
            .subsystem("circuit_breaker"),
            &["provider"],
        )?;

        let trips_total = CounterVec::new(
            Opts::new(
                "circuit_breaker_trips_total",
                "Total number of times circuit breaker tripped to open state",
            )
            .namespace("kiro")
            .subsystem("circuit_breaker"),
            &["provider"],
        )?;

        let requests_blocked = CounterVec::new(
            Opts::new(
                "circuit_breaker_requests_blocked_total",
                "Total number of requests blocked by open circuit",
            )
            .namespace("kiro")
            .subsystem("circuit_breaker"),
            &["provider"],
        )?;

        let requests_allowed = CounterVec::new(
            Opts::new(
                "circuit_breaker_requests_allowed_total",
                "Total number of requests allowed through circuit breaker",
            )
            .namespace("kiro")
            .subsystem("circuit_breaker"),
            &["provider"],
        )?;

        registry.register(Box::new(state_gauge.clone()))?;
        registry.register(Box::new(trips_total.clone()))?;
        registry.register(Box::new(requests_blocked.clone()))?;
        registry.register(Box::new(requests_allowed.clone()))?;

        Ok(Self {
            state_gauge,
            trips_total,
            requests_blocked,
            requests_allowed,
        })
    }

    fn set_state(&self, provider: &str, state: CircuitBreakerStateEnum) {
        let value = match state {
            CircuitBreakerStateEnum::Closed => 0,
            CircuitBreakerStateEnum::Open => 1,
            CircuitBreakerStateEnum::HalfOpen => 2,
        };
        self.state_gauge.with_label_values(&[provider]).set(value);
    }

    fn record_trip(&self, provider: &str) {
        self.trips_total.with_label_values(&[provider]).inc();
    }

    fn record_blocked(&self, provider: &str) {
        self.requests_blocked.with_label_values(&[provider]).inc();
    }

    fn record_allowed(&self, provider: &str) {
        self.requests_allowed.with_label_values(&[provider]).inc();
    }
}

/// Circuit breaker service for provider APIs
pub struct CircuitBreakerService {
    config: CircuitBreakerConfig,
    circuits: Arc<RwLock<HashMap<String, CircuitState>>>,
    metrics: Option<CircuitBreakerMetrics>,
}

impl CircuitBreakerService {
    /// Create a new circuit breaker service with default configuration
    pub fn new() -> Self {
        Self {
            config: CircuitBreakerConfig::default(),
            circuits: Arc::new(RwLock::new(HashMap::new())),
            metrics: None,
        }
    }

    /// Create a new circuit breaker service with custom configuration
    pub fn with_config(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            circuits: Arc::new(RwLock::new(HashMap::new())),
            metrics: None,
        }
    }

    /// Register metrics with a Prometheus registry
    pub fn with_metrics(mut self, registry: &Registry) -> Result<Self, prometheus::Error> {
        self.metrics = Some(CircuitBreakerMetrics::new(registry)?);
        Ok(self)
    }

    /// Get the current state of a circuit
    pub async fn get_state(&self, provider: &str) -> CircuitBreakerStateEnum {
        let circuits = self.circuits.read().await;
        circuits
            .get(provider)
            .map(|c| c.state)
            .unwrap_or(CircuitBreakerStateEnum::Closed)
    }

    /// Check if a request can proceed (and perform state transitions if needed)
    pub async fn can_proceed(&self, provider: &str) -> bool {
        let mut circuits = self.circuits.write().await;
        let circuit = circuits
            .entry(provider.to_string())
            .or_insert_with(CircuitState::default);

        let now = Instant::now();

        match circuit.state {
            CircuitBreakerStateEnum::Closed => {
                if let Some(metrics) = &self.metrics {
                    metrics.record_allowed(provider);
                }
                true
            }
            CircuitBreakerStateEnum::Open => {
                // Check if we should transition to half-open
                if let Some(opened_at) = circuit.opened_at {
                    if now.duration_since(opened_at).as_secs() >= self.config.open_timeout_seconds {
                        info!(
                            provider = provider,
                            "Circuit breaker transitioning from Open to HalfOpen"
                        );
                        circuit.state = CircuitBreakerStateEnum::HalfOpen;
                        circuit.half_open_successes = 0;
                        circuit.last_half_open_test = Some(now);

                        if let Some(metrics) = &self.metrics {
                            metrics.set_state(provider, CircuitBreakerStateEnum::HalfOpen);
                            metrics.record_allowed(provider);
                        }
                        return true;
                    }
                }

                if let Some(metrics) = &self.metrics {
                    metrics.record_blocked(provider);
                }
                false
            }
            CircuitBreakerStateEnum::HalfOpen => {
                // Allow one test request every half_open_test_interval_seconds
                let should_allow = circuit.last_half_open_test.map_or(true, |last| {
                    now.duration_since(last).as_secs()
                        >= self.config.half_open_test_interval_seconds
                });

                if should_allow {
                    circuit.last_half_open_test = Some(now);
                    if let Some(metrics) = &self.metrics {
                        metrics.record_allowed(provider);
                    }
                    true
                } else {
                    if let Some(metrics) = &self.metrics {
                        metrics.record_blocked(provider);
                    }
                    false
                }
            }
        }
    }

    /// Record a successful request
    pub async fn record_success(&self, provider: &str) {
        let mut circuits = self.circuits.write().await;
        let circuit = circuits
            .entry(provider.to_string())
            .or_insert_with(CircuitState::default);

        match circuit.state {
            CircuitBreakerStateEnum::Closed => {
                // Clear old failures
                circuit.failure_timestamps.clear();
            }
            CircuitBreakerStateEnum::HalfOpen => {
                circuit.half_open_successes += 1;
                info!(
                    provider = provider,
                    successes = circuit.half_open_successes,
                    threshold = self.config.half_open_success_threshold,
                    "Circuit breaker recorded success in HalfOpen state"
                );

                if circuit.half_open_successes >= self.config.half_open_success_threshold {
                    info!(
                        provider = provider,
                        "Circuit breaker transitioning from HalfOpen to Closed"
                    );
                    circuit.state = CircuitBreakerStateEnum::Closed;
                    circuit.failure_timestamps.clear();
                    circuit.opened_at = None;
                    circuit.last_half_open_test = None;
                    circuit.half_open_successes = 0;

                    if let Some(metrics) = &self.metrics {
                        metrics.set_state(provider, CircuitBreakerStateEnum::Closed);
                    }
                }
            }
            CircuitBreakerStateEnum::Open => {
                // Shouldn't happen, but reset just in case
                circuit.state = CircuitBreakerStateEnum::Closed;
                circuit.failure_timestamps.clear();
                circuit.opened_at = None;

                if let Some(metrics) = &self.metrics {
                    metrics.set_state(provider, CircuitBreakerStateEnum::Closed);
                }
            }
        }
    }

    /// Record a failed request
    pub async fn record_failure(&self, provider: &str) {
        let mut circuits = self.circuits.write().await;
        let circuit = circuits
            .entry(provider.to_string())
            .or_insert_with(CircuitState::default);

        let now = Instant::now();

        match circuit.state {
            CircuitBreakerStateEnum::Closed => {
                // Add this failure
                circuit.failure_timestamps.push(now);

                // Remove failures outside the window
                let window = Duration::from_secs(self.config.failure_window_seconds);
                circuit
                    .failure_timestamps
                    .retain(|&ts| now.duration_since(ts) < window);

                // Check if we should open the circuit
                if circuit.failure_timestamps.len() >= self.config.failure_threshold as usize {
                    warn!(
                        provider = provider,
                        failures = circuit.failure_timestamps.len(),
                        "Circuit breaker tripping to Open state"
                    );
                    circuit.state = CircuitBreakerStateEnum::Open;
                    circuit.opened_at = Some(now);

                    if let Some(metrics) = &self.metrics {
                        metrics.set_state(provider, CircuitBreakerStateEnum::Open);
                        metrics.record_trip(provider);
                    }
                }
            }
            CircuitBreakerStateEnum::HalfOpen => {
                warn!(
                    provider = provider,
                    "Circuit breaker transitioning from HalfOpen to Open (failure in test request)"
                );
                circuit.state = CircuitBreakerStateEnum::Open;
                circuit.opened_at = Some(now);
                circuit.half_open_successes = 0;

                if let Some(metrics) = &self.metrics {
                    metrics.set_state(provider, CircuitBreakerStateEnum::Open);
                    metrics.record_trip(provider);
                }
            }
            CircuitBreakerStateEnum::Open => {
                // Already open, just update the timestamp
                circuit.opened_at = Some(now);
            }
        }
    }

    /// Execute a function with circuit breaker protection using typed provider
    ///
    /// Returns `ProviderUnavailable` error if the circuit is open.
    pub async fn execute_typed<F, Fut, T>(
        &self,
        provider_type: OAuthProviderType,
        operation: F,
    ) -> Result<T, AppError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, AppError>>,
    {
        let provider = provider_type.to_string();

        // Check if we can proceed
        if !self.can_proceed(&provider).await {
            return Err(AppError::OAuth(OAuthError::ProviderUnavailable {
                provider: provider_type,
                reason: "Circuit breaker is open - provider temporarily unavailable".to_string(),
                estimated_recovery: None,
                retry_after: Some(self.config.open_timeout_seconds),
            }));
        }

        // Execute the operation
        match operation().await {
            Ok(result) => {
                self.record_success(&provider).await;
                Ok(result)
            }
            Err(err) => {
                // Only count certain errors as circuit breaker failures
                if Self::is_transient_error(&err) {
                    self.record_failure(&provider).await;
                }
                Err(err)
            }
        }
    }

    /// Execute a function with circuit breaker protection using string provider name
    ///
    /// Returns `ExternalServiceUnavailable` error if the circuit is open.
    /// Use this for providers that don't have an OAuthProviderType variant.
    pub async fn execute<F, Fut, T>(&self, provider: &str, operation: F) -> Result<T, AppError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, AppError>>,
    {
        // Check if we can proceed
        if !self.can_proceed(provider).await {
            return Err(AppError::ExternalServiceUnavailable {
                service: format!(
                    "{} (circuit breaker open - retry after {} seconds)",
                    provider, self.config.open_timeout_seconds
                ),
            });
        }

        // Execute the operation
        match operation().await {
            Ok(result) => {
                self.record_success(provider).await;
                Ok(result)
            }
            Err(err) => {
                // Only count certain errors as circuit breaker failures
                if Self::is_transient_error(&err) {
                    self.record_failure(provider).await;
                }
                Err(err)
            }
        }
    }

    /// Determine if an error should count toward the circuit breaker threshold
    fn is_transient_error(err: &AppError) -> bool {
        matches!(
            err,
            AppError::ExternalServiceUnavailable { .. }
                | AppError::ExternalServiceError(_)
                | AppError::ServiceUnavailable
                | AppError::OAuth(OAuthError::ProviderUnavailable { .. })
                | AppError::OAuth(OAuthError::NetworkError { .. })
                | AppError::OAuth(OAuthError::ApiTimeout { .. })
                | AppError::OAuth(OAuthError::RateLimitExceeded { .. })
        )
    }

    /// Execute with circuit breaker, converting anyhow errors to AppError
    pub async fn execute_anyhow<F, Fut, T>(&self, provider: &str, operation: F) -> anyhow::Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = anyhow::Result<T>>,
    {
        // Check if we can proceed
        if !self.can_proceed(provider).await {
            return Err(anyhow::anyhow!(
                "Circuit breaker is open for provider '{}' - temporarily unavailable",
                provider
            ));
        }

        // Execute the operation
        match operation().await {
            Ok(result) => {
                self.record_success(provider).await;
                Ok(result)
            }
            Err(err) => {
                // Check if this looks like a transient error
                let err_str = err.to_string().to_lowercase();
                let is_transient = err_str.contains("timeout")
                    || err_str.contains("connection")
                    || err_str.contains("unavailable")
                    || err_str.contains("503")
                    || err_str.contains("502")
                    || err_str.contains("429")
                    || err_str.contains("rate limit");

                if is_transient {
                    self.record_failure(provider).await;
                }
                Err(err)
            }
        }
    }

    /// Execute with circuit breaker, returning CircuitBreakerError
    ///
    /// This method is useful when handlers want to explicitly handle circuit breaker errors
    /// separately from execution errors.
    pub async fn execute_with_cb_error<F, Fut, T>(
        &self,
        provider: &str,
        operation: F,
    ) -> Result<T, CircuitBreakerError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = anyhow::Result<T>>,
    {
        // Check if we can proceed
        if !self.can_proceed(provider).await {
            return Err(CircuitBreakerError::CircuitOpen {
                provider: provider.to_string(),
            });
        }

        // Execute the operation
        match operation().await {
            Ok(result) => {
                self.record_success(provider).await;
                Ok(result)
            }
            Err(err) => {
                // Check if this looks like a transient error
                let err_str = err.to_string().to_lowercase();
                let is_transient = err_str.contains("timeout")
                    || err_str.contains("connection")
                    || err_str.contains("unavailable")
                    || err_str.contains("503")
                    || err_str.contains("502")
                    || err_str.contains("429")
                    || err_str.contains("rate limit");

                if is_transient {
                    self.record_failure(provider).await;
                }
                Err(CircuitBreakerError::ExecutionFailed(err))
            }
        }
    }

    /// Reset a circuit to closed state (for testing or manual recovery)
    pub async fn reset(&self, provider: &str) {
        let mut circuits = self.circuits.write().await;
        if let Some(circuit) = circuits.get_mut(provider) {
            info!(
                provider = provider,
                "Circuit breaker manually reset to Closed"
            );
            circuit.state = CircuitBreakerStateEnum::Closed;
            circuit.failure_timestamps.clear();
            circuit.opened_at = None;
            circuit.last_half_open_test = None;
            circuit.half_open_successes = 0;

            if let Some(metrics) = &self.metrics {
                metrics.set_state(provider, CircuitBreakerStateEnum::Closed);
            }
        }
    }
}

impl Default for CircuitBreakerService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_starts_closed() {
        let cb = CircuitBreakerService::new();
        assert_eq!(
            cb.get_state("spotify").await,
            CircuitBreakerStateEnum::Closed
        );
        assert!(cb.can_proceed("spotify").await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 5,
            failure_window_seconds: 60,
            open_timeout_seconds: 30,
            half_open_success_threshold: 3,
            half_open_test_interval_seconds: 30,
        };
        let cb = CircuitBreakerService::with_config(config);

        // Record 5 failures
        for _ in 0..5 {
            cb.record_failure("spotify").await;
        }

        assert_eq!(cb.get_state("spotify").await, CircuitBreakerStateEnum::Open);
        assert!(!cb.can_proceed("spotify").await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_success_clears_failures() {
        let cb = CircuitBreakerService::new();

        // Record some failures but not enough to trip
        for _ in 0..3 {
            cb.record_failure("spotify").await;
        }

        // Record success
        cb.record_success("spotify").await;

        // More failures should not immediately trip circuit
        for _ in 0..3 {
            cb.record_failure("spotify").await;
        }

        // Should still be closed (success cleared previous failures)
        assert_eq!(
            cb.get_state("spotify").await,
            CircuitBreakerStateEnum::Closed
        );
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_success_closes() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            failure_window_seconds: 60,
            open_timeout_seconds: 0, // Immediate transition for testing
            half_open_success_threshold: 2,
            half_open_test_interval_seconds: 0, // Immediate for testing
        };
        let cb = CircuitBreakerService::with_config(config);

        // Trip the circuit
        cb.record_failure("spotify").await;
        cb.record_failure("spotify").await;
        assert_eq!(cb.get_state("spotify").await, CircuitBreakerStateEnum::Open);

        // Can proceed should transition to half-open (since open_timeout is 0)
        assert!(cb.can_proceed("spotify").await);
        assert_eq!(
            cb.get_state("spotify").await,
            CircuitBreakerStateEnum::HalfOpen
        );

        // Record successes to close
        cb.record_success("spotify").await;
        cb.record_success("spotify").await;
        assert_eq!(
            cb.get_state("spotify").await,
            CircuitBreakerStateEnum::Closed
        );
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_failure_reopens() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            failure_window_seconds: 60,
            open_timeout_seconds: 0,
            half_open_success_threshold: 3,
            half_open_test_interval_seconds: 0,
        };
        let cb = CircuitBreakerService::with_config(config);

        // Trip the circuit
        cb.record_failure("spotify").await;
        cb.record_failure("spotify").await;

        // Transition to half-open
        cb.can_proceed("spotify").await;
        assert_eq!(
            cb.get_state("spotify").await,
            CircuitBreakerStateEnum::HalfOpen
        );

        // Failure in half-open reopens
        cb.record_failure("spotify").await;
        assert_eq!(cb.get_state("spotify").await, CircuitBreakerStateEnum::Open);
    }

    #[tokio::test]
    async fn test_circuit_breaker_independent_providers() {
        let cb = CircuitBreakerService::new();

        // Trip spotify circuit
        for _ in 0..5 {
            cb.record_failure("spotify").await;
        }

        assert_eq!(cb.get_state("spotify").await, CircuitBreakerStateEnum::Open);
        assert_eq!(
            cb.get_state("apple_music").await,
            CircuitBreakerStateEnum::Closed
        );

        assert!(!cb.can_proceed("spotify").await);
        assert!(cb.can_proceed("apple_music").await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_reset() {
        let cb = CircuitBreakerService::new();

        // Trip the circuit
        for _ in 0..5 {
            cb.record_failure("spotify").await;
        }
        assert_eq!(cb.get_state("spotify").await, CircuitBreakerStateEnum::Open);

        // Reset
        cb.reset("spotify").await;
        assert_eq!(
            cb.get_state("spotify").await,
            CircuitBreakerStateEnum::Closed
        );
        assert!(cb.can_proceed("spotify").await);
    }
}
