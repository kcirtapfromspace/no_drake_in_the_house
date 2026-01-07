//! Collaboration Service
//!
//! Builds and manages collaboration relationships between artists.
//! Detects collaborations from tracks, features, and production credits.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

use crate::services::databases::{KuzuClient, Collaboration};

/// Types of collaborations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CollaborationType {
    /// Featured artist on a track
    Feature,
    /// Producer of the track
    Producer,
    /// Songwriter credit
    Songwriter,
    /// Remix credit
    Remix,
    /// Sample credit
    Sample,
    /// Guest appearance
    Guest,
    /// Unknown/other
    Other,
}

impl std::fmt::Display for CollaborationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollaborationType::Feature => write!(f, "feature"),
            CollaborationType::Producer => write!(f, "producer"),
            CollaborationType::Songwriter => write!(f, "songwriter"),
            CollaborationType::Remix => write!(f, "remix"),
            CollaborationType::Sample => write!(f, "sample"),
            CollaborationType::Guest => write!(f, "guest"),
            CollaborationType::Other => write!(f, "other"),
        }
    }
}

/// A track collaboration record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackCollaboration {
    /// Track ID
    pub track_id: Uuid,
    /// Track title
    pub track_title: String,
    /// Primary artist ID
    pub primary_artist_id: Uuid,
    /// Collaborating artist ID
    pub collaborator_id: Uuid,
    /// Type of collaboration
    pub collaboration_type: CollaborationType,
    /// Year of release
    pub year: Option<i32>,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Collaboration statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CollaborationStats {
    pub total_collaborations: u32,
    pub unique_artist_pairs: u32,
    pub most_common_type: Option<String>,
    pub top_collaborators: Vec<(Uuid, u32)>,
}

/// Collaboration builder configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationBuilderConfig {
    /// Minimum collaborations to consider as connection
    pub min_collaborations: u32,
    /// Maximum age of collaborations to consider (years)
    pub max_age_years: Option<u32>,
    /// Types of collaborations to include
    pub include_types: Vec<CollaborationType>,
}

impl Default for CollaborationBuilderConfig {
    fn default() -> Self {
        Self {
            min_collaborations: 1,
            max_age_years: None,
            include_types: vec![
                CollaborationType::Feature,
                CollaborationType::Producer,
                CollaborationType::Songwriter,
                CollaborationType::Remix,
                CollaborationType::Guest,
            ],
        }
    }
}

/// Collaboration builder service
pub struct CollaborationBuilder {
    config: CollaborationBuilderConfig,
    kuzu: Arc<KuzuClient>,
    pool: PgPool,
}

impl CollaborationBuilder {
    /// Create a new collaboration builder
    pub fn new(kuzu: Arc<KuzuClient>, pool: PgPool, config: CollaborationBuilderConfig) -> Self {
        Self {
            config,
            kuzu,
            pool,
        }
    }

    /// Build collaboration graph from track data
    pub async fn build_from_tracks(&self) -> Result<CollaborationStats> {
        let mut stats = CollaborationStats::default();
        let mut artist_pairs: HashSet<(Uuid, Uuid)> = HashSet::new();
        let mut type_counts: HashMap<CollaborationType, u32> = HashMap::new();
        let mut collaborator_counts: HashMap<Uuid, u32> = HashMap::new();

        // Fetch tracks with multiple artists
        let tracks = self.fetch_multi_artist_tracks().await?;

        for track in tracks {
            // Create collaboration edges for each pair
            for collab in &track.collaborations {
                let (a1, a2) = if collab.primary_artist_id < collab.collaborator_id {
                    (collab.primary_artist_id, collab.collaborator_id)
                } else {
                    (collab.collaborator_id, collab.primary_artist_id)
                };

                artist_pairs.insert((a1, a2));
                *type_counts.entry(collab.collaboration_type).or_insert(0) += 1;
                *collaborator_counts.entry(collab.collaborator_id).or_insert(0) += 1;

                // Add to Kùzu
                let kuzu_collab = Collaboration {
                    artist1_id: a1.to_string(),
                    artist2_id: a2.to_string(),
                    track_id: collab.track_id.to_string(),
                    track_title: collab.track_title.clone(),
                    collaboration_type: collab.collaboration_type.to_string(),
                    year: collab.year.map(|y| y as i64),
                };

                if self.kuzu.add_collaboration(&kuzu_collab).is_ok() {
                    stats.total_collaborations += 1;
                }
            }
        }

        stats.unique_artist_pairs = artist_pairs.len() as u32;

        // Find most common type
        stats.most_common_type = type_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(t, _)| t.to_string());

        // Top collaborators
        let mut top: Vec<_> = collaborator_counts.into_iter().collect();
        top.sort_by(|a, b| b.1.cmp(&a.1));
        stats.top_collaborators = top.into_iter().take(10).collect();

        tracing::info!(
            total = stats.total_collaborations,
            pairs = stats.unique_artist_pairs,
            "Built collaboration graph"
        );

