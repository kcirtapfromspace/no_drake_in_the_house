use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: Option<String>, // None for OAuth-only users
    pub email_verified: bool,
    pub totp_secret: Option<String>,
    pub totp_enabled: bool,
    pub oauth_providers: Vec<OAuthProviderInfo>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub settings: UserSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub two_factor_enabled: bool,
    pub email_notifications: bool,
    pub privacy_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OAuthProvider {
    Google,
    Apple,
    Spotify,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OAuthProviderInfo {
    pub provider: OAuthProvider,
    pub provider_user_id: String,
    pub email: String,
    pub verified: bool,
    pub connected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub totp_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthLoginRequest {
    pub provider: OAuthProvider,
    pub authorization_code: String,
    pub redirect_uri: String,
    pub state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64, // seconds
    pub token_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSetupRequest {
    pub totp_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSetupResponse {
    pub secret: String,
    pub qr_code_url: String,
    pub backup_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpEnableRequest {
    pub totp_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpDisableRequest {
    pub totp_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: UserProfile,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub email_verified: bool,
    pub totp_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub settings: UserSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpVerifyRequest {
    pub user_id: Uuid,
    pub totp_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpStatusResponse {
    pub enabled: bool,
}

impl User {
    pub fn new(email: String, password_hash: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            email,
            password_hash,
            email_verified: false,
            totp_secret: None,
            totp_enabled: false,
            oauth_providers: Vec::new(),
            created_at: now,
            updated_at: now,
            last_login: None,
            settings: UserSettings::default(),
        }
    }

    pub fn with_oauth_provider(email: String, provider: OAuthProviderInfo) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            email,
            password_hash: None,
            email_verified: provider.verified,
            totp_secret: None,
            totp_enabled: false,
            oauth_providers: vec![provider],
            created_at: now,
            updated_at: now,
            last_login: None,
            settings: UserSettings::default(),
        }
    }

    pub fn add_oauth_provider(&mut self, provider: OAuthProviderInfo) {
        // Remove existing provider of same type
        self.oauth_providers.retain(|p| p.provider != provider.provider);
        self.oauth_providers.push(provider);
        self.updated_at = Utc::now();
    }

    pub fn enable_totp(&mut self, secret: String) {
        self.totp_secret = Some(secret);
        self.totp_enabled = true;
        self.settings.two_factor_enabled = true;
        self.updated_at = Utc::now();
    }

    pub fn disable_totp(&mut self) {
        self.totp_secret = None;
        self.totp_enabled = false;
        self.settings.two_factor_enabled = false;
        self.updated_at = Utc::now();
    }

    pub fn update_last_login(&mut self) {
        let now = Utc::now();
        self.last_login = Some(now);
        self.updated_at = now;
    }
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            two_factor_enabled: false,
            email_notifications: true,
            privacy_mode: false,
        }
    }
}

impl OAuthProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "google",
            OAuthProvider::Apple => "apple",
            OAuthProvider::Spotify => "spotify",
        }
    }
}

impl OAuthProviderInfo {
    pub fn new(provider: OAuthProvider, provider_user_id: String, email: String, verified: bool) -> Self {
        Self {
            provider,
            provider_user_id,
            email,
            verified,
            connected_at: Utc::now(),
        }
    }
}