use base64::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a connection to a streaming service provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: StreamingProvider,
    pub provider_user_id: String,
    pub scopes: Vec<String>,
    pub access_token_encrypted: Option<String>,
    pub refresh_token_encrypted: Option<String>,
    pub token_version: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: ConnectionStatus,
    pub last_health_check: Option<DateTime<Utc>>,
    pub error_code: Option<String>,
    /// Key ID used for envelope encryption (e.g., user-{uuid}-spotify)
    pub data_key_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StreamingProvider {
    Spotify,
    Apple,
    AppleMusic,
    YouTubeMusic,
    Tidal,
}

impl std::fmt::Display for StreamingProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamingProvider::Spotify => write!(f, "spotify"),
            StreamingProvider::Apple => write!(f, "apple"),
            StreamingProvider::AppleMusic => write!(f, "apple_music"),
            StreamingProvider::YouTubeMusic => write!(f, "youtube_music"),
            StreamingProvider::Tidal => write!(f, "tidal"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionStatus {
    Active,
    Expired,
    Revoked,
    Error,
    NeedsReauth,
}

/// Encrypted token data with envelope encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedToken {
    pub encrypted_data: String, // Base64 encoded encrypted token
    pub encrypted_key: String,  // Base64 encoded encrypted data key
    pub nonce: String,          // Base64 encoded nonce/IV
    pub key_id: String,         // KMS key ID used for envelope encryption
    pub version: i32,           // Encryption version for key rotation
}

/// Data encryption key for envelope encryption
#[derive(Debug, Clone)]
pub struct DataKey {
    pub key_id: String,
    pub plaintext_key: Vec<u8>,
    pub encrypted_key: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub version: i32,
}

/// Token health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenHealthCheck {
    pub connection_id: Uuid,
    pub is_valid: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub checked_at: DateTime<Utc>,
    pub needs_refresh: bool,
}

/// Request to store a new provider token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreTokenRequest {
    pub user_id: Uuid,
    pub provider: StreamingProvider,
    pub provider_user_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Response when retrieving a decrypted token
#[derive(Debug, Clone)]
pub struct DecryptedToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub scopes: Vec<String>,
}

/// Token refresh result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRefreshResult {
    pub connection_id: Uuid,
    pub success: bool,
    pub new_access_token: Option<String>,
    pub new_refresh_token: Option<String>,
    pub new_expires_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

impl Connection {
    pub fn new(
        user_id: Uuid,
        provider: StreamingProvider,
        provider_user_id: String,
        scopes: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            provider,
            provider_user_id,
            scopes,
            access_token_encrypted: None,
            refresh_token_encrypted: None,
            token_version: 1,
            expires_at: None,
            status: ConnectionStatus::Active,
            last_health_check: None,
            error_code: None,
            data_key_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new connection with a specific data key ID for encryption
    pub fn with_data_key_id(
        user_id: Uuid,
        provider: StreamingProvider,
        provider_user_id: String,
        scopes: Vec<String>,
        data_key_id: String,
    ) -> Self {
        let mut conn = Self::new(user_id, provider, provider_user_id, scopes);
        conn.data_key_id = Some(data_key_id);
        conn
    }

    /// Set the data key ID for this connection
    pub fn set_data_key_id(&mut self, key_id: String) {
        self.data_key_id = Some(key_id);
        self.updated_at = Utc::now();
    }

    pub fn update_tokens(
        &mut self,
        access_token_encrypted: String,
        refresh_token_encrypted: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    ) {
        self.access_token_encrypted = Some(access_token_encrypted);
        self.refresh_token_encrypted = refresh_token_encrypted;
        self.expires_at = expires_at;
        self.token_version += 1;
        self.updated_at = Utc::now();
        self.status = ConnectionStatus::Active;
        self.error_code = None;
    }

    pub fn mark_error(&mut self, error_code: String) {
        self.status = ConnectionStatus::Error;
        self.error_code = Some(error_code);
        self.updated_at = Utc::now();
    }

    pub fn mark_expired(&mut self) {
        self.status = ConnectionStatus::Expired;
        self.updated_at = Utc::now();
    }

    pub fn mark_needs_reauth(&mut self, reason: String) {
        self.status = ConnectionStatus::NeedsReauth;
        self.error_code = Some(reason);
        self.updated_at = Utc::now();
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    pub fn needs_refresh(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            // Refresh if token expires within 5 minutes
            let refresh_threshold = Utc::now() + chrono::Duration::minutes(5);
            expires_at < refresh_threshold
        } else {
            false
        }
    }
}

impl StreamingProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            StreamingProvider::Spotify => "spotify",
            StreamingProvider::Apple => "apple",
            StreamingProvider::AppleMusic => "apple_music",
            StreamingProvider::YouTubeMusic => "youtube_music",
            StreamingProvider::Tidal => "tidal",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "spotify" => Some(StreamingProvider::Spotify),
            "apple" => Some(StreamingProvider::Apple),
            "apple_music" => Some(StreamingProvider::AppleMusic),
            "youtube_music" | "youtube" => Some(StreamingProvider::YouTubeMusic),
            "tidal" => Some(StreamingProvider::Tidal),
            _ => None,
        }
    }
}

impl EncryptedToken {
    pub fn new(
        encrypted_data: Vec<u8>,
        encrypted_key: Vec<u8>,
        nonce: Vec<u8>,
        key_id: String,
        version: i32,
    ) -> Self {
        Self {
            encrypted_data: base64::prelude::BASE64_STANDARD.encode(encrypted_data),
            encrypted_key: base64::prelude::BASE64_STANDARD.encode(encrypted_key),
            nonce: base64::prelude::BASE64_STANDARD.encode(nonce),
            key_id,
            version,
        }
    }

    pub fn get_encrypted_data(&self) -> Result<Vec<u8>, base64::DecodeError> {
        base64::prelude::BASE64_STANDARD.decode(&self.encrypted_data)
    }

    pub fn get_encrypted_key(&self) -> Result<Vec<u8>, base64::DecodeError> {
        base64::prelude::BASE64_STANDARD.decode(&self.encrypted_key)
    }

    pub fn get_nonce(&self) -> Result<Vec<u8>, base64::DecodeError> {
        base64::prelude::BASE64_STANDARD.decode(&self.nonce)
    }
}

impl DataKey {
    pub fn new(
        key_id: String,
        plaintext_key: Vec<u8>,
        encrypted_key: Vec<u8>,
        version: i32,
    ) -> Self {
        Self {
            key_id,
            plaintext_key,
            encrypted_key,
            created_at: Utc::now(),
            version,
        }
    }

    pub fn should_rotate(&self, rotation_days: i64) -> bool {
        let rotation_threshold = self.created_at + chrono::Duration::days(rotation_days);
        Utc::now() > rotation_threshold
    }
}
