use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use prometheus::{
    Gauge, GaugeVec, HistogramVec, IntCounterVec,
    IntGauge, IntGaugeVec, Registry, TextEncoder, Encoder,
};
use serde::{Deserialize, Serialize};
use sysinfo::System;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// Correlation ID for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationId(pub String);

impl CorrelationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Structured log entry with correlation ID
#[derive(Debug, Serialize)]
pub struct StructuredLogEntry {
    pub timestamp: DateTime<Utc>,
    pub correlation_id: String,
    pub level: String,
    pub service: String,
    pub operation: String,
    pub user_id: Option<Uuid>,
    pub duration_ms: Option<u64>,
    pub status: String,
    pub message: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Health check status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Health check result for a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub service: String,
    pub status: HealthStatus,
    pub last_check: DateTime<Utc>,
    pub response_time_ms: u64,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// System metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage_percent: f32,
    pub memory_usage_percent: f32,
    pub disk_usage_percent: f32,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub active_connections: u64,
    pub goroutines: u64,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

/// Alert definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub name: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// SLO (Service Level Objective) definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLO {
    pub name: String,
    pub target_percentage: f64,
    pub measurement_window_hours: u64,
    pub current_percentage: f64,
    pub error_budget_remaining: f64,
}

/// Monitoring service for application observability
pub struct MonitoringService {
    registry: Registry,
    system: Arc<RwLock<System>>,
    health_checks: Arc<RwLock<HashMap<String, HealthCheck>>>,
    alerts: Arc<RwLock<Vec<Alert>>>,
    slos: Arc<RwLock<HashMap<String, SLO>>>,
    
    // Prometheus metrics
    http_requests_total: IntCounterVec,
    http_request_duration: HistogramVec,
    database_connections_active: IntGauge,
    database_query_duration: HistogramVec,
    enforcement_operations_total: IntCounterVec,
    enforcement_success_rate: GaugeVec,
    api_rate_limit_hits: IntCounterVec,
    external_api_calls_total: IntCounterVec,
    external_api_errors_total: IntCounterVec,
    job_queue_size: IntGaugeVec,
    job_processing_duration: HistogramVec,
    system_cpu_usage: Gauge,
    system_memory_usage: Gauge,
    system_disk_usage: Gauge,
    active_users: IntGauge,
    dnp_list_size: HistogramVec,
    community_list_subscriptions: IntGaugeVec,
}

impl MonitoringService {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let registry = Registry::new();
        let system = Arc::new(RwLock::new(System::new_all()));

        // Initialize Prometheus metrics
        let http_requests_total = IntCounterVec::new(
            prometheus::Opts::new("http_requests_total", "Total HTTP requests"),
            &["method", "endpoint", "status_code"],
        )?;
        registry.register(Box::new(http_requests_total.clone()))?;

        let http_request_duration = HistogramVec::new(
            prometheus::HistogramOpts::new("http_request_duration_seconds", "HTTP request duration"),
            &["method", "endpoint"],
        )?;
        registry.register(Box::new(http_request_duration.clone()))?;

        let database_connections_active = IntGauge::new(
            "database_connections_active",
            "Active database connections",
        )?;
        registry.register(Box::new(database_connections_active.clone()))?;

        let database_query_duration = HistogramVec::new(
            prometheus::HistogramOpts::new("database_query_duration_seconds", "Database query duration"),
            &["query_type", "table"],
        )?;
        registry.register(Box::new(database_query_duration.clone()))?;

        let enforcement_operations_total = IntCounterVec::new(
            prometheus::Opts::new("enforcement_operations_total", "Total enforcement operations"),
            &["provider", "operation_type", "status"],
        )?;
        registry.register(Box::new(enforcement_operations_total.clone()))?;

        let enforcement_success_rate = GaugeVec::new(
            prometheus::Opts::new("enforcement_success_rate", "Enforcement operation success rate"),
            &["provider", "operation_type"],
        )?;
        registry.register(Box::new(enforcement_success_rate.clone()))?;

