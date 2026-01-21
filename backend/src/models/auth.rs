use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User roles for role-based access control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    /// Regular user with standard permissions
    User,
    /// Moderator can verify offenses and manage content
    Moderator,
    /// Admin has full system access
    Admin,
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::User
    }
}

impl UserRole {
    /// Check if this role has moderator-level access or higher
    pub fn is_moderator_or_higher(&self) -> bool {
        matches!(self, UserRole::Moderator | UserRole::Admin)
    }

    /// Check if this role has admin-level access
    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    /// Convert from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => UserRole::Admin,
            "moderator" => UserRole::Moderator,
            _ => UserRole::User,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub email: String,
    pub exp: i64,    // expiration timestamp
    pub iat: i64,    // issued at timestamp
    pub jti: String, // JWT ID for token tracking
    pub token_type: TokenType,
    pub scopes: Vec<String>,
    /// User's role for RBAC (defaults to User if not present for backwards compatibility)
    #[serde(default)]
    pub role: UserRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub family_id: Uuid, // For token rotation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token_family: Uuid,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
}

impl Claims {
    pub fn new_access_token(user_id: Uuid, email: String, expires_in_seconds: i64) -> Self {
        let now = Utc::now().timestamp();
        Self {
            sub: user_id.to_string(),
            email,
            exp: now + expires_in_seconds,
            iat: now,
            jti: Uuid::new_v4().to_string(),
            token_type: TokenType::Access,
            scopes: vec!["read".to_string(), "write".to_string()],
            role: UserRole::User,
        }
    }

    /// Create access token with a specific role
    pub fn new_access_token_with_role(
        user_id: Uuid,
        email: String,
        expires_in_seconds: i64,
        role: UserRole,
    ) -> Self {
        let mut claims = Self::new_access_token(user_id, email, expires_in_seconds);
        claims.role = role;
        // Add admin scope if admin role
        if role.is_admin() {
            claims.scopes.push("admin".to_string());
        }
        // Add moderator scope if moderator or higher
        if role.is_moderator_or_higher() {
            claims.scopes.push("moderate".to_string());
        }
        claims
    }

    pub fn new_refresh_token(user_id: Uuid, email: String, expires_in_seconds: i64) -> Self {
        let now = Utc::now().timestamp();
        Self {
            sub: user_id.to_string(),
            email,
            exp: now + expires_in_seconds,
            iat: now,
            jti: Uuid::new_v4().to_string(),
            token_type: TokenType::Refresh,
            scopes: vec!["refresh".to_string()],
            role: UserRole::User,
        }
    }

    /// Check if this token has moderator-level access
    pub fn has_moderator_access(&self) -> bool {
        self.role.is_moderator_or_higher() || self.scopes.contains(&"moderate".to_string())
    }

    /// Check if this token has admin-level access
    pub fn has_admin_access(&self) -> bool {
        self.role.is_admin() || self.scopes.contains(&"admin".to_string())
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    pub fn user_id(&self) -> Result<Uuid, uuid::Error> {
        Uuid::parse_str(&self.sub)
    }
}

impl RefreshToken {
    pub fn new(
        user_id: Uuid,
        token_hash: String,
        expires_in_seconds: i64,
        family_id: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            token_hash,
            expires_at: now + chrono::Duration::seconds(expires_in_seconds),
            created_at: now,
            revoked: false,
            revoked_at: None,
            family_id,
        }
    }

    pub fn revoke(&mut self) {
        self.revoked = true;
        self.revoked_at = Some(Utc::now());
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.revoked && !self.is_expired()
    }
}

// User session for database storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub revoked: Option<bool>,
}

impl Session {
    pub fn new(user_id: Uuid, refresh_token_family: Uuid, expires_in_seconds: i64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            refresh_token_family,
            ip_address: None,
            user_agent: None,
            created_at: now,
            last_used: now,
            expires_at: now + chrono::Duration::seconds(expires_in_seconds),
            revoked: false,
        }
    }

    pub fn update_last_used(&mut self) {
        self.last_used = Utc::now();
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.revoked && !self.is_expired()
    }
}

impl UserSession {
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires_at) => Utc::now() > expires_at,
            None => true,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.revoked.unwrap_or(true) && !self.is_expired()
    }
}
