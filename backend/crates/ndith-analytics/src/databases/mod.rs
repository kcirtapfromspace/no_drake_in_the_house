//! Analytics Database Clients
//!
//! Provides access to:
//! - DuckDB: Analytics and OLAP queries
//! - Graph backends: pluggable store for artist relationships

pub mod duckdb_client;
pub mod graph_store;
#[cfg(feature = "graph-kuzu")]
pub mod kuzu_client;

pub use duckdb_client::*;
pub use graph_store::*;
#[cfg(feature = "graph-kuzu")]
pub use kuzu_client::*;
