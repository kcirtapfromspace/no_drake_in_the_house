use std::collections::{BTreeMap, HashMap};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sysinfo::System;
use uuid::Uuid;

use crate::metrics::MetricsCollector;
use crate::services::monitoring::{AlertSeverity, HealthStatus, MonitoringService};

/// Enforcement success report for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementSuccessReport {
    pub user_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub success_rate: f64,
    pub provider_breakdown: HashMap<String, ProviderEnforcementStats>,
    pub operation_breakdown: HashMap<String, OperationStats>,
    pub recent_failures: Vec<EnforcementFailure>,
}

/// Provider-specific enforcement statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderEnforcementStats {
    pub provider: String,
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub success_rate: f64,
    pub avg_duration_ms: f64,
    pub most_common_operations: Vec<String>,
}

/// Operation-specific statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationStats {
    pub operation_type: String,
    pub total_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub success_rate: f64,
    pub avg_duration_ms: f64,
}

/// Enforcement failure details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementFailure {
    pub timestamp: DateTime<Utc>,
    pub provider: String,
    pub operation_type: String,
    pub error_message: String,
    pub retry_count: u32,
}

/// DNP list effectiveness analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnpListEffectivenessReport {
    pub user_id: Uuid,
    pub total_artists_blocked: u64,
    pub total_content_filtered: u64,
    pub content_breakdown: HashMap<String, ContentFilterStats>,
    pub top_blocked_artists: Vec<BlockedArtistStats>,
    pub filter_effectiveness_score: f64,
    pub recommendations: Vec<String>,
}

/// Content filtering statistics by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentFilterStats {
    pub content_type: String, // "liked_songs", "playlists", "recommendations", etc.
    pub total_items_scanned: u64,
    pub items_filtered: u64,
    pub filter_rate: f64,
}

/// Statistics for blocked artists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedArtistStats {
    pub artist_name: String,
    pub times_blocked: u64,
    pub content_types_blocked: Vec<String>,
    pub last_blocked: DateTime<Utc>,
}

/// Community list impact report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityListImpactReport {
    pub list_id: Uuid,
    pub list_name: String,
    pub curator_id: Uuid,
    pub total_subscribers: u64,
    pub active_subscribers: u64,
    pub total_artists: u64,
    pub total_content_filtered: u64,
    pub subscriber_satisfaction: f64,
    pub impact_metrics: CommunityListImpactMetrics,
    pub growth_metrics: CommunityListGrowthMetrics,
}

/// Detailed impact metrics for community lists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityListImpactMetrics {
    pub avg_content_filtered_per_user: f64,
    pub most_effective_artists: Vec<String>,
    pub provider_coverage: HashMap<String, f64>,
    pub user_feedback_score: f64,
}

/// Growth metrics for community lists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityListGrowthMetrics {
    pub subscribers_last_30_days: i64,
    pub subscribers_growth_rate: f64,
    pub content_additions_last_30_days: u64,
    pub engagement_score: f64,
}

/// System performance dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPerformanceDashboard {
    pub timestamp: DateTime<Utc>,
    pub overall_health: String,
    pub api_performance: ApiPerformanceMetrics,
    pub enforcement_performance: EnforcementPerformanceMetrics,
    pub user_activity: UserActivityMetrics,
    pub system_resources: SystemResourceMetrics,
    pub alerts: Vec<SystemAlert>,
}

/// API performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiPerformanceMetrics {
    pub requests_per_minute: f64,
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub error_rate: f64,
    pub uptime_percentage: f64,
}

/// Enforcement performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementPerformanceMetrics {
    pub operations_per_hour: f64,
    pub success_rate: f64,
    pub avg_operation_duration_ms: f64,
    pub queue_depth: u64,
    pub processing_capacity: f64,
}

/// User activity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivityMetrics {
    pub active_users_last_24h: u64,
    pub new_registrations_last_24h: u64,
    pub enforcement_operations_last_24h: u64,
    pub community_list_subscriptions_last_24h: u64,
}

/// System resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResourceMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub disk_usage_percent: f64,
    pub database_connections: u64,
    pub redis_memory_usage_mb: f64,
}

/// System alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAlert {
    pub id: String,
    pub severity: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub resolved: bool,
}

