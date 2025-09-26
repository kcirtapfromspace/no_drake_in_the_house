use crate::models::{Artist, ArtistAlias, ArtistSearchQuery, ArtistResolutionResult, MatchType};
use crate::services::ExternalApiService;
use dashmap::DashMap;
use futures::future::join_all;
use std::sync::Arc;
use uuid::Uuid;
use anyhow::{Result, anyhow};

/// High-performance entity resolution service with concurrent processing
#[derive(Clone)]
pub struct EntityResolutionService {
    /// In-memory cache for fast lookups
    artist_cache: Arc<DashMap<String, Artist>>,
    /// External ID to artist ID mapping
    external_id_cache: Arc<DashMap<String, Uuid>>,
    /// Name to artist ID mapping for fuzzy matching
    name_cache: Arc<DashMap<String, Vec<Uuid>>>,
    /// Alias confidence threshold
    confidence_threshold: f64,
    /// External API service for MusicBrainz and ISNI integration
    pub external_api_service: ExternalApiService,
}

impl EntityResolutionService {
    pub fn new() -> Self {
        Self {
            artist_cache: Arc::new(DashMap::new()),
            external_id_cache: Arc::new(DashMap::new()),
            name_cache: Arc::new(DashMap::new()),
            confidence_threshold: 0.7, // Default confidence threshold
            external_api_service: ExternalApiService::new(),
        }
    }

    pub fn with_confidence_threshold(mut self, threshold: f64) -> Self {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Add an artist to the resolution cache
    pub async fn add_artist(&self, artist: Artist) -> Result<()> {
        let artist_id = artist.id;
        let canonical_name_key = self.normalize_name(&artist.canonical_name);
        
        // Cache by ID
        self.artist_cache.insert(artist_id.to_string(), artist.clone());
        
        // Cache by external IDs
        for (provider, external_id) in artist.external_ids.get_all_ids() {
            let cache_key = format!("{}:{}", provider, external_id);
            self.external_id_cache.insert(cache_key, artist_id);
        }
        
        // Cache by canonical name
        self.name_cache
            .entry(canonical_name_key.clone())
            .or_insert_with(Vec::new)
            .push(artist_id);
        
        // Cache by aliases
        for alias in &artist.aliases {
            let alias_key = self.normalize_name(&alias.name);
            if alias_key != canonical_name_key {
                self.name_cache
                    .entry(alias_key)
                    .or_insert_with(Vec::new)
                    .push(artist_id);
            }
        }
        
        println!("Added artist to cache: {} ({})", artist.canonical_name, artist_id);
        Ok(())
    }

    /// Get an artist by ID from the cache
    pub async fn get_artist_by_id(&self, artist_id: Uuid) -> Result<Option<Artist>> {
        if let Some(artist) = self.artist_cache.get(&artist_id.to_string()) {
            Ok(Some(artist.clone()))
        } else {
            Ok(None)
        }
    }

    /// Resolve multiple artists concurrently
    pub async fn resolve_concurrent(&self, queries: &[ArtistSearchQuery]) -> Result<Vec<Vec<ArtistResolutionResult>>> {
        let tasks: Vec<_> = queries
            .iter()
            .map(|query| {
                let service = self.clone();
                let query = query.clone();
                tokio::spawn(async move {
                    service.resolve_artist(&query).await
                })
            })
            .collect();

        let results = join_all(tasks).await;
        
        let mut resolved_results = Vec::new();
        for result in results {
            match result {
                Ok(Ok(artist_results)) => resolved_results.push(artist_results),
                Ok(Err(e)) => {
                    println!("Artist resolution failed: {}", e);
                    resolved_results.push(Vec::new());
                }
                Err(e) => {
                    println!("Task join failed: {}", e);
                    resolved_results.push(Vec::new());
                }
            }
        }
        
        Ok(resolved_results)
    }

    /// Resolve a single artist with disambiguation
    pub async fn resolve_artist(&self, query: &ArtistSearchQuery) -> Result<Vec<ArtistResolutionResult>> {
        let mut results = Vec::new();
        
        // 1. Try exact external ID match first (highest confidence)
        if let Some(ref external_id) = query.external_id {
            if let Some(ref provider) = query.provider {
                let cache_key = format!("{}:{}", provider, external_id);
                if let Some(artist_id) = self.external_id_cache.get(&cache_key) {
                    if let Some(artist) = self.artist_cache.get(&artist_id.to_string()) {
                        results.push(ArtistResolutionResult::new(
                            artist.clone(),
                            1.0,
                            MatchType::ExactExternalId,
                            provider.clone(),
                        ));
                        return Ok(results);
                    }
                }
            }
        }
        
        // 2. Try exact name match
        let normalized_name = self.normalize_name(&query.name);
        if let Some(artist_ids) = self.name_cache.get(&normalized_name) {
            for artist_id in artist_ids.iter() {
                if let Some(artist) = self.artist_cache.get(&artist_id.to_string()) {
                    let confidence = if artist.canonical_name.to_lowercase() == query.name.to_lowercase() {
                        0.95
                    } else {
                        0.85 // Alias match
                    };
                    
                    results.push(ArtistResolutionResult::new(
                        artist.clone(),
                        confidence,
                        MatchType::ExactName,
                        "cache".to_string(),
                    ));
                }
            }
        }
        
        // 3. Try fuzzy name matching with ML-based confidence scoring
        if results.is_empty() {
            results.extend(self.fuzzy_name_search(&query.name, query.limit.unwrap_or(10)).await?);
        }

        // 4. If still no results, search external APIs (MusicBrainz, ISNI)
        if results.is_empty() {
            results.extend(self.search_external_apis(&query.name, query.limit.unwrap_or(10)).await?);
        }
        
        // 5. Sort by confidence and apply limit
        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }
        
        // Filter by confidence threshold
        results.retain(|r| r.confidence >= self.confidence_threshold);
        
        Ok(results)
    }

