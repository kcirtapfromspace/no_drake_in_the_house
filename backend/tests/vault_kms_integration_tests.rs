//! Integration tests for HashiCorp Vault KMS provider.
//!
//! These tests require a running Vault dev server. To run:
//!
//! ```bash
//! # Start Vault dev server (root token is "root")
//! vault server -dev -dev-root-token-id="root"
//!
//! # Enable Transit secrets engine and create key
//! export VAULT_ADDR='http://127.0.0.1:8200'
//! vault secrets enable transit
//! vault write -f transit/keys/token-vault
//!
//! # Run the tests
//! cargo test vault_kms --test vault_kms_integration_tests
//! ```
//!
//! Tests are marked with #[ignore] by default since they require external infrastructure.

use music_streaming_blocklist_backend::services::{
    KmsProvider, VaultAuthMethod, VaultConfig, VaultKmsProvider,
};

/// Helper to check if Vault dev server is available.
async fn vault_is_available() -> bool {
    let client = reqwest::Client::new();
    let addr = std::env::var("VAULT_ADDR").unwrap_or_else(|_| "http://127.0.0.1:8200".to_string());

    match client
        .get(format!("{}/v1/sys/health", addr))
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
    {
        Ok(resp) => resp.status().is_success() || resp.status().as_u16() == 429,
        Err(_) => false,
    }
}

/// Helper to create a test provider connected to Vault dev server.
fn create_test_provider() -> VaultKmsProvider {
    let addr = std::env::var("VAULT_ADDR").unwrap_or_else(|_| "http://127.0.0.1:8200".to_string());
    let token = std::env::var("VAULT_TOKEN").unwrap_or_else(|_| "root".to_string());

    let config = VaultConfig {
        addr,
        auth: VaultAuthMethod::Token(token),
        transit_mount: "transit".to_string(),
        transit_key: "token-vault".to_string(),
        namespace: None,
    };

    VaultKmsProvider::new(config)
}

#[tokio::test]
#[ignore = "requires Vault dev server"]
async fn test_vault_generate_data_key() {
    if !vault_is_available().await {
        eprintln!("Vault not available, skipping test");
        return;
    }

    let provider = create_test_provider();

    let result = provider.generate_data_key("test-key-1");
    assert!(
        result.is_ok(),
        "generate_data_key failed: {:?}",
        result.err()
    );

    let data_key = result.unwrap();
    assert_eq!(data_key.key_id, "test-key-1");
    assert_eq!(data_key.plaintext_key.len(), 32); // 256-bit key
    assert!(!data_key.encrypted_key.is_empty());

    // Vault ciphertext format: vault:vN:base64data
    let ciphertext = String::from_utf8(data_key.encrypted_key.clone()).unwrap();
    assert!(
        ciphertext.starts_with("vault:v"),
        "Expected Vault ciphertext format, got: {}",
        ciphertext
    );

    assert_eq!(data_key.version, 1);
}

#[tokio::test]
#[ignore = "requires Vault dev server"]
async fn test_vault_decrypt_data_key() {
    if !vault_is_available().await {
        eprintln!("Vault not available, skipping test");
        return;
    }

    let provider = create_test_provider();

    // Generate a key
    let data_key = provider.generate_data_key("test-key-2").unwrap();

    // Decrypt it
    let decrypted = provider
        .decrypt_data_key(&data_key.encrypted_key, "test-key-2")
        .expect("decrypt_data_key failed");

    assert_eq!(
        decrypted, data_key.plaintext_key,
        "Decrypted key doesn't match original"
    );
}

#[tokio::test]
#[ignore = "requires Vault dev server"]
async fn test_vault_rotate_key() {
    if !vault_is_available().await {
        eprintln!("Vault not available, skipping test");
        return;
    }

    let provider = create_test_provider();

    // Generate a key
    let data_key = provider.generate_data_key("test-key-3").unwrap();
    let original_plaintext = data_key.plaintext_key.clone();

    // Rewrap the key (re-encrypt under same/latest key version)
    let rewrapped = provider
        .rotate_key(&data_key.encrypted_key, "test-key-3")
        .expect("rotate_key failed");

    // Ciphertext may be the same or different depending on Vault version
    // but decryption should work and yield same plaintext
    let decrypted = provider
        .decrypt_data_key(&rewrapped, "test-key-3")
        .expect("decrypt after rotate failed");

    assert_eq!(
        decrypted, original_plaintext,
        "Rotated key decrypts to different value"
    );
}

#[tokio::test]
#[ignore = "requires Vault dev server"]
async fn test_vault_multiple_keys() {
    if !vault_is_available().await {
        eprintln!("Vault not available, skipping test");
        return;
    }

    let provider = create_test_provider();

    // Generate multiple keys
    let key1 = provider.generate_data_key("user-1-spotify").unwrap();
    let key2 = provider.generate_data_key("user-2-apple").unwrap();
    let key3 = provider.generate_data_key("user-1-apple").unwrap();

    // All keys should be different
    assert_ne!(key1.plaintext_key, key2.plaintext_key);
    assert_ne!(key2.plaintext_key, key3.plaintext_key);
    assert_ne!(key1.plaintext_key, key3.plaintext_key);

    // All should decrypt correctly
    let dec1 = provider
        .decrypt_data_key(&key1.encrypted_key, "user-1-spotify")
        .unwrap();
    let dec2 = provider
        .decrypt_data_key(&key2.encrypted_key, "user-2-apple")
        .unwrap();
    let dec3 = provider
        .decrypt_data_key(&key3.encrypted_key, "user-1-apple")
        .unwrap();

    assert_eq!(dec1, key1.plaintext_key);
    assert_eq!(dec2, key2.plaintext_key);
    assert_eq!(dec3, key3.plaintext_key);
}

