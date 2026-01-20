use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// External provider identifiers for an artist
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ExternalIds {
    pub spotify: Option<String>,
    pub apple: Option<String>,
    pub youtube: Option<String>,
    pub tidal: Option<String>,
    pub musicbrainz: Option<String>,
    pub isni: Option<String>,
}

impl ExternalIds {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_spotify(mut self, id: String) -> Self {
        self.spotify = Some(id);
        self
    }

    pub fn with_apple(mut self, id: String) -> Self {
        self.apple = Some(id);
        self
    }

    pub fn with_youtube(mut self, id: String) -> Self {
        self.youtube = Some(id);
        self
    }

    pub fn with_tidal(mut self, id: String) -> Self {
        self.tidal = Some(id);
        self
    }

    pub fn with_musicbrainz(mut self, id: String) -> Self {
        self.musicbrainz = Some(id);
        self
    }

    pub fn with_isni(mut self, id: String) -> Self {
        self.isni = Some(id);
        self
    }

    /// Check if any external ID is present
    pub fn has_any_id(&self) -> bool {
        self.spotify.is_some()
            || self.apple.is_some()
            || self.youtube.is_some()
            || self.tidal.is_some()
            || self.musicbrainz.is_some()
            || self.isni.is_some()
    }

    /// Get all non-empty IDs as a vector of (provider, id) tuples
    pub fn get_all_ids(&self) -> Vec<(String, String)> {
        let mut ids = Vec::new();

        if let Some(ref id) = self.spotify {
            ids.push(("spotify".to_string(), id.clone()));
        }
        if let Some(ref id) = self.apple {
            ids.push(("apple".to_string(), id.clone()));
        }
        if let Some(ref id) = self.youtube {
            ids.push(("youtube".to_string(), id.clone()));
        }
        if let Some(ref id) = self.tidal {
            ids.push(("tidal".to_string(), id.clone()));
        }
        if let Some(ref id) = self.musicbrainz {
            ids.push(("musicbrainz".to_string(), id.clone()));
        }
        if let Some(ref id) = self.isni {
            ids.push(("isni".to_string(), id.clone()));
        }

        ids
    }

    pub fn get_spotify_id(&self) -> Option<&str> {
        self.spotify.as_deref()
    }

    pub fn get_apple_id(&self) -> Option<&str> {
        self.apple.as_deref()
    }

    pub fn get_musicbrainz_id(&self) -> Option<&str> {
        self.musicbrainz.as_deref()
    }

    pub fn get_isni_id(&self) -> Option<&str> {
        self.isni.as_deref()
    }

    // Setter methods for tests
    pub fn set_spotify_id(&mut self, id: String) {
        self.spotify = Some(id);
    }

    pub fn set_apple_id(&mut self, id: String) {
        self.apple = Some(id);
    }

    pub fn set_musicbrainz_id(&mut self, id: String) {
        self.musicbrainz = Some(id);
    }

    pub fn set_isni_id(&mut self, id: String) {
        self.isni = Some(id);
    }
}

/// Artist alias with confidence scoring
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArtistAlias {
    pub name: String,
    pub source: String,
    pub confidence: f64, // 0.0 to 1.0
    #[serde(default)]
    pub locale: Option<String>,
}

impl ArtistAlias {
    pub fn new(name: String, source: String, confidence: f64) -> Self {
        Self {
            name,
            source,
            confidence: confidence.clamp(0.0, 1.0),
            locale: None,
        }
    }

    pub fn with_locale(mut self, locale: String) -> Self {
        self.locale = Some(locale);
        self
    }
}

/// Artist metadata including images, genres, and other attributes
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ArtistMetadata {
    pub image_url: Option<String>,
    pub genres: Vec<String>,
    pub popularity: Option<u32>, // 0-100
    pub follower_count: Option<u64>,
    pub verified: Option<bool>,
    pub country: Option<String>,
    pub formed_year: Option<u32>,
    pub isrc_codes: Vec<String>, // International Standard Recording Codes
    pub upc_codes: Vec<String>,  // Universal Product Codes
}

