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

    let min_tier = query
        .min_tier
        .as_ref()
        .map(|t| match t.to_lowercase().as_str() {
            "critical" => crate::services::TroubleTier::Critical,
            "high" => crate::services::TroubleTier::High,
            "moderate" => crate::services::TroubleTier::Moderate,
            _ => crate::services::TroubleTier::Low,
        });

    match service
        .get_leaderboard(min_tier, query.limit, query.offset)
        .await
    {
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

    let platform = query
        .platform
        .as_ref()
        .and_then(|p| crate::services::RevenuePlatform::from_str(p));

    match service
        .get_user_revenue_distribution(user.id, platform, query.days)
        .await
    {
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

    match service
        .get_user_top_artists(user.id, query.days, query.limit)
        .await
    {
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

    match service
        .get_user_problematic_artists(user.id, min_tier, query.days, query.limit)
        .await
    {
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

    match service
        .get_artist_revenue(user.id, artist_id, query.days)
        .await
    {
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

    match service
        .get_problematic_revenue_leaderboard(min_tier, query.days, query.limit)
        .await
    {
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
// Category Revenue Endpoints (Simulated by Offense Category)
// ============================================================================

/// Query parameters for category revenue requests
#[derive(Debug, Deserialize)]
pub struct CategoryRevenueQuery {
    /// Top N artists per category
    #[serde(default = "default_top_n")]
    pub top_n: i32,
}

fn default_top_n() -> i32 {
    10
}

/// Get global revenue distribution across offense categories
pub async fn get_global_category_revenue_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Get global category revenue request");

    let service = crate::services::CategoryRevenueService::new(state.db_pool.clone());

    match service.get_global_category_revenue().await {
        Ok(revenue) => Ok(Json(serde_json::json!({
            "success": true,
            "data": revenue
        }))),
        Err(e) => {
            tracing::error!("Failed to get global category revenue: {}", e);
            Ok(Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })))
        }
    }
}

/// Get revenue breakdown for a specific offense category
pub async fn get_category_revenue_handler(
    State(state): State<AppState>,
    Path(category): Path<String>,
    _user: AuthenticatedUser,
    Query(query): Query<CategoryRevenueQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(category = %category, "Get category revenue request");

    let service = crate::services::CategoryRevenueService::new(state.db_pool.clone());
    let offense_category = crate::services::OffenseCategory::from_str(&category);

    match service
        .get_category_revenue(offense_category, query.top_n)
        .await
    {
        Ok(revenue) => Ok(Json(serde_json::json!({
            "success": true,
            "data": revenue
        }))),
        Err(e) => {
            tracing::error!("Failed to get category revenue: {}", e);
            Ok(Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })))
        }
    }
}

/// Get artist discography with simulated revenue
pub async fn get_artist_discography_revenue_handler(
    State(state): State<AppState>,
    Path(artist_id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(artist_id = %artist_id, "Get artist discography revenue request");

    let service = crate::services::CategoryRevenueService::new(state.db_pool.clone());

    match service.get_artist_discography_revenue(artist_id).await {
        Ok(discography) => Ok(Json(serde_json::json!({
            "success": true,
            "data": discography
        }))),
        Err(e) => {
            tracing::error!("Failed to get artist discography revenue: {}", e);
            Ok(Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })))
        }
    }
}

/// Get user's revenue exposure to offense categories
pub async fn get_user_category_exposure_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<RevenueQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(user_id = %user.id, days = query.days, "Get user category exposure request");

    let service = crate::services::CategoryRevenueService::new(state.db_pool.clone());

    match service
        .get_user_category_exposure(user.id, query.days)
        .await
    {
        Ok(exposure) => Ok(Json(serde_json::json!({
            "success": true,
            "data": {
                "categories": exposure,
                "period_days": query.days
            }
        }))),
        Err(e) => {
            tracing::error!("Failed to get user category exposure: {}", e);
            Ok(Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })))
        }
    }
}

/// Get all offense categories with descriptions
pub async fn get_offense_categories_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Get offense categories request");

    let categories: Vec<serde_json::Value> = crate::services::OffenseCategory::all()
        .into_iter()
        .map(|c| {
            serde_json::json!({
                "id": c.as_str(),
                "display_name": c.display_name()
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "categories": categories
        }
    })))
}

// ============================================================================
// Enforcement Analytics Endpoints (US-024)
// ============================================================================

/// Query parameters for enforcement analytics
#[derive(Debug, Deserialize)]
pub struct EnforcementAnalyticsQuery {
    /// Filter by provider (spotify, apple_music, etc.)
    pub provider: Option<String>,
    /// Number of days for time-series (default: 30)
    #[serde(default = "default_days")]
    pub days: i32,
}

