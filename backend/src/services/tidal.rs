//! Tidal API service implementation
//!
//! Provides functionality for interacting with the Tidal API including:
//! - User profile retrieval
//! - Library scanning (favorites, playlists)
//! - Library modification (remove favorites, modify playlists)

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::models::tidal::{
    TidalAlbum, TidalArtist, TidalFavoriteAlbum, TidalFavoriteArtist, TidalFavoriteTrack,
    TidalLibrary, TidalLibraryScanResult, TidalPaginatedResponse, TidalPlaylist, TidalTrack,
    TidalUser,
};
use crate::models::token_vault::{Connection, ConnectionStatus, StreamingProvider};

/// Tidal OAuth token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: u64,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(rename = "user")]
    pub user_info: Option<TidalTokenUserInfo>,
}

/// User info included in token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TidalTokenUserInfo {
    #[serde(rename = "userId")]
    pub user_id: u64,
    pub email: Option<String>,
    #[serde(rename = "countryCode")]
    pub country_code: Option<String>,
}

/// Tidal API base URL
const TIDAL_API_BASE: &str = "https://api.tidal.com/v1";

/// Tidal OAuth base URL
const TIDAL_AUTH_BASE: &str = "https://auth.tidal.com/v1/oauth2";

/// Default page size for Tidal API requests
const DEFAULT_PAGE_SIZE: u32 = 100;

/// Rate limit: max requests per minute
const RATE_LIMIT_REQUESTS_PER_MINUTE: u32 = 100;

/// Tidal service configuration
#[derive(Debug, Clone)]
pub struct TidalConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

impl TidalConfig {
    /// Create a new TidalConfig from environment variables
    pub fn from_env() -> Result<Self> {
        let client_id = std::env::var("TIDAL_CLIENT_ID")
            .map_err(|_| anyhow!("TIDAL_CLIENT_ID environment variable is required"))?;
        let client_secret = std::env::var("TIDAL_CLIENT_SECRET")
            .map_err(|_| anyhow!("TIDAL_CLIENT_SECRET environment variable is required"))?;
        let redirect_uri = std::env::var("TIDAL_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:3000/auth/callback/tidal".to_string());

        Ok(Self {
            client_id,
            client_secret,
            redirect_uri,
        })
    }

    /// Create a TidalConfig with explicit values
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
        }
    }
}

/// Rate limit tracking for Tidal API
#[derive(Debug, Clone)]
struct RateLimitState {
    requests_made: u32,
    window_start: DateTime<Utc>,
    retry_after: Option<DateTime<Utc>>,
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self {
            requests_made: 0,
            window_start: Utc::now(),
            retry_after: None,
        }
    }
}

/// Tidal API service
pub struct TidalService {
    config: TidalConfig,
    client: Client,
    rate_limit: Arc<RwLock<RateLimitState>>,
}

impl TidalService {
    /// Create a new TidalService
    pub fn new(config: TidalConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            rate_limit: Arc::new(RwLock::new(RateLimitState::default())),
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        let config = TidalConfig::from_env()?;
        Ok(Self::new(config))
    }

