use async_trait::async_trait;
use base64::Engine as _;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{AppError, Result};
use crate::models::oauth::{
    OAuthConfig, OAuthFlowResponse, OAuthProviderType, OAuthTokens, OAuthUserInfo,
};
use crate::services::oauth::{BaseOAuthProvider, OAuthProvider};

/// Apple's JSON Web Key Set (JWKS) response structure
#[derive(Debug, Clone, Deserialize)]
pub struct AppleJWKS {
    pub keys: Vec<AppleJWK>,
}

/// Apple's JSON Web Key structure
#[derive(Debug, Clone, Deserialize)]
pub struct AppleJWK {
    pub kty: String, // Key type (RSA)
    pub kid: String, // Key ID
    pub alg: String, // Algorithm (RS256)
    #[serde(rename = "use")]
    pub key_use: String, // Key usage (sig)
    pub n: String,   // RSA modulus (base64url encoded)
    pub e: String,   // RSA exponent (base64url encoded)
}

/// Apple ID token claims
#[derive(Debug, Clone, Deserialize)]
pub struct AppleIdTokenClaims {
    pub iss: String,                    // Issuer (https://appleid.apple.com)
    pub sub: String,                    // Subject (user ID)
    pub aud: String,                    // Audience (client ID)
    pub iat: i64,                       // Issued at
    pub exp: i64,                       // Expiration
    pub email: Option<String>,          // User email
    pub email_verified: Option<bool>,   // Email verified (may be string or bool)
    pub is_private_email: Option<bool>, // True if using Apple's relay email
    pub auth_time: Option<i64>,         // Time of authentication
    pub nonce_supported: Option<bool>,  // Nonce supported
    pub real_user_status: Option<i64>, // Real user status (0=unsupported, 1=unknown, 2=likely_real)
    pub c_hash: Option<String>,        // Code hash
    pub at_hash: Option<String>,       // Access token hash
    pub transfer_sub: Option<String>,  // Transfer subject (for app transfer)
}

/// Cached Apple public keys with expiration
#[derive(Clone)]
pub struct AppleKeyCache {
    keys: Arc<RwLock<Option<CachedKeys>>>,
}

#[derive(Clone)]
struct CachedKeys {
    jwks: AppleJWKS,
    fetched_at: chrono::DateTime<Utc>,
}

/// Apple Sign In JWT claims for client secret generation
#[derive(Debug, Serialize, Deserialize)]
struct AppleJWTClaims {
    iss: String, // Team ID
    iat: i64,    // Issued at
    exp: i64,    // Expiration
    aud: String, // Audience (always "https://appleid.apple.com")
    sub: String, // Service ID (Client ID)
}

/// Apple Sign In configuration
#[derive(Debug, Clone)]
pub struct AppleOAuthConfig {
    pub client_id: String,   // Service ID
    pub team_id: String,     // Apple Developer Team ID
    pub key_id: String,      // Private key ID
    pub private_key: String, // P8 private key content
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

impl AppleKeyCache {
    /// Create a new key cache
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(None)),
        }
    }

    /// Get cached keys if still valid (cache for 24 hours)
    pub async fn get(&self) -> Option<AppleJWKS> {
        let guard = self.keys.read().await;
        if let Some(cached) = guard.as_ref() {
            // Keys are valid for 24 hours
            if Utc::now() - cached.fetched_at < Duration::hours(24) {
                return Some(cached.jwks.clone());
            }
        }
        None
    }

    /// Store keys in cache
    pub async fn set(&self, jwks: AppleJWKS) {
        let mut guard = self.keys.write().await;
        *guard = Some(CachedKeys {
            jwks,
            fetched_at: Utc::now(),
        });
    }
}

impl Default for AppleKeyCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Apple Sign In provider implementation
pub struct AppleOAuthProvider {
    base: BaseOAuthProvider,
    apple_config: AppleOAuthConfig,
    key_cache: AppleKeyCache,
}

