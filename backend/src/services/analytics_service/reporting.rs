//! Reporting Service
//!
//! Generates reports for analytics data export and scheduling.

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::services::databases::DuckDbClient;
use super::dashboard::{DashboardService, DashboardMetrics, TimeRange};
use super::trends::{TrendAnalysisService, TrendSummary};

/// Report types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportType {
    DailySummary,
    WeeklySummary,
    MonthlySummary,
    TrendAnalysis,
    PlatformHealth,
    OffenseReport,
    UserActivity,
    Custom,
}

impl std::fmt::Display for ReportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportType::DailySummary => write!(f, "daily_summary"),
            ReportType::WeeklySummary => write!(f, "weekly_summary"),
            ReportType::MonthlySummary => write!(f, "monthly_summary"),
            ReportType::TrendAnalysis => write!(f, "trend_analysis"),
            ReportType::PlatformHealth => write!(f, "platform_health"),
            ReportType::OffenseReport => write!(f, "offense_report"),
            ReportType::UserActivity => write!(f, "user_activity"),
            ReportType::Custom => write!(f, "custom"),
        }
    }
}

/// Report format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    Json,
    Csv,
    Parquet,
    Html,
}

/// Report status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportStatus {
    Pending,
    Generating,
    Ready,
    Failed,
    Expired,
}

/// A generated report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: Uuid,
    pub report_type: ReportType,
    pub format: ReportFormat,
    pub status: ReportStatus,
    pub time_range: String,
    pub generated_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub file_path: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub summary: Option<ReportSummary>,
    pub error: Option<String>,
}

/// Summary of report contents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_records: i64,
    pub key_metrics: Vec<KeyMetric>,
    pub highlights: Vec<String>,
}

/// Key metric in report summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetric {
    pub name: String,
    pub value: String,
    pub change: Option<f64>,
}

/// Report request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRequest {
    pub report_type: ReportType,
    pub format: ReportFormat,
    pub time_range: TimeRange,
    pub include_details: bool,
    pub filters: Option<ReportFilters>,
}

/// Report filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportFilters {
    pub artist_ids: Option<Vec<Uuid>>,
    pub platforms: Option<Vec<String>>,
    pub offense_categories: Option<Vec<String>>,
    pub min_mentions: Option<i64>,
}

/// Scheduled report configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledReport {
    pub id: Uuid,
    pub report_type: ReportType,
    pub format: ReportFormat,
    pub schedule: ReportSchedule,
    pub recipients: Vec<String>,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
}

/// Report schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportSchedule {
    Daily { hour: u32 },
    Weekly { day: u32, hour: u32 },
    Monthly { day: u32, hour: u32 },
}

/// Reporting service
pub struct ReportingService {
    duckdb: Arc<DuckDbClient>,
    dashboard: DashboardService,
    trends: TrendAnalysisService,
    output_dir: String,
}

impl ReportingService {
    pub fn new(
        duckdb: Arc<DuckDbClient>,
        dashboard: DashboardService,
        trends: TrendAnalysisService,
        output_dir: String,
    ) -> Self {
        Self {
            duckdb,
            dashboard,
            trends,
            output_dir,
        }
    }

    /// Generate a report
    pub async fn generate_report(&self, request: ReportRequest) -> Result<Report> {
        let id = Uuid::new_v4();
        let mut report = Report {
            id,
            report_type: request.report_type,
            format: request.format,
            status: ReportStatus::Generating,
            time_range: format!("{:?}", request.time_range),
            generated_at: None,
            expires_at: None,
            file_path: None,
            file_size_bytes: None,
            summary: None,
            error: None,
        };

        tracing::info!(
            report_id = %id,
            report_type = %request.report_type,
            "Generating report"
        );

        match self.generate_report_data(&request).await {
            Ok((summary, file_path, file_size)) => {
                report.status = ReportStatus::Ready;
                report.generated_at = Some(Utc::now());
                report.expires_at = Some(Utc::now() + Duration::days(7));
                report.file_path = file_path;
                report.file_size_bytes = file_size;
                report.summary = Some(summary);
            }
            Err(e) => {
                report.status = ReportStatus::Failed;
                report.error = Some(e.to_string());
            }
        }

        Ok(report)
    }

