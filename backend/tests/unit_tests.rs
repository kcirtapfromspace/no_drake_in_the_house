// Unit tests for core functionality that doesn't require external dependencies

use music_streaming_blocklist_backend::services::CircuitBreaker;
use std::time::Duration;

#[tokio::test]
async fn test_circuit_breaker_basic_functionality() {
    let mut cb = CircuitBreaker::new(2, Duration::from_millis(100));

    // Test initial state
    assert!(cb.can_execute());

    // Test success recording
    cb.record_success();
    assert!(cb.can_execute());

    // Test failure threshold
    cb.record_failure();
    assert!(cb.can_execute()); // Still closed after 1 failure

    cb.record_failure();
    assert!(!cb.can_execute()); // Now open after 2 failures

    // Test that it stays open
    assert!(!cb.can_execute());
}

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

#[tokio::test]
async fn test_circuit_breaker_half_open_failure() {
    let mut cb = CircuitBreaker::new(1, Duration::from_millis(50));

    // Trigger circuit breaker
    cb.record_failure();
    assert!(!cb.can_execute());

    // Wait for timeout to enter half-open
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert!(cb.can_execute());

    // Another failure should open it again
    cb.record_failure();
    assert!(!cb.can_execute());
}

// Test that demonstrates the external API integration structure is correct
#[test]
fn test_external_api_structure() {
    // This test verifies that the external API integration structure is properly designed
    // without requiring actual network calls

    // Test that we can create the clients (this tests the structure)
    use music_streaming_blocklist_backend::services::{
        ExternalApiService, IsniClient, MusicBrainzClient,
    };

    let mb_client = MusicBrainzClient::new();
    assert_eq!(mb_client.base_url, "https://musicbrainz.org/ws/2");

    let isni_client = IsniClient::new();
    assert_eq!(isni_client.base_url, "https://isni.oclc.org/sru");

    let service = ExternalApiService::new();
    assert_eq!(
        service.musicbrainz_client.base_url,
        "https://musicbrainz.org/ws/2"
    );
    assert_eq!(service.isni_client.base_url, "https://isni.oclc.org/sru");
}

// Test the ISNI URL extraction logic
#[test]
fn test_isni_url_extraction() {
    let client = music_streaming_blocklist_backend::services::MusicBrainzClient::new();

    // Test valid ISNI URL
    let isni_url = "https://isni.org/isni/0000000123456789";
    let extracted = client.extract_isni_from_url(isni_url);
    assert!(extracted.is_some());
    assert_eq!(extracted.unwrap(), "0000000123456789");

    // Test invalid URL
    let invalid_url = "https://example.com/not-isni";
    let extracted = client.extract_isni_from_url(invalid_url);
    assert!(extracted.is_none());

    // Test another valid ISNI URL format
    let isni_url2 = "https://isni.org/isni/0000000987654321";
    let extracted2 = client.extract_isni_from_url(isni_url2);
    assert!(extracted2.is_some());
    assert_eq!(extracted2.unwrap(), "0000000987654321");
}
