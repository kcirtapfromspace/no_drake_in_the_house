//! News Pipeline Module
//!
//! Live news tracking and processing for artist offense detection:
//! - RSS feed ingestion from music news sources
//! - NewsAPI integration for broader coverage
//! - Social media monitoring (Twitter, Reddit)
//! - Web scraping for additional sources
//! - Entity extraction (NER) for artist identification
//! - Offense classification
//! - Vector embeddings for semantic search

pub mod ingestion;
pub mod processing;
pub mod orchestrator;

// Re-export main types
pub use ingestion::{
    RssFetcher, RssFetcherConfig, FetchedArticle,
    NewsApiClient, NewsApiConfig,
    TwitterMonitor, TwitterConfig,
    RedditMonitor, RedditConfig,
    WebScraper, WebScraperConfig,
};

pub use processing::{
    EntityExtractor, ExtractedEntity, EntityType,
    OffenseClassifier, OffenseClassification,
    EmbeddingGenerator, ArticleEmbedding,
};

pub use orchestrator::{
    NewsPipelineOrchestrator, NewsPipelineConfig,
    ProcessedArticle, PipelineStats,
    ScheduledPipelineRunner, ScheduledPipelineHandle,
};
