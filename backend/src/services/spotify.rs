use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, RefreshToken, Scope, TokenResponse, TokenUrl,
};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::models::{
    Connection, ConnectionStatus, DecryptedToken, SpotifyAlbum, SpotifyArtist,
    SpotifyFollowedArtist, SpotifyLibrary, SpotifyLibraryScanResult, SpotifyPlaylist,
    SpotifySavedTrack, StreamingProvider, TokenHealthCheck,
};
use crate::services::TokenVaultService;

/// Spotify OAuth configuration
#[derive(Debug, Clone)]
pub struct SpotifyConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub auth_url: String,
    pub token_url: String,
}

impl Default for SpotifyConfig {
    fn default() -> Self {
        Self {
            client_id: std::env::var("SPOTIFY_CLIENT_ID")
                .unwrap_or_else(|_| "your_spotify_client_id".to_string()),
            client_secret: std::env::var("SPOTIFY_CLIENT_SECRET")
                .unwrap_or_else(|_| "your_spotify_client_secret".to_string()),
            redirect_uri: std::env::var("SPOTIFY_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:3000/auth/spotify/callback".to_string()),
            auth_url: "https://accounts.spotify.com/authorize".to_string(),
            token_url: "https://accounts.spotify.com/api/token".to_string(),
        }
    }
}

/// Spotify OAuth authorization URL response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyAuthUrl {
    pub auth_url: String,
    pub state: String,
    pub code_verifier: String,
}

/// Spotify OAuth callback request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyCallbackRequest {
    pub code: String,
    pub state: String,
    pub code_verifier: String,
}

/// Spotify user profile from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyUserProfile {
    pub id: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub country: Option<String>,
    pub product: Option<String>,
}

/// Rate limiting state for Spotify API
#[derive(Debug, Clone)]
pub struct RateLimit {
    pub remaining: u32,
    pub reset_at: DateTime<Utc>,
    pub retry_after: Option<u32>,
}

/// Spotify API client with OAuth and rate limiting
pub struct SpotifyService {
    config: SpotifyConfig,
    oauth_client: BasicClient,
    http_client: Client,
    token_vault: Arc<TokenVaultService>,
    rate_limits: Arc<RwLock<HashMap<String, RateLimit>>>,
    pending_auth_sessions: Arc<RwLock<HashMap<String, PkceCodeVerifier>>>,
}

impl SpotifyService {
    pub fn new(config: SpotifyConfig, token_vault: Arc<TokenVaultService>) -> Result<Self> {
        let oauth_client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new(config.auth_url.clone())?,
            Some(TokenUrl::new(config.token_url.clone())?),
        )
        .set_redirect_uri(RedirectUrl::new(config.redirect_uri.clone())?);

