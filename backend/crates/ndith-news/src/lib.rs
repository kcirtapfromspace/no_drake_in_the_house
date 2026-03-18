//! ndith-news: LanceDB vector search, fastembed embeddings, and news pipeline.
//!
//! This crate isolates the heavyweight dependencies: lancedb, arrow, fastembed, scraper.

pub mod databases;
pub mod news_pipeline;

// Re-export database clients
pub use databases::LanceDbClient;

// Re-export news pipeline components
pub use news_pipeline::{
    ArticleEmbedding, EmbeddingGenerator, EntityExtractor, EntityType, ExtractedEntity,
    FetchedArticle, NewsApiClient, NewsApiConfig, NewsPipelineConfig, NewsPipelineOrchestrator,
    OffenseClassification, OffenseClassifier, PipelineStats, ProcessedArticle, RedditConfig,
    RedditMonitor, RssFetcher, RssFetcherConfig, ScheduledPipelineHandle, ScheduledPipelineRunner,
    TwitterConfig, TwitterMonitor, WebScraper, WebScraperConfig,
};
