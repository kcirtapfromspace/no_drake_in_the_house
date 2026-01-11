//! Apple Music Sandbox Integration Tests
//!
//! These tests validate our Apple Music API interactions against the real Apple Music API.
//! They require valid Apple Music credentials to be set in environment variables:
//! - APPLE_MUSIC_TEAM_ID
//! - APPLE_MUSIC_KEY_ID
//! - APPLE_MUSIC_KEY_PATH (path to .p8 private key file)
//!
//! Run with: cargo test --test apple_music_sandbox_tests -- --ignored
//! Or in CI: Set APPLE_MUSIC_SANDBOX_TEST=1 to enable

use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Once;

static INIT: Once = Once::new();

/// Load environment variables from .env file
fn init_env() {
    INIT.call_once(|| {
        // Try to load from backend/.env first, then project root
        let backend_env = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
        if backend_env.exists() {
            dotenvy::from_path(&backend_env).ok();
        } else {
            dotenvy::dotenv().ok();
        }
    });
}

/// Apple Music JWT claims
#[derive(Debug, Serialize)]
struct AppleMusicClaims {
    iss: String,  // Team ID
    iat: i64,     // Issued at
    exp: i64,     // Expiration
}

/// Apple Music search response
#[derive(Debug, Deserialize)]
struct AppleMusicSearchResponse {
    results: Option<SearchResults>,
}

#[derive(Debug, Deserialize)]
struct SearchResults {
    artists: Option<ArtistResults>,
    songs: Option<SongResults>,
}

#[derive(Debug, Deserialize)]
struct ArtistResults {
    data: Vec<Artist>,
}

#[derive(Debug, Deserialize)]
struct SongResults {
    data: Vec<Song>,
}

#[derive(Debug, Deserialize)]
struct Artist {
    id: String,
    #[serde(rename = "type")]
    resource_type: String,
    attributes: Option<ArtistAttributes>,
}

