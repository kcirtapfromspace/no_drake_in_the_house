use crate::services::TokenVaultService;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval as tokio_interval, sleep};
use tracing::{error, info, warn};

/// Background service for token vault maintenance tasks
pub struct TokenVaultBackgroundService {
    vault: Arc<TokenVaultService>,
    health_check_interval: Duration,
    key_rotation_interval: Duration,
}

impl TokenVaultBackgroundService {
    pub fn new(vault: Arc<TokenVaultService>) -> Self {
        Self {
            vault,
            health_check_interval: Duration::from_secs(3600), // 1 hour
            key_rotation_interval: Duration::from_secs(86400), // 24 hours
        }
    }

    pub fn with_intervals(
        mut self,
        health_check_interval: Duration,
        key_rotation_interval: Duration,
    ) -> Self {
        self.health_check_interval = health_check_interval;
        self.key_rotation_interval = key_rotation_interval;
        self
    }

    /// Start the background service with all maintenance tasks
    pub async fn start(&self) -> Result<()> {
        info!("Starting token vault background service");

        // Start health check task
        let vault_health = Arc::clone(&self.vault);
        let health_interval = self.health_check_interval;
        let health_task = tokio::spawn(async move {
            Self::run_health_check_task(vault_health, health_interval).await;
        });

        // Start key rotation task
        let vault_rotation = Arc::clone(&self.vault);
        let rotation_interval = self.key_rotation_interval;
        let rotation_task = tokio::spawn(async move {
            Self::run_key_rotation_task(vault_rotation, rotation_interval).await;
        });

        // Start token refresh task
        let vault_refresh = Arc::clone(&self.vault);
        let refresh_interval = Duration::from_secs(300); // 5 minutes
        let refresh_task = tokio::spawn(async move {
            Self::run_token_refresh_task(vault_refresh, refresh_interval).await;
        });

        // Wait for all tasks (they run indefinitely)
        let _ = tokio::try_join!(health_task, rotation_task, refresh_task);

        Ok(())
    }

    /// Run periodic health checks on all connections
    async fn run_health_check_task(vault: Arc<TokenVaultService>, interval: Duration) {
        let mut ticker = tokio_interval(interval);

        loop {
            ticker.tick().await;

            info!("Starting periodic token health check");

            match vault.health_check_all_connections().await {
                Ok(health_checks) => {
                    let total_checks = health_checks.len();
                    let valid_tokens = health_checks.iter().filter(|hc| hc.is_valid).count();
                    let expired_tokens = health_checks.iter().filter(|hc| !hc.is_valid).count();
                    let needs_refresh = health_checks.iter().filter(|hc| hc.needs_refresh).count();

                    info!(
                        "Health check completed: {} total, {} valid, {} expired, {} need refresh",
                        total_checks, valid_tokens, expired_tokens, needs_refresh
                    );

                    // Log any problematic connections
                    for health_check in health_checks {
                        if !health_check.is_valid {
                            warn!(
                                "Connection {} is invalid: {:?}",
                                health_check.connection_id, health_check.error_message
                            );
                        } else if health_check.needs_refresh {
                            info!(
                                "Connection {} needs token refresh",
                                health_check.connection_id
                            );
                        }
                    }
                }
                Err(e) => {
                    error!("Health check failed: {}", e);
                }
            }
        }
    }

    /// Run periodic data key rotation
    async fn run_key_rotation_task(vault: Arc<TokenVaultService>, interval: Duration) {
        let mut ticker = tokio_interval(interval);

        loop {
            ticker.tick().await;

            info!("Starting periodic data key rotation");

            match vault.rotate_data_keys().await {
                Ok(rotated_count) => {
                    if rotated_count > 0 {
                        info!("Rotated {} data keys", rotated_count);
                    } else {
                        info!("No data keys needed rotation");
                    }
                }
                Err(e) => {
                    error!("Key rotation failed: {}", e);
                }
            }
        }
    }

