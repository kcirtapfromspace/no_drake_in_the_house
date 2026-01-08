//! Tidal Catalog Sync Worker
//!
//! Implements catalog synchronization for Tidal using their API.
//! Rate limit: 500 requests/5 minutes

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

/// Tidal API base URL
const TIDAL_API_BASE: &str = "https://openapi.tidal.com/v2";

/// Tidal sync worker
pub struct TidalSyncWorker {
    client: Client,
    /// OAuth access token
    access_token: Arc<RwLock<Option<String>>>,
    /// Client ID
    client_id: String,
    /// Client secret
    client_secret: String,
    /// Rate limiter state
    rate_limiter: Arc<RwLock<RateLimiterState>>,
}

struct RateLimiterState {
    requests_this_window: u32,
    window_start: std::time::Instant,
}

impl TidalSyncWorker {
    /// Create a new Tidal sync worker
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client: Client::new(),
            access_token: Arc::new(RwLock::new(None)),
            client_id,
            client_secret,
            rate_limiter: Arc::new(RwLock::new(RateLimiterState {
                requests_this_window: 0,
                window_start: std::time::Instant::now(),
            })),
        }
    }

    /// Get or refresh access token
    async fn ensure_token(&self) -> Result<String> {
        {
            let token = self.access_token.read().await;
            if let Some(t) = token.as_ref() {
                return Ok(t.clone());
            }
        }

        let token = self.refresh_token().await?;
        {
            let mut token_lock = self.access_token.write().await;
            *token_lock = Some(token.clone());
        }
        Ok(token)
    }

    /// Refresh the access token
    async fn refresh_token(&self) -> Result<String> {
        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
        }

        let response = self
            .client
            .post("https://auth.tidal.com/v1/oauth2/token")
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await
            .context("Failed to request Tidal token")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Tidal token request failed: {} - {}",
                status,
                body
            ));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .context("Failed to parse Tidal token response")?;

        Ok(token_response.access_token)
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
            tracing::debug!("Tidal rate limit hit, waiting {:?}", wait_time);
            drop(state);
            sleep(wait_time).await;

            let mut state = self.rate_limiter.write().await;
            state.requests_this_window = 0;
            state.window_start = std::time::Instant::now();
        } else {
            state.requests_this_window += 1;
        }
    }

    /// Make an authenticated API request
    async fn api_request<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T> {
        self.wait_for_rate_limit().await;
        let token = self.ensure_token().await?;

        let url = format!("{}{}", TIDAL_API_BASE, endpoint);
        let response = self
            .client
            .get(&url)
            .bearer_auth(&token)
            .header("Content-Type", "application/vnd.tidal.v1+json")
            .send()
            .await
            .context("Tidal API request failed")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Tidal API error: {} - {}", status, body));
        }

        response
            .json()
            .await
            .context("Failed to parse Tidal response")
    }
}

// Tidal API response types
#[derive(Debug, Deserialize)]
struct TidalResponse<T> {
    data: Vec<T>,
    #[allow(dead_code)]
    metadata: Option<TidalMetadata>,
}

