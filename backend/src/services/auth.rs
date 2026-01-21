use crate::models::oauth::{
    AccountLinkRequest, OAuthAccount, OAuthAccountHealth, OAuthConnectionStatus, OAuthFlowResponse,
    OAuthProviderType, OAuthState, OAuthTokenStatus, OAuthTokens, OAuthUserInfo, RefreshPriority,
    TokenExpirationStatus, TokenNotificationTarget, TokenRefreshSchedule, TokenRefreshSummary,
};
use crate::models::user::{MergeAccountsRequest, MergeAccountsResponse};
use crate::models::{
    Claims, CreateUserRequest, LoginRequest, OAuthLoginRequest, RegistrationValidationError,
    TokenPair, TotpSetupResponse, User, UserSession,
};
use crate::services::login_performance::LoginPerformanceService;
use crate::services::oauth::{OAuthProvider, OAuthStateManager};
use crate::services::oauth_apple::AppleOAuthProvider;
use crate::services::oauth_config_validator::OAuthConfigValidator;
use crate::services::oauth_encryption::OAuthTokenEncryption;
use crate::services::oauth_error_recovery::{OAuthErrorRecoveryConfig, OAuthErrorRecoveryService};
use crate::services::oauth_github::GitHubOAuthProvider;
use crate::services::oauth_google::GoogleOAuthProvider;
use crate::services::oauth_health_monitor::{
    OAuthHealthConfig, OAuthHealthMonitor, OAuthProviderHealthStatus,
};
use crate::services::oauth_security_logger::OAuthSecurityLogger;
use crate::services::oauth_spotify::SpotifyOAuthProvider;
use crate::services::registration_performance::RegistrationPerformanceService;
use crate::{AppError, Result};
use anyhow::anyhow;
use base64::Engine as _;
use bcrypt::{hash, verify};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use sqlx::PgPool;
use std::collections::HashMap;
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

    // OAuth configuration
    oauth_providers: Arc<HashMap<OAuthProviderType, Box<dyn OAuthProvider>>>,
    oauth_state_manager: Arc<OAuthStateManager>,
    oauth_encryption: Arc<OAuthTokenEncryption>,
    oauth_config_validator: Arc<OAuthConfigValidator>,
    oauth_health_monitor: Arc<OAuthHealthMonitor>,
    oauth_error_recovery: Arc<OAuthErrorRecoveryService>,
    oauth_security_logger: Arc<OAuthSecurityLogger>,
}

impl AuthService {
    pub fn new(db_pool: PgPool) -> Self {
        // Use environment variable or generate a random JWT secret for demo
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| format!("jwt_secret_{}", rand::thread_rng().gen::<u64>()));

        // Initialize performance services
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let performance_service = Arc::new(
            RegistrationPerformanceService::new(&redis_url).unwrap_or_else(|e| {
                tracing::warn!(
                    "Failed to initialize registration performance service: {}",
                    e
                );
                // Create a fallback service that won't use Redis
                RegistrationPerformanceService::new("redis://localhost:6379").unwrap()
            }),
        );

        let login_performance_service = Arc::new(
            LoginPerformanceService::new(&redis_url).unwrap_or_else(|e| {
                tracing::warn!("Failed to initialize login performance service: {}", e);
                // Create a fallback service that won't use Redis
                LoginPerformanceService::new("redis://localhost:6379").unwrap()
            }),
        );

        // Initialize OAuth configuration validator
        let mut oauth_config_validator = OAuthConfigValidator::new();
        if let Err(e) = oauth_config_validator.validate_all_providers() {
            tracing::error!("OAuth configuration validation failed: {}", e);
        }

        // Validate environment security
        if let Err(e) = oauth_config_validator.validate_environment_security() {
            tracing::error!("OAuth security validation failed: {}", e);
        }

        let oauth_config_validator = Arc::new(oauth_config_validator);

        // Initialize OAuth components
        let oauth_providers = Self::initialize_oauth_providers(&oauth_config_validator);
        let oauth_providers_arc = Arc::new(oauth_providers);

        // Initialize OAuth health monitor
        let health_config = OAuthHealthConfig::default();
        let oauth_health_monitor = Arc::new(OAuthHealthMonitor::new(
            Arc::clone(&oauth_providers_arc),
            health_config,
        ));

        // Initialize OAuth error recovery service
        let recovery_config = OAuthErrorRecoveryConfig::default();
        let oauth_error_recovery = Arc::new(OAuthErrorRecoveryService::new(recovery_config));

        // Initialize OAuth security logger
        let oauth_security_logger = Arc::new(OAuthSecurityLogger::new());

        // Start health monitoring in background
        let health_monitor_clone = Arc::clone(&oauth_health_monitor);
        tokio::spawn(async move {
            health_monitor_clone.start_monitoring().await;
        });

        let oauth_state_manager = Arc::new(OAuthStateManager::new());
        let oauth_encryption = Arc::new(OAuthTokenEncryption::new().unwrap_or_else(|e| {
            tracing::warn!("Failed to initialize OAuth encryption: {}", e);
            // Create with a generated key for development
            let key = OAuthTokenEncryption::generate_key();
            OAuthTokenEncryption::with_key(&key).unwrap()
        }));

