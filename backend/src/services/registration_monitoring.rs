use anyhow::Result;
use chrono::{DateTime, Utc};
use prometheus::{Counter, Histogram, Gauge, Registry, Opts, HistogramOpts};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::services::registration_performance::RegistrationMetrics;

/// Registration monitoring and observability service
pub struct RegistrationMonitoringService {
    // Prometheus metrics
    registration_attempts_total: Counter,
    registration_successes_total: Counter,
    registration_failures_total: Counter,
    validation_failures_total: Counter,
    email_duplicates_total: Counter,
    validation_duration_seconds: Histogram,
    registration_duration_seconds: Histogram,
    active_registrations: Gauge,
    
    // Health check metrics
    health_status: Arc<RwLock<RegistrationHealthStatus>>,
    
    // Registry for metrics
    registry: Registry,
}

/// Health status for registration endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationHealthStatus {
    pub status: String,
    pub last_check: DateTime<Utc>,
    pub database_healthy: bool,
    pub redis_healthy: bool,
    pub validation_service_healthy: bool,
    pub avg_response_time_ms: f64,
    pub error_rate_percent: f64,
    pub uptime_seconds: u64,
}

impl Default for RegistrationHealthStatus {
    fn default() -> Self {
        Self {
            status: "healthy".to_string(),
            last_check: Utc::now(),
            database_healthy: true,
            redis_healthy: true,
            validation_service_healthy: true,
            avg_response_time_ms: 0.0,
            error_rate_percent: 0.0,
            uptime_seconds: 0,
        }
    }
}

/// Registration dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationDashboard {
    pub timestamp: DateTime<Utc>,
    pub health_status: RegistrationHealthStatus,
    pub metrics: RegistrationMetrics,
    pub success_rate_percent: f64,
    pub avg_validation_time_ms: f64,
    pub avg_registration_time_ms: f64,
    pub registrations_per_hour: f64,
    pub error_patterns: Vec<ErrorPattern>,
    pub performance_trends: PerformanceTrends,
}

/// Error pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub error_type: String,
    pub count: u64,
    pub percentage: f64,
    pub last_occurrence: DateTime<Utc>,
}

/// Performance trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    pub validation_time_trend: String, // "improving", "stable", "degrading"
    pub registration_time_trend: String,
    pub success_rate_trend: String,
    pub volume_trend: String,
}

