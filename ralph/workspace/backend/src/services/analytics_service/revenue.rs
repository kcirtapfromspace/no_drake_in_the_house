//! Revenue Service
//!
//! Tracks user streaming activity and calculates revenue contribution to artists.
//! Supports all major streaming platforms with per-platform payout rates.

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::trouble_score::TroubleTier;

/// Supported streaming platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Spotify,
    AppleMusic,
    Tidal,
    YouTubeMusic,
    Deezer,
    AmazonMusic,
    Pandora,
}

impl Platform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::Spotify => "spotify",
            Platform::AppleMusic => "apple_music",
            Platform::Tidal => "tidal",
            Platform::YouTubeMusic => "youtube_music",
            Platform::Deezer => "deezer",
            Platform::AmazonMusic => "amazon_music",
            Platform::Pandora => "pandora",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "spotify" => Some(Platform::Spotify),
            "apple_music" => Some(Platform::AppleMusic),
            "tidal" => Some(Platform::Tidal),
            "youtube_music" => Some(Platform::YouTubeMusic),
            "deezer" => Some(Platform::Deezer),
            "amazon_music" => Some(Platform::AmazonMusic),
            "pandora" => Some(Platform::Pandora),
            _ => None,
        }
    }

    pub fn all() -> Vec<Platform> {
        vec![
            Platform::Spotify,
            Platform::AppleMusic,
            Platform::Tidal,
            Platform::YouTubeMusic,
            Platform::Deezer,
            Platform::AmazonMusic,
            Platform::Pandora,
        ]
    }
}

/// Platform payout rate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutRate {
    pub platform: String,
    pub rate_per_stream: Decimal,
    pub rate_per_minute: Option<Decimal>,
    pub subscription_monthly: Option<Decimal>,
    pub rate_tier: String,
    pub effective_date: NaiveDate,
    pub source_url: Option<String>,
}

/// Revenue breakdown for a single platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformRevenue {
    pub platform: String,
    pub streams: i64,
    pub listening_time_ms: Option<i64>,
    pub estimated_revenue: Decimal,
    pub percentage_of_total: f64,
    pub rate_applied: Decimal,
}

/// Complete revenue breakdown for an artist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistRevenueBreakdown {
    pub artist_id: Uuid,
    pub artist_name: String,
    pub trouble_tier: Option<TroubleTier>,
    pub trouble_score: Option<f64>,
    pub total_streams: i64,
    pub total_revenue: Decimal,
    pub percentage_of_user_spend: f64,
    pub by_platform: Vec<PlatformRevenue>,
}

/// User's overall revenue distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRevenueDistribution {
    pub user_id: Uuid,
    pub platform: String,
    pub period: String,
    pub total_streams: i64,
    pub total_revenue: Decimal,
    pub subscription_cost: Option<Decimal>,

    /// Revenue breakdown
    pub revenue_to_clean_artists: Decimal,
    pub revenue_to_problematic_artists: Decimal,
    pub problematic_percentage: f64,

    /// Top artists receiving revenue
    pub top_artists: Vec<ArtistRevenueBreakdown>,
    /// Top problematic artists receiving revenue
    pub top_problematic_artists: Vec<ArtistRevenueBreakdown>,
}

/// Playcount record for a user-artist pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPlaycount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub artist_id: Uuid,
    pub artist_name: String,
    pub platform: String,
    pub play_count: i32,
    pub listening_time_ms: Option<i64>,
    pub estimated_revenue: Decimal,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
}

/// Revenue service
pub struct RevenueService {
    pool: PgPool,
}

