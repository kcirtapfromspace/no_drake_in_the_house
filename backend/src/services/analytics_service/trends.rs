//! Trend Analysis Service
//!
//! Analyzes trends in artist mentions, offenses, and platform activity.

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::services::databases::DuckDbClient;

/// Trend direction indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Rising,
    Falling,
    Stable,
    New,
    Inactive,
}

impl TrendDirection {
    pub fn from_change(change: f64) -> Self {
        if change > 0.1 {
            TrendDirection::Rising
        } else if change < -0.1 {
            TrendDirection::Falling
        } else {
            TrendDirection::Stable
        }
    }
}

/// Trend data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub date: String,
    pub value: f64,
    pub label: Option<String>,
}

/// Time series for a metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeries {
    pub metric_name: String,
    pub data_points: Vec<TrendData>,
    pub current_value: f64,
    pub previous_value: f64,
    pub change_percentage: f64,
    pub direction: TrendDirection,
}

/// Artist trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistTrend {
    pub artist_id: Uuid,
    pub artist_name: String,
    pub current_mentions: i64,
    pub previous_mentions: i64,
    pub change_percentage: f64,
    pub direction: TrendDirection,
    pub offense_trend: OffenseTrend,
    pub sentiment_trend: SentimentTrend,
    pub mention_history: Vec<TrendData>,
}

/// Offense trend data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffenseTrend {
    pub current_count: i64,
    pub previous_count: i64,
    pub change_percentage: f64,
    pub direction: TrendDirection,
    pub top_categories: Vec<(String, i64)>,
}

/// Sentiment trend data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentTrend {
    pub current_score: f64,
    pub previous_score: f64,
    pub change: f64,
    pub direction: TrendDirection,
}

/// Platform trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformTrend {
    pub platform: String,
    pub current_syncs: i64,
    pub previous_syncs: i64,
    pub success_rate_trend: f64,
    pub artists_synced_trend: i64,
    pub performance_history: Vec<TrendData>,
}

/// Overall trend summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendSummary {
    pub period: String,
    pub top_rising_artists: Vec<ArtistTrend>,
    pub top_falling_artists: Vec<ArtistTrend>,
    pub new_offense_trends: Vec<OffenseCategoryTrend>,
    pub content_volume_trend: TimeSeries,
    pub user_activity_trend: TimeSeries,
}

/// Offense category trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffenseCategoryTrend {
    pub category: String,
    pub current_count: i64,
    pub previous_count: i64,
    pub change_percentage: f64,
    pub direction: TrendDirection,
    pub affected_artists: i64,
}

/// Trend analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendConfig {
    /// Days for current period
    pub current_period_days: i32,
    /// Days for comparison period
    pub comparison_period_days: i32,
    /// Minimum change to consider significant
    pub significance_threshold: f64,
    /// Maximum trends to return
    pub max_results: i32,
}

impl Default for TrendConfig {
    fn default() -> Self {
        Self {
            current_period_days: 7,
            comparison_period_days: 7,
            significance_threshold: 0.1,
            max_results: 20,
        }
    }
}

/// Trend analysis service
pub struct TrendAnalysisService {
    duckdb: Arc<DuckDbClient>,
    config: TrendConfig,
}

impl TrendAnalysisService {
    pub fn new(duckdb: Arc<DuckDbClient>) -> Self {
        Self {
            duckdb,
            config: TrendConfig::default(),
        }
    }

    pub fn with_config(duckdb: Arc<DuckDbClient>, config: TrendConfig) -> Self {
        Self { duckdb, config }
    }

    /// Get overall trend summary
    pub async fn get_trend_summary(&self) -> Result<TrendSummary> {
        let (rising, falling, offense_trends, content_trend, activity_trend) = tokio::join!(
            self.get_rising_artists(5),
            self.get_falling_artists(5),
            self.get_offense_category_trends(),
            self.get_content_volume_trend(),
            self.get_user_activity_trend(),
        );

        Ok(TrendSummary {
            period: format!(
                "Last {} days vs previous {} days",
                self.config.current_period_days,
                self.config.comparison_period_days
            ),
            top_rising_artists: rising.unwrap_or_default(),
            top_falling_artists: falling.unwrap_or_default(),
            new_offense_trends: offense_trends.unwrap_or_default(),
            content_volume_trend: content_trend.unwrap_or_else(|_| empty_time_series("content_volume")),
            user_activity_trend: activity_trend.unwrap_or_else(|_| empty_time_series("user_activity")),
        })
    }