    /// Fuzzy name search with ML-based confidence scoring
    async fn fuzzy_name_search(&self, query_name: &str, limit: usize) -> Result<Vec<ArtistResolutionResult>> {
        let mut results = Vec::new();
        let query_normalized = self.normalize_name(query_name);
        
        // Parallel processing of all cached artists
        let tasks: Vec<_> = self.artist_cache
            .iter()
            .map(|entry| {
                let artist = entry.value().clone();
                let query_name = query_name.to_string();
                let query_normalized = query_normalized.clone();
                
                tokio::spawn(async move {
                    Self::calculate_artist_similarity(&artist, &query_name, &query_normalized)
                })
            })
            .collect();
        
        let similarity_results = join_all(tasks).await;
        
        for result in similarity_results {
            if let Ok(Some((artist, confidence))) = result {
                if confidence >= 0.5 { // Minimum fuzzy match threshold
                    results.push(ArtistResolutionResult::new(
                        artist,
                        confidence,
                        MatchType::FuzzyName,
                        "fuzzy_match".to_string(),
                    ));
                }
            }
        }
        
        // Sort by confidence and limit results
        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        
        Ok(results)
    }

    /// Search external APIs (MusicBrainz, ISNI) with circuit breaker fallback
    async fn search_external_apis(&self, query_name: &str, limit: usize) -> Result<Vec<ArtistResolutionResult>> {
        let mut results = Vec::new();

        match self.external_api_service.search_artists_with_fallback(query_name, Some(limit as u32)).await {
            Ok(external_artists) => {
                for mut artist in external_artists {
                    // Enrich the artist with additional data
                    if let Err(e) = self.external_api_service.enrich_artist(&mut artist).await {
                        println!("Failed to enrich artist {}: {}", artist.canonical_name, e);
                    }

                    // Add to cache for future lookups
                    if let Err(e) = self.add_artist(artist.clone()).await {
                        println!("Failed to cache external artist {}: {}", artist.canonical_name, e);
                    }

                    // Calculate confidence based on name similarity
                    let confidence = Self::calculate_name_similarity(&artist.canonical_name, query_name);
                    let confidence = confidence.max(0.6); // Minimum confidence for external API results

                    results.push(ArtistResolutionResult::new(
                        artist,
                        confidence,
                        MatchType::ExactName, // External APIs typically return exact matches
                        "external_api".to_string(),
                    ));
                }
            }
            Err(e) => {
                println!("External API search failed for '{}': {}", query_name, e);
            }
        }

        Ok(results)
    }

    /// Calculate similarity between an artist and query using ML-based scoring
    fn calculate_artist_similarity(
        artist: &Artist,
        query_name: &str,
        query_normalized: &str,
    ) -> Option<(Artist, f64)> {
        let mut max_confidence: f64 = 0.0;
        
        // Check canonical name
        let canonical_confidence = Self::calculate_name_similarity(&artist.canonical_name, query_name);
        max_confidence = max_confidence.max(canonical_confidence);
        
        // Check aliases with their confidence scores
        for alias in &artist.aliases {
            let alias_similarity = Self::calculate_name_similarity(&alias.name, query_name);
            // Weight alias similarity by the alias confidence
            let weighted_similarity = alias_similarity * alias.confidence;
            max_confidence = max_confidence.max(weighted_similarity);
        }
        
        if max_confidence > 0.0 {
            Some((artist.clone(), max_confidence))
        } else {
            None
        }
    }

