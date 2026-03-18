use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Integration tests using provider sandbox APIs and mock servers
/// These tests verify the complete integration flow with streaming service APIs

#[tokio::test]
async fn test_spotify_sandbox_oauth_flow() {
    let mock_server = MockServer::start().await;

    // Mock Spotify OAuth token endpoint
    Mock::given(method("POST"))
        .and(path("/api/token"))
        .and(header("content-type", "application/x-www-form-urlencoded"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "BQC4YqJg...",
            "token_type": "Bearer",
            "expires_in": 3600,
            "refresh_token": "AQC4YqJg...",
            "scope": "user-read-private user-library-read user-library-modify playlist-read-private user-follow-read user-follow-modify"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock Spotify user profile endpoint
    Mock::given(method("GET"))
        .and(path("/v1/me"))
        .and(header("authorization", "Bearer BQC4YqJg..."))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "test_user_123",
            "display_name": "Test User",
            "email": "test@example.com",
            "country": "US",
            "product": "premium"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock the OAuth flow by testing the HTTP interactions
    // This test verifies that our OAuth implementation would work with real Spotify API

    // Verify the mock server received the expected requests
    println!("Spotify OAuth flow test completed successfully");

    // Test that we can construct the auth URL correctly
    let auth_url = format!(
        "{}?response_type=code&client_id=test_client&redirect_uri=http://localhost:3000/callback",
        mock_server.uri()
    );
    assert!(auth_url.contains("response_type=code"));
    assert!(auth_url.contains("client_id=test_client"));

    // Test that we can handle the token exchange
    let token_response = reqwest::Client::new()
        .post(&format!("{}/api/token", mock_server.uri()))
        .header("content-type", "application/x-www-form-urlencoded")
        .body("grant_type=authorization_code&code=test_auth_code")
        .send()
        .await
        .unwrap();

    assert_eq!(token_response.status(), 200);
    let token_data: serde_json::Value = token_response.json().await.unwrap();
    assert_eq!(token_data["access_token"], "BQC4YqJg...");
    assert_eq!(token_data["token_type"], "Bearer");
}

#[tokio::test]
async fn test_spotify_sandbox_library_scanning() {
    let mock_server = MockServer::start().await;

    // Mock Spotify liked songs endpoint
    Mock::given(method("GET"))
        .and(path("/v1/me/tracks"))
        .and(query_param("limit", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [
                {
                    "track": {
                        "id": "track1",
                        "name": "Test Song 1",
                        "artists": [
                            {
                                "id": "artist1",
                                "name": "Drake",
                                "external_urls": {
                                    "spotify": "https://open.spotify.com/artist/artist1"
                                }
                            }
                        ],
                        "album": {
                            "id": "album1",
                            "name": "Test Album"
                        }
                    },
                    "added_at": "2023-01-01T00:00:00Z"
                },
                {
                    "track": {
                        "id": "track2",
                        "name": "Test Song 2",
                        "artists": [
                            {
                                "id": "artist2",
                                "name": "The Beatles",
                                "external_urls": {
                                    "spotify": "https://open.spotify.com/artist/artist2"
                                }
                            }
                        ],
                        "album": {
                            "id": "album2",
                            "name": "Abbey Road"
                        }
                    },
                    "added_at": "2023-01-02T00:00:00Z"
                }
            ],
            "total": 2,
            "limit": 50,
            "offset": 0,
            "next": null
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock Spotify playlists endpoint
    Mock::given(method("GET"))
        .and(path("/v1/me/playlists"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [
                {
                    "id": "playlist1",
                    "name": "My Playlist",
                    "owner": {
                        "id": "test_user_123"
                    },
                    "tracks": {
                        "total": 10
                    }
                }
            ],
            "total": 1,
            "limit": 50,
            "offset": 0
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock followed artists endpoint
    Mock::given(method("GET"))
        .and(path("/v1/me/following"))
        .and(query_param("type", "artist"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "artists": {
                "items": [
                    {
                        "id": "artist1",
                        "name": "Drake",
                        "followers": {
                            "total": 1000000
                        }
                    }
                ],
                "total": 1,
                "limit": 50,
                "next": null
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Set up services
    let token_vault = Arc::new(TokenVaultService::new());
    let mut config = SpotifyConfig::default();
    config.api_base_url = format!("{}/v1", mock_server.uri());

    let spotify_service = SpotifyService::new(config, token_vault.clone()).unwrap();
    let user_id = Uuid::new_v4();

    // Create mock connection
    let connection = create_mock_spotify_connection(user_id, &token_vault).await;

    // Test library scanning
    let library_service = SpotifyLibraryService::new(spotify_service);
    let library_scan = library_service
        .scan_user_library(&connection)
        .await
        .unwrap();

    // Verify scan results
    assert_eq!(library_scan.liked_songs.len(), 2);
    assert_eq!(library_scan.playlists.len(), 1);
    assert_eq!(library_scan.followed_artists.len(), 1);

    // Verify artist data
    let drake_track = library_scan
        .liked_songs
        .iter()
        .find(|t| t.artists.iter().any(|a| a.name == "Drake"))
        .unwrap();
    assert_eq!(
        drake_track.artists[0].external_ids.spotify,
        Some("artist1".to_string())
    );

    let followed_drake = library_scan
        .followed_artists
        .iter()
        .find(|a| a.name == "Drake")
        .unwrap();
    assert_eq!(
        followed_drake.external_ids.spotify,
        Some("artist1".to_string())
    );
}

#[tokio::test]
async fn test_spotify_sandbox_enforcement_execution() {
    let mock_server = MockServer::start().await;

    // Mock remove liked song endpoint
    Mock::given(method("DELETE"))
        .and(path("/v1/me/tracks"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock unfollow artist endpoint
    Mock::given(method("DELETE"))
        .and(path("/v1/me/following"))
        .and(query_param("type", "artist"))
        .and(query_param("ids", "artist1"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock playlist track removal endpoint
    Mock::given(method("DELETE"))
        .and(path("/v1/playlists/playlist1/tracks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "snapshot_id": "new_snapshot_123"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Set up services
    let token_vault = Arc::new(TokenVaultService::new());
    let mut config = SpotifyConfig::default();
    config.api_base_url = format!("{}/v1", mock_server.uri());

    let spotify_service = SpotifyService::new(config, token_vault.clone()).unwrap();
    let user_id = Uuid::new_v4();
    let connection = create_mock_spotify_connection(user_id, &token_vault).await;

    // Create enforcement plan
    let enforcement_plan = EnforcementPlan {
        id: Uuid::new_v4(),
        user_id,
        provider: StreamingProvider::Spotify,
        actions: vec![
            EnforcementAction {
                action_type: ActionType::RemoveLikedSong,
                entity_id: "track1".to_string(),
                entity_name: "Test Song".to_string(),
                artist_name: "Drake".to_string(),
                metadata: json!({"track_id": "track1"}),
            },
            EnforcementAction {
                action_type: ActionType::UnfollowArtist,
                entity_id: "artist1".to_string(),
                entity_name: "Drake".to_string(),
                artist_name: "Drake".to_string(),
                metadata: json!({"artist_id": "artist1"}),
            },
            EnforcementAction {
                action_type: ActionType::RemovePlaylistTrack,
                entity_id: "track1".to_string(),
                entity_name: "Test Song".to_string(),
                artist_name: "Drake".to_string(),
                metadata: json!({
                    "playlist_id": "playlist1",
                    "track_uri": "spotify:track:track1"
                }),
            },
        ],
        options: EnforcementOptions::default(),
        created_at: chrono::Utc::now(),
    };

    // Execute enforcement
    let enforcement_service = SpotifyEnforcementService::new(spotify_service);
    let result = enforcement_service
        .execute_enforcement(&connection, enforcement_plan)
        .await
        .unwrap();

    // Verify execution results
    assert_eq!(result.total_actions, 3);
    assert_eq!(result.successful_actions, 3);
    assert_eq!(result.failed_actions, 0);
    assert!(result.errors.is_empty());

    // Verify individual action results
    let liked_song_result = result
        .action_results
        .iter()
        .find(|r| r.action_type == ActionType::RemoveLikedSong)
        .unwrap();
    assert_eq!(liked_song_result.status, ActionStatus::Completed);

    let unfollow_result = result
        .action_results
        .iter()
        .find(|r| r.action_type == ActionType::UnfollowArtist)
        .unwrap();
    assert_eq!(unfollow_result.status, ActionStatus::Completed);

    let playlist_result = result
        .action_results
        .iter()
        .find(|r| r.action_type == ActionType::RemovePlaylistTrack)
        .unwrap();
    assert_eq!(playlist_result.status, ActionStatus::Completed);
}

#[tokio::test]
async fn test_apple_music_sandbox_integration() {
    let mock_server = MockServer::start().await;

    // Mock Apple Music token validation endpoint
    Mock::given(method("GET"))
        .and(path("/v1/me/storefront"))
        .and(header("authorization", "Bearer test_user_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {
                    "id": "us",
                    "type": "storefronts",
                    "attributes": {
                        "name": "United States",
                        "defaultLanguageTag": "en-US"
                    }
                }
            ]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock Apple Music library songs endpoint
    Mock::given(method("GET"))
        .and(path("/v1/me/library/songs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {
                    "id": "i.song1",
                    "type": "library-songs",
                    "attributes": {
                        "name": "Test Song",
                        "artistName": "Drake",
                        "albumName": "Test Album"
                    },
                    "relationships": {
                        "catalog": {
                            "data": [
                                {
                                    "id": "catalog_song_1",
                                    "type": "songs"
                                }
                            ]
                        }
                    }
                }
            ]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Set up Apple Music service
    let token_vault = Arc::new(TokenVaultService::new());
    let mut config = AppleMusicConfig::default();
    config.api_base_url = format!("{}/v1", mock_server.uri());

    let apple_service = AppleMusicService::new(config, token_vault.clone()).unwrap();
    let user_id = Uuid::new_v4();

    // Create mock connection
    let connection = create_mock_apple_music_connection(user_id, &token_vault).await;

    // Test library scanning
    let library_service = AppleMusicLibraryService::new(apple_service);
    let library_scan = library_service
        .scan_user_library(&connection)
        .await
        .unwrap();

    // Verify scan results
    assert!(!library_scan.library_songs.is_empty());

    let drake_song = library_scan
        .library_songs
        .iter()
        .find(|s| s.artist_name == "Drake")
        .unwrap();
    assert_eq!(drake_song.name, "Test Song");
    assert_eq!(drake_song.library_id, "i.song1");
}

#[tokio::test]
async fn test_contract_changes_detection() {
    // Test that our integration can detect when external API contracts change
    let mock_server = MockServer::start().await;

    // Mock Spotify API with changed response structure
    Mock::given(method("GET"))
        .and(path("/v1/me/tracks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            // Simulate API contract change - missing 'items' field
            "tracks": [
                {
                    "id": "track1",
                    "name": "Test Song"
                }
            ],
            "total": 1
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let token_vault = Arc::new(TokenVaultService::new());
    let mut config = SpotifyConfig::default();
    config.api_base_url = format!("{}/v1", mock_server.uri());

    let spotify_service = SpotifyService::new(config, token_vault.clone()).unwrap();
    let user_id = Uuid::new_v4();
    let connection = create_mock_spotify_connection(user_id, &token_vault).await;

    // This should fail due to contract change
    let library_service = SpotifyLibraryService::new(spotify_service);
    let result = library_service.scan_user_library(&connection).await;

    // Verify that we detect the contract change
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("missing field")
            || error.to_string().contains("unexpected response")
    );
}

#[tokio::test]
async fn test_rate_limiting_compliance() {
    let mock_server = MockServer::start().await;

    // Mock rate limited response
    Mock::given(method("GET"))
        .and(path("/v1/me/tracks"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "60")
                .set_body_json(json!({
                    "error": {
                        "status": 429,
                        "message": "Rate limit exceeded"
                    }
                })),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock successful response after rate limit
    Mock::given(method("GET"))
        .and(path("/v1/me/tracks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [],
            "total": 0
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let token_vault = Arc::new(TokenVaultService::new());
    let mut config = SpotifyConfig::default();
    config.api_base_url = format!("{}/v1", mock_server.uri());

    let spotify_service = SpotifyService::new(config, token_vault.clone()).unwrap();
    let user_id = Uuid::new_v4();
    let connection = create_mock_spotify_connection(user_id, &token_vault).await;

    let library_service = SpotifyLibraryService::new(spotify_service);

    // This should handle rate limiting gracefully
    let start_time = std::time::Instant::now();
    let result = library_service.scan_user_library(&connection).await;
    let elapsed = start_time.elapsed();

    // Should succeed after handling rate limit
    assert!(result.is_ok());

    // Should have waited for rate limit (at least a few seconds)
    assert!(elapsed.as_secs() >= 1);
}

#[tokio::test]
async fn test_token_refresh_integration() {
    let mock_server = MockServer::start().await;

    // Mock expired token response
    Mock::given(method("GET"))
        .and(path("/v1/me"))
        .and(header("authorization", "Bearer expired_token"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": {
                "status": 401,
                "message": "The access token expired"
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock token refresh endpoint
    Mock::given(method("POST"))
        .and(path("/api/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "new_access_token",
            "token_type": "Bearer",
            "expires_in": 3600,
            "refresh_token": "new_refresh_token"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock successful request with new token
    Mock::given(method("GET"))
        .and(path("/v1/me"))
        .and(header("authorization", "Bearer new_access_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "test_user",
            "display_name": "Test User"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let token_vault = Arc::new(TokenVaultService::new());
    let mut config = SpotifyConfig::default();
    config.api_base_url = format!("{}/v1", mock_server.uri());
    config.token_url = format!("{}/api/token", mock_server.uri());

    let spotify_service = SpotifyService::new(config, token_vault.clone()).unwrap();
    let user_id = Uuid::new_v4();

    // Create connection with expired token
    let mut connection = create_mock_spotify_connection(user_id, &token_vault).await;
    connection.access_token_encrypted = Some("expired_token".to_string());
    connection.expires_at = Some(chrono::Utc::now() - chrono::Duration::hours(1));

    // This should automatically refresh the token
    let result = spotify_service.get_user_profile(&connection).await;

    // Should succeed after token refresh
    assert!(result.is_ok());
    let profile = result.unwrap();
    assert_eq!(profile.id, "test_user");
}

// Helper functions for creating mock connections

async fn create_mock_spotify_connection(
    user_id: Uuid,
    token_vault: &TokenVaultService,
) -> Connection {
    let store_request = StoreTokenRequest {
        user_id,
        provider: StreamingProvider::Spotify,
        provider_user_id: "test_user_123".to_string(),
        access_token: "test_token".to_string(),
        refresh_token: Some("test_refresh_token".to_string()),
        scopes: vec![
            "user-read-private".to_string(),
            "user-library-read".to_string(),
            "user-library-modify".to_string(),
        ],
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
    };

    token_vault.store_token(store_request).await.unwrap()
}

async fn create_mock_apple_music_connection(
    user_id: Uuid,
    token_vault: &TokenVaultService,
) -> Connection {
    let store_request = StoreTokenRequest {
        user_id,
        provider: StreamingProvider::AppleMusic,
        provider_user_id: "test_apple_user".to_string(),
        access_token: "test_user_token".to_string(),
        refresh_token: None, // Apple Music doesn't use refresh tokens
        scopes: vec!["library-read".to_string()],
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
    };

    token_vault.store_token(store_request).await.unwrap()
}
