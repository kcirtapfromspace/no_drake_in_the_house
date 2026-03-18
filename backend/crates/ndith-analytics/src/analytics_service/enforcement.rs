//! Enforcement Analytics Service
//!
//! Provides analytics for enforcement history including:
//! - Total batches and actions
//! - Success rates
//! - Actions by type
//! - Time-series data for the last 30 days
//! - Provider-specific filtering

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Enforcement analytics service
pub struct EnforcementAnalyticsService {
    pool: PgPool,
}

/// Overall enforcement statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementStats {
    /// Total number of enforcement batches
    pub total_batches: i64,
    /// Total number of individual actions
    pub total_actions: i64,
    /// Number of successful actions
    pub successful_actions: i64,
    /// Number of failed actions
    pub failed_actions: i64,
    /// Number of skipped actions
    pub skipped_actions: i64,
    /// Overall success rate (0.0 - 1.0)
    pub success_rate: f64,
    /// Breakdown by action type
    pub actions_by_type: Vec<ActionTypeCount>,
    /// Breakdown by provider
    pub actions_by_provider: Vec<ProviderStats>,
}

/// Count of actions by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionTypeCount {
    pub action_type: String,
    pub count: i64,
}

/// Provider-specific statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStats {
    pub provider: String,
    pub total_batches: i64,
    pub total_actions: i64,
    pub success_rate: f64,
}

/// Time-series data point for enforcement activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementTimeSeriesPoint {
    pub date: NaiveDate,
    pub batches_count: i64,
    pub actions_count: i64,
    pub successful_count: i64,
    pub failed_count: i64,
}

/// Full enforcement analytics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementAnalytics {
    pub stats: EnforcementStats,
    pub time_series: Vec<EnforcementTimeSeriesPoint>,
    pub generated_at: DateTime<Utc>,
}

/// Query parameters for enforcement analytics
#[derive(Debug, Clone, Deserialize)]
pub struct EnforcementAnalyticsQuery {
    /// Filter by provider (spotify, apple_music, etc.)
    pub provider: Option<String>,
    /// Number of days for time-series (default: 30)
    #[serde(default = "default_days")]
    pub days: i32,
}

fn default_days() -> i32 {
    30
}

impl EnforcementAnalyticsService {
    /// Create a new enforcement analytics service
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get enforcement analytics for a user
    pub async fn get_user_enforcement_analytics(
        &self,
        user_id: Uuid,
        query: &EnforcementAnalyticsQuery,
    ) -> Result<EnforcementAnalytics, sqlx::Error> {
        let stats = self
            .get_user_stats(user_id, query.provider.as_deref())
            .await?;
        let time_series = self
            .get_user_time_series(user_id, query.provider.as_deref(), query.days)
            .await?;

        Ok(EnforcementAnalytics {
            stats,
            time_series,
            generated_at: Utc::now(),
        })
    }

    /// Get overall enforcement stats for a user
    async fn get_user_stats(
        &self,
        user_id: Uuid,
        provider: Option<&str>,
    ) -> Result<EnforcementStats, sqlx::Error> {
        // Get batch counts
        let batch_stats = if let Some(provider) = provider {
            sqlx::query_as::<_, (i64,)>(
                r#"
                SELECT COUNT(*) as total_batches
                FROM action_batches
                WHERE user_id = $1 AND provider = $2
                "#,
            )
            .bind(user_id)
            .bind(provider)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, (i64,)>(
                r#"
                SELECT COUNT(*) as total_batches
                FROM action_batches
                WHERE user_id = $1
                "#,
            )
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?
        };

        let total_batches = batch_stats.0;

        // Get action counts by status
        let action_stats = if let Some(provider) = provider {
            sqlx::query_as::<_, (i64, i64, i64, i64)>(
                r#"
                SELECT
                    COUNT(*) as total_actions,
                    COUNT(*) FILTER (WHERE ai.status = 'completed') as successful,
                    COUNT(*) FILTER (WHERE ai.status = 'failed') as failed,
                    COUNT(*) FILTER (WHERE ai.status = 'skipped') as skipped
                FROM action_items ai
                JOIN action_batches ab ON ai.batch_id = ab.id
                WHERE ab.user_id = $1 AND ab.provider = $2
                "#,
            )
            .bind(user_id)
            .bind(provider)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, (i64, i64, i64, i64)>(
                r#"
                SELECT
                    COUNT(*) as total_actions,
                    COUNT(*) FILTER (WHERE ai.status = 'completed') as successful,
                    COUNT(*) FILTER (WHERE ai.status = 'failed') as failed,
                    COUNT(*) FILTER (WHERE ai.status = 'skipped') as skipped
                FROM action_items ai
                JOIN action_batches ab ON ai.batch_id = ab.id
                WHERE ab.user_id = $1
                "#,
            )
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?
        };

        let (total_actions, successful_actions, failed_actions, skipped_actions) = action_stats;

        let success_rate = if total_actions > 0 {
            successful_actions as f64 / total_actions as f64
        } else {
            1.0 // No actions = 100% success (nothing failed)
        };

        // Get actions by type
        let actions_by_type = self.get_actions_by_type(user_id, provider).await?;

        // Get actions by provider
        let actions_by_provider = self.get_actions_by_provider(user_id).await?;

        Ok(EnforcementStats {
            total_batches,
            total_actions,
            successful_actions,
            failed_actions,
            skipped_actions,
            success_rate,
            actions_by_type,
            actions_by_provider,
        })
    }

