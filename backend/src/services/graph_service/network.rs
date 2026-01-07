//! Network Analysis Service
//!
//! Provides network analysis and traversal operations:
//! - Artist collaboration networks
//! - Path finding between artists
//! - Blocked network analysis
//! - Network statistics and metrics

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

use crate::services::databases::{
    KuzuClient, ArtistNetwork, ArtistPath, BlockedNetworkAnalysis,
    GraphArtistNode, CollaborationEdge, ConnectedArtist,
};

/// Network analysis response for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistNetworkResponse {
    /// Center artist information
    pub center: ArtistNodeResponse,
    /// All nodes in the network
    pub nodes: Vec<ArtistNodeResponse>,
    /// Edges (collaborations)
    pub edges: Vec<EdgeResponse>,
    /// Network statistics
    pub stats: NetworkStatsResponse,
}

/// Artist node in network response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistNodeResponse {
    pub id: Uuid,
    pub name: String,
    pub is_blocked: bool,
    pub genres: Vec<String>,
    pub collaboration_count: u32,
    /// Risk level based on blocked connections
    pub risk_level: Option<RiskLevel>,
}

/// Edge (collaboration) in network response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeResponse {
    pub source: Uuid,
    pub target: Uuid,
    pub collaboration_type: String,
    pub track_count: u32,
    pub most_recent_year: Option<i32>,
    pub weight: f64,
}

/// Network statistics response
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkStatsResponse {
    pub total_nodes: u32,
    pub total_edges: u32,
    pub blocked_nodes: u32,
    pub blocked_percentage: f64,
    pub average_degree: f64,
    pub density: f64,
    pub clustering_coefficient: Option<f64>,
}

/// Path between artists response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathResponse {
    pub found: bool,
    pub distance: Option<u32>,
    pub path: Vec<PathNodeResponse>,
    pub via_blocked: bool,
}

/// Node in path response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathNodeResponse {
    pub artist: ArtistNodeResponse,
    pub connection: Option<ConnectionResponse>,
}

/// Connection info in path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionResponse {
    pub track_title: String,
    pub collaboration_type: String,
    pub year: Option<i32>,
}

/// Risk level for artists based on blocked connections
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    fn from_score(score: f64) -> Self {
        if score <= 0.0 {
            RiskLevel::None
        } else if score < 0.25 {
            RiskLevel::Low
        } else if score < 0.5 {
            RiskLevel::Medium
        } else if score < 0.75 {
            RiskLevel::High
        } else {
            RiskLevel::Critical
        }
    }
}

/// Blocked network analysis response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedNetworkResponse {
    pub blocked_count: u32,
    pub connected_artists: Vec<ConnectedArtistResponse>,
    pub risk_summary: RiskSummary,
    pub recommendations: Vec<BlockRecommendation>,
}

/// Connected artist in blocked network response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectedArtistResponse {
    pub id: Uuid,
    pub name: String,
    pub distance: u32,
    pub connection_strength: u32,
    pub risk_score: f64,
    pub risk_level: RiskLevel,
    pub blocked_connections: Vec<BlockedConnectionInfo>,
}

/// Info about connection to blocked artist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedConnectionInfo {
    pub blocked_artist_id: Uuid,
    pub blocked_artist_name: String,
    pub track_title: Option<String>,
    pub distance: u32,
}

/// Summary of risk in blocked network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSummary {
    pub high_risk_count: u32,
    pub medium_risk_count: u32,
    pub low_risk_count: u32,
    pub total_risk_score: f64,
    pub average_distance: f64,
}

/// Recommendation to block an artist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRecommendation {
    pub artist_id: Uuid,
    pub artist_name: String,
    pub reason: String,
    pub risk_score: f64,
    pub blocked_connections: u32,
}

/// Network analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAnalysisConfig {
    /// Maximum depth for network traversal
    pub max_depth: u32,
    /// Maximum nodes to return
    pub max_nodes: u32,
    /// Include blocked artists in analysis
    pub include_blocked: bool,
    /// Calculate clustering coefficient (expensive)
    pub calculate_clustering: bool,
    /// Risk score threshold for recommendations
    pub risk_threshold: f64,
}