    /// Get the OAuth authorization URL
    pub fn get_auth_url(&self, state: &str) -> String {
        let scopes = [
            "r_usr",        // Read user info
            "w_usr",        // Write user info
            "r_sub",        // Read subscription info
            "r_collection", // Read collection/favorites
            "w_collection", // Write collection/favorites
            "r_playlist",   // Read playlists
            "w_playlist",   // Write playlists
        ]
        .join(" ");

        let params = [
            ("client_id", self.config.client_id.as_str()),
            ("response_type", "code"),
            ("redirect_uri", self.config.redirect_uri.as_str()),
            ("scope", &scopes),
            ("state", state),
        ];

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        format!("{}/authorize?{}", TIDAL_AUTH_BASE, query_string)
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(&self, code: &str) -> Result<TidalTokenResponse> {
        let url = format!("{}/token", TIDAL_AUTH_BASE);

        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("redirect_uri", &self.config.redirect_uri),
        ];

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Tidal token exchange failed: {} - {}",
                status,
                error_text
            ));
        }

        let token_response: TidalTokenResponse = response.json().await?;
        Ok(token_response)
    }

    /// Refresh an access token using the refresh token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TidalTokenResponse> {
        let url = format!("{}/token", TIDAL_AUTH_BASE);

        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
        ];

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Tidal token refresh failed: {} - {}",
                status,
                error_text
            ));
        }

        let token_response: TidalTokenResponse = response.json().await?;
        Ok(token_response)
    }

    /// Wait for rate limit if necessary
    async fn wait_for_rate_limit(&self) -> Result<()> {
        let mut state = self.rate_limit.write().await;

        // Check if we need to wait for a retry-after
        if let Some(retry_after) = state.retry_after {
            let now = Utc::now();
            if now < retry_after {
                let wait_duration = (retry_after - now)
                    .to_std()
                    .unwrap_or(Duration::from_secs(1));
                drop(state); // Release lock while waiting
                info!("Rate limited, waiting {:?}", wait_duration);
                sleep(wait_duration).await;
                state = self.rate_limit.write().await;
                state.retry_after = None;
            }
        }

        // Check if we need to reset the window
        let now = Utc::now();
        let window_duration = chrono::Duration::minutes(1);
        if now - state.window_start > window_duration {
            state.requests_made = 0;
            state.window_start = now;
        }

        // Check if we've hit the rate limit
        if state.requests_made >= RATE_LIMIT_REQUESTS_PER_MINUTE {
            let wait_until = state.window_start + window_duration;
            let wait_duration = (wait_until - now)
                .to_std()
                .unwrap_or(Duration::from_secs(60));
            drop(state);
            warn!(
                "Rate limit reached, waiting {:?} for window reset",
                wait_duration
            );
            sleep(wait_duration).await;
            let mut state = self.rate_limit.write().await;
            state.requests_made = 0;
            state.window_start = Utc::now();
        } else {
            state.requests_made += 1;
        }

        Ok(())
    }

    /// Handle rate limit response from API
    async fn handle_rate_limit_response(&self, retry_after_secs: Option<u64>) {
        let mut state = self.rate_limit.write().await;
        let wait_secs = retry_after_secs.unwrap_or(60);
        state.retry_after = Some(Utc::now() + chrono::Duration::seconds(wait_secs as i64));
    }

    /// Make an authenticated GET request to the Tidal API
    async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        access_token: &str,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        self.wait_for_rate_limit().await?;

        let url = format!("{}{}", TIDAL_API_BASE, endpoint);
        let mut request = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/json");

        for (key, value) in params {
            request = request.query(&[(key, value)]);
        }

        debug!("Tidal API GET: {}", endpoint);
        let response = request.send().await?;
        let status = response.status();

        if status == StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok());
            self.handle_rate_limit_response(retry_after).await;
            return Err(anyhow!("Rate limited by Tidal API"));
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Tidal API request failed: {} - {}",
                status,
                error_text
            ));
        }

        let result: T = response.json().await?;
        Ok(result)
    }

    /// Make an authenticated DELETE request to the Tidal API
    async fn delete(&self, access_token: &str, endpoint: &str) -> Result<()> {
        self.wait_for_rate_limit().await?;

        let url = format!("{}{}", TIDAL_API_BASE, endpoint);
        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        let status = response.status();

        if status == StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok());
            self.handle_rate_limit_response(retry_after).await;
            return Err(anyhow!("Rate limited by Tidal API"));
        }

        if !status.is_success() && status != StatusCode::NO_CONTENT {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Tidal API DELETE failed: {} - {}",
                status,
                error_text
            ));
        }

        Ok(())
    }

    /// Make an authenticated POST request to the Tidal API (form-encoded)
    async fn post(
        &self,
        access_token: &str,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<()> {
        self.wait_for_rate_limit().await?;

        let url = format!("{}{}", TIDAL_API_BASE, endpoint);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(params)
            .send()
            .await?;

        let status = response.status();

        if status == StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok());
            self.handle_rate_limit_response(retry_after).await;
            return Err(anyhow!("Rate limited by Tidal API"));
        }

        if !status.is_success() && status != StatusCode::CREATED && status != StatusCode::NO_CONTENT
        {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Tidal API POST failed: {} - {}",
                status,
                error_text
            ));
        }

        Ok(())
    }

    /// Get the current user's profile
    pub async fn get_current_user(&self, access_token: &str) -> Result<TidalUser> {
        #[derive(Deserialize)]
        struct SessionResponse {
            #[serde(rename = "userId")]
            user_id: u64,
            #[serde(rename = "countryCode")]
            country_code: String,
        }

        // First get the session to get the user ID
        let session: SessionResponse = self.get(access_token, "/sessions", &[]).await?;

        // Then get the full user profile
        let user: TidalUser = self
            .get(
                access_token,
                &format!("/users/{}", session.user_id),
                &[("countryCode", &session.country_code)],
            )
            .await?;

        Ok(user)
    }

    /// Get user's favorite tracks with pagination
    pub async fn get_favorite_tracks(
        &self,
        access_token: &str,
        user_id: u64,
        country_code: &str,
        limit: u32,
        offset: u32,
    ) -> Result<TidalPaginatedResponse<TidalFavoriteTrack>> {
        let limit_str = limit.to_string();
        let offset_str = offset.to_string();

        self.get(
            access_token,
            &format!("/users/{}/favorites/tracks", user_id),
            &[
                ("countryCode", country_code),
                ("limit", &limit_str),
                ("offset", &offset_str),
            ],
        )
        .await
    }

    /// Get all user's favorite tracks
    pub async fn get_all_favorite_tracks(
        &self,
        access_token: &str,
        user_id: u64,
        country_code: &str,
    ) -> Result<Vec<TidalFavoriteTrack>> {
        let mut all_tracks = Vec::new();
        let mut offset = 0;

        loop {
            let response = self
                .get_favorite_tracks(
                    access_token,
                    user_id,
                    country_code,
                    DEFAULT_PAGE_SIZE,
                    offset,
                )
                .await?;

            all_tracks.extend(response.items);

            if all_tracks.len() as u32 >= response.total_number_of_items {
                break;
            }

            offset += DEFAULT_PAGE_SIZE;
        }

        Ok(all_tracks)
    }

    /// Get user's favorite artists with pagination
    pub async fn get_favorite_artists(
        &self,
        access_token: &str,
        user_id: u64,
        country_code: &str,
        limit: u32,
        offset: u32,
    ) -> Result<TidalPaginatedResponse<TidalFavoriteArtist>> {
        let limit_str = limit.to_string();
        let offset_str = offset.to_string();

        self.get(
            access_token,
            &format!("/users/{}/favorites/artists", user_id),
            &[
                ("countryCode", country_code),
                ("limit", &limit_str),
                ("offset", &offset_str),
            ],
        )
        .await
    }

    /// Get all user's favorite artists
    pub async fn get_all_favorite_artists(
        &self,
        access_token: &str,
        user_id: u64,
        country_code: &str,
    ) -> Result<Vec<TidalFavoriteArtist>> {
        let mut all_artists = Vec::new();
        let mut offset = 0;

        loop {
            let response = self
                .get_favorite_artists(
                    access_token,
                    user_id,
                    country_code,
                    DEFAULT_PAGE_SIZE,
                    offset,
                )
                .await?;

            all_artists.extend(response.items);

            if all_artists.len() as u32 >= response.total_number_of_items {
                break;
            }

            offset += DEFAULT_PAGE_SIZE;
        }

        Ok(all_artists)
    }

    /// Get user's favorite albums with pagination
    pub async fn get_favorite_albums(
        &self,
        access_token: &str,
        user_id: u64,
        country_code: &str,
        limit: u32,
        offset: u32,
    ) -> Result<TidalPaginatedResponse<TidalFavoriteAlbum>> {
        let limit_str = limit.to_string();
        let offset_str = offset.to_string();

        self.get(
            access_token,
            &format!("/users/{}/favorites/albums", user_id),
            &[
                ("countryCode", country_code),
                ("limit", &limit_str),
                ("offset", &offset_str),
            ],
        )
        .await
    }

    /// Get all user's favorite albums
    pub async fn get_all_favorite_albums(
        &self,
        access_token: &str,
        user_id: u64,
        country_code: &str,
    ) -> Result<Vec<TidalFavoriteAlbum>> {
        let mut all_albums = Vec::new();
        let mut offset = 0;

        loop {
            let response = self
                .get_favorite_albums(
                    access_token,
                    user_id,
                    country_code,
                    DEFAULT_PAGE_SIZE,
                    offset,
                )
                .await?;

            all_albums.extend(response.items);

            if all_albums.len() as u32 >= response.total_number_of_items {
                break;
            }

            offset += DEFAULT_PAGE_SIZE;
        }

        Ok(all_albums)
    }

    /// Get user's playlists with pagination
    pub async fn get_playlists(
        &self,
        access_token: &str,
        user_id: u64,
        country_code: &str,
        limit: u32,
        offset: u32,
    ) -> Result<TidalPaginatedResponse<TidalPlaylist>> {
        let limit_str = limit.to_string();
        let offset_str = offset.to_string();

        self.get(
            access_token,
            &format!("/users/{}/playlists", user_id),
            &[
                ("countryCode", country_code),
                ("limit", &limit_str),
                ("offset", &offset_str),
            ],
        )
        .await
    }

    /// Get all user's playlists
    pub async fn get_all_playlists(
        &self,
        access_token: &str,
        user_id: u64,
        country_code: &str,
    ) -> Result<Vec<TidalPlaylist>> {
        let mut all_playlists = Vec::new();
        let mut offset = 0;

        loop {
            let response = self
                .get_playlists(
                    access_token,
                    user_id,
                    country_code,
                    DEFAULT_PAGE_SIZE,
                    offset,
                )
                .await?;

            all_playlists.extend(response.items);

            if all_playlists.len() as u32 >= response.total_number_of_items {
                break;
            }

            offset += DEFAULT_PAGE_SIZE;
        }

        Ok(all_playlists)
    }

    /// Remove a track from favorites
    pub async fn remove_favorite_track(
        &self,
        access_token: &str,
        user_id: u64,
        track_id: u64,
    ) -> Result<()> {
        self.delete(
            access_token,
            &format!("/users/{}/favorites/tracks/{}", user_id, track_id),
        )
        .await
    }

    /// Remove an artist from favorites
    pub async fn remove_favorite_artist(
        &self,
        access_token: &str,
        user_id: u64,
        artist_id: u64,
    ) -> Result<()> {
        self.delete(
            access_token,
            &format!("/users/{}/favorites/artists/{}", user_id, artist_id),
        )
        .await
    }

    /// Remove an album from favorites
    pub async fn remove_favorite_album(
        &self,
        access_token: &str,
        user_id: u64,
        album_id: u64,
    ) -> Result<()> {
        self.delete(
            access_token,
            &format!("/users/{}/favorites/albums/{}", user_id, album_id),
        )
        .await
    }

    /// Remove a track from a playlist
    pub async fn remove_playlist_track(
        &self,
        access_token: &str,
        playlist_uuid: &str,
        track_index: u32,
    ) -> Result<()> {
        self.delete(
            access_token,
            &format!("/playlists/{}/items/{}", playlist_uuid, track_index),
        )
        .await
    }

    /// Add a track to favorites (for rollback support)
    pub async fn add_favorite_track(
        &self,
        access_token: &str,
        user_id: u64,
        track_id: u64,
    ) -> Result<()> {
        self.post(
            access_token,
            &format!("/users/{}/favorites/tracks", user_id),
            &[("trackIds", &track_id.to_string())],
        )
        .await
    }

    /// Add an artist to favorites (for rollback support)
    pub async fn add_favorite_artist(
        &self,
        access_token: &str,
        user_id: u64,
        artist_id: u64,
    ) -> Result<()> {
        self.post(
            access_token,
            &format!("/users/{}/favorites/artists", user_id),
            &[("artistIds", &artist_id.to_string())],
        )
        .await
    }

    /// Add an album to favorites (for rollback support)
    pub async fn add_favorite_album(
        &self,
        access_token: &str,
        user_id: u64,
        album_id: u64,
    ) -> Result<()> {
        self.post(
            access_token,
            &format!("/users/{}/favorites/albums", user_id),
            &[("albumIds", &album_id.to_string())],
        )
        .await
    }

    /// Scan user's full library
    pub async fn scan_library(
        &self,
        access_token: &str,
        internal_user_id: Uuid,
    ) -> Result<TidalLibraryScanResult> {
        let started_at = Utc::now();
        let mut api_requests_count = 0;
        let mut rate_limit_retries = 0;
        let mut warnings = Vec::new();

        // Get user profile first
        let user = match self.get_current_user(access_token).await {
            Ok(u) => {
                api_requests_count += 1;
                u
            }
            Err(e) => {
                return Err(anyhow!("Failed to get user profile: {}", e));
            }
        };

        let country_code = user
            .country_code
            .clone()
            .unwrap_or_else(|| "US".to_string());

        // Get favorite tracks
        let favorite_tracks = match self
            .get_all_favorite_tracks(access_token, user.id, &country_code)
            .await
        {
            Ok(tracks) => {
                api_requests_count += (tracks.len() as u32 / DEFAULT_PAGE_SIZE) + 1;
                tracks
            }
            Err(e) => {
                warnings.push(format!("Failed to get favorite tracks: {}", e));
                Vec::new()
            }
        };

        // Get favorite artists
        let favorite_artists = match self
            .get_all_favorite_artists(access_token, user.id, &country_code)
            .await
        {
            Ok(artists) => {
                api_requests_count += (artists.len() as u32 / DEFAULT_PAGE_SIZE) + 1;
                artists
            }
            Err(e) => {
                warnings.push(format!("Failed to get favorite artists: {}", e));
                Vec::new()
            }
        };

        // Get favorite albums
        let favorite_albums = match self
            .get_all_favorite_albums(access_token, user.id, &country_code)
            .await
        {
            Ok(albums) => {
                api_requests_count += (albums.len() as u32 / DEFAULT_PAGE_SIZE) + 1;
                albums
            }
            Err(e) => {
                warnings.push(format!("Failed to get favorite albums: {}", e));
                Vec::new()
            }
        };

        // Get playlists
        let playlists = match self
            .get_all_playlists(access_token, user.id, &country_code)
            .await
        {
            Ok(playlists) => {
                api_requests_count += (playlists.len() as u32 / DEFAULT_PAGE_SIZE) + 1;
                playlists
            }
            Err(e) => {
                warnings.push(format!("Failed to get playlists: {}", e));
                Vec::new()
            }
        };

        let library = TidalLibrary {
            user_id: internal_user_id,
            tidal_user_id: user.id,
            favorite_tracks,
            favorite_artists,
            favorite_albums,
            playlists,
            scanned_at: Utc::now(),
        };

        Ok(TidalLibraryScanResult::new(
            library,
            started_at,
            api_requests_count,
            rate_limit_retries,
            warnings,
        ))
    }

    /// Get current rate limit status
    pub async fn get_rate_limit_status(&self) -> (u32, u32) {
        let state = self.rate_limit.read().await;
        (state.requests_made, RATE_LIMIT_REQUESTS_PER_MINUTE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tidal_config_new() {
        let config = TidalConfig::new(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            "http://localhost:3000/callback".to_string(),
        );

        assert_eq!(config.client_id, "test_client_id");
        assert_eq!(config.client_secret, "test_client_secret");
        assert_eq!(config.redirect_uri, "http://localhost:3000/callback");
    }

    #[test]
    fn test_get_auth_url() {
        let config = TidalConfig::new(
            "test_client_id".to_string(),
            "test_secret".to_string(),
            "http://localhost:3000/callback".to_string(),
        );
        let service = TidalService::new(config);
        let url = service.get_auth_url("test_state");

        assert!(url.starts_with("https://auth.tidal.com/v1/oauth2/authorize"));
        assert!(url.contains("client_id=test_client_id"));
        assert!(url.contains("state=test_state"));
        assert!(url.contains("response_type=code"));
    }

    #[test]
    fn test_rate_limit_state_default() {
        let state = RateLimitState::default();
        assert_eq!(state.requests_made, 0);
        assert!(state.retry_after.is_none());
    }
}