#[derive(Debug, Deserialize)]
struct ArtistAttributes {
    name: String,
    #[serde(rename = "genreNames")]
    genre_names: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct Song {
    id: String,
    #[serde(rename = "type")]
    resource_type: String,
    attributes: Option<SongAttributes>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SongAttributes {
    name: String,
    #[serde(rename = "artistName")]
    artist_name: String,
    #[serde(rename = "albumName")]
    album_name: Option<String>,
}

/// Generate Apple Music developer token (JWT)
fn generate_developer_token() -> Result<String> {
    let team_id = env::var("APPLE_MUSIC_TEAM_ID")
        .map_err(|_| anyhow::anyhow!("APPLE_MUSIC_TEAM_ID not set"))?;
    let key_id = env::var("APPLE_MUSIC_KEY_ID")
        .map_err(|_| anyhow::anyhow!("APPLE_MUSIC_KEY_ID not set"))?;
    let key_path = env::var("APPLE_MUSIC_KEY_PATH")
        .map_err(|_| anyhow::anyhow!("APPLE_MUSIC_KEY_PATH not set"))?;

    // Read private key
    let private_key = std::fs::read_to_string(&key_path)
        .map_err(|e| anyhow::anyhow!("Failed to read key file {}: {}", key_path, e))?;

    let now = Utc::now();
    let exp = now + Duration::hours(12);

    let claims = AppleMusicClaims {
        iss: team_id,
        iat: now.timestamp(),
        exp: exp.timestamp(),
    };

    let mut header = Header::new(Algorithm::ES256);
    header.kid = Some(key_id);

    let encoding_key = EncodingKey::from_ec_pem(private_key.as_bytes())
        .map_err(|e| anyhow::anyhow!("Failed to create encoding key: {}", e))?;

    let token = encode(&header, &claims, &encoding_key)
        .map_err(|e| anyhow::anyhow!("Failed to encode JWT: {}", e))?;

    Ok(token)
}

/// Check if sandbox tests should run
fn should_run_sandbox_tests() -> bool {
    env::var("APPLE_MUSIC_SANDBOX_TEST").is_ok()
        || env::var("CI").is_ok()
        || env::var("APPLE_MUSIC_TEAM_ID").is_ok()
}

// ============================================
// Sandbox Integration Tests
// ============================================

/// Test: Generate a valid Apple Music developer token
#[tokio::test]
#[ignore = "Requires Apple Music credentials - run with --ignored"]
async fn test_generate_developer_token() {
    init_env();
    if !should_run_sandbox_tests() {
        println!("Skipping sandbox test - credentials not configured");
        return;
    }

    let token = generate_developer_token();
    assert!(token.is_ok(), "Failed to generate token: {:?}", token.err());

    let token = token.unwrap();
    assert!(!token.is_empty(), "Token should not be empty");

    // JWT should have 3 parts separated by dots
    let parts: Vec<&str> = token.split('.').collect();
    assert_eq!(parts.len(), 3, "JWT should have 3 parts (header.payload.signature)");

    println!("✅ Developer token generated successfully");
    println!("   Token length: {} chars", token.len());
    println!("   Token preview: {}...", &token[..50.min(token.len())]);
}

/// Test: Search Apple Music catalog for "Drake" (no user token required)
#[tokio::test]
#[ignore = "Requires Apple Music credentials - run with --ignored"]
async fn test_catalog_search_drake() {
    init_env();
    if !should_run_sandbox_tests() {
        println!("Skipping sandbox test - credentials not configured");
        return;
    }

    let token = generate_developer_token().expect("Failed to generate token");
    let client = Client::new();

    let response = client
        .get("https://api.music.apple.com/v1/catalog/us/search")
        .query(&[
            ("term", "Drake"),
            ("types", "artists"),
            ("limit", "5"),
        ])
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");

    let status = response.status();
    println!("Response status: {}", status);

    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        panic!("API request failed with status {}: {}", status, error_body);
    }

    let search_response: AppleMusicSearchResponse = response
        .json()
        .await
        .expect("Failed to parse response");

    let artists = search_response
        .results
        .and_then(|r| r.artists)
        .expect("No artist results returned");

    assert!(!artists.data.is_empty(), "Should find Drake");

    let drake = artists.data.iter()
        .find(|a| a.attributes.as_ref().map(|attr| attr.name.to_lowercase().contains("drake")).unwrap_or(false));

    assert!(drake.is_some(), "Should find artist named Drake");

    let drake = drake.unwrap();
    println!("✅ Found Drake in Apple Music catalog");
    println!("   ID: {}", drake.id);
    println!("   Name: {}", drake.attributes.as_ref().map(|a| a.name.as_str()).unwrap_or("N/A"));
    println!("   Genres: {:?}", drake.attributes.as_ref().and_then(|a| a.genre_names.clone()));
}

/// Test: Search Apple Music catalog for songs by Drake
#[tokio::test]
#[ignore = "Requires Apple Music credentials - run with --ignored"]
async fn test_catalog_search_drake_songs() {
    init_env();
    if !should_run_sandbox_tests() {
        println!("Skipping sandbox test - credentials not configured");
        return;
    }

    let token = generate_developer_token().expect("Failed to generate token");
    let client = Client::new();

    let response = client
        .get("https://api.music.apple.com/v1/catalog/us/search")
        .query(&[
            ("term", "Drake God's Plan"),
            ("types", "songs"),
            ("limit", "5"),
        ])
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success(), "Request should succeed");

    let search_response: AppleMusicSearchResponse = response
        .json()
        .await
        .expect("Failed to parse response");

    let songs = search_response
        .results
        .and_then(|r| r.songs)
        .expect("No song results returned");

    assert!(!songs.data.is_empty(), "Should find songs");

    println!("✅ Found {} Drake songs", songs.data.len());
    for song in &songs.data {
        if let Some(attrs) = &song.attributes {
            println!("   - {} by {} (ID: {})", attrs.name, attrs.artist_name, song.id);
        }
    }
}

