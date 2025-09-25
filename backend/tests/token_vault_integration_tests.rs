use music_streaming_blocklist_backend::{
    TokenVaultService, TokenVaultBackgroundService, StoreTokenRequest, 
    StreamingProvider, ConnectionStatus
};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
use chrono::Utc;

#[tokio::test]
async fn test_complete_token_vault_workflow() {
    // Initialize token vault service
    let vault = Arc::new(TokenVaultService::new());
    let user_id = Uuid::new_v4();

    // Test 1: Store tokens for multiple providers
    println!("=== Test 1: Storing tokens for multiple providers ===");
    
    let spotify_request = StoreTokenRequest {
        user_id,
        provider: StreamingProvider::Spotify,
        provider_user_id: "spotify_user_123".to_string(),
        access_token: "spotify_access_token_abc123".to_string(),
        refresh_token: Some("spotify_refresh_token_def456".to_string()),
        scopes: vec!["user-read-private".to_string(), "user-modify-playback-state".to_string()],
        expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
    };

    let apple_request = StoreTokenRequest {
        user_id,
        provider: StreamingProvider::AppleMusic,
        provider_user_id: "apple_user_456".to_string(),
        access_token: "apple_access_token_xyz789".to_string(),
        refresh_token: None, // Apple Music uses different token model
        scopes: vec!["read".to_string(), "write".to_string()],
        expires_at: Some(Utc::now() + chrono::Duration::minutes(30)), // Shorter expiry
    };

    let spotify_connection = vault.store_token(spotify_request).await.unwrap();
    let apple_connection = vault.store_token(apple_request).await.unwrap();

    println!("✓ Stored Spotify connection: {}", spotify_connection.id);
    println!("✓ Stored Apple Music connection: {}", apple_connection.id);

    // Test 2: Retrieve and decrypt tokens
    println!("\n=== Test 2: Retrieving and decrypting tokens ===");
    
    let spotify_token = vault.get_token(user_id, StreamingProvider::Spotify).await.unwrap();
    assert_eq!(spotify_token.access_token, "spotify_access_token_abc123");
    assert_eq!(spotify_token.refresh_token, Some("spotify_refresh_token_def456".to_string()));
    assert_eq!(spotify_token.scopes.len(), 2);
    println!("✓ Successfully retrieved and decrypted Spotify token");

    let apple_token = vault.get_token(user_id, StreamingProvider::AppleMusic).await.unwrap();
    assert_eq!(apple_token.access_token, "apple_access_token_xyz789");
    assert_eq!(apple_token.refresh_token, None);
    println!("✓ Successfully retrieved and decrypted Apple Music token");

    // Test 3: Health checks
    println!("\n=== Test 3: Token health checks ===");
    
    let spotify_health = vault.check_token_health(spotify_connection.id).await.unwrap();
    assert!(spotify_health.is_valid);
    assert!(!spotify_health.needs_refresh); // 1 hour expiry
    println!("✓ Spotify token health: valid={}, needs_refresh={}", 
             spotify_health.is_valid, spotify_health.needs_refresh);

    let apple_health = vault.check_token_health(apple_connection.id).await.unwrap();
    assert!(apple_health.is_valid);
    assert!(apple_health.needs_refresh); // 30 minute expiry
    println!("✓ Apple Music token health: valid={}, needs_refresh={}", 
             apple_health.is_valid, apple_health.needs_refresh);

    // Test 4: Get all user connections
    println!("\n=== Test 4: Retrieving all user connections ===");
    
    let user_connections = vault.get_user_connections(user_id).await;
    assert_eq!(user_connections.len(), 2);
    
    let providers: Vec<_> = user_connections.iter()
        .map(|c| c.provider.clone())
        .collect();
    assert!(providers.contains(&StreamingProvider::Spotify));
    assert!(providers.contains(&StreamingProvider::AppleMusic));
    println!("✓ Retrieved {} connections for user", user_connections.len());

    // Test 5: Update existing token
    println!("\n=== Test 5: Updating existing token ===");
    
    let updated_spotify_request = StoreTokenRequest {
        user_id,
        provider: StreamingProvider::Spotify,
        provider_user_id: "spotify_user_123".to_string(),
        access_token: "new_spotify_access_token_updated".to_string(),
        refresh_token: Some("new_spotify_refresh_token_updated".to_string()),
        scopes: vec!["user-read-private".to_string(), "user-modify-playback-state".to_string()],
        expires_at: Some(Utc::now() + chrono::Duration::hours(2)),
    };

    let updated_connection = vault.store_token(updated_spotify_request).await.unwrap();
    assert_eq!(updated_connection.id, spotify_connection.id); // Same connection ID
    assert_eq!(updated_connection.token_version, 2); // Version incremented

    let updated_token = vault.get_token(user_id, StreamingProvider::Spotify).await.unwrap();
    assert_eq!(updated_token.access_token, "new_spotify_access_token_updated");
    println!("✓ Successfully updated Spotify token (version {})", updated_connection.token_version);

    // Test 6: Token revocation
    println!("\n=== Test 6: Token revocation ===");
    
    vault.revoke_connection(apple_connection.id).await.unwrap();
    
    let revoke_result = vault.get_token(user_id, StreamingProvider::AppleMusic).await;
    assert!(revoke_result.is_err());
    println!("✓ Successfully revoked Apple Music connection");

    // Verify only Spotify connection remains active
    let remaining_connections = vault.get_user_connections(user_id).await;
    let active_connections: Vec<_> = remaining_connections.iter()
        .filter(|c| c.status == ConnectionStatus::Active)
        .collect();
    assert_eq!(active_connections.len(), 1);
    assert_eq!(active_connections[0].provider, StreamingProvider::Spotify);
    println!("✓ Verified only 1 active connection remains");

    println!("\n=== All token vault tests passed! ===");
}

