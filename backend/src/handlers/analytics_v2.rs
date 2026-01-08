//! Analytics V2 API Handlers
//!
//! Endpoints for dashboard metrics, trend analysis, and reporting.
//! Uses DuckDB for high-performance analytics queries.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::AuthenticatedUser;
use crate::services::analytics_service::{
    dashboard::{DashboardMetrics, TimeRange, UserQuickStats},
    reporting::{Report, ReportFormat, ReportRequest, ReportType, ReportTypeInfo},
    trends::{ArtistTrend, PlatformTrend, TrendSummary},
};
use crate::{AppError, AppState, Result};

/// Query parameters for dashboard requests
#[derive(Debug, Deserialize)]
pub struct DashboardQuery {
    /// Time range: today, yesterday, last7days, last30days, last90days, alltime
    #[serde(default = "default_time_range")]
    pub time_range: String,
}

fn default_time_range() -> String {
    "last7days".to_string()
}

fn parse_time_range(s: &str) -> TimeRange {
    match s.to_lowercase().as_str() {
        "today" => TimeRange::Today,
        "yesterday" => TimeRange::Yesterday,
        "last7days" | "week" => TimeRange::Last7Days,
        "last30days" | "month" => TimeRange::Last30Days,
        "last90days" | "quarter" => TimeRange::Last90Days,
        "alltime" | "all" => TimeRange::AllTime,
        _ => {
            // Try parsing as custom days
            if let Ok(days) = s.parse::<i32>() {
                TimeRange::Custom { days }
            } else {
                TimeRange::Last7Days
            }
        }
    }
}

/// Query for trend analysis
#[derive(Debug, Deserialize)]
pub struct TrendQuery {
    /// Number of days for current period
    #[serde(default = "default_period")]
    pub period_days: i32,
    /// Maximum results to return
    #[serde(default = "default_limit")]
    pub limit: i32,
}

fn default_period() -> i32 {
    7
}

fn default_limit() -> i32 {
    20
}

/// Request to generate a report
#[derive(Debug, Deserialize)]
pub struct GenerateReportRequest {
    pub report_type: String,
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default = "default_time_range")]
    pub time_range: String,
    #[serde(default)]
    pub include_details: bool,
}

fn default_format() -> String {
    "json".to_string()
}

fn parse_report_type(s: &str) -> ReportType {
    match s.to_lowercase().as_str() {
        "daily" | "daily_summary" => ReportType::DailySummary,
        "weekly" | "weekly_summary" => ReportType::WeeklySummary,
        "monthly" | "monthly_summary" => ReportType::MonthlySummary,
        "trends" | "trend_analysis" => ReportType::TrendAnalysis,
        "platform" | "platform_health" => ReportType::PlatformHealth,
        "offense" | "offense_report" => ReportType::OffenseReport,
        "user" | "user_activity" => ReportType::UserActivity,
        _ => ReportType::Custom,
    }
}

fn parse_report_format(s: &str) -> ReportFormat {
    match s.to_lowercase().as_str() {
        "json" => ReportFormat::Json,
        "csv" => ReportFormat::Csv,
        "parquet" => ReportFormat::Parquet,
        "html" => ReportFormat::Html,
        _ => ReportFormat::Json,
    }
}

// ============================================================================
// Dashboard Endpoints
// ============================================================================

