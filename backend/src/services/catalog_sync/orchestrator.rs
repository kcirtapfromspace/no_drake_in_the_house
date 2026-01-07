//! Catalog Sync Orchestrator
//!
//! Coordinates multi-platform catalog synchronization, managing:
//! - Parallel sync jobs across platforms
//! - Rate limit coordination
//! - Progress tracking and checkpointing
//! - Identity resolution coordination

use super::identity_resolver::{CanonicalArtist, CrossPlatformIdentityResolver, IdentityMatch};
use super::traits::*;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

/// Catalog sync orchestrator
pub struct CatalogSyncOrchestrator {
    /// Platform workers
    workers: HashMap<Platform, Arc<dyn PlatformCatalogWorker + Send + Sync>>,
    /// Identity resolver
    identity_resolver: Arc<CrossPlatformIdentityResolver>,
    /// Active sync runs
    active_runs: Arc<RwLock<HashMap<Uuid, SyncRunState>>>,
    /// Progress broadcast channel
    progress_tx: broadcast::Sender<SyncProgress>,
    /// Canonical artist cache
    canonical_artists: Arc<RwLock<Vec<CanonicalArtist>>>,
}

/// State of a sync run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRunState {
    pub run_id: Uuid,
    pub platform: Platform,
    pub sync_type: SyncType,
    pub status: SyncStatus,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub progress: SyncProgress,
    pub checkpoint: Option<SyncCheckpoint>,
}

/// Overall sync status across all platforms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallSyncStatus {
    pub platforms: HashMap<Platform, PlatformSyncStatus>,
    pub total_artists: u32,
    pub last_full_sync: Option<DateTime<Utc>>,
    pub last_incremental_sync: Option<DateTime<Utc>>,
}

/// Status for a single platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformSyncStatus {
    pub platform: Platform,
    pub is_healthy: bool,
    pub last_sync: Option<DateTime<Utc>>,
    pub last_sync_result: Option<SyncResult>,
    pub current_run: Option<SyncRunState>,
    pub artists_synced: u32,
    pub rate_limit_status: RateLimitStatus,
}

/// Rate limit status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStatus {
    pub requests_remaining: u32,
    pub window_reset_at: Option<DateTime<Utc>>,
    pub is_throttled: bool,
}

/// Sync trigger request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncTriggerRequest {
    pub platforms: Vec<Platform>,
    pub sync_type: SyncType,
    pub priority: SyncPriority,
    pub artist_ids: Option<Vec<Uuid>>,
}

/// Sync priority
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SyncPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Sync run history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRunHistory {
    pub run_id: Uuid,
    pub platform: Platform,
    pub sync_type: SyncType,
    pub status: SyncStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<SyncResult>,
    pub error: Option<String>,
}

