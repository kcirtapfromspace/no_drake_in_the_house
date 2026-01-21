use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::User;
use crate::services::analytics::AnalyticsService;

/// Query parameters for enforcement success report
#[derive(Deserialize)]
pub struct EnforcementReportQuery {
    pub days_back: Option<u32>,
}

/// Query parameters for community list impact report
#[derive(Deserialize)]
pub struct CommunityListReportQuery {
    pub include_growth: Option<bool>,
    pub include_detailed_metrics: Option<bool>,
}

/// Response wrapper for analytics data
#[derive(Serialize)]
pub struct AnalyticsResponse<T> {
    pub success: bool,
    pub data: T,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// Get enforcement success report for the authenticated user
pub async fn get_enforcement_success_report(
    State(analytics_service): State<Arc<AnalyticsService>>,
    Extension(user): Extension<User>,
    Query(params): Query<EnforcementReportQuery>,
) -> Result<Json<AnalyticsResponse<crate::services::analytics::EnforcementSuccessReport>>, StatusCode> {
    let days_back = params.days_back.unwrap_or(30);
    
    match analytics_service
        .generate_enforcement_success_report(user.id, days_back)
        .await
    {
        Ok(report) => Ok(Json(AnalyticsResponse {
            success: true,
            data: report,
            generated_at: chrono::Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to generate enforcement success report: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get DNP list effectiveness report for the authenticated user
pub async fn get_dnp_effectiveness_report(
    State(analytics_service): State<Arc<AnalyticsService>>,
    Extension(user): Extension<User>,
) -> Result<Json<AnalyticsResponse<crate::services::analytics::DnpListEffectivenessReport>>, StatusCode> {
    match analytics_service
        .generate_dnp_effectiveness_report(user.id)
        .await
    {
        Ok(report) => Ok(Json(AnalyticsResponse {
            success: true,
            data: report,
            generated_at: chrono::Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to generate DNP effectiveness report: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get community list impact report (for list owners or public lists)
pub async fn get_community_list_impact_report(
    State(analytics_service): State<Arc<AnalyticsService>>,
    Path(list_id): Path<Uuid>,
    Extension(_user): Extension<User>,
    Query(_params): Query<CommunityListReportQuery>,
) -> Result<Json<AnalyticsResponse<crate::services::analytics::CommunityListImpactReport>>, StatusCode> {
    // TODO: Add authorization check - only list owners or public lists
    
    match analytics_service
        .generate_community_list_impact_report(list_id)
        .await
    {
        Ok(report) => Ok(Json(AnalyticsResponse {
            success: true,
            data: report,
            generated_at: chrono::Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to generate community list impact report: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get system performance dashboard (admin only)
pub async fn get_system_performance_dashboard(
    State(analytics_service): State<Arc<AnalyticsService>>,
    Extension(user): Extension<User>,
) -> Result<Json<AnalyticsResponse<crate::services::analytics::SystemPerformanceDashboard>>, StatusCode> {
    // TODO: Add admin authorization check
    if !user.email.ends_with("@admin.com") {
        return Err(StatusCode::FORBIDDEN);
    }
    
    match analytics_service
        .generate_system_performance_dashboard()
        .await
    {
        Ok(dashboard) => Ok(Json(AnalyticsResponse {
            success: true,
            data: dashboard,
            generated_at: chrono::Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to generate system performance dashboard: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user analytics summary
pub async fn get_user_analytics_summary(
    State(analytics_service): State<Arc<AnalyticsService>>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Generate a combined summary of user analytics
    let enforcement_report = analytics_service
        .generate_enforcement_success_report(user.id, 30)
        .await;
    
    let dnp_report = analytics_service
        .generate_dnp_effectiveness_report(user.id)
        .await;
    
    match (enforcement_report, dnp_report) {
        (Ok(enforcement), Ok(dnp)) => {
            let summary = serde_json::json!({
                "user_id": user.id,
                "generated_at": chrono::Utc::now(),
                "enforcement_summary": {
                    "total_operations_last_30_days": enforcement.total_operations,
                    "success_rate": enforcement.success_rate,
                    "most_active_provider": enforcement.provider_breakdown
                        .iter()
                        .max_by_key(|(_, stats)| stats.total_operations)
                        .map(|(provider, _)| provider)
                        .unwrap_or(&"none".to_string()),
                },
                "dnp_summary": {
                    "total_artists_blocked": dnp.total_artists_blocked,
                    "total_content_filtered": dnp.total_content_filtered,
                    "effectiveness_score": dnp.filter_effectiveness_score,
                    "top_blocked_artist": dnp.top_blocked_artists
                        .first()
                        .map(|artist| &artist.artist_name)
                        .unwrap_or(&"none".to_string()),
                },
                "recommendations": dnp.recommendations,
            });
            
            Ok(Json(summary))
        }
        _ => {
            tracing::error!("Failed to generate user analytics summary");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get analytics for a specific time period
#[derive(Deserialize)]
pub struct TimeRangeQuery {
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub granularity: Option<String>, // "day", "week", "month"
}

pub async fn get_analytics_time_series(
    State(_analytics_service): State<Arc<AnalyticsService>>,
    Extension(user): Extension<User>,
    Query(params): Query<TimeRangeQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let start_date = params.start_date.unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30));
    let end_date = params.end_date.unwrap_or_else(chrono::Utc::now);
    let granularity = params.granularity.unwrap_or_else(|| "day".to_string());
    
    // TODO: Implement time series analytics
    let time_series = serde_json::json!({
        "user_id": user.id,
        "period": {
            "start": start_date,
            "end": end_date,
            "granularity": granularity
        },
        "metrics": {
            "enforcement_operations": [],
            "content_filtered": [],
            "success_rates": []
        },
        "note": "Time series analytics implementation pending"
    });
    
    Ok(Json(time_series))
}

/// Get comparative analytics (compare with other users anonymously)
pub async fn get_comparative_analytics(
    State(_analytics_service): State<Arc<AnalyticsService>>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Implement comparative analytics with privacy protection
    let comparative = serde_json::json!({
        "user_id": user.id,
        "generated_at": chrono::Utc::now(),
        "comparisons": {
            "dnp_list_size": {
                "user_value": 0,
                "percentile": 0,
                "average": 0
            },
            "enforcement_frequency": {
                "user_value": 0,
                "percentile": 0,
                "average": 0
            },
            "success_rate": {
                "user_value": 0,
                "percentile": 0,
                "average": 0
            }
        },
        "note": "Comparative analytics implementation pending"
    });
    
    Ok(Json(comparative))
}

/// Export analytics data
#[derive(Deserialize)]
pub struct ExportQuery {
    pub format: Option<String>, // "json", "csv"
    pub include_raw_data: Option<bool>,
}

pub async fn export_analytics_data(
    State(analytics_service): State<Arc<AnalyticsService>>,
    Extension(user): Extension<User>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let format = params.format.unwrap_or_else(|| "json".to_string());
    let include_raw_data = params.include_raw_data.unwrap_or(false);
    
    match analytics_service
        .generate_enforcement_success_report(user.id, 90) // Last 90 days
        .await
    {
        Ok(enforcement_report) => {
            match analytics_service
                .generate_dnp_effectiveness_report(user.id)
                .await
            {
                Ok(dnp_report) => {
                    let export_data = serde_json::json!({
                        "export_info": {
                            "user_id": user.id,
                            "generated_at": chrono::Utc::now(),
                            "format": format,
                            "include_raw_data": include_raw_data,
                            "period": "last_90_days"
                        },
                        "enforcement_data": enforcement_report,
                        "dnp_effectiveness_data": dnp_report,
                        "download_url": format!("/api/v1/analytics/export/{}/download", user.id),
                        "expires_at": chrono::Utc::now() + chrono::Duration::hours(24)
                    });
                    
                    Ok(Json(export_data))
                }
                Err(e) => {
                    tracing::error!("Failed to generate DNP report for export: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to generate enforcement report for export: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}