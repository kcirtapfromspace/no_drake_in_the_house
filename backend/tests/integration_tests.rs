use music_streaming_blocklist_backend::services::{MusicBrainzClient, ExternalApiService};

#[tokio::test]
async fn test_musicbrainz_client_creation() {
    let client = MusicBrainzClient::new();
    // Just test that the client can be created without panicking
    assert_eq!(client.base_url, "https://musicbrainz.org/ws/2");
}

#[tokio::test]
async fn test_external_api_service_fallback() {
    let service = ExternalApiService::new();
    
    // Test with a query that should trigger fallback behavior
    let result = service.search_artists_with_fallback("NonexistentArtist12345", Some(5)).await;
    
    // Should not panic and should return empty results when APIs are unavailable
    assert!(result.is_ok());
    let artists = result.unwrap();
    // May be empty due to network issues or rate limiting, which is expected
    println!("Found {} artists for nonexistent query", artists.len());
}

#[tokio::test]
async fn test_circuit_breaker_behavior() {
    use music_streaming_blocklist_backend::services::CircuitBreaker;
    use std::time::Duration;

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

#[tokio::test]
async fn test_entity_resolution_with_external_apis() {
    use music_streaming_blocklist_backend::services::EntityResolutionService;
    use music_streaming_blocklist_backend::models::ArtistSearchQuery;

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

// Test the integration between entity resolution and external APIs
#[tokio::test]
async fn test_artist_enrichment() {
    use music_streaming_blocklist_backend::models::{Artist, ExternalIds};
    use music_streaming_blocklist_backend::services::ExternalApiService;

    let service = ExternalApiService::new();
    
    // Create an artist with minimal data
    let mut artist = Artist::with_external_ids(
        "Test Artist".to_string(),
        ExternalIds::new().with_musicbrainz("test-mbid-123".to_string())
    );
    
    // Try to enrich it (may fail due to invalid MBID, which is expected)
    let result = service.enrich_artist(&mut artist).await;
    
    // Should not panic even with invalid data
    assert!(result.is_ok());
    println!("Artist enrichment completed for: {}", artist.canonical_name);
}