#[tokio::test]
async fn test_background_service_integration() {
    println!("=== Testing Token Vault Background Service ===");
    
    let vault = Arc::new(TokenVaultService::new());
    let background_service = TokenVaultBackgroundService::new(Arc::clone(&vault))
        .with_intervals(
            Duration::from_secs(1), // Fast health checks for testing
            Duration::from_secs(2), // Fast key rotation for testing
        );

    let user_id = Uuid::new_v4();

    // Add some test connections with different expiry times
    let soon_expiring_request = StoreTokenRequest {
        user_id,
        provider: StreamingProvider::Spotify,
        provider_user_id: "spotify_user_expiring".to_string(),
        access_token: "expiring_access_token".to_string(),
        refresh_token: Some("expiring_refresh_token".to_string()),
        scopes: vec!["read".to_string()],
        expires_at: Some(Utc::now() + chrono::Duration::minutes(3)), // Expires soon
    };

    let long_lived_request = StoreTokenRequest {
        user_id,
        provider: StreamingProvider::AppleMusic,
        provider_user_id: "apple_user_longlived".to_string(),
        access_token: "longlived_access_token".to_string(),
        refresh_token: None,
        scopes: vec!["read".to_string()],
        expires_at: Some(Utc::now() + chrono::Duration::hours(24)), // Long lived
    };

    vault.store_token(soon_expiring_request).await.unwrap();
    vault.store_token(long_lived_request).await.unwrap();

    // Test immediate health check
    let health_check_count = background_service.immediate_health_check().await.unwrap();
    assert_eq!(health_check_count, 2);
    println!("✓ Immediate health check completed on {} connections", health_check_count);

    // Test immediate key rotation
    let rotation_count = background_service.immediate_key_rotation().await.unwrap();
    assert_eq!(rotation_count, 0); // No keys old enough to rotate
    println!("✓ Immediate key rotation completed, {} keys rotated", rotation_count);

    // Test statistics
    let stats = background_service.get_statistics().await.unwrap();
    assert_eq!(stats.total_connections, 2);
    assert_eq!(stats.active_connections, 2);
    assert_eq!(stats.expired_connections, 0);
    assert_eq!(stats.connections_needing_refresh, 1); // The soon-expiring one
    
    println!("✓ Statistics: {} total, {} active, {} expired, {} need refresh",
             stats.total_connections, stats.active_connections, 
             stats.expired_connections, stats.connections_needing_refresh);

    println!("✓ Background service integration test passed!");
}

