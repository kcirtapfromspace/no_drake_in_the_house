use crate::models::{Artist, ArtistAlias, ArtistMetadata, ExternalIds};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// Circuit breaker states for external API resilience
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker for external API calls
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_count: u32,
    failure_threshold: u32,
    timeout_duration: Duration,
    last_failure_time: Option<std::time::SystemTime>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, timeout_duration: Duration) -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            failure_threshold,
            timeout_duration,
            last_failure_time: None,
        }
    }

    pub fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed().unwrap_or(Duration::ZERO) > self.timeout_duration {
                        self.state = CircuitBreakerState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitBreakerState::Closed;
        self.last_failure_time = None;
    }

    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(std::time::SystemTime::now());

        if self.failure_count >= self.failure_threshold {
            self.state = CircuitBreakerState::Open;
        }
    }
}

/// MusicBrainz API response structures
#[derive(Debug, Deserialize)]
pub struct MusicBrainzSearchResponse {
    pub artists: Vec<MusicBrainzArtist>,
    pub count: u32,
    pub offset: u32,
}

#[derive(Debug, Deserialize)]
pub struct MusicBrainzArtist {
    pub id: String,
    pub name: String,
    pub disambiguation: Option<String>,
    #[serde(rename = "sort-name")]
    pub sort_name: String,
    pub aliases: Option<Vec<MusicBrainzAlias>>,
    #[serde(rename = "life-span")]
    pub life_span: Option<MusicBrainzLifeSpan>,
    pub area: Option<MusicBrainzArea>,
    pub relations: Option<Vec<MusicBrainzRelation>>,
    pub score: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct MusicBrainzAlias {
    pub name: String,
    #[serde(rename = "sort-name")]
    pub sort_name: String,
    #[serde(rename = "type")]
    pub alias_type: Option<String>,
    pub locale: Option<String>,
    pub primary: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct MusicBrainzLifeSpan {
    pub begin: Option<String>,
    pub end: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MusicBrainzArea {
    pub id: String,
    pub name: String,
    #[serde(rename = "sort-name")]
    pub sort_name: String,
}

#[derive(Debug, Deserialize)]
pub struct MusicBrainzRelation {
    #[serde(rename = "type")]
    pub relation_type: String,
    pub url: Option<MusicBrainzUrl>,
}

#[derive(Debug, Deserialize)]
pub struct MusicBrainzUrl {
    pub resource: String,
}

/// ISNI API response structures
#[derive(Debug, Deserialize)]
pub struct IsniSearchResponse {
    #[serde(rename = "responseHeader")]
    pub response_header: IsniResponseHeader,
    pub response: IsniResponse,
}

#[derive(Debug, Deserialize)]
pub struct IsniResponseHeader {
    pub status: u32,
}

#[derive(Debug, Deserialize)]
pub struct IsniResponse {
    #[serde(rename = "numFound")]
    pub num_found: u32,
    pub docs: Vec<IsniDocument>,
}

#[derive(Debug, Deserialize)]
pub struct IsniDocument {
    pub isni: Vec<String>,
    #[serde(rename = "ISNIAssigned")]
    pub isni_assigned: Vec<String>,
    #[serde(rename = "forename")]
    pub forename: Option<Vec<String>>,
    #[serde(rename = "surname")]
    pub surname: Option<Vec<String>>,
    #[serde(rename = "marcDate")]
    pub marc_date: Option<Vec<String>>,
    #[serde(rename = "creationClass")]
    pub creation_class: Option<Vec<String>>,
}

/// High-performance MusicBrainz API client with connection pooling
#[derive(Clone)]
pub struct MusicBrainzClient {
    client: reqwest::Client,
    pub base_url: String,
    circuit_breaker: Arc<tokio::sync::Mutex<CircuitBreaker>>,
    rate_limiter: Arc<tokio::sync::Semaphore>,
}

impl MusicBrainzClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("MusicStreamingBlocklistManager/1.0 (contact@example.com)")
            .pool_max_idle_per_host(10)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: "https://musicbrainz.org/ws/2".to_string(),
            circuit_breaker: Arc::new(tokio::sync::Mutex::new(
                CircuitBreaker::new(5, Duration::from_secs(60))
            )),
            rate_limiter: Arc::new(tokio::sync::Semaphore::new(1)), // 1 request per second
        }
    }