impl Default for NetworkAnalysisConfig {
    fn default() -> Self {
        Self {
            max_depth: 3,
            max_nodes: 100,
            include_blocked: true,
            calculate_clustering: false,
            risk_threshold: 0.5,
        }
    }
}

/// Network analysis service
pub struct NetworkAnalysisService {
    config: NetworkAnalysisConfig,
    kuzu: Arc<KuzuClient>,
    pool: PgPool,
}

impl NetworkAnalysisService {
    /// Create a new network analysis service
    pub fn new(kuzu: Arc<KuzuClient>, pool: PgPool) -> Self {
        Self {
            config: NetworkAnalysisConfig::default(),
            kuzu,
            pool,
        }
    }

    /// Create with custom config
    pub fn with_config(kuzu: Arc<KuzuClient>, pool: PgPool, config: NetworkAnalysisConfig) -> Self {
        Self {
            config,
            kuzu,
            pool,
        }
    }

    /// Get artist collaboration network
    pub fn get_artist_network(&self, artist_id: Uuid, depth: Option<u32>) -> Result<ArtistNetworkResponse> {
        let depth = depth.unwrap_or(self.config.max_depth).min(self.config.max_depth);

        let network = self.kuzu.get_artist_network(&artist_id.to_string(), depth)?;

        self.convert_network_response(network)
    }

    /// Find shortest path between two artists
    pub fn find_path(&self, from_id: Uuid, to_id: Uuid) -> Result<PathResponse> {
        let path = self.kuzu.find_path(&from_id.to_string(), &to_id.to_string())?;

        match path {
            Some(p) => {
                let path_nodes: Vec<PathNodeResponse> = p.artists
                    .iter()
                    .enumerate()
                    .map(|(i, artist_id)| {
                        let connection = if i < p.collaborations.len() {
                            Some(ConnectionResponse {
                                track_title: p.collaborations[i].clone(),
                                collaboration_type: "unknown".to_string(),
                                year: None,
                            })
                        } else {
                            None
                        };

                        PathNodeResponse {
                            artist: ArtistNodeResponse {
                                id: Uuid::parse_str(artist_id).unwrap_or_default(),
                                name: artist_id.clone(), // Would need lookup
                                is_blocked: false,
                                genres: vec![],
                                collaboration_count: 0,
                                risk_level: None,
                            },
                            connection,
                        }
                    })
                    .collect();

                // Check if path goes through blocked artists
                let via_blocked = path_nodes.iter().any(|n| n.artist.is_blocked);

                Ok(PathResponse {
                    found: true,
                    distance: Some(p.distance),
                    path: path_nodes,
                    via_blocked,
                })
            }
            None => Ok(PathResponse {
                found: false,
                distance: None,
                path: vec![],
                via_blocked: false,
            }),
        }
    }

    /// Analyze network of blocked artists
    pub async fn analyze_blocked_network(
        &self,
        user_id: Uuid,
        max_distance: Option<u32>,
    ) -> Result<BlockedNetworkResponse> {
        // Get user's blocked artists
        let blocked_ids = self.get_user_blocked_artists(user_id).await?;

        if blocked_ids.is_empty() {
            return Ok(BlockedNetworkResponse {
                blocked_count: 0,
                connected_artists: vec![],
                risk_summary: RiskSummary {
                    high_risk_count: 0,
                    medium_risk_count: 0,
                    low_risk_count: 0,
                    total_risk_score: 0.0,
                    average_distance: 0.0,
                },
                recommendations: vec![],
            });
        }

        let max_distance = max_distance.unwrap_or(3);
        let blocked_id_strings: Vec<String> = blocked_ids.iter().map(|id| id.to_string()).collect();

        let analysis = self.kuzu.analyze_blocked_network(&blocked_id_strings, max_distance)?;

        self.convert_blocked_analysis(analysis, &blocked_ids).await
    }

