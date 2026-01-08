//! Kùzu Graph Database Client
//!
//! Provides graph database capabilities for:
//! - Artist collaboration networks
//! - Label relationships
//! - Track/album associations
//! - Network traversal queries

use anyhow::{Context, Result};
use kuzu::{Connection, Database, SystemConfig, Value};
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

/// Kùzu graph database client
pub struct KuzuClient {
    db: Arc<Database>,
    db_path: String,
}

impl KuzuClient {
    /// Create a new Kùzu client with persistent storage
    pub fn new(db_path: &str) -> Result<Self> {
        // Create directory if it doesn't exist
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db = Database::new(db_path, SystemConfig::default())
            .context("Failed to open Kùzu database")?;

        Ok(Self {
            db: Arc::new(db),
            db_path: db_path.to_string(),
        })
    }

    /// Get a connection to the database
    fn connection(&self) -> Result<Connection> {
        Connection::new(&self.db).context("Failed to create Kùzu connection")
    }

    /// Initialize the graph schema
    pub fn initialize_schema(&self) -> Result<()> {
        let conn = self.connection()?;

        // Create node tables
        // Artist node
        conn.query(
            r#"
            CREATE NODE TABLE IF NOT EXISTS Artist (
                id STRING PRIMARY KEY,
                canonical_name STRING,
                genres STRING[],
                country STRING,
                formed_year INT64,
                is_blocked BOOLEAN,
                block_count INT64
            )
            "#,
        )?;

        // Label node
        conn.query(
            r#"
            CREATE NODE TABLE IF NOT EXISTS Label (
                id STRING PRIMARY KEY,
                name STRING,
                parent_label_id STRING,
                country STRING
            )
            "#,
        )?;

        // Track node
        conn.query(
            r#"
            CREATE NODE TABLE IF NOT EXISTS Track (
                id STRING PRIMARY KEY,
                title STRING,
                isrc STRING,
                duration_ms INT64,
                release_year INT64
            )
            "#,
        )?;

        // Album node
        conn.query(
            r#"
            CREATE NODE TABLE IF NOT EXISTS Album (
                id STRING PRIMARY KEY,
                title STRING,
                upc STRING,
                release_year INT64,
                album_type STRING
            )
            "#,
        )?;

        // NewsArticle node (for linking mentions)
        conn.query(
            r#"
            CREATE NODE TABLE IF NOT EXISTS NewsArticle (
                id STRING PRIMARY KEY,
                title STRING,
                url STRING,
                published_at TIMESTAMP,
                sentiment DOUBLE
            )
            "#,
        )?;

        // Create relationship tables
        // Artist collaborations
        conn.query(
            r#"
            CREATE REL TABLE IF NOT EXISTS COLLABORATED_WITH (
                FROM Artist TO Artist,
                track_id STRING,
                track_title STRING,
                collaboration_type STRING,
                year INT64
            )
            "#,
        )?;

        // Artist signed to label
        conn.query(
            r#"
            CREATE REL TABLE IF NOT EXISTS SIGNED_TO (
                FROM Artist TO Label,
                start_year INT64,
                end_year INT64,
                is_current BOOLEAN
            )
            "#,
        )?;

        // Artist performed on track
        conn.query(
            r#"
            CREATE REL TABLE IF NOT EXISTS PERFORMED (
                FROM Artist TO Track,
                role STRING,
                credit_position INT64
            )
            "#,
        )?;

        // Track appears on album
        conn.query(
            r#"
            CREATE REL TABLE IF NOT EXISTS APPEARS_ON (
                FROM Track TO Album,
                track_number INT64
            )
            "#,
        )?;

        // Artist mentioned in news
        conn.query(
            r#"
            CREATE REL TABLE IF NOT EXISTS MENTIONED_IN (
                FROM Artist TO NewsArticle,
                context_snippet STRING,
                sentiment DOUBLE,
                is_offense_related BOOLEAN
            )
            "#,
        )?;

        // Artist associations (group members, aliases, side projects)
        conn.query(
            r#"
            CREATE REL TABLE IF NOT EXISTS ASSOCIATED_WITH (
                FROM Artist TO Artist,
                relationship_type STRING,
                start_year INT64,
                end_year INT64
            )
            "#,
        )?;

        tracing::info!("Kùzu graph schema initialized");
        Ok(())
    }

