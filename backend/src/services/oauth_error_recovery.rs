//! OAuth error recovery strategies and retry logic

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::error::oauth::OAuthError;
use crate::models::oauth::OAuthProviderType;
use crate::{AppError, Result};

/// Configuration for OAuth error recovery
#[derive(Debug, Clone)]
pub struct OAuthErrorRecoveryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Base delay for exponential backoff (in seconds)
    pub base_delay: u64,
    /// Maximum delay between retries (in seconds)
    pub max_delay: u64,
    /// Jitter factor for randomizing delays (0.0 to 1.0)
    pub jitter_factor: f64,
    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: u32,
    /// Circuit breaker recovery timeout (in seconds)
    pub circuit_breaker_timeout: u64,
    /// Enable automatic fallback to alternative providers
    pub enable_provider_fallback: bool,
}

impl Default for OAuthErrorRecoveryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: 1,
            max_delay: 300, // 5 minutes
            jitter_factor: 0.1,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout: 300,    // 5 minutes
            enable_provider_fallback: false, // Disabled by default for security
        }
    }
}

/// Circuit breaker state for OAuth providers
#[derive(Debug, Clone)]
enum CircuitBreakerState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen,
}

/// Circuit breaker for OAuth providers
#[derive(Debug)]
struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_count: u32,
    config: OAuthErrorRecoveryConfig,
}