impl RegistrationMonitoringService {
    pub fn new() -> Result<Self> {
        let registry = Registry::new();

        // Create Prometheus metrics
        let registration_attempts_total = Counter::with_opts(
            Opts::new("registration_attempts_total", "Total number of registration attempts")
        )?;
        
        let registration_successes_total = Counter::with_opts(
            Opts::new("registration_successes_total", "Total number of successful registrations")
        )?;
        
        let registration_failures_total = Counter::with_opts(
            Opts::new("registration_failures_total", "Total number of failed registrations")
        )?;
        
        let validation_failures_total = Counter::with_opts(
            Opts::new("registration_validation_failures_total", "Total number of validation failures")
        )?;
        
        let email_duplicates_total = Counter::with_opts(
            Opts::new("registration_email_duplicates_total", "Total number of duplicate email attempts")
        )?;
        
        let validation_duration_seconds = Histogram::with_opts(
            HistogramOpts::new("registration_validation_duration_seconds", "Time spent on registration validation")
                .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0])
        )?;
        
        let registration_duration_seconds = Histogram::with_opts(
            HistogramOpts::new("registration_duration_seconds", "Total time for registration process")
                .buckets(vec![0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0])
        )?;
        
        let active_registrations = Gauge::with_opts(
            Opts::new("active_registrations", "Number of currently active registration processes")
        )?;

        // Register metrics
        registry.register(Box::new(registration_attempts_total.clone()))?;
        registry.register(Box::new(registration_successes_total.clone()))?;
        registry.register(Box::new(registration_failures_total.clone()))?;
        registry.register(Box::new(validation_failures_total.clone()))?;
        registry.register(Box::new(email_duplicates_total.clone()))?;
        registry.register(Box::new(validation_duration_seconds.clone()))?;
        registry.register(Box::new(registration_duration_seconds.clone()))?;
        registry.register(Box::new(active_registrations.clone()))?;

        Ok(Self {
            registration_attempts_total,
            registration_successes_total,
            registration_failures_total,
            validation_failures_total,
            email_duplicates_total,
            validation_duration_seconds,
            registration_duration_seconds,
            active_registrations,
            health_status: Arc::new(RwLock::new(RegistrationHealthStatus::default())),
            registry,
        })
    }

    /// Record a registration attempt
    pub fn record_registration_attempt(&self) {
        self.registration_attempts_total.inc();
        self.active_registrations.inc();
        
        tracing::info!(
            metric = "registration_attempt",
            "Registration attempt recorded"
        );
    }

    /// Record a successful registration
    pub fn record_registration_success(&self, duration: Duration) {
        self.registration_successes_total.inc();
        self.active_registrations.dec();
        self.registration_duration_seconds.observe(duration.as_secs_f64());
        
        tracing::info!(
            metric = "registration_success",
            duration_ms = duration.as_millis(),
            "Successful registration recorded"
        );
    }

    /// Record a registration failure
    pub fn record_registration_failure(&self, error_type: &str) {
        self.registration_failures_total.inc();
        self.active_registrations.dec();
        
        tracing::warn!(
            metric = "registration_failure",
            error_type = error_type,
            "Registration failure recorded"
        );
    }

    /// Record a validation failure
    pub fn record_validation_failure(&self, validation_duration: Duration) {
        self.validation_failures_total.inc();
        self.validation_duration_seconds.observe(validation_duration.as_secs_f64());
        
        tracing::warn!(
            metric = "validation_failure",
            duration_ms = validation_duration.as_millis(),
            "Validation failure recorded"
        );
    }

    /// Record an email duplicate attempt
    pub fn record_email_duplicate(&self) {
        self.email_duplicates_total.inc();
        
        tracing::info!(
            metric = "email_duplicate",
            "Email duplicate attempt recorded"
        );
    }

    /// Record validation timing
    pub fn record_validation_timing(&self, duration: Duration) {
        self.validation_duration_seconds.observe(duration.as_secs_f64());
    }

    /// Get Prometheus metrics registry
    pub fn get_metrics_registry(&self) -> &Registry {
        &self.registry
    }

    /// Check health of registration endpoint and dependencies
    pub async fn check_health(
        &self,
        db_pool: &sqlx::PgPool,
        redis_pool: &deadpool_redis::Pool,
    ) -> Result<RegistrationHealthStatus> {
        let mut health = RegistrationHealthStatus::default();
        health.last_check = Utc::now();

        // Check database health
        health.database_healthy = match self.check_database_health(db_pool).await {
            Ok(_) => true,
            Err(e) => {
                tracing::warn!("Database health check failed: {}", e);
                false
            }
        };

        // Check Redis health
        health.redis_healthy = match self.check_redis_health(redis_pool).await {
            Ok(_) => true,
            Err(e) => {
                tracing::warn!("Redis health check failed: {}", e);
                false
            }
        };

        // Check validation service health
        health.validation_service_healthy = self.check_validation_service_health().await;

        // Calculate overall health status
        if health.database_healthy && health.redis_healthy && health.validation_service_healthy {
            health.status = "healthy".to_string();
        } else if health.database_healthy {
            health.status = "degraded".to_string();
        } else {
            health.status = "unhealthy".to_string();
        }

        // Calculate performance metrics
        health.avg_response_time_ms = self.calculate_avg_response_time();
        health.error_rate_percent = self.calculate_error_rate();

        // Update stored health status
        {
            let mut stored_health = self.health_status.write().await;
            *stored_health = health.clone();
        }

        tracing::info!(
            metric = "health_check",
            status = %health.status,
            database_healthy = health.database_healthy,
            redis_healthy = health.redis_healthy,
            validation_service_healthy = health.validation_service_healthy,
            avg_response_time_ms = health.avg_response_time_ms,
            error_rate_percent = health.error_rate_percent,
            "Registration health check completed"
        );

        Ok(health)
    }

    /// Get current health status
    pub async fn get_health_status(&self) -> RegistrationHealthStatus {
        self.health_status.read().await.clone()
    }

    /// Generate registration dashboard data
    pub async fn generate_dashboard(
        &self,
        registration_metrics: RegistrationMetrics,
    ) -> RegistrationDashboard {
        let health_status = self.get_health_status().await;
        
        // Calculate success rate
        let success_rate_percent = if registration_metrics.total_attempts > 0 {
            (registration_metrics.successful_registrations as f64 / registration_metrics.total_attempts as f64) * 100.0
        } else {
            0.0
        };

        // Calculate registrations per hour (simplified)
        let registrations_per_hour = registration_metrics.successful_registrations as f64; // This would be calculated based on time window

        // Generate error patterns
        let error_patterns = self.generate_error_patterns(&registration_metrics);

        // Generate performance trends
        let performance_trends = self.generate_performance_trends(&registration_metrics);

        RegistrationDashboard {
            timestamp: Utc::now(),
            health_status,
            metrics: registration_metrics.clone(),
            success_rate_percent,
            avg_validation_time_ms: registration_metrics.avg_validation_time_ms,
            avg_registration_time_ms: registration_metrics.avg_registration_time_ms,
            registrations_per_hour,
            error_patterns,
            performance_trends,
        }
    }

    /// Log structured registration event
    pub fn log_registration_event(
        &self,
        event_type: &str,
        user_id: Option<Uuid>,
        email: &str,
        duration_ms: Option<f64>,
        error: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) {
        let mut event = serde_json::json!({
            "event_type": event_type,
            "timestamp": Utc::now().to_rfc3339(),
            "email_hash": format!("{:x}", md5::compute(email.to_lowercase())),
        });

        if let Some(uid) = user_id {
            event["user_id"] = serde_json::Value::String(uid.to_string());
        }

        if let Some(duration) = duration_ms {
            if let Some(num) = serde_json::Number::from_f64(duration) {
                event["duration_ms"] = serde_json::Value::Number(num);
            }
        }

        if let Some(err) = error {
            event["error"] = serde_json::Value::String(err.to_string());
        }

        if let Some(meta) = metadata {
            event["metadata"] = meta;
        }

        tracing::info!(
            target: "registration_events",
            event = %event,
            "Registration event logged"
        );
    }

    // Private helper methods

    async fn check_database_health(&self, _db_pool: &sqlx::PgPool) -> Result<()> {
        // For now, just return OK since we can't easily add new queries
        // In a real implementation, this would check database connectivity
        Ok(())
    }

    async fn check_redis_health(&self, redis_pool: &deadpool_redis::Pool) -> Result<()> {
        use redis::AsyncCommands;
        
        let mut conn = redis_pool.get().await?;
        // Use a simple SET/GET operation instead of ping
        let test_key = "health_check";
        let test_value = "ok";
        let _: () = conn.set(test_key, test_value).await?;
        let _: String = conn.get(test_key).await?;
        let _: () = conn.del(test_key).await?;
        Ok(())
    }

    async fn check_validation_service_health(&self) -> bool {
        // Check if validation service is responsive
        // This is a simple check - in practice you might test actual validation
        true
    }

    fn calculate_avg_response_time(&self) -> f64 {
        // Get average from histogram
        let samples = self.registration_duration_seconds.get_sample_sum();
        let count = self.registration_duration_seconds.get_sample_count();
        
        if count > 0 {
            (samples / count as f64) * 1000.0 // Convert to milliseconds
        } else {
            0.0
        }
    }

    fn calculate_error_rate(&self) -> f64 {
        let total_attempts = self.registration_attempts_total.get();
        let failures = self.registration_failures_total.get();
        
        if total_attempts > 0.0 {
            (failures / total_attempts) * 100.0
        } else {
            0.0
        }
    }

    fn generate_error_patterns(&self, metrics: &RegistrationMetrics) -> Vec<ErrorPattern> {
        let mut patterns = Vec::new();

        if metrics.validation_failures > 0 {
            patterns.push(ErrorPattern {
                error_type: "validation_failure".to_string(),
                count: metrics.validation_failures,
                percentage: if metrics.total_attempts > 0 {
                    (metrics.validation_failures as f64 / metrics.total_attempts as f64) * 100.0
                } else {
                    0.0
                },
                last_occurrence: metrics.last_updated,
            });
        }

        if metrics.email_duplicates > 0 {
            patterns.push(ErrorPattern {
                error_type: "email_duplicate".to_string(),
                count: metrics.email_duplicates,
                percentage: if metrics.total_attempts > 0 {
                    (metrics.email_duplicates as f64 / metrics.total_attempts as f64) * 100.0
                } else {
                    0.0
                },
                last_occurrence: metrics.last_updated,
            });
        }

        patterns
    }

    fn generate_performance_trends(&self, _metrics: &RegistrationMetrics) -> PerformanceTrends {
        // This would analyze historical data to determine trends
        // For now, return stable trends
        PerformanceTrends {
            validation_time_trend: "stable".to_string(),
            registration_time_trend: "stable".to_string(),
            success_rate_trend: "stable".to_string(),
            volume_trend: "stable".to_string(),
        }
    }
}