        let http_client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        Ok(Self {
            config,
            oauth_client,
            http_client,
            token_vault,
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            pending_auth_sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Generate Spotify OAuth authorization URL with PKCE
    pub async fn get_auth_url(&self) -> Result<SpotifyAuthUrl> {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = self
            .oauth_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("user-read-private".to_string()))
            .add_scope(Scope::new("user-read-email".to_string()))
            .add_scope(Scope::new("user-library-read".to_string()))
            .add_scope(Scope::new("user-library-modify".to_string()))
            .add_scope(Scope::new("playlist-read-private".to_string()))
            .add_scope(Scope::new("playlist-modify-private".to_string()))
            .add_scope(Scope::new("playlist-modify-public".to_string()))
            .add_scope(Scope::new("user-follow-read".to_string()))
            .add_scope(Scope::new("user-follow-modify".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        let state = csrf_token.secret().clone();

        // Store the PKCE verifier for later use
        self.pending_auth_sessions
            .write()
            .await
            .insert(state.clone(), pkce_verifier);

        Ok(SpotifyAuthUrl {
            auth_url: auth_url.to_string(),
            state,
            code_verifier: "stored_internally".to_string(), // Don't expose the actual verifier
        })
    }

    /// Handle OAuth callback and exchange code for tokens
    pub async fn handle_callback(
        &self,
        user_id: Uuid,
        callback_request: SpotifyCallbackRequest,
    ) -> Result<Connection> {
        // Retrieve the stored PKCE verifier
        let pkce_verifier = {
            let mut sessions = self.pending_auth_sessions.write().await;
            sessions
                .remove(&callback_request.state)
                .ok_or_else(|| anyhow!("Invalid or expired OAuth state"))?
        };

        // Exchange authorization code for tokens
        let token_result = self
            .oauth_client
            .exchange_code(AuthorizationCode::new(callback_request.code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
            .map_err(|e| anyhow!("Token exchange failed: {}", e))?;

        let access_token = token_result.access_token().secret().clone();
        let refresh_token = token_result.refresh_token().map(|rt| rt.secret().clone());
        let expires_at = token_result
            .expires_in()
            .map(|duration| Utc::now() + chrono::Duration::seconds(duration.as_secs() as i64));

        // Get user profile to obtain Spotify user ID
        let user_profile = self.get_user_profile(&access_token).await?;

        // Determine scopes (Spotify doesn't return scopes in token response)
        let scopes = vec![
            "user-read-private".to_string(),
            "user-read-email".to_string(),
            "user-library-read".to_string(),
            "user-library-modify".to_string(),
            "playlist-read-private".to_string(),
            "playlist-modify-private".to_string(),
            "playlist-modify-public".to_string(),
            "user-follow-read".to_string(),
            "user-follow-modify".to_string(),
        ];

        // Store the connection using the token vault
        let store_request = crate::models::StoreTokenRequest {
            user_id,
            provider: StreamingProvider::Spotify,
            provider_user_id: user_profile.id,
            access_token,
            refresh_token,
            scopes,
            expires_at,
        };

        self.token_vault.store_token(store_request).await
    }

    /// Get Spotify user profile
    async fn get_user_profile(&self, access_token: &str) -> Result<SpotifyUserProfile> {
        let response = self
            .http_client
            .get("https://api.spotify.com/v1/me")
            .bearer_auth(access_token)
            .send()
            .await?;

        if response.status().is_success() {
            let profile: SpotifyUserProfile = response.json().await?;
            Ok(profile)
        } else {
            Err(anyhow!("Failed to get user profile: {}", response.status()))
        }
    }

    /// Check token health and validity
    pub async fn check_token_health(&self, connection: &Connection) -> Result<TokenHealthCheck> {
        let decrypted_token = self.token_vault.get_decrypted_token(connection.id).await?;

        let response = self
            .http_client
            .get("https://api.spotify.com/v1/me")
            .bearer_auth(&decrypted_token.access_token)
            .send()
            .await?;

        let is_valid = response.status().is_success();
        let needs_refresh = connection.needs_refresh();

        Ok(TokenHealthCheck {
            connection_id: connection.id,
            is_valid,
            expires_at: connection.expires_at,
            error_message: if !is_valid {
                Some(format!("HTTP {}", response.status()))
            } else {
                None
            },
            checked_at: Utc::now(),
            needs_refresh,
        })
    }

    /// Refresh access token using refresh token
    pub async fn refresh_token(&self, connection: &mut Connection) -> Result<()> {
        let decrypted_token = self.token_vault.get_decrypted_token(connection.id).await?;

        let refresh_token = decrypted_token
            .refresh_token
            .clone()
            .ok_or_else(|| anyhow!("No refresh token available"))?;

        let token_result = self
            .oauth_client
            .exchange_refresh_token(&RefreshToken::new(refresh_token))
            .request_async(async_http_client)
            .await
            .map_err(|e| anyhow!("Token refresh failed: {}", e))?;

        let new_access_token = token_result.access_token().secret().clone();
        let new_refresh_token = token_result
            .refresh_token()
            .map(|rt| rt.secret().clone())
            .or(decrypted_token.refresh_token); // Keep old refresh token if new one not provided
        let new_expires_at = token_result
            .expires_in()
            .map(|duration| Utc::now() + chrono::Duration::seconds(duration.as_secs() as i64));

        // Update the connection with new tokens
        let store_request = crate::models::StoreTokenRequest {
            user_id: connection.user_id,
            provider: StreamingProvider::Spotify,
            provider_user_id: connection.provider_user_id.clone(),
            access_token: new_access_token,
            refresh_token: new_refresh_token,
            scopes: connection.scopes.clone(),
            expires_at: new_expires_at,
        };

        let updated_connection = self.token_vault.store_token(store_request).await?;

        // Update the connection object
        connection.access_token_encrypted = updated_connection.access_token_encrypted;
        connection.refresh_token_encrypted = updated_connection.refresh_token_encrypted;
        connection.expires_at = updated_connection.expires_at;
        connection.token_version = updated_connection.token_version;
        connection.status = ConnectionStatus::Active;
        connection.error_code = None;
        connection.updated_at = Utc::now();

        Ok(())
    }

    /// Make authenticated API request with rate limiting
    pub async fn make_api_request(
        &self,
        connection: &Connection,
        method: &str,
        url: &str,
        body: Option<serde_json::Value>,
    ) -> Result<reqwest::Response> {
        // Check and wait for rate limits
        self.wait_for_rate_limit().await?;

        // Get decrypted token
        let decrypted_token = self.token_vault.get_decrypted_token(connection.id).await?;

        // Build request
        let mut request_builder = match method.to_uppercase().as_str() {
            "GET" => self.http_client.get(url),
            "POST" => self.http_client.post(url),
            "PUT" => self.http_client.put(url),
            "DELETE" => self.http_client.delete(url),
            _ => return Err(anyhow!("Unsupported HTTP method: {}", method)),
        };

        request_builder = request_builder.bearer_auth(&decrypted_token.access_token);

        if let Some(json_body) = body {
            request_builder = request_builder.json(&json_body);
        }

        let response = request_builder.send().await?;

        // Update rate limit information from response headers
        self.update_rate_limit_from_response(&response).await;

        // Handle rate limiting
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(60);

            self.set_rate_limit_retry_after(retry_after).await;
            return Err(anyhow!("Rate limited, retry after {} seconds", retry_after));
        }

        Ok(response)
    }

    /// Wait for rate limit to reset if necessary
    async fn wait_for_rate_limit(&self) -> Result<()> {
        let rate_limits = self.rate_limits.read().await;
        if let Some(rate_limit) = rate_limits.get("global") {
            if rate_limit.remaining == 0 && Utc::now() < rate_limit.reset_at {
                let wait_duration = (rate_limit.reset_at - Utc::now()).to_std()?;
                drop(rate_limits); // Release the lock before sleeping
                tokio::time::sleep(wait_duration).await;
            }
        }
        Ok(())
    }

    /// Update rate limit information from API response headers
    async fn update_rate_limit_from_response(&self, response: &reqwest::Response) {
        let mut rate_limits = self.rate_limits.write().await;

        // Spotify doesn't provide standard rate limit headers, so we use a simple approach
        // In a real implementation, you'd parse actual Spotify rate limit headers
        let remaining = response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(100); // Default assumption

        let reset_at = Utc::now() + chrono::Duration::seconds(60); // Default 1-minute window

        rate_limits.insert(
            "global".to_string(),
            RateLimit {
                remaining,
                reset_at,
                retry_after: None,
            },
        );
    }

    /// Set rate limit retry-after from 429 response
    async fn set_rate_limit_retry_after(&self, retry_after_seconds: u32) {
        let mut rate_limits = self.rate_limits.write().await;
        let reset_at = Utc::now() + chrono::Duration::seconds(retry_after_seconds as i64);

        rate_limits.insert(
            "global".to_string(),
            RateLimit {
                remaining: 0,
                reset_at,
                retry_after: Some(retry_after_seconds),
            },
        );
    }

    /// Get current connection for a user
    pub async fn get_user_connection(&self, user_id: Uuid) -> Result<Option<Connection>> {
        let connections = self.token_vault.get_user_connections(user_id).await;
        Ok(connections
            .into_iter()
            .find(|c| c.provider == StreamingProvider::Spotify))
    }

    /// Disconnect user's Spotify account
    pub async fn disconnect_user(&self, user_id: Uuid) -> Result<()> {
        if let Some(connection) = self.get_user_connection(user_id).await? {
            // Revoke the token with Spotify if possible
            let _ = self.revoke_token(&connection).await; // Don't fail if revocation fails

            // Remove from token vault
            self.token_vault.delete_connection(connection.id).await?;
        }
        Ok(())
    }

    /// Revoke token with Spotify
    async fn revoke_token(&self, connection: &Connection) -> Result<()> {
        let decrypted_token = self.token_vault.get_decrypted_token(connection.id).await?;

        // Spotify doesn't have a standard revocation endpoint, but we can try to revoke
        // This is a placeholder - in practice, you might just delete the token locally
        tracing::info!("Revoking Spotify token for connection {}", connection.id);

        Ok(())
    }

    // Batch enforcement operations

    /// Remove multiple liked songs in a single API call
    pub async fn remove_liked_songs_batch(
        &self,
        connection: &Connection,
        track_ids: &[String],
    ) -> Result<()> {
        if track_ids.is_empty() {
            return Ok(());
        }

        // Spotify allows up to 50 tracks per request
        if track_ids.len() > 50 {
            return Err(anyhow!("Cannot remove more than 50 tracks at once"));
        }

        let url = "https://api.spotify.com/v1/me/tracks";
        let body = serde_json::json!({ "ids": track_ids });

        let response = self
            .make_api_request(connection, "DELETE", url, Some(body))
            .await?;

        if response.status().is_success() {
            tracing::info!("Successfully removed {} liked songs", track_ids.len());
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow!(
                "Failed to remove liked songs: {} - {}",
                status,
                error_text
            ))
        }
    }

    /// Remove multiple tracks from a playlist with delta removal
    pub async fn remove_playlist_tracks_batch(
        &self,
        connection: &Connection,
        playlist_id: &str,
        tracks: &[serde_json::Value],
    ) -> Result<String> {
        if tracks.is_empty() {
            return Err(anyhow!("No tracks to remove"));
        }

        // Spotify allows up to 100 tracks per request for playlist modifications
        if tracks.len() > 100 {
            return Err(anyhow!("Cannot remove more than 100 tracks at once"));
        }

        let url = format!(
            "https://api.spotify.com/v1/playlists/{}/tracks",
            playlist_id
        );
        let body = serde_json::json!({ "tracks": tracks });

        let response = self
            .make_api_request(connection, "DELETE", &url, Some(body))
            .await?;

        if response.status().is_success() {
            let response_json: serde_json::Value = response.json().await?;
            let snapshot_id = response_json
                .get("snapshot_id")
                .and_then(|s| s.as_str())
                .unwrap_or("unknown")
                .to_string();

            tracing::info!(
                "Successfully removed {} tracks from playlist {}",
                tracks.len(),
                playlist_id
            );
            Ok(snapshot_id)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow!(
                "Failed to remove playlist tracks: {} - {}",
                status,
                error_text
            ))
        }
    }

    /// Unfollow multiple artists in a single API call
    pub async fn unfollow_artists_batch(
        &self,
        connection: &Connection,
        artist_ids: &[String],
    ) -> Result<()> {
        if artist_ids.is_empty() {
            return Ok(());
        }

        // Spotify allows up to 50 artists per request
        if artist_ids.len() > 50 {
            return Err(anyhow!("Cannot unfollow more than 50 artists at once"));
        }

        let url = format!(
            "https://api.spotify.com/v1/me/following?type=artist&ids={}",
            artist_ids.join(",")
        );

        let response = self
            .make_api_request(connection, "DELETE", &url, None)
            .await?;

        if response.status().is_success() {
            tracing::info!("Successfully unfollowed {} artists", artist_ids.len());
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow!(
                "Failed to unfollow artists: {} - {}",
                status,
                error_text
            ))
        }
    }

    /// Remove multiple saved albums in a single API call
    pub async fn remove_saved_albums_batch(
        &self,
        connection: &Connection,
        album_ids: &[String],
    ) -> Result<()> {
        if album_ids.is_empty() {
            return Ok(());
        }

        // Spotify allows up to 50 albums per request
        if album_ids.len() > 50 {
            return Err(anyhow!("Cannot remove more than 50 albums at once"));
        }

        let url = "https://api.spotify.com/v1/me/albums";
        let body = serde_json::json!({ "ids": album_ids });

        let response = self
            .make_api_request(connection, "DELETE", url, Some(body))
            .await?;

        if response.status().is_success() {
            tracing::info!("Successfully removed {} saved albums", album_ids.len());
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow!(
                "Failed to remove saved albums: {} - {}",
                status,
                error_text
            ))
        }
    }

