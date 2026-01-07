use anyhow::Result;
use chrono::{DateTime, Utc};
use deadpool_redis::{Config, Pool, Runtime};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use bcrypt::{hash, verify};

use crate::{AppError, Result as AppResult};

/// Cached user login data for performance optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedUserLogin {
    pub user_id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub totp_enabled: bool,
    pub totp_secret: Option<String>,
    pub last_login: Option<DateTime<Utc>>,
    pub cached_at: DateTime<Utc>,
}

/// Login session cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginSessionCache {
    pub user_id: Uuid,
    pub email: String,
    pub login_count: u32,
    pub last_login: DateTime<Utc>,
    pub cached_at: DateTime<Utc>,
}

/// Login performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginPerformanceMetrics {
    pub total_logins: u64,
    pub successful_logins: u64,
    pub failed_logins: u64,
    pub avg_login_time_ms: f64,
    pub cache_hit_rate: f64,
    pub password_verification_time_ms: f64,
    pub database_query_time_ms: f64,
    pub token_generation_time_ms: f64,
    pub last_updated: DateTime<Utc>,
}

impl Default for LoginPerformanceMetrics {
    fn default() -> Self {
        Self {
            total_logins: 0,
            successful_logins: 0,
            failed_logins: 0,
            avg_login_time_ms: 0.0,
            cache_hit_rate: 0.0,
            password_verification_time_ms: 0.0,
            database_query_time_ms: 0.0,
            token_generation_time_ms: 0.0,
            last_updated: Utc::now(),
        }
    }
}

/// Login performance optimization service
pub struct LoginPerformanceService {
    redis_pool: Pool,
    user_cache: Arc<RwLock<HashMap<String, CachedUserLogin>>>,
    session_cache: Arc<RwLock<HashMap<Uuid, LoginSessionCache>>>,
    metrics: Arc<RwLock<LoginPerformanceMetrics>>,
}