        Ok(stats)
    }

    /// Fetch tracks with multiple artists
    async fn fetch_multi_artist_tracks(&self) -> Result<Vec<TrackWithCollaborations>> {
        // Try to fetch from a tracks/track_artists table if available
        // This is a placeholder implementation

        // First, check if the tables exist
        let result: Result<Vec<TrackArtistRow>, _> = sqlx::query_as(
            r#"
            SELECT
                t.id as track_id,
                t.name as track_name,
                ta.artist_id,
                ta.is_primary,
                ta.role,
                EXTRACT(YEAR FROM t.created_at)::INT as year
            FROM tracks t
            JOIN track_artists ta ON t.id = ta.track_id
            WHERE t.id IN (
                SELECT track_id
                FROM track_artists
                GROUP BY track_id
                HAVING COUNT(*) > 1
            )
            ORDER BY t.id
            LIMIT 100000
            "#,
        )
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => self.build_collaborations_from_rows(rows),
            Err(_) => {
                // Tables don't exist - try alternative approach or return empty
                tracing::debug!("No tracks/track_artists tables found");
                Ok(Vec::new())
            }
        }
    }

    /// Build collaboration records from raw rows
    fn build_collaborations_from_rows(&self, rows: Vec<TrackArtistRow>) -> Result<Vec<TrackWithCollaborations>> {
        let mut tracks: HashMap<Uuid, TrackWithCollaborations> = HashMap::new();

        for row in rows {
            let track = tracks.entry(row.track_id).or_insert_with(|| TrackWithCollaborations {
                track_id: row.track_id,
                track_name: row.track_name.clone(),
                artists: Vec::new(),
                collaborations: Vec::new(),
            });

            track.artists.push(TrackArtist {
                artist_id: row.artist_id,
                is_primary: row.is_primary,
                role: row.role.clone(),
            });
        }

        // Build collaboration pairs
        for track in tracks.values_mut() {
            let primary = track.artists.iter().find(|a| a.is_primary);
            let primary_id = match primary {
                Some(p) => p.artist_id,
                None => track.artists.first().map(|a| a.artist_id).unwrap_or(Uuid::nil()),
            };

            for artist in &track.artists {
                if artist.artist_id != primary_id {
                    let collab_type = match artist.role.as_deref() {
                        Some("producer") => CollaborationType::Producer,
                        Some("songwriter") | Some("writer") => CollaborationType::Songwriter,
                        Some("remix") | Some("remixer") => CollaborationType::Remix,
                        Some("feature") | Some("featured") => CollaborationType::Feature,
                        _ => CollaborationType::Feature,
                    };

                    track.collaborations.push(TrackCollaboration {
                        track_id: track.track_id,
                        track_title: track.track_name.clone(),
                        primary_artist_id: primary_id,
                        collaborator_id: artist.artist_id,
                        collaboration_type: collab_type,
                        year: None, // Would come from track data
                        metadata: None,
                    });
                }
            }
        }

        Ok(tracks.into_values().collect())
    }

    /// Add a single collaboration manually
    pub fn add_collaboration(&self, collab: &TrackCollaboration) -> Result<()> {
        let kuzu_collab = Collaboration {
            artist1_id: collab.primary_artist_id.to_string(),
            artist2_id: collab.collaborator_id.to_string(),
            track_id: collab.track_id.to_string(),
            track_title: collab.track_title.clone(),
            collaboration_type: collab.collaboration_type.to_string(),
            year: collab.year.map(|y| y as i64),
        };

        self.kuzu.add_collaboration(&kuzu_collab)
    }

    /// Import collaborations from external data (e.g., MusicBrainz)
    pub async fn import_external_collaborations(
        &self,
        collaborations: Vec<ExternalCollaboration>,
    ) -> Result<u32> {
        let mut imported = 0u32;

        for ext_collab in collaborations {
            // Look up artist IDs
            let artist1_id = self.resolve_artist_id(&ext_collab.artist1_name).await;
            let artist2_id = self.resolve_artist_id(&ext_collab.artist2_name).await;

            if let (Some(a1), Some(a2)) = (artist1_id, artist2_id) {
                let collab = TrackCollaboration {
                    track_id: ext_collab.track_id.unwrap_or_else(Uuid::new_v4),
                    track_title: ext_collab.track_title,
                    primary_artist_id: a1,
                    collaborator_id: a2,
                    collaboration_type: parse_collaboration_type(&ext_collab.collaboration_type),
                    year: ext_collab.year,
                    metadata: ext_collab.metadata,
                };

                if self.add_collaboration(&collab).is_ok() {
                    imported += 1;
                }
            }
        }

        Ok(imported)
    }

    /// Resolve artist name to ID
    async fn resolve_artist_id(&self, name: &str) -> Option<Uuid> {
        sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM artists WHERE LOWER(name) = LOWER($1) LIMIT 1"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten()
    }

    /// Get collaboration statistics for an artist
    pub async fn get_artist_collaboration_stats(&self, artist_id: Uuid) -> Result<ArtistCollaborationStats> {
        // Query Kùzu for collaboration data
        let network = self.kuzu.get_artist_network(&artist_id.to_string(), 1)?;

        let total_collaborations = network.edges.len() as u32;
        let unique_collaborators = network.nodes.len().saturating_sub(1) as u32; // Exclude self

        // Count by type
        let mut type_breakdown: HashMap<String, u32> = HashMap::new();
        for edge in &network.edges {
            *type_breakdown.entry(edge.collaboration_type.clone()).or_insert(0) += 1;
        }

        // Top collaborators (by track count)
        let mut collaborator_counts: Vec<(String, u32)> = network.edges
            .iter()
            .filter(|e| e.source_id == artist_id.to_string())
            .map(|e| (e.target_id.clone(), e.track_count))
            .collect();
        collaborator_counts.sort_by(|a, b| b.1.cmp(&a.1));
        let top_collaborators: Vec<_> = collaborator_counts.into_iter().take(5).collect();

        Ok(ArtistCollaborationStats {
            artist_id,
            total_collaborations,
            unique_collaborators,
            type_breakdown,
            top_collaborators,
            blocked_collaborators: network.stats.blocked_artists,
        })
    }
}