impl CatalogSyncOrchestrator {
    /// Create a new orchestrator
    pub fn new(identity_resolver: CrossPlatformIdentityResolver) -> Self {
        let (progress_tx, _) = broadcast::channel(100);

        Self {
            workers: HashMap::new(),
            identity_resolver: Arc::new(identity_resolver),
            active_runs: Arc::new(RwLock::new(HashMap::new())),
            progress_tx,
            canonical_artists: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a platform worker
    pub fn register_worker<W>(&mut self, worker: W)
    where
        W: PlatformCatalogWorker + Send + Sync + 'static,
    {
        let platform = worker.platform();
        self.workers.insert(platform, Arc::new(worker));
    }

    /// Get the progress broadcast receiver
    pub fn subscribe_progress(&self) -> broadcast::Receiver<SyncProgress> {
        self.progress_tx.subscribe()
    }

    /// Get overall sync status
    pub async fn get_status(&self) -> Result<OverallSyncStatus> {
        let mut platforms = HashMap::new();
        let active_runs = self.active_runs.read().await;

        for (platform, worker) in &self.workers {
            let is_healthy = worker.health_check().await.unwrap_or(false);
            let current_run = active_runs
                .values()
                .find(|r| &r.platform == platform)
                .cloned();

            let rate_config = worker.rate_limit_config();
            let rate_limit_status = RateLimitStatus {
                requests_remaining: rate_config.requests_per_window,
                window_reset_at: None,
                is_throttled: false,
            };

            platforms.insert(
                platform.clone(),
                PlatformSyncStatus {
                    platform: platform.clone(),
                    is_healthy,
                    last_sync: None,
                    last_sync_result: None,
                    current_run,
                    artists_synced: 0,
                    rate_limit_status,
                },
            );
        }

        Ok(OverallSyncStatus {
            platforms,
            total_artists: self.canonical_artists.read().await.len() as u32,
            last_full_sync: None,
            last_incremental_sync: None,
        })
    }

    /// Trigger a sync
    pub async fn trigger_sync(&self, request: SyncTriggerRequest) -> Result<Vec<Uuid>> {
        let mut run_ids = Vec::new();

        for platform in &request.platforms {
            if let Some(worker) = self.workers.get(platform) {
                let run_id = self
                    .start_platform_sync(worker.clone(), request.sync_type.clone())
                    .await?;
                run_ids.push(run_id);
            } else {
                tracing::warn!("No worker registered for platform {:?}", platform);
            }
        }

        Ok(run_ids)
    }

    /// Start sync for a single platform
    async fn start_platform_sync(
        &self,
        worker: Arc<dyn PlatformCatalogWorker + Send + Sync>,
        sync_type: SyncType,
    ) -> Result<Uuid> {
        let run_id = Uuid::new_v4();
        let platform = worker.platform();
        let started_at = Utc::now();

        // Create initial run state
        let run_state = SyncRunState {
            run_id,
            platform: platform.clone(),
            sync_type: sync_type.clone(),
            status: SyncStatus::Running,
            started_at,
            updated_at: started_at,
            progress: SyncProgress {
                platform: platform.clone(),
                sync_run_id: run_id,
                status: SyncStatus::Running,
                total_items: None,
                items_processed: 0,
                errors: 0,
                started_at,
                updated_at: started_at,
                estimated_completion: None,
            },
            checkpoint: None,
        };

        // Register the run
        {
            let mut active_runs = self.active_runs.write().await;
            active_runs.insert(run_id, run_state);
        }

        // Spawn the sync task
        let progress_tx = self.progress_tx.clone();
        let active_runs = self.active_runs.clone();
        let identity_resolver = self.identity_resolver.clone();
        let canonical_artists = self.canonical_artists.clone();

        tokio::spawn(async move {
            let progress_callback = Box::new({
                let progress_tx = progress_tx.clone();
                let active_runs = active_runs.clone();
                let run_id = run_id;
                move |progress: SyncProgress| {
                    // Update active run state
                    let active_runs = active_runs.clone();
                    let progress_clone = progress.clone();
                    tokio::spawn(async move {
                        let mut runs = active_runs.write().await;
                        if let Some(state) = runs.get_mut(&run_id) {
                            state.status = progress_clone.status.clone();
                            state.progress = progress_clone;
                            state.updated_at = Utc::now();
                        }
                    });

                    // Broadcast progress
                    let _ = progress_tx.send(progress);
                }
            });

            let result = match sync_type {
                SyncType::Full => worker.sync_full(progress_callback).await,
                SyncType::Incremental | SyncType::Targeted => worker.sync_incremental(None, progress_callback).await,
            };

            // Handle result
            match result {
                Ok(sync_result) => {
                    tracing::info!(
                        "Platform {:?} sync completed: {} artists processed",
                        platform,
                        sync_result.artists_processed
                    );
                }
                Err(e) => {
                    tracing::error!("Platform {:?} sync failed: {}", platform, e);
                    let mut runs = active_runs.write().await;
                    if let Some(state) = runs.get_mut(&run_id) {
                        state.status = SyncStatus::Failed;
                        state.updated_at = Utc::now();
                    }
                }
            }

            // Remove from active runs
            let mut runs = active_runs.write().await;
            runs.remove(&run_id);
        });

        Ok(run_id)
    }

    /// Get active sync runs
    pub async fn get_active_runs(&self) -> Vec<SyncRunState> {
        self.active_runs.read().await.values().cloned().collect()
    }

    /// Get sync run by ID
    pub async fn get_run(&self, run_id: Uuid) -> Option<SyncRunState> {
        self.active_runs.read().await.get(&run_id).cloned()
    }

    /// Cancel a sync run
    pub async fn cancel_run(&self, run_id: Uuid) -> Result<bool> {
        let mut runs = self.active_runs.write().await;
        if let Some(state) = runs.get_mut(&run_id) {
            state.status = SyncStatus::Cancelled;
            state.updated_at = Utc::now();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Search for an artist across all platforms
    pub async fn search_artist_all_platforms(
        &self,
        query: &str,
        limit_per_platform: u32,
    ) -> Result<HashMap<Platform, Vec<PlatformArtist>>> {
        let mut results = HashMap::new();

        for (platform, worker) in &self.workers {
            match worker.search_artist(query, limit_per_platform).await {
                Ok(artists) => {
                    results.insert(platform.clone(), artists);
                }
                Err(e) => {
                    tracing::warn!("Search failed on {:?}: {}", platform, e);
                    results.insert(platform.clone(), vec![]);
                }
            }
        }

        Ok(results)
    }

    /// Resolve an artist identity and add to catalog
    pub async fn resolve_and_add_artist(
        &self,
        platform_artist: &PlatformArtist,
    ) -> Result<IdentityMatch> {
        let existing = self.canonical_artists.read().await;
        let identity_match = self
            .identity_resolver
            .resolve(platform_artist, &existing)
            .await?;

        // Add or update canonical artist
        if identity_match.method == super::identity_resolver::MatchMethod::NewArtist {
            let mut artists = self.canonical_artists.write().await;
            artists.push(identity_match.artist.clone());
        }

        Ok(identity_match)
    }

    /// Get all canonical artists
    pub async fn get_canonical_artists(&self) -> Vec<CanonicalArtist> {
        self.canonical_artists.read().await.clone()
    }

    /// Manually merge two artists
    pub async fn merge_artists(
        &self,
        primary_id: Uuid,
        secondary_id: Uuid,
    ) -> Result<CanonicalArtist> {
        let mut artists = self.canonical_artists.write().await;

        let primary_idx = artists
            .iter()
            .position(|a| a.id == primary_id)
            .context("Primary artist not found")?;

        let secondary_idx = artists
            .iter()
            .position(|a| a.id == secondary_id)
            .context("Secondary artist not found")?;

        let secondary = artists.remove(if secondary_idx > primary_idx {
            secondary_idx
        } else {
            // Adjust index after removal
            secondary_idx
        });

        let primary = &artists[if secondary_idx > primary_idx {
            primary_idx
        } else {
            primary_idx - 1
        }];

        let merged = self.identity_resolver.merge_artists(primary, &secondary);

        // Update in place
        let idx = if secondary_idx > primary_idx {
            primary_idx
        } else {
            primary_idx - 1
        };
        artists[idx] = merged.clone();

        Ok(merged)
    }

    /// Health check all platforms
    pub async fn health_check_all(&self) -> HashMap<Platform, bool> {
        let mut results = HashMap::new();

        for (platform, worker) in &self.workers {
            let is_healthy = worker.health_check().await.unwrap_or(false);
            results.insert(platform.clone(), is_healthy);
        }

        results
    }

    /// Get platform worker
    pub fn get_worker(&self, platform: &Platform) -> Option<Arc<dyn PlatformCatalogWorker + Send + Sync>> {
        self.workers.get(platform).cloned()
    }

    /// Sync a specific artist across all platforms
    pub async fn sync_artist_all_platforms(
        &self,
        canonical_artist: &CanonicalArtist,
    ) -> Result<HashMap<Platform, PlatformArtist>> {
        let mut results = HashMap::new();

        // First, try to fetch from platforms where we have IDs
        for (platform, platform_id) in &canonical_artist.platform_ids {
            if let Some(worker) = self.workers.get(platform) {
                if let Ok(Some(artist)) = worker.get_artist(platform_id).await {
                    results.insert(platform.clone(), artist);
                }
            }
        }

        // Search on platforms where we don't have IDs
        for (platform, worker) in &self.workers {
            if !canonical_artist.platform_ids.contains_key(platform) {
                if let Ok(artists) = worker.search_artist(&canonical_artist.name, 5).await {
                    // Try to find a match
                    for artist in artists {
                        let score = self.quick_match_score(&artist, canonical_artist);
                        if score >= 0.85 {
                            results.insert(platform.clone(), artist);
                            break;
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Quick match score for artist lookup
    fn quick_match_score(
        &self,
        platform_artist: &PlatformArtist,
        canonical: &CanonicalArtist,
    ) -> f64 {
        let name_match = if platform_artist.name.to_lowercase() == canonical.name.to_lowercase() {
            1.0
        } else {
            0.0
        };

        // Could add genre matching here too
        name_match
    }
}

/// Builder for CatalogSyncOrchestrator
pub struct OrchestratorBuilder {
    identity_resolver: Option<CrossPlatformIdentityResolver>,
    workers: Vec<Box<dyn PlatformCatalogWorker + Send + Sync>>,
}

impl OrchestratorBuilder {
    pub fn new() -> Self {
        Self {
            identity_resolver: None,
            workers: Vec::new(),
        }
    }

    pub fn with_identity_resolver(mut self, resolver: CrossPlatformIdentityResolver) -> Self {
        self.identity_resolver = Some(resolver);
        self
    }

    pub fn with_worker<W>(mut self, worker: W) -> Self
    where
        W: PlatformCatalogWorker + Send + Sync + 'static,
    {
        self.workers.push(Box::new(worker));
        self
    }

    pub fn build(self) -> Result<CatalogSyncOrchestrator> {
        let resolver = self
            .identity_resolver
            .unwrap_or_else(|| CrossPlatformIdentityResolver::new("NoDrake", "1.0", "admin@example.com"));

        let mut orchestrator = CatalogSyncOrchestrator::new(resolver);

        for worker in self.workers {
            let platform = worker.platform();
            orchestrator.workers.insert(platform, Arc::from(worker));
        }

        Ok(orchestrator)
    }
}

impl Default for OrchestratorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let resolver = CrossPlatformIdentityResolver::new("test", "1.0", "test@example.com");
        let orchestrator = CatalogSyncOrchestrator::new(resolver);

        assert!(orchestrator.workers.is_empty());
        assert!(orchestrator.get_active_runs().await.is_empty());
    }

    #[tokio::test]
    async fn test_get_status() {
        let resolver = CrossPlatformIdentityResolver::new("test", "1.0", "test@example.com");
        let orchestrator = CatalogSyncOrchestrator::new(resolver);

        let status = orchestrator.get_status().await.unwrap();
        assert!(status.platforms.is_empty());
        assert_eq!(status.total_artists, 0);
    }
}
