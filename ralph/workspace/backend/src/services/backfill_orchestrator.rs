//! Backfill Orchestrator Service
//!
//! Coordinates offense discovery for artists by searching news sources.
//! Uses the news pipeline to search for and classify artist offenses.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::news_pipeline::NewsPipelineOrchestrator;

/// Backfill progress tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackfillProgress {
    pub artists_total: usize,
    pub artists_processed: usize,
    pub offenses_found: usize,
    pub errors: usize,
    pub started_at: Option<DateTime<Utc>>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Backfill result summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackfillResult {
    pub artists_processed: usize,
    pub offenses_created: usize,
    pub articles_found: usize,
    pub errors: usize,
    pub duration_seconds: f64,
}

/// Artist record for backfill
#[derive(Debug, Clone, sqlx::FromRow)]
struct ArtistForBackfill {
    id: Uuid,
    canonical_name: String,
    last_offense_search: Option<DateTime<Utc>>,
}

/// Backfill orchestrator for offense discovery
pub struct BackfillOrchestrator {
    db_pool: PgPool,
    news_pipeline: Option<Arc<NewsPipelineOrchestrator>>,
    /// Current progress
    progress: Arc<RwLock<BackfillProgress>>,
    /// Is currently running
    is_running: Arc<RwLock<bool>>,
}

impl BackfillOrchestrator {
    /// Create a new backfill orchestrator
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool,
            news_pipeline: None,
            progress: Arc::new(RwLock::new(BackfillProgress::default())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Create with news pipeline for offense searching
    pub fn with_news_pipeline(
        db_pool: PgPool,
        news_pipeline: Arc<NewsPipelineOrchestrator>,
    ) -> Self {
        Self {
            db_pool,
            news_pipeline: Some(news_pipeline),
            progress: Arc::new(RwLock::new(BackfillProgress::default())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Get current progress
    pub async fn get_progress(&self) -> BackfillProgress {
        self.progress.read().await.clone()
    }

    /// Check if backfill is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Get artists that need offense backfill
    /// Filters out artists that have been searched recently
    async fn get_artists_needing_backfill(
        &self,
        limit: usize,
        skip_recent_days: i32,
    ) -> Result<Vec<ArtistForBackfill>> {
        let cutoff = Utc::now() - chrono::Duration::days(skip_recent_days as i64);

        let artists = sqlx::query_as::<_, ArtistForBackfill>(
            r#"
            SELECT
                a.id,
                a.canonical_name,
                obs.last_search_at as last_offense_search
            FROM artists a
            LEFT JOIN offense_backfill_status obs ON a.id = obs.artist_id
            WHERE obs.last_search_at IS NULL
               OR obs.last_search_at < $1
            ORDER BY
                CASE WHEN obs.last_search_at IS NULL THEN 0 ELSE 1 END,
                obs.last_search_at ASC
            LIMIT $2
            "#,
        )
        .bind(cutoff)
        .bind(limit as i64)
        .fetch_all(&self.db_pool)
        .await
        .context("Failed to fetch artists for backfill")?;

        Ok(artists)
    }

    /// Record that we searched for an artist's offenses
    async fn record_search(&self, artist_id: Uuid, offenses_found: i32) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO offense_backfill_status (artist_id, last_search_at, offenses_found)
            VALUES ($1, NOW(), $2)
            ON CONFLICT (artist_id) DO UPDATE SET
                last_search_at = NOW(),
                offenses_found = offense_backfill_status.offenses_found + $2,
                search_count = offense_backfill_status.search_count + 1
            "#,
        )
        .bind(artist_id)
        .bind(offenses_found)
        .execute(&self.db_pool)
        .await
        .context("Failed to record backfill search")?;

        Ok(())
    }

    /// Search for offenses for a single artist using news pipeline
    async fn search_artist_offenses(&self, artist: &ArtistForBackfill) -> Result<usize> {
        let news_pipeline = self
            .news_pipeline
            .as_ref()
            .context("News pipeline not configured")?;

        // Search for articles about this artist
        let processed = news_pipeline.search_artist(&artist.canonical_name).await?;

        // Count offenses detected
        let offenses_found: usize = processed.iter().map(|p| p.offenses.len()).sum();

        Ok(offenses_found)
    }

    /// Run backfill for artists without offense data
    pub async fn backfill_artist_offenses(
        &self,
        batch_size: usize,
        max_artists: Option<usize>,
        skip_recent_days: i32,
    ) -> Result<BackfillResult> {
        // Check if already running
        {
            let mut running = self.is_running.write().await;
            if *running {
                return Err(anyhow::anyhow!("Backfill is already running"));
            }
            *running = true;
        }

        let start_time = std::time::Instant::now();
        let mut result = BackfillResult::default();

        // Reset progress
        {
            let mut progress = self.progress.write().await;
            *progress = BackfillProgress {
                started_at: Some(Utc::now()),
                ..Default::default()
            };
        }

        // Get total count of artists needing backfill
        let total_needing: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM artists a
            LEFT JOIN offense_backfill_status obs ON a.id = obs.artist_id
            WHERE obs.last_search_at IS NULL
               OR obs.last_search_at < NOW() - INTERVAL '1 day' * $1
            "#,
        )
        .bind(skip_recent_days)
        .fetch_one(&self.db_pool)
        .await
        .unwrap_or(0);

        let target_count = max_artists
            .unwrap_or(total_needing as usize)
            .min(total_needing as usize);

        {
            let mut progress = self.progress.write().await;
            progress.artists_total = target_count;
        }

        tracing::info!(
            "Starting offense backfill for {} artists (batch size: {})",
            target_count,
            batch_size
        );

        let mut processed_count = 0;

        while processed_count < target_count {
            let artists = self
                .get_artists_needing_backfill(batch_size, skip_recent_days)
                .await?;

            if artists.is_empty() {
                break;
            }

            for artist in artists {
                if processed_count >= target_count {
                    break;
                }

                tracing::debug!(
                    "Searching offenses for: {} ({}/{})",
                    artist.canonical_name,
                    processed_count + 1,
                    target_count
                );

                match self.search_artist_offenses(&artist).await {
                    Ok(offenses_found) => {
                        result.offenses_created += offenses_found;
                        if let Err(e) = self.record_search(artist.id, offenses_found as i32).await {
                            tracing::warn!(
                                "Failed to record search for {}: {}",
                                artist.canonical_name,
                                e
                            );
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to search offenses for {}: {}",
                            artist.canonical_name,
                            e
                        );
                        result.errors += 1;
                        // Still record that we tried
                        let _ = self.record_search(artist.id, 0).await;
                    }
                }

                processed_count += 1;
                result.artists_processed = processed_count;

                // Update progress
                {
                    let mut progress = self.progress.write().await;
                    progress.artists_processed = processed_count;
                    progress.offenses_found = result.offenses_created;
                    progress.errors = result.errors;

                    // Estimate completion time
                    if processed_count > 0 {
                        let elapsed = start_time.elapsed().as_secs_f64();
                        let rate = processed_count as f64 / elapsed;
                        let remaining = (target_count - processed_count) as f64;
                        let eta_seconds = remaining / rate;
                        progress.estimated_completion =
                            Some(Utc::now() + chrono::Duration::seconds(eta_seconds as i64));
                    }
                }
            }
        }

        result.duration_seconds = start_time.elapsed().as_secs_f64();

        // Clear running flag
        {
            let mut running = self.is_running.write().await;
            *running = false;
        }

        tracing::info!(
            "Backfill complete: {} artists processed, {} offenses found, {} errors in {:.1}s",
            result.artists_processed,
            result.offenses_created,
            result.errors,
            result.duration_seconds
        );

        Ok(result)
    }

    /// Create the backfill status table if it doesn't exist
    pub async fn ensure_backfill_table(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS offense_backfill_status (
                artist_id UUID PRIMARY KEY REFERENCES artists(id) ON DELETE CASCADE,
                last_search_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                offenses_found INTEGER NOT NULL DEFAULT 0,
                search_count INTEGER NOT NULL DEFAULT 1,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.db_pool)
        .await
        .context("Failed to create backfill status table")?;

        // Create index for efficient queries
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_backfill_last_search
            ON offense_backfill_status(last_search_at)
            "#,
        )
        .execute(&self.db_pool)
        .await
        .context("Failed to create backfill index")?;

        Ok(())
    }

