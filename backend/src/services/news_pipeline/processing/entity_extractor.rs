//! Entity Extractor
//!
//! Extracts entities (artists, labels, venues) from article text.
//! Uses pattern matching and fuzzy matching against known artist database.

use anyhow::Result;
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Entity types that can be extracted
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EntityType {
    Artist,
    Band,
    Label,
    Venue,
    Event,
    Song,
    Album,
}

/// An extracted entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    /// Unique identifier
    pub id: Uuid,
    /// Entity name as found in text
    pub name: String,
    /// Normalized/canonical name
    pub normalized_name: Option<String>,
    /// Entity type
    pub entity_type: EntityType,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Position in text (start, end)
    pub position: (usize, usize),
    /// Context snippet around the entity
    pub context: String,
    /// Matched artist ID if resolved
    pub artist_id: Option<Uuid>,
}

/// Entity extractor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityExtractorConfig {
    /// Minimum confidence threshold
    pub min_confidence: f64,
    /// Context window size (characters before/after)
    pub context_window: usize,
    /// Maximum entities to extract per article
    pub max_entities: usize,
}

impl Default for EntityExtractorConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
            context_window: 100,
            max_entities: 50,
        }
    }
}

/// Known artist entry for matching
#[derive(Debug, Clone)]
pub struct KnownArtist {
    pub id: Uuid,
    pub name: String,
    pub aliases: Vec<String>,
    pub genres: Vec<String>,
}

/// Entity extractor
pub struct EntityExtractor {
    config: EntityExtractorConfig,
    /// Known artists for matching
    known_artists: Arc<RwLock<HashMap<String, KnownArtist>>>,
    /// Artist name patterns (regex)
    name_patterns: Vec<Regex>,
    /// Common title prefixes to strip
    title_prefixes: HashSet<String>,
    /// Common title suffixes to strip
    title_suffixes: HashSet<String>,
}

