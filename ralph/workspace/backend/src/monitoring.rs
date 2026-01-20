//! Comprehensive monitoring and observability system
//!
//! This module provides a unified monitoring system that combines health checks,
//! metrics collection, and performance monitoring for the application.

use crate::error::{AppError, Result};
use crate::health::{HealthCheckConfig, HealthCheckResponse, HealthChecker};
use crate::metrics::{DatabaseMetrics, MetricsCollector, RedisMetrics};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::System;
use tokio::time::interval;
use tracing::{error, info, warn};

/// Comprehensive monitoring system
#[derive(Clone)]
pub struct MonitoringSystem {
    metrics: Arc<MetricsCollector>,
    health_checker: Arc<HealthChecker>,
    db_metrics: Arc<DatabaseMetrics>,
    redis_metrics: Arc<RedisMetrics>,
    start_time: Instant,
}

/// Monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    pub health_check_interval: Duration,
    pub metrics_update_interval: Duration,
    pub system_metrics_enabled: bool,
    pub detailed_health_checks: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            health_check_interval: Duration::from_secs(30),
            metrics_update_interval: Duration::from_secs(10),
            system_metrics_enabled: true,
            detailed_health_checks: true,
        }
    }
}

/// System performance metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub memory_usage_bytes: u64,
    pub memory_usage_percent: f32,
    pub cpu_usage_percent: f32,
    pub uptime_seconds: u64,
    pub active_connections: u32,
    pub thread_count: u32,
}