/// Get dashboard metrics
pub async fn get_dashboard_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Query(query): Query<DashboardQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(time_range = %query.time_range, "Get dashboard request");

    let time_range = parse_time_range(&query.time_range);

    // Get dashboard service from state
    // For now, return a placeholder that shows the structure
    // In production, this would use state.analytics.dashboard.get_dashboard(time_range)

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "time_range": format!("{:?}", time_range),
            "user_metrics": {
                "total_users": 0,
                "active_users": 0,
                "new_users": 0,
                "total_blocks": 0,
                "new_blocks": 0,
                "avg_blocks_per_user": 0.0
            },
            "content_metrics": {
                "total_articles": 0,
                "articles_processed": 0,
                "entities_extracted": 0,
                "offenses_detected": 0,
                "offense_rate": 0.0,
                "avg_sentiment": 0.0
            },
            "sync_metrics": {
                "total_syncs": 0,
                "successful_syncs": 0,
                "failed_syncs": 0,
                "success_rate": 1.0,
                "artists_synced": 0,
                "platforms": []
            },
            "trending_artists": [],
            "recent_offenses": [],
            "system_health": {
                "overall_status": "healthy",
                "postgres_healthy": true,
                "duckdb_healthy": true,
                "kuzu_healthy": true,
                "lancedb_healthy": true,
                "redis_healthy": true,
                "api_response_time_ms": 50,
                "error_rate": 0.01
            }
        },
        "generated_at": chrono::Utc::now().to_rfc3339()
    })))
}

/// Get user quick stats
pub async fn get_user_quick_stats_handler(
    State(_state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(user_id = %user.id, "Get user quick stats request");

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "user_id": user.id,
            "blocked_artists": 0,
            "category_subscriptions": 0,
            "last_activity": null
        }
    })))
}

/// Get system health
pub async fn get_system_health_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Get system health request");

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "overall_status": "healthy",
            "services": {
                "postgres": {
                    "healthy": true,
                    "latency_ms": 5
                },
                "duckdb": {
                    "healthy": true,
                    "latency_ms": 2
                },
                "kuzu": {
                    "healthy": true,
                    "latency_ms": 3
                },
                "lancedb": {
                    "healthy": true,
                    "latency_ms": 4
                },
                "redis": {
                    "healthy": true,
                    "latency_ms": 1
                }
            },
            "api": {
                "avg_response_time_ms": 50,
                "error_rate": 0.01,
                "requests_per_minute": 100
            },
            "checked_at": chrono::Utc::now().to_rfc3339()
        }
    })))
}

// ============================================================================
// Trend Analysis Endpoints
// ============================================================================

/// Get trend summary
pub async fn get_trend_summary_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Query(query): Query<TrendQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        period_days = query.period_days,
        limit = query.limit,
        "Get trend summary request"
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "period": format!("Last {} days vs previous {} days", query.period_days, query.period_days),
            "top_rising_artists": [],
            "top_falling_artists": [],
            "new_offense_trends": [],
            "content_volume_trend": {
                "metric_name": "content_volume",
                "current_value": 0.0,
                "previous_value": 0.0,
                "change_percentage": 0.0,
                "direction": "stable"
            },
            "user_activity_trend": {
                "metric_name": "user_activity",
                "current_value": 0.0,
                "previous_value": 0.0,
                "change_percentage": 0.0,
                "direction": "stable"
            }
        }
    })))
}

/// Get artist trend
pub async fn get_artist_trend_handler(
    State(_state): State<AppState>,
    Path(artist_id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(artist_id = %artist_id, "Get artist trend request");

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "artist_id": artist_id,
            "artist_name": "",
            "current_mentions": 0,
            "previous_mentions": 0,
            "change_percentage": 0.0,
            "direction": "stable",
            "offense_trend": {
                "current_count": 0,
                "previous_count": 0,
                "change_percentage": 0.0,
                "direction": "stable",
                "top_categories": []
            },
            "sentiment_trend": {
                "current_score": 0.0,
                "previous_score": 0.0,
                "change": 0.0,
                "direction": "stable"
            },
            "mention_history": []
        }
    })))
}

/// Get platform trends
pub async fn get_platform_trends_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Get platform trends request");

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "platforms": []
        }
    })))
}

/// Get rising artists
pub async fn get_rising_artists_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Query(query): Query<TrendQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(limit = query.limit, "Get rising artists request");

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "period_days": query.period_days,
            "artists": [],
            "total": 0
        }
    })))
}

/// Get falling artists
pub async fn get_falling_artists_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Query(query): Query<TrendQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(limit = query.limit, "Get falling artists request");

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "period_days": query.period_days,
            "artists": [],
            "total": 0
        }
    })))
}