        let api_rate_limit_hits = IntCounterVec::new(
            prometheus::Opts::new("api_rate_limit_hits_total", "API rate limit hits"),
            &["provider", "endpoint"],
        )?;
        registry.register(Box::new(api_rate_limit_hits.clone()))?;

        let external_api_calls_total = IntCounterVec::new(
            prometheus::Opts::new("external_api_calls_total", "Total external API calls"),
            &["provider", "endpoint", "status"],
        )?;
        registry.register(Box::new(external_api_calls_total.clone()))?;

        let external_api_errors_total = IntCounterVec::new(
            prometheus::Opts::new("external_api_errors_total", "Total external API errors"),
            &["provider", "endpoint", "error_type"],
        )?;
        registry.register(Box::new(external_api_errors_total.clone()))?;

        let job_queue_size = IntGaugeVec::new(
            prometheus::Opts::new("job_queue_size", "Job queue size"),
            &["queue_name", "status"],
        )?;
        registry.register(Box::new(job_queue_size.clone()))?;

        let job_processing_duration = HistogramVec::new(
            prometheus::HistogramOpts::new("job_processing_duration_seconds", "Job processing duration"),
            &["job_type", "status"],
        )?;
        registry.register(Box::new(job_processing_duration.clone()))?;

        let system_cpu_usage = Gauge::new("system_cpu_usage_percent", "System CPU usage percentage")?;
        registry.register(Box::new(system_cpu_usage.clone()))?;

        let system_memory_usage = Gauge::new("system_memory_usage_percent", "System memory usage percentage")?;
        registry.register(Box::new(system_memory_usage.clone()))?;

        let system_disk_usage = Gauge::new("system_disk_usage_percent", "System disk usage percentage")?;
        registry.register(Box::new(system_disk_usage.clone()))?;

        let active_users = IntGauge::new("active_users", "Number of active users")?;
        registry.register(Box::new(active_users.clone()))?;

        let dnp_list_size = HistogramVec::new(
            prometheus::HistogramOpts::new("dnp_list_size", "DNP list size distribution"),
            &["user_type"],
        )?;
        registry.register(Box::new(dnp_list_size.clone()))?;

        let community_list_subscriptions = IntGaugeVec::new(
            prometheus::Opts::new("community_list_subscriptions", "Community list subscriptions"),
            &["list_id", "list_name"],
        )?;
        registry.register(Box::new(community_list_subscriptions.clone()))?;