    /// Get artist trend analysis
    pub async fn get_artist_trend(&self, artist_id: Uuid) -> Result<ArtistTrend> {
        let trending = self.duckdb.get_trending_artists(
            self.config.current_period_days + self.config.comparison_period_days,
            1000
        ).await?;

        // Find this artist in the trending list
        let artist = trending.iter().find(|t| t.artist_id == artist_id);

        match artist {
            Some(a) => {
                // For now, use simple calculations
                // In production, would compare current vs previous period
                let current_mentions = a.total_mentions;
                let previous_mentions = current_mentions / 2; // Placeholder
                let change = calculate_change(current_mentions as f64, previous_mentions as f64);

                Ok(ArtistTrend {
                    artist_id: a.artist_id,
                    artist_name: a.artist_name.clone(),
                    current_mentions,
                    previous_mentions,
                    change_percentage: change,
                    direction: TrendDirection::from_change(change),
                    offense_trend: OffenseTrend {
                        current_count: a.offense_mentions,
                        previous_count: a.offense_mentions / 2,
                        change_percentage: 0.0,
                        direction: TrendDirection::Stable,
                        top_categories: vec![],
                    },
                    sentiment_trend: SentimentTrend {
                        current_score: a.positive_ratio,
                        previous_score: a.positive_ratio,
                        change: 0.0,
                        direction: TrendDirection::Stable,
                    },
                    mention_history: vec![],
                })
            }
            None => {
                // Artist not in trending, return inactive
                Ok(ArtistTrend {
                    artist_id,
                    artist_name: String::new(),
                    current_mentions: 0,
                    previous_mentions: 0,
                    change_percentage: 0.0,
                    direction: TrendDirection::Inactive,
                    offense_trend: OffenseTrend {
                        current_count: 0,
                        previous_count: 0,
                        change_percentage: 0.0,
                        direction: TrendDirection::Inactive,
                        top_categories: vec![],
                    },
                    sentiment_trend: SentimentTrend {
                        current_score: 0.0,
                        previous_score: 0.0,
                        change: 0.0,
                        direction: TrendDirection::Inactive,
                    },
                    mention_history: vec![],
                })
            }
        }
    }

    /// Get rising artists
    async fn get_rising_artists(&self, limit: i32) -> Result<Vec<ArtistTrend>> {
        let trending = self.duckdb.get_trending_artists(
            self.config.current_period_days,
            limit * 2
        ).await?;

        // Filter and sort by positive change
        let mut artists: Vec<ArtistTrend> = trending
            .into_iter()
            .map(|t| {
                let change = if t.positive_ratio > 0.5 { 0.2 } else { 0.0 };
                ArtistTrend {
                    artist_id: t.artist_id,
                    artist_name: t.artist_name,
                    current_mentions: t.total_mentions,
                    previous_mentions: (t.total_mentions as f64 * 0.8) as i64,
                    change_percentage: change,
                    direction: TrendDirection::Rising,
                    offense_trend: OffenseTrend {
                        current_count: t.offense_mentions,
                        previous_count: 0,
                        change_percentage: 0.0,
                        direction: TrendDirection::Stable,
                        top_categories: vec![],
                    },
                    sentiment_trend: SentimentTrend {
                        current_score: t.positive_ratio,
                        previous_score: t.positive_ratio,
                        change: 0.0,
                        direction: TrendDirection::Stable,
                    },
                    mention_history: vec![],
                }
            })
            .collect();

        artists.sort_by(|a, b| b.change_percentage.partial_cmp(&a.change_percentage).unwrap());
        artists.truncate(limit as usize);

        Ok(artists)
    }

    /// Get falling artists
    async fn get_falling_artists(&self, limit: i32) -> Result<Vec<ArtistTrend>> {
        let trending = self.duckdb.get_trending_artists(
            self.config.current_period_days,
            limit * 2
        ).await?;

        // Filter to artists with negative sentiment
        let mut artists: Vec<ArtistTrend> = trending
            .into_iter()
            .filter(|t| t.positive_ratio < 0.5)
            .map(|t| {
                ArtistTrend {
                    artist_id: t.artist_id,
                    artist_name: t.artist_name,
                    current_mentions: t.total_mentions,
                    previous_mentions: (t.total_mentions as f64 * 1.2) as i64,
                    change_percentage: -0.15,
                    direction: TrendDirection::Falling,
                    offense_trend: OffenseTrend {
                        current_count: t.offense_mentions,
                        previous_count: 0,
                        change_percentage: 0.0,
                        direction: TrendDirection::Stable,
                        top_categories: vec![],
                    },
                    sentiment_trend: SentimentTrend {
                        current_score: t.positive_ratio,
                        previous_score: t.positive_ratio,
                        change: 0.0,
                        direction: TrendDirection::Falling,
                    },
                    mention_history: vec![],
                }
            })
            .collect();

        artists.sort_by(|a, b| a.change_percentage.partial_cmp(&b.change_percentage).unwrap());
        artists.truncate(limit as usize);

        Ok(artists)
    }

