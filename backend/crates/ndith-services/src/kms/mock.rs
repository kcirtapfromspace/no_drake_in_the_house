//! Mock KMS provider for development and testing.
//!
//! This provider uses a locally generated master key for encryption.
//! It is NOT suitable for production use - use a real KMS provider instead.

use super::KmsProvider;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm,
};
use anyhow::{anyhow, Result};
use base64::prelude::*;
use ndith_core::models::DataKey;
use ring::rand::{SecureRandom, SystemRandom};

/// Mock KMS provider for development and testing.
///
/// Uses a locally generated 256-bit master key to encrypt/decrypt data keys.
/// The master key is generated at initialization and stored in memory.
///
/// # Security Warning
/// This implementation is NOT suitable for production use:
/// - Master key is stored in memory and lost on restart
/// - No key rotation or versioning of the master key itself
/// - No audit logging or access controls
/// - No hardware security module (HSM) protection
///
/// For production, use AWS KMS, GCP KMS, Azure Key Vault, or HashiCorp Vault.
pub struct MockKmsProvider {
    master_key: [u8; 32],
    rng: SystemRandom,
}

impl MockKmsProvider {
    /// Create a new MockKmsProvider with a randomly generated master key.
    ///
    /// For local persistence across backend restarts, set `KMS_MOCK_MASTER_KEY`
    /// as one of:
    /// - 32-byte raw string
    /// - 64-char hex string
    /// - base64-encoded 32-byte key
    pub fn new() -> Self {
        let rng = SystemRandom::new();
        let master_key = Self::master_key_from_env().unwrap_or_else(|| {
            let mut generated = [0u8; 32];
            rng.fill(&mut generated)
                .expect("Failed to generate master key");
            tracing::warn!(
                "KMS_MOCK_MASTER_KEY is not set; generated ephemeral master key (tokens won't survive backend restarts)"
            );
            generated
        });

        Self { master_key, rng }
    }

    fn master_key_from_env() -> Option<[u8; 32]> {
        let raw = std::env::var("KMS_MOCK_MASTER_KEY").ok()?;
        if raw.trim().is_empty() {
            return None;
        }

        match Self::parse_master_key(raw.trim()) {
            Ok(key) => {
                tracing::info!("Loaded mock KMS master key from KMS_MOCK_MASTER_KEY");
                Some(key)
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "Invalid KMS_MOCK_MASTER_KEY; falling back to ephemeral master key"
                );
                None
            }
        }
    }

    fn parse_master_key(value: &str) -> Result<[u8; 32]> {
        // Support hex-encoded 32-byte key.
        if value.len() == 64 && value.chars().all(|c| c.is_ascii_hexdigit()) {
            let mut key = [0u8; 32];
            for (i, chunk) in value.as_bytes().chunks(2).enumerate() {
                let hex_pair = std::str::from_utf8(chunk)
                    .map_err(|e| anyhow!("Invalid UTF-8 in hex key: {}", e))?;
                key[i] = u8::from_str_radix(hex_pair, 16)
                    .map_err(|e| anyhow!("Invalid hex key byte '{}': {}", hex_pair, e))?;
            }
            return Ok(key);
        }

        // Support base64-encoded 32-byte key.
        if let Ok(decoded) = BASE64_STANDARD.decode(value) {
            if decoded.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&decoded);
                return Ok(key);
            }
        }

        // Support raw 32-byte string.
        let bytes = value.as_bytes();
        if bytes.len() == 32 {
            let mut key = [0u8; 32];
            key.copy_from_slice(bytes);
            return Ok(key);
        }

        Err(anyhow!(
            "master key must be 32 raw bytes, 64 hex chars, or base64-encoded 32 bytes"
        ))
    }

    /// Create a MockKmsProvider with a specific master key (for testing).
    #[cfg(test)]
    pub fn with_key(master_key: [u8; 32]) -> Self {
        Self {
            master_key,
            rng: SystemRandom::new(),
        }
    }

    /// Encrypt data using the master key.
    ///
    /// Uses AES-256-GCM with a random 12-byte nonce prepended to the ciphertext.
    fn encrypt_with_master_key(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);

        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        self.rng
            .fill(&mut nonce_bytes)
            .map_err(|_| anyhow!("Failed to generate nonce"))?;
        let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| anyhow!("Failed to encrypt data"))?;

        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);

        Ok(result)
    }

    /// Decrypt data using the master key.
    ///
    /// Expects the nonce to be prepended to the ciphertext (first 12 bytes).
    fn decrypt_with_master_key(&self, encrypted: &[u8]) -> Result<Vec<u8>> {
        if encrypted.len() < 12 {
            return Err(anyhow!("Invalid encrypted data: too short"));
        }

        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);

        // Extract nonce and ciphertext
        let nonce = aes_gcm::Nonce::from_slice(&encrypted[..12]);
        let ciphertext = &encrypted[12..];

        // Decrypt
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| anyhow!("Failed to decrypt data"))
    }
}