        Ok(Self {
            registry,
            system,
            health_checks: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            slos: Arc::new(RwLock::new(HashMap::new())),
            http_requests_total,
            http_request_duration,
            database_connections_active,
            database_query_duration,
            enforcement_operations_total,
            enforcement_success_rate,
            api_rate_limit_hits,
            external_api_calls_total,
            external_api_errors_total,
            job_queue_size,
            job_processing_duration,
            system_cpu_usage,
            system_memory_usage,
            system_disk_usage,
            active_users,
            dnp_list_size,
            community_list_subscriptions,
        })
    }

    /// Record HTTP request metrics
    pub fn record_http_request(&self, method: &str, endpoint: &str, status_code: u16, duration: Duration) {
        self.http_requests_total
            .with_label_values(&[method, endpoint, &status_code.to_string()])
            .inc();
        
        self.http_request_duration
            .with_label_values(&[method, endpoint])
            .observe(duration.as_secs_f64());
    }

    /// Record database query metrics
    pub fn record_database_query(&self, query_type: &str, table: &str, duration: Duration) {
        self.database_query_duration
            .with_label_values(&[query_type, table])
            .observe(duration.as_secs_f64());
    }

    /// Record enforcement operation metrics
    pub fn record_enforcement_operation(&self, provider: &str, operation_type: &str, success: bool, _duration: Duration) {
        let status = if success { "success" } else { "failure" };
        
        self.enforcement_operations_total
            .with_label_values(&[provider, operation_type, status])
            .inc();

        // Update success rate (simplified - in production, use a sliding window)
        let current_rate = if success { 1.0 } else { 0.0 };
        self.enforcement_success_rate
            .with_label_values(&[provider, operation_type])
            .set(current_rate);
    }

    /// Record API rate limit hit
    pub fn record_rate_limit_hit(&self, provider: &str, endpoint: &str) {
        self.api_rate_limit_hits
            .with_label_values(&[provider, endpoint])
            .inc();
    }

    /// Record external API call
    pub fn record_external_api_call(&self, provider: &str, endpoint: &str, status: &str) {
        self.external_api_calls_total
            .with_label_values(&[provider, endpoint, status])
            .inc();
    }

    /// Record external API error
    pub fn record_external_api_error(&self, provider: &str, endpoint: &str, error_type: &str) {
        self.external_api_errors_total
            .with_label_values(&[provider, endpoint, error_type])
            .inc();
    }

    /// Update job queue metrics
    pub fn update_job_queue_size(&self, queue_name: &str, pending: i64, processing: i64, failed: i64) {
        self.job_queue_size
            .with_label_values(&[queue_name, "pending"])
            .set(pending);
        self.job_queue_size
            .with_label_values(&[queue_name, "processing"])
            .set(processing);
        self.job_queue_size
            .with_label_values(&[queue_name, "failed"])
            .set(failed);
    }

    /// Record job processing duration
    pub fn record_job_processing(&self, job_type: &str, success: bool, duration: Duration) {
        let status = if success { "success" } else { "failure" };
        self.job_processing_duration
            .with_label_values(&[job_type, status])
            .observe(duration.as_secs_f64());
    }

    /// Update system metrics
    pub async fn update_system_metrics(&self) {
        let mut system = self.system.write().await;
        system.refresh_all();

        // CPU usage
        let cpu_usage = system.global_cpu_info().cpu_usage();
        self.system_cpu_usage.set(cpu_usage as f64);

        // Memory usage
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let memory_usage = (used_memory as f64 / total_memory as f64) * 100.0;
        self.system_memory_usage.set(memory_usage);

        // Disk usage - simplified for now
        // TODO: Fix disk access when sysinfo API is stable
        self.system_disk_usage.set(0.0);
    }

    /// Update active users count
    pub fn update_active_users(&self, count: i64) {
        self.active_users.set(count);
    }

    /// Record DNP list size
    pub fn record_dnp_list_size(&self, user_type: &str, size: usize) {
        self.dnp_list_size
            .with_label_values(&[user_type])
            .observe(size as f64);
    }

    /// Update community list subscription count
    pub fn update_community_list_subscriptions(&self, list_id: &str, list_name: &str, count: i64) {
        self.community_list_subscriptions
            .with_label_values(&[list_id, list_name])
            .set(count);
    }

    /// Log structured entry with correlation ID
    pub fn log_structured(&self, entry: StructuredLogEntry) {
        let json_entry = serde_json::to_string(&entry).unwrap_or_else(|_| "Failed to serialize log entry".to_string());
        
        match entry.level.as_str() {
            "ERROR" => error!(correlation_id = %entry.correlation_id, "{}", json_entry),
            "WARN" => warn!(correlation_id = %entry.correlation_id, "{}", json_entry),
            "INFO" => info!(correlation_id = %entry.correlation_id, "{}", json_entry),
            "DEBUG" => debug!(correlation_id = %entry.correlation_id, "{}", json_entry),
            _ => info!(correlation_id = %entry.correlation_id, "{}", json_entry),
        }
    }

    /// Add or update health check
    pub async fn update_health_check(&self, health_check: HealthCheck) {
        let mut health_checks = self.health_checks.write().await;
        health_checks.insert(health_check.service.clone(), health_check);
    }

    /// Get all health checks
    pub async fn get_health_checks(&self) -> HashMap<String, HealthCheck> {
        self.health_checks.read().await.clone()
    }

    /// Check overall system health
    pub async fn get_overall_health(&self) -> HealthStatus {
        let health_checks = self.health_checks.read().await;
        
        let mut _healthy_count = 0;
        let mut degraded_count = 0;
        let mut unhealthy_count = 0;
        
        for health_check in health_checks.values() {
            match health_check.status {
                HealthStatus::Healthy => _healthy_count += 1,
                HealthStatus::Degraded => degraded_count += 1,
                HealthStatus::Unhealthy => unhealthy_count += 1,
            }
        }
        
        if unhealthy_count > 0 {
            HealthStatus::Unhealthy
        } else if degraded_count > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    /// Trigger alert
    pub async fn trigger_alert(&self, alert: Alert) {
        let mut alerts = self.alerts.write().await;
        alerts.push(alert.clone());
        
        // Log the alert
        let log_entry = StructuredLogEntry {
            timestamp: Utc::now(),
            correlation_id: alert.correlation_id.unwrap_or_else(|| CorrelationId::new().0),
            level: match alert.severity {
                AlertSeverity::Critical => "ERROR".to_string(),
                AlertSeverity::Warning => "WARN".to_string(),
                AlertSeverity::Info => "INFO".to_string(),
            },
            service: "monitoring".to_string(),
            operation: "alert_triggered".to_string(),
            user_id: None,
            duration_ms: None,
            status: "triggered".to_string(),
            message: alert.message.clone(),
            metadata: alert.metadata.clone(),
        };
        
        self.log_structured(log_entry);
        
        // In production, this would send to alerting systems like PagerDuty, Slack, etc.
        match alert.severity {
            AlertSeverity::Critical => {
                error!("CRITICAL ALERT: {} - {}", alert.name, alert.message);
            }
            AlertSeverity::Warning => {
                warn!("WARNING ALERT: {} - {}", alert.name, alert.message);
            }
            AlertSeverity::Info => {
                info!("INFO ALERT: {} - {}", alert.name, alert.message);
            }
        }
    }

    /// Get recent alerts
    pub async fn get_recent_alerts(&self, limit: usize) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts.iter().rev().take(limit).cloned().collect()
    }

    /// Update SLO
    pub async fn update_slo(&self, slo: SLO) {
        let mut slos = self.slos.write().await;
        slos.insert(slo.name.clone(), slo);
    }

    /// Get all SLOs
    pub async fn get_slos(&self) -> HashMap<String, SLO> {
        self.slos.read().await.clone()
    }

    /// Check SLO violations and trigger alerts
    pub async fn check_slo_violations(&self) {
        let slos = self.slos.read().await;
        
        for slo in slos.values() {
            if slo.current_percentage < slo.target_percentage {
                let error_budget_burned = ((slo.target_percentage - slo.current_percentage) / slo.target_percentage) * 100.0;
                
                if error_budget_burned > 50.0 {
                    let alert = Alert {
                        id: Uuid::new_v4().to_string(),
                        name: format!("SLO Violation: {}", slo.name),
                        severity: AlertSeverity::Critical,
                        message: format!(
                            "SLO '{}' is at {:.2}% (target: {:.2}%), error budget {:.2}% burned",
                            slo.name, slo.current_percentage, slo.target_percentage, error_budget_burned
                        ),
                        timestamp: Utc::now(),
                        correlation_id: None,
                        metadata: {
                            let mut metadata = HashMap::new();
                            metadata.insert("slo_name".to_string(), serde_json::Value::String(slo.name.clone()));
                            metadata.insert("current_percentage".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(slo.current_percentage).unwrap()));
                            metadata.insert("target_percentage".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(slo.target_percentage).unwrap()));
                            metadata.insert("error_budget_burned".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(error_budget_burned).unwrap()));
                            metadata
                        },
                    };
                    
                    drop(slos); // Release the lock before triggering alert
                    self.trigger_alert(alert).await;
                    return;
                }
            }
        }
    }

    /// Export Prometheus metrics
    pub fn export_metrics(&self) -> Result<String, Box<dyn std::error::Error>> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    /// Start background monitoring tasks
    pub async fn start_background_tasks(&self) {
        let monitoring_service = Arc::new(self.clone());
        
        // System metrics update task
        let system_monitor = monitoring_service.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                system_monitor.update_system_metrics().await;
            }
        });
        
        // SLO violation check task
        let slo_monitor = monitoring_service.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                slo_monitor.check_slo_violations().await;
            }
        });
        
        // Health check task
        let health_monitor = monitoring_service.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                // This would perform actual health checks on services
                // For now, we'll just log that the health check task is running
                debug!("Health check task running");
            }
        });
    }
}

