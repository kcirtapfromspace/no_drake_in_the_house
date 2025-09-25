use crate::models::{
    Connection, StreamingProvider, ConnectionStatus, EncryptedToken, DataKey, 
    TokenHealthCheck, StoreTokenRequest, DecryptedToken, TokenRefreshResult
};
use anyhow::{Result, anyhow};
use aes_gcm::{Aes256Gcm, aead::{Aead, KeyInit}};
use base64::prelude::*;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use rand::RngCore;
use ring::rand::{SystemRandom, SecureRandom};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Mock KMS service for development - replace with actual KMS in production
pub struct MockKmsService {
    master_key: [u8; 32],
    rng: SystemRandom,
}

impl MockKmsService {
    pub fn new() -> Self {
        let mut master_key = [0u8; 32];
        let rng = SystemRandom::new();
        rng.fill(&mut master_key).expect("Failed to generate master key");
        
        Self {
            master_key,
            rng,
        }
    }

    /// Generate a new data encryption key
    pub fn generate_data_key(&self, key_id: &str) -> Result<DataKey> {
        // Generate 256-bit data key
        let mut plaintext_key = [0u8; 32];
        self.rng.fill(&mut plaintext_key)
            .map_err(|_| anyhow!("Failed to generate data key"))?;

        // Encrypt the data key with master key (simplified - use actual KMS in production)
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);
        let mut nonce_bytes = [0u8; 12];
        self.rng.fill(&mut nonce_bytes)
            .map_err(|_| anyhow!("Failed to generate nonce"))?;
        let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);

        let encrypted_key = cipher.encrypt(nonce, plaintext_key.as_ref())
            .map_err(|_| anyhow!("Failed to encrypt data key"))?;

        Ok(DataKey::new(
            key_id.to_string(),
            plaintext_key.to_vec(),
            encrypted_key,
            1,
        ))
    }

    /// Decrypt a data encryption key
    pub fn decrypt_data_key(&self, encrypted_key: &[u8], _key_id: &str) -> Result<Vec<u8>> {
        // In a real implementation, this would use the KMS to decrypt
        // For now, we'll use our master key directly
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);
        
        // Extract nonce from the beginning of encrypted data (first 12 bytes)
        if encrypted_key.len() < 12 {
            return Err(anyhow!("Invalid encrypted key format"));
        }
        
        let nonce = aes_gcm::Nonce::from_slice(&encrypted_key[..12]);
        let ciphertext = &encrypted_key[12..];
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|_| anyhow!("Failed to decrypt data key"))
    }
}

/// Token vault service with KMS-based envelope encryption
pub struct TokenVaultService {
    // In-memory storage for demo - replace with database in production
    connections: Arc<DashMap<Uuid, Connection>>,
    connections_by_user_provider: Arc<DashMap<(Uuid, StreamingProvider), Uuid>>,
    
    // Data key cache for performance
    data_key_cache: Arc<RwLock<DashMap<String, DataKey>>>,
    
    // KMS service for envelope encryption
    kms: Arc<MockKmsService>,
    
    // Configuration
    key_rotation_days: i64,
    health_check_interval_hours: i64,
}

