//! News Processing Module
//!
//! Processes fetched articles:
//! - Entity extraction (NER) for identifying artists
//! - Offense classification (keyword-based)
//! - Embedding generation for semantic search
//! - Research quality scoring

pub mod embedding_generator;
pub mod entity_extractor;
pub mod hybrid_classifier;
pub mod llm_client;
pub mod offense_classifier;
pub mod research_quality;

pub use embedding_generator::{ArticleEmbedding, EmbeddingConfig, EmbeddingGenerator};
pub use entity_extractor::{
    EntityExtractor, EntityExtractorConfig, EntityType, ExtractedEntity, KnownArtist,
};
pub use hybrid_classifier::{HybridClassifier, HybridClassifierConfig};
pub use offense_classifier::{
    OffenseCategory, OffenseClassification, OffenseClassifier, OffenseClassifierConfig,
    OffenseSeverity,
};
pub use research_quality::{ResearchQualityScore, ResearchQualityScorer};

// LLM client types kept available but not prominently re-exported.
// The module is still accessible as `processing::llm_client::*` for anyone who needs it,
// but we no longer advertise ClaudeClient/LlmClassification/LlmEntity at this level.
pub use llm_client::{BudgetGuard, ClaudeClient, ClaudeClientConfig, LlmBudgetConfig};
