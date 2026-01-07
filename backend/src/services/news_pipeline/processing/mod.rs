//! News Processing Module
//!
//! Processes fetched articles:
//! - Entity extraction (NER) for identifying artists
//! - Offense classification
//! - Embedding generation for semantic search

pub mod entity_extractor;
pub mod offense_classifier;
pub mod embedding_generator;

pub use entity_extractor::{EntityExtractor, EntityExtractorConfig, ExtractedEntity, EntityType, KnownArtist};
pub use offense_classifier::{OffenseClassifier, OffenseClassifierConfig, OffenseClassification, OffenseCategory, OffenseSeverity};
pub use embedding_generator::{EmbeddingGenerator, EmbeddingConfig, ArticleEmbedding};
