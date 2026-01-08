use music_streaming_blocklist_backend::*;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Contract tests for external API integrations
/// These tests verify that our code can handle changes in external API contracts

#[tokio::test]
async fn test_spotify_api_contract_changes() {
    // Test various Spotify API contract changes that could break our integration

    let test_cases = vec![
        SpotifyContractTest {
            name: "Missing items field in tracks response",
            endpoint: "/v1/me/tracks",
            response: json!({
                "tracks": [], // Changed from "items" to "tracks"
                "total": 0
            }),
            should_fail: true,
            error_contains: "missing field",
        },
        SpotifyContractTest {
            name: "Changed artist ID field name",
            endpoint: "/v1/me/tracks",
            response: json!({
                "items": [{
                    "track": {
                        "id": "track1",
                        "name": "Test Song",
                        "artists": [{
                            "artist_id": "artist1", // Changed from "id" to "artist_id"
                            "name": "Drake"
                        }]
                    }
                }],
                "total": 1
            }),
            should_fail: true,
            error_contains: "missing field",
        },
        SpotifyContractTest {
            name: "Additional required field",
            endpoint: "/v1/me/tracks",
            response: json!({
                "items": [{
                    "track": {
                        "id": "track1",
                        "name": "Test Song",
                        "artists": [{
                            "id": "artist1",
                            "name": "Drake"
                        }],
                        "required_new_field": "value" // New required field
                    }
                }],
                "total": 1
            }),
            should_fail: false, // Should handle gracefully
            error_contains: "",
        },
        SpotifyContractTest {
            name: "Changed error response format",
            endpoint: "/v1/me/tracks",
            response: json!({
                "error_details": { // Changed from "error" to "error_details"
                    "status": 401,
                    "message": "Unauthorized"
                }
            }),
            should_fail: true,
            error_contains: "unexpected response",
        },
        SpotifyContractTest {
            name: "Nested structure change",
            endpoint: "/v1/me/tracks",
            response: json!({
                "items": [{
                    "track_data": { // Changed from "track" to "track_data"
                        "id": "track1",
                        "name": "Test Song",
                        "artists": [{
                            "id": "artist1",
                            "name": "Drake"
                        }]
                    }
                }],
                "total": 1
            }),
            should_fail: true,
            error_contains: "missing field",
        },
    ];

    for test_case in test_cases {
        println!("Testing contract change: {}", test_case.name);

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path(test_case.endpoint))
            .respond_with(ResponseTemplate::new(200).set_body_json(test_case.response))
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
        let result = library_service.scan_user_library(&connection).await;

        if test_case.should_fail {
            assert!(result.is_err(), "Expected failure for: {}", test_case.name);
            if !test_case.error_contains.is_empty() {
                let error_msg = result.unwrap_err().to_string();
                assert!(
                    error_msg
                        .to_lowercase()
                        .contains(&test_case.error_contains.to_lowercase()),
                    "Error message '{}' should contain '{}'",
                    error_msg,
                    test_case.error_contains
                );
            }
        } else {
            assert!(result.is_ok(), "Expected success for: {}", test_case.name);
        }
    }
}

