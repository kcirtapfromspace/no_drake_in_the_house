//! Artist Researcher
//!
//! Single-pass, deterministic research pipeline for artist offense investigation.
//! Runs through all free sources in a fixed order — no iterative loops, no LLM.
//!
//! Flow:
//!   1. Wikipedia fetch (free, unlimited)
//!   2. Existing RSS articles from DB (free, already ingested)
//!   3. Brave Search (free, within monthly quota)
//!   4. NewsAPI search (free, within daily quota)
//!   5. Persist results to Convex
//!   6. Calculate quality score
//!   7. Mark artist as researched — DONE

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use uuid::Uuid;

use crate::convex_client::{ConvexClient, UpdateResearchQualityArgs};

use super::ingestion::{WebSearchClient, WikipediaClient};
use super::orchestrator::NewsPipelineOrchestrator;
use super::processing::ResearchQualityScorer;

/// Artist researcher configuration
#[derive(Debug, Clone)]
pub struct ArtistResearcherConfig {
    /// Target quality score (informational — does NOT drive loops)
    pub target_quality: f64,
    /// Delay between artists in milliseconds (rate-limit pacing)
    pub inter_artist_delay_ms: u64,
    /// Maximum Brave Search calls per month (free tier = 2000)
    pub brave_monthly_limit: u64,
    /// Maximum NewsAPI calls per day (free tier = 100)
    pub newsapi_daily_limit: u64,
}

impl Default for ArtistResearcherConfig {
    fn default() -> Self {
        Self {
            target_quality: 70.0,
            inter_artist_delay_ms: 2000,
            brave_monthly_limit: 2000,
            newsapi_daily_limit: 100,
        }
    }
}

/// Free-tier budget tracker (no persistence needed — resets on restart are fine)
pub struct FreeTierBudget {
    brave_calls_this_month: AtomicU64,
    newsapi_calls_today: AtomicU64,
    brave_monthly_limit: u64,
    newsapi_daily_limit: u64,
}

impl FreeTierBudget {
    fn new(brave_limit: u64, newsapi_limit: u64) -> Self {
        Self {
            brave_calls_this_month: AtomicU64::new(0),
            newsapi_calls_today: AtomicU64::new(0),
            brave_monthly_limit: brave_limit,
            newsapi_daily_limit: newsapi_limit,
        }
    }

    pub fn can_use_brave(&self) -> bool {
        self.brave_calls_this_month.load(Ordering::Relaxed) < self.brave_monthly_limit
    }

    pub fn record_brave(&self) {
        self.brave_calls_this_month.fetch_add(1, Ordering::Relaxed);
    }

    pub fn can_use_newsapi(&self) -> bool {
        self.newsapi_calls_today.load(Ordering::Relaxed) < self.newsapi_daily_limit
    }

    pub fn record_newsapi(&self) {
        self.newsapi_calls_today.fetch_add(1, Ordering::Relaxed);
    }

    pub fn brave_remaining(&self) -> u64 {
        self.brave_monthly_limit
            .saturating_sub(self.brave_calls_this_month.load(Ordering::Relaxed))
    }

    pub fn newsapi_remaining(&self) -> u64 {
        self.newsapi_daily_limit
            .saturating_sub(self.newsapi_calls_today.load(Ordering::Relaxed))
    }
}

/// Result of a complete research run for one artist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResult {
    pub artist_id: Uuid,
    pub artist_name: String,
    pub final_quality_score: f64,
    pub total_articles_found: usize,
    pub total_offenses_detected: usize,
    pub sources_used: Vec<String>,
    pub duration_seconds: f64,
}

/// Single-pass artist researcher — no loops, no LLM, all free sources.
///
/// Quality score persistence goes to Convex via `ConvexClient`.
/// The `ResearchQualityScorer` (PostgreSQL) is still used for score *calculation*
/// since it reads from existing news tables. The final result is written to Convex.
pub struct ArtistResearcher {
    convex: ConvexClient,
    #[allow(dead_code)]
    config: ArtistResearcherConfig,
    quality_scorer: ResearchQualityScorer,
    news_pipeline: Option<Arc<NewsPipelineOrchestrator>>,
    wikipedia_client: Option<WikipediaClient>,
    web_search_client: Option<WebSearchClient>,
    free_tier_budget: Arc<FreeTierBudget>,
}

