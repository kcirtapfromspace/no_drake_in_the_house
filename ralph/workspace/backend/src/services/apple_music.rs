use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;
use jsonwebtoken::{encode, Header, EncodingKey, Algorithm};

use crate::models::{
    Connection, ConnectionStatus, StreamingProvider, TokenHealthCheck, DecryptedToken,
    AppleMusicLibrary, AppleMusicCapabilities, AppleMusicEnforcementOptions,
    AppleMusicEnforcementResult, AppleMusicResponse, AppleMusicTrack, AppleMusicAlbum,
    AppleMusicPlaylist, AppleMusicLibraryTrack, AppleMusicLibraryAlbum, AppleMusicLibraryPlaylist,
    AppleMusicSearchRequest, AppleMusicSearchResponse, AppleMusicTokenInfo,
    AppleMusicDeveloperToken, AppleMusicUserTokenResponse, AppleMusicErrorResponse,
    BatchRatingResult, RatingError,
};
use crate::services::TokenVaultService;

/// Rating value constants for Apple Music API
pub const RATING_LIKE: i8 = 1;
pub const RATING_DISLIKE: i8 = -1;

/// Apple Music API configuration
#[derive(Debug, Clone)]
pub struct AppleMusicConfig {
    pub team_id: String,
    pub key_id: String,
    pub private_key: String,
    pub bundle_id: String,
    pub api_base_url: String,
}

impl Default for AppleMusicConfig {
    fn default() -> Self {
        // Try to read private key from file path first, then fall back to direct env var
        let private_key = if let Ok(key_path) = std::env::var("APPLE_MUSIC_KEY_PATH") {
            std::fs::read_to_string(&key_path)
                .unwrap_or_else(|e| {
                    tracing::warn!("Failed to read Apple Music private key from {}: {}", key_path, e);
                    "-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----".to_string()
                })
        } else {
            std::env::var("APPLE_MUSIC_PRIVATE_KEY")
                .unwrap_or_else(|_| "-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----".to_string())
        };

        Self {
            team_id: std::env::var("APPLE_MUSIC_TEAM_ID")
                .unwrap_or_else(|_| "YOUR_TEAM_ID".to_string()),
            key_id: std::env::var("APPLE_MUSIC_KEY_ID")
                .unwrap_or_else(|_| "YOUR_KEY_ID".to_string()),
            private_key,
            bundle_id: std::env::var("APPLE_MUSIC_BUNDLE_ID")
                .unwrap_or_else(|_| "com.nodrakeinthehouse".to_string()),
            api_base_url: "https://api.music.apple.com".to_string(),
        }
    }
}

/// JWT claims for Apple Music developer token
#[derive(Debug, Serialize, Deserialize)]
struct AppleMusicJWTClaims {
    iss: String, // Team ID
    iat: i64,    // Issued at
    exp: i64,    // Expires at
}

/// Rate limiting state for Apple Music API
#[derive(Debug, Clone)]
pub struct AppleMusicRateLimit {
    pub remaining: u32,
    pub reset_at: DateTime<Utc>,
    pub retry_after: Option<u32>,
}

/// Apple Music API client with MusicKit integration
pub struct AppleMusicService {
    config: AppleMusicConfig,
    http_client: Client,
    token_vault: Arc<TokenVaultService>,
    rate_limits: Arc<RwLock<HashMap<String, AppleMusicRateLimit>>>,
    developer_token_cache: Arc<RwLock<Option<AppleMusicDeveloperToken>>>,
}

