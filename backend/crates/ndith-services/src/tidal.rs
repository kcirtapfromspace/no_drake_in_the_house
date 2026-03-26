//! Tidal API service implementation
//!
//! Provides functionality for interacting with the Tidal API including:
//! - User profile retrieval
//! - Library scanning (favorites, playlists)
//! - Library modification (remove favorites, modify playlists)

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use reqwest::{Client, Method, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{debug, info, warn};
use uuid::Uuid;

use ndith_core::config::provider_callback_uri;
use ndith_core::models::tidal::{
    TidalAlbum, TidalArtist, TidalFavoriteAlbum, TidalFavoriteArtist, TidalFavoriteTrack,
    TidalLibrary, TidalLibraryScanResult, TidalPaginatedResponse, TidalPlaylist,
    TidalPlaylistTrack, TidalTrack, TidalUser,
};

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

/// Tidal OpenAPI (Web API) base URL.
///
/// This is the official developer platform base and matches the OpenAPI reference.
const TIDAL_API_BASE: &str = "https://openapi.tidal.com/v2";

/// Tidal OAuth authorization endpoint (legacy apps)
const TIDAL_AUTHORIZATION_ENDPOINT_LEGACY: &str = "https://auth.tidal.com/v1/oauth2/authorize";

/// Tidal OAuth authorization endpoint (modern OAuth 2.1 apps)
const TIDAL_AUTHORIZATION_ENDPOINT_MODERN: &str = "https://login.tidal.com/authorize";

/// Tidal OAuth token endpoint
const TIDAL_TOKEN_ENDPOINT: &str = "https://auth.tidal.com/v1/oauth2/token";

/// Rate limit: max requests per minute
const RATE_LIMIT_REQUESTS_PER_MINUTE: u32 = 100;

/// Default locale for TIDAL Web API requests.
const TIDAL_DEFAULT_LOCALE: &str = "en-US";

/// Conservative batch size for `filter[id]` calls to avoid URL length limits.
const TIDAL_BULK_FETCH_IDS_BATCH_SIZE: usize = 50;

// Legacy pagination constant (kept for compatibility with older helper methods).
const DEFAULT_PAGE_SIZE: u32 = 100;

fn parse_rfc3339_datetime(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn parse_iso8601_duration_seconds(value: &str) -> Option<u32> {
    // Supports `PT2M58S`, `PT58S`, `PT1H2M3S` and the (non-standard but seen-in-the-wild) `P30M5S`.
    let mut s = value.trim();
    if !s.starts_with('P') {
        return None;
    }
    s = &s[1..];

    // Drop date component and optional `T`.
    if let Some(t_idx) = s.find('T') {
        s = &s[t_idx + 1..];
    }

    let mut hours: f64 = 0.0;
    let mut minutes: f64 = 0.0;
    let mut seconds: f64 = 0.0;
    let mut number_buf = String::new();

    for ch in s.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            number_buf.push(ch);
            continue;
        }

        let value = number_buf.parse::<f64>().ok()?;
        number_buf.clear();

        match ch {
            'H' => hours = value,
            'M' => minutes = value,
            'S' => seconds = value,
            _ => {}
        }
    }

    let total = hours * 3600.0 + minutes * 60.0 + seconds;
    if total.is_finite() && total >= 0.0 {
        Some(total.round() as u32)
    } else {
        None
    }
}

/// Tidal service configuration
#[derive(Debug, Clone)]
pub struct TidalConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub client_unique_key: Option<String>,
}

impl TidalConfig {
    /// Create a new TidalConfig from environment variables
    pub fn from_env() -> Result<Self> {
        let client_id = std::env::var("TIDAL_CLIENT_ID")
            .map_err(|_| anyhow!("TIDAL_CLIENT_ID environment variable is required"))?;
        let client_secret = std::env::var("TIDAL_CLIENT_SECRET")
            .map_err(|_| anyhow!("TIDAL_CLIENT_SECRET environment variable is required"))?;
        // Prefer the explicit TIDAL_REDIRECT_URI env var (must match what is registered
        // in the Tidal Developer Portal). Fall back to the auto-derived callback URL.
        let redirect_uri = std::env::var("TIDAL_REDIRECT_URI")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| provider_callback_uri("tidal"));
        let client_unique_key = std::env::var("TIDAL_CLIENT_UNIQUE_KEY")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        Ok(Self {
            client_id,
            client_secret,
            redirect_uri,
            client_unique_key,
        })
    }

    /// Create a TidalConfig with explicit values
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            client_unique_key: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TidalOAuthMode {
    Legacy,
    Modern,
}

impl TidalOAuthMode {
    fn from_env() -> Self {
        match std::env::var("TIDAL_OAUTH_MODE")
            .ok()
            .map(|value| value.trim().to_ascii_lowercase())
        {
            // Keep an explicit escape hatch only for legacy troubleshooting.
            Some(value)
                if matches!(
                    value.as_str(),
                    "legacy_force" | "legacy-strict" | "legacy_strict"
                ) =>
            {
                Self::Legacy
            }
            Some(value)
                if matches!(
                    value.as_str(),
                    "modern" | "oauth21" | "oauth2.1" | "oauth_2_1"
                ) =>
            {
                Self::Modern
            }
            // Legacy authorize endpoint now commonly returns:
            // {"status":403,"sub_status":1005,"error_description":"Bearer token is missing or empty"}
            // so default to modern OAuth 2.1 flow.
            _ => Self::Modern,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Legacy => "legacy",
            Self::Modern => "modern",
        }
    }
}

/// Rate limit tracking for Tidal API.
///
/// NOTE: Each `TidalService::from_env()` call creates a fresh `RateLimitState`.
/// This means rate limit state is NOT shared across multiple service instances
/// (e.g. different request handlers or background tasks). Properly sharing state
/// would require storing it in `AppState` and passing it into `TidalService`.
/// For now this is acceptable because Tidal's rate limits are generous (100 req/min)
/// and the window resets quickly.
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
    oauth_mode: TidalOAuthMode,
}