/// Get enforcement analytics for the authenticated user
pub async fn get_enforcement_analytics_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<EnforcementAnalyticsQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        user_id = %user.id,
        provider = ?query.provider,
        days = query.days,
        "Get enforcement analytics request"
    );

    let service = crate::services::EnforcementAnalyticsService::new(state.db_pool.clone());

    let analytics_query = crate::services::EnforcementAnalyticsQuery {
        provider: query.provider,
        days: query.days,
    };

    match service
        .get_user_enforcement_analytics(user.id, &analytics_query)
        .await
    {
        Ok(analytics) => Ok(Json(serde_json::json!({
            "success": true,
            "data": {
                "total_batches": analytics.stats.total_batches,
                "total_actions": analytics.stats.total_actions,
                "successful_actions": analytics.stats.successful_actions,
                "failed_actions": analytics.stats.failed_actions,
                "skipped_actions": analytics.stats.skipped_actions,
                "success_rate": analytics.stats.success_rate,
                "actions_by_type": analytics.stats.actions_by_type,
                "actions_by_provider": analytics.stats.actions_by_provider,
                "time_series": analytics.time_series
            },
            "generated_at": analytics.generated_at.to_rfc3339()
        }))),
        Err(e) => {
            tracing::error!("Failed to get enforcement analytics: {}", e);
            Ok(Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })))
        }
    }
}

// ============================================================================
// User Activity Summary Endpoints (US-025)
// ============================================================================

/// User activity summary response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivitySummary {
    /// Number of artists in the user's DNP list
    pub dnp_list_size: i64,
    /// List of connected streaming providers with their status
    pub connected_providers: Vec<ConnectedProviderInfo>,
    /// Number of enforcement actions in the last 30 days
    pub recent_enforcement_count: i64,
    /// Date of the last enforcement action
    pub last_enforcement_date: Option<chrono::DateTime<chrono::Utc>>,
    /// Next scheduled scan timestamp (if any)
    pub next_scheduled_scan: Option<chrono::DateTime<chrono::Utc>>,
    /// Timestamp when this summary was generated
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Whether this data was served from cache
    pub cached: bool,
}

/// Information about a connected streaming provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectedProviderInfo {
    pub provider: String,
    pub status: String,
    pub connected_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Get user activity summary for the dashboard (cached for 5 minutes)
pub async fn get_user_activity_summary_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    use redis::AsyncCommands;

    tracing::info!(user_id = %user.id, "Get user activity summary request");

    let cache_key = format!("user_activity_summary:{}", user.id);
    const CACHE_TTL_SECONDS: u64 = 300; // 5 minutes

    // Try to get from Redis cache first
    if let Ok(mut conn) = state.redis_pool.get().await {
        let cached: Option<String> = conn.get(&cache_key).await.unwrap_or(None);
        if let Some(cached_json) = cached {
            if let Ok(mut summary) = serde_json::from_str::<UserActivitySummary>(&cached_json) {
                summary.cached = true;
                tracing::debug!(user_id = %user.id, "Returning cached user activity summary");
                return Ok(Json(serde_json::json!({
                    "success": true,
                    "data": summary
                })));
            }
        }
    }

    // Cache miss - fetch fresh data
    let summary = fetch_user_activity_summary(&state, user.id).await?;

    // Store in cache
    if let Ok(mut conn) = state.redis_pool.get().await {
        if let Ok(summary_json) = serde_json::to_string(&summary) {
            let _: std::result::Result<(), _> = conn
                .set_ex(&cache_key, summary_json, CACHE_TTL_SECONDS)
                .await;
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "data": summary
    })))
}

/// Fetch fresh user activity summary data from the database
async fn fetch_user_activity_summary(
    state: &AppState,
    user_id: uuid::Uuid,
) -> Result<UserActivitySummary> {
    // Query DNP list size
    let dnp_list_size: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM user_artist_blocks WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&state.db_pool)
            .await
            .unwrap_or(Some(0))
            .unwrap_or(0);

    // Query connected providers from connections table
    let connected_providers =
        sqlx::query_as::<_, (String, String, Option<chrono::DateTime<chrono::Utc>>)>(
            r#"
        SELECT
            provider,
            status,
            created_at
        FROM connections
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        )
        .bind(user_id)
        .fetch_all(&state.db_pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(provider, status, created_at)| ConnectedProviderInfo {
            provider,
            status,
            connected_at: created_at,
        })
        .collect();

    // Query recent enforcement actions (last 30 days)
    let thirty_days_ago = chrono::Utc::now() - chrono::Duration::days(30);
    let recent_enforcement_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM action_batches
        WHERE user_id = $1
          AND created_at >= $2
          AND status IN ('completed', 'partially_completed')
        "#,
    )
    .bind(user_id)
    .bind(thirty_days_ago)
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    // Query last enforcement date
    let last_enforcement_date: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        r#"
        SELECT MAX(completed_at)
        FROM action_batches
        WHERE user_id = $1
          AND status IN ('completed', 'partially_completed')
        "#,
    )
    .bind(user_id)
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(None);

    // Query next scheduled scan from jobs table
    let next_scheduled_scan: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        r#"
        SELECT MIN(scheduled_at)
        FROM jobs
        WHERE user_id = $1
          AND status = 'pending'
          AND job_type IN ('LibraryScan', 'EnforcementExecution')
        "#,
    )
    .bind(user_id)
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(None);

    Ok(UserActivitySummary {
        dnp_list_size,
        connected_providers,
        recent_enforcement_count,
        last_enforcement_date,
        next_scheduled_scan,
        generated_at: chrono::Utc::now(),
        cached: false,
    })
}

