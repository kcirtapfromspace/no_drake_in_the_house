//! Graph Sync Service
//!
//! Synchronizes data from PostgreSQL to Kùzu graph database.
//! Handles incremental and full syncs of artists, labels, and tracks.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::services::databases::{Collaboration, GraphArtist, KuzuClient};

/// Sync job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncJobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Type of sync operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncType {
    Full,
    Incremental,
    Artists,
    Labels,
    Collaborations,
}

/// Sync job record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncJob {
    pub id: Uuid,
    pub sync_type: SyncType,
    pub status: SyncJobStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub artists_synced: u32,
    pub collaborations_synced: u32,
    pub labels_synced: u32,
    pub errors: Vec<String>,
}

impl SyncJob {
    fn new(sync_type: SyncType) -> Self {
        Self {
            id: Uuid::new_v4(),
            sync_type,
            status: SyncJobStatus::Pending,
            started_at: None,
            completed_at: None,
            artists_synced: 0,
            collaborations_synced: 0,
            labels_synced: 0,
            errors: Vec::new(),
        }
    }
}

/// Sync statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncStats {
    pub total_artists_in_pg: u32,
    pub total_artists_in_graph: u32,
    pub total_collaborations: u32,
    pub total_labels: u32,
    pub last_full_sync: Option<DateTime<Utc>>,
    pub last_incremental_sync: Option<DateTime<Utc>>,
    pub pending_sync_count: u32,
}

/// Graph sync service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSyncConfig {
    /// Batch size for sync operations
    pub batch_size: usize,
    /// Enable automatic background sync
    pub auto_sync_enabled: bool,
    /// Interval for incremental syncs (seconds)
    pub incremental_sync_interval_secs: u64,
    /// Interval for full syncs (seconds)
    pub full_sync_interval_secs: u64,
}

impl Default for GraphSyncConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            auto_sync_enabled: true,
            incremental_sync_interval_secs: 3600, // 1 hour
            full_sync_interval_secs: 86400,       // 24 hours
        }
    }
}

/// Graph sync service
pub struct GraphSyncService {
    config: GraphSyncConfig,
    kuzu: Arc<KuzuClient>,
    pool: PgPool,
    current_job: Arc<RwLock<Option<SyncJob>>>,
    stats: Arc<RwLock<SyncStats>>,
}

impl GraphSyncService {
    /// Create a new graph sync service
    pub fn new(kuzu: Arc<KuzuClient>, pool: PgPool, config: GraphSyncConfig) -> Self {
        Self {
            config,
            kuzu,
            pool,
            current_job: Arc::new(RwLock::new(None)),
            stats: Arc::new(RwLock::new(SyncStats::default())),
        }
    }

    /// Initialize the graph schema
    pub fn initialize(&self) -> Result<()> {
        self.kuzu.initialize_schema()
    }

    /// Run a full sync of all data
    pub async fn run_full_sync(&self) -> Result<SyncJob> {
        let mut job = SyncJob::new(SyncType::Full);
        job.status = SyncJobStatus::Running;
        job.started_at = Some(Utc::now());

        {
            let mut current = self.current_job.write().await;
            *current = Some(job.clone());
        }

        tracing::info!("Starting full graph sync");

        // Sync artists
        match self.sync_artists(&mut job).await {
            Ok(count) => {
                job.artists_synced = count;
                tracing::info!(count = count, "Artists synced to graph");
            }
            Err(e) => {
                job.errors.push(format!("Artist sync failed: {}", e));
                tracing::error!(error = %e, "Failed to sync artists");
            }
        }

        // Sync labels
        match self.sync_labels(&mut job).await {
            Ok(count) => {
                job.labels_synced = count;
                tracing::info!(count = count, "Labels synced to graph");
            }
            Err(e) => {
                job.errors.push(format!("Label sync failed: {}", e));
                tracing::error!(error = %e, "Failed to sync labels");
            }
        }

        // Build collaboration edges
        match self.sync_collaborations(&mut job).await {
            Ok(count) => {
                job.collaborations_synced = count;
                tracing::info!(count = count, "Collaborations synced to graph");
            }
            Err(e) => {
                job.errors.push(format!("Collaboration sync failed: {}", e));
                tracing::error!(error = %e, "Failed to sync collaborations");
            }
        }

        job.status = if job.errors.is_empty() {
            SyncJobStatus::Completed
        } else {
            SyncJobStatus::Failed
        };
        job.completed_at = Some(Utc::now());

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_artists_in_graph = job.artists_synced;
            stats.total_collaborations = job.collaborations_synced;
            stats.total_labels = job.labels_synced;
            stats.last_full_sync = Some(Utc::now());
        }

