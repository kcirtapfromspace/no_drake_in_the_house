//! Trouble Score Service
//!
//! Calculates and manages artist "trouble scores" based on:
//! - Offense severity (35% weight)
//! - Evidence credibility (20% weight)
//! - Recency of offenses (15% weight)
//! - Community consensus/block count (15% weight)
//! - Revenue contribution (15% weight)

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Trouble tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "trouble_tier", rename_all = "lowercase")]
pub enum TroubleTier {
    Low,
    Moderate,
    High,
    Critical,
}

impl TroubleTier {
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s >= 0.75 => TroubleTier::Critical,
            s if s >= 0.50 => TroubleTier::High,
            s if s >= 0.25 => TroubleTier::Moderate,
            _ => TroubleTier::Low,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TroubleTier::Low => "low",
            TroubleTier::Moderate => "moderate",
            TroubleTier::High => "high",
            TroubleTier::Critical => "critical",
        }
    }
}

/// Component breakdown of trouble score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TroubleScoreComponents {
    /// Based on max offense severity (Minor=0.25, Moderate=0.5, Severe=0.75, Egregious=1.0)
    pub severity_score: f64,
    /// Average evidence credibility (Verified=1.0, Pending=0.5, Disputed=0.25)
    pub evidence_score: f64,
    /// How recent are the offenses (1.0 if <1yr, decay over time)
    pub recency_score: f64,
    /// Block count normalized against max
    pub community_score: f64,
    /// Revenue contribution factor
    pub revenue_score: f64,
}

impl Default for TroubleScoreComponents {
    fn default() -> Self {
        Self {
            severity_score: 0.0,
            evidence_score: 0.0,
            recency_score: 0.0,
            community_score: 0.0,
            revenue_score: 0.0,
        }
    }
}

/// Score weights configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreWeights {
    pub severity_weight: f64,
    pub evidence_weight: f64,
    pub recency_weight: f64,
    pub community_weight: f64,
    pub revenue_weight: f64,
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            severity_weight: 0.35,
            evidence_weight: 0.20,
            recency_weight: 0.15,
            community_weight: 0.15,
            revenue_weight: 0.15,
        }
    }
}

/// Complete trouble score for an artist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistTroubleScore {
    pub id: Uuid,
    pub artist_id: Uuid,
    pub artist_name: String,

    /// Component scores
    pub components: TroubleScoreComponents,

    /// Composite score (0.0 - 1.0)
    pub total_score: f64,
    /// Tier classification
    pub trouble_tier: TroubleTier,

    /// Raw metrics
    pub offense_count: i32,
    pub verified_offense_count: i32,
    pub block_count: i32,

    /// Offense breakdown
    pub egregious_count: i32,
    pub severe_count: i32,
    pub moderate_count: i32,
    pub minor_count: i32,

    /// Dates
    pub first_offense_date: Option<DateTime<Utc>>,
    pub last_offense_date: Option<DateTime<Utc>>,
    pub last_calculated_at: DateTime<Utc>,
}

/// Leaderboard entry for problematic artists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TroubleLeaderboardEntry {
    pub rank: i32,
    pub artist_id: Uuid,
    pub artist_name: String,
    pub total_score: f64,
    pub trouble_tier: TroubleTier,
    pub offense_count: i32,
    pub block_count: i32,
    pub most_severe_category: Option<String>,
}

/// Score history entry for tracking changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreHistoryEntry {
    pub id: Uuid,
    pub artist_id: Uuid,
    pub total_score: f64,
    pub trouble_tier: TroubleTier,
    pub trigger_reason: String,
    pub calculated_at: DateTime<Utc>,
}

/// Trouble score service
pub struct TroubleScoreService {
    pool: PgPool,
    weights: ScoreWeights,
}

