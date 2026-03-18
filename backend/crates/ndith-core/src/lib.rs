//! ndith-core: Shared types for the No Drake in the House backend.
//!
//! Contains error types, configuration, models, and validation helpers.
//! This crate has no heavyweight C/C++ dependencies.

pub mod config;
pub mod error;
pub mod models;
pub mod validation;

// Re-export commonly used types
pub use config::{
    AppConfig, AppleMusicCredentials, AuthConfig, ConfigError, DatabaseSettings, DeezerConfig,
    Environment, OAuthSettings, PlatformSyncConfig, RedisSettings, ServerConfig,
    SpotifyCredentials, TidalCredentials, TokenRefreshConfig, TokenVaultConfig, YouTubeCredentials,
};
pub use error::{AppError, ErrorResponse, Result};
pub use models::*;
pub use validation::{validate_email, validate_password, validate_totp_code, ValidatedJson};
