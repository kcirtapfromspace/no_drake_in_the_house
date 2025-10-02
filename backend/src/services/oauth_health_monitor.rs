use crate::models::oauth::OAuthProviderType;
use crate::services::oauth::OAuthProvider;
use crate::error::{AppError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// OAuth provider health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OAuthProviderHealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
    Unknown,
}

/// OAuth provider health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProviderHealth {
    pub provider: OAuthProviderType,
    pub status: OAuthProviderHealthStatus,
    pub last_check: DateTime<Utc>,
    pub response_time_ms: Option<u64>,
    pub consecutive_failures: u32,
    pub last_success: Option<DateTime<Utc>>,
    pub last_failure: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub rate_limit_info: Option<RateLimitInfo>,
}

/// Rate limiting information for OAuth providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub requests_remaining: Option<u32>,
    pub reset_time: Option<DateTime<Utc>>,
    pub retry_after: Option<Duration>,
    pub is_rate_limited: bool,
}

/// OAuth provider health monitoring configuration
#[derive(Debug, Clone)]
pub struct OAuthHealthConfig {
    pub check_interval: Duration,
    pub timeout: Duration,
    pub max_consecutive_failures: u32,
    pub exponential_backoff_base: Duration,
    pub max_backoff: Duration,
    pub enable_detailed_checks: bool,
}

impl Default for OAuthHealthConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(300), // 5 minutes
            timeout: Duration::from_secs(10),
            max_consecutive_failures: 3,
            exponential_backoff_base: Duration::from_secs(30),
            max_backoff: Duration::from_secs(3600), // 1 hour
            enable_detailed_checks: true,
        }
    }
}

/// OAuth provider health monitor
pub struct OAuthHealthMonitor {
    providers: Arc<HashMap<OAuthProviderType, Box<dyn OAuthProvider>>>,
    health_status: Arc<RwLock<HashMap<OAuthProviderType, OAuthProviderHealth>>>,
    config: OAuthHealthConfig,
    client: reqwest::Client,
}

impl OAuthHealthMonitor {
    /// Create a new OAuth health monitor
    pub fn new(
        providers: Arc<HashMap<OAuthProviderType, Box<dyn OAuthProvider>>>,
        config: OAuthHealthConfig,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .user_agent("no-drake-oauth-health-monitor/1.0")
            .build()
            .expect("Failed to create HTTP client for health monitoring");

        Self {
            providers,
            health_status: Arc::new(RwLock::new(HashMap::new())),
            config,
            client,
        }
    }

    /// Start background health monitoring
    pub async fn start_monitoring(&self) {
        info!("🏥 Starting OAuth provider health monitoring...");
        
        // Initialize health status for all providers
        self.initialize_health_status().await;
        
        // Start periodic health checks
        let health_status = Arc::clone(&self.health_status);
        let providers = Arc::clone(&self.providers);
        let config = self.config.clone();
        let client = self.client.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.check_interval);
            
