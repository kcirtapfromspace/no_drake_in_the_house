//! News Pipeline Module
//!
//! Live news tracking and processing for artist offense detection:
//! - RSS feed ingestion from music news sources
//! - NewsAPI integration for broader coverage
//! - Social media monitoring (Twitter, Reddit)
//! - Web scraping for additional sources
//! - Wikipedia and web search for proactive investigation
//! - Entity extraction (NER) for artist identification
//! - Offense classification (keyword-based)
//! - Vector embeddings for semantic search
//! - Automatic offense creation from news detections
//! - Single-pass artist researcher for deep investigation

pub mod autoresearch;
pub mod ingestion;
pub mod offense_creator;
pub mod orchestrator;
pub mod processing;
pub mod repository;

// Re-export main types
pub use ingestion::{
    FetchedArticle, NewsApiClient, NewsApiConfig, RedditConfig, RedditMonitor, RssFetcher,
    RssFetcherConfig, TwitterConfig, TwitterMonitor, WebScraper, WebScraperConfig,
    WebSearchClient, WebSearchConfig, WikipediaClient,
};

pub use processing::{
    ArticleEmbedding, BudgetGuard, ClaudeClient, ClaudeClientConfig, EmbeddingGenerator,
    EntityExtractor, EntityType, ExtractedEntity, HybridClassifier, HybridClassifierConfig,
    LlmBudgetConfig, OffenseCategory, OffenseClassification, OffenseClassifier,
    ResearchQualityScore, ResearchQualityScorer,
};

pub use orchestrator::{
    NewsPipelineConfig, NewsPipelineOrchestrator, PipelineStats, ProcessedArticle,
    ScheduledPipelineHandle, ScheduledPipelineRunner,
};

pub use repository::{ArticleFilters, ArticleSummary, NewsRepository};

pub use offense_creator::{OffenseCreationResult, OffenseCreator};

pub use autoresearch::{
    ArtistResearcher, ArtistResearcherConfig, ResearchResult,
    // Backward-compatible aliases
    AutoresearchAgent, AutoresearchConfig, AutoresearchResult,
};
