use music_streaming_blocklist_backend::*;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_spotify_oauth_flow() {
    // Initialize services
    let token_vault = Arc::new(TokenVaultService::new());
    let config = SpotifyConfig::default();
    let spotify_service = SpotifyService::new(config, token_vault.clone()).unwrap();

    // Test getting auth URL
    let auth_url_response = spotify_service.get_auth_url().await.unwrap();

    assert!(auth_url_response.auth_url.contains("accounts.spotify.com"));
    assert!(auth_url_response.auth_url.contains("client_id"));
    assert!(auth_url_response.auth_url.contains("code_challenge"));
    assert!(!auth_url_response.state.is_empty());

    // Verify the auth URL contains required scopes
    assert!(auth_url_response.auth_url.contains("user-read-private"));
    assert!(auth_url_response.auth_url.contains("user-library-read"));
    assert!(auth_url_response.auth_url.contains("user-library-modify"));
    assert!(auth_url_response.auth_url.contains("playlist-read-private"));
    assert!(auth_url_response.auth_url.contains("user-follow-read"));
}

#[tokio::test]
async fn test_spotify_connection_management() {
    let token_vault = Arc::new(TokenVaultService::new());
    let config = SpotifyConfig::default();
    let spotify_service = SpotifyService::new(config, token_vault.clone()).unwrap();

    let user_id = Uuid::new_v4();

    // Initially no connection should exist
    let connection = spotify_service.get_user_connection(user_id).await.unwrap();
    assert!(connection.is_none());

    // Store a mock connection directly via token vault
    let store_request = StoreTokenRequest {
        user_id,
        provider: StreamingProvider::Spotify,
        provider_user_id: "spotify_user_123".to_string(),
        access_token: "mock_access_token".to_string(),
        refresh_token: Some("mock_refresh_token".to_string()),
        scopes: vec![
            "user-read-private".to_string(),
            "user-library-read".to_string(),
            "user-library-modify".to_string(),
        ],
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
    };

    let stored_connection = token_vault.store_token(store_request).await.unwrap();

    // Now connection should exist
    let connection = spotify_service.get_user_connection(user_id).await.unwrap();
    assert!(connection.is_some());

    let connection = connection.unwrap();
    assert_eq!(connection.provider, StreamingProvider::Spotify);
    assert_eq!(connection.provider_user_id, "spotify_user_123");
    assert_eq!(connection.user_id, user_id);

    // Test disconnection
    spotify_service.disconnect_user(user_id).await.unwrap();

    // Connection should no longer exist
    let connection = spotify_service.get_user_connection(user_id).await.unwrap();
    assert!(connection.is_none());
}

#[tokio::test]
async fn test_spotify_token_health_check() {
    let token_vault = Arc::new(TokenVaultService::new());
    let config = SpotifyConfig::default();
    let spotify_service = SpotifyService::new(config, token_vault.clone()).unwrap();

    let user_id = Uuid::new_v4();

    // Store a connection with a token that expires soon
    let store_request = StoreTokenRequest {
        user_id,
        provider: StreamingProvider::Spotify,
        provider_user_id: "spotify_user_123".to_string(),
        access_token: "mock_access_token".to_string(),
        refresh_token: Some("mock_refresh_token".to_string()),
        scopes: vec!["user-read-private".to_string()],
        expires_at: Some(chrono::Utc::now() + chrono::Duration::minutes(2)), // Expires soon
    };

    let connection = token_vault.store_token(store_request).await.unwrap();

    // Note: This will fail because we're using a mock token, but we can test the structure
    let health_check_result = spotify_service.check_token_health(&connection).await;

    // The health check should return an error because the token is invalid
    // but the structure should be correct
    assert!(health_check_result.is_err());

    // Test with a connection that needs refresh
    assert!(connection.needs_refresh());
}

#[tokio::test]
async fn test_spotify_config_validation() {
    let config = SpotifyConfig::default();

    // Verify default config has required fields
    assert!(!config.client_id.is_empty());
    assert!(!config.client_secret.is_empty());
    assert!(!config.redirect_uri.is_empty());
    assert_eq!(config.auth_url, "https://accounts.spotify.com/authorize");
    assert_eq!(config.token_url, "https://accounts.spotify.com/api/token");

    // Test creating service with config
    let token_vault = Arc::new(TokenVaultService::new());
    let spotify_service = SpotifyService::new(config, token_vault);
    assert!(spotify_service.is_ok());
}