/// Service performance metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceMetrics {
    pub database: DatabaseServiceMetrics,
    pub redis: RedisServiceMetrics,
    pub http: HttpServiceMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseServiceMetrics {
    pub active_connections: u32,
    pub idle_connections: u32,
    pub total_queries: u64,
    pub avg_query_duration_ms: f64,
    pub error_rate_percent: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RedisServiceMetrics {
    pub active_connections: u32,
    pub total_operations: u64,
    pub avg_operation_duration_ms: f64,
    pub error_rate_percent: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpServiceMetrics {
    pub total_requests: u64,
    pub requests_per_second: f64,
    pub avg_response_time_ms: f64,
    pub error_rate_percent: f32,
    pub active_requests: u32,
}

/// Combined monitoring response
#[derive(Debug, Serialize, Deserialize)]
pub struct MonitoringResponse {
    pub health: HealthCheckResponse,
    pub system_metrics: SystemMetrics,
    pub service_metrics: ServiceMetrics,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl MonitoringSystem {
    /// Create a new monitoring system
    pub fn new(config: MonitoringConfig) -> Result<Self> {
        let metrics = Arc::new(MetricsCollector::new().map_err(|e| AppError::Internal {
            message: Some(format!("Failed to create metrics collector: {}", e)),
        })?);

        let health_config = HealthCheckConfig {
            timeout: Duration::from_secs(5),
            include_system_info: config.system_metrics_enabled,
            detailed_checks: config.detailed_health_checks,
        };

        let health_checker = Arc::new(HealthChecker::new(health_config));
        let db_metrics = Arc::new(DatabaseMetrics::new(metrics.clone()));
        let redis_metrics = Arc::new(RedisMetrics::new(metrics.clone()));

        Ok(Self {
            metrics,
            health_checker,
            db_metrics,
            redis_metrics,
            start_time: Instant::now(),
        })
    }

    /// Get metrics collector
    pub fn metrics(&self) -> Arc<MetricsCollector> {
        self.metrics.clone()
    }

    /// Get database metrics helper
    pub fn db_metrics(&self) -> Arc<DatabaseMetrics> {
        self.db_metrics.clone()
    }

    /// Get Redis metrics helper
    pub fn redis_metrics(&self) -> Arc<RedisMetrics> {
        self.redis_metrics.clone()
    }

    /// Perform comprehensive monitoring check
    pub async fn comprehensive_check(
        &self,
        db_pool: &sqlx::PgPool,
        redis_pool: &deadpool_redis::Pool,
    ) -> MonitoringResponse {
        // Get health check
        let health = self.health_checker.check_health(db_pool, redis_pool).await;

        // Get system metrics
        let system_metrics = self.get_system_metrics().await;

        // Get service metrics (placeholder - would need actual metric collection)
        let service_metrics = self.get_service_metrics(db_pool, redis_pool).await;

        MonitoringResponse {
            health,
            system_metrics,
            service_metrics,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Get current system metrics
    async fn get_system_metrics(&self) -> SystemMetrics {
        let mut sys = System::new_all();
        sys.refresh_all();

        let memory_usage_bytes = sys.used_memory();
        let total_memory = sys.total_memory();
        let memory_usage_percent = if total_memory > 0 {
            (memory_usage_bytes as f32 / total_memory as f32) * 100.0
        } else {
            0.0
        };

        let cpu_usage_percent = sys.global_cpu_info().cpu_usage();
        let uptime_seconds = self.start_time.elapsed().as_secs();

        // Update metrics
        self.metrics.update_system_metrics(
            memory_usage_bytes,
            cpu_usage_percent as f64,
            uptime_seconds,
        );

        SystemMetrics {
            memory_usage_bytes,
            memory_usage_percent,
            cpu_usage_percent,
            uptime_seconds,
            active_connections: 0, // Would need to track this in application state
            thread_count: sys.processes().len() as u32,
        }
    }

    /// Get service-specific metrics
    async fn get_service_metrics(
        &self,
        db_pool: &sqlx::PgPool,
        redis_pool: &deadpool_redis::Pool,
    ) -> ServiceMetrics {
        // Update connection pool metrics
        self.db_metrics.update_pool_metrics(db_pool);
        self.redis_metrics.update_pool_metrics(redis_pool);

        // Database metrics
        let db_active = (db_pool.size() as usize).saturating_sub(db_pool.num_idle()) as u32;
        let db_idle = db_pool.num_idle() as u32;

        // Redis metrics
        let redis_status = redis_pool.status();
        let redis_active = (redis_status.size.saturating_sub(redis_status.available)) as u32;

        ServiceMetrics {
            database: DatabaseServiceMetrics {
                active_connections: db_active,
                idle_connections: db_idle,
                total_queries: 0,           // Would need to track this
                avg_query_duration_ms: 0.0, // Would calculate from histogram
                error_rate_percent: 0.0,    // Would calculate from counters
            },
            redis: RedisServiceMetrics {
                active_connections: redis_active,
                total_operations: 0,            // Would need to track this
                avg_operation_duration_ms: 0.0, // Would calculate from histogram
                error_rate_percent: 0.0,        // Would calculate from counters
            },
            http: HttpServiceMetrics {
                total_requests: 0,         // Would need to track this
                requests_per_second: 0.0,  // Would calculate from rate
                avg_response_time_ms: 0.0, // Would calculate from histogram
                error_rate_percent: 0.0,   // Would calculate from counters
                active_requests: 0,        // Would track in-flight requests
            },
        }
    }

    /// Start background monitoring tasks
    pub async fn start_background_monitoring(
        &self,
        config: MonitoringConfig,
        db_pool: sqlx::PgPool,
        redis_pool: deadpool_redis::Pool,
    ) {
        let monitoring = self.clone();

        tokio::spawn(async move {
            let mut health_interval = interval(config.health_check_interval);
            let mut metrics_interval = interval(config.metrics_update_interval);

            loop {
                tokio::select! {
                    _ = health_interval.tick() => {
                        let health = monitoring.health_checker.check_health(&db_pool, &redis_pool).await;

                        match health.status {
                            crate::health::HealthStatus::Healthy => {
                                info!("Health check passed: all services healthy");
                            }
                            crate::health::HealthStatus::Degraded => {
                                warn!("Health check warning: some services degraded");
                            }
                            crate::health::HealthStatus::Unhealthy => {
                                error!("Health check failed: services unhealthy");
                            }
                        }
                    }

                    _ = metrics_interval.tick() => {
                        if config.system_metrics_enabled {
                            let _system_metrics = monitoring.get_system_metrics().await;
                            // System metrics are automatically updated in the call above
                        }

                        // Update service metrics
                        monitoring.db_metrics.update_pool_metrics(&db_pool);
                        monitoring.redis_metrics.update_pool_metrics(&redis_pool);
                    }
                }
            }
        });
    }
}

/// Monitoring middleware for tracking request performance
pub struct MonitoringMiddleware {
    monitoring: Arc<MonitoringSystem>,
}

impl MonitoringMiddleware {
    pub fn new(monitoring: Arc<MonitoringSystem>) -> Self {
        Self { monitoring }
    }

    /// Get the monitoring system
    pub fn monitoring(&self) -> Arc<MonitoringSystem> {
        self.monitoring.clone()
    }
}

/// Performance profiler for critical operations
pub struct PerformanceProfiler {
    metrics: Arc<MetricsCollector>,
}

impl PerformanceProfiler {
    pub fn new(metrics: Arc<MetricsCollector>) -> Self {
        Self { metrics }
    }

    /// Profile a database operation
    pub async fn profile_db_operation<F, T, E>(
        &self,
        operation: &str,
        table: &str,
        future: F,
    ) -> std::result::Result<T, E>
    where
        F: std::future::Future<Output = std::result::Result<T, E>>,
    {
        let start = Instant::now();
        let result = future.await;
        let duration = start.elapsed();
        let success = result.is_ok();

        self.metrics
            .record_db_operation(operation, table, duration, success);

        if duration > Duration::from_millis(100) {
            warn!(
                operation = operation,
                table = table,
                duration_ms = duration.as_millis(),
                success = success,
                "Slow database operation detected"
            );
        }

        result
    }

    /// Profile a Redis operation
    pub async fn profile_redis_operation<F, T, E>(
        &self,
        operation: &str,
        future: F,
    ) -> std::result::Result<T, E>
    where
        F: std::future::Future<Output = std::result::Result<T, E>>,
    {
        let start = Instant::now();
        let result = future.await;
        let duration = start.elapsed();
        let success = result.is_ok();

        self.metrics
            .record_redis_operation(operation, duration, success);

        if duration > Duration::from_millis(50) {
            warn!(
                operation = operation,
                duration_ms = duration.as_millis(),
                success = success,
                "Slow Redis operation detected"
            );
        }

        result
    }
}

/// Alert thresholds for monitoring
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub max_response_time_ms: u64,
    pub max_error_rate_percent: f32,
    pub max_memory_usage_percent: f32,
    pub max_cpu_usage_percent: f32,
    pub min_available_connections: u32,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            max_response_time_ms: 1000,
            max_error_rate_percent: 5.0,
            max_memory_usage_percent: 80.0,
            max_cpu_usage_percent: 80.0,
            min_available_connections: 5,
        }
    }
}

/// Alert manager for monitoring thresholds
pub struct AlertManager {
    thresholds: AlertThresholds,
}

impl AlertManager {
    pub fn new(thresholds: AlertThresholds) -> Self {
        Self { thresholds }
    }

    /// Check if metrics exceed alert thresholds
    pub fn check_alerts(&self, monitoring_response: &MonitoringResponse) -> Vec<Alert> {
        let mut alerts = Vec::new();

        // Check system metrics
        if monitoring_response.system_metrics.memory_usage_percent
            > self.thresholds.max_memory_usage_percent
        {
            alerts.push(Alert {
                severity: AlertSeverity::Warning,
                message: format!(
                    "High memory usage: {:.1}% (threshold: {:.1}%)",
                    monitoring_response.system_metrics.memory_usage_percent,
                    self.thresholds.max_memory_usage_percent
                ),
                metric: "memory_usage_percent".to_string(),
                value: monitoring_response.system_metrics.memory_usage_percent as f64,
                threshold: self.thresholds.max_memory_usage_percent as f64,
            });
        }

        if monitoring_response.system_metrics.cpu_usage_percent
            > self.thresholds.max_cpu_usage_percent
        {
            alerts.push(Alert {
                severity: AlertSeverity::Warning,
                message: format!(
                    "High CPU usage: {:.1}% (threshold: {:.1}%)",
                    monitoring_response.system_metrics.cpu_usage_percent,
                    self.thresholds.max_cpu_usage_percent
                ),
                metric: "cpu_usage_percent".to_string(),
                value: monitoring_response.system_metrics.cpu_usage_percent as f64,
                threshold: self.thresholds.max_cpu_usage_percent as f64,
            });
        }

        // Check database connections
        let db_available = monitoring_response
            .service_metrics
            .database
            .idle_connections;
        if db_available < self.thresholds.min_available_connections {
            alerts.push(Alert {
                severity: AlertSeverity::Critical,
                message: format!(
                    "Low database connections available: {} (threshold: {})",
                    db_available, self.thresholds.min_available_connections
                ),
                metric: "db_available_connections".to_string(),
                value: db_available as f64,
                threshold: self.thresholds.min_available_connections as f64,
            });
        }

        alerts
    }
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Alert structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub severity: AlertSeverity,
    pub message: String,
    pub metric: String,
    pub value: f64,
    pub threshold: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitoring_system_creation() {
        let config = MonitoringConfig::default();
        let monitoring = MonitoringSystem::new(config).expect("Failed to create monitoring system");

        // Test that we can get metrics
        let metrics_text = monitoring
            .metrics()
            .get_metrics()
            .expect("Failed to get metrics");
        assert!(!metrics_text.is_empty());
    }

    #[test]
    fn test_alert_manager() {
        let thresholds = AlertThresholds {
            max_memory_usage_percent: 80.0,
            max_cpu_usage_percent: 80.0,
            ..Default::default()
        };

        let alert_manager = AlertManager::new(thresholds);

        // Create a monitoring response with high memory usage
        let monitoring_response = MonitoringResponse {
            health: HealthCheckResponse {
                status: crate::health::HealthStatus::Healthy,
                timestamp: chrono::Utc::now(),
                correlation_id: "test".to_string(),
                version: "test".to_string(),
                uptime_seconds: 100,
                services: std::collections::HashMap::new(),
                system_info: crate::health::SystemInfo {
                    memory_usage_mb: 1000,
                    cpu_usage_percent: 50.0,
                    disk_usage_percent: 30.0,
                    active_connections: 10,
                },
            },
            system_metrics: SystemMetrics {
                memory_usage_bytes: 1000000000,
                memory_usage_percent: 85.0, // Above threshold
                cpu_usage_percent: 50.0,
                uptime_seconds: 100,
                active_connections: 10,
                thread_count: 5,
            },
            service_metrics: ServiceMetrics {
                database: DatabaseServiceMetrics {
                    active_connections: 5,
                    idle_connections: 10,
                    total_queries: 100,
                    avg_query_duration_ms: 50.0,
                    error_rate_percent: 1.0,
                },
                redis: RedisServiceMetrics {
                    active_connections: 2,
                    total_operations: 200,
                    avg_operation_duration_ms: 10.0,
                    error_rate_percent: 0.5,
                },
                http: HttpServiceMetrics {
                    total_requests: 1000,
                    requests_per_second: 10.0,
                    avg_response_time_ms: 100.0,
                    error_rate_percent: 2.0,
                    active_requests: 5,
                },
            },
            timestamp: chrono::Utc::now(),
        };

        let alerts = alert_manager.check_alerts(&monitoring_response);
        assert!(!alerts.is_empty());
        assert!(alerts.iter().any(|a| a.metric == "memory_usage_percent"));
    }
}
