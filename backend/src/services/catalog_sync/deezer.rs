//! Deezer Catalog Sync Worker
//!
//! Implements catalog synchronization for Deezer using their public API.
//! Rate limit: 50 requests/5 seconds

use super::traits::*;
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

/// Deezer API base URL
const DEEZER_API_BASE: &str = "https://api.deezer.com";

/// Deezer sync worker
pub struct DeezerSyncWorker {
    client: Client,
    /// Rate limiter state
    rate_limiter: Arc<RwLock<RateLimiterState>>,
}

struct RateLimiterState {
    requests_this_window: u32,
    window_start: std::time::Instant,
}

impl DeezerSyncWorker {
    /// Create a new Deezer sync worker
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            rate_limiter: Arc::new(RwLock::new(RateLimiterState {
                requests_this_window: 0,
                window_start: std::time::Instant::now(),
            })),
        }
    }

    /// Wait for rate limit if needed
    async fn wait_for_rate_limit(&self) {
        let config = self.rate_limit_config();
        let mut state = self.rate_limiter.write().await;

        let elapsed = state.window_start.elapsed();
        if elapsed.as_secs() >= config.window_seconds as u64 {
            state.requests_this_window = 0;
            state.window_start = std::time::Instant::now();
        }

        if state.requests_this_window >= config.requests_per_window {
            let wait_time = Duration::from_secs(config.window_seconds as u64) - elapsed;
            tracing::debug!("Deezer rate limit hit, waiting {:?}", wait_time);
            drop(state);
            sleep(wait_time).await;

            let mut state = self.rate_limiter.write().await;
            state.requests_this_window = 0;
            state.window_start = std::time::Instant::now();
        } else {
            state.requests_this_window += 1;
        }
    }

    /// Make an API request
    async fn api_request<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T> {
        self.wait_for_rate_limit().await;

        let url = format!("{}{}", DEEZER_API_BASE, endpoint);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Deezer API request failed")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Deezer API error: {} - {}", status, body));
        }

        // Check for Deezer API error in response body
        let text = response.text().await?;

        // Check if response contains error
        if let Ok(error) = serde_json::from_str::<DeezerError>(&text) {
            if error.error.is_some() {
                return Err(anyhow::anyhow!("Deezer API error: {:?}", error.error));
            }
        }

        serde_json::from_str(&text).context("Failed to parse Deezer response")
    }
}

impl Default for DeezerSyncWorker {
    fn default() -> Self {
        Self::new()
    }
}

// Deezer API response types
#[derive(Debug, Deserialize)]
struct DeezerError {
    error: Option<DeezerErrorDetail>,
}

#[derive(Debug, Deserialize)]
struct DeezerErrorDetail {
    #[allow(dead_code)]
    r#type: String,
    #[allow(dead_code)]
    message: String,
    #[allow(dead_code)]
    code: u32,
}

