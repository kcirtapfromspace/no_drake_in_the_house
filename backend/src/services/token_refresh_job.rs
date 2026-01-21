//! Proactive Token Refresh Background Job (US-011)
//!
//! Refreshes tokens before they expire to prevent service interruptions.
//! Runs periodically (configurable via TOKEN_REFRESH_INTERVAL_HOURS),
//! queries connections expiring within 24 hours, and refreshes them in batches.

use crate::config::TokenRefreshConfig;
use crate::metrics::MetricsCollector;
use crate::models::{Connection, ConnectionStatus};
use crate::services::token_vault_repository::TokenVaultRepository;
use crate::services::{NotificationService, TokenVaultService};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{interval as tokio_interval, sleep};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Metrics for token refresh operations
#[derive(Debug, Default)]
pub struct TokenRefreshMetrics {
    /// Total number of tokens successfully refreshed
    pub tokens_refreshed_total: AtomicU64,
    /// Total number of token refresh failures
    pub token_refresh_failures_total: AtomicU64,
    /// Total number of connections marked as NeedsReauth after max retries
    pub connections_marked_reauth_total: AtomicU64,
    /// Last run timestamp
    pub last_run_at: RwLock<Option<DateTime<Utc>>>,
    /// Last run duration in milliseconds
    pub last_run_duration_ms: AtomicU64,
}

