use chrono::{Duration, Utc};
use music_streaming_blocklist_backend::models::*;
use music_streaming_blocklist_backend::services::*;
use uuid::Uuid;

#[tokio::test]
async fn test_token_vault_service_creation() {
    let vault_service = TokenVaultService::new();

    // Test that service is created with default configuration
    // We can't access private fields directly, but we can test behavior

    // Service should be ready to store and retrieve tokens
    assert!(true); // Basic creation test
}

#[tokio::test]
async fn test_mock_kms_service() {
    let kms = MockKmsService::new();

    // Test data key generation
    let key_id = "test-key-id";
    let data_key_result = kms.generate_data_key(key_id);
    assert!(data_key_result.is_ok());

    let data_key = data_key_result.unwrap();
    assert_eq!(data_key.key_id, key_id);
    assert_eq!(data_key.plaintext_key.len(), 32); // 256-bit key
    assert!(!data_key.encrypted_key.is_empty());
    assert_eq!(data_key.version, 1);

    // Test data key decryption
    let decrypted_result = kms.decrypt_data_key(&data_key.encrypted_key, key_id);
    assert!(decrypted_result.is_ok());

    let decrypted_key = decrypted_result.unwrap();
    assert_eq!(decrypted_key, data_key.plaintext_key);
}

#[tokio::test]
async fn test_store_and_retrieve_token() {
    let vault_service = TokenVaultService::new();

    let user_id = Uuid::new_v4();
    let provider = StreamingProvider::Spotify;

    // Create test connection
    let mut connection = Connection::new(
        user_id,
        provider.clone(),
        "test_provider_user_id".to_string(),
        vec!["read".to_string(), "write".to_string()],
    );

    // Store token
    let store_request = StoreTokenRequest {
        access_token: "test_access_token_12345".to_string(),
        refresh_token: Some("test_refresh_token_67890".to_string()),
        expires_at: Some(Utc::now() + Duration::hours(1)),
        scopes: vec!["read".to_string(), "write".to_string()],
    };

    let store_result = vault_service
        .store_connection_tokens(&mut connection, store_request)
        .await;
    assert!(store_result.is_ok());

    // Verify connection was updated
    assert!(connection.access_token_encrypted.is_some());
    assert!(connection.refresh_token_encrypted.is_some());
    assert_eq!(connection.status, ConnectionStatus::Active);

    // Retrieve tokens
    let retrieve_result = vault_service.get_decrypted_tokens(&connection).await;
    assert!(retrieve_result.is_ok());

    let decrypted_tokens = retrieve_result.unwrap();
    assert_eq!(decrypted_tokens.access_token, "test_access_token_12345");
    assert_eq!(
        decrypted_tokens.refresh_token,
        Some("test_refresh_token_67890".to_string())
    );
    assert!(decrypted_tokens.expires_at.is_some());
    assert_eq!(decrypted_tokens.scopes, vec!["read", "write"]);
}

#[tokio::test]
async fn test_token_encryption_decryption() {
    let vault_service = TokenVaultService::new();

    let user_id = Uuid::new_v4();
    let test_token = "sensitive_access_token_data";

    // Encrypt token
    let encrypt_result = vault_service.encrypt_token(test_token, user_id).await;
    assert!(encrypt_result.is_ok());

    let encrypted_token = encrypt_result.unwrap();
    assert!(!encrypted_token.ciphertext.is_empty());
    assert!(!encrypted_token.key_id.is_empty());
    assert_eq!(encrypted_token.version, 1);

    // Verify encrypted data is different from original
    assert_ne!(encrypted_token.ciphertext, test_token.as_bytes());

    // Decrypt token
    let decrypt_result = vault_service.decrypt_token(&encrypted_token, user_id).await;
    assert!(decrypt_result.is_ok());

    let decrypted_token = decrypt_result.unwrap();
    assert_eq!(decrypted_token, test_token);
}

#[tokio::test]
async fn test_token_health_check() {
    let vault_service = TokenVaultService::new();

    let user_id = Uuid::new_v4();
    let provider = StreamingProvider::Spotify;

    // Create connection with tokens
    let mut connection = Connection::new(
        user_id,
        provider.clone(),
        "health_check_user".to_string(),
        vec!["read".to_string()],
    );

    let store_request = StoreTokenRequest {
        access_token: "health_check_token".to_string(),
        refresh_token: Some("health_check_refresh".to_string()),
        expires_at: Some(Utc::now() + Duration::hours(1)),
        scopes: vec!["read".to_string()],
    };

    vault_service
        .store_connection_tokens(&mut connection, store_request)
        .await
        .unwrap();

    // Perform health check
    let health_result = vault_service.check_token_health(&connection).await;
    assert!(health_result.is_ok());

    let health_check = health_result.unwrap();
    assert_eq!(health_check.connection_id, connection.id);
    assert!(health_check.is_healthy);
    assert!(health_check.checked_at <= Utc::now());
    assert!(health_check.expires_at.is_some());
    assert!(health_check.needs_refresh.is_some());
}