    /// Add multiple liked songs (for rollback operations)
    pub async fn add_liked_songs_batch(
        &self,
        connection: &Connection,
        track_ids: &[String],
    ) -> Result<()> {
        if track_ids.is_empty() {
            return Ok(());
        }

        if track_ids.len() > 50 {
            return Err(anyhow!("Cannot add more than 50 tracks at once"));
        }

        let url = "https://api.spotify.com/v1/me/tracks";
        let body = serde_json::json!({ "ids": track_ids });

        let response = self
            .make_api_request(connection, "PUT", url, Some(body))
            .await?;

        if response.status().is_success() {
            tracing::info!("Successfully added {} liked songs", track_ids.len());
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow!(
                "Failed to add liked songs: {} - {}",
                status,
                error_text
            ))
        }
    }

    /// Follow multiple artists (for rollback operations)
    pub async fn follow_artists_batch(
        &self,
        connection: &Connection,
        artist_ids: &[String],
    ) -> Result<()> {
        if artist_ids.is_empty() {
            return Ok(());
        }

        if artist_ids.len() > 50 {
            return Err(anyhow!("Cannot follow more than 50 artists at once"));
        }

        let url = format!(
            "https://api.spotify.com/v1/me/following?type=artist&ids={}",
            artist_ids.join(",")
        );

        let response = self.make_api_request(connection, "PUT", &url, None).await?;

        if response.status().is_success() {
            tracing::info!("Successfully followed {} artists", artist_ids.len());
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow!(
                "Failed to follow artists: {} - {}",
                status,
                error_text
            ))
        }
    }

    /// Add multiple saved albums (for rollback operations)
    pub async fn add_saved_albums_batch(
        &self,
        connection: &Connection,
        album_ids: &[String],
    ) -> Result<()> {
        if album_ids.is_empty() {
            return Ok(());
        }

        if album_ids.len() > 50 {
            return Err(anyhow!("Cannot add more than 50 albums at once"));
        }

        let url = "https://api.spotify.com/v1/me/albums";
        let body = serde_json::json!({ "ids": album_ids });

        let response = self
            .make_api_request(connection, "PUT", url, Some(body))
            .await?;

        if response.status().is_success() {
            tracing::info!("Successfully added {} saved albums", album_ids.len());
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow!(
                "Failed to add saved albums: {} - {}",
                status,
                error_text
            ))
        }
    }

    /// Add tracks back to a playlist (for rollback operations)
    pub async fn add_playlist_tracks_batch(
        &self,
        connection: &Connection,
        playlist_id: &str,
        track_uris: &[String],
        position: Option<u32>,
    ) -> Result<String> {
        if track_uris.is_empty() {
            return Err(anyhow!("No tracks to add"));
        }

        if track_uris.len() > 100 {
            return Err(anyhow!("Cannot add more than 100 tracks at once"));
        }

        let url = format!(
            "https://api.spotify.com/v1/playlists/{}/tracks",
            playlist_id
        );
        let mut body = serde_json::json!({ "uris": track_uris });

        if let Some(pos) = position {
            body["position"] = serde_json::json!(pos);
        }

        let response = self
            .make_api_request(connection, "POST", &url, Some(body))
            .await?;

        if response.status().is_success() {
            let response_json: serde_json::Value = response.json().await?;
            let snapshot_id = response_json
                .get("snapshot_id")
                .and_then(|s| s.as_str())
                .unwrap_or("unknown")
                .to_string();

            tracing::info!(
                "Successfully added {} tracks to playlist {}",
                track_uris.len(),
                playlist_id
            );
            Ok(snapshot_id)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow!(
                "Failed to add playlist tracks: {} - {}",
                status,
                error_text
            ))
        }
    }

    // ===========================================
    // Library Scanning Methods
    // ===========================================

    /// Scan user's complete Spotify library
    ///
    /// This fetches all liked songs, saved albums, followed artists, and playlists
    /// using the Spotify API with proper pagination (50 items max per request) and
    /// rate limit handling (exponential backoff on 429 responses).
    pub async fn scan_library(&self, connection: &Connection) -> Result<SpotifyLibraryScanResult> {
        self.scan_library_with_progress::<fn(u32, u32, &str)>(connection, None)
            .await
    }

    /// Scan user's complete Spotify library with optional progress callback
    ///
    /// The progress callback receives (current_step, total_steps, step_name).
    pub async fn scan_library_with_progress<F>(
        &self,
        connection: &Connection,
        progress_callback: Option<F>,
    ) -> Result<SpotifyLibraryScanResult>
    where
        F: Fn(u32, u32, &str) + Send + Sync,
    {
        let started_at = Utc::now();
        let api_requests = Arc::new(AtomicU32::new(0));
        let rate_limit_retries = Arc::new(AtomicU32::new(0));
        let mut warnings: Vec<String> = Vec::new();

        // Step 1: Get user profile
        if let Some(ref callback) = progress_callback {
            callback(1, 5, "Fetching user profile");
        }
        let user_profile = self
            .get_user_profile_for_scan(connection, &api_requests, &rate_limit_retries)
            .await?;

        // Step 2-5: Scan all library components concurrently
        let api_requests_clone1 = api_requests.clone();
        let api_requests_clone2 = api_requests.clone();
        let api_requests_clone3 = api_requests.clone();
        let api_requests_clone4 = api_requests.clone();

        let rate_limit_retries_clone1 = rate_limit_retries.clone();
        let rate_limit_retries_clone2 = rate_limit_retries.clone();
        let rate_limit_retries_clone3 = rate_limit_retries.clone();
        let rate_limit_retries_clone4 = rate_limit_retries.clone();

        if let Some(ref callback) = progress_callback {
            callback(2, 5, "Scanning library components");
        }

        let (liked_songs_result, playlists_result, followed_artists_result, saved_albums_result) = tokio::join!(
            self.scan_liked_songs_internal(
                connection,
                &api_requests_clone1,
                &rate_limit_retries_clone1
            ),
            self.scan_playlists_internal(
                connection,
                &api_requests_clone2,
                &rate_limit_retries_clone2
            ),
            self.scan_followed_artists_internal(
                connection,
                &api_requests_clone3,
                &rate_limit_retries_clone3
            ),
            self.scan_saved_albums_internal(
                connection,
                &api_requests_clone4,
                &rate_limit_retries_clone4
            )
        );

        // Handle results, collecting warnings for non-fatal errors
        let liked_songs = match liked_songs_result {
            Ok(songs) => songs,
            Err(e) => {
                warnings.push(format!("Failed to scan liked songs: {}", e));
                Vec::new()
            }
        };

        if let Some(ref callback) = progress_callback {
            callback(3, 5, "Processing playlists");
        }

        let playlists = match playlists_result {
            Ok(lists) => lists,
            Err(e) => {
                warnings.push(format!("Failed to scan playlists: {}", e));
                Vec::new()
            }
        };

        if let Some(ref callback) = progress_callback {
            callback(4, 5, "Processing followed artists");
        }

        let followed_artists = match followed_artists_result {
            Ok(artists) => artists,
            Err(e) => {
                warnings.push(format!("Failed to scan followed artists: {}", e));
                Vec::new()
            }
        };

        if let Some(ref callback) = progress_callback {
            callback(5, 5, "Processing saved albums");
        }

        let saved_albums = match saved_albums_result {
            Ok(albums) => albums,
            Err(e) => {
                warnings.push(format!("Failed to scan saved albums: {}", e));
                Vec::new()
            }
        };

        let library = SpotifyLibrary {
            user_id: connection.user_id,
            spotify_user_id: user_profile.id,
            liked_songs,
            playlists,
            followed_artists,
            saved_albums,
            scanned_at: Utc::now(),
        };

        let result = SpotifyLibraryScanResult::new(
            library,
            started_at,
            api_requests.load(Ordering::Relaxed),
            rate_limit_retries.load(Ordering::Relaxed),
            warnings,
        );

        tracing::info!(
            "Library scan completed: {} liked songs, {} playlists, {} followed artists, {} saved albums in {}ms ({} API requests, {} rate limit retries)",
            result.counts.liked_songs_count,
            result.counts.playlists_count,
            result.counts.followed_artists_count,
            result.counts.saved_albums_count,
            result.metadata.duration_ms,
            result.metadata.api_requests_count,
            result.metadata.rate_limit_retries,
        );

        Ok(result)
    }

    /// Make an API request with exponential backoff on rate limiting
    async fn make_api_request_with_backoff(
        &self,
        connection: &Connection,
        method: &str,
        url: &str,
        body: Option<serde_json::Value>,
        api_requests: &AtomicU32,
        rate_limit_retries: &AtomicU32,
    ) -> Result<reqwest::Response> {
        const MAX_RETRIES: u32 = 5;
        const BASE_DELAY_MS: u64 = 1000;
        const MAX_DELAY_MS: u64 = 300000; // 5 minutes

        let mut attempt = 0;

        loop {
            api_requests.fetch_add(1, Ordering::Relaxed);

            match self
                .make_api_request(connection, method, url, body.clone())
                .await
            {
                Ok(response) => {
                    if response.status() == StatusCode::TOO_MANY_REQUESTS {
                        attempt += 1;
                        rate_limit_retries.fetch_add(1, Ordering::Relaxed);

                        if attempt > MAX_RETRIES {
                            return Err(anyhow!("Rate limited after {} retries", MAX_RETRIES));
                        }

                        // Get retry-after from header or calculate exponential backoff
                        let delay_ms = response
                            .headers()
                            .get("retry-after")
                            .and_then(|h| h.to_str().ok())
                            .and_then(|s| s.parse::<u64>().ok())
                            .map(|secs| secs * 1000)
                            .unwrap_or_else(|| {
                                // Exponential backoff with jitter
                                let base_delay = BASE_DELAY_MS * 2_u64.pow(attempt - 1);
                                let jitter = rand::random::<u64>() % (base_delay / 4 + 1);
                                std::cmp::min(base_delay + jitter, MAX_DELAY_MS)
                            });

                        tracing::warn!(
                            "Rate limited on {} (attempt {}), waiting {}ms before retry",
                            url,
                            attempt,
                            delay_ms
                        );

                        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                        continue;
                    }
                    return Ok(response);
                }
                Err(e) => {
                    // For other errors, check if it's a rate limit error message
                    if e.to_string().contains("Rate limited") {
                        attempt += 1;
                        rate_limit_retries.fetch_add(1, Ordering::Relaxed);

                        if attempt > MAX_RETRIES {
                            return Err(e);
                        }

                        let delay_ms = BASE_DELAY_MS * 2_u64.pow(attempt - 1);
                        let jitter = rand::random::<u64>() % (delay_ms / 4 + 1);
                        let total_delay = std::cmp::min(delay_ms + jitter, MAX_DELAY_MS);

                        tracing::warn!(
                            "Rate limit error on {} (attempt {}), waiting {}ms before retry",
                            url,
                            attempt,
                            total_delay
                        );

                        tokio::time::sleep(Duration::from_millis(total_delay)).await;
                        continue;
                    }
                    return Err(e);
                }
            }
        }
    }

    /// Get user profile for scanning
    async fn get_user_profile_for_scan(
        &self,
        connection: &Connection,
        api_requests: &AtomicU32,
        rate_limit_retries: &AtomicU32,
    ) -> Result<SpotifyUserProfile> {
        let response = self
            .make_api_request_with_backoff(
                connection,
                "GET",
                "https://api.spotify.com/v1/me",
                None,
                api_requests,
                rate_limit_retries,
            )
            .await?;

        if response.status().is_success() {
            let profile: SpotifyUserProfile = response.json().await?;
            Ok(profile)
        } else {
            Err(anyhow!("Failed to get user profile: {}", response.status()))
        }
    }

    /// Scan user's liked songs with pagination
    async fn scan_liked_songs_internal(
        &self,
        connection: &Connection,
        api_requests: &AtomicU32,
        rate_limit_retries: &AtomicU32,
    ) -> Result<Vec<SpotifySavedTrack>> {
        let mut liked_songs = Vec::new();
        let mut offset = 0;
        const LIMIT: usize = 50;

        loop {
            let url = format!(
                "https://api.spotify.com/v1/me/tracks?limit={}&offset={}",
                LIMIT, offset
            );

            let response = self
                .make_api_request_with_backoff(
                    connection,
                    "GET",
                    &url,
                    None,
                    api_requests,
                    rate_limit_retries,
                )
                .await?;

            if !response.status().is_success() {
                return Err(anyhow!(
                    "Failed to fetch liked songs: {}",
                    response.status()
                ));
            }

            let data: Value = response.json().await?;
            let empty_vec = vec![];
            let items = data["items"].as_array().unwrap_or(&empty_vec);

            if items.is_empty() {
                break;
            }

            for item in items {
                if let Ok(saved_track) = serde_json::from_value::<SpotifySavedTrack>(item.clone()) {
                    liked_songs.push(saved_track);
                }
            }

            offset += LIMIT;

            // Check if we've reached the end
            if items.len() < LIMIT {
                break;
            }
        }

        tracing::debug!("Scanned {} liked songs", liked_songs.len());
        Ok(liked_songs)
    }

    /// Scan user's playlists with pagination and fetch track details
    async fn scan_playlists_internal(
        &self,
        connection: &Connection,
        api_requests: &AtomicU32,
        rate_limit_retries: &AtomicU32,
    ) -> Result<Vec<SpotifyPlaylist>> {
        let mut playlists = Vec::new();
        let mut offset = 0;
        const LIMIT: usize = 50;

        loop {
            let url = format!(
                "https://api.spotify.com/v1/me/playlists?limit={}&offset={}",
                LIMIT, offset
            );

            let response = self
                .make_api_request_with_backoff(
                    connection,
                    "GET",
                    &url,
                    None,
                    api_requests,
                    rate_limit_retries,
                )
                .await?;

            if !response.status().is_success() {
                return Err(anyhow!("Failed to fetch playlists: {}", response.status()));
            }

            let data: Value = response.json().await?;
            let empty_vec = vec![];
            let items = data["items"].as_array().unwrap_or(&empty_vec);

            if items.is_empty() {
                break;
            }

            for item in items {
                if let Ok(mut playlist) = serde_json::from_value::<SpotifyPlaylist>(item.clone()) {
                    // Fetch full playlist details including tracks
                    match self
                        .fetch_playlist_details_internal(
                            connection,
                            &playlist.id,
                            api_requests,
                            rate_limit_retries,
                        )
                        .await
                    {
                        Ok(full_playlist) => playlists.push(full_playlist),
                        Err(e) => {
                            tracing::warn!(
                                "Failed to fetch details for playlist {}: {}",
                                playlist.id,
                                e
                            );
                            playlists.push(playlist); // Use partial data
                        }
                    }
                }
            }

            offset += LIMIT;

            if items.len() < LIMIT {
                break;
            }
        }

        tracing::debug!("Scanned {} playlists", playlists.len());
        Ok(playlists)
    }

    /// Fetch detailed playlist information including tracks
    async fn fetch_playlist_details_internal(
        &self,
        connection: &Connection,
        playlist_id: &str,
        api_requests: &AtomicU32,
        rate_limit_retries: &AtomicU32,
    ) -> Result<SpotifyPlaylist> {
        let url = format!(
            "https://api.spotify.com/v1/playlists/{}?fields=id,name,description,owner,public,collaborative,tracks.total,tracks.items(added_at,added_by,is_local,track(id,name,artists,album,duration_ms,explicit,popularity,preview_url,external_urls,is_local,is_playable)),external_urls,images,snapshot_id",
            playlist_id
        );

        let response = self
            .make_api_request_with_backoff(
                connection,
                "GET",
                &url,
                None,
                api_requests,
                rate_limit_retries,
            )
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch playlist details: {}",
                response.status()
            ));
        }

        let playlist: SpotifyPlaylist = response.json().await?;
        Ok(playlist)
    }

    /// Scan user's followed artists with cursor-based pagination
    async fn scan_followed_artists_internal(
        &self,
        connection: &Connection,
        api_requests: &AtomicU32,
        rate_limit_retries: &AtomicU32,
    ) -> Result<Vec<SpotifyFollowedArtist>> {
        let mut followed_artists = Vec::new();
        let mut after: Option<String> = None;
        const LIMIT: usize = 50;

        loop {
            let mut url = format!(
                "https://api.spotify.com/v1/me/following?type=artist&limit={}",
                LIMIT
            );

            if let Some(ref after_cursor) = after {
                url.push_str(&format!("&after={}", after_cursor));
            }

            let response = self
                .make_api_request_with_backoff(
                    connection,
                    "GET",
                    &url,
                    None,
                    api_requests,
                    rate_limit_retries,
                )
                .await?;

            if !response.status().is_success() {
                return Err(anyhow!(
                    "Failed to fetch followed artists: {}",
                    response.status()
                ));
            }

            let data: Value = response.json().await?;
            let empty_vec = vec![];
            let artists = data["artists"]["items"].as_array().unwrap_or(&empty_vec);

            if artists.is_empty() {
                break;
            }

            for artist_data in artists {
                if let Ok(artist) = serde_json::from_value::<SpotifyArtist>(artist_data.clone()) {
                    followed_artists.push(SpotifyFollowedArtist {
                        artist,
                        followed_at: None, // Spotify doesn't provide follow date
                    });
                }
            }

            // Get cursor for next page (Spotify uses cursor-based pagination for followers)
            if let Some(cursors) = data["artists"]["cursors"].as_object() {
                after = cursors["after"].as_str().map(|s| s.to_string());
                if after.is_none() {
                    break;
                }
            } else {
                break;
            }

            if artists.len() < LIMIT {
                break;
            }
        }

        tracing::debug!("Scanned {} followed artists", followed_artists.len());
        Ok(followed_artists)
    }

    /// Scan user's saved albums with pagination
    async fn scan_saved_albums_internal(
        &self,
        connection: &Connection,
        api_requests: &AtomicU32,
        rate_limit_retries: &AtomicU32,
    ) -> Result<Vec<SpotifyAlbum>> {
        let mut saved_albums = Vec::new();
        let mut offset = 0;
        const LIMIT: usize = 50;

        loop {
            let url = format!(
                "https://api.spotify.com/v1/me/albums?limit={}&offset={}",
                LIMIT, offset
            );

            let response = self
                .make_api_request_with_backoff(
                    connection,
                    "GET",
                    &url,
                    None,
                    api_requests,
                    rate_limit_retries,
                )
                .await?;

            if !response.status().is_success() {
                return Err(anyhow!(
                    "Failed to fetch saved albums: {}",
                    response.status()
                ));
            }

            let data: Value = response.json().await?;
            let empty_vec = vec![];
            let items = data["items"].as_array().unwrap_or(&empty_vec);

            if items.is_empty() {
                break;
            }

            for item in items {
                if let Some(album_data) = item["album"].as_object() {
                    if let Ok(album) =
                        serde_json::from_value::<SpotifyAlbum>(Value::Object(album_data.clone()))
                    {
                        saved_albums.push(album);
                    }
                }
            }

            offset += LIMIT;

            if items.len() < LIMIT {
                break;
            }
        }

        tracing::debug!("Scanned {} saved albums", saved_albums.len());
        Ok(saved_albums)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_spotify_config_default() {
        let config = SpotifyConfig::default();
        assert!(!config.client_id.is_empty());
        assert!(!config.redirect_uri.is_empty());
        assert_eq!(config.auth_url, "https://accounts.spotify.com/authorize");
        assert_eq!(config.token_url, "https://accounts.spotify.com/api/token");
    }

    #[tokio::test]
    async fn test_get_auth_url() {
        let config = SpotifyConfig::default();
        let token_vault = Arc::new(TokenVaultService::new());
        let service = SpotifyService::new(config, token_vault).unwrap();

        let auth_url_response = service.get_auth_url().await.unwrap();

        assert!(auth_url_response.auth_url.contains("accounts.spotify.com"));
        assert!(auth_url_response.auth_url.contains("client_id"));
        assert!(auth_url_response.auth_url.contains("code_challenge"));
        assert!(!auth_url_response.state.is_empty());
    }
}
