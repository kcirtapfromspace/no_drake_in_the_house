//! News Processing Module
//!
//! Processes fetched articles:
//! - Entity extraction (NER) for identifying artists
//! - Offense classification
//! - Embedding generation for semantic search

pub mod embedding_generator;
pub mod entity_extractor;
pub mod offense_classifier;

pub use embedding_generator::{ArticleEmbedding, EmbeddingConfig, EmbeddingGenerator};
pub use entity_extractor::{
    EntityExtractor, EntityExtractorConfig, EntityType, ExtractedEntity, KnownArtist,
};
pub use offense_classifier::{
    OffenseCategory, OffenseClassification, OffenseClassifier, OffenseClassifierConfig,
    OffenseSeverity,
};