    /// Add or update an artist node
    pub fn upsert_artist(&self, artist: &GraphArtist) -> Result<()> {
        let conn = self.connection()?;

        conn.query(&format!(
            r#"
            MERGE (a:Artist {{id: '{}'}})
            SET a.canonical_name = '{}',
                a.genres = {},
                a.country = {},
                a.formed_year = {},
                a.is_blocked = {},
                a.block_count = {}
            "#,
            artist.id,
            escape_string(&artist.canonical_name),
            format_string_array(&artist.genres),
            artist
                .country
                .as_ref()
                .map_or("NULL".to_string(), |c| format!("'{}'", escape_string(c))),
            artist
                .formed_year
                .map_or("NULL".to_string(), |y| y.to_string()),
            artist.is_blocked,
            artist.block_count,
        ))?;

        Ok(())
    }

    /// Add a collaboration relationship
    pub fn add_collaboration(&self, collab: &Collaboration) -> Result<()> {
        let conn = self.connection()?;

        conn.query(&format!(
            r#"
            MATCH (a1:Artist {{id: '{}'}}), (a2:Artist {{id: '{}'}})
            MERGE (a1)-[r:COLLABORATED_WITH {{track_id: '{}'}}]->(a2)
            SET r.track_title = '{}',
                r.collaboration_type = '{}',
                r.year = {}
            "#,
            collab.artist1_id,
            collab.artist2_id,
            collab.track_id,
            escape_string(&collab.track_title),
            collab.collaboration_type,
            collab.year.map_or("NULL".to_string(), |y| y.to_string()),
        ))?;

        Ok(())
    }

    /// Get artist's collaboration network
    pub fn get_artist_network(&self, artist_id: &str, depth: u32) -> Result<ArtistNetwork> {
        let conn = self.connection()?;

        // Get the center artist
        let center_result = conn.query(&format!(
            r#"
            MATCH (a:Artist {{id: '{}'}})
            RETURN a.id, a.canonical_name, a.is_blocked, a.genres
            "#,
            artist_id,
        ))?;

        let center = if let Some(row) = center_result.into_iter().next() {
            GraphArtistNode {
                id: value_to_string(&row[0]),
                name: value_to_string(&row[1]),
                is_blocked: value_to_bool(&row[2]),
                genres: vec![], // Would need proper array parsing
                collaboration_count: 0,
            }
        } else {
            return Err(anyhow::anyhow!("Artist not found"));
        };

        // Get collaborators within depth
        let collab_result = conn.query(&format!(
            r#"
            MATCH (a:Artist {{id: '{}'}})-[r:COLLABORATED_WITH*1..{}]-(b:Artist)
            RETURN DISTINCT b.id, b.canonical_name, b.is_blocked, COUNT(r) as collab_count
            ORDER BY collab_count DESC
            LIMIT 50
            "#,
            artist_id, depth,
        ))?;

        let mut nodes = vec![center.clone()];
        let mut edges = Vec::new();

        for row in collab_result {
            nodes.push(GraphArtistNode {
                id: value_to_string(&row[0]),
                name: value_to_string(&row[1]),
                is_blocked: value_to_bool(&row[2]),
                genres: vec![],
                collaboration_count: value_to_u32(&row[3]),
            });
        }

        // Get edges
        let edge_result = conn.query(&format!(
            r#"
            MATCH (a:Artist {{id: '{}'}})-[r:COLLABORATED_WITH]-(b:Artist)
            RETURN a.id, b.id, r.collaboration_type, COUNT(*) as track_count, MAX(r.year) as most_recent
            "#,
            artist_id,
        ))?;

        for row in edge_result {
            edges.push(CollaborationEdge {
                source_id: value_to_string(&row[0]),
                target_id: value_to_string(&row[1]),
                collaboration_type: value_to_string(&row[2]),
                track_count: value_to_u32(&row[3]),
                most_recent_year: value_to_i64_opt(&row[4]),
            });
        }

        let stats = NetworkStats {
            total_nodes: nodes.len() as u32,
            total_edges: edges.len() as u32,
            blocked_artists: nodes.iter().filter(|n| n.is_blocked).count() as u32,
        };

        Ok(ArtistNetwork {
            center_artist: center,
            nodes,
            edges,
            stats,
        })
    }

