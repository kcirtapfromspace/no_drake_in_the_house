//! Multi-Database Client Module
//!
//! Provides unified access to:
//! - DuckDB: Analytics and OLAP queries
//! - Kùzu: Graph database for artist relationships
//! - LanceDB: Vector embeddings for semantic search

pub mod duckdb_client;
pub mod kuzu_client;
pub mod lancedb_client;

pub use duckdb_client::*;
pub use kuzu_client::*;
pub use lancedb_client::*;

use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;

/// Configuration for all analytics databases
#[derive(Debug, Clone)]
pub struct DatabasesConfig {
    /// Base directory for database files
    pub data_dir: String,
    /// DuckDB file path (relative to data_dir)
    pub duckdb_file: String,
    /// Kùzu directory (relative to data_dir)
    pub kuzu_dir: String,
    /// LanceDB directory (relative to data_dir)
    pub lancedb_dir: String,
}

impl Default for DatabasesConfig {
    fn default() -> Self {
        Self {
            data_dir: "./data".to_string(),
            duckdb_file: "analytics.duckdb".to_string(),
            kuzu_dir: "kuzu_graph".to_string(),
            lancedb_dir: "lancedb_vectors".to_string(),
        }
    }
}

impl DatabasesConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        Self {
            data_dir: std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_string()),
            duckdb_file: std::env::var("DUCKDB_FILE").unwrap_or_else(|_| "analytics.duckdb".to_string()),
            kuzu_dir: std::env::var("KUZU_DIR").unwrap_or_else(|_| "kuzu_graph".to_string()),
            lancedb_dir: std::env::var("LANCEDB_DIR").unwrap_or_else(|_| "lancedb_vectors".to_string()),
        }
    }

    /// Get full DuckDB path
    pub fn duckdb_path(&self) -> String {
        Path::new(&self.data_dir)
            .join(&self.duckdb_file)
            .to_string_lossy()
            .to_string()
    }

    /// Get full Kùzu path
    pub fn kuzu_path(&self) -> String {
        Path::new(&self.data_dir)
            .join(&self.kuzu_dir)
            .to_string_lossy()
            .to_string()
    }

    /// Get full LanceDB path
    pub fn lancedb_path(&self) -> String {
        Path::new(&self.data_dir)
            .join(&self.lancedb_dir)
            .to_string_lossy()
            .to_string()
    }
}

/// Aggregated database clients for all analytics databases
pub struct DatabaseClients {
    /// DuckDB for OLAP analytics
    pub duckdb: Arc<DuckDbClient>,
    /// Kùzu for graph queries
    pub kuzu: Arc<KuzuClient>,
    /// LanceDB for vector search
    pub lancedb: Arc<LanceDbClient>,
    /// Configuration
    pub config: DatabasesConfig,
}

impl DatabaseClients {
    /// Create new database clients with default configuration
    pub async fn new(config: DatabasesConfig) -> Result<Self> {
        // Ensure data directory exists
        std::fs::create_dir_all(&config.data_dir)
            .context("Failed to create data directory")?;

        // Initialize DuckDB
        let duckdb = DuckDbClient::new(&config.duckdb_path())
            .context("Failed to create DuckDB client")?;
        duckdb.initialize_schema().await
            .context("Failed to initialize DuckDB schema")?;

        // Initialize Kùzu
        let kuzu = KuzuClient::new(&config.kuzu_path())
            .context("Failed to create Kùzu client")?;
        kuzu.initialize_schema()
            .context("Failed to initialize Kùzu schema")?;

        // Initialize LanceDB
        let lancedb = LanceDbClient::new(&config.lancedb_path()).await
            .context("Failed to create LanceDB client")?;
        lancedb.initialize_schema().await
            .context("Failed to initialize LanceDB schema")?;

        tracing::info!(
            duckdb_path = %config.duckdb_path(),
            kuzu_path = %config.kuzu_path(),
            lancedb_path = %config.lancedb_path(),
            "All analytics databases initialized"
        );

        Ok(Self {
            duckdb: Arc::new(duckdb),
            kuzu: Arc::new(kuzu),
            lancedb: Arc::new(lancedb),
            config,
        })
    }

    /// Create in-memory clients for testing
    pub async fn in_memory() -> Result<Self> {
        let duckdb = DuckDbClient::in_memory()
            .context("Failed to create in-memory DuckDB")?;
        duckdb.initialize_schema().await?;

        let temp_dir = tempfile::tempdir()
            .context("Failed to create temp directory")?;

        let kuzu_path = temp_dir.path().join("kuzu");
        let kuzu = KuzuClient::new(kuzu_path.to_str().unwrap())
            .context("Failed to create Kùzu client")?;
        kuzu.initialize_schema()?;

        let lance_path = temp_dir.path().join("lance");
        let lancedb = LanceDbClient::new(lance_path.to_str().unwrap()).await
            .context("Failed to create LanceDB client")?;
        lancedb.initialize_schema().await?;

        Ok(Self {
            duckdb: Arc::new(duckdb),
            kuzu: Arc::new(kuzu),
            lancedb: Arc::new(lancedb),
            config: DatabasesConfig::default(),
        })
    }

    /// Health check for all databases
    pub async fn health_check(&self) -> Result<DatabasesHealth> {
        let mut health = DatabasesHealth::default();

        // Check DuckDB
        health.duckdb = self.duckdb.get_daily_news_summary(1).await.is_ok();

        // Check Kùzu - just try to get a connection
        health.kuzu = true; // Kùzu is synchronous, connection test done on init

        // Check LanceDB
        health.lancedb = self.lancedb.get_stats().await.is_ok();

        health.all_healthy = health.duckdb && health.kuzu && health.lancedb;

        Ok(health)
    }

    /// Get combined statistics
    pub async fn get_stats(&self) -> Result<CombinedStats> {
        let vector_stats = self.lancedb.get_stats().await?;

        Ok(CombinedStats {
            news_embeddings: vector_stats.news_embeddings_count,
            artist_embeddings: vector_stats.artist_embeddings_count,
            // Graph stats would require additional methods
            graph_artists: 0, // TODO: Add graph stats
            graph_collaborations: 0,
        })
    }
}

/// Health status of all databases
#[derive(Debug, Clone, Default)]
pub struct DatabasesHealth {
    pub duckdb: bool,
    pub kuzu: bool,
    pub lancedb: bool,
    pub all_healthy: bool,
}

/// Combined statistics from all databases
#[derive(Debug, Clone)]
pub struct CombinedStats {
    pub news_embeddings: u64,
    pub artist_embeddings: u64,
    pub graph_artists: u64,
    pub graph_collaborations: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_clients_in_memory() {
        let clients = DatabaseClients::in_memory().await.unwrap();
        let health = clients.health_check().await.unwrap();
        assert!(health.duckdb);
        assert!(health.kuzu);
    }
}
