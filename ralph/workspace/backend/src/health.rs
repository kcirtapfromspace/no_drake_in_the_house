//! Comprehensive health check system with error recovery

use crate::error::Result;
use crate::recovery::{database_health_check_with_recovery, redis_health_check_with_recovery};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Health check response with detailed service information
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: HealthStatus,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub correlation_id: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub services: HashMap<String, ServiceHealthInfo>,
    pub system_info: SystemInfo,
}

/// Overall health status
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Individual service health information
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceHealthInfo {
    pub status: HealthStatus,
    pub response_time_ms: u64,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub error_message: Option<String>,
    pub details: Option<serde_json::Value>,
}

/// System information
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f32,
    pub disk_usage_percent: f32,
    pub active_connections: u32,
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    pub timeout: Duration,
    pub include_system_info: bool,
    pub detailed_checks: bool,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            include_system_info: true,
            detailed_checks: true,
        }
    }
}

/// Health checker with recovery mechanisms
pub struct HealthChecker {
    config: HealthCheckConfig,
    start_time: Instant,
}

impl HealthChecker {
    pub fn new(config: HealthCheckConfig) -> Self {
        Self {
            config,
            start_time: Instant::now(),
        }
    }

    /// Perform comprehensive health check
    pub async fn check_health(
        &self,
        db_pool: &sqlx::PgPool,
        redis_pool: &deadpool_redis::Pool,
    ) -> HealthCheckResponse {
        let correlation_id = Uuid::new_v4().to_string();
        let mut services = HashMap::new();

        // Check database health
        let db_health = self.check_database_health(db_pool).await;
        services.insert("database".to_string(), db_health);

        // Check Redis health
        let redis_health = self.check_redis_health(redis_pool).await;
        services.insert("redis".to_string(), redis_health);

        // Check API health (always healthy if we can respond)
        let api_health = ServiceHealthInfo {
            status: HealthStatus::Healthy,
            response_time_ms: 0,
            last_check: chrono::Utc::now(),
            error_message: None,
            details: Some(serde_json::json!({
                "endpoints_available": true,
                "middleware_active": true
            })),
        };
        services.insert("api".to_string(), api_health);

        // Determine overall status
        let overall_status = self.determine_overall_status(&services);

        // Get system information if enabled
        let system_info = if self.config.include_system_info {
            self.get_system_info().await
        } else {
            SystemInfo {
                memory_usage_mb: 0,
                cpu_usage_percent: 0.0,
                disk_usage_percent: 0.0,
                active_connections: 0,
            }
        };

        HealthCheckResponse {
            status: overall_status,
            timestamp: chrono::Utc::now(),
            correlation_id,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            services,
            system_info,
        }
    }

    /// Check database health with recovery
    async fn check_database_health(&self, db_pool: &sqlx::PgPool) -> ServiceHealthInfo {
        let start = Instant::now();
        let last_check = chrono::Utc::now();

        match database_health_check_with_recovery(db_pool).await {
            Ok(()) => {
                let response_time = start.elapsed().as_millis() as u64;

                // Additional detailed checks if enabled
                let details = if self.config.detailed_checks {
                    self.get_database_details(db_pool).await
                } else {
                    None
                };

                ServiceHealthInfo {
                    status: HealthStatus::Healthy,
                    response_time_ms: response_time,
                    last_check,
                    error_message: None,
                    details,
                }
            }
            Err(err) => {
                let response_time = start.elapsed().as_millis() as u64;

                tracing::error!(
                    correlation_id = %Uuid::new_v4(),
                    error = %err,
                    response_time_ms = response_time,
                    "Database health check failed"
                );

                ServiceHealthInfo {
                    status: HealthStatus::Unhealthy,
                    response_time_ms: response_time,
                    last_check,
                    error_message: Some(err.to_string()),
                    details: Some(serde_json::json!({
                        "connection_pool_status": "failed",
                        "retry_attempts": "exhausted"
                    })),
                }
            }
        }
    }

    /// Check Redis health with recovery
    async fn check_redis_health(&self, redis_pool: &deadpool_redis::Pool) -> ServiceHealthInfo {
        let start = Instant::now();
        let last_check = chrono::Utc::now();

        match redis_health_check_with_recovery(redis_pool).await {
            Ok(()) => {
                let response_time = start.elapsed().as_millis() as u64;

                // Additional detailed checks if enabled
                let details = if self.config.detailed_checks {
                    self.get_redis_details(redis_pool).await
                } else {
                    None
                };

                ServiceHealthInfo {
                    status: HealthStatus::Healthy,
                    response_time_ms: response_time,
                    last_check,
                    error_message: None,
                    details,
                }
            }
            Err(err) => {
                let response_time = start.elapsed().as_millis() as u64;

                tracing::error!(
                    correlation_id = %Uuid::new_v4(),
                    error = %err,
                    response_time_ms = response_time,
                    "Redis health check failed"
                );

                ServiceHealthInfo {
                    status: HealthStatus::Unhealthy,
                    response_time_ms: response_time,
                    last_check,
                    error_message: Some(err.to_string()),
                    details: Some(serde_json::json!({
                        "connection_pool_status": "failed",
                        "retry_attempts": "exhausted"
                    })),
                }
            }
        }
    }