#[tokio::test]
async fn test_apple_music_api_contract_changes() {
    let test_cases = vec![
        AppleMusicContractTest {
            name: "Changed data structure",
            endpoint: "/v1/me/library/songs",
            response: json!({
                "results": [{ // Changed from "data" to "results"
                    "id": "i.song1",
                    "type": "library-songs",
                    "attributes": {
                        "name": "Test Song",
                        "artistName": "Drake"
                    }
                }]
            }),
            should_fail: true,
            error_contains: "missing field",
        },
        AppleMusicContractTest {
            name: "Changed attribute names",
            endpoint: "/v1/me/library/songs",
            response: json!({
                "data": [{
                    "id": "i.song1",
                    "type": "library-songs",
                    "attributes": {
                        "title": "Test Song", // Changed from "name" to "title"
                        "artist": "Drake" // Changed from "artistName" to "artist"
                    }
                }]
            }),
            should_fail: true,
            error_contains: "missing field",
        },
        AppleMusicContractTest {
            name: "Additional metadata fields",
            endpoint: "/v1/me/library/songs",
            response: json!({
                "data": [{
                    "id": "i.song1",
                    "type": "library-songs",
                    "attributes": {
                        "name": "Test Song",
                        "artistName": "Drake",
                        "newMetadataField": "value",
                        "anotherNewField": 123
                    }
                }]
            }),
            should_fail: false, // Should handle gracefully
            error_contains: "",
        },
    ];

    for test_case in test_cases {
        println!("Testing Apple Music contract change: {}", test_case.name);

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path(test_case.endpoint))
            .respond_with(ResponseTemplate::new(200).set_body_json(test_case.response))
            .expect(1)
            .mount(&mock_server)
            .await;

        let token_vault = Arc::new(TokenVaultService::new());
        let mut config = AppleMusicConfig::default();
        config.api_base_url = format!("{}/v1", mock_server.uri());

        let apple_service = AppleMusicService::new(config, token_vault.clone()).unwrap();
        let user_id = Uuid::new_v4();
        let connection = create_mock_apple_music_connection(user_id, &token_vault).await;

        let library_service = AppleMusicLibraryService::new(apple_service);
        let result = library_service.scan_user_library(&connection).await;

        if test_case.should_fail {
            assert!(result.is_err(), "Expected failure for: {}", test_case.name);
            if !test_case.error_contains.is_empty() {
                let error_msg = result.unwrap_err().to_string();
                assert!(
                    error_msg
                        .to_lowercase()
                        .contains(&test_case.error_contains.to_lowercase()),
                    "Error message '{}' should contain '{}'",
                    error_msg,
                    test_case.error_contains
                );
            }
        } else {
            assert!(result.is_ok(), "Expected success for: {}", test_case.name);
        }
    }
}

#[tokio::test]
async fn test_musicbrainz_api_contract_changes() {
    let test_cases = vec![
        MusicBrainzContractTest {
            name: "Changed artist array field name",
            response: json!({
                "artist_results": [{ // Changed from "artists" to "artist_results"
                    "id": "b10bbbfc-cf9e-42e0-be17-e2c3e1d2600d",
                    "name": "The Beatles",
                    "score": 100
                }],
                "count": 1
            }),
            should_fail: true,
            error_contains: "missing field",
        },
        MusicBrainzContractTest {
            name: "Changed score field type",
            response: json!({
                "artists": [{
                    "id": "b10bbbfc-cf9e-42e0-be17-e2c3e1d2600d",
                    "name": "The Beatles",
                    "score": "100" // Changed from number to string
                }],
                "count": 1
            }),
            should_fail: false, // Should handle type coercion
            error_contains: "",
        },
        MusicBrainzContractTest {
            name: "Added required pagination fields",
            response: json!({
                "artists": [{
                    "id": "b10bbbfc-cf9e-42e0-be17-e2c3e1d2600d",
                    "name": "The Beatles",
                    "score": 100
                }],
                "count": 1,
                "offset": 0,
                "limit": 25,
                "has_more": false // New required field
            }),
            should_fail: false, // Should handle additional fields
            error_contains: "",
        },
        MusicBrainzContractTest {
            name: "Removed optional fields",
            response: json!({
                "artists": [{
                    "id": "b10bbbfc-cf9e-42e0-be17-e2c3e1d2600d",
                    "name": "The Beatles"
                    // Removed optional "score", "disambiguation", etc.
                }],
                "count": 1
            }),
            should_fail: false, // Should handle missing optional fields
            error_contains: "",
        },
    ];

    for test_case in test_cases {
        println!("Testing MusicBrainz contract change: {}", test_case.name);

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/ws/2/artist"))
            .respond_with(ResponseTemplate::new(200).set_body_json(test_case.response))
            .expect(1)
            .mount(&mock_server)
            .await;

        let mut client = MusicBrainzClient::new();
        client.base_url = mock_server.uri();

        let result = client.search_artists("The Beatles", Some(10)).await;

        if test_case.should_fail {
            assert!(result.is_err(), "Expected failure for: {}", test_case.name);
            if !test_case.error_contains.is_empty() {
                let error_msg = result.unwrap_err().to_string();
                assert!(
                    error_msg
                        .to_lowercase()
                        .contains(&test_case.error_contains.to_lowercase()),
                    "Error message '{}' should contain '{}'",
                    error_msg,
                    test_case.error_contains
                );
            }
        } else {
            assert!(result.is_ok(), "Expected success for: {}", test_case.name);
        }
    }
}

