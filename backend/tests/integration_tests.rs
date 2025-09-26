use music_streaming_blocklist_backend::services::{MusicBrainzClient, IsniClient, ExternalApiService, CircuitBreaker};
use music_streaming_blocklist_backend::models::{Artist, ExternalIds, ArtistSearchQuery};
use std::time::Duration;

#[tokio::test]
async fn test_musicbrainz_client_creation() {
    let client = MusicBrainzClient::new();
    // Just test that the client can be created without panicking
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

#[tokio::test]
async fn test_circuit_breaker_behavior() {
    let mut cb = CircuitBreaker::new(2, Duration::from_millis(100));
    
    // Test normal operation
    assert!(cb.can_execute());
    cb.record_success();
    assert!(cb.can_execute());
    
    // Test failure threshold
    cb.record_failure();
    assert!(cb.can_execute()); // Still closed
    
    cb.record_failure();
    assert!(!cb.can_execute()); // Now open
    
    // Test timeout recovery
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert!(cb.can_execute()); // Should be half-open now
    
    cb.record_success();
    assert!(cb.can_execute()); // Back to closed
}

// Integration tests for MusicBrainz and ISNI external API integration
// Note: These tests verify the integration logic without requiring external mock libraries
// due to Rust version compatibility constraints

#[tokio::test]
async fn test_musicbrainz_client_configuration() {
    let client = MusicBrainzClient::new();
    
    // Verify client is properly configured
    assert_eq!(client.base_url, "https://musicbrainz.org/ws/2");
    
    // Test that client can handle invalid URLs gracefully
    let mut test_client = MusicBrainzClient::new();
    test_client.base_url = "http://invalid-url-that-does-not-exist".to_string();
    
    let result = test_client.search_artists("Test Artist", Some(1)).await;
    assert!(result.is_err()); // Should fail gracefully, not panic
}

#[tokio::test]
async fn test_isni_client_configuration() {
    let client = IsniClient::new();
    
    // Verify client is properly configured
    assert_eq!(client.base_url, "https://isni.oclc.org/sru");
    
    // Test that client can handle invalid URLs gracefully
    let mut test_client = IsniClient::new();
    test_client.base_url = "http://invalid-url-that-does-not-exist".to_string();
    
    let result = test_client.search_artists("Test Artist", Some(1)).await;
    assert!(result.is_err()); // Should fail gracefully, not panic
}

#[tokio::test]
async fn test_external_api_service_fallback_behavior() {
    let service = ExternalApiService::new();
    
    // Test with a query that should trigger fallback behavior
    // This may succeed or fail depending on network conditions, but should not panic
    let result = service.search_artists_with_fallback("NonexistentArtist12345", Some(5)).await;
    
    assert!(result.is_ok());
    let artists = result.unwrap();
    // May be empty due to network issues or rate limiting, which is expected
    println!("Found {} artists for nonexistent query", artists.len());
}

#[tokio::test]
async fn test_artist_enrichment_with_invalid_data() {
    let service = ExternalApiService::new();
    
    // Create an artist with invalid MusicBrainz ID
    let mut artist = Artist::with_external_ids(
        "Test Artist".to_string(),
        ExternalIds::new().with_musicbrainz("invalid-mbid-123".to_string())
    );
    
    // Try to enrich it (should fail gracefully)
    let result = service.enrich_artist(&mut artist).await;
    
    // Should not panic even with invalid data
    assert!(result.is_ok());
    println!("Artist enrichment completed for: {}", artist.canonical_name);
}

#[tokio::test]
async fn test_musicbrainz_response_parsing() {
    // Test the conversion logic with sample data
    use music_streaming_blocklist_backend::services::external_apis::MusicBrainzArtist;
    
    let client = MusicBrainzClient::new();
    
    // Create a sample MusicBrainz artist response
    let mb_artist = MusicBrainzArtist {
        id: "b10bbbfc-cf9e-42e0-be17-e2c3e1d2600d".to_string(),
        name: "The Beatles".to_string(),
        disambiguation: Some("English rock band".to_string()),
        sort_name: "Beatles, The".to_string(),
        aliases: Some(vec![]),
        life_span: None,
        area: None,
        relations: None,
        score: Some(100),
    };
    
    let artist = client.convert_musicbrainz_artist(mb_artist);
    
    assert_eq!(artist.canonical_name, "The Beatles");
    assert!(artist.external_ids.musicbrainz.is_some());
    assert_eq!(artist.external_ids.musicbrainz.unwrap(), "b10bbbfc-cf9e-42e0-be17-e2c3e1d2600d");
}

#[tokio::test]
async fn test_isni_url_extraction() {
    let client = MusicBrainzClient::new();
    
    // Test ISNI URL extraction
    let isni_url = "https://isni.org/isni/0000000123456789";
    let extracted = client.extract_isni_from_url(isni_url);
    
    assert!(extracted.is_some());
    assert_eq!(extracted.unwrap(), "0000000123456789");
    
    // Test invalid URL
    let invalid_url = "https://example.com/not-isni";
    let extracted = client.extract_isni_from_url(invalid_url);
    assert!(extracted.is_none());
}

#[tokio::test]
async fn test_rate_limiting_behavior() {
    use std::time::Instant;
    
    let client = MusicBrainzClient::new();
    
    // Make multiple requests and ensure they're rate limited
    let start = Instant::now();
    
    // These should be rate limited to 1 per second
    let tasks = vec![
        client.search_artists("Artist1", Some(1)),
        client.search_artists("Artist2", Some(1)),
    ];
    
    let results = futures::future::join_all(tasks).await;
    let elapsed = start.elapsed();
    
    // Should take at least 1 second due to rate limiting
    assert!(elapsed >= Duration::from_millis(900)); // Allow some tolerance
    
    // All requests should complete (though they may fail due to network)
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_entity_resolution_with_external_apis() {
    use music_streaming_blocklist_backend::services::EntityResolutionService;

    let service = EntityResolutionService::new();
    
    // Test search that will fall back to external APIs
    let query = ArtistSearchQuery::new("The Beatles".to_string()).with_limit(5);
    let result = service.resolve_artist(&query).await;
    
    assert!(result.is_ok());
    let results = result.unwrap();
    
    // Results may be empty due to network issues, rate limiting, or circuit breaker
    // The important thing is that it doesn't panic and handles errors gracefully
    println!("Entity resolution found {} results", results.len());
    
    for result in &results {
        println!("Found artist: {} (confidence: {:.2})", 
                result.artist.canonical_name, result.confidence);
    }
}

// Test MusicBrainz integration with mock API responses using wiremock
#[tokio::test]
async fn test_musicbrainz_integration_with_mock() {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, query_param};
    use music_streaming_blocklist_backend::services::external_apis::MusicBrainzClient;
    
    // Start a mock server
    let mock_server = MockServer::start().await;
    
    // Mock MusicBrainz search response
    let mock_response = r#"{
        "artists": [
            {
                "id": "b10bbbfc-cf9e-42e0-be17-e2c3e1d2600d",
                "name": "The Beatles",
                "sort-name": "Beatles, The",
                "disambiguation": "English rock band",
                "score": 100,
                "aliases": [
                    {
                        "name": "Beatles",
                        "sort-name": "Beatles",
                        "type": "Artist name",
                        "primary": true,
                        "locale": "en"
                    },
                    {
                        "name": "Fab Four",
                        "sort-name": "Fab Four",
                        "type": "Artist name",
                        "primary": false
                    }
                ],
                "life-span": {
                    "begin": "1960",
                    "end": "1970"
                },
                "area": {
                    "id": "8a754a16-0027-3a29-b6d7-2b40ea0481ed",
                    "name": "United Kingdom",
                    "sort-name": "United Kingdom"
                },
                "relations": [
                    {
                        "type": "isni",
                        "url": {
                            "resource": "https://isni.org/isni/0000000123456789"
                        }
                    }
                ]
            }
        ],
        "count": 1,
        "offset": 0
    }"#;
    
    // Mock any GET request to /ws/2/artist
    Mock::given(method("GET"))
        .and(path("/ws/2/artist"))
        .respond_with(ResponseTemplate::new(200).set_body_string(mock_response))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    // Create client with mock server URL
    let mut client = MusicBrainzClient::new();
    client.base_url = mock_server.uri();
    
    // Test the search
    let results = client.search_artists("The Beatles", Some(10)).await.expect("MusicBrainz search should succeed with mock");
    
    assert_eq!(results.len(), 1);
    let artist = &results[0];
    assert_eq!(artist.canonical_name, "The Beatles");
    assert_eq!(artist.external_ids.musicbrainz, Some("b10bbbfc-cf9e-42e0-be17-e2c3e1d2600d".to_string()));
    assert_eq!(artist.external_ids.isni, Some("0000000123456789".to_string()));
    assert_eq!(artist.aliases.len(), 2);
    assert_eq!(artist.metadata.country, Some("United Kingdom".to_string()));
    assert_eq!(artist.metadata.formed_year, Some(1960));
    
    // Verify alias confidence scoring
    let primary_alias = artist.aliases.iter().find(|a| a.name == "Beatles").unwrap();
    assert_eq!(primary_alias.confidence, 0.95); // Primary alias
    
    let secondary_alias = artist.aliases.iter().find(|a| a.name == "Fab Four").unwrap();
    assert_eq!(secondary_alias.confidence, 0.8); // Non-primary alias
}