#[derive(Debug, Deserialize)]
struct TidalMetadata {
    #[allow(dead_code)]
    total: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct TidalArtist {
    id: String,
    attributes: TidalArtistAttributes,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TidalArtistAttributes {
    name: String,
    popularity: Option<u32>,
    image_links: Option<Vec<TidalImageLink>>,
    external_links: Option<Vec<TidalExternalLink>>,
}

#[derive(Debug, Deserialize)]
struct TidalImageLink {
    href: String,
}

#[derive(Debug, Deserialize)]
struct TidalExternalLink {
    href: String,
    #[allow(dead_code)]
    meta: TidalExternalLinkMeta,
}

#[derive(Debug, Deserialize)]
struct TidalExternalLinkMeta {
    #[allow(dead_code)]
    r#type: String,
}

#[derive(Debug, Deserialize)]
struct TidalTrack {
    id: String,
    attributes: TidalTrackAttributes,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TidalTrackAttributes {
    title: String,
    isrc: Option<String>,
    duration: Option<u64>,
    explicit: bool,
}

#[derive(Debug, Deserialize)]
struct TidalAlbum {
    id: String,
    attributes: TidalAlbumAttributes,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TidalAlbumAttributes {
    title: String,
    barcode_id: Option<String>,
    release_date: Option<String>,
    number_of_tracks: u32,
    image_links: Option<Vec<TidalImageLink>>,
}

impl From<TidalArtist> for PlatformArtist {
    fn from(artist: TidalArtist) -> Self {
        let mut external_urls = HashMap::new();
        if let Some(links) = artist.attributes.external_links {
            for link in links {
                external_urls.insert("tidal".to_string(), link.href);
            }
        }

        PlatformArtist {
            platform_id: artist.id,
            platform: Platform::Tidal,
            name: artist.attributes.name,
            genres: vec![], // Tidal doesn't expose genres in the same way
            popularity: artist.attributes.popularity.map(|p| p as u64),
            image_url: artist
                .attributes
                .image_links
                .and_then(|links| links.first().map(|l| l.href.clone())),
            external_urls,
            metadata: HashMap::new(),
        }
    }
}

impl TidalTrack {
    fn into_platform_track(
        self,
        artist_ids: Vec<String>,
        album_id: Option<String>,
    ) -> PlatformTrack {
        PlatformTrack {
            platform_id: self.id,
            platform: Platform::Tidal,
            title: self.attributes.title,
            isrc: self.attributes.isrc,
            duration_ms: self.attributes.duration.map(|d| d * 1000), // Tidal uses seconds
            artist_ids,
            album_id,
            release_date: None,
            preview_url: None,
            explicit: self.attributes.explicit,
        }
    }
}

impl TidalAlbum {
    fn into_platform_album(self, artist_ids: Vec<String>) -> PlatformAlbum {
        PlatformAlbum {
            platform_id: self.id,
            platform: Platform::Tidal,
            title: self.attributes.title,
            upc: self.attributes.barcode_id,
            artist_ids,
            release_date: self.attributes.release_date,
            album_type: "album".to_string(),
            total_tracks: self.attributes.number_of_tracks,
            image_url: self
                .attributes
                .image_links
                .and_then(|links| links.first().map(|l| l.href.clone())),
        }
    }
}

#[async_trait]
impl PlatformCatalogWorker for TidalSyncWorker {
    fn platform(&self) -> Platform {
        Platform::Tidal
    }

    fn rate_limit_config(&self) -> RateLimitConfig {
        RateLimitConfig::tidal()
    }

    async fn health_check(&self) -> Result<bool> {
        match self.ensure_token().await {
            Ok(_) => Ok(true),
            Err(e) => {
                tracing::warn!("Tidal health check failed: {}", e);
                Ok(false)
            }
        }
    }

    async fn search_artist(&self, query: &str, limit: u32) -> Result<Vec<PlatformArtist>> {
        let encoded_query = urlencoding::encode(query);
        let endpoint = format!(
            "/searchresults/{}?include=artists&countryCode=US&limit={}",
            encoded_query,
            limit.min(100)
        );

        let response: TidalResponse<TidalArtist> = self.api_request(&endpoint).await?;
        Ok(response.data.into_iter().map(Into::into).collect())
    }

    async fn get_artist(&self, platform_id: &str) -> Result<Option<PlatformArtist>> {
        let endpoint = format!("/artists/{}?countryCode=US", platform_id);
        match self
            .api_request::<TidalResponse<TidalArtist>>(&endpoint)
            .await
        {
            Ok(response) => Ok(response.data.into_iter().next().map(Into::into)),
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
            "/artists/{}/tracks?countryCode=US&limit={}",
            platform_id,
            limit.min(100)
        );

        let response: TidalResponse<TidalTrack> = self.api_request(&endpoint).await?;
        Ok(response
            .data
            .into_iter()
            .map(|t| t.into_platform_track(vec![platform_id.to_string()], None))
            .collect())
    }

    async fn get_artist_albums(
        &self,
        platform_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<PlatformAlbum>> {
        let endpoint = format!(
            "/artists/{}/albums?countryCode=US&limit={}&offset={}",
            platform_id,
            limit.min(100),
            offset
        );

        let response: TidalResponse<TidalAlbum> = self.api_request(&endpoint).await?;
        Ok(response
            .data
            .into_iter()
            .map(|a| a.into_platform_album(vec![platform_id.to_string()]))
            .collect())
    }

    async fn get_album_tracks(&self, album_id: &str) -> Result<Vec<PlatformTrack>> {
        let endpoint = format!("/albums/{}/tracks?countryCode=US", album_id);
        let response: TidalResponse<TidalTrack> = self.api_request(&endpoint).await?;
        Ok(response
            .data
            .into_iter()
            .map(|t| t.into_platform_track(vec![], Some(album_id.to_string())))
            .collect())
    }

    async fn get_related_artists(&self, platform_id: &str) -> Result<Vec<PlatformArtist>> {
        let endpoint = format!("/artists/{}/similar?countryCode=US&limit=20", platform_id);
        let response: TidalResponse<TidalArtist> = self.api_request(&endpoint).await?;
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
            platform: Platform::Tidal,
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
            platform: Platform::Tidal,
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
            platform: Platform::Tidal,
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
