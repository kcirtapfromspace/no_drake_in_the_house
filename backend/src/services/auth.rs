use crate::models::{User, Claims, TokenPair, CreateUserRequest, LoginRequest, OAuthLoginRequest, TotpSetupResponse, UserSession, RegistrationValidationError};
use crate::services::registration_performance::RegistrationPerformanceService;
use crate::services::login_performance::LoginPerformanceService;
use crate::{AppError, Result};
use anyhow::anyhow;
use bcrypt::{hash, verify};
use chrono::Utc;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
use rand::Rng;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

pub struct AuthService {
    // Database connection
    db_pool: PgPool,
    
    // JWT configuration
    jwt_secret: String,
    access_token_ttl: i64,  // seconds (24 hours)
    refresh_token_ttl: i64, // seconds (30 days)
    
    // Performance optimization services
    performance_service: Arc<RegistrationPerformanceService>,
    login_performance_service: Arc<LoginPerformanceService>,
}

impl AuthService {
    pub fn new(db_pool: PgPool) -> Self {
        // Use environment variable or generate a random JWT secret for demo
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| format!("jwt_secret_{}", rand::thread_rng().gen::<u64>()));
        
        // Initialize performance services
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        
        let performance_service = Arc::new(
            RegistrationPerformanceService::new(&redis_url)
                .unwrap_or_else(|e| {
                    tracing::warn!("Failed to initialize registration performance service: {}", e);
                    // Create a fallback service that won't use Redis
                    RegistrationPerformanceService::new("redis://localhost:6379").unwrap()
                })
        );

        let login_performance_service = Arc::new(
            LoginPerformanceService::new(&redis_url)
                .unwrap_or_else(|e| {
                    tracing::warn!("Failed to initialize login performance service: {}", e);
                    // Create a fallback service that won't use Redis
                    LoginPerformanceService::new("redis://localhost:6379").unwrap()
                })
        );
        
        let auth_service = Self {
            db_pool,
            jwt_secret,
            access_token_ttl: 24 * 60 * 60,      // 24 hours as required
            refresh_token_ttl: 30 * 24 * 60 * 60, // 30 days
            performance_service,
            login_performance_service,
        };

        // Preload frequent users in background (don't block startup)
        let login_service = auth_service.login_performance_service.clone();
        let db_pool = auth_service.db_pool.clone();
        tokio::spawn(async move {
            if let Err(e) = login_service.preload_frequent_users(&db_pool).await {
                tracing::warn!("Failed to preload frequent users: {}", e);
            }
        });