// Clone implementation for Arc usage
impl Clone for MonitoringService {
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            system: self.system.clone(),
            health_checks: self.health_checks.clone(),
            alerts: self.alerts.clone(),
            slos: self.slos.clone(),
            http_requests_total: self.http_requests_total.clone(),
            http_request_duration: self.http_request_duration.clone(),
            database_connections_active: self.database_connections_active.clone(),
            database_query_duration: self.database_query_duration.clone(),
            enforcement_operations_total: self.enforcement_operations_total.clone(),
            enforcement_success_rate: self.enforcement_success_rate.clone(),
            api_rate_limit_hits: self.api_rate_limit_hits.clone(),
            external_api_calls_total: self.external_api_calls_total.clone(),
            external_api_errors_total: self.external_api_errors_total.clone(),
            job_queue_size: self.job_queue_size.clone(),
            job_processing_duration: self.job_processing_duration.clone(),
            system_cpu_usage: self.system_cpu_usage.clone(),
            system_memory_usage: self.system_memory_usage.clone(),
            system_disk_usage: self.system_disk_usage.clone(),
            active_users: self.active_users.clone(),
            dnp_list_size: self.dnp_list_size.clone(),
            community_list_subscriptions: self.community_list_subscriptions.clone(),
        }
    }
}

/// Health check implementations for various services
pub struct HealthCheckService {
    monitoring_service: Arc<MonitoringService>,
}

