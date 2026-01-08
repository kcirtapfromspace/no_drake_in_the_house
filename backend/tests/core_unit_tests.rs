// Core unit tests for basic functionality that doesn't require external dependencies
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

#[tokio::test]
async fn test_circuit_breaker_with_different_thresholds() {
    // Test with threshold of 3
    let mut cb3 = CircuitBreaker::new(3, Duration::from_millis(100));

    // Should remain closed through 2 failures
    cb3.record_failure();
    assert!(cb3.can_execute());
    cb3.record_failure();
    assert!(cb3.can_execute());

    // Should open on 3rd failure
    cb3.record_failure();
    assert!(!cb3.can_execute());

    // Test with threshold of 1
    let mut cb1 = CircuitBreaker::new(1, Duration::from_millis(100));

    // Should open immediately on first failure
    cb1.record_failure();
    assert!(!cb1.can_execute());
}

#[tokio::test]
async fn test_circuit_breaker_success_resets_failure_count() {
    let mut cb = CircuitBreaker::new(3, Duration::from_millis(100));

    // Record some failures but not enough to open
    cb.record_failure();
    cb.record_failure();
    assert!(cb.can_execute());

    // Success should reset failure count
    cb.record_success();

    // Should now take 3 more failures to open
    cb.record_failure();
    assert!(cb.can_execute());
    cb.record_failure();
    assert!(cb.can_execute());
    cb.record_failure();
    assert!(!cb.can_execute()); // Now open
}