    /// Calculate similarity between two names using multiple algorithms
    fn calculate_name_similarity(name1: &str, name2: &str) -> f64 {
        let name1_norm = name1.to_lowercase();
        let name2_norm = name2.to_lowercase();
        
        // Exact match
        if name1_norm == name2_norm {
            return 1.0;
        }
        
        // Contains match (partial) - improved logic
        if name1_norm.contains(&name2_norm) || name2_norm.contains(&name1_norm) {
            let shorter_len = name1_norm.len().min(name2_norm.len()) as f64;
            let longer_len = name1_norm.len().max(name2_norm.len()) as f64;
            let ratio = shorter_len / longer_len;
            
            // Higher score for better ratios
            if ratio >= 0.8 {
                return 0.95; // Very close match
            } else if ratio >= 0.6 {
                return 0.85; // Good partial match
            } else {
                return ratio * 0.8; // Scaled partial match
            }
        }
        
        // Levenshtein distance-based similarity
        let distance = levenshtein::levenshtein(&name1_norm, &name2_norm);
        let max_len = name1_norm.len().max(name2_norm.len());
        
        if max_len == 0 {
            return 0.0;
        }
        
        let similarity = 1.0 - (distance as f64 / max_len as f64);
        
        // More lenient threshold for fuzzy matching
        if similarity >= 0.5 {
            similarity * 0.7 // Penalty for fuzzy match
        } else {
            0.0
        }
    }

    /// Normalize name for consistent matching
    fn normalize_name(&self, name: &str) -> String {
        name.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Add artist alias with confidence scoring
    pub async fn add_artist_alias(&self, artist_id: Uuid, alias: ArtistAlias) -> Result<()> {
        if let Some(mut artist_entry) = self.artist_cache.get_mut(&artist_id.to_string()) {
            artist_entry.add_alias(alias.clone());
            
            // Update name cache for the new alias
            let alias_key = self.normalize_name(&alias.name);
            self.name_cache
                .entry(alias_key)
                .or_insert_with(Vec::new)
                .push(artist_id);
            
            println!("Added alias '{}' to artist {}", alias.name, artist_id);
            Ok(())
        } else {
            Err(anyhow!("Artist not found: {}", artist_id))
        }
    }

    /// Merge two artists (combine their external IDs and aliases)
    pub async fn merge_artists(&self, primary_id: Uuid, secondary_id: Uuid) -> Result<()> {
        let secondary_artist = self.artist_cache
            .get(&secondary_id.to_string())
            .ok_or_else(|| anyhow!("Secondary artist not found: {}", secondary_id))?
            .clone();
        
        if let Some(mut primary_entry) = self.artist_cache.get_mut(&primary_id.to_string()) {
            // Merge external IDs
            primary_entry.merge_external_ids(&secondary_artist.external_ids);
            
            // Merge aliases
            for alias in secondary_artist.aliases {
                primary_entry.add_alias(alias);
            }
            
            // Update external ID cache to point to primary artist
            for (provider, external_id) in secondary_artist.external_ids.get_all_ids() {
                let cache_key = format!("{}:{}", provider, external_id);
                self.external_id_cache.insert(cache_key, primary_id);
            }
            
            // Remove secondary artist from cache
            self.artist_cache.remove(&secondary_id.to_string());
            
            println!("Merged artist {} into {}", secondary_id, primary_id);
            Ok(())
        } else {
            Err(anyhow!("Primary artist not found: {}", primary_id))
        }
    }

    /// Get cache statistics for monitoring
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            artist_count: self.artist_cache.len(),
            external_id_count: self.external_id_cache.len(),
            name_mapping_count: self.name_cache.len(),
        }
    }

    /// Clear all caches
    pub fn clear_cache(&self) {
        self.artist_cache.clear();
        self.external_id_cache.clear();
        self.name_cache.clear();
        println!("Cleared all entity resolution caches");
    }
}

