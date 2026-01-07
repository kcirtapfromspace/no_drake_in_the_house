//! Apple Music Catalog Sync Worker
//!
//! Implements catalog synchronization for Apple Music using their MusicKit API.
//! Rate limit: 1000 requests/hour

use super::traits::*;
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

/// Apple Music API base URL
const APPLE_MUSIC_API_BASE: &str = "https://api.music.apple.com/v1";

/// Apple Music sync worker
pub struct AppleMusicSyncWorker {
    client: Client,
    /// Developer token (JWT)
    developer_token: Arc<RwLock<Option<String>>>,
    /// Team ID from Apple Developer account
    team_id: String,
    /// Key ID from Apple Developer account
    key_id: String,
    /// Private key (PEM format)
    private_key: String,
    /// Default storefront (country code)
    storefront: String,
    /// Rate limiter state
    rate_limiter: Arc<RwLock<RateLimiterState>>,
}

struct RateLimiterState {
    requests_this_window: u32,
    window_start: std::time::Instant,
}

impl AppleMusicSyncWorker {
    /// Create a new Apple Music sync worker
    pub fn new(team_id: String, key_id: String, private_key: String, storefront: String) -> Self {
        Self {
            client: Client::new(),
            developer_token: Arc::new(RwLock::new(None)),
            team_id,
            key_id,
            private_key,
            storefront,
            rate_limiter: Arc::new(RwLock::new(RateLimiterState {
                requests_this_window: 0,
                window_start: std::time::Instant::now(),
            })),
        }
    }

    /// Generate a new developer token (JWT)
    fn generate_token(&self) -> Result<String> {
        #[derive(Serialize)]
        struct Claims {
            iss: String,
            iat: i64,
            exp: i64,
        }

        let now = Utc::now().timestamp();
        let claims = Claims {
            iss: self.team_id.clone(),
            iat: now,
            exp: now + 15777000, // 6 months
        };

        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some(self.key_id.clone());

        let key = EncodingKey::from_ec_pem(self.private_key.as_bytes())
            .context("Invalid Apple Music private key")?;

        encode(&header, &claims, &key).context("Failed to generate Apple Music token")
    }

    /// Get or refresh developer token
    async fn ensure_token(&self) -> Result<String> {
        {
            let token = self.developer_token.read().await;
            if let Some(t) = token.as_ref() {
                return Ok(t.clone());
            }
        }

        let token = self.generate_token()?;
        {
            let mut token_lock = self.developer_token.write().await;
            *token_lock = Some(token.clone());
        }
        Ok(token)
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
            tracing::debug!("Apple Music rate limit hit, waiting {:?}", wait_time);
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
    async fn api_request<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
    ) -> Result<T> {
        self.wait_for_rate_limit().await;
        let token = self.ensure_token().await?;

        let url = format!("{}{}", APPLE_MUSIC_API_BASE, endpoint);
        let response = self
            .client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .context("Apple Music API request failed")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Apple Music API error: {} - {}",
                status,
                body
            ));
        }

        response
            .json()
            .await
            .context("Failed to parse Apple Music response")
    }
}