impl TidalService {
    fn default_oauth_scopes_for_mode(mode: TidalOAuthMode) -> Vec<String> {
        match mode {
            TidalOAuthMode::Legacy => vec![
                "r_usr".to_string(),
                "r_collection".to_string(),
                "r_playlist".to_string(),
            ],
            TidalOAuthMode::Modern => vec![
                // Official developer platform scopes (see the Web API reference).
                "user.read".to_string(),
                "collection.read".to_string(),
                "playlists.read".to_string(),
            ],
        }
    }

    fn authorization_endpoint_for_mode(mode: TidalOAuthMode) -> &'static str {
        match mode {
            TidalOAuthMode::Legacy => TIDAL_AUTHORIZATION_ENDPOINT_LEGACY,
            TidalOAuthMode::Modern => TIDAL_AUTHORIZATION_ENDPOINT_MODERN,
        }
    }

    fn parse_scope_list(raw_scopes: &str) -> Vec<String> {
        raw_scopes
            .split(|ch: char| ch.is_whitespace() || ch == ',')
            .map(str::trim)
            .filter(|scope| !scope.is_empty())
            .map(str::to_string)
            .collect()
    }

    fn normalize_scopes_for_mode(mode: TidalOAuthMode, scopes: Vec<String>) -> Vec<String> {
        if scopes.is_empty() {
            return scopes;
        }

        let mapped = match mode {
            // Legacy mode expects legacy scope names.
            TidalOAuthMode::Legacy => scopes
                .into_iter()
                .map(|scope| match scope.as_str() {
                    "user.read" => "r_usr".to_string(),
                    "collection.read" => "r_collection".to_string(),
                    "playlists.read" => "r_playlist".to_string(),
                    "playlists.write" => "w_playlist".to_string(),
                    _ => scope,
                })
                .collect::<Vec<_>>(),
            // Modern mode expects official Web API scopes. If callers provide legacy names,
            // map them to modern names so the authorize URL doesn't break.
            TidalOAuthMode::Modern => scopes
                .into_iter()
                .map(|scope| match scope.as_str() {
                    "r_usr" => "user.read".to_string(),
                    "r_collection" => "collection.read".to_string(),
                    "r_playlist" => "playlists.read".to_string(),
                    "w_playlist" => "playlists.write".to_string(),
                    _ => scope,
                })
                .collect::<Vec<_>>(),
        };

        let mut deduped: Vec<String> = Vec::new();
        for scope in mapped {
            if !deduped.iter().any(|existing| existing == &scope) {
                deduped.push(scope);
            }
        }

        deduped
    }

    pub fn configured_oauth_scopes() -> Vec<String> {
        let mode = TidalOAuthMode::from_env();
        Self::configured_oauth_scopes_for_mode(mode)
    }

    fn configured_oauth_scopes_for_mode(mode: TidalOAuthMode) -> Vec<String> {
        std::env::var("TIDAL_OAUTH_SCOPES")
            .ok()
            .map(|raw| Self::parse_scope_list(&raw))
            .map(|scopes| Self::normalize_scopes_for_mode(mode, scopes))
            .filter(|scopes| !scopes.is_empty())
            .unwrap_or_else(|| Self::default_oauth_scopes_for_mode(mode))
    }

    /// Create a new TidalService
    pub fn new(config: TidalConfig) -> Self {
        let oauth_mode = TidalOAuthMode::from_env();
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            rate_limit: Arc::new(RwLock::new(RateLimitState::default())),
            oauth_mode,
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        let config = TidalConfig::from_env()?;
        Ok(Self::new(config))
    }

    pub fn oauth_mode_name(&self) -> &'static str {
        self.oauth_mode.as_str()
    }

    pub fn uses_pkce(&self) -> bool {
        matches!(self.oauth_mode, TidalOAuthMode::Modern)
    }

    /// Get the OAuth authorization URL
    pub fn get_auth_url(&self, state: &str, code_challenge: Option<&str>) -> String {
        let scopes = Self::configured_oauth_scopes_for_mode(self.oauth_mode).join(" ");

        let mut params: Vec<(&str, &str)> = vec![
            ("client_id", self.config.client_id.as_str()),
            ("response_type", "code"),
            ("redirect_uri", self.config.redirect_uri.as_str()),
            ("scope", &scopes),
            ("state", state),
        ];
        if matches!(self.oauth_mode, TidalOAuthMode::Modern) {
            if let Some(challenge) = code_challenge {
                params.push(("code_challenge", challenge));
                params.push(("code_challenge_method", "S256"));
            }
        }

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        format!(
            "{}?{}",
            Self::authorization_endpoint_for_mode(self.oauth_mode),
            query_string
        )
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(
        &self,
        code: &str,
        code_verifier: Option<&str>,
    ) -> Result<TidalTokenResponse> {
        if matches!(self.oauth_mode, TidalOAuthMode::Legacy) {
            let params = vec![
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", self.config.redirect_uri.as_str()),
                ("client_id", self.config.client_id.as_str()),
                ("client_secret", self.config.client_secret.as_str()),
            ];

            let response = self
                .client
                .post(TIDAL_TOKEN_ENDPOINT)
                .form(&params)
                .send()
                .await?;

            if response.status().is_success() {
                let token_response: TidalTokenResponse = response.json().await?;
                return Ok(token_response);
            }

            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Tidal token exchange failed (legacy {} - {})",
                status,
                error_text
            ));
        }

        let mut params = vec![
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", self.config.redirect_uri.as_str()),
            ("client_id", self.config.client_id.as_str()),
        ];
        if let Some(verifier) = code_verifier {
            params.push(("code_verifier", verifier));
        }

        // OAuth 2.1 style exchange first (PKCE + client_id).
        let response = self
            .client
            .post(TIDAL_TOKEN_ENDPOINT)
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let token_response: TidalTokenResponse = response.json().await?;
            return Ok(token_response);
        }

        let oauth21_status = response.status();
        let oauth21_error = response.text().await.unwrap_or_default();

        // Fallback to legacy basic-auth token exchange for older app configs.
        if !self.config.client_secret.is_empty() {
            let credentials = format!("{}:{}", self.config.client_id, self.config.client_secret);
            let basic_auth = general_purpose::STANDARD.encode(credentials.as_bytes());
            let legacy_response = self
                .client
                .post(TIDAL_TOKEN_ENDPOINT)
                .header("Authorization", format!("Basic {}", basic_auth))
                .form(&params)
                .send()
                .await?;

            if legacy_response.status().is_success() {
                let token_response: TidalTokenResponse = legacy_response.json().await?;
                return Ok(token_response);
            }

            let legacy_status = legacy_response.status();
            let legacy_error = legacy_response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Tidal token exchange failed (oauth2.1 {} - {}; legacy {} - {})",
                oauth21_status,
                oauth21_error,
                legacy_status,
                legacy_error
            ));
        }

        Err(anyhow!(
            "Tidal token exchange failed: {} - {}",
            oauth21_status,
            oauth21_error
        ))
    }

    /// Refresh an access token using the refresh token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TidalTokenResponse> {
        if matches!(self.oauth_mode, TidalOAuthMode::Legacy) {
            let params = vec![
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("client_id", self.config.client_id.as_str()),
                ("client_secret", self.config.client_secret.as_str()),
            ];

            let response = self
                .client
                .post(TIDAL_TOKEN_ENDPOINT)
                .form(&params)
                .send()
                .await?;

            if response.status().is_success() {
                let token_response: TidalTokenResponse = response.json().await?;
                return Ok(token_response);
            }

            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Tidal token refresh failed (legacy {} - {})",
                status,
                error_text
            ));
        }

        let params = vec![
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", self.config.client_id.as_str()),
        ];

        let response = self
            .client
            .post(TIDAL_TOKEN_ENDPOINT)
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let token_response: TidalTokenResponse = response.json().await?;
            return Ok(token_response);
        }

        let oauth21_status = response.status();
        let oauth21_error = response.text().await.unwrap_or_default();

        if !self.config.client_secret.is_empty() {
            let credentials = format!("{}:{}", self.config.client_id, self.config.client_secret);
            let basic_auth = general_purpose::STANDARD.encode(credentials.as_bytes());
            let legacy_response = self
                .client
                .post(TIDAL_TOKEN_ENDPOINT)
                .header("Authorization", format!("Basic {}", basic_auth))
                .form(&params)
                .send()
                .await?;

            if legacy_response.status().is_success() {
                let token_response: TidalTokenResponse = legacy_response.json().await?;
                return Ok(token_response);
            }

            let legacy_status = legacy_response.status();
            let legacy_error = legacy_response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Tidal token refresh failed (oauth2.1 {} - {}; legacy {} - {})",
                oauth21_status,
                oauth21_error,
                legacy_status,
                legacy_error
            ));
        }

        Err(anyhow!(
            "Tidal token refresh failed: {} - {}",
            oauth21_status,
            oauth21_error
        ))
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
        let access_token = access_token.trim();
        if access_token.is_empty() {
            return Err(anyhow!(
                "Tidal access token is missing or empty. Disconnect and reconnect Tidal."
            ));
        }

        self.wait_for_rate_limit().await?;

        let url = if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            endpoint.to_string()
        } else {
            format!("{}{}", TIDAL_API_BASE, endpoint)
        };
        let mut request = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/vnd.api+json, application/json");

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
    #[allow(dead_code)]
    async fn delete(&self, access_token: &str, endpoint: &str) -> Result<()> {
        let access_token = access_token.trim();
        if access_token.is_empty() {
            return Err(anyhow!(
                "Tidal access token is missing or empty. Disconnect and reconnect Tidal."
            ));
        }

        self.wait_for_rate_limit().await?;

        let url = if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            endpoint.to_string()
        } else {
            format!("{}{}", TIDAL_API_BASE, endpoint)
        };
        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/vnd.api+json, application/json")
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
    #[allow(dead_code)]
    async fn post(
        &self,
        access_token: &str,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<()> {
        let access_token = access_token.trim();
        if access_token.is_empty() {
            return Err(anyhow!(
                "Tidal access token is missing or empty. Disconnect and reconnect Tidal."
            ));
        }

        self.wait_for_rate_limit().await?;

        let url = if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            endpoint.to_string()
        } else {
            format!("{}{}", TIDAL_API_BASE, endpoint)
        };
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/vnd.api+json, application/json")
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
        #[derive(Debug, Default, Deserialize)]
        struct UsersMeAttributes {
            #[serde(default)]
            country: Option<String>,
            #[serde(default)]
            username: Option<String>,
            #[serde(rename = "firstName", default)]
            first_name: Option<String>,
            #[serde(rename = "lastName", default)]
            last_name: Option<String>,
            #[serde(default)]
            email: Option<String>,
        }

        #[derive(Debug, Deserialize)]
        struct UsersMeResource {
            id: String,
            #[serde(default)]
            attributes: UsersMeAttributes,
        }

        #[derive(Debug, Deserialize)]
        struct UsersMeDocument {
            data: UsersMeResource,
        }

        match self
            .get::<UsersMeDocument>(access_token, "/users/me", &[])
            .await
        {
            Ok(me) => {
                let id = me.data.id.parse::<u64>().map_err(|_| {
                    anyhow!("Tidal /users/me returned non-numeric id {}", me.data.id)
                })?;

                return Ok(TidalUser {
                    id,
                    username: me.data.attributes.username,
                    first_name: me.data.attributes.first_name,
                    last_name: me.data.attributes.last_name,
                    email: me.data.attributes.email,
                    country_code: me.data.attributes.country,
                    picture: None,
                });
            }
            Err(e) => {
                warn!(error = %e, "Tidal /users/me lookup failed; falling back to token claims");
            }
        }

        // Final fallback: decode user identifier from JWT-style access token claims.
        if let Some(user_id) = Self::extract_user_id_from_access_token(access_token) {
            warn!(
                user_id,
                "Falling back to user id extracted from access token claims"
            );
            return Ok(TidalUser {
                id: user_id,
                username: None,
                first_name: None,
                last_name: None,
                email: None,
                country_code: None,
                picture: None,
            });
        }

        Err(anyhow!(
            "Unable to resolve Tidal user profile from /users/me or access token claims"
        ))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn parse_user_profile_value(value: &serde_json::Value) -> Option<TidalUser> {
        let id = value
            .get("id")
            .and_then(|v| v.as_u64())
            .or_else(|| value.get("userId").and_then(|v| v.as_u64()))?;

        let username = value
            .get("username")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let first_name = value
            .get("firstName")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let last_name = value
            .get("lastName")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let email = value
            .get("email")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let country_code = value
            .get("countryCode")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                value
                    .get("country")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            });
        let picture = value
            .get("picture")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Some(TidalUser {
            id,
            username,
            first_name,
            last_name,
            email,
            country_code,
            picture,
        })
    }

    fn extract_user_id_from_access_token(access_token: &str) -> Option<u64> {
        let payload_segment = access_token.split('.').nth(1)?;
        let payload_bytes = general_purpose::URL_SAFE_NO_PAD
            .decode(payload_segment)
            .or_else(|_| general_purpose::URL_SAFE.decode(payload_segment))
            .ok()?;
        let payload: serde_json::Value = serde_json::from_slice(&payload_bytes).ok()?;

        payload
            .get("sub")
            .and_then(|v| v.as_u64())
            .or_else(|| payload.get("uid").and_then(|v| v.as_u64()))
            .or_else(|| payload.get("user_id").and_then(|v| v.as_u64()))
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

    /// Get tracks within a specific playlist (single page).
    pub async fn get_playlist_tracks(
        &self,
        access_token: &str,
        playlist_uuid: &str,
        country_code: &str,
        limit: u32,
        offset: u32,
    ) -> Result<TidalPaginatedResponse<TidalPlaylistTrack>> {
        let limit_str = limit.to_string();
        let offset_str = offset.to_string();

        self.get(
            access_token,
            &format!("/playlists/{}/items", playlist_uuid),
            &[
                ("countryCode", country_code),
                ("limit", &limit_str),
                ("offset", &offset_str),
            ],
        )
        .await
    }

    /// Get all tracks within a specific playlist (handles pagination).
    pub async fn get_all_playlist_tracks(
        &self,
        access_token: &str,
        playlist_uuid: &str,
        country_code: &str,
    ) -> Result<Vec<TidalPlaylistTrack>> {
        let mut all_tracks = Vec::new();
        let mut offset = 0;

        loop {
            let response = self
                .get_playlist_tracks(
                    access_token,
                    playlist_uuid,
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

    async fn send_json_api_request(
        &self,
        method: Method,
        access_token: &str,
        endpoint: &str,
        body: Option<&Value>,
    ) -> Result<Value> {
        let access_token = access_token.trim();
        if access_token.is_empty() {
            return Err(anyhow!(
                "Tidal access token is missing or empty. Disconnect and reconnect Tidal."
            ));
        }

        self.wait_for_rate_limit().await?;

        let url = if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            endpoint.to_string()
        } else {
            format!("{}{}", TIDAL_API_BASE, endpoint)
        };

        let mut request = self
            .client
            .request(method, &url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/vnd.api+json, application/json");

        if let Some(body) = body {
            request = request
                .header("Content-Type", "application/vnd.api+json")
                .json(body);
        }

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

        if status == StatusCode::NO_CONTENT {
            return Ok(Value::Null);
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Tidal API request failed: {} - {}",
                status,
                error_text
            ));
        }

        let text = response.text().await.unwrap_or_default();
        if text.trim().is_empty() {
            return Ok(Value::Null);
        }

        Ok(serde_json::from_str(&text).unwrap_or(Value::Null))
    }

    /// Remove a track from favorites
    pub async fn remove_favorite_track(
        &self,
        access_token: &str,
        user_id: u64,
        track_id: u64,
    ) -> Result<()> {
        let payload = serde_json::json!({
            "data": [{
                "id": track_id.to_string(),
                "type": "tracks"
            }]
        });

        self.send_json_api_request(
            Method::DELETE,
            access_token,
            &format!("/userCollections/{}/relationships/tracks", user_id),
            Some(&payload),
        )
        .await?;

        Ok(())
    }

    /// Remove an artist from favorites
    pub async fn remove_favorite_artist(
        &self,
        access_token: &str,
        user_id: u64,
        artist_id: u64,
    ) -> Result<()> {
        let payload = serde_json::json!({
            "data": [{
                "id": artist_id.to_string(),
                "type": "artists"
            }]
        });

        self.send_json_api_request(
            Method::DELETE,
            access_token,
            &format!("/userCollections/{}/relationships/artists", user_id),
            Some(&payload),
        )
        .await?;

        Ok(())
    }

    /// Remove an album from favorites
    pub async fn remove_favorite_album(
        &self,
        access_token: &str,
        user_id: u64,
        album_id: u64,
    ) -> Result<()> {
        let payload = serde_json::json!({
            "data": [{
                "id": album_id.to_string(),
                "type": "albums"
            }]
        });

        self.send_json_api_request(
            Method::DELETE,
            access_token,
            &format!("/userCollections/{}/relationships/albums", user_id),
            Some(&payload),
        )
        .await?;

        Ok(())
    }

    /// Remove a track from a playlist
    pub async fn remove_playlist_track(
        &self,
        _access_token: &str,
        _playlist_uuid: &str,
        _track_index: u32,
    ) -> Result<()> {
        Err(anyhow!(
            "Removing playlist items is not implemented for the TIDAL Web API v2 integration (playlist item ids required)."
        ))
    }

    /// Add a track to favorites (for rollback support)
    pub async fn add_favorite_track(
        &self,
        access_token: &str,
        user_id: u64,
        track_id: u64,
    ) -> Result<()> {
        let payload = serde_json::json!({
            "data": [{
                "id": track_id.to_string(),
                "type": "tracks"
            }]
        });

        self.send_json_api_request(
            Method::POST,
            access_token,
            &format!("/userCollections/{}/relationships/tracks", user_id),
            Some(&payload),
        )
        .await?;

        Ok(())
    }

    /// Add an artist to favorites (for rollback support)
    pub async fn add_favorite_artist(
        &self,
        access_token: &str,
        user_id: u64,
        artist_id: u64,
    ) -> Result<()> {
        let payload = serde_json::json!({
            "data": [{
                "id": artist_id.to_string(),
                "type": "artists"
            }]
        });

        self.send_json_api_request(
            Method::POST,
            access_token,
            &format!("/userCollections/{}/relationships/artists", user_id),
            Some(&payload),
        )
        .await?;

        Ok(())
    }

    /// Add an album to favorites (for rollback support)
    pub async fn add_favorite_album(
        &self,
        access_token: &str,
        user_id: u64,
        album_id: u64,
    ) -> Result<()> {
        let payload = serde_json::json!({
            "data": [{
                "id": album_id.to_string(),
                "type": "albums"
            }]
        });

        self.send_json_api_request(
            Method::POST,
            access_token,
            &format!("/userCollections/{}/relationships/albums", user_id),
            Some(&payload),
        )
        .await?;

        Ok(())
    }

    async fn fetch_relationship_ids_with_added_at(
        &self,
        access_token: &str,
        initial_endpoint: &str,
        country_code: &str,
    ) -> Result<(Vec<(String, DateTime<Utc>)>, u32)> {
        #[derive(Debug, Deserialize)]
        struct RelationshipMeta {
            #[serde(rename = "addedAt")]
            added_at: String,
        }

        #[derive(Debug, Deserialize)]
        struct RelationshipDataItem {
            id: String,
            #[serde(default)]
            meta: Option<RelationshipMeta>,
        }

        #[derive(Debug, Deserialize)]
        struct RelationshipLinks {
            #[serde(default)]
            next: Option<String>,
        }

        #[derive(Debug, Deserialize)]
        struct RelationshipDocument {
            #[serde(default)]
            data: Vec<RelationshipDataItem>,
            links: RelationshipLinks,
        }

        let mut endpoint = initial_endpoint.to_string();
        let mut out: Vec<(String, DateTime<Utc>)> = Vec::new();
        let mut requests: u32 = 0;

        loop {
            let params_owned = [
                ("countryCode".to_string(), country_code.to_string()),
                ("locale".to_string(), TIDAL_DEFAULT_LOCALE.to_string()),
            ];
            let params_ref: Vec<(&str, &str)> = params_owned
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();

            let response: RelationshipDocument =
                self.get(access_token, &endpoint, &params_ref).await?;
            requests += 1;

            for item in response.data {
                let added_at = item
                    .meta
                    .as_ref()
                    .and_then(|meta| parse_rfc3339_datetime(&meta.added_at))
                    .unwrap_or_else(Utc::now);
                out.push((item.id, added_at));
            }

            let next = response
                .links
                .next
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty());
            match next {
                Some(next_endpoint) => endpoint = next_endpoint.to_string(),
                None => break,
            }
        }

        Ok((out, requests))
    }

    fn parse_artist_resource(value: &Value) -> Option<(String, TidalArtist)> {
        let id_str = value.get("id")?.as_str()?.to_string();
        let id = id_str.parse::<u64>().ok()?;
        let attributes = value.get("attributes")?;
        let name = attributes.get("name")?.as_str()?.to_string();

        Some((
            id_str,
            TidalArtist {
                id,
                name,
                url: None,
                picture: None,
                artist_type: None,
            },
        ))
    }

    fn parse_album_resource(
        value: &Value,
        artists_by_id: &HashMap<String, TidalArtist>,
    ) -> Option<(String, TidalAlbum)> {
        let id_str = value.get("id")?.as_str()?.to_string();
        let id = id_str.parse::<u64>().ok()?;
        let attributes = value.get("attributes")?;
        let title = attributes.get("title")?.as_str()?.to_string();

        let release_date = attributes
            .get("releaseDate")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let duration = attributes
            .get("duration")
            .and_then(|v| v.as_str())
            .and_then(parse_iso8601_duration_seconds);
        let number_of_tracks = attributes
            .get("numberOfItems")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        let mut artists: Vec<TidalArtist> = Vec::new();
        if let Some(relationship_data) = value
            .get("relationships")
            .and_then(|r| r.get("artists"))
            .and_then(|r| r.get("data"))
            .and_then(|d| d.as_array())
        {
            for rel in relationship_data {
                if let Some(artist_id) = rel.get("id").and_then(|v| v.as_str()) {
                    if let Some(artist) = artists_by_id.get(artist_id) {
                        artists.push(artist.clone());
                    }
                }
            }
        }

        Some((
            id_str,
            TidalAlbum {
                id,
                title,
                artists,
                cover: None,
                release_date,
                duration,
                number_of_tracks,
                url: None,
            },
        ))
    }

    fn parse_track_resource(
        value: &Value,
        artists_by_id: &HashMap<String, TidalArtist>,
        albums_by_id: &HashMap<String, TidalAlbum>,
    ) -> Option<(String, TidalTrack)> {
        let id_str = value.get("id")?.as_str()?.to_string();
        let id = id_str.parse::<u64>().ok()?;
        let attributes = value.get("attributes")?;

        let title = attributes.get("title")?.as_str()?.to_string();
        let explicit = attributes
            .get("explicit")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let duration = attributes
            .get("duration")
            .and_then(|v| v.as_str())
            .and_then(parse_iso8601_duration_seconds)
            .unwrap_or(0);
        let popularity = attributes
            .get("popularity")
            .and_then(|v| v.as_f64())
            .map(|v| (v * 100.0).round() as u32);
        let isrc = attributes
            .get("isrc")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());

        let audio_quality = attributes
            .get("mediaTags")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .unwrap_or("UNKNOWN")
            .to_string();

        let mut artists: Vec<TidalArtist> = Vec::new();
        if let Some(relationship_data) = value
            .get("relationships")
            .and_then(|r| r.get("artists"))
            .and_then(|r| r.get("data"))
            .and_then(|d| d.as_array())
        {
            for rel in relationship_data {
                if let Some(artist_id) = rel.get("id").and_then(|v| v.as_str()) {
                    if let Some(artist) = artists_by_id.get(artist_id) {
                        artists.push(artist.clone());
                    } else if let Ok(parsed_id) = artist_id.parse::<u64>() {
                        artists.push(TidalArtist {
                            id: parsed_id,
                            name: "Unknown Artist".to_string(),
                            url: None,
                            picture: None,
                            artist_type: None,
                        });
                    }
                }
            }
        }
        if artists.is_empty() {
            artists.push(TidalArtist {
                id: 0,
                name: "Unknown Artist".to_string(),
                url: None,
                picture: None,
                artist_type: None,
            });
        }

        let album = value
            .get("relationships")
            .and_then(|r| r.get("albums"))
            .and_then(|r| r.get("data"))
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .and_then(|rel| rel.get("id"))
            .and_then(|v| v.as_str())
            .and_then(|album_id| albums_by_id.get(album_id))
            .cloned()
            .unwrap_or_else(|| TidalAlbum {
                id: 0,
                title: "Unknown Album".to_string(),
                artists: Vec::new(),
                cover: None,
                release_date: None,
                duration: None,
                number_of_tracks: None,
                url: None,
            });

        Some((
            id_str,
            TidalTrack {
                id,
                title,
                artists,
                album,
                duration,
                explicit,
                popularity,
                audio_quality,
                url: String::new(),
                isrc,
            },
        ))
    }

    fn parse_playlist_resource(value: &Value) -> Option<(String, TidalPlaylist)> {
        let id_str = value.get("id")?.as_str()?.to_string();
        let attributes = value.get("attributes")?;

        let title = attributes.get("name")?.as_str()?.to_string();
        let description = attributes
            .get("description")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());

        let duration = attributes
            .get("duration")
            .and_then(|v| v.as_str())
            .and_then(parse_iso8601_duration_seconds)
            .unwrap_or(0);

        let number_of_tracks = attributes
            .get("numberOfItems")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let public_playlist = attributes
            .get("accessType")
            .and_then(|v| v.as_str())
            .map(|v| v == "PUBLIC")
            .unwrap_or(false);

        let created = attributes
            .get("createdAt")
            .and_then(|v| v.as_str())
            .and_then(parse_rfc3339_datetime)
            .unwrap_or_else(Utc::now);
        let last_updated = attributes
            .get("lastModifiedAt")
            .and_then(|v| v.as_str())
            .and_then(parse_rfc3339_datetime)
            .unwrap_or(created);

        Some((
            id_str.clone(),
            TidalPlaylist {
                uuid: id_str,
                title,
                description,
                creator: None,
                duration,
                number_of_tracks,
                public_playlist,
                url: None,
                image: None,
                created,
                last_updated,
            },
        ))
    }

    async fn fetch_tracks_by_ids(
        &self,
        access_token: &str,
        ids: &[String],
        country_code: &str,
    ) -> Result<(HashMap<String, TidalTrack>, u32)> {
        let mut out: HashMap<String, TidalTrack> = HashMap::new();
        let mut requests = 0u32;

        for chunk in ids.chunks(TIDAL_BULK_FETCH_IDS_BATCH_SIZE) {
            if chunk.is_empty() {
                continue;
            }

            let mut params_owned: Vec<(String, String)> = Vec::new();
            params_owned.push(("countryCode".to_string(), country_code.to_string()));
            params_owned.push(("locale".to_string(), TIDAL_DEFAULT_LOCALE.to_string()));
            for id in chunk {
                params_owned.push(("filter[id]".to_string(), id.to_string()));
            }
            params_owned.push(("include".to_string(), "artists".to_string()));
            params_owned.push(("include".to_string(), "albums".to_string()));

            let params_ref: Vec<(&str, &str)> = params_owned
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();

            let response: Value = self.get(access_token, "/tracks", &params_ref).await?;
            requests += 1;

            let included = response
                .get("included")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);

            let mut artists_by_id: HashMap<String, TidalArtist> = HashMap::new();
            for item in included {
                if item.get("type").and_then(|v| v.as_str()) != Some("artists") {
                    continue;
                }
                if let Some((id, artist)) = Self::parse_artist_resource(item) {
                    artists_by_id.insert(id, artist);
                }
            }

            let mut albums_by_id: HashMap<String, TidalAlbum> = HashMap::new();
            for item in included {
                if item.get("type").and_then(|v| v.as_str()) != Some("albums") {
                    continue;
                }
                if let Some((id, album)) = Self::parse_album_resource(item, &artists_by_id) {
                    albums_by_id.insert(id, album);
                }
            }

            let data = response
                .get("data")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);

            for item in data {
                if let Some((id, track)) =
                    Self::parse_track_resource(item, &artists_by_id, &albums_by_id)
                {
                    out.insert(id, track);
                }
            }
        }

        Ok((out, requests))
    }

    async fn fetch_artists_by_ids(
        &self,
        access_token: &str,
        ids: &[String],
        country_code: &str,
    ) -> Result<(HashMap<String, TidalArtist>, u32)> {
        let mut out: HashMap<String, TidalArtist> = HashMap::new();
        let mut requests = 0u32;

        for chunk in ids.chunks(TIDAL_BULK_FETCH_IDS_BATCH_SIZE) {
            if chunk.is_empty() {
                continue;
            }

            let mut params_owned: Vec<(String, String)> = Vec::new();
            params_owned.push(("countryCode".to_string(), country_code.to_string()));
            params_owned.push(("locale".to_string(), TIDAL_DEFAULT_LOCALE.to_string()));
            for id in chunk {
                params_owned.push(("filter[id]".to_string(), id.to_string()));
            }

            let params_ref: Vec<(&str, &str)> = params_owned
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();

            let response: Value = self.get(access_token, "/artists", &params_ref).await?;
            requests += 1;

            let data = response
                .get("data")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);

            for item in data {
                if let Some((id, artist)) = Self::parse_artist_resource(item) {
                    out.insert(id, artist);
                }
            }
        }

        Ok((out, requests))
    }

    async fn fetch_albums_by_ids(
        &self,
        access_token: &str,
        ids: &[String],
        country_code: &str,
    ) -> Result<(HashMap<String, TidalAlbum>, u32)> {
        let mut out: HashMap<String, TidalAlbum> = HashMap::new();
        let mut requests = 0u32;

        for chunk in ids.chunks(TIDAL_BULK_FETCH_IDS_BATCH_SIZE) {
            if chunk.is_empty() {
                continue;
            }

            let mut params_owned: Vec<(String, String)> = Vec::new();
            params_owned.push(("countryCode".to_string(), country_code.to_string()));
            params_owned.push(("locale".to_string(), TIDAL_DEFAULT_LOCALE.to_string()));
            for id in chunk {
                params_owned.push(("filter[id]".to_string(), id.to_string()));
            }
            params_owned.push(("include".to_string(), "artists".to_string()));

            let params_ref: Vec<(&str, &str)> = params_owned
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();

            let response: Value = self.get(access_token, "/albums", &params_ref).await?;
            requests += 1;

            let included = response
                .get("included")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);

            let mut artists_by_id: HashMap<String, TidalArtist> = HashMap::new();
            for item in included {
                if item.get("type").and_then(|v| v.as_str()) != Some("artists") {
                    continue;
                }
                if let Some((id, artist)) = Self::parse_artist_resource(item) {
                    artists_by_id.insert(id, artist);
                }
            }

            let data = response
                .get("data")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);

            for item in data {
                if let Some((id, album)) = Self::parse_album_resource(item, &artists_by_id) {
                    out.insert(id, album);
                }
            }
        }

        Ok((out, requests))
    }

    async fn fetch_playlists_by_ids(
        &self,
        access_token: &str,
        ids: &[String],
        country_code: &str,
    ) -> Result<(HashMap<String, TidalPlaylist>, u32)> {
        let mut out: HashMap<String, TidalPlaylist> = HashMap::new();
        let mut requests = 0u32;

        for chunk in ids.chunks(TIDAL_BULK_FETCH_IDS_BATCH_SIZE) {
            if chunk.is_empty() {
                continue;
            }

            let mut params_owned: Vec<(String, String)> = Vec::new();
            params_owned.push(("countryCode".to_string(), country_code.to_string()));
            params_owned.push(("locale".to_string(), TIDAL_DEFAULT_LOCALE.to_string()));
            for id in chunk {
                params_owned.push(("filter[id]".to_string(), id.to_string()));
            }

            let params_ref: Vec<(&str, &str)> = params_owned
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();

            let response: Value = self.get(access_token, "/playlists", &params_ref).await?;
            requests += 1;

            let data = response
                .get("data")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);

            for item in data {
                if let Some((id, playlist)) = Self::parse_playlist_resource(item) {
                    out.insert(id, playlist);
                }
            }
        }

        Ok((out, requests))
    }

    /// Scan user's full library
    pub async fn scan_library(
        &self,
        access_token: &str,
        internal_user_id: Uuid,
    ) -> Result<TidalLibraryScanResult> {
        let started_at = Utc::now();
        let mut api_requests_count = 0;
        let rate_limit_retries = 0;
        let mut warnings = Vec::new();

        // Get user profile first
        let user = self
            .get_current_user(access_token)
            .await
            .map_err(|e| anyhow!("Failed to get user profile: {}", e))?;
        api_requests_count += 1;

        let country_code = user
            .country_code
            .clone()
            .unwrap_or_else(|| "US".to_string());
        let collection_id = user.id.to_string();

        // Relationship documents hold ids + addedAt timestamps (cursor-paginated).
        let (track_refs, track_ref_requests) = match self
            .fetch_relationship_ids_with_added_at(
                access_token,
                &format!("/userCollections/{}/relationships/tracks", &collection_id),
                &country_code,
            )
            .await
        {
            Ok(result) => result,
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("403")
                    || msg.contains("401")
                    || msg.to_ascii_lowercase().contains("scope")
                    || msg.to_ascii_lowercase().contains("insufficient")
                {
                    warnings.push(format!(
                        "Missing required scope for tracks collection access: {}",
                        msg
                    ));
                } else {
                    warnings.push(format!("Failed to fetch Tidal track collection: {}", msg));
                }
                (Vec::new(), 0)
            }
        };
        api_requests_count += track_ref_requests;

        let track_ids: Vec<String> = track_refs.iter().map(|(id, _)| id.clone()).collect();
        let (tracks_by_id, track_detail_requests) = match self
            .fetch_tracks_by_ids(access_token, &track_ids, &country_code)
            .await
        {
            Ok(result) => result,
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("403")
                    || msg.contains("401")
                    || msg.to_ascii_lowercase().contains("scope")
                    || msg.to_ascii_lowercase().contains("insufficient")
                {
                    warnings.push(format!(
                        "Missing required scope for track details lookup: {}",
                        msg
                    ));
                } else {
                    warnings.push(format!("Failed to fetch track details: {}", msg));
                }
                (HashMap::new(), 0)
            }
        };
        api_requests_count += track_detail_requests;

        let mut favorite_tracks: Vec<TidalFavoriteTrack> = Vec::new();
        for (id, added_at) in track_refs {
            match tracks_by_id.get(&id) {
                Some(track) => favorite_tracks.push(TidalFavoriteTrack {
                    created: added_at,
                    item: track.clone(),
                }),
                None => warnings.push(format!(
                    "Track {} referenced in collection but missing from /tracks response",
                    id
                )),
            }
        }

        let (artist_refs, artist_ref_requests) = match self
            .fetch_relationship_ids_with_added_at(
                access_token,
                &format!("/userCollections/{}/relationships/artists", &collection_id),
                &country_code,
            )
            .await
        {
            Ok(result) => result,
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("403")
                    || msg.contains("401")
                    || msg.to_ascii_lowercase().contains("scope")
                    || msg.to_ascii_lowercase().contains("insufficient")
                {
                    warnings.push(format!(
                        "Missing required scope for artists collection access: {}",
                        msg
                    ));
                } else {
                    warnings.push(format!("Failed to fetch artist collection: {}", msg));
                }
                (Vec::new(), 0)
            }
        };
        api_requests_count += artist_ref_requests;

        let artist_ids: Vec<String> = artist_refs.iter().map(|(id, _)| id.clone()).collect();
        let (artists_by_id, artist_detail_requests) = match self
            .fetch_artists_by_ids(access_token, &artist_ids, &country_code)
            .await
        {
            Ok(result) => result,
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("403")
                    || msg.contains("401")
                    || msg.to_ascii_lowercase().contains("scope")
                    || msg.to_ascii_lowercase().contains("insufficient")
                {
                    warnings.push(format!(
                        "Missing required scope for artist details lookup: {}",
                        msg
                    ));
                } else {
                    warnings.push(format!("Failed to fetch artist details: {}", msg));
                }
                (HashMap::new(), 0)
            }
        };
        api_requests_count += artist_detail_requests;

        let mut favorite_artists: Vec<TidalFavoriteArtist> = Vec::new();
        for (id, added_at) in artist_refs {
            match artists_by_id.get(&id) {
                Some(artist) => favorite_artists.push(TidalFavoriteArtist {
                    created: added_at,
                    item: artist.clone(),
                }),
                None => warnings.push(format!(
                    "Artist {} referenced in collection but missing from /artists response",
                    id
                )),
            }
        }

        let (album_refs, album_ref_requests) = match self
            .fetch_relationship_ids_with_added_at(
                access_token,
                &format!("/userCollections/{}/relationships/albums", &collection_id),
                &country_code,
            )
            .await
        {
            Ok(result) => result,
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("403")
                    || msg.contains("401")
                    || msg.to_ascii_lowercase().contains("scope")
                    || msg.to_ascii_lowercase().contains("insufficient")
                {
                    warnings.push(format!(
                        "Missing required scope for albums collection access: {}",
                        msg
                    ));
                } else {
                    warnings.push(format!("Failed to fetch album collection: {}", msg));
                }
                (Vec::new(), 0)
            }
        };
        api_requests_count += album_ref_requests;

        let album_ids: Vec<String> = album_refs.iter().map(|(id, _)| id.clone()).collect();
        let (albums_by_id, album_detail_requests) = match self
            .fetch_albums_by_ids(access_token, &album_ids, &country_code)
            .await
        {
            Ok(result) => result,
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("403")
                    || msg.contains("401")
                    || msg.to_ascii_lowercase().contains("scope")
                    || msg.to_ascii_lowercase().contains("insufficient")
                {
                    warnings.push(format!(
                        "Missing required scope for album details lookup: {}",
                        msg
                    ));
                } else {
                    warnings.push(format!("Failed to fetch album details: {}", msg));
                }
                (HashMap::new(), 0)
            }
        };
        api_requests_count += album_detail_requests;

        let mut favorite_albums: Vec<TidalFavoriteAlbum> = Vec::new();
        for (id, added_at) in album_refs {
            match albums_by_id.get(&id) {
                Some(album) => favorite_albums.push(TidalFavoriteAlbum {
                    created: added_at,
                    item: album.clone(),
                }),
                None => warnings.push(format!(
                    "Album {} referenced in collection but missing from /albums response",
                    id
                )),
            }
        }

        let (playlist_refs, playlist_ref_requests) = match self
            .fetch_relationship_ids_with_added_at(
                access_token,
                &format!(
                    "/userCollections/{}/relationships/playlists",
                    &collection_id
                ),
                &country_code,
            )
            .await
        {
            Ok(result) => result,
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("403")
                    || msg.contains("401")
                    || msg.to_ascii_lowercase().contains("scope")
                    || msg.to_ascii_lowercase().contains("insufficient")
                {
                    warnings.push(format!(
                        "Missing required scope for playlists collection access: {}",
                        msg
                    ));
                } else {
                    warnings.push(format!("Failed to fetch playlist collection: {}", msg));
                }
                (Vec::new(), 0)
            }
        };
        api_requests_count += playlist_ref_requests;

        let playlist_ids: Vec<String> = playlist_refs.iter().map(|(id, _)| id.clone()).collect();
        let (playlists_by_id, playlist_detail_requests) = match self
            .fetch_playlists_by_ids(access_token, &playlist_ids, &country_code)
            .await
        {
            Ok(result) => result,
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("403")
                    || msg.contains("401")
                    || msg.to_ascii_lowercase().contains("scope")
                    || msg.to_ascii_lowercase().contains("insufficient")
                {
                    warnings.push(format!(
                        "Missing required scope for playlist details lookup: {}",
                        msg
                    ));
                } else {
                    warnings.push(format!("Failed to fetch playlist details: {}", msg));
                }
                (HashMap::new(), 0)
            }
        };
        api_requests_count += playlist_detail_requests;

        let mut playlists: Vec<TidalPlaylist> = Vec::new();
        for (id, added_at) in playlist_refs {
            match playlists_by_id.get(&id) {
                Some(playlist) => {
                    let mut playlist = playlist.clone();
                    // Treat "addedAt" as the most relevant timestamp for library imports.
                    playlist.last_updated = added_at;
                    playlists.push(playlist);
                }
                None => warnings.push(format!(
                    "Playlist {} referenced in collection but missing from /playlists response",
                    id
                )),
            }
        }

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
        assert!(config.client_unique_key.is_none());
    }

    #[test]
    fn test_get_auth_url() {
        let config = TidalConfig::new(
            "test_client_id".to_string(),
            "test_secret".to_string(),
            "http://localhost:3000/callback".to_string(),
        );
        let service = TidalService::new(config);
        let url = service.get_auth_url("test_state", Some("pkce_challenge"));

        if service.uses_pkce() {
            assert!(url.starts_with("https://login.tidal.com/authorize"));
            assert!(url.contains("code_challenge=pkce_challenge"));
            assert!(url.contains("code_challenge_method=S256"));
        } else {
            assert!(url.starts_with("https://auth.tidal.com/v1/oauth2/authorize"));
            assert!(!url.contains("code_challenge=pkce_challenge"));
        }
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

    #[test]
    fn test_parse_user_profile_value_supports_user_id_shape() {
        let payload = serde_json::json!({
            "userId": 12345u64,
            "countryCode": "US",
            "username": "test-user"
        });

        let user = TidalService::parse_user_profile_value(&payload).expect("user should parse");
        assert_eq!(user.id, 12345);
        assert_eq!(user.country_code.as_deref(), Some("US"));
        assert_eq!(user.username.as_deref(), Some("test-user"));
    }

    #[test]
    fn test_extract_user_id_from_access_token_jwt_payload() {
        let header = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(r#"{"alg":"none"}"#);
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(r#"{"sub":987654}"#);
        let token = format!("{}.{}.sig", header, payload);

        let extracted = TidalService::extract_user_id_from_access_token(&token);
        assert_eq!(extracted, Some(987654));
    }
}