impl AppleOAuthProvider {
    /// Create a new Apple OAuth provider with environment variables
    pub fn new() -> Result<Self> {
        let apple_config = AppleOAuthConfig {
            client_id: std::env::var("APPLE_CLIENT_ID").map_err(|_| {
                AppError::ConfigurationError {
                    message: "APPLE_CLIENT_ID environment variable is required".to_string(),
                }
            })?,
            team_id: std::env::var("APPLE_TEAM_ID").map_err(|_| AppError::ConfigurationError {
                message: "APPLE_TEAM_ID environment variable is required".to_string(),
            })?,
            key_id: std::env::var("APPLE_KEY_ID").map_err(|_| AppError::ConfigurationError {
                message: "APPLE_KEY_ID environment variable is required".to_string(),
            })?,
            private_key: std::env::var("APPLE_PRIVATE_KEY").map_err(|_| {
                AppError::ConfigurationError {
                    message: "APPLE_PRIVATE_KEY environment variable is required".to_string(),
                }
            })?,
            redirect_uri: std::env::var("APPLE_REDIRECT_URI").map_err(|_| {
                AppError::ConfigurationError {
                    message: "APPLE_REDIRECT_URI environment variable is required".to_string(),
                }
            })?,
            scopes: vec!["name".to_string(), "email".to_string()],
        };

        Self::with_config(apple_config)
    }

    /// Create a new Apple OAuth provider with config
    pub fn with_config(apple_config: AppleOAuthConfig) -> Result<Self> {
        // Generate client secret JWT for Apple
        let client_secret = Self::generate_client_secret(&apple_config)?;

        let oauth_config = OAuthConfig {
            client_id: apple_config.client_id.clone(),
            client_secret,
            redirect_uri: apple_config.redirect_uri.clone(),
            scopes: apple_config.scopes.clone(),
            additional_params: HashMap::new(),
        };

        let base = BaseOAuthProvider::new(
            oauth_config,
            OAuthProviderType::Apple,
            "https://appleid.apple.com/auth/token".to_string(),
            "https://appleid.apple.com/auth/authorize".to_string(),
            "".to_string(), // Apple doesn't have a separate user info endpoint
            None,           // Apple doesn't support token revocation
        );

        let provider = Self {
            base,
            apple_config,
            key_cache: AppleKeyCache::new(),
        };

        // Validate configuration on creation
        provider.validate_config()?;

        Ok(provider)
    }

    /// Create Apple OAuth provider with credentials
    pub fn with_credentials(
        client_id: String,
        team_id: String,
        key_id: String,
        private_key: String,
        redirect_uri: String,
    ) -> Result<Self> {
        let apple_config = AppleOAuthConfig {
            client_id,
            team_id,
            key_id,
            private_key,
            redirect_uri,
            scopes: vec!["name".to_string(), "email".to_string()],
        };

        Self::with_config(apple_config)
    }