    /// Get detailed database information
    async fn get_database_details(&self, db_pool: &sqlx::PgPool) -> Option<serde_json::Value> {
        // Try to get database statistics
        let stats_result = sqlx::query!(
            r#"
            SELECT 
                (SELECT count(*) FROM users) as user_count,
                (SELECT count(*) FROM artists) as artist_count,
                (SELECT count(*) FROM user_artist_blocks) as dnp_entries_count
            "#
        )
        .fetch_optional(db_pool)
        .await;

        match stats_result {
            Ok(Some(stats)) => Some(serde_json::json!({
                "connection_pool_size": db_pool.size(),
                "idle_connections": db_pool.num_idle(),
                "user_count": stats.user_count,
                "artist_count": stats.artist_count,
                "dnp_entries_count": stats.dnp_entries_count
            })),
            _ => Some(serde_json::json!({
                "connection_pool_size": db_pool.size(),
                "idle_connections": db_pool.num_idle(),
                "statistics": "unavailable"
            })),
        }
    }

    /// Get detailed Redis information
    async fn get_redis_details(
        &self,
        redis_pool: &deadpool_redis::Pool,
    ) -> Option<serde_json::Value> {
        let mut conn = redis_pool.get().await.ok()?;

        // Try to get Redis info
        let info_result: std::result::Result<String, redis::RedisError> = redis::cmd("INFO")
            .arg("memory")
            .query_async(&mut conn)
            .await;

        match info_result {
            Ok(info) => {
                // Parse memory usage from INFO output
                let memory_used = info
                    .lines()
                    .find(|line| line.starts_with("used_memory_human:"))
                    .and_then(|line| line.split(':').nth(1))
                    .unwrap_or("unknown");

                Some(serde_json::json!({
                    "connection_pool_size": redis_pool.status().size,
                    "available_connections": redis_pool.status().available,
                    "memory_used": memory_used
                }))
            }
            Err(_) => Some(serde_json::json!({
                "connection_pool_size": redis_pool.status().size,
                "available_connections": redis_pool.status().available,
                "info": "unavailable"
            })),
        }
    }

    /// Determine overall health status based on service statuses
    fn determine_overall_status(
        &self,
        services: &HashMap<String, ServiceHealthInfo>,
    ) -> HealthStatus {
        let mut _healthy_count = 0;
        let mut degraded_count = 0;
        let mut unhealthy_count = 0;

        for service in services.values() {
            match service.status {
                HealthStatus::Healthy => _healthy_count += 1,
                HealthStatus::Degraded => degraded_count += 1,
                HealthStatus::Unhealthy => unhealthy_count += 1,
            }
        }

        // If any critical service is unhealthy, overall status is unhealthy
        if unhealthy_count > 0 {
            HealthStatus::Unhealthy
        } else if degraded_count > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    /// Get system information
    async fn get_system_info(&self) -> SystemInfo {
        // Use sysinfo crate to get system information
        use sysinfo::System;

        let mut sys = System::new_all();
        sys.refresh_all();

        let memory_usage_mb = sys.used_memory() / 1024 / 1024;
        let cpu_usage_percent = sys.global_cpu_info().cpu_usage();

        // Calculate disk usage - simplified for compatibility
        let disk_usage_percent = 0.0; // Placeholder - would need platform-specific implementation

        SystemInfo {
            memory_usage_mb,
            cpu_usage_percent,
            disk_usage_percent,
            active_connections: 0, // Would need to track this in application state
        }
    }
}

/// Readiness check for Kubernetes
pub async fn readiness_check(
    db_pool: &sqlx::PgPool,
    redis_pool: &deadpool_redis::Pool,
) -> Result<()> {
    // Simple checks that must pass for the service to be ready
    database_health_check_with_recovery(db_pool).await?;
    redis_health_check_with_recovery(redis_pool).await?;
    Ok(())
}

/// Liveness check for Kubernetes
pub async fn liveness_check() -> Result<()> {
    // Basic check that the service is alive
    // This should only fail if the service is completely broken
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_overall_status() {
        let checker = HealthChecker::new(HealthCheckConfig::default());
        let mut services = HashMap::new();

        // All healthy
        services.insert(
            "db".to_string(),
            ServiceHealthInfo {
                status: HealthStatus::Healthy,
                response_time_ms: 10,
                last_check: chrono::Utc::now(),
                error_message: None,
                details: None,
            },
        );
        services.insert(
            "redis".to_string(),
            ServiceHealthInfo {
                status: HealthStatus::Healthy,
                response_time_ms: 5,
                last_check: chrono::Utc::now(),
                error_message: None,
                details: None,
            },
        );
        assert_eq!(
            checker.determine_overall_status(&services),
            HealthStatus::Healthy
        );

        // One degraded
        services.insert(
            "redis".to_string(),
            ServiceHealthInfo {
                status: HealthStatus::Degraded,
                response_time_ms: 100,
                last_check: chrono::Utc::now(),
                error_message: Some("Slow response".to_string()),
                details: None,
            },
        );
        assert_eq!(
            checker.determine_overall_status(&services),
            HealthStatus::Degraded
        );

        // One unhealthy
        services.insert(
            "db".to_string(),
            ServiceHealthInfo {
                status: HealthStatus::Unhealthy,
                response_time_ms: 5000,
                last_check: chrono::Utc::now(),
                error_message: Some("Connection failed".to_string()),
                details: None,
            },
        );
        assert_eq!(
            checker.determine_overall_status(&services),
            HealthStatus::Unhealthy
        );
    }
}
