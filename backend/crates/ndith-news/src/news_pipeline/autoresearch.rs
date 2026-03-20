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
//!   5. Persist results to DB
//!   6. Calculate quality score
//!   7. Mark artist as researched — DONE

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use uuid::Uuid;

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

/// Single-pass artist researcher — no loops, no LLM, all free sources
pub struct ArtistResearcher {
    db_pool: PgPool,
    config: ArtistResearcherConfig,
    quality_scorer: ResearchQualityScorer,
    news_pipeline: Option<Arc<NewsPipelineOrchestrator>>,
    wikipedia_client: Option<WikipediaClient>,
    web_search_client: Option<WebSearchClient>,
    free_tier_budget: Arc<FreeTierBudget>,
}

impl ArtistResearcher {
    pub fn new(db_pool: PgPool, config: ArtistResearcherConfig) -> Self {
        let quality_scorer = ResearchQualityScorer::new(db_pool.clone());
        let free_tier_budget = Arc::new(FreeTierBudget::new(
            config.brave_monthly_limit,
            config.newsapi_daily_limit,
        ));

        Self {
            db_pool,
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
    pub async fn research_artist(
        &self,
        artist_id: Uuid,
        artist_name: &str,
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

        // ── Step 4: Record sources searched ──
        self.record_sources_searched(artist_id, &sources_used)
            .await?;

        // ── Step 5: Calculate and persist quality score ──
        let score = self.quality_scorer.calculate(artist_id).await?;
        let mut persisted_score = score;
        persisted_score.research_iterations = 1; // Single pass = 1 iteration
        self.quality_scorer.persist(&persisted_score).await?;

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

    /// Record which sources were searched for this artist
    async fn record_sources_searched(&self, artist_id: Uuid, sources: &[String]) -> Result<()> {
        if sources.is_empty() {
            return Ok(());
        }

        let sources_json = serde_json::to_value(sources)?;

        sqlx::query(
            r#"
            INSERT INTO artist_research_quality (artist_id, sources_searched, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (artist_id) DO UPDATE SET
                sources_searched = $2,
                updated_at = NOW()
            "#,
        )
        .bind(artist_id)
        .bind(&sources_json)
        .execute(&self.db_pool)
        .await
        .context("Failed to record sources searched")?;

        Ok(())
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