    /// Get action counts by type
    async fn get_actions_by_type(
        &self,
        user_id: Uuid,
        provider: Option<&str>,
    ) -> Result<Vec<ActionTypeCount>, sqlx::Error> {
        let rows = if let Some(provider) = provider {
            sqlx::query_as::<_, (String, i64)>(
                r#"
                SELECT ai.action as action_type, COUNT(*) as count
                FROM action_items ai
                JOIN action_batches ab ON ai.batch_id = ab.id
                WHERE ab.user_id = $1 AND ab.provider = $2
                GROUP BY ai.action
                ORDER BY count DESC
                "#,
            )
            .bind(user_id)
            .bind(provider)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, (String, i64)>(
                r#"
                SELECT ai.action as action_type, COUNT(*) as count
                FROM action_items ai
                JOIN action_batches ab ON ai.batch_id = ab.id
                WHERE ab.user_id = $1
                GROUP BY ai.action
                ORDER BY count DESC
                "#,
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows
            .into_iter()
            .map(|(action_type, count)| ActionTypeCount { action_type, count })
            .collect())
    }

    /// Get statistics by provider
    async fn get_actions_by_provider(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<ProviderStats>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (String, i64, i64, i64)>(
            r#"
            SELECT
                ab.provider,
                COUNT(DISTINCT ab.id) as total_batches,
                COUNT(ai.id) as total_actions,
                COUNT(*) FILTER (WHERE ai.status = 'completed') as successful
            FROM action_batches ab
            LEFT JOIN action_items ai ON ai.batch_id = ab.id
            WHERE ab.user_id = $1
            GROUP BY ab.provider
            ORDER BY total_actions DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(provider, total_batches, total_actions, successful)| {
                let success_rate = if total_actions > 0 {
                    successful as f64 / total_actions as f64
                } else {
                    1.0
                };
                ProviderStats {
                    provider,
                    total_batches,
                    total_actions,
                    success_rate,
                }
            })
            .collect())
    }

    /// Get time-series data for enforcement activity
    async fn get_user_time_series(
        &self,
        user_id: Uuid,
        provider: Option<&str>,
        days: i32,
    ) -> Result<Vec<EnforcementTimeSeriesPoint>, sqlx::Error> {
        let rows = if let Some(provider) = provider {
            sqlx::query_as::<_, (NaiveDate, i64, i64, i64, i64)>(
                r#"
                WITH date_series AS (
                    SELECT generate_series(
                        CURRENT_DATE - $3::integer,
                        CURRENT_DATE,
                        '1 day'::interval
                    )::date AS date
                )
                SELECT
                    ds.date,
                    COUNT(DISTINCT ab.id) as batches_count,
                    COUNT(ai.id) as actions_count,
                    COUNT(*) FILTER (WHERE ai.status = 'completed') as successful_count,
                    COUNT(*) FILTER (WHERE ai.status = 'failed') as failed_count
                FROM date_series ds
                LEFT JOIN action_batches ab ON
                    DATE(ab.created_at) = ds.date
                    AND ab.user_id = $1
                    AND ab.provider = $2
                LEFT JOIN action_items ai ON ai.batch_id = ab.id
                GROUP BY ds.date
                ORDER BY ds.date
                "#,
            )
            .bind(user_id)
            .bind(provider)
            .bind(days)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, (NaiveDate, i64, i64, i64, i64)>(
                r#"
                WITH date_series AS (
                    SELECT generate_series(
                        CURRENT_DATE - $2::integer,
                        CURRENT_DATE,
                        '1 day'::interval
                    )::date AS date
                )
                SELECT
                    ds.date,
                    COUNT(DISTINCT ab.id) as batches_count,
                    COUNT(ai.id) as actions_count,
                    COUNT(*) FILTER (WHERE ai.status = 'completed') as successful_count,
                    COUNT(*) FILTER (WHERE ai.status = 'failed') as failed_count
                FROM date_series ds
                LEFT JOIN action_batches ab ON
                    DATE(ab.created_at) = ds.date
                    AND ab.user_id = $1
                LEFT JOIN action_items ai ON ai.batch_id = ab.id
                GROUP BY ds.date
                ORDER BY ds.date
                "#,
            )
            .bind(user_id)
            .bind(days)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows
            .into_iter()
            .map(
                |(date, batches_count, actions_count, successful_count, failed_count)| {
                    EnforcementTimeSeriesPoint {
                        date,
                        batches_count,
                        actions_count,
                        successful_count,
                        failed_count,
                    }
                },
            )
            .collect())
    }
}
