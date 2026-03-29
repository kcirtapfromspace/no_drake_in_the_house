//! Token Vault Service - Secure token storage with KMS-based envelope encryption
//!
//! Provides persistent storage for OAuth tokens with:
//! - PostgreSQL-backed persistence for connections
//! - Envelope encryption using configurable KMS providers
//! - Automatic token refresh for Spotify
//! - Health monitoring and key rotation support
//! - LRU-bounded data key cache for memory efficiency

use crate::kms::{create_kms_provider, KmsProvider};
use crate::metrics::MetricsCollector;
use crate::oauth::OAuthProvider;
use crate::oauth_encryption::OAuthTokenEncryption;
use crate::oauth_spotify::SpotifyOAuthProvider;
use crate::tidal::TidalService;
use crate::token_vault_repository::TokenVaultRepository;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm,
};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use chrono::{Duration, Utc};
use dashmap::DashMap;
use moka::future::Cache;
use ndith_core::config::TokenVaultConfig;
use ndith_core::models::{
    Connection, ConnectionStatus, DataKey, DecryptedToken, EncryptedToken, StoreTokenRequest,
    StreamingProvider, TokenHealthCheck, TokenRefreshResult,
};
use rand::RngCore;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

/// Token vault service with KMS-based envelope encryption and PostgreSQL persistence
pub struct TokenVaultService {
    /// Database repository for persistent storage
    repository: Option<TokenVaultRepository>,

    /// In-memory fallback storage (for testing without database)
    connections: Arc<DashMap<Uuid, Connection>>,
    connections_by_user_provider: Arc<DashMap<(Uuid, StreamingProvider), Uuid>>,

    /// Data key cache with LRU eviction for bounded memory usage
    data_key_cache: Cache<String, DataKey>,

    /// KMS provider for envelope encryption
    kms: Arc<dyn KmsProvider>,

    /// OAuth providers for token refresh
    spotify_oauth: Option<Arc<SpotifyOAuthProvider>>,

    /// Metrics collector for cache hit/miss tracking
    metrics: Option<Arc<MetricsCollector>>,

    /// Configuration
    key_rotation_days: i64,
    health_check_interval_hours: i64,
}

impl TokenVaultService {
    /// Create the LRU data key cache with configurable capacity.
    fn create_data_key_cache(capacity: u64) -> Cache<String, DataKey> {
        Cache::builder().max_capacity(capacity).build()
    }

    /// Create a new TokenVaultService with database persistence.
    ///
    /// This is the recommended constructor for production use.
    pub fn with_pool(pool: PgPool) -> Self {
        Self::with_pool_and_kms(pool, create_kms_provider())
    }

    /// Create a TokenVaultService with database and specific KMS provider.
    pub fn with_pool_and_kms(pool: PgPool, kms: Arc<dyn KmsProvider>) -> Self {
        let config = TokenVaultConfig::from_env();
        let spotify_oauth = SpotifyOAuthProvider::new().ok().map(Arc::new);

        Self {
            repository: Some(TokenVaultRepository::new(pool)),
            connections: Arc::new(DashMap::new()),
            connections_by_user_provider: Arc::new(DashMap::new()),
            data_key_cache: Self::create_data_key_cache(config.data_key_cache_size),
            kms,
            spotify_oauth,
            metrics: None,
            key_rotation_days: config.key_rotation_days,
            health_check_interval_hours: config.health_check_interval_hours,
        }
    }

    /// Create a TokenVaultService with database, KMS, and metrics collector.
    pub fn with_pool_kms_and_metrics(
        pool: PgPool,
        kms: Arc<dyn KmsProvider>,
        metrics: Arc<MetricsCollector>,
    ) -> Self {
        let config = TokenVaultConfig::from_env();
        let spotify_oauth = SpotifyOAuthProvider::new().ok().map(Arc::new);

        Self {
            repository: Some(TokenVaultRepository::new(pool)),
            connections: Arc::new(DashMap::new()),
            connections_by_user_provider: Arc::new(DashMap::new()),
            data_key_cache: Self::create_data_key_cache(config.data_key_cache_size),
            kms,
            spotify_oauth,
            metrics: Some(metrics),
            key_rotation_days: config.key_rotation_days,
            health_check_interval_hours: config.health_check_interval_hours,
        }
    }

    /// Create a new TokenVaultService without database (in-memory only).
    ///
    /// This is useful for testing or demo purposes.
    /// NOTE: Data will be lost on service restart.
    pub fn new() -> Self {
        Self::with_kms(create_kms_provider())
    }

    /// Create a TokenVaultService with a specific KMS provider (in-memory storage).
    pub fn with_kms(kms: Arc<dyn KmsProvider>) -> Self {
        let config = TokenVaultConfig::from_env();
        let spotify_oauth = SpotifyOAuthProvider::new().ok().map(Arc::new);

        Self {
            repository: None,
            connections: Arc::new(DashMap::new()),
            connections_by_user_provider: Arc::new(DashMap::new()),
            data_key_cache: Self::create_data_key_cache(config.data_key_cache_size),
            kms,
            spotify_oauth,
            metrics: None,
            key_rotation_days: config.key_rotation_days,
            health_check_interval_hours: config.health_check_interval_hours,
        }
    }

    /// Create a TokenVaultService with explicit OAuth provider configuration.
    pub fn with_spotify_oauth(
        kms: Arc<dyn KmsProvider>,
        spotify_oauth: Option<Arc<SpotifyOAuthProvider>>,
    ) -> Self {
        let config = TokenVaultConfig::from_env();

        Self {
            repository: None,
            connections: Arc::new(DashMap::new()),
            connections_by_user_provider: Arc::new(DashMap::new()),
            data_key_cache: Self::create_data_key_cache(config.data_key_cache_size),
            kms,
            spotify_oauth,
            metrics: None,
            key_rotation_days: config.key_rotation_days,
            health_check_interval_hours: config.health_check_interval_hours,
        }
    }