    /// Get blocked artists with network context
    pub async fn get_blocked_artists_network(&self, user_id: Uuid) -> Result<Vec<ArtistNodeResponse>> {
        let blocked_ids = self.get_user_blocked_artists(user_id).await?;

        let mut artists = Vec::new();
        for artist_id in blocked_ids {
            if let Ok(network) = self.kuzu.get_artist_network(&artist_id.to_string(), 1) {
                artists.push(ArtistNodeResponse {
                    id: artist_id,
                    name: network.center_artist.name,
                    is_blocked: true,
                    genres: network.center_artist.genres,
                    collaboration_count: network.edges.len() as u32,
                    risk_level: Some(RiskLevel::Critical),
                });
            }
        }

        Ok(artists)
    }

    /// Get direct collaborators of an artist
    pub fn get_collaborators(&self, artist_id: Uuid) -> Result<Vec<ArtistNodeResponse>> {
        let network = self.kuzu.get_artist_network(&artist_id.to_string(), 1)?;

        let collaborators: Vec<ArtistNodeResponse> = network.nodes
            .iter()
            .filter(|n| n.id != artist_id.to_string())
            .map(|node| ArtistNodeResponse {
                id: Uuid::parse_str(&node.id).unwrap_or_default(),
                name: node.name.clone(),
                is_blocked: node.is_blocked,
                genres: node.genres.clone(),
                collaboration_count: node.collaboration_count,
                risk_level: if node.is_blocked { Some(RiskLevel::High) } else { None },
            })
            .collect();

        Ok(collaborators)
    }

    /// Get network statistics for an artist
    pub fn get_network_stats(&self, artist_id: Uuid, depth: u32) -> Result<NetworkStatsResponse> {
        let network = self.kuzu.get_artist_network(&artist_id.to_string(), depth)?;
        self.calculate_network_stats(&network)
    }

    /// Get user's blocked artists from PostgreSQL
    async fn get_user_blocked_artists(&self, user_id: Uuid) -> Result<Vec<Uuid>> {
        let blocked: Vec<Uuid> = sqlx::query_scalar(
            "SELECT artist_id FROM user_artist_blocks WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch blocked artists")?;

        Ok(blocked)
    }

    /// Convert internal network to API response
    fn convert_network_response(&self, network: ArtistNetwork) -> Result<ArtistNetworkResponse> {
        let center = ArtistNodeResponse {
            id: Uuid::parse_str(&network.center_artist.id).unwrap_or_default(),
            name: network.center_artist.name.clone(),
            is_blocked: network.center_artist.is_blocked,
            genres: network.center_artist.genres.clone(),
            collaboration_count: network.center_artist.collaboration_count,
            risk_level: None,
        };

        let nodes: Vec<ArtistNodeResponse> = network.nodes
            .iter()
            .map(|node| ArtistNodeResponse {
                id: Uuid::parse_str(&node.id).unwrap_or_default(),
                name: node.name.clone(),
                is_blocked: node.is_blocked,
                genres: node.genres.clone(),
                collaboration_count: node.collaboration_count,
                risk_level: if node.is_blocked { Some(RiskLevel::High) } else { None },
            })
            .collect();

        let edges: Vec<EdgeResponse> = network.edges
            .iter()
            .map(|edge| EdgeResponse {
                source: Uuid::parse_str(&edge.source_id).unwrap_or_default(),
                target: Uuid::parse_str(&edge.target_id).unwrap_or_default(),
                collaboration_type: edge.collaboration_type.clone(),
                track_count: edge.track_count,
                most_recent_year: edge.most_recent_year.map(|y| y as i32),
                weight: edge.track_count as f64,
            })
            .collect();

        let stats = self.calculate_network_stats(&network)?;

        Ok(ArtistNetworkResponse {
            center,
            nodes,
            edges,
            stats,
        })
    }

    /// Calculate network statistics
    fn calculate_network_stats(&self, network: &ArtistNetwork) -> Result<NetworkStatsResponse> {
        let total_nodes = network.nodes.len() as u32;
        let total_edges = network.edges.len() as u32;
        let blocked_nodes = network.stats.blocked_artists;

        let blocked_percentage = if total_nodes > 0 {
            (blocked_nodes as f64 / total_nodes as f64) * 100.0
        } else {
            0.0
        };

        let average_degree = if total_nodes > 0 {
            (2.0 * total_edges as f64) / total_nodes as f64
        } else {
            0.0
        };

        // Network density = 2E / (N * (N-1))
        let density = if total_nodes > 1 {
            (2.0 * total_edges as f64) / (total_nodes as f64 * (total_nodes - 1) as f64)
        } else {
            0.0
        };

        Ok(NetworkStatsResponse {
            total_nodes,
            total_edges,
            blocked_nodes,
            blocked_percentage,
            average_degree,
            density,
            clustering_coefficient: None, // Expensive to calculate
        })
    }