impl AppleMusicService {
    pub fn new(config: AppleMusicConfig, token_vault: Arc<TokenVaultService>) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            config,
            http_client,
            token_vault,
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            developer_token_cache: Arc::new(RwLock::new(None)),
        })
    }

    /// Generate Apple Music developer token (JWT)
    pub async fn generate_developer_token(&self) -> Result<AppleMusicDeveloperToken> {
        // Check if we have a valid cached token
        {
            let cache = self.developer_token_cache.read().await;
            if let Some(cached_token) = cache.as_ref() {
                if cached_token.expires_at > Utc::now() + chrono::Duration::minutes(5) {
                    return Ok(cached_token.clone());
                }
            }
        }

        let now = Utc::now();
        let expires_at = now + chrono::Duration::hours(1); // Apple Music tokens expire after 6 months, but we refresh hourly

        let claims = AppleMusicJWTClaims {
            iss: self.config.team_id.clone(),
            iat: now.timestamp(),
            exp: expires_at.timestamp(),
        };

        let header = Header {
            alg: Algorithm::ES256,
            kid: Some(self.config.key_id.clone()),
            ..Default::default()
        };

        let encoding_key = EncodingKey::from_ec_pem(self.config.private_key.as_bytes())
            .map_err(|e| {
                tracing::error!("Failed to parse private key: {}. Key starts with: {:?}", e, &self.config.private_key.chars().take(50).collect::<String>());
                anyhow!("Failed to parse private key: {}", e)
            })?;

        let token = encode(&header, &claims, &encoding_key)
            .map_err(|e| {
                tracing::error!("Failed to generate JWT: {}", e);
                anyhow!("Failed to generate JWT: {}", e)
            })?;

        let developer_token = AppleMusicDeveloperToken {
            token,
            expires_at,
        };

        // Cache the token
        {
            let mut cache = self.developer_token_cache.write().await;
            *cache = Some(developer_token.clone());
        }

        Ok(developer_token)
    }

    /// Get Apple Music capabilities (what operations are supported)
    pub fn get_capabilities(&self) -> AppleMusicCapabilities {
        AppleMusicCapabilities::default()
    }

    /// Create a connection for Apple Music (requires user token from frontend)
    pub async fn create_connection(
        &self,
        user_id: Uuid,
        user_token: String,
        music_user_token: Option<String>,
    ) -> Result<Connection> {
        // Validate the user token by making a test API call
        let developer_token = self.generate_developer_token().await?;
        
        let test_response = self
            .make_api_request_with_tokens(
                &developer_token.token,
                &user_token,
                "GET",
                "/v1/me/library/songs?limit=1",
                None,
            )
            .await;

        if test_response.is_err() {
            return Err(anyhow!("Invalid Apple Music user token"));
        }

        // Store the connection using the token vault
        let store_request = crate::models::StoreTokenRequest {
            user_id,
            provider: StreamingProvider::AppleMusic,
            provider_user_id: format!("apple_music_{}", user_id), // Apple Music doesn't provide user IDs
            access_token: user_token,
            refresh_token: music_user_token,
            scopes: vec!["library-read".to_string(), "library-modify".to_string()],
            expires_at: None, // Apple Music user tokens don't expire
        };

        self.token_vault
            .store_token(store_request)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    /// Check token health and validity
    pub async fn check_token_health(&self, connection: &Connection) -> Result<TokenHealthCheck> {
        let decrypted_token = self
            .token_vault
            .get_decrypted_token(connection.id)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let developer_token = self.generate_developer_token().await?;

        let response = self
            .make_api_request_with_tokens(
                &developer_token.token,
                &decrypted_token.access_token,
                "GET",
                "/v1/me/library/songs?limit=1",
                None,
            )
            .await;

        let is_valid = response.is_ok();

        Ok(TokenHealthCheck {
            connection_id: connection.id,
            is_valid,
            expires_at: connection.expires_at,
            error_message: if !is_valid {
                Some(format!("Token validation failed: {:?}", response.err()))
            } else {
                None
            },
            checked_at: Utc::now(),
            needs_refresh: false, // Apple Music user tokens don't expire
        })
    }

    /// Make authenticated API request to Apple Music
    async fn make_api_request_with_tokens(
        &self,
        developer_token: &str,
        user_token: &str,
        method: &str,
        endpoint: &str,
        body: Option<serde_json::Value>,
    ) -> Result<reqwest::Response> {
        // Check and wait for rate limits
        self.wait_for_rate_limit().await?;

        let url = format!("{}{}", self.config.api_base_url, endpoint);

        // Build request
        let mut request_builder = match method.to_uppercase().as_str() {
            "GET" => self.http_client.get(&url),
            "POST" => self.http_client.post(&url),
            "PUT" => self.http_client.put(&url),
            "DELETE" => self.http_client.delete(&url),
            _ => return Err(anyhow!("Unsupported HTTP method: {}", method)),
        };

        request_builder = request_builder
            .header("Authorization", format!("Bearer {}", developer_token))
            .header("Music-User-Token", user_token);

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

    /// Make authenticated API request using a connection
    pub async fn make_api_request(
        &self,
        connection: &Connection,
        method: &str,
        endpoint: &str,
        body: Option<serde_json::Value>,
    ) -> Result<reqwest::Response> {
        let decrypted_token = self
            .token_vault
            .get_decrypted_token(connection.id)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let developer_token = self.generate_developer_token().await?;

        self.make_api_request_with_tokens(
            &developer_token.token,
            &decrypted_token.access_token,
            method,
            endpoint,
            body,
        )
        .await
    }

    /// Scan user's Apple Music library
    pub async fn scan_library(&self, connection: &Connection) -> Result<AppleMusicLibrary> {
        let mut library = AppleMusicLibrary::new(
            connection.user_id,
            connection.provider_user_id.clone(),
        );

        // Scan library tracks
        library.library_tracks = self.get_library_tracks(connection).await?;
        
        // Scan library albums
        library.library_albums = self.get_library_albums(connection).await?;
        
        // Scan library playlists
        library.library_playlists = self.get_library_playlists(connection).await?;

        library.scanned_at = Utc::now();

        Ok(library)
    }

    /// Get user's library tracks
    pub async fn get_library_tracks(&self, connection: &Connection) -> Result<Vec<AppleMusicLibraryTrack>> {
        let mut tracks = Vec::new();
        let mut next_url: Option<String> = Some("/v1/me/library/songs?limit=100".to_string());

        while let Some(url) = next_url {
            let response = self.make_api_request(connection, "GET", &url, None).await?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                return Err(anyhow!("Failed to get library tracks: {} - {}", status, error_text));
            }

            let response_data: AppleMusicResponse<AppleMusicLibraryTrack> = response.json().await?;
            tracks.extend(response_data.data);

            next_url = response_data.next;
        }

        tracing::info!("Retrieved {} library tracks", tracks.len());
        Ok(tracks)
    }

    /// Get user's library albums
    pub async fn get_library_albums(&self, connection: &Connection) -> Result<Vec<AppleMusicLibraryAlbum>> {
        let mut albums = Vec::new();
        let mut next_url: Option<String> = Some("/v1/me/library/albums?limit=100".to_string());

        while let Some(url) = next_url {
            let response = self.make_api_request(connection, "GET", &url, None).await?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                return Err(anyhow!("Failed to get library albums: {} - {}", status, error_text));
            }

            let response_data: AppleMusicResponse<AppleMusicLibraryAlbum> = response.json().await?;
            albums.extend(response_data.data);

            next_url = response_data.next;
        }

        tracing::info!("Retrieved {} library albums", albums.len());
        Ok(albums)
    }

    /// Get user's library playlists
    pub async fn get_library_playlists(&self, connection: &Connection) -> Result<Vec<AppleMusicLibraryPlaylist>> {
        let mut playlists = Vec::new();
        let mut next_url: Option<String> = Some("/v1/me/library/playlists?limit=100".to_string());

        while let Some(url) = next_url {
            let response = self.make_api_request(connection, "GET", &url, None).await?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                return Err(anyhow!("Failed to get library playlists: {} - {}", status, error_text));
            }

            let response_data: AppleMusicResponse<AppleMusicLibraryPlaylist> = response.json().await?;
            playlists.extend(response_data.data);

            next_url = response_data.next;
        }

        tracing::info!("Retrieved {} library playlists", playlists.len());
        Ok(playlists)
    }

    /// Search Apple Music catalog
    pub async fn search(
        &self,
        connection: &Connection,
        search_request: AppleMusicSearchRequest,
    ) -> Result<AppleMusicSearchResponse> {
        let mut query_params = vec![
            ("term", search_request.term),
            ("types", search_request.types.join(",")),
        ];

        if let Some(limit) = search_request.limit {
            query_params.push(("limit", limit.to_string()));
        }

        if let Some(offset) = search_request.offset {
            query_params.push(("offset", offset.to_string()));
        }

        let query_string = query_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let endpoint = format!("/v1/catalog/us/search?{}", query_string);

        let response = self.make_api_request(connection, "GET", &endpoint, None).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Search failed: {} - {}", status, error_text));
        }

        let search_response: AppleMusicSearchResponse = response.json().await?;
        Ok(search_response)
    }

    /// Analyze library for blocked content (read-only enforcement)
    pub async fn analyze_library_for_blocked_content(
        &self,
        connection: &Connection,
        dnp_artist_ids: &[String],
        options: AppleMusicEnforcementOptions,
    ) -> Result<AppleMusicEnforcementResult> {
        let mut result = AppleMusicEnforcementResult::new(connection.user_id);

        // Add limitation note about Apple Music's limited API
        result.add_limitation("Apple Music API has limited write capabilities - only scanning and reporting available".to_string());

        if options.scan_library {
            let library = self.scan_library(connection).await?;

            result.library_tracks_scanned = library.library_tracks.len() as u32;
            result.library_albums_scanned = library.library_albums.len() as u32;
            result.playlists_scanned = library.library_playlists.len() as u32;

            // Analyze tracks for blocked artists
            for track in &library.library_tracks {
                if self.is_track_blocked(&track.attributes.artist_name, dnp_artist_ids) {
                    result.blocked_tracks_found += 1;
                }
            }

            // Analyze albums for blocked artists
            for album in &library.library_albums {
                if self.is_track_blocked(&album.attributes.artist_name, dnp_artist_ids) {
                    result.blocked_albums_found += 1;
                }
            }

            // Note: Playlist analysis would require additional API calls to get track details
            if options.scan_playlists {
                result.add_limitation("Playlist track analysis requires additional API calls - not implemented in this version".to_string());
            }
        }

        if options.export_blocked_content && result.total_blocked_items() > 0 {
            let export_path = self.export_blocked_content(&result, connection).await?;
            result.export_file_path = Some(export_path);
        }

        if options.generate_report {
            result.report_generated = true;
        }

        result.scan_completed_at = Utc::now();
        Ok(result)
    }

    /// Check if a track is blocked based on artist name matching
    fn is_track_blocked(&self, artist_name: &str, dnp_artist_ids: &[String]) -> bool {
        // This is a simplified implementation - in practice, you'd use the entity resolution service
        // to match artist names to canonical IDs
        dnp_artist_ids.iter().any(|blocked_id| {
            artist_name.to_lowercase().contains(&blocked_id.to_lowercase())
        })
    }

    /// Export blocked content to a file for manual action
    async fn export_blocked_content(
        &self,
        result: &AppleMusicEnforcementResult,
        _connection: &Connection,
    ) -> Result<String> {
        // In a real implementation, this would generate a CSV or JSON file
        // with details of blocked content for manual removal
        let export_path = format!("/tmp/apple_music_blocked_content_{}.json", result.user_id);
        
        // This is a placeholder - you'd implement actual file generation here
        tracing::info!("Would export blocked content to: {}", export_path);
        
        Ok(export_path)
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
        
        // Apple Music API rate limiting is not well documented
        // This is a conservative approach
        let remaining = response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(100); // Conservative default

        let reset_at = Utc::now() + chrono::Duration::seconds(60); // Default 1-minute window

        rate_limits.insert(
            "global".to_string(),
            AppleMusicRateLimit {
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
            AppleMusicRateLimit {
                remaining: 0,
                reset_at,
                retry_after: Some(retry_after_seconds),
            },
        );
    }

    /// Get current connection for a user
    pub async fn get_user_connection(&self, user_id: Uuid) -> Result<Option<Connection>> {
        let connections = self.token_vault
            .get_user_connections(user_id)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(connections
            .into_iter()
            .find(|c| c.provider == StreamingProvider::AppleMusic))
    }

    /// Disconnect user's Apple Music account
    pub async fn disconnect_user(&self, user_id: Uuid) -> Result<()> {
        if let Some(connection) = self.get_user_connection(user_id).await? {
            // Apple Music doesn't have a token revocation endpoint
            // Just remove from token vault
            self.token_vault
                .delete_connection(connection.id)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
        }
        Ok(())
    }

    /// Get storefront for user (required for some API calls)
    pub async fn get_user_storefront(&self, connection: &Connection) -> Result<String> {
        let response = self.make_api_request(connection, "GET", "/v1/me/storefront", None).await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to get user storefront: {}", response.status()));
        }

        let response_data: serde_json::Value = response.json().await?;
        let storefront = response_data
            .get("data")
            .and_then(|data| data.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("id"))
            .and_then(|id| id.as_str())
            .unwrap_or("us") // Default to US storefront
            .to_string();

        Ok(storefront)
    }

    // ============================================
    // Rating Methods for Enforcement
    // ============================================

    /// Rate a catalog song (like: 1, dislike: -1)
    pub async fn rate_song(
        &self,
        connection: &Connection,
        song_id: &str,
        value: i8,
    ) -> Result<()> {
        let endpoint = format!("/v1/me/ratings/songs/{}", song_id);
        let body = serde_json::json!({
            "type": "rating",
            "attributes": {
                "value": value
            }
        });

        let response = self.make_api_request(connection, "PUT", &endpoint, Some(body)).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to rate song {}: {} - {}", song_id, status, error_text));
        }

        tracing::debug!("Rated song {} with value {}", song_id, value);
        Ok(())
    }

    /// Rate a library song (like: 1, dislike: -1)
    pub async fn rate_library_song(
        &self,
        connection: &Connection,
        library_song_id: &str,
        value: i8,
    ) -> Result<()> {
        let endpoint = format!("/v1/me/ratings/library-songs/{}", library_song_id);
        let body = serde_json::json!({
            "type": "rating",
            "attributes": {
                "value": value
            }
        });

        let response = self.make_api_request(connection, "PUT", &endpoint, Some(body)).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to rate library song {}: {} - {}", library_song_id, status, error_text));
        }

        tracing::debug!("Rated library song {} with value {}", library_song_id, value);
        Ok(())
    }

    /// Rate a catalog album (like: 1, dislike: -1)
    pub async fn rate_album(
        &self,
        connection: &Connection,
        album_id: &str,
        value: i8,
    ) -> Result<()> {
        let endpoint = format!("/v1/me/ratings/albums/{}", album_id);
        let body = serde_json::json!({
            "type": "rating",
            "attributes": {
                "value": value
            }
        });

        let response = self.make_api_request(connection, "PUT", &endpoint, Some(body)).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to rate album {}: {} - {}", album_id, status, error_text));
        }

        tracing::debug!("Rated album {} with value {}", album_id, value);
        Ok(())
    }

    /// Rate a library album (like: 1, dislike: -1)
    pub async fn rate_library_album(
        &self,
        connection: &Connection,
        library_album_id: &str,
        value: i8,
    ) -> Result<()> {
        let endpoint = format!("/v1/me/ratings/library-albums/{}", library_album_id);
        let body = serde_json::json!({
            "type": "rating",
            "attributes": {
                "value": value
            }
        });

        let response = self.make_api_request(connection, "PUT", &endpoint, Some(body)).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to rate library album {}: {} - {}", library_album_id, status, error_text));
        }

        tracing::debug!("Rated library album {} with value {}", library_album_id, value);
        Ok(())
    }

    /// Get current rating for a song
    pub async fn get_song_rating(
        &self,
        connection: &Connection,
        song_id: &str,
    ) -> Result<Option<i8>> {
        let endpoint = format!("/v1/me/ratings/songs/{}", song_id);
        let response = self.make_api_request(connection, "GET", &endpoint, None).await?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to get song rating {}: {} - {}", song_id, status, error_text));
        }

        let response_data: serde_json::Value = response.json().await?;
        let value = response_data
            .get("data")
            .and_then(|data| data.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("attributes"))
            .and_then(|attrs| attrs.get("value"))
            .and_then(|v| v.as_i64())
            .map(|v| v as i8);

        Ok(value)
    }

    /// Delete a song rating (removes like/dislike)
    pub async fn delete_song_rating(
        &self,
        connection: &Connection,
        song_id: &str,
    ) -> Result<()> {
        let endpoint = format!("/v1/me/ratings/songs/{}", song_id);
        let response = self.make_api_request(connection, "DELETE", &endpoint, None).await?;

        // 204 No Content is success, 404 means no rating existed
        if response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::NO_CONTENT {
            return Ok(());
        }

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to delete song rating {}: {} - {}", song_id, status, error_text));
        }

        tracing::debug!("Deleted rating for song {}", song_id);
        Ok(())
    }

    /// Delete a library song rating
    pub async fn delete_library_song_rating(
        &self,
        connection: &Connection,
        library_song_id: &str,
    ) -> Result<()> {
        let endpoint = format!("/v1/me/ratings/library-songs/{}", library_song_id);
        let response = self.make_api_request(connection, "DELETE", &endpoint, None).await?;

        if response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::NO_CONTENT {
            return Ok(());
        }

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to delete library song rating {}: {} - {}", library_song_id, status, error_text));
        }

        tracing::debug!("Deleted rating for library song {}", library_song_id);
        Ok(())
    }

    /// Delete an album rating
    pub async fn delete_album_rating(
        &self,
        connection: &Connection,
        album_id: &str,
    ) -> Result<()> {
        let endpoint = format!("/v1/me/ratings/albums/{}", album_id);
        let response = self.make_api_request(connection, "DELETE", &endpoint, None).await?;

        if response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::NO_CONTENT {
            return Ok(());
        }

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to delete album rating {}: {} - {}", album_id, status, error_text));
        }

        tracing::debug!("Deleted rating for album {}", album_id);
        Ok(())
    }

    /// Delete a library album rating
    pub async fn delete_library_album_rating(
        &self,
        connection: &Connection,
        library_album_id: &str,
    ) -> Result<()> {
        let endpoint = format!("/v1/me/ratings/library-albums/{}", library_album_id);
        let response = self.make_api_request(connection, "DELETE", &endpoint, None).await?;

        if response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::NO_CONTENT {
            return Ok(());
        }

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to delete library album rating {}: {} - {}", library_album_id, status, error_text));
        }

        tracing::debug!("Deleted rating for library album {}", library_album_id);
        Ok(())
    }

    /// Batch dislike multiple library songs (with rate limiting)
    pub async fn batch_dislike_library_songs<F>(
        &self,
        connection: &Connection,
        song_ids: Vec<String>,
        progress_callback: F,
    ) -> Result<BatchRatingResult>
    where
        F: Fn(usize, usize),
    {
        let total = song_ids.len();
        let mut successful = 0;
        let mut failed = 0;
        let mut errors = Vec::new();

        for (idx, song_id) in song_ids.iter().enumerate() {
            match self.rate_library_song(connection, song_id, RATING_DISLIKE).await {
                Ok(()) => {
                    successful += 1;
                }
                Err(e) => {
                    failed += 1;
                    errors.push(RatingError {
                        resource_id: song_id.clone(),
                        resource_type: "library-song".to_string(),
                        error_message: e.to_string(),
                    });
                }
            }
            progress_callback(idx + 1, total);

            // Small delay to respect rate limits (~20 req/sec)
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        Ok(BatchRatingResult {
            total,
            successful,
            failed,
            errors,
        })
    }

    /// Batch dislike multiple library albums (with rate limiting)
    pub async fn batch_dislike_library_albums<F>(
        &self,
        connection: &Connection,
        album_ids: Vec<String>,
        progress_callback: F,
    ) -> Result<BatchRatingResult>
    where
        F: Fn(usize, usize),
    {
        let total = album_ids.len();
        let mut successful = 0;
        let mut failed = 0;
        let mut errors = Vec::new();

        for (idx, album_id) in album_ids.iter().enumerate() {
            match self.rate_library_album(connection, album_id, RATING_DISLIKE).await {
                Ok(()) => {
                    successful += 1;
                }
                Err(e) => {
                    failed += 1;
                    errors.push(RatingError {
                        resource_id: album_id.clone(),
                        resource_type: "library-album".to_string(),
                        error_message: e.to_string(),
                    });
                }
            }
            progress_callback(idx + 1, total);

            // Small delay to respect rate limits
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        Ok(BatchRatingResult {
            total,
            successful,
            failed,
            errors,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_apple_music_config_default() {
        let config = AppleMusicConfig::default();
        assert!(!config.team_id.is_empty());
        assert!(!config.key_id.is_empty());
        assert!(!config.bundle_id.is_empty());
        assert_eq!(config.api_base_url, "https://api.music.apple.com");
    }

    #[tokio::test]
    async fn test_get_capabilities() {
        let config = AppleMusicConfig::default();
        let token_vault = Arc::new(TokenVaultService::new());
        let service = AppleMusicService::new(config, token_vault).unwrap();

        let capabilities = service.get_capabilities();
        
        assert!(capabilities.library_read);
        assert!(!capabilities.library_modify); // Limited by Apple Music API
        assert!(capabilities.playlist_read);
        assert!(!capabilities.playlist_modify); // Limited by Apple Music API
    }

    #[test]
    fn test_apple_music_enforcement_result() {
        let user_id = Uuid::new_v4();
        let mut result = AppleMusicEnforcementResult::new(user_id);
        
        assert_eq!(result.user_id, user_id);
        assert_eq!(result.total_blocked_items(), 0);
        
        result.blocked_tracks_found = 5;
        result.blocked_albums_found = 2;
        assert_eq!(result.total_blocked_items(), 7);
        
        result.add_limitation("Test limitation".to_string());
        assert_eq!(result.limitations_encountered.len(), 1);
    }
}