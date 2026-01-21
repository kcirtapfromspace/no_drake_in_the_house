//! Application configuration module
//!
//! Provides centralized, environment-aware configuration with validation.

use std::time::Duration;
use thiserror::Error;

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required configuration: {0}")]
    MissingRequired(String),

    #[error("Invalid configuration value for {key}: {message}")]
    InvalidValue { key: String, message: String },

    #[error("Production requires {0} to be set")]
    ProductionRequired(String),
}

/// Application environment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Environment {
    pub fn from_env() -> Self {
        match std::env::var("ENVIRONMENT")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "production" | "prod" => Self::Production,
            "staging" | "stage" => Self::Staging,
            _ => Self::Development,
        }
    }

    pub fn is_production(&self) -> bool {
        matches!(self, Self::Production)
    }

    pub fn is_development(&self) -> bool {
        matches!(self, Self::Development)
    }
}

/// Complete application configuration
#[derive(Clone)]
pub struct AppConfig {
    pub environment: Environment,
    pub server: ServerConfig,
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
    pub auth: AuthConfig,
    pub oauth: OAuthSettings,
    pub token_refresh: TokenRefreshConfig,
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        let environment = Environment::from_env();

        let config = Self {
            environment,
            server: ServerConfig::from_env(environment)?,
            database: DatabaseSettings::from_env(environment)?,
            redis: RedisSettings::from_env(environment)?,
            auth: AuthConfig::from_env(environment)?,
            oauth: OAuthSettings::from_env(environment)?,
            token_refresh: TokenRefreshConfig::from_env(),
        };

        // Validate production requirements
        if environment.is_production() {
            config.validate_production()?;
        }

        Ok(config)
    }

    /// Validate all production requirements are met
    fn validate_production(&self) -> Result<(), ConfigError> {
        // JWT_SECRET must be set (not default)
        if self.auth.jwt_secret == AuthConfig::default_jwt_secret() {
            return Err(ConfigError::ProductionRequired("JWT_SECRET".to_string()));
        }

        // Database URL must not use localhost
        if self.database.url.contains("localhost") || self.database.url.contains("127.0.0.1") {
            return Err(ConfigError::InvalidValue {
                key: "DATABASE_URL".to_string(),
                message: "Production must not use localhost database".to_string(),
            });
        }

        // Redis URL must not use localhost
        if self.redis.url.contains("localhost") || self.redis.url.contains("127.0.0.1") {
            return Err(ConfigError::InvalidValue {
                key: "REDIS_URL".to_string(),
                message: "Production must not use localhost Redis".to_string(),
            });
        }

        Ok(())
    }
}

/// Server configuration
#[derive(Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub request_timeout: Duration,
}

impl ServerConfig {
    pub fn from_env(_env: Environment) -> Result<Self, ConfigError> {
        Ok(Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            request_timeout: Duration::from_secs(
                std::env::var("REQUEST_TIMEOUT_SECS")
                    .ok()
                    .and_then(|t| t.parse().ok())
                    .unwrap_or(30),
            ),
        })
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Database settings
#[derive(Clone)]
pub struct DatabaseSettings {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
}

impl DatabaseSettings {
    pub fn from_env(env: Environment) -> Result<Self, ConfigError> {
        let default_url = if env.is_development() {
            "postgres://kiro:kiro_dev_password@localhost:5432/kiro_dev".to_string()
        } else {
            return Err(ConfigError::MissingRequired("DATABASE_URL".to_string()));
        };

        Ok(Self {
            url: std::env::var("DATABASE_URL").unwrap_or(default_url),
            max_connections: std::env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|c| c.parse().ok())
                .unwrap_or(if env.is_production() { 20 } else { 10 }),
            connection_timeout: Duration::from_secs(
                std::env::var("DB_CONNECTION_TIMEOUT_SECS")
                    .ok()
                    .and_then(|t| t.parse().ok())
                    .unwrap_or(30),
            ),
            idle_timeout: Duration::from_secs(
                std::env::var("DB_IDLE_TIMEOUT_SECS")
                    .ok()
                    .and_then(|t| t.parse().ok())
                    .unwrap_or(600),
            ),
        })
    }
}