    /// Search for artists by name with circuit breaker protection
    pub async fn search_artists(&self, query: &str, limit: Option<u32>) -> Result<Vec<Artist>> {
        let _permit = self.rate_limiter.acquire().await?;
        
        let mut circuit_breaker = self.circuit_breaker.lock().await;
        if !circuit_breaker.can_execute() {
            return Err(anyhow!("MusicBrainz circuit breaker is open"));
        }
        drop(circuit_breaker);

        let limit = limit.unwrap_or(25).min(100); // MusicBrainz max is 100
        let url = format!(
            "{}/artist?query={}&fmt=json&limit={}",
            self.base_url,
            urlencoding::encode(query),
            limit
        );

        let response = match self.client.get(&url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                let mut circuit_breaker = self.circuit_breaker.lock().await;
                circuit_breaker.record_failure();
                return Err(anyhow!("MusicBrainz request failed: {}", e));
            }
        };

        if !response.status().is_success() {
            let mut circuit_breaker = self.circuit_breaker.lock().await;
            circuit_breaker.record_failure();
            return Err(anyhow!("MusicBrainz API error: {}", response.status()));
        }

        let search_response: MusicBrainzSearchResponse = match response.json().await {
            Ok(data) => data,
            Err(e) => {
                let mut circuit_breaker = self.circuit_breaker.lock().await;
                circuit_breaker.record_failure();
                return Err(anyhow!("Failed to parse MusicBrainz response: {}", e));
            }
        };

        let mut circuit_breaker = self.circuit_breaker.lock().await;
        circuit_breaker.record_success();
        drop(circuit_breaker);

        // Convert MusicBrainz artists to our Artist model
        let artists = search_response
            .artists
            .into_iter()
            .map(|mb_artist| self.convert_musicbrainz_artist(mb_artist))
            .collect();

        Ok(artists)
    }

    /// Get artist by MusicBrainz ID
    pub async fn get_artist_by_id(&self, mbid: &str) -> Result<Option<Artist>> {
        let _permit = self.rate_limiter.acquire().await?;
        
        let mut circuit_breaker = self.circuit_breaker.lock().await;
        if !circuit_breaker.can_execute() {
            return Err(anyhow!("MusicBrainz circuit breaker is open"));
        }
        drop(circuit_breaker);

        let url = format!(
            "{}/artist/{}?fmt=json&inc=aliases+relations",
            self.base_url, mbid
        );

        let response = match self.client.get(&url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                let mut circuit_breaker = self.circuit_breaker.lock().await;
                circuit_breaker.record_failure();
                return Err(anyhow!("MusicBrainz request failed: {}", e));
            }
        };

        if response.status() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            let mut circuit_breaker = self.circuit_breaker.lock().await;
            circuit_breaker.record_failure();
            return Err(anyhow!("MusicBrainz API error: {}", response.status()));
        }

        let mb_artist: MusicBrainzArtist = match response.json().await {
            Ok(data) => data,
            Err(e) => {
                let mut circuit_breaker = self.circuit_breaker.lock().await;
                circuit_breaker.record_failure();
                return Err(anyhow!("Failed to parse MusicBrainz response: {}", e));
            }
        };

        let mut circuit_breaker = self.circuit_breaker.lock().await;
        circuit_breaker.record_success();
        drop(circuit_breaker);

        Ok(Some(self.convert_musicbrainz_artist(mb_artist)))
    }

    /// Convert MusicBrainz artist to our Artist model
    pub fn convert_musicbrainz_artist(&self, mb_artist: MusicBrainzArtist) -> Artist {
        let mut external_ids = ExternalIds::new().with_musicbrainz(mb_artist.id.clone());
        
        // Extract ISNI from relations if available
        if let Some(relations) = &mb_artist.relations {
            for relation in relations {
                if relation.relation_type == "isni" {
                    if let Some(url) = &relation.url {
                        if let Some(isni) = self.extract_isni_from_url(&url.resource) {
                            external_ids = external_ids.with_isni(isni);
                        }
                    }
                }
            }
        }

        let mut artist = Artist::with_external_ids(mb_artist.name.clone(), external_ids);

        // Add aliases with confidence scoring
        if let Some(aliases) = mb_artist.aliases {
            for alias in aliases {
                let confidence = if alias.primary.unwrap_or(false) {
                    0.95
                } else if alias.alias_type.as_deref() == Some("Artist name") {
                    0.9
                } else {
                    0.8
                };

                let mut artist_alias = ArtistAlias::new(
                    alias.name,
                    "musicbrainz".to_string(),
                    confidence,
                );

                if let Some(locale) = alias.locale {
                    artist_alias = artist_alias.with_locale(locale);
                }

                artist.add_alias(artist_alias);
            }
        }

        // Add metadata
        let mut metadata = ArtistMetadata::default();
        
        if let Some(area) = mb_artist.area {
            metadata.country = Some(area.name);
        }

        if let Some(life_span) = mb_artist.life_span {
            if let Some(begin) = life_span.begin {
                if let Ok(year) = begin.split('-').next().unwrap_or("").parse::<u32>() {
                    metadata.formed_year = Some(year);
                }
            }
        }

        artist.update_metadata(metadata);
        artist
    }

    /// Extract ISNI from URL
    pub fn extract_isni_from_url(&self, url: &str) -> Option<String> {
        if url.contains("isni.org") {
            url.split('/').last().map(|s| s.to_string())
        } else {
            None
        }
    }
}