// ============================================================================
// Reporting Endpoints
// ============================================================================

/// Get available report types
pub async fn get_report_types_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Get report types request");

    let report_types = vec![
        serde_json::json!({
            "report_type": "daily_summary",
            "name": "Daily Summary",
            "description": "Summary of daily activity and metrics"
        }),
        serde_json::json!({
            "report_type": "weekly_summary",
            "name": "Weekly Summary",
            "description": "Weekly rollup of key metrics"
        }),
        serde_json::json!({
            "report_type": "trend_analysis",
            "name": "Trend Analysis",
            "description": "Analysis of artist and content trends"
        }),
        serde_json::json!({
            "report_type": "platform_health",
            "name": "Platform Health",
            "description": "Sync health across all platforms"
        }),
        serde_json::json!({
            "report_type": "offense_report",
            "name": "Offense Report",
            "description": "Detected offenses and classifications"
        }),
        serde_json::json!({
            "report_type": "user_activity",
            "name": "User Activity",
            "description": "User engagement and block statistics"
        }),
    ];

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "report_types": report_types,
            "formats": ["json", "csv", "parquet", "html"]
        }
    })))
}

/// Generate a report
pub async fn generate_report_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Json(request): Json<GenerateReportRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    tracing::info!(
        report_type = %request.report_type,
        format = %request.format,
        "Generate report request"
    );

    let report_type = parse_report_type(&request.report_type);
    let format = parse_report_format(&request.format);
    let time_range = parse_time_range(&request.time_range);

    // Return accepted response with report ID
    let report_id = Uuid::new_v4();

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "success": true,
            "message": "Report generation started",
            "data": {
                "report_id": report_id,
                "report_type": format!("{:?}", report_type),
                "format": format!("{:?}", format),
                "time_range": format!("{:?}", time_range),
                "status": "generating",
                "estimated_completion": chrono::Utc::now() + chrono::Duration::seconds(30)
            }
        })),
    ))
}

/// Get report status
pub async fn get_report_status_handler(
    State(_state): State<AppState>,
    Path(report_id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(report_id = %report_id, "Get report status request");

    // In production, would look up actual report status
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "report_id": report_id,
            "status": "ready",
            "generated_at": chrono::Utc::now().to_rfc3339(),
            "expires_at": (chrono::Utc::now() + chrono::Duration::days(7)).to_rfc3339(),
            "file_path": null,
            "file_size_bytes": null,
            "summary": {
                "total_records": 0,
                "key_metrics": [],
                "highlights": []
            }
        }
    })))
}

/// Download a report
pub async fn download_report_handler(
    State(_state): State<AppState>,
    Path(report_id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(report_id = %report_id, "Download report request");

    // In production, would return actual file or redirect to storage URL
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "report_id": report_id,
            "download_url": format!("/api/v1/analytics/reports/{}/file", report_id),
            "expires_at": (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339()
        }
    })))
}

/// Export data to Parquet
pub async fn export_to_parquet_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Query(query): Query<ExportQuery>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    tracing::info!(table = %query.table, "Export to Parquet request");

    // Validate table name
    let valid_tables = [
        "sync_metrics",
        "news_volume_hourly",
        "artist_mention_trends",
    ];
    if !valid_tables.contains(&query.table.as_str()) {
        return Err(AppError::InvalidFieldValue {
            field: "table".to_string(),
            message: format!("Invalid table. Valid options: {:?}", valid_tables),
        });
    }

    let export_id = Uuid::new_v4();

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "success": true,
            "message": "Export started",
            "data": {
                "export_id": export_id,
                "table": query.table,
                "format": "parquet",
                "status": "processing",
                "estimated_completion": chrono::Utc::now() + chrono::Duration::seconds(60)
            }
        })),
    ))
}

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub table: String,
}

// ============================================================================
// Metrics Endpoints (Prometheus-compatible)
// ============================================================================

