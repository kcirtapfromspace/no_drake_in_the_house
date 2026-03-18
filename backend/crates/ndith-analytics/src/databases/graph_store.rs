//! Graph storage abstractions.
//!
//! The analytics crate should depend on a graph API, not a specific database.
//! That keeps the current application insulated from graph engine churn and gives
//! us a clean seam for the LadybugDB adapter.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

/// Supported graph backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphBackendKind {
    LadybugDb,
}

impl fmt::Display for GraphBackendKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphBackendKind::LadybugDb => write!(f, "ladybugdb"),
        }
    }
}

impl FromStr for GraphBackendKind {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "ladybugdb" | "ladybug" => Ok(Self::LadybugDb),
            other => Err(anyhow!("Unsupported graph backend: {other}")),
        }
    }
}

/// Shared graph store handle used by the graph services.
pub type SharedGraphStore = Arc<dyn GraphStore>;

/// Backend-agnostic graph storage contract.
pub trait GraphStore: Send + Sync {
    fn backend_kind(&self) -> GraphBackendKind;
    fn initialize_schema(&self) -> Result<()>;
    fn upsert_artist(&self, artist: &GraphArtist) -> Result<()>;
    fn add_collaboration(&self, collab: &Collaboration) -> Result<()>;
    fn get_artist_network(&self, artist_id: &str, depth: u32) -> Result<ArtistNetwork>;
    fn find_path(&self, artist1_id: &str, artist2_id: &str) -> Result<Option<ArtistPath>>;
    fn analyze_blocked_network(
        &self,
        user_blocked_ids: &[String],
        max_distance: u32,
    ) -> Result<BlockedNetworkAnalysis>;
    fn get_stats(&self) -> Result<GraphStats>;
}

/// Create a graph store for the selected backend.
pub fn create_graph_store(backend: GraphBackendKind, _location: &str) -> Result<SharedGraphStore> {
    match backend {
        GraphBackendKind::LadybugDb => Err(anyhow!(
            "LadybugDB is now the target graph backend, but the adapter is not implemented yet."
        )),
    }
}

/// Artist for graph operations.
#[derive(Debug, Clone)]
pub struct GraphArtist {
    pub id: String,
    pub canonical_name: String,
    pub genres: Vec<String>,
    pub country: Option<String>,
    pub formed_year: Option<i64>,
    pub is_blocked: bool,
    pub block_count: i64,
}

/// Collaboration record.
#[derive(Debug, Clone)]
pub struct Collaboration {
    pub artist1_id: String,
    pub artist2_id: String,
    pub track_id: String,
    pub track_title: String,
    pub collaboration_type: String,
    pub year: Option<i64>,
}

/// Artist node in network response.
#[derive(Debug, Clone)]
pub struct GraphArtistNode {
    pub id: String,
    pub name: String,
    pub is_blocked: bool,
    pub genres: Vec<String>,
    pub collaboration_count: u32,
}

/// Collaboration edge in network.
#[derive(Debug, Clone)]
pub struct CollaborationEdge {
    pub source_id: String,
    pub target_id: String,
    pub collaboration_type: String,
    pub track_count: u32,
    pub most_recent_year: Option<i64>,
}

/// Network statistics.
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub total_nodes: u32,
    pub total_edges: u32,
    pub blocked_artists: u32,
}

/// Artist collaboration network.
#[derive(Debug, Clone)]
pub struct ArtistNetwork {
    pub center_artist: GraphArtistNode,
    pub nodes: Vec<GraphArtistNode>,
    pub edges: Vec<CollaborationEdge>,
    pub stats: NetworkStats,
}

/// Path between artists.
#[derive(Debug, Clone)]
pub struct ArtistPath {
    pub artists: Vec<String>,
    pub collaborations: Vec<String>,
    pub distance: u32,
}

/// Connected artist in blocked network.
#[derive(Debug, Clone)]
pub struct ConnectedArtist {
    pub id: String,
    pub name: String,
    pub distance: u32,
    pub connection_strength: u32,
}

/// Blocked network analysis.
#[derive(Debug, Clone)]
pub struct BlockedNetworkAnalysis {
    pub blocked_artists_count: u32,
    pub connected_artists: Vec<ConnectedArtist>,
    pub risk_scores: std::collections::HashMap<String, f64>,
}

/// Graph database statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphStats {
    pub artist_count: u64,
    pub collaboration_count: u64,
    pub label_count: u64,
    pub track_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_graph_backend_aliases() {
        assert_eq!(
            "ladybug".parse::<GraphBackendKind>().unwrap(),
            GraphBackendKind::LadybugDb
        );
        assert_eq!(
            "ladybugdb".parse::<GraphBackendKind>().unwrap(),
            GraphBackendKind::LadybugDb
        );
    }

    #[test]
    fn ladybug_backend_is_stubbed_cleanly() {
        let error = match create_graph_store(GraphBackendKind::LadybugDb, "/tmp/graph") {
            Ok(_) => panic!("expected LadybugDB backend to be unimplemented"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("target graph backend"));
    }
}