#[tokio::test]
async fn test_circuit_breaker_concurrent_access() {
    use std::sync::{Arc, Mutex};

    let cb = Arc::new(Mutex::new(CircuitBreaker::new(
        2,
        Duration::from_millis(100),
    )));
    let mut handles = vec![];

    // Spawn multiple tasks that interact with the circuit breaker
    for i in 0..10 {
        let cb_clone = cb.clone();
        let handle = tokio::spawn(async move {
            let mut breaker = cb_clone.lock().unwrap();
            if i % 3 == 0 {
                breaker.record_failure();
            } else {
                breaker.record_success();
            }
            breaker.can_execute()
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // Verify all tasks completed successfully
    for result in results {
        assert!(result.is_ok());
    }
}

// Test basic data structures and models
#[test]
fn test_basic_model_creation() {
    use music_streaming_blocklist_backend::models::*;
    use uuid::Uuid;

    // Test User creation
    let user = User::new(
        "test@example.com".to_string(),
        Some("password_hash".to_string()),
    );
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.password_hash, Some("password_hash".to_string()));
    assert!(!user.totp_enabled);
    assert!(user.totp_secret.is_none());

    // Test Artist creation
    let artist = Artist::new("Test Artist".to_string());
    assert_eq!(artist.canonical_name, "Test Artist");
    assert!(artist.aliases.is_empty());

    // Test ExternalIds
    let mut external_ids = ExternalIds::new();
    external_ids.set_spotify_id("spotify_123".to_string());
    external_ids.set_apple_id("apple_456".to_string());

    assert_eq!(external_ids.get_spotify_id(), Some("spotify_123"));
    assert_eq!(external_ids.get_apple_id(), Some("apple_456"));
    assert_eq!(external_ids.get_musicbrainz_id(), None);
}

#[test]
fn test_external_ids_operations() {
    use music_streaming_blocklist_backend::models::*;

    let mut external_ids = ExternalIds::new();

    // Test setting and getting various provider IDs
    external_ids.set_spotify_id("spotify_test".to_string());
    external_ids.set_apple_id("apple_test".to_string());
    external_ids.set_musicbrainz_id("mb_test".to_string());
    external_ids.set_isni_id("isni_test".to_string());

    assert_eq!(external_ids.get_spotify_id(), Some("spotify_test"));
    assert_eq!(external_ids.get_apple_id(), Some("apple_test"));
    assert_eq!(external_ids.get_musicbrainz_id(), Some("mb_test"));
    assert_eq!(external_ids.get_isni_id(), Some("isni_test"));

    // Test get_all_ids
    let all_ids = external_ids.get_all_ids();
    assert!(all_ids.len() >= 4);
    assert!(all_ids.contains(&("spotify".to_string(), "spotify_test".to_string())));
    assert!(all_ids.contains(&("apple".to_string(), "apple_test".to_string())));
    assert!(all_ids.contains(&("musicbrainz".to_string(), "mb_test".to_string())));
    assert!(all_ids.contains(&("isni".to_string(), "isni_test".to_string())));
}

#[test]
fn test_artist_alias_operations() {
    use music_streaming_blocklist_backend::models::*;

    let mut artist = Artist::new("Main Artist".to_string());

    // Add aliases
    artist.aliases.push(ArtistAlias {
        name: "Alias 1".to_string(),
        source: "musicbrainz".to_string(),
        confidence: 0.9,
        locale: None,
    });

    artist.aliases.push(ArtistAlias {
        name: "Alias 2".to_string(),
        source: "spotify".to_string(),
        confidence: 0.7,
        locale: None,
    });

    assert_eq!(artist.aliases.len(), 2);

    // Test finding aliases by confidence
    let high_confidence_aliases: Vec<_> = artist
        .aliases
        .iter()
        .filter(|alias| alias.confidence > 0.8)
        .collect();
    assert_eq!(high_confidence_aliases.len(), 1);
    assert_eq!(high_confidence_aliases[0].name, "Alias 1");

    // Test finding aliases by source
    let mb_aliases: Vec<_> = artist
        .aliases
        .iter()
        .filter(|alias| alias.source == "musicbrainz")
        .collect();
    assert_eq!(mb_aliases.len(), 1);
    assert_eq!(mb_aliases[0].confidence, 0.9);
}

#[test]
fn test_connection_model() {
    use music_streaming_blocklist_backend::models::*;
    use uuid::Uuid;

    let user_id = Uuid::new_v4();
    let provider = StreamingProvider::Spotify;

    let connection = Connection::new(
        user_id,
        provider.clone(),
        "spotify_user_123".to_string(),
        vec!["read".to_string(), "write".to_string()],
    );

    assert_eq!(connection.user_id, user_id);
    assert_eq!(connection.provider, provider);
    assert_eq!(connection.provider_user_id, "spotify_user_123");
    assert_eq!(connection.scopes, vec!["read", "write"]);
    assert_eq!(connection.status, ConnectionStatus::Active);
    assert!(connection.access_token_encrypted.is_none());
    assert!(connection.refresh_token_encrypted.is_none());
}

#[test]
fn test_streaming_provider_enum() {
    use music_streaming_blocklist_backend::models::*;

    // Test enum variants
    let spotify = StreamingProvider::Spotify;
    let apple = StreamingProvider::Apple;

    // Test serialization/deserialization if implemented
    assert_ne!(spotify, apple);

    // Test that we can clone and compare
    let spotify_clone = spotify.clone();
    assert_eq!(spotify, spotify_clone);
}

#[test]
fn test_connection_status_enum() {
    use music_streaming_blocklist_backend::models::*;

    // Test enum variants
    let active = ConnectionStatus::Active;
    let error = ConnectionStatus::Error;
    let revoked = ConnectionStatus::Revoked;

    // Test that all variants are different
    assert_ne!(active, error);
    assert_ne!(active, revoked);
    assert_ne!(error, revoked);

    // Test cloning
    let active_clone = active.clone();
    assert_eq!(active, active_clone);
}

#[test]
fn test_user_model_oauth_providers() {
    use music_streaming_blocklist_backend::models::*;

    let mut user = User::new("oauth@example.com".to_string(), None);

    // Initially no OAuth providers
    assert_eq!(user.oauth_providers.len(), 0);

    // Add OAuth providers
    let google_provider = OAuthProviderInfo::new(
        OAuthProvider::Google,
        "google_user_123".to_string(),
        "oauth@example.com".to_string(),
        true,
    );
    let apple_provider = OAuthProviderInfo::new(
        OAuthProvider::Apple,
        "apple_user_456".to_string(),
        "oauth@example.com".to_string(),
        true,
    );

    user.oauth_providers.push(google_provider.clone());
    user.oauth_providers.push(apple_provider.clone());

    assert_eq!(user.oauth_providers.len(), 2);
    assert!(user.oauth_providers.contains(&google_provider));
    assert!(user.oauth_providers.contains(&apple_provider));
}

#[test]
fn test_claims_model() {
    use chrono::Utc;
    use music_streaming_blocklist_backend::models::*;
    use uuid::Uuid;

    let user_id = Uuid::new_v4();
    let now = Utc::now().timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        email: "claims@example.com".to_string(),
        exp: now + 3600, // 1 hour from now
        iat: now,
        jti: Uuid::new_v4().to_string(),
        token_type: TokenType::Access,
        scopes: vec!["read".to_string(), "write".to_string()],
    };

    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.email, "claims@example.com");
    assert_eq!(claims.token_type, TokenType::Access);
    assert!(claims.exp > claims.iat);
}