impl Default for MockKmsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl KmsProvider for MockKmsProvider {
    fn generate_data_key(&self, key_id: &str) -> Result<DataKey> {
        // Generate 256-bit data key
        let mut plaintext_key = [0u8; 32];
        self.rng
            .fill(&mut plaintext_key)
            .map_err(|_| anyhow!("Failed to generate data key"))?;

        // Encrypt the data key with master key
        let encrypted_key = self.encrypt_with_master_key(&plaintext_key)?;

        Ok(DataKey::new(
            key_id.to_string(),
            plaintext_key.to_vec(),
            encrypted_key,
            1, // Initial version
        ))
    }

    fn decrypt_data_key(&self, encrypted_key: &[u8], _key_id: &str) -> Result<Vec<u8>> {
        self.decrypt_with_master_key(encrypted_key)
    }

    fn rotate_key(&self, encrypted_key: &[u8], _key_id: &str) -> Result<Vec<u8>> {
        // For MockKmsProvider, rotation just re-encrypts with the same master key
        // (since we don't have master key versioning)
        // In a real KMS, this would re-encrypt under a new master key version

        // First decrypt with current key
        let plaintext = self.decrypt_with_master_key(encrypted_key)?;

        // Re-encrypt (with new nonce)
        self.encrypt_with_master_key(&plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_data_key() {
        let kms = MockKmsProvider::new();

        let result = kms.generate_data_key("test-key-id");
        assert!(result.is_ok());

        let data_key = result.unwrap();
        assert_eq!(data_key.key_id, "test-key-id");
        assert_eq!(data_key.plaintext_key.len(), 32); // 256-bit key
        assert!(!data_key.encrypted_key.is_empty());
        assert!(data_key.encrypted_key.len() > 12); // At least nonce + some ciphertext
        assert_eq!(data_key.version, 1);
    }

    #[test]
    fn test_decrypt_data_key() {
        let kms = MockKmsProvider::new();

        // Generate a key
        let data_key = kms.generate_data_key("test-key").unwrap();

        // Decrypt it
        let decrypted = kms
            .decrypt_data_key(&data_key.encrypted_key, "test-key")
            .unwrap();

        assert_eq!(decrypted, data_key.plaintext_key);
    }

    #[test]
    fn test_rotate_key() {
        let kms = MockKmsProvider::new();

        // Generate a key
        let data_key = kms.generate_data_key("test-key").unwrap();
        let original_plaintext = data_key.plaintext_key.clone();

        // Rotate
        let rotated_encrypted = kms.rotate_key(&data_key.encrypted_key, "test-key").unwrap();

        // Encrypted form should be different (different nonce)
        assert_ne!(rotated_encrypted, data_key.encrypted_key);

        // But plaintext should be the same
        let decrypted = kms
            .decrypt_data_key(&rotated_encrypted, "test-key")
            .unwrap();
        assert_eq!(decrypted, original_plaintext);
    }

    #[test]
    fn test_decrypt_invalid_data() {
        let kms = MockKmsProvider::new();

        // Too short
        let result = kms.decrypt_data_key(&[1, 2, 3], "test-key");
        assert!(result.is_err());

        // Invalid ciphertext
        let invalid_data = vec![0u8; 32];
        let result = kms.decrypt_data_key(&invalid_data, "test-key");
        assert!(result.is_err());
    }

    #[test]
    fn test_different_instances_have_different_master_keys() {
        let kms1 = MockKmsProvider::new();
        let kms2 = MockKmsProvider::new();

        // Generate key with first instance
        let data_key = kms1.generate_data_key("test-key").unwrap();

        // Try to decrypt with second instance (should fail)
        let result = kms2.decrypt_data_key(&data_key.encrypted_key, "test-key");
        assert!(result.is_err());
    }

    #[test]
    fn test_with_known_key() {
        let master_key = [42u8; 32];
        let kms1 = MockKmsProvider::with_key(master_key);
        let kms2 = MockKmsProvider::with_key(master_key);

        // Generate key with first instance
        let data_key = kms1.generate_data_key("test-key").unwrap();

        // Decrypt with second instance (same master key)
        let decrypted = kms2
            .decrypt_data_key(&data_key.encrypted_key, "test-key")
            .unwrap();
        assert_eq!(decrypted, data_key.plaintext_key);
    }
}
