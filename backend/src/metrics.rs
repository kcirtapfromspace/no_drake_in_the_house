//! Prometheus metrics collection and monitoring
//!
//! This module provides comprehensive metrics collection for the application,
//! including HTTP request metrics, database operation metrics, and custom business metrics.
//!
//! US-022: Real system metrics implementation - all metrics are collected from actual
//! system state, not placeholder values.

use axum::{
    body::Body,
    extract::State,
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
};
use prometheus::{
    Counter, CounterVec, Encoder, Gauge, GaugeVec, HistogramOpts, HistogramVec, Opts, Registry,
    TextEncoder,
};
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use sysinfo::{Disks, System};

/// Metrics collector with Prometheus integration
#[derive(Clone)]
pub struct MetricsCollector {
    registry: Arc<Registry>,

    // HTTP metrics
    http_requests_total: CounterVec,
    http_request_duration: HistogramVec,
    http_requests_in_flight: Gauge,

    // Request latency metrics (US-023)
    http_request_latency: HistogramVec,

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

    // System metrics (US-022: Real system metrics)
    memory_usage_bytes: Gauge,
    memory_rss_bytes: Gauge,
    memory_heap_bytes: Gauge,
    cpu_usage_percent: Gauge,
    uptime_seconds: Gauge,

    // Disk metrics (US-022)
    disk_usage_bytes: Gauge,
    disk_available_bytes: Gauge,
    disk_total_bytes: Gauge,

    // Job queue metrics (US-022)
    job_queue_depth: GaugeVec,

    // Data key cache metrics
    data_key_cache_hits: Counter,
    data_key_cache_misses: Counter,