    /// Run periodic token refresh for tokens that are about to expire
    async fn run_token_refresh_task(vault: Arc<TokenVaultService>, interval: Duration) {
        let mut ticker = tokio_interval(interval);

        loop {
            ticker.tick().await;

            info!("Starting periodic token refresh check");

            // Get all connections that need refresh
            match vault.health_check_all_connections().await {
                Ok(health_checks) => {
                    let refresh_needed: Vec<_> = health_checks
                        .into_iter()
                        .filter(|hc| hc.needs_refresh && hc.is_valid)
                        .collect();

                    if refresh_needed.is_empty() {
                        continue;
                    }

                    info!(
                        "Found {} connections that need token refresh",
                        refresh_needed.len()
                    );

                    for health_check in refresh_needed {
                        match vault.refresh_token(health_check.connection_id).await {
                            Ok(refresh_result) => {
                                if refresh_result.success {
                                    info!(
                                        "Successfully refreshed token for connection {}",
                                        health_check.connection_id
                                    );
                                } else {
                                    warn!(
                                        "Token refresh failed for connection {}: {:?}",
                                        health_check.connection_id, refresh_result.error_message
                                    );
                                }
                            }
                            Err(e) => {
                                error!(
                                    "Token refresh error for connection {}: {}",
                                    health_check.connection_id, e
                                );
                            }
                        }

                        // Small delay between refresh attempts to avoid rate limiting
                        sleep(Duration::from_millis(100)).await;
                    }
                }
                Err(e) => {
                    error!("Failed to check connections for refresh: {}", e);
                }
            }
        }
    }

    /// Perform immediate health check on all connections
    pub async fn immediate_health_check(&self) -> Result<usize> {
        info!("Performing immediate health check");

        let health_checks = self.vault.health_check_all_connections().await?;
        let total_checks = health_checks.len();

        info!(
            "Immediate health check completed on {} connections",
            total_checks
        );

        Ok(total_checks)
    }

    /// Perform immediate key rotation
    pub async fn immediate_key_rotation(&self) -> Result<usize> {
        info!("Performing immediate key rotation");

        let rotated_count = self.vault.rotate_data_keys().await?;

        info!(
            "Immediate key rotation completed, rotated {} keys",
            rotated_count
        );

        Ok(rotated_count)
    }

    /// Get service statistics
    pub async fn get_statistics(&self) -> Result<TokenVaultStatistics> {
        // Get all connections directly instead of running health checks
        let all_connections = self.vault.get_all_connections().await;

        let total_connections = all_connections.len();
        let active_connections = all_connections
            .iter()
            .filter(|c| c.status == crate::models::ConnectionStatus::Active)
            .count();
        let expired_connections = all_connections
            .iter()
            .filter(|c| c.status != crate::models::ConnectionStatus::Active)
            .count();
        let connections_needing_refresh =
            all_connections.iter().filter(|c| c.needs_refresh()).count();

        Ok(TokenVaultStatistics {
            total_connections,
            active_connections,
            expired_connections,
            connections_needing_refresh,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TokenVaultStatistics {
    pub total_connections: usize,
    pub active_connections: usize,
    pub expired_connections: usize,
    pub connections_needing_refresh: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{StoreTokenRequest, StreamingProvider};
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_background_service_creation() {
        let vault = Arc::new(TokenVaultService::new());
        let background_service = TokenVaultBackgroundService::new(Arc::clone(&vault));

        // Test that the service can be created with custom intervals
        let _custom_service =
            background_service.with_intervals(Duration::from_secs(60), Duration::from_secs(3600));

        // Just verify the service was created successfully
        assert!(true);
    }

    #[tokio::test]
    async fn test_statistics_with_no_connections() {
        let vault = Arc::new(TokenVaultService::new());
        let background_service = TokenVaultBackgroundService::new(Arc::clone(&vault));

        // Get statistics with no connections
        let stats = background_service.get_statistics().await.unwrap();

        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.expired_connections, 0);
        assert_eq!(stats.connections_needing_refresh, 0);
    }

    #[tokio::test]
    async fn test_statistics_with_connections() {
        let vault = Arc::new(TokenVaultService::new());
        let background_service = TokenVaultBackgroundService::new(Arc::clone(&vault));

        // Add a test connection
        let user_id = Uuid::new_v4();
        let request = StoreTokenRequest {
            user_id,
            provider: StreamingProvider::Spotify,
            provider_user_id: "spotify_user_1".to_string(),
            access_token: "access_token_1".to_string(),
            refresh_token: None,
            scopes: vec!["read".to_string()],
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
        };

        vault.store_token(request).await.unwrap();

        // Get statistics
        let stats = background_service.get_statistics().await.unwrap();

        assert_eq!(stats.total_connections, 1);
        assert_eq!(stats.active_connections, 1);
        assert_eq!(stats.expired_connections, 0);
    }
}
