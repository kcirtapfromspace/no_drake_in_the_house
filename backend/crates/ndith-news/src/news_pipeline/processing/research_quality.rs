//! Research Quality Scoring
//!
//! Scores the quality of evidence research per artist (0–100).
//! Drives the autoresearch loop — lower scores trigger more investigation.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Research quality score for an artist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchQualityScore {
    pub artist_id: Uuid,
    /// Overall quality score (0–100)
    pub quality_score: f64,
    /// How many different source types provided evidence (0–20)
    pub source_diversity_score: f64,
    /// Does evidence span the artist's career (0–20)
    pub temporal_coverage_score: f64,
    /// Offenses confirmed by multiple independent sources (0–20)
    pub corroboration_score: f64,
    /// Average LLM/classifier confidence (0–20)
    pub confidence_score: f64,
    /// Has the system searched all available source types (0–20)
    pub completeness_score: f64,
    /// Which sources have been searched
    pub sources_searched: Vec<String>,
    /// When last researched
    pub last_research_at: Option<DateTime<Utc>>,
    /// Number of research iterations
    pub research_iterations: i32,
    /// Whether more research is needed
    pub needs_more_research: bool,
}

/// All available source types for completeness scoring
pub const ALL_SOURCE_TYPES: &[&str] = &[
    "rss",
    "newsapi",
    "twitter",
    "reddit",
    "wikipedia",
    "web_search",
];

/// Research quality scorer
pub struct ResearchQualityScorer {
    db_pool: PgPool,
}

impl ResearchQualityScorer {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Calculate the research quality score for an artist
    pub async fn calculate(&self, artist_id: Uuid) -> Result<ResearchQualityScore> {
        let source_diversity = self.calc_source_diversity(artist_id).await?;
        let temporal_coverage = self.calc_temporal_coverage(artist_id).await?;
        let corroboration = self.calc_corroboration(artist_id).await?;
        let confidence = self.calc_confidence(artist_id).await?;
        let (completeness, sources_searched) = self.calc_completeness(artist_id).await?;

        let quality_score =
            source_diversity + temporal_coverage + corroboration + confidence + completeness;

        let needs_more_research = quality_score < 70.0;

        Ok(ResearchQualityScore {
            artist_id,
            quality_score,
            source_diversity_score: source_diversity,
            temporal_coverage_score: temporal_coverage,
            corroboration_score: corroboration,
            confidence_score: confidence,
            completeness_score: completeness,
            sources_searched,
            last_research_at: Some(Utc::now()),
            research_iterations: 0, // Caller increments
            needs_more_research,
        })
    }