    /// Generate report data based on type
    async fn generate_report_data(
        &self,
        request: &ReportRequest,
    ) -> Result<(ReportSummary, Option<String>, Option<i64>)> {
        match request.report_type {
            ReportType::DailySummary | ReportType::WeeklySummary | ReportType::MonthlySummary => {
                self.generate_summary_report(request).await
            }
            ReportType::TrendAnalysis => self.generate_trend_report(request).await,
            ReportType::PlatformHealth => self.generate_platform_report(request).await,
            ReportType::OffenseReport => self.generate_offense_report(request).await,
            ReportType::UserActivity => self.generate_user_activity_report(request).await,
            ReportType::Custom => self.generate_custom_report(request).await,
        }
    }

    /// Generate summary report
    async fn generate_summary_report(
        &self,
        request: &ReportRequest,
    ) -> Result<(ReportSummary, Option<String>, Option<i64>)> {
        let metrics = self.dashboard.get_dashboard(request.time_range).await?;

        let highlights = vec![
            format!("{} new articles processed", metrics.content_metrics.total_articles),
            format!("{} offenses detected", metrics.content_metrics.offenses_detected),
            format!("{} new users joined", metrics.user_metrics.new_users),
            format!("System health: {}", metrics.system_health.overall_status),
        ];

        let key_metrics = vec![
            KeyMetric {
                name: "Total Articles".to_string(),
                value: metrics.content_metrics.total_articles.to_string(),
                change: None,
            },
            KeyMetric {
                name: "Offenses Detected".to_string(),
                value: metrics.content_metrics.offenses_detected.to_string(),
                change: None,
            },
            KeyMetric {
                name: "Active Users".to_string(),
                value: metrics.user_metrics.active_users.to_string(),
                change: None,
            },
            KeyMetric {
                name: "Sync Success Rate".to_string(),
                value: format!("{:.1}%", metrics.sync_metrics.success_rate * 100.0),
                change: None,
            },
        ];

        let summary = ReportSummary {
            total_records: metrics.content_metrics.total_articles,
            key_metrics,
            highlights,
        };

        // Generate file based on format
        let file_info = self.write_report_file(
            &request.report_type.to_string(),
            request.format,
            &serde_json::to_string_pretty(&metrics)?,
        ).await?;

        Ok((summary, file_info.0, file_info.1))
    }

    /// Generate trend report
    async fn generate_trend_report(
        &self,
        request: &ReportRequest,
    ) -> Result<(ReportSummary, Option<String>, Option<i64>)> {
        let trends = self.trends.get_trend_summary().await?;

        let highlights = vec![
            format!("{} rising artists", trends.top_rising_artists.len()),
            format!("{} falling artists", trends.top_falling_artists.len()),
        ];

        let key_metrics = vec![
            KeyMetric {
                name: "Content Volume Change".to_string(),
                value: format!("{:+.1}%", trends.content_volume_trend.change_percentage * 100.0),
                change: Some(trends.content_volume_trend.change_percentage),
            },
        ];

        let summary = ReportSummary {
            total_records: (trends.top_rising_artists.len() + trends.top_falling_artists.len()) as i64,
            key_metrics,
            highlights,
        };

        let file_info = self.write_report_file(
            &request.report_type.to_string(),
            request.format,
            &serde_json::to_string_pretty(&trends)?,
        ).await?;

        Ok((summary, file_info.0, file_info.1))
    }

    /// Generate platform health report
    async fn generate_platform_report(
        &self,
        request: &ReportRequest,
    ) -> Result<(ReportSummary, Option<String>, Option<i64>)> {
        let health = self.duckdb.get_platform_health(request.time_range.days()).await?;

        let highlights: Vec<String> = health
            .iter()
            .map(|p| format!("{}: {:.1}% success rate", p.platform, p.success_rate * 100.0))
            .collect();

        let key_metrics = health
            .iter()
            .map(|p| KeyMetric {
                name: p.platform.clone(),
                value: format!("{:.1}%", p.success_rate * 100.0),
                change: None,
            })
            .collect();

        let summary = ReportSummary {
            total_records: health.len() as i64,
            key_metrics,
            highlights,
        };

        let file_info = self.write_report_file(
            &request.report_type.to_string(),
            request.format,
            &serde_json::to_string_pretty(&health)?,
        ).await?;

        Ok((summary, file_info.0, file_info.1))
    }

    /// Generate offense report
    async fn generate_offense_report(
        &self,
        _request: &ReportRequest,
    ) -> Result<(ReportSummary, Option<String>, Option<i64>)> {
        // Placeholder - would query offense data
        let summary = ReportSummary {
            total_records: 0,
            key_metrics: vec![],
            highlights: vec!["No offense data available".to_string()],
        };

        Ok((summary, None, None))
    }

