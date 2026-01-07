//! Analytics Service Module
//!
//! Provides comprehensive analytics and reporting:
//! - Dashboard metrics and summaries
//! - Trend analysis for artists and offenses
//! - Platform sync performance
//! - System health monitoring

pub mod dashboard;
pub mod trends;
pub mod reporting;

pub use dashboard::{DashboardService, DashboardMetrics, TimeRange};
pub use trends::{TrendAnalysisService, TrendData, TrendDirection};
pub use reporting::{ReportingService, Report, ReportType};
