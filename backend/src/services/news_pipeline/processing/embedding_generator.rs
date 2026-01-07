//! Embedding Generator
//!
//! Generates vector embeddings for articles using fastembed.
//! Embeddings are used for semantic search and similarity matching.

use anyhow::{Context, Result};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Embedding generator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Model to use for embeddings
    pub model_name: String,
    /// Maximum text length (will be truncated)
    pub max_length: usize,
    /// Batch size for processing multiple texts
    pub batch_size: usize,
    /// Cache embeddings in memory
    pub enable_cache: bool,
    /// Maximum cache size
    pub max_cache_size: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model_name: "BAAI/bge-small-en-v1.5".to_string(),
            max_length: 512,
            batch_size: 32,
            enable_cache: true,
            max_cache_size: 10000,
        }
    }
}

/// An article embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleEmbedding {
    /// Article ID
    pub article_id: Uuid,
    /// Embedding vector
    pub embedding: Vec<f32>,
    /// Dimension of the embedding
    pub dimension: usize,
    /// Text that was embedded (truncated)
    pub text_preview: String,
    /// Model used
    pub model: String,
}

/// Embedding cache entry
struct CacheEntry {
    embedding: Vec<f32>,
    created_at: std::time::Instant,
}

/// Embedding generator
pub struct EmbeddingGenerator {
    config: EmbeddingConfig,
    model: Arc<RwLock<Option<TextEmbedding>>>,
    cache: Arc<RwLock<std::collections::HashMap<String, CacheEntry>>>,
    initialized: Arc<RwLock<bool>>,
}

