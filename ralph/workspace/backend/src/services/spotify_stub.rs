use crate::services::TokenVaultService;
use std::sync::Arc;
use anyhow::Result;

/// Stub Spotify configuration
#[derive(Clone, Default)]
pub struct SpotifyConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

/// Stub implementation of SpotifyService for tests
#[derive(Clone)]
pub struct SpotifyService {
    config: SpotifyConfig,
    token_vault: Arc<TokenVaultService>,
}

impl SpotifyService {
    pub fn new(config: SpotifyConfig, token_vault: Arc<TokenVaultService>) -> Result<Self> {
        Ok(Self { config, token_vault })
    }

    pub async fn get_user_profile(&self, _user_id: uuid::Uuid) -> Result<serde_json::Value> {
        // Stub implementation
        Ok(serde_json::json!({"id": "test_user", "display_name": "Test User"}))
    }
}

/// Stub implementation of SpotifyLibraryService for tests
#[derive(Clone)]
pub struct SpotifyLibraryService {
    spotify_service: SpotifyService,
}

impl SpotifyLibraryService {
    pub fn new(spotify_service: SpotifyService) -> Self {
        Self { spotify_service }
    }

    pub async fn scan_library(&self, _user_id: uuid::Uuid) -> Result<Vec<serde_json::Value>> {
        // Stub implementation
        Ok(vec![])
    }
}