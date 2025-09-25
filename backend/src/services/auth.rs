use crate::models::{User, Claims, RefreshToken, Session, TokenPair, CreateUserRequest, LoginRequest, OAuthLoginRequest, OAuthProvider, TotpSetupResponse};
use anyhow::{Result, anyhow};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use dashmap::DashMap;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
// OAuth functionality removed for compatibility - can be added back with proper dependencies
use rand::Rng;
use std::sync::Arc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

pub struct AuthService {
    // In-memory storage for demo - replace with database in production
    users: Arc<DashMap<Uuid, User>>,
    users_by_email: Arc<DashMap<String, Uuid>>,
    refresh_tokens: Arc<DashMap<Uuid, RefreshToken>>,
    sessions: Arc<DashMap<Uuid, Session>>,
    
    // JWT configuration
    jwt_secret: String,
    access_token_ttl: i64,  // seconds
    refresh_token_ttl: i64, // seconds
    
    // OAuth configuration (simplified for demo)
    oauth_enabled: bool,
}

impl AuthService {
    pub fn new() -> Self {
        // Generate a random JWT secret for demo - use proper secret management in production
        let jwt_secret = format!("jwt_secret_{}", rand::thread_rng().gen::<u64>());
        
        Self {
            users: Arc::new(DashMap::new()),
            users_by_email: Arc::new(DashMap::new()),
            refresh_tokens: Arc::new(DashMap::new()),
            sessions: Arc::new(DashMap::new()),
            jwt_secret,
            access_token_ttl: 15 * 60,      // 15 minutes
            refresh_token_ttl: 7 * 24 * 3600, // 7 days
            oauth_enabled: false,
        }
    }

    pub fn with_oauth_enabled(mut self) -> Self {
        self.oauth_enabled = true;
        self
    }

    // User registration with email/password
    pub async fn register_user(&self, request: CreateUserRequest) -> Result<User> {
        // Check if user already exists
        if self.users_by_email.contains_key(&request.email) {
            return Err(anyhow!("User with this email already exists"));
        }

        // Hash password
        let password_hash = hash(&request.password, DEFAULT_COST)?;
        
        // Create user
        let user = User::new(request.email.clone(), Some(password_hash));
        let user_id = user.id;
        
        // Store user
        self.users.insert(user_id, user.clone());
        self.users_by_email.insert(request.email, user_id);
        
        Ok(user)
    }

    // Login with email/password
    pub async fn login_user(&self, request: LoginRequest) -> Result<TokenPair> {
        // Find user by email
        let user_id = self.users_by_email.get(&request.email)
            .ok_or_else(|| anyhow!("Invalid credentials"))?
            .clone();
        
        let mut user = self.users.get_mut(&user_id)
            .ok_or_else(|| anyhow!("User not found"))?;

        // Verify password
        let password_hash = user.password_hash.as_ref()
            .ok_or_else(|| anyhow!("Password authentication not available for this user"))?;
        
        if !verify(&request.password, password_hash)? {
            return Err(anyhow!("Invalid credentials"));
        }

        // Check 2FA if enabled
        if user.totp_enabled {
            let totp_code = request.totp_code
                .ok_or_else(|| anyhow!("TOTP code required"))?;
            
            if !self.verify_totp(&user, &totp_code)? {
                return Err(anyhow!("Invalid TOTP code"));
            }
        }

        // Update last login
        user.update_last_login();
        
        // Generate tokens
        self.generate_token_pair(user_id, &user.email).await
    }

    // OAuth login (simplified for demo)
    pub async fn oauth_login(&self, _request: OAuthLoginRequest) -> Result<TokenPair> {
        Err(anyhow!("OAuth not implemented in this demo version"))
    }

    // Generate JWT token pair
    async fn generate_token_pair(&self, user_id: Uuid, email: &str) -> Result<TokenPair> {
        // Create session
        let session = Session::new(user_id, Uuid::new_v4(), self.refresh_token_ttl);
        let session_id = session.id;
        self.sessions.insert(session_id, session);

        // Generate access token
        let access_claims = Claims::new_access_token(user_id, email.to_string(), self.access_token_ttl);
        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;

        // Generate refresh token
        let refresh_claims = Claims::new_refresh_token(user_id, email.to_string(), self.refresh_token_ttl);
        let refresh_token_jwt = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;

        // Store refresh token
        let refresh_token_hash = hash(&refresh_token_jwt, DEFAULT_COST)?;
        let refresh_token = RefreshToken::new(
            user_id,
            refresh_token_hash,
            self.refresh_token_ttl,
            session_id,
        );
        self.refresh_tokens.insert(refresh_token.id, refresh_token);

        Ok(TokenPair {
            access_token,
            refresh_token: refresh_token_jwt,
            expires_in: self.access_token_ttl,
            token_type: "Bearer".to_string(),
        })
    }

    // Refresh access token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenPair> {
        // Decode refresh token
        let token_data = decode::<Claims>(
            refresh_token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )?;

        let claims = token_data.claims;
        
        // Verify it's a refresh token
        if !matches!(claims.token_type, crate::models::TokenType::Refresh) {
            return Err(anyhow!("Invalid token type"));
        }