/// Core Artist entity with canonical identification and external mappings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Artist {
    pub id: Uuid,
    pub canonical_name: String,
    pub canonical_artist_id: Option<Uuid>, // Points to canonical version if this is an alias
    pub external_ids: ExternalIds,
    pub aliases: Vec<ArtistAlias>,
    pub metadata: ArtistMetadata,
    pub created_at: u64, // Unix timestamp
    pub updated_at: u64, // Unix timestamp
}

impl Artist {
    pub fn new(canonical_name: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            id: Uuid::new_v4(),
            canonical_name,
            canonical_artist_id: None,
            external_ids: ExternalIds::default(),
            aliases: Vec::new(),
            metadata: ArtistMetadata::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new artist with external IDs
    pub fn with_external_ids(canonical_name: String, external_ids: ExternalIds) -> Self {
        let mut artist = Self::new(canonical_name);
        artist.external_ids = external_ids;
        artist
    }

    /// Check if this artist is an alias (points to another canonical artist)
    pub fn is_alias(&self) -> bool {
        self.canonical_artist_id.is_some()
    }

    /// Get the canonical artist ID (self if not an alias, or the referenced canonical ID)
    pub fn get_canonical_id(&self) -> Uuid {
        self.canonical_artist_id.unwrap_or(self.id)
    }

    /// Add an alias to this artist
    pub fn add_alias(&mut self, alias: ArtistAlias) {
        // Avoid duplicate aliases
        if !self
            .aliases
            .iter()
            .any(|a| a.name == alias.name && a.source == alias.source)
        {
            self.aliases.push(alias);
            self.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
    }

    /// Get all names (canonical + aliases) for this artist
    pub fn get_all_names(&self) -> Vec<String> {
        let mut names = vec![self.canonical_name.clone()];
        names.extend(self.aliases.iter().map(|a| a.name.clone()));
        names
    }

    /// Find the best matching alias for a given name
    pub fn find_best_alias_match(&self, name: &str) -> Option<&ArtistAlias> {
        self.aliases
            .iter()
            .filter(|alias| {
                alias.name.to_lowercase().contains(&name.to_lowercase())
                    || name.to_lowercase().contains(&alias.name.to_lowercase())
            })
            .max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Update metadata while preserving timestamps
    pub fn update_metadata(&mut self, metadata: ArtistMetadata) {
        self.metadata = metadata;
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Merge external IDs from another artist
    pub fn merge_external_ids(&mut self, other_ids: &ExternalIds) {
        if self.external_ids.spotify.is_none() && other_ids.spotify.is_some() {
            self.external_ids.spotify = other_ids.spotify.clone();
        }
        if self.external_ids.apple.is_none() && other_ids.apple.is_some() {
            self.external_ids.apple = other_ids.apple.clone();
        }
        if self.external_ids.youtube.is_none() && other_ids.youtube.is_some() {
            self.external_ids.youtube = other_ids.youtube.clone();
        }
        if self.external_ids.tidal.is_none() && other_ids.tidal.is_some() {
            self.external_ids.tidal = other_ids.tidal.clone();
        }
        if self.external_ids.musicbrainz.is_none() && other_ids.musicbrainz.is_some() {
            self.external_ids.musicbrainz = other_ids.musicbrainz.clone();
        }
        if self.external_ids.isni.is_none() && other_ids.isni.is_some() {
            self.external_ids.isni = other_ids.isni.clone();
        }
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

/// Search query for artist resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistSearchQuery {
    pub query: String,
    pub provider: Option<String>,
    pub external_id: Option<String>,
    pub limit: Option<usize>,
}

impl ArtistSearchQuery {
    pub fn new(query: String) -> Self {
        Self {
            query,
            provider: None,
            external_id: None,
            limit: Some(10),
        }
    }

    pub fn with_provider(mut self, provider: String) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn with_external_id(mut self, external_id: String) -> Self {
        self.external_id = Some(external_id);
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Result of artist resolution with confidence scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistResolutionResult {
    pub artist: Artist,
    pub confidence: f64,
    pub match_type: MatchType,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MatchType {
    ExactName,
    ExactExternalId,
    FuzzyName,
    Alias,
    Canonical,
}

impl ArtistResolutionResult {
    pub fn new(artist: Artist, confidence: f64, match_type: MatchType, source: String) -> Self {
        Self {
            artist,
            confidence: confidence.clamp(0.0, 1.0),
            match_type,
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artist_creation() {
        let artist = Artist::new("Test Artist".to_string());
        assert_eq!(artist.canonical_name, "Test Artist");
        assert!(!artist.is_alias());
        assert_eq!(artist.get_canonical_id(), artist.id);
    }

    #[test]
    fn test_external_ids() {
        let external_ids = ExternalIds::new()
            .with_spotify("spotify123".to_string())
            .with_apple("apple456".to_string());

        assert_eq!(external_ids.spotify, Some("spotify123".to_string()));
        assert_eq!(external_ids.apple, Some("apple456".to_string()));
        assert!(external_ids.has_any_id());

        let all_ids = external_ids.get_all_ids();
        assert_eq!(all_ids.len(), 2);
        assert!(all_ids.contains(&("spotify".to_string(), "spotify123".to_string())));
        assert!(all_ids.contains(&("apple".to_string(), "apple456".to_string())));
    }

    #[test]
    fn test_artist_aliases() {
        let mut artist = Artist::new("The Beatles".to_string());

        let alias1 = ArtistAlias::new("Beatles".to_string(), "musicbrainz".to_string(), 0.9);
        let alias2 = ArtistAlias::new("Fab Four".to_string(), "wikipedia".to_string(), 0.7);

        artist.add_alias(alias1);
        artist.add_alias(alias2);

        assert_eq!(artist.aliases.len(), 2);

        let all_names = artist.get_all_names();
        assert_eq!(all_names.len(), 3);
        assert!(all_names.contains(&"The Beatles".to_string()));
        assert!(all_names.contains(&"Beatles".to_string()));
        assert!(all_names.contains(&"Fab Four".to_string()));

        let best_match = artist.find_best_alias_match("Beatles");
        assert!(best_match.is_some());
        assert_eq!(best_match.unwrap().confidence, 0.9);
    }

    #[test]
    fn test_merge_external_ids() {
        let mut artist = Artist::new("Test Artist".to_string());
        artist.external_ids.spotify = Some("spotify123".to_string());

        let other_ids = ExternalIds::new()
            .with_apple("apple456".to_string())
            .with_spotify("spotify789".to_string()); // Should not overwrite existing

        artist.merge_external_ids(&other_ids);

        assert_eq!(artist.external_ids.spotify, Some("spotify123".to_string())); // Unchanged
        assert_eq!(artist.external_ids.apple, Some("apple456".to_string())); // Added
    }

    #[test]
    fn test_artist_search_query() {
        let query = ArtistSearchQuery::new("Test Artist".to_string())
            .with_provider("spotify".to_string())
            .with_limit(5);

        assert_eq!(query.query, "Test Artist");
        assert_eq!(query.provider, Some("spotify".to_string()));
        assert_eq!(query.limit, Some(5));
    }

    #[test]
    fn test_artist_resolution_result() {
        let artist = Artist::new("Test Artist".to_string());
        let result =
            ArtistResolutionResult::new(artist, 0.95, MatchType::ExactName, "spotify".to_string());

        assert_eq!(result.confidence, 0.95);
        assert_eq!(result.match_type, MatchType::ExactName);
        assert_eq!(result.source, "spotify");
    }
}