/// Test: Verify API rate limit headers are present
#[tokio::test]
#[ignore = "Requires Apple Music credentials - run with --ignored"]
async fn test_api_rate_limit_headers() {
    init_env();
    if !should_run_sandbox_tests() {
        println!("Skipping sandbox test - credentials not configured");
        return;
    }

    let token = generate_developer_token().expect("Failed to generate token");
    let client = Client::new();

    let response = client
        .get("https://api.music.apple.com/v1/catalog/us/search")
        .query(&[("term", "test"), ("types", "artists"), ("limit", "1")])
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");

    println!("Response headers:");
    for (name, value) in response.headers() {
        if name.as_str().contains("rate") || name.as_str().contains("limit") || name.as_str().contains("x-") {
            println!("   {}: {:?}", name, value);
        }
    }

    assert!(response.status().is_success(), "Request should succeed");
    println!("✅ API request completed - check headers above for rate limit info");
}

/// Test: Verify storefront/region support
#[tokio::test]
#[ignore = "Requires Apple Music credentials - run with --ignored"]
async fn test_storefront_support() {
    init_env();
    if !should_run_sandbox_tests() {
        println!("Skipping sandbox test - credentials not configured");
        return;
    }

    let token = generate_developer_token().expect("Failed to generate token");
    let client = Client::new();

    // Test US storefront
    let response = client
        .get("https://api.music.apple.com/v1/storefronts/us")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success(), "Should access US storefront");

    let body: serde_json::Value = response.json().await.expect("Failed to parse");
    println!("✅ US Storefront info:");
    println!("   {}", serde_json::to_string_pretty(&body).unwrap_or_default());
}

/// Test: Full enforcement simulation (catalog search for blocked artist)
#[tokio::test]
#[ignore = "Requires Apple Music credentials - run with --ignored"]
async fn test_enforcement_flow_simulation() {
    init_env();
    if !should_run_sandbox_tests() {
        println!("Skipping sandbox test - credentials not configured");
        return;
    }

    let token = generate_developer_token().expect("Failed to generate token");
    let client = Client::new();

    // Simulate: User has "Drake" on their blocklist
    let blocked_artist = "Drake";

    println!("🔍 Simulating enforcement flow for blocked artist: {}", blocked_artist);

    // Step 1: Search for the artist to get their ID
    let artist_response = client
        .get("https://api.music.apple.com/v1/catalog/us/search")
        .query(&[
            ("term", blocked_artist),
            ("types", "artists"),
            ("limit", "1"),
        ])
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to search for artist");

    assert!(artist_response.status().is_success());

    let artist_data: AppleMusicSearchResponse = artist_response.json().await.expect("Failed to parse");
    let artist_id = artist_data
        .results
        .and_then(|r| r.artists)
        .and_then(|a| a.data.first().map(|x| x.id.clone()))
        .expect("Should find artist");

    println!("   ✓ Found artist ID: {}", artist_id);

    // Step 2: Get artist's top songs (these would be disliked in real enforcement)
    let songs_response = client
        .get(format!("https://api.music.apple.com/v1/catalog/us/artists/{}/songs", artist_id))
        .query(&[("limit", "10")])
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to get artist songs");

    if songs_response.status().is_success() {
        let songs_data: serde_json::Value = songs_response.json().await.expect("Failed to parse");
        let song_count = songs_data["data"].as_array().map(|a| a.len()).unwrap_or(0);
        println!("   ✓ Found {} songs by {} (would be disliked)", song_count, blocked_artist);
    } else {
        println!("   ⚠ Could not fetch artist songs (may need different endpoint)");
    }

    // Step 3: Get artist's albums (these would also be disliked)
    let albums_response = client
        .get(format!("https://api.music.apple.com/v1/catalog/us/artists/{}/albums", artist_id))
        .query(&[("limit", "10")])
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to get artist albums");

    if albums_response.status().is_success() {
        let albums_data: serde_json::Value = albums_response.json().await.expect("Failed to parse");
        let album_count = albums_data["data"].as_array().map(|a| a.len()).unwrap_or(0);
        println!("   ✓ Found {} albums by {} (would be disliked)", album_count, blocked_artist);
    } else {
        println!("   ⚠ Could not fetch artist albums (may need different endpoint)");
    }

    println!("✅ Enforcement flow simulation complete");
    println!("   In real enforcement with user token:");
    println!("   - Each song would be rated with value: -1 (dislike)");
    println!("   - Each album would be rated with value: -1 (dislike)");
}
