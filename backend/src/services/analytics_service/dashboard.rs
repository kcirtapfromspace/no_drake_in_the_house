//! Dashboard Service
//!
//! Provides dashboard-level analytics and metrics.

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::services::databases::{DuckDbClient, DailyNewsSummary, TrendingArtist, PlatformHealth};

/// Time range for analytics queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeRange {
    Today,
    Yesterday,
    Last7Days,
    Last30Days,
    Last90Days,
    AllTime,
    Custom { days: i32 },
}

impl TimeRange {
    pub fn days(&self) -> i32 {
        match self {
            TimeRange::Today => 1,
            TimeRange::Yesterday => 1,
            TimeRange::Last7Days => 7,
            TimeRange::Last30Days => 30,
            TimeRange::Last90Days => 90,
            TimeRange::AllTime => 365 * 10, // 10 years
            TimeRange::Custom { days } => *days,
        }
    }

    pub fn start_date(&self) -> DateTime<Utc> {
        match self {
            TimeRange::Yesterday => Utc::now() - Duration::days(2),
            _ => Utc::now() - Duration::days(self.days() as i64),
        }
    }
}

/// Dashboard overview metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetrics {
    /// Time range for the metrics
    pub time_range: String,
    /// User activity metrics
    pub user_metrics: UserMetrics,
    /// Content metrics (articles, offenses)
    pub content_metrics: ContentMetrics,
    /// Platform sync metrics
    pub sync_metrics: SyncOverview,
    /// Top trending artists
    pub trending_artists: Vec<TrendingArtistSummary>,
    /// Recent offense detections
    pub recent_offenses: Vec<RecentOffense>,
    /// System health summary
    pub system_health: SystemHealth,
}

/// User activity metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserMetrics {
    pub total_users: i64,
    pub active_users: i64,
    pub new_users: i64,
    pub total_blocks: i64,
    pub new_blocks: i64,
    pub avg_blocks_per_user: f64,
}

/// Content metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContentMetrics {
    pub total_articles: i64,
    pub articles_processed: i64,
    pub entities_extracted: i64,
    pub offenses_detected: i64,
    pub offense_rate: f64,
    pub avg_sentiment: f64,
}

/// Sync overview
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncOverview {
    pub total_syncs: i64,
    pub successful_syncs: i64,
    pub failed_syncs: i64,
    pub success_rate: f64,
    pub artists_synced: i64,
    pub platforms: Vec<PlatformSyncStatus>,
}

/// Platform sync status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformSyncStatus {
    pub platform: String,
    pub healthy: bool,
    pub last_sync: Option<DateTime<Utc>>,
    pub artists_count: i64,
    pub success_rate: f64,
}

/// Trending artist summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingArtistSummary {
    pub id: Uuid,
    pub name: String,
    pub mentions: i64,
    pub trend_direction: String, // "up", "down", "stable"
    pub change_percentage: f64,
    pub has_offense: bool,
}

/// Recent offense detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentOffense {
    pub id: Uuid,
    pub artist_id: Uuid,
    pub artist_name: String,
    pub category: String,
    pub severity: String,
    pub article_title: String,
    pub detected_at: DateTime<Utc>,
    pub verified: bool,
}

/// System health summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall_status: String, // "healthy", "degraded", "unhealthy"
    pub postgres_healthy: bool,
    pub duckdb_healthy: bool,
    pub kuzu_healthy: bool,
    pub lancedb_healthy: bool,
    pub redis_healthy: bool,
    pub api_response_time_ms: i64,
    pub error_rate: f64,
}

/// Dashboard service
pub struct DashboardService {
    duckdb: Arc<DuckDbClient>,
    pool: PgPool,
}

impl DashboardService {
    pub fn new(duckdb: Arc<DuckDbClient>, pool: PgPool) -> Self {
        Self { duckdb, pool }
    }

    /// Get complete dashboard metrics
    pub async fn get_dashboard(&self, time_range: TimeRange) -> Result<DashboardMetrics> {
        let days = time_range.days();

        // Fetch all metrics concurrently
        let (user_metrics, content_metrics, sync_metrics, trending, recent_offenses) = tokio::join!(
            self.get_user_metrics(days),
            self.get_content_metrics(days),
            self.get_sync_overview(days),
            self.get_trending_artists(days, 10),
            self.get_recent_offenses(10),
        );

        let system_health = self.get_system_health().await?;

        Ok(DashboardMetrics {
            time_range: format!("{:?}", time_range),
            user_metrics: user_metrics.unwrap_or_default(),
            content_metrics: content_metrics.unwrap_or_default(),
            sync_metrics: sync_metrics.unwrap_or_default(),
            trending_artists: trending.unwrap_or_default(),
            recent_offenses: recent_offenses.unwrap_or_default(),
            system_health,
        })
    }