            loop {
                interval.tick().await;
                
                debug!("🔍 Running OAuth provider health checks...");
                
                for (provider_type, provider) in providers.iter() {
                    let health_check_result = Self::perform_health_check(
                        provider_type,
                        provider.as_ref(),
                        &client,
                        &config,
                    ).await;
                    
                    // Update health status
                    let mut status_map = health_status.write().await;
                    if let Some(current_health) = status_map.get_mut(provider_type) {
                        Self::update_health_status(current_health, health_check_result, &config);
                    }
                }
                
                // Log health summary
                Self::log_health_summary(&health_status).await;
            }
        });
    }

    /// Initialize health status for all providers
    async fn initialize_health_status(&self) {
        let mut status_map = self.health_status.write().await;
        
        for provider_type in self.providers.keys() {
            status_map.insert(
                provider_type.clone(),
                OAuthProviderHealth {
                    provider: provider_type.clone(),
                    status: OAuthProviderHealthStatus::Unknown,
                    last_check: Utc::now(),
                    response_time_ms: None,
                    consecutive_failures: 0,
                    last_success: None,
                    last_failure: None,
                    error_message: None,
                    rate_limit_info: None,
                },
            );
        }
    }

    /// Perform health check for a specific provider
    async fn perform_health_check(
        provider_type: &OAuthProviderType,
        _provider: &dyn OAuthProvider,
        client: &reqwest::Client,
        config: &OAuthHealthConfig,
    ) -> HealthCheckResult {
        let start_time = Instant::now();
        
        debug!("🔍 Checking health for {} OAuth provider", provider_type);
        
        // Perform provider-specific health checks
        let result = match provider_type {
            OAuthProviderType::Google => Self::check_google_health(client).await,
            OAuthProviderType::Apple => Self::check_apple_health(client).await,
            OAuthProviderType::GitHub => Self::check_github_health(client).await,
        };
        
        let response_time = start_time.elapsed();
        
        match result {
            Ok(rate_limit_info) => {
                debug!("✅ {} OAuth provider is healthy ({}ms)", provider_type, response_time.as_millis());
                HealthCheckResult {
                    is_healthy: true,
                    response_time_ms: response_time.as_millis() as u64,
                    error_message: None,
                    rate_limit_info,
                }
            }
            Err(e) => {
                warn!("❌ {} OAuth provider health check failed: {}", provider_type, e);
                HealthCheckResult {
                    is_healthy: false,
                    response_time_ms: response_time.as_millis() as u64,
                    error_message: Some(e.to_string()),
                    rate_limit_info: None,
                }
            }
        }
    }

    /// Check Google OAuth provider health
    async fn check_google_health(client: &reqwest::Client) -> Result<Option<RateLimitInfo>> {
        // Check Google's OAuth2 discovery document
        let response = client
            .get("https://accounts.google.com/.well-known/openid_configuration")
            .send()
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("Google health check failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalServiceError(format!(
                "Google OAuth health check returned status: {}",
                response.status()
            )));
        }

        // Parse rate limit headers if present
        let rate_limit_info = Self::parse_rate_limit_headers(&response);

        // Verify the response contains expected OAuth endpoints
        let discovery_doc: serde_json::Value = response.json().await
            .map_err(|e| AppError::ExternalServiceError(format!("Failed to parse Google discovery document: {}", e)))?;

        if !discovery_doc.get("authorization_endpoint").is_some() ||
           !discovery_doc.get("token_endpoint").is_some() {
            return Err(AppError::ExternalServiceError(
                "Google OAuth endpoints not found in discovery document".to_string()
            ));
        }

        Ok(rate_limit_info)
    }

    /// Check Apple OAuth provider health
    async fn check_apple_health(client: &reqwest::Client) -> Result<Option<RateLimitInfo>> {
        // Check Apple's Sign In endpoint availability
        let response = client
            .head("https://appleid.apple.com/auth/authorize")
            .send()
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("Apple health check failed: {}", e)))?;

        // Apple returns 400 for HEAD requests without parameters, which is expected
        if response.status().as_u16() != 400 && !response.status().is_success() {
            return Err(AppError::ExternalServiceError(format!(
                "Apple OAuth health check returned unexpected status: {}",
                response.status()
            )));
        }

        let rate_limit_info = Self::parse_rate_limit_headers(&response);
        Ok(rate_limit_info)
    }

    /// Check GitHub OAuth provider health
    async fn check_github_health(client: &reqwest::Client) -> Result<Option<RateLimitInfo>> {
        // Check GitHub API status
        let response = client
            .get("https://api.github.com/rate_limit")
            .header("User-Agent", "no-drake-oauth-health-monitor/1.0")
            .send()
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("GitHub health check failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalServiceError(format!(
                "GitHub OAuth health check returned status: {}",
                response.status()
            )));
        }

        // Parse GitHub rate limit information
        let rate_limit_data: serde_json::Value = response.json().await
            .map_err(|e| AppError::ExternalServiceError(format!("Failed to parse GitHub rate limit response: {}", e)))?;

        let rate_limit_info = if let Some(core) = rate_limit_data.get("rate") {
            Some(RateLimitInfo {
                requests_remaining: core.get("remaining").and_then(|v| v.as_u64()).map(|v| v as u32),
                reset_time: core.get("reset").and_then(|v| v.as_i64()).map(|timestamp| {
                    DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now)
                }),
                retry_after: None,
                is_rate_limited: core.get("remaining").and_then(|v| v.as_u64()).unwrap_or(1) == 0,
            })
        } else {
            None
        };

        Ok(rate_limit_info)
    }

    /// Parse rate limit headers from HTTP response
    fn parse_rate_limit_headers(response: &reqwest::Response) -> Option<RateLimitInfo> {
        let headers = response.headers();
        
        // Check for standard rate limit headers
        let requests_remaining = headers.get("x-ratelimit-remaining")
            .or_else(|| headers.get("x-rate-limit-remaining"))
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());

        let reset_time = headers.get("x-ratelimit-reset")
            .or_else(|| headers.get("x-rate-limit-reset"))
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok())
            .map(|timestamp| DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now));

        let retry_after = headers.get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_secs);

        if requests_remaining.is_some() || reset_time.is_some() || retry_after.is_some() {
            Some(RateLimitInfo {
                requests_remaining,
                reset_time,
                retry_after,
                is_rate_limited: requests_remaining.unwrap_or(1) == 0 || retry_after.is_some(),
            })
        } else {
            None
        }
    }

    /// Update health status based on check result
    fn update_health_status(
        current_health: &mut OAuthProviderHealth,
        check_result: HealthCheckResult,
        config: &OAuthHealthConfig,
    ) {
        current_health.last_check = Utc::now();
        current_health.response_time_ms = Some(check_result.response_time_ms);
        current_health.rate_limit_info = check_result.rate_limit_info;

        if check_result.is_healthy {
            current_health.consecutive_failures = 0;
            current_health.last_success = Some(Utc::now());
            current_health.error_message = None;
            
            // Check if rate limited
            if let Some(ref rate_limit) = current_health.rate_limit_info {
                if rate_limit.is_rate_limited {
                    current_health.status = OAuthProviderHealthStatus::Degraded {
                        reason: "Rate limited".to_string(),
                    };
                } else {
                    current_health.status = OAuthProviderHealthStatus::Healthy;
                }
            } else {
                current_health.status = OAuthProviderHealthStatus::Healthy;
            }
        } else {
            current_health.consecutive_failures += 1;
            current_health.last_failure = Some(Utc::now());
            current_health.error_message = check_result.error_message;

            if current_health.consecutive_failures >= config.max_consecutive_failures {
                current_health.status = OAuthProviderHealthStatus::Unhealthy {
                    reason: current_health.error_message.clone()
                        .unwrap_or_else(|| "Multiple consecutive failures".to_string()),
                };
            } else {
                current_health.status = OAuthProviderHealthStatus::Degraded {
                    reason: current_health.error_message.clone()
                        .unwrap_or_else(|| "Health check failed".to_string()),
                };
            }
        }
    }

    /// Log health summary for all providers
    async fn log_health_summary(health_status: &Arc<RwLock<HashMap<OAuthProviderType, OAuthProviderHealth>>>) {
        let status_map = health_status.read().await;
        
        let healthy_count = status_map.values()
            .filter(|h| matches!(h.status, OAuthProviderHealthStatus::Healthy))
            .count();
        
        let degraded_count = status_map.values()
            .filter(|h| matches!(h.status, OAuthProviderHealthStatus::Degraded { .. }))
            .count();
        
        let unhealthy_count = status_map.values()
            .filter(|h| matches!(h.status, OAuthProviderHealthStatus::Unhealthy { .. }))
            .count();

        if unhealthy_count > 0 {
            error!("🚨 OAuth Health Summary: {} healthy, {} degraded, {} unhealthy", 
                   healthy_count, degraded_count, unhealthy_count);
        } else if degraded_count > 0 {
            warn!("⚠️  OAuth Health Summary: {} healthy, {} degraded, {} unhealthy", 
                  healthy_count, degraded_count, unhealthy_count);
        } else {
            debug!("✅ OAuth Health Summary: {} healthy, {} degraded, {} unhealthy", 
                   healthy_count, degraded_count, unhealthy_count);
        }

        // Log details for unhealthy providers
        for (provider, health) in status_map.iter() {
            match &health.status {
                OAuthProviderHealthStatus::Unhealthy { reason } => {
                    error!("❌ {} OAuth provider is unhealthy: {} (failures: {})", 
                           provider, reason, health.consecutive_failures);
                }
                OAuthProviderHealthStatus::Degraded { reason } => {
                    warn!("⚠️  {} OAuth provider is degraded: {}", provider, reason);
                }
                _ => {}
            }
        }
    }

    /// Get current health status for all providers
    pub async fn get_health_status(&self) -> HashMap<OAuthProviderType, OAuthProviderHealth> {
        self.health_status.read().await.clone()
    }

    /// Get health status for a specific provider
    pub async fn get_provider_health(&self, provider: &OAuthProviderType) -> Option<OAuthProviderHealth> {
        self.health_status.read().await.get(provider).cloned()
    }

    /// Check if a provider is healthy
    pub async fn is_provider_healthy(&self, provider: &OAuthProviderType) -> bool {
        if let Some(health) = self.get_provider_health(provider).await {
            matches!(health.status, OAuthProviderHealthStatus::Healthy)
        } else {
            false
        }
    }

    /// Get providers that are currently healthy
    pub async fn get_healthy_providers(&self) -> Vec<OAuthProviderType> {
        let status_map = self.health_status.read().await;
        status_map
            .iter()
            .filter(|(_, health)| matches!(health.status, OAuthProviderHealthStatus::Healthy))
            .map(|(provider, _)| provider.clone())
            .collect()
    }

    /// Force a health check for all providers
    pub async fn force_health_check(&self) {
        info!("🔄 Forcing OAuth provider health checks...");
        
        for (provider_type, provider) in self.providers.iter() {
            let health_check_result = Self::perform_health_check(
                provider_type,
                provider.as_ref(),
                &self.client,
                &self.config,
            ).await;
            
            // Update health status
            let mut status_map = self.health_status.write().await;
            if let Some(current_health) = status_map.get_mut(provider_type) {
                Self::update_health_status(current_health, health_check_result, &self.config);
            }
        }
    }

    /// Get exponential backoff delay for a provider
    pub async fn get_backoff_delay(&self, provider: &OAuthProviderType) -> Duration {
        if let Some(health) = self.get_provider_health(provider).await {
            let backoff_multiplier = 2_u32.pow(health.consecutive_failures.min(10)); // Cap at 2^10
            let delay = self.config.exponential_backoff_base * backoff_multiplier;
            delay.min(self.config.max_backoff)
        } else {
            self.config.exponential_backoff_base
        }
    }
}