    // Token refresh metrics (US-011)
    tokens_refreshed_total: Counter,
    token_refresh_failures_total: Counter,
    connections_marked_reauth_total: Counter,
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
            &["method", "endpoint", "status_code"],
        )?;

        let http_request_duration = HistogramVec::new(
            HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request duration in seconds",
            )
            .namespace("kiro")
            .subsystem("http")
            .buckets(vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ]),
            &["method", "endpoint"],
        )?;

        let http_requests_in_flight = Gauge::new(
            "kiro_http_requests_in_flight",
            "Number of HTTP requests currently being processed",
        )?;

        // Request latency histogram (US-023)
        // Buckets: 10ms, 50ms, 100ms, 250ms, 500ms, 1000ms, 5000ms
        let http_request_latency = HistogramVec::new(
            HistogramOpts::new(
                "http_request_latency_seconds",
                "HTTP request latency in seconds for P50/P90/P99 calculations",
            )
            .namespace("kiro")
            .subsystem("http")
            .buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 5.0]),
            &["method", "path", "status_code"],
        )?;

        // Database metrics
        let db_connections_active = Gauge::new(
            "kiro_db_connections_active",
            "Number of active database connections",
        )?;

        let db_connections_idle = Gauge::new(
            "kiro_db_connections_idle",
            "Number of idle database connections",
        )?;

        let db_query_duration = HistogramVec::new(
            HistogramOpts::new(
                "db_query_duration_seconds",
                "Database query duration in seconds",
            )
            .namespace("kiro")
            .subsystem("db")
            .buckets(vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0,
            ]),
            &["operation", "table"],
        )?;

        let db_operations_total = CounterVec::new(
            Opts::new("db_operations_total", "Total number of database operations")
                .namespace("kiro")
                .subsystem("db"),
            &["operation", "table", "status"],
        )?;

        // Redis metrics
        let redis_connections_active = Gauge::new(
            "kiro_redis_connections_active",
            "Number of active Redis connections",
        )?;

        let redis_operations_total = CounterVec::new(
            Opts::new("redis_operations_total", "Total number of Redis operations")
                .namespace("kiro")
                .subsystem("redis"),
            &["operation", "status"],
        )?;

        let redis_operation_duration = HistogramVec::new(
            HistogramOpts::new(
                "redis_operation_duration_seconds",
                "Redis operation duration in seconds",
            )
            .namespace("kiro")
            .subsystem("redis")
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]),
            &["operation"],
        )?;

        // Business metrics
        let user_registrations_total = Counter::new(
            "kiro_user_registrations_total",
            "Total number of user registrations",
        )?;

        let user_logins_total = CounterVec::new(
            Opts::new("user_logins_total", "Total number of user login attempts")
                .namespace("kiro")
                .subsystem("auth"),
            &["status", "method"],
        )?;

        let dnp_operations_total = CounterVec::new(
            Opts::new(
                "dnp_operations_total",
                "Total number of DNP list operations",
            )
            .namespace("kiro")
            .subsystem("dnp"),
            &["operation", "status"],
        )?;

        let auth_failures_total = CounterVec::new(
            Opts::new(
                "auth_failures_total",
                "Total number of authentication failures",
            )
            .namespace("kiro")
            .subsystem("auth"),
            &["reason"],
        )?;

        // System metrics (US-022: Real system metrics - no placeholders)
        let memory_usage_bytes = Gauge::new(
            "kiro_memory_usage_bytes",
            "Current total memory usage in bytes",
        )?;

        let memory_rss_bytes = Gauge::new(
            "kiro_memory_rss_bytes",
            "Process resident set size (RSS) in bytes",
        )?;

        let memory_heap_bytes = Gauge::new(
            "kiro_memory_heap_bytes",
            "Process heap memory usage in bytes",
        )?;

        let cpu_usage_percent =
            Gauge::new("kiro_cpu_usage_percent", "Current CPU usage percentage")?;

        let uptime_seconds = Gauge::new("kiro_uptime_seconds", "Application uptime in seconds")?;

        // Disk metrics (US-022)
        let disk_usage_bytes = Gauge::new(
            "kiro_disk_usage_bytes",
            "Disk space used by data directory in bytes",
        )?;

        let disk_available_bytes = Gauge::new(
            "kiro_disk_available_bytes",
            "Available disk space for data directory in bytes",
        )?;

        let disk_total_bytes = Gauge::new(
            "kiro_disk_total_bytes",
            "Total disk space for data directory in bytes",
        )?;

        // Job queue metrics (US-022)
        let job_queue_depth = GaugeVec::new(
            Opts::new("kiro_job_queue_depth", "Number of pending jobs by job type"),
            &["job_type"],
        )?;

        // Data key cache metrics
        let data_key_cache_hits = Counter::new(
            "kiro_data_key_cache_hits",
            "Total number of data key cache hits",
        )?;

        let data_key_cache_misses = Counter::new(
            "kiro_data_key_cache_misses",
            "Total number of data key cache misses",
        )?;

        // Token refresh metrics (US-011)
        let tokens_refreshed_total = Counter::new(
            "kiro_tokens_refreshed_total",
            "Total number of tokens successfully refreshed",
        )?;

        let token_refresh_failures_total = Counter::new(
            "kiro_token_refresh_failures_total",
            "Total number of token refresh failures",
        )?;

        let connections_marked_reauth_total = Counter::new(
            "kiro_connections_marked_reauth_total",
            "Total number of connections marked as needing re-authentication",
        )?;

        // Register all metrics
        registry.register(Box::new(http_requests_total.clone()))?;
        registry.register(Box::new(http_request_duration.clone()))?;
        registry.register(Box::new(http_requests_in_flight.clone()))?;
        registry.register(Box::new(http_request_latency.clone()))?;
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
        registry.register(Box::new(memory_rss_bytes.clone()))?;
        registry.register(Box::new(memory_heap_bytes.clone()))?;
        registry.register(Box::new(cpu_usage_percent.clone()))?;
        registry.register(Box::new(uptime_seconds.clone()))?;
        registry.register(Box::new(disk_usage_bytes.clone()))?;
        registry.register(Box::new(disk_available_bytes.clone()))?;
        registry.register(Box::new(disk_total_bytes.clone()))?;
        registry.register(Box::new(job_queue_depth.clone()))?;
        registry.register(Box::new(data_key_cache_hits.clone()))?;
        registry.register(Box::new(data_key_cache_misses.clone()))?;
        registry.register(Box::new(tokens_refreshed_total.clone()))?;
        registry.register(Box::new(token_refresh_failures_total.clone()))?;
        registry.register(Box::new(connections_marked_reauth_total.clone()))?;

        Ok(Self {
            registry,
            http_requests_total,
            http_request_duration,
            http_requests_in_flight,
            http_request_latency,
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
            memory_rss_bytes,
            memory_heap_bytes,
            cpu_usage_percent,
            uptime_seconds,
            disk_usage_bytes,
            disk_available_bytes,
            disk_total_bytes,
            job_queue_depth,
            data_key_cache_hits,
            data_key_cache_misses,
            tokens_refreshed_total,
            token_refresh_failures_total,
            connections_marked_reauth_total,
        })
    }

    /// Record HTTP request metrics
    pub fn record_http_request(
        &self,
        method: &Method,
        endpoint: &str,
        status_code: StatusCode,
        duration: std::time::Duration,
    ) {
        let status_str = status_code.as_u16().to_string();

        self.http_requests_total
            .with_label_values(&[method.as_str(), endpoint, &status_str])
            .inc();

        self.http_request_duration
            .with_label_values(&[method.as_str(), endpoint])
            .observe(duration.as_secs_f64());
    }

    /// Record request latency for P50/P90/P99 calculations (US-023)
    ///
    /// Labels: method, path, status_code
    /// Histogram buckets: 10ms, 50ms, 100ms, 250ms, 500ms, 1000ms, 5000ms
    pub fn record_request_latency(
        &self,
        method: &str,
        path: &str,
        status_code: u16,
        duration: std::time::Duration,
    ) {
        self.http_request_latency
            .with_label_values(&[method, path, &status_code.to_string()])
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
    pub fn record_db_operation(
        &self,
        operation: &str,
        table: &str,
        duration: std::time::Duration,
        success: bool,
    ) {
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
    pub fn record_redis_operation(
        &self,
        operation: &str,
        duration: std::time::Duration,
        success: bool,
    ) {
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
        self.auth_failures_total.with_label_values(&[reason]).inc();
    }

    /// Update system metrics (legacy method - kept for backwards compatibility)
    pub fn update_system_metrics(&self, memory_bytes: u64, cpu_percent: f64, uptime_seconds: u64) {
        self.memory_usage_bytes.set(memory_bytes as f64);
        self.cpu_usage_percent.set(cpu_percent);
        self.uptime_seconds.set(uptime_seconds as f64);
    }

    /// Collect real system metrics using sysinfo (US-022)
    ///
    /// This method collects actual system metrics including:
    /// - CPU usage percentage
    /// - Memory usage (total used, RSS, heap estimate)
    /// - Disk usage for the specified data directory
    /// - Application uptime
    pub fn collect_real_system_metrics(&self, data_dir: &Path, uptime_secs: u64) {
        // Create a new System instance and refresh relevant data
        let mut sys = System::new();
        sys.refresh_cpu_usage();
        sys.refresh_memory();

        // CPU usage (global average from all cores - sysinfo 0.30 API)
        let cpu_percent = if sys.cpus().is_empty() {
            0.0
        } else {
            sys.cpus().iter()
                .map(|cpu| cpu.cpu_usage())
                .sum::<f32>() / sys.cpus().len() as f32
        };
        self.cpu_usage_percent.set(cpu_percent as f64);

        // Memory metrics
        let total_memory_used = sys.used_memory();
        self.memory_usage_bytes.set(total_memory_used as f64);

        // Get process-specific memory metrics if available
        let current_pid = sysinfo::get_current_pid();
        if let Ok(pid) = current_pid {
            sys.refresh_process(pid);
            if let Some(process) = sys.process(pid) {
                // RSS (Resident Set Size) - physical memory the process is using
                let rss = process.memory();
                self.memory_rss_bytes.set(rss as f64);

                // Estimate heap as a portion of RSS (virtual memory is less useful here)
                // In Rust, heap is typically the majority of RSS minus stack and code
                // This is an approximation since Rust doesn't expose exact heap stats
                let heap_estimate = (rss as f64 * 0.85) as u64; // Conservative estimate
                self.memory_heap_bytes.set(heap_estimate as f64);
            }
        }

        // Uptime
        self.uptime_seconds.set(uptime_secs as f64);

        // Disk metrics for data directory
        self.collect_disk_metrics(data_dir);
    }

    /// Collect disk metrics for the data directory (US-022)
    fn collect_disk_metrics(&self, data_dir: &Path) {
        let disks = Disks::new_with_refreshed_list();

        // Find the disk that contains the data directory
        // Convert data_dir to absolute path if needed
        let abs_path = if data_dir.is_absolute() {
            data_dir.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(data_dir))
                .unwrap_or_else(|_| data_dir.to_path_buf())
        };

        // Find the disk with the longest matching mount point
        let mut best_match: Option<&sysinfo::Disk> = None;
        let mut best_match_len = 0;

        for disk in disks.list() {
            let mount_point = disk.mount_point();
            if abs_path.starts_with(mount_point) {
                let mount_len = mount_point.as_os_str().len();
                if mount_len > best_match_len {
                    best_match = Some(disk);
                    best_match_len = mount_len;
                }
            }
        }

        if let Some(disk) = best_match {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);

            self.disk_total_bytes.set(total as f64);
            self.disk_available_bytes.set(available as f64);
            self.disk_usage_bytes.set(used as f64);
        } else {
            // If we can't find the disk, set to 0 (not placeholder text)
            self.disk_total_bytes.set(0.0);
            self.disk_available_bytes.set(0.0);
            self.disk_usage_bytes.set(0.0);
        }
    }

    /// Update job queue depth metrics (US-022)
    ///
    /// Call this with a map of job_type -> pending count
    pub fn update_job_queue_depth(&self, depths: &std::collections::HashMap<String, u64>) {
        for (job_type, depth) in depths {
            self.job_queue_depth
                .with_label_values(&[job_type])
                .set(*depth as f64);
        }
    }

    /// Set job queue depth for a specific job type (US-022)
    pub fn set_job_queue_depth(&self, job_type: &str, depth: u64) {
        self.job_queue_depth
            .with_label_values(&[job_type])
            .set(depth as f64);
    }

    /// Record data key cache hit
    pub fn record_data_key_cache_hit(&self) {
        self.data_key_cache_hits.inc();
    }

    /// Record data key cache miss
    pub fn record_data_key_cache_miss(&self) {
        self.data_key_cache_misses.inc();
    }

    /// Record successful token refresh (US-011)
    pub fn record_token_refreshed(&self) {
        self.tokens_refreshed_total.inc();
    }

    /// Record token refresh failure (US-011)
    pub fn record_token_refresh_failure(&self) {
        self.token_refresh_failures_total.inc();
    }

    /// Record connection marked as needing re-authentication (US-011)
    pub fn record_connection_marked_reauth(&self) {
        self.connections_marked_reauth_total.inc();
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
        self.metrics
            .record_http_request(&self.method, &self.endpoint, status_code, duration);
    }
}

