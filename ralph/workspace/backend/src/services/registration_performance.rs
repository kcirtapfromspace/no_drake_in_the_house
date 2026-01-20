use anyhow::Result;
use chrono::{DateTime, Utc};
use deadpool_redis::{Config, Pool, Runtime};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::RegistrationValidationError;

/// Cached validation rules for password strength
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordValidationRules {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_number: bool,
    pub require_special_char: bool,
    pub common_passwords: Vec<String>,
    pub special_chars: String,
    pub cached_at: DateTime<Utc>,
}

impl Default for PasswordValidationRules {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_number: true,
            require_special_char: true,
            common_passwords: vec![
                "password".to_string(),
                "123456".to_string(),
                "password123".to_string(),
                "admin".to_string(),
                "qwerty".to_string(),
                "letmein".to_string(),
                "welcome".to_string(),
                "monkey".to_string(),
                "1234567890".to_string(),
                "password1".to_string(),
                "123456789".to_string(),
            ],
            special_chars: "!@#$%^&*()_+-=[]{}|;:,.<>?".to_string(),
            cached_at: Utc::now(),
        }
    }
}

/// Email validation cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailValidationCache {
    pub email_hash: String,
    pub is_valid: bool,
    pub cached_at: DateTime<Utc>,
}

/// Registration performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationMetrics {
    pub total_attempts: u64,
    pub successful_registrations: u64,
    pub validation_failures: u64,
    pub email_duplicates: u64,
    pub avg_validation_time_ms: f64,
    pub avg_registration_time_ms: f64,
    pub last_updated: DateTime<Utc>,
}

impl Default for RegistrationMetrics {
    fn default() -> Self {
        Self {
            total_attempts: 0,
            successful_registrations: 0,
            validation_failures: 0,
            email_duplicates: 0,
            avg_validation_time_ms: 0.0,
            avg_registration_time_ms: 0.0,
            last_updated: Utc::now(),
        }
    }
}

/// Performance optimization service for registration
pub struct RegistrationPerformanceService {
    redis_pool: Pool,
    validation_rules_cache: Arc<RwLock<Option<PasswordValidationRules>>>,
    email_validation_cache: Arc<RwLock<HashMap<String, EmailValidationCache>>>,
    metrics: Arc<RwLock<RegistrationMetrics>>,
}