/// Redis settings
#[derive(Clone)]
pub struct RedisSettings {
    pub url: String,
    pub max_size: usize,
    pub timeout: Duration,
}

impl RedisSettings {
    pub fn from_env(env: Environment) -> Result<Self, ConfigError> {
        let default_url = if env.is_development() {
            "redis://localhost:6379".to_string()
        } else {
            return Err(ConfigError::MissingRequired("REDIS_URL".to_string()));
        };

        Ok(Self {
            url: std::env::var("REDIS_URL").unwrap_or(default_url),
            max_size: std::env::var("REDIS_MAX_CONNECTIONS")
                .ok()
                .and_then(|c| c.parse().ok())
                .unwrap_or(10),
            timeout: Duration::from_secs(
                std::env::var("REDIS_TIMEOUT_SECS")
                    .ok()
                    .and_then(|t| t.parse().ok())
                    .unwrap_or(5),
            ),
        })
    }
}

/// Authentication configuration
#[derive(Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub access_token_expiry: Duration,
    pub refresh_token_expiry: Duration,
    pub bcrypt_cost: u32,
}

impl AuthConfig {
    pub fn from_env(env: Environment) -> Result<Self, ConfigError> {
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
            if env.is_development() {
                Self::default_jwt_secret()
            } else {
                String::new()
            }
        });

        if jwt_secret.is_empty() {
            return Err(ConfigError::MissingRequired("JWT_SECRET".to_string()));
        }

        Ok(Self {
            jwt_secret,
            access_token_expiry: Duration::from_secs(
                std::env::var("ACCESS_TOKEN_EXPIRY_SECS")
                    .ok()
                    .and_then(|t| t.parse().ok())
                    .unwrap_or(3600), // 1 hour
            ),
            refresh_token_expiry: Duration::from_secs(
                std::env::var("REFRESH_TOKEN_EXPIRY_SECS")
                    .ok()
                    .and_then(|t| t.parse().ok())
                    .unwrap_or(604800), // 7 days
            ),
            bcrypt_cost: std::env::var("BCRYPT_COST")
                .ok()
                .and_then(|c| c.parse().ok())
                .unwrap_or(12),
        })
    }

    /// Default JWT secret for development only
    pub fn default_jwt_secret() -> String {
        "dev_secret_key_do_not_use_in_production_1234567890".to_string()
    }
}

/// Token refresh background job configuration
#[derive(Clone)]
pub struct TokenRefreshConfig {
    /// Interval between token refresh job runs (in hours)
    pub interval_hours: u64,
    /// Refresh tokens expiring within this many hours
    pub expiry_threshold_hours: u64,
    /// Maximum number of tokens to refresh per batch
    pub batch_size: usize,
    /// Rate limit delay between refresh attempts (in milliseconds)
    pub rate_limit_delay_ms: u64,
    /// Maximum retry attempts for failed refreshes
    pub max_retries: u32,
    /// Base delay for exponential backoff (in seconds)
    pub retry_base_delay_secs: u64,
}

/// Token vault configuration
#[derive(Clone)]
pub struct TokenVaultConfig {
    /// Maximum number of data keys to cache in memory (LRU eviction)
    pub data_key_cache_size: u64,
    /// Number of days before rotating data keys
    pub key_rotation_days: i64,
    /// Hours between health checks
    pub health_check_interval_hours: i64,
}

impl TokenVaultConfig {
    /// Default cache size for data keys
    pub const DEFAULT_CACHE_SIZE: u64 = 10_000;