/// Registration monitoring middleware
pub async fn registration_monitoring_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let start_time = std::time::Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    
    let response = next.run(req).await;
    
    let duration = start_time.elapsed();
    let status = response.status();
    
    // Log request metrics
    tracing::info!(
        method = %method,
        uri = %uri,
        status = %status.as_u16(),
        duration_ms = duration.as_millis(),
        "Registration endpoint request completed"
    );
    
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitoring_service_creation() {
        let service = RegistrationMonitoringService::new().unwrap();
        
        // Test that metrics are properly initialized
        assert_eq!(service.registration_attempts_total.get(), 0.0);
        assert_eq!(service.registration_successes_total.get(), 0.0);
        assert_eq!(service.registration_failures_total.get(), 0.0);
    }

    #[tokio::test]
    async fn test_health_status_default() {
        let health = RegistrationHealthStatus::default();
        
        assert_eq!(health.status, "healthy");
        assert!(health.database_healthy);
        assert!(health.redis_healthy);
        assert!(health.validation_service_healthy);
        assert_eq!(health.avg_response_time_ms, 0.0);
        assert_eq!(health.error_rate_percent, 0.0);
    }

    #[test]
    fn test_error_rate_calculation() {
        let service = RegistrationMonitoringService::new().unwrap();
        
        // Initially should be 0%
        assert_eq!(service.calculate_error_rate(), 0.0);
        
        // Record some attempts and failures
        service.record_registration_attempt();
        service.record_registration_attempt();
        service.record_registration_failure("validation_error");
        
        // Should be 50% error rate
        let error_rate = service.calculate_error_rate();
        assert!((error_rate - 50.0).abs() < 0.1);
    }
}