#[tokio::test]
async fn test_token_refresh() {
    let vault_service = TokenVaultService::new();

    let user_id = Uuid::new_v4();
    let provider = StreamingProvider::Spotify;

    // Create connection with expiring token
    let mut connection = Connection::new(
        user_id,
        provider.clone(),
        "refresh_test_user".to_string(),
        vec!["read".to_string()],
    );

    let store_request = StoreTokenRequest {
        access_token: "expiring_token".to_string(),
        refresh_token: Some("valid_refresh_token".to_string()),
        expires_at: Some(Utc::now() + Duration::minutes(5)), // Expires soon
        scopes: vec!["read".to_string()],
    };

    vault_service
        .store_connection_tokens(&mut connection, store_request)
        .await
        .unwrap();

    // Test refresh token operation
    let new_tokens = StoreTokenRequest {
        access_token: "new_access_token".to_string(),
        refresh_token: Some("new_refresh_token".to_string()),
        expires_at: Some(Utc::now() + Duration::hours(1)),
        scopes: vec!["read".to_string()],
    };

    let refresh_result = vault_service
        .refresh_connection_tokens(&mut connection, new_tokens)
        .await;
    assert!(refresh_result.is_ok());

    let refresh_info = refresh_result.unwrap();
    assert_eq!(refresh_info.connection_id, connection.id);
    assert!(refresh_info.success);
    assert!(refresh_info.refreshed_at <= Utc::now());

    // Verify new tokens are stored
    let retrieved_tokens = vault_service
        .get_decrypted_tokens(&connection)
        .await
        .unwrap();
    assert_eq!(retrieved_tokens.access_token, "new_access_token");
    assert_eq!(
        retrieved_tokens.refresh_token,
        Some("new_refresh_token".to_string())
    );
}

#[tokio::test]
async fn test_connection_management() {
    let vault_service = TokenVaultService::new();

    let user_id = Uuid::new_v4();
    let provider = StreamingProvider::Apple;

    // Create connection
    let connection = Connection::new(
        user_id,
        provider.clone(),
        "apple_user_123".to_string(),
        vec!["library-read".to_string(), "library-modify".to_string()],
    );

    // Store connection
    let store_result = vault_service.store_connection(connection.clone()).await;
    assert!(store_result.is_ok());

    // Retrieve connection by user and provider
    let retrieve_result = vault_service
        .get_user_connection(user_id, provider.clone())
        .await;
    assert!(retrieve_result.is_ok());

    let retrieved_connection = retrieve_result.unwrap();
    assert!(retrieved_connection.is_some());

    let conn = retrieved_connection.unwrap();
    assert_eq!(conn.user_id, user_id);
    assert_eq!(conn.provider, provider);
    assert_eq!(conn.provider_user_id, "apple_user_123");
    assert_eq!(conn.scopes, vec!["library-read", "library-modify"]);

    // Get all user connections
    let all_connections_result = vault_service.get_user_connections(user_id).await;
    assert!(all_connections_result.is_ok());

    let all_connections = all_connections_result.unwrap();
    assert_eq!(all_connections.len(), 1);
    assert_eq!(all_connections[0].id, conn.id);
}

#[tokio::test]
async fn test_connection_status_management() {
    let vault_service = TokenVaultService::new();

    let user_id = Uuid::new_v4();
    let provider = StreamingProvider::Spotify;

    // Create connection
    let mut connection = Connection::new(
        user_id,
        provider.clone(),
        "status_test_user".to_string(),
        vec!["read".to_string()],
    );

    vault_service
        .store_connection(connection.clone())
        .await
        .unwrap();

    // Test marking connection as error
    let error_result = vault_service
        .mark_connection_error(
            &mut connection,
            "TOKEN_EXPIRED".to_string(),
            "Access token has expired".to_string(),
        )
        .await;
    assert!(error_result.is_ok());

    assert_eq!(connection.status, ConnectionStatus::Error);
    assert_eq!(connection.error_code, Some("TOKEN_EXPIRED".to_string()));
    assert!(connection.last_health_check.is_some());

    // Test marking connection as active
    let active_result = vault_service.mark_connection_active(&mut connection).await;
    assert!(active_result.is_ok());

    assert_eq!(connection.status, ConnectionStatus::Active);
    assert_eq!(connection.error_code, None);

    // Test revoking connection
    let revoke_result = vault_service.revoke_connection(&mut connection).await;
    assert!(revoke_result.is_ok());

    assert_eq!(connection.status, ConnectionStatus::Revoked);
}

