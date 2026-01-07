use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, warn, error};

use crate::services::auth::AuthService;
use crate::error::Result;

/// Background service for managing OAuth token lifecycle
pub struct OAuthTokenManager {
    auth_service: Arc<AuthService>,
    refresh_interval: Duration,
    cleanup_interval: Duration,
}

impl OAuthTokenManager {
    /// Create a new OAuth token manager
    pub fn new(auth_service: Arc<AuthService>) -> Self {
        Self {
            auth_service,
            refresh_interval: Duration::from_secs(3600), // Check every hour
            cleanup_interval: Duration::from_secs(86400), // Cleanup daily
        }
    }

    /// Create with custom intervals (for testing)
    pub fn with_intervals(
        auth_service: Arc<AuthService>,
        refresh_interval: Duration,
        cleanup_interval: Duration,
    ) -> Self {
        Self {
            auth_service,
            refresh_interval,
            cleanup_interval,
        }
    }

    /// Start the background token management service
    pub async fn start(&self) -> Result<()> {
        info!("Starting OAuth token management service");

        // Start refresh task
        let refresh_service = self.auth_service.clone();
        let refresh_interval = self.refresh_interval;
        let refresh_task = tokio::spawn(async move {
            Self::run_refresh_task(refresh_service, refresh_interval).await;
        });

        // Start cleanup task
        let cleanup_service = self.auth_service.clone();
        let cleanup_interval = self.cleanup_interval;
        let cleanup_task = tokio::spawn(async move {
            Self::run_cleanup_task(cleanup_service, cleanup_interval).await;
        });

        // Start monitoring task for proactive token refresh
        let monitoring_service = self.auth_service.clone();
        let monitoring_interval = Duration::from_secs(1800); // 30 minutes
        let monitoring_task = tokio::spawn(async move {
            Self::run_monitoring_task(monitoring_service, monitoring_interval).await;
        });

        // Wait for all tasks (they run indefinitely)
        tokio::select! {
            result = refresh_task => {
                error!("OAuth token refresh task ended unexpectedly: {:?}", result);
            }
            result = cleanup_task => {
                error!("OAuth token cleanup task ended unexpectedly: {:?}", result);
            }
            result = monitoring_task => {
                error!("OAuth token monitoring task ended unexpectedly: {:?}", result);
            }
        }

        Ok(())
    }

    /// Run the token refresh task
    async fn run_refresh_task(auth_service: Arc<AuthService>, refresh_interval: Duration) {
        let mut interval = interval(refresh_interval);
        
        loop {
            interval.tick().await;
            
            match auth_service.refresh_all_expired_tokens().await {
                Ok(refreshed_count) => {
                    if refreshed_count > 0 {
                        info!(
                            refreshed_count = refreshed_count,
                            "OAuth token refresh cycle completed"
                        );
                    }
                }
                Err(e) => {
                    error!(
                        error = %e,
                        "OAuth token refresh cycle failed"
                    );
                }
            }
        }
    }

    /// Run the token cleanup task
    async fn run_cleanup_task(auth_service: Arc<AuthService>, cleanup_interval: Duration) {
        let mut interval = interval(cleanup_interval);
        
        loop {
            interval.tick().await;
            
            match auth_service.cleanup_expired_tokens().await {
                Ok(cleaned_count) => {
                    if cleaned_count > 0 {
                        info!(
                            cleaned_count = cleaned_count,
                            "OAuth token cleanup cycle completed"
                        );
                    }
                }
                Err(e) => {
                    error!(
                        error = %e,
                        "OAuth token cleanup cycle failed"
                    );
                }
            }
        }
    }

    /// Run the token monitoring and proactive refresh task
    async fn run_monitoring_task(auth_service: Arc<AuthService>, monitoring_interval: Duration) {
        let mut interval = interval(monitoring_interval);
        
        loop {
            interval.tick().await;
            
            // Execute proactive token refresh for high-priority tokens
            match auth_service.execute_proactive_token_refresh().await {
                Ok(summary) => {
                    if summary.total_attempted > 0 {
                        info!(
                            total_attempted = summary.total_attempted,
                            successful = summary.successful_refreshes,
                            failed = summary.failed_refreshes,
                            "OAuth token monitoring cycle completed"
                        );
                        
                        // Log any errors for debugging
                        if !summary.errors.is_empty() {
                            warn!(
                                errors = ?summary.errors,
                                "Some token refreshes failed during monitoring cycle"
                            );
                        }
                    }
                }
                Err(e) => {
                    error!(
                        error = %e,
                        "OAuth token monitoring cycle failed"
                    );
                }
            }
        }
    }