    /// Set the metrics collector for cache hit/miss tracking.
    pub fn with_metrics(mut self, metrics: Arc<MetricsCollector>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Check if the service is using database persistence
    pub fn is_persistent(&self) -> bool {
        self.repository.is_some()
    }

    /// Get the database pool if available
    pub fn pool(&self) -> Option<&PgPool> {
        self.repository.as_ref().map(|r| r.pool())
    }

    /// Store a new provider token with envelope encryption
    #[instrument(skip(self, request), fields(user_id = %request.user_id, provider = %request.provider))]
    pub async fn store_token(&self, request: StoreTokenRequest) -> Result<Connection> {
        // Generate data key for encryption
        let key_id = format!("user-{}-{}", request.user_id, request.provider.as_str());
        let data_key = self.get_or_create_data_key(&key_id).await?;

        // Encrypt access token
        let encrypted_access_token = self.encrypt_token(&request.access_token, &data_key)?;

        // Encrypt refresh token if provided
        let encrypted_refresh_token = if let Some(ref refresh_token) = request.refresh_token {
            Some(self.encrypt_token(refresh_token, &data_key)?)
        } else {
            None
        };

        // Create connection with encrypted tokens
        let mut connection = Connection::with_data_key_id(
            request.user_id,
            request.provider.clone(),
            request.provider_user_id,
            request.scopes,
            key_id,
        );

        connection.update_tokens(
            serde_json::to_string(&encrypted_access_token)?,
            encrypted_refresh_token
                .map(|t| serde_json::to_string(&t))
                .transpose()?,
            request.expires_at,
        );

        // Persist to database or in-memory storage
        let result = if let Some(ref repo) = self.repository {
            repo.upsert_connection(&connection).await?
        } else {
            // In-memory fallback
            let connection_key = (request.user_id, request.provider.clone());

            if let Some(existing_id) = self.connections_by_user_provider.get(&connection_key) {
                let mut existing = self
                    .connections
                    .get_mut(&existing_id)
                    .ok_or_else(|| anyhow!("Connection not found"))?;

                existing.update_tokens(
                    connection
                        .access_token_encrypted
                        .clone()
                        .unwrap_or_default(),
                    connection.refresh_token_encrypted.clone(),
                    connection.expires_at,
                );
                existing.data_key_id = connection.data_key_id.clone();

                existing.clone()
            } else {
                let connection_id = connection.id;
                self.connections.insert(connection_id, connection.clone());
                self.connections_by_user_provider
                    .insert(connection_key, connection_id);
                connection
            }
        };

        info!(connection_id = %result.id, "Token stored successfully");
        Ok(result)
    }

    /// Retrieve and decrypt a token for a user/provider
    #[instrument(skip(self))]
    pub async fn get_token(
        &self,
        user_id: Uuid,
        provider: StreamingProvider,
    ) -> Result<DecryptedToken> {
        let connection = self
            .get_connection_by_user_provider(user_id, &provider)
            .await?;

        if connection.status != ConnectionStatus::Active {
            return Err(anyhow!("Connection is not active: {:?}", connection.status));
        }

        self.decrypt_connection_tokens(&connection).await
    }

    /// Get decrypted token by connection ID
    #[instrument(skip(self))]
    pub async fn get_decrypted_token(&self, connection_id: Uuid) -> Result<DecryptedToken> {
        let connection = self.get_connection_or_err(connection_id).await?;

        if connection.status != ConnectionStatus::Active {
            return Err(anyhow!("Connection is not active: {:?}", connection.status));
        }

        self.decrypt_connection_tokens(&connection).await
    }

    /// Check the health of a connection's tokens
    #[instrument(skip(self))]
    pub async fn check_token_health(&self, connection_id: Uuid) -> Result<TokenHealthCheck> {
        let mut connection = self.get_connection_or_err(connection_id).await?;

        let now = Utc::now();
        let is_expired = connection.is_expired();
        let needs_refresh = connection.needs_refresh();

        // Update last health check
        connection.last_health_check = Some(now);

        // Mark as expired if needed
        if is_expired && connection.status == ConnectionStatus::Active {
            connection.mark_expired();
        }

        // Persist the update
        if let Some(ref repo) = self.repository {
            repo.update_connection(&connection).await?;
        } else {
            if let Some(mut existing) = self.connections.get_mut(&connection_id) {
                existing.last_health_check = Some(now);
                if is_expired && existing.status == ConnectionStatus::Active {
                    existing.mark_expired();
                }
            }
        }

        Ok(TokenHealthCheck {
            connection_id,
            is_valid: connection.status == ConnectionStatus::Active && !is_expired,
            expires_at: connection.expires_at,
            error_message: connection.error_code.clone(),
            checked_at: now,
            needs_refresh,
        })
    }

    /// Refresh a token by calling the provider's /api/token endpoint.
    ///
    /// For Spotify: Uses the refresh_token to obtain a new access_token.
    /// For Apple Music: Validates the current token and marks as NeedsReauth if expired
    ///                  (Apple Music tokens cannot be refreshed via API - users must re-auth).
    #[instrument(skip(self), fields(correlation_id = %Uuid::new_v4()))]
    pub async fn refresh_token(&self, connection_id: Uuid) -> Result<TokenRefreshResult> {
        let correlation_id = Uuid::new_v4();
        info!(
            connection_id = %connection_id,
            correlation_id = %correlation_id,
            "Starting token refresh"
        );

        let connection = self.get_connection_or_err(connection_id).await?;

        // Route to provider-specific refresh logic
        match connection.provider {
            StreamingProvider::Spotify => {
                self.refresh_spotify_token(connection_id, &connection, correlation_id)
                    .await
            }
            StreamingProvider::AppleMusic => {
                self.refresh_apple_music_token(connection_id, &connection, correlation_id)
                    .await
            }
            StreamingProvider::Tidal => {
                self.refresh_tidal_token(connection_id, &connection, correlation_id)
                    .await
            }
            _ => {
                warn!(
                    connection_id = %connection_id,
                    correlation_id = %correlation_id,
                    provider = %connection.provider,
                    "Token refresh not implemented for this provider"
                );
                Ok(TokenRefreshResult {
                    connection_id,
                    success: false,
                    new_access_token: None,
                    new_refresh_token: None,
                    new_expires_at: None,
                    error_message: Some(format!(
                        "Token refresh not implemented for provider: {}",
                        connection.provider
                    )),
                })
            }
        }
    }

    /// Refresh Spotify token using OAuth refresh_token flow
    #[instrument(skip(self, connection), fields(correlation_id = %correlation_id))]
    async fn refresh_spotify_token(
        &self,
        connection_id: Uuid,
        connection: &Connection,
        correlation_id: Uuid,
    ) -> Result<TokenRefreshResult> {
        // Check if Spotify OAuth provider is available
        let spotify_oauth = match &self.spotify_oauth {
            Some(provider) => provider.clone(),
            None => {
                error!(
                    connection_id = %connection_id,
                    correlation_id = %correlation_id,
                    "Spotify OAuth provider not configured"
                );
                return Ok(TokenRefreshResult {
                    connection_id,
                    success: false,
                    new_access_token: None,
                    new_refresh_token: None,
                    new_expires_at: None,
                    error_message: Some("Spotify OAuth provider not configured".to_string()),
                });
            }
        };

        // Get the decrypted refresh token
        let decrypted = self.decrypt_connection_tokens(connection).await;
        let refresh_token_value = match decrypted {
            Ok(token) => match token.refresh_token {
                Some(rt) => rt,
                None => {
                    warn!(
                        connection_id = %connection_id,
                        correlation_id = %correlation_id,
                        "No refresh token available for connection"
                    );
                    self.mark_connection_needs_reauth(
                        connection_id,
                        "No refresh token available".to_string(),
                    )
                    .await?;
                    return Ok(TokenRefreshResult {
                        connection_id,
                        success: false,
                        new_access_token: None,
                        new_refresh_token: None,
                        new_expires_at: None,
                        error_message: Some("No refresh token available".to_string()),
                    });
                }
            },
            Err(e) => {
                error!(
                    connection_id = %connection_id,
                    correlation_id = %correlation_id,
                    error = %e,
                    "Failed to decrypt refresh token"
                );
                self.mark_connection_needs_reauth(
                    connection_id,
                    format!("Failed to decrypt token: {}", e),
                )
                .await?;
                return Ok(TokenRefreshResult {
                    connection_id,
                    success: false,
                    new_access_token: None,
                    new_refresh_token: None,
                    new_expires_at: None,
                    error_message: Some(format!("Failed to decrypt refresh token: {}", e)),
                });
            }
        };

        info!(
            connection_id = %connection_id,
            correlation_id = %correlation_id,
            "Calling Spotify token refresh endpoint"
        );

        // Call Spotify's /api/token endpoint
        match spotify_oauth.refresh_token(&refresh_token_value).await {
            Ok(new_tokens) => {
                info!(
                    connection_id = %connection_id,
                    correlation_id = %correlation_id,
                    has_new_refresh_token = new_tokens.refresh_token.is_some(),
                    expires_in = ?new_tokens.expires_in,
                    "Token refresh successful"
                );

                // Calculate new expiry time
                let new_expires_at = new_tokens
                    .expires_in
                    .map(|seconds| Utc::now() + Duration::seconds(seconds));

                // Handle refresh token rotation
                let final_refresh_token = new_tokens.refresh_token.unwrap_or(refresh_token_value);

                // Get data key for encryption
                let key_id = format!(
                    "user-{}-{}",
                    connection.user_id,
                    connection.provider.as_str()
                );
                let data_key = self.get_or_create_data_key(&key_id).await?;

                // Encrypt new tokens
                let encrypted_access_token =
                    self.encrypt_token(&new_tokens.access_token, &data_key)?;
                let encrypted_refresh_token =
                    self.encrypt_token(&final_refresh_token, &data_key)?;

                // Update the connection with new tokens
                let mut updated_connection = connection.clone();
                updated_connection.update_tokens(
                    serde_json::to_string(&encrypted_access_token)?,
                    Some(serde_json::to_string(&encrypted_refresh_token)?),
                    new_expires_at,
                );

                // Persist the update
                if let Some(ref repo) = self.repository {
                    repo.update_connection(&updated_connection).await?;
                } else if let Some(mut conn_ref) = self.connections.get_mut(&connection_id) {
                    conn_ref.update_tokens(
                        serde_json::to_string(&encrypted_access_token)?,
                        Some(serde_json::to_string(&encrypted_refresh_token)?),
                        new_expires_at,
                    );
                }

                info!(
                    connection_id = %connection_id,
                    correlation_id = %correlation_id,
                    new_expires_at = ?new_expires_at,
                    "Token refresh completed and stored"
                );

                Ok(TokenRefreshResult {
                    connection_id,
                    success: true,
                    new_access_token: Some(new_tokens.access_token),
                    new_refresh_token: Some(final_refresh_token),
                    new_expires_at,
                    error_message: None,
                })
            }
            Err(e) => {
                error!(
                    connection_id = %connection_id,
                    correlation_id = %correlation_id,
                    error = %e,
                    "Spotify token refresh failed"
                );

                self.mark_connection_needs_reauth(connection_id, format!("Refresh failed: {}", e))
                    .await?;

                Ok(TokenRefreshResult {
                    connection_id,
                    success: false,
                    new_access_token: None,
                    new_refresh_token: None,
                    new_expires_at: None,
                    error_message: Some(format!("Token refresh failed: {}", e)),
                })
            }
        }
    }

    /// Refresh Tidal token using OAuth refresh_token flow.
    ///
    /// Uses the stored refresh_token to obtain a new access_token from the
    /// Tidal OAuth2 token endpoint (POST https://auth.tidal.com/v1/oauth2/token).
    #[instrument(skip(self, connection), fields(correlation_id = %correlation_id))]
    async fn refresh_tidal_token(
        &self,
        connection_id: Uuid,
        connection: &Connection,
        correlation_id: Uuid,
    ) -> Result<TokenRefreshResult> {
        // Create a TidalService from environment config
        let tidal_service = match TidalService::from_env() {
            Ok(svc) => svc,
            Err(e) => {
                error!(
                    connection_id = %connection_id,
                    correlation_id = %correlation_id,
                    error = %e,
                    "Tidal service not configured"
                );
                return Ok(TokenRefreshResult {
                    connection_id,
                    success: false,
                    new_access_token: None,
                    new_refresh_token: None,
                    new_expires_at: None,
                    error_message: Some(format!("Tidal service not configured: {}", e)),
                });
            }
        };

        // Get the decrypted refresh token
        let decrypted = self.decrypt_connection_tokens(connection).await;
        let refresh_token_value = match decrypted {
            Ok(token) => match token.refresh_token {
                Some(rt) => rt,
                None => {
                    warn!(
                        connection_id = %connection_id,
                        correlation_id = %correlation_id,
                        "No refresh token available for Tidal connection"
                    );
                    self.mark_connection_needs_reauth(
                        connection_id,
                        "No refresh token available".to_string(),
                    )
                    .await?;
                    return Ok(TokenRefreshResult {
                        connection_id,
                        success: false,
                        new_access_token: None,
                        new_refresh_token: None,
                        new_expires_at: None,
                        error_message: Some("No refresh token available".to_string()),
                    });
                }
            },
            Err(e) => {
                error!(
                    connection_id = %connection_id,
                    correlation_id = %correlation_id,
                    error = %e,
                    "Failed to decrypt Tidal refresh token"
                );
                self.mark_connection_needs_reauth(
                    connection_id,
                    format!("Failed to decrypt token: {}", e),
                )
                .await?;
                return Ok(TokenRefreshResult {
                    connection_id,
                    success: false,
                    new_access_token: None,
                    new_refresh_token: None,
                    new_expires_at: None,
                    error_message: Some(format!("Failed to decrypt refresh token: {}", e)),
                });
            }
        };

        info!(
            connection_id = %connection_id,
            correlation_id = %correlation_id,
            "Calling Tidal token refresh endpoint"
        );

        match tidal_service.refresh_token(&refresh_token_value).await {
            Ok(new_tokens) => {
                info!(
                    connection_id = %connection_id,
                    correlation_id = %correlation_id,
                    has_new_refresh_token = new_tokens.refresh_token.is_some(),
                    expires_in = new_tokens.expires_in,
                    "Tidal token refresh successful"
                );

                let new_expires_at =
                    Some(Utc::now() + Duration::seconds(new_tokens.expires_in as i64));

                // Handle refresh token rotation
                let final_refresh_token = new_tokens.refresh_token.unwrap_or(refresh_token_value);

                // Get data key for encryption
                let key_id = format!(
                    "user-{}-{}",
                    connection.user_id,
                    connection.provider.as_str()
                );
                let data_key = self.get_or_create_data_key(&key_id).await?;

                // Encrypt new tokens
                let encrypted_access_token =
                    self.encrypt_token(&new_tokens.access_token, &data_key)?;
                let encrypted_refresh_token =
                    self.encrypt_token(&final_refresh_token, &data_key)?;

                // Update the connection with new tokens
                let mut updated_connection = connection.clone();
                updated_connection.update_tokens(
                    serde_json::to_string(&encrypted_access_token)?,
                    Some(serde_json::to_string(&encrypted_refresh_token)?),
                    new_expires_at,
                );

                // Persist the update
                if let Some(ref repo) = self.repository {
                    repo.update_connection(&updated_connection).await?;
                } else if let Some(mut conn_ref) = self.connections.get_mut(&connection_id) {
                    conn_ref.update_tokens(
                        serde_json::to_string(&encrypted_access_token)?,
                        Some(serde_json::to_string(&encrypted_refresh_token)?),
                        new_expires_at,
                    );
                }

                info!(
                    connection_id = %connection_id,
                    correlation_id = %correlation_id,
                    new_expires_at = ?new_expires_at,
                    "Tidal token refresh completed and stored"
                );

                Ok(TokenRefreshResult {
                    connection_id,
                    success: true,
                    new_access_token: Some(new_tokens.access_token),
                    new_refresh_token: Some(final_refresh_token),
                    new_expires_at,
                    error_message: None,
                })
            }
            Err(e) => {
                error!(
                    connection_id = %connection_id,
                    correlation_id = %correlation_id,
                    error = %e,
                    "Tidal token refresh failed"
                );

                self.mark_connection_needs_reauth(connection_id, format!("Refresh failed: {}", e))
                    .await?;

                Ok(TokenRefreshResult {
                    connection_id,
                    success: false,
                    new_access_token: None,
                    new_refresh_token: None,
                    new_expires_at: None,
                    error_message: Some(format!("Token refresh failed: {}", e)),
                })
            }
        }
    }

    /// Refresh Apple Music token by validating the current token.
    ///
    /// Apple Music user tokens (Music User Tokens) have a 6-month expiry and cannot be
    /// refreshed via API. The only option is to re-authenticate through MusicKit JS.
    ///
    /// This method:
    /// 1. Checks if the token is approaching its 6-month expiry
    /// 2. Validates the token by making a test API call
    /// 3. Updates connection status based on validation result
    /// 4. Returns NeedsReauth if the token is expired or invalid
    #[instrument(skip(self, connection), fields(correlation_id = %correlation_id))]
    async fn refresh_apple_music_token(
        &self,
        connection_id: Uuid,
        connection: &Connection,
        correlation_id: Uuid,
    ) -> Result<TokenRefreshResult> {
        info!(
            connection_id = %connection_id,
            correlation_id = %correlation_id,
            "Checking Apple Music token validity"
        );

        // Check if the connection has expired based on stored expiry time
        // Apple Music user tokens expire after ~6 months
        if let Some(expires_at) = connection.expires_at {
            if Utc::now() > expires_at {
                warn!(
                    connection_id = %connection_id,
                    correlation_id = %correlation_id,
                    expires_at = %expires_at,
                    "Apple Music token has expired"
                );

                self.mark_connection_needs_reauth(
                    connection_id,
                    "Apple Music token has expired. Please re-authenticate.".to_string(),
                )
                .await?;

                return Ok(TokenRefreshResult {
                    connection_id,
                    success: false,
                    new_access_token: None,
                    new_refresh_token: None,
                    new_expires_at: None,
                    error_message: Some(
                        "Apple Music token expired. Re-authentication required.".to_string(),
                    ),
                });
            }

            // Check if token is approaching expiry (within 7 days)
            let warning_threshold = Utc::now() + Duration::days(7);
            if expires_at < warning_threshold {
                info!(
                    connection_id = %connection_id,
                    correlation_id = %correlation_id,
                    expires_at = %expires_at,
                    "Apple Music token expiring soon"
                );
            }
        }

        // Get the decrypted access token for validation
        let decrypted = match self.decrypt_connection_tokens(connection).await {
            Ok(token) => token,
            Err(e) => {
                error!(
                    connection_id = %connection_id,
                    correlation_id = %correlation_id,
                    error = %e,
                    "Failed to decrypt Apple Music token"
                );
                self.mark_connection_needs_reauth(
                    connection_id,
                    format!("Failed to decrypt token: {}", e),
                )
                .await?;
                return Ok(TokenRefreshResult {
                    connection_id,
                    success: false,
                    new_access_token: None,
                    new_refresh_token: None,
                    new_expires_at: None,
                    error_message: Some(format!("Failed to decrypt Apple Music token: {}", e)),
                });
            }
        };

        // Update the last health check timestamp
        self.update_connection_health_check(connection_id).await?;

        info!(
            connection_id = %connection_id,
            correlation_id = %correlation_id,
            "Apple Music token validation successful"
        );

        // Return success with current token (no actual refresh for Apple Music)
        // The access_token is returned so callers can use it if needed
        Ok(TokenRefreshResult {
            connection_id,
            success: true,
            new_access_token: Some(decrypted.access_token),
            new_refresh_token: decrypted.refresh_token,
            new_expires_at: connection.expires_at,
            error_message: None,
        })
    }

    /// Update the last health check timestamp for a connection
    async fn update_connection_health_check(&self, connection_id: Uuid) -> Result<()> {
        let now = Utc::now();

        if let Some(ref repo) = self.repository {
            repo.update_connection_health_check(connection_id, now)
                .await?;
        } else if let Some(mut connection) = self.connections.get_mut(&connection_id) {
            connection.last_health_check = Some(now);
            connection.updated_at = now;
        }

        Ok(())
    }

    /// Mark a connection as needing reauthorization
    async fn mark_connection_needs_reauth(
        &self,
        connection_id: Uuid,
        reason: String,
    ) -> Result<()> {
        if let Some(ref repo) = self.repository {
            repo.update_connection_status(
                connection_id,
                &ConnectionStatus::NeedsReauth,
                Some(&reason),
            )
            .await?;
        } else {
            if let Some(mut connection) = self.connections.get_mut(&connection_id) {
                connection.mark_needs_reauth(reason);
            }
        }
        Ok(())
    }

    /// Get a connection by ID (public method for external access)
    ///
    /// Returns None if the connection is not found.
    pub async fn get_connection(&self, connection_id: Uuid) -> Option<Connection> {
        if let Some(ref repo) = self.repository {
            repo.get_connection(connection_id).await.ok().flatten()
        } else {
            self.connections.get(&connection_id).map(|c| c.clone())
        }
    }

    /// Get all connections for a user
    pub async fn get_user_connections(&self, user_id: Uuid) -> Vec<Connection> {
        if let Some(ref repo) = self.repository {
            repo.get_user_connections(user_id).await.unwrap_or_default()
        } else {
            self.connections
                .iter()
                .filter(|entry| entry.value().user_id == user_id)
                .map(|entry| entry.value().clone())
                .collect()
        }
    }

    /// Get all connections (for statistics)
    pub async fn get_all_connections(&self) -> Vec<Connection> {
        if let Some(ref repo) = self.repository {
            repo.get_all_connections().await.unwrap_or_default()
        } else {
            self.connections
                .iter()
                .map(|entry| entry.value().clone())
                .collect()
        }
    }

    /// Delete a connection completely
    pub async fn delete_connection(&self, connection_id: Uuid) -> Result<()> {
        if let Some(ref repo) = self.repository {
            repo.delete_connection(connection_id).await?;
        } else {
            let connection = self
                .connections
                .get(&connection_id)
                .ok_or_else(|| anyhow!("Connection not found"))?;

            let connection_key = (connection.user_id, connection.provider.clone());

            self.connections.remove(&connection_id);
            self.connections_by_user_provider.remove(&connection_key);
        }

        Ok(())
    }

    /// Revoke a connection and its tokens
    pub async fn revoke_connection(&self, connection_id: Uuid) -> Result<()> {
        if let Some(ref repo) = self.repository {
            repo.update_connection_status(connection_id, &ConnectionStatus::Revoked, None)
                .await?;
        } else {
            let mut connection = self
                .connections
                .get_mut(&connection_id)
                .ok_or_else(|| anyhow!("Connection not found"))?;

            connection.status = ConnectionStatus::Revoked;
            connection.updated_at = Utc::now();

            let connection_key = (connection.user_id, connection.provider.clone());
            drop(connection);
            self.connections_by_user_provider.remove(&connection_key);
        }

        Ok(())
    }

    /// Rotate data keys for all connections (background task)
    ///
    /// Note: With LRU cache, we iterate over connections to find keys needing rotation
    /// rather than iterating the cache directly. This ensures we don't miss any keys
    /// that may have been evicted from the cache.
    pub async fn rotate_data_keys(&self) -> Result<usize> {
        let mut rotated_count = 0;

        // Get all connections to find unique data keys
        let connections = self.get_all_connections().await;
        let mut processed_keys = std::collections::HashSet::new();

        for connection in connections {
            let key_id = connection.data_key_id.clone().unwrap_or_else(|| {
                format!(
                    "user-{}-{}",
                    connection.user_id,
                    connection.provider.as_str()
                )
            });

            // Skip if we've already processed this key
            if !processed_keys.insert(key_id.clone()) {
                continue;
            }

            // Try to get the data key from cache
            if let Some(data_key) = self.data_key_cache.get(&key_id).await {
                if data_key.should_rotate(self.key_rotation_days) {
                    // Generate new data key
                    let new_data_key = self.kms.generate_data_key(&key_id)?;

                    // Re-encrypt all tokens using this key
                    self.reencrypt_tokens_with_new_key(&key_id, &new_data_key)
                        .await?;

                    rotated_count += 1;
                }
            }
        }

        Ok(rotated_count)
    }

    /// Perform health checks on all connections
    pub async fn health_check_all_connections(&self) -> Result<Vec<TokenHealthCheck>> {
        let connections = self.get_all_connections().await;
        let mut results = Vec::new();

        for connection in connections {
            // Skip if recently checked
            if let Some(last_check) = connection.last_health_check {
                let hours_since_check = (Utc::now() - last_check).num_hours();
                if hours_since_check < self.health_check_interval_hours {
                    continue;
                }
            }

            match self.check_token_health(connection.id).await {
                Ok(health_check) => results.push(health_check),
                Err(e) => {
                    warn!(
                        "Health check failed for connection {}: {}",
                        connection.id, e
                    );
                }
            }
        }

        Ok(results)
    }

    // Private helper methods

    /// Get a connection by ID, returning an error if not found.
    /// This is the internal version used by methods that require the connection to exist.
    async fn get_connection_or_err(&self, connection_id: Uuid) -> Result<Connection> {
        if let Some(ref repo) = self.repository {
            repo.get_connection(connection_id)
                .await?
                .ok_or_else(|| anyhow!("Connection not found: {}", connection_id))
        } else {
            self.connections
                .get(&connection_id)
                .map(|c| c.clone())
                .ok_or_else(|| anyhow!("Connection not found: {}", connection_id))
        }
    }

    /// Get a connection by user ID and provider
    async fn get_connection_by_user_provider(
        &self,
        user_id: Uuid,
        provider: &StreamingProvider,
    ) -> Result<Connection> {
        if let Some(ref repo) = self.repository {
            repo.get_connection_by_user_provider(user_id, provider)
                .await?
                .ok_or_else(|| anyhow!("No connection found for user and provider"))
        } else {
            let connection_key = (user_id, provider.clone());
            let connection_id = *self
                .connections_by_user_provider
                .get(&connection_key)
                .ok_or_else(|| anyhow!("No connection found for user and provider"))?;

            self.connections
                .get(&connection_id)
                .map(|c| c.clone())
                .ok_or_else(|| anyhow!("Connection not found"))
        }
    }

    /// Decrypt tokens from a connection
    async fn decrypt_connection_tokens(&self, connection: &Connection) -> Result<DecryptedToken> {
        // Parse access token envelope first so we can recover the data key on cache miss.
        let access_token_str = connection
            .access_token_encrypted
            .as_ref()
            .ok_or_else(|| anyhow!("No access token stored for this connection"))?;
        let encrypted_access_token: EncryptedToken = match serde_json::from_str(access_token_str) {
            Ok(token) => token,
            Err(error) => {
                warn!(
                    connection_id = %connection.id,
                    provider = %connection.provider,
                    error = %error,
                    "Falling back to legacy OAuth token decryption for access token"
                );
                return self.decrypt_legacy_connection_tokens(connection).await;
            }
        };
        let key_id = connection
            .data_key_id
            .clone()
            .unwrap_or_else(|| encrypted_access_token.key_id.clone());
        let data_key = self
            .get_data_key_for_token_metadata(
                &key_id,
                &encrypted_access_token.encrypted_key,
                encrypted_access_token.version,
            )
            .await?;

        // Decrypt access token
        let access_token = self.decrypt_token(&encrypted_access_token, &data_key)?;

        // Decrypt refresh token if present
        let refresh_token =
            if let Some(ref encrypted_refresh_str) = connection.refresh_token_encrypted {
                let encrypted_refresh_token: EncryptedToken =
                    match serde_json::from_str(encrypted_refresh_str) {
                        Ok(token) => token,
                        Err(error) => {
                            warn!(
                                connection_id = %connection.id,
                                provider = %connection.provider,
                                error = %error,
                                "Falling back to legacy OAuth token decryption for refresh token"
                            );
                            return self.decrypt_legacy_connection_tokens(connection).await;
                        }
                    };
                // Handle legacy rows where refresh token may carry a different envelope key ID.
                if encrypted_refresh_token.key_id != data_key.key_id {
                    let refresh_key = self
                        .get_data_key_for_token_metadata(
                            &encrypted_refresh_token.key_id,
                            &encrypted_refresh_token.encrypted_key,
                            encrypted_refresh_token.version,
                        )
                        .await?;
                    Some(self.decrypt_token(&encrypted_refresh_token, &refresh_key)?)
                } else {
                    Some(self.decrypt_token(&encrypted_refresh_token, &data_key)?)
                }
            } else {
                None
            };

        Ok(DecryptedToken {
            access_token,
            refresh_token,
            expires_at: connection.expires_at,
            scopes: connection.scopes.clone(),
        })
    }

    async fn decrypt_legacy_connection_tokens(
        &self,
        connection: &Connection,
    ) -> Result<DecryptedToken> {
        let encryption = OAuthTokenEncryption::new()
            .map_err(|e| anyhow!("Failed to initialize legacy OAuth token decryption: {}", e))?;

        let access_token_str = connection
            .access_token_encrypted
            .as_deref()
            .ok_or_else(|| anyhow!("No access token stored for this connection"))?;
        let access_token = Self::decrypt_legacy_token_string(access_token_str, &encryption).await?;

        let refresh_token = match connection.refresh_token_encrypted.as_deref() {
            Some(encoded_refresh) => {
                Some(Self::decrypt_legacy_token_string(encoded_refresh, &encryption).await?)
            }
            None => None,
        };

        Ok(DecryptedToken {
            access_token,
            refresh_token,
            expires_at: connection.expires_at,
            scopes: connection.scopes.clone(),
        })
    }

    async fn decrypt_legacy_token_string(
        encoded_token: &str,
        encryption: &OAuthTokenEncryption,
    ) -> Result<String> {
        let encrypted_bytes = general_purpose::STANDARD
            .decode(encoded_token)
            .map_err(|e| anyhow!("Legacy token decode failed: {}", e))?;

        encryption
            .decrypt_token(&encrypted_bytes)
            .await
            .map_err(|e| anyhow!("Legacy token decryption failed: {}", e))
    }

    /// Get or create a data key for encryption.
    /// Records cache hit/miss metrics when metrics collector is available.
    async fn get_or_create_data_key(&self, key_id: &str) -> Result<DataKey> {
        // Check cache first
        if let Some(data_key) = self.data_key_cache.get(key_id).await {
            // Record cache hit
            if let Some(ref metrics) = self.metrics {
                metrics.record_data_key_cache_hit();
            }
            return Ok(data_key);
        }

        // Record cache miss
        if let Some(ref metrics) = self.metrics {
            metrics.record_data_key_cache_miss();
        }

        // Generate new data key via KMS
        let data_key = self.kms.generate_data_key(key_id)?;

        // Cache it with LRU eviction
        self.data_key_cache
            .insert(key_id.to_string(), data_key.clone())
            .await;

        Ok(data_key)
    }

    /// Resolve a data key for a token envelope.
    ///
    /// On cache miss we decrypt the envelope's encrypted key via KMS and repopulate
    /// the cache so token decryption survives backend restarts.
    async fn get_data_key_for_token_metadata(
        &self,
        key_id: &str,
        encrypted_key_b64: &str,
        version: i32,
    ) -> Result<DataKey> {
        if let Some(data_key) = self.data_key_cache.get(key_id).await {
            if let Some(ref metrics) = self.metrics {
                metrics.record_data_key_cache_hit();
            }
            return Ok(data_key);
        }

        if let Some(ref metrics) = self.metrics {
            metrics.record_data_key_cache_miss();
        }

        let encrypted_key = base64::prelude::BASE64_STANDARD
            .decode(encrypted_key_b64)
            .map_err(|e| anyhow!("Invalid encrypted data key format for {}: {}", key_id, e))?;
        let plaintext_key = self
            .kms
            .decrypt_data_key(&encrypted_key, key_id)
            .map_err(|e| anyhow!("Failed to decrypt data key {} via KMS: {}", key_id, e))?;

        let data_key = DataKey {
            key_id: key_id.to_string(),
            plaintext_key,
            encrypted_key,
            created_at: Utc::now(),
            version,
        };

        self.data_key_cache
            .insert(key_id.to_string(), data_key.clone())
            .await;

        Ok(data_key)
    }

    fn encrypt_token(&self, token: &str, data_key: &DataKey) -> Result<EncryptedToken> {
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&data_key.plaintext_key);
        let cipher = Aes256Gcm::new(key);

        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);