/// Collaboration service (wrapper for higher-level operations)
pub struct CollaborationService {
    builder: CollaborationBuilder,
}

impl CollaborationService {
    pub fn new(kuzu: Arc<KuzuClient>, pool: PgPool) -> Self {
        Self {
            builder: CollaborationBuilder::new(kuzu, pool, CollaborationBuilderConfig::default()),
        }
    }

    pub fn with_config(kuzu: Arc<KuzuClient>, pool: PgPool, config: CollaborationBuilderConfig) -> Self {
        Self {
            builder: CollaborationBuilder::new(kuzu, pool, config),
        }
    }

    pub fn builder(&self) -> &CollaborationBuilder {
        &self.builder
    }

    pub async fn rebuild_all(&self) -> Result<CollaborationStats> {
        self.builder.build_from_tracks().await
    }

    pub async fn get_artist_stats(&self, artist_id: Uuid) -> Result<ArtistCollaborationStats> {
        self.builder.get_artist_collaboration_stats(artist_id).await
    }
}

/// External collaboration data for import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalCollaboration {
    pub artist1_name: String,
    pub artist2_name: String,
    pub track_title: String,
    pub track_id: Option<Uuid>,
    pub collaboration_type: String,
    pub year: Option<i32>,
    pub source: String, // e.g., "musicbrainz", "discogs"
    pub metadata: Option<serde_json::Value>,
}

/// Artist collaboration statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistCollaborationStats {
    pub artist_id: Uuid,
    pub total_collaborations: u32,
    pub unique_collaborators: u32,
    pub type_breakdown: HashMap<String, u32>,
    pub top_collaborators: Vec<(String, u32)>,
    pub blocked_collaborators: u32,
}

/// Track with collaborations
#[derive(Debug, Clone)]
struct TrackWithCollaborations {
    track_id: Uuid,
    track_name: String,
    artists: Vec<TrackArtist>,
    collaborations: Vec<TrackCollaboration>,
}

/// Track artist record
#[derive(Debug, Clone)]
struct TrackArtist {
    artist_id: Uuid,
    is_primary: bool,
    role: Option<String>,
}

/// Track artist row from database
#[derive(Debug, sqlx::FromRow)]
struct TrackArtistRow {
    track_id: Uuid,
    track_name: String,
    artist_id: Uuid,
    is_primary: bool,
    role: Option<String>,
    year: Option<i32>,
}

/// Parse collaboration type from string
fn parse_collaboration_type(s: &str) -> CollaborationType {
    match s.to_lowercase().as_str() {
        "feature" | "featured" | "feat" | "ft" => CollaborationType::Feature,
        "producer" | "produced" => CollaborationType::Producer,
        "songwriter" | "writer" | "written" => CollaborationType::Songwriter,
        "remix" | "remixer" | "remixed" => CollaborationType::Remix,
        "sample" | "sampled" => CollaborationType::Sample,
        "guest" => CollaborationType::Guest,
        _ => CollaborationType::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collaboration_type_display() {
        assert_eq!(CollaborationType::Feature.to_string(), "feature");
        assert_eq!(CollaborationType::Producer.to_string(), "producer");
    }

    #[test]
    fn test_parse_collaboration_type() {
        assert_eq!(parse_collaboration_type("feature"), CollaborationType::Feature);
        assert_eq!(parse_collaboration_type("feat"), CollaborationType::Feature);
        assert_eq!(parse_collaboration_type("producer"), CollaborationType::Producer);
        assert_eq!(parse_collaboration_type("unknown"), CollaborationType::Other);
    }

    #[test]
    fn test_default_config() {
        let config = CollaborationBuilderConfig::default();
        assert_eq!(config.min_collaborations, 1);
        assert!(config.include_types.contains(&CollaborationType::Feature));
    }
}