    /// Perform immediate token refresh for all expired tokens
    pub async fn refresh_now(&self) -> Result<u32> {
        info!("Performing immediate OAuth token refresh");
        self.auth_service.refresh_all_expired_tokens().await
    }

    /// Perform immediate cleanup of expired tokens
    pub async fn cleanup_now(&self) -> Result<u32> {
        info!("Performing immediate OAuth token cleanup");
        self.auth_service.cleanup_expired_tokens().await
    }

    /// Get token refresh schedule for monitoring
    pub async fn get_refresh_schedule(&self) -> Result<Vec<crate::models::oauth::TokenRefreshSchedule>> {
        self.auth_service.schedule_token_refresh().await
    }

    /// Start with graceful shutdown support
    pub async fn start_with_shutdown(&self, mut shutdown_rx: tokio::sync::oneshot::Receiver<()>) -> Result<()> {
        info!("Starting OAuth token management service with shutdown support");

        // Start refresh task
        let refresh_service = self.auth_service.clone();
        let refresh_interval = self.refresh_interval;
        let mut refresh_task = tokio::spawn(async move {
            Self::run_refresh_task(refresh_service, refresh_interval).await;
        });

        // Start cleanup task
        let cleanup_service = self.auth_service.clone();
        let cleanup_interval = self.cleanup_interval;
        let mut cleanup_task = tokio::spawn(async move {
            Self::run_cleanup_task(cleanup_service, cleanup_interval).await;
        });

        // Wait for shutdown signal or task completion
        tokio::select! {
            _ = &mut shutdown_rx => {
                info!("Received shutdown signal, stopping OAuth token management service");
                refresh_task.abort();
                cleanup_task.abort();
            }
            result = &mut refresh_task => {
                error!("OAuth token refresh task ended unexpectedly: {:?}", result);
                cleanup_task.abort();
            }
            result = &mut cleanup_task => {
                error!("OAuth token cleanup task ended unexpectedly: {:?}", result);
                refresh_task.abort();
            }
        }

        info!("OAuth token management service stopped");
        Ok(())
    }
}

/// Configuration for OAuth token management
#[derive(Debug, Clone)]
pub struct TokenManagerConfig {
    pub refresh_interval_seconds: u64,
    pub cleanup_interval_seconds: u64,
    pub enable_background_refresh: bool,
    pub enable_background_cleanup: bool,
}

impl Default for TokenManagerConfig {
    fn default() -> Self {
        Self {
            refresh_interval_seconds: 3600,  // 1 hour
            cleanup_interval_seconds: 86400, // 24 hours
            enable_background_refresh: true,
            enable_background_cleanup: true,
        }
    }
}

impl TokenManagerConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            refresh_interval_seconds: std::env::var("OAUTH_REFRESH_INTERVAL_SECONDS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3600),
            cleanup_interval_seconds: std::env::var("OAUTH_CLEANUP_INTERVAL_SECONDS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(86400),
            enable_background_refresh: std::env::var("OAUTH_ENABLE_BACKGROUND_REFRESH")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(true),
            enable_background_cleanup: std::env::var("OAUTH_ENABLE_BACKGROUND_CLEANUP")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(true),
        }
    }

    /// Get refresh interval as Duration
    pub fn refresh_interval(&self) -> Duration {
        Duration::from_secs(self.refresh_interval_seconds)
    }

    /// Get cleanup interval as Duration
    pub fn cleanup_interval(&self) -> Duration {
        Duration::from_secs(self.cleanup_interval_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_token_manager_config_default() {
        let config = TokenManagerConfig::default();
        assert_eq!(config.refresh_interval_seconds, 3600);
        assert_eq!(config.cleanup_interval_seconds, 86400);
        assert!(config.enable_background_refresh);
        assert!(config.enable_background_cleanup);
    }

    #[test]
    fn test_token_manager_config_intervals() {
        let config = TokenManagerConfig {
            refresh_interval_seconds: 1800,
            cleanup_interval_seconds: 43200,
            enable_background_refresh: true,
            enable_background_cleanup: true,
        };

        assert_eq!(config.refresh_interval(), Duration::from_secs(1800));
        assert_eq!(config.cleanup_interval(), Duration::from_secs(43200));
    }

    #[tokio::test]
    async fn test_token_manager_creation() {
        // This test would require a mock AuthService
        // For now, just test that the struct can be created
        let config = TokenManagerConfig::default();
        assert!(config.enable_background_refresh);
    }
}