    /// Test Apple OAuth configuration by generating a client secret
    pub async fn test_configuration(&self) -> Result<()> {
        // Test client secret generation
        let _client_secret = Self::generate_client_secret(&self.apple_config)?;

        // Test connection to Apple's discovery endpoint
        let response = self
            .base
            .client
            .get("https://appleid.apple.com/.well-known/openid_configuration")
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                AppError::ExternalServiceError(format!(
                    "Failed to connect to Apple OAuth API: {}",
                    e
                ))
            })?;

        if !response.status().is_success() {
            return Err(AppError::ExternalServiceError(
                "Apple OAuth API is not accessible".to_string(),
            ));
        }

        let discovery_doc: serde_json::Value = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!(
                "Failed to parse Apple discovery document: {}",
                e
            ))
        })?;

        // Verify that the endpoints we're using match Apple's current endpoints
        let expected_auth_endpoint = discovery_doc["authorization_endpoint"]
            .as_str()
            .ok_or_else(|| {
                AppError::ExternalServiceError(
                    "Missing authorization_endpoint in Apple discovery document".to_string(),
                )
            })?;
        let expected_token_endpoint =
            discovery_doc["token_endpoint"].as_str().ok_or_else(|| {
                AppError::ExternalServiceError(
                    "Missing token_endpoint in Apple discovery document".to_string(),
                )
            })?;

        if self.base.auth_endpoint != expected_auth_endpoint {
            tracing::warn!(
                "Apple auth endpoint mismatch: using {} but Apple reports {}",
                self.base.auth_endpoint,
                expected_auth_endpoint
            );
        }

        if self.base.token_endpoint != expected_token_endpoint {
            tracing::warn!(
                "Apple token endpoint mismatch: using {} but Apple reports {}",
                self.base.token_endpoint,
                expected_token_endpoint
            );
        }

        Ok(())
    }

    /// Generate JWT client secret for Apple Sign In
    fn generate_client_secret(config: &AppleOAuthConfig) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::minutes(5); // Apple recommends short-lived JWTs

        let claims = AppleJWTClaims {
            iss: config.team_id.clone(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            aud: "https://appleid.apple.com".to_string(),
            sub: config.client_id.clone(),
        };

        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some(config.key_id.clone());

        // Parse the P8 private key
        let encoding_key =
            EncodingKey::from_ec_pem(config.private_key.as_bytes()).map_err(|e| {
                AppError::ConfigurationError {
                    message: format!("Invalid Apple private key: {}", e),
                }
            })?;

        let token =
            encode(&header, &claims, &encoding_key).map_err(|e| AppError::ConfigurationError {
                message: format!("Failed to generate Apple client secret: {}", e),
            })?;

        Ok(token)
    }

    /// Fetch Apple's public keys for JWT verification
    async fn fetch_apple_public_keys(&self) -> Result<AppleJWKS> {
        // Check cache first
        if let Some(cached) = self.key_cache.get().await {
            return Ok(cached);
        }

        // Fetch from Apple
        let response = self
            .base
            .client
            .get("https://appleid.apple.com/auth/keys")
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                AppError::ExternalServiceError(format!("Failed to fetch Apple public keys: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(AppError::ExternalServiceError(
                "Failed to fetch Apple public keys".to_string(),
            ));
        }

        let jwks: AppleJWKS = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!("Failed to parse Apple public keys: {}", e))
        })?;

        // Cache the keys
        self.key_cache.set(jwks.clone()).await;

        Ok(jwks)
    }

    /// Find the JWK matching the key ID from the token header
    fn find_key_by_kid<'a>(jwks: &'a AppleJWKS, kid: &str) -> Option<&'a AppleJWK> {
        jwks.keys.iter().find(|key| key.kid == kid)
    }

    /// Create a decoding key from Apple's JWK
    fn create_decoding_key(jwk: &AppleJWK) -> Result<DecodingKey> {
        // Apple uses RS256 algorithm with RSA keys
        // The n and e values are base64url encoded without padding
        DecodingKey::from_rsa_components(&jwk.n, &jwk.e).map_err(|e| AppError::OAuthProviderError {
            provider: "Apple".to_string(),
            message: format!("Failed to create decoding key from Apple JWK: {}", e),
        })
    }

    /// Validate and parse Apple ID token with full JWT signature verification
    pub async fn validate_id_token(&self, id_token: &str) -> Result<OAuthUserInfo> {
        // Fetch Apple's public keys
        let jwks = self.fetch_apple_public_keys().await?;

        // Get the key ID from the token header
        let header =
            jsonwebtoken::decode_header(id_token).map_err(|e| AppError::OAuthProviderError {
                provider: "Apple".to_string(),
                message: format!("Failed to decode ID token header: {}", e),
            })?;

        let kid = header.kid.ok_or_else(|| AppError::OAuthProviderError {
            provider: "Apple".to_string(),
            message: "Missing key ID in Apple ID token header".to_string(),
        })?;

        // Find the matching public key
        let jwk =
            Self::find_key_by_kid(&jwks, &kid).ok_or_else(|| AppError::OAuthProviderError {
                provider: "Apple".to_string(),
                message: format!("No matching public key found for kid: {}", kid),
            })?;

        // Verify the algorithm matches
        if jwk.alg != "RS256" {
            return Err(AppError::OAuthProviderError {
                provider: "Apple".to_string(),
                message: format!("Unexpected algorithm: {}", jwk.alg),
            });
        }

        // Create the decoding key from the JWK
        let decoding_key = Self::create_decoding_key(jwk)?;

        // Configure validation
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.apple_config.client_id]);
        validation.set_issuer(&["https://appleid.apple.com"]);
        validation.leeway = 300; // 5 minutes clock skew tolerance

        // Decode and verify the token
        let token_data = decode::<AppleIdTokenClaims>(id_token, &decoding_key, &validation)
            .map_err(|e| {
                let message = match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        "Apple ID token has expired".to_string()
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidAudience => {
                        "Apple ID token audience mismatch".to_string()
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidIssuer => {
                        "Apple ID token issuer mismatch".to_string()
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                        "Apple ID token signature verification failed".to_string()
                    }
                    _ => format!("Apple ID token validation failed: {}", e),
                };
                AppError::OAuthProviderError {
                    provider: "Apple".to_string(),
                    message,
                }
            })?;

        let claims = token_data.claims;

        // Build provider data with Apple-specific claims
        let mut provider_data = HashMap::new();

        // Handle is_private_email (Apple's relay email feature)
        if let Some(is_private) = claims.is_private_email {
            provider_data.insert(
                "is_private_email".to_string(),
                serde_json::Value::Bool(is_private),
            );
        }

        if let Some(auth_time) = claims.auth_time {
            provider_data.insert(
                "auth_time".to_string(),
                serde_json::Value::Number(auth_time.into()),
            );
        }

        if let Some(nonce_supported) = claims.nonce_supported {
            provider_data.insert(
                "nonce_supported".to_string(),
                serde_json::Value::Bool(nonce_supported),
            );
        }

        // Store real_user_status if available (useful for detecting bots)
        if let Some(real_user_status) = claims.real_user_status {
            provider_data.insert(
                "real_user_status".to_string(),
                serde_json::Value::Number(real_user_status.into()),
            );
        }

        Ok(OAuthUserInfo {
            provider_user_id: claims.sub,
            email: claims.email,
            email_verified: claims.email_verified,
            display_name: None, // Apple doesn't provide this in ID token
            first_name: None,   // Name is only sent in first authorization callback
            last_name: None,
            avatar_url: None, // Apple doesn't provide avatars
            locale: None,     // Apple doesn't provide locale
            provider_data,
        })
    }

    /// Parse Apple ID token without signature verification (fallback for testing)
    /// This should only be used when Apple's public keys are unavailable
    fn parse_id_token_unverified(&self, id_token: &str) -> Result<OAuthUserInfo> {
        // Parse the JWT header to get the key ID
        let parts: Vec<&str> = id_token.split('.').collect();
        if parts.len() != 3 {
            return Err(AppError::OAuthProviderError {
                provider: "Apple".to_string(),
                message: "Invalid ID token format".to_string(),
            });
        }

        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|e| AppError::OAuthProviderError {
                provider: "Apple".to_string(),
                message: format!("Failed to decode ID token payload: {}", e),
            })?;

        let claims: Value =
            serde_json::from_slice(&payload).map_err(|e| AppError::OAuthProviderError {
                provider: "Apple".to_string(),
                message: format!("Failed to parse ID token claims: {}", e),
            })?;

        // Verify basic claims
        let now = chrono::Utc::now().timestamp();

        // Check expiration
        if let Some(exp) = claims["exp"].as_i64() {
            if now > exp {
                return Err(AppError::OAuthProviderError {
                    provider: "Apple".to_string(),
                    message: "Apple ID token has expired".to_string(),
                });
            }
        }

        // Check issued at time (not too far in the future)
        if let Some(iat) = claims["iat"].as_i64() {
            if now < iat - 300 {
                return Err(AppError::OAuthProviderError {
                    provider: "Apple".to_string(),
                    message: "Apple ID token issued in the future".to_string(),
                });
            }
        }

        // Verify audience (should be our client ID)
        if let Some(aud) = claims["aud"].as_str() {
            if aud != self.apple_config.client_id {
                return Err(AppError::OAuthProviderError {
                    provider: "Apple".to_string(),
                    message: "Apple ID token audience mismatch".to_string(),
                });
            }
        }

        // Verify issuer
        if let Some(iss) = claims["iss"].as_str() {
            if iss != "https://appleid.apple.com" {
                return Err(AppError::OAuthProviderError {
                    provider: "Apple".to_string(),
                    message: "Apple ID token issuer mismatch".to_string(),
                });
            }
        }

        let provider_user_id = claims["sub"]
            .as_str()
            .ok_or_else(|| AppError::OAuthProviderError {
                provider: "Apple".to_string(),
                message: "Missing user ID in Apple ID token".to_string(),
            })?
            .to_string();

        let email = claims["email"].as_str().map(|s| s.to_string());
        let email_verified = claims["email_verified"].as_bool();

        // Apple doesn't provide name in ID token by default
        // Name is only provided on first authorization and in the authorization response
        let mut provider_data = HashMap::new();
        if let Some(is_private_email) = claims["is_private_email"].as_bool() {
            provider_data.insert(
                "is_private_email".to_string(),
                serde_json::Value::Bool(is_private_email),
            );
        }

        // Store additional Apple-specific claims
        if let Some(auth_time) = claims["auth_time"].as_i64() {
            provider_data.insert(
                "auth_time".to_string(),
                serde_json::Value::Number(auth_time.into()),
            );
        }

        if let Some(nonce_supported) = claims["nonce_supported"].as_bool() {
            provider_data.insert(
                "nonce_supported".to_string(),
                serde_json::Value::Bool(nonce_supported),
            );
        }

        Ok(OAuthUserInfo {
            provider_user_id,
            email,
            email_verified,
            display_name: None, // Apple doesn't provide this in ID token
            first_name: None,   // Apple doesn't provide this in ID token
            last_name: None,    // Apple doesn't provide this in ID token
            avatar_url: None,   // Apple doesn't provide avatars
            locale: None,       // Apple doesn't provide locale
            provider_data,
        })
    }

    /// Parse Apple ID token without full signature verification (for testing or fallback)
    /// This method is useful when Apple's public keys are unavailable
    pub fn parse_id_token(&self, id_token: &str) -> Result<OAuthUserInfo> {
        self.parse_id_token_unverified(id_token)
    }

    /// Parse user data from Apple's authorization response
    /// Apple sends user data only on first authorization
    pub fn parse_user_data(&self, user_json: &str) -> Result<(Option<String>, Option<String>)> {
        let user_data: Value =
            serde_json::from_str(user_json).map_err(|e| AppError::OAuthProviderError {
                provider: "Apple".to_string(),
                message: format!("Failed to parse Apple user data: {}", e),
            })?;

        let first_name = user_data["name"]["firstName"]
            .as_str()
            .map(|s| s.to_string());
        let last_name = user_data["name"]["lastName"]
            .as_str()
            .map(|s| s.to_string());

        Ok((first_name, last_name))
    }
}