impl HealthCheckService {
    pub fn new(monitoring_service: Arc<MonitoringService>) -> Self {
        Self { monitoring_service }
    }

    /// Check database health
    pub async fn check_database_health(&self, db_pool: &sqlx::PgPool) -> HealthCheck {
        let start = Instant::now();
        
        match sqlx::query("SELECT 1").fetch_one(db_pool).await {
            Ok(_) => HealthCheck {
                service: "database".to_string(),
                status: HealthStatus::Healthy,
                last_check: Utc::now(),
                response_time_ms: start.elapsed().as_millis() as u64,
                error_message: None,
                metadata: HashMap::new(),
            },
            Err(e) => HealthCheck {
                service: "database".to_string(),
                status: HealthStatus::Unhealthy,
                last_check: Utc::now(),
                response_time_ms: start.elapsed().as_millis() as u64,
                error_message: Some(e.to_string()),
                metadata: HashMap::new(),
            },
        }
    }

    /// Check Redis health
    pub async fn check_redis_health(&self, redis_url: &str) -> HealthCheck {
        let start = Instant::now();
        
        match redis::Client::open(redis_url) {
            Ok(client) => {
                match client.get_connection() {
                    Ok(mut conn) => {
                        match redis::cmd("PING").query::<String>(&mut conn) {
                            Ok(_) => HealthCheck {
                                service: "redis".to_string(),
                                status: HealthStatus::Healthy,
                                last_check: Utc::now(),
                                response_time_ms: start.elapsed().as_millis() as u64,
                                error_message: None,
                                metadata: HashMap::new(),
                            },
                            Err(e) => HealthCheck {
                                service: "redis".to_string(),
                                status: HealthStatus::Unhealthy,
                                last_check: Utc::now(),
                                response_time_ms: start.elapsed().as_millis() as u64,
                                error_message: Some(e.to_string()),
                                metadata: HashMap::new(),
                            },
                        }
                    }
                    Err(e) => HealthCheck {
                        service: "redis".to_string(),
                        status: HealthStatus::Unhealthy,
                        last_check: Utc::now(),
                        response_time_ms: start.elapsed().as_millis() as u64,
                        error_message: Some(e.to_string()),
                        metadata: HashMap::new(),
                    },
                }
            }
            Err(e) => HealthCheck {
                service: "redis".to_string(),
                status: HealthStatus::Unhealthy,
                last_check: Utc::now(),
                response_time_ms: start.elapsed().as_millis() as u64,
                error_message: Some(e.to_string()),
                metadata: HashMap::new(),
            },
        }
    }