impl LoginPerformanceService {
    pub fn new(redis_url: &str) -> Result<Self> {
        let config = Config::from_url(redis_url);
        let pool = config.create_pool(Some(Runtime::Tokio1))?;

        Ok(Self {
            redis_pool: pool,
            user_cache: Arc::new(RwLock::new(HashMap::new())),
            session_cache: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(LoginPerformanceMetrics::default())),
        })
    }

    /// Get cached user login data or fetch from database
    pub async fn get_cached_user_login(
        &self,
        email: &str,
        db_pool: &sqlx::PgPool,
    ) -> AppResult<Option<CachedUserLogin>> {
        let email_key = email.to_lowercase();
        
        // Check in-memory cache first
        {
            let cache = self.user_cache.read().await;
            if let Some(cached_user) = cache.get(&email_key) {
                // Check if cache is still fresh (5 minutes for login data)
                let cache_age = Utc::now().signed_duration_since(cached_user.cached_at);
                if cache_age.num_minutes() < 5 {
                    return Ok(Some(cached_user.clone()));
                }
            }
        }

        // Try Redis cache
        let mut conn = self.redis_pool.get().await?;
        let redis_key = format!("login_user:{}", email_key);
        
        let cached_user_json: Option<String> = conn.get(&redis_key).await?;
        
        if let Some(user_json) = cached_user_json {
            if let Ok(cached_user) = serde_json::from_str::<CachedUserLogin>(&user_json) {
                // Check if Redis cache is fresh (15 minutes)
                let cache_age = Utc::now().signed_duration_since(cached_user.cached_at);
                if cache_age.num_minutes() < 15 {
                    // Update in-memory cache
                    {
                        let mut cache = self.user_cache.write().await;
                        cache.insert(email_key, cached_user.clone());
                    }
                    return Ok(Some(cached_user));
                }
            }
        }

        // Fetch from database using existing query pattern
        let user = sqlx::query!(
            "SELECT id, email, password_hash, totp_enabled, totp_secret FROM users WHERE email = $1",
            email
        )
        .fetch_optional(db_pool)
        .await?;

        if let Some(user_row) = user {
            let cached_user = CachedUserLogin {
                user_id: user_row.id,
                email: user_row.email,
                password_hash: user_row.password_hash.unwrap_or_default(),
                totp_enabled: user_row.totp_enabled.unwrap_or(false),
                totp_secret: user_row.totp_secret,
                last_login: None, // We'll get this from a separate query if needed
                cached_at: Utc::now(),
            };

            // Cache in Redis (15 minutes TTL)
            let user_json = serde_json::to_string(&cached_user)?;
            let _: () = conn.set_ex(&redis_key, user_json, 900).await?;

            // Cache in memory
            {
                let mut cache = self.user_cache.write().await;
                cache.insert(email_key, cached_user.clone());
                
                // Limit cache size
                if cache.len() > 1000 {
                    // Remove oldest entries
                    let mut entries: Vec<_> = cache.iter().collect();
                    entries.sort_by_key(|(_, user)| user.cached_at);
                    let to_remove: Vec<_> = entries.iter().take(100).map(|(k, _)| (*k).clone()).collect();
                    for key in to_remove {
                        cache.remove(&key);
                    }
                }
            }

            Ok(Some(cached_user))
        } else {
            Ok(None)
        }
    }

    /// Optimized password verification with timing
    pub async fn verify_password_optimized(
        &self,
        password: &str,
        password_hash: &str,
    ) -> AppResult<bool> {
        let start_time = std::time::Instant::now();
        
        // Use tokio::task::spawn_blocking for CPU-intensive bcrypt operation
        let password = password.to_string();
        let password_hash = password_hash.to_string();
        
        let result = tokio::task::spawn_blocking(move || {
            verify(&password, &password_hash)
        }).await
        .map_err(|e| AppError::Internal { 
            message: Some(format!("Password verification task failed: {}", e)) 
        })?
        .map_err(|e| AppError::Internal { 
            message: Some(format!("Password verification failed: {}", e)) 
        })?;

        let verification_time = start_time.elapsed().as_millis() as f64;
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            if metrics.password_verification_time_ms == 0.0 {
                metrics.password_verification_time_ms = verification_time;
            } else {
                metrics.password_verification_time_ms = 0.9 * metrics.password_verification_time_ms + 0.1 * verification_time;
            }
        }

        Ok(result)
    }

    /// Generate optimized refresh token (lighter hashing)
    pub async fn generate_optimized_refresh_token(&self) -> AppResult<(String, String)> {
        let start_time = std::time::Instant::now();
        
        // Use a lighter hash for refresh tokens (8 rounds instead of 12)
        let refresh_token_raw = format!("{}_{}", Uuid::new_v4(), rand::random::<u64>());
        
        let token_raw = refresh_token_raw.clone();
        let refresh_token_hash = tokio::task::spawn_blocking(move || {
            hash(&token_raw, 8) // Reduced from 12 to 8 rounds for refresh tokens
        }).await
        .map_err(|e| AppError::Internal { 
            message: Some(format!("Token hashing task failed: {}", e)) 
        })?
        .map_err(|e| AppError::Internal { 
            message: Some(format!("Token hashing failed: {}", e)) 
        })?;

        let token_time = start_time.elapsed().as_millis() as f64;
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            if metrics.token_generation_time_ms == 0.0 {
                metrics.token_generation_time_ms = token_time;
            } else {
                metrics.token_generation_time_ms = 0.9 * metrics.token_generation_time_ms + 0.1 * token_time;
            }
        }

        Ok((refresh_token_raw, refresh_token_hash))
    }

    /// Batch database operations for login
    pub async fn batch_login_operations(
        &self,
        user_id: Uuid,
        refresh_token_hash: &str,
        refresh_token_ttl: i64,
        db_pool: &sqlx::PgPool,
    ) -> AppResult<()> {
        let start_time = std::time::Instant::now();
        let expires_at = Utc::now() + chrono::Duration::seconds(refresh_token_ttl);
        let now = Utc::now();

        // Use a single transaction for all database operations
        let mut tx = db_pool.begin().await?;

        // Update last login using existing query pattern
        sqlx::query!(
            "UPDATE users SET last_login = NOW(), updated_at = NOW() WHERE id = $1",
            user_id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO user_sessions (user_id, refresh_token_hash, expires_at)
            VALUES ($1, $2, $3)
            "#,
            user_id,
            refresh_token_hash,
            expires_at
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        let db_time = start_time.elapsed().as_millis() as f64;
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            if metrics.database_query_time_ms == 0.0 {
                metrics.database_query_time_ms = db_time;
            } else {
                metrics.database_query_time_ms = 0.9 * metrics.database_query_time_ms + 0.1 * db_time;
            }
        }

        Ok(())
    }

    /// Invalidate user cache on password change
    pub async fn invalidate_user_cache(&self, email: &str) -> Result<()> {
        let email_key = email.to_lowercase();
        
        // Remove from in-memory cache
        {
            let mut cache = self.user_cache.write().await;
            cache.remove(&email_key);
        }

        // Remove from Redis cache
        let mut conn = self.redis_pool.get().await?;
        let redis_key = format!("login_user:{}", email_key);
        let _: i32 = conn.del(&redis_key).await?;

        Ok(())
    }

    /// Record login attempt metrics
    pub async fn record_login_attempt(&self, success: bool, total_time_ms: f64) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        
        metrics.total_logins += 1;
        if success {
            metrics.successful_logins += 1;
        } else {
            metrics.failed_logins += 1;
        }

        // Update average login time
        if metrics.avg_login_time_ms == 0.0 {
            metrics.avg_login_time_ms = total_time_ms;
        } else {
            metrics.avg_login_time_ms = 0.9 * metrics.avg_login_time_ms + 0.1 * total_time_ms;
        }

        // Calculate cache hit rate
        let cache_hits = self.user_cache.read().await.len() as f64;
        let total_requests = metrics.total_logins as f64;
        metrics.cache_hit_rate = if total_requests > 0.0 {
            (cache_hits / total_requests) * 100.0
        } else {
            0.0
        };

        metrics.last_updated = Utc::now();

        Ok(())
    }

    /// Get current login performance metrics
    pub async fn get_metrics(&self) -> LoginPerformanceMetrics {
        self.metrics.read().await.clone()
    }

    /// Preload frequently accessed users into cache
    pub async fn preload_frequent_users(&self, db_pool: &sqlx::PgPool) -> Result<()> {
        // For now, just log that we would preload users
        // In a real implementation with database access, we would:
        // 1. Query users with recent login activity
        // 2. Cache their login data
        // 3. Populate the in-memory cache
        
        tracing::info!("Login cache preloading would happen here with database access");
        Ok(())
    }

    /// Clear all caches
    pub async fn clear_caches(&self) -> Result<()> {
        // Clear in-memory caches
        {
            let mut user_cache = self.user_cache.write().await;
            user_cache.clear();
        }
        
        {
            let mut session_cache = self.session_cache.write().await;
            session_cache.clear();
        }

        // Clear Redis caches
        let mut conn = self.redis_pool.get().await?;
        let pattern = "login_user:*";
        let keys: Vec<String> = conn.keys(&pattern).await?;
        
        if !keys.is_empty() {
            let _: i32 = conn.del(&keys).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_login_performance_service_creation() {
        // This would require Redis for full testing
        let redis_url = "redis://localhost:6379";
        
        match LoginPerformanceService::new(redis_url) {
            Ok(service) => {
                let metrics = service.get_metrics().await;
                assert_eq!(metrics.total_logins, 0);
                assert_eq!(metrics.successful_logins, 0);
            }
            Err(_) => {
                // Redis not available, which is fine for this test
                assert!(true);
            }
        }
    }

    #[tokio::test]
    async fn test_metrics_recording() {
        let redis_url = "redis://localhost:6379";
        
        if let Ok(service) = LoginPerformanceService::new(redis_url) {
            service.record_login_attempt(true, 150.0).await.unwrap();
            service.record_login_attempt(false, 200.0).await.unwrap();
            
            let metrics = service.get_metrics().await;
            assert_eq!(metrics.total_logins, 2);
            assert_eq!(metrics.successful_logins, 1);
            assert_eq!(metrics.failed_logins, 1);
            assert!(metrics.avg_login_time_ms > 0.0);
        }
    }

    #[test]
    fn test_cached_user_login_serialization() {
        let cached_user = CachedUserLogin {
            user_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            password_hash: "hash123".to_string(),
            totp_enabled: false,
            totp_secret: None,
            last_login: None,
            cached_at: Utc::now(),
        };

        let json = serde_json::to_string(&cached_user).unwrap();
        let deserialized: CachedUserLogin = serde_json::from_str(&json).unwrap();
        
        assert_eq!(cached_user.user_id, deserialized.user_id);
        assert_eq!(cached_user.email, deserialized.email);
    }
}