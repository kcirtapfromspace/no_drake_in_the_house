//! Prometheus metrics collection and monitoring
//! 
//! This module provides comprehensive metrics collection for the application,
//! including HTTP request metrics, database operation metrics, and custom business metrics.

use prometheus::{
    Counter, Gauge, Registry, Encoder, TextEncoder,
    HistogramOpts, Opts, CounterVec, HistogramVec,
};
use std::sync::Arc;
use std::time::Instant;
use axum::{
    extract::State,
    http::{StatusCode, Method},
    response::{Response, IntoResponse},
    body::Body,
};
use serde_json::json;

/// Metrics collector with Prometheus integration
#[derive(Clone)]
pub struct MetricsCollector {
    registry: Arc<Registry>,
    
    // HTTP metrics
    http_requests_total: CounterVec,
    http_request_duration: HistogramVec,
    http_requests_in_flight: Gauge,
    
    // Database metrics
    db_connections_active: Gauge,
    db_connections_idle: Gauge,
    db_query_duration: HistogramVec,
    db_operations_total: CounterVec,
    
    // Redis metrics
    redis_connections_active: Gauge,
    redis_operations_total: CounterVec,
    redis_operation_duration: HistogramVec,
    
    // Business metrics
    user_registrations_total: Counter,
    user_logins_total: CounterVec,
    dnp_operations_total: CounterVec,
    auth_failures_total: CounterVec,
    
    // System metrics
    memory_usage_bytes: Gauge,
    cpu_usage_percent: Gauge,
    uptime_seconds: Gauge,
}

impl MetricsCollector {
    /// Create a new metrics collector with all metrics registered
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Arc::new(Registry::new());
        
        // HTTP metrics
        let http_requests_total = CounterVec::new(
            Opts::new("http_requests_total", "Total number of HTTP requests")
                .namespace("kiro")
                .subsystem("http"),
            &["method", "endpoint", "status_code"]
        )?;
        
        let http_request_duration = HistogramVec::new(
            HistogramOpts::new("http_request_duration_seconds", "HTTP request duration in seconds")
                .namespace("kiro")
                .subsystem("http")
                .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
            &["method", "endpoint"]
        )?;
        
        let http_requests_in_flight = Gauge::new(
            "kiro_http_requests_in_flight",
            "Number of HTTP requests currently being processed"
        )?;
        
        // Database metrics
        let db_connections_active = Gauge::new(
            "kiro_db_connections_active",
            "Number of active database connections"
        )?;
        
        let db_connections_idle = Gauge::new(
            "kiro_db_connections_idle", 
            "Number of idle database connections"
        )?;
        