impl ArtistResearcher {
    pub fn new(db_pool: PgPool, convex: ConvexClient, config: ArtistResearcherConfig) -> Self {
        let quality_scorer = ResearchQualityScorer::new(db_pool);
        let free_tier_budget = Arc::new(FreeTierBudget::new(
            config.brave_monthly_limit,
            config.newsapi_daily_limit,
        ));

        Self {
            convex,
            config,
            quality_scorer,
            news_pipeline: None,
            wikipedia_client: None,
            web_search_client: None,
            free_tier_budget,
        }
    }

    /// Set the news pipeline for NewsAPI searches
    pub fn with_news_pipeline(mut self, pipeline: Arc<NewsPipelineOrchestrator>) -> Self {
        self.news_pipeline = Some(pipeline);
        self
    }

    /// Set the Wikipedia client
    pub fn with_wikipedia(mut self) -> Self {
        self.wikipedia_client = Some(WikipediaClient::new());
        self
    }

    /// Set the web search client
    pub fn with_web_search(mut self, client: WebSearchClient) -> Self {
        self.web_search_client = Some(client);
        self
    }

    /// Get the free-tier budget tracker (shared across artists)
    pub fn free_tier_budget(&self) -> &Arc<FreeTierBudget> {
        &self.free_tier_budget
    }

    /// Get the inter-artist delay
    pub fn inter_artist_delay_ms(&self) -> u64 {
        self.config.inter_artist_delay_ms
    }

