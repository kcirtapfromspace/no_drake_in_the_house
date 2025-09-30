use anyhow::{anyhow, Result};
use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use deadpool_redis::{Config, Pool, Runtime};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn, error};
use uuid;

/// Rate limiting configuration for different endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_window: u32,
    pub window_seconds: u64,
    pub burst_allowance: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_window: 60,
            window_seconds: 60,
            burst_allowance: 10,
        }
    }
}

/// Rate limiting service with Redis backend
#[derive(Clone)]
pub struct RateLimitService {
    redis_pool: Pool,
    configs: Arc<RwLock<HashMap<String, RateLimitConfig>>>,
}

impl RateLimitService {
    pub fn new(redis_url: &str) -> Result<Self> {
        let config = Config::from_url(redis_url);
        let pool = config.create_pool(Some(Runtime::Tokio1))?;

        let mut configs = HashMap::new();
        
        // Default configurations for different endpoint types
        configs.insert("auth".to_string(), RateLimitConfig {
            requests_per_window: 10,
            window_seconds: 300, // 5 minutes
            burst_allowance: 3,
        });
        
        configs.insert("registration".to_string(), RateLimitConfig {
            requests_per_window: 3,
            window_seconds: 60, // 1 minute
            burst_allowance: 1,
        });
        
        configs.insert("api".to_string(), RateLimitConfig {
            requests_per_window: 100,
            window_seconds: 60, // 1 minute
            burst_allowance: 20,
        });
        
        configs.insert("health".to_string(), RateLimitConfig {
            requests_per_window: 1000,
            window_seconds: 60, // 1 minute
            burst_allowance: 100,
        });

        Ok(Self {
            redis_pool: pool,
            configs: Arc::new(RwLock::new(configs)),
        })
    }

    /// Check if a request should be rate limited
    pub async fn check_rate_limit(
        &self,
        identifier: &str,
        endpoint_type: &str,
    ) -> Result<RateLimitResult> {
        let configs = self.configs.read().await;
        let config = configs.get(endpoint_type)
            .cloned()
            .unwrap_or_default();

        let mut conn = self.redis_pool.get().await
            .map_err(|e| anyhow!("Failed to get Redis connection: {}", e))?;

        let now = Utc::now().timestamp() as u64;
        let window_start = now - (now % config.window_seconds);
        let key = format!("rate_limit:{}:{}:{}", endpoint_type, identifier, window_start);

        // Get current count for this window
        let current_count: u32 = conn.get(&key).await.unwrap_or(0);

        // Check if we're over the limit
        if current_count >= config.requests_per_window {
            let reset_time = window_start + config.window_seconds;
            return Ok(RateLimitResult {
                allowed: false,
                current_count: current_count + 1,
                limit: config.requests_per_window,
                reset_time,
                retry_after: reset_time - now,
            });
        }

        // Increment counter
        let new_count: u32 = conn.incr(&key, 1).await
            .map_err(|e| anyhow!("Failed to increment rate limit counter: {}", e))?;

        // Set expiration for the key
        let _: () = conn.expire(&key, config.window_seconds as i64).await
            .map_err(|e| anyhow!("Failed to set expiration: {}", e))?;

        let reset_time = window_start + config.window_seconds;
        Ok(RateLimitResult {
            allowed: true,
            current_count: new_count,
            limit: config.requests_per_window,
            reset_time,
            retry_after: 0,
        })
    }

    /// Update rate limit configuration for an endpoint type
    pub async fn update_config(&self, endpoint_type: String, config: RateLimitConfig) {
        let mut configs = self.configs.write().await;
        configs.insert(endpoint_type, config);
    }