impl TokenRefreshMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_success(&self) {
        self.tokens_refreshed_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_failure(&self) {
        self.token_refresh_failures_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_reauth(&self) {
        self.connections_marked_reauth_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub async fn record_run(&self, duration_ms: u64) {
        let mut last_run = self.last_run_at.write().await;
        *last_run = Some(Utc::now());
        self.last_run_duration_ms
            .store(duration_ms, Ordering::Relaxed);
    }

    pub fn get_refreshed_total(&self) -> u64 {
        self.tokens_refreshed_total.load(Ordering::Relaxed)
    }

    pub fn get_failures_total(&self) -> u64 {
        self.token_refresh_failures_total.load(Ordering::Relaxed)
    }

    pub fn get_reauth_total(&self) -> u64 {
        self.connections_marked_reauth_total.load(Ordering::Relaxed)
    }
}

/// Retry state for a connection's token refresh
#[derive(Debug, Clone)]
struct RefreshRetryState {
    connection_id: Uuid,
    retry_count: u32,
    last_error: Option<String>,
    next_retry_at: DateTime<Utc>,
}

impl RefreshRetryState {
    fn new(connection_id: Uuid) -> Self {
        Self {
            connection_id,
            retry_count: 0,
            last_error: None,
            next_retry_at: Utc::now(),
        }
    }

    fn increment_retry(&mut self, base_delay_secs: u64, error: String) {
        self.retry_count += 1;
        self.last_error = Some(error);
        // Exponential backoff: base_delay * 2^retry_count, capped at 32x base delay
        let delay_multiplier = 2_u64.pow(self.retry_count.min(5));
        let delay_secs = base_delay_secs.saturating_mul(delay_multiplier);
        self.next_retry_at = Utc::now() + chrono::Duration::seconds(delay_secs as i64);
    }

    fn can_retry(&self, max_retries: u32) -> bool {
        self.retry_count < max_retries && Utc::now() >= self.next_retry_at
    }

    fn exceeded_max_retries(&self, max_retries: u32) -> bool {
        self.retry_count >= max_retries
    }
}

/// Proactive token refresh background job
pub struct TokenRefreshBackgroundJob {
    vault: Arc<TokenVaultService>,
    config: TokenRefreshConfig,
    metrics: Arc<TokenRefreshMetrics>,
    /// Track retry state for connections that failed refresh
    retry_states: Arc<RwLock<std::collections::HashMap<Uuid, RefreshRetryState>>>,
    /// Notification service for user alerts (US-027)
    notification_service: Option<Arc<NotificationService>>,
    /// Prometheus metrics collector for external metrics emission
    prometheus_metrics: Option<Arc<MetricsCollector>>,
}

impl TokenRefreshBackgroundJob {
    /// Create a new token refresh background job
    pub fn new(vault: Arc<TokenVaultService>, config: TokenRefreshConfig) -> Self {
        Self {
            vault,
            config,
            metrics: Arc::new(TokenRefreshMetrics::new()),
            retry_states: Arc::new(RwLock::new(std::collections::HashMap::new())),
            notification_service: None,
            prometheus_metrics: None,
        }
    }

    /// Create a new token refresh background job with notification service (US-027)
    pub fn with_notification_service(
        vault: Arc<TokenVaultService>,
        config: TokenRefreshConfig,
        notification_service: Arc<NotificationService>,
    ) -> Self {
        Self {
            vault,
            config,
            metrics: Arc::new(TokenRefreshMetrics::new()),
            retry_states: Arc::new(RwLock::new(std::collections::HashMap::new())),
            notification_service: Some(notification_service),
            prometheus_metrics: None,
        }
    }

    /// Create a new token refresh background job with all services configured
    pub fn with_all_services(
        vault: Arc<TokenVaultService>,
        config: TokenRefreshConfig,
        notification_service: Option<Arc<NotificationService>>,
        prometheus_metrics: Option<Arc<MetricsCollector>>,
    ) -> Self {
        Self {
            vault,
            config,
            metrics: Arc::new(TokenRefreshMetrics::new()),
            retry_states: Arc::new(RwLock::new(std::collections::HashMap::new())),
            notification_service,
            prometheus_metrics,
        }
    }

    /// Set Prometheus metrics collector
    pub fn with_prometheus_metrics(mut self, metrics: Arc<MetricsCollector>) -> Self {
        self.prometheus_metrics = Some(metrics);
        self
    }

    /// Get reference to metrics
    pub fn metrics(&self) -> Arc<TokenRefreshMetrics> {
        Arc::clone(&self.metrics)
    }

    /// Start the background job
    pub async fn start(&self) -> Result<()> {
        info!(
            "Starting proactive token refresh background job (interval: {} hours, threshold: {} hours)",
            self.config.interval_hours,
            self.config.expiry_threshold_hours
        );

        let interval_duration = Duration::from_secs(self.config.interval_hours * 3600);
        let mut ticker = tokio_interval(interval_duration);

        // Run first tick immediately
        ticker.tick().await;

        loop {
            if let Err(e) = self.run_refresh_cycle().await {
                error!("Token refresh cycle failed: {}", e);
            }

            ticker.tick().await;
        }
    }

    /// Run a single refresh cycle
    pub async fn run_refresh_cycle(&self) -> Result<RefreshCycleResult> {
        let start_time = std::time::Instant::now();

        info!("Starting token refresh cycle");

        // Get connections expiring within threshold
        let connections = self.get_expiring_connections().await?;

        if connections.is_empty() {
            info!("No connections need token refresh");
            let duration_ms = start_time.elapsed().as_millis() as u64;
            self.metrics.record_run(duration_ms).await;
            return Ok(RefreshCycleResult {
                total_connections: 0,
                refreshed: 0,
                failed: 0,
                marked_reauth: 0,
                duration_ms,
            });
        }

        info!(
            "Found {} connections expiring within {} hours",
            connections.len(),
            self.config.expiry_threshold_hours
        );

        // Process connections in batches with rate limiting
        let result = self.process_connections_in_batches(connections).await;

        let duration_ms = start_time.elapsed().as_millis() as u64;
        self.metrics.record_run(duration_ms).await;

        info!(
            "Token refresh cycle completed: {} refreshed, {} failed, {} marked reauth ({}ms)",
            result.refreshed, result.failed, result.marked_reauth, duration_ms
        );

        Ok(RefreshCycleResult {
            duration_ms,
            ..result
        })
    }

    /// Get connections that are expiring within the configured threshold
    async fn get_expiring_connections(&self) -> Result<Vec<Connection>> {
        if let Some(pool) = self.vault.pool() {
            let repo = TokenVaultRepository::new(pool.clone());
            repo.get_connections_expiring_within_hours(
                self.config.expiry_threshold_hours as i64,
                Some(self.config.batch_size as i64 * 10), // Get up to 10 batches worth
            )
            .await
        } else {
            // In-memory mode: filter connections manually
            let all_connections = self.vault.get_all_connections().await;
            let threshold =
                Utc::now() + chrono::Duration::hours(self.config.expiry_threshold_hours as i64);

            Ok(all_connections
                .into_iter()
                .filter(|c| {
                    c.status == ConnectionStatus::Active
                        && c.refresh_token_encrypted.is_some()
                        && c.expires_at
                            .map(|exp| exp > Utc::now() && exp < threshold)
                            .unwrap_or(false)
                })
                .collect())
        }
    }

    /// Process connections in batches with rate limiting
    async fn process_connections_in_batches(
        &self,
        connections: Vec<Connection>,
    ) -> RefreshCycleResult {
        let mut result = RefreshCycleResult {
            total_connections: connections.len(),
            refreshed: 0,
            failed: 0,
            marked_reauth: 0,
            duration_ms: 0,
        };

        // Process in batches
        for batch in connections.chunks(self.config.batch_size) {
            for connection in batch {
                let refresh_result = self.refresh_with_retry(connection).await;

                match refresh_result {
                    RefreshOutcome::Success => {
                        result.refreshed += 1;
                        self.metrics.record_success();
                        // Emit Prometheus metric
                        if let Some(ref prom_metrics) = self.prometheus_metrics {
                            prom_metrics.record_token_refreshed();
                        }
                    }
                    RefreshOutcome::Failed => {
                        result.failed += 1;
                        self.metrics.record_failure();
                        // Emit Prometheus metric
                        if let Some(ref prom_metrics) = self.prometheus_metrics {
                            prom_metrics.record_token_refresh_failure();
                        }
                    }
                    RefreshOutcome::MarkedReauth => {
                        result.marked_reauth += 1;
                        self.metrics.record_reauth();
                        // Emit Prometheus metric
                        if let Some(ref prom_metrics) = self.prometheus_metrics {
                            prom_metrics.record_connection_marked_reauth();
                        }
                    }
                    RefreshOutcome::Skipped => {
                        // Connection is scheduled for retry later
                    }
                }

                // Rate limit delay between refresh attempts
                sleep(Duration::from_millis(self.config.rate_limit_delay_ms)).await;
            }
        }

        // Clean up completed retry states
        self.cleanup_retry_states().await;

        result
    }

    /// Refresh a connection's token with retry logic
    async fn refresh_with_retry(&self, connection: &Connection) -> RefreshOutcome {
        let connection_id = connection.id;

        // Check if this connection has pending retry state
        {
            let retry_states = self.retry_states.read().await;
            if let Some(state) = retry_states.get(&connection_id) {
                if !state.can_retry(self.config.max_retries) {
                    if state.exceeded_max_retries(self.config.max_retries) {
                        // Will be handled below after releasing lock
                    } else {
                        // Not ready for retry yet
                        return RefreshOutcome::Skipped;
                    }
                }
            }
        }

        // Check if max retries exceeded and mark as NeedsReauth
        {
            let retry_states = self.retry_states.read().await;
            if let Some(state) = retry_states.get(&connection_id) {
                if state.exceeded_max_retries(self.config.max_retries) {
                    let last_error = state.last_error.clone();
                    drop(retry_states);
                    return self
                        .mark_needs_reauth(connection_id, last_error)
                        .await;
                }
            }
        }

        // Attempt refresh
        match self.vault.refresh_token(connection_id).await {
            Ok(refresh_result) => {
                if refresh_result.success {
                    // Remove from retry states on success
                    let mut retry_states = self.retry_states.write().await;
                    retry_states.remove(&connection_id);

                    info!(
                        connection_id = %connection_id,
                        provider = %connection.provider,
                        "Token refreshed successfully"
                    );
                    RefreshOutcome::Success
                } else {
                    let error_msg = refresh_result
                        .error_message
                        .unwrap_or_else(|| "Unknown error".to_string());

                    self.handle_refresh_failure(connection_id, error_msg).await
                }
            }
            Err(e) => {
                let error_msg = e.to_string();
                self.handle_refresh_failure(connection_id, error_msg).await
            }
        }
    }

    /// Handle a refresh failure with exponential backoff
    async fn handle_refresh_failure(&self, connection_id: Uuid, error: String) -> RefreshOutcome {
        let mut retry_states = self.retry_states.write().await;

        let state = retry_states
            .entry(connection_id)
            .or_insert_with(|| RefreshRetryState::new(connection_id));

        state.increment_retry(self.config.retry_base_delay_secs, error.clone());

        if state.exceeded_max_retries(self.config.max_retries) {
            let last_error = state.last_error.clone();
            drop(retry_states);

            warn!(
                connection_id = %connection_id,
                retry_count = self.config.max_retries,
                error = %error,
                "Max retries exceeded, marking connection as NeedsReauth"
            );

            return self.mark_needs_reauth(connection_id, last_error).await;
        }

        warn!(
            connection_id = %connection_id,
            retry_count = state.retry_count,
            next_retry_at = %state.next_retry_at,
            error = %error,
            "Token refresh failed, scheduled for retry"
        );

        RefreshOutcome::Failed
    }

    /// Mark a connection as needing re-authentication and notify user (US-027)
    async fn mark_needs_reauth(
        &self,
        connection_id: Uuid,
        last_error: Option<String>,
    ) -> RefreshOutcome {
        let reason = last_error.unwrap_or_else(|| "Max refresh retries exceeded".to_string());

        // Get connection details for notification before updating status
        let connection_details = self.vault.get_connection(connection_id).await;

        if let Some(pool) = self.vault.pool() {
            let repo = TokenVaultRepository::new(pool.clone());
            if let Err(e) = repo
                .update_connection_status(
                    connection_id,
                    &ConnectionStatus::NeedsReauth,
                    Some(&reason),
                )
                .await
            {
                error!(
                    connection_id = %connection_id,
                    error = %e,
                    "Failed to mark connection as NeedsReauth"
                );
            }
        }

        // US-027: Send notification to user about connection needing re-auth
        if let (Some(notification_service), Some(connection)) =
            (&self.notification_service, connection_details)
        {
            if let Err(e) = notification_service
                .notify_connection_needs_reauth(connection.user_id, &connection.provider, &reason)
                .await
            {
                error!(
                    connection_id = %connection_id,
                    user_id = %connection.user_id,
                    error = %e,
                    "Failed to send NeedsReauth notification"
                );
            } else {
                info!(
                    connection_id = %connection_id,
                    user_id = %connection.user_id,
                    provider = %connection.provider,
                    "Sent NeedsReauth notification to user"
                );
            }
        }

        // Remove from retry states
        let mut retry_states = self.retry_states.write().await;
        retry_states.remove(&connection_id);

        RefreshOutcome::MarkedReauth
    }

    /// Clean up retry states for connections that have been resolved
    async fn cleanup_retry_states(&self) {
        let mut retry_states = self.retry_states.write().await;

        // Remove states older than 24 hours that haven't been resolved
        let cutoff = Utc::now() - chrono::Duration::hours(24);
        retry_states.retain(|_, state| state.next_retry_at > cutoff);
    }

    /// Get pending retry states (for debugging/monitoring)
    pub async fn get_pending_retries(&self) -> Vec<(Uuid, u32, Option<String>)> {
        let retry_states = self.retry_states.read().await;
        retry_states
            .values()
            .map(|s| (s.connection_id, s.retry_count, s.last_error.clone()))
            .collect()
    }
}

/// Outcome of a single token refresh attempt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RefreshOutcome {
    /// Token was refreshed successfully
    Success,
    /// Refresh failed, scheduled for retry
    Failed,
    /// Connection marked as NeedsReauth after max retries
    MarkedReauth,
    /// Refresh was skipped (not ready for retry)
    Skipped,
}

/// Result of a complete refresh cycle
#[derive(Debug, Clone, Default)]
pub struct RefreshCycleResult {
    /// Total connections processed
    pub total_connections: usize,
    /// Successfully refreshed
    pub refreshed: usize,
    /// Failed (will be retried)
    pub failed: usize,
    /// Marked as NeedsReauth
    pub marked_reauth: usize,
    /// Duration of the cycle in milliseconds
    pub duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refresh_retry_state_backoff() {
        let mut state = RefreshRetryState::new(Uuid::new_v4());
        assert_eq!(state.retry_count, 0);
        assert!(state.can_retry(3));

        // First failure: delay = 30 * 2^1 = 60s
        state.increment_retry(30, "Error 1".to_string());
        assert_eq!(state.retry_count, 1);
        assert_eq!(state.last_error, Some("Error 1".to_string()));

        // Second failure: delay = 30 * 2^2 = 120s
        state.increment_retry(30, "Error 2".to_string());
        assert_eq!(state.retry_count, 2);

        // Third failure: delay = 30 * 2^3 = 240s
        state.increment_retry(30, "Error 3".to_string());
        assert_eq!(state.retry_count, 3);
        assert!(state.exceeded_max_retries(3));
        assert!(!state.can_retry(3));
    }

    #[test]
    fn test_metrics() {
        let metrics = TokenRefreshMetrics::new();

        metrics.record_success();
        metrics.record_success();
        metrics.record_failure();
        metrics.record_reauth();

        assert_eq!(metrics.get_refreshed_total(), 2);
        assert_eq!(metrics.get_failures_total(), 1);
        assert_eq!(metrics.get_reauth_total(), 1);
    }

    #[tokio::test]
    async fn test_refresh_cycle_result_default() {
        let result = RefreshCycleResult::default();
        assert_eq!(result.total_connections, 0);
        assert_eq!(result.refreshed, 0);
        assert_eq!(result.failed, 0);
        assert_eq!(result.marked_reauth, 0);
    }
}