#[test]
fn test_token_pair_model() {
    use music_streaming_blocklist_backend::models::*;

    let token_pair = TokenPair {
        access_token: "access_token_123".to_string(),
        refresh_token: "refresh_token_456".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: 3600,
    };

    assert_eq!(token_pair.access_token, "access_token_123");
    assert_eq!(token_pair.refresh_token, "refresh_token_456");
    assert_eq!(token_pair.token_type, "Bearer");
    assert_eq!(token_pair.expires_in, 3600);
}

// Test utility functions and helpers
#[test]
fn test_name_normalization() {
    // Test basic name normalization logic
    fn normalize_name(name: &str) -> String {
        name.to_lowercase()
            .trim()
            .replace("the ", "")
            .replace(" ", "_")
    }

    assert_eq!(normalize_name("The Beatles"), "beatles");
    assert_eq!(normalize_name("  Drake  "), "drake");
    assert_eq!(normalize_name("Twenty One Pilots"), "twenty_one_pilots");
    assert_eq!(normalize_name("THE WEEKND"), "weeknd");
}

#[test]
fn test_confidence_scoring() {
    // Test confidence scoring logic
    fn calculate_confidence(source: &str, exact_match: bool) -> f64 {
        let base_score = match source {
            "musicbrainz" => 0.9,
            "spotify" => 0.8,
            "apple" => 0.8,
            "user_generated" => 0.5,
            _ => 0.3,
        };

        if exact_match {
            base_score
        } else {
            base_score * 0.7 // Reduce confidence for fuzzy matches
        }
    }

    assert_eq!(calculate_confidence("musicbrainz", true), 0.9);
    assert_eq!(calculate_confidence("spotify", true), 0.8);
    assert_eq!(calculate_confidence("user_generated", false), 0.35);
    assert_eq!(calculate_confidence("unknown", true), 0.3);
}

#[test]
fn test_levenshtein_distance() {
    // Test string similarity using levenshtein distance
    use levenshtein::levenshtein;

    // Identical strings
    assert_eq!(levenshtein("test", "test"), 0);

    // Single character difference
    assert_eq!(levenshtein("test", "best"), 1);

    // Multiple differences
    assert_eq!(levenshtein("kitten", "sitting"), 3);

    // Artist name variations
    assert_eq!(levenshtein("The Beatles", "Beatles"), 4); // "The " prefix
    assert_eq!(levenshtein("Twenty One Pilots", "21 Pilots"), 8);
}

#[test]
fn test_similarity_threshold() {
    use levenshtein::levenshtein;

    fn is_similar(name1: &str, name2: &str, threshold: f64) -> bool {
        let max_len = name1.len().max(name2.len());
        if max_len == 0 {
            return true;
        }

        let distance = levenshtein(name1, name2);
        let similarity = 1.0 - (distance as f64 / max_len as f64);
        similarity >= threshold
    }

    // Test similar names
    assert!(is_similar("The Beatles", "Beatles", 0.7));
    assert!(is_similar("Drake", "Drake", 1.0));
    assert!(!is_similar("Drake", "Taylor Swift", 0.7));

    // Test with different thresholds
    assert!(is_similar("Twenty One Pilots", "21 Pilots", 0.5));
    assert!(!is_similar("Twenty One Pilots", "21 Pilots", 0.8));
}
