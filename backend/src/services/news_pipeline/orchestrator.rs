//! News Pipeline Orchestrator
//!
//! Coordinates all news ingestion sources and processing components.
//! Manages scheduling, deduplication, and the overall processing pipeline.

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::ingestion::{
    FetchedArticle, NewsApiClient, NewsApiConfig, RedditConfig, RedditMonitor, RssFetcher,
    RssFetcherConfig, TwitterConfig, TwitterMonitor, WebScraper, WebScraperConfig,
};
use super::processing::{
    ArticleEmbedding, EmbeddingConfig, EmbeddingGenerator, EntityExtractor, EntityExtractorConfig,
    ExtractedEntity, OffenseClassification, OffenseClassifier, OffenseClassifierConfig,
};

/// Overall pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsPipelineConfig {
    /// RSS fetcher config
    pub rss: RssFetcherConfig,
    /// NewsAPI config
    pub newsapi: NewsApiConfig,
    /// Twitter config
    pub twitter: TwitterConfig,
    /// Reddit config
    pub reddit: RedditConfig,
    /// Web scraper config
    pub scraper: WebScraperConfig,
    /// Entity extractor config
    pub entity_extractor: EntityExtractorConfig,
    /// Offense classifier config
    pub offense_classifier: OffenseClassifierConfig,
    /// Embedding generator config
    pub embedding: EmbeddingConfig,
    /// Enable web scraping for full article content
    pub enable_scraping: bool,
    /// Enable embedding generation
    pub enable_embeddings: bool,
    /// Maximum articles to process per batch
    pub batch_size: usize,
    /// Deduplication window (hours)
    pub dedup_window_hours: i64,
}

impl Default for NewsPipelineConfig {
    fn default() -> Self {
        Self {
            rss: RssFetcherConfig::default(),
            newsapi: NewsApiConfig::default(),
            twitter: TwitterConfig::default(),
            reddit: RedditConfig::default(),
            scraper: WebScraperConfig::default(),
            entity_extractor: EntityExtractorConfig::default(),
            offense_classifier: OffenseClassifierConfig::default(),
            embedding: EmbeddingConfig::default(),
            enable_scraping: true,
            enable_embeddings: true,
            batch_size: 50,
            dedup_window_hours: 24,
        }
    }
}

/// A processed article with all extracted information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedArticle {
    /// Original article data
    pub article: FetchedArticle,
    /// Extracted entities
    pub entities: Vec<ExtractedEntity>,
    /// Offense classifications
    pub offenses: Vec<OffenseClassification>,
    /// Article embedding (if generated)
    pub embedding: Option<ArticleEmbedding>,
    /// Processing timestamp
    pub processed_at: DateTime<Utc>,
    /// Processing duration in milliseconds
    pub processing_duration_ms: u64,
}

/// Pipeline statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PipelineStats {
    /// Total articles fetched
    pub articles_fetched: usize,
    /// Articles from RSS
    pub rss_articles: usize,
    /// Articles from NewsAPI
    pub newsapi_articles: usize,
    /// Posts from Twitter
    pub twitter_posts: usize,
    /// Posts from Reddit
    pub reddit_posts: usize,
    /// Articles successfully scraped
    pub articles_scraped: usize,
    /// Entities extracted
    pub entities_extracted: usize,
    /// Offenses detected
    pub offenses_detected: usize,
    /// Embeddings generated
    pub embeddings_generated: usize,
    /// Processing errors
    pub errors: usize,
    /// Last run timestamp
    pub last_run: Option<DateTime<Utc>>,
    /// Last run duration in seconds
    pub last_run_duration_secs: Option<f64>,
}

