//! Hybrid Classifier
//!
//! Wraps the keyword-based classifier with stats tracking.
//! LLM escalation has been removed — the keyword classifier covers 15 categories
//! with scoped negation and severity levels, which is sufficient and free.

use anyhow::Result;
use uuid::Uuid;

use super::entity_extractor::ExtractedEntity;
use super::offense_classifier::{
    OffenseClassification, OffenseClassifier, OffenseClassifierConfig,
};

/// Hybrid classifier configuration
#[derive(Debug, Clone)]
pub struct HybridClassifierConfig {
    /// Known artist names (kept for future extensibility)
    pub known_artist_names: Vec<String>,
}

impl Default for HybridClassifierConfig {
    fn default() -> Self {
        Self {
            known_artist_names: Vec::new(),
        }
    }
}

/// Classification statistics
#[derive(Debug, Clone, Default)]
pub struct ClassificationStats {
    pub total_articles: u64,
    pub articles_with_offenses: u64,
}

/// Classifier wrapping keyword-based classification with stats tracking.
pub struct HybridClassifier {
    keyword_classifier: OffenseClassifier,
    #[allow(dead_code)]
    config: HybridClassifierConfig,
    stats: tokio::sync::Mutex<ClassificationStats>,
}

impl HybridClassifier {
    /// Create a hybrid classifier (keyword-only)
    pub fn new(keyword_config: OffenseClassifierConfig, config: HybridClassifierConfig) -> Self {
        Self {
            keyword_classifier: OffenseClassifier::new(keyword_config),
            config,
            stats: tokio::sync::Mutex::new(ClassificationStats::default()),
        }
    }

    /// Get the underlying keyword classifier (for direct access when needed)
    pub fn keyword_classifier(&self) -> &OffenseClassifier {
        &self.keyword_classifier
    }

    /// Classify an article — direct pass-through to keyword classifier
    pub async fn classify(
        &self,
        article_id: Uuid,
        text: &str,
        title: Option<&str>,
        entities: &[ExtractedEntity],
    ) -> Result<Vec<OffenseClassification>> {
        let results = self
            .keyword_classifier
            .classify(article_id, text, title, entities)?;

        {
            let mut stats = self.stats.lock().await;
            stats.total_articles += 1;
            if !results.is_empty() {
                stats.articles_with_offenses += 1;
            }
        }

        Ok(results)
    }

    /// Get classification statistics
    pub async fn get_stats(&self) -> ClassificationStats {
        self.stats.lock().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = HybridClassifierConfig::default();
        assert!(config.known_artist_names.is_empty());
    }

    #[tokio::test]
    async fn test_keyword_only_classification() {
        let classifier = HybridClassifier::new(
            OffenseClassifierConfig::default(),
            HybridClassifierConfig::default(),
        );

        let article_id = Uuid::new_v4();
        let text = "The artist committed massive fraud and embezzlement.";
        let results = classifier
            .classify(article_id, text, None, &[])
            .await
            .unwrap();

        let stats = classifier.get_stats().await;
        assert_eq!(stats.total_articles, 1);
        assert!(!results.is_empty());
    }
}