impl TroubleScoreService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            weights: ScoreWeights::default(),
        }
    }

    pub fn with_weights(pool: PgPool, weights: ScoreWeights) -> Self {
        Self { pool, weights }
    }

    /// Get trouble score for a specific artist
    pub async fn get_artist_score(&self, artist_id: Uuid) -> Result<Option<ArtistTroubleScore>> {
        let record = sqlx::query_as::<_, TroubleScoreRow>(
            r#"
            SELECT
                ts.id,
                ts.artist_id,
                a.canonical_name as artist_name,
                ts.severity_score,
                ts.evidence_score,
                ts.recency_score,
                ts.community_score,
                ts.revenue_score,
                ts.total_score,
                ts.trouble_tier::text as trouble_tier,
                ts.offense_count,
                ts.verified_offense_count,
                ts.block_count,
                ts.egregious_count,
                ts.severe_count,
                ts.moderate_count,
                ts.minor_count,
                ts.first_offense_date,
                ts.last_offense_date,
                ts.last_calculated_at
            FROM artist_trouble_scores ts
            JOIN artists a ON a.id = ts.artist_id
            WHERE ts.artist_id = $1
            "#,
        )
        .bind(artist_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch trouble score")?;

        Ok(record.map(|r| r.into_score()))
    }

    /// Get leaderboard of most problematic artists
    pub async fn get_leaderboard(
        &self,
        min_tier: Option<TroubleTier>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<TroubleLeaderboardEntry>> {
        let tier_filter = min_tier.map(|t| t.as_str()).unwrap_or("low");

        let records = sqlx::query_as::<_, LeaderboardRow>(
            r#"
            SELECT
                ROW_NUMBER() OVER (ORDER BY ts.total_score DESC) as rank,
                ts.artist_id,
                a.canonical_name as artist_name,
                ts.total_score,
                ts.trouble_tier::text as trouble_tier,
                ts.offense_count,
                ts.block_count,
                (
                    SELECT ao.category::text
                    FROM artist_offenses ao
                    WHERE ao.artist_id = ts.artist_id
                    ORDER BY ao.severity DESC
                    LIMIT 1
                ) as most_severe_category
            FROM artist_trouble_scores ts
            JOIN artists a ON a.id = ts.artist_id
            WHERE ts.trouble_tier::text >= $1
            ORDER BY ts.total_score DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tier_filter)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch leaderboard")?;

        Ok(records.into_iter().map(|r| r.into_entry()).collect())
    }

    /// Get artists by trouble tier
    pub async fn get_artists_by_tier(&self, tier: TroubleTier) -> Result<Vec<ArtistTroubleScore>> {
        let records = sqlx::query_as::<_, TroubleScoreRow>(
            r#"
            SELECT
                ts.id,
                ts.artist_id,
                a.canonical_name as artist_name,
                ts.severity_score,
                ts.evidence_score,
                ts.recency_score,
                ts.community_score,
                ts.revenue_score,
                ts.total_score,
                ts.trouble_tier::text as trouble_tier,
                ts.offense_count,
                ts.verified_offense_count,
                ts.block_count,
                ts.egregious_count,
                ts.severe_count,
                ts.moderate_count,
                ts.minor_count,
                ts.first_offense_date,
                ts.last_offense_date,
                ts.last_calculated_at
            FROM artist_trouble_scores ts
            JOIN artists a ON a.id = ts.artist_id
            WHERE ts.trouble_tier = $1::trouble_tier
            ORDER BY ts.total_score DESC
            "#,
        )
        .bind(tier.as_str())
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch artists by tier")?;

        Ok(records.into_iter().map(|r| r.into_score()).collect())
    }

    /// Trigger recalculation for a specific artist
    pub async fn recalculate_artist(&self, artist_id: Uuid) -> Result<ArtistTroubleScore> {
        // Call the PostgreSQL function we created in migration
        sqlx::query("SELECT recalculate_trouble_score($1, 'api_request')")
            .bind(artist_id)
            .execute(&self.pool)
            .await
            .context("Failed to recalculate trouble score")?;

        // Fetch the updated score
        self.get_artist_score(artist_id)
            .await?
            .context("Artist score not found after recalculation")
    }

    /// Recalculate all artist scores
    pub async fn recalculate_all(&self) -> Result<RecalculationSummary> {
        let mut recalculated = 0;
        let mut errors = 0;

        // Get all artists with offenses
        let artist_ids: Vec<Uuid> =
            sqlx::query_scalar("SELECT DISTINCT artist_id FROM artist_offenses")
                .fetch_all(&self.pool)
                .await
                .context("Failed to fetch artist IDs")?;

        for artist_id in &artist_ids {
            match sqlx::query("SELECT recalculate_trouble_score($1, 'bulk_recalculation')")
                .bind(artist_id)
                .execute(&self.pool)
                .await
            {
                Ok(_) => recalculated += 1,
                Err(e) => {
                    tracing::warn!(
                        "Failed to recalculate score for artist {}: {}",
                        artist_id,
                        e
                    );
                    errors += 1;
                }
            }
        }

        Ok(RecalculationSummary {
            total_artists: artist_ids.len() as i32,
            recalculated,
            errors,
            completed_at: Utc::now(),
        })
    }

    /// Get score history for an artist
    pub async fn get_score_history(
        &self,
        artist_id: Uuid,
        limit: i32,
    ) -> Result<Vec<ScoreHistoryEntry>> {
        let records = sqlx::query_as::<_, ScoreHistoryRow>(
            r#"
            SELECT
                id,
                artist_id,
                total_score,
                trouble_tier::text as trouble_tier,
                trigger_reason,
                calculated_at
            FROM trouble_score_history
            WHERE artist_id = $1
            ORDER BY calculated_at DESC
            LIMIT $2
            "#,
        )
        .bind(artist_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch score history")?;

        Ok(records.into_iter().map(|r| r.into_entry()).collect())
    }

    /// Get tier distribution statistics
    pub async fn get_tier_distribution(&self) -> Result<TierDistribution> {
        let row = sqlx::query_as::<_, TierDistributionRow>(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE trouble_tier = 'low') as low_count,
                COUNT(*) FILTER (WHERE trouble_tier = 'moderate') as moderate_count,
                COUNT(*) FILTER (WHERE trouble_tier = 'high') as high_count,
                COUNT(*) FILTER (WHERE trouble_tier = 'critical') as critical_count,
                COUNT(*) as total_count
            FROM artist_trouble_scores
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch tier distribution")?;

        Ok(TierDistribution {
            low: row.low_count.unwrap_or(0),
            moderate: row.moderate_count.unwrap_or(0),
            high: row.high_count.unwrap_or(0),
            critical: row.critical_count.unwrap_or(0),
            total: row.total_count.unwrap_or(0),
        })
    }

    /// Update score weights (admin function)
    pub async fn update_weights(&mut self, weights: ScoreWeights) -> Result<()> {
        // Validate weights sum to 1.0
        let sum = weights.severity_weight
            + weights.evidence_weight
            + weights.recency_weight
            + weights.community_weight
            + weights.revenue_weight;

        if (sum - 1.0).abs() > 0.01 {
            anyhow::bail!("Weights must sum to 1.0, got {}", sum);
        }

        // Update in database
        sqlx::query(
            r#"
            UPDATE trouble_score_weights
            SET
                severity_weight = $1,
                evidence_weight = $2,
                recency_weight = $3,
                community_weight = $4,
                revenue_weight = $5,
                updated_at = NOW()
            WHERE is_active = TRUE
            "#,
        )
        .bind(weights.severity_weight)
        .bind(weights.evidence_weight)
        .bind(weights.recency_weight)
        .bind(weights.community_weight)
        .bind(weights.revenue_weight)
        .execute(&self.pool)
        .await
        .context("Failed to update weights")?;

        self.weights = weights;
        Ok(())
    }
}

/// Recalculation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecalculationSummary {
    pub total_artists: i32,
    pub recalculated: i32,
    pub errors: i32,
    pub completed_at: DateTime<Utc>,
}

/// Tier distribution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierDistribution {
    pub low: i64,
    pub moderate: i64,
    pub high: i64,
    pub critical: i64,
    pub total: i64,
}