impl EntityExtractor {
    /// Create a new entity extractor
    pub fn new(config: EntityExtractorConfig) -> Self {
        let name_patterns = vec![
            // "Artist Name" announced/released/performed/etc.
            Regex::new(r#"(?i)([A-Z][a-z]+(?:\s+[A-Z][a-z]+)*)\s+(?:announced|released|performed|revealed|confirmed|denied|apologized|addressed)"#).unwrap(),
            // Rapper/Singer/Artist Name
            Regex::new(r#"(?i)(?:rapper|singer|artist|musician|producer|dj)\s+([A-Z][a-z]+(?:\s+[A-Z][a-z]+)*)"#).unwrap(),
            // Name's new album/song/tour
            Regex::new(r#"([A-Z][a-z]+(?:\s+[A-Z][a-z]+)*)'s\s+(?:new|latest|upcoming)"#).unwrap(),
            // Featuring Name
            Regex::new(r#"(?i)(?:featuring|feat\.?|ft\.?|with)\s+([A-Z][a-z]+(?:\s+[A-Z][a-z]+)*)"#).unwrap(),
        ];

        let title_prefixes: HashSet<String> = [
            "The", "DJ", "MC", "Lil", "Young", "Big", "King", "Queen",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        let title_suffixes: HashSet<String> = [
            "Jr", "Jr.", "Sr", "Sr.", "III", "II", "IV",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        Self {
            config,
            known_artists: Arc::new(RwLock::new(HashMap::new())),
            name_patterns,
            title_prefixes,
            title_suffixes,
        }
    }

    /// Add a known artist for matching
    pub async fn add_known_artist(&self, artist: KnownArtist) {
        let mut artists = self.known_artists.write().await;

        // Add main name
        let normalized = self.normalize_name(&artist.name);
        artists.insert(normalized.clone(), artist.clone());

        // Add aliases
        for alias in &artist.aliases {
            let normalized_alias = self.normalize_name(alias);
            artists.insert(normalized_alias, artist.clone());
        }
    }

    /// Add multiple known artists
    pub async fn add_known_artists(&self, artists: Vec<KnownArtist>) {
        for artist in artists {
            self.add_known_artist(artist).await;
        }
    }

    /// Normalize an artist name for matching
    fn normalize_name(&self, name: &str) -> String {
        name.to_lowercase()
            .trim()
            .replace(['\'', '"', '.', ','], "")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Extract entities from text
    pub async fn extract(&self, text: &str, title: Option<&str>) -> Result<Vec<ExtractedEntity>> {
        let mut entities = Vec::new();
        let mut seen_names: HashSet<String> = HashSet::new();

        // Combine title and text for extraction
        let full_text = if let Some(t) = title {
            format!("{}\n\n{}", t, text)
        } else {
            text.to_string()
        };

        // Extract using patterns
        for pattern in &self.name_patterns {
            for capture in pattern.captures_iter(&full_text) {
                if let Some(name_match) = capture.get(1) {
                    let name = name_match.as_str().trim();
                    let normalized = self.normalize_name(name);

                    // Skip if already seen
                    if seen_names.contains(&normalized) {
                        continue;
                    }
                    seen_names.insert(normalized.clone());

                    // Skip common words
                    if self.is_common_word(name) {
                        continue;
                    }

                    let (confidence, artist_id, entity_type) = self.match_entity(name).await;

                    if confidence >= self.config.min_confidence {
                        let start = name_match.start();
                        let end = name_match.end();
                        let context = self.extract_context(&full_text, start, end);

                        entities.push(ExtractedEntity {
                            id: Uuid::new_v4(),
                            name: name.to_string(),
                            normalized_name: Some(normalized),
                            entity_type,
                            confidence,
                            position: (start, end),
                            context,
                            artist_id,
                        });
                    }
                }
            }
        }

        // Also try direct matching against known artists
        let known_artists = self.known_artists.read().await;
        for (normalized_name, artist) in known_artists.iter() {
            if seen_names.contains(normalized_name) {
                continue;
            }

            // Check if artist name appears in text
            let search_terms = vec![
                artist.name.clone(),
                artist.name.to_uppercase(),
                artist.name.to_lowercase(),
            ];

            for term in search_terms {
                if let Some(pos) = full_text.find(&term) {
                    seen_names.insert(normalized_name.clone());

                    let context = self.extract_context(&full_text, pos, pos + term.len());

                    entities.push(ExtractedEntity {
                        id: Uuid::new_v4(),
                        name: term.clone(),
                        normalized_name: Some(normalized_name.clone()),
                        entity_type: EntityType::Artist,
                        confidence: 0.9,
                        position: (pos, pos + term.len()),
                        context,
                        artist_id: Some(artist.id),
                    });
                    break;
                }
            }
        }

        // Sort by confidence and limit
        entities.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        entities.truncate(self.config.max_entities);

        Ok(entities)
    }

    /// Match an entity name against known artists
    async fn match_entity(&self, name: &str) -> (f64, Option<Uuid>, EntityType) {
        let normalized = self.normalize_name(name);
        let known_artists = self.known_artists.read().await;

        // Exact match
        if let Some(artist) = known_artists.get(&normalized) {
            return (0.95, Some(artist.id), EntityType::Artist);
        }

        // Fuzzy match
        let mut best_match: Option<(f64, &KnownArtist)> = None;

        for (known_name, artist) in known_artists.iter() {
            let similarity = self.string_similarity(&normalized, known_name);
            if similarity >= 0.85 {
                if best_match.is_none() || similarity > best_match.unwrap().0 {
                    best_match = Some((similarity, artist));
                }
            }
        }

        if let Some((similarity, artist)) = best_match {
            return (similarity * 0.9, Some(artist.id), EntityType::Artist);
        }

        // Unknown entity - estimate based on patterns
        let confidence = self.estimate_entity_confidence(name);
        let entity_type = self.guess_entity_type(name);

        (confidence, None, entity_type)
    }

    /// Estimate confidence for unknown entity
    fn estimate_entity_confidence(&self, name: &str) -> f64 {
        let mut confidence: f64 = 0.5;

        // Multi-word names are more likely to be real entities
        let word_count = name.split_whitespace().count();
        if word_count >= 2 && word_count <= 4 {
            confidence += 0.1;
        }

        // Names with common artist prefixes
        if self.title_prefixes.iter().any(|p| name.starts_with(p)) {
            confidence += 0.1;
        }

        // Proper capitalization pattern
        let words: Vec<&str> = name.split_whitespace().collect();
        if words.iter().all(|w| {
            w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
        }) {
            confidence += 0.1;
        }

        confidence.min(0.8)
    }

    /// Guess entity type from name patterns
    fn guess_entity_type(&self, name: &str) -> EntityType {
        let lower = name.to_lowercase();

        if lower.contains("records") || lower.contains("entertainment") || lower.contains("music group") {
            EntityType::Label
        } else if lower.contains("arena") || lower.contains("stadium") || lower.contains("theater") {
            EntityType::Venue
        } else if lower.contains("festival") || lower.contains("tour") {
            EntityType::Event
        } else if lower.contains("the ") && !self.title_prefixes.contains(&"The".to_string()) {
            EntityType::Band
        } else {
            EntityType::Artist
        }
    }

    /// Check if a word is a common word (not an entity)
    fn is_common_word(&self, word: &str) -> bool {
        let common_words: HashSet<&str> = [
            "The", "This", "That", "These", "Those", "Their", "There",
            "When", "Where", "What", "Which", "While", "After", "Before",
            "Music", "Album", "Song", "Track", "Video", "News", "Report",
            "According", "Sources", "Statement", "Today", "Yesterday",
        ]
        .iter()
        .copied()
        .collect();

        common_words.contains(word) || word.len() < 2
    }

    /// Extract context around a position
    fn extract_context(&self, text: &str, start: usize, end: usize) -> String {
        let context_start = start.saturating_sub(self.config.context_window);
        let context_end = (end + self.config.context_window).min(text.len());

        // Find word boundaries
        let context_start = text[..context_start]
            .rfind(' ')
            .map(|p| p + 1)
            .unwrap_or(context_start);
        let context_end = text[context_end..]
            .find(' ')
            .map(|p| context_end + p)
            .unwrap_or(context_end);

        text[context_start..context_end].to_string()
    }

    /// Calculate string similarity (Levenshtein-based)
    fn string_similarity(&self, a: &str, b: &str) -> f64 {
        if a == b {
            return 1.0;
        }

        let distance = levenshtein::levenshtein(a, b);
        let max_len = a.len().max(b.len());

        if max_len == 0 {
            return 1.0;
        }

        1.0 - (distance as f64 / max_len as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_name() {
        let config = EntityExtractorConfig::default();
        let extractor = EntityExtractor::new(config);

        assert_eq!(extractor.normalize_name("Drake"), "drake");
        assert_eq!(extractor.normalize_name("Kanye West"), "kanye west");
        assert_eq!(extractor.normalize_name("The Weeknd"), "the weeknd");
    }

    #[test]
    fn test_is_common_word() {
        let config = EntityExtractorConfig::default();
        let extractor = EntityExtractor::new(config);

        assert!(extractor.is_common_word("The"));
        assert!(extractor.is_common_word("Music"));
        assert!(!extractor.is_common_word("Drake"));
    }

    #[tokio::test]
    async fn test_add_known_artist() {
        let config = EntityExtractorConfig::default();
        let extractor = EntityExtractor::new(config);

        let artist = KnownArtist {
            id: Uuid::new_v4(),
            name: "Kanye West".to_string(),
            aliases: vec!["Ye".to_string(), "Yeezy".to_string()],
            genres: vec!["hip hop".to_string()],
        };

        extractor.add_known_artist(artist).await;

        let artists = extractor.known_artists.read().await;
        assert!(artists.contains_key("kanye west"));
        assert!(artists.contains_key("ye"));
        assert!(artists.contains_key("yeezy"));
    }
}