        let db_query_duration = HistogramVec::new(
            HistogramOpts::new("db_query_duration_seconds", "Database query duration in seconds")
                .namespace("kiro")
                .subsystem("db")
                .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]),
            &["operation", "table"]
        )?;
        
        let db_operations_total = CounterVec::new(
            Opts::new("db_operations_total", "Total number of database operations")
                .namespace("kiro")
                .subsystem("db"),
            &["operation", "table", "status"]
        )?;
        
        // Redis metrics
        let redis_connections_active = Gauge::new(
            "kiro_redis_connections_active",
            "Number of active Redis connections"
        )?;
        
        let redis_operations_total = CounterVec::new(
            Opts::new("redis_operations_total", "Total number of Redis operations")
                .namespace("kiro")
                .subsystem("redis"),
            &["operation", "status"]
        )?;
        
        let redis_operation_duration = HistogramVec::new(
            HistogramOpts::new("redis_operation_duration_seconds", "Redis operation duration in seconds")
                .namespace("kiro")
                .subsystem("redis")
                .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]),
            &["operation"]
        )?;
        
        // Business metrics
        let user_registrations_total = Counter::new(
            "kiro_user_registrations_total",
            "Total number of user registrations"
        )?;
        
        let user_logins_total = CounterVec::new(
            Opts::new("user_logins_total", "Total number of user login attempts")
                .namespace("kiro")
                .subsystem("auth"),
            &["status", "method"]
        )?;
        
        let dnp_operations_total = CounterVec::new(
            Opts::new("dnp_operations_total", "Total number of DNP list operations")
                .namespace("kiro")
                .subsystem("dnp"),
            &["operation", "status"]
        )?;
        
        let auth_failures_total = CounterVec::new(
            Opts::new("auth_failures_total", "Total number of authentication failures")
                .namespace("kiro")
                .subsystem("auth"),
            &["reason"]
        )?;
        
        // System metrics
        let memory_usage_bytes = Gauge::new(
            "kiro_memory_usage_bytes",
            "Current memory usage in bytes"
        )?;
        
        let cpu_usage_percent = Gauge::new(
            "kiro_cpu_usage_percent",
            "Current CPU usage percentage"
        )?;
        
        let uptime_seconds = Gauge::new(
            "kiro_uptime_seconds",
            "Application uptime in seconds"
        )?;
        
        // Register all metrics
        registry.register(Box::new(http_requests_total.clone()))?;
        registry.register(Box::new(http_request_duration.clone()))?;
        registry.register(Box::new(http_requests_in_flight.clone()))?;
        registry.register(Box::new(db_connections_active.clone()))?;
        registry.register(Box::new(db_connections_idle.clone()))?;
        registry.register(Box::new(db_query_duration.clone()))?;
        registry.register(Box::new(db_operations_total.clone()))?;
        registry.register(Box::new(redis_connections_active.clone()))?;
        registry.register(Box::new(redis_operations_total.clone()))?;
        registry.register(Box::new(redis_operation_duration.clone()))?;
        registry.register(Box::new(user_registrations_total.clone()))?;
        registry.register(Box::new(user_logins_total.clone()))?;
        registry.register(Box::new(dnp_operations_total.clone()))?;
        registry.register(Box::new(auth_failures_total.clone()))?;
        registry.register(Box::new(memory_usage_bytes.clone()))?;
        registry.register(Box::new(cpu_usage_percent.clone()))?;
        registry.register(Box::new(uptime_seconds.clone()))?;
        
        Ok(Self {
            registry,
            http_requests_total,
            http_request_duration,
            http_requests_in_flight,
            db_connections_active,
            db_connections_idle,
            db_query_duration,
            db_operations_total,
            redis_connections_active,
            redis_operations_total,
            redis_operation_duration,
            user_registrations_total,
            user_logins_total,
            dnp_operations_total,
            auth_failures_total,
            memory_usage_bytes,
            cpu_usage_percent,
            uptime_seconds,
        })
    }
    
    /// Record HTTP request metrics
    pub fn record_http_request(&self, method: &Method, endpoint: &str, status_code: StatusCode, duration: std::time::Duration) {
        let status_str = status_code.as_u16().to_string();
        
        self.http_requests_total
            .with_label_values(&[method.as_str(), endpoint, &status_str])
            .inc();
            
        self.http_request_duration
            .with_label_values(&[method.as_str(), endpoint])
            .observe(duration.as_secs_f64());
    }
    
    /// Increment in-flight requests
    pub fn increment_in_flight_requests(&self) {
        self.http_requests_in_flight.inc();
    }
    
    /// Decrement in-flight requests
    pub fn decrement_in_flight_requests(&self) {
        self.http_requests_in_flight.dec();
    }
    
    /// Update database connection metrics
    pub fn update_db_connections(&self, active: u32, idle: u32) {
        self.db_connections_active.set(active as f64);
        self.db_connections_idle.set(idle as f64);
    }
    
    /// Record database operation metrics
    pub fn record_db_operation(&self, operation: &str, table: &str, duration: std::time::Duration, success: bool) {
        let status = if success { "success" } else { "error" };
        
        self.db_operations_total
            .with_label_values(&[operation, table, status])
            .inc();
            
        self.db_query_duration
            .with_label_values(&[operation, table])
            .observe(duration.as_secs_f64());
    }
    
    /// Update Redis connection metrics
    pub fn update_redis_connections(&self, active: u32) {
        self.redis_connections_active.set(active as f64);
    }
    
    /// Record Redis operation metrics
    pub fn record_redis_operation(&self, operation: &str, duration: std::time::Duration, success: bool) {
        let status = if success { "success" } else { "error" };
        
        self.redis_operations_total
            .with_label_values(&[operation, status])
            .inc();
            
        self.redis_operation_duration
            .with_label_values(&[operation])
            .observe(duration.as_secs_f64());
    }
    
    /// Record user registration
    pub fn record_user_registration(&self) {
        self.user_registrations_total.inc();
    }
    
    /// Record user login attempt
    pub fn record_user_login(&self, success: bool, method: &str) {
        let status = if success { "success" } else { "failure" };
        self.user_logins_total
            .with_label_values(&[status, method])
            .inc();
    }
    
    /// Record DNP operation
    pub fn record_dnp_operation(&self, operation: &str, success: bool) {
        let status = if success { "success" } else { "error" };
        self.dnp_operations_total
            .with_label_values(&[operation, status])
            .inc();
    }
    
    /// Record authentication failure
    pub fn record_auth_failure(&self, reason: &str) {
        self.auth_failures_total
            .with_label_values(&[reason])
            .inc();
    }
    
    /// Update system metrics
    pub fn update_system_metrics(&self, memory_bytes: u64, cpu_percent: f64, uptime_seconds: u64) {
        self.memory_usage_bytes.set(memory_bytes as f64);
        self.cpu_usage_percent.set(cpu_percent);
        self.uptime_seconds.set(uptime_seconds as f64);
    }
    
    /// Get metrics in Prometheus format
    pub fn get_metrics(&self) -> Result<String, prometheus::Error> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8_lossy(&buffer).to_string())
    }
    
    /// Get registry for custom metrics
    pub fn registry(&self) -> Arc<Registry> {
        self.registry.clone()
    }
}