    /// Generate user activity report
    async fn generate_user_activity_report(
        &self,
        request: &ReportRequest,
    ) -> Result<(ReportSummary, Option<String>, Option<i64>)> {
        let metrics = self.dashboard.get_dashboard(request.time_range).await?;

        let highlights = vec![
            format!("{} total users", metrics.user_metrics.total_users),
            format!("{} active users", metrics.user_metrics.active_users),
            format!("{} new blocks", metrics.user_metrics.new_blocks),
        ];

        let key_metrics = vec![
            KeyMetric {
                name: "Total Users".to_string(),
                value: metrics.user_metrics.total_users.to_string(),
                change: None,
            },
            KeyMetric {
                name: "Active Users".to_string(),
                value: metrics.user_metrics.active_users.to_string(),
                change: None,
            },
            KeyMetric {
                name: "Avg Blocks/User".to_string(),
                value: format!("{:.1}", metrics.user_metrics.avg_blocks_per_user),
                change: None,
            },
        ];

        let summary = ReportSummary {
            total_records: metrics.user_metrics.total_users,
            key_metrics,
            highlights,
        };

        let file_info = self.write_report_file(
            &request.report_type.to_string(),
            request.format,
            &serde_json::to_string_pretty(&metrics.user_metrics)?,
        ).await?;

        Ok((summary, file_info.0, file_info.1))
    }

    /// Generate custom report
    async fn generate_custom_report(
        &self,
        _request: &ReportRequest,
    ) -> Result<(ReportSummary, Option<String>, Option<i64>)> {
        let summary = ReportSummary {
            total_records: 0,
            key_metrics: vec![],
            highlights: vec!["Custom reports require specific configuration".to_string()],
        };

        Ok((summary, None, None))
    }

    /// Write report to file
    async fn write_report_file(
        &self,
        name: &str,
        format: ReportFormat,
        content: &str,
    ) -> Result<(Option<String>, Option<i64>)> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let extension = match format {
            ReportFormat::Json => "json",
            ReportFormat::Csv => "csv",
            ReportFormat::Parquet => "parquet",
            ReportFormat::Html => "html",
        };

        let filename = format!("{}_{}.{}", name, timestamp, extension);
        let file_path = format!("{}/{}", self.output_dir, filename);

        // For JSON format, write the content directly
        if format == ReportFormat::Json {
            if let Err(e) = tokio::fs::write(&file_path, content).await {
                tracing::warn!(error = %e, "Failed to write report file");
                return Ok((None, None));
            }

            let size = content.len() as i64;
            return Ok((Some(file_path), Some(size)));
        }

        // For other formats, would need conversion
        Ok((None, None))
    }

    /// Export data to Parquet
    pub async fn export_to_parquet(&self, table: &str) -> Result<String> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let file_path = format!("{}/{}_{}.parquet", self.output_dir, table, timestamp);

        self.duckdb.export_to_parquet(table, &file_path).await?;

        Ok(file_path)
    }

    /// Get available report types
    pub fn get_report_types() -> Vec<ReportTypeInfo> {
        vec![
            ReportTypeInfo {
                report_type: ReportType::DailySummary,
                name: "Daily Summary".to_string(),
                description: "Summary of daily activity and metrics".to_string(),
            },
            ReportTypeInfo {
                report_type: ReportType::WeeklySummary,
                name: "Weekly Summary".to_string(),
                description: "Weekly rollup of key metrics".to_string(),
            },
            ReportTypeInfo {
                report_type: ReportType::TrendAnalysis,
                name: "Trend Analysis".to_string(),
                description: "Analysis of artist and content trends".to_string(),
            },
            ReportTypeInfo {
                report_type: ReportType::PlatformHealth,
                name: "Platform Health".to_string(),
                description: "Sync health across all platforms".to_string(),
            },
            ReportTypeInfo {
                report_type: ReportType::OffenseReport,
                name: "Offense Report".to_string(),
                description: "Detected offenses and classifications".to_string(),
            },
            ReportTypeInfo {
                report_type: ReportType::UserActivity,
                name: "User Activity".to_string(),
                description: "User engagement and block statistics".to_string(),
            },
        ]
    }
}

/// Report type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTypeInfo {
    pub report_type: ReportType,
    pub name: String,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_type_display() {
        assert_eq!(ReportType::DailySummary.to_string(), "daily_summary");
        assert_eq!(ReportType::TrendAnalysis.to_string(), "trend_analysis");
    }
}
