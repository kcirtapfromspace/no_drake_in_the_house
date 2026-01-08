use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Duration, Utc};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{AppError, Result};

/// OAuth token encryption service using AES-GCM with key rotation support
pub struct OAuthTokenEncryption {
    /// Current active cipher for encryption
    current_cipher: Aes256Gcm,
    /// Current key ID for tracking
    current_key_id: String,
    /// Historical ciphers for decryption of old tokens (key rotation support)
    historical_ciphers: Arc<RwLock<HashMap<String, Aes256Gcm>>>,
    /// Key rotation configuration
    key_rotation_config: KeyRotationConfig,
}

/// Configuration for key rotation
#[derive(Debug, Clone)]
pub struct KeyRotationConfig {
    /// How often to rotate keys (in days)
    pub rotation_interval_days: i64,
    /// How many old keys to keep for decryption
    pub max_historical_keys: usize,
    /// Last rotation timestamp
    pub last_rotation: Option<DateTime<Utc>>,
}

impl Default for KeyRotationConfig {
    fn default() -> Self {
        Self {
            rotation_interval_days: 90, // Rotate every 90 days
            max_historical_keys: 5,     // Keep 5 old keys
            last_rotation: None,
        }
    }
}

impl OAuthTokenEncryption {
    /// Create a new encryption service with key from environment
    pub fn new() -> Result<Self> {
        let key_base64 =
            env::var("OAUTH_ENCRYPTION_KEY").map_err(|_| AppError::ConfigurationError {
                message: "OAUTH_ENCRYPTION_KEY environment variable not set".to_string(),
            })?;

        let key_bytes = general_purpose::STANDARD.decode(&key_base64).map_err(|e| {
            AppError::ConfigurationError {
                message: format!("Invalid OAUTH_ENCRYPTION_KEY format: {}", e),
            }
        })?;

        if key_bytes.len() != 32 {
            return Err(AppError::ConfigurationError {
                message: "OAUTH_ENCRYPTION_KEY must be 32 bytes (256 bits) when base64 decoded"
                    .to_string(),
            });
        }

        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        let current_key_id = Self::generate_key_id(&key_bytes);

        Ok(Self {
            current_cipher: cipher,
            current_key_id,
            historical_ciphers: Arc::new(RwLock::new(HashMap::new())),
            key_rotation_config: KeyRotationConfig::default(),
        })
    }

    /// Create a new encryption service with a provided key
    pub fn with_key(key: &[u8]) -> Result<Self> {
        if key.len() != 32 {
            return Err(AppError::ConfigurationError {
                message: "Encryption key must be 32 bytes (256 bits)".to_string(),
            });
        }

        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        let current_key_id = Self::generate_key_id(key);

        Ok(Self {
            current_cipher: cipher,
            current_key_id,
            historical_ciphers: Arc::new(RwLock::new(HashMap::new())),
            key_rotation_config: KeyRotationConfig::default(),
        })
    }

    /// Create encryption service with key rotation configuration
    pub fn with_key_rotation(key: &[u8], config: KeyRotationConfig) -> Result<Self> {
        let mut service = Self::with_key(key)?;
        service.key_rotation_config = config;
        Ok(service)
    }