/// News pipeline orchestrator
pub struct NewsPipelineOrchestrator {
    config: NewsPipelineConfig,
    rss_fetcher: RssFetcher,
    newsapi_client: NewsApiClient,
    twitter_monitor: TwitterMonitor,
    reddit_monitor: RedditMonitor,
    web_scraper: WebScraper,
    entity_extractor: EntityExtractor,
    offense_classifier: OffenseClassifier,
    embedding_generator: EmbeddingGenerator,
    /// Seen URLs for deduplication
    seen_urls: Arc<RwLock<HashSet<String>>>,
    /// Pipeline statistics
    stats: Arc<RwLock<PipelineStats>>,
    /// Processing state
    is_running: Arc<RwLock<bool>>,
}

impl NewsPipelineOrchestrator {
    /// Create a new pipeline orchestrator
    pub fn new(config: NewsPipelineConfig) -> Self {
        Self {
            rss_fetcher: RssFetcher::new(config.rss.clone()),
            newsapi_client: NewsApiClient::new(config.newsapi.clone()),
            twitter_monitor: TwitterMonitor::new(config.twitter.clone()),
            reddit_monitor: RedditMonitor::new(config.reddit.clone()),
            web_scraper: WebScraper::new(config.scraper.clone()),
            entity_extractor: EntityExtractor::new(config.entity_extractor.clone()),
            offense_classifier: OffenseClassifier::new(config.offense_classifier.clone()),
            embedding_generator: EmbeddingGenerator::new(config.embedding.clone()),
            config,
            seen_urls: Arc::new(RwLock::new(HashSet::new())),
            stats: Arc::new(RwLock::new(PipelineStats::default())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Run the full pipeline
    pub async fn run(&self) -> Result<Vec<ProcessedArticle>> {
        // Check if already running
        {
            let mut is_running = self.is_running.write().await;
            if *is_running {
                return Err(anyhow::anyhow!("Pipeline is already running"));
            }
            *is_running = true;
        }

        let start = std::time::Instant::now();
        tracing::info!("Starting news pipeline run");

        let result = self.run_internal().await;

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.last_run = Some(Utc::now());
            stats.last_run_duration_secs = Some(start.elapsed().as_secs_f64());
        }

        // Clear running flag
        {
            let mut is_running = self.is_running.write().await;
            *is_running = false;
        }

        result
    }

    /// Internal pipeline execution
    async fn run_internal(&self) -> Result<Vec<ProcessedArticle>> {
        let mut all_articles = Vec::new();
        let mut stats = PipelineStats::default();

        // 1. Fetch from all sources concurrently
        let (rss_result, newsapi_result, twitter_result, reddit_result) = tokio::join!(
            self.fetch_rss(),
            self.fetch_newsapi(),
            self.fetch_twitter(),
            self.fetch_reddit(),
        );

        // Process RSS results
        match rss_result {
            Ok(articles) => {
                stats.rss_articles = articles.len();
                all_articles.extend(articles);
            }
            Err(e) => {
                tracing::error!(error = %e, "RSS fetch failed");
                stats.errors += 1;
            }
        }

        // Process NewsAPI results
        match newsapi_result {
            Ok(articles) => {
                stats.newsapi_articles = articles.len();
                all_articles.extend(articles);
            }
            Err(e) => {
                tracing::error!(error = %e, "NewsAPI fetch failed");
                stats.errors += 1;
            }
        }

        // Process Twitter results
        match twitter_result {
            Ok(articles) => {
                stats.twitter_posts = articles.len();
                all_articles.extend(articles);
            }
            Err(e) => {
                tracing::warn!(error = %e, "Twitter fetch failed (may be disabled)");
                // Don't count as error since Twitter is optional
            }
        }

        // Process Reddit results
        match reddit_result {
            Ok(articles) => {
                stats.reddit_posts = articles.len();
                all_articles.extend(articles);
            }
            Err(e) => {
                tracing::error!(error = %e, "Reddit fetch failed");
                stats.errors += 1;
            }
        }

        tracing::info!(
            total = all_articles.len(),
            rss = stats.rss_articles,
            newsapi = stats.newsapi_articles,
            twitter = stats.twitter_posts,
            reddit = stats.reddit_posts,
            "Fetched articles from all sources"
        );

        // 2. Deduplicate
        let deduplicated = self.deduplicate(all_articles).await;
        stats.articles_fetched = deduplicated.len();

        tracing::info!(count = deduplicated.len(), "Articles after deduplication");

        // 3. Batch for processing
        let batches: Vec<_> = deduplicated
            .chunks(self.config.batch_size)
            .map(|c| c.to_vec())
            .collect();

        let mut processed_articles = Vec::new();

        for batch in batches {
            match self.process_batch(batch).await {
                Ok(processed) => {
                    for article in &processed {
                        stats.entities_extracted += article.entities.len();
                        stats.offenses_detected += article.offenses.len();
                        if article.embedding.is_some() {
                            stats.embeddings_generated += 1;
                        }
                    }
                    processed_articles.extend(processed);
                }
                Err(e) => {
                    tracing::error!(error = %e, "Batch processing failed");
                    stats.errors += 1;
                }
            }
        }

        // Update global stats
        {
            let mut global_stats = self.stats.write().await;
            global_stats.articles_fetched += stats.articles_fetched;
            global_stats.rss_articles += stats.rss_articles;
            global_stats.newsapi_articles += stats.newsapi_articles;
            global_stats.twitter_posts += stats.twitter_posts;
            global_stats.reddit_posts += stats.reddit_posts;
            global_stats.entities_extracted += stats.entities_extracted;
            global_stats.offenses_detected += stats.offenses_detected;
            global_stats.embeddings_generated += stats.embeddings_generated;
            global_stats.errors += stats.errors;
        }

        tracing::info!(
            processed = processed_articles.len(),
            entities = stats.entities_extracted,
            offenses = stats.offenses_detected,
            embeddings = stats.embeddings_generated,
            "Pipeline run complete"
        );

        Ok(processed_articles)
    }

    /// Fetch from RSS sources
    async fn fetch_rss(&self) -> Result<Vec<FetchedArticle>> {
        self.rss_fetcher.fetch_all().await
    }

    /// Fetch from NewsAPI
    async fn fetch_newsapi(&self) -> Result<Vec<FetchedArticle>> {
        self.newsapi_client.search_music_news(None).await
    }

    /// Fetch from Twitter
    async fn fetch_twitter(&self) -> Result<Vec<FetchedArticle>> {
        let tweets = self.twitter_monitor.search_music_news().await?;

        // Convert tweets to FetchedArticle format
        Ok(tweets
            .into_iter()
            .map(|tweet| {
                let title_preview = if tweet.text.len() > 50 {
                    format!("{}...", &tweet.text[..50])
                } else {
                    tweet.text.clone()
                };
                FetchedArticle {
                    id: tweet.id,
                    source_id: Uuid::nil(), // Twitter source
                    url: format!(
                        "https://twitter.com/{}/status/{}",
                        tweet.author_username, tweet.tweet_id
                    ),
                    title: format!("@{}: {}", tweet.author_username, title_preview),
                    content: Some(tweet.text),
                    published_at: tweet.created_at,
                    fetched_at: Utc::now(),
                    authors: vec![tweet.author_username],
                    categories: vec!["twitter".to_string()],
                    image_url: None,
                }
            })
            .collect())
    }

    /// Fetch from Reddit
    async fn fetch_reddit(&self) -> Result<Vec<FetchedArticle>> {
        let posts = self.reddit_monitor.fetch_all_subreddits().await?;

        // Convert Reddit posts to FetchedArticle format
        Ok(posts
            .into_iter()
            .map(|post| FetchedArticle {
                id: post.id,
                source_id: Uuid::nil(), // Reddit source
                url: post.url,
                title: post.title,
                content: post.content,
                published_at: Some(post.created_at),
                fetched_at: Utc::now(),
                authors: vec![post.author],
                categories: vec!["reddit".to_string(), post.subreddit],
                image_url: None,
            })
            .collect())
    }

    /// Deduplicate articles by URL
    async fn deduplicate(&self, articles: Vec<FetchedArticle>) -> Vec<FetchedArticle> {
        let mut seen = self.seen_urls.write().await;
        let mut deduplicated = Vec::new();

        for article in articles {
            if !seen.contains(&article.url) {
                seen.insert(article.url.clone());
                deduplicated.push(article);
            }
        }

        // Prune old entries periodically (simple approach)
        if seen.len() > 100000 {
            seen.clear();
            for article in &deduplicated {
                seen.insert(article.url.clone());
            }
        }

        deduplicated
    }

    /// Process a batch of articles
    async fn process_batch(&self, articles: Vec<FetchedArticle>) -> Result<Vec<ProcessedArticle>> {
        let mut processed = Vec::with_capacity(articles.len());

        for article in articles {
            let start = std::time::Instant::now();

            // Optionally scrape for full content
            let article = if self.config.enable_scraping && article.content.is_none() {
                match self.web_scraper.enrich_article(&article).await {
                    Ok(enriched) => enriched,
                    Err(e) => {
                        tracing::debug!(url = %article.url, error = %e, "Scraping failed, using original");
                        article
                    }
                }
            } else {
                article
            };

            // Extract entities
            let content = article.content.as_deref().unwrap_or("");
            let entities = self
                .entity_extractor
                .extract(content, Some(&article.title))
                .await
                .unwrap_or_default();

            // Classify offenses
            let offenses = self
                .offense_classifier
                .classify(article.id, content, Some(&article.title), &entities)
                .unwrap_or_default();

            // Generate embedding if enabled
            let embedding = if self.config.enable_embeddings {
                self.embedding_generator
                    .embed_article(article.id, &article.title, article.content.as_deref())
                    .await
                    .ok()
            } else {
                None
            };

            let duration = start.elapsed().as_millis() as u64;

            processed.push(ProcessedArticle {
                article,
                entities,
                offenses,
                embedding,
                processed_at: Utc::now(),
                processing_duration_ms: duration,
            });
        }

        Ok(processed)
    }

    /// Get pipeline statistics
    pub async fn get_stats(&self) -> PipelineStats {
        self.stats.read().await.clone()
    }

    /// Check if pipeline is currently running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Get the entity extractor for adding known artists
    pub fn entity_extractor(&self) -> &EntityExtractor {
        &self.entity_extractor
    }

    /// Run RSS-only fetch (for scheduled jobs)
    pub async fn run_rss_only(&self) -> Result<Vec<ProcessedArticle>> {
        let articles = self.fetch_rss().await?;
        let deduplicated = self.deduplicate(articles).await;
        self.process_batch(deduplicated).await
    }

    /// Run social media only (Twitter + Reddit)
    pub async fn run_social_only(&self) -> Result<Vec<ProcessedArticle>> {
        let (twitter_result, reddit_result) =
            tokio::join!(self.fetch_twitter(), self.fetch_reddit(),);

        let mut all_articles = Vec::new();

        if let Ok(articles) = twitter_result {
            all_articles.extend(articles);
        }

        if let Ok(articles) = reddit_result {
            all_articles.extend(articles);
        }

        let deduplicated = self.deduplicate(all_articles).await;
        self.process_batch(deduplicated).await
    }

    /// Search for specific artist news
    pub async fn search_artist(&self, artist_name: &str) -> Result<Vec<ProcessedArticle>> {
        let articles = self.newsapi_client.search_artist_news(artist_name).await?;
        self.process_batch(articles).await
    }

    /// Get articles with detected offenses
    pub fn filter_with_offenses(articles: &[ProcessedArticle]) -> Vec<&ProcessedArticle> {
        articles.iter().filter(|a| !a.offenses.is_empty()).collect()
    }

    /// Get articles mentioning specific artist
    pub fn filter_by_artist(
        articles: &[ProcessedArticle],
        artist_id: Uuid,
    ) -> Vec<&ProcessedArticle> {
        articles
            .iter()
            .filter(|a| a.entities.iter().any(|e| e.artist_id == Some(artist_id)))
            .collect()
    }

    /// Reset statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = PipelineStats::default();
    }

    /// Clear seen URLs (force re-processing)
    pub async fn clear_seen_urls(&self) {
        let mut seen = self.seen_urls.write().await;
        seen.clear();
    }
}

/// Scheduled pipeline runner
pub struct ScheduledPipelineRunner {
    orchestrator: Arc<NewsPipelineOrchestrator>,
    /// RSS poll interval
    rss_interval: Duration,
    /// Social media poll interval
    social_interval: Duration,
    /// Full run interval
    full_interval: Duration,
}

impl ScheduledPipelineRunner {
    /// Create a new scheduled runner
    pub fn new(orchestrator: Arc<NewsPipelineOrchestrator>) -> Self {
        Self {
            orchestrator,
            rss_interval: Duration::minutes(30),
            social_interval: Duration::hours(1),
            full_interval: Duration::hours(6),
        }
    }