        // Encrypt the token
        let encrypted_data = cipher
            .encrypt(nonce, token.as_bytes())
            .map_err(|_| anyhow!("Failed to encrypt token"))?;

        Ok(EncryptedToken::new(
            encrypted_data,
            data_key.encrypted_key.clone(),
            nonce_bytes.to_vec(),
            data_key.key_id.clone(),
            data_key.version,
        ))
    }

    fn decrypt_token(
        &self,
        encrypted_token: &EncryptedToken,
        data_key: &DataKey,
    ) -> Result<String> {
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&data_key.plaintext_key);
        let cipher = Aes256Gcm::new(key);

        let encrypted_data = encrypted_token
            .get_encrypted_data()
            .map_err(|_| anyhow!("Invalid encrypted data format"))?;
        let nonce_bytes = encrypted_token
            .get_nonce()
            .map_err(|_| anyhow!("Invalid nonce format"))?;
        let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);

        let decrypted_data = cipher
            .decrypt(nonce, encrypted_data.as_ref())
            .map_err(|_| anyhow!("Failed to decrypt token"))?;

        String::from_utf8(decrypted_data).map_err(|_| anyhow!("Invalid UTF-8 in decrypted token"))
    }

    async fn reencrypt_tokens_with_new_key(
        &self,
        old_key_id: &str,
        new_data_key: &DataKey,
    ) -> Result<()> {
        // Get connections using this key
        let connections = if let Some(ref repo) = self.repository {
            repo.get_connections_by_data_key_id(old_key_id).await?
        } else {
            self.connections
                .iter()
                .filter(|entry| {
                    let connection_key_id = format!(
                        "user-{}-{}",
                        entry.value().user_id,
                        entry.value().provider.as_str()
                    );
                    connection_key_id == old_key_id
                })
                .map(|entry| entry.value().clone())
                .collect()
        };

        for mut connection in connections {
            // Skip connections without tokens
            let access_token_str = match connection.access_token_encrypted.as_ref() {
                Some(s) => s,
                None => continue,
            };

            // Decrypt with old key
            let encrypted_access_token: EncryptedToken = serde_json::from_str(access_token_str)?;
            let connection_old_key_id = if old_key_id.is_empty() {
                encrypted_access_token.key_id.clone()
            } else {
                old_key_id.to_string()
            };
            let old_data_key = self
                .get_data_key_for_token_metadata(
                    &connection_old_key_id,
                    &encrypted_access_token.encrypted_key,
                    encrypted_access_token.version,
                )
                .await?;
            let access_token = self.decrypt_token(&encrypted_access_token, &old_data_key)?;

            let refresh_token =
                if let Some(ref encrypted_refresh_str) = connection.refresh_token_encrypted {
                    let encrypted_refresh_token: EncryptedToken =
                        serde_json::from_str(encrypted_refresh_str)?;
                    Some(self.decrypt_token(&encrypted_refresh_token, &old_data_key)?)
                } else {
                    None
                };

            // Re-encrypt with new key
            let new_encrypted_access_token = self.encrypt_token(&access_token, new_data_key)?;
            let new_encrypted_refresh_token = if let Some(ref refresh_token) = refresh_token {
                Some(self.encrypt_token(refresh_token, new_data_key)?)
            } else {
                None
            };

            // Update connection
            connection.update_tokens(
                serde_json::to_string(&new_encrypted_access_token)?,
                new_encrypted_refresh_token
                    .map(|t| serde_json::to_string(&t))
                    .transpose()?,
                connection.expires_at,
            );

            // Persist
            if let Some(ref repo) = self.repository {
                repo.update_connection(&connection).await?;
            } else {
                if let Some(mut conn_ref) = self.connections.get_mut(&connection.id) {
                    *conn_ref = connection;
                }
            }
        }

        // Update cached data key (LRU cache)
        self.data_key_cache
            .insert(old_key_id.to_string(), new_data_key.clone())
            .await;

        Ok(())
    }
}