    /// Generate a key ID from key bytes for tracking
    fn generate_key_id(key: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key);
        let hash = hasher.finalize();
        general_purpose::STANDARD.encode(&hash[..8]) // Use first 8 bytes as ID
    }

    /// Generate a new random encryption key (for setup/testing)
    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    /// Generate a base64-encoded key for environment variable
    pub fn generate_key_base64() -> String {
        let key = Self::generate_key();
        general_purpose::STANDARD.encode(key)
    }

    /// Encrypt a token string with current key
    pub fn encrypt_token(&self, token: &str) -> Result<Vec<u8>> {
        // Generate a random nonce for each encryption
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the token
        let ciphertext = self
            .current_cipher
            .encrypt(nonce, token.as_bytes())
            .map_err(|e| AppError::EncryptionError(format!("Token encryption failed: {}", e)))?;

        // Create versioned encrypted data: [version(1)] + [key_id_len(1)] + [key_id] + [nonce(12)] + [ciphertext]
        let key_id_bytes = self.current_key_id.as_bytes();
        let mut encrypted_data =
            Vec::with_capacity(1 + 1 + key_id_bytes.len() + 12 + ciphertext.len());

        encrypted_data.push(1); // Version 1
        encrypted_data.push(key_id_bytes.len() as u8); // Key ID length
        encrypted_data.extend_from_slice(key_id_bytes); // Key ID
        encrypted_data.extend_from_slice(&nonce_bytes); // Nonce
        encrypted_data.extend_from_slice(&ciphertext); // Ciphertext

        Ok(encrypted_data)
    }

    /// Decrypt a token from encrypted bytes with key rotation support
    pub async fn decrypt_token(&self, encrypted_data: &[u8]) -> Result<String> {
        if encrypted_data.is_empty() {
            return Err(AppError::EncryptionError(
                "Encrypted data is empty".to_string(),
            ));
        }

        // Check if this is versioned data (version 1+) or legacy data (version 0)
        if encrypted_data[0] == 1 {
            // Version 1: [version(1)] + [key_id_len(1)] + [key_id] + [nonce(12)] + [ciphertext]
            if encrypted_data.len() < 3 {
                return Err(AppError::EncryptionError(
                    "Versioned encrypted data too short".to_string(),
                ));
            }

            let key_id_len = encrypted_data[1] as usize;
            if encrypted_data.len() < 2 + key_id_len + 12 {
                return Err(AppError::EncryptionError(
                    "Versioned encrypted data incomplete".to_string(),
                ));
            }

            let key_id =
                String::from_utf8(encrypted_data[2..2 + key_id_len].to_vec()).map_err(|e| {
                    AppError::EncryptionError(format!("Invalid key ID in encrypted data: {}", e))
                })?;

            let nonce_start = 2 + key_id_len;
            let nonce_bytes = &encrypted_data[nonce_start..nonce_start + 12];
            let ciphertext = &encrypted_data[nonce_start + 12..];
            let nonce = Nonce::from_slice(nonce_bytes);

            // Try current key first (most common case)
            if key_id == self.current_key_id {
                let plaintext = self
                    .current_cipher
                    .decrypt(nonce, ciphertext)
                    .map_err(|e| {
                        AppError::EncryptionError(format!(
                            "Token decryption failed with current key: {}",
                            e
                        ))
                    })?;

                return String::from_utf8(plaintext).map_err(|e| {
                    AppError::EncryptionError(format!("Decrypted token is not valid UTF-8: {}", e))
                });
            }

            // Try historical keys
            let historical_ciphers = self.historical_ciphers.read().await;
            if let Some(cipher) = historical_ciphers.get(&key_id) {
                let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| {
                    AppError::EncryptionError(format!(
                        "Token decryption failed with historical key {}: {}",
                        key_id, e
                    ))
                })?;

                return String::from_utf8(plaintext).map_err(|e| {
                    AppError::EncryptionError(format!("Decrypted token is not valid UTF-8: {}", e))
                });
            }

            return Err(AppError::EncryptionError(format!(
                "No key found for key ID: {}",
                key_id
            )));
        } else {
            // Legacy format (version 0): [nonce(12)] + [ciphertext]
            if encrypted_data.len() < 12 {
                return Err(AppError::EncryptionError(
                    "Legacy encrypted data too short to contain nonce".to_string(),
                ));
            }

            let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
            let nonce = Nonce::from_slice(nonce_bytes);

            // Try current key first
            if let Ok(plaintext) = self.current_cipher.decrypt(nonce, ciphertext) {
                return String::from_utf8(plaintext).map_err(|e| {
                    AppError::EncryptionError(format!("Decrypted token is not valid UTF-8: {}", e))
                });
            }

            // Try historical keys
            let historical_ciphers = self.historical_ciphers.read().await;
            for (key_id, cipher) in historical_ciphers.iter() {
                if let Ok(plaintext) = cipher.decrypt(nonce, ciphertext) {
                    if let Ok(token) = String::from_utf8(plaintext) {
                        tracing::debug!(
                            "Successfully decrypted legacy token with historical key: {}",
                            key_id
                        );
                        return Ok(token);
                    }
                }
            }

            return Err(AppError::EncryptionError(
                "Token decryption failed with all available keys".to_string(),
            ));
        }
    }

    /// Encrypt multiple tokens (access and refresh)
    pub fn encrypt_token_pair(
        &self,
        access_token: &str,
        refresh_token: Option<&str>,
    ) -> Result<(Vec<u8>, Option<Vec<u8>>)> {
        let encrypted_access = self.encrypt_token(access_token)?;
        let encrypted_refresh = match refresh_token {
            Some(token) => Some(self.encrypt_token(token)?),
            None => None,
        };
        Ok((encrypted_access, encrypted_refresh))
    }

    /// Decrypt multiple tokens (access and refresh) with async support
    pub async fn decrypt_token_pair(
        &self,
        encrypted_access: &[u8],
        encrypted_refresh: Option<&[u8]>,
    ) -> Result<(String, Option<String>)> {
        let access_token = self.decrypt_token(encrypted_access).await?;
        let refresh_token = match encrypted_refresh {
            Some(encrypted) => Some(self.decrypt_token(encrypted).await?),
            None => None,
        };
        Ok((access_token, refresh_token))
    }

    /// Rotate encryption key and move current key to historical keys
    pub async fn rotate_key(&mut self, new_key: &[u8]) -> Result<()> {
        if new_key.len() != 32 {
            return Err(AppError::ConfigurationError {
                message: "New encryption key must be 32 bytes (256 bits)".to_string(),
            });
        }

        // Move current key to historical keys
        let old_key_id = self.current_key_id.clone();
        let old_cipher = self.current_cipher.clone();

        {
            let mut historical_ciphers = self.historical_ciphers.write().await;
            historical_ciphers.insert(old_key_id, old_cipher);

            // Cleanup old keys if we exceed the limit
            if historical_ciphers.len() > self.key_rotation_config.max_historical_keys {
                // Remove oldest keys (this is a simple implementation - in production you might want to track timestamps)
                let keys_to_remove =
                    historical_ciphers.len() - self.key_rotation_config.max_historical_keys;
                let keys: Vec<String> = historical_ciphers
                    .keys()
                    .take(keys_to_remove)
                    .cloned()
                    .collect();
                for key in keys {
                    historical_ciphers.remove(&key);
                    tracing::info!("Removed old encryption key from historical keys: {}", key);
                }
            }
        }

        // Set new current key
        let key = Key::<Aes256Gcm>::from_slice(new_key);
        self.current_cipher = Aes256Gcm::new(key);
        self.current_key_id = Self::generate_key_id(new_key);
        self.key_rotation_config.last_rotation = Some(Utc::now());

        tracing::info!(
            new_key_id = %self.current_key_id,
            historical_keys_count = self.historical_ciphers.read().await.len(),
            "OAuth encryption key rotated successfully"
        );

        Ok(())
    }

    /// Check if key rotation is needed based on configuration
    pub fn needs_key_rotation(&self) -> bool {
        match self.key_rotation_config.last_rotation {
            Some(last_rotation) => {
                let rotation_threshold =
                    last_rotation + Duration::days(self.key_rotation_config.rotation_interval_days);
                Utc::now() > rotation_threshold
            }
            None => true, // Never rotated, should rotate
        }
    }

    /// Get current key ID for tracking
    pub fn current_key_id(&self) -> &str {
        &self.current_key_id
    }

    /// Get number of historical keys
    pub async fn historical_keys_count(&self) -> usize {
        self.historical_ciphers.read().await.len()
    }

    /// Re-encrypt token with current key (for key rotation migration)
    pub async fn re_encrypt_token(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        let token = self.decrypt_token(encrypted_data).await?;
        self.encrypt_token(&token)
    }

    /// Batch re-encrypt multiple tokens for performance
    pub async fn batch_re_encrypt_tokens(
        &self,
        encrypted_tokens: Vec<&[u8]>,
    ) -> Result<Vec<Vec<u8>>> {
        let mut results = Vec::with_capacity(encrypted_tokens.len());

        for encrypted_token in encrypted_tokens {
            let re_encrypted = self.re_encrypt_token(encrypted_token).await?;
            results.push(re_encrypted);
        }

        Ok(results)
    }
}