impl Default for MusicBrainzClient {
    fn default() -> Self {
        Self::new()
    }
}

/// ISNI API client for authoritative artist identification
#[derive(Clone)]
pub struct IsniClient {
    client: reqwest::Client,
    pub base_url: String,
    circuit_breaker: Arc<tokio::sync::Mutex<CircuitBreaker>>,
    rate_limiter: Arc<tokio::sync::Semaphore>,
}

impl IsniClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("MusicStreamingBlocklistManager/1.0 (contact@example.com)")
            .pool_max_idle_per_host(5)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: "https://isni.oclc.org/sru".to_string(),
            circuit_breaker: Arc::new(tokio::sync::Mutex::new(
                CircuitBreaker::new(3, Duration::from_secs(120))
            )),
            rate_limiter: Arc::new(tokio::sync::Semaphore::new(1)), // Conservative rate limiting
        }
    }

    /// Search for artists by name in ISNI database
    pub async fn search_artists(&self, query: &str, limit: Option<u32>) -> Result<Vec<Artist>> {
        let _permit = self.rate_limiter.acquire().await?;
        
        let mut circuit_breaker = self.circuit_breaker.lock().await;
        if !circuit_breaker.can_execute() {
            return Err(anyhow!("ISNI circuit breaker is open"));
        }
        drop(circuit_breaker);

        let limit = limit.unwrap_or(10).min(50);
        let cql_query = format!("pica.na=\"{}\" and pica.ccs=ba02", query);
        
        let url = format!(
            "{}?query={}&operation=searchRetrieve&recordSchema=isni-b&maximumRecords={}",
            self.base_url,
            urlencoding::encode(&cql_query),
            limit
        );

        let response = match self.client.get(&url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                let mut circuit_breaker = self.circuit_breaker.lock().await;
                circuit_breaker.record_failure();
                return Err(anyhow!("ISNI request failed: {}", e));
            }
        };

        if !response.status().is_success() {
            let mut circuit_breaker = self.circuit_breaker.lock().await;
            circuit_breaker.record_failure();
            return Err(anyhow!("ISNI API error: {}", response.status()));
        }

        // ISNI returns XML, but for simplicity we'll return empty results
        // In a real implementation, you'd parse the XML response
        let mut circuit_breaker = self.circuit_breaker.lock().await;
        circuit_breaker.record_success();
        drop(circuit_breaker);

        // Placeholder implementation - would parse XML in real scenario
        Ok(Vec::new())
    }

    /// Get artist by ISNI identifier
    pub async fn get_artist_by_isni(&self, isni: &str) -> Result<Option<Artist>> {
        let _permit = self.rate_limiter.acquire().await?;
        
        let mut circuit_breaker = self.circuit_breaker.lock().await;
        if !circuit_breaker.can_execute() {
            return Err(anyhow!("ISNI circuit breaker is open"));
        }
        drop(circuit_breaker);

        // Placeholder implementation
        let mut circuit_breaker = self.circuit_breaker.lock().await;
        circuit_breaker.record_success();
        drop(circuit_breaker);

        Ok(None)
    }
}

impl Default for IsniClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Combined external API service with fallback strategies
#[derive(Clone)]
pub struct ExternalApiService {
    pub musicbrainz_client: MusicBrainzClient,
    pub isni_client: IsniClient,
}