// Internal row types for sqlx

#[derive(Debug, sqlx::FromRow)]
struct TroubleScoreRow {
    id: Uuid,
    artist_id: Uuid,
    artist_name: String,
    severity_score: f64,
    evidence_score: f64,
    recency_score: f64,
    community_score: f64,
    revenue_score: f64,
    total_score: f64,
    trouble_tier: String,
    offense_count: Option<i32>,
    verified_offense_count: Option<i32>,
    block_count: Option<i32>,
    egregious_count: Option<i32>,
    severe_count: Option<i32>,
    moderate_count: Option<i32>,
    minor_count: Option<i32>,
    first_offense_date: Option<DateTime<Utc>>,
    last_offense_date: Option<DateTime<Utc>>,
    last_calculated_at: Option<DateTime<Utc>>,
}

impl TroubleScoreRow {
    fn into_score(self) -> ArtistTroubleScore {
        ArtistTroubleScore {
            id: self.id,
            artist_id: self.artist_id,
            artist_name: self.artist_name,
            components: TroubleScoreComponents {
                severity_score: self.severity_score,
                evidence_score: self.evidence_score,
                recency_score: self.recency_score,
                community_score: self.community_score,
                revenue_score: self.revenue_score,
            },
            total_score: self.total_score,
            trouble_tier: match self.trouble_tier.as_str() {
                "critical" => TroubleTier::Critical,
                "high" => TroubleTier::High,
                "moderate" => TroubleTier::Moderate,
                _ => TroubleTier::Low,
            },
            offense_count: self.offense_count.unwrap_or(0),
            verified_offense_count: self.verified_offense_count.unwrap_or(0),
            block_count: self.block_count.unwrap_or(0),
            egregious_count: self.egregious_count.unwrap_or(0),
            severe_count: self.severe_count.unwrap_or(0),
            moderate_count: self.moderate_count.unwrap_or(0),
            minor_count: self.minor_count.unwrap_or(0),
            first_offense_date: self.first_offense_date,
            last_offense_date: self.last_offense_date,
            last_calculated_at: self.last_calculated_at.unwrap_or_else(Utc::now),
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct LeaderboardRow {
    rank: i64,
    artist_id: Uuid,
    artist_name: String,
    total_score: f64,
    trouble_tier: String,
    offense_count: Option<i32>,
    block_count: Option<i32>,
    most_severe_category: Option<String>,
}

impl LeaderboardRow {
    fn into_entry(self) -> TroubleLeaderboardEntry {
        TroubleLeaderboardEntry {
            rank: self.rank as i32,
            artist_id: self.artist_id,
            artist_name: self.artist_name,
            total_score: self.total_score,
            trouble_tier: match self.trouble_tier.as_str() {
                "critical" => TroubleTier::Critical,
                "high" => TroubleTier::High,
                "moderate" => TroubleTier::Moderate,
                _ => TroubleTier::Low,
            },
            offense_count: self.offense_count.unwrap_or(0),
            block_count: self.block_count.unwrap_or(0),
            most_severe_category: self.most_severe_category,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ScoreHistoryRow {
    id: Uuid,
    artist_id: Uuid,
    total_score: f64,
    trouble_tier: String,
    trigger_reason: Option<String>,
    calculated_at: DateTime<Utc>,
}

impl ScoreHistoryRow {
    fn into_entry(self) -> ScoreHistoryEntry {
        ScoreHistoryEntry {
            id: self.id,
            artist_id: self.artist_id,
            total_score: self.total_score,
            trouble_tier: match self.trouble_tier.as_str() {
                "critical" => TroubleTier::Critical,
                "high" => TroubleTier::High,
                "moderate" => TroubleTier::Moderate,
                _ => TroubleTier::Low,
            },
            trigger_reason: self.trigger_reason.unwrap_or_else(|| "unknown".to_string()),
            calculated_at: self.calculated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct TierDistributionRow {
    low_count: Option<i64>,
    moderate_count: Option<i64>,
    high_count: Option<i64>,
    critical_count: Option<i64>,
    total_count: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trouble_tier_from_score() {
        assert_eq!(TroubleTier::from_score(0.0), TroubleTier::Low);
        assert_eq!(TroubleTier::from_score(0.24), TroubleTier::Low);
        assert_eq!(TroubleTier::from_score(0.25), TroubleTier::Moderate);
        assert_eq!(TroubleTier::from_score(0.49), TroubleTier::Moderate);
        assert_eq!(TroubleTier::from_score(0.50), TroubleTier::High);
        assert_eq!(TroubleTier::from_score(0.74), TroubleTier::High);
        assert_eq!(TroubleTier::from_score(0.75), TroubleTier::Critical);
        assert_eq!(TroubleTier::from_score(1.0), TroubleTier::Critical);
    }

    #[test]
    fn test_score_weights_default() {
        let weights = ScoreWeights::default();
        let sum = weights.severity_weight
            + weights.evidence_weight
            + weights.recency_weight
            + weights.community_weight
            + weights.revenue_weight;
        assert!((sum - 1.0).abs() < 0.01);
    }
}