    /// Set RSS polling interval
    pub fn with_rss_interval(mut self, interval: Duration) -> Self {
        self.rss_interval = interval;
        self
    }

    /// Set social media polling interval
    pub fn with_social_interval(mut self, interval: Duration) -> Self {
        self.social_interval = interval;
        self
    }

    /// Set full run interval
    pub fn with_full_interval(mut self, interval: Duration) -> Self {
        self.full_interval = interval;
        self
    }

    /// Start the scheduled runner (spawns background tasks)
    pub fn start(self) -> ScheduledPipelineHandle {
        let handle = ScheduledPipelineHandle {
            stop_flag: Arc::new(RwLock::new(false)),
        };

        let stop_flag = handle.stop_flag.clone();
        let orchestrator = self.orchestrator.clone();
        let rss_interval = self.rss_interval;

        // RSS polling task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(
                rss_interval.num_seconds() as u64,
            ));

            loop {
                interval.tick().await;

                if *stop_flag.read().await {
                    break;
                }

                if let Err(e) = orchestrator.run_rss_only().await {
                    tracing::error!(error = %e, "Scheduled RSS fetch failed");
                }
            }
        });

        let stop_flag = handle.stop_flag.clone();
        let orchestrator = self.orchestrator.clone();
        let social_interval = self.social_interval;

        // Social media polling task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(
                social_interval.num_seconds() as u64,
            ));

            loop {
                interval.tick().await;

                if *stop_flag.read().await {
                    break;
                }

                if let Err(e) = orchestrator.run_social_only().await {
                    tracing::error!(error = %e, "Scheduled social fetch failed");
                }
            }
        });

        let stop_flag = handle.stop_flag.clone();
        let orchestrator = self.orchestrator;
        let full_interval = self.full_interval;

        // Full run task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(
                full_interval.num_seconds() as u64,
            ));

            loop {
                interval.tick().await;

                if *stop_flag.read().await {
                    break;
                }

                if let Err(e) = orchestrator.run().await {
                    tracing::error!(error = %e, "Scheduled full run failed");
                }
            }
        });

        handle
    }
}

/// Handle for stopping the scheduled runner
pub struct ScheduledPipelineHandle {
    stop_flag: Arc<RwLock<bool>>,
}

impl ScheduledPipelineHandle {
    /// Stop all scheduled tasks
    pub async fn stop(&self) {
        let mut flag = self.stop_flag.write().await;
        *flag = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NewsPipelineConfig::default();
        assert!(config.enable_scraping);
        assert!(config.enable_embeddings);
        assert_eq!(config.batch_size, 50);
    }

    #[test]
    fn test_pipeline_creation() {
        let config = NewsPipelineConfig::default();
        let _orchestrator = NewsPipelineOrchestrator::new(config);
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let config = NewsPipelineConfig::default();
        let orchestrator = NewsPipelineOrchestrator::new(config);

        let stats = orchestrator.get_stats().await;
        assert_eq!(stats.articles_fetched, 0);
        assert_eq!(stats.errors, 0);
    }
}
