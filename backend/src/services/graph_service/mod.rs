//! Graph Service Module
//!
//! Provides high-level graph operations for artist networks:
//! - Syncing artists from PostgreSQL to Kùzu
//! - Building collaboration networks from track data
//! - Network traversal and analysis
//! - Integration with news pipeline for mention tracking

pub mod collaboration;
pub mod network;
pub mod sync;

pub use collaboration::{CollaborationBuilder, CollaborationService, TrackCollaboration};
pub use network::{
    ArtistNetworkResponse, NetworkAnalysisService, NetworkStatsResponse, PathResponse,
};
pub use sync::{GraphSyncService, SyncJob, SyncStats};