/// Analytics service for user-facing reports and dashboards
pub struct AnalyticsService {
    db_pool: PgPool,
    monitoring_service: Option<std::sync::Arc<MonitoringService>>,
    metrics_collector: Option<std::sync::Arc<MetricsCollector>>,
}

impl AnalyticsService {
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool,
            monitoring_service: None,
            metrics_collector: None,
        }
    }

    pub fn with_monitoring_service(
        mut self,
        monitoring_service: std::sync::Arc<MonitoringService>,
    ) -> Self {
        self.monitoring_service = Some(monitoring_service);
        self
    }

    pub fn with_metrics_collector(
        mut self,
        metrics_collector: std::sync::Arc<MetricsCollector>,
    ) -> Self {
        self.metrics_collector = Some(metrics_collector);
        self
    }

    /// Generate enforcement success report for a user
    pub async fn generate_enforcement_success_report(
        &self,
        user_id: Uuid,
        days_back: u32,
    ) -> Result<EnforcementSuccessReport, Box<dyn std::error::Error>> {
        let period_start = Utc::now() - Duration::days(days_back as i64);
        let period_end = Utc::now();

        // Get overall statistics
        let total_operations = self.get_total_operations(user_id, period_start, period_end).await?;
        let successful_operations = self.get_successful_operations(user_id, period_start, period_end).await?;
        let failed_operations = total_operations - successful_operations;
        let success_rate = if total_operations > 0 {
            (successful_operations as f64 / total_operations as f64) * 100.0
        } else {
            0.0
        };

        // Get provider breakdown
        let provider_breakdown = self.get_provider_breakdown(user_id, period_start, period_end).await?;

        // Get operation breakdown
        let operation_breakdown = self.get_operation_breakdown(user_id, period_start, period_end).await?;

        // Get recent failures
        let recent_failures = self.get_recent_failures(user_id, 10).await?;

        Ok(EnforcementSuccessReport {
            user_id,
            period_start,
            period_end,
            total_operations,
            successful_operations,
            failed_operations,
            success_rate,
            provider_breakdown,
            operation_breakdown,
            recent_failures,
        })
    }

    /// Generate DNP list effectiveness report
    pub async fn generate_dnp_effectiveness_report(
        &self,
        user_id: Uuid,
    ) -> Result<DnpListEffectivenessReport, Box<dyn std::error::Error>> {
        let total_artists_blocked = self.get_total_artists_blocked(user_id).await?;
        let total_content_filtered = self.get_total_content_filtered(user_id).await?;
        let content_breakdown = self.get_content_breakdown(user_id).await?;
        let top_blocked_artists = self.get_top_blocked_artists(user_id, 10).await?;
        
        // Calculate effectiveness score based on content filtered vs. total content
        let filter_effectiveness_score = self.calculate_filter_effectiveness_score(user_id).await?;
        
        // Generate recommendations
        let recommendations = self.generate_dnp_recommendations(user_id).await?;

        Ok(DnpListEffectivenessReport {
            user_id,
            total_artists_blocked,
            total_content_filtered,
            content_breakdown,
            top_blocked_artists,
            filter_effectiveness_score,
            recommendations,
        })
    }

    /// Generate community list impact report
    pub async fn generate_community_list_impact_report(
        &self,
        list_id: Uuid,
    ) -> Result<CommunityListImpactReport, Box<dyn std::error::Error>> {
        let list_info = self.get_community_list_info(list_id).await?;
        let total_subscribers = self.get_total_subscribers(list_id).await?;
        let active_subscribers = self.get_active_subscribers(list_id).await?;
        let total_artists = self.get_community_list_artist_count(list_id).await?;
        let total_content_filtered = self.get_community_list_content_filtered(list_id).await?;
        let subscriber_satisfaction = self.get_subscriber_satisfaction(list_id).await?;
        
        let impact_metrics = self.get_community_list_impact_metrics(list_id).await?;
        let growth_metrics = self.get_community_list_growth_metrics(list_id).await?;

        Ok(CommunityListImpactReport {
            list_id,
            list_name: list_info.0,
            curator_id: list_info.1,
            total_subscribers,
            active_subscribers,
            total_artists,
            total_content_filtered,
            subscriber_satisfaction,
            impact_metrics,
            growth_metrics,
        })
    }

    /// Generate system performance dashboard
    pub async fn generate_system_performance_dashboard(
        &self,
    ) -> Result<SystemPerformanceDashboard, Box<dyn std::error::Error>> {
        let timestamp = Utc::now();
        let overall_health = self.get_overall_system_health().await?;
        let api_performance = self.get_api_performance_metrics().await?;
        let enforcement_performance = self.get_enforcement_performance_metrics().await?;
        let user_activity = self.get_user_activity_metrics().await?;
        let system_resources = self.get_system_resource_metrics().await?;
        let alerts = self.get_active_system_alerts().await?;

        Ok(SystemPerformanceDashboard {
            timestamp,
            overall_health,
            api_performance,
            enforcement_performance,
            user_activity,
            system_resources,
            alerts,
        })
    }

    // Private helper methods for data retrieval

    async fn get_total_operations(
        &self,
        user_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM action_batches WHERE user_id = $1 AND created_at BETWEEN $2 AND $3",
            user_id,
            start,
            end
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        Ok(count as u64)
    }

    async fn get_successful_operations(
        &self,
        user_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM action_batches WHERE user_id = $1 AND created_at BETWEEN $2 AND $3 AND status = 'completed'",
            user_id,
            start,
            end
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        Ok(count as u64)
    }

    async fn get_provider_breakdown(
        &self,
        user_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<HashMap<String, ProviderEnforcementStats>, Box<dyn std::error::Error>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                provider,
                COUNT(*) as total_operations,
                COUNT(CASE WHEN status = 'completed' THEN 1 END) as successful_operations,
                AVG(EXTRACT(EPOCH FROM (completed_at - created_at)) * 1000) as avg_duration_ms
            FROM action_batches 
            WHERE user_id = $1 AND created_at BETWEEN $2 AND $3
            GROUP BY provider
            "#,
            user_id,
            start,
            end
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut breakdown = HashMap::new();
        for row in rows {
            let total = row.total_operations.unwrap_or(0) as u64;
            let successful = row.successful_operations.unwrap_or(0) as u64;
            let failed = total - successful;
            let success_rate = if total > 0 {
                (successful as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            breakdown.insert(
                row.provider.clone(),
                ProviderEnforcementStats {
                    provider: row.provider.clone(),
                    total_operations: total,
                    successful_operations: successful,
                    failed_operations: failed,
                    success_rate,
                    avg_duration_ms: row.avg_duration_ms.unwrap_or(0.0),
                    most_common_operations: vec![], // TODO: Implement
                },
            );
        }

        Ok(breakdown)
    }

    async fn get_operation_breakdown(
        &self,
        user_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<HashMap<String, OperationStats>, Box<dyn std::error::Error>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                ai.action,
                COUNT(*) as total_count,
                COUNT(CASE WHEN ai.status = 'completed' THEN 1 END) as success_count
            FROM action_items ai
            JOIN action_batches ab ON ai.batch_id = ab.id
            WHERE ab.user_id = $1 AND ab.created_at BETWEEN $2 AND $3
            GROUP BY ai.action
            "#,
            user_id,
            start,
            end
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut breakdown = HashMap::new();
        for row in rows {
            let total = row.total_count.unwrap_or(0) as u64;
            let success = row.success_count.unwrap_or(0) as u64;
            let failure = total - success;
            let success_rate = if total > 0 {
                (success as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            breakdown.insert(
                row.action.clone(),
                OperationStats {
                    operation_type: row.action,
                    total_count: total,
                    success_count: success,
                    failure_count: failure,
                    success_rate,
                    avg_duration_ms: 0.0, // TODO: Calculate from timing data
                },
            );
        }

        Ok(breakdown)
    }

    async fn get_recent_failures(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<EnforcementFailure>, Box<dyn std::error::Error>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                ab.created_at,
                ab.provider,
                ai.action,
                ai.error_message
            FROM action_items ai
            JOIN action_batches ab ON ai.batch_id = ab.id
            WHERE ab.user_id = $1 AND ai.status = 'failed'
            ORDER BY ab.created_at DESC
            LIMIT $2
            "#,
            user_id,
            limit
        )
        .fetch_all(&self.db_pool)
        .await?;

        let failures = rows
            .into_iter()
            .map(|row| EnforcementFailure {
                timestamp: row.created_at.unwrap_or_else(Utc::now),
                provider: row.provider,
                operation_type: row.action,
                error_message: row.error_message.unwrap_or_default(),
                retry_count: 0, // TODO: Track retry count
            })
            .collect();

        Ok(failures)
    }

    async fn get_total_artists_blocked(&self, user_id: Uuid) -> Result<u64, Box<dyn std::error::Error>> {
        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM user_artist_blocks WHERE user_id = $1",
            user_id
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        Ok(count as u64)
    }

    async fn get_total_content_filtered(&self, user_id: Uuid) -> Result<u64, Box<dyn std::error::Error>> {
        let count: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM action_items ai
            JOIN action_batches ab ON ai.batch_id = ab.id
            WHERE ab.user_id = $1 AND ai.status = 'completed'
            "#,
            user_id
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        Ok(count as u64)
    }

    async fn get_content_breakdown(&self, user_id: Uuid) -> Result<HashMap<String, ContentFilterStats>, Box<dyn std::error::Error>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                ai.entity_type,
                COUNT(*) as items_filtered
            FROM action_items ai
            JOIN action_batches ab ON ai.batch_id = ab.id
            WHERE ab.user_id = $1 AND ai.status = 'completed'
            GROUP BY ai.entity_type
            "#,
            user_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut breakdown = HashMap::new();
        for row in rows {
            breakdown.insert(
                row.entity_type.clone(),
                ContentFilterStats {
                    content_type: row.entity_type,
                    total_items_scanned: 0, // TODO: Track scanned items
                    items_filtered: row.items_filtered.unwrap_or(0) as u64,
                    filter_rate: 0.0, // TODO: Calculate filter rate
                },
            );
        }

        Ok(breakdown)
    }

    async fn get_top_blocked_artists(&self, user_id: Uuid, limit: i64) -> Result<Vec<BlockedArtistStats>, Box<dyn std::error::Error>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                a.canonical_name,
                COUNT(ai.id) as times_blocked,
                MAX(ai.created_at) as last_blocked
            FROM user_artist_blocks uab
            JOIN artists a ON uab.artist_id = a.id
            LEFT JOIN action_items ai ON ai.entity_id = a.id::text
            LEFT JOIN action_batches ab ON ai.batch_id = ab.id AND ab.user_id = $1
            WHERE uab.user_id = $1
            GROUP BY a.canonical_name
            ORDER BY times_blocked DESC
            LIMIT $2
            "#,
            user_id,
            limit
        )
        .fetch_all(&self.db_pool)
        .await?;

        let stats = rows
            .into_iter()
            .map(|row| BlockedArtistStats {
                artist_name: row.canonical_name,
                times_blocked: row.times_blocked.unwrap_or(0) as u64,
                content_types_blocked: vec![], // TODO: Implement
                last_blocked: row.last_blocked.unwrap_or_else(Utc::now),
            })
            .collect();

        Ok(stats)
    }

    async fn calculate_filter_effectiveness_score(&self, user_id: Uuid) -> Result<f64, Box<dyn std::error::Error>> {
        // Simple effectiveness score based on successful operations vs total operations
        let total = self.get_total_operations(user_id, Utc::now() - Duration::days(30), Utc::now()).await?;
        let successful = self.get_successful_operations(user_id, Utc::now() - Duration::days(30), Utc::now()).await?;
        
        if total > 0 {
            Ok((successful as f64 / total as f64) * 100.0)
        } else {
            Ok(0.0)
        }
    }

    async fn generate_dnp_recommendations(&self, _user_id: Uuid) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // TODO: Implement ML-based recommendations
        Ok(vec![
            "Consider adding more artists from genres you frequently block".to_string(),
            "Review your community list subscriptions for better coverage".to_string(),
            "Enable auto-skip for better user experience".to_string(),
        ])
    }

    // Community list methods
    async fn get_community_list_info(&self, list_id: Uuid) -> Result<(String, Uuid), Box<dyn std::error::Error>> {
        let row = sqlx::query!(
            "SELECT name, owner_user_id FROM community_lists WHERE id = $1",
            list_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok((row.name, row.owner_user_id.unwrap_or_default()))
    }

    async fn get_total_subscribers(&self, list_id: Uuid) -> Result<u64, Box<dyn std::error::Error>> {
        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM user_list_subscriptions WHERE list_id = $1",
            list_id
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        Ok(count as u64)
    }

    async fn get_active_subscribers(&self, list_id: Uuid) -> Result<u64, Box<dyn std::error::Error>> {
        // Active subscribers are those who have used the list in the last 30 days
        let count: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(DISTINCT uls.user_id) 
            FROM user_list_subscriptions uls
            JOIN action_batches ab ON ab.user_id = uls.user_id
            WHERE uls.list_id = $1 AND ab.created_at > NOW() - INTERVAL '30 days'
            "#,
            list_id
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        Ok(count as u64)
    }

    async fn get_community_list_artist_count(&self, list_id: Uuid) -> Result<u64, Box<dyn std::error::Error>> {
        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM community_list_items WHERE list_id = $1",
            list_id
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        Ok(count as u64)
    }

    async fn get_community_list_content_filtered(&self, _list_id: Uuid) -> Result<u64, Box<dyn std::error::Error>> {
        // TODO: Implement tracking of content filtered by community lists
        Ok(0)
    }

    async fn get_subscriber_satisfaction(&self, _list_id: Uuid) -> Result<f64, Box<dyn std::error::Error>> {
        // TODO: Implement subscriber satisfaction tracking
        Ok(85.0) // Placeholder
    }

    async fn get_community_list_impact_metrics(&self, _list_id: Uuid) -> Result<CommunityListImpactMetrics, Box<dyn std::error::Error>> {
        // TODO: Implement detailed impact metrics
        Ok(CommunityListImpactMetrics {
            avg_content_filtered_per_user: 0.0,
            most_effective_artists: vec![],
            provider_coverage: HashMap::new(),
            user_feedback_score: 0.0,
        })
    }

    async fn get_community_list_growth_metrics(&self, list_id: Uuid) -> Result<CommunityListGrowthMetrics, Box<dyn std::error::Error>> {
        let subscribers_last_30_days: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM user_list_subscriptions WHERE list_id = $1 AND created_at > NOW() - INTERVAL '30 days'",
            list_id
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        // TODO: Calculate growth rate and other metrics
        Ok(CommunityListGrowthMetrics {
            subscribers_last_30_days,
            subscribers_growth_rate: 0.0,
            content_additions_last_30_days: 0,
            engagement_score: 0.0,
        })
    }

    // System performance methods
    async fn get_overall_system_health(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut signals: Vec<HealthStatus> = Vec::new();

        if let Some(monitoring_service) = &self.monitoring_service {
            let health_checks = monitoring_service.get_health_checks().await;
            if health_checks.is_empty() {
                // No explicit health checks configured, avoid defaulting to "Healthy".
            } else {
                signals.extend(health_checks.values().map(|check| check.status.clone()));
            }

            let alerts = monitoring_service.get_recent_alerts(10).await;
            if alerts
                .iter()
                .any(|alert| matches!(alert.severity, AlertSeverity::Critical))
            {
                signals.push(HealthStatus::Unhealthy);
            } else if alerts
                .iter()
                .any(|alert| matches!(alert.severity, AlertSeverity::Warning))
            {
                signals.push(HealthStatus::Degraded);
            }
        }

        if let Some(snapshot) = self.telemetry_snapshot().await {
            if let Some(status) = health_from_telemetry(&snapshot) {
                signals.push(status);
            }
        }

        let status_label = match aggregate_health_status(&signals) {
            Some(HealthStatus::Healthy) => "Healthy",
            Some(HealthStatus::Degraded) => "Degraded",
            Some(HealthStatus::Unhealthy) => "Unhealthy",
            None => "Unknown",
        };

        Ok(status_label.to_string())
    }

    async fn get_api_performance_metrics(&self) -> Result<ApiPerformanceMetrics, Box<dyn std::error::Error>> {
        if let Some(snapshot) = self.telemetry_snapshot().await {
            let total_requests = snapshot.total_requests;
            let uptime_minutes = if snapshot.uptime_seconds > 0.0 {
                snapshot.uptime_seconds / 60.0
            } else {
                0.0
            };
            let requests_per_minute = if uptime_minutes > 0.0 {
                total_requests / uptime_minutes
            } else {
                0.0
            };
            let avg_response_time_ms = if snapshot.request_duration_count > 0.0 {
                (snapshot.request_duration_sum / snapshot.request_duration_count) * 1000.0
            } else {
                0.0
            };
            let p95_response_time_ms = percentile_from_buckets(
                &snapshot.request_duration_buckets,
                snapshot.request_duration_count,
                0.95,
            ) * 1000.0;
            let error_rate = if total_requests > 0.0 {
                (snapshot.error_requests / total_requests) * 100.0
            } else {
                0.0
            };
            let uptime_percentage = if snapshot.uptime_seconds > 0.0 {
                100.0
            } else {
                0.0
            };

            return Ok(ApiPerformanceMetrics {
                requests_per_minute,
                avg_response_time_ms,
                p95_response_time_ms,
                error_rate,
                uptime_percentage,
            });
        }

        Ok(ApiPerformanceMetrics {
            requests_per_minute: 0.0,
            avg_response_time_ms: 0.0,
            p95_response_time_ms: 0.0,
            error_rate: 0.0,
            uptime_percentage: 0.0,
        })
    }

    async fn get_enforcement_performance_metrics(&self) -> Result<EnforcementPerformanceMetrics, Box<dyn std::error::Error>> {
        let operations_last_hour: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM action_batches WHERE created_at > NOW() - INTERVAL '1 hour'"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        let operations_last_24h: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM action_batches WHERE created_at > NOW() - INTERVAL '24 hours'"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        let successful_last_24h: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM action_batches WHERE created_at > NOW() - INTERVAL '24 hours' AND status = 'completed'"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        let avg_duration_ms: Option<f64> = sqlx::query_scalar!(
            r#"
            SELECT AVG(EXTRACT(EPOCH FROM (completed_at - created_at)) * 1000)
            FROM action_batches
            WHERE created_at > NOW() - INTERVAL '24 hours'
              AND completed_at IS NOT NULL
            "#
        )
        .fetch_one(&self.db_pool)
        .await?;

        let queue_depth: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM action_batches WHERE status IN ('pending', 'processing')"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        let success_rate = if operations_last_24h > 0 {
            (successful_last_24h as f64 / operations_last_24h as f64) * 100.0
        } else {
            0.0
        };

        let avg_operation_duration_ms = avg_duration_ms.unwrap_or(0.0);
        let processing_capacity = if avg_operation_duration_ms > 0.0 {
            3_600_000.0 / avg_operation_duration_ms
        } else {
            0.0
        };

        Ok(EnforcementPerformanceMetrics {
            operations_per_hour: operations_last_hour as f64,
            success_rate,
            avg_operation_duration_ms,
            queue_depth: queue_depth.max(0) as u64,
            processing_capacity,
        })
    }

    async fn get_user_activity_metrics(&self) -> Result<UserActivityMetrics, Box<dyn std::error::Error>> {
        let active_users_last_24h: i64 = sqlx::query_scalar!(
            "SELECT COUNT(DISTINCT user_id) FROM action_batches WHERE created_at > NOW() - INTERVAL '24 hours'"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        let new_registrations_last_24h: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE created_at > NOW() - INTERVAL '24 hours'"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        let enforcement_operations_last_24h: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM action_batches WHERE created_at > NOW() - INTERVAL '24 hours'"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        let community_list_subscriptions_last_24h: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM user_list_subscriptions WHERE created_at > NOW() - INTERVAL '24 hours'"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        Ok(UserActivityMetrics {
            active_users_last_24h: active_users_last_24h as u64,
            new_registrations_last_24h: new_registrations_last_24h as u64,
            enforcement_operations_last_24h: enforcement_operations_last_24h as u64,
            community_list_subscriptions_last_24h: community_list_subscriptions_last_24h as u64,
        })
    }

    async fn get_system_resource_metrics(&self) -> Result<SystemResourceMetrics, Box<dyn std::error::Error>> {
        let mut cpu_usage_percent = 0.0;
        let mut memory_usage_percent = 0.0;
        let mut disk_usage_percent = 0.0;

        if let Some(snapshot) = self.telemetry_snapshot().await {
            if let Some(cpu) = snapshot.cpu_usage_percent {
                cpu_usage_percent = cpu;
            }
            if let Some(memory_percent) = snapshot.memory_usage_percent {
                memory_usage_percent = memory_percent;
            } else if let Some(memory_bytes) = snapshot.memory_usage_bytes {
                let mut system = System::new_all();
                system.refresh_memory();
                let total_memory = system.total_memory() as f64;
                if total_memory > 0.0 {
                    memory_usage_percent = (memory_bytes / total_memory) * 100.0;
                }
            }
            if let Some(disk_percent) = snapshot.disk_usage_percent {
                disk_usage_percent = disk_percent;
            }
        }

        if cpu_usage_percent == 0.0 || memory_usage_percent == 0.0 {
            let mut system = System::new_all();
            system.refresh_all();
            if cpu_usage_percent == 0.0 {
                cpu_usage_percent = system.global_cpu_info().cpu_usage() as f64;
            }
            if memory_usage_percent == 0.0 {
                let total_memory = system.total_memory() as f64;
                if total_memory > 0.0 {
                    memory_usage_percent =
                        (system.used_memory() as f64 / total_memory) * 100.0;
                }
            }
        }

        let database_connections = (self.db_pool.size() as u64)
            .saturating_sub(self.db_pool.num_idle() as u64);

        Ok(SystemResourceMetrics {
            cpu_usage_percent,
            memory_usage_percent,
            disk_usage_percent,
            database_connections,
            redis_memory_usage_mb: 0.0,
        })
    }

    async fn get_active_system_alerts(&self) -> Result<Vec<SystemAlert>, Box<dyn std::error::Error>> {
        if let Some(monitoring_service) = &self.monitoring_service {
            let alerts = monitoring_service.get_recent_alerts(25).await;
            let mapped = alerts
                .into_iter()
                .map(|alert| SystemAlert {
                    id: alert.id,
                    severity: match alert.severity {
                        AlertSeverity::Critical => "Critical".to_string(),
                        AlertSeverity::Warning => "Warning".to_string(),
                        AlertSeverity::Info => "Info".to_string(),
                    },
                    message: alert.message,
                    timestamp: alert.timestamp,
                    resolved: false,
                })
                .collect();
            return Ok(mapped);
        }

        Ok(Vec::new())
    }

    async fn telemetry_snapshot(&self) -> Option<TelemetrySnapshot> {
        if let Some(monitoring_service) = &self.monitoring_service {
            if let Ok(metrics) = monitoring_service.export_metrics() {
                return Some(parse_prometheus_metrics(&metrics));
            }
        }

        if let Some(metrics_collector) = &self.metrics_collector {
            if let Ok(metrics) = metrics_collector.get_metrics() {
                return Some(parse_prometheus_metrics(&metrics));
            }
        }

        None
    }
}