#[tokio::test]
async fn test_encryption_security() {
    println!("=== Testing Encryption Security ===");
    
    let vault = Arc::new(TokenVaultService::new());
    let user_id = Uuid::new_v4();

    // Store a token
    let request = StoreTokenRequest {
        user_id,
        provider: StreamingProvider::Spotify,
        provider_user_id: "security_test_user".to_string(),
        access_token: "super_secret_access_token_12345".to_string(),
        refresh_token: Some("super_secret_refresh_token_67890".to_string()),
        scopes: vec!["read".to_string(), "write".to_string()],
        expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
    };

    let connection = vault.store_token(request).await.unwrap();
    
    // Verify that the stored encrypted tokens don't contain the plaintext
    assert!(!connection.access_token_encrypted.contains("super_secret_access_token_12345"));
    assert!(!connection.refresh_token_encrypted.as_ref().unwrap().contains("super_secret_refresh_token_67890"));
    println!("✓ Verified tokens are encrypted in storage");

    // Verify we can still decrypt them correctly
    let decrypted = vault.get_token(user_id, StreamingProvider::Spotify).await.unwrap();
    assert_eq!(decrypted.access_token, "super_secret_access_token_12345");
    assert_eq!(decrypted.refresh_token, Some("super_secret_refresh_token_67890".to_string()));
    println!("✓ Verified tokens can be decrypted correctly");

    // Test with different user - should not be able to decrypt
    let different_user_id = Uuid::new_v4();
    let different_user_result = vault.get_token(different_user_id, StreamingProvider::Spotify).await;
    assert!(different_user_result.is_err());
    println!("✓ Verified tokens are isolated per user");

    println!("✓ Encryption security test passed!");
}

#[tokio::test]
async fn test_multiple_providers_same_user() {
    println!("=== Testing Multiple Providers for Same User ===");
    
    let vault = Arc::new(TokenVaultService::new());
    let user_id = Uuid::new_v4();

    // Store tokens for all supported providers
    let providers_and_tokens = vec![
        (StreamingProvider::Spotify, "spotify_token", Some("spotify_refresh")),
        (StreamingProvider::AppleMusic, "apple_token", None),
        (StreamingProvider::YouTubeMusic, "youtube_token", Some("youtube_refresh")),
        (StreamingProvider::Tidal, "tidal_token", Some("tidal_refresh")),
    ];

    for (provider, access_token, refresh_token) in &providers_and_tokens {
        let request = StoreTokenRequest {
            user_id,
            provider: provider.clone(),
            provider_user_id: format!("{}_user_123", provider.as_str()),
            access_token: access_token.to_string(),
            refresh_token: refresh_token.map(|s| s.to_string()),
            scopes: vec!["read".to_string()],
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
        };

        vault.store_token(request).await.unwrap();
        println!("✓ Stored token for {}", provider.as_str());
    }

    // Verify all tokens can be retrieved
    for (provider, expected_access_token, expected_refresh_token) in &providers_and_tokens {
        let token = vault.get_token(user_id, provider.clone()).await.unwrap();
        assert_eq!(token.access_token, *expected_access_token);
        assert_eq!(token.refresh_token, expected_refresh_token.map(|s| s.to_string()));
        println!("✓ Retrieved and verified token for {}", provider.as_str());
    }

    // Verify user has all 4 connections
    let connections = vault.get_user_connections(user_id).await;
    assert_eq!(connections.len(), 4);
    println!("✓ User has {} total connections", connections.len());

    println!("✓ Multiple providers test passed!");
}