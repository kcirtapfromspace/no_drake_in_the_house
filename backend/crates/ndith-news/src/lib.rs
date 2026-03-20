//! ndith-news: LanceDB vector search, fastembed embeddings, and news pipeline.
//!
//! This crate isolates the heavyweight dependencies: lancedb, arrow, fastembed, scraper.

pub mod databases;
pub mod news_pipeline;

// Re-export database clients
pub use databases::LanceDbClient;

// Re-export news pipeline components
pub use news_pipeline::{
    // Processing
    ArticleEmbedding,
    // Autoresearch (renamed)
    ArtistResearcher,
    ArtistResearcherConfig,
    // Backward-compatible aliases
    AutoresearchAgent,
    AutoresearchConfig,
    AutoresearchResult,
    // Budget controls
    BudgetGuard,
    ClaudeClient,
    ClaudeClientConfig,
    EmbeddingGenerator,
    EntityExtractor,
    EntityType,
    ExtractedEntity,
    // Ingestion
    FetchedArticle,
    HybridClassifier,
    HybridClassifierConfig,
    LlmBudgetConfig,
    NewsApiClient,
    NewsApiConfig,
    // Orchestration
    NewsPipelineConfig,
    NewsPipelineOrchestrator,
    OffenseCategory,
    OffenseClassification,
    OffenseClassifier,
    PipelineStats,
    ProcessedArticle,
    RedditConfig,
    RedditMonitor,
    ResearchQualityScore,
    ResearchQualityScorer,
    ResearchResult,
    RssFetcher,
    RssFetcherConfig,
    ScheduledPipelineHandle,
    ScheduledPipelineRunner,
    TwitterConfig,
    TwitterMonitor,
    WebScraper,
    WebScraperConfig,
    WebSearchClient,
    WebSearchConfig,
    WikipediaClient,
};