#[derive(Default)]
struct TelemetrySnapshot {
    total_requests: f64,
    error_requests: f64,
    request_duration_sum: f64,
    request_duration_count: f64,
    request_duration_buckets: BTreeMap<f64, f64>,
    uptime_seconds: f64,
    cpu_usage_percent: Option<f64>,
    memory_usage_bytes: Option<f64>,
    memory_usage_percent: Option<f64>,
    disk_usage_percent: Option<f64>,
}

fn parse_prometheus_metrics(metrics: &str) -> TelemetrySnapshot {
    let mut snapshot = TelemetrySnapshot::default();

    for line in metrics.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.split_whitespace();
        let name_with_labels = match parts.next() {
            Some(value) => value,
            None => continue,
        };
        let value_str = match parts.next() {
            Some(value) => value,
            None => continue,
        };
        let value: f64 = match value_str.parse() {
            Ok(parsed) => parsed,
            Err(_) => continue,
        };

        let (name, labels) = split_metric_labels(name_with_labels);

        match name {
            "kiro_http_requests_total" | "kiro_http_http_requests_total" | "http_requests_total" => {
                snapshot.total_requests += value;
                if let Some(status_code) = labels.get("status_code") {
                    if let Ok(code) = status_code.parse::<u16>() {
                        if code >= 400 {
                            snapshot.error_requests += value;
                        }
                    }
                }
            }
            "kiro_http_request_duration_seconds_sum"
            | "kiro_http_http_request_duration_seconds_sum"
            | "http_request_duration_seconds_sum" => {
                snapshot.request_duration_sum += value;
            }
            "kiro_http_request_duration_seconds_count"
            | "kiro_http_http_request_duration_seconds_count"
            | "http_request_duration_seconds_count" => {
                snapshot.request_duration_count += value;
            }
            "kiro_http_request_duration_seconds_bucket"
            | "kiro_http_http_request_duration_seconds_bucket"
            | "http_request_duration_seconds_bucket" => {
                if let Some(le_value) = labels.get("le") {
                    if let Ok(bound) = parse_le_bound(le_value) {
                        let entry = snapshot.request_duration_buckets.entry(bound).or_insert(0.0);
                        *entry += value;
                    }
                }
            }
            "kiro_uptime_seconds" => {
                if value > snapshot.uptime_seconds {
                    snapshot.uptime_seconds = value;
                }
            }
            "system_cpu_usage_percent" | "kiro_cpu_usage_percent" => {
                snapshot.cpu_usage_percent = Some(value);
            }
            "system_memory_usage_percent" => {
                snapshot.memory_usage_percent = Some(value);
            }
            "system_disk_usage_percent" => {
                snapshot.disk_usage_percent = Some(value);
            }
            "kiro_memory_usage_bytes" => {
                snapshot.memory_usage_bytes = Some(value);
            }
            _ => {}
        }
    }

    snapshot
}