    /// Research a single artist — one pass through all free sources, then done.
    ///
    /// `convex_artist_id` is the Convex document ID (e.g., `"j57394xkh..."`) passed
    /// from the evidence finder. When provided, it is used for the quality score
    /// write so the update lands on the correct artist record in Convex.
    pub async fn research_artist(
        &self,
        artist_id: Uuid,
        artist_name: &str,
        convex_artist_id: Option<&str>,
    ) -> Result<ResearchResult> {
        let start = std::time::Instant::now();
        let mut total_articles = 0usize;
        let mut total_offenses = 0usize;
        let mut sources_used = Vec::new();

        tracing::info!(
            artist = artist_name,
            artist_id = %artist_id,
            "Starting single-pass research"
        );

        // ── Step 1: Wikipedia (free, unlimited) ──
        if let Some(wiki) = &self.wikipedia_client {
            match wiki.fetch_artist_controversies(artist_name).await {
                Ok(articles) => {
                    let count = articles.len();
                    if count > 0 {
                        sources_used.push("wikipedia".to_string());
                        if let Some(pipeline) = &self.news_pipeline {
                            let processed = pipeline
                                .process_research_articles(articles)
                                .await
                                .unwrap_or_default();
                            let offenses: usize = processed.iter().map(|p| p.offenses.len()).sum();
                            total_articles += count;
                            total_offenses += offenses;
                        } else {
                            total_articles += count;
                        }
                    }
                    tracing::debug!(artist = artist_name, articles = count, "Wikipedia done");
                }
                Err(e) => {
                    tracing::warn!(artist = artist_name, error = %e, "Wikipedia fetch failed");
                }
            }
        }

        // ── Step 2: Brave Search (free, within monthly quota) ──
        if let Some(search) = &self.web_search_client {
            if self.free_tier_budget.can_use_brave() {
                match search.search_artist_controversies(artist_name).await {
                    Ok(articles) => {
                        self.free_tier_budget.record_brave();
                        let count = articles.len();
                        if count > 0 {
                            sources_used.push("web_search".to_string());
                            if let Some(pipeline) = &self.news_pipeline {
                                let processed = pipeline
                                    .process_research_articles(articles)
                                    .await
                                    .unwrap_or_default();
                                let offenses: usize =
                                    processed.iter().map(|p| p.offenses.len()).sum();
                                total_articles += count;
                                total_offenses += offenses;
                            } else {
                                total_articles += count;
                            }
                        }
                        tracing::debug!(
                            artist = artist_name,
                            articles = count,
                            remaining = self.free_tier_budget.brave_remaining(),
                            "Brave Search done"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(artist = artist_name, error = %e, "Brave Search failed");
                    }
                }
            } else {
                tracing::debug!(
                    artist = artist_name,
                    "Brave Search skipped — monthly quota exhausted"
                );
            }
        }

        // ── Step 3: NewsAPI (free, within daily quota) ──
        if let Some(pipeline) = &self.news_pipeline {
            if self.free_tier_budget.can_use_newsapi() {
                match pipeline.search_artist(artist_name).await {
                    Ok(processed) => {
                        self.free_tier_budget.record_newsapi();
                        if !processed.is_empty() {
                            sources_used.push("newsapi".to_string());
                            let offenses: usize = processed.iter().map(|p| p.offenses.len()).sum();
                            total_articles += processed.len();
                            total_offenses += offenses;
                        }
                        tracing::debug!(
                            artist = artist_name,
                            articles = processed.len(),
                            remaining = self.free_tier_budget.newsapi_remaining(),
                            "NewsAPI done"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(artist = artist_name, error = %e, "NewsAPI search failed");
                    }
                }
            } else {
                tracing::debug!(
                    artist = artist_name,
                    "NewsAPI skipped — daily quota exhausted"
                );
            }
        }

        // ── Step 4: Calculate quality score (still reads from PG for calculation) ──
        let score = self.quality_scorer.calculate(artist_id).await?;
        let mut persisted_score = score;
        persisted_score.research_iterations = 1; // Single pass = 1 iteration
        persisted_score.sources_searched = sources_used.clone();

        // ── Step 5: Persist quality score to Convex ──
        // Use Convex document ID when available (passed from evidenceFinder),
        // otherwise fall back to the transient UUID (will likely fail in Convex
        // but we still try + fall back to PG).
        let id_for_convex = convex_artist_id
            .map(|s| s.to_string())
            .unwrap_or_else(|| artist_id.to_string());

        let quality_args = UpdateResearchQualityArgs {
            artist_id: id_for_convex.clone(),
            quality_score: persisted_score.quality_score,
            sources_searched: persisted_score.sources_searched.clone(),
            research_iterations: persisted_score.research_iterations as f64,
        };

        match self
            .convex
            .update_artist_research_quality(&quality_args)
            .await
        {
            Ok(resp) => {
                tracing::debug!(
                    artist_id = %id_for_convex,
                    convex_id = %resp.id,
                    quality = persisted_score.quality_score,
                    "Persisted research quality to Convex"
                );
            }
            Err(e) => {
                tracing::warn!(
                    artist_id = %id_for_convex,
                    error = %e,
                    "Failed to persist research quality to Convex — falling back to PG"
                );
                // Fall back to PostgreSQL persist so we don't lose the score
                if let Err(pg_err) = self.quality_scorer.persist(&persisted_score).await {
                    tracing::error!(
                        artist_id = %artist_id,
                        error = %pg_err,
                        "PostgreSQL fallback also failed for quality score"
                    );
                }
            }
        }

        let result = ResearchResult {
            artist_id,
            artist_name: artist_name.to_string(),
            final_quality_score: persisted_score.quality_score,
            total_articles_found: total_articles,
            total_offenses_detected: total_offenses,
            sources_used,
            duration_seconds: start.elapsed().as_secs_f64(),
        };

        tracing::info!(
            artist = artist_name,
            quality = result.final_quality_score,
            articles = result.total_articles_found,
            offenses = result.total_offenses_detected,
            sources = ?result.sources_used,
            duration = result.duration_seconds,
            "Research complete"
        );

        Ok(result)
    }
}

// ── Backward-compatible type aliases ──

/// Legacy alias — use `ArtistResearcher` instead
pub type AutoresearchAgent = ArtistResearcher;

/// Legacy alias — use `ArtistResearcherConfig` instead
pub type AutoresearchConfig = ArtistResearcherConfig;

/// Legacy alias — use `ResearchResult` instead
pub type AutoresearchResult = ResearchResult;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ArtistResearcherConfig::default();
        assert_eq!(config.target_quality, 70.0);
        assert_eq!(config.inter_artist_delay_ms, 2000);
        assert_eq!(config.brave_monthly_limit, 2000);
        assert_eq!(config.newsapi_daily_limit, 100);
    }

    #[test]
    fn test_free_tier_budget() {
        let budget = FreeTierBudget::new(3, 2);
        assert!(budget.can_use_brave());
        assert!(budget.can_use_newsapi());

        budget.record_brave();
        budget.record_brave();
        budget.record_brave();
        assert!(!budget.can_use_brave());
        assert_eq!(budget.brave_remaining(), 0);

        budget.record_newsapi();
        budget.record_newsapi();
        assert!(!budget.can_use_newsapi());
        assert_eq!(budget.newsapi_remaining(), 0);
    }

    #[test]
    fn test_type_aliases() {
        // Ensure backward-compatible type aliases compile
        let _config: AutoresearchConfig = ArtistResearcherConfig::default();
    }
}