#[tokio::test]
async fn test_musicbrainz_circuit_breaker_with_mock() {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};
    use music_streaming_blocklist_backend::services::external_apis::MusicBrainzClient;
    
    let mock_server = MockServer::start().await;
    
    // Mock server errors to trigger circuit breaker
    Mock::given(method("GET"))
        .and(path("/ws/2/artist"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;
    
    let mut client = MusicBrainzClient::new();
    client.base_url = mock_server.uri();
    
    // First few requests should fail and eventually open the circuit breaker
    for i in 0..6 {
        let result = client.search_artists("Test Artist", Some(10)).await;
        assert!(result.is_err());
        println!("Request {} failed as expected", i + 1);
    }
    
    // Circuit breaker should now be open and reject requests immediately
    let result = client.search_artists("Test Artist", Some(10)).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("circuit breaker is open"));
}

#[tokio::test]
async fn test_entity_resolution_with_mock_external_apis() {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, query_param};
    use music_streaming_blocklist_backend::services::{EntityResolutionService, ExternalApiService};
    use music_streaming_blocklist_backend::services::external_apis::MusicBrainzClient;
    use music_streaming_blocklist_backend::models::ArtistSearchQuery;
    
    let mock_server = MockServer::start().await;
    
    // Mock MusicBrainz response for Drake
    let drake_response = r#"{
        "artists": [
            {
                "id": "3aa81c21-3c3b-4c8e-8f5e-5c5c5c5c5c5c",
                "name": "Drake",
                "sort-name": "Drake",
                "disambiguation": "Canadian rapper",
                "score": 100,
                "aliases": [
                    {
                        "name": "Aubrey Graham",
                        "sort-name": "Graham, Aubrey",
                        "type": "Legal name",
                        "primary": false
                    },
                    {
                        "name": "Champagne Papi",
                        "sort-name": "Champagne Papi",
                        "type": "Artist name",
                        "primary": false
                    }
                ],
                "area": {
                    "id": "71bbafaa-e825-3e15-8ca9-017dcad1748b",
                    "name": "Canada",
                    "sort-name": "Canada"
                }
            }
        ],
        "count": 1,
        "offset": 0
    }"#;
    
    Mock::given(method("GET"))
        .and(path("/ws/2/artist"))
        .and(query_param("query", "Drake"))
        .respond_with(ResponseTemplate::new(200).set_body_string(drake_response))
        .mount(&mock_server)
        .await;
    
    // Create entity resolution service with mock external API
    let mut musicbrainz_client = MusicBrainzClient::new();
    musicbrainz_client.base_url = mock_server.uri();
    
    let external_api_service = ExternalApiService {
        musicbrainz_client,
        isni_client: Default::default(),
    };
    
    let mut entity_service = EntityResolutionService::new();
    entity_service.external_api_service = external_api_service;
    
    // Test resolution that falls back to external API
    let query = ArtistSearchQuery::new("Drake".to_string()).with_limit(5);
    let results = entity_service.resolve_artist(&query).await.unwrap();
    
    assert!(!results.is_empty());
    let result = &results[0];
    assert_eq!(result.artist.canonical_name, "Drake");
    assert_eq!(result.artist.external_ids.musicbrainz, Some("3aa81c21-3c3b-4c8e-8f5e-5c5c5c5c5c5c".to_string()));
    assert_eq!(result.artist.metadata.country, Some("Canada".to_string()));
    assert_eq!(result.artist.aliases.len(), 2);
    
    // Verify aliases were added with correct confidence
    let legal_name_alias = result.artist.aliases.iter().find(|a| a.name == "Aubrey Graham").unwrap();
    assert_eq!(legal_name_alias.source, "musicbrainz");
    assert!(legal_name_alias.confidence >= 0.8);
}