impl TokenVaultService {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
            connections_by_user_provider: Arc::new(DashMap::new()),
            data_key_cache: Arc::new(RwLock::new(DashMap::new())),
            kms: Arc::new(MockKmsService::new()),
            key_rotation_days: 30,
            health_check_interval_hours: 24,
        }
    }

    /// Store a new provider token with envelope encryption
    pub async fn store_token(&self, request: StoreTokenRequest) -> Result<Connection> {
        // Check if connection already exists for this user/provider
        let connection_key = (request.user_id, request.provider.clone());
        
        // Generate or get data key for encryption
        let key_id = format!("user-{}-{}", request.user_id, request.provider.as_str());
        let data_key = self.get_or_create_data_key(&key_id).await?;

        // Encrypt access token
        let encrypted_access_token = self.encrypt_token(&request.access_token, &data_key)?;
        
        // Encrypt refresh token if provided
        let encrypted_refresh_token = if let Some(ref refresh_token) = request.refresh_token {
            Some(self.encrypt_token(refresh_token, &data_key)?)
        } else {
            None
        };

        // Create or update connection
        let connection = if let Some(existing_id) = self.connections_by_user_provider.get(&connection_key) {
            // Update existing connection
            let mut existing = self.connections.get_mut(&existing_id)
                .ok_or_else(|| anyhow!("Connection not found"))?;
            
            existing.update_tokens(
                serde_json::to_string(&encrypted_access_token)?,
                encrypted_refresh_token.map(|t| serde_json::to_string(&t)).transpose()?,
                request.expires_at,
            );
            
            existing.clone()
        } else {
            // Create new connection
            let connection = Connection::new(
                request.user_id,
                request.provider.clone(),
                request.provider_user_id,
                request.scopes,
                serde_json::to_string(&encrypted_access_token)?,
                encrypted_refresh_token.map(|t| serde_json::to_string(&t)).transpose()?,
                request.expires_at,
            );
            
            let connection_id = connection.id;
            self.connections.insert(connection_id, connection.clone());
            self.connections_by_user_provider.insert(connection_key, connection_id);
            
            connection
        };

        Ok(connection)
    }

    /// Retrieve and decrypt a token for a user/provider
    pub async fn get_token(&self, user_id: Uuid, provider: StreamingProvider) -> Result<DecryptedToken> {
        let connection_key = (user_id, provider.clone());
        let connection_id = self.connections_by_user_provider.get(&connection_key)
            .ok_or_else(|| anyhow!("No connection found for user and provider"))?
            .clone();

        let connection = self.connections.get(&connection_id)
            .ok_or_else(|| anyhow!("Connection not found"))?;

        if connection.status != ConnectionStatus::Active {
            return Err(anyhow!("Connection is not active: {:?}", connection.status));
        }

        // Get data key for decryption
        let key_id = format!("user-{}-{}", user_id, provider.as_str());
        let data_key = self.get_data_key(&key_id).await?;

        // Decrypt access token
        let encrypted_access_token: EncryptedToken = serde_json::from_str(&connection.access_token_encrypted)?;
        let access_token = self.decrypt_token(&encrypted_access_token, &data_key)?;

        // Decrypt refresh token if present
        let refresh_token = if let Some(ref encrypted_refresh_str) = connection.refresh_token_encrypted {
            let encrypted_refresh_token: EncryptedToken = serde_json::from_str(encrypted_refresh_str)?;
            Some(self.decrypt_token(&encrypted_refresh_token, &data_key)?)
        } else {
            None
        };

        Ok(DecryptedToken {
            access_token,
            refresh_token,
            expires_at: connection.expires_at,
            scopes: connection.scopes.clone(),
        })
    }

    /// Check the health of a connection's tokens
    pub async fn check_token_health(&self, connection_id: Uuid) -> Result<TokenHealthCheck> {
        let mut connection = self.connections.get_mut(&connection_id)
            .ok_or_else(|| anyhow!("Connection not found"))?;

        let now = Utc::now();
        let is_expired = connection.is_expired();
        let needs_refresh = connection.needs_refresh();

        // Update last health check
        connection.last_health_check = Some(now);

        // Mark as expired if needed
        if is_expired && connection.status == ConnectionStatus::Active {
            connection.mark_expired();
        }

        Ok(TokenHealthCheck {
            connection_id,
            is_valid: connection.status == ConnectionStatus::Active && !is_expired,
            expires_at: connection.expires_at,
            error_message: connection.error_code.clone(),
            checked_at: now,
            needs_refresh,
        })
    }

    /// Refresh a token (placeholder - actual implementation would call provider APIs)
    pub async fn refresh_token(&self, connection_id: Uuid) -> Result<TokenRefreshResult> {
        let connection = self.connections.get(&connection_id)
            .ok_or_else(|| anyhow!("Connection not found"))?;

        // In a real implementation, this would:
        // 1. Get the current refresh token
        // 2. Call the provider's token refresh endpoint
        // 3. Store the new tokens
        // 4. Return the result

        // For now, return a placeholder result
        Ok(TokenRefreshResult {
            connection_id,
            success: false,
            new_access_token: None,
            new_refresh_token: None,
            new_expires_at: None,
            error_message: Some("Token refresh not implemented for this provider".to_string()),
        })
    }

    /// Get all connections for a user
    pub async fn get_user_connections(&self, user_id: Uuid) -> Vec<Connection> {
        self.connections
            .iter()
            .filter(|entry| entry.value().user_id == user_id)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Revoke a connection and its tokens
    pub async fn revoke_connection(&self, connection_id: Uuid) -> Result<()> {
        let mut connection = self.connections.get_mut(&connection_id)
            .ok_or_else(|| anyhow!("Connection not found"))?;

        connection.status = ConnectionStatus::Revoked;
        connection.updated_at = Utc::now();

        // Remove from user/provider index
        let connection_key = (connection.user_id, connection.provider.clone());
        self.connections_by_user_provider.remove(&connection_key);

        Ok(())
    }

    /// Rotate data keys for all connections (background task)
    pub async fn rotate_data_keys(&self) -> Result<usize> {
        let mut rotated_count = 0;
        let cache = self.data_key_cache.read().await;
        
        for entry in cache.iter() {
            let data_key = entry.value();
            if data_key.should_rotate(self.key_rotation_days) {
                // Generate new data key
                let new_data_key = self.kms.generate_data_key(&data_key.key_id)?;
                
                // Re-encrypt all tokens using this key
                // In a real implementation, this would be done in batches
                self.reencrypt_tokens_with_new_key(&data_key.key_id, &new_data_key).await?;
                
                rotated_count += 1;
            }
        }

        Ok(rotated_count)
    }

    /// Perform health checks on all connections
    pub async fn health_check_all_connections(&self) -> Result<Vec<TokenHealthCheck>> {
        let mut results = Vec::new();
        
        for entry in self.connections.iter() {
            let connection = entry.value();
            
            // Skip if recently checked
            if let Some(last_check) = connection.last_health_check {
                let hours_since_check = (Utc::now() - last_check).num_hours();
                if hours_since_check < self.health_check_interval_hours {
                    continue;
                }
            }

            match self.check_token_health(connection.id).await {
                Ok(health_check) => results.push(health_check),
                Err(e) => {
                    tracing::warn!("Health check failed for connection {}: {}", connection.id, e);
                }
            }
        }

        Ok(results)
    }

    // Private helper methods

    async fn get_or_create_data_key(&self, key_id: &str) -> Result<DataKey> {
        let cache = self.data_key_cache.read().await;
        
        if let Some(data_key) = cache.get(key_id) {
            return Ok(data_key.clone());
        }
        
        drop(cache);

        // Generate new data key
        let data_key = self.kms.generate_data_key(key_id)?;
        
        // Cache it
        let mut cache = self.data_key_cache.write().await;
        cache.insert(key_id.to_string(), data_key.clone());
        
        Ok(data_key)
    }

    async fn get_data_key(&self, key_id: &str) -> Result<DataKey> {
        let cache = self.data_key_cache.read().await;
        
        cache.get(key_id)
            .map(|k| k.clone())
            .ok_or_else(|| anyhow!("Data key not found: {}", key_id))
    }

    fn encrypt_token(&self, token: &str, data_key: &DataKey) -> Result<EncryptedToken> {
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&data_key.plaintext_key);
        let cipher = Aes256Gcm::new(key);
        
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);

        // Encrypt the token
        let encrypted_data = cipher.encrypt(nonce, token.as_bytes())
            .map_err(|_| anyhow!("Failed to encrypt token"))?;

        Ok(EncryptedToken::new(
            encrypted_data,
            data_key.encrypted_key.clone(),
            nonce_bytes.to_vec(),
            data_key.key_id.clone(),
            data_key.version,
        ))
    }

    fn decrypt_token(&self, encrypted_token: &EncryptedToken, data_key: &DataKey) -> Result<String> {
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&data_key.plaintext_key);
        let cipher = Aes256Gcm::new(key);
        
        let encrypted_data = encrypted_token.get_encrypted_data()
            .map_err(|_| anyhow!("Invalid encrypted data format"))?;
        let nonce_bytes = encrypted_token.get_nonce()
            .map_err(|_| anyhow!("Invalid nonce format"))?;
        let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);

        let decrypted_data = cipher.decrypt(nonce, encrypted_data.as_ref())
            .map_err(|_| anyhow!("Failed to decrypt token"))?;

        String::from_utf8(decrypted_data)
            .map_err(|_| anyhow!("Invalid UTF-8 in decrypted token"))
    }

    async fn reencrypt_tokens_with_new_key(&self, old_key_id: &str, new_data_key: &DataKey) -> Result<()> {
        // Find all connections using the old key
        for mut entry in self.connections.iter_mut() {
            let connection = entry.value_mut();
            
            // Check if this connection uses the old key
            let connection_key_id = format!("user-{}-{}", connection.user_id, connection.provider.as_str());
            if connection_key_id != old_key_id {
                continue;
            }

            // Get old data key to decrypt
            let old_data_key = self.get_data_key(old_key_id).await?;

            // Decrypt with old key
            let encrypted_access_token: EncryptedToken = serde_json::from_str(&connection.access_token_encrypted)?;
            let access_token = self.decrypt_token(&encrypted_access_token, &old_data_key)?;

            let refresh_token = if let Some(ref encrypted_refresh_str) = connection.refresh_token_encrypted {
                let encrypted_refresh_token: EncryptedToken = serde_json::from_str(encrypted_refresh_str)?;
                Some(self.decrypt_token(&encrypted_refresh_token, &old_data_key)?)
            } else {
                None
            };

            // Re-encrypt with new key
            let new_encrypted_access_token = self.encrypt_token(&access_token, new_data_key)?;
            let new_encrypted_refresh_token = if let Some(ref refresh_token) = refresh_token {
                Some(self.encrypt_token(refresh_token, new_data_key)?)
            } else {
                None
            };

            // Update connection
            connection.update_tokens(
                serde_json::to_string(&new_encrypted_access_token)?,
                new_encrypted_refresh_token.map(|t| serde_json::to_string(&t)).transpose()?,
                connection.expires_at,
            );
        }

        // Update cached data key
        let mut cache = self.data_key_cache.write().await;
        cache.insert(old_key_id.to_string(), new_data_key.clone());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_and_retrieve_token() {
        let vault = TokenVaultService::new();
        let user_id = Uuid::new_v4();
        
        let request = StoreTokenRequest {
            user_id,
            provider: StreamingProvider::Spotify,
            provider_user_id: "spotify_user_123".to_string(),
            access_token: "access_token_123".to_string(),
            refresh_token: Some("refresh_token_123".to_string()),
            scopes: vec!["read".to_string(), "write".to_string()],
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
        };

        // Store token
        let connection = vault.store_token(request).await.unwrap();
        assert_eq!(connection.user_id, user_id);
        assert_eq!(connection.provider, StreamingProvider::Spotify);

        // Retrieve token
        let decrypted = vault.get_token(user_id, StreamingProvider::Spotify).await.unwrap();
        assert_eq!(decrypted.access_token, "access_token_123");
        assert_eq!(decrypted.refresh_token, Some("refresh_token_123".to_string()));
    }

    #[tokio::test]
    async fn test_token_health_check() {
        let vault = TokenVaultService::new();
        let user_id = Uuid::new_v4();
        
        let request = StoreTokenRequest {
            user_id,
            provider: StreamingProvider::Spotify,
            provider_user_id: "spotify_user_123".to_string(),
            access_token: "access_token_123".to_string(),
            refresh_token: None,
            scopes: vec!["read".to_string()],
            expires_at: Some(Utc::now() + chrono::Duration::minutes(1)), // Expires soon
        };

        let connection = vault.store_token(request).await.unwrap();
        let health_check = vault.check_token_health(connection.id).await.unwrap();
        
        assert!(health_check.is_valid);
        assert!(health_check.needs_refresh);
    }

    #[tokio::test]
    async fn test_revoke_connection() {
        let vault = TokenVaultService::new();
        let user_id = Uuid::new_v4();
        
        let request = StoreTokenRequest {
            user_id,
            provider: StreamingProvider::Spotify,
            provider_user_id: "spotify_user_123".to_string(),
            access_token: "access_token_123".to_string(),
            refresh_token: None,
            scopes: vec!["read".to_string()],
            expires_at: None,
        };

        let connection = vault.store_token(request).await.unwrap();
        
        // Revoke connection
        vault.revoke_connection(connection.id).await.unwrap();
        
        // Should not be able to retrieve token
        let result = vault.get_token(user_id, StreamingProvider::Spotify).await;
        assert!(result.is_err());
    }
}