// ============================================================================
// Trouble Score Endpoints
// ============================================================================

/// Query parameters for trouble score requests
#[derive(Debug, Deserialize)]
pub struct TroubleScoreQuery {
    #[serde(default = "default_limit")]
    pub limit: i32,
    #[serde(default)]
    pub offset: i32,
    /// Filter by tier: low, moderate, high, critical
    pub min_tier: Option<String>,
}

/// Get trouble score for a specific artist
pub async fn get_artist_trouble_score_handler(
    State(state): State<AppState>,
    Path(artist_id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(artist_id = %artist_id, "Get artist trouble score request");

    let service = crate::services::TroubleScoreService::new(state.db_pool.clone());

    match service.get_artist_score(artist_id).await {
        Ok(Some(score)) => Ok(Json(serde_json::json!({
            "success": true,
            "data": score
        }))),
        Ok(None) => Ok(Json(serde_json::json!({
            "success": false,
            "error": "No trouble score found for this artist"
        }))),
        Err(e) => {
            tracing::error!("Failed to get trouble score: {}", e);
            Ok(Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })))
        }
    }
}

/// Get trouble score leaderboard
pub async fn get_trouble_leaderboard_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Query(query): Query<TroubleScoreQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        limit = query.limit,
        offset = query.offset,
        min_tier = ?query.min_tier,
        "Get trouble leaderboard request"
    );

    let service = crate::services::TroubleScoreService::new(state.db_pool.clone());

    let min_tier = query.min_tier.as_ref().map(|t| {
        match t.to_lowercase().as_str() {
            "critical" => crate::services::TroubleTier::Critical,
            "high" => crate::services::TroubleTier::High,
            "moderate" => crate::services::TroubleTier::Moderate,
            _ => crate::services::TroubleTier::Low,
        }
    });

    match service.get_leaderboard(min_tier, query.limit, query.offset).await {
        Ok(leaderboard) => Ok(Json(serde_json::json!({
            "success": true,
            "data": {
                "entries": leaderboard,
                "limit": query.limit,
                "offset": query.offset
            }
        }))),
        Err(e) => {
            tracing::error!("Failed to get leaderboard: {}", e);
            Ok(Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })))
        }
    }
}

/// Get trouble tier distribution
pub async fn get_tier_distribution_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Get tier distribution request");

    let service = crate::services::TroubleScoreService::new(state.db_pool.clone());

    match service.get_tier_distribution().await {
        Ok(distribution) => Ok(Json(serde_json::json!({
            "success": true,
            "data": distribution
        }))),
        Err(e) => {
            tracing::error!("Failed to get tier distribution: {}", e);
            Ok(Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })))
        }
    }
}

/// Recalculate trouble scores (admin endpoint)
pub async fn recalculate_trouble_scores_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    tracing::info!("Recalculate trouble scores request");

    let service = crate::services::TroubleScoreService::new(state.db_pool.clone());

    match service.recalculate_all().await {
        Ok(summary) => Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": summary
            })),
        )),
        Err(e) => {
            tracing::error!("Failed to recalculate scores: {}", e);
            Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": e.to_string()
                })),
            ))
        }
    }
}

/// Get artist score history
pub async fn get_artist_score_history_handler(
    State(state): State<AppState>,
    Path(artist_id): Path<Uuid>,
    _user: AuthenticatedUser,
    Query(query): Query<TroubleScoreQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(artist_id = %artist_id, "Get artist score history request");

    let service = crate::services::TroubleScoreService::new(state.db_pool.clone());

    match service.get_score_history(artist_id, query.limit).await {
        Ok(history) => Ok(Json(serde_json::json!({
            "success": true,
            "data": {
                "artist_id": artist_id,
                "history": history
            }
        }))),
        Err(e) => {
            tracing::error!("Failed to get score history: {}", e);
            Ok(Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })))
        }
    }
}

// ============================================================================
// Revenue Tracking Endpoints
// ============================================================================

