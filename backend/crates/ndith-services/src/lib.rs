//! ndith-services: Business logic layer for the No Drake in the House backend.
//!
//! Contains auth, OAuth, token vault, catalog sync, enforcement, and other services.
//! No heavyweight native dependencies (duckdb, graph backends, lancedb, fastembed are in separate crates).
#![allow(clippy::result_large_err)]

// Auth
pub mod auth;
pub mod auth_simple;

// Metrics
pub mod metrics;

// Circuit breaker
pub mod circuit_breaker;

// KMS
pub mod kms;

// OAuth
pub mod oauth;
pub mod oauth_apple;
pub mod oauth_config_validator;
pub mod oauth_encryption;
pub mod oauth_error_recovery;
pub mod oauth_github;
pub mod oauth_google;
pub mod oauth_health_monitor;
pub mod oauth_security_logger;
pub mod oauth_spotify;
pub mod oauth_token_manager;
pub mod oauth_youtube_music;

// Platform services
pub mod apple_music;
pub mod apple_music_enforcement;
pub mod playlist_sanitizer;
pub mod spotify;
pub mod spotify_library;
pub mod tidal;

// Catalog sync
pub mod catalog_sync;

// Token vault
pub mod token_vault;
pub mod token_vault_background;
pub mod token_vault_repository;

// Token refresh
pub mod token_refresh_job;

// Core services
pub mod audit_logging;
pub mod dnp_list;
pub mod offense;
pub mod user;

// Rate limiting and job queue
pub mod job_queue;
pub mod rate_limiting;
pub mod rate_limiting_middleware;

// Monitoring and performance
pub mod login_performance;
pub mod monitoring;
pub mod registration_monitoring;
pub mod registration_performance;

// Notification service
pub mod notification_service;

// Stubs for testing
pub mod stubs;

// ---- Re-exports ----

pub use audit_logging::*;
pub use auth::AuthService;
pub use auth_simple::AuthService as SimpleAuthService;
pub use dnp_list::DnpListService;
pub use monitoring::*;
pub use oauth::{BaseOAuthProvider, OAuthProvider, OAuthStateManager};
pub use oauth_apple::{AppleOAuthConfig, AppleOAuthProvider, AppleOAuthService};
pub use oauth_config_validator::{OAuthConfigValidator, OAuthProviderValidation};
pub use oauth_encryption::OAuthTokenEncryption;
pub use oauth_github::{GitHubEmail, GitHubOAuthProvider, GitHubOAuthService};
pub use oauth_google::{GoogleOAuthProvider, GoogleOAuthService};
pub use oauth_health_monitor::{
    OAuthHealthConfig, OAuthHealthMonitor, OAuthProviderHealth, OAuthProviderHealthStatus,
    RateLimitInfo,
};
pub use oauth_spotify::SpotifyOAuthProvider;
pub use oauth_youtube_music::YouTubeMusicOAuthProvider;
pub use offense::OffenseService;
pub use rate_limiting_middleware::{registration_rate_limit_middleware, RateLimitService};
pub use user::UserService;

pub use kms::{
    create_kms_provider, KmsProvider, KmsProviderType, MockKmsProvider, VaultAuthMethod,
    VaultConfig, VaultKmsProvider,
};

pub use stubs::*;

pub use catalog_sync::{
    AppleMusicSyncWorker, CanonicalArtist, CatalogSyncOrchestrator, CrossPlatformIdentityResolver,
    DeezerSyncWorker, IdentityMatch, MatchMethod, MusicBrainzImportStats, MusicBrainzImporter,
    OrchestratorBuilder, OverallSyncStatus, Platform, PlatformAlbum, PlatformArtist,
    PlatformCatalogWorker, PlatformTrack, RateLimitConfig, SpotifySyncWorker, SyncCheckpoint,
    SyncProgress, SyncResult, SyncStatus, SyncTriggerRequest, SyncType, TidalSyncWorker,
    YouTubeMusicSyncWorker,
};

pub use apple_music::{AppleMusicConfig, AppleMusicService, RATING_DISLIKE, RATING_LIKE};
pub use apple_music_enforcement::{AppleMusicEnforcementService, EnforcementHistoryItem};

pub use token_vault::TokenVaultService;
pub use token_vault_background::{TokenVaultBackgroundService, TokenVaultStatistics};
pub use token_vault_repository::{ConnectionStatistics, TokenVaultRepository};

pub use token_refresh_job::{RefreshCycleResult, TokenRefreshBackgroundJob, TokenRefreshMetrics};

pub use notification_service::NotificationService;

pub use playlist_sanitizer::PlaylistSanitizerService;
pub use spotify::{SpotifyConfig, SpotifyService};
pub use spotify_library::SpotifyLibraryService;

pub use job_queue::{
    Job, JobHandler, JobPriority, JobProgress, JobQueueService, JobResult, JobStatus, JobType,
    WorkerConfig, WorkerStats,
};
pub use rate_limiting::RateLimitingService;

pub use circuit_breaker::{
    CircuitBreakerConfig, CircuitBreakerError, CircuitBreakerMetrics, CircuitBreakerService,
    CircuitBreakerStateEnum,
};

pub use metrics::{
    metrics_handler, DatabaseMetrics, MetricsCollector, MetricsMiddleware, RedisMetrics,
    RequestTimer,
};