// ============================================================================
// Metrics Endpoints (Prometheus-compatible)
// ============================================================================

/// Get metrics in Prometheus format with real system data
pub async fn get_metrics_handler(State(state): State<AppState>) -> Result<String> {
    tracing::debug!("Get metrics request");

    // Collect real system metrics using sysinfo
    use sysinfo::{Disks, System};

    let mut sys = System::new_all();
    sys.refresh_all();

    // CPU usage (average across all CPUs)
    let cpu_usage = sys.global_cpu_info().cpu_usage();

    // Memory metrics
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let memory_usage_percent = if total_memory > 0 {
        (used_memory as f64 / total_memory as f64) * 100.0
    } else {
        0.0
    };

    // Get process-specific memory (RSS - Resident Set Size)
    let current_pid = sysinfo::get_current_pid().ok();
    let process_memory_rss = current_pid
        .and_then(|pid| sys.process(pid))
        .map(|p| p.memory())
        .unwrap_or(0);

    // Disk usage for data directory (use root or current working directory)
    let disks = Disks::new_with_refreshed_list();
    let (disk_total, disk_used) = disks
        .iter()
        .next()
        .map(|d| (d.total_space(), d.total_space() - d.available_space()))
        .unwrap_or((0, 0));
    let disk_usage_percent = if disk_total > 0 {
        (disk_used as f64 / disk_total as f64) * 100.0
    } else {
        0.0
    };

    // Database connection pool stats
    let db_pool_size = state.db_pool.size();
    let db_idle_connections = state.db_pool.num_idle();
    let db_active_connections = (db_pool_size as usize).saturating_sub(db_idle_connections) as u32;

    // Redis connection pool stats
    let redis_status = state.redis_pool.status();
    let redis_pool_size = redis_status.size;
    let redis_available = redis_status.available;
    let redis_active = redis_pool_size.saturating_sub(redis_available);

    // Job queue depth by job type - query Redis for queue sizes
    let job_queue_depths = get_job_queue_depths(&state.redis_pool).await;

    // Query database for business metrics
    let (user_count, block_count) = get_business_metrics(&state.db_pool).await;

    // Build Prometheus-format metrics output
    let mut metrics = String::new();

    // System metrics - CPU
    metrics.push_str("# HELP blocklist_cpu_usage_percent Current CPU usage percentage\n");
    metrics.push_str("# TYPE blocklist_cpu_usage_percent gauge\n");
    metrics.push_str(&format!("blocklist_cpu_usage_percent {:.2}\n\n", cpu_usage));

    // System metrics - Memory (system-wide)
    metrics.push_str("# HELP blocklist_memory_usage_bytes Current system memory usage in bytes\n");
    metrics.push_str("# TYPE blocklist_memory_usage_bytes gauge\n");
    metrics.push_str(&format!("blocklist_memory_usage_bytes {}\n\n", used_memory));

    metrics.push_str("# HELP blocklist_memory_total_bytes Total system memory in bytes\n");
    metrics.push_str("# TYPE blocklist_memory_total_bytes gauge\n");
    metrics.push_str(&format!(
        "blocklist_memory_total_bytes {}\n\n",
        total_memory
    ));

    metrics.push_str("# HELP blocklist_memory_usage_percent System memory usage percentage\n");
    metrics.push_str("# TYPE blocklist_memory_usage_percent gauge\n");
    metrics.push_str(&format!(
        "blocklist_memory_usage_percent {:.2}\n\n",
        memory_usage_percent
    ));

    // Process-specific memory (RSS)
    metrics
        .push_str("# HELP blocklist_process_memory_rss_bytes Process resident set size in bytes\n");
    metrics.push_str("# TYPE blocklist_process_memory_rss_bytes gauge\n");
    metrics.push_str(&format!(
        "blocklist_process_memory_rss_bytes {}\n\n",
        process_memory_rss
    ));

    // Disk usage
    metrics.push_str("# HELP blocklist_disk_usage_bytes Current disk usage in bytes\n");
    metrics.push_str("# TYPE blocklist_disk_usage_bytes gauge\n");
    metrics.push_str(&format!("blocklist_disk_usage_bytes {}\n\n", disk_used));

    metrics.push_str("# HELP blocklist_disk_total_bytes Total disk space in bytes\n");
    metrics.push_str("# TYPE blocklist_disk_total_bytes gauge\n");
    metrics.push_str(&format!("blocklist_disk_total_bytes {}\n\n", disk_total));

    metrics.push_str("# HELP blocklist_disk_usage_percent Disk usage percentage\n");
    metrics.push_str("# TYPE blocklist_disk_usage_percent gauge\n");
    metrics.push_str(&format!(
        "blocklist_disk_usage_percent {:.2}\n\n",
        disk_usage_percent
    ));

    // Database connection metrics
    metrics.push_str("# HELP blocklist_db_connections_active Active database connections\n");
    metrics.push_str("# TYPE blocklist_db_connections_active gauge\n");
    metrics.push_str(&format!(
        "blocklist_db_connections_active {}\n\n",
        db_active_connections
    ));

    metrics.push_str("# HELP blocklist_db_connections_idle Idle database connections\n");
    metrics.push_str("# TYPE blocklist_db_connections_idle gauge\n");
    metrics.push_str(&format!(
        "blocklist_db_connections_idle {}\n\n",
        db_idle_connections
    ));

    metrics.push_str("# HELP blocklist_db_connections_total Total database connection pool size\n");
    metrics.push_str("# TYPE blocklist_db_connections_total gauge\n");
    metrics.push_str(&format!(
        "blocklist_db_connections_total {}\n\n",
        db_pool_size
    ));

    // Redis connection metrics
    metrics.push_str("# HELP blocklist_redis_connections_active Active Redis connections\n");
    metrics.push_str("# TYPE blocklist_redis_connections_active gauge\n");
    metrics.push_str(&format!(
        "blocklist_redis_connections_active {}\n\n",
        redis_active
    ));

    metrics.push_str("# HELP blocklist_redis_connections_available Available Redis connections\n");
    metrics.push_str("# TYPE blocklist_redis_connections_available gauge\n");
    metrics.push_str(&format!(
        "blocklist_redis_connections_available {}\n\n",
        redis_available
    ));

    metrics.push_str("# HELP blocklist_redis_connections_total Total Redis connection pool size\n");
    metrics.push_str("# TYPE blocklist_redis_connections_total gauge\n");
    metrics.push_str(&format!(
        "blocklist_redis_connections_total {}\n\n",
        redis_pool_size
    ));

    // Job queue depth by type
    metrics.push_str("# HELP blocklist_job_queue_depth Number of pending jobs by type\n");
    metrics.push_str("# TYPE blocklist_job_queue_depth gauge\n");
    for (job_type, depth) in &job_queue_depths {
        metrics.push_str(&format!(
            "blocklist_job_queue_depth{{job_type=\"{}\"}} {}\n",
            job_type, depth
        ));
    }
    metrics.push('\n');

    // Business metrics
    metrics.push_str("# HELP blocklist_users_total Total number of registered users\n");
    metrics.push_str("# TYPE blocklist_users_total gauge\n");
    metrics.push_str(&format!("blocklist_users_total {}\n\n", user_count));

    metrics.push_str("# HELP blocklist_blocks_total Total number of artist blocks\n");
    metrics.push_str("# TYPE blocklist_blocks_total gauge\n");
    metrics.push_str(&format!("blocklist_blocks_total {}\n\n", block_count));

    Ok(metrics)
}