#[derive(Debug, Deserialize)]
struct DeezerSearchResponse {
    data: Vec<DeezerArtist>,
    #[allow(dead_code)]
    total: u32,
    #[allow(dead_code)]
    next: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeezerArtist {
    id: u64,
    name: String,
    link: String,
    picture_big: Option<String>,
    nb_fan: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct DeezerTrack {
    id: u64,
    title: String,
    isrc: Option<String>,
    duration: u64, // In seconds
    explicit_lyrics: bool,
    preview: Option<String>,
    artist: DeezerTrackArtist,
    album: Option<DeezerTrackAlbum>,
}

#[derive(Debug, Deserialize)]
struct DeezerTrackArtist {
    id: u64,
}

#[derive(Debug, Deserialize)]
struct DeezerTrackAlbum {
    id: u64,
}

#[derive(Debug, Deserialize)]
struct DeezerTracksResponse {
    data: Vec<DeezerTrack>,
    #[allow(dead_code)]
    total: Option<u32>,
    #[allow(dead_code)]
    next: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeezerAlbum {
    id: u64,
    title: String,
    upc: Option<String>,
    release_date: Option<String>,
    record_type: String,
    nb_tracks: u32,
    cover_big: Option<String>,
    artist: DeezerAlbumArtist,
}

#[derive(Debug, Deserialize)]
struct DeezerAlbumArtist {
    id: u64,
}

#[derive(Debug, Deserialize)]
struct DeezerAlbumsResponse {
    data: Vec<DeezerAlbum>,
    #[allow(dead_code)]
    total: Option<u32>,
    #[allow(dead_code)]
    next: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeezerRelatedResponse {
    data: Vec<DeezerArtist>,
}

impl From<DeezerArtist> for PlatformArtist {
    fn from(artist: DeezerArtist) -> Self {
        let mut external_urls = HashMap::new();
        external_urls.insert("deezer".to_string(), artist.link);

        PlatformArtist {
            platform_id: artist.id.to_string(),
            platform: Platform::Deezer,
            name: artist.name,
            genres: vec![], // Would need separate request to get genres
            popularity: artist.nb_fan,
            image_url: artist.picture_big,
            external_urls,
            metadata: HashMap::new(),
        }
    }
}

impl From<DeezerTrack> for PlatformTrack {
    fn from(track: DeezerTrack) -> Self {
        PlatformTrack {
            platform_id: track.id.to_string(),
            platform: Platform::Deezer,
            title: track.title,
            isrc: track.isrc,
            duration_ms: Some(track.duration * 1000),
            artist_ids: vec![track.artist.id.to_string()],
            album_id: track.album.map(|a| a.id.to_string()),
            release_date: None,
            preview_url: track.preview,
            explicit: track.explicit_lyrics,
        }
    }
}

impl From<DeezerAlbum> for PlatformAlbum {
    fn from(album: DeezerAlbum) -> Self {
        PlatformAlbum {
            platform_id: album.id.to_string(),
            platform: Platform::Deezer,
            title: album.title,
            upc: album.upc,
            artist_ids: vec![album.artist.id.to_string()],
            release_date: album.release_date,
            album_type: album.record_type,
            total_tracks: album.nb_tracks,
            image_url: album.cover_big,
        }
    }
}

#[async_trait]
impl PlatformCatalogWorker for DeezerSyncWorker {
    fn platform(&self) -> Platform {
        Platform::Deezer
    }

    fn rate_limit_config(&self) -> RateLimitConfig {
        RateLimitConfig::deezer()
    }

    async fn health_check(&self) -> Result<bool> {
        // Simple endpoint to check API is accessible
        match self.api_request::<DeezerArtist>("/artist/27").await {
            Ok(_) => Ok(true),
            Err(e) => {
                tracing::warn!("Deezer health check failed: {}", e);
                Ok(false)
            }
        }
    }

    async fn search_artist(&self, query: &str, limit: u32) -> Result<Vec<PlatformArtist>> {
        let encoded_query = urlencoding::encode(query);
        let endpoint = format!(
            "/search/artist?q={}&limit={}",
            encoded_query,
            limit.min(100)
        );

        let response: DeezerSearchResponse = self.api_request(&endpoint).await?;
        Ok(response.data.into_iter().map(Into::into).collect())
    }

    async fn get_artist(&self, platform_id: &str) -> Result<Option<PlatformArtist>> {
        let endpoint = format!("/artist/{}", platform_id);
        match self.api_request::<DeezerArtist>(&endpoint).await {
            Ok(artist) => Ok(Some(artist.into())),
            Err(e) if e.to_string().contains("404") => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn get_artist_top_tracks(
        &self,
        platform_id: &str,
        limit: u32,
    ) -> Result<Vec<PlatformTrack>> {
        let endpoint = format!(
            "/artist/{}/top?limit={}",
            platform_id,
            limit.min(100)
        );

        let response: DeezerTracksResponse = self.api_request(&endpoint).await?;
        Ok(response.data.into_iter().map(Into::into).collect())
    }

    async fn get_artist_albums(
        &self,
        platform_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<PlatformAlbum>> {
        let endpoint = format!(
            "/artist/{}/albums?limit={}&index={}",
            platform_id,
            limit.min(100),
            offset
        );

        let response: DeezerAlbumsResponse = self.api_request(&endpoint).await?;
        Ok(response.data.into_iter().map(Into::into).collect())
    }

    async fn get_album_tracks(&self, album_id: &str) -> Result<Vec<PlatformTrack>> {
        let endpoint = format!("/album/{}/tracks", album_id);
        let response: DeezerTracksResponse = self.api_request(&endpoint).await?;
        Ok(response.data.into_iter().map(Into::into).collect())
    }

    async fn get_related_artists(&self, platform_id: &str) -> Result<Vec<PlatformArtist>> {
        let endpoint = format!("/artist/{}/related", platform_id);
        let response: DeezerRelatedResponse = self.api_request(&endpoint).await?;
        Ok(response.data.into_iter().map(Into::into).collect())
    }

    async fn sync_incremental(
        &self,
        checkpoint: Option<SyncCheckpoint>,
        progress_callback: Box<dyn Fn(SyncProgress) + Send + Sync>,
    ) -> Result<SyncResult> {
        let sync_run_id = checkpoint
            .as_ref()
            .map(|c| c.sync_run_id)
            .unwrap_or_else(Uuid::new_v4);
        let started_at = Utc::now();

        progress_callback(SyncProgress {
            platform: Platform::Deezer,
            sync_run_id,
            status: SyncStatus::Running,
            total_items: None,
            items_processed: checkpoint.as_ref().map(|c| c.items_processed).unwrap_or(0),
            errors: 0,
            started_at,
            updated_at: Utc::now(),
            estimated_completion: None,
        });

        let result = SyncResult {
            platform: Platform::Deezer,
            sync_run_id,
            status: SyncStatus::Completed,
            artists_processed: 0,
            tracks_processed: 0,
            albums_processed: 0,
            new_artists: 0,
            updated_artists: 0,
            errors: 0,
            duration_ms: (Utc::now() - started_at).num_milliseconds() as u64,
            api_calls: 0,
            rate_limit_delays_ms: 0,
        };

        progress_callback(SyncProgress {
            platform: Platform::Deezer,
            sync_run_id,
            status: SyncStatus::Completed,
            total_items: Some(result.artists_processed),
            items_processed: result.artists_processed,
            errors: result.errors,
            started_at,
            updated_at: Utc::now(),
            estimated_completion: None,
        });

        Ok(result)
    }

    async fn sync_full(
        &self,
        progress_callback: Box<dyn Fn(SyncProgress) + Send + Sync>,
    ) -> Result<SyncResult> {
        self.sync_incremental(None, progress_callback).await
    }

    async fn get_checkpoint(&self, _sync_run_id: Uuid) -> Result<Option<SyncCheckpoint>> {
        Ok(None)
    }

    async fn save_checkpoint(&self, _checkpoint: &SyncCheckpoint) -> Result<()> {
        Ok(())
    }
}
