// Services module - include working services only
pub mod auth_simple;
pub mod auth;
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

pub mod oauth_encryption;
pub mod oauth_google;
pub mod oauth_apple;
pub mod oauth_github;
pub mod oauth_spotify;
pub mod oauth_token_manager;
pub mod oauth_config_validator;
pub mod oauth_health_monitor;
pub mod oauth_error_recovery;
pub mod oauth_security_logger;
pub mod monitoring;
pub mod rate_limiting_middleware;
pub mod audit_logging;
pub mod dnp_list;
pub mod registration_performance;
pub mod registration_monitoring;
pub mod login_performance;
pub mod user;
pub mod offense;

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

pub use auth_simple::AuthService as SimpleAuthService;
pub use auth::AuthService;
pub use oauth::{OAuthProvider, BaseOAuthProvider, OAuthStateManager};
pub use oauth_encryption::OAuthTokenEncryption;
pub use oauth_google::{GoogleOAuthProvider, GoogleOAuthService};
pub use oauth_apple::{AppleOAuthProvider, AppleOAuthService, AppleOAuthConfig};
pub use oauth_github::{GitHubOAuthProvider, GitHubOAuthService, GitHubEmail};
pub use oauth_spotify::SpotifyOAuthProvider;
pub use oauth_config_validator::{OAuthConfigValidator, OAuthProviderValidation};
pub use oauth_health_monitor::{OAuthHealthMonitor, OAuthProviderHealth, OAuthProviderHealthStatus, OAuthHealthConfig, RateLimitInfo};
pub use monitoring::*;
pub use rate_limiting_middleware::{RateLimitService, registration_rate_limit_middleware};
pub use audit_logging::*;
pub use dnp_list::DnpListService;
pub use user::UserService;
pub use offense::OffenseService;

// Export stub services for tests
pub use stubs::*;

// Export database clients
pub use databases::{DatabaseClients, DatabasesConfig, DuckDbClient, KuzuClient, LanceDbClient};

// Export catalog sync components
pub use catalog_sync::{
    Platform, SyncType, SyncStatus, PlatformArtist, PlatformTrack, PlatformAlbum,
    PlatformCatalogWorker, RateLimitConfig, SyncProgress, SyncResult, SyncCheckpoint,
    CatalogSyncOrchestrator, OrchestratorBuilder, OverallSyncStatus, SyncTriggerRequest,
    CrossPlatformIdentityResolver, CanonicalArtist, IdentityMatch, MatchMethod,
    SpotifySyncWorker, AppleMusicSyncWorker, TidalSyncWorker, YouTubeMusicSyncWorker, DeezerSyncWorker,
};

// Export news pipeline components
pub use news_pipeline::{
    // Ingestion
    RssFetcher, RssFetcherConfig, FetchedArticle,
    NewsApiClient, NewsApiConfig,
    TwitterMonitor, TwitterConfig,
    RedditMonitor, RedditConfig,
    WebScraper, WebScraperConfig,
    // Processing
    EntityExtractor, ExtractedEntity, EntityType,
    OffenseClassifier, OffenseClassification,
    EmbeddingGenerator, ArticleEmbedding,
    // Orchestration
    NewsPipelineOrchestrator, NewsPipelineConfig,
    ProcessedArticle, PipelineStats,
    ScheduledPipelineRunner, ScheduledPipelineHandle,
};

// Export graph service components
pub use graph_service::{
    // Sync
    GraphSyncService, SyncStats, SyncJob,
    // Network analysis
    NetworkAnalysisService, ArtistNetworkResponse, PathResponse, NetworkStatsResponse,
    // Collaboration
    CollaborationService, CollaborationBuilder, TrackCollaboration,
};

// Export analytics service components
pub use analytics_service::{
    // Dashboard
    DashboardService, DashboardMetrics, TimeRange,
    // Trends
    TrendAnalysisService, TrendData, TrendDirection,
    // Reporting
    ReportingService, Report, ReportType,
};