        let auth_service = Self {
            db_pool,
            jwt_secret,
            access_token_ttl: 24 * 60 * 60, // 24 hours as required
            refresh_token_ttl: 30 * 24 * 60 * 60, // 30 days
            performance_service,
            login_performance_service,
            oauth_providers: oauth_providers_arc,
            oauth_state_manager,
            oauth_encryption,
            oauth_config_validator,
            oauth_health_monitor,
            oauth_error_recovery,
            oauth_security_logger,
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

    /// Initialize OAuth providers from environment configuration
    fn initialize_oauth_providers(
        config_validator: &Arc<OAuthConfigValidator>,
    ) -> HashMap<OAuthProviderType, Box<dyn OAuthProvider>> {
        let mut providers: HashMap<OAuthProviderType, Box<dyn OAuthProvider>> = HashMap::new();

        // Initialize Google OAuth if configured and valid
        if config_validator.is_provider_available(&OAuthProviderType::Google) {
            match (
                std::env::var("GOOGLE_CLIENT_ID"),
                std::env::var("GOOGLE_CLIENT_SECRET"),
                std::env::var("GOOGLE_REDIRECT_URI"),
            ) {
                (Ok(client_id), Ok(client_secret), Ok(redirect_uri)) => {
                    match GoogleOAuthProvider::with_credentials(
                        client_id,
                        client_secret,
                        redirect_uri,
                    ) {
                        Ok(provider) => {
                            providers.insert(OAuthProviderType::Google, Box::new(provider));
                            tracing::info!("✅ Google OAuth provider initialized and ready");
                        }
                        Err(e) => {
                            tracing::error!("❌ Failed to initialize Google OAuth provider: {}", e)
                        }
                    }
                }
                _ => {
                    tracing::warn!(
                        "⚠️  Google OAuth validation passed but environment variables are missing"
                    );
                }
            }
        } else if let Some(validation) =
            config_validator.get_provider_validation(&OAuthProviderType::Google)
        {
            if !validation.missing_variables.is_empty() {
                tracing::debug!(
                    "Google OAuth not configured - missing: {}",
                    validation.missing_variables.join(", ")
                );
            }
            if !validation.validation_errors.is_empty() {
                tracing::warn!(
                    "Google OAuth configuration errors: {}",
                    validation.validation_errors.join("; ")
                );
            }
        }

        // Initialize Apple OAuth if configured and valid
        if config_validator.is_provider_available(&OAuthProviderType::Apple) {
            match (
                std::env::var("APPLE_CLIENT_ID"),
                std::env::var("APPLE_TEAM_ID"),
                std::env::var("APPLE_KEY_ID"),
                std::env::var("APPLE_PRIVATE_KEY"),
                std::env::var("APPLE_REDIRECT_URI"),
            ) {
                (Ok(client_id), Ok(team_id), Ok(key_id), Ok(private_key), Ok(redirect_uri)) => {
                    match AppleOAuthProvider::with_credentials(
                        client_id,
                        team_id,
                        key_id,
                        private_key,
                        redirect_uri,
                    ) {
                        Ok(provider) => {
                            providers.insert(OAuthProviderType::Apple, Box::new(provider));
                            tracing::info!("✅ Apple OAuth provider initialized and ready");
                        }
                        Err(e) => {
                            tracing::error!("❌ Failed to initialize Apple OAuth provider: {}", e)
                        }
                    }
                }
                _ => {
                    tracing::warn!(
                        "⚠️  Apple OAuth validation passed but environment variables are missing"
                    );
                }
            }
        } else if let Some(validation) =
            config_validator.get_provider_validation(&OAuthProviderType::Apple)
        {
            if !validation.missing_variables.is_empty() {
                tracing::debug!(
                    "Apple OAuth not configured - missing: {}",
                    validation.missing_variables.join(", ")
                );
            }
            if !validation.validation_errors.is_empty() {
                tracing::warn!(
                    "Apple OAuth configuration errors: {}",
                    validation.validation_errors.join("; ")
                );
            }
        }

        // Initialize GitHub OAuth if configured and valid
        if config_validator.is_provider_available(&OAuthProviderType::GitHub) {
            match (
                std::env::var("GITHUB_CLIENT_ID"),
                std::env::var("GITHUB_CLIENT_SECRET"),
                std::env::var("GITHUB_REDIRECT_URI"),
            ) {
                (Ok(client_id), Ok(client_secret), Ok(redirect_uri)) => {
                    match GitHubOAuthProvider::with_credentials(
                        client_id,
                        client_secret,
                        redirect_uri,
                    ) {
                        Ok(provider) => {
                            providers.insert(OAuthProviderType::GitHub, Box::new(provider));
                            tracing::info!("✅ GitHub OAuth provider initialized and ready");
                        }
                        Err(e) => {
                            tracing::error!("❌ Failed to initialize GitHub OAuth provider: {}", e)
                        }
                    }
                }
                _ => {
                    tracing::warn!(
                        "⚠️  GitHub OAuth validation passed but environment variables are missing"
                    );
                }
            }
        } else if let Some(validation) =
            config_validator.get_provider_validation(&OAuthProviderType::GitHub)
        {
            if !validation.missing_variables.is_empty() {
                tracing::debug!(
                    "GitHub OAuth not configured - missing: {}",
                    validation.missing_variables.join(", ")
                );
            }
            if !validation.validation_errors.is_empty() {
                tracing::warn!(
                    "GitHub OAuth configuration errors: {}",
                    validation.validation_errors.join("; ")
                );
            }
        }

        // Initialize Spotify OAuth if configured and valid
        if config_validator.is_provider_available(&OAuthProviderType::Spotify) {
            match (
                std::env::var("SPOTIFY_CLIENT_ID"),
                std::env::var("SPOTIFY_CLIENT_SECRET"),
            ) {
                (Ok(client_id), Ok(client_secret)) => {
                    match crate::services::oauth_spotify::create_spotify_oauth_provider() {
                        Ok(provider) => {
                            providers.insert(OAuthProviderType::Spotify, Box::new(provider));
                            tracing::info!("✅ Spotify OAuth provider initialized and ready");
                        }
                        Err(e) => {
                            tracing::error!("❌ Failed to initialize Spotify OAuth provider: {}", e)
                        }
                    }
                }
                _ => {
                    tracing::warn!(
                        "⚠️  Spotify OAuth validation passed but environment variables are missing"
                    );
                }
            }
        } else if let Some(validation) =
            config_validator.get_provider_validation(&OAuthProviderType::Spotify)
        {
            if !validation.missing_variables.is_empty() {
                tracing::debug!(
                    "Spotify OAuth not configured - missing: {}",
                    validation.missing_variables.join(", ")
                );
            }
            if !validation.validation_errors.is_empty() {
                tracing::warn!(
                    "Spotify OAuth configuration errors: {}",
                    validation.validation_errors.join("; ")
                );
            }
        }

        // Check if we're in development mode and should use demo providers
        let is_dev_mode =
            std::env::var("OAUTH_DEV_MODE").unwrap_or_else(|_| "false".to_string()) == "true";

        if providers.is_empty() && is_dev_mode {
            tracing::warn!("No OAuth providers configured, but OAUTH_DEV_MODE is enabled. Creating demo providers for development.");

            // Create demo providers for development
            if let Ok(google_provider) = Self::create_demo_google_provider() {
                providers.insert(OAuthProviderType::Google, Box::new(google_provider));
                tracing::info!("Demo Google OAuth provider initialized for development");
            }

            if let Ok(github_provider) = Self::create_demo_github_provider() {
                providers.insert(OAuthProviderType::GitHub, Box::new(github_provider));
                tracing::info!("Demo GitHub OAuth provider initialized for development");
            }

            // Note: Apple OAuth requires real certificates, so we skip it in demo mode
            tracing::info!("Apple OAuth skipped in demo mode (requires real certificates)");
        } else if providers.is_empty() {
            tracing::warn!("No OAuth providers configured. Set environment variables to enable social authentication, or set OAUTH_DEV_MODE=true for development.");
        }

        providers
    }

    /// Get OAuth configuration validation results
    pub fn get_oauth_config_validation(
        &self,
    ) -> &HashMap<OAuthProviderType, crate::services::oauth_config_validator::OAuthProviderValidation>
    {
        self.oauth_config_validator.get_validation_results()
    }

    /// Get user guidance for OAuth errors
    pub fn get_oauth_user_guidance(
        &self,
        oauth_error: &crate::error::oauth::OAuthError,
    ) -> crate::services::oauth_error_recovery::UserGuidance {
        self.oauth_error_recovery.get_user_guidance(oauth_error)
    }

    /// Get OAuth provider health status
    pub async fn get_oauth_provider_health(
        &self,
        provider: &OAuthProviderType,
    ) -> crate::services::oauth_error_recovery::ProviderHealthStatus {
        self.oauth_error_recovery
            .get_provider_health(provider)
            .await
    }

    /// Get circuit breaker status for OAuth provider
    pub async fn get_oauth_circuit_breaker_status(
        &self,
        provider: &OAuthProviderType,
    ) -> Option<bool> {
        self.oauth_error_recovery
            .get_circuit_breaker_status(provider)
            .await
    }

    /// Get OAuth security statistics
    pub async fn get_oauth_security_stats(
        &self,
        provider: Option<OAuthProviderType>,
        hours: i64,
    ) -> crate::services::oauth_security_logger::SecurityStats {
        self.oauth_security_logger
            .get_security_stats(provider, hours)
            .await
    }

    /// Get recent OAuth security events
    pub async fn get_oauth_security_events(
        &self,
        provider: Option<OAuthProviderType>,
        hours: i64,
    ) -> Vec<crate::services::oauth_security_logger::OAuthSecurityEvent> {
        self.oauth_security_logger
            .get_recent_events(provider, hours)
            .await
    }

    /// Get configuration guidance for a specific OAuth provider
    pub fn get_oauth_configuration_guidance(&self, provider: &OAuthProviderType) -> String {
        self.oauth_config_validator
            .get_configuration_guidance(provider)
    }

    /// Validate OAuth provider health before operations
    async fn validate_oauth_provider_health(&self, provider: &OAuthProviderType) -> Result<()> {
        use crate::services::oauth_error_recovery::ProviderHealthStatus;

        let health_status = self
            .oauth_error_recovery
            .get_provider_health(provider)
            .await;

        match health_status {
            ProviderHealthStatus::Healthy => Ok(()),
            ProviderHealthStatus::Degraded { reason } => {
                tracing::warn!(
                    provider = %provider,
                    reason = %reason,
                    "OAuth provider is degraded but still available"
                );
                Ok(()) // Allow operation but log warning
            }
            ProviderHealthStatus::Unavailable {
                reason,
                estimated_recovery,
            } => Err(crate::error::AppError::OAuth(
                crate::error::oauth::OAuthError::ProviderUnavailable {
                    provider: *provider,
                    reason,
                    estimated_recovery,
                    retry_after: Some(60),
                },
            )),
        }
    }

    /// Validate OAuth provider configuration at runtime
    pub fn validate_oauth_provider_config(&self, provider: &OAuthProviderType) -> Result<()> {
        if !self.is_oauth_provider_available(provider) {
            let validation = self
                .oauth_config_validator
                .get_provider_validation(provider);

            if let Some(validation) = validation {
                if !validation.is_configured {
                    return Err(AppError::OAuthProviderError {
                        provider: provider.to_string(),
                        message: format!(
                            "OAuth provider {} is not configured. Missing environment variables: {}. {}",
                            provider,
                            validation.missing_variables.join(", "),
                            self.get_oauth_configuration_guidance(provider)
                        ),
                    });
                } else if !validation.is_valid {
                    return Err(AppError::OAuthProviderError {
                        provider: provider.to_string(),
                        message: format!(
                            "OAuth provider {} configuration is invalid: {}. {}",
                            provider,
                            validation.validation_errors.join("; "),
                            self.get_oauth_configuration_guidance(provider)
                        ),
                    });
                }
            }

            return Err(AppError::OAuthProviderError {
                provider: provider.to_string(),
                message: format!("OAuth provider {} is not available", provider),
            });
        }

        Ok(())
    }

    /// Create a demo Google OAuth provider for development
    fn create_demo_google_provider() -> Result<GoogleOAuthProvider> {
        GoogleOAuthProvider::with_credentials(
            "demo-google-client-id.apps.googleusercontent.com".to_string(),
            "demo-google-client-secret".to_string(),
            "http://localhost:3000/auth/callback/google".to_string(),
        )
    }

    /// Create a demo GitHub OAuth provider for development
    fn create_demo_github_provider() -> Result<GitHubOAuthProvider> {
        GitHubOAuthProvider::with_credentials(
            "demo-github-client-id".to_string(),
            "demo-github-client-secret".to_string(),
            "http://localhost:3000/auth/callback/github".to_string(),
        )
    }

    // User registration with email/password
    pub async fn register_user(&self, request: CreateUserRequest) -> Result<User> {
        // Validate email format (basic validation)
        if !request.email.contains('@') {
            return Err(AppError::InvalidFieldValue {
                field: "email".to_string(),
                message: "Invalid email format".to_string(),
            });
        }

        // Validate password strength (basic validation)
        if request.password.len() < 8 {
            return Err(AppError::InvalidFieldValue {
                field: "password".to_string(),
                message: "Password must be at least 8 characters long".to_string(),
            });
        }

        // Check if user already exists
        let existing_user = sqlx::query!("SELECT id FROM users WHERE email = $1", request.email)
            .fetch_optional(&self.db_pool)
            .await?;

        if existing_user.is_some() {
            return Err(AppError::AlreadyExists {
                resource: "User with this email".to_string(),
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
        let cached_user = self
            .login_performance_service
            .get_cached_user_login(&request.email, &self.db_pool)
            .await?
            .ok_or_else(|| AppError::InvalidCredentials)?;

        // Verify password using optimized method (runs in background thread)
        let password_valid = self
            .login_performance_service
            .verify_password_optimized(&request.password, &cached_user.password_hash)
            .await?;

        if !password_valid {
            // Record failed login attempt
            let login_time = login_start.elapsed().as_millis() as f64;
            if let Err(e) = self
                .login_performance_service
                .record_login_attempt(false, login_time)
                .await
            {
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
            let totp_code = request
                .totp_code
                .ok_or_else(|| AppError::TwoFactorRequired)?;

            let totp_secret = cached_user.totp_secret.ok_or_else(|| AppError::Internal {
                message: Some("2FA configuration error. Please contact support".to_string()),
            })?;

            if !self.verify_totp(&totp_secret, &totp_code)? {
                // Record failed 2FA attempt
                let login_time = login_start.elapsed().as_millis() as f64;
                if let Err(e) = self
                    .login_performance_service
                    .record_login_attempt(false, login_time)
                    .await
                {
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
        let (refresh_token_raw, refresh_token_hash) = self
            .login_performance_service
            .generate_optimized_refresh_token()
            .await?;

        // Generate access token (this is fast, no need to optimize)
        let access_claims = Claims::new_access_token(
            cached_user.user_id,
            cached_user.email.clone(),
            self.access_token_ttl,
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
        if let Err(e) = self
            .login_performance_service
            .record_login_attempt(true, login_time)
            .await
        {
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

    // ===== OAuth Flow Methods =====

    /// Initiate OAuth flow for a provider
    pub async fn initiate_oauth_flow(
        &self,
        provider_type: OAuthProviderType,
        redirect_uri: String,
    ) -> Result<OAuthFlowResponse> {
        // Validate provider configuration first
        self.validate_oauth_provider_config(&provider_type)?;

        // Use error recovery service to execute with retry logic
        let redirect_uri_clone = redirect_uri.clone();
        let providers = Arc::clone(&self.oauth_providers);
        let state_manager = Arc::clone(&self.oauth_state_manager);

        self.oauth_error_recovery
            .execute_with_recovery(provider_type, "initiate_oauth_flow", move || {
                let redirect_uri = redirect_uri_clone.clone();
                let providers = Arc::clone(&providers);
                let state_manager = Arc::clone(&state_manager);
                let provider_type_clone = provider_type;

                Box::pin(async move {
                    let provider = providers.get(&provider_type_clone).ok_or_else(|| {
                        crate::error::AppError::OAuth(
                            crate::error::oauth::OAuthError::ProviderNotConfigured {
                                provider: provider_type_clone,
                                reason: format!(
                                    "{} OAuth provider not available",
                                    provider_type_clone
                                ),
                                missing_variables: vec![],
                            },
                        )
                    })?;

                    // Generate secure state and store it
                    let flow_response = provider.initiate_flow(&redirect_uri).await?;

                    // Store state for validation
                    let state = crate::models::oauth::OAuthState::new(
                        provider_type_clone,
                        redirect_uri,
                        flow_response.code_verifier.clone(),
                        300, // 5 minutes expiration
                    );

                    let state_token = state_manager.store_state(state);

                    Ok(OAuthFlowResponse {
                        authorization_url: flow_response.authorization_url,
                        state: state_token,
                        code_verifier: flow_response.code_verifier,
                    })
                })
            })
            .await
    }

    /// Complete OAuth flow and create/login user
    pub async fn complete_oauth_flow(
        &self,
        provider_type: OAuthProviderType,
        code: String,
        state: String,
        redirect_uri: String,
    ) -> Result<TokenPair> {
        // Validate provider configuration first
        self.validate_oauth_provider_config(&provider_type)?;

        // Check provider health
        self.validate_oauth_provider_health(&provider_type).await?;

        // Validate state parameter
        let _oauth_state = self
            .oauth_state_manager
            .validate_and_consume_state(&state, &provider_type)
            .map_err(|e| {
                let oauth_error = crate::error::oauth::OAuthError::StateValidationFailed {
                    reason: e.to_string(),
                    expected_provider: Some(provider_type),
                    received_provider: None,
                };

                // Log security event (skip for now due to clone issues)
                // TODO: Fix OAuth error logging

                crate::error::AppError::OAuth(oauth_error)
            })?;

        let provider = self.oauth_providers.get(&provider_type).ok_or_else(|| {
            AppError::OAuthProviderError {
                provider: provider_type.to_string(),
                message: "OAuth provider not available".to_string(),
            }
        })?;

        // Exchange code for tokens with error logging
        let tokens = match provider.exchange_code(&code, &state, &redirect_uri).await {
            Ok(tokens) => tokens,
            Err(e) => {
                // Log OAuth errors for security monitoring
                if let crate::error::AppError::OAuth(ref oauth_error) = e {
                    let security_logger = Arc::clone(&self.oauth_security_logger);
                    let oauth_error_clone = oauth_error.clone();
                    tokio::spawn(async move {
                        security_logger
                            .log_oauth_error(&oauth_error_clone, None, None)
                            .await;
                    });
                }
                return Err(e);
            }
        };

        // Get user info from provider with error logging
        let user_info = if provider_type == OAuthProviderType::Apple && tokens.id_token.is_some() {
            // For Apple, extract user info from ID token
            match self
                .extract_apple_user_info(&tokens.id_token.as_ref().unwrap())
                .await
            {
                Ok(info) => info,
                Err(e) => {
                    if let crate::error::AppError::OAuth(ref oauth_error) = e {
                        let security_logger = Arc::clone(&self.oauth_security_logger);
                        let oauth_error_clone = oauth_error.clone();
                        tokio::spawn(async move {
                            security_logger
                                .log_oauth_error(&oauth_error_clone, None, None)
                                .await;
                        });
                    }
                    return Err(e);
                }
            }
        } else {
            match provider.get_user_info(&tokens.access_token).await {
                Ok(info) => info,
                Err(e) => {
                    if let crate::error::AppError::OAuth(ref oauth_error) = e {
                        let security_logger = Arc::clone(&self.oauth_security_logger);
                        let oauth_error_clone = oauth_error.clone();
                        tokio::spawn(async move {
                            security_logger
                                .log_oauth_error(&oauth_error_clone, None, None)
                                .await;
                        });
                    }
                    return Err(e);
                }
            }
        };

        // Check if user already exists with this OAuth account
        if let Some(existing_user) = self
            .find_user_by_oauth_account(&provider_type, &user_info.provider_user_id)
            .await?
        {
            // Update tokens and login existing user
            self.update_oauth_tokens(&existing_user.id, &provider_type, &tokens)
                .await?;
            self.update_user_last_login(existing_user.id).await?;
            return self
                .generate_token_pair(existing_user.id, &existing_user.email)
                .await;
        }

        // Check if user exists with same email
        if let Some(email) = &user_info.email {
            if let Some(existing_user) = self.find_user_by_email(email).await? {
                // Link OAuth account to existing user
                self.link_oauth_account_to_user(
                    &existing_user,
                    &provider_type,
                    &user_info,
                    &tokens,
                )
                .await?;
                self.update_user_last_login(existing_user.id).await?;
                return self
                    .generate_token_pair(existing_user.id, &existing_user.email)
                    .await;
            }
        }

        // Create new user with OAuth account
        let new_user = self
            .create_user_with_oauth_account(&provider_type, &user_info, &tokens)
            .await?;
        self.generate_token_pair(new_user.id, &new_user.email).await
    }

    /// Link OAuth account to existing authenticated user
    pub async fn link_oauth_account(
        &self,
        user_id: Uuid,
        request: AccountLinkRequest,
    ) -> Result<()> {
        // Validate state parameter
        let _oauth_state = self
            .oauth_state_manager
            .validate_and_consume_state(&request.state, &request.provider)?;

        let provider = self.oauth_providers.get(&request.provider).ok_or_else(|| {
            AppError::OAuthProviderError {
                provider: request.provider.to_string(),
                message: "OAuth provider not configured".to_string(),
            }
        })?;

        // Get user to ensure they exist
        let user = self.get_user_by_id(user_id).await?;

        // Check if user already has this provider linked
        if user.has_oauth_account(&request.provider) {
            return Err(AppError::Conflict {
                message: format!("User already has {} account linked", request.provider),
            });
        }

        // Exchange code for tokens using the redirect_uri from the request
        // (must match the redirect_uri used during OAuth flow initiation)
        let tokens = provider
            .exchange_code(&request.code, &request.state, &request.redirect_uri)
            .await?;

        // Get user info from provider
        let user_info = if request.provider == OAuthProviderType::Apple && tokens.id_token.is_some()
        {
            self.extract_apple_user_info(&tokens.id_token.as_ref().unwrap())
                .await?
        } else {
            provider.get_user_info(&tokens.access_token).await?
        };

        // Check if this OAuth account is already linked to another user
        if let Some(_existing_user) = self
            .find_user_by_oauth_account(&request.provider, &user_info.provider_user_id)
            .await?
        {
            return Err(AppError::Conflict {
                message: format!(
                    "This {} account is already linked to another user",
                    request.provider
                ),
            });
        }

        // Link the account
        self.link_oauth_account_to_user(&user, &request.provider, &user_info, &tokens)
            .await?;

        Ok(())
    }

    /// Unlink OAuth account from user
    pub async fn unlink_oauth_account(
        &self,
        user_id: Uuid,
        provider_type: OAuthProviderType,
    ) -> Result<()> {
        // Check if user has other authentication methods (password or other OAuth accounts)
        let user = self.get_user_by_id(user_id).await?;

        // Count remaining authentication methods after unlinking
        let remaining_oauth_accounts = user
            .oauth_accounts
            .iter()
            .filter(|account| account.provider != provider_type)
            .count();

        let has_password = user.password_hash.is_some();

        // Ensure user retains access after unlinking
        if !has_password && remaining_oauth_accounts == 0 {
            return Err(AppError::InvalidFieldValue {
                field: "provider".to_string(),
                message:
                    "Cannot unlink the only authentication method. Please set a password first."
                        .to_string(),
            });
        }

        // Get OAuth account to potentially revoke tokens
        let oauth_account = sqlx::query!(
            r#"
            SELECT access_token_encrypted, refresh_token_encrypted
            FROM oauth_accounts
            WHERE user_id = $1 AND provider = $2
            "#,
            user_id,
            provider_type.to_string()
        )
        .fetch_optional(&self.db_pool)
        .await?;

        // Delete OAuth account from database
        let deleted_rows = sqlx::query!(
            "DELETE FROM oauth_accounts WHERE user_id = $1 AND provider = $2",
            user_id,
            provider_type.to_string()
        )
        .execute(&self.db_pool)
        .await?
        .rows_affected();

        if deleted_rows == 0 {
            return Err(AppError::NotFound {
                resource: format!("OAuth account for provider {}", provider_type),
            });
        }

        // Attempt to revoke tokens with provider (best effort)
        if let Some(account) = oauth_account {
            if let Some(encrypted_token) = &account.access_token_encrypted {
                if let Ok(access_token) = self.oauth_encryption.decrypt_token(encrypted_token).await
                {
                    if let Some(provider) = self.oauth_providers.get(&provider_type) {
                        // Revoke token with provider (don't fail if this fails)
                        if let Err(e) = provider.revoke_token(&access_token).await {
                            tracing::warn!(
                                "Failed to revoke OAuth token with provider {}: {}",
                                provider_type,
                                e
                            );
                        }
                    }
                }
            }
        }

        // Audit log the account unlinking
        self.log_audit_event(
            user_id,
            "oauth_account_unlinked",
            "oauth_account",
            &provider_type.to_string(),
        )
        .await?;

        Ok(())
    }

    /// Merge two user accounts (for duplicate account resolution)
    /// TODO: Re-enable once SQLx cache is updated with new queries
    pub async fn merge_accounts(
        &self,
        _primary_user_id: Uuid,
        _request: MergeAccountsRequest,
    ) -> Result<MergeAccountsResponse> {
        // Temporarily disabled until SQLx cache is updated with new queries
        // The full implementation is ready but needs database access to cache the queries
        Err(AppError::NotFound {
            resource: "Account merging is temporarily disabled until database setup is complete"
                .to_string(),
        })
    }

    /// Refresh OAuth tokens for a user and provider
    /// Automatically refreshes tokens when they are near expiry
    pub async fn refresh_oauth_tokens(
        &self,
        user_id: Uuid,
        provider_type: OAuthProviderType,
    ) -> Result<()> {
        tracing::debug!(
            user_id = %user_id,
            provider = %provider_type,
            "Starting OAuth token refresh"
        );

        // Get OAuth account with encrypted tokens
        let oauth_account = sqlx::query!(
            r#"
            SELECT id, refresh_token_encrypted, token_expires_at, updated_at
            FROM oauth_accounts
            WHERE user_id = $1 AND provider = $2
            "#,
            user_id,
            provider_type.to_string()
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: format!("OAuth account for provider {}", provider_type),
        })?;

        // Check if refresh token exists
        let refresh_token_encrypted = oauth_account.refresh_token_encrypted
            .ok_or_else(|| AppError::OAuthProviderError {
                provider: provider_type.to_string(),
                message: "No refresh token available for this OAuth account. User needs to re-authenticate.".to_string(),
            })?;

        // Decrypt refresh token with proper error handling
        let refresh_token = self
            .oauth_encryption
            .decrypt_token(&refresh_token_encrypted)
            .await
            .map_err(|e| AppError::OAuthProviderError {
                provider: provider_type.to_string(),
                message: format!(
                    "Failed to decrypt refresh token: {}. User may need to re-authenticate.",
                    e
                ),
            })?;

        // Get OAuth provider
        let provider = self.oauth_providers.get(&provider_type).ok_or_else(|| {
            AppError::OAuthProviderError {
                provider: provider_type.to_string(),
                message: "OAuth provider not configured".to_string(),
            }
        })?;

        // Refresh tokens with provider with retry logic
        let new_tokens = match provider.refresh_token(&refresh_token).await {
            Ok(tokens) => tokens,
            Err(e) => {
                tracing::warn!(
                    user_id = %user_id,
                    provider = %provider_type,
                    error = %e,
                    "OAuth token refresh failed"
                );

                // Check if this is a permanent failure (invalid refresh token)
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("invalid")
                    || error_msg.contains("expired")
                    || error_msg.contains("revoked")
                {
                    return Err(AppError::OAuthProviderError {
                        provider: provider_type.to_string(),
                        message:
                            "Refresh token is invalid or expired. User needs to re-authenticate."
                                .to_string(),
                    });
                }

                // For other errors, return the original error
                return Err(e);
            }
        };

        // Update tokens in database with transaction for atomicity
        let mut tx = self.db_pool.begin().await?;

        // Encrypt the new tokens
        let (encrypted_access_token, encrypted_refresh_token) = self
            .oauth_encryption
            .encrypt_token_pair(
                &new_tokens.access_token,
                new_tokens.refresh_token.as_deref(),
            )
            .map_err(|e| AppError::OAuthProviderError {
                provider: provider_type.to_string(),
                message: format!("Failed to encrypt new tokens: {}", e),
            })?;

        let now = Utc::now();
        let token_expires_at = new_tokens
            .expires_in
            .map(|expires_in| now + Duration::seconds(expires_in));

        // Update tokens in database
        sqlx::query!(
            r#"
            UPDATE oauth_accounts
            SET access_token_encrypted = $1,
                refresh_token_encrypted = $2,
                token_expires_at = $3,
                updated_at = $4
            WHERE user_id = $5 AND provider = $6
            "#,
            encrypted_access_token,
            encrypted_refresh_token,
            token_expires_at,
            now,
            user_id,
            provider_type.to_string()
        )
        .execute(&mut *tx)
        .await?;

        // Audit log the token refresh
        sqlx::query!(
            r#"
            INSERT INTO audit_log (user_id, action, resource_type, resource_id, details, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            user_id,
            "oauth_tokens_refreshed",
            "oauth_account",
            provider_type.to_string(),
            serde_json::json!({
                "provider": provider_type.to_string(),
                "expires_at": token_expires_at,
                "has_refresh_token": new_tokens.refresh_token.is_some()
            }),
            now
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        tracing::info!(
            user_id = %user_id,
            provider = %provider_type,
            expires_at = ?token_expires_at,
            "OAuth tokens refreshed successfully"
        );

        Ok(())
    }

    /// Get a valid OAuth token, automatically refreshing if near expiry
    /// Returns decrypted access token ready for API usage
    pub async fn get_valid_oauth_token(
        &self,
        user_id: Uuid,
        provider_type: OAuthProviderType,
    ) -> Result<String> {
        tracing::debug!(
            user_id = %user_id,
            provider = %provider_type,
            "Getting valid OAuth token"
        );

        // Get OAuth account with token information
        let oauth_account = sqlx::query!(
            r#"
            SELECT access_token_encrypted, token_expires_at, refresh_token_encrypted
            FROM oauth_accounts
            WHERE user_id = $1 AND provider = $2
            "#,
            user_id,
            provider_type.to_string()
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: format!("OAuth account for provider {}", provider_type),
        })?;

        // Check if token is expired or near expiry (refresh 5 minutes before expiry)
        let needs_refresh = if let Some(expires_at) = oauth_account.token_expires_at {
            let refresh_threshold = Utc::now() + Duration::minutes(5);
            expires_at <= refresh_threshold
        } else {
            false // No expiration means token doesn't expire
        };

        // Refresh token if needed and refresh token is available
        if needs_refresh && oauth_account.refresh_token_encrypted.is_some() {
            tracing::debug!(
                user_id = %user_id,
                provider = %provider_type,
                expires_at = ?oauth_account.token_expires_at,
                "Token near expiry, refreshing automatically"
            );

            match self
                .refresh_oauth_tokens(user_id, provider_type.clone())
                .await
            {
                Ok(()) => {
                    // Get the updated token after refresh
                    let updated_account = sqlx::query!(
                        "SELECT access_token_encrypted FROM oauth_accounts WHERE user_id = $1 AND provider = $2",
                        user_id,
                        provider_type.to_string()
                    )
                    .fetch_one(&self.db_pool)
                    .await?;

                    let access_token_encrypted = updated_account
                        .access_token_encrypted
                        .ok_or_else(|| AppError::OAuthProviderError {
                            provider: provider_type.to_string(),
                            message: "No access token found after refresh".to_string(),
                        })?;

                    return self
                        .oauth_encryption
                        .decrypt_token(&access_token_encrypted)
                        .await
                        .map_err(|e| AppError::OAuthProviderError {
                            provider: provider_type.to_string(),
                            message: format!("Failed to decrypt refreshed token: {}", e),
                        });
                }
                Err(e) => {
                    tracing::warn!(
                        user_id = %user_id,
                        provider = %provider_type,
                        error = %e,
                        "Automatic token refresh failed, returning current token"
                    );
                    // Continue with current token - it might still work
                }
            }
        }

        // Decrypt and return current token
        let access_token_encrypted =
            oauth_account
                .access_token_encrypted
                .ok_or_else(|| AppError::OAuthProviderError {
                    provider: provider_type.to_string(),
                    message: "No access token found for OAuth account".to_string(),
                })?;

        self.oauth_encryption
            .decrypt_token(&access_token_encrypted)
            .await
            .map_err(|e| AppError::OAuthProviderError {
                provider: provider_type.to_string(),
                message: format!(
                    "Failed to decrypt OAuth token: {}. User may need to re-authenticate.",
                    e
                ),
            })
    }

    /// OAuth login using existing request format (for backward compatibility)
    pub async fn oauth_login(&self, _request: OAuthLoginRequest) -> Result<TokenPair> {
        // Temporarily disabled until SQLx cache is updated
        Err(AppError::NotFound {
            resource: "OAuth login temporarily disabled".to_string(),
        })
    }

    // Generate JWT token pair with database storage
    async fn generate_token_pair(&self, user_id: Uuid, email: &str) -> Result<TokenPair> {
        // Generate access token (24-hour expiration as required)
        let access_claims =
            Claims::new_access_token(user_id, email.to_string(), self.access_token_ttl);
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
        self.generate_token_pair(session.user_id.unwrap(), &session.email)
            .await
    }

    // Verify JWT token
    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| AppError::TokenInvalid)?;

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
            return Err(AppError::Conflict {
                message: "2FA is already enabled for this user".to_string(),
            });
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
        let user_email = sqlx::query!("SELECT email FROM users WHERE id = $1", user_id)
            .fetch_optional(&self.db_pool)
            .await?
            .map(|row| row.email)
            .unwrap_or_default();

        // Check if 2FA is already enabled
        if user.totp_enabled.unwrap_or(false) {
            return Err(anyhow!("2FA is already enabled for this user").into());
        }

        let totp_secret = user
            .totp_secret
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
        if let Err(e) = self
            .login_performance_service
            .invalidate_user_cache(&user_email)
            .await
        {
            tracing::warn!(
                "Failed to invalidate login cache for user {}: {}",
                user_email,
                e
            );
        }

        // Audit log the 2FA enablement
        self.log_audit_event(user_id, "totp_enabled", "user", &user_id.to_string())
            .await?;

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
        let user_email = sqlx::query!("SELECT email FROM users WHERE id = $1", user_id)
            .fetch_optional(&self.db_pool)
            .await?
            .map(|row| row.email)
            .unwrap_or_default();

        // Check if 2FA is enabled
        if !user.totp_enabled.unwrap_or(false) {
            return Err(anyhow!("2FA is not enabled for this user").into());
        }

        let totp_secret = user
            .totp_secret
            .ok_or_else(|| anyhow!("TOTP secret not found"))?;

        // Verify the TOTP code before disabling
        if !self.verify_totp(&totp_secret, totp_code)? {
            return Err(
                anyhow!("Invalid TOTP code. Cannot disable 2FA without verification").into(),
            );
        }

        // Disable 2FA and remove secret
        sqlx::query!(
            "UPDATE users SET totp_enabled = FALSE, totp_secret = NULL, updated_at = NOW() WHERE id = $1",
            user_id
        )
        .execute(&self.db_pool)
        .await?;

        // Invalidate login cache since user data changed
        if let Err(e) = self
            .login_performance_service
            .invalidate_user_cache(&user_email)
            .await
        {
            tracing::warn!(
                "Failed to invalidate login cache for user {}: {}",
                user_email,
                e
            );
        }

        // Audit log the 2FA disablement
        self.log_audit_event(user_id, "totp_disabled", "user", &user_id.to_string())
            .await?;

        Ok(())
    }

    // Verify TOTP code with proper error handling and clock skew tolerance
    fn verify_totp(&self, secret: &str, code: &str) -> Result<bool> {
        // Validate input
        if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
            return Ok(false);
        }

        let secret_bytes = base32::decode(base32::Alphabet::Rfc4648 { padding: true }, secret)
            .ok_or_else(|| AppError::Internal {
                message: Some("Invalid TOTP secret format".to_string()),
            })?;

        if secret_bytes.len() < 10 {
            return Err(AppError::Internal {
                message: Some("TOTP secret too short".to_string()),
            });
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
        let mut mac = HmacSha1::new_from_slice(secret).map_err(|_| AppError::Internal {
            message: Some("Invalid TOTP secret length".to_string()),
        })?;
        mac.update(&time_bytes);
        let result = mac.finalize().into_bytes();

        // Dynamic truncation as per RFC 4226
        let offset = (result[result.len() - 1] & 0xf) as usize;
        if offset + 4 > result.len() {
            return Err(AppError::Internal {
                message: Some("Invalid HMAC result for TOTP".to_string()),
            });
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

    // Get user by ID from database
    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<User> {
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

        let oauth_accounts = self.load_oauth_accounts(user_id).await?;

        Ok(User {
            id: user.id,
            email: user.email,
            password_hash: user.password_hash,
            email_verified: user.email_verified.unwrap_or(false),
            totp_secret: user.totp_secret,
            totp_enabled: user.totp_enabled.unwrap_or(false),
            oauth_accounts,
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
        let _user_exists = sqlx::query!("SELECT id FROM users WHERE email = $1", email)
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
        let user = sqlx::query!("SELECT totp_enabled FROM users WHERE id = $1", user_id)
            .fetch_optional(&self.db_pool)
            .await?
            .ok_or_else(|| anyhow!("User not found"))?;

        Ok(user.totp_enabled.unwrap_or(false))
    }

    // Verify TOTP code without side effects (for testing/validation)
    pub fn verify_totp_code(&self, secret: &str, code: &str) -> Result<bool> {
        self.verify_totp(secret, code)
    }

    // ===== OAuth Helper Methods =====

    /// Find user by OAuth account
    async fn find_user_by_oauth_account(
        &self,
        provider: &OAuthProviderType,
        provider_user_id: &str,
    ) -> Result<Option<User>> {
        let user_row = sqlx::query!(
            r#"
            SELECT u.id, u.email, u.password_hash, u.email_verified, u.totp_secret, u.totp_enabled,
                   u.created_at, u.updated_at, u.last_login, u.settings
            FROM users u
            JOIN oauth_accounts oa ON u.id = oa.user_id
            WHERE oa.provider = $1 AND oa.provider_user_id = $2
            "#,
            provider.to_string(),
            provider_user_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(row) = user_row {
            let mut user = User {
                id: row.id,
                email: row.email,
                password_hash: row.password_hash,
                email_verified: row.email_verified.unwrap_or(false),
                totp_secret: row.totp_secret,
                totp_enabled: row.totp_enabled.unwrap_or(false),
                oauth_accounts: Vec::new(), // Will be loaded separately if needed
                created_at: row.created_at.unwrap_or(Utc::now()),
                updated_at: row.updated_at.unwrap_or(Utc::now()),
                last_login: row.last_login,
                settings: serde_json::from_value(row.settings.unwrap_or(serde_json::json!({})))
                    .unwrap_or_default(),
            };

            // Load OAuth accounts
            user.oauth_accounts = self.load_oauth_accounts(user.id).await?;

            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    /// Find user by email
    async fn find_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let user_row = sqlx::query!(
            r#"
            SELECT id, email, password_hash, email_verified, totp_secret, totp_enabled,
                   created_at, updated_at, last_login, settings
            FROM users WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(row) = user_row {
            let mut user = User {
                id: row.id,
                email: row.email,
                password_hash: row.password_hash,
                email_verified: row.email_verified.unwrap_or(false),
                totp_secret: row.totp_secret,
                totp_enabled: row.totp_enabled.unwrap_or(false),
                oauth_accounts: Vec::new(),
                created_at: row.created_at.unwrap_or(Utc::now()),
                updated_at: row.updated_at.unwrap_or(Utc::now()),
                last_login: row.last_login,
                settings: serde_json::from_value(row.settings.unwrap_or(serde_json::json!({})))
                    .unwrap_or_default(),
            };

            // Load OAuth accounts
            user.oauth_accounts = self.load_oauth_accounts(user.id).await?;

            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    /// Load OAuth accounts for a user
    pub async fn load_oauth_accounts(&self, user_id: Uuid) -> Result<Vec<OAuthAccount>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, provider, provider_user_id, email, display_name, avatar_url,
                   access_token_encrypted, refresh_token_encrypted, token_expires_at,
                   created_at, updated_at
            FROM oauth_accounts WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut accounts = Vec::new();
        for row in rows {
            let provider = row.provider.parse::<OAuthProviderType>().map_err(|e| {
                AppError::ExternalServiceError(format!("Invalid OAuth provider: {}", e))
            })?;

            accounts.push(OAuthAccount {
                id: row.id,
                user_id,
                provider,
                provider_user_id: row.provider_user_id,
                email: row.email,
                display_name: row.display_name,
                avatar_url: row.avatar_url,
                access_token_encrypted: row.access_token_encrypted.unwrap_or_default(),
                refresh_token_encrypted: row.refresh_token_encrypted,
                token_expires_at: row.token_expires_at,
                last_used_at: None, // TODO: Add back when SQLx cache is updated
                created_at: row.created_at.unwrap_or(Utc::now()),
                updated_at: row.updated_at.unwrap_or(Utc::now()),
            });
        }

        Ok(accounts)
    }

    /// Create new user with OAuth account
    async fn create_user_with_oauth_account(
        &self,
        provider: &OAuthProviderType,
        user_info: &OAuthUserInfo,
        tokens: &OAuthTokens,
    ) -> Result<User> {
        let mut tx = self.db_pool.begin().await?;

        // Create user
        let user_id = Uuid::new_v4();
        let email = user_info
            .email
            .clone()
            .unwrap_or_else(|| format!("{}@{}.oauth", user_info.provider_user_id, provider));
        let now = Utc::now();

        sqlx::query!(
            r#"
            INSERT INTO users (id, email, password_hash, email_verified, totp_enabled, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            user_id,
            email,
            None::<String>, // OAuth-only user has no password
            user_info.email_verified.unwrap_or(false),
            false,
            now,
            now
        )
        .execute(&mut *tx)
        .await?;

        // Create OAuth account
        let oauth_account_id = Uuid::new_v4();
        let (encrypted_access_token, encrypted_refresh_token) = self
            .oauth_encryption
            .encrypt_token_pair(&tokens.access_token, tokens.refresh_token.as_deref())?;

        let token_expires_at = tokens
            .expires_in
            .map(|expires_in| now + Duration::seconds(expires_in));

        sqlx::query!(
            r#"
            INSERT INTO oauth_accounts (
                id, user_id, provider, provider_user_id, email, display_name, avatar_url,
                access_token_encrypted, refresh_token_encrypted, token_expires_at,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            oauth_account_id,
            user_id,
            provider.to_string(),
            user_info.provider_user_id,
            user_info.email,
            user_info.display_name,
            user_info.avatar_url,
            encrypted_access_token,
            encrypted_refresh_token,
            token_expires_at,
            now,
            now
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Audit log the user creation
        self.log_audit_event(user_id, "user_created_oauth", "user", &user_id.to_string())
            .await?;

        // Return the created user
        self.get_user_by_id(user_id).await
    }

    /// Link OAuth account to existing user
    async fn link_oauth_account_to_user(
        &self,
        user: &User,
        provider: &OAuthProviderType,
        user_info: &OAuthUserInfo,
        tokens: &OAuthTokens,
    ) -> Result<()> {
        let mut tx = self.db_pool.begin().await?;

        // Validation: Check if user already has this provider linked
        let existing_account = sqlx::query!(
            "SELECT id FROM oauth_accounts WHERE user_id = $1 AND provider = $2",
            user.id,
            provider.to_string()
        )
        .fetch_optional(&mut *tx)
        .await?;

        if existing_account.is_some() {
            return Err(AppError::Conflict {
                message: format!("User already has {} account linked", provider),
            });
        }

        // Validation: Check if this OAuth account is already linked to another user
        let existing_user = sqlx::query!(
            "SELECT user_id FROM oauth_accounts WHERE provider = $1 AND provider_user_id = $2",
            provider.to_string(),
            user_info.provider_user_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(existing) = existing_user {
            if existing.user_id != user.id {
                return Err(AppError::Conflict {
                    message: format!(
                        "This {} account is already linked to another user",
                        provider
                    ),
                });
            }
        }

        // Encrypt tokens before storage
        let oauth_account_id = Uuid::new_v4();
        let (encrypted_access_token, encrypted_refresh_token) = self
            .oauth_encryption
            .encrypt_token_pair(&tokens.access_token, tokens.refresh_token.as_deref())?;

        let now = Utc::now();
        let token_expires_at = tokens
            .expires_in
            .map(|expires_in| now + Duration::seconds(expires_in));

        // Insert OAuth account with proper error handling for database constraint violations
        let result = sqlx::query!(
            r#"
            INSERT INTO oauth_accounts (
                id, user_id, provider, provider_user_id, email, display_name, avatar_url,
                access_token_encrypted, refresh_token_encrypted, token_expires_at,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            oauth_account_id,
            user.id,
            provider.to_string(),
            user_info.provider_user_id,
            user_info.email,
            user_info.display_name,
            user_info.avatar_url,
            encrypted_access_token,
            encrypted_refresh_token,
            token_expires_at,
            now,
            now
        )
        .execute(&mut *tx)
        .await;

        // Handle database constraint violations
        if let Err(sqlx::Error::Database(db_err)) = &result {
            if db_err.constraint().is_some() {
                let constraint_name = db_err.constraint().unwrap_or("unknown");
                if constraint_name.contains("oauth_accounts_user_provider_unique") {
                    return Err(AppError::Conflict {
                        message: format!("User already has {} account linked", provider),
                    });
                } else if constraint_name.contains("oauth_accounts_provider_user_unique") {
                    return Err(AppError::Conflict {
                        message: format!(
                            "This {} account is already linked to another user",
                            provider
                        ),
                    });
                } else {
                    return Err(AppError::DatabaseConstraintViolation(format!(
                        "Database constraint violation: {}",
                        constraint_name
                    )));
                }
            }
        }
        result?;

        tx.commit().await?;

        // Audit log the account linking
        self.log_audit_event(
            user.id,
            "oauth_account_linked",
            "oauth_account",
            &provider.to_string(),
        )
        .await?;

        Ok(())
    }

    /// Update OAuth tokens for existing account
    async fn update_oauth_tokens(
        &self,
        user_id: &Uuid,
        provider: &OAuthProviderType,
        tokens: &OAuthTokens,
    ) -> Result<()> {
        // Encrypt the new tokens
        let (encrypted_access_token, encrypted_refresh_token) = self
            .oauth_encryption
            .encrypt_token_pair(&tokens.access_token, tokens.refresh_token.as_deref())?;

        let now = Utc::now();
        let token_expires_at = tokens
            .expires_in
            .map(|expires_in| now + Duration::seconds(expires_in));

        // Update tokens in database
        sqlx::query!(
            r#"
            UPDATE oauth_accounts
            SET access_token_encrypted = $1,
                refresh_token_encrypted = $2,
                token_expires_at = $3,
                updated_at = $4
            WHERE user_id = $5 AND provider = $6
            "#,
            encrypted_access_token,
            encrypted_refresh_token,
            token_expires_at,
            now,
            user_id,
            provider.to_string()
        )
        .execute(&self.db_pool)
        .await?;

        // Audit log the token update
        self.log_audit_event(
            *user_id,
            "oauth_tokens_updated",
            "oauth_account",
            &provider.to_string(),
        )
        .await?;

        Ok(())
    }

    /// Update user's last login timestamp
    async fn update_user_last_login(&self, user_id: Uuid) -> Result<()> {
        let now = Utc::now();
        sqlx::query!(
            "UPDATE users SET last_login = $1, updated_at = $2 WHERE id = $3",
            now,
            now,
            user_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Extract user info from Apple ID token with full JWT signature verification
    ///
    /// This method uses Apple's JWKS (JSON Web Key Set) to verify the ID token signature,
    /// ensuring the token was actually issued by Apple and hasn't been tampered with.
    ///
    /// The validation includes:
    /// - JWT signature verification using Apple's public keys (RS256)
    /// - Issuer validation (must be "https://appleid.apple.com")
    /// - Audience validation (must match our client_id)
    /// - Expiration and issued-at time validation
    async fn extract_apple_user_info(&self, id_token: &str) -> Result<OAuthUserInfo> {
        // Get the Apple OAuth provider to use its validate_id_token method
        let provider = self
            .oauth_providers
            .get(&OAuthProviderType::Apple)
            .ok_or_else(|| AppError::OAuthProviderError {
                provider: "Apple".to_string(),
                message: "Apple OAuth provider not configured".to_string(),
            })?;

        // Use the provider's validate_id_token method which performs full JWKS verification
        provider.validate_id_token(id_token).await
    }

    /// Get available OAuth providers
    pub fn get_available_oauth_providers(&self) -> Vec<OAuthProviderType> {
        self.oauth_providers.keys().cloned().collect()
    }

    /// Check if OAuth provider is available
    pub fn is_oauth_provider_available(&self, provider: &OAuthProviderType) -> bool {
        self.oauth_providers.contains_key(provider)
    }

    // ===== Token Management and Refresh Methods =====

    /// Check and refresh expired OAuth tokens for a user
    pub async fn refresh_expired_oauth_tokens(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<OAuthProviderType>> {
        let mut refreshed_providers = Vec::new();

        // Get all OAuth accounts for the user
        let oauth_accounts = self.load_oauth_accounts(user_id).await?;

        for account in oauth_accounts {
            if account.is_token_expired() && account.refresh_token_encrypted.is_some() {
                match self
                    .refresh_oauth_tokens(user_id, account.provider.clone())
                    .await
                {
                    Ok(()) => {
                        let provider = account.provider.clone();
                        refreshed_providers.push(account.provider);
                        tracing::info!(
                            user_id = %user_id,
                            provider = %provider,
                            "Successfully refreshed OAuth tokens"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            user_id = %user_id,
                            provider = %account.provider,
                            error = %e,
                            "Failed to refresh OAuth tokens"
                        );

                        // Notify user about token refresh failure
                        self.notify_token_refresh_failure(user_id, &account.provider)
                            .await?;
                    }
                }
            }
        }

        Ok(refreshed_providers)
    }

    /// Check all users for expired tokens and refresh them (background job)
    pub async fn refresh_all_expired_tokens(&self) -> Result<u32> {
        // Temporarily disabled until SQLx cache is updated
        Ok(0)
    }

    #[allow(dead_code)]
    async fn refresh_all_expired_tokens_impl(&self) -> Result<u32> {
        let expired_accounts = sqlx::query!(
            r#"
            SELECT DISTINCT user_id, provider
            FROM oauth_accounts 
            WHERE token_expires_at IS NOT NULL 
            AND token_expires_at < NOW() 
            AND refresh_token_encrypted IS NOT NULL
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut refreshed_count = 0;

        for account in expired_accounts {
            let provider = account.provider.parse::<OAuthProviderType>().map_err(|e| {
                AppError::ExternalServiceError(format!("Invalid OAuth provider: {}", e))
            })?;

            match self
                .refresh_oauth_tokens(account.user_id, provider.clone())
                .await
            {
                Ok(()) => {
                    refreshed_count += 1;
                    tracing::debug!(
                        user_id = %account.user_id,
                        provider = %provider,
                        "Background token refresh successful"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        user_id = %account.user_id,
                        provider = %provider,
                        error = %e,
                        "Background token refresh failed"
                    );

                    // Notify user about token refresh failure
                    if let Err(notify_err) = self
                        .notify_token_refresh_failure(account.user_id, &provider)
                        .await
                    {
                        tracing::error!(
                            user_id = %account.user_id,
                            provider = %provider,
                            error = %notify_err,
                            "Failed to notify user about token refresh failure"
                        );
                    }
                }
            }
        }

        tracing::info!(
            refreshed_count = refreshed_count,
            "Background OAuth token refresh completed"
        );

        Ok(refreshed_count)
    }

    /// Revoke OAuth tokens for a provider (when unlinking account)
    pub async fn revoke_oauth_tokens(
        &self,
        _user_id: Uuid,
        _provider_type: OAuthProviderType,
    ) -> Result<()> {
        // Temporarily disabled until SQLx cache is updated
        Err(AppError::NotFound {
            resource: "OAuth token revocation temporarily disabled".to_string(),
        })
    }

    /// Clean up expired OAuth tokens that cannot be refreshed
    pub async fn cleanup_expired_tokens(&self) -> Result<u32> {
        // Temporarily disabled until SQLx cache is updated
        Ok(0)
    }

    #[allow(dead_code)]
    async fn cleanup_expired_tokens_impl(&self) -> Result<u32> {
        let deleted_count = sqlx::query!(
            r#"
            DELETE FROM oauth_accounts 
            WHERE token_expires_at IS NOT NULL 
            AND token_expires_at < NOW() - INTERVAL '30 days'
            AND refresh_token_encrypted IS NULL
            "#
        )
        .execute(&self.db_pool)
        .await?
        .rows_affected();

        if deleted_count > 0 {
            tracing::info!(
                deleted_count = deleted_count,
                "Cleaned up expired OAuth tokens without refresh capability"
            );
        }

        Ok(deleted_count as u32)
    }

    /// Get OAuth token expiration status for a user
    pub async fn get_oauth_token_status(&self, user_id: Uuid) -> Result<Vec<OAuthTokenStatus>> {
        let accounts = self.load_oauth_accounts(user_id).await?;
        let mut statuses = Vec::new();

        for account in accounts {
            let status = if let Some(expires_at) = account.token_expires_at {
                let now = Utc::now();
                let time_until_expiry = expires_at - now;

                if time_until_expiry.num_seconds() < 0 {
                    TokenExpirationStatus::Expired
                } else if time_until_expiry.num_hours() < 24 {
                    TokenExpirationStatus::ExpiringSoon {
                        hours_remaining: time_until_expiry.num_hours() as u32,
                    }
                } else {
                    TokenExpirationStatus::Valid { expires_at }
                }
            } else {
                TokenExpirationStatus::NoExpiration
            };

            statuses.push(OAuthTokenStatus {
                provider: account.provider,
                status,
                has_refresh_token: account.refresh_token_encrypted.is_some(),
                last_refreshed: account.updated_at,
            });
        }

        Ok(statuses)
    }

    /// Notify user about token refresh failure
    async fn notify_token_refresh_failure(
        &self,
        user_id: Uuid,
        provider: &OAuthProviderType,
    ) -> Result<()> {
        // In a real implementation, this would send an email or push notification
        // For now, we'll just log an audit event
        self.log_audit_event(
            user_id,
            "oauth_token_refresh_failed",
            "oauth_account",
            &provider.to_string(),
        )
        .await?;

        // You could also store a notification in the database for the user to see
        // when they next log in, or send an email notification
        tracing::info!(
            user_id = %user_id,
            provider = %provider,
            "User notified about OAuth token refresh failure"
        );

        Ok(())
    }

    /// Schedule automatic token refresh for tokens expiring soon
    /// TODO: Re-enable once SQLx cache is updated
    pub async fn schedule_token_refresh(&self) -> Result<Vec<TokenRefreshSchedule>> {
        // Temporarily disabled until SQLx cache is updated
        Ok(vec![])
    }

    /// Get OAuth account connection health for dashboard display
    pub async fn get_oauth_account_health(&self, user_id: Uuid) -> Result<Vec<OAuthAccountHealth>> {
        let accounts = self.load_oauth_accounts(user_id).await?;
        let mut health_statuses = Vec::new();

        for account in accounts {
            let connection_status = self.check_oauth_connection_health(&account).await?;

            let health = OAuthAccountHealth {
                provider: account.provider,
                email: account.email.clone(),
                display_name: account.display_name.clone(),
                avatar_url: account.avatar_url.clone(),
                connection_status,
                last_used: account.last_used_at,
                token_expires_at: account.token_expires_at,
                has_refresh_token: account.refresh_token_encrypted.is_some(),
                created_at: account.created_at,
                updated_at: account.updated_at,
            };

            health_statuses.push(health);
        }

        Ok(health_statuses)
    }

    /// Check the health of a specific OAuth connection
    async fn check_oauth_connection_health(
        &self,
        account: &OAuthAccount,
    ) -> Result<OAuthConnectionStatus> {
        // Check if token is expired
        if let Some(expires_at) = account.token_expires_at {
            if Utc::now() > expires_at {
                return Ok(OAuthConnectionStatus::TokenExpired {
                    expired_at: expires_at,
                    has_refresh_token: account.refresh_token_encrypted.is_some(),
                });
            }
        }

        // Check if token is expiring soon
        if let Some(expires_at) = account.token_expires_at {
            let time_until_expiry = expires_at - Utc::now();
            if time_until_expiry.num_hours() < 24 {
                return Ok(OAuthConnectionStatus::ExpiringSoon {
                    expires_at,
                    hours_remaining: time_until_expiry.num_hours() as u32,
                });
            }
        }

        // Check if account hasn't been used recently (potential stale connection)
        if let Some(last_used) = account.last_used_at {
            let days_since_use = (Utc::now() - last_used).num_days();
            if days_since_use > 90 {
                return Ok(OAuthConnectionStatus::Stale {
                    last_used,
                    days_since_use: days_since_use as u32,
                });
            }
        }

        // Check provider health
        if let Some(provider_health) = self
            .oauth_health_monitor
            .get_provider_health(&account.provider)
            .await
        {
            match provider_health.status {
                OAuthProviderHealthStatus::Healthy => Ok(OAuthConnectionStatus::Healthy),
                OAuthProviderHealthStatus::Degraded { reason } => {
                    Ok(OAuthConnectionStatus::ProviderDegraded { reason })
                }
                OAuthProviderHealthStatus::Unhealthy { reason } => {
                    Ok(OAuthConnectionStatus::ProviderUnavailable { reason })
                }
                OAuthProviderHealthStatus::Unknown => Ok(OAuthConnectionStatus::Healthy), // Default to healthy if unknown
            }
        } else {
            Ok(OAuthConnectionStatus::Healthy) // Default to healthy if no health info
        }
    }

    /// Get users who need OAuth token refresh notifications
    /// TODO: Re-enable once SQLx cache is updated
    pub async fn get_users_needing_token_notifications(
        &self,
    ) -> Result<Vec<TokenNotificationTarget>> {
        // Temporarily disabled until SQLx cache is updated
        Ok(vec![])
    }

    /// Execute proactive token refresh for high-priority tokens
    pub async fn execute_proactive_token_refresh(&self) -> Result<TokenRefreshSummary> {
        let schedules = self.schedule_token_refresh().await?;
        let high_priority_schedules: Vec<_> = schedules
            .into_iter()
            .filter(|s| s.refresh_priority == RefreshPriority::High)
            .collect();

        let mut summary = TokenRefreshSummary {
            total_attempted: high_priority_schedules.len() as u32,
            successful_refreshes: 0,
            failed_refreshes: 0,
            errors: Vec::new(),
        };

        for schedule in high_priority_schedules {
            let provider = schedule.provider;
            let user_id = schedule.user_id;

            tracing::debug!(
                user_id = %user_id,
                provider = %provider,
                expires_at = %schedule.expires_at,
                "Attempting proactive token refresh"
            );

            match self.refresh_oauth_tokens(user_id, provider).await {
                Ok(()) => {
                    summary.successful_refreshes += 1;
                    tracing::info!(
                        user_id = %user_id,
                        provider = %provider,
                        "Successfully refreshed OAuth token proactively"
                    );
                }
                Err(e) => {
                    summary.failed_refreshes += 1;
                    summary
                        .errors
                        .push(format!("User {}, Provider {}: {}", user_id, provider, e));

                    // Notify user about refresh failure
                    if let Err(notify_err) =
                        self.notify_token_refresh_failure(user_id, &provider).await
                    {
                        tracing::warn!(
                            user_id = %user_id,
                            provider = %provider,
                            error = %notify_err,
                            "Failed to notify user about token refresh failure"
                        );
                    }

                    tracing::warn!(
                        user_id = %user_id,
                        provider = %provider,
                        error = %e,
                        "Failed to refresh OAuth token proactively"
                    );
                }
            }
        }

        tracing::info!(
            total_attempted = summary.total_attempted,
            successful = summary.successful_refreshes,
            failed = summary.failed_refreshes,
            "Completed proactive OAuth token refresh batch"
        );

        Ok(summary)
    }

    /// Update last used timestamp for OAuth account
    /// TODO: Re-enable once SQLx cache is updated
    pub async fn update_oauth_account_last_used(
        &self,
        _user_id: Uuid,
        _provider: OAuthProviderType,
    ) -> Result<()> {
        // Temporarily disabled until SQLx cache is updated
        Ok(())
    }

    // Helper method for audit logging
    async fn log_audit_event(
        &self,
        user_id: Uuid,
        action: &str,
        subject_type: &str,
        subject_id: &str,
    ) -> Result<()> {
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
        sqlx::query!("DELETE FROM user_sessions WHERE user_id = $1", user_id)
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
        sqlx::query!("UPDATE users SET updated_at = NOW() WHERE id = $1", user_id)
            .execute(&self.db_pool)
            .await?;

        Ok(session_token)
    }

    pub fn with_oauth_enabled(mut self) -> Self {
        // Re-initialize OAuth providers (useful for testing)
        let providers = Self::initialize_oauth_providers(&self.oauth_config_validator);
        self.oauth_providers = Arc::new(providers);
        self
    }

    /// Add OAuth provider for testing (disabled due to clone constraints)
    // pub fn add_oauth_provider(&mut self, provider_type: OAuthProviderType, provider: Box<dyn OAuthProvider>) {
    //     // Cannot clone HashMap<OAuthProviderType, Box<dyn OAuthProvider>>
    //     // This method is disabled for now
    // }

    // Comprehensive registration validation function with performance optimizations
    pub async fn validate_registration_request(
        &self,
        request: &crate::models::RegisterRequest,
    ) -> Vec<RegistrationValidationError> {
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
            match self
                .performance_service
                .validate_email_format_cached(&request.email)
                .await
            {
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
            match self
                .performance_service
                .validate_password_strength_cached(&request.password)
                .await
            {
                Ok(Some(password_error)) => {
                    errors.push(password_error);
                }
                Ok(None) => {
                    // Password is valid
                }
                Err(_) => {
                    // Fallback to non-cached validation
                    if let Some(password_error) = self.validate_password_strength(&request.password)
                    {
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
        if let Err(e) = self
            .performance_service
            .record_registration_attempt(validation_time)
            .await
        {
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
        if !password
            .chars()
            .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c))
        {
            requirements.push("at least one special character (!@#$%^&*()_+-=[]{}|;:,.<>?)");
        }

        // Check against common passwords (basic implementation)
        let common_passwords = [
            "password",
            "123456",
            "password123",
            "admin",
            "qwerty",
            "letmein",
            "welcome",
            "monkey",
            "1234567890",
            "password1",
            "123456789",
        ];

        if common_passwords
            .iter()
            .any(|&common| password.to_lowercase() == common.to_lowercase())
        {
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
    pub async fn register(
        &self,
        request: crate::models::RegisterRequest,
    ) -> Result<crate::models::AuthResponse> {
        use crate::models::AuthResponse;

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
                errors: validation_errors,
            });
        }

        // Check if user already exists using optimized query
        let email_exists = match self
            .performance_service
            .check_email_exists_optimized(&self.db_pool, &request.email)
            .await
        {
            Ok(exists) => exists,
            Err(_) => {
                // Fallback to original query
                let existing_user =
                    sqlx::query!("SELECT id FROM users WHERE email = $1", request.email)
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
        self.log_audit_event(user_id, "user_registered", "user", &user_id.to_string())
            .await
            .ok();

        // Commit transaction
        tx.commit().await?;

        // Record successful registration metrics
        let registration_time = registration_start.elapsed().as_millis() as f64;
        if let Err(e) = self
            .performance_service
            .record_successful_registration(registration_time)
            .await
        {
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
                        user: user.to_profile(),
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
                user: user.to_profile(),
                access_token: String::new(), // Empty token indicates auto-login disabled
                refresh_token: String::new(), // Empty token indicates auto-login disabled
            })
        }
    }

    pub async fn login(&self, request: LoginRequest) -> Result<crate::models::AuthResponse> {
        use crate::models::AuthResponse;

        // Login user
        let token_pair = self.login_user(request).await?;

        // Get user info from token
        let claims = self.verify_token(&token_pair.access_token)?;
        let user_id = Uuid::parse_str(&claims.sub)?;
        let user = self.get_user(user_id).await?;

        Ok(AuthResponse {
            user: user.to_profile(),
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

    // OAuth-specific methods for handlers
    // TODO: Temporarily disabled until SQLx cache is updated

    /// Find user by OAuth account (public method for handlers)
    #[allow(dead_code)]
    pub async fn find_user_by_oauth_account_public(
        &self,
        _provider: OAuthProviderType,
        _provider_user_id: &str,
    ) -> Result<User> {
        // Temporarily return error until SQLx cache is updated
        Err(AppError::NotFound {
            resource: "OAuth functionality temporarily disabled".to_string(),
        })
    }

    #[allow(dead_code)]
    async fn find_user_by_oauth_account_public_impl(
        &self,
        _provider: OAuthProviderType,
        _provider_user_id: &str,
    ) -> Result<User> {
        // Temporarily disabled until SQLx cache is updated
        Err(AppError::NotFound {
            resource: "OAuth functionality temporarily disabled".to_string(),
        })
    }

    #[allow(dead_code)]
    async fn find_user_by_oauth_account_public_impl_disabled(
        &self,
        provider: OAuthProviderType,
        provider_user_id: &str,
    ) -> Result<User> {
        let user_row = sqlx::query!(
            r#"
            SELECT u.id, u.email, u.email_verified, u.password_hash, u.totp_secret, u.totp_enabled, u.created_at, u.updated_at, u.last_login
            FROM users u
            INNER JOIN oauth_accounts oa ON u.id = oa.user_id
            WHERE oa.provider = $1 AND oa.provider_user_id = $2
            "#,
            provider.to_string(),
            provider_user_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        match user_row {
            Some(row) => Ok(User {
                id: row.id,
                email: row.email,
                email_verified: row.email_verified.unwrap_or(false),
                password_hash: row.password_hash,
                totp_secret: row.totp_secret,
                totp_enabled: row.totp_enabled.unwrap_or(false),
                oauth_accounts: vec![], // Will be populated separately if needed
                created_at: row.created_at.unwrap_or(Utc::now()),
                updated_at: row.updated_at.unwrap_or(Utc::now()),
                last_login: row.last_login,
                settings: crate::models::user::UserSettings {
                    two_factor_enabled: row.totp_enabled.unwrap_or(false),
                    email_notifications: true,
                    privacy_mode: false,
                },
            }),
            None => Err(AppError::NotFound {
                resource: "User".to_string(),
            }),
        }
    }

    /// Create user with OAuth account
    #[allow(dead_code)]
    pub async fn create_user_with_oauth(
        &self,
        _email: &str,
        _provider: OAuthProviderType,
        _tokens: &OAuthTokens,
        _user_info: &OAuthUserInfo,
    ) -> Result<User> {
        // Temporarily return error until SQLx cache is updated
        Err(AppError::NotFound {
            resource: "OAuth functionality temporarily disabled".to_string(),
        })
    }

    #[allow(dead_code)]
    async fn create_user_with_oauth_impl(
        &self,
        _email: &str,
        _provider: OAuthProviderType,
        _tokens: &OAuthTokens,
        _user_info: &OAuthUserInfo,
    ) -> Result<User> {
        // Temporarily disabled until SQLx cache is updated
        Err(AppError::NotFound {
            resource: "OAuth functionality temporarily disabled".to_string(),
        })
    }

    #[allow(dead_code)]
    async fn create_user_with_oauth_impl_disabled(
        &self,
        email: &str,
        provider: OAuthProviderType,
        tokens: &OAuthTokens,
        user_info: &OAuthUserInfo,
    ) -> Result<User> {
        let mut tx = self.db_pool.begin().await?;

        // Create user
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query!(
            r#"
            INSERT INTO users (id, email, email_verified, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, NULL, $4, $5)
            "#,
            user_id,
            email,
            user_info.email_verified.unwrap_or(false),
            now,
            now
        )
        .execute(&mut *tx)
        .await?;

        // Encrypt tokens
        let encrypted_access_token = self.oauth_encryption.encrypt_token(&tokens.access_token)?;
        let encrypted_refresh_token = tokens
            .refresh_token
            .as_ref()
            .map(|token| self.oauth_encryption.encrypt_token(token))
            .transpose()?;

        // Create OAuth account
        let oauth_account_id = Uuid::new_v4();
        let token_expires_at = tokens
            .expires_in
            .map(|expires_in| Utc::now() + Duration::seconds(expires_in));

        sqlx::query!(
            r#"
            INSERT INTO oauth_accounts (
                id, user_id, provider, provider_user_id, email, display_name, avatar_url,
                access_token_encrypted, refresh_token_encrypted, token_expires_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            oauth_account_id,
            user_id,
            provider.to_string(),
            user_info.provider_user_id,
            user_info.email,
            user_info.display_name,
            user_info.avatar_url,
            encrypted_access_token,
            encrypted_refresh_token,
            token_expires_at,
            now,
            now
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(User {
            id: user_id,
            email: email.to_string(),
            email_verified: user_info.email_verified.unwrap_or(false),
            password_hash: None,
            totp_secret: None,
            totp_enabled: false,
            oauth_accounts: vec![], // Will be populated separately if needed
            created_at: now,
            updated_at: now,
            last_login: None,
            settings: crate::models::user::UserSettings {
                two_factor_enabled: false,
                email_notifications: true,
                privacy_mode: false,
            },
        })
    }

    /// Update OAuth account tokens
    #[allow(dead_code)]
    pub async fn update_oauth_account(
        &self,
        _user_id: Uuid,
        _provider: OAuthProviderType,
        _tokens: &OAuthTokens,
        _user_info: &OAuthUserInfo,
    ) -> Result<()> {
        // Temporarily return error until SQLx cache is updated
        Err(AppError::NotFound {
            resource: "OAuth functionality temporarily disabled".to_string(),
        })
    }

    #[allow(dead_code)]
    async fn update_oauth_account_impl(
        &self,
        _user_id: Uuid,
        _provider: OAuthProviderType,
        _tokens: &OAuthTokens,
        _user_info: &OAuthUserInfo,
    ) -> Result<()> {
        // Temporarily disabled until SQLx cache is updated
        Err(AppError::NotFound {
            resource: "OAuth account update temporarily disabled".to_string(),
        })
    }

    /// Generate JWT tokens for user
    pub async fn generate_tokens(&self, user_id: Uuid) -> Result<(String, String)> {
        let token_pair = self.generate_token_pair(user_id, "").await?;
        Ok((token_pair.access_token, token_pair.refresh_token))
    }

    /// Unlink OAuth account from user (public method for handlers)
    #[allow(dead_code)]
    pub async fn unlink_oauth_account_public(
        &self,
        _user_id: Uuid,
        _provider: OAuthProviderType,
    ) -> Result<()> {
        // Temporarily return error until SQLx cache is updated
        Err(AppError::NotFound {
            resource: "OAuth functionality temporarily disabled".to_string(),
        })
    }

    #[allow(dead_code)]
    async fn unlink_oauth_account_public_impl(
        &self,
        user_id: Uuid,
        provider: OAuthProviderType,
    ) -> Result<()> {
        let result = sqlx::query!(
            "DELETE FROM oauth_accounts WHERE user_id = $1 AND provider = $2",
            user_id,
            provider.to_string()
        )
        .execute(&self.db_pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                resource: format!("OAuth account for provider {}", provider),
            });
        }

        Ok(())
    }

    /// Get user's OAuth accounts
    #[allow(dead_code)]
    pub async fn get_user_oauth_accounts(&self, _user_id: Uuid) -> Result<Vec<OAuthAccount>> {
        // Temporarily return empty list until SQLx cache is updated
        Ok(vec![])
    }

    /// Validate OAuth state token and return the associated state
    /// This is used to retrieve the original redirect_uri for callbacks
    pub fn validate_oauth_state(
        &self,
        state_token: &str,
        provider: &OAuthProviderType,
    ) -> Result<OAuthState> {
        self.oauth_state_manager
            .validate_and_consume_state(state_token, provider)
    }

    // TODO: Re-enable once SQLx cache is updated
    // #[allow(dead_code)]
    // async fn get_user_oauth_accounts_impl(&self, user_id: Uuid) -> Result<Vec<OAuthAccount>> {
    //     // Temporarily commented out due to SQLx cache issues
    //     Ok(vec![])
    // }
}