        // Check if token is expired
        if claims.is_expired() {
            return Err(anyhow!("Refresh token expired"));
        }

        let user_id = Uuid::parse_str(&claims.sub)?;
        
        // Find and validate stored refresh token
        let refresh_token_hash = hash(refresh_token, DEFAULT_COST)?;
        let mut found_token = None;
        
        for mut token_entry in self.refresh_tokens.iter_mut() {
            if token_entry.user_id == user_id && verify(refresh_token, &token_entry.token_hash).unwrap_or(false) {
                if token_entry.is_valid() {
                    found_token = Some(token_entry.id);
                    break;
                } else {
                    // Revoke invalid token
                    token_entry.revoke();
                    return Err(anyhow!("Refresh token revoked or expired"));
                }
            }
        }

        if found_token.is_none() {
            return Err(anyhow!("Invalid refresh token"));
        }

        // Generate new token pair (token rotation)
        self.generate_token_pair(user_id, &claims.email).await
    }

    // Verify JWT token
    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        )?;

        let claims = token_data.claims;
        
        if claims.is_expired() {
            return Err(anyhow!("Token expired"));
        }

        Ok(claims)
    }

    // Setup TOTP for user
    pub async fn setup_totp(&self, user_id: Uuid) -> Result<TotpSetupResponse> {
        let mut user = self.users.get_mut(&user_id)
            .ok_or_else(|| anyhow!("User not found"))?;

        // Generate TOTP secret
        let secret = self.generate_totp_secret();
        let secret_b32 = base32::encode(base32::Alphabet::Rfc4648 { padding: true }, &secret);
        
        // Generate QR code URL
        let qr_code_url = format!(
            "otpauth://totp/MusicBlocklist:{}?secret={}&issuer=MusicBlocklist",
            urlencoding::encode(&user.email),
            secret_b32
        );

        // Generate backup codes
        let backup_codes = self.generate_backup_codes();
        
        // Store secret (not enabled until verified)
        user.totp_secret = Some(secret_b32.clone());
        
        Ok(TotpSetupResponse {
            secret: secret_b32,
            qr_code_url,
            backup_codes,
        })
    }

    // Enable TOTP after verification
    pub async fn enable_totp(&self, user_id: Uuid, totp_code: &str) -> Result<()> {
        let mut user = self.users.get_mut(&user_id)
            .ok_or_else(|| anyhow!("User not found"))?;

        if !self.verify_totp(&user, totp_code)? {
            return Err(anyhow!("Invalid TOTP code"));
        }

        user.totp_enabled = true;
        user.settings.two_factor_enabled = true;
        
        Ok(())
    }

    // Verify TOTP code (simplified implementation)
    fn verify_totp(&self, user: &User, code: &str) -> Result<bool> {
        let secret = user.totp_secret.as_ref()
            .ok_or_else(|| anyhow!("TOTP not configured for user"))?;
        
        let secret_bytes = base32::decode(base32::Alphabet::Rfc4648 { padding: true }, secret)
            .ok_or_else(|| anyhow!("Invalid TOTP secret"))?;
        
        let current_time = Utc::now().timestamp() as u64;
        
        // Simple TOTP implementation - check current 30-second window
        let time_step = current_time / 30;
        let expected_code = self.generate_totp_code(&secret_bytes, time_step)?;
        
        Ok(expected_code == code)
    }

    // Simple TOTP code generation
    fn generate_totp_code(&self, secret: &[u8], time_step: u64) -> Result<String> {
        let time_bytes = time_step.to_be_bytes();
        
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(secret)?;
        mac.update(&time_bytes);
        let result = mac.finalize().into_bytes();
        
        let offset = (result[result.len() - 1] & 0xf) as usize;
        let code = ((result[offset] as u32 & 0x7f) << 24)
            | ((result[offset + 1] as u32 & 0xff) << 16)
            | ((result[offset + 2] as u32 & 0xff) << 8)
            | (result[offset + 3] as u32 & 0xff);
        
        Ok(format!("{:06}", code % 1000000))
    }

    // Revoke all user sessions
    pub async fn revoke_all_sessions(&self, user_id: Uuid) -> Result<()> {
        // Revoke all refresh tokens for user
        for mut token in self.refresh_tokens.iter_mut() {
            if token.user_id == user_id {
                token.revoke();
            }
        }

        // Revoke all sessions for user
        for mut session in self.sessions.iter_mut() {
            if session.user_id == user_id {
                session.revoked = true;
            }
        }

        Ok(())
    }

    // Get user by ID
    pub async fn get_user(&self, user_id: Uuid) -> Result<User> {
        self.users.get(&user_id)
            .map(|user| user.clone())
            .ok_or_else(|| anyhow!("User not found"))
    }

    // Helper methods
    fn generate_totp_secret(&self) -> Vec<u8> {
        let mut secret = vec![0u8; 20];
        rand::thread_rng().fill(&mut secret[..]);
        secret
    }

    fn generate_backup_codes(&self) -> Vec<String> {
        (0..8)
            .map(|_| {
                let code: u32 = rand::thread_rng().gen_range(100000..999999);
                format!("{:06}", code)
            })
            .collect()
    }

}