impl Default for EntityResolutionService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub artist_count: usize,
    pub external_id_count: usize,
    pub name_mapping_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ExternalIds;

    async fn create_test_service() -> EntityResolutionService {
        let service = EntityResolutionService::new().with_confidence_threshold(0.6);
        
        // Add test artists
        let artist1 = Artist::with_external_ids(
            "The Beatles".to_string(),
            ExternalIds::new().with_spotify("spotify123".to_string()),
        );
        
        let mut artist2 = Artist::new("Drake".to_string());
        artist2.external_ids.apple = Some("apple456".to_string());
        artist2.add_alias(ArtistAlias::new("Aubrey Graham".to_string(), "real_name".to_string(), 0.9));
        
        let artist3 = Artist::new("Taylor Swift".to_string());
        
        service.add_artist(artist1).await.unwrap();
        service.add_artist(artist2).await.unwrap();
        service.add_artist(artist3).await.unwrap();
        
        service
    }

    #[tokio::test]
    async fn test_exact_name_resolution() {
        let service = create_test_service().await;
        
        let query = ArtistSearchQuery::new("The Beatles".to_string());
        let results = service.resolve_artist(&query).await.unwrap();
        
        assert!(!results.is_empty());
        assert_eq!(results[0].artist.canonical_name, "The Beatles");
        assert!(results[0].confidence >= 0.9);
        assert_eq!(results[0].match_type, MatchType::ExactName);
    }

    #[tokio::test]
    async fn test_external_id_resolution() {
        let service = create_test_service().await;
        
        let query = ArtistSearchQuery::new("Unknown".to_string())
            .with_provider("spotify".to_string())
            .with_external_id("spotify123".to_string());
        
        let results = service.resolve_artist(&query).await.unwrap();
        
        assert!(!results.is_empty());
        assert_eq!(results[0].artist.canonical_name, "The Beatles");
        assert_eq!(results[0].confidence, 1.0);
        assert_eq!(results[0].match_type, MatchType::ExactExternalId);
    }

    #[tokio::test]
    async fn test_alias_resolution() {
        let service = create_test_service().await;
        
        let query = ArtistSearchQuery::new("Aubrey Graham".to_string());
        let results = service.resolve_artist(&query).await.unwrap();
        
        assert!(!results.is_empty());
        assert_eq!(results[0].artist.canonical_name, "Drake");
        assert!(results[0].confidence >= 0.6);
    }

    #[tokio::test]
    async fn test_fuzzy_name_resolution() {
        let service = create_test_service().await;
        
        let query = ArtistSearchQuery::new("Beatles".to_string()); // Partial match
        let results = service.resolve_artist(&query).await.unwrap();
        
        assert!(!results.is_empty());
        assert_eq!(results[0].artist.canonical_name, "The Beatles");
        assert!(results[0].confidence >= 0.6);
    }

    #[tokio::test]
    async fn test_concurrent_resolution() {
        let service = create_test_service().await;
        
        let queries = vec![
            ArtistSearchQuery::new("The Beatles".to_string()),
            ArtistSearchQuery::new("Drake".to_string()),
            ArtistSearchQuery::new("Taylor Swift".to_string()),
        ];
        
        let results = service.resolve_concurrent(&queries).await.unwrap();
        
        assert_eq!(results.len(), 3);
        assert!(!results[0].is_empty());
        assert!(!results[1].is_empty());
        assert!(!results[2].is_empty());
    }

    #[tokio::test]
    async fn test_name_similarity_calculation() {
        assert_eq!(EntityResolutionService::calculate_name_similarity("The Beatles", "The Beatles"), 1.0);
        assert!(EntityResolutionService::calculate_name_similarity("The Beatles", "Beatles") > 0.8);
        assert!(EntityResolutionService::calculate_name_similarity("Taylor Swift", "T Swift") > 0.3);
        assert_eq!(EntityResolutionService::calculate_name_similarity("Completely Different", "Artist Name"), 0.0);
    }

    #[tokio::test]
    async fn test_artist_merging() {
        let service = create_test_service().await;
        
        // Add duplicate artist with different external ID
        let duplicate = Artist::with_external_ids(
            "The Beatles Duplicate".to_string(),
            ExternalIds::new().with_apple("apple789".to_string()),
        );
        let duplicate_id = duplicate.id;
        service.add_artist(duplicate).await.unwrap();
        
        // Find original Beatles artist
        let query = ArtistSearchQuery::new("The Beatles".to_string());
        let results = service.resolve_artist(&query).await.unwrap();
        let original_id = results[0].artist.id;
        
        // Merge duplicate into original
        service.merge_artists(original_id, duplicate_id).await.unwrap();
        
        // Verify merge
        let merged_artist = service.artist_cache.get(&original_id.to_string()).unwrap();
        assert!(merged_artist.external_ids.apple.is_some());
        assert_eq!(merged_artist.external_ids.apple, Some("apple789".to_string()));
        
        // Verify duplicate is removed
        assert!(service.artist_cache.get(&duplicate_id.to_string()).is_none());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let service = create_test_service().await;
        let stats = service.get_cache_stats();
        
        assert_eq!(stats.artist_count, 3);
        assert!(stats.external_id_count >= 2); // At least spotify and apple IDs
        assert!(stats.name_mapping_count >= 3); // At least 3 canonical names
    }
}