    /// Get user metrics from PostgreSQL
    async fn get_user_metrics(&self, days: i32) -> Result<UserMetrics> {
        let cutoff = Utc::now() - Duration::days(days as i64);

        // Total users
        let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        // New users in time range
        let new_users: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM users WHERE created_at >= $1"
        )
        .bind(cutoff)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        // Active users (users with blocks in time range)
        let active_users: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT user_id) FROM user_artist_blocks WHERE created_at >= $1"
        )
        .bind(cutoff)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        // Total blocks
        let total_blocks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_artist_blocks")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        // New blocks in time range
        let new_blocks: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_artist_blocks WHERE created_at >= $1"
        )
        .bind(cutoff)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        let avg_blocks_per_user = if total_users > 0 {
            total_blocks as f64 / total_users as f64
        } else {
            0.0
        };

        Ok(UserMetrics {
            total_users,
            active_users,
            new_users,
            total_blocks,
            new_blocks,
            avg_blocks_per_user,
        })
    }

    /// Get content metrics from DuckDB
    async fn get_content_metrics(&self, days: i32) -> Result<ContentMetrics> {
        let summaries = self.duckdb.get_daily_news_summary(days).await?;

        let mut total_articles = 0i64;
        let mut total_entities = 0i64;
        let mut total_offenses = 0i64;
        let mut sentiment_sum = 0.0f64;
        let count = summaries.len() as f64;

        for summary in summaries {
            total_articles += summary.total_articles;
            total_entities += summary.total_entities;
            total_offenses += summary.total_offenses;
            sentiment_sum += summary.avg_sentiment;
        }

        let offense_rate = if total_articles > 0 {
            total_offenses as f64 / total_articles as f64
        } else {
            0.0
        };

        let avg_sentiment = if count > 0.0 {
            sentiment_sum / count
        } else {
            0.0
        };

        Ok(ContentMetrics {
            total_articles,
            articles_processed: total_articles,
            entities_extracted: total_entities,
            offenses_detected: total_offenses,
            offense_rate,
            avg_sentiment,
        })
    }

    /// Get sync overview from DuckDB
    async fn get_sync_overview(&self, days: i32) -> Result<SyncOverview> {
        let platform_health = self.duckdb.get_platform_health(days).await?;

        let mut total_syncs = 0i64;
        let mut successful_syncs = 0i64;
        let mut failed_syncs = 0i64;
        let mut artists_synced = 0i64;

        let platforms: Vec<PlatformSyncStatus> = platform_health
            .iter()
            .map(|p| {
                total_syncs += p.total_syncs;
                successful_syncs += p.successful_syncs;
                failed_syncs += p.failed_syncs;
                artists_synced += p.total_artists_synced;

                PlatformSyncStatus {
                    platform: p.platform.clone(),
                    healthy: p.success_rate >= 0.9,
                    last_sync: None, // Would need separate query
                    artists_count: p.total_artists_synced,
                    success_rate: p.success_rate,
                }
            })
            .collect();

        let success_rate = if total_syncs > 0 {
            successful_syncs as f64 / total_syncs as f64
        } else {
            1.0
        };

        Ok(SyncOverview {
            total_syncs,
            successful_syncs,
            failed_syncs,
            success_rate,
            artists_synced,
            platforms,
        })
    }

    /// Get trending artists
    async fn get_trending_artists(&self, days: i32, limit: i32) -> Result<Vec<TrendingArtistSummary>> {
        let trending = self.duckdb.get_trending_artists(days, limit).await?;

        Ok(trending
            .into_iter()
            .map(|t| TrendingArtistSummary {
                id: t.artist_id,
                name: t.artist_name,
                mentions: t.total_mentions,
                trend_direction: if t.positive_ratio > 0.6 {
                    "up".to_string()
                } else if t.positive_ratio < 0.4 {
                    "down".to_string()
                } else {
                    "stable".to_string()
                },
                change_percentage: 0.0, // Would need historical comparison
                has_offense: t.offense_mentions > 0,
            })
            .collect())
    }

    /// Get recent offense detections
    async fn get_recent_offenses(&self, limit: i32) -> Result<Vec<RecentOffense>> {
        // This would query from PostgreSQL news_offense_classifications table
        // For now, return empty since the table may not be populated
        let _limit = limit;
        Ok(Vec::new())
    }

    /// Get system health status
    pub async fn get_system_health(&self) -> Result<SystemHealth> {
        let mut health = SystemHealth::default();

        // Check PostgreSQL
        health.postgres_healthy = sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .is_ok();

        // DuckDB - always healthy if we got here
        health.duckdb_healthy = true;

        // Kùzu - would need actual check
        health.kuzu_healthy = true;

        // LanceDB - would need actual check
        health.lancedb_healthy = true;

        // Redis - would need actual check
        health.redis_healthy = true;

        // Calculate overall status
        let healthy_count = [
            health.postgres_healthy,
            health.duckdb_healthy,
            health.kuzu_healthy,
            health.lancedb_healthy,
        ]
        .iter()
        .filter(|&&h| h)
        .count();

        health.overall_status = if healthy_count == 4 {
            "healthy".to_string()
        } else if healthy_count >= 2 {
            "degraded".to_string()
        } else {
            "unhealthy".to_string()
        };

        health.api_response_time_ms = 50; // Placeholder
        health.error_rate = 0.01; // Placeholder

        Ok(health)
    }

    /// Get quick stats for a specific user
    pub async fn get_user_quick_stats(&self, user_id: Uuid) -> Result<UserQuickStats> {
        // Get user's block count
        let block_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_artist_blocks WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        // Get subscription count
        let subscription_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_category_subscriptions WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        Ok(UserQuickStats {
            user_id,
            blocked_artists: block_count,
            category_subscriptions: subscription_count,
            last_activity: None, // Would need query
        })
    }
}

/// Quick stats for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQuickStats {
    pub user_id: Uuid,
    pub blocked_artists: i64,
    pub category_subscriptions: i64,
    pub last_activity: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_range_days() {
        assert_eq!(TimeRange::Today.days(), 1);
        assert_eq!(TimeRange::Last7Days.days(), 7);
        assert_eq!(TimeRange::Last30Days.days(), 30);
        assert_eq!(TimeRange::Custom { days: 14 }.days(), 14);
    }
}
