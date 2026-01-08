//! Analytics Service Module
//!
//! Provides comprehensive analytics and reporting:
//! - Dashboard metrics and summaries
//! - Trend analysis for artists and offenses
//! - Platform sync performance
//! - System health monitoring

pub mod dashboard;
pub mod reporting;
pub mod trends;

pub use dashboard::{DashboardMetrics, DashboardService, TimeRange};
pub use reporting::{Report, ReportType, ReportingService};
pub use trends::{TrendAnalysisService, TrendData, TrendDirection};
