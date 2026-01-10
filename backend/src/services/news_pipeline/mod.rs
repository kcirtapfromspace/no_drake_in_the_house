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
//! - Automatic offense creation from news detections

pub mod ingestion;
pub mod offense_creator;
pub mod orchestrator;
pub mod processing;
pub mod repository;

// Re-export main types
pub use ingestion::{
    FetchedArticle, NewsApiClient, NewsApiConfig, RedditConfig, RedditMonitor, RssFetcher,
    RssFetcherConfig, TwitterConfig, TwitterMonitor, WebScraper, WebScraperConfig,
};

pub use processing::{
    ArticleEmbedding, EmbeddingGenerator, EntityExtractor, EntityType, ExtractedEntity,
    OffenseClassification, OffenseClassifier,
};

pub use orchestrator::{
    NewsPipelineConfig, NewsPipelineOrchestrator, PipelineStats, ProcessedArticle,
    ScheduledPipelineHandle, ScheduledPipelineRunner,
};

pub use repository::{ArticleFilters, ArticleSummary, NewsRepository};

pub use offense_creator::{OffenseCreationResult, OffenseCreator};