#[tokio::test]
async fn test_http_status_code_changes() {
    // Test handling of different HTTP status codes that might change

    let status_tests = vec![
        (200, true, "Success"),
        (201, true, "Created - should handle as success"),
        (204, true, "No Content - should handle as success"),
        (400, false, "Bad Request"),
        (401, false, "Unauthorized"),
        (403, false, "Forbidden"),
        (404, false, "Not Found"),
        (429, false, "Rate Limited"),
        (500, false, "Internal Server Error"),
        (502, false, "Bad Gateway"),
        (503, false, "Service Unavailable"),
    ];

    for (status_code, should_succeed, description) in status_tests {
        println!("Testing HTTP status {}: {}", status_code, description);

        let mock_server = MockServer::start().await;

        let response_body = if status_code == 204 {
            // No content for 204
            ResponseTemplate::new(status_code)
        } else if status_code >= 400 {
            // Error response
            ResponseTemplate::new(status_code).set_body_json(json!({
                "error": {
                    "status": status_code,
                    "message": description
                }
            }))
        } else {
            // Success response
            ResponseTemplate::new(status_code).set_body_json(json!({
                "items": [],
                "total": 0
            }))
        };

        Mock::given(method("GET"))
            .and(path("/v1/me/tracks"))
            .respond_with(response_body)
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
        let result = library_service.scan_user_library(&connection).await;

        if should_succeed {
            assert!(
                result.is_ok(),
                "Expected success for status {}: {}",
                status_code,
                description
            );
        } else {
            assert!(
                result.is_err(),
                "Expected failure for status {}: {}",
                status_code,
                description
            );
        }
    }
}