        {
            let mut current = self.current_job.write().await;
            *current = Some(job.clone());
        }

        tracing::info!(
            artists = job.artists_synced,
            collaborations = job.collaborations_synced,
            labels = job.labels_synced,
            errors = job.errors.len(),
            "Full graph sync completed"
        );

        Ok(job)
    }

    /// Run an incremental sync (only new/changed data)
    pub async fn run_incremental_sync(&self, since: DateTime<Utc>) -> Result<SyncJob> {
        let mut job = SyncJob::new(SyncType::Incremental);
        job.status = SyncJobStatus::Running;
        job.started_at = Some(Utc::now());

        tracing::info!(since = %since, "Starting incremental graph sync");

        // Sync recently updated artists
        match self.sync_artists_since(&mut job, since).await {
            Ok(count) => {
                job.artists_synced = count;
            }
            Err(e) => {
                job.errors
                    .push(format!("Incremental artist sync failed: {}", e));
            }
        }

        job.status = if job.errors.is_empty() {
            SyncJobStatus::Completed
        } else {
            SyncJobStatus::Failed
        };
        job.completed_at = Some(Utc::now());

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.last_incremental_sync = Some(Utc::now());
        }

        Ok(job)
    }

    /// Sync all artists from PostgreSQL to Kùzu
    async fn sync_artists(&self, _job: &mut SyncJob) -> Result<u32> {
        let mut count = 0u32;
        let mut offset = 0i64;

        loop {
            // Fetch batch of artists from PostgreSQL
            let artists: Vec<ArtistRow> = sqlx::query_as(
                r#"
                SELECT
                    a.id,
                    a.name,
                    a.spotify_id,
                    COALESCE(array_agg(DISTINCT g.name) FILTER (WHERE g.name IS NOT NULL), '{}') as genres,
                    COUNT(DISTINCT uab.user_id) as block_count
                FROM artists a
                LEFT JOIN artist_genres ag ON a.id = ag.artist_id
                LEFT JOIN genres g ON ag.genre_id = g.id
                LEFT JOIN user_artist_blocks uab ON a.id = uab.artist_id
                GROUP BY a.id
                ORDER BY a.id
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(self.config.batch_size as i64)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch artists from PostgreSQL")?;

            if artists.is_empty() {
                break;
            }

            let batch_size = artists.len();

            // Insert each artist into Kùzu
            for artist in artists {
                let graph_artist = GraphArtist {
                    id: artist.id.to_string(),
                    canonical_name: artist.name,
                    genres: artist.genres.unwrap_or_default(),
                    country: None, // Would come from external data
                    formed_year: None,
                    is_blocked: artist.block_count > 0,
                    block_count: artist.block_count,
                };

                if let Err(e) = self.kuzu.upsert_artist(&graph_artist) {
                    tracing::warn!(artist_id = %artist.id, error = %e, "Failed to upsert artist");
                } else {
                    count += 1;
                }
            }

            offset += batch_size as i64;

            if batch_size < self.config.batch_size {
                break;
            }
        }

        Ok(count)
    }

    /// Sync artists updated since a specific time
    async fn sync_artists_since(&self, _job: &mut SyncJob, since: DateTime<Utc>) -> Result<u32> {
        let mut count = 0u32;

        // Fetch recently updated artists
        let artists: Vec<ArtistRow> = sqlx::query_as(
            r#"
            SELECT
                a.id,
                a.name,
                a.spotify_id,
                COALESCE(array_agg(DISTINCT g.name) FILTER (WHERE g.name IS NOT NULL), '{}') as genres,
                COUNT(DISTINCT uab.user_id) as block_count
            FROM artists a
            LEFT JOIN artist_genres ag ON a.id = ag.artist_id
            LEFT JOIN genres g ON ag.genre_id = g.id
            LEFT JOIN user_artist_blocks uab ON a.id = uab.artist_id
            WHERE a.updated_at > $1 OR a.created_at > $1
            GROUP BY a.id
            ORDER BY a.updated_at DESC
            LIMIT 10000
            "#,
        )
        .bind(since)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch updated artists")?;

        for artist in artists {
            let graph_artist = GraphArtist {
                id: artist.id.to_string(),
                canonical_name: artist.name,
                genres: artist.genres.unwrap_or_default(),
                country: None,
                formed_year: None,
                is_blocked: artist.block_count > 0,
                block_count: artist.block_count,
            };

            if self.kuzu.upsert_artist(&graph_artist).is_ok() {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Sync labels from PostgreSQL to Kùzu
    async fn sync_labels(&self, _job: &mut SyncJob) -> Result<u32> {
        // Labels table may not exist yet - this is a placeholder
        // Would sync from a labels table if available
        Ok(0)
    }

    /// Sync/build collaboration relationships
    async fn sync_collaborations(&self, _job: &mut SyncJob) -> Result<u32> {
        let mut count = 0u32;

        // Fetch tracks with multiple artists (collaborations)
        // This assumes there's a track_artists table or similar
        // For now, we'll try to find collaborations from a tracks table if it exists

        // Try to fetch collaboration data
        let collabs: Result<Vec<CollabRow>, _> = sqlx::query_as(
            r#"
            SELECT
                t.id as track_id,
                t.name as track_name,
                ta1.artist_id as artist1_id,
                ta2.artist_id as artist2_id,
                EXTRACT(YEAR FROM t.created_at)::INT as year
            FROM tracks t
            JOIN track_artists ta1 ON t.id = ta1.track_id
            JOIN track_artists ta2 ON t.id = ta2.track_id
            WHERE ta1.artist_id < ta2.artist_id
            LIMIT 100000
            "#,
        )
        .fetch_all(&self.pool)
        .await;

        match collabs {
            Ok(rows) => {
                for row in rows {
                    let collab = Collaboration {
                        artist1_id: row.artist1_id.to_string(),
                        artist2_id: row.artist2_id.to_string(),
                        track_id: row.track_id.to_string(),
                        track_title: row.track_name,
                        collaboration_type: "feature".to_string(),
                        year: row.year.map(|y| y as i64),
                    };

                    if self.kuzu.add_collaboration(&collab).is_ok() {
                        count += 1;
                    }
                }
            }
            Err(_) => {
                // Table doesn't exist or no collaborations - this is fine
                tracing::debug!("No collaboration data found (tracks table may not exist)");
            }
        }

        Ok(count)
    }

    /// Sync a single artist by ID
    pub async fn sync_artist(&self, artist_id: Uuid) -> Result<()> {
        let artist: ArtistRow = sqlx::query_as(
            r#"
            SELECT
                a.id,
                a.name,
                a.spotify_id,
                COALESCE(array_agg(DISTINCT g.name) FILTER (WHERE g.name IS NOT NULL), '{}') as genres,
                COUNT(DISTINCT uab.user_id) as block_count
            FROM artists a
            LEFT JOIN artist_genres ag ON a.id = ag.artist_id
            LEFT JOIN genres g ON ag.genre_id = g.id
            LEFT JOIN user_artist_blocks uab ON a.id = uab.artist_id
            WHERE a.id = $1
            GROUP BY a.id
            "#,
        )
        .bind(artist_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch artist")?;

        let graph_artist = GraphArtist {
            id: artist.id.to_string(),
            canonical_name: artist.name,
            genres: artist.genres.unwrap_or_default(),
            country: None,
            formed_year: None,
            is_blocked: artist.block_count > 0,
            block_count: artist.block_count,
        };

        self.kuzu.upsert_artist(&graph_artist)?;
        Ok(())
    }

    /// Get current sync status
    pub async fn get_status(&self) -> (Option<SyncJob>, SyncStats) {
        let job = self.current_job.read().await.clone();
        let stats = self.stats.read().await.clone();
        (job, stats)
    }

    /// Get sync statistics
    pub async fn get_stats(&self) -> SyncStats {
        self.stats.read().await.clone()
    }

    /// Check if a sync is currently running
    pub async fn is_syncing(&self) -> bool {
        self.current_job
            .read()
            .await
            .as_ref()
            .map(|j| j.status == SyncJobStatus::Running)
            .unwrap_or(false)
    }
}

/// Artist row from PostgreSQL
#[derive(Debug, sqlx::FromRow)]
struct ArtistRow {
    id: Uuid,
    name: String,
    spotify_id: Option<String>,
    genres: Option<Vec<String>>,
    block_count: i64,
}

/// Collaboration row from PostgreSQL
#[derive(Debug, sqlx::FromRow)]
struct CollabRow {
    track_id: Uuid,
    track_name: String,
    artist1_id: Uuid,
    artist2_id: Uuid,
    year: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_job_creation() {
        let job = SyncJob::new(SyncType::Full);
        assert_eq!(job.status, SyncJobStatus::Pending);
        assert_eq!(job.artists_synced, 0);
    }

    #[test]
    fn test_default_config() {
        let config = GraphSyncConfig::default();
        assert_eq!(config.batch_size, 1000);
        assert!(config.auto_sync_enabled);
    }
}