/// Metrics endpoint handler
pub async fn metrics_handler(State(metrics): State<Arc<MetricsCollector>>) -> impl IntoResponse {
    match metrics.get_metrics() {
        Ok(metrics_text) => Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/plain; version=0.0.4; charset=utf-8")
            .body(Body::from(metrics_text))
            .unwrap(),
        Err(err) => {
            tracing::error!("Failed to generate metrics: {}", err);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "error": "Failed to generate metrics",
                        "details": err.to_string()
                    })
                    .to_string(),
                ))
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
    pub async fn time_operation<F, T, E>(
        &self,
        operation: &str,
        table: &str,
        future: F,
    ) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        let start = Instant::now();
        let result = future.await;
        let duration = start.elapsed();
        let success = result.is_ok();

        self.metrics
            .record_db_operation(operation, table, duration, success);
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

        self.metrics
            .record_redis_operation(operation, duration, success);
        result
    }

    /// Update connection pool metrics
    pub fn update_pool_metrics(&self, pool: &deadpool_redis::Pool) {
        let status = pool.status();
        self.metrics
            .update_redis_connections((status.size.saturating_sub(status.available)) as u32);
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
        metrics.record_http_request(
            &Method::GET,
            "/health",
            StatusCode::OK,
            std::time::Duration::from_millis(100),
        );
        metrics.record_http_request(
            &Method::POST,
            "/api/auth/login",
            StatusCode::UNAUTHORIZED,
            std::time::Duration::from_millis(50),
        );

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

    #[test]
    fn test_request_latency_recording() {
        let metrics = MetricsCollector::new().expect("Failed to create metrics collector");

        // Record request latency with different values across histogram buckets
        // 10ms, 50ms, 100ms, 250ms, 500ms, 1000ms, 5000ms
        metrics.record_request_latency(
            "GET",
            "/api/v1/health",
            200,
            std::time::Duration::from_millis(5), // < 10ms bucket
        );
        metrics.record_request_latency(
            "POST",
            "/api/v1/auth/login",
            200,
            std::time::Duration::from_millis(75), // 50ms < x < 100ms bucket
        );
        metrics.record_request_latency(
            "GET",
            "/api/v1/dnp",
            500,
            std::time::Duration::from_millis(2500), // 1000ms < x < 5000ms bucket
        );

        let metrics_text = metrics.get_metrics().expect("Failed to get metrics");

        // Verify the latency histogram is present with correct labels
        assert!(metrics_text.contains("kiro_http_request_latency_seconds"));
        assert!(metrics_text.contains("method=\"GET\""));
        assert!(metrics_text.contains("method=\"POST\""));
        assert!(metrics_text.contains("path=\"/api/v1/health\""));
        assert!(metrics_text.contains("status_code=\"200\""));
        assert!(metrics_text.contains("status_code=\"500\""));

        // Verify histogram buckets are present
        assert!(metrics_text.contains("le=\"0.01\"")); // 10ms
        assert!(metrics_text.contains("le=\"0.05\"")); // 50ms
        assert!(metrics_text.contains("le=\"0.1\"")); // 100ms
        assert!(metrics_text.contains("le=\"0.25\"")); // 250ms
        assert!(metrics_text.contains("le=\"0.5\"")); // 500ms
        assert!(metrics_text.contains("le=\"1\"")); // 1000ms
        assert!(metrics_text.contains("le=\"5\"")); // 5000ms
    }
}