    pub fn from_env() -> Self {
        Self {
            data_key_cache_size: std::env::var("DATA_KEY_CACHE_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(Self::DEFAULT_CACHE_SIZE),
            key_rotation_days: std::env::var("TOKEN_VAULT_KEY_ROTATION_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            health_check_interval_hours: std::env::var("TOKEN_VAULT_HEALTH_CHECK_INTERVAL_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24),
        }
    }
}

impl Default for TokenVaultConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl TokenRefreshConfig {
    pub fn from_env() -> Self {
        Self {
            interval_hours: std::env::var("TOKEN_REFRESH_INTERVAL_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1), // Default: 1 hour
            expiry_threshold_hours: std::env::var("TOKEN_REFRESH_EXPIRY_THRESHOLD_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24), // Default: 24 hours
            batch_size: std::env::var("TOKEN_REFRESH_BATCH_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(50), // Default: 50 tokens per batch
            rate_limit_delay_ms: std::env::var("TOKEN_REFRESH_RATE_LIMIT_DELAY_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100), // Default: 100ms between refreshes
            max_retries: std::env::var("TOKEN_REFRESH_MAX_RETRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3), // Default: 3 retries
            retry_base_delay_secs: std::env::var("TOKEN_REFRESH_RETRY_BASE_DELAY_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30), // Default: 30 seconds base delay
        }
    }
}

impl Default for TokenRefreshConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

/// OAuth provider settings
#[derive(Clone)]
pub struct OAuthSettings {
    pub google: Option<OAuthProviderConfig>,
    pub apple: Option<AppleOAuthConfig>,
    pub github: Option<OAuthProviderConfig>,
    pub spotify: Option<OAuthProviderConfig>,
}

impl OAuthSettings {
    pub fn from_env(env: Environment) -> Result<Self, ConfigError> {
        Ok(Self {
            google: OAuthProviderConfig::from_env("GOOGLE", env).ok(),
            apple: AppleOAuthConfig::from_env(env).ok(),
            github: OAuthProviderConfig::from_env("GITHUB", env).ok(),
            spotify: OAuthProviderConfig::from_env("SPOTIFY", env).ok(),
        })
    }

    /// Check if any OAuth provider is configured
    pub fn has_any_provider(&self) -> bool {
        self.google.is_some()
            || self.apple.is_some()
            || self.github.is_some()
            || self.spotify.is_some()
    }
}

/// Standard OAuth provider configuration
#[derive(Clone)]
pub struct OAuthProviderConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

impl OAuthProviderConfig {
    pub fn from_env(prefix: &str, env: Environment) -> Result<Self, ConfigError> {
        let client_id = std::env::var(format!("{}_CLIENT_ID", prefix))
            .map_err(|_| ConfigError::MissingRequired(format!("{}_CLIENT_ID", prefix)))?;

        let client_secret = std::env::var(format!("{}_CLIENT_SECRET", prefix))
            .map_err(|_| ConfigError::MissingRequired(format!("{}_CLIENT_SECRET", prefix)))?;

        let default_redirect = if env.is_development() {
            format!(
                "http://localhost:3000/auth/callback/{}",
                prefix.to_lowercase()
            )
        } else {
            String::new()
        };

        let redirect_uri =
            std::env::var(format!("{}_REDIRECT_URI", prefix)).unwrap_or(default_redirect);

        // Validate redirect URI in production
        if env.is_production() && !redirect_uri.starts_with("https://") {
            return Err(ConfigError::InvalidValue {
                key: format!("{}_REDIRECT_URI", prefix),
                message: "Production OAuth redirect URIs must use HTTPS".to_string(),
            });
        }

        Ok(Self {
            client_id,
            client_secret,
            redirect_uri,
        })
    }
}

/// Apple-specific OAuth configuration
#[derive(Clone)]
pub struct AppleOAuthConfig {
    pub client_id: String,
    pub team_id: String,
    pub key_id: String,
    pub private_key: String,
    pub redirect_uri: String,
}

impl AppleOAuthConfig {
    pub fn from_env(env: Environment) -> Result<Self, ConfigError> {
        let client_id = std::env::var("APPLE_CLIENT_ID")
            .map_err(|_| ConfigError::MissingRequired("APPLE_CLIENT_ID".to_string()))?;

        let team_id = std::env::var("APPLE_TEAM_ID")
            .map_err(|_| ConfigError::MissingRequired("APPLE_TEAM_ID".to_string()))?;

        let key_id = std::env::var("APPLE_KEY_ID")
            .map_err(|_| ConfigError::MissingRequired("APPLE_KEY_ID".to_string()))?;

        let private_key = std::env::var("APPLE_PRIVATE_KEY")
            .map_err(|_| ConfigError::MissingRequired("APPLE_PRIVATE_KEY".to_string()))?;

        let default_redirect = if env.is_development() {
            "http://localhost:3000/auth/callback/apple".to_string()
        } else {
            String::new()
        };

        let redirect_uri = std::env::var("APPLE_REDIRECT_URI").unwrap_or(default_redirect);

        if env.is_production() && !redirect_uri.starts_with("https://") {
            return Err(ConfigError::InvalidValue {
                key: "APPLE_REDIRECT_URI".to_string(),
                message: "Production OAuth redirect URIs must use HTTPS".to_string(),
            });
        }

        Ok(Self {
            client_id,
            team_id,
            key_id,
            private_key,
            redirect_uri,
        })
    }
}

/// Platform sync credentials configuration
/// All platforms except Deezer are optional - if not configured, those platforms will be skipped
#[derive(Clone, Default)]
pub struct PlatformSyncConfig {
    pub spotify: Option<SpotifyCredentials>,
    pub apple_music: Option<AppleMusicCredentials>,
    pub tidal: Option<TidalCredentials>,
    pub youtube: Option<YouTubeCredentials>,
    pub deezer: DeezerConfig, // Always available (no auth needed)
}

impl PlatformSyncConfig {
    pub fn from_env() -> Self {
        Self {
            spotify: SpotifyCredentials::from_env().ok(),
            apple_music: AppleMusicCredentials::from_env().ok(),
            tidal: TidalCredentials::from_env().ok(),
            youtube: YouTubeCredentials::from_env().ok(),
            deezer: DeezerConfig::default(),
        }
    }

    /// Get list of platforms that have valid credentials configured
    pub fn available_platforms(&self) -> Vec<&'static str> {
        let mut platforms = vec!["deezer"]; // Always available
        if self.spotify.is_some() {
            platforms.push("spotify");
        }
        if self.apple_music.is_some() {
            platforms.push("apple_music");
        }
        if self.tidal.is_some() {
            platforms.push("tidal");
        }
        if self.youtube.is_some() {
            platforms.push("youtube");
        }
        platforms
    }

    /// Check if any platform is available (always true since Deezer is public)
    pub fn has_any_platform(&self) -> bool {
        true
    }
}

/// Spotify API credentials (OAuth Client Credentials flow)
#[derive(Clone)]
pub struct SpotifyCredentials {
    pub client_id: String,
    pub client_secret: String,
}

impl SpotifyCredentials {
    pub fn from_env() -> Result<Self, ConfigError> {
        let client_id = std::env::var("SPOTIFY_CLIENT_ID")
            .map_err(|_| ConfigError::MissingRequired("SPOTIFY_CLIENT_ID".to_string()))?;
        let client_secret = std::env::var("SPOTIFY_CLIENT_SECRET")
            .map_err(|_| ConfigError::MissingRequired("SPOTIFY_CLIENT_SECRET".to_string()))?;

        if client_id.is_empty() || client_secret.is_empty() {
            return Err(ConfigError::InvalidValue {
                key: "SPOTIFY_*".to_string(),
                message: "Spotify credentials cannot be empty".to_string(),
            });
        }

        Ok(Self {
            client_id,
            client_secret,
        })
    }
}

/// Apple Music API credentials (JWT-based)
#[derive(Clone)]
pub struct AppleMusicCredentials {
    pub team_id: String,
    pub key_id: String,
    pub private_key: String, // PEM-encoded private key content
}

impl AppleMusicCredentials {
    pub fn from_env() -> Result<Self, ConfigError> {
        let team_id = std::env::var("APPLE_MUSIC_TEAM_ID")
            .map_err(|_| ConfigError::MissingRequired("APPLE_MUSIC_TEAM_ID".to_string()))?;
        let key_id = std::env::var("APPLE_MUSIC_KEY_ID")
            .map_err(|_| ConfigError::MissingRequired("APPLE_MUSIC_KEY_ID".to_string()))?;

        // Try file path first, then direct key env var
        let private_key = if let Ok(key_path) = std::env::var("APPLE_MUSIC_KEY_PATH") {
            // Read key from .p8 file
            std::fs::read_to_string(&key_path).map_err(|e| ConfigError::InvalidValue {
                key: "APPLE_MUSIC_KEY_PATH".to_string(),
                message: format!("Failed to read key file '{}': {}", key_path, e),
            })?
        } else {
            // Fall back to direct key content
            std::env::var("APPLE_MUSIC_PRIVATE_KEY").map_err(|_| {
                ConfigError::MissingRequired(
                    "APPLE_MUSIC_KEY_PATH or APPLE_MUSIC_PRIVATE_KEY".to_string(),
                )
            })?
        };

        if team_id.is_empty() || key_id.is_empty() || private_key.is_empty() {
            return Err(ConfigError::InvalidValue {
                key: "APPLE_MUSIC_*".to_string(),
                message: "Apple Music credentials cannot be empty".to_string(),
            });
        }

        // Validate PEM format
        if !private_key.contains("-----BEGIN PRIVATE KEY-----") {
            return Err(ConfigError::InvalidValue {
                key: "APPLE_MUSIC_PRIVATE_KEY".to_string(),
                message: "Private key must be in PEM format (-----BEGIN PRIVATE KEY-----)"
                    .to_string(),
            });
        }

        Ok(Self {
            team_id,
            key_id,
            private_key,
        })
    }
}

/// Tidal API credentials (OAuth Client Credentials)
#[derive(Clone)]
pub struct TidalCredentials {
    pub client_id: String,
    pub client_secret: String,
}

impl TidalCredentials {
    pub fn from_env() -> Result<Self, ConfigError> {
        let client_id = std::env::var("TIDAL_CLIENT_ID")
            .map_err(|_| ConfigError::MissingRequired("TIDAL_CLIENT_ID".to_string()))?;
        let client_secret = std::env::var("TIDAL_CLIENT_SECRET")
            .map_err(|_| ConfigError::MissingRequired("TIDAL_CLIENT_SECRET".to_string()))?;

        if client_id.is_empty() || client_secret.is_empty() {
            return Err(ConfigError::InvalidValue {
                key: "TIDAL_*".to_string(),
                message: "Tidal credentials cannot be empty".to_string(),
            });
        }

        Ok(Self {
            client_id,
            client_secret,
        })
    }
}

/// YouTube Data API credentials
#[derive(Clone)]
pub struct YouTubeCredentials {
    pub api_key: String,
}

impl YouTubeCredentials {
    pub fn from_env() -> Result<Self, ConfigError> {
        let api_key = std::env::var("YOUTUBE_API_KEY")
            .map_err(|_| ConfigError::MissingRequired("YOUTUBE_API_KEY".to_string()))?;

        if api_key.is_empty() {
            return Err(ConfigError::InvalidValue {
                key: "YOUTUBE_API_KEY".to_string(),
                message: "YouTube API key cannot be empty".to_string(),
            });
        }

        Ok(Self { api_key })
    }
}

/// Deezer config (no auth needed - public API)
#[derive(Clone, Default)]
pub struct DeezerConfig {
    /// Rate limit: 50 requests per 5 seconds
    pub rate_limit_per_5_sec: u32,
}

impl DeezerConfig {
    pub fn new() -> Self {
        Self {
            rate_limit_per_5_sec: 50,
        }
    }
}

/// Helper function to get a required environment variable
pub fn require_env(key: &str) -> Result<String, ConfigError> {
    std::env::var(key).map_err(|_| ConfigError::MissingRequired(key.to_string()))
}

/// Helper function to get an optional environment variable with a default
pub fn env_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_detection() {
        // Default should be development
        assert!(Environment::from_env().is_development());
    }

    #[test]
    fn test_default_jwt_secret() {
        let secret = AuthConfig::default_jwt_secret();
        assert!(secret.len() > 32);
    }
}
