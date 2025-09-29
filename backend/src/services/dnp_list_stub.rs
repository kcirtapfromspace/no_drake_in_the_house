use crate::models::{Artist, User};
use crate::services::EntityResolutionService;
use sqlx::PgPool;
use std::sync::Arc;
use anyhow::Result;
use uuid::Uuid;

/// Stub implementation of DnpListService for tests
#[derive(Clone)]
pub struct DnpListService {
    pool: PgPool,
    entity_service: Arc<EntityResolutionService>,
}

impl DnpListService {
    pub fn new(pool: PgPool, entity_service: Arc<EntityResolutionService>) -> Self {
        Self { pool, entity_service }
    }

    pub async fn add_artist_to_dnp(&self, _user_id: Uuid, _artist_query: &str) -> Result<()> {
        // Stub implementation
        Ok(())
    }

    pub async fn remove_artist_from_dnp(&self, _user_id: Uuid, _artist_id: Uuid) -> Result<()> {
        // Stub implementation
        Ok(())
    }

    pub async fn get_user_dnp_list(&self, _user_id: Uuid) -> Result<Vec<Artist>> {
        // Stub implementation
        Ok(vec![])
    }
}