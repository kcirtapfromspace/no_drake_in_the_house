//! Analytics Service Module
//!
//! Provides comprehensive analytics and reporting:
//! - Dashboard metrics and summaries
//! - Trend analysis for artists and offenses
//! - Platform sync performance
//! - System health monitoring
//! - Artist trouble scores
//! - Revenue tracking and distribution

pub mod dashboard;
pub mod reporting;
pub mod revenue;
pub mod trends;
pub mod trouble_score;

pub use dashboard::{DashboardMetrics, DashboardService, TimeRange};
pub use reporting::{Report, ReportType, ReportingService};
pub use revenue::{
    ArtistRevenueBreakdown, GlobalArtistRevenue, PayoutRate, Platform as RevenuePlatform,
    PlatformRevenue, RevenueService, UserPlaycount, UserRevenueDistribution,
};
pub use trends::{TrendAnalysisService, TrendData, TrendDirection};
pub use trouble_score::{
    ArtistTroubleScore, RecalculationSummary, ScoreHistoryEntry, ScoreWeights, TierDistribution,
    TroubleLeaderboardEntry, TroubleScoreComponents, TroubleScoreService, TroubleTier,
};
