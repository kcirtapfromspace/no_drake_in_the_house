//! Cross-Platform Identity Resolver
//!
//! Resolves artist identities across multiple streaming platforms using:
//! - MusicBrainz IDs (canonical identifier)
//! - ISNI (International Standard Name Identifier)
//! - ISRC codes (track-level matching)
//! - Fuzzy name matching with genre/country context

use super::traits::*;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Confidence thresholds for identity matching
const AUTO_MERGE_THRESHOLD: f64 = 0.85;
const REVIEW_THRESHOLD: f64 = 0.70;

/// Cross-platform identity resolver
pub struct CrossPlatformIdentityResolver {
    client: Client,
    /// MusicBrainz API base URL
    musicbrainz_base: String,
    /// User agent for MusicBrainz (required)
    user_agent: String,
}

/// Canonical artist identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalArtist {
    /// Internal UUID
    pub id: Uuid,
    /// Canonical name
    pub name: String,
    /// MusicBrainz ID if known
    pub musicbrainz_id: Option<String>,
    /// ISNI if known
    pub isni: Option<String>,
    /// Known aliases
    pub aliases: Vec<String>,
    /// Genres
    pub genres: Vec<String>,
    /// Country of origin
    pub country: Option<String>,
    /// Platform-specific IDs
    pub platform_ids: HashMap<Platform, String>,
}

/// Result of identity resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityMatch {
    /// Matched canonical artist
    pub artist: CanonicalArtist,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Match method used
    pub method: MatchMethod,
    /// Whether this needs human review
    pub needs_review: bool,
}

/// Method used to match identity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MatchMethod {
    /// Exact MusicBrainz ID match
    MusicBrainzId,
    /// Exact ISNI match
    Isni,
    /// ISRC code correlation
    IsrcCorrelation,
    /// Fuzzy name match with context
    FuzzyName,
    /// Existing platform ID lookup
    ExistingMapping,
    /// No match found - new artist
    NewArtist,
}

/// MusicBrainz artist response
#[derive(Debug, Deserialize)]
struct MusicBrainzArtist {
    id: String,
    name: String,
    #[serde(rename = "sort-name")]
    sort_name: Option<String>,
    #[serde(default)]
    aliases: Vec<MusicBrainzAlias>,
    country: Option<String>,
    #[serde(default)]
    isnis: Vec<String>,
    #[serde(rename = "life-span")]
    life_span: Option<MusicBrainzLifeSpan>,
    #[serde(default)]
    tags: Vec<MusicBrainzTag>,
}