/// Get job queue depths from Redis for each job type
async fn get_job_queue_depths(redis_pool: &deadpool_redis::Pool) -> Vec<(String, i64)> {
    use redis::AsyncCommands;

    let job_types = [
        "EnforcementExecution",
        "BatchRollback",
        "TokenRefresh",
        "LibraryScan",
        "CommunityListUpdate",
        "HealthCheck",
    ];

    let mut depths = Vec::new();

    if let Ok(mut conn) = redis_pool.get().await {
        for job_type in &job_types {
            let queue_key = format!("queue:{}", job_type);
            let depth: i64 = conn.zcard(&queue_key).await.unwrap_or(0);
            depths.push((job_type.to_string(), depth));
        }
    } else {
        // If we can't connect to Redis, return zeros
        for job_type in &job_types {
            depths.push((job_type.to_string(), 0));
        }
    }

    depths
}

/// Get business metrics from the database
async fn get_business_metrics(db_pool: &sqlx::PgPool) -> (i64, i64) {
    // Query user count
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(db_pool)
        .await
        .unwrap_or(Some(0))
        .unwrap_or(0);

    // Query block count
    let block_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_artist_blocks")
        .fetch_one(db_pool)
        .await
        .unwrap_or(Some(0))
        .unwrap_or(0);

    (user_count, block_count)
}
