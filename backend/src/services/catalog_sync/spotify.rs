//! Spotify Catalog Sync Worker
//!
//! Implements catalog synchronization for Spotify using their Web API.
//! Rate limit: 100 requests/minute

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

/// Spotify API base URL
const SPOTIFY_API_BASE: &str = "https://api.spotify.com/v1";

/// Spotify sync worker
pub struct SpotifySyncWorker {
    client: Client,
    access_token: Arc<RwLock<Option<String>>>,
    client_id: String,
    client_secret: String,
    /// Rate limiter state
    rate_limiter: Arc<RwLock<RateLimiterState>>,
}

struct RateLimiterState {
    requests_this_window: u32,
    window_start: std::time::Instant,
}

impl SpotifySyncWorker {
    /// Create a new Spotify sync worker
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

    /// Get or refresh access token using client credentials flow
    async fn ensure_token(&self) -> Result<String> {
        // Check if we have a valid token
        {
            let token = self.access_token.read().await;
            if let Some(t) = token.as_ref() {
                return Ok(t.clone());
            }
        }

        // Get new token
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
            #[allow(dead_code)]
            token_type: String,
            #[allow(dead_code)]
            expires_in: u64,
        }

        let response = self
            .client
            .post("https://accounts.spotify.com/api/token")
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await
            .context("Failed to request Spotify token")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Spotify token request failed: {} - {}",
                status,
                body
            ));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .context("Failed to parse Spotify token response")?;

        Ok(token_response.access_token)
    }

    /// Wait for rate limit if needed
    async fn wait_for_rate_limit(&self) {
        let config = self.rate_limit_config();
        let mut state = self.rate_limiter.write().await;

        // Check if we need to reset the window
        let elapsed = state.window_start.elapsed();
        if elapsed.as_secs() >= config.window_seconds as u64 {
            state.requests_this_window = 0;
            state.window_start = std::time::Instant::now();
        }

        // Check if we've hit the limit
        if state.requests_this_window >= config.requests_per_window {
            let wait_time = Duration::from_secs(config.window_seconds as u64) - elapsed;
            tracing::debug!("Rate limit hit, waiting {:?}", wait_time);
            drop(state); // Release lock while waiting
            sleep(wait_time).await;

            // Reset after waiting
            let mut state = self.rate_limiter.write().await;
            state.requests_this_window = 0;
            state.window_start = std::time::Instant::now();
        } else {
            state.requests_this_window += 1;
        }
    }

    /// Make an authenticated API request
    async fn api_request<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
    ) -> Result<T> {
        self.wait_for_rate_limit().await;
        let token = self.ensure_token().await?;

        let url = format!("{}{}", SPOTIFY_API_BASE, endpoint);
        let response = self
            .client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .context("Spotify API request failed")?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            // Token expired, refresh and retry
            let mut token_lock = self.access_token.write().await;
            *token_lock = None;
            drop(token_lock);

            let new_token = self.refresh_token().await?;
            {
                let mut token_lock = self.access_token.write().await;
                *token_lock = Some(new_token.clone());
            }

            let response = self
                .client
                .get(&url)
                .bearer_auth(&new_token)
                .send()
                .await
                .context("Spotify API retry failed")?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!("Spotify API error: {} - {}", status, body));
            }

            response.json().await.context("Failed to parse Spotify response")
        } else if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Spotify API error: {} - {}", status, body))
        } else {
            response.json().await.context("Failed to parse Spotify response")
        }
    }
}

// Spotify API response types
#[derive(Debug, Deserialize)]
struct SpotifyArtist {
    id: String,
    name: String,
    genres: Vec<String>,
    popularity: u32,
    images: Vec<SpotifyImage>,
    external_urls: HashMap<String, String>,
    followers: Option<SpotifyFollowers>,
}