/// Internal health check result
struct HealthCheckResult {
    is_healthy: bool,
    response_time_ms: u64,
    error_message: Option<String>,
    rate_limit_info: Option<RateLimitInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_oauth_health_config_default() {
        let config = OAuthHealthConfig::default();
        assert_eq!(config.check_interval, Duration::from_secs(300));
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert_eq!(config.max_consecutive_failures, 3);
    }

    #[test]
    fn test_oauth_provider_health_status() {
        let health = OAuthProviderHealth {
            provider: OAuthProviderType::Google,
            status: OAuthProviderHealthStatus::Healthy,
            last_check: Utc::now(),
            response_time_ms: Some(150),
            consecutive_failures: 0,
            last_success: Some(Utc::now()),
            last_failure: None,
            error_message: None,
            rate_limit_info: None,
        };

        assert_eq!(health.provider, OAuthProviderType::Google);
        assert!(matches!(health.status, OAuthProviderHealthStatus::Healthy));
        assert_eq!(health.consecutive_failures, 0);
    }

    #[test]
    fn test_rate_limit_info() {
        let rate_limit = RateLimitInfo {
            requests_remaining: Some(100),
            reset_time: Some(Utc::now()),
            retry_after: None,
            is_rate_limited: false,
        };

        assert_eq!(rate_limit.requests_remaining, Some(100));
        assert!(!rate_limit.is_rate_limited);
    }
}