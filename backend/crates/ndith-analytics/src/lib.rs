//! ndith-analytics: DuckDB analytics, pluggable graph backends, and related services.
//!
//! This crate isolates heavyweight native dependencies and keeps the graph layer
//! behind a backend trait so the application can converge on LadybugDB without
//! rewriting the services.

pub mod analytics_service;
pub mod databases;
pub mod graph_service;

// Re-export database clients
pub use databases::{
    create_graph_store, DuckDbClient, GraphBackendKind, GraphStats, GraphStore, SharedGraphStore,
};

// Re-export analytics service components
pub use analytics_service::{
    ActionTypeCount, AlbumRevenue, ArtistDiscographyRevenue, ArtistRevenueBreakdown,
    ArtistTroubleScore, CategoryArtistRevenue, CategoryRevenue, CategoryRevenueService,
    DashboardMetrics, DashboardService, EnforcementAnalytics, EnforcementAnalyticsQuery,
    EnforcementAnalyticsService, EnforcementStats, EnforcementTimeSeriesPoint, GlobalArtistRevenue,
    GlobalCategoryRevenue, OffenseCategory, PayoutRate, PlatformRevenue, ProviderStats,
    RecalculationSummary, RecordPlaycountParams, Report, ReportType, ReportingService,
    RevenuePlatform, RevenueService, ScoreHistoryEntry, ScoreWeights, SimulationParams,
    TierDistribution, TimeRange, TrendAnalysisService, TrendData, TrendDirection,
    TroubleLeaderboardEntry, TroubleScoreComponents, TroubleScoreService, TroubleTier,
    UserPlaycount, UserRevenueDistribution,
};

// Re-export graph service components
pub use graph_service::{
    ArtistNetworkResponse, CollaborationBuilder, CollaborationService, GraphSyncService,
    NetworkAnalysisService, NetworkStatsResponse, PathResponse, SyncJob, SyncStats,
    TrackCollaboration,
};