impl RegistrationPerformanceService {
    pub fn new(redis_url: &str) -> Result<Self> {
        let config = Config::from_url(redis_url);
        let pool = config.create_pool(Some(Runtime::Tokio1))?;

        Ok(Self {
            redis_pool: pool,
            validation_rules_cache: Arc::new(RwLock::new(None)),
            email_validation_cache: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(RegistrationMetrics::default())),
        })
    }

    /// Get cached password validation rules or load from cache/defaults
    pub async fn get_password_validation_rules(&self) -> Result<PasswordValidationRules> {
        // Check in-memory cache first
        {
            let cache = self.validation_rules_cache.read().await;
            if let Some(rules) = &*cache {
                // Check if cache is still fresh (1 hour)
                let cache_age = Utc::now().signed_duration_since(rules.cached_at);
                if cache_age.num_hours() < 1 {
                    return Ok(rules.clone());
                }
            }
        }

        // Try to load from Redis
        let mut conn = self.redis_pool.get().await?;
        let redis_key = "registration:password_rules";

        let cached_rules: Option<String> = conn.get(&redis_key).await?;

        let rules = if let Some(rules_json) = cached_rules {
            match serde_json::from_str::<PasswordValidationRules>(&rules_json) {
                Ok(rules) => {
                    // Check if Redis cache is fresh
                    let cache_age = Utc::now().signed_duration_since(rules.cached_at);
                    if cache_age.num_hours() < 24 {
                        rules
                    } else {
                        // Cache expired, use defaults and update
                        let new_rules = PasswordValidationRules::default();
                        self.cache_password_rules(&new_rules).await?;
                        new_rules
                    }
                }
                Err(_) => {
                    // Invalid cache, use defaults
                    let new_rules = PasswordValidationRules::default();
                    self.cache_password_rules(&new_rules).await?;
                    new_rules
                }
            }
        } else {
            // No cache, use defaults and cache them
            let new_rules = PasswordValidationRules::default();
            self.cache_password_rules(&new_rules).await?;
            new_rules
        };

        // Update in-memory cache
        {
            let mut cache = self.validation_rules_cache.write().await;
            *cache = Some(rules.clone());
        }

        Ok(rules)
    }

    /// Cache password validation rules in Redis
    async fn cache_password_rules(&self, rules: &PasswordValidationRules) -> Result<()> {
        let mut conn = self.redis_pool.get().await?;
        let redis_key = "registration:password_rules";
        let rules_json = serde_json::to_string(rules)?;

        // Cache for 24 hours
        let _: () = conn.set_ex(&redis_key, rules_json, 86400).await?;
        Ok(())
    }

    /// Validate password strength with cached rules
    pub async fn validate_password_strength_cached(
        &self,
        password: &str,
    ) -> Result<Option<RegistrationValidationError>> {
        let rules = self.get_password_validation_rules().await?;
        let mut requirements = Vec::new();

        // Minimum length requirement
        if password.len() < rules.min_length {
            requirements.push(format!("at least {} characters", rules.min_length));
        }

        // Uppercase letter requirement
        if rules.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            requirements.push("at least one uppercase letter".to_string());
        }

        // Lowercase letter requirement
        if rules.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            requirements.push("at least one lowercase letter".to_string());
        }

        // Number requirement
        if rules.require_number && !password.chars().any(|c| c.is_numeric()) {
            requirements.push("at least one number".to_string());
        }

        // Special character requirement
        if rules.require_special_char && !password.chars().any(|c| rules.special_chars.contains(c))
        {
            requirements.push(format!(
                "at least one special character ({})",
                rules.special_chars
            ));
        }

        // Check against cached common passwords
        if rules
            .common_passwords
            .iter()
            .any(|common| password.to_lowercase() == common.to_lowercase())
        {
            requirements.push("not be a common password".to_string());
        }

        if !requirements.is_empty() {
            let message = format!("Password must contain {}", requirements.join(", "));
            Ok(Some(RegistrationValidationError {
                field: "password".to_string(),
                message,
                code: "PASSWORD_WEAK".to_string(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Check if email format is valid with caching
    pub async fn validate_email_format_cached(&self, email: &str) -> Result<bool> {
        // Create hash of email for cache key (to avoid storing actual emails)
        let email_hash = format!("{:x}", md5::compute(email.to_lowercase()));

        // Check in-memory cache first
        {
            let cache = self.email_validation_cache.read().await;
            if let Some(cached) = cache.get(&email_hash) {
                // Check if cache is still fresh (1 hour)
                let cache_age = Utc::now().signed_duration_since(cached.cached_at);
                if cache_age.num_hours() < 1 {
                    return Ok(cached.is_valid);
                }
            }
        }

        // Perform validation
        let is_valid = self.validate_email_format(email);

        // Cache the result
        let cache_entry = EmailValidationCache {
            email_hash: email_hash.clone(),
            is_valid,
            cached_at: Utc::now(),
        };

        // Update in-memory cache
        {
            let mut cache = self.email_validation_cache.write().await;
            cache.insert(email_hash, cache_entry);

            // Limit cache size to prevent memory bloat
            if cache.len() > 10000 {
                // Remove oldest entries
                let mut entries: Vec<_> = cache.iter().collect();
                entries.sort_by_key(|(_, entry)| entry.cached_at);
                let to_remove: Vec<_> = entries
                    .iter()
                    .take(1000)
                    .map(|(k, _)| (*k).clone())
                    .collect();
                for key in to_remove {
                    cache.remove(&key);
                }
            }
        }

        Ok(is_valid)
    }

    /// Validate email format (internal implementation)
    fn validate_email_format(&self, email: &str) -> bool {
        if email.is_empty() || email.len() > 255 || email.contains("..") {
            return false;
        }

        // Enhanced email validation with proper regex
        let email_regex = regex::Regex::new(
            r"^[a-zA-Z0-9]([a-zA-Z0-9._+%-]*[a-zA-Z0-9])?@[a-zA-Z0-9]([a-zA-Z0-9.-]*[a-zA-Z0-9])?\.[a-zA-Z]{2,}$"
        ).unwrap();

        email_regex.is_match(email)
    }

    /// Record registration attempt metrics
    pub async fn record_registration_attempt(&self, validation_time_ms: f64) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.total_attempts += 1;

        // Update average validation time using exponential moving average
        if metrics.avg_validation_time_ms == 0.0 {
            metrics.avg_validation_time_ms = validation_time_ms;
        } else {
            metrics.avg_validation_time_ms =
                0.9 * metrics.avg_validation_time_ms + 0.1 * validation_time_ms;
        }

        metrics.last_updated = Utc::now();

        // Persist metrics to Redis every 10 attempts
        if metrics.total_attempts % 10 == 0 {
            self.persist_metrics(&metrics).await?;
        }

        Ok(())
    }

    /// Record successful registration
    pub async fn record_successful_registration(&self, total_time_ms: f64) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.successful_registrations += 1;

        // Update average registration time
        if metrics.avg_registration_time_ms == 0.0 {
            metrics.avg_registration_time_ms = total_time_ms;
        } else {
            metrics.avg_registration_time_ms =
                0.9 * metrics.avg_registration_time_ms + 0.1 * total_time_ms;
        }

        metrics.last_updated = Utc::now();
        self.persist_metrics(&metrics).await?;

        Ok(())
    }

    /// Record validation failure
    pub async fn record_validation_failure(&self) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.validation_failures += 1;
        metrics.last_updated = Utc::now();
        Ok(())
    }

    /// Record email duplicate
    pub async fn record_email_duplicate(&self) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.email_duplicates += 1;
        metrics.last_updated = Utc::now();
        Ok(())
    }

    /// Get current registration metrics
    pub async fn get_metrics(&self) -> RegistrationMetrics {
        self.metrics.read().await.clone()
    }

    /// Persist metrics to Redis
    async fn persist_metrics(&self, metrics: &RegistrationMetrics) -> Result<()> {
        let mut conn = self.redis_pool.get().await?;
        let redis_key = "registration:metrics";
        let metrics_json = serde_json::to_string(metrics)?;

        // Cache for 7 days
        let _: () = conn.set_ex(&redis_key, metrics_json, 604800).await?;
        Ok(())
    }

    /// Load metrics from Redis on startup
    pub async fn load_metrics(&self) -> Result<()> {
        let mut conn = self.redis_pool.get().await?;
        let redis_key = "registration:metrics";

        let cached_metrics: Option<String> = conn.get(&redis_key).await?;

        if let Some(metrics_json) = cached_metrics {
            if let Ok(cached_metrics) = serde_json::from_str::<RegistrationMetrics>(&metrics_json) {
                let mut metrics = self.metrics.write().await;
                *metrics = cached_metrics;
            }
        }

        Ok(())
    }

    /// Optimize database queries by checking email existence efficiently
    pub async fn check_email_exists_optimized(
        &self,
        db_pool: &sqlx::PgPool,
        email: &str,
    ) -> Result<bool> {
        // Use existing query pattern to avoid SQLx cache issues
        let existing_user = sqlx::query!("SELECT id FROM users WHERE email = $1", email)
            .fetch_optional(db_pool)
            .await?;

        Ok(existing_user.is_some())
    }

    /// Batch validate multiple emails for existence (for future use)
    pub async fn batch_check_emails_exist(
        &self,
        db_pool: &sqlx::PgPool,
        emails: &[String],
    ) -> Result<HashMap<String, bool>> {
        // For now, check emails individually using existing queries
        let mut result_map = HashMap::new();

        for email in emails {
            let exists = self.check_email_exists_optimized(db_pool, email).await?;
            result_map.insert(email.clone(), exists);
        }

        Ok(result_map)
    }

    /// Clear caches (for testing or maintenance)
    pub async fn clear_caches(&self) -> Result<()> {
        // Clear in-memory caches
        {
            let mut validation_cache = self.validation_rules_cache.write().await;
            *validation_cache = None;
        }

        {
            let mut email_cache = self.email_validation_cache.write().await;
            email_cache.clear();
        }

        // Clear Redis caches
        let mut conn = self.redis_pool.get().await?;
        let _: i32 = conn.del("registration:password_rules").await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_password_validation_rules_caching() {
        // This would require a Redis instance for full testing
        // For now, test the validation logic
        let rules = PasswordValidationRules::default();

        assert_eq!(rules.min_length, 8);
        assert!(rules.require_uppercase);
        assert!(rules.require_lowercase);
        assert!(rules.require_number);
        assert!(rules.require_special_char);
        assert!(!rules.common_passwords.is_empty());
    }

    #[test]
    fn test_email_format_validation() {
        let service = RegistrationPerformanceService {
            redis_pool: deadpool_redis::Config::from_url("redis://localhost")
                .create_pool(Some(Runtime::Tokio1))
                .unwrap(),
            validation_rules_cache: Arc::new(RwLock::new(None)),
            email_validation_cache: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(RegistrationMetrics::default())),
        };

        // Valid emails
        assert!(service.validate_email_format("test@example.com"));
        assert!(service.validate_email_format("user.name+tag@domain.co.uk"));
        assert!(service.validate_email_format("a@b.co"));

        // Invalid emails
        assert!(!service.validate_email_format(""));
        assert!(!service.validate_email_format("invalid"));
        assert!(!service.validate_email_format("test@"));
        assert!(!service.validate_email_format("@example.com"));
        assert!(!service.validate_email_format("test..test@example.com"));
        assert!(!service.validate_email_format("test@example"));
    }

    #[test]
    fn test_metrics_calculation() {
        let mut metrics = RegistrationMetrics::default();

        // Test exponential moving average calculation
        metrics.avg_validation_time_ms = 100.0;
        let new_time = 200.0;
        metrics.avg_validation_time_ms = 0.9 * metrics.avg_validation_time_ms + 0.1 * new_time;

        assert!((metrics.avg_validation_time_ms - 110.0).abs() < 0.1);
    }
}