    /// Get backfill statistics
    pub async fn get_stats(&self) -> Result<BackfillStats> {
        let stats = sqlx::query_as::<_, BackfillStatsRow>(
            r#"
            SELECT
                COUNT(*) as total_artists,
                COUNT(obs.artist_id) as artists_searched,
                COALESCE(SUM(obs.offenses_found), 0) as total_offenses_found,
                MAX(obs.last_search_at) as last_search_at
            FROM artists a
            LEFT JOIN offense_backfill_status obs ON a.id = obs.artist_id
            "#,
        )
        .fetch_one(&self.db_pool)
        .await
        .context("Failed to get backfill stats")?;

        Ok(BackfillStats {
            total_artists: stats.total_artists as usize,
            artists_searched: stats.artists_searched as usize,
            artists_pending: (stats.total_artists - stats.artists_searched) as usize,
            total_offenses_found: stats.total_offenses_found as usize,
            last_search_at: stats.last_search_at,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackfillStats {
    pub total_artists: usize,
    pub artists_searched: usize,
    pub artists_pending: usize,
    pub total_offenses_found: usize,
    pub last_search_at: Option<DateTime<Utc>>,
}

#[derive(Debug, sqlx::FromRow)]
struct BackfillStatsRow {
    total_artists: i64,
    artists_searched: i64,
    total_offenses_found: i64,
    last_search_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backfill_progress_default() {
        let progress = BackfillProgress::default();
        assert_eq!(progress.artists_total, 0);
        assert_eq!(progress.artists_processed, 0);
        assert_eq!(progress.offenses_found, 0);
    }
}