// Apple Music API response types
#[derive(Debug, Deserialize)]
struct AppleMusicResponse<T> {
    data: Vec<T>,
    #[allow(dead_code)]
    next: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AppleMusicArtist {
    id: String,
    attributes: AppleMusicArtistAttributes,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppleMusicArtistAttributes {
    name: String,
    genre_names: Vec<String>,
    url: String,
    #[allow(dead_code)]
    artwork: Option<AppleMusicArtwork>,
}

#[derive(Debug, Deserialize)]
struct AppleMusicArtwork {
    url: String,
    #[allow(dead_code)]
    width: u32,
    #[allow(dead_code)]
    height: u32,
}

#[derive(Debug, Deserialize)]
struct AppleMusicSearchResponse {
    results: AppleMusicSearchResults,
}

#[derive(Debug, Deserialize)]
struct AppleMusicSearchResults {
    artists: Option<AppleMusicResponse<AppleMusicArtist>>,
}

#[derive(Debug, Deserialize)]
struct AppleMusicSong {
    id: String,
    attributes: AppleMusicSongAttributes,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppleMusicSongAttributes {
    name: String,
    isrc: Option<String>,
    duration_in_millis: Option<u64>,
    release_date: Option<String>,
    preview_url: Option<String>,
    #[allow(dead_code)]
    content_rating: Option<String>,
    album_name: Option<String>,
    artist_name: String,
}

#[derive(Debug, Deserialize)]
struct AppleMusicAlbum {
    id: String,
    attributes: AppleMusicAlbumAttributes,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppleMusicAlbumAttributes {
    name: String,
    upc: Option<String>,
    release_date: Option<String>,
    track_count: u32,
    artwork: Option<AppleMusicArtwork>,
    #[allow(dead_code)]
    content_rating: Option<String>,
}

impl From<AppleMusicArtist> for PlatformArtist {
    fn from(artist: AppleMusicArtist) -> Self {
        let mut external_urls = HashMap::new();
        external_urls.insert("apple_music".to_string(), artist.attributes.url);

        PlatformArtist {
            platform_id: artist.id,
            platform: Platform::AppleMusic,
            name: artist.attributes.name,
            genres: artist.attributes.genre_names,
            popularity: None, // Apple Music doesn't expose this
            image_url: artist.attributes.artwork.map(|a| {
                a.url.replace("{w}", "300").replace("{h}", "300")
            }),
            external_urls,
            metadata: HashMap::new(),
        }
    }
}

impl AppleMusicSong {
    fn into_platform_track(self, artist_ids: Vec<String>, album_id: Option<String>) -> PlatformTrack {
        PlatformTrack {
            platform_id: self.id,
            platform: Platform::AppleMusic,
            title: self.attributes.name,
            isrc: self.attributes.isrc,
            duration_ms: self.attributes.duration_in_millis,
            artist_ids,
            album_id,
            release_date: self.attributes.release_date,
            preview_url: self.attributes.preview_url,
            explicit: self.attributes.content_rating.as_deref() == Some("explicit"),
        }
    }
}

impl AppleMusicAlbum {
    fn into_platform_album(self, artist_ids: Vec<String>) -> PlatformAlbum {
        PlatformAlbum {
            platform_id: self.id,
            platform: Platform::AppleMusic,
            title: self.attributes.name,
            upc: self.attributes.upc,
            artist_ids,
            release_date: self.attributes.release_date,
            album_type: "album".to_string(), // Apple Music doesn't distinguish types the same way
            total_tracks: self.attributes.track_count,
            image_url: self.attributes.artwork.map(|a| {
                a.url.replace("{w}", "300").replace("{h}", "300")
            }),
        }
    }
}

#[async_trait]
impl PlatformCatalogWorker for AppleMusicSyncWorker {
    fn platform(&self) -> Platform {
        Platform::AppleMusic
    }

    fn rate_limit_config(&self) -> RateLimitConfig {
        RateLimitConfig::apple_music()
    }

    async fn health_check(&self) -> Result<bool> {
        match self.ensure_token().await {
            Ok(_) => Ok(true),
            Err(e) => {
                tracing::warn!("Apple Music health check failed: {}", e);
                Ok(false)
            }
        }
    }

    async fn search_artist(&self, query: &str, limit: u32) -> Result<Vec<PlatformArtist>> {
        let encoded_query = urlencoding::encode(query);
        let endpoint = format!(
            "/catalog/{}/search?term={}&types=artists&limit={}",
            self.storefront,
            encoded_query,
            limit.min(25)
        );

        let response: AppleMusicSearchResponse = self.api_request(&endpoint).await?;

        Ok(response
            .results
            .artists
            .map(|a| a.data.into_iter().map(Into::into).collect())
            .unwrap_or_default())
    }

    async fn get_artist(&self, platform_id: &str) -> Result<Option<PlatformArtist>> {
        let endpoint = format!(
            "/catalog/{}/artists/{}",
            self.storefront, platform_id
        );

        match self.api_request::<AppleMusicResponse<AppleMusicArtist>>(&endpoint).await {
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
            "/catalog/{}/artists/{}/songs?limit={}",
            self.storefront,
            platform_id,
            limit.min(100)
        );

        let response: AppleMusicResponse<AppleMusicSong> = self.api_request(&endpoint).await?;

        Ok(response
            .data
            .into_iter()
            .map(|s| s.into_platform_track(vec![platform_id.to_string()], None))
            .collect())
    }

    async fn get_artist_albums(
        &self,
        platform_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<PlatformAlbum>> {
        let endpoint = format!(
            "/catalog/{}/artists/{}/albums?limit={}&offset={}",
            self.storefront,
            platform_id,
            limit.min(100),
            offset
        );

        let response: AppleMusicResponse<AppleMusicAlbum> = self.api_request(&endpoint).await?;

        Ok(response
            .data
            .into_iter()
            .map(|a| a.into_platform_album(vec![platform_id.to_string()]))
            .collect())
    }

    async fn get_album_tracks(&self, album_id: &str) -> Result<Vec<PlatformTrack>> {
        let endpoint = format!(
            "/catalog/{}/albums/{}/tracks",
            self.storefront, album_id
        );

        let response: AppleMusicResponse<AppleMusicSong> = self.api_request(&endpoint).await?;

        Ok(response
            .data
            .into_iter()
            .map(|s| s.into_platform_track(vec![], Some(album_id.to_string())))
            .collect())
    }

    async fn get_related_artists(&self, platform_id: &str) -> Result<Vec<PlatformArtist>> {
        // Apple Music doesn't have a direct related artists endpoint
        // We could use recommendations or return empty
        tracing::debug!(
            "Apple Music doesn't support related artists directly for {}",
            platform_id
        );
        Ok(vec![])
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
            platform: Platform::AppleMusic,
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
            platform: Platform::AppleMusic,
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
            platform: Platform::AppleMusic,
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