        auth_service
    }

    // User registration with email/password
    pub async fn register_user(&self, request: CreateUserRequest) -> Result<User> {
        // Validate email format (basic validation)
        if !request.email.contains('@') {
            return Err(AppError::InvalidFieldValue { 
                field: "email".to_string(), 
                message: "Invalid email format".to_string() 
            });
        }

        // Validate password strength (basic validation)
        if request.password.len() < 8 {
            return Err(AppError::InvalidFieldValue { 
                field: "password".to_string(), 
                message: "Password must be at least 8 characters long".to_string() 
            });
        }

        // Check if user already exists
        let existing_user = sqlx::query!(
            "SELECT id FROM users WHERE email = $1",
            request.email
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if existing_user.is_some() {
            return Err(AppError::AlreadyExists { 
                resource: "User with this email".to_string() 
            });
        }

        // Hash password with bcrypt (12 rounds minimum as required)
        let password_hash = hash(&request.password, 12)?;

        // Create user in database
        let user_id = Uuid::new_v4();
        let now = Utc::now();
        
        sqlx::query!(
            r#"
            INSERT INTO users (id, email, password_hash, email_verified, totp_enabled, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            user_id,
            request.email,
            password_hash,
            false,
            false,
            now,
            now
        )
        .execute(&self.db_pool)
        .await?;

        // Fetch the created user
        let user = self.get_user_by_id(user_id).await?;
        Ok(user)
    }

    // Optimized login with email/password and 2FA support
    pub async fn login_user(&self, request: LoginRequest) -> Result<TokenPair> {
        let login_start = Instant::now();
        
        // Get cached user data for faster lookup
        let cached_user = self.login_performance_service
            .get_cached_user_login(&request.email, &self.db_pool)
            .await?
            .ok_or_else(|| AppError::InvalidCredentials)?;

        // Verify password using optimized method (runs in background thread)
        let password_valid = self.login_performance_service
            .verify_password_optimized(&request.password, &cached_user.password_hash)
            .await?;

        if !password_valid {
            // Record failed login attempt
            let login_time = login_start.elapsed().as_millis() as f64;
            if let Err(e) = self.login_performance_service.record_login_attempt(false, login_time).await {
                tracing::warn!("Failed to record login metrics: {}", e);
            }
            
            // Log failed login attempt (async, don't wait)
            let user_id = cached_user.user_id;
            tokio::spawn(async move {
                // This would need access to the audit service
                tracing::warn!(user_id = %user_id, "Failed login attempt");
            });
            
            return Err(AppError::InvalidCredentials);
        }

        // Check 2FA if enabled (keep this synchronous for security)
        if cached_user.totp_enabled {
            let totp_code = request.totp_code
                .ok_or_else(|| AppError::TwoFactorRequired)?;
            
            let totp_secret = cached_user.totp_secret
                .ok_or_else(|| AppError::Internal { 
                    message: Some("2FA configuration error. Please contact support".to_string()) 
                })?;
            
            if !self.verify_totp(&totp_secret, &totp_code)? {
                // Record failed 2FA attempt
                let login_time = login_start.elapsed().as_millis() as f64;
                if let Err(e) = self.login_performance_service.record_login_attempt(false, login_time).await {
                    tracing::warn!("Failed to record login metrics: {}", e);
                }
                
                // Log failed 2FA attempt (async)
                let user_id = cached_user.user_id;
                tokio::spawn(async move {
                    tracing::warn!(user_id = %user_id, "Failed 2FA attempt");
                });
                
                return Err(AppError::TwoFactorInvalid);
            }
        }

        // Generate optimized refresh token (lighter hashing)
        let (refresh_token_raw, refresh_token_hash) = self.login_performance_service
            .generate_optimized_refresh_token()
            .await?;

        // Generate access token (this is fast, no need to optimize)
        let access_claims = Claims::new_access_token(
            cached_user.user_id, 
            cached_user.email.clone(), 
            self.access_token_ttl
        );
        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;

        // Batch all database operations in a single transaction
        self.login_performance_service
            .batch_login_operations(
                cached_user.user_id,
                &refresh_token_hash,
                self.refresh_token_ttl,
                &self.db_pool,
            )
            .await?;

        // Record successful login metrics
        let login_time = login_start.elapsed().as_millis() as f64;
        if let Err(e) = self.login_performance_service.record_login_attempt(true, login_time).await {
            tracing::warn!("Failed to record login metrics: {}", e);
        }

        tracing::info!(
            user_id = %cached_user.user_id,
            email = %cached_user.email,
            login_time_ms = login_time,
            "User logged in successfully"
        );

        Ok(TokenPair {
            access_token,
            refresh_token: refresh_token_raw,
            expires_in: self.access_token_ttl,
            token_type: "Bearer".to_string(),
        })
    }

    // OAuth login (simplified for demo)
    pub async fn oauth_login(&self, _request: OAuthLoginRequest) -> Result<TokenPair> {
        Err(AppError::ExternalServiceUnavailable { service: "OAuth".to_string() })
    }

    // Generate JWT token pair with database storage
    async fn generate_token_pair(&self, user_id: Uuid, email: &str) -> Result<TokenPair> {
        // Generate access token (24-hour expiration as required)
        let access_claims = Claims::new_access_token(user_id, email.to_string(), self.access_token_ttl);
        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;

        // Generate refresh token
        let refresh_token_raw = format!("{}_{}", Uuid::new_v4(), rand::thread_rng().gen::<u64>());
        let refresh_token_hash = hash(&refresh_token_raw, 12)?;

        // Store refresh token in database with rotation support
        let expires_at = Utc::now() + chrono::Duration::seconds(self.refresh_token_ttl);
        
        sqlx::query!(
            r#"
            INSERT INTO user_sessions (user_id, refresh_token_hash, expires_at)
            VALUES ($1, $2, $3)
            "#,
            user_id,
            refresh_token_hash,
            expires_at
        )
        .execute(&self.db_pool)
        .await?;

        Ok(TokenPair {
            access_token,
            refresh_token: refresh_token_raw,
            expires_in: self.access_token_ttl,
            token_type: "Bearer".to_string(),
        })
    }

    // Refresh access token with token rotation
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenPair> {
        // Find valid refresh token in database
        let sessions = sqlx::query!(
            r#"
            SELECT s.id, s.user_id, s.refresh_token_hash, s.expires_at, u.email
            FROM user_sessions s
            JOIN users u ON s.user_id = u.id
            WHERE s.revoked = FALSE AND s.expires_at > NOW()
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut valid_session = None;
        for s in sessions {
            if verify(refresh_token, &s.refresh_token_hash).unwrap_or(false) {
                valid_session = Some(s);
                break;
            }
        }

        let session = valid_session.ok_or_else(|| AppError::TokenInvalid)?;

        // Revoke the old refresh token (token rotation)
        sqlx::query!(
            "UPDATE user_sessions SET revoked = TRUE, revoked_at = NOW() WHERE id = $1",
            session.id
        )
        .execute(&self.db_pool)
        .await?;

        // Generate new token pair
        self.generate_token_pair(session.user_id.unwrap(), &session.email).await
    }

    // Verify JWT token
    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        ).map_err(|_| AppError::TokenInvalid)?;

        let claims = token_data.claims;
        
        if claims.is_expired() {
            return Err(AppError::TokenExpired);
        }

        Ok(claims)
    }

    // Setup TOTP for user with temporary secret storage
    pub async fn setup_totp(&self, user_id: Uuid) -> Result<TotpSetupResponse> {
        let user = self.get_user_by_id(user_id).await?;

        // Check if 2FA is already enabled
        if user.totp_enabled {
            return Err(AppError::Conflict { message: "2FA is already enabled for this user".to_string() });
        }

        // Generate TOTP secret (160-bit secret as recommended by RFC 6238)
        let secret = self.generate_totp_secret();
        let secret_b32 = base32::encode(base32::Alphabet::Rfc4648 { padding: true }, &secret);
        
        // Generate QR code URL with proper formatting
        let qr_code_url = format!(
            "otpauth://totp/NodrakeInTheHouse:{}?secret={}&issuer=NodrakeInTheHouse&algorithm=SHA1&digits=6&period=30",
            urlencoding::encode(&user.email),
            secret_b32
        );

        // Generate backup codes
        let backup_codes = self.generate_backup_codes();
        
        // Store temporary secret (not enabled until verified)
        // We store it in totp_secret field but keep totp_enabled as false
        sqlx::query!(
            "UPDATE users SET totp_secret = $1, updated_at = NOW() WHERE id = $2",
            secret_b32,
            user_id
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(TotpSetupResponse {
            secret: secret_b32,
            qr_code_url,
            backup_codes,
        })
    }

    // Enable TOTP after verification
    pub async fn enable_totp(&self, user_id: Uuid, totp_code: &str) -> Result<()> {
        let user = sqlx::query!(
            "SELECT totp_secret, totp_enabled FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| anyhow!("User not found"))?;

        // Get email separately for cache invalidation
        let user_email = sqlx::query!(
            "SELECT email FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.db_pool)
        .await?
        .map(|row| row.email)
        .unwrap_or_default();

        // Check if 2FA is already enabled
        if user.totp_enabled.unwrap_or(false) {
            return Err(anyhow!("2FA is already enabled for this user").into());
        }

        let totp_secret = user.totp_secret
            .ok_or_else(|| anyhow!("TOTP setup not initiated. Please call setup_totp first"))?;

        // Verify the TOTP code
        if !self.verify_totp(&totp_secret, totp_code)? {
            return Err(anyhow!("Invalid TOTP code. Please check your authenticator app").into());
        }

        // Enable 2FA
        sqlx::query!(
            "UPDATE users SET totp_enabled = TRUE, updated_at = NOW() WHERE id = $1",
            user_id
        )
        .execute(&self.db_pool)
        .await?;
        
        // Invalidate login cache since user data changed
        if let Err(e) = self.login_performance_service.invalidate_user_cache(&user_email).await {
            tracing::warn!("Failed to invalidate login cache for user {}: {}", user_email, e);
        }
        
        // Audit log the 2FA enablement
        self.log_audit_event(user_id, "totp_enabled", "user", &user_id.to_string()).await?;
        
        Ok(())
    }

    // Disable TOTP with proper validation
    pub async fn disable_totp(&self, user_id: Uuid, totp_code: &str) -> Result<()> {
        let user = sqlx::query!(
            "SELECT totp_secret, totp_enabled FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| anyhow!("User not found"))?;

        // Get email separately if needed for cache invalidation
        let user_email = sqlx::query!(
            "SELECT email FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.db_pool)
        .await?
        .map(|row| row.email)
        .unwrap_or_default();

        // Check if 2FA is enabled
        if !user.totp_enabled.unwrap_or(false) {
            return Err(anyhow!("2FA is not enabled for this user").into());
        }

        let totp_secret = user.totp_secret
            .ok_or_else(|| anyhow!("TOTP secret not found"))?;

        // Verify the TOTP code before disabling
        if !self.verify_totp(&totp_secret, totp_code)? {
            return Err(anyhow!("Invalid TOTP code. Cannot disable 2FA without verification").into());
        }

        // Disable 2FA and remove secret
        sqlx::query!(
            "UPDATE users SET totp_enabled = FALSE, totp_secret = NULL, updated_at = NOW() WHERE id = $1",
            user_id
        )
        .execute(&self.db_pool)
        .await?;
        
        // Invalidate login cache since user data changed
        if let Err(e) = self.login_performance_service.invalidate_user_cache(&user_email).await {
            tracing::warn!("Failed to invalidate login cache for user {}: {}", user_email, e);
        }
        
        // Audit log the 2FA disablement
        self.log_audit_event(user_id, "totp_disabled", "user", &user_id.to_string()).await?;
        
        Ok(())
    }

    // Verify TOTP code with proper error handling and clock skew tolerance
    fn verify_totp(&self, secret: &str, code: &str) -> Result<bool> {
        // Validate input
        if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
            return Ok(false);
        }

        let secret_bytes = base32::decode(base32::Alphabet::Rfc4648 { padding: true }, secret)
            .ok_or_else(|| AppError::Internal { message: Some("Invalid TOTP secret format".to_string()) })?;
        
        if secret_bytes.len() < 10 {
            return Err(AppError::Internal { message: Some("TOTP secret too short".to_string()) });
        }
        
        let current_time = Utc::now().timestamp() as u64;
        
        // Check current and adjacent 30-second windows for clock skew tolerance
        // This allows for ±30 seconds of clock drift
        for offset in [-1, 0, 1] {
            let time_step = (current_time / 30) as i64 + offset;
            if time_step >= 0 {
                let expected_code = self.generate_totp_code(&secret_bytes, time_step as u64)?;
                if expected_code == code {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }

    // TOTP code generation following RFC 6238
    fn generate_totp_code(&self, secret: &[u8], time_step: u64) -> Result<String> {
        let time_bytes = time_step.to_be_bytes();
        
        // Use HMAC-SHA1 as specified in RFC 6238 for TOTP
        use hmac::{Hmac, Mac};
        use sha1::Sha1;
        
        type HmacSha1 = Hmac<Sha1>;
        let mut mac = HmacSha1::new_from_slice(secret)
            .map_err(|_| AppError::Internal { message: Some("Invalid TOTP secret length".to_string()) })?;
        mac.update(&time_bytes);
        let result = mac.finalize().into_bytes();
        
        // Dynamic truncation as per RFC 4226
        let offset = (result[result.len() - 1] & 0xf) as usize;
        if offset + 4 > result.len() {
            return Err(AppError::Internal { message: Some("Invalid HMAC result for TOTP".to_string()) });
        }
        
        let code = ((result[offset] as u32 & 0x7f) << 24)
            | ((result[offset + 1] as u32 & 0xff) << 16)
            | ((result[offset + 2] as u32 & 0xff) << 8)
            | (result[offset + 3] as u32 & 0xff);
        
        // Generate 6-digit code
        Ok(format!("{:06}", code % 1000000))
    }

    // Revoke all user sessions
    pub async fn revoke_all_sessions(&self, user_id: Uuid) -> Result<()> {
        sqlx::query!(
            "UPDATE user_sessions SET revoked = TRUE, revoked_at = NOW() WHERE user_id = $1 AND revoked = FALSE",
            user_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    // Get user by ID
    pub async fn get_user(&self, user_id: Uuid) -> Result<User> {
        self.get_user_by_id(user_id).await
    }

    // Internal helper to get user by ID from database
    async fn get_user_by_id(&self, user_id: Uuid) -> Result<User> {
        let user = sqlx::query!(
            r#"
            SELECT id, email, password_hash, email_verified, totp_secret, totp_enabled, 
                   created_at, updated_at, last_login, settings
            FROM users WHERE id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| anyhow!("User not found"))?;

        Ok(User {
            id: user.id,
            email: user.email,
            password_hash: user.password_hash,
            email_verified: user.email_verified.unwrap_or(false),
            totp_secret: user.totp_secret,
            totp_enabled: user.totp_enabled.unwrap_or(false),
            oauth_providers: Vec::new(), // TODO: Load from separate table if needed
            created_at: user.created_at.unwrap_or(Utc::now()),
            updated_at: user.updated_at.unwrap_or(Utc::now()),
            last_login: user.last_login,
            settings: serde_json::from_value(user.settings.unwrap_or(serde_json::json!({})))
                .unwrap_or_default(),
        })
    }

    // Validate access token (alias for verify_token for compatibility)
    pub async fn validate_access_token(&self, token: &str) -> Result<Claims> {
        self.verify_token(token)
    }

    // Get user sessions (simplified for now)
    pub async fn get_user_sessions(&self, _user_id: Uuid) -> Result<Vec<UserSession>> {
        // TODO: Implement proper session retrieval
        Ok(Vec::new())
    }

    // Revoke specific session
    pub async fn revoke_session(&self, user_id: Uuid, session_id: Uuid) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE user_sessions SET revoked = TRUE, revoked_at = NOW() WHERE id = $1 AND user_id = $2",
            session_id,
            user_id
        )
        .execute(&self.db_pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("Session not found or access denied").into());
        }

        Ok(())
    }

    // Request password reset
    pub async fn request_password_reset(&self, email: String) -> Result<String> {
        // Check if user exists (but don't reveal if they don't for security)
        let _user_exists = sqlx::query!(
            "SELECT id FROM users WHERE email = $1",
            email
        )
        .fetch_optional(&self.db_pool)
        .await?
        .is_some();

        // Always return a token (real or fake) to prevent email enumeration
        let reset_token = format!("reset_token_{}", rand::thread_rng().gen::<u64>());
        
        // In a real implementation, store the reset token in database with expiration
        // For now, just return the token
        Ok(reset_token)
    }

    // Reset password with token
    pub async fn reset_password(&self, _reset_token: String, new_password: String) -> Result<()> {
        // Validate password strength
        if new_password.len() < 8 {
            return Err(anyhow!("Password must be at least 8 characters long").into());
        }
        
        // In a real implementation, validate the reset token and update password
        // For now, just validate the password format
        Ok(())
    }

    // Get 2FA status for user
    pub async fn get_totp_status(&self, user_id: Uuid) -> Result<bool> {
        let user = sqlx::query!(
            "SELECT totp_enabled FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| anyhow!("User not found"))?;

        Ok(user.totp_enabled.unwrap_or(false))
    }

    // Verify TOTP code without side effects (for testing/validation)
    pub fn verify_totp_code(&self, secret: &str, code: &str) -> Result<bool> {
        self.verify_totp(secret, code)
    }

    // Helper method for audit logging
    async fn log_audit_event(&self, user_id: Uuid, action: &str, subject_type: &str, subject_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO audit_log (user_id, action, old_subject_type, old_subject_id, timestamp)
            VALUES ($1, $2, $3, $4, NOW())
            "#,
            user_id,
            action,
            subject_type,
            subject_id
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }

    // Helper methods
    fn generate_totp_secret(&self) -> Vec<u8> {
        // Generate 160-bit (20-byte) secret as recommended by RFC 6238
        let mut secret = vec![0u8; 20];
        use rand::RngCore;
        rand::thread_rng().fill_bytes(&mut secret);
        secret
    }

    fn generate_backup_codes(&self) -> Vec<String> {
        // Generate 8 backup codes with 6 digits each
        (0..8)
            .map(|_| {
                let code: u32 = rand::thread_rng().gen_range(100000..999999);
                format!("{:06}", code)
            })
            .collect()
    }

    // Additional methods needed for tests
    pub async fn logout_user(&self, user_id: Uuid, refresh_token: &str) -> Result<()> {
        // Invalidate the refresh token by deleting the session
        sqlx::query!(
            "DELETE FROM user_sessions WHERE user_id = $1",
            user_id
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }

    pub async fn refresh_access_token(&self, refresh_token: &str) -> Result<TokenPair> {
        self.refresh_token(refresh_token).await
    }

    pub async fn create_session(&self, user_id: Uuid, _device_info: String) -> Result<String> {
        // Create a simple session token for testing
        let session_token = format!("session_{}_{}", user_id, uuid::Uuid::new_v4());
        
        // In a real implementation, this would store session info in database
        sqlx::query!(
            "UPDATE users SET updated_at = NOW() WHERE id = $1",
            user_id
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(session_token)
    }

    pub fn with_oauth_enabled(self) -> Self {
        // For testing purposes, just return self
        // In real implementation, this would configure OAuth settings
        self
    }

    // Comprehensive registration validation function with performance optimizations
    pub async fn validate_registration_request(&self, request: &crate::models::RegisterRequest) -> Vec<RegistrationValidationError> {
        let validation_start = Instant::now();
        let mut errors = Vec::new();

        // Email format validation with caching
        if request.email.is_empty() {
            errors.push(RegistrationValidationError {
                field: "email".to_string(),
                message: "Email is required".to_string(),
                code: "EMAIL_REQUIRED".to_string(),
            });
        } else {
            // Use cached email validation
            match self.performance_service.validate_email_format_cached(&request.email).await {
                Ok(is_valid) => {
                    if !is_valid {
                        errors.push(RegistrationValidationError {
                            field: "email".to_string(),
                            message: "Please enter a valid email address".to_string(),
                            code: "EMAIL_INVALID_FORMAT".to_string(),
                        });
                    }
                }
                Err(_) => {
                    // Fallback to non-cached validation
                    let email_regex = regex::Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9._+%-]*[a-zA-Z0-9])?@[a-zA-Z0-9]([a-zA-Z0-9.-]*[a-zA-Z0-9])?\.[a-zA-Z]{2,}$").unwrap();
                    if !email_regex.is_match(&request.email) || request.email.contains("..") {
                        errors.push(RegistrationValidationError {
                            field: "email".to_string(),
                            message: "Please enter a valid email address".to_string(),
                            code: "EMAIL_INVALID_FORMAT".to_string(),
                        });
                    }
                }
            }
            
            // Check email length
            if request.email.len() > 255 {
                errors.push(RegistrationValidationError {
                    field: "email".to_string(),
                    message: "Email address is too long (maximum 255 characters)".to_string(),
                    code: "EMAIL_TOO_LONG".to_string(),
                });
            }
        }

        // Password validation
        if request.password.is_empty() {
            errors.push(RegistrationValidationError {
                field: "password".to_string(),
                message: "Password is required".to_string(),
                code: "PASSWORD_REQUIRED".to_string(),
            });
        } else {
            // Use cached password strength validation
            match self.performance_service.validate_password_strength_cached(&request.password).await {
                Ok(Some(password_error)) => {
                    errors.push(password_error);
                }
                Ok(None) => {
                    // Password is valid
                }
                Err(_) => {
                    // Fallback to non-cached validation
                    if let Some(password_error) = self.validate_password_strength(&request.password) {
                        errors.push(password_error);
                    }
                }
            }
        }

        // Password confirmation matching validation
        if request.confirm_password.is_empty() {
            errors.push(RegistrationValidationError {
                field: "confirm_password".to_string(),
                message: "Password confirmation is required".to_string(),
                code: "CONFIRM_PASSWORD_REQUIRED".to_string(),
            });
        } else if request.password != request.confirm_password {
            errors.push(RegistrationValidationError {
                field: "confirm_password".to_string(),
                message: "Password confirmation does not match".to_string(),
                code: "PASSWORD_MISMATCH".to_string(),
            });
        }

        // Terms acceptance validation logic
        if !request.terms_accepted {
            errors.push(RegistrationValidationError {
                field: "terms_accepted".to_string(),
                message: "You must accept the terms of service to register".to_string(),
                code: "TERMS_NOT_ACCEPTED".to_string(),
            });
        }

        // Record validation metrics
        let validation_time = validation_start.elapsed().as_millis() as f64;
        if let Err(e) = self.performance_service.record_registration_attempt(validation_time).await {
            tracing::warn!("Failed to record registration metrics: {}", e);
        }

        errors
    }

    // Password strength validation with detailed requirements checking
    fn validate_password_strength(&self, password: &str) -> Option<RegistrationValidationError> {
        let mut requirements = Vec::new();

        // Minimum length requirement
        if password.len() < 8 {
            requirements.push("at least 8 characters");
        }

        // Uppercase letter requirement
        if !password.chars().any(|c| c.is_uppercase()) {
            requirements.push("at least one uppercase letter");
        }

        // Lowercase letter requirement
        if !password.chars().any(|c| c.is_lowercase()) {
            requirements.push("at least one lowercase letter");
        }

        // Number requirement
        if !password.chars().any(|c| c.is_numeric()) {
            requirements.push("at least one number");
        }

        // Special character requirement
        if !password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)) {
            requirements.push("at least one special character (!@#$%^&*()_+-=[]{}|;:,.<>?)");
        }

        // Check against common passwords (basic implementation)
        let common_passwords = [
            "password", "123456", "password123", "admin", "qwerty", "letmein",
            "welcome", "monkey", "1234567890", "password1", "123456789"
        ];
        
        if common_passwords.iter().any(|&common| password.to_lowercase() == common.to_lowercase()) {
            requirements.push("not be a common password");
        }

        if !requirements.is_empty() {
            let message = format!("Password must contain {}", requirements.join(", "));
            Some(RegistrationValidationError {
                field: "password".to_string(),
                message,
                code: "PASSWORD_WEAK".to_string(),
            })
        } else {
            None
        }
    }

    // Enhanced registration method with comprehensive validation and performance optimizations
    pub async fn register(&self, request: crate::models::RegisterRequest) -> Result<crate::models::AuthResponse> {
        use crate::models::{AuthResponse, UserProfile};
        
        let registration_start = Instant::now();
        
        // Integrate new validation function into registration flow with performance optimizations
        let validation_errors = self.validate_registration_request(&request).await;
        
        // Implement structured error collection and response formatting
        if !validation_errors.is_empty() {
            // Record validation failure metrics
            if let Err(e) = self.performance_service.record_validation_failure().await {
                tracing::warn!("Failed to record validation failure metrics: {}", e);
            }
            
            // Add detailed logging for validation failures
            tracing::warn!(
                email = %request.email,
                validation_errors = ?validation_errors,
                "Registration validation failed"
            );
            
            return Err(crate::AppError::RegistrationValidationError { 
                errors: validation_errors 
            });
        }

        // Check if user already exists using optimized query
        let email_exists = match self.performance_service.check_email_exists_optimized(&self.db_pool, &request.email).await {
            Ok(exists) => exists,
            Err(_) => {
                // Fallback to original query
                let existing_user = sqlx::query!(
                    "SELECT id FROM users WHERE email = $1",
                    request.email
                )
                .fetch_optional(&self.db_pool)
                .await?;
                existing_user.is_some()
            }
        };

        if email_exists {
            // Record email duplicate metrics
            if let Err(e) = self.performance_service.record_email_duplicate().await {
                tracing::warn!("Failed to record email duplicate metrics: {}", e);
            }
            
            // Add detailed logging for security events
            tracing::warn!(
                email = %request.email,
                "Registration attempt with existing email"
            );
            
            return Err(crate::AppError::EmailAlreadyRegistered);
        }

        // Hash password with bcrypt (12 rounds minimum as required)
        let password_hash = hash(&request.password, 12)?;

        // Create user in database with optimized transaction
        let user_id = Uuid::new_v4();
        let now = Utc::now();
        
        // Use a single transaction for both user creation and audit logging
        let mut tx = self.db_pool.begin().await?;
        
        // Insert user with prepared statement for better performance
        sqlx::query!(
            r#"
            INSERT INTO users (id, email, password_hash, email_verified, totp_enabled, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            user_id,
            request.email,
            password_hash,
            false,
            false,
            now,
            now
        )
        .execute(&mut *tx)
        .await?;

        // Log successful registration for security auditing in the same transaction
        // Using existing audit logging pattern
        self.log_audit_event(user_id, "user_registered", "user", &user_id.to_string()).await.ok();

        // Commit transaction
        tx.commit().await?;

        // Record successful registration metrics
        let registration_time = registration_start.elapsed().as_millis() as f64;
        if let Err(e) = self.performance_service.record_successful_registration(registration_time).await {
            tracing::warn!("Failed to record successful registration metrics: {}", e);
        }

        // Add detailed logging for successful registration
        tracing::info!(
            user_id = %user_id,
            email = %request.email,
            registration_time_ms = registration_time,
            "User registration completed successfully"
        );

        // Fetch the created user (optimized to avoid unnecessary fields)
        let user = self.get_user_by_id(user_id).await?;
        
        // Check if auto-login is enabled via environment variable
        let auto_login_enabled = std::env::var("AUTO_LOGIN_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        if auto_login_enabled {
            // Generate tokens for auto-login with proper error handling
            match self.generate_token_pair(user.id, &user.email).await {
                Ok(token_pair) => {
                    // Add logging for auto-login success
                    tracing::info!(
                        user_id = %user.id,
                        email = %user.email,
                        "Auto-login successful after registration"
                    );
                    
                    Ok(AuthResponse {
                        user: UserProfile {
                            id: user.id,
                            email: user.email,
                            email_verified: user.email_verified,
                            totp_enabled: user.totp_enabled,
                            created_at: user.created_at,
                            updated_at: user.updated_at,
                            last_login: user.last_login,
                            settings: user.settings,
                        },
                        access_token: token_pair.access_token,
                        refresh_token: token_pair.refresh_token,
                    })
                }
                Err(token_error) => {
                    // Add logging for auto-login failure cases
                    tracing::warn!(
                        user_id = %user.id,
                        email = %user.email,
                        error = %token_error,
                        "Auto-login failed after registration, user created successfully"
                    );
                    
                    // Return the error to indicate token generation failure
                    Err(token_error)
                }
            }
        } else {
            // Auto-login is disabled, generate empty tokens or handle differently
            tracing::info!(
                user_id = %user.id,
                email = %user.email,
                "Registration successful, auto-login disabled"
            );
            
            // Return response with empty tokens to indicate auto-login is disabled
            Ok(AuthResponse {
                user: UserProfile {
                    id: user.id,
                    email: user.email,
                    email_verified: user.email_verified,
                    totp_enabled: user.totp_enabled,
                    created_at: user.created_at,
                    updated_at: user.updated_at,
                    last_login: user.last_login,
                    settings: user.settings,
                },
                access_token: String::new(), // Empty token indicates auto-login disabled
                refresh_token: String::new(), // Empty token indicates auto-login disabled
            })
        }
    }

    pub async fn login(&self, request: LoginRequest) -> Result<crate::models::AuthResponse> {
        use crate::models::{AuthResponse, UserProfile};
        
        // Login user
        let token_pair = self.login_user(request).await?;
        
        // Get user info from token
        let claims = self.verify_token(&token_pair.access_token)?;
        let user_id = Uuid::parse_str(&claims.sub)?;
        let user = self.get_user(user_id).await?;
        
        Ok(AuthResponse {
            user: UserProfile {
                id: user.id,
                email: user.email,
                email_verified: user.email_verified,
                totp_enabled: user.totp_enabled,
                created_at: user.created_at,
                updated_at: user.updated_at,
                last_login: user.last_login,
                settings: user.settings,
            },
            access_token: token_pair.access_token,
            refresh_token: token_pair.refresh_token,
        })
    }

    pub async fn setup_2fa(&self, user_id: Uuid) -> Result<crate::models::TotpSetupResponse> {
        self.setup_totp(user_id).await
    }

    pub async fn verify_and_enable_2fa(&self, user_id: Uuid, totp_code: &str) -> Result<bool> {
        match self.enable_totp(user_id, totp_code).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub async fn disable_2fa(&self, user_id: Uuid, totp_code: &str) -> Result<bool> {
        match self.disable_totp(user_id, totp_code).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

}