impl EmbeddingGenerator {
    /// Create a new embedding generator
    pub fn new(config: EmbeddingConfig) -> Self {
        Self {
            config,
            model: Arc::new(RwLock::new(None)),
            cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            initialized: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize the embedding model (lazy loading)
    pub async fn initialize(&self) -> Result<()> {
        let mut initialized = self.initialized.write().await;
        if *initialized {
            return Ok(());
        }

        tracing::info!(model = %self.config.model_name, "Initializing embedding model");

        // Initialize the model with defaults
        let mut options = InitOptions::default();
        options.model_name = EmbeddingModel::BGESmallENV15;
        options.show_download_progress = true;

        let model = TextEmbedding::try_new(options)
            .context("Failed to initialize embedding model")?;

        let mut model_lock = self.model.write().await;
        *model_lock = Some(model);
        *initialized = true;

        tracing::info!("Embedding model initialized successfully");
        Ok(())
    }

    /// Generate embedding for a single text
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        self.initialize().await?;

        // Check cache
        if self.config.enable_cache {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(text) {
                return Ok(entry.embedding.clone());
            }
        }

        // Truncate text if needed
        let truncated = self.truncate_text(text);

        // Generate embedding
        let model = self.model.read().await;
        let model = model.as_ref().ok_or_else(|| anyhow::anyhow!("Model not initialized"))?;

        let embeddings = model
            .embed(vec![truncated.clone()], None)
            .context("Failed to generate embedding")?;

        let embedding = embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No embedding generated"))?;

        // Cache the result
        if self.config.enable_cache {
            let mut cache = self.cache.write().await;

            // Clean cache if too large
            if cache.len() >= self.config.max_cache_size {
                self.clean_cache(&mut cache);
            }

            cache.insert(text.to_string(), CacheEntry {
                embedding: embedding.clone(),
                created_at: std::time::Instant::now(),
            });
        }

        Ok(embedding)
    }

    /// Generate embeddings for multiple texts
    pub async fn embed_texts(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        self.initialize().await?;

        let model = self.model.read().await;
        let model = model.as_ref().ok_or_else(|| anyhow::anyhow!("Model not initialized"))?;

        let mut all_embeddings = Vec::with_capacity(texts.len());

        // Process in batches
        for batch in texts.chunks(self.config.batch_size) {
            let truncated: Vec<String> = batch
                .iter()
                .map(|t| self.truncate_text(t))
                .collect();

            let embeddings = model
                .embed(truncated, None)
                .context("Failed to generate batch embeddings")?;

            all_embeddings.extend(embeddings);
        }

        Ok(all_embeddings)
    }

    /// Generate embedding for an article
    pub async fn embed_article(
        &self,
        article_id: Uuid,
        title: &str,
        content: Option<&str>,
    ) -> Result<ArticleEmbedding> {
        // Combine title and content for embedding
        let text = if let Some(c) = content {
            format!("{}\n\n{}", title, c)
        } else {
            title.to_string()
        };

        let embedding = self.embed_text(&text).await?;
        let dimension = embedding.len();

        let text_preview = if text.len() > 100 {
            format!("{}...", &text[..100])
        } else {
            text.clone()
        };

        Ok(ArticleEmbedding {
            article_id,
            embedding,
            dimension,
            text_preview,
            model: self.config.model_name.clone(),
        })
    }

    /// Generate embeddings for multiple articles
    pub async fn embed_articles(
        &self,
        articles: Vec<(Uuid, String, Option<String>)>,
    ) -> Result<Vec<ArticleEmbedding>> {
        let texts: Vec<String> = articles
            .iter()
            .map(|(_, title, content)| {
                if let Some(c) = content {
                    format!("{}\n\n{}", title, c)
                } else {
                    title.clone()
                }
            })
            .collect();

        let embeddings = self.embed_texts(&texts).await?;

        let mut results = Vec::with_capacity(articles.len());

        for ((article_id, title, content), embedding) in articles.into_iter().zip(embeddings) {
            let text = if let Some(c) = &content {
                format!("{}\n\n{}", title, c)
            } else {
                title.clone()
            };

            let text_preview = if text.len() > 100 {
                format!("{}...", &text[..100])
            } else {
                text
            };

            results.push(ArticleEmbedding {
                article_id,
                embedding,
                dimension: 384, // BGE-small dimension
                text_preview,
                model: self.config.model_name.clone(),
            });
        }

        Ok(results)
    }

    /// Calculate cosine similarity between two embeddings
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    /// Find most similar embeddings
    pub fn find_similar(
        &self,
        query: &[f32],
        candidates: &[ArticleEmbedding],
        top_k: usize,
    ) -> Vec<(Uuid, f32)> {
        let mut similarities: Vec<(Uuid, f32)> = candidates
            .iter()
            .map(|c| (c.article_id, Self::cosine_similarity(query, &c.embedding)))
            .collect();

        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        similarities.truncate(top_k);

        similarities
    }

    /// Truncate text to max length
    fn truncate_text(&self, text: &str) -> String {
        if text.len() <= self.config.max_length {
            text.to_string()
        } else {
            // Truncate at word boundary
            let truncated = &text[..self.config.max_length];
            if let Some(last_space) = truncated.rfind(' ') {
                truncated[..last_space].to_string()
            } else {
                truncated.to_string()
            }
        }
    }

    /// Clean old cache entries
    fn clean_cache(&self, cache: &mut std::collections::HashMap<String, CacheEntry>) {
        let cutoff = std::time::Instant::now() - std::time::Duration::from_secs(3600);

        cache.retain(|_, entry| entry.created_at > cutoff);

        // If still too large, remove oldest entries
        if cache.len() >= self.config.max_cache_size {
            let mut entries: Vec<_> = cache.iter()
                .map(|(k, e)| (k.clone(), e.created_at))
                .collect();
            entries.sort_by_key(|(_, created_at)| *created_at);

            let to_remove = cache.len() - (self.config.max_cache_size / 2);
            let keys_to_remove: Vec<String> = entries.into_iter()
                .take(to_remove)
                .map(|(k, _)| k)
                .collect();

            for key in keys_to_remove {
                cache.remove(&key);
            }
        }
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.read().await;
        (cache.len(), self.config.max_cache_size)
    }

    /// Clear the cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get embedding dimension
    pub fn dimension(&self) -> usize {
        384 // BGE-small-en-v1.5 dimension
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EmbeddingConfig::default();
        assert_eq!(config.max_length, 512);
        assert_eq!(config.batch_size, 32);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((EmbeddingGenerator::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((EmbeddingGenerator::cosine_similarity(&a, &c) - 0.0).abs() < 0.001);

        let d = vec![0.707, 0.707, 0.0];
        assert!((EmbeddingGenerator::cosine_similarity(&a, &d) - 0.707).abs() < 0.01);
    }

    #[test]
    fn test_truncate_text() {
        let config = EmbeddingConfig {
            max_length: 20,
            ..Default::default()
        };
        let generator = EmbeddingGenerator::new(config);

        let short = "Hello world";
        assert_eq!(generator.truncate_text(short), short);

        let long = "This is a very long text that should be truncated";
        let truncated = generator.truncate_text(long);
        assert!(truncated.len() <= 20);
    }
}