    /// Source diversity (0–20): how many different source types provided evidence
    async fn calc_source_diversity(&self, artist_id: Uuid) -> Result<f64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(DISTINCT na.source_type)
            FROM news_offense_classifications noc
            JOIN news_articles na ON noc.article_id = na.id
            WHERE noc.artist_id = $1
            "#,
        )
        .bind(artist_id)
        .fetch_one(&self.db_pool)
        .await
        .unwrap_or(0);

        // 0 sources = 0, 1 = 5, 2 = 10, 3 = 15, 4+ = 20
        Ok((count as f64 * 5.0).min(20.0))
    }

    /// Temporal coverage (0–20): does evidence span the artist's career
    async fn calc_temporal_coverage(&self, artist_id: Uuid) -> Result<f64> {
        let row: Option<(Option<DateTime<Utc>>, Option<DateTime<Utc>>)> = sqlx::query_as(
            r#"
            SELECT MIN(na.published_at), MAX(na.published_at)
            FROM news_offense_classifications noc
            JOIN news_articles na ON noc.article_id = na.id
            WHERE noc.artist_id = $1
              AND na.published_at IS NOT NULL
            "#,
        )
        .bind(artist_id)
        .fetch_optional(&self.db_pool)
        .await
        .unwrap_or(None);

        match row {
            Some((Some(min_date), Some(max_date))) => {
                let span_days = (max_date - min_date).num_days().abs();
                // 0 days = 2, 30+ days = 8, 365+ days = 15, 1825+ days (5yr) = 20
                let score = if span_days >= 1825 {
                    20.0
                } else if span_days >= 365 {
                    15.0
                } else if span_days >= 30 {
                    8.0
                } else {
                    2.0
                };
                Ok(score)
            }
            _ => Ok(0.0),
        }
    }

    /// Corroboration (0–20): offenses confirmed by multiple independent sources
    async fn calc_corroboration(&self, artist_id: Uuid) -> Result<f64> {
        // Count offense categories that have evidence from 2+ different articles
        let corroborated: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(DISTINCT offense_category)
            FROM (
                SELECT offense_category, COUNT(DISTINCT article_id) as source_count
                FROM news_offense_classifications
                WHERE artist_id = $1
                GROUP BY offense_category
                HAVING COUNT(DISTINCT article_id) >= 2
            ) sub
            "#,
        )
        .bind(artist_id)
        .fetch_one(&self.db_pool)
        .await
        .unwrap_or(0);

        // 0 = 0, 1 = 7, 2 = 14, 3+ = 20
        Ok((corroborated as f64 * 7.0).min(20.0))
    }

    /// Confidence (0–20): average classifier confidence for this artist
    async fn calc_confidence(&self, artist_id: Uuid) -> Result<f64> {
        let avg_confidence: Option<f64> = sqlx::query_scalar(
            r#"
            SELECT AVG(confidence)::float8
            FROM news_offense_classifications
            WHERE artist_id = $1
            "#,
        )
        .bind(artist_id)
        .fetch_optional(&self.db_pool)
        .await
        .unwrap_or(None);

        match avg_confidence {
            Some(avg) => Ok((avg * 20.0).min(20.0)),
            None => Ok(0.0),
        }
    }

    /// Completeness (0–20): has the system searched all available source types
    async fn calc_completeness(&self, artist_id: Uuid) -> Result<(f64, Vec<String>)> {
        // Check what sources have been searched from the research quality table
        let sources: Option<serde_json::Value> = sqlx::query_scalar(
            r#"
            SELECT sources_searched
            FROM artist_research_quality
            WHERE artist_id = $1
            "#,
        )
        .bind(artist_id)
        .fetch_optional(&self.db_pool)
        .await
        .unwrap_or(None);

        let searched: Vec<String> = sources
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        let total = ALL_SOURCE_TYPES.len() as f64;
        let searched_count = searched.len() as f64;

        let score = (searched_count / total * 20.0).min(20.0);
        Ok((score, searched))
    }

    /// Persist the quality score to database
    pub async fn persist(&self, score: &ResearchQualityScore) -> Result<()> {
        let sources_json =
            serde_json::to_value(&score.sources_searched).unwrap_or(serde_json::json!([]));

        sqlx::query(
            r#"
            INSERT INTO artist_research_quality (
                artist_id, quality_score,
                source_diversity_score, temporal_coverage_score,
                corroboration_score, confidence_score, completeness_score,
                sources_searched, last_research_at,
                research_iterations, needs_more_research,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW())
            ON CONFLICT (artist_id) DO UPDATE SET
                quality_score = $2,
                source_diversity_score = $3,
                temporal_coverage_score = $4,
                corroboration_score = $5,
                confidence_score = $6,
                completeness_score = $7,
                sources_searched = $8,
                last_research_at = $9,
                research_iterations = $10,
                needs_more_research = $11,
                updated_at = NOW()
            "#,
        )
        .bind(score.artist_id)
        .bind(score.quality_score)
        .bind(score.source_diversity_score)
        .bind(score.temporal_coverage_score)
        .bind(score.corroboration_score)
        .bind(score.confidence_score)
        .bind(score.completeness_score)
        .bind(&sources_json)
        .bind(score.last_research_at)
        .bind(score.research_iterations)
        .bind(score.needs_more_research)
        .execute(&self.db_pool)
        .await
        .context("Failed to persist research quality score")?;

        Ok(())
    }

    /// Get the weakest dimension for targeted research
    pub fn weakest_dimension(score: &ResearchQualityScore) -> &'static str {
        let dimensions = [
            ("source_diversity", score.source_diversity_score),
            ("temporal_coverage", score.temporal_coverage_score),
            ("corroboration", score.corroboration_score),
            ("confidence", score.confidence_score),
            ("completeness", score.completeness_score),
        ];

        dimensions
            .iter()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(name, _)| *name)
            .unwrap_or("completeness")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weakest_dimension() {
        let score = ResearchQualityScore {
            artist_id: Uuid::new_v4(),
            quality_score: 50.0,
            source_diversity_score: 15.0,
            temporal_coverage_score: 10.0,
            corroboration_score: 5.0, // Weakest
            confidence_score: 12.0,
            completeness_score: 8.0,
            sources_searched: vec!["rss".to_string()],
            last_research_at: None,
            research_iterations: 0,
            needs_more_research: true,
        };

        assert_eq!(
            ResearchQualityScorer::weakest_dimension(&score),
            "corroboration"
        );
    }

    #[test]
    fn test_all_source_types() {
        assert!(ALL_SOURCE_TYPES.contains(&"wikipedia"));
        assert!(ALL_SOURCE_TYPES.contains(&"web_search"));
        assert_eq!(ALL_SOURCE_TYPES.len(), 6);
    }
}
