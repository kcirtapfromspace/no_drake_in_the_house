use music_streaming_blocklist_backend::{
    models::oauth::OAuthProviderType,
    services::{OAuthHealthConfig, OAuthHealthMonitor, OAuthProviderHealthStatus},
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_oauth_health_monitor_creation() {
    let providers = Arc::new(HashMap::new());
    let config = OAuthHealthConfig::default();

    let monitor = OAuthHealthMonitor::new(providers, config);

    // Test that we can get health status (should be empty initially)
    let health_status = monitor.get_health_status().await;
    assert!(health_status.is_empty());
}

#[tokio::test]
async fn test_oauth_health_config_defaults() {
    let config = OAuthHealthConfig::default();

    assert_eq!(config.check_interval, Duration::from_secs(300));
    assert_eq!(config.timeout, Duration::from_secs(10));
    assert_eq!(config.max_consecutive_failures, 3);
    assert_eq!(config.exponential_backoff_base, Duration::from_secs(30));
    assert_eq!(config.max_backoff, Duration::from_secs(3600));
    assert!(config.enable_detailed_checks);
}

#[tokio::test]
async fn test_oauth_provider_health_status_serialization() {
    use serde_json;

    let status = OAuthProviderHealthStatus::Healthy;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"Healthy\"");

    let status = OAuthProviderHealthStatus::Degraded {
        reason: "Rate limited".to_string(),
    };
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("Degraded"));
    assert!(json.contains("Rate limited"));

    let status = OAuthProviderHealthStatus::Unhealthy {
        reason: "Service unavailable".to_string(),
    };
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("Unhealthy"));
    assert!(json.contains("Service unavailable"));
}

#[tokio::test]
async fn test_oauth_health_monitor_provider_availability() {
    let providers = Arc::new(HashMap::new());
    let config = OAuthHealthConfig::default();

    let monitor = OAuthHealthMonitor::new(providers, config);

    // Test with non-existent provider
    let is_healthy = monitor
        .is_provider_healthy(&OAuthProviderType::Google)
        .await;
    assert!(!is_healthy);

    let health = monitor
        .get_provider_health(&OAuthProviderType::Google)
        .await;
    assert!(health.is_none());
}

#[tokio::test]
async fn test_oauth_health_monitor_backoff_calculation() {
    let providers = Arc::new(HashMap::new());
    let config = OAuthHealthConfig {
        exponential_backoff_base: Duration::from_secs(10),
        max_backoff: Duration::from_secs(300),
        ..Default::default()
    };

    let monitor = OAuthHealthMonitor::new(providers, config);

    // Test backoff for non-existent provider (should return base delay)
    let delay = monitor.get_backoff_delay(&OAuthProviderType::Google).await;
    assert_eq!(delay, Duration::from_secs(10));
}

#[tokio::test]
async fn test_oauth_health_monitor_healthy_providers() {
    let providers = Arc::new(HashMap::new());
    let config = OAuthHealthConfig::default();

    let monitor = OAuthHealthMonitor::new(providers, config);

    // Test with no providers
    let healthy_providers = monitor.get_healthy_providers().await;
    assert!(healthy_providers.is_empty());
}

// Integration test that would require actual OAuth providers
#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_oauth_health_check_integration() {
    use reqwest::Client;
    use std::time::Instant;

    let client = Client::new();
    let start = Instant::now();

    // Test Google OAuth discovery endpoint
    let response = client
        .get("https://accounts.google.com/.well-known/openid_configuration")
        .send()
        .await;

    let elapsed = start.elapsed();

    match response {
        Ok(resp) => {
            assert!(resp.status().is_success());
            assert!(elapsed < Duration::from_secs(5)); // Should be fast

            let discovery_doc: serde_json::Value = resp.json().await.unwrap();
            assert!(discovery_doc.get("authorization_endpoint").is_some());
            assert!(discovery_doc.get("token_endpoint").is_some());
        }
        Err(e) => {
            // Network might not be available in test environment
            println!("Network test skipped: {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_github_api_health_check() {
    use reqwest::Client;

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("oauth-health-test/1.0")
        .build()
        .unwrap();

    let response = client.get("https://api.github.com/rate_limit").send().await;

    match response {
        Ok(resp) => {
            assert!(resp.status().is_success());

            let rate_limit_data: serde_json::Value = resp.json().await.unwrap();
            assert!(rate_limit_data.get("rate").is_some());
        }
        Err(e) => {
            // Network might not be available in test environment
            println!("Network test skipped: {}", e);
        }
    }
}