    /// Check external API health (Spotify)
    pub async fn check_spotify_api_health(&self) -> HealthCheck {
        let start = Instant::now();
        let client = reqwest::Client::new();
        
        // Check Spotify Web API status (public endpoint)
        match client
            .get("https://api.spotify.com/v1/browse/categories?limit=1")
            .header("Authorization", "Bearer invalid_token") // We expect 401, but service should be up
            .send()
            .await
        {
            Ok(response) => {
                let status = if response.status() == 401 {
                    HealthStatus::Healthy // 401 means API is up but token is invalid (expected)
                } else if response.status().is_server_error() {
                    HealthStatus::Unhealthy
                } else {
                    HealthStatus::Healthy
                };
                
                HealthCheck {
                    service: "spotify_api".to_string(),
                    status,
                    last_check: Utc::now(),
                    response_time_ms: start.elapsed().as_millis() as u64,
                    error_message: None,
                    metadata: {
                        let mut metadata = HashMap::new();
                        metadata.insert("status_code".to_string(), serde_json::Value::Number(serde_json::Number::from(response.status().as_u16())));
                        metadata
                    },
                }
            }
            Err(e) => HealthCheck {
                service: "spotify_api".to_string(),
                status: HealthStatus::Unhealthy,
                last_check: Utc::now(),
                response_time_ms: start.elapsed().as_millis() as u64,
                error_message: Some(e.to_string()),
                metadata: HashMap::new(),
            },
        }
    }

    /// Check Apple Music API health
    pub async fn check_apple_music_api_health(&self) -> HealthCheck {
        let start = Instant::now();
        let client = reqwest::Client::new();
        
        // Check Apple Music API status (public endpoint)
        match client
            .get("https://api.music.apple.com/v1/catalog/us/songs?ids=1441164670")
            .header("Authorization", "Bearer invalid_token") // We expect 401, but service should be up
            .send()
            .await
        {
            Ok(response) => {
                let status = if response.status() == 401 {
                    HealthStatus::Healthy // 401 means API is up but token is invalid (expected)
                } else if response.status().is_server_error() {
                    HealthStatus::Unhealthy
                } else {
                    HealthStatus::Healthy
                };
                
                HealthCheck {
                    service: "apple_music_api".to_string(),
                    status,
                    last_check: Utc::now(),
                    response_time_ms: start.elapsed().as_millis() as u64,
                    error_message: None,
                    metadata: {
                        let mut metadata = HashMap::new();
                        metadata.insert("status_code".to_string(), serde_json::Value::Number(serde_json::Number::from(response.status().as_u16())));
                        metadata
                    },
                }
            }
            Err(e) => HealthCheck {
                service: "apple_music_api".to_string(),
                status: HealthStatus::Unhealthy,
                last_check: Utc::now(),
                response_time_ms: start.elapsed().as_millis() as u64,
                error_message: Some(e.to_string()),
                metadata: HashMap::new(),
            },
        }
    }

    /// Run all health checks and update monitoring service
    pub async fn run_all_health_checks(&self, db_pool: &sqlx::PgPool, redis_url: &str) {
        let checks = vec![
            self.check_database_health(db_pool).await,
            self.check_redis_health(redis_url).await,
            self.check_spotify_api_health().await,
            self.check_apple_music_api_health().await,
        ];
        
        for check in checks {
            self.monitoring_service.update_health_check(check).await;
        }
    }
}