fn split_metric_labels(metric: &str) -> (&str, HashMap<String, String>) {
    if let Some(start) = metric.find('{') {
        if let Some(end) = metric.rfind('}') {
            let name = &metric[..start];
            let label_str = &metric[start + 1..end];
            return (name, parse_labels(label_str));
        }
    }

    (metric, HashMap::new())
}

fn parse_labels(label_str: &str) -> HashMap<String, String> {
    let mut labels = HashMap::new();

    for part in label_str.split(',') {
        let mut split = part.splitn(2, '=');
        let key = match split.next() {
            Some(value) => value.trim(),
            None => continue,
        };
        let value = match split.next() {
            Some(value) => value.trim(),
            None => continue,
        };
        let trimmed = value.trim_matches('"');
        if !key.is_empty() {
            labels.insert(key.to_string(), trimmed.to_string());
        }
    }

    labels
}

fn parse_le_bound(value: &str) -> Result<f64, std::num::ParseFloatError> {
    if value == "+Inf" {
        Ok(f64::INFINITY)
    } else {
        value.parse()
    }
}

fn aggregate_health_status(statuses: &[HealthStatus]) -> Option<HealthStatus> {
    if statuses.is_empty() {
        return None;
    }

    if statuses.iter().any(|status| matches!(status, HealthStatus::Unhealthy)) {
        return Some(HealthStatus::Unhealthy);
    }
    if statuses.iter().any(|status| matches!(status, HealthStatus::Degraded)) {
        return Some(HealthStatus::Degraded);
    }

    Some(HealthStatus::Healthy)
}