#[tokio::test]
async fn test_content_type_changes() {
    // Test handling of different content types

    let content_type_tests = vec![
        ("application/json", true, "Standard JSON"),
        ("application/json; charset=utf-8", true, "JSON with charset"),
        ("text/json", true, "Alternative JSON content type"),
        ("application/xml", false, "XML instead of JSON"),
        ("text/plain", false, "Plain text"),
        ("text/html", false, "HTML response"),
    ];

    for (content_type, should_succeed, description) in content_type_tests {
        println!("Testing content type '{}': {}", content_type, description);

        let mock_server = MockServer::start().await;

        let response_body = if content_type.contains("json") {
            json!({
                "items": [],
                "total": 0
            })
            .to_string()
        } else if content_type == "application/xml" {
            r#"<?xml version="1.0"?><response><items></items><total>0</total></response>"#
                .to_string()
        } else {
            "Plain text response".to_string()
        };

        Mock::given(method("GET"))
            .and(path("/v1/me/tracks"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", content_type)
                    .set_body_string(response_body),
            )
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
        let result = library_service.scan_user_library(&connection).await;

        if should_succeed {
            assert!(
                result.is_ok(),
                "Expected success for content type '{}': {}",
                content_type,
                description
            );
        } else {
            assert!(
                result.is_err(),
                "Expected failure for content type '{}': {}",
                content_type,
                description
            );
        }
    }
}

#[tokio::test]
async fn test_rate_limit_header_changes() {
    // Test handling of different rate limit header formats

    let rate_limit_tests = vec![
        RateLimitTest {
            name: "Standard Spotify headers",
            headers: vec![
                ("X-RateLimit-Remaining", "100"),
                ("X-RateLimit-Limit", "1000"),
                ("X-RateLimit-Reset", "1640995200"),
            ],
            should_parse: true,
        },
        RateLimitTest {
            name: "GitHub-style headers",
            headers: vec![
                ("X-RateLimit-Remaining", "100"),
                ("X-RateLimit-Limit", "1000"),
                ("X-RateLimit-Reset", "1640995200"),
            ],
            should_parse: true,
        },
        RateLimitTest {
            name: "Twitter-style headers",
            headers: vec![
                ("x-rate-limit-remaining", "100"), // lowercase
                ("x-rate-limit-limit", "1000"),
                ("x-rate-limit-reset", "1640995200"),
            ],
            should_parse: true,
        },
        RateLimitTest {
            name: "Custom header names",
            headers: vec![
                ("Rate-Limit-Remaining", "100"),
                ("Rate-Limit-Total", "1000"),
                ("Rate-Limit-Reset-Time", "1640995200"),
            ],
            should_parse: false, // Different header names
        },
        RateLimitTest {
            name: "Missing headers",
            headers: vec![],
            should_parse: false,
        },
    ];

    for test_case in rate_limit_tests {
        println!("Testing rate limit headers: {}", test_case.name);

        let mock_server = MockServer::start().await;

        let mut response = ResponseTemplate::new(200).set_body_json(json!({
            "items": [],
            "total": 0
        }));

        for (header_name, header_value) in &test_case.headers {
            response = response.insert_header(header_name, header_value);
        }

        Mock::given(method("GET"))
            .and(path("/v1/me/tracks"))
            .respond_with(response)
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
        let result = library_service.scan_user_library(&connection).await;

        // Request should succeed regardless of rate limit header parsing
        assert!(
            result.is_ok(),
            "Request should succeed for: {}",
            test_case.name
        );

        // TODO: Add actual rate limit parsing verification
        // This would require exposing rate limit state from the service
    }
}

#[tokio::test]
async fn test_pagination_format_changes() {
    // Test handling of different pagination formats

    let pagination_tests = vec![
        PaginationTest {
            name: "Spotify-style pagination",
            response: json!({
                "items": [{"id": "1", "name": "Item 1"}],
                "total": 100,
                "limit": 50,
                "offset": 0,
                "next": "https://api.spotify.com/v1/me/tracks?offset=50&limit=50",
                "previous": null
            }),
            should_parse: true,
            expected_total: 100,
        },
        PaginationTest {
            name: "GitHub-style pagination",
            response: json!({
                "items": [{"id": "1", "name": "Item 1"}],
                "total_count": 100, // Different field name
                "per_page": 50,
                "page": 1
            }),
            should_parse: false, // Different field names
            expected_total: 0,
        },
        PaginationTest {
            name: "Cursor-based pagination",
            response: json!({
                "data": [{"id": "1", "name": "Item 1"}],
                "paging": {
                    "cursors": {
                        "before": "cursor_before",
                        "after": "cursor_after"
                    },
                    "next": "https://api.example.com/data?after=cursor_after"
                }
            }),
            should_parse: false, // Different structure
            expected_total: 0,
        },
    ];

    for test_case in pagination_tests {
        println!("Testing pagination format: {}", test_case.name);

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/me/tracks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(test_case.response))
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
        let result = library_service.scan_user_library(&connection).await;

        if test_case.should_parse {
            assert!(result.is_ok(), "Expected success for: {}", test_case.name);
            // TODO: Verify pagination was parsed correctly
        } else {
            // May succeed or fail depending on how gracefully we handle unknown formats
            println!("Result for {}: {:?}", test_case.name, result.is_ok());
        }
    }
}

// Helper structures

#[derive(Debug)]
struct SpotifyContractTest {
    name: &'static str,
    endpoint: &'static str,
    response: serde_json::Value,
    should_fail: bool,
    error_contains: &'static str,
}

#[derive(Debug)]
struct AppleMusicContractTest {
    name: &'static str,
    endpoint: &'static str,
    response: serde_json::Value,
    should_fail: bool,
    error_contains: &'static str,
}

#[derive(Debug)]
struct MusicBrainzContractTest {
    name: &'static str,
    response: serde_json::Value,
    should_fail: bool,
    error_contains: &'static str,
}

#[derive(Debug)]
struct RateLimitTest {
    name: &'static str,
    headers: Vec<(&'static str, &'static str)>,
    should_parse: bool,
}

#[derive(Debug)]
struct PaginationTest {
    name: &'static str,
    response: serde_json::Value,
    should_parse: bool,
    expected_total: u32,
}

// Helper functions

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
        refresh_token: None,
        scopes: vec!["library-read".to_string()],
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
    };

    token_vault.store_token(store_request).await.unwrap()
}