    /// Convert blocked network analysis to API response
    async fn convert_blocked_analysis(
        &self,
        analysis: BlockedNetworkAnalysis,
        blocked_ids: &[Uuid],
    ) -> Result<BlockedNetworkResponse> {
        let mut connected_artists = Vec::new();
        let mut high_risk = 0u32;
        let mut medium_risk = 0u32;
        let mut low_risk = 0u32;
        let mut total_risk = 0.0f64;
        let mut total_distance = 0u32;

        for artist in &analysis.connected_artists {
            let risk_score = analysis.risk_scores.get(&artist.id).copied().unwrap_or(0.0);
            let normalized_risk = (risk_score / 10.0).min(1.0); // Normalize to 0-1
            let risk_level = RiskLevel::from_score(normalized_risk);

            match risk_level {
                RiskLevel::High | RiskLevel::Critical => high_risk += 1,
                RiskLevel::Medium => medium_risk += 1,
                RiskLevel::Low => low_risk += 1,
                RiskLevel::None => {}
            }

            total_risk += normalized_risk;
            total_distance += artist.distance;

            connected_artists.push(ConnectedArtistResponse {
                id: Uuid::parse_str(&artist.id).unwrap_or_default(),
                name: artist.name.clone(),
                distance: artist.distance,
                connection_strength: artist.connection_strength,
                risk_score: normalized_risk,
                risk_level,
                blocked_connections: vec![], // Would need additional query
            });
        }

        // Sort by risk score
        connected_artists.sort_by(|a, b| b.risk_score.partial_cmp(&a.risk_score).unwrap());

        let average_distance = if !analysis.connected_artists.is_empty() {
            total_distance as f64 / analysis.connected_artists.len() as f64
        } else {
            0.0
        };

        // Generate recommendations for high-risk artists
        let recommendations: Vec<BlockRecommendation> = connected_artists
            .iter()
            .filter(|a| a.risk_level == RiskLevel::High || a.risk_level == RiskLevel::Critical)
            .take(5)
            .map(|a| BlockRecommendation {
                artist_id: a.id,
                artist_name: a.name.clone(),
                reason: format!(
                    "Closely connected to {} blocked artist(s) with distance {}",
                    a.blocked_connections.len().max(1),
                    a.distance
                ),
                risk_score: a.risk_score,
                blocked_connections: a.blocked_connections.len() as u32,
            })
            .collect();

        Ok(BlockedNetworkResponse {
            blocked_count: blocked_ids.len() as u32,
            connected_artists,
            risk_summary: RiskSummary {
                high_risk_count: high_risk,
                medium_risk_count: medium_risk,
                low_risk_count: low_risk,
                total_risk_score: total_risk,
                average_distance,
            },
            recommendations,
        })
    }

    /// Find artists at risk based on blocked network
    pub async fn find_at_risk_artists(
        &self,
        user_id: Uuid,
        min_risk_score: f64,
    ) -> Result<Vec<ConnectedArtistResponse>> {
        let analysis = self.analyze_blocked_network(user_id, Some(2)).await?;

        Ok(analysis
            .connected_artists
            .into_iter()
            .filter(|a| a.risk_score >= min_risk_score)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_from_score() {
        assert_eq!(RiskLevel::from_score(0.0), RiskLevel::None);
        assert_eq!(RiskLevel::from_score(0.1), RiskLevel::Low);
        assert_eq!(RiskLevel::from_score(0.3), RiskLevel::Medium);
        assert_eq!(RiskLevel::from_score(0.6), RiskLevel::High);
        assert_eq!(RiskLevel::from_score(0.9), RiskLevel::Critical);
    }

    #[test]
    fn test_default_config() {
        let config = NetworkAnalysisConfig::default();
        assert_eq!(config.max_depth, 3);
        assert_eq!(config.max_nodes, 100);
        assert!(config.include_blocked);
    }
}
