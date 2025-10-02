use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// OAuth provider types supported by the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OAuthProviderType {
    Google,
    Apple,
    GitHub,
}

impl std::fmt::Display for OAuthProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OAuthProviderType::Google => write!(f, "google"),
            OAuthProviderType::Apple => write!(f, "apple"),
            OAuthProviderType::GitHub => write!(f, "github"),
        }
    }
}

impl std::str::FromStr for OAuthProviderType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "google" => Ok(OAuthProviderType::Google),
            "apple" => Ok(OAuthProviderType::Apple),
            "github" => Ok(OAuthProviderType::GitHub),
            _ => Err(format!("Unknown OAuth provider: {}", s)),
        }
    }
}

/// OAuth tokens returned from provider token exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    pub token_type: String,
    pub scope: Option<String>,
    pub id_token: Option<String>, // For OpenID Connect providers
}

/// User information retrieved from OAuth provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub provider_user_id: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub display_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub locale: Option<String>,
    pub provider_data: HashMap<String, serde_json::Value>, // Additional provider-specific data
}

/// OAuth flow initiation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthFlowResponse {
    pub authorization_url: String,
    pub state: String,
    pub code_verifier: Option<String>, // For PKCE flow
}

/// OAuth account stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: OAuthProviderType,
    pub provider_user_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub access_token_encrypted: Vec<u8>, // AES-GCM encrypted
    pub refresh_token_encrypted: Option<Vec<u8>>, // AES-GCM encrypted
    pub token_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// OAuth flow state for CSRF protection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthState {
    pub state_token: String,
    pub provider: OAuthProviderType,
    pub redirect_uri: String,
    pub code_verifier: Option<String>, // For PKCE
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// OAuth configuration for a provider
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub additional_params: HashMap<String, String>,
}

/// Account linking request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountLinkRequest {
    pub provider: OAuthProviderType,
    pub code: String,
    pub state: String,
}

/// Account merge audit entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountMerge {
    pub id: Uuid,
    pub primary_user_id: Uuid,
    pub merged_user_id: Uuid,
    pub merged_at: DateTime<Utc>,
    pub merged_by: Option<Uuid>,
    pub merge_reason: String,
}

impl OAuthAccount {
    pub fn new(
        user_id: Uuid,
        provider: OAuthProviderType,
        provider_user_id: String,
        email: Option<String>,
        display_name: Option<String>,
        avatar_url: Option<String>,
        access_token_encrypted: Vec<u8>,
        refresh_token_encrypted: Option<Vec<u8>>,
        token_expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            provider,
            provider_user_id,
            email,
            display_name,
            avatar_url,
            access_token_encrypted,
            refresh_token_encrypted,
            token_expires_at,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_token_expired(&self) -> bool {
        match self.token_expires_at {
            Some(expires_at) => Utc::now() > expires_at,
            None => false, // No expiration means token doesn't expire
        }
    }
}

impl OAuthState {
    pub fn new(
        provider: OAuthProviderType,
        redirect_uri: String,
        code_verifier: Option<String>,
        expires_in_seconds: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            state_token: Uuid::new_v4().to_string(),
            provider,
            redirect_uri,
            code_verifier,
            created_at: now,
            expires_at: now + chrono::Duration::seconds(expires_in_seconds),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_valid(&self, state_token: &str, provider: &OAuthProviderType) -> bool {
        !self.is_expired() && self.state_token == state_token && &self.provider == provider
    }
}

/// OAuth token status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenStatus {
    pub provider: OAuthProviderType,
    pub status: TokenExpirationStatus,
    pub has_refresh_token: bool,
    pub last_refreshed: DateTime<Utc>,
}

/// Token expiration status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenExpirationStatus {
    Valid { expires_at: DateTime<Utc> },
    ExpiringSoon { hours_remaining: u32 },
    Expired,
    NoExpiration,
}

/// Token refresh schedule entry
#[derive(Debug, Clone)]
pub struct TokenRefreshSchedule {
    pub user_id: Uuid,
    pub provider: OAuthProviderType,
    pub expires_at: DateTime<Utc>,
    pub refresh_priority: RefreshPriority,
}

/// Priority for token refresh operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefreshPriority {
    High,   // Expires within 6 hours
    Normal, // Expires within 24 hours
}