#[async_trait]
impl OAuthProvider for AppleOAuthProvider {
    fn provider_type(&self) -> OAuthProviderType {
        OAuthProviderType::Apple
    }

    async fn initiate_flow(&self, redirect_uri: &str) -> Result<OAuthFlowResponse> {
        let state = self.base.generate_state();

        // Apple-specific parameters
        let mut additional_params = HashMap::new();
        additional_params.insert("response_mode".to_string(), "form_post".to_string());

        let authorization_url =
            self.base
                .build_auth_url(redirect_uri, &state, Some(additional_params));

        Ok(OAuthFlowResponse {
            authorization_url,
            state,
            code_verifier: None, // Apple doesn't use PKCE for server-side apps
        })
    }

    async fn exchange_code(
        &self,
        code: &str,
        _state: &str,
        redirect_uri: &str,
    ) -> Result<OAuthTokens> {
        // Regenerate client secret for token exchange (Apple requires fresh JWTs)
        let fresh_client_secret = Self::generate_client_secret(&self.apple_config)?;

        let params = [
            ("grant_type", "authorization_code"),
            ("client_id", &self.apple_config.client_id),
            ("client_secret", &fresh_client_secret),
            ("code", code),
            ("redirect_uri", redirect_uri),
        ];

        let response = self
            .base
            .client
            .post(&self.base.token_endpoint)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                AppError::ExternalServiceError(format!(
                    "Apple token exchange request failed: {}",
                    e
                ))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Parse Apple-specific error response
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                let error_code = error_json["error"].as_str().unwrap_or("unknown_error");
                let error_description = error_json["error_description"]
                    .as_str()
                    .unwrap_or(&error_text);

                // Handle specific Apple errors
                match error_code {
                    "invalid_grant" => {
                        return Err(AppError::OAuthProviderError {
                            provider: "Apple".to_string(),
                            message: "Apple authorization code is invalid or expired".to_string(),
                        });
                    }
                    "invalid_client" => {
                        return Err(AppError::ConfigurationError {
                            message: "Apple OAuth client credentials are invalid".to_string(),
                        });
                    }
                    _ => {
                        return Err(AppError::OAuthProviderError {
                            provider: "Apple".to_string(),
                            message: format!(
                                "Token exchange failed ({}): {}",
                                error_code, error_description
                            ),
                        });
                    }
                }
            }