#[derive(Debug, Deserialize)]
struct SpotifyImage {
    url: String,
    #[allow(dead_code)]
    height: Option<u32>,
    #[allow(dead_code)]
    width: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct SpotifyFollowers {
    total: u64,
}

#[derive(Debug, Deserialize)]
struct SpotifySearchResponse {
    artists: SpotifyArtistPage,
}

#[derive(Debug, Deserialize)]
struct SpotifyArtistPage {
    items: Vec<SpotifyArtist>,
    #[allow(dead_code)]
    total: u32,
    #[allow(dead_code)]
    limit: u32,
    #[allow(dead_code)]
    offset: u32,
}

#[derive(Debug, Deserialize)]
struct SpotifyTrack {
    id: String,
    name: String,
    duration_ms: u64,
    explicit: bool,
    external_ids: Option<SpotifyExternalIds>,
    artists: Vec<SpotifySimpleArtist>,
    album: Option<SpotifySimpleAlbum>,
    preview_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SpotifyExternalIds {
    isrc: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SpotifySimpleArtist {
    id: String,
    #[allow(dead_code)]
    name: String,
}

#[derive(Debug, Deserialize)]
struct SpotifySimpleAlbum {
    id: String,
    release_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SpotifyTopTracksResponse {
    tracks: Vec<SpotifyTrack>,
}

#[derive(Debug, Deserialize)]
struct SpotifyAlbum {
    id: String,
    name: String,
    album_type: String,
    total_tracks: u32,
    release_date: Option<String>,
    images: Vec<SpotifyImage>,
    artists: Vec<SpotifySimpleArtist>,
    external_ids: Option<SpotifyAlbumExternalIds>,
}

#[derive(Debug, Deserialize)]
struct SpotifyAlbumExternalIds {
    upc: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SpotifyAlbumsResponse {
    items: Vec<SpotifyAlbum>,
    total: u32,
    #[allow(dead_code)]
    limit: u32,
    #[allow(dead_code)]
    offset: u32,
}

#[derive(Debug, Deserialize)]
struct SpotifyAlbumTracksResponse {
    items: Vec<SpotifyTrack>,
}

#[derive(Debug, Deserialize)]
struct SpotifyRelatedArtistsResponse {
    artists: Vec<SpotifyArtist>,
}

impl From<SpotifyArtist> for PlatformArtist {
    fn from(artist: SpotifyArtist) -> Self {
        PlatformArtist {
            platform_id: artist.id,
            platform: Platform::Spotify,
            name: artist.name,
            genres: artist.genres,
            popularity: artist.followers.map(|f| f.total),
            image_url: artist.images.first().map(|i| i.url.clone()),
            external_urls: artist.external_urls,
            metadata: HashMap::new(),
        }
    }
}

impl From<SpotifyTrack> for PlatformTrack {
    fn from(track: SpotifyTrack) -> Self {
        PlatformTrack {
            platform_id: track.id,
            platform: Platform::Spotify,
            title: track.name,
            isrc: track.external_ids.and_then(|e| e.isrc),
            duration_ms: Some(track.duration_ms),
            artist_ids: track.artists.into_iter().map(|a| a.id).collect(),
            album_id: track.album.as_ref().map(|a| a.id.clone()),
            release_date: track.album.and_then(|a| a.release_date),
            preview_url: track.preview_url,
            explicit: track.explicit,
        }
    }
}

impl From<SpotifyAlbum> for PlatformAlbum {
    fn from(album: SpotifyAlbum) -> Self {
        PlatformAlbum {
            platform_id: album.id,
            platform: Platform::Spotify,
            title: album.name,
            upc: album.external_ids.and_then(|e| e.upc),
            artist_ids: album.artists.into_iter().map(|a| a.id).collect(),
            release_date: album.release_date,
            album_type: album.album_type,
            total_tracks: album.total_tracks,
            image_url: album.images.first().map(|i| i.url.clone()),
        }
    }
}

#[async_trait]
impl PlatformCatalogWorker for SpotifySyncWorker {
    fn platform(&self) -> Platform {
        Platform::Spotify
    }

    fn rate_limit_config(&self) -> RateLimitConfig {
        RateLimitConfig::spotify()
    }

    async fn health_check(&self) -> Result<bool> {
        match self.ensure_token().await {
            Ok(_) => Ok(true),
            Err(e) => {
                tracing::warn!("Spotify health check failed: {}", e);
                Ok(false)
            }
        }
    }

    async fn search_artist(&self, query: &str, limit: u32) -> Result<Vec<PlatformArtist>> {
        let encoded_query = urlencoding::encode(query);
        let endpoint = format!(
            "/search?q={}&type=artist&limit={}",
            encoded_query,
            limit.min(50)
        );

        let response: SpotifySearchResponse = self.api_request(&endpoint).await?;
        Ok(response.artists.items.into_iter().map(Into::into).collect())
    }

    async fn get_artist(&self, platform_id: &str) -> Result<Option<PlatformArtist>> {
        let endpoint = format!("/artists/{}", platform_id);
        match self.api_request::<SpotifyArtist>(&endpoint).await {
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
        let endpoint = format!("/artists/{}/top-tracks?market=US", platform_id);
        let response: SpotifyTopTracksResponse = self.api_request(&endpoint).await?;
        Ok(response
            .tracks
            .into_iter()
            .take(limit as usize)
            .map(Into::into)
            .collect())
    }

    async fn get_artist_albums(
        &self,
        platform_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<PlatformAlbum>> {
        let endpoint = format!(
            "/artists/{}/albums?include_groups=album,single&limit={}&offset={}",
            platform_id,
            limit.min(50),
            offset
        );
        let response: SpotifyAlbumsResponse = self.api_request(&endpoint).await?;
        Ok(response.items.into_iter().map(Into::into).collect())
    }

    async fn get_album_tracks(&self, album_id: &str) -> Result<Vec<PlatformTrack>> {
        let endpoint = format!("/albums/{}/tracks?limit=50", album_id);
        let response: SpotifyAlbumTracksResponse = self.api_request(&endpoint).await?;
        Ok(response.items.into_iter().map(Into::into).collect())
    }

    async fn get_related_artists(&self, platform_id: &str) -> Result<Vec<PlatformArtist>> {
        let endpoint = format!("/artists/{}/related-artists", platform_id);
        let response: SpotifyRelatedArtistsResponse = self.api_request(&endpoint).await?;
        Ok(response.artists.into_iter().map(Into::into).collect())
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

        // Report initial progress
        progress_callback(SyncProgress {
            platform: Platform::Spotify,
            sync_run_id,
            status: SyncStatus::Running,
            total_items: None,
            items_processed: checkpoint.as_ref().map(|c| c.items_processed).unwrap_or(0),
            errors: 0,
            started_at,
            updated_at: Utc::now(),
            estimated_completion: None,
        });

        // For incremental sync, we would typically:
        // 1. Get new releases since last sync
        // 2. Check for updates to known artists
        // For now, return a placeholder result
        let result = SyncResult {
            platform: Platform::Spotify,
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
            platform: Platform::Spotify,
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
        // Full sync delegates to incremental with no checkpoint
        self.sync_incremental(None, progress_callback).await
    }

    async fn get_checkpoint(&self, _sync_run_id: Uuid) -> Result<Option<SyncCheckpoint>> {
        // Would load from database
        Ok(None)
    }

    async fn save_checkpoint(&self, _checkpoint: &SyncCheckpoint) -> Result<()> {
        // Would save to database
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spotify_artist_conversion() {
        let spotify_artist = SpotifyArtist {
            id: "123".to_string(),
            name: "Test Artist".to_string(),
            genres: vec!["rock".to_string()],
            popularity: 80,
            images: vec![SpotifyImage {
                url: "https://example.com/image.jpg".to_string(),
                height: Some(300),
                width: Some(300),
            }],
            external_urls: HashMap::new(),
            followers: Some(SpotifyFollowers { total: 1000000 }),
        };

        let platform_artist: PlatformArtist = spotify_artist.into();
        assert_eq!(platform_artist.platform, Platform::Spotify);
        assert_eq!(platform_artist.name, "Test Artist");
        assert_eq!(platform_artist.popularity, Some(1000000));
    }
}