#[derive(Debug, Deserialize)]
struct MusicBrainzAlias {
    name: String,
    #[serde(rename = "sort-name")]
    sort_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MusicBrainzLifeSpan {
    begin: Option<String>,
    end: Option<String>,
    ended: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct MusicBrainzTag {
    name: String,
    count: i32,
}

#[derive(Debug, Deserialize)]
struct MusicBrainzSearchResponse {
    artists: Vec<MusicBrainzArtist>,
    count: u32,
}

impl CrossPlatformIdentityResolver {
    /// Create a new identity resolver
    pub fn new(app_name: &str, app_version: &str, contact: &str) -> Self {
        // MusicBrainz requires a specific user agent format
        let user_agent = format!("{}/{} ({})", app_name, app_version, contact);

        Self {
            client: Client::new(),
            musicbrainz_base: "https://musicbrainz.org/ws/2".to_string(),
            user_agent,
        }
    }

    /// Resolve a platform artist to a canonical identity
    pub async fn resolve(
        &self,
        platform_artist: &PlatformArtist,
        existing_artists: &[CanonicalArtist],
    ) -> Result<IdentityMatch> {
        // 1. Check for existing platform ID mapping
        if let Some(match_result) = self.check_existing_mapping(platform_artist, existing_artists) {
            return Ok(match_result);
        }

        // 2. Try MusicBrainz lookup
        if let Some(match_result) = self
            .lookup_musicbrainz(&platform_artist.name, &platform_artist.genres)
            .await?
        {
            // Check if MusicBrainz result matches an existing artist
            if let Some(existing) = existing_artists.iter().find(|a| {
                a.musicbrainz_id.as_ref()
                    == Some(
                        &match_result
                            .artist
                            .musicbrainz_id
                            .clone()
                            .unwrap_or_default(),
                    )
            }) {
                return Ok(IdentityMatch {
                    artist: existing.clone(),
                    confidence: match_result.confidence,
                    method: MatchMethod::MusicBrainzId,
                    needs_review: false,
                });
            }
            return Ok(match_result);
        }

        // 3. Fuzzy name matching against existing artists
        if let Some(match_result) = self.fuzzy_match(platform_artist, existing_artists) {
            return Ok(match_result);
        }

        // 4. No match found - create new canonical artist
        Ok(self.create_new_artist(platform_artist))
    }

    /// Check if platform ID is already mapped
    fn check_existing_mapping(
        &self,
        platform_artist: &PlatformArtist,
        existing_artists: &[CanonicalArtist],
    ) -> Option<IdentityMatch> {
        for artist in existing_artists {
            if let Some(existing_id) = artist.platform_ids.get(&platform_artist.platform) {
                if existing_id == &platform_artist.platform_id {
                    return Some(IdentityMatch {
                        artist: artist.clone(),
                        confidence: 1.0,
                        method: MatchMethod::ExistingMapping,
                        needs_review: false,
                    });
                }
            }
        }
        None
    }

    /// Look up artist on MusicBrainz
    async fn lookup_musicbrainz(
        &self,
        name: &str,
        genres: &[String],
    ) -> Result<Option<IdentityMatch>> {
        // Rate limit: 1 request per second for MusicBrainz
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let encoded_name = urlencoding::encode(name);
        let url = format!(
            "{}/artist/?query=artist:{}&fmt=json&limit=5",
            self.musicbrainz_base, encoded_name
        );

        let response = self
            .client
            .get(&url)
            .header("User-Agent", &self.user_agent)
            .send()
            .await
            .context("MusicBrainz API request failed")?;

        if !response.status().is_success() {
            tracing::warn!(
                "MusicBrainz search failed for '{}': {}",
                name,
                response.status()
            );
            return Ok(None);
        }

        let search_result: MusicBrainzSearchResponse = response
            .json()
            .await
            .context("Failed to parse MusicBrainz response")?;

        if search_result.artists.is_empty() {
            return Ok(None);
        }

        // Find best match
        let mut best_match: Option<(MusicBrainzArtist, f64)> = None;

        for mb_artist in search_result.artists {
            let score = self.score_musicbrainz_match(&mb_artist, name, genres);
            if score >= REVIEW_THRESHOLD {
                if best_match.is_none() || score > best_match.as_ref().unwrap().1 {
                    best_match = Some((mb_artist, score));
                }
            }
        }

        if let Some((mb_artist, score)) = best_match {
            let canonical = self.musicbrainz_to_canonical(&mb_artist);
            return Ok(Some(IdentityMatch {
                artist: canonical,
                confidence: score,
                method: MatchMethod::MusicBrainzId,
                needs_review: score < AUTO_MERGE_THRESHOLD,
            }));
        }

        Ok(None)
    }

    /// Score a MusicBrainz match
    fn score_musicbrainz_match(
        &self,
        mb_artist: &MusicBrainzArtist,
        search_name: &str,
        search_genres: &[String],
    ) -> f64 {
        let mut score = 0.0;

        // Name similarity (weight: 0.6)
        let name_sim = self.string_similarity(&mb_artist.name, search_name);
        score += name_sim * 0.6;

        // Check aliases for better name match
        for alias in &mb_artist.aliases {
            let alias_sim = self.string_similarity(&alias.name, search_name);
            if alias_sim > name_sim {
                score = score.max(alias_sim * 0.6);
            }
        }

        // Genre overlap (weight: 0.2)
        if !search_genres.is_empty() && !mb_artist.tags.is_empty() {
            let mb_genres: Vec<String> = mb_artist.tags.iter().map(|t| t.name.clone()).collect();
            let genre_overlap = self.genre_overlap(search_genres, &mb_genres);
            score += genre_overlap * 0.2;
        } else {
            // No genre info - neutral
            score += 0.1;
        }

        // Active status bonus (weight: 0.1)
        if let Some(life_span) = &mb_artist.life_span {
            if life_span.ended != Some(true) {
                score += 0.1;
            }
        } else {
            score += 0.05;
        }

        // Has ISNI bonus (weight: 0.1)
        if !mb_artist.isnis.is_empty() {
            score += 0.1;
        }

        score.min(1.0)
    }

    /// Convert MusicBrainz artist to canonical format
    fn musicbrainz_to_canonical(&self, mb_artist: &MusicBrainzArtist) -> CanonicalArtist {
        CanonicalArtist {
            id: Uuid::new_v4(),
            name: mb_artist.name.clone(),
            musicbrainz_id: Some(mb_artist.id.clone()),
            isni: mb_artist.isnis.first().cloned(),
            aliases: mb_artist.aliases.iter().map(|a| a.name.clone()).collect(),
            genres: mb_artist
                .tags
                .iter()
                .filter(|t| t.count > 0)
                .map(|t| t.name.clone())
                .collect(),
            country: mb_artist.country.clone(),
            platform_ids: HashMap::new(),
        }
    }

    /// Fuzzy match against existing artists
    fn fuzzy_match(
        &self,
        platform_artist: &PlatformArtist,
        existing_artists: &[CanonicalArtist],
    ) -> Option<IdentityMatch> {
        let mut best_match: Option<(&CanonicalArtist, f64)> = None;

        for existing in existing_artists {
            let score = self.score_artist_match(platform_artist, existing);
            if score >= REVIEW_THRESHOLD {
                if best_match.is_none() || score > best_match.as_ref().unwrap().1 {
                    best_match = Some((existing, score));
                }
            }
        }

        best_match.map(|(artist, score)| IdentityMatch {
            artist: artist.clone(),
            confidence: score,
            method: MatchMethod::FuzzyName,
            needs_review: score < AUTO_MERGE_THRESHOLD,
        })
    }

    /// Score a match between platform artist and canonical artist
    fn score_artist_match(
        &self,
        platform_artist: &PlatformArtist,
        canonical: &CanonicalArtist,
    ) -> f64 {
        let mut score = 0.0;

        // Name similarity (weight: 0.5)
        let mut best_name_sim = self.string_similarity(&platform_artist.name, &canonical.name);
        for alias in &canonical.aliases {
            let alias_sim = self.string_similarity(&platform_artist.name, alias);
            best_name_sim = best_name_sim.max(alias_sim);
        }
        score += best_name_sim * 0.5;

        // Genre overlap (weight: 0.3)
        if !platform_artist.genres.is_empty() && !canonical.genres.is_empty() {
            let canonical_genres: Vec<&str> = canonical.genres.iter().map(|s| s.as_str()).collect();
            let platform_genres: Vec<&str> =
                platform_artist.genres.iter().map(|s| s.as_str()).collect();
            let genre_overlap = self.genre_overlap(&platform_genres, &canonical_genres);
            score += genre_overlap * 0.3;
        } else {
            score += 0.15; // Neutral if no genre info
        }

        // Already has other platform mappings (weight: 0.2)
        if !canonical.platform_ids.is_empty() {
            score += 0.2;
        }

        score.min(1.0)
    }

    /// Create a new canonical artist from platform artist
    fn create_new_artist(&self, platform_artist: &PlatformArtist) -> IdentityMatch {
        let mut platform_ids = HashMap::new();
        platform_ids.insert(
            platform_artist.platform.clone(),
            platform_artist.platform_id.clone(),
        );

        let canonical = CanonicalArtist {
            id: Uuid::new_v4(),
            name: platform_artist.name.clone(),
            musicbrainz_id: None,
            isni: None,
            aliases: vec![],
            genres: platform_artist.genres.clone(),
            country: None,
            platform_ids,
        };

        IdentityMatch {
            artist: canonical,
            confidence: 1.0,
            method: MatchMethod::NewArtist,
            needs_review: false,
        }
    }

    /// Calculate string similarity using normalized Levenshtein distance
    fn string_similarity(&self, a: &str, b: &str) -> f64 {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();

        if a_lower == b_lower {
            return 1.0;
        }

        let distance = self.levenshtein_distance(&a_lower, &b_lower);
        let max_len = a_lower.len().max(b_lower.len());

        if max_len == 0 {
            return 1.0;
        }

        1.0 - (distance as f64 / max_len as f64)
    }

    /// Calculate Levenshtein distance
    fn levenshtein_distance(&self, a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let a_len = a_chars.len();
        let b_len = b_chars.len();

        if a_len == 0 {
            return b_len;
        }
        if b_len == 0 {
            return a_len;
        }

        let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

        for i in 0..=a_len {
            matrix[i][0] = i;
        }
        for j in 0..=b_len {
            matrix[0][j] = j;
        }

        for i in 1..=a_len {
            for j in 1..=b_len {
                let cost = if a_chars[i - 1] == b_chars[j - 1] {
                    0
                } else {
                    1
                };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[a_len][b_len]
    }

    /// Calculate genre overlap score
    fn genre_overlap<S: AsRef<str>>(&self, genres_a: &[S], genres_b: &[S]) -> f64 {
        if genres_a.is_empty() || genres_b.is_empty() {
            return 0.0;
        }

        let mut matches = 0;
        for genre_a in genres_a {
            for genre_b in genres_b {
                if self.genres_match(genre_a.as_ref(), genre_b.as_ref()) {
                    matches += 1;
                    break;
                }
            }
        }

        matches as f64 / genres_a.len().min(genres_b.len()) as f64
    }

    /// Check if two genre strings match (accounting for variations)
    fn genres_match(&self, a: &str, b: &str) -> bool {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();

        // Exact match
        if a_lower == b_lower {
            return true;
        }

        // One contains the other
        if a_lower.contains(&b_lower) || b_lower.contains(&a_lower) {
            return true;
        }

        // Common genre mappings
        let mappings = [
            ("hip-hop", "hip hop"),
            ("hip-hop", "rap"),
            ("hip hop", "rap"),
            ("r&b", "rnb"),
            ("r&b", "rhythm and blues"),
            ("rock", "rock and roll"),
            ("pop", "pop rock"),
            ("electronic", "edm"),
            ("electronic", "dance"),
        ];

        for (g1, g2) in &mappings {
            if (a_lower.contains(g1) && b_lower.contains(g2))
                || (a_lower.contains(g2) && b_lower.contains(g1))
            {
                return true;
            }
        }

        false
    }

    /// Resolve identities for multiple artists in batch
    pub async fn resolve_batch(
        &self,
        platform_artists: &[PlatformArtist],
        existing_artists: &[CanonicalArtist],
    ) -> Result<Vec<IdentityMatch>> {
        let mut results = Vec::with_capacity(platform_artists.len());

        for artist in platform_artists {
            let match_result = self.resolve(artist, existing_artists).await?;
            results.push(match_result);
        }

        Ok(results)
    }

    /// Merge two canonical artists (when confirmed as same)
    pub fn merge_artists(
        &self,
        primary: &CanonicalArtist,
        secondary: &CanonicalArtist,
    ) -> CanonicalArtist {
        let mut merged = primary.clone();

        // Keep MusicBrainz ID from either
        if merged.musicbrainz_id.is_none() {
            merged.musicbrainz_id = secondary.musicbrainz_id.clone();
        }

        // Keep ISNI from either
        if merged.isni.is_none() {
            merged.isni = secondary.isni.clone();
        }

        // Merge aliases (deduplicated)
        for alias in &secondary.aliases {
            if !merged.aliases.contains(alias) {
                merged.aliases.push(alias.clone());
            }
        }

        // Merge genres (deduplicated)
        for genre in &secondary.genres {
            if !merged.genres.contains(genre) {
                merged.genres.push(genre.clone());
            }
        }

        // Keep country from either
        if merged.country.is_none() {
            merged.country = secondary.country.clone();
        }

        // Merge platform IDs
        for (platform, id) in &secondary.platform_ids {
            merged
                .platform_ids
                .entry(platform.clone())
                .or_insert_with(|| id.clone());
        }

        merged
    }
}

/// Pending identity review item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityReviewItem {
    /// Review ID
    pub id: Uuid,
    /// Platform artist being resolved
    pub platform_artist: PlatformArtist,
    /// Proposed match
    pub proposed_match: IdentityMatch,
    /// Alternative candidates
    pub alternatives: Vec<IdentityMatch>,
    /// Status
    pub status: ReviewStatus,
    /// Created at
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Review status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReviewStatus {
    Pending,
    Approved,
    Rejected,
    MergedWithAlternative(Uuid),
    CreatedNew,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_similarity() {
        let resolver = CrossPlatformIdentityResolver::new("test", "1.0", "test@example.com");

        assert_eq!(resolver.string_similarity("Drake", "Drake"), 1.0);
        assert!(resolver.string_similarity("Drake", "drake") == 1.0);
        assert!(resolver.string_similarity("Drake", "Drakeo") > 0.7);
        assert!(resolver.string_similarity("Drake", "Kanye West") < 0.5);
    }

    #[test]
    fn test_genres_match() {
        let resolver = CrossPlatformIdentityResolver::new("test", "1.0", "test@example.com");

        assert!(resolver.genres_match("Hip-Hop", "hip hop"));
        assert!(resolver.genres_match("hip hop", "rap"));
        assert!(resolver.genres_match("R&B", "rnb"));
        assert!(!resolver.genres_match("rock", "jazz"));
    }

    #[test]
    fn test_genre_overlap() {
        let resolver = CrossPlatformIdentityResolver::new("test", "1.0", "test@example.com");

        let genres_a = vec!["hip hop", "rap", "r&b"];
        let genres_b = vec!["hip-hop", "soul", "r&b"];

        let overlap = resolver.genre_overlap(&genres_a, &genres_b);
        assert!(overlap > 0.5);
    }

    #[test]
    fn test_create_new_artist() {
        let resolver = CrossPlatformIdentityResolver::new("test", "1.0", "test@example.com");

        let platform_artist = PlatformArtist {
            platform_id: "123".to_string(),
            platform: Platform::Spotify,
            name: "Test Artist".to_string(),
            genres: vec!["pop".to_string()],
            popularity: Some(1000),
            image_url: None,
            external_urls: HashMap::new(),
            metadata: HashMap::new(),
        };

        let result = resolver.create_new_artist(&platform_artist);
        assert_eq!(result.method, MatchMethod::NewArtist);
        assert_eq!(result.artist.name, "Test Artist");
        assert!(result.artist.platform_ids.contains_key(&Platform::Spotify));
    }
}