impl Default for OAuthTokenEncryption {
    fn default() -> Self {
        Self::new().expect("Failed to initialize OAuth token encryption")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn setup_test_key() {
        let test_key = OAuthTokenEncryption::generate_key_base64();
        env::set_var("OAUTH_ENCRYPTION_KEY", test_key);
    }

    #[test]
    fn test_key_generation() {
        let key1 = OAuthTokenEncryption::generate_key();
        let key2 = OAuthTokenEncryption::generate_key();

        // Keys should be different
        assert_ne!(key1, key2);

        // Keys should be 32 bytes
        assert_eq!(key1.len(), 32);
        assert_eq!(key2.len(), 32);
    }

    #[test]
    fn test_base64_key_generation() {
        let key_base64 = OAuthTokenEncryption::generate_key_base64();

        // Should be valid base64
        let decoded = general_purpose::STANDARD.decode(&key_base64).unwrap();
        assert_eq!(decoded.len(), 32);
    }

    #[tokio::test]
    async fn test_encryption_with_key() {
        let key = OAuthTokenEncryption::generate_key();
        let encryption = OAuthTokenEncryption::with_key(&key).unwrap();

        let token = "test_access_token_12345";
        let encrypted = encryption.encrypt_token(token).unwrap();
        let decrypted = encryption.decrypt_token(&encrypted).await.unwrap();

        assert_eq!(token, decrypted);
    }

    #[tokio::test]
    async fn test_encryption_with_env_key() {
        setup_test_key();

        let encryption = OAuthTokenEncryption::new().unwrap();

        let token = "test_access_token_67890";
        let encrypted = encryption.encrypt_token(token).unwrap();
        let decrypted = encryption.decrypt_token(&encrypted).await.unwrap();

        assert_eq!(token, decrypted);
    }

    #[tokio::test]
    async fn test_token_pair_encryption() {
        let key = OAuthTokenEncryption::generate_key();
        let encryption = OAuthTokenEncryption::with_key(&key).unwrap();

        let access_token = "access_token_123";
        let refresh_token = Some("refresh_token_456");

        let (encrypted_access, encrypted_refresh) = encryption
            .encrypt_token_pair(access_token, refresh_token.as_deref())
            .unwrap();

        let (decrypted_access, decrypted_refresh) = encryption
            .decrypt_token_pair(&encrypted_access, encrypted_refresh.as_deref())
            .await
            .unwrap();

        assert_eq!(access_token, decrypted_access);
        assert_eq!(refresh_token, decrypted_refresh.as_deref());
    }

    #[tokio::test]
    async fn test_token_pair_encryption_no_refresh() {
        let key = OAuthTokenEncryption::generate_key();
        let encryption = OAuthTokenEncryption::with_key(&key).unwrap();

        let access_token = "access_token_only";

        let (encrypted_access, encrypted_refresh) =
            encryption.encrypt_token_pair(access_token, None).unwrap();

        assert!(encrypted_refresh.is_none());

        let (decrypted_access, decrypted_refresh) = encryption
            .decrypt_token_pair(&encrypted_access, None)
            .await
            .unwrap();

        assert_eq!(access_token, decrypted_access);
        assert!(decrypted_refresh.is_none());
    }

    #[tokio::test]
    async fn test_encryption_different_nonces() {
        let key = OAuthTokenEncryption::generate_key();
        let encryption = OAuthTokenEncryption::with_key(&key).unwrap();

        let token = "same_token";
        let encrypted1 = encryption.encrypt_token(token).unwrap();
        let encrypted2 = encryption.encrypt_token(token).unwrap();

        // Same token should produce different ciphertext due to random nonces
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to the same token
        let decrypted1 = encryption.decrypt_token(&encrypted1).await.unwrap();
        let decrypted2 = encryption.decrypt_token(&encrypted2).await.unwrap();

        assert_eq!(token, decrypted1);
        assert_eq!(token, decrypted2);
    }

    #[test]
    fn test_invalid_key_length() {
        let short_key = [0u8; 16]; // Too short
        let result = OAuthTokenEncryption::with_key(&short_key);
        assert!(result.is_err());

        let long_key = [0u8; 64]; // Too long
        let result = OAuthTokenEncryption::with_key(&long_key);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_encrypted_data() {
        let key = OAuthTokenEncryption::generate_key();
        let encryption = OAuthTokenEncryption::with_key(&key).unwrap();

        // Too short data
        let short_data = [0u8; 5];
        let result = encryption.decrypt_token(&short_data).await;
        assert!(result.is_err());

        // Invalid ciphertext
        let invalid_data = [0u8; 32];
        let result = encryption.decrypt_token(&invalid_data).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_key_rotation() {
        let key1 = OAuthTokenEncryption::generate_key();
        let mut encryption = OAuthTokenEncryption::with_key(&key1).unwrap();

        let token = "test_token_for_rotation";
        let encrypted_with_key1 = encryption.encrypt_token(token).unwrap();

        // Rotate to new key
        let key2 = OAuthTokenEncryption::generate_key();
        encryption.rotate_key(&key2).await.unwrap();

        // Should still be able to decrypt old token
        let decrypted_old = encryption
            .decrypt_token(&encrypted_with_key1)
            .await
            .unwrap();
        assert_eq!(token, decrypted_old);

        // New encryptions should use new key
        let encrypted_with_key2 = encryption.encrypt_token(token).unwrap();
        let decrypted_new = encryption
            .decrypt_token(&encrypted_with_key2)
            .await
            .unwrap();
        assert_eq!(token, decrypted_new);

        // Encrypted data should be different (different keys)
        assert_ne!(encrypted_with_key1, encrypted_with_key2);
    }

    #[tokio::test]
    async fn test_legacy_format_compatibility() {
        let key = OAuthTokenEncryption::generate_key();
        let encryption = OAuthTokenEncryption::with_key(&key).unwrap();

        let token = "legacy_token";

        // Create legacy format manually: [nonce(12)] + [ciphertext]
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = encryption
            .current_cipher
            .encrypt(nonce, token.as_bytes())
            .unwrap();

        let mut legacy_encrypted = Vec::with_capacity(12 + ciphertext.len());
        legacy_encrypted.extend_from_slice(&nonce_bytes);
        legacy_encrypted.extend_from_slice(&ciphertext);

        // Should be able to decrypt legacy format
        let decrypted = encryption.decrypt_token(&legacy_encrypted).await.unwrap();
        assert_eq!(token, decrypted);
    }

    #[tokio::test]
    async fn test_re_encryption() {
        let key1 = OAuthTokenEncryption::generate_key();
        let mut encryption = OAuthTokenEncryption::with_key(&key1).unwrap();

        let token = "token_to_re_encrypt";
        let encrypted_old = encryption.encrypt_token(token).unwrap();

        // Rotate key
        let key2 = OAuthTokenEncryption::generate_key();
        encryption.rotate_key(&key2).await.unwrap();

        // Re-encrypt with new key
        let encrypted_new = encryption.re_encrypt_token(&encrypted_old).await.unwrap();

        // Should decrypt to same token
        let decrypted = encryption.decrypt_token(&encrypted_new).await.unwrap();
        assert_eq!(token, decrypted);

        // But encrypted data should be different
        assert_ne!(encrypted_old, encrypted_new);
    }
}