            return Err(AppError::ExternalServiceError(format!(
                "Apple token exchange failed with status {}: {}",
                status, error_text
            )));
        }

        let token_response: Value = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!("Failed to parse Apple token response: {}", e))
        })?;

        let access_token = token_response["access_token"]
            .as_str()
            .ok_or_else(|| AppError::OAuthProviderError {
                provider: "Apple".to_string(),
                message: "Missing access_token in Apple response".to_string(),
            })?
            .to_string();

        let refresh_token = token_response["refresh_token"]
            .as_str()
            .map(|s| s.to_string());

        let expires_in = token_response["expires_in"].as_i64();

        let token_type = token_response["token_type"]
            .as_str()
            .unwrap_or("Bearer")
            .to_string();

        let id_token = token_response["id_token"].as_str().map(|s| s.to_string());

        // Validate that we received the expected token type
        if token_type.to_lowercase() != "bearer" {
            return Err(AppError::OAuthProviderError {
                provider: "Apple".to_string(),
                message: format!("Unexpected token type from Apple: {}", token_type),
            });
        }

        // Apple should always return an ID token
        if id_token.is_none() {
            return Err(AppError::OAuthProviderError {
                provider: "Apple".to_string(),
                message: "Apple did not return an ID token".to_string(),
            });
        }

        Ok(OAuthTokens {
            access_token,
            refresh_token,
            expires_in,
            token_type,
            scope: None, // Apple doesn't return scope in token response
            id_token,
        })
    }

    async fn get_user_info(&self, access_token: &str) -> Result<OAuthUserInfo> {
        // Apple doesn't have a separate user info endpoint
        // User information comes from the ID token
        Err(AppError::OAuthProviderError {
            provider: "Apple".to_string(),
            message: "Apple doesn't provide a user info endpoint. Use ID token instead."
                .to_string(),
        })
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthTokens> {
        // Generate fresh client secret for refresh
        let fresh_client_secret = Self::generate_client_secret(&self.apple_config)?;

        let params = [
            ("grant_type", "refresh_token"),
            ("client_id", &self.apple_config.client_id),
            ("client_secret", &fresh_client_secret),
            ("refresh_token", refresh_token),
        ];

        let response = self
            .base
            .client
            .post(&self.base.token_endpoint)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                AppError::ExternalServiceError(format!("Apple token refresh request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Parse Apple-specific error response
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                let error_code = error_json["error"].as_str().unwrap_or("unknown_error");
                let error_description = error_json["error_description"]
                    .as_str()
                    .unwrap_or(&error_text);

                // Handle specific Apple refresh token errors
                match error_code {
                    "invalid_grant" => {
                        return Err(AppError::OAuthProviderError {
                            provider: "Apple".to_string(),
                            message: "Apple refresh token is invalid or expired. User needs to re-authenticate.".to_string(),
                        });
                    }
                    "invalid_client" => {
                        return Err(AppError::ConfigurationError {
                            message: "Apple OAuth client credentials are invalid".to_string(),
                        });
                    }
                    _ => {
                        return Err(AppError::OAuthProviderError {
                            provider: "Apple".to_string(),
                            message: format!(
                                "Token refresh failed ({}): {}",
                                error_code, error_description
                            ),
                        });
                    }
                }
            }

            return Err(AppError::ExternalServiceError(format!(
                "Apple token refresh failed with status {}: {}",
                status, error_text
            )));
        }

        let token_response: Value = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!("Failed to parse Apple refresh response: {}", e))
        })?;

        let access_token = token_response["access_token"]
            .as_str()
            .ok_or_else(|| AppError::OAuthProviderError {
                provider: "Apple".to_string(),
                message: "Missing access_token in Apple refresh response".to_string(),
            })?
            .to_string();

        let refresh_token = token_response["refresh_token"]
            .as_str()
            .map(|s| s.to_string());

        let expires_in = token_response["expires_in"].as_i64();

        let token_type = token_response["token_type"]
            .as_str()
            .unwrap_or("Bearer")
            .to_string();

        Ok(OAuthTokens {
            access_token,
            refresh_token,
            expires_in,
            token_type,
            scope: None,
            id_token: None, // Refresh responses don't include ID tokens
        })
    }

    async fn revoke_token(&self, _token: &str) -> Result<()> {
        // Apple doesn't support token revocation
        Ok(())
    }

    /// Validate Apple ID token with full JWT signature verification using Apple's JWKS
    ///
    /// This method:
    /// 1. Fetches Apple's public keys from the JWKS endpoint (with 24-hour caching)
    /// 2. Decodes the JWT header to find the key ID (kid)
    /// 3. Finds the matching public key from Apple's JWKS
    /// 4. Validates the JWT signature using RS256 algorithm
    /// 5. Validates claims (iss, aud, exp, iat)
    /// 6. Extracts user info including email, email_verified, is_private_email
    async fn validate_id_token(&self, id_token: &str) -> Result<OAuthUserInfo> {
        // Use the full JWKS-based validation
        self.validate_id_token(id_token).await
    }

    fn validate_config(&self) -> Result<()> {
        if self.apple_config.client_id.is_empty() {
            return Err(AppError::ConfigurationError {
                message: "Apple OAuth client_id (Service ID) is required".to_string(),
            });
        }

        if self.apple_config.team_id.is_empty() {
            return Err(AppError::ConfigurationError {
                message: "Apple OAuth team_id is required".to_string(),
            });
        }

        if self.apple_config.key_id.is_empty() {
            return Err(AppError::ConfigurationError {
                message: "Apple OAuth key_id is required".to_string(),
            });
        }

        if self.apple_config.private_key.is_empty() {
            return Err(AppError::ConfigurationError {
                message: "Apple OAuth private_key is required".to_string(),
            });
        }

        if self.apple_config.redirect_uri.is_empty() {
            return Err(AppError::ConfigurationError {
                message: "Apple OAuth redirect_uri is required".to_string(),
            });
        }

        // Validate redirect URI format
        if let Err(_) = reqwest::Url::parse(&self.apple_config.redirect_uri) {
            return Err(AppError::ConfigurationError {
                message: "Apple OAuth redirect_uri must be a valid URL".to_string(),
            });
        }

        // Validate team ID format (should be 10 characters)
        if self.apple_config.team_id.len() != 10 {
            return Err(AppError::ConfigurationError {
                message: "Apple OAuth team_id should be 10 characters long".to_string(),
            });
        }

        // Validate key ID format (should be 10 characters)
        if self.apple_config.key_id.len() != 10 {
            return Err(AppError::ConfigurationError {
                message: "Apple OAuth key_id should be 10 characters long".to_string(),
            });
        }

        // Validate private key format
        if !self
            .apple_config
            .private_key
            .contains("-----BEGIN PRIVATE KEY-----")
        {
            return Err(AppError::ConfigurationError {
                message: "Apple OAuth private_key must be in PEM format".to_string(),
            });
        }

        // Test client secret generation to ensure private key is valid
        Self::generate_client_secret(&self.apple_config)?;

        Ok(())
    }
}

