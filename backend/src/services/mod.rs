// Services module - include working services only
pub mod auth;
pub mod auth_simple;
pub mod oauth;

// Analytics databases (DuckDB, Kùzu, LanceDB)
pub mod databases;

// Catalog sync (multi-platform artist synchronization)
pub mod catalog_sync;

// News pipeline (live news tracking and offense detection)
pub mod news_pipeline;

// Graph service (artist collaboration networks and analysis)
pub mod graph_service;

// Analytics service (dashboard metrics, trend analysis, reporting)
pub mod analytics_service;

pub mod audit_logging;
pub mod dnp_list;
pub mod login_performance;
pub mod monitoring;
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
pub mod offense;
pub mod rate_limiting_middleware;
pub mod registration_monitoring;
pub mod registration_performance;
pub mod user;

pub mod stubs;

// Disabled services - require significant fixes before re-enabling:
// - Missing model types (ArtistAlias, ArtistResolutionResult, MatchType)
// - Trait method mismatches (JobHandler::max_execution_time)
// - Constructor/struct issues
// - Lifetime bounds problems
// pub mod entity_resolution;
// pub mod external_apis;
// pub mod token_vault;
// pub mod token_vault_background;
// pub mod spotify;
// pub mod spotify_library;
// pub mod spotify_enforcement;
// pub mod apple_music;
// pub mod apple_music_library;
// pub mod community_list;
// pub mod rate_limiting;
// pub mod job_queue;
// pub mod enforcement_job_handler;
// pub mod audit;
// pub mod content_moderation;
// pub mod analytics;

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
pub use offense::OffenseService;
pub use rate_limiting_middleware::{registration_rate_limit_middleware, RateLimitService};
pub use user::UserService;

// Export stub services for tests
pub use stubs::*;

// Export database clients
pub use databases::{DatabaseClients, DatabasesConfig, DuckDbClient, KuzuClient, LanceDbClient};

// Export catalog sync components
pub use catalog_sync::{
    AppleMusicSyncWorker, CanonicalArtist, CatalogSyncOrchestrator, CrossPlatformIdentityResolver,
    DeezerSyncWorker, IdentityMatch, MatchMethod, OrchestratorBuilder, OverallSyncStatus, Platform,
    PlatformAlbum, PlatformArtist, PlatformCatalogWorker, PlatformTrack, RateLimitConfig,
    SpotifySyncWorker, SyncCheckpoint, SyncProgress, SyncResult, SyncStatus, SyncTriggerRequest,
    SyncType, TidalSyncWorker, YouTubeMusicSyncWorker,
};

// Export news pipeline components
pub use news_pipeline::{
    ArticleEmbedding,
    EmbeddingGenerator,
    // Processing
    EntityExtractor,
    EntityType,
    ExtractedEntity,
    FetchedArticle,
    NewsApiClient,
    NewsApiConfig,
    NewsPipelineConfig,
    // Orchestration
    NewsPipelineOrchestrator,
    OffenseClassification,
    OffenseClassifier,
    PipelineStats,
    ProcessedArticle,
    RedditConfig,
    RedditMonitor,
    // Ingestion
    RssFetcher,
    RssFetcherConfig,
    ScheduledPipelineHandle,
    ScheduledPipelineRunner,
    TwitterConfig,
    TwitterMonitor,
    WebScraper,
    WebScraperConfig,
};

// Export graph service components
pub use graph_service::{
    ArtistNetworkResponse,
    CollaborationBuilder,
    // Collaboration
    CollaborationService,
    // Sync
    GraphSyncService,
    // Network analysis
    NetworkAnalysisService,
    NetworkStatsResponse,
    PathResponse,
    SyncJob,
    SyncStats,
    TrackCollaboration,
};

// Export analytics service components
pub use analytics_service::{
    DashboardMetrics,
    // Dashboard
    DashboardService,
    Report,
    ReportType,
    // Reporting
    ReportingService,
    TimeRange,
    // Trends
    TrendAnalysisService,
    TrendData,
    TrendDirection,
};