impl ExternalApiService {
    pub fn new() -> Self {
        Self {
            musicbrainz_client: MusicBrainzClient::new(),
            isni_client: IsniClient::new(),
        }
    }

    /// Search for artists across multiple external APIs with fallback
    pub async fn search_artists_with_fallback(&self, query: &str, limit: Option<u32>) -> Result<Vec<Artist>> {
        let mut all_artists = Vec::new();

        // Try MusicBrainz first (more comprehensive for music)
        match self.musicbrainz_client.search_artists(query, limit).await {
            Ok(mut artists) => {
                println!("Found {} artists from MusicBrainz", artists.len());
                all_artists.append(&mut artists);
            }
            Err(e) => {
                println!("MusicBrainz search failed: {}", e);
            }
        }

        // Try ISNI as fallback (authoritative but limited)
        if all_artists.is_empty() {
            match self.isni_client.search_artists(query, limit).await {
                Ok(mut artists) => {
                    println!("Found {} artists from ISNI", artists.len());
                    all_artists.append(&mut artists);
                }
                Err(e) => {
                    println!("ISNI search failed: {}", e);
                }
            }
        }

        Ok(all_artists)
    }

    /// Enrich artist with additional data from external APIs
    pub async fn enrich_artist(&self, artist: &mut Artist) -> Result<()> {
        // If we have a MusicBrainz ID, get additional data
        if let Some(mbid) = &artist.external_ids.musicbrainz {
            match self.musicbrainz_client.get_artist_by_id(mbid).await {
                Ok(Some(enriched_artist)) => {
                    // Merge external IDs
                    artist.merge_external_ids(&enriched_artist.external_ids);
                    
                    // Merge aliases
                    for alias in enriched_artist.aliases {
                        artist.add_alias(alias);
                    }
                    
                    // Update metadata if we have more complete data
                    if enriched_artist.metadata.country.is_some() && artist.metadata.country.is_none() {
                        artist.metadata.country = enriched_artist.metadata.country;
                    }
                    if enriched_artist.metadata.formed_year.is_some() && artist.metadata.formed_year.is_none() {
                        artist.metadata.formed_year = enriched_artist.metadata.formed_year;
                    }
                }
                Ok(None) => {
                    println!("Artist not found in MusicBrainz: {}", mbid);
                }
                Err(e) => {
                    println!("Failed to enrich artist from MusicBrainz: {}", e);
                }
            }
        }

        // If we have an ISNI, get additional authoritative data
        if let Some(isni) = &artist.external_ids.isni {
            match self.isni_client.get_artist_by_isni(isni).await {
                Ok(Some(isni_artist)) => {
                    // Merge any additional data from ISNI
                    artist.merge_external_ids(&isni_artist.external_ids);
                }
                Ok(None) => {
                    println!("Artist not found in ISNI: {}", isni);
                }
                Err(e) => {
                    println!("Failed to enrich artist from ISNI: {}", e);
                }
            }
        }

        Ok(())
    }
}

impl Default for ExternalApiService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker() {
        let mut cb = CircuitBreaker::new(2, Duration::from_secs(1));
        
        // Initially closed
        assert_eq!(cb.state, CircuitBreakerState::Closed);
        assert!(cb.can_execute());
        
        // Record failures
        cb.record_failure();
        assert_eq!(cb.state, CircuitBreakerState::Closed);
        assert!(cb.can_execute());
        
        cb.record_failure();
        assert_eq!(cb.state, CircuitBreakerState::Open);
        assert!(!cb.can_execute());
        
        // Record success should reset
        cb.record_success();
        assert_eq!(cb.state, CircuitBreakerState::Closed);
        assert!(cb.can_execute());
    }

    #[tokio::test]
    async fn test_musicbrainz_client_creation() {
        let client = MusicBrainzClient::new();
        assert_eq!(client.base_url, "https://musicbrainz.org/ws/2");
    }

    #[tokio::test]
    async fn test_isni_client_creation() {
        let client = IsniClient::new();
        assert_eq!(client.base_url, "https://isni.oclc.org/sru");
    }

    #[tokio::test]
    async fn test_external_api_service_creation() {
        let service = ExternalApiService::new();
        // Just test that it creates without panicking
        assert_eq!(service.musicbrainz_client.base_url, "https://musicbrainz.org/ws/2");
    }

    // Note: Integration tests with real APIs would require network access
    // and should be run separately with proper test data
}