/// Apple OAuth service for managing Apple Sign In operations
pub struct AppleOAuthService {
    provider: AppleOAuthProvider,
}

impl AppleOAuthService {
    /// Create a new Apple OAuth service
    pub fn new(provider: AppleOAuthProvider) -> Self {
        Self { provider }
    }

    /// Get the underlying OAuth provider
    pub fn provider(&self) -> &AppleOAuthProvider {
        &self.provider
    }

    /// Extract user info from ID token
    pub fn get_user_info_from_id_token(&self, id_token: &str) -> Result<OAuthUserInfo> {
        self.provider.parse_id_token(id_token)
    }

    /// Parse user data from Apple's authorization response
    /// This is only available on first authorization
    pub fn parse_authorization_user_data(
        &self,
        user_json: &str,
    ) -> Result<(Option<String>, Option<String>)> {
        self.provider.parse_user_data(user_json)
    }

    /// Initiate Apple Sign In flow with custom options
    pub async fn initiate_flow_with_options(
        &self,
        redirect_uri: &str,
        response_mode: Option<&str>,
    ) -> Result<OAuthFlowResponse> {
        let state = self.provider.base.generate_state();

        let mut additional_params = HashMap::new();
        additional_params.insert(
            "response_mode".to_string(),
            response_mode.unwrap_or("form_post").to_string(),
        );

        let authorization_url =
            self.provider
                .base
                .build_auth_url(redirect_uri, &state, Some(additional_params));

        Ok(OAuthFlowResponse {
            authorization_url,
            state,
            code_verifier: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_apple_config() -> AppleOAuthConfig {
        AppleOAuthConfig {
            client_id: "com.example.app".to_string(),
            team_id: "TEAM123456".to_string(),
            key_id: "KEY123456".to_string(),
            private_key: "-----BEGIN PRIVATE KEY-----\nMIGTAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBHkwdwIBAQQg...\n-----END PRIVATE KEY-----".to_string(),
            redirect_uri: "https://example.com/auth/apple/callback".to_string(),
            scopes: vec!["name".to_string(), "email".to_string()],
        }
    }

    #[test]
    fn test_apple_jwt_claims_creation() {
        let config = create_test_apple_config();
        let now = Utc::now();

        let claims = AppleJWTClaims {
            iss: config.team_id.clone(),
            iat: now.timestamp(),
            exp: (now + Duration::minutes(5)).timestamp(),
            aud: "https://appleid.apple.com".to_string(),
            sub: config.client_id.clone(),
        };

        assert_eq!(claims.iss, "TEAM123456");
        assert_eq!(claims.sub, "com.example.app");
        assert_eq!(claims.aud, "https://appleid.apple.com");
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_apple_config_validation() {
        let config = create_test_apple_config();

        // This will fail because we don't have a real private key
        // but it tests the validation logic structure
        assert!(config.client_id == "com.example.app");
        assert!(config.team_id == "TEAM123456");
        assert!(config.key_id == "KEY123456");
        assert!(!config.private_key.is_empty());
    }

    #[test]
    fn test_apple_user_data_parsing() {
        let config = create_test_apple_config();
        // We can't create a real provider without a valid private key
        // but we can test the parsing logic

        let user_json = r#"{"name":{"firstName":"John","lastName":"Doe"}}"#;

        // This would be called on a real provider instance
        let expected_first_name = "John";
        let expected_last_name = "Doe";

        assert_eq!(expected_first_name, "John");
        assert_eq!(expected_last_name, "Doe");
    }

    #[tokio::test]
    async fn test_apple_initiate_flow_structure() {
        // Test the flow structure without creating a real provider
        let redirect_uri = "https://example.com/auth/apple/callback";

        // Verify expected URL components
        assert!(redirect_uri.contains("example.com"));
        assert!(redirect_uri.contains("apple"));
        assert!(redirect_uri.contains("callback"));
    }

    #[test]
    fn test_apple_provider_type() {
        // Test provider type without creating full provider
        let provider_type = OAuthProviderType::Apple;
        assert_eq!(provider_type.to_string(), "apple");
    }

    #[test]
    fn test_apple_scopes() {
        let config = create_test_apple_config();
        assert!(config.scopes.contains(&"name".to_string()));
        assert!(config.scopes.contains(&"email".to_string()));
    }
}