#[tokio::test]
#[ignore = "requires Vault dev server"]
async fn test_vault_invalid_ciphertext() {
    if !vault_is_available().await {
        eprintln!("Vault not available, skipping test");
        return;
    }

    let provider = create_test_provider();

    // Try to decrypt invalid ciphertext
    let invalid = b"not-a-valid-vault-ciphertext";
    let result = provider.decrypt_data_key(invalid, "test-key");

    assert!(
        result.is_err(),
        "Expected error for invalid ciphertext, got: {:?}",
        result
    );
}

#[tokio::test]
#[ignore = "requires Vault dev server"]
async fn test_vault_connection_error_handling() {
    // Use a non-existent address
    let config = VaultConfig {
        addr: "http://127.0.0.1:59999".to_string(),
        auth: VaultAuthMethod::Token("invalid".to_string()),
        transit_mount: "transit".to_string(),
        transit_key: "token-vault".to_string(),
        namespace: None,
    };

    let provider = VaultKmsProvider::new(config);

    // Should return an error, not panic
    let result = provider.generate_data_key("test-key");
    assert!(result.is_err(), "Expected connection error");

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Failed to connect") || err_msg.contains("error"),
        "Unexpected error message: {}",
        err_msg
    );
}

#[tokio::test]
#[ignore = "requires Vault dev server with AppRole"]
async fn test_vault_approle_auth() {
    // This test requires AppRole to be configured in Vault:
    // vault auth enable approle
    // vault write auth/approle/role/test-role \
    //     token_policies="transit-policy" \
    //     token_ttl=1h \
    //     token_max_ttl=4h
    // vault read -field=role_id auth/approle/role/test-role/role-id
    // vault write -f -field=secret_id auth/approle/role/test-role/secret-id

    let addr = std::env::var("VAULT_ADDR").unwrap_or_else(|_| "http://127.0.0.1:8200".to_string());
    let role_id = match std::env::var("VAULT_ROLE_ID") {
        Ok(id) => id,
        Err(_) => {
            eprintln!("VAULT_ROLE_ID not set, skipping AppRole test");
            return;
        }
    };
    let secret_id = match std::env::var("VAULT_SECRET_ID") {
        Ok(id) => id,
        Err(_) => {
            eprintln!("VAULT_SECRET_ID not set, skipping AppRole test");
            return;
        }
    };

    let config = VaultConfig {
        addr,
        auth: VaultAuthMethod::AppRole { role_id, secret_id },
        transit_mount: "transit".to_string(),
        transit_key: "token-vault".to_string(),
        namespace: None,
    };

    let provider = VaultKmsProvider::new(config);

    // Should authenticate via AppRole and generate a key
    let result = provider.generate_data_key("approle-test-key");
    assert!(
        result.is_ok(),
        "AppRole auth generate_data_key failed: {:?}",
        result.err()
    );
}

/// Unit tests that don't require Vault
mod unit_tests {
    use super::*;

    #[test]
    fn test_vault_config_builder() {
        let config = VaultConfig {
            addr: "https://vault.example.com:8200".to_string(),
            auth: VaultAuthMethod::Token("s.XXXXX".to_string()),
            transit_mount: "secret-transit".to_string(),
            transit_key: "my-key".to_string(),
            namespace: Some("admin".to_string()),
        };

        assert_eq!(config.addr, "https://vault.example.com:8200");
        assert!(matches!(config.auth, VaultAuthMethod::Token(ref t) if t == "s.XXXXX"));
        assert_eq!(config.transit_mount, "secret-transit");
        assert_eq!(config.transit_key, "my-key");
        assert_eq!(config.namespace, Some("admin".to_string()));
    }

    #[test]
    fn test_vault_provider_creation_doesnt_panic() {
        let config = VaultConfig {
            addr: "http://localhost:8200".to_string(),
            auth: VaultAuthMethod::Token("test".to_string()),
            transit_mount: "transit".to_string(),
            transit_key: "test-key".to_string(),
            namespace: None,
        };

        // Should not panic even with invalid/unreachable server
        let _provider = VaultKmsProvider::new(config);
    }

    #[test]
    fn test_vault_approle_config() {
        let config = VaultConfig {
            addr: "http://localhost:8200".to_string(),
            auth: VaultAuthMethod::AppRole {
                role_id: "role-123".to_string(),
                secret_id: "secret-456".to_string(),
            },
            transit_mount: "transit".to_string(),
            transit_key: "token-vault".to_string(),
            namespace: None,
        };

        assert!(matches!(
            config.auth,
            VaultAuthMethod::AppRole { ref role_id, ref secret_id }
            if role_id == "role-123" && secret_id == "secret-456"
        ));
    }
}