/// Metrics middleware for HTTP requests
pub struct MetricsMiddleware {
    metrics: Arc<MetricsCollector>,
}

impl MetricsMiddleware {
    pub fn new(metrics: Arc<MetricsCollector>) -> Self {
        Self { metrics }
    }
}

/// HTTP request timer for measuring request duration
pub struct RequestTimer {
    start: Instant,
    metrics: Arc<MetricsCollector>,
    method: Method,
    endpoint: String,
}

impl RequestTimer {
    pub fn new(metrics: Arc<MetricsCollector>, method: Method, endpoint: String) -> Self {
        metrics.increment_in_flight_requests();
        Self {
            start: Instant::now(),
            metrics,
            method,
            endpoint,
        }
    }
    
    pub fn finish(self, status_code: StatusCode) {
        let duration = self.start.elapsed();
        self.metrics.decrement_in_flight_requests();
        self.metrics.record_http_request(&self.method, &self.endpoint, status_code, duration);
    }
}

/// Metrics endpoint handler
pub async fn metrics_handler(State(metrics): State<Arc<MetricsCollector>>) -> impl IntoResponse {
    match metrics.get_metrics() {
        Ok(metrics_text) => {
            Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/plain; version=0.0.4; charset=utf-8")
                .body(Body::from(metrics_text))
                .unwrap()
        }
        Err(err) => {
            tracing::error!("Failed to generate metrics: {}", err);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "error": "Failed to generate metrics",
                    "details": err.to_string()
                }).to_string()))
                .unwrap()
        }
    }
}

/// Performance monitoring for database operations
pub struct DatabaseMetrics {
    metrics: Arc<MetricsCollector>,
}

impl DatabaseMetrics {
    pub fn new(metrics: Arc<MetricsCollector>) -> Self {
        Self { metrics }
    }
    
    /// Time a database operation
    pub async fn time_operation<F, T, E>(&self, operation: &str, table: &str, future: F) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        let start = Instant::now();
        let result = future.await;
        let duration = start.elapsed();
        let success = result.is_ok();
        
        self.metrics.record_db_operation(operation, table, duration, success);
        result
    }
    
    /// Update connection pool metrics
    pub fn update_pool_metrics(&self, pool: &sqlx::PgPool) {
        let active = (pool.size() as usize).saturating_sub(pool.num_idle()) as u32;
        let idle = pool.num_idle() as u32;
        self.metrics.update_db_connections(active, idle);
    }
}

/// Performance monitoring for Redis operations
pub struct RedisMetrics {
    metrics: Arc<MetricsCollector>,
}

impl RedisMetrics {
    pub fn new(metrics: Arc<MetricsCollector>) -> Self {
        Self { metrics }
    }
    
    /// Time a Redis operation
    pub async fn time_operation<F, T, E>(&self, operation: &str, future: F) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        let start = Instant::now();
        let result = future.await;
        let duration = start.elapsed();
        let success = result.is_ok();
        
        self.metrics.record_redis_operation(operation, duration, success);
        result
    }
    
    /// Update connection pool metrics
    pub fn update_pool_metrics(&self, pool: &deadpool_redis::Pool) {
        let status = pool.status();
        self.metrics.update_redis_connections((status.size.saturating_sub(status.available)) as u32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Method;
    
    #[test]
    fn test_metrics_collector_creation() {
        let metrics = MetricsCollector::new().expect("Failed to create metrics collector");
        
        // Test that we can get metrics without error
        let metrics_text = metrics.get_metrics().expect("Failed to get metrics");
        assert!(!metrics_text.is_empty());
    }
    
    #[test]
    fn test_http_metrics_recording() {
        let metrics = MetricsCollector::new().expect("Failed to create metrics collector");
        
        // Record some HTTP requests
        metrics.record_http_request(&Method::GET, "/health", StatusCode::OK, std::time::Duration::from_millis(100));
        metrics.record_http_request(&Method::POST, "/api/auth/login", StatusCode::UNAUTHORIZED, std::time::Duration::from_millis(50));
        
        let metrics_text = metrics.get_metrics().expect("Failed to get metrics");
        assert!(metrics_text.contains("kiro_http_requests_total"));
        assert!(metrics_text.contains("kiro_http_request_duration_seconds"));
    }
    
    #[test]
    fn test_business_metrics_recording() {
        let metrics = MetricsCollector::new().expect("Failed to create metrics collector");
        
        // Record business metrics
        metrics.record_user_registration();
        metrics.record_user_login(true, "password");
        metrics.record_dnp_operation("add_artist", true);
        metrics.record_auth_failure("invalid_credentials");
        
        let metrics_text = metrics.get_metrics().expect("Failed to get metrics");
        assert!(metrics_text.contains("kiro_user_registrations_total"));
        assert!(metrics_text.contains("kiro_auth_user_logins_total"));
        assert!(metrics_text.contains("kiro_dnp_dnp_operations_total"));
        assert!(metrics_text.contains("kiro_auth_auth_failures_total"));
    }
}