use crate::models::{Artist, ArtistSearchQuery, ArtistResolutionResult};
use sqlx::PgPool;
use std::sync::Arc;
use anyhow::Result;

/// Stub implementation of EntityResolutionService for tests
#[derive(Clone)]
pub struct EntityResolutionService {
    pool: PgPool,
}

impl EntityResolutionService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn with_confidence_threshold(self, _threshold: f64) -> Self {
        self
    }

    pub async fn resolve_artist(&self, _query: &ArtistSearchQuery) -> Result<Vec<ArtistResolutionResult>> {
        // Stub implementation - return empty results
        Ok(vec![])
    }

    pub async fn resolve_artists_batch(&self, _queries: Vec<ArtistSearchQuery>) -> Result<Vec<Vec<ArtistResolutionResult>>> {
        // Stub implementation - return empty results for each query
        Ok(vec![])
    }

    pub async fn get_artist_by_id(&self, _id: uuid::Uuid) -> Result<Option<Artist>> {
        // Stub implementation
        Ok(None)
    }
}