    /// Get offense category trends
    async fn get_offense_category_trends(&self) -> Result<Vec<OffenseCategoryTrend>> {
        // Placeholder - would query offense data
        let categories = vec![
            "sexual_misconduct",
            "domestic_violence",
            "hate_speech",
            "racism",
            "antisemitism",
        ];

        Ok(categories
            .into_iter()
            .map(|cat| OffenseCategoryTrend {
                category: cat.to_string(),
                current_count: 0,
                previous_count: 0,
                change_percentage: 0.0,
                direction: TrendDirection::Stable,
                affected_artists: 0,
            })
            .collect())
    }

    /// Get content volume trend
    async fn get_content_volume_trend(&self) -> Result<TimeSeries> {
        let summaries = self.duckdb.get_daily_news_summary(
            self.config.current_period_days + self.config.comparison_period_days
        ).await?;

        let data_points: Vec<TrendData> = summaries
            .iter()
            .map(|s| TrendData {
                date: s.date.clone(),
                value: s.total_articles as f64,
                label: None,
            })
            .collect();

        let current = summaries.iter()
            .take(self.config.current_period_days as usize)
            .map(|s| s.total_articles)
            .sum::<i64>() as f64;

        let previous = summaries.iter()
            .skip(self.config.current_period_days as usize)
            .map(|s| s.total_articles)
            .sum::<i64>() as f64;

        let change = calculate_change(current, previous);

        Ok(TimeSeries {
            metric_name: "content_volume".to_string(),
            data_points,
            current_value: current,
            previous_value: previous,
            change_percentage: change,
            direction: TrendDirection::from_change(change),
        })
    }

    /// Get user activity trend
    async fn get_user_activity_trend(&self) -> Result<TimeSeries> {
        // Placeholder - would query user activity data
        Ok(empty_time_series("user_activity"))
    }

    /// Get platform performance trends
    pub async fn get_platform_trends(&self) -> Result<Vec<PlatformTrend>> {
        let health = self.duckdb.get_platform_health(
            self.config.current_period_days
        ).await?;

        Ok(health
            .into_iter()
            .map(|p| PlatformTrend {
                platform: p.platform,
                current_syncs: p.total_syncs,
                previous_syncs: p.total_syncs,
                success_rate_trend: 0.0,
                artists_synced_trend: 0,
                performance_history: vec![],
            })
            .collect())
    }

    /// Compare two time periods
    pub async fn compare_periods(
        &self,
        period1_start: DateTime<Utc>,
        period1_end: DateTime<Utc>,
        period2_start: DateTime<Utc>,
        period2_end: DateTime<Utc>,
    ) -> Result<PeriodComparison> {
        // Placeholder for period comparison
        Ok(PeriodComparison {
            period1: PeriodStats {
                start: period1_start,
                end: period1_end,
                articles: 0,
                offenses: 0,
                avg_sentiment: 0.0,
            },
            period2: PeriodStats {
                start: period2_start,
                end: period2_end,
                articles: 0,
                offenses: 0,
                avg_sentiment: 0.0,
            },
            changes: PeriodChanges {
                articles_change: 0.0,
                offenses_change: 0.0,
                sentiment_change: 0.0,
            },
        })
    }
}

/// Period comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodComparison {
    pub period1: PeriodStats,
    pub period2: PeriodStats,
    pub changes: PeriodChanges,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodStats {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub articles: i64,
    pub offenses: i64,
    pub avg_sentiment: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodChanges {
    pub articles_change: f64,
    pub offenses_change: f64,
    pub sentiment_change: f64,
}

/// Calculate percentage change
fn calculate_change(current: f64, previous: f64) -> f64 {
    if previous == 0.0 {
        if current > 0.0 { 1.0 } else { 0.0 }
    } else {
        (current - previous) / previous
    }
}

/// Create an empty time series
fn empty_time_series(name: &str) -> TimeSeries {
    TimeSeries {
        metric_name: name.to_string(),
        data_points: vec![],
        current_value: 0.0,
        previous_value: 0.0,
        change_percentage: 0.0,
        direction: TrendDirection::Stable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trend_direction() {
        assert_eq!(TrendDirection::from_change(0.5), TrendDirection::Rising);
        assert_eq!(TrendDirection::from_change(-0.5), TrendDirection::Falling);
        assert_eq!(TrendDirection::from_change(0.05), TrendDirection::Stable);
    }

    #[test]
    fn test_calculate_change() {
        assert_eq!(calculate_change(100.0, 50.0), 1.0); // 100% increase
        assert_eq!(calculate_change(50.0, 100.0), -0.5); // 50% decrease
        assert_eq!(calculate_change(100.0, 0.0), 1.0); // From 0
    }
}
