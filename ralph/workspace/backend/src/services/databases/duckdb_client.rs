#![allow(clippy::arc_with_non_send_sync)]
//! DuckDB Analytics Database Client
//!
//! Provides OLAP capabilities for:
//! - Sync performance metrics
//! - News volume analytics
//! - Artist mention trends
//! - Time-series aggregations

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use duckdb::{params, Connection};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// DuckDB client for analytics queries
pub struct DuckDbClient {
    conn: Arc<RwLock<Connection>>,
    #[allow(dead_code)]
    db_path: String,
}

impl DuckDbClient {
    /// Create a new DuckDB client with persistent storage
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path).context("Failed to open DuckDB database")?;

        let client = Self {
            conn: Arc::new(RwLock::new(conn)),
            db_path: db_path.to_string(),
        };

        Ok(client)
    }

    /// Create an in-memory DuckDB client (for testing)
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("Failed to open in-memory DuckDB")?;

        let client = Self {
            conn: Arc::new(RwLock::new(conn)),
            db_path: ":memory:".to_string(),
        };

        Ok(client)
    }

    /// Initialize the analytics schema
    pub async fn initialize_schema(&self) -> Result<()> {
        let conn = self.conn.write().await;

        // Sync metrics table - tracks platform sync performance
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS sync_metrics (
                timestamp VARCHAR NOT NULL,
                platform VARCHAR NOT NULL,
                sync_run_id VARCHAR NOT NULL,
                artists_processed INTEGER DEFAULT 0,
                api_calls_made INTEGER DEFAULT 0,
                rate_limit_delays_ms BIGINT DEFAULT 0,
                errors_count INTEGER DEFAULT 0,
                duration_ms BIGINT DEFAULT 0
            )
            "#,
            [],
        )?;

        // News volume hourly aggregates
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS news_volume_hourly (
                hour VARCHAR NOT NULL,
                source_type VARCHAR NOT NULL,
                articles_count INTEGER DEFAULT 0,
                entities_extracted INTEGER DEFAULT 0,
                offenses_detected INTEGER DEFAULT 0,
                avg_sentiment FLOAT DEFAULT 0.0,
                PRIMARY KEY (hour, source_type)
            )
            "#,
            [],
        )?;

        // Artist mention trends - daily rollup
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS artist_mention_trends (
                date VARCHAR NOT NULL,
                artist_id VARCHAR NOT NULL,
                artist_name VARCHAR,
                mention_count INTEGER DEFAULT 0,
                positive_mentions INTEGER DEFAULT 0,
                negative_mentions INTEGER DEFAULT 0,
                neutral_mentions INTEGER DEFAULT 0,
                offense_mentions INTEGER DEFAULT 0,
                platforms_mentioned VARCHAR,
                PRIMARY KEY (date, artist_id)
            )
            "#,
            [],
        )?;

        // Platform sync performance - daily rollup
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS sync_performance_daily (
                date VARCHAR NOT NULL,
                platform VARCHAR NOT NULL,
                total_syncs INTEGER DEFAULT 0,
                successful_syncs INTEGER DEFAULT 0,
                failed_syncs INTEGER DEFAULT 0,
                avg_duration_ms BIGINT DEFAULT 0,
                total_artists_synced INTEGER DEFAULT 0,
                total_api_calls INTEGER DEFAULT 0,
                total_rate_limit_delays_ms BIGINT DEFAULT 0,
                PRIMARY KEY (date, platform)
            )
            "#,
            [],
        )?;

        // Entity resolution metrics
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS resolution_metrics (
                timestamp VARCHAR NOT NULL,
                total_artists INTEGER DEFAULT 0,
                fully_resolved INTEGER DEFAULT 0,
                partially_resolved INTEGER DEFAULT 0,
                unresolved INTEGER DEFAULT 0,
                merge_operations INTEGER DEFAULT 0,
                avg_confidence FLOAT DEFAULT 0.0
            )
            "#,
            [],
        )?;

        tracing::info!("DuckDB analytics schema initialized");
        Ok(())
    }

    /// Record sync metrics
    pub async fn record_sync_metrics(&self, metrics: SyncMetrics) -> Result<()> {
        let conn = self.conn.write().await;

        conn.execute(
            r#"
            INSERT INTO sync_metrics
            (timestamp, platform, sync_run_id, artists_processed, api_calls_made,
             rate_limit_delays_ms, errors_count, duration_ms)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                metrics.timestamp.to_rfc3339(),
                metrics.platform,
                metrics.sync_run_id.to_string(),
                metrics.artists_processed,
                metrics.api_calls_made,
                metrics.rate_limit_delays_ms,
                metrics.errors_count,
                metrics.duration_ms,
            ],
        )?;

        Ok(())
    }

    /// Record news article processing
    pub async fn record_news_volume(&self, record: NewsVolumeRecord) -> Result<()> {
        let conn = self.conn.write().await;

        conn.execute(
            r#"
            INSERT INTO news_volume_hourly
            (hour, source_type, articles_count, entities_extracted, offenses_detected, avg_sentiment)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT (hour, source_type) DO UPDATE SET
                articles_count = news_volume_hourly.articles_count + excluded.articles_count,
                entities_extracted = news_volume_hourly.entities_extracted + excluded.entities_extracted,
                offenses_detected = news_volume_hourly.offenses_detected + excluded.offenses_detected,
                avg_sentiment = (news_volume_hourly.avg_sentiment + excluded.avg_sentiment) / 2
            "#,
            params![
                record.hour.to_rfc3339(),
                record.source_type,
                record.articles_count,
                record.entities_extracted,
                record.offenses_detected,
                record.avg_sentiment,
            ],
        )?;

        Ok(())
    }

    /// Record artist mention
    pub async fn record_artist_mention(&self, record: ArtistMentionRecord) -> Result<()> {
        let conn = self.conn.write().await;

        // This uses UPSERT to aggregate mentions per day
        conn.execute(
            r#"
            INSERT INTO artist_mention_trends
            (date, artist_id, artist_name, mention_count, positive_mentions,
             negative_mentions, neutral_mentions, offense_mentions, platforms_mentioned)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT (date, artist_id) DO UPDATE SET
                mention_count = artist_mention_trends.mention_count + excluded.mention_count,
                positive_mentions = artist_mention_trends.positive_mentions + excluded.positive_mentions,
                negative_mentions = artist_mention_trends.negative_mentions + excluded.negative_mentions,
                neutral_mentions = artist_mention_trends.neutral_mentions + excluded.neutral_mentions,
                offense_mentions = artist_mention_trends.offense_mentions + excluded.offense_mentions
            "#,
            params![
                record.date.to_string(),
                record.artist_id.to_string(),
                record.artist_name,
                record.mention_count,
                record.positive_mentions,
                record.negative_mentions,
                record.neutral_mentions,
                record.offense_mentions,
                record.platforms_mentioned.join(","),
            ],
        )?;

        Ok(())
    }

    /// Get daily news summary
    pub async fn get_daily_news_summary(&self, days: i32) -> Result<Vec<DailyNewsSummary>> {
        let conn = self.conn.read().await;

        let mut stmt = conn.prepare(
            r#"
            SELECT
                CAST(hour AS DATE) as date,
                SUM(articles_count) as total_articles,
                SUM(entities_extracted) as total_entities,
                SUM(offenses_detected) as total_offenses,
                AVG(avg_sentiment) as avg_sentiment
            FROM news_volume_hourly
            WHERE CAST(hour AS TIMESTAMP) >= CURRENT_TIMESTAMP - INTERVAL ? DAY
            GROUP BY CAST(hour AS DATE)
            ORDER BY date DESC
            "#,
        )?;

        let rows = stmt.query_map(params![days], |row| {
            Ok(DailyNewsSummary {
                date: row.get::<_, String>(0)?,
                total_articles: row.get(1)?,
                total_entities: row.get(2)?,
                total_offenses: row.get(3)?,
                avg_sentiment: row.get(4)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    /// Get trending artists by mentions
    pub async fn get_trending_artists(&self, days: i32, limit: i32) -> Result<Vec<TrendingArtist>> {
        let conn = self.conn.read().await;

        let mut stmt = conn.prepare(
            r#"
            SELECT
                artist_id,
                artist_name,
                SUM(mention_count) as total_mentions,
                SUM(offense_mentions) as offense_mentions,
                AVG(CAST(positive_mentions AS FLOAT) / NULLIF(mention_count, 0)) as positive_ratio
            FROM artist_mention_trends
            WHERE CAST(date AS DATE) >= CURRENT_DATE - INTERVAL ? DAY
            GROUP BY artist_id, artist_name
            ORDER BY total_mentions DESC
            LIMIT ?
            "#,
        )?;

        let rows = stmt.query_map(params![days, limit], |row| {
            let artist_id_str: String = row.get(0)?;
            Ok(TrendingArtist {
                artist_id: Uuid::parse_str(&artist_id_str).unwrap_or_default(),
                artist_name: row.get(1)?,
                total_mentions: row.get(2)?,
                offense_mentions: row.get(3)?,
                positive_ratio: row.get::<_, Option<f64>>(4)?.unwrap_or(0.0),
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    /// Get platform sync health
    pub async fn get_platform_health(&self, days: i32) -> Result<Vec<PlatformHealth>> {
        let conn = self.conn.read().await;

        let mut stmt = conn.prepare(
            r#"
            SELECT
                platform,
                SUM(total_syncs) as total_syncs,
                SUM(successful_syncs) as successful_syncs,
                SUM(failed_syncs) as failed_syncs,
                AVG(avg_duration_ms) as avg_duration_ms,
                SUM(total_artists_synced) as total_artists_synced,
                SUM(total_api_calls) as total_api_calls
            FROM sync_performance_daily
            WHERE CAST(date AS DATE) >= CURRENT_DATE - INTERVAL ? DAY
            GROUP BY platform
            "#,
        )?;

        let rows = stmt.query_map(params![days], |row| {
            let total_syncs: i64 = row.get(1)?;
            let successful_syncs: i64 = row.get(2)?;
            Ok(PlatformHealth {
                platform: row.get(0)?,
                total_syncs,
                successful_syncs,
                failed_syncs: row.get(3)?,
                success_rate: if total_syncs > 0 {
                    successful_syncs as f64 / total_syncs as f64
                } else {
                    0.0
                },
                avg_duration_ms: row.get(4)?,
                total_artists_synced: row.get(5)?,
                total_api_calls: row.get(6)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    /// Export data to Parquet for archival
    pub async fn export_to_parquet(&self, table: &str, output_path: &str) -> Result<()> {
        let conn = self.conn.read().await;

        conn.execute(
            &format!("COPY {} TO '{}' (FORMAT PARQUET)", table, output_path),
            [],
        )?;

        tracing::info!("Exported {} to {}", table, output_path);
        Ok(())
    }

    /// Run a custom analytics query
    pub async fn query<T, F>(
        &self,
        sql: &str,
        params: &[&dyn duckdb::ToSql],
        mapper: F,
    ) -> Result<Vec<T>>
    where
        F: Fn(&duckdb::Row) -> Result<T, duckdb::Error>,
    {
        let conn = self.conn.read().await;
        let mut stmt = conn.prepare(sql)?;

        let rows = stmt.query_map(params, mapper)?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }
}

/// Sync metrics record
#[derive(Debug, Clone)]
pub struct SyncMetrics {
    pub timestamp: DateTime<Utc>,
    pub platform: String,
    pub sync_run_id: Uuid,
    pub artists_processed: i32,
    pub api_calls_made: i32,
    pub rate_limit_delays_ms: i64,
    pub errors_count: i32,
    pub duration_ms: i64,
}

/// News volume record
#[derive(Debug, Clone)]
pub struct NewsVolumeRecord {
    pub hour: DateTime<Utc>,
    pub source_type: String,
    pub articles_count: i32,
    pub entities_extracted: i32,
    pub offenses_detected: i32,
    pub avg_sentiment: f64,
}

/// Artist mention record
#[derive(Debug, Clone)]
pub struct ArtistMentionRecord {
    pub date: NaiveDate,
    pub artist_id: Uuid,
    pub artist_name: String,
    pub mention_count: i32,
    pub positive_mentions: i32,
    pub negative_mentions: i32,
    pub neutral_mentions: i32,
    pub offense_mentions: i32,
    pub platforms_mentioned: Vec<String>,
}

/// Daily news summary
#[derive(Debug, Clone)]
pub struct DailyNewsSummary {
    pub date: String,
    pub total_articles: i64,
    pub total_entities: i64,
    pub total_offenses: i64,
    pub avg_sentiment: f64,
}

/// Trending artist
#[derive(Debug, Clone)]
pub struct TrendingArtist {
    pub artist_id: Uuid,
    pub artist_name: String,
    pub total_mentions: i64,
    pub offense_mentions: i64,
    pub positive_ratio: f64,
}

/// Platform health metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlatformHealth {
    pub platform: String,
    pub total_syncs: i64,
    pub successful_syncs: i64,
    pub failed_syncs: i64,
    pub success_rate: f64,
    pub avg_duration_ms: i64,
    pub total_artists_synced: i64,
    pub total_api_calls: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_duckdb_initialization() {
        let client = DuckDbClient::in_memory().unwrap();
        client.initialize_schema().await.unwrap();
    }

    #[tokio::test]
    async fn test_record_sync_metrics() {
        let client = DuckDbClient::in_memory().unwrap();
        client.initialize_schema().await.unwrap();

        let metrics = SyncMetrics {
            timestamp: Utc::now(),
            platform: "spotify".to_string(),
            sync_run_id: Uuid::new_v4(),
            artists_processed: 100,
            api_calls_made: 50,
            rate_limit_delays_ms: 0,
            errors_count: 0,
            duration_ms: 5000,
        };

        client.record_sync_metrics(metrics).await.unwrap();
    }
}