#[tokio::test]
async fn test_musicbrainz_rate_limiting_with_mock() {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};
    use music_streaming_blocklist_backend::services::external_apis::MusicBrainzClient;
    use std::time::Instant;
    
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/ws/2/artist"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"artists":[],"count":0,"offset":0}"#))
        .mount(&mock_server)
        .await;
    
    let mut client = MusicBrainzClient::new();
    client.base_url = mock_server.uri();
    
    // Test that rate limiting is enforced (should take at least 1 second for 2 requests)
    let start = Instant::now();
    
    let _result1 = client.search_artists("Artist 1", Some(10)).await.unwrap();
    let _result2 = client.search_artists("Artist 2", Some(10)).await.unwrap();
    
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() >= 900); // Allow some tolerance for timing
}

// Test MusicBrainz alias handling with confidence scoring
#[tokio::test]
async fn test_musicbrainz_alias_confidence_scoring() {
    use music_streaming_blocklist_backend::services::external_apis::{MusicBrainzArtist, MusicBrainzAlias, MusicBrainzClient};
    
    let client = MusicBrainzClient::new();
    
    let mb_artist = MusicBrainzArtist {
        id: "test-id".to_string(),
        name: "Test Artist".to_string(),
        disambiguation: None,
        sort_name: "Artist, Test".to_string(),
        aliases: Some(vec![
            MusicBrainzAlias {
                name: "Primary Alias".to_string(),
                sort_name: "Alias, Primary".to_string(),
                alias_type: Some("Artist name".to_string()),
                locale: Some("en".to_string()),
                primary: Some(true),
            },
            MusicBrainzAlias {
                name: "Secondary Alias".to_string(),
                sort_name: "Alias, Secondary".to_string(),
                alias_type: Some("Artist name".to_string()),
                locale: None,
                primary: Some(false),
            },
            MusicBrainzAlias {
                name: "Other Type Alias".to_string(),
                sort_name: "Alias, Other".to_string(),
                alias_type: Some("Legal name".to_string()),
                locale: None,
                primary: None,
            },
        ]),
        life_span: None,
        area: None,
        relations: None,
        score: Some(100),
    };
    
    let artist = client.convert_musicbrainz_artist(mb_artist);
    
    assert_eq!(artist.canonical_name, "Test Artist");
    assert_eq!(artist.aliases.len(), 3);
    
    // Check confidence scoring
    let primary_alias = artist.aliases.iter().find(|a| a.name == "Primary Alias").unwrap();
    assert_eq!(primary_alias.confidence, 0.95); // Primary alias gets highest confidence
    
    let secondary_alias = artist.aliases.iter().find(|a| a.name == "Secondary Alias").unwrap();
    assert_eq!(secondary_alias.confidence, 0.9); // Artist name type but not primary
    
    let other_alias = artist.aliases.iter().find(|a| a.name == "Other Type Alias").unwrap();
    assert_eq!(other_alias.confidence, 0.8); // Different type gets lower confidence
}