    /// Find shortest path between two artists
    pub fn find_path(&self, artist1_id: &str, artist2_id: &str) -> Result<Option<ArtistPath>> {
        let conn = self.connection()?;

        let result = conn.query(&format!(
            r#"
            MATCH p = shortestPath((a1:Artist {{id: '{}'}})-[*..6]-(a2:Artist {{id: '{}'}}))
            RETURN nodes(p), relationships(p)
            "#,
            artist1_id, artist2_id,
        ))?;

        if let Some(_row) = result.into_iter().next() {
            // Parse path nodes and relationships
            // Simplified for now
            Ok(Some(ArtistPath {
                artists: vec![artist1_id.to_string(), artist2_id.to_string()],
                collaborations: vec![],
                distance: 1,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get blocked artist network analysis
    pub fn analyze_blocked_network(
        &self,
        user_blocked_ids: &[String],
        max_distance: u32,
    ) -> Result<BlockedNetworkAnalysis> {
        let conn = self.connection()?;

        // For each blocked artist, find connected artists within distance
        let mut connected_artists = Vec::new();
        let mut risk_scores = std::collections::HashMap::new();

        for blocked_id in user_blocked_ids {
            let result = conn.query(&format!(
                r#"
                MATCH (blocked:Artist {{id: '{}'}})-[r:COLLABORATED_WITH*1..{}]-(connected:Artist)
                WHERE NOT connected.is_blocked
                RETURN DISTINCT connected.id, connected.canonical_name,
                       length(shortestPath((blocked)-[*]-(connected))) as distance,
                       COUNT(r) as connection_strength
                ORDER BY distance, connection_strength DESC
                LIMIT 20
                "#,
                blocked_id, max_distance,
            ))?;

            for row in result {
                let artist_id = value_to_string(&row[0]);
                let distance = value_to_u32(&row[2]);

                // Calculate risk score based on proximity and connection strength
                let strength = value_to_u32(&row[3]);
                let risk = (max_distance - distance + 1) as f64 * strength as f64;

                risk_scores
                    .entry(artist_id.clone())
                    .and_modify(|r: &mut f64| *r += risk)
                    .or_insert(risk);

                connected_artists.push(ConnectedArtist {
                    id: artist_id,
                    name: value_to_string(&row[1]),
                    distance,
                    connection_strength: strength,
                });
            }
        }

        // Deduplicate and sort by risk
        connected_artists.sort_by(|a, b| {
            let risk_a = risk_scores.get(&a.id).unwrap_or(&0.0);
            let risk_b = risk_scores.get(&b.id).unwrap_or(&0.0);
            risk_b
                .partial_cmp(risk_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        connected_artists.dedup_by(|a, b| a.id == b.id);

        Ok(BlockedNetworkAnalysis {
            blocked_artists_count: user_blocked_ids.len() as u32,
            connected_artists,
            risk_scores,
        })
    }

    /// Get graph database statistics
    pub fn get_stats(&self) -> Result<GraphStats> {
        let conn = self.connection()?;

        // Count artists
        let artist_result = conn.query("MATCH (a:Artist) RETURN COUNT(a)")?;
        let artist_count = artist_result
            .into_iter()
            .next()
            .map(|row| value_to_u32(&row[0]) as u64)
            .unwrap_or(0);

        // Count collaborations
        let collab_result = conn.query("MATCH ()-[r:COLLABORATED_WITH]->() RETURN COUNT(r)")?;
        let collaboration_count = collab_result
            .into_iter()
            .next()
            .map(|row| value_to_u32(&row[0]) as u64)
            .unwrap_or(0);

        // Count labels
        let label_result = conn.query("MATCH (l:Label) RETURN COUNT(l)")?;
        let label_count = label_result
            .into_iter()
            .next()
            .map(|row| value_to_u32(&row[0]) as u64)
            .unwrap_or(0);

        // Count tracks
        let track_result = conn.query("MATCH (t:Track) RETURN COUNT(t)")?;
        let track_count = track_result
            .into_iter()
            .next()
            .map(|row| value_to_u32(&row[0]) as u64)
            .unwrap_or(0);

        Ok(GraphStats {
            artist_count,
            collaboration_count,
            label_count,
            track_count,
        })
    }
}

/// Graph database statistics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct GraphStats {
    pub artist_count: u64,
    pub collaboration_count: u64,
    pub label_count: u64,
    pub track_count: u64,
}

// Helper functions
fn escape_string(s: &str) -> String {
    s.replace('\'', "\\'").replace('\\', "\\\\")
}

fn format_string_array(arr: &[String]) -> String {
    if arr.is_empty() {
        "[]".to_string()
    } else {
        format!(
            "[{}]",
            arr.iter()
                .map(|s| format!("'{}'", escape_string(s)))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// Convert kuzu Value to String
fn value_to_string(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Null(_) => String::new(),
        other => format!("{:?}", other),
    }
}

/// Convert kuzu Value to bool
fn value_to_bool(val: &Value) -> bool {
    match val {
        Value::Bool(b) => *b,
        Value::Null(_) => false,
        _ => false,
    }
}

/// Convert kuzu Value to u32
fn value_to_u32(val: &Value) -> u32 {
    match val {
        Value::Int64(i) => *i as u32,
        Value::Int32(i) => *i as u32,
        Value::Int16(i) => *i as u32,
        Value::Null(_) => 0,
        _ => 0,
    }
}

/// Convert kuzu Value to optional i64
fn value_to_i64_opt(val: &Value) -> Option<i64> {
    match val {
        Value::Int64(i) => Some(*i),
        Value::Int32(i) => Some(*i as i64),
        Value::Int16(i) => Some(*i as i64),
        Value::Null(_) => None,
        _ => None,
    }
}

/// Artist for graph operations
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

/// Collaboration record
#[derive(Debug, Clone)]
pub struct Collaboration {
    pub artist1_id: String,
    pub artist2_id: String,
    pub track_id: String,
    pub track_title: String,
    pub collaboration_type: String, // "feature", "producer", "songwriter", "remix"
    pub year: Option<i64>,
}

/// Artist node in network response
#[derive(Debug, Clone)]
pub struct GraphArtistNode {
    pub id: String,
    pub name: String,
    pub is_blocked: bool,
    pub genres: Vec<String>,
    pub collaboration_count: u32,
}

/// Collaboration edge in network
#[derive(Debug, Clone)]
pub struct CollaborationEdge {
    pub source_id: String,
    pub target_id: String,
    pub collaboration_type: String,
    pub track_count: u32,
    pub most_recent_year: Option<i64>,
}

/// Network statistics
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub total_nodes: u32,
    pub total_edges: u32,
    pub blocked_artists: u32,
}

/// Artist collaboration network
#[derive(Debug, Clone)]
pub struct ArtistNetwork {
    pub center_artist: GraphArtistNode,
    pub nodes: Vec<GraphArtistNode>,
    pub edges: Vec<CollaborationEdge>,
    pub stats: NetworkStats,
}

/// Path between artists
#[derive(Debug, Clone)]
pub struct ArtistPath {
    pub artists: Vec<String>,
    pub collaborations: Vec<String>,
    pub distance: u32,
}

/// Connected artist in blocked network
#[derive(Debug, Clone)]
pub struct ConnectedArtist {
    pub id: String,
    pub name: String,
    pub distance: u32,
    pub connection_strength: u32,
}

/// Blocked network analysis
#[derive(Debug, Clone)]
pub struct BlockedNetworkAnalysis {
    pub blocked_artists_count: u32,
    pub connected_artists: Vec<ConnectedArtist>,
    pub risk_scores: std::collections::HashMap<String, f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_kuzu_initialization() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_kuzu");
        let client = KuzuClient::new(db_path.to_str().unwrap()).unwrap();
        client.initialize_schema().unwrap();
    }
}