fn health_from_telemetry(snapshot: &TelemetrySnapshot) -> Option<HealthStatus> {
    let mut has_signal = false;
    let mut degraded = false;
    let mut unhealthy = false;

    if snapshot.total_requests > 0.0 {
        has_signal = true;
        let error_rate = (snapshot.error_requests / snapshot.total_requests) * 100.0;
        if error_rate >= 10.0 {
            unhealthy = true;
        } else if error_rate >= 5.0 {
            degraded = true;
        }
    }

    if let Some(cpu) = snapshot.cpu_usage_percent {
        has_signal = true;
        if cpu >= 95.0 {
            unhealthy = true;
        } else if cpu >= 85.0 {
            degraded = true;
        }
    }

    if let Some(memory_percent) = snapshot.memory_usage_percent {
        has_signal = true;
        if memory_percent >= 95.0 {
            unhealthy = true;
        } else if memory_percent >= 85.0 {
            degraded = true;
        }
    }

    if let Some(disk_percent) = snapshot.disk_usage_percent {
        has_signal = true;
        if disk_percent >= 95.0 {
            unhealthy = true;
        } else if disk_percent >= 90.0 {
            degraded = true;
        }
    }

    if !has_signal {
        return None;
    }

    if unhealthy {
        Some(HealthStatus::Unhealthy)
    } else if degraded {
        Some(HealthStatus::Degraded)
    } else {
        Some(HealthStatus::Healthy)
    }
}

fn percentile_from_buckets(buckets: &BTreeMap<f64, f64>, count: f64, percentile: f64) -> f64 {
    if count <= 0.0 || buckets.is_empty() {
        return 0.0;
    }

    let target = count * percentile;
    let mut last_finite = 0.0;
    for (bound, cumulative) in buckets {
        if bound.is_finite() {
            last_finite = *bound;
        }
        if *cumulative >= target {
            if bound.is_finite() {
                return *bound;
            }
            return last_finite;
        }
    }

    last_finite
}