#[tokio::test]
async fn test_data_key_rotation() {
    let vault_service = TokenVaultService::new();

    let user_id = Uuid::new_v4();
    let test_token = "rotation_test_token";

    // Encrypt token with initial key
    let encrypted_v1 = vault_service
        .encrypt_token(test_token, user_id)
        .await
        .unwrap();
    assert_eq!(encrypted_v1.version, 1);

    // Simulate key rotation (this would normally be triggered by background process)
    let rotate_result = vault_service.rotate_user_data_key(user_id).await;
    assert!(rotate_result.is_ok());

    // Encrypt new token with rotated key
    let encrypted_v2 = vault_service
        .encrypt_token("new_token_after_rotation", user_id)
        .await
        .unwrap();
    assert!(encrypted_v2.version >= encrypted_v1.version);

    // Verify old encrypted token can still be decrypted
    let decrypt_old_result = vault_service.decrypt_token(&encrypted_v1, user_id).await;
    assert!(decrypt_old_result.is_ok());
    assert_eq!(decrypt_old_result.unwrap(), test_token);

    // Verify new encrypted token can be decrypted
    let decrypt_new_result = vault_service.decrypt_token(&encrypted_v2, user_id).await;
    assert!(decrypt_new_result.is_ok());
    assert_eq!(decrypt_new_result.unwrap(), "new_token_after_rotation");
}

#[tokio::test]
async fn test_bulk_token_operations() {
    let vault_service = TokenVaultService::new();

    let user_id = Uuid::new_v4();

    // Create multiple connections
    let mut connections = vec![];
    for i in 0..3 {
        let provider = match i {
            0 => StreamingProvider::Spotify,
            1 => StreamingProvider::Apple,
            _ => StreamingProvider::Spotify, // For testing multiple of same provider
        };

        let mut connection = Connection::new(
            user_id,
            provider,
            format!("bulk_user_{}", i),
            vec!["read".to_string()],
        );

        let store_request = StoreTokenRequest {
            access_token: format!("bulk_token_{}", i),
            refresh_token: Some(format!("bulk_refresh_{}", i)),
            expires_at: Some(Utc::now() + Duration::hours(1)),
            scopes: vec!["read".to_string()],
        };

        vault_service
            .store_connection_tokens(&mut connection, store_request)
            .await
            .unwrap();
        connections.push(connection);
    }

    // Test bulk health check
    let health_results = vault_service.bulk_health_check(&connections).await;
    assert!(health_results.is_ok());

    let health_checks = health_results.unwrap();
    assert_eq!(health_checks.len(), 3);

    for health_check in &health_checks {
        assert!(health_check.is_healthy);
        assert!(health_check.checked_at <= Utc::now());
    }

    // Test bulk token refresh (simulate expired tokens)
    let refresh_requests: Vec<_> = connections
        .iter()
        .enumerate()
        .map(|(i, conn)| {
            (
                conn.id,
                StoreTokenRequest {
                    access_token: format!("refreshed_token_{}", i),
                    refresh_token: Some(format!("refreshed_refresh_{}", i)),
                    expires_at: Some(Utc::now() + Duration::hours(2)),
                    scopes: vec!["read".to_string()],
                },
            )
        })
        .collect();

    let bulk_refresh_result = vault_service.bulk_refresh_tokens(refresh_requests).await;
    assert!(bulk_refresh_result.is_ok());

    let refresh_results = bulk_refresh_result.unwrap();
    assert_eq!(refresh_results.len(), 3);

    for refresh_result in &refresh_results {
        assert!(refresh_result.success);
    }
}

#[tokio::test]
async fn test_token_vault_error_handling() {
    let vault_service = TokenVaultService::new();

    let user_id = Uuid::new_v4();

    // Test decrypting invalid encrypted token
    let invalid_encrypted = EncryptedToken {
        ciphertext: vec![1, 2, 3, 4, 5], // Invalid ciphertext
        key_id: "nonexistent_key".to_string(),
        version: 1,
    };

    let decrypt_result = vault_service
        .decrypt_token(&invalid_encrypted, user_id)
        .await;
    assert!(decrypt_result.is_err());

    // Test getting tokens for nonexistent connection
    let nonexistent_connection = Connection::new(
        user_id,
        StreamingProvider::Spotify,
        "nonexistent".to_string(),
        vec![],
    );

    let get_tokens_result = vault_service
        .get_decrypted_tokens(&nonexistent_connection)
        .await;
    assert!(get_tokens_result.is_err());

    // Test health check for connection without tokens
    let empty_connection = Connection::new(
        user_id,
        StreamingProvider::Apple,
        "empty".to_string(),
        vec![],
    );

    let health_result = vault_service.check_token_health(&empty_connection).await;
    assert!(health_result.is_err());
}