    /// Get current rate limit status for debugging
    pub async fn get_rate_limit_status(
        &self,
        identifier: &str,
        endpoint_type: &str,
    ) -> Result<RateLimitStatus> {
        let configs = self.configs.read().await;
        let config = configs.get(endpoint_type)
            .cloned()
            .unwrap_or_default();

        let mut conn = self.redis_pool.get().await
            .map_err(|e| anyhow!("Failed to get Redis connection: {}", e))?;

        let now = Utc::now().timestamp() as u64;
        let window_start = now - (now % config.window_seconds);
        let key = format!("rate_limit:{}:{}:{}", endpoint_type, identifier, window_start);

        let current_count: u32 = conn.get(&key).await.unwrap_or(0);
        let reset_time = window_start + config.window_seconds;

        Ok(RateLimitStatus {
            identifier: identifier.to_string(),
            endpoint_type: endpoint_type.to_string(),
            current_count,
            limit: config.requests_per_window,
            window_seconds: config.window_seconds,
            reset_time,
            time_until_reset: reset_time.saturating_sub(now),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub current_count: u32,
    pub limit: u32,
    pub reset_time: u64,
    pub retry_after: u64,
}

#[derive(Debug, Serialize)]
pub struct RateLimitStatus {
    pub identifier: String,
    pub endpoint_type: String,
    pub current_count: u32,
    pub limit: u32,
    pub window_seconds: u64,
    pub reset_time: u64,
    pub time_until_reset: u64,
}

/// Extract client IP address from request
pub fn extract_client_ip(headers: &HeaderMap, connect_info: Option<&ConnectInfo<SocketAddr>>) -> String {
    // Try X-Forwarded-For header first (for proxies)
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // Take the first IP in the chain
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    // Try X-Real-IP header (for nginx)
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }

    // Fall back to connection info
    if let Some(ConnectInfo(addr)) = connect_info {
        return addr.ip().to_string();
    }

    // Default fallback
    "unknown".to_string()
}

/// Rate limiting middleware for authentication endpoints
pub async fn auth_rate_limit_middleware(
    State(rate_limiter): State<Arc<RateLimitService>>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let client_ip = extract_client_ip(request.headers(), connect_info.as_ref());
    
    debug!("Rate limiting check for auth endpoint from IP: {}", client_ip);

    match rate_limiter.check_rate_limit(&client_ip, "auth").await {
        Ok(result) => {
            if !result.allowed {
                warn!(
                    "Rate limit exceeded for auth endpoint from IP: {} ({}/{})",
                    client_ip, result.current_count, result.limit
                );

                let response = Json(json!({
                    "error": "Rate limit exceeded",
                    "message": "Too many authentication attempts. Please try again later.",
                    "retry_after": result.retry_after,
                    "limit": result.limit,
                    "reset_time": result.reset_time
                }));

                return Err((StatusCode::TOO_MANY_REQUESTS, response));
            }

            debug!(
                "Rate limit check passed for IP: {} ({}/{})",
                client_ip, result.current_count, result.limit
            );

            // Add rate limit headers to response
            let mut response = next.run(request).await;
            let headers = response.headers_mut();
            
            headers.insert("X-RateLimit-Limit", result.limit.to_string().parse().unwrap());
            headers.insert("X-RateLimit-Remaining", 
                (result.limit.saturating_sub(result.current_count)).to_string().parse().unwrap());
            headers.insert("X-RateLimit-Reset", result.reset_time.to_string().parse().unwrap());

            Ok(response)
        }
        Err(e) => {
            error!("Rate limiting error: {}", e);
            // Allow request to proceed if rate limiting fails
            Ok(next.run(request).await)
        }
    }
}

/// Rate limiting middleware specifically for registration endpoints
pub async fn registration_rate_limit_middleware(
    State(rate_limiter): State<Arc<RateLimitService>>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let client_ip = extract_client_ip(request.headers(), connect_info.as_ref());
    
    debug!("Rate limiting check for registration endpoint from IP: {}", client_ip);

    match rate_limiter.check_rate_limit(&client_ip, "registration").await {
        Ok(result) => {
            if !result.allowed {
                warn!(
                    "Rate limit exceeded for registration endpoint from IP: {} ({}/{})",
                    client_ip, result.current_count, result.limit
                );

                // Return structured error response matching AppError format
                let error_response = crate::error::ErrorResponse {
                    error: "Rate limit exceeded".to_string(),
                    error_code: "RATE_LIMIT_EXCEEDED".to_string(),
                    message: "Too many registration attempts. Please try again later.".to_string(),
                    details: Some(json!({
                        "retry_after_seconds": result.retry_after,
                        "limit": result.limit,
                        "reset_time": result.reset_time,
                        "current_count": result.current_count
                    })),
                    correlation_id: uuid::Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };

                return Err((StatusCode::TOO_MANY_REQUESTS, Json(error_response)));
            }

            debug!(
                "Rate limit check passed for registration IP: {} ({}/{})",
                client_ip, result.current_count, result.limit
            );

            // Add rate limit headers to response
            let mut response = next.run(request).await;
            let headers = response.headers_mut();
            
            headers.insert("X-RateLimit-Limit", result.limit.to_string().parse().unwrap());
            headers.insert("X-RateLimit-Remaining", 
                (result.limit.saturating_sub(result.current_count)).to_string().parse().unwrap());
            headers.insert("X-RateLimit-Reset", result.reset_time.to_string().parse().unwrap());

            Ok(response)
        }
        Err(e) => {
            error!("Registration rate limiting error: {}", e);
            // Allow request to proceed if rate limiting fails
            Ok(next.run(request).await)
        }
    }
}

/// General API rate limiting middleware
pub async fn api_rate_limit_middleware(
    State(rate_limiter): State<Arc<RateLimitService>>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let client_ip = extract_client_ip(request.headers(), connect_info.as_ref());
    
    debug!("Rate limiting check for API endpoint from IP: {}", client_ip);

    match rate_limiter.check_rate_limit(&client_ip, "api").await {
        Ok(result) => {
            if !result.allowed {
                warn!(
                    "Rate limit exceeded for API endpoint from IP: {} ({}/{})",
                    client_ip, result.current_count, result.limit
                );

                let response = Json(json!({
                    "error": "Rate limit exceeded",
                    "message": "Too many API requests. Please try again later.",
                    "retry_after": result.retry_after,
                    "limit": result.limit,
                    "reset_time": result.reset_time
                }));

                return Err((StatusCode::TOO_MANY_REQUESTS, response));
            }

            // Add rate limit headers to response
            let mut response = next.run(request).await;
            let headers = response.headers_mut();
            
            headers.insert("X-RateLimit-Limit", result.limit.to_string().parse().unwrap());
            headers.insert("X-RateLimit-Remaining", 
                (result.limit.saturating_sub(result.current_count)).to_string().parse().unwrap());
            headers.insert("X-RateLimit-Reset", result.reset_time.to_string().parse().unwrap());

            Ok(response)
        }
        Err(e) => {
            error!("Rate limiting error: {}", e);
            // Allow request to proceed if rate limiting fails
            Ok(next.run(request).await)
        }
    }
}

/// IP-based brute force protection middleware
pub async fn brute_force_protection_middleware(
    State(rate_limiter): State<Arc<RateLimitService>>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let client_ip = extract_client_ip(request.headers(), connect_info.as_ref());
    
    // Check for brute force patterns (very restrictive limits)
    let brute_force_key = format!("brute_force:{}", client_ip);
    
    match rate_limiter.check_rate_limit(&brute_force_key, "auth").await {
        Ok(result) => {
            if !result.allowed {
                warn!(
                    "Potential brute force attack detected from IP: {} - blocking for {} seconds",
                    client_ip, result.retry_after
                );

                let response = Json(json!({
                    "error": "Security violation detected",
                    "message": "Your IP has been temporarily blocked due to suspicious activity.",
                    "retry_after": result.retry_after,
                    "contact": "Please contact support if you believe this is an error."
                }));

                return Err((StatusCode::TOO_MANY_REQUESTS, response));
            }

            Ok(next.run(request).await)
        }
        Err(e) => {
            error!("Brute force protection error: {}", e);
            // Allow request to proceed if protection fails
            Ok(next.run(request).await)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    #[ignore] // Requires Redis connection
    async fn test_rate_limiting_basic() {
        let redis_url = "redis://localhost:6379";
        let rate_limiter = RateLimitService::new(redis_url).unwrap();

        // First request should be allowed
        let result1 = rate_limiter.check_rate_limit("test_ip", "auth").await.unwrap();
        assert!(result1.allowed);
        assert_eq!(result1.current_count, 1);

        // Multiple requests within limit should be allowed
        for i in 2..=5 {
            let result = rate_limiter.check_rate_limit("test_ip", "auth").await.unwrap();
            assert!(result.allowed);
            assert_eq!(result.current_count, i);
        }
    }

    #[tokio::test]
    #[ignore] // Requires Redis connection
    async fn test_rate_limiting_exceeded() {
        let redis_url = "redis://localhost:6379";
        let rate_limiter = RateLimitService::new(redis_url).unwrap();

        // Make requests up to the limit (10 for auth)
        for i in 1..=10 {
            let result = rate_limiter.check_rate_limit("test_ip_2", "auth").await.unwrap();
            assert!(result.allowed);
            assert_eq!(result.current_count, i);
        }

        // Next request should be denied
        let result = rate_limiter.check_rate_limit("test_ip_2", "auth").await.unwrap();
        assert!(!result.allowed);
        assert_eq!(result.current_count, 11);
        assert!(result.retry_after > 0);
    }

    #[tokio::test]
    #[ignore] // Requires Redis connection
    async fn test_rate_limiting_window_reset() {
        let redis_url = "redis://localhost:6379";
        let mut rate_limiter = RateLimitService::new(redis_url).unwrap();

        // Set a very short window for testing
        rate_limiter.update_config("test".to_string(), RateLimitConfig {
            requests_per_window: 2,
            window_seconds: 1,
            burst_allowance: 0,
        }).await;

        // Make requests up to limit
        let result1 = rate_limiter.check_rate_limit("test_ip_3", "test").await.unwrap();
        assert!(result1.allowed);

        let result2 = rate_limiter.check_rate_limit("test_ip_3", "test").await.unwrap();
        assert!(result2.allowed);

        // Should be rate limited now
        let result3 = rate_limiter.check_rate_limit("test_ip_3", "test").await.unwrap();
        assert!(!result3.allowed);

        // Wait for window to reset
        sleep(Duration::from_secs(2)).await;

        // Should be allowed again
        let result4 = rate_limiter.check_rate_limit("test_ip_3", "test").await.unwrap();
        assert!(result4.allowed);
        assert_eq!(result4.current_count, 1);
    }
}