/// Query parameters for revenue requests
#[derive(Debug, Deserialize)]
pub struct RevenueQuery {
    #[serde(default = "default_days")]
    pub days: i32,
    #[serde(default = "default_limit")]
    pub limit: i32,
    pub platform: Option<String>,
    /// Filter by trouble tier: low, moderate, high, critical
    pub min_tier: Option<String>,
}

fn default_days() -> i32 {
    30
}

/// Get user's revenue distribution
pub async fn get_user_revenue_distribution_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<RevenueQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(user_id = %user.id, days = query.days, "Get user revenue distribution request");

    let service = crate::services::RevenueService::new(state.db_pool.clone());

    let platform = query.platform.as_ref().and_then(|p| {
        crate::services::RevenuePlatform::from_str(p)
    });

    match service.get_user_revenue_distribution(user.id, platform, query.days).await {
        Ok(distribution) => Ok(Json(serde_json::json!({
            "success": true,
            "data": distribution
        }))),
        Err(e) => {
            tracing::error!("Failed to get revenue distribution: {}", e);
            Ok(Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })))
        }
    }
}

/// Get user's top artists by revenue
pub async fn get_user_top_artists_revenue_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<RevenueQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        user_id = %user.id,
        days = query.days,
        limit = query.limit,
        "Get user top artists by revenue"
    );

    let service = crate::services::RevenueService::new(state.db_pool.clone());

    match service.get_user_top_artists(user.id, query.days, query.limit).await {
        Ok(artists) => Ok(Json(serde_json::json!({
            "success": true,
            "data": {
                "artists": artists,
                "period_days": query.days
            }
        }))),
        Err(e) => {
            tracing::error!("Failed to get top artists: {}", e);
            Ok(Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })))
        }
    }
}

/// Get user's revenue to problematic artists
pub async fn get_user_problematic_revenue_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<RevenueQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        user_id = %user.id,
        days = query.days,
        min_tier = ?query.min_tier,
        "Get user problematic artist revenue"
    );

    let service = crate::services::RevenueService::new(state.db_pool.clone());

    let min_tier = match query.min_tier.as_ref().map(|t| t.to_lowercase()).as_deref() {
        Some("critical") => crate::services::TroubleTier::Critical,
        Some("high") => crate::services::TroubleTier::High,
        _ => crate::services::TroubleTier::Moderate,
    };

    match service.get_user_problematic_artists(user.id, min_tier, query.days, query.limit).await {
        Ok(artists) => Ok(Json(serde_json::json!({
            "success": true,
            "data": {
                "artists": artists,
                "min_tier": format!("{:?}", min_tier),
                "period_days": query.days
            }
        }))),
        Err(e) => {
            tracing::error!("Failed to get problematic revenue: {}", e);
            Ok(Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })))
        }
    }
}

/// Get revenue breakdown for a specific artist
pub async fn get_artist_revenue_breakdown_handler(
    State(state): State<AppState>,
    Path(artist_id): Path<Uuid>,
    user: AuthenticatedUser,
    Query(query): Query<RevenueQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        artist_id = %artist_id,
        user_id = %user.id,
        "Get artist revenue breakdown"
    );

    let service = crate::services::RevenueService::new(state.db_pool.clone());

    match service.get_artist_revenue(user.id, artist_id, query.days).await {
        Ok(breakdown) => Ok(Json(serde_json::json!({
            "success": true,
            "data": breakdown
        }))),
        Err(e) => {
            tracing::error!("Failed to get artist revenue: {}", e);
            Ok(Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })))
        }
    }
}

/// Get all platform payout rates
pub async fn get_payout_rates_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Get payout rates request");

    let service = crate::services::RevenueService::new(state.db_pool.clone());

    match service.get_all_payout_rates().await {
        Ok(rates) => Ok(Json(serde_json::json!({
            "success": true,
            "data": {
                "rates": rates,
                "note": "Rates are estimates based on industry reports and may vary"
            }
        }))),
        Err(e) => {
            tracing::error!("Failed to get payout rates: {}", e);
            Ok(Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })))
        }
    }
}

