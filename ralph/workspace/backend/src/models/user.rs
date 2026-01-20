use crate::models::oauth::{OAuthAccount, OAuthProviderType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: Option<String>, // None for OAuth-only users
    pub email_verified: bool,
    pub totp_secret: Option<String>,
    pub totp_enabled: bool,
    pub oauth_accounts: Vec<OAuthAccount>, // Updated to use new OAuth account structure
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

// OAuth provider structures moved to oauth.rs module

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
    pub provider: OAuthProviderType,
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
    pub confirm_password: String,
    pub terms_accepted: bool,
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
    pub oauth_accounts: Vec<OAuthAccountInfo>, // Simplified OAuth account info for profile
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub settings: UserSettings,
}

/// Simplified OAuth account information for user profiles (no sensitive data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthAccountInfo {
    pub provider: OAuthProviderType,
    pub provider_user_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub connected_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

/// Request to link an OAuth account to existing user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkOAuthAccountRequest {
    pub provider: OAuthProviderType,
    pub code: String,
    pub state: String,
}

/// Request to unlink an OAuth account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlinkOAuthAccountRequest {
    pub provider: OAuthProviderType,
}

/// Account merge request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeAccountsRequest {
    pub secondary_user_id: Uuid,
    pub merge_reason: String,
}

/// Account merge result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeAccountsResponse {
    pub primary_user_id: Uuid,
    pub merged_user_id: Uuid,
    pub merged_oauth_accounts: u32,
    pub merged_connections: u32,
    pub merged_at: DateTime<Utc>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationValidationError {
    pub field: String,
    pub message: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationErrorResponse {
    pub errors: Vec<RegistrationValidationError>,
    pub message: String,
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
            oauth_accounts: Vec::new(),
            created_at: now,
            updated_at: now,
            last_login: None,
            settings: UserSettings::default(),
        }
    }

    pub fn with_oauth_account(email: String, oauth_account: OAuthAccount) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            email,
            password_hash: None,
            email_verified: oauth_account.email.is_some(),
            totp_secret: None,
            totp_enabled: false,
            oauth_accounts: vec![oauth_account],
            created_at: now,
            updated_at: now,
            last_login: None,
            settings: UserSettings::default(),
        }
    }

    /// Add an OAuth account to this user
    pub fn add_oauth_account(&mut self, oauth_account: OAuthAccount) {
        // Remove existing account for same provider
        self.oauth_accounts
            .retain(|acc| acc.provider != oauth_account.provider);
        self.oauth_accounts.push(oauth_account);
        self.updated_at = Utc::now();
    }

    /// Remove an OAuth account by provider
    pub fn remove_oauth_account(&mut self, provider: &OAuthProviderType) -> bool {
        let initial_len = self.oauth_accounts.len();
        self.oauth_accounts.retain(|acc| &acc.provider != provider);
        let removed = self.oauth_accounts.len() < initial_len;
        if removed {
            self.updated_at = Utc::now();
        }
        removed
    }

    /// Get OAuth account by provider
    pub fn get_oauth_account(&self, provider: &OAuthProviderType) -> Option<&OAuthAccount> {
        self.oauth_accounts
            .iter()
            .find(|acc| &acc.provider == provider)
    }

    /// Check if user has OAuth account for provider
    pub fn has_oauth_account(&self, provider: &OAuthProviderType) -> bool {
        self.oauth_accounts
            .iter()
            .any(|acc| &acc.provider == provider)
    }

    /// Check if user is OAuth-only (no password)
    pub fn is_oauth_only(&self) -> bool {
        self.password_hash.is_none() && !self.oauth_accounts.is_empty()
    }

    /// Get all linked OAuth providers
    pub fn linked_providers(&self) -> Vec<OAuthProviderType> {
        self.oauth_accounts.iter().map(|acc| acc.provider).collect()
    }

    /// Convert to user profile (safe for API responses)
    pub fn to_profile(&self) -> UserProfile {
        UserProfile {
            id: self.id,
            email: self.email.clone(),
            email_verified: self.email_verified,
            totp_enabled: self.totp_enabled,
            oauth_accounts: self
                .oauth_accounts
                .iter()
                .map(|acc| OAuthAccountInfo {
                    provider: acc.provider,
                    provider_user_id: acc.provider_user_id.clone(),
                    email: acc.email.clone(),
                    display_name: acc.display_name.clone(),
                    avatar_url: acc.avatar_url.clone(),
                    connected_at: acc.created_at,
                    last_used_at: None, // This would come from a separate tracking mechanism
                })
                .collect(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            last_login: self.last_login,
            settings: self.settings.clone(),
        }
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

    /// Merge another user's data into this user
    pub fn merge_from(&mut self, other_user: &User) -> Result<(), String> {
        // Merge OAuth accounts (avoid duplicates)
        for other_account in &other_user.oauth_accounts {
            if !self.has_oauth_account(&other_account.provider) {
                let mut merged_account = other_account.clone();
                merged_account.user_id = self.id; // Update user_id to this user
                self.oauth_accounts.push(merged_account);
            }
        }

        // If this user doesn't have email verification but other does, update it
        if !self.email_verified && other_user.email_verified {
            self.email_verified = true;
        }

        // Update timestamp
        self.updated_at = Utc::now();

        Ok(())
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

impl OAuthAccountInfo {
    pub fn from_oauth_account(account: &OAuthAccount) -> Self {
        Self {
            provider: account.provider,
            provider_user_id: account.provider_user_id.clone(),
            email: account.email.clone(),
            display_name: account.display_name.clone(),
            avatar_url: account.avatar_url.clone(),
            connected_at: account.created_at,
            last_used_at: None, // This would be tracked separately
        }
    }
}