impl CircuitBreaker {
    fn new(config: OAuthErrorRecoveryConfig) -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            config,
        }
    }

    fn can_execute(&mut self) -> bool {
        match &self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open { opened_at } => {
                if opened_at.elapsed().as_secs() >= self.config.circuit_breaker_timeout {
                    self.state = CircuitBreakerState::HalfOpen;
                    true
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitBreakerState::Closed;
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;

        match &self.state {
            CircuitBreakerState::Closed => {
                if self.failure_count >= self.config.circuit_breaker_threshold {
                    self.state = CircuitBreakerState::Open {
                        opened_at: Instant::now(),
                    };
                    warn!(
                        failure_count = self.failure_count,
                        threshold = self.config.circuit_breaker_threshold,
                        "OAuth circuit breaker opened"
                    );
                }
            }
            CircuitBreakerState::HalfOpen => {
                self.state = CircuitBreakerState::Open {
                    opened_at: Instant::now(),
                };
                warn!("OAuth circuit breaker re-opened after half-open failure");
            }
            CircuitBreakerState::Open { .. } => {
                // Already open, just increment counter
            }
        }
    }

    fn is_open(&self) -> bool {
        matches!(self.state, CircuitBreakerState::Open { .. })
    }
}

/// OAuth error recovery service
pub struct OAuthErrorRecoveryService {
    config: OAuthErrorRecoveryConfig,
    circuit_breakers: Arc<tokio::sync::RwLock<HashMap<OAuthProviderType, CircuitBreaker>>>,
    security_monitor: Arc<OAuthSecurityMonitor>,
}

impl OAuthErrorRecoveryService {
    /// Create a new OAuth error recovery service
    pub fn new(config: OAuthErrorRecoveryConfig) -> Self {
        Self {
            config: config.clone(),
            circuit_breakers: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            security_monitor: Arc::new(OAuthSecurityMonitor::new(config)),
        }
    }

    /// Execute an OAuth operation with automatic retry and error recovery
    pub async fn execute_with_recovery<F, T>(
        &self,
        provider: OAuthProviderType,
        operation_name: &str,
        operation: F,
    ) -> Result<T>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>
            + Send
            + Sync,
        T: Send,
    {
        let mut attempt = 0;
        let mut last_error: Option<AppError> = None;

        while attempt <= self.config.max_retries {
            // Check circuit breaker
            if !self.can_execute_operation(&provider).await {
                return Err(AppError::OAuth(OAuthError::ProviderUnavailable {
                    provider,
                    reason: "Circuit breaker is open due to repeated failures".to_string(),
                    estimated_recovery: Some(
                        chrono::Utc::now()
                            + chrono::Duration::seconds(self.config.circuit_breaker_timeout as i64),
                    ),
                    retry_after: Some(self.config.circuit_breaker_timeout),
                }));
            }

            // Execute the operation
            match operation().await {
                Ok(result) => {
                    // Record success
                    self.record_success(&provider).await;

                    if attempt > 0 {
                        info!(
                            provider = %provider,
                            operation = operation_name,
                            attempt = attempt + 1,
                            "OAuth operation succeeded after retry"
                        );
                    }

                    return Ok(result);
                }
                Err(error) => {
                    attempt += 1;
                    // Store error without cloning (since AppError doesn't implement Clone)
                    // We'll create a new error for storage
                    last_error = Some(AppError::Internal {
                        message: Some(format!("OAuth operation failed: {}", error)),
                    });

                    // Check if this is an OAuth error that we can handle
                    if let AppError::OAuth(oauth_error) = &error {
                        // Record security violations
                        self.security_monitor.record_error(oauth_error).await;

                        // Check if error is retryable
                        if !oauth_error.is_retryable() || attempt > self.config.max_retries {
                            self.record_failure(&provider).await;
                            return Err(error);
                        }

                        // Calculate retry delay
                        let delay = self.calculate_retry_delay(attempt, oauth_error);

                        warn!(
                            provider = %provider,
                            operation = operation_name,
                            attempt = attempt,
                            max_retries = self.config.max_retries,
                            delay_seconds = delay,
                            error = %oauth_error,
                            "OAuth operation failed, retrying"
                        );

                        // Wait before retry
                        sleep(Duration::from_secs(delay)).await;
                    } else {
                        // Non-OAuth error, don't retry
                        self.record_failure(&provider).await;
                        return Err(error);
                    }
                }
            }
        }

        // All retries exhausted
        self.record_failure(&provider).await;

        error!(
            provider = %provider,
            operation = operation_name,
            attempts = attempt,
            "OAuth operation failed after all retry attempts"
        );

        Err(last_error.unwrap_or_else(|| {
            AppError::OAuth(OAuthError::ProviderUnavailable {
                provider,
                reason: "Maximum retry attempts exceeded".to_string(),
                estimated_recovery: None,
                retry_after: Some(300),
            })
        }))
    }

    /// Check if an operation can be executed (circuit breaker check)
    async fn can_execute_operation(&self, provider: &OAuthProviderType) -> bool {
        let mut circuit_breakers = self.circuit_breakers.write().await;
        let circuit_breaker = circuit_breakers
            .entry(provider.clone())
            .or_insert_with(|| CircuitBreaker::new(self.config.clone()));

        circuit_breaker.can_execute()
    }

    /// Record a successful operation
    async fn record_success(&self, provider: &OAuthProviderType) {
        let mut circuit_breakers = self.circuit_breakers.write().await;
        if let Some(circuit_breaker) = circuit_breakers.get_mut(provider) {
            circuit_breaker.record_success();
        }
    }

    /// Record a failed operation
    async fn record_failure(&self, provider: &OAuthProviderType) {
        let mut circuit_breakers = self.circuit_breakers.write().await;
        let circuit_breaker = circuit_breakers
            .entry(provider.clone())
            .or_insert_with(|| CircuitBreaker::new(self.config.clone()));

        circuit_breaker.record_failure();
    }

    /// Calculate retry delay with exponential backoff and jitter
    fn calculate_retry_delay(&self, attempt: u32, oauth_error: &OAuthError) -> u64 {
        // Use provider-specific retry delay if available
        if let Some(provider_delay) = oauth_error.retry_delay() {
            return provider_delay;
        }

        // Calculate exponential backoff
        let base_delay = self.config.base_delay;
        let exponential_delay = base_delay * 2_u64.pow(attempt.saturating_sub(1));
        let capped_delay = std::cmp::min(exponential_delay, self.config.max_delay);

        // Add jitter to prevent thundering herd
        let jitter =
            (capped_delay as f64 * self.config.jitter_factor * rand::random::<f64>()) as u64;

        capped_delay + jitter
    }

    /// Get circuit breaker status for a provider
    pub async fn get_circuit_breaker_status(&self, provider: &OAuthProviderType) -> Option<bool> {
        let circuit_breakers = self.circuit_breakers.read().await;
        circuit_breakers.get(provider).map(|cb| cb.is_open())
    }

    /// Get provider health status
    pub async fn get_provider_health(&self, provider: &OAuthProviderType) -> ProviderHealthStatus {
        let is_circuit_open = self
            .get_circuit_breaker_status(provider)
            .await
            .unwrap_or(false);
        let security_violations = self.security_monitor.get_recent_violations(provider).await;

        if is_circuit_open {
            ProviderHealthStatus::Unavailable {
                reason: "Circuit breaker is open".to_string(),
                estimated_recovery: Some(
                    chrono::Utc::now()
                        + chrono::Duration::seconds(self.config.circuit_breaker_timeout as i64),
                ),
            }
        } else if security_violations > 10 {
            ProviderHealthStatus::Degraded {
                reason: format!(
                    "High number of security violations: {}",
                    security_violations
                ),
            }
        } else {
            ProviderHealthStatus::Healthy
        }
    }

    /// Provide user guidance for resolving OAuth issues
    pub fn get_user_guidance(&self, oauth_error: &OAuthError) -> UserGuidance {
        match oauth_error {
            OAuthError::ProviderNotConfigured { provider, .. } => UserGuidance {
                title: format!("{} Authentication Not Available", provider),
                message: format!(
                    "{} authentication is not configured on this server.",
                    provider
                ),
                actions: vec![
                    "Contact your system administrator to enable this authentication method"
                        .to_string(),
                    "Try using a different authentication method".to_string(),
                ],
                is_user_actionable: false,
                contact_support: true,
            },
            OAuthError::InvalidConfiguration { provider, .. } => UserGuidance {
                title: format!("{} Authentication Error", provider),
                message: format!(
                    "{} authentication is temporarily unavailable due to a configuration issue.",
                    provider
                ),
                actions: vec![
                    "Try again in a few minutes".to_string(),
                    "Contact support if the problem persists".to_string(),
                ],
                is_user_actionable: false,
                contact_support: true,
            },
            OAuthError::StateValidationFailed { .. } => UserGuidance {
                title: "Authentication Security Error".to_string(),
                message: "The authentication request is invalid or has expired.".to_string(),
                actions: vec![
                    "Close this window and try signing in again".to_string(),
                    "Clear your browser cookies and try again".to_string(),
                    "Make sure you're not using an old or bookmarked authentication link"
                        .to_string(),
                ],
                is_user_actionable: true,
                contact_support: false,
            },
            OAuthError::TokenRefreshFailed {
                provider,
                requires_reauth,
                ..
            } => {
                if *requires_reauth {
                    UserGuidance {
                        title: format!("{} Authentication Expired", provider),
                        message: format!(
                            "Your {} authentication has expired and needs to be renewed.",
                            provider
                        ),
                        actions: vec![
                            format!("Click 'Sign in with {}' to re-authenticate", provider),
                            "You may need to grant permissions again".to_string(),
                        ],
                        is_user_actionable: true,
                        contact_support: false,
                    }
                } else {
                    UserGuidance {
                        title: format!("{} Authentication Error", provider),
                        message: "There was a temporary issue refreshing your authentication."
                            .to_string(),
                        actions: vec![
                            "Try the action again".to_string(),
                            "If the problem persists, sign out and sign in again".to_string(),
                        ],
                        is_user_actionable: true,
                        contact_support: false,
                    }
                }
            }
            OAuthError::InsufficientScopes {
                provider,
                required_scopes,
                ..
            } => UserGuidance {
                title: format!("{} Permissions Required", provider),
                message: format!(
                    "Additional permissions are needed to complete this action with {}.",
                    provider
                ),
                actions: vec![
                    format!(
                        "Re-authorize with {} to grant the required permissions",
                        provider
                    ),
                    format!("Required permissions: {}", required_scopes.join(", ")),
                ],
                is_user_actionable: true,
                contact_support: false,
            },
            OAuthError::RateLimitExceeded {
                provider,
                retry_after,
                ..
            } => UserGuidance {
                title: format!("{} Rate Limit", provider),
                message: format!(
                    "Too many requests to {}. Please wait before trying again.",
                    provider
                ),
                actions: vec![
                    format!("Wait {} seconds before trying again", retry_after),
                    "Reduce the frequency of your requests".to_string(),
                ],
                is_user_actionable: true,
                contact_support: false,
            },
            OAuthError::ProviderUnavailable {
                provider,
                estimated_recovery,
                ..
            } => {
                let recovery_message = if let Some(recovery_time) = estimated_recovery {
                    format!(
                        "Expected to be available again around {}",
                        recovery_time.format("%H:%M")
                    )
                } else {
                    "Please try again later".to_string()
                };

                UserGuidance {
                    title: format!("{} Temporarily Unavailable", provider),
                    message: format!("{} authentication is temporarily unavailable.", provider),
                    actions: vec![
                        recovery_message,
                        "Try using a different authentication method if available".to_string(),
                    ],
                    is_user_actionable: false,
                    contact_support: false,
                }
            }
            OAuthError::NetworkError { provider, .. } => UserGuidance {
                title: "Connection Error".to_string(),
                message: format!("Unable to connect to {} for authentication.", provider),
                actions: vec![
                    "Check your internet connection".to_string(),
                    "Try again in a few moments".to_string(),
                    "Contact support if the problem persists".to_string(),
                ],
                is_user_actionable: true,
                contact_support: false,
            },
            _ => UserGuidance {
                title: "Authentication Error".to_string(),
                message: "An unexpected authentication error occurred.".to_string(),
                actions: vec![
                    "Try the authentication process again".to_string(),
                    "Contact support if the problem persists".to_string(),
                ],
                is_user_actionable: true,
                contact_support: true,
            },
        }
    }
}

/// Provider health status
#[derive(Debug, Clone)]
pub enum ProviderHealthStatus {
    Healthy,
    Degraded {
        reason: String,
    },
    Unavailable {
        reason: String,
        estimated_recovery: Option<chrono::DateTime<chrono::Utc>>,
    },
}

/// User guidance for resolving OAuth issues
#[derive(Debug, Clone)]
pub struct UserGuidance {
    pub title: String,
    pub message: String,
    pub actions: Vec<String>,
    pub is_user_actionable: bool,
    pub contact_support: bool,
}

/// OAuth security monitoring
struct OAuthSecurityMonitor {
    config: OAuthErrorRecoveryConfig,
    violation_counts: Arc<tokio::sync::RwLock<HashMap<OAuthProviderType, u32>>>,
    last_reset: Arc<tokio::sync::RwLock<Instant>>,
}

impl OAuthSecurityMonitor {
    fn new(config: OAuthErrorRecoveryConfig) -> Self {
        Self {
            config,
            violation_counts: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            last_reset: Arc::new(tokio::sync::RwLock::new(Instant::now())),
        }
    }

    async fn record_error(&self, oauth_error: &OAuthError) {
        // Reset counters every hour
        {
            let mut last_reset = self.last_reset.write().await;
            if last_reset.elapsed().as_secs() >= 3600 {
                let mut violation_counts = self.violation_counts.write().await;
                violation_counts.clear();
                *last_reset = Instant::now();
            }
        }

        // Record security violations
        if let Some(provider) = oauth_error.get_provider() {
            match oauth_error {
                OAuthError::SecurityViolation { .. } | OAuthError::CsrfAttackDetected { .. } => {
                    let mut violation_counts = self.violation_counts.write().await;
                    let count = violation_counts.entry(provider.clone()).or_insert(0);
                    *count += 1;

                    warn!(
                        provider = %provider,
                        violation_count = *count,
                        error = %oauth_error,
                        "OAuth security violation recorded"
                    );
                }
                _ => {}
            }
        }
    }

    async fn get_recent_violations(&self, provider: &OAuthProviderType) -> u32 {
        let violation_counts = self.violation_counts.read().await;
        violation_counts.get(provider).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let config = OAuthErrorRecoveryConfig {
            circuit_breaker_threshold: 2,
            ..Default::default()
        };
        let mut circuit_breaker = CircuitBreaker::new(config);

        // Initially closed
        assert!(circuit_breaker.can_execute());

        // Record failures
        circuit_breaker.record_failure();
        assert!(circuit_breaker.can_execute());

        circuit_breaker.record_failure();
        assert!(!circuit_breaker.can_execute());
        assert!(circuit_breaker.is_open());
    }

    #[tokio::test]
    async fn test_retry_with_exponential_backoff() {
        let config = OAuthErrorRecoveryConfig {
            max_retries: 2,
            base_delay: 1,
            jitter_factor: 0.0, // No jitter for predictable testing
            ..Default::default()
        };
        let recovery_service = OAuthErrorRecoveryService::new(config);

        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = Arc::clone(&attempt_count);

        let result = recovery_service
            .execute_with_recovery(OAuthProviderType::Google, "test_operation", move || {
                let count = attempt_count_clone.fetch_add(1, Ordering::SeqCst);
                Box::pin(async move {
                    if count < 2 {
                        Err(AppError::OAuth(OAuthError::NetworkError {
                            provider: OAuthProviderType::Google,
                            reason: "Connection timeout".to_string(),
                            is_transient: true,
                            retry_count: count,
                        }))
                    } else {
                        Ok("success")
                    }
                })
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3); // Initial + 2 retries
    }

    #[tokio::test]
    async fn test_non_retryable_error_fails_immediately() {
        let config = OAuthErrorRecoveryConfig::default();
        let recovery_service = OAuthErrorRecoveryService::new(config);

        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = Arc::clone(&attempt_count);

        let result: Result<()> = recovery_service
            .execute_with_recovery(OAuthProviderType::Google, "test_operation", move || {
                attempt_count_clone.fetch_add(1, Ordering::SeqCst);
                Box::pin(async move {
                    Err(AppError::OAuth(OAuthError::InvalidConfiguration {
                        provider: OAuthProviderType::Google,
                        reason: "Invalid client credentials".to_string(),
                        validation_errors: vec![],
                    }))
                })
            })
            .await;

        assert!(result.is_err());
        assert_eq!(attempt_count.load(Ordering::SeqCst), 1); // Only one attempt
    }

    #[test]
    fn test_user_guidance_generation() {
        let config = OAuthErrorRecoveryConfig::default();
        let recovery_service = OAuthErrorRecoveryService::new(config);

        let error = OAuthError::TokenRefreshFailed {
            provider: OAuthProviderType::Google,
            reason: "Refresh token expired".to_string(),
            requires_reauth: true,
        };

        let guidance = recovery_service.get_user_guidance(&error);
        assert!(guidance.title.contains("Google"));
        assert!(guidance.is_user_actionable);
        assert!(!guidance.contact_support);
    }
}
