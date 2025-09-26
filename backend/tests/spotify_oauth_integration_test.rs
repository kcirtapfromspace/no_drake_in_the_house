use music_streaming_blocklist_backend::*;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_spotify_oauth_url_generation() {
    // Initialize services
    let token_vault = Arc::new(TokenVaultService::new());
    let spotify_config = SpotifyConfig::default();
    let spotify_service = SpotifyService::new(spotify_config, token_vault.clone()).unwrap();

    // Test: Generate auth URL
    let auth_response = spotify_service.get_auth_url().await.unwrap();
    
    assert!(auth_response.auth_url.contains("accounts.spotify.com"));
    assert!(auth_response.auth_url.contains("client_id"));
    assert!(auth_response.auth_url.contains("code_challenge"));
    assert!(auth_response.auth_url.contains("scope"));
    assert!(!auth_response.state.is_empty());
    
    // Verify required scopes are included
    assert!(auth_response.auth_url.contains("user-read-private"));
    assert!(auth_response.auth_url.contains("user-library-read"));
    assert!(auth_response.auth_url.contains("user-library-modify"));
    assert!(auth_response.auth_url.contains("playlist-modify-private"));
    assert!(auth_response.auth_url.contains("user-follow-modify"));

    println!("✅ Spotify OAuth URL generation works correctly");
}

#[tokio::test]
async fn test_spotify_connection_storage() {
    let token_vault = Arc::new(TokenVaultService::new());
    let spotify_config = SpotifyConfig::default();
    let spotify_service = SpotifyService::new(spotify_config, token_vault.clone()).unwrap();
    
    let user_id = Uuid::new_v4();
    
    // Test: No connection initially
    let connection = spotify_service.get_user_connection(user_id).await.unwrap();
    assert!(connection.is_none());
    
    // Test: Store a mock connection via token vault
    let store_request = StoreTokenRequest {
        user_id,
        provider: StreamingProvider::Spotify,
        provider_user_id: "spotify_test_user".to_string(),
        access_token: "mock_access_token".to_string(),
        refresh_token: Some("mock_refresh_token".to_string()),
        scopes: vec![
            "user-read-private".to_string(),
            "user-library-read".to_string(),
            "user-library-modify".to_string(),
        ],
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
    };
    
    let _stored_connection = token_vault.store_token(store_request).await.unwrap();
    
    // Test: Connection should now exist
    let connection = spotify_service.get_user_connection(user_id).await.unwrap();
    assert!(connection.is_some());
    
    let connection = connection.unwrap();
    assert_eq!(connection.user_id, user_id);
    assert_eq!(connection.provider, StreamingProvider::Spotify);
    assert_eq!(connection.provider_user_id, "spotify_test_user");
    assert_eq!(connection.status, ConnectionStatus::Active);
    
    // Test: Disconnect
    spotify_service.disconnect_user(user_id).await.unwrap();
    
    let connection_after_disconnect = spotify_service.get_user_connection(user_id).await.unwrap();
    assert!(connection_after_disconnect.is_none());
    
    println!("✅ Spotify connection management works correctly");
}

#[tokio::test]
async fn test_token_encryption_and_decryption() {
    let token_vault = Arc::new(TokenVaultService::new());
    let user_id = Uuid::new_v4();
    
    let original_access_token = "test_access_token_12345";
    let original_refresh_token = "test_refresh_token_67890";
    
    // Store tokens
    let store_request = StoreTokenRequest {
        user_id,
        provider: StreamingProvider::Spotify,
        provider_user_id: "spotify_test_user".to_string(),
        access_token: original_access_token.to_string(),
        refresh_token: Some(original_refresh_token.to_string()),
        scopes: vec!["user-read-private".to_string()],
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
    };
    
    let connection = token_vault.store_token(store_request).await.unwrap();
    
    // Verify tokens are encrypted (should not contain original values)
    assert!(!connection.access_token_encrypted.contains(original_access_token));
    assert!(connection.refresh_token_encrypted.is_some());
    assert!(!connection.refresh_token_encrypted.as_ref().unwrap().contains(original_refresh_token));
    
    // Retrieve and decrypt tokens
    let decrypted = token_vault.get_decrypted_token(connection.id).await.unwrap();
    
    // Verify decryption worked correctly
    assert_eq!(decrypted.access_token, original_access_token);
    assert_eq!(decrypted.refresh_token, Some(original_refresh_token.to_string()));
    
    println!("✅ Token encryption and decryption works correctly");
}

#[tokio::test]
async fn test_spotify_service_initialization() {
    let token_vault = Arc::new(TokenVaultService::new());
    let spotify_config = SpotifyConfig::default();
    
    // Test: Service can be created successfully
    let spotify_service = SpotifyService::new(spotify_config, token_vault.clone());
    assert!(spotify_service.is_ok());
    
    println!("✅ Spotify service initialization works correctly");
}