impl RevenueService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get current payout rate for a platform
    pub async fn get_payout_rate(
        &self,
        platform: Platform,
        tier: Option<&str>,
    ) -> Result<PayoutRate> {
        let tier = tier.unwrap_or("standard");

        let rate = sqlx::query_as::<_, PayoutRateRow>(
            r#"
            SELECT
                platform,
                rate_per_stream,
                rate_per_minute,
                subscription_monthly,
                rate_tier,
                effective_date,
                source_url
            FROM platform_payout_rates
            WHERE platform = $1
            AND rate_tier = $2
            AND effective_date <= CURRENT_DATE
            AND (end_date IS NULL OR end_date >= CURRENT_DATE)
            ORDER BY effective_date DESC
            LIMIT 1
            "#,
        )
        .bind(platform.as_str())
        .bind(tier)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch payout rate")?;

        rate.map(|r| r.into_rate())
            .context("No payout rate found for platform")
    }

    /// Get all current payout rates
    pub async fn get_all_payout_rates(&self) -> Result<Vec<PayoutRate>> {
        let rates = sqlx::query_as::<_, PayoutRateRow>(
            r#"
            SELECT DISTINCT ON (platform, rate_tier)
                platform,
                rate_per_stream,
                rate_per_minute,
                subscription_monthly,
                rate_tier,
                effective_date,
                source_url
            FROM platform_payout_rates
            WHERE effective_date <= CURRENT_DATE
            AND (end_date IS NULL OR end_date >= CURRENT_DATE)
            ORDER BY platform, rate_tier, effective_date DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch payout rates")?;

        Ok(rates.into_iter().map(|r| r.into_rate()).collect())
    }

    /// Record playcount data for a user-artist pair
    pub async fn record_playcount(
        &self,
        user_id: Uuid,
        artist_id: Uuid,
        platform: Platform,
        play_count: i32,
        listening_time_ms: Option<i64>,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<UserPlaycount> {
        // Get current rate
        let rate = self.get_payout_rate(platform, None).await?;
        let estimated_revenue = rate.rate_per_stream * Decimal::from(play_count);

        let row = sqlx::query_as::<_, PlaycountRow>(
            r#"
            INSERT INTO user_artist_playcounts (
                user_id, artist_id, platform, play_count, listening_time_ms,
                estimated_revenue, rate_used, period_type, period_start, period_end
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'monthly', $8, $9)
            ON CONFLICT (user_id, artist_id, platform, period_type, period_start) DO UPDATE SET
                play_count = user_artist_playcounts.play_count + EXCLUDED.play_count,
                listening_time_ms = COALESCE(user_artist_playcounts.listening_time_ms, 0) + COALESCE(EXCLUDED.listening_time_ms, 0),
                estimated_revenue = user_artist_playcounts.estimated_revenue + EXCLUDED.estimated_revenue,
                updated_at = NOW()
            RETURNING
                id, user_id, artist_id, platform, play_count, listening_time_ms,
                estimated_revenue, period_start, period_end
            "#,
        )
        .bind(user_id)
        .bind(artist_id)
        .bind(platform.as_str())
        .bind(play_count)
        .bind(listening_time_ms)
        .bind(estimated_revenue)
        .bind(rate.rate_per_stream)
        .bind(period_start)
        .bind(period_end)
        .fetch_one(&self.pool)
        .await
        .context("Failed to record playcount")?;

        // Get artist name
        let artist_name: String =
            sqlx::query_scalar("SELECT canonical_name FROM artists WHERE id = $1")
                .bind(artist_id)
                .fetch_one(&self.pool)
                .await
                .unwrap_or_else(|_| "Unknown".to_string());

        Ok(UserPlaycount {
            id: row.id,
            user_id: row.user_id,
            artist_id: row.artist_id,
            artist_name,
            platform: row.platform,
            play_count: row.play_count,
            listening_time_ms: row.listening_time_ms,
            estimated_revenue: row.estimated_revenue,
            period_start: row.period_start,
            period_end: row.period_end,
        })
    }

    /// Get user's top artists by revenue
    pub async fn get_user_top_artists(
        &self,
        user_id: Uuid,
        days: i32,
        limit: i32,
    ) -> Result<Vec<ArtistRevenueBreakdown>> {
        let rows = sqlx::query_as::<_, ArtistRevenueRow>(
            r#"
            SELECT
                pc.artist_id,
                a.canonical_name as artist_name,
                ts.trouble_tier::text as trouble_tier,
                ts.total_score as trouble_score,
                SUM(pc.play_count)::bigint as total_streams,
                SUM(pc.estimated_revenue) as total_revenue
            FROM user_artist_playcounts pc
            JOIN artists a ON a.id = pc.artist_id
            LEFT JOIN artist_trouble_scores ts ON ts.artist_id = pc.artist_id
            WHERE pc.user_id = $1
            AND pc.period_start >= CURRENT_DATE - $2::integer
            GROUP BY pc.artist_id, a.canonical_name, ts.trouble_tier, ts.total_score
            ORDER BY SUM(pc.estimated_revenue) DESC
            LIMIT $3
            "#,
        )
        .bind(user_id)
        .bind(days)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch user top artists")?;

        // Get total revenue for percentage calculation
        let total_revenue: Decimal = rows.iter().map(|r| r.total_revenue).sum();

        Ok(rows
            .into_iter()
            .map(|r| {
                let percentage = if total_revenue > Decimal::ZERO {
                    (r.total_revenue / total_revenue * Decimal::from(100))
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0)
                } else {
                    0.0
                };

                ArtistRevenueBreakdown {
                    artist_id: r.artist_id,
                    artist_name: r.artist_name,
                    trouble_tier: r.trouble_tier.and_then(|t| match t.as_str() {
                        "critical" => Some(TroubleTier::Critical),
                        "high" => Some(TroubleTier::High),
                        "moderate" => Some(TroubleTier::Moderate),
                        "low" => Some(TroubleTier::Low),
                        _ => None,
                    }),
                    trouble_score: r.trouble_score,
                    total_streams: r.total_streams.unwrap_or(0),
                    total_revenue: r.total_revenue,
                    percentage_of_user_spend: percentage,
                    by_platform: vec![], // Would need separate query
                }
            })
            .collect())
    }

    /// Get user's top problematic artists by revenue
    pub async fn get_user_problematic_artists(
        &self,
        user_id: Uuid,
        min_tier: TroubleTier,
        days: i32,
        limit: i32,
    ) -> Result<Vec<ArtistRevenueBreakdown>> {
        let tier_values = match min_tier {
            TroubleTier::Low => vec!["low", "moderate", "high", "critical"],
            TroubleTier::Moderate => vec!["moderate", "high", "critical"],
            TroubleTier::High => vec!["high", "critical"],
            TroubleTier::Critical => vec!["critical"],
        };

        let rows = sqlx::query_as::<_, ArtistRevenueRow>(
            r#"
            SELECT
                pc.artist_id,
                a.canonical_name as artist_name,
                ts.trouble_tier::text as trouble_tier,
                ts.total_score as trouble_score,
                SUM(pc.play_count)::bigint as total_streams,
                SUM(pc.estimated_revenue) as total_revenue
            FROM user_artist_playcounts pc
            JOIN artists a ON a.id = pc.artist_id
            JOIN artist_trouble_scores ts ON ts.artist_id = pc.artist_id
            WHERE pc.user_id = $1
            AND pc.period_start >= CURRENT_DATE - $2::integer
            AND ts.trouble_tier::text = ANY($3)
            GROUP BY pc.artist_id, a.canonical_name, ts.trouble_tier, ts.total_score
            ORDER BY SUM(pc.estimated_revenue) DESC
            LIMIT $4
            "#,
        )
        .bind(user_id)
        .bind(days)
        .bind(&tier_values)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch problematic artists")?;

        // Get total user revenue for percentage calculation
        let total_user_revenue: Decimal = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(estimated_revenue), 0)
            FROM user_artist_playcounts
            WHERE user_id = $1
            AND period_start >= CURRENT_DATE - $2::integer
            "#,
        )
        .bind(user_id)
        .bind(days)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(Decimal::ZERO);

        Ok(rows
            .into_iter()
            .map(|r| {
                let percentage = if total_user_revenue > Decimal::ZERO {
                    (r.total_revenue / total_user_revenue * Decimal::from(100))
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0)
                } else {
                    0.0
                };

                ArtistRevenueBreakdown {
                    artist_id: r.artist_id,
                    artist_name: r.artist_name,
                    trouble_tier: r.trouble_tier.and_then(|t| match t.as_str() {
                        "critical" => Some(TroubleTier::Critical),
                        "high" => Some(TroubleTier::High),
                        "moderate" => Some(TroubleTier::Moderate),
                        "low" => Some(TroubleTier::Low),
                        _ => None,
                    }),
                    trouble_score: r.trouble_score,
                    total_streams: r.total_streams.unwrap_or(0),
                    total_revenue: r.total_revenue,
                    percentage_of_user_spend: percentage,
                    by_platform: vec![],
                }
            })
            .collect())
    }

    /// Get user's complete revenue distribution
    pub async fn get_user_revenue_distribution(
        &self,
        user_id: Uuid,
        platform: Option<Platform>,
        days: i32,
    ) -> Result<UserRevenueDistribution> {
        let platform_str = platform.map(|p| p.as_str()).unwrap_or("all");

        // Get totals
        let totals = if platform.is_some() {
            sqlx::query_as::<_, RevenueTotalsRow>(
                r#"
                SELECT
                    COALESCE(SUM(play_count), 0)::bigint as total_streams,
                    COALESCE(SUM(estimated_revenue), 0) as total_revenue,
                    COALESCE(SUM(CASE WHEN ts.trouble_tier IS NULL OR ts.trouble_tier = 'low'
                        THEN pc.estimated_revenue ELSE 0 END), 0) as clean_revenue,
                    COALESCE(SUM(CASE WHEN ts.trouble_tier IN ('moderate', 'high', 'critical')
                        THEN pc.estimated_revenue ELSE 0 END), 0) as problematic_revenue
                FROM user_artist_playcounts pc
                LEFT JOIN artist_trouble_scores ts ON ts.artist_id = pc.artist_id
                WHERE pc.user_id = $1
                AND pc.platform = $2
                AND pc.period_start >= CURRENT_DATE - $3::integer
                "#,
            )
            .bind(user_id)
            .bind(platform_str)
            .bind(days)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, RevenueTotalsRow>(
                r#"
                SELECT
                    COALESCE(SUM(play_count), 0)::bigint as total_streams,
                    COALESCE(SUM(estimated_revenue), 0) as total_revenue,
                    COALESCE(SUM(CASE WHEN ts.trouble_tier IS NULL OR ts.trouble_tier = 'low'
                        THEN pc.estimated_revenue ELSE 0 END), 0) as clean_revenue,
                    COALESCE(SUM(CASE WHEN ts.trouble_tier IN ('moderate', 'high', 'critical')
                        THEN pc.estimated_revenue ELSE 0 END), 0) as problematic_revenue
                FROM user_artist_playcounts pc
                LEFT JOIN artist_trouble_scores ts ON ts.artist_id = pc.artist_id
                WHERE pc.user_id = $1
                AND pc.period_start >= CURRENT_DATE - $2::integer
                "#,
            )
            .bind(user_id)
            .bind(days)
            .fetch_one(&self.pool)
            .await?
        };

        let problematic_percentage = if totals.total_revenue > Decimal::ZERO {
            (totals.problematic_revenue / totals.total_revenue * Decimal::from(100))
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        // Get top artists
        let top_artists = self.get_user_top_artists(user_id, days, 10).await?;

        // Get top problematic artists
        let top_problematic = self
            .get_user_problematic_artists(user_id, TroubleTier::Moderate, days, 10)
            .await?;

        Ok(UserRevenueDistribution {
            user_id,
            platform: platform_str.to_string(),
            period: format!("last_{}_days", days),
            total_streams: totals.total_streams.unwrap_or(0),
            total_revenue: totals.total_revenue,
            subscription_cost: None, // Would need user subscription info
            revenue_to_clean_artists: totals.clean_revenue,
            revenue_to_problematic_artists: totals.problematic_revenue,
            problematic_percentage,
            top_artists,
            top_problematic_artists: top_problematic,
        })
    }

    /// Get revenue breakdown for a specific artist
    pub async fn get_artist_revenue(
        &self,
        user_id: Uuid,
        artist_id: Uuid,
        days: i32,
    ) -> Result<ArtistRevenueBreakdown> {
        // Get artist info and trouble score
        let artist_info = sqlx::query_as::<_, ArtistInfoRow>(
            r#"
            SELECT
                a.id as artist_id,
                a.canonical_name as artist_name,
                ts.trouble_tier::text as trouble_tier,
                ts.total_score as trouble_score
            FROM artists a
            LEFT JOIN artist_trouble_scores ts ON ts.artist_id = a.id
            WHERE a.id = $1
            "#,
        )
        .bind(artist_id)
        .fetch_one(&self.pool)
        .await
        .context("Artist not found")?;

        // Get platform breakdown
        let platform_breakdown = sqlx::query_as::<_, PlatformBreakdownRow>(
            r#"
            SELECT
                platform,
                SUM(play_count)::bigint as streams,
                SUM(listening_time_ms)::bigint as listening_time_ms,
                SUM(estimated_revenue) as estimated_revenue,
                AVG(rate_used) as rate_applied
            FROM user_artist_playcounts
            WHERE user_id = $1
            AND artist_id = $2
            AND period_start >= CURRENT_DATE - $3::integer
            GROUP BY platform
            ORDER BY SUM(estimated_revenue) DESC
            "#,
        )
        .bind(user_id)
        .bind(artist_id)
        .bind(days)
        .fetch_all(&self.pool)
        .await?;

        let total_streams: i64 = platform_breakdown
            .iter()
            .map(|p| p.streams.unwrap_or(0))
            .sum();
        let total_revenue: Decimal = platform_breakdown.iter().map(|p| p.estimated_revenue).sum();

        // Get user total for percentage
        let user_total: Decimal = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(estimated_revenue), 0)
            FROM user_artist_playcounts
            WHERE user_id = $1
            AND period_start >= CURRENT_DATE - $2::integer
            "#,
        )
        .bind(user_id)
        .bind(days)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(Decimal::ZERO);

        let percentage = if user_total > Decimal::ZERO {
            (total_revenue / user_total * Decimal::from(100))
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let by_platform: Vec<PlatformRevenue> = platform_breakdown
            .into_iter()
            .map(|p| {
                let platform_pct = if total_revenue > Decimal::ZERO {
                    (p.estimated_revenue / total_revenue * Decimal::from(100))
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0)
                } else {
                    0.0
                };

                PlatformRevenue {
                    platform: p.platform,
                    streams: p.streams.unwrap_or(0),
                    listening_time_ms: p.listening_time_ms,
                    estimated_revenue: p.estimated_revenue,
                    percentage_of_total: platform_pct,
                    rate_applied: p.rate_applied.unwrap_or(Decimal::ZERO),
                }
            })
            .collect();

        Ok(ArtistRevenueBreakdown {
            artist_id: artist_info.artist_id,
            artist_name: artist_info.artist_name,
            trouble_tier: artist_info.trouble_tier.and_then(|t| match t.as_str() {
                "critical" => Some(TroubleTier::Critical),
                "high" => Some(TroubleTier::High),
                "moderate" => Some(TroubleTier::Moderate),
                "low" => Some(TroubleTier::Low),
                _ => None,
            }),
            trouble_score: artist_info.trouble_score,
            total_streams,
            total_revenue,
            percentage_of_user_spend: percentage,
            by_platform,
        })
    }

    /// Get global revenue leaderboard for problematic artists
    pub async fn get_problematic_revenue_leaderboard(
        &self,
        min_tier: TroubleTier,
        days: i32,
        limit: i32,
    ) -> Result<Vec<GlobalArtistRevenue>> {
        let tier_values = match min_tier {
            TroubleTier::Low => vec!["low", "moderate", "high", "critical"],
            TroubleTier::Moderate => vec!["moderate", "high", "critical"],
            TroubleTier::High => vec!["high", "critical"],
            TroubleTier::Critical => vec!["critical"],
        };

        let rows = sqlx::query_as::<_, GlobalRevenueRow>(
            r#"
            SELECT
                ROW_NUMBER() OVER (ORDER BY SUM(ars.total_revenue) DESC) as rank,
                ars.artist_id,
                a.canonical_name as artist_name,
                ts.trouble_tier::text as trouble_tier,
                ts.total_score as trouble_score,
                SUM(ars.total_streams)::bigint as total_streams,
                SUM(ars.total_revenue) as total_revenue,
                SUM(ars.unique_listeners)::integer as unique_listeners
            FROM artist_revenue_summary ars
            JOIN artists a ON a.id = ars.artist_id
            JOIN artist_trouble_scores ts ON ts.artist_id = ars.artist_id
            WHERE ars.period_date >= CURRENT_DATE - $1::integer
            AND ts.trouble_tier::text = ANY($2)
            GROUP BY ars.artist_id, a.canonical_name, ts.trouble_tier, ts.total_score
            ORDER BY SUM(ars.total_revenue) DESC
            LIMIT $3
            "#,
        )
        .bind(days)
        .bind(&tier_values)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch revenue leaderboard")?;

        Ok(rows
            .into_iter()
            .map(|r| GlobalArtistRevenue {
                rank: r.rank as i32,
                artist_id: r.artist_id,
                artist_name: r.artist_name,
                trouble_tier: r.trouble_tier.and_then(|t| match t.as_str() {
                    "critical" => Some(TroubleTier::Critical),
                    "high" => Some(TroubleTier::High),
                    "moderate" => Some(TroubleTier::Moderate),
                    "low" => Some(TroubleTier::Low),
                    _ => None,
                }),
                trouble_score: r.trouble_score,
                total_streams: r.total_streams.unwrap_or(0),
                total_revenue: r.total_revenue,
                unique_listeners: r.unique_listeners.unwrap_or(0),
            })
            .collect())
    }
}

