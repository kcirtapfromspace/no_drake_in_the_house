//! Error recovery mechanisms for database and Redis failures

use crate::error::{AppError, Result};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

/// Configuration for retry policies
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        }
    }
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker for external services
#[derive(Debug)]
pub struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_count: u32,
    failure_threshold: u32,
    recovery_timeout: Duration,
    last_failure_time: Option<std::time::Instant>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            failure_threshold,
            recovery_timeout,
            last_failure_time: None,
        }
    }

    pub fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.recovery_timeout {
                        self.state = CircuitBreakerState::HalfOpen;
                        info!("Circuit breaker transitioning to half-open state");
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    pub fn record_success(&mut self) {
        match self.state {
            CircuitBreakerState::HalfOpen => {
                self.state = CircuitBreakerState::Closed;
                self.failure_count = 0;
                self.last_failure_time = None;
                info!("Circuit breaker closed after successful recovery");
            }
            CircuitBreakerState::Closed => {
                self.failure_count = 0;
            }
            _ => {}
        }
    }

    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(std::time::Instant::now());

        match self.state {
            CircuitBreakerState::Closed => {
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitBreakerState::Open;
                    warn!(
                        failure_count = self.failure_count,
                        threshold = self.failure_threshold,
                        "Circuit breaker opened due to failures"
                    );
                }
            }
            CircuitBreakerState::HalfOpen => {
                self.state = CircuitBreakerState::Open;
                warn!("Circuit breaker reopened after failed recovery attempt");
            }
            _ => {}
        }
    }

    pub fn state(&self) -> &CircuitBreakerState {
        &self.state
    }
}

/// Retry a database operation with exponential backoff
pub async fn retry_database_operation<F, Fut, T>(
    operation: F,
    config: RetryConfig,
    operation_name: &str,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut delay = config.initial_delay;
    let mut last_error = None;

    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    info!(
                        operation = operation_name,
                        attempt = attempt,
                        "Database operation succeeded after retry"
                    );
                }
                return Ok(result);
            }
            Err(err) => {
                last_error = Some(err);
                
                if attempt < config.max_attempts {
                    warn!(
                        operation = operation_name,
                        attempt = attempt,
                        max_attempts = config.max_attempts,
                        delay_ms = delay.as_millis(),
                        error = %last_error.as_ref().unwrap(),
                        "Database operation failed, retrying"
                    );
                    
                    sleep(delay).await;
                    delay = std::cmp::min(
                        Duration::from_millis((delay.as_millis() as f64 * config.backoff_multiplier) as u64),
                        config.max_delay,
                    );
                } else {
                    error!(
                        operation = operation_name,
                        attempt = attempt,
                        error = %last_error.as_ref().unwrap(),
                        "Database operation failed after all retry attempts"
                    );
                }
            }
        }
    }

    Err(last_error.unwrap_or(AppError::DatabaseConnectionFailed))
}

/// Retry a Redis operation with exponential backoff
pub async fn retry_redis_operation<F, Fut, T>(
    operation: F,
    config: RetryConfig,
    operation_name: &str,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut delay = config.initial_delay;
    let mut last_error = None;

    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    info!(
                        operation = operation_name,
                        attempt = attempt,
                        "Redis operation succeeded after retry"
                    );
                }
                return Ok(result);
            }
            Err(err) => {
                last_error = Some(err);
                
                if attempt < config.max_attempts {
                    warn!(
                        operation = operation_name,
                        attempt = attempt,
                        max_attempts = config.max_attempts,
                        delay_ms = delay.as_millis(),
                        error = %last_error.as_ref().unwrap(),
                        "Redis operation failed, retrying"
                    );
                    
                    sleep(delay).await;
                    delay = std::cmp::min(
                        Duration::from_millis((delay.as_millis() as f64 * config.backoff_multiplier) as u64),
                        config.max_delay,
                    );
                } else {
                    error!(
                        operation = operation_name,
                        attempt = attempt,
                        error = %last_error.as_ref().unwrap(),
                        "Redis operation failed after all retry attempts"
                    );
                }
            }
        }
    }

    Err(last_error.unwrap_or(AppError::RedisConnectionFailed))
}

/// Execute an operation with circuit breaker protection
pub async fn with_circuit_breaker<F, Fut, T>(
    circuit_breaker: &mut CircuitBreaker,
    operation: F,
    service_name: &str,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    if !circuit_breaker.can_execute() {
        warn!(
            service = service_name,
            state = ?circuit_breaker.state(),
            "Circuit breaker is open, rejecting request"
        );
        return Err(AppError::ExternalServiceUnavailable {
            service: service_name.to_string(),
        });
    }

    match operation().await {
        Ok(result) => {
            circuit_breaker.record_success();
            Ok(result)
        }
        Err(err) => {
            circuit_breaker.record_failure();
            Err(err)
        }
    }
}

/// Health check with recovery for database connections
pub async fn database_health_check_with_recovery(
    pool: &sqlx::PgPool,
) -> Result<()> {
    retry_database_operation(
        || async {
            sqlx::query("SELECT 1")
                .execute(pool)
                .await
                .map_err(AppError::from)?;
            Ok(())
        },
        RetryConfig::default(),
        "database_health_check",
    ).await
}

/// Health check with recovery for Redis connections
pub async fn redis_health_check_with_recovery(
    redis_pool: &deadpool_redis::Pool,
) -> Result<()> {
    retry_redis_operation(
        || async {
            let mut conn = redis_pool
                .get()
                .await
                .map_err(|e| AppError::RedisOperationFailed(e.to_string()))?;
            
            redis::cmd("PING")
                .query_async::<_, String>(&mut conn)
                .await
                .map_err(|e| AppError::RedisOperationFailed(e.to_string()))?;
            
            Ok(())
        },
        RetryConfig::default(),
        "redis_health_check",
    ).await
}

/// Graceful degradation helper for optional features
pub async fn with_graceful_degradation<F, Fut, T>(
    operation: F,
    fallback_value: T,
    feature_name: &str,
) -> T
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    match operation().await {
        Ok(result) => result,
        Err(err) => {
            warn!(
                feature = feature_name,
                error = %err,
                "Feature failed, using fallback value"
            );
            fallback_value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success_after_failure() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            backoff_multiplier: 2.0,
        };

        let result = retry_database_operation(
            || {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err(AppError::DatabaseConnectionFailed)
                    } else {
                        Ok("success")
                    }
                }
            },
            config,
            "test_operation",
        ).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_exhaustion() {
        let config = RetryConfig {
            max_attempts: 2,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            backoff_multiplier: 2.0,
        };

        let result: Result<&str> = retry_database_operation(
            || async { Err(AppError::DatabaseConnectionFailed) },
            config,
            "test_operation",
        ).await;

        assert!(result.is_err());
    }

    #[test]
    fn test_circuit_breaker_states() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(100));

        // Initially closed
        assert_eq!(cb.state(), &CircuitBreakerState::Closed);
        assert!(cb.can_execute());

        // Record failures
        cb.record_failure();
        assert_eq!(cb.state(), &CircuitBreakerState::Closed);
        assert!(cb.can_execute());

        cb.record_failure();
        assert_eq!(cb.state(), &CircuitBreakerState::Open);
        assert!(!cb.can_execute());

        // Record success in half-open state
        std::thread::sleep(Duration::from_millis(101));
        assert!(cb.can_execute()); // Should transition to half-open
        assert_eq!(cb.state(), &CircuitBreakerState::HalfOpen);

        cb.record_success();
        assert_eq!(cb.state(), &CircuitBreakerState::Closed);
    }
}