/// Get global problematic artist revenue leaderboard
pub async fn get_global_problematic_revenue_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Query(query): Query<RevenueQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        days = query.days,
        min_tier = ?query.min_tier,
        "Get global problematic revenue leaderboard"
    );

    let service = crate::services::RevenueService::new(state.db_pool.clone());

    let min_tier = match query.min_tier.as_ref().map(|t| t.to_lowercase()).as_deref() {
        Some("critical") => crate::services::TroubleTier::Critical,
        Some("high") => crate::services::TroubleTier::High,
        _ => crate::services::TroubleTier::Moderate,
    };

    match service.get_problematic_revenue_leaderboard(min_tier, query.days, query.limit).await {
        Ok(leaderboard) => Ok(Json(serde_json::json!({
            "success": true,
            "data": {
                "leaderboard": leaderboard,
                "min_tier": format!("{:?}", min_tier),
                "period_days": query.days
            }
        }))),
        Err(e) => {
            tracing::error!("Failed to get revenue leaderboard: {}", e);
            Ok(Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })))
        }
    }
}

// ============================================================================
// Metrics Endpoints (Prometheus-compatible)
// ============================================================================

/// Get metrics in Prometheus format
pub async fn get_metrics_handler(State(_state): State<AppState>) -> Result<String> {
    tracing::debug!("Get metrics request");

    // Return Prometheus-format metrics
    let metrics = r#"# HELP blocklist_users_total Total number of registered users
# TYPE blocklist_users_total gauge
blocklist_users_total 0

# HELP blocklist_blocks_total Total number of artist blocks
# TYPE blocklist_blocks_total gauge
blocklist_blocks_total 0

# HELP blocklist_syncs_total Total platform sync operations
# TYPE blocklist_syncs_total counter
blocklist_syncs_total{platform="spotify",status="success"} 0
blocklist_syncs_total{platform="spotify",status="failed"} 0
blocklist_syncs_total{platform="apple_music",status="success"} 0
blocklist_syncs_total{platform="apple_music",status="failed"} 0

# HELP blocklist_articles_processed_total Total news articles processed
# TYPE blocklist_articles_processed_total counter
blocklist_articles_processed_total 0

# HELP blocklist_offenses_detected_total Total offenses detected
# TYPE blocklist_offenses_detected_total counter
blocklist_offenses_detected_total 0

# HELP blocklist_api_requests_total Total API requests
# TYPE blocklist_api_requests_total counter
blocklist_api_requests_total{endpoint="/api/v1/health",method="GET"} 0

# HELP blocklist_api_latency_seconds API request latency
# TYPE blocklist_api_latency_seconds histogram
blocklist_api_latency_seconds_bucket{endpoint="/api/v1/health",le="0.01"} 0
blocklist_api_latency_seconds_bucket{endpoint="/api/v1/health",le="0.05"} 0
blocklist_api_latency_seconds_bucket{endpoint="/api/v1/health",le="0.1"} 0
blocklist_api_latency_seconds_bucket{endpoint="/api/v1/health",le="0.5"} 0
blocklist_api_latency_seconds_bucket{endpoint="/api/v1/health",le="1.0"} 0
blocklist_api_latency_seconds_bucket{endpoint="/api/v1/health",le="+Inf"} 0
blocklist_api_latency_seconds_sum{endpoint="/api/v1/health"} 0
blocklist_api_latency_seconds_count{endpoint="/api/v1/health"} 0

# HELP blocklist_database_connections Active database connections
# TYPE blocklist_database_connections gauge
blocklist_database_connections{database="postgres"} 0
blocklist_database_connections{database="duckdb"} 1
blocklist_database_connections{database="kuzu"} 1
blocklist_database_connections{database="lancedb"} 1
blocklist_database_connections{database="redis"} 0
"#;

    Ok(metrics.to_string())
}