/// Global artist revenue entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalArtistRevenue {
    pub rank: i32,
    pub artist_id: Uuid,
    pub artist_name: String,
    pub trouble_tier: Option<TroubleTier>,
    pub trouble_score: Option<f64>,
    pub total_streams: i64,
    pub total_revenue: Decimal,
    pub unique_listeners: i32,
}

// Internal row types for sqlx

#[derive(Debug, sqlx::FromRow)]
struct PayoutRateRow {
    platform: String,
    rate_per_stream: Decimal,
    rate_per_minute: Option<Decimal>,
    subscription_monthly: Option<Decimal>,
    rate_tier: String,
    effective_date: NaiveDate,
    source_url: Option<String>,
}

impl PayoutRateRow {
    fn into_rate(self) -> PayoutRate {
        PayoutRate {
            platform: self.platform,
            rate_per_stream: self.rate_per_stream,
            rate_per_minute: self.rate_per_minute,
            subscription_monthly: self.subscription_monthly,
            rate_tier: self.rate_tier,
            effective_date: self.effective_date,
            source_url: self.source_url,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct PlaycountRow {
    id: Uuid,
    user_id: Uuid,
    artist_id: Uuid,
    platform: String,
    play_count: i32,
    listening_time_ms: Option<i64>,
    estimated_revenue: Decimal,
    period_start: NaiveDate,
    period_end: NaiveDate,
}

#[derive(Debug, sqlx::FromRow)]
struct ArtistRevenueRow {
    artist_id: Uuid,
    artist_name: String,
    trouble_tier: Option<String>,
    trouble_score: Option<f64>,
    total_streams: Option<i64>,
    total_revenue: Decimal,
}

#[derive(Debug, sqlx::FromRow)]
struct RevenueTotalsRow {
    total_streams: Option<i64>,
    total_revenue: Decimal,
    clean_revenue: Decimal,
    problematic_revenue: Decimal,
}

#[derive(Debug, sqlx::FromRow)]
struct ArtistInfoRow {
    artist_id: Uuid,
    artist_name: String,
    trouble_tier: Option<String>,
    trouble_score: Option<f64>,
}

#[derive(Debug, sqlx::FromRow)]
struct PlatformBreakdownRow {
    platform: String,
    streams: Option<i64>,
    listening_time_ms: Option<i64>,
    estimated_revenue: Decimal,
    rate_applied: Option<Decimal>,
}

#[derive(Debug, sqlx::FromRow)]
struct GlobalRevenueRow {
    rank: i64,
    artist_id: Uuid,
    artist_name: String,
    trouble_tier: Option<String>,
    trouble_score: Option<f64>,
    total_streams: Option<i64>,
    total_revenue: Decimal,
    unique_listeners: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_from_str() {
        assert_eq!(Platform::from_str("spotify"), Some(Platform::Spotify));
        assert_eq!(
            Platform::from_str("apple_music"),
            Some(Platform::AppleMusic)
        );
        assert_eq!(Platform::from_str("invalid"), None);
    }

    #[test]
    fn test_platform_all() {
        let all = Platform::all();
        assert_eq!(all.len(), 7);
        assert!(all.contains(&Platform::Spotify));
        assert!(all.contains(&Platform::AppleMusic));
    }
}