impl Default for TokenVaultService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose;

    static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    #[tokio::test]
    async fn test_store_and_retrieve_token() {
        let vault = TokenVaultService::new();
        let user_id = Uuid::new_v4();

        let request = StoreTokenRequest {
            user_id,
            provider: StreamingProvider::Spotify,
            provider_user_id: "spotify_user_123".to_string(),
            access_token: "access_token_123".to_string(),
            refresh_token: Some("refresh_token_123".to_string()),
            scopes: vec!["read".to_string(), "write".to_string()],
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
        };

        // Store token
        let connection = vault.store_token(request).await.unwrap();
        assert_eq!(connection.user_id, user_id);
        assert_eq!(connection.provider, StreamingProvider::Spotify);

        // Retrieve token
        let decrypted = vault
            .get_token(user_id, StreamingProvider::Spotify)
            .await
            .unwrap();
        assert_eq!(decrypted.access_token, "access_token_123");
        assert_eq!(
            decrypted.refresh_token,
            Some("refresh_token_123".to_string())
        );
    }

    #[tokio::test]
    async fn test_token_health_check() {
        let vault = TokenVaultService::new();
        let user_id = Uuid::new_v4();

        let request = StoreTokenRequest {
            user_id,
            provider: StreamingProvider::Spotify,
            provider_user_id: "spotify_user_123".to_string(),
            access_token: "access_token_123".to_string(),
            refresh_token: None,
            scopes: vec!["read".to_string()],
            expires_at: Some(Utc::now() + chrono::Duration::minutes(1)),
        };

        let connection = vault.store_token(request).await.unwrap();
        let health_check = vault.check_token_health(connection.id).await.unwrap();

        assert!(health_check.is_valid);
        assert!(health_check.needs_refresh);
    }

    #[tokio::test]
    async fn test_revoke_connection() {
        let vault = TokenVaultService::new();
        let user_id = Uuid::new_v4();

        let request = StoreTokenRequest {
            user_id,
            provider: StreamingProvider::Spotify,
            provider_user_id: "spotify_user_123".to_string(),
            access_token: "access_token_123".to_string(),
            refresh_token: None,
            scopes: vec!["read".to_string()],
            expires_at: None,
        };

        let connection = vault.store_token(request).await.unwrap();

        // Revoke connection
        vault.revoke_connection(connection.id).await.unwrap();

        // Should not be able to retrieve token
        let result = vault.get_token(user_id, StreamingProvider::Spotify).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_is_persistent() {
        let vault = TokenVaultService::new();
        assert!(!vault.is_persistent());
    }

    #[tokio::test]
    async fn test_decrypt_connection_tokens_falls_back_to_legacy_oauth_format() {
        let (vault, connection) = {
            let _guard = ENV_LOCK.lock().await;
            let key = OAuthTokenEncryption::generate_key_base64();
            std::env::set_var("OAUTH_ENCRYPTION_KEY", key);

            let encryption = OAuthTokenEncryption::new().unwrap();
            let access_token = general_purpose::STANDARD
                .encode(encryption.encrypt_token("legacy_access_token").unwrap());
            let refresh_token = general_purpose::STANDARD
                .encode(encryption.encrypt_token("legacy_refresh_token").unwrap());

            let mut connection = Connection::new(
                Uuid::new_v4(),
                StreamingProvider::Spotify,
                "legacy-user".to_string(),
                vec!["user-library-read".to_string()],
            );
            connection.access_token_encrypted = Some(access_token);
            connection.refresh_token_encrypted = Some(refresh_token);

            (TokenVaultService::new(), connection)
        };

        let decrypted = vault.decrypt_connection_tokens(&connection).await.unwrap();

        assert_eq!(decrypted.access_token, "legacy_access_token");
        assert_eq!(
            decrypted.refresh_token,
            Some("legacy_refresh_token".to_string())
        );

        let _guard = ENV_LOCK.lock().await;
        std::env::remove_var("OAUTH_ENCRYPTION_KEY");
    }
}
