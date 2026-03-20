//! ndith-news: LanceDB vector search, fastembed embeddings, and news pipeline.
//!
//! This crate isolates the heavyweight dependencies: lancedb, arrow, fastembed, scraper.

pub mod databases;
pub mod news_pipeline;

// Re-export database clients
pub use databases::LanceDbClient;

// Re-export news pipeline components
pub use news_pipeline::{
    // Autoresearch (renamed)
    ArtistResearcher, ArtistResearcherConfig, ResearchResult,
    // Backward-compatible aliases
    AutoresearchAgent, AutoresearchConfig, AutoresearchResult,
    // Budget controls
    BudgetGuard, ClaudeClient, ClaudeClientConfig, LlmBudgetConfig,
    // Ingestion
    FetchedArticle, NewsApiClient, NewsApiConfig, RedditConfig, RedditMonitor, RssFetcher,
    RssFetcherConfig, TwitterConfig, TwitterMonitor, WebSearchClient, WebSearchConfig, WebScraper,
    WebScraperConfig, WikipediaClient,
    // Processing
    ArticleEmbedding, EmbeddingGenerator, EntityExtractor, EntityType, ExtractedEntity,
    HybridClassifier, HybridClassifierConfig, OffenseCategory, OffenseClassification,
    OffenseClassifier, ResearchQualityScore, ResearchQualityScorer,
    // Orchestration
    NewsPipelineConfig, NewsPipelineOrchestrator, PipelineStats, ProcessedArticle,
    ScheduledPipelineHandle, ScheduledPipelineRunner,
};