#[tokio::test]
async fn test_concurrent_entity_resolution_with_mock() {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};
    use music_streaming_blocklist_backend::services::{EntityResolutionService, ExternalApiService};
    use music_streaming_blocklist_backend::services::external_apis::MusicBrainzClient;
    use music_streaming_blocklist_backend::models::ArtistSearchQuery;
    
    let mock_server = MockServer::start().await;
    
    // Mock responses for multiple artists
    Mock::given(method("GET"))
        .and(path("/ws/2/artist"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"artists":[],"count":0,"offset":0}"#))
        .mount(&mock_server)
        .await;
    
    let mut musicbrainz_client = MusicBrainzClient::new();
    musicbrainz_client.base_url = mock_server.uri();
    
    let external_api_service = ExternalApiService {
        musicbrainz_client,
        isni_client: Default::default(),
    };
    
    let mut entity_service = EntityResolutionService::new();
    entity_service.external_api_service = external_api_service;
    
    // Test concurrent resolution
    let queries = vec![
        ArtistSearchQuery::new("Artist 1".to_string()),
        ArtistSearchQuery::new("Artist 2".to_string()),
        ArtistSearchQuery::new("Artist 3".to_string()),
    ];
    
    let results = entity_service.resolve_concurrent(&queries).await.unwrap();
    assert_eq!(results.len(), 3);
    
    // All requests should complete (though they may return empty results due to mock)
    for result_set in results {
        // Results may be empty due to mock returning empty response
        // The important thing is that concurrent processing works without errors
        println!("Concurrent resolution completed with {} results", result_set.len());
    }
}

// Test circuit breaker timeout and recovery
#[tokio::test]
async fn test_circuit_breaker_timeout_recovery() {
    let mut cb = CircuitBreaker::new(1, Duration::from_millis(50));
    
    // Trigger circuit breaker
    cb.record_failure();
    assert!(!cb.can_execute()); // Should be open
    
    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(60)).await;
    
    // Should be half-open now
    assert!(cb.can_execute());
    
    // Success should close it
    cb.record_success();
    assert!(cb.can_execute());
}

// Test external API service with both MusicBrainz and ISNI fallback
#[tokio::test]
async fn test_external_api_service_comprehensive_fallback() {
    let service = ExternalApiService::new();
    
    // Test with a very specific query that's unlikely to exist
    let result = service.search_artists_with_fallback("VerySpecificNonexistentArtistName12345", Some(3)).await;
    
    // Should handle gracefully regardless of network conditions
    assert!(result.is_ok());
    
    // Test artist enrichment with empty artist
    let mut empty_artist = Artist::new("Empty Artist".to_string());
    let enrich_result = service.enrich_artist(&mut empty_artist).await;
    assert!(result.is_ok());
}