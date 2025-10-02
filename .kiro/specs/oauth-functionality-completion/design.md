# Design Document

## Overview

This design completes the OAuth functionality by replacing placeholder implementations and re-enabling disabled methods in the existing OAuth infrastructure. The system already has the basic OAuth framework, database schema, and handlers in place, but many core methods return placeholder responses or are temporarily disabled.

The design focuses on:
1. **Re-enabling OAuth Service Methods** - Replacing "temporarily disabled" methods with real implementations
2. **Real OAuth Provider Integration** - Implementing actual API calls to Google, Apple, and GitHub
3. **Complete Token Management** - Proper encryption, storage, and refresh of OAuth tokens
4. **Database Operations** - Full CRUD operations for OAuth accounts with proper error handling

## Architecture

### Current OAuth Infrastructure

The existing system has these components already in place:

```mermaid
graph TB
    subgraph "Existing Infrastructure"
        Handlers[OAuth Handlers<br/>oauth.rs]
        AuthService[Auth Service<br/>auth.rs]
        Models[OAuth Models<br/>oauth.rs]
        Database[(PostgreSQL<br/>oauth_accounts table)]
        TokenVault[Token Vault<br/>AES-GCM encryption)]
    end
    
    subgraph "OAuth Providers (Placeholder)"
        Google[Google OAuth<br/>Disabled]
        Apple[Apple OAuth<br/>Disabled]  
        GitHub[GitHub OAuth<br/>Disabled]
    end
    
    Handlers --> AuthService
    AuthService --> Models
    AuthService --> Database
    AuthService --> TokenVault
    AuthService -.-> Google
    AuthService -.-> Apple
    AuthService -.-> GitHub
    
    style Google fill:#ffcccc
    style Apple fill:#ffcccc
    style GitHub fill:#ffcccc
```

### Target OAuth Architecture

After completion, the OAuth system will have fully functional providers:

```mermaid
graph TB
    subgraph "Completed OAuth System"
        Handlers[OAuth Handlers<br/>Fully Functional]
        AuthService[Auth Service<br/>All Methods Enabled]
        Models[OAuth Models<br/>Complete]
        Database[(PostgreSQL<br/>Full CRUD Operations)]
        TokenVault[Token Vault<br/>Encryption + Refresh)]
    end
    
    subgraph "OAuth Providers (Active)"
        Google[Google OAuth2<br/>Real API Integration]
        Apple[Apple Sign In<br/>JWT + Real API]  
        GitHub[GitHub OAuth<br/>Real API Integration]
    end
    
    subgraph "External APIs"
        GoogleAPI[Google OAuth2 API]
        AppleAPI[Apple ID API]
        GitHubAPI[GitHub OAuth API]
    end
    
    Handlers --> AuthService
    AuthService --> Models
    AuthService --> Database
    AuthService --> TokenVault
    AuthService --> Google
    AuthService --> Apple
    AuthService --> GitHub
    
    Google --> GoogleAPI
    Apple --> AppleAPI
    GitHub --> GitHubAPI
    
    style Google fill:#ccffcc
    style Apple fill:#ccffcc
    style GitHub fill:#ccffcc
```

## Components and Interfaces

### 1. OAuth Service Method Completion

#### Currently Disabled Methods to Re-enable

```rust
impl AuthService {
    // Currently returns "temporarily disabled" - needs real implementation
    pub async fn initiate_oauth_flow(&self, provider: OAuthProviderType, redirect_uri: String) -> Result<OAuthFlowResponse>;
    
    // Currently returns "temporarily disabled" - needs real implementation  
    pub async fn complete_oauth_flow(&self, provider: OAuthProviderType, code: String, state: String, redirect_uri: String) -> Result<TokenPair>;
    
    // Currently returns "temporarily disabled" - needs real implementation
    pub async fn find_user_by_oauth_account(&self, provider: &OAuthProviderType, provider_user_id: &str) -> Result<Option<User>>;
    
    // Currently returns "temporarily disabled" - needs real implementation
    pub async fn create_user_with_oauth_account(&self, provider: OAuthProviderType, user_info: OAuthUserInfo, tokens: OAuthTokens) -> Result<User>;
    
    // Currently returns "temporarily disabled" - needs real implementation
    pub async fn link_oauth_account_to_user(&self, user_id: Uuid, provider: OAuthProviderType, user_info: OAuthUserInfo, tokens: OAuthTokens) -> Result<()>;
    
    // Currently returns "temporarily disabled" - needs real implementation
    pub async fn unlink_oauth_account(&self, user_id: Uuid, provider_type: OAuthProviderType) -> Result<()>;
    
    // Currently returns "temporarily disabled" - needs real implementation
    pub async fn refresh_oauth_tokens(&self, user_id: Uuid, provider_type: OAuthProviderType) -> Result<()>;
    
    // Currently returns "temporarily disabled" - needs real implementation
    pub async fn load_oauth_accounts(&self, user_id: Uuid) -> Result<Vec<OAuthAccount>>;
}
```

#### Implementation Strategy for Each Method

**1. initiate_oauth_flow**
```rust
pub async fn initiate_oauth_flow(&self, provider: OAuthProviderType, redirect_uri: String) -> Result<OAuthFlowResponse> {
    // 1. Get the OAuth provider instance
    let oauth_provider = self.oauth_providers.get(&provider)
        .ok_or_else(|| AppError::InvalidFieldValue { 
            field: "provider".to_string(),
            message: format!("OAuth provider {} not configured", provider) 
        })?;
    
    // 2. Generate secure state parameter
    let state = self.oauth_state_manager.generate_state().await?;
    
    // 3. Get authorization URL from provider
    let auth_url = oauth_provider.get_authorization_url(&redirect_uri, &state).await?;
    
    // 4. Store state for validation
    self.oauth_state_manager.store_state(&state, &provider, &redirect_uri).await?;
    
    Ok(OAuthFlowResponse {
        authorization_url: auth_url,
        state,
        code_verifier: None, // PKCE not implemented yet
    })
}
```

**2. complete_oauth_flow**
```rust
pub async fn complete_oauth_flow(&self, provider: OAuthProviderType, code: String, state: String, redirect_uri: String) -> Result<TokenPair> {
    // 1. Validate state parameter
    self.oauth_state_manager.validate_state(&state, &provider, &redirect_uri).await?;
    
    // 2. Get OAuth provider
    let oauth_provider = self.oauth_providers.get(&provider)
        .ok_or_else(|| AppError::InvalidFieldValue { 
            field: "provider".to_string(),
            message: format!("OAuth provider {} not configured", provider) 
        })?;
    
    // 3. Exchange code for tokens
    let tokens = oauth_provider.exchange_code(&code, &redirect_uri).await?;
    
    // 4. Get user info from provider
    let user_info = oauth_provider.get_user_info(&tokens.access_token).await?;
    
    // 5. Find existing user or create new one
    let user = match self.find_user_by_oauth_account(&provider, &user_info.provider_user_id).await? {
        Some(existing_user) => {
            // Update tokens for existing OAuth account
            self.update_oauth_tokens(existing_user.id, provider, &tokens).await?;
            existing_user
        },
        None => {
            // Check if user exists by email
            if let Some(email) = &user_info.email {
                if let Some(existing_user) = self.find_user_by_email(email).await? {
                    // Link OAuth account to existing user
                    self.link_oauth_account_to_user(existing_user.id, provider, user_info, tokens).await?;
                    existing_user
                } else {
                    // Create new user with OAuth account
                    self.create_user_with_oauth_account(provider, user_info, tokens).await?
                }
            } else {
                // Create new user without email
                self.create_user_with_oauth_account(provider, user_info, tokens).await?
            }
        }
    };
    
    // 6. Generate JWT tokens
    self.generate_token_pair(user.id, &user.email).await
}
```

### 2. OAuth Provider Implementation

#### Real OAuth Provider Integration

**Google OAuth Provider**
```rust
impl GoogleOAuthProvider {
    pub async fn get_authorization_url(&self, redirect_uri: &str, state: &str) -> Result<String> {
        let mut url = Url::parse("https://accounts.google.com/o/oauth2/v2/auth")?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", "openid email profile")
            .append_pair("state", state)
            .append_pair("access_type", "offline") // For refresh tokens
            .append_pair("prompt", "consent");
        
        Ok(url.to_string())
    }
    
    pub async fn exchange_code(&self, code: &str, redirect_uri: &str) -> Result<OAuthTokens> {
        let client = reqwest::Client::new();
        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri),
        ];
        
        let response = client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(AppError::ExternalServiceError(format!("Google OAuth error: {}", error_text)));
        }
        
        let token_response: GoogleTokenResponse = response.json().await?;
        
        Ok(OAuthTokens {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            expires_in: Some(token_response.expires_in),
            token_type: token_response.token_type,
            scope: token_response.scope,
            id_token: token_response.id_token,
        })
    }
    
    pub async fn get_user_info(&self, access_token: &str) -> Result<OAuthUserInfo> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .bearer_auth(access_token)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(AppError::ExternalServiceError(format!("Google user info error: {}", error_text)));
        }
        
        let user_response: GoogleUserResponse = response.json().await?;
        
        Ok(OAuthUserInfo {
            provider_user_id: user_response.id,
            email: Some(user_response.email),
            email_verified: Some(user_response.verified_email),
            display_name: user_response.name,
            avatar_url: user_response.picture,
        })
    }
}
```

**Apple OAuth Provider**
```rust
impl AppleOAuthProvider {
    pub async fn get_authorization_url(&self, redirect_uri: &str, state: &str) -> Result<String> {
        let mut url = Url::parse("https://appleid.apple.com/auth/authorize")?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", "name email")
            .append_pair("state", state)
            .append_pair("response_mode", "form_post");
        
        Ok(url.to_string())
    }
    
    pub async fn exchange_code(&self, code: &str, redirect_uri: &str) -> Result<OAuthTokens> {
        // Apple requires JWT client secret
        let client_secret = self.create_client_secret().await?;
        
        let client = reqwest::Client::new();
        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri),
        ];
        
        let response = client
            .post("https://appleid.apple.com/auth/token")
            .form(&params)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(AppError::ExternalServiceError(format!("Apple OAuth error: {}", error_text)));
        }
        
        let token_response: AppleTokenResponse = response.json().await?;
        
        Ok(OAuthTokens {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            expires_in: Some(token_response.expires_in),
            token_type: token_response.token_type,
            scope: None,
            id_token: token_response.id_token,
        })
    }
    
    async fn create_client_secret(&self) -> Result<String> {
        // Create JWT for Apple client secret
        let now = Utc::now().timestamp();
        let claims = AppleClientSecretClaims {
            iss: self.team_id.clone(),
            iat: now,
            exp: now + 3600, // 1 hour
            aud: "https://appleid.apple.com".to_string(),
            sub: self.client_id.clone(),
        };
        
        let header = Header {
            alg: Algorithm::ES256,
            kid: Some(self.key_id.clone()),
            ..Default::default()
        };
        
        let encoding_key = EncodingKey::from_ec_pem(self.private_key.as_bytes())?;
        let token = encode(&header, &claims, &encoding_key)?;
        
        Ok(token)
    }
}
```

**GitHub OAuth Provider**
```rust
impl GitHubOAuthProvider {
    pub async fn get_authorization_url(&self, redirect_uri: &str, state: &str) -> Result<String> {
        let mut url = Url::parse("https://github.com/login/oauth/authorize")?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", redirect_uri)
            .append_pair("scope", "user:email")
            .append_pair("state", state);
        
        Ok(url.to_string())
    }
    
    pub async fn exchange_code(&self, code: &str, redirect_uri: &str) -> Result<OAuthTokens> {
        let client = reqwest::Client::new();
        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("code", code),
            ("redirect_uri", redirect_uri),
        ];
        
        let response = client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(AppError::ExternalServiceError(format!("GitHub OAuth error: {}", error_text)));
        }
        
        let token_response: GitHubTokenResponse = response.json().await?;
        
        Ok(OAuthTokens {
            access_token: token_response.access_token,
            refresh_token: None, // GitHub doesn't provide refresh tokens
            expires_in: None,
            token_type: token_response.token_type,
            scope: token_response.scope,
            id_token: None,
        })
    }
    
    pub async fn get_user_info(&self, access_token: &str) -> Result<OAuthUserInfo> {
        let client = reqwest::Client::new();
        
        // Get user profile
        let user_response = client
            .get("https://api.github.com/user")
            .bearer_auth(access_token)
            .header("User-Agent", "no-drake-in-the-house")
            .send()
            .await?;
        
        if !user_response.status().is_success() {
            let error_text = user_response.text().await?;
            return Err(AppError::ExternalServiceError(format!("GitHub user info error: {}", error_text)));
        }
        
        let user_data: GitHubUserResponse = user_response.json().await?;
        
        // Get user emails (GitHub may not include email in profile)
        let emails_response = client
            .get("https://api.github.com/user/emails")
            .bearer_auth(access_token)
            .header("User-Agent", "no-drake-in-the-house")
            .send()
            .await?;
        
        let primary_email = if emails_response.status().is_success() {
            let emails: Vec<GitHubEmailResponse> = emails_response.json().await?;
            emails.into_iter()
                .find(|e| e.primary && e.verified)
                .map(|e| e.email)
        } else {
            user_data.email
        };
        
        Ok(OAuthUserInfo {
            provider_user_id: user_data.id.to_string(),
            email: primary_email,
            email_verified: Some(true), // GitHub emails are verified
            display_name: user_data.name.or(Some(user_data.login)),
            avatar_url: Some(user_data.avatar_url),
        })
    }
}
```

### 3. Database Operations Implementation

#### OAuth Account CRUD Operations

```rust
impl AuthService {
    pub async fn find_user_by_oauth_account(&self, provider: &OAuthProviderType, provider_user_id: &str) -> Result<Option<User>> {
        let user_row = sqlx::query!(
            r#"
            SELECT u.id, u.email, u.password_hash, u.email_verified, u.totp_secret, u.totp_enabled,
                   u.created_at, u.updated_at, u.last_login, u.settings
            FROM users u
            JOIN oauth_accounts oa ON u.id = oa.user_id
            WHERE oa.provider = $1 AND oa.provider_user_id = $2
            "#,
            provider.to_string(),
            provider_user_id
        )
        .fetch_optional(&self.db_pool)
        .await?;
        
        if let Some(row) = user_row {
            let mut user = User {
                id: row.id,
                email: row.email,
                password_hash: row.password_hash,
                email_verified: row.email_verified.unwrap_or(false),
                totp_secret: row.totp_secret,
                totp_enabled: row.totp_enabled.unwrap_or(false),
                oauth_accounts: Vec::new(),
                created_at: row.created_at.unwrap_or(Utc::now()),
                updated_at: row.updated_at.unwrap_or(Utc::now()),
                last_login: row.last_login,
                settings: serde_json::from_value(row.settings.unwrap_or(serde_json::json!({})))
                    .unwrap_or_default(),
            };
            
            // Load OAuth accounts
            user.oauth_accounts = self.load_oauth_accounts(user.id).await?;
            
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    
    pub async fn create_user_with_oauth_account(&self, provider: OAuthProviderType, user_info: OAuthUserInfo, tokens: OAuthTokens) -> Result<User> {
        let mut tx = self.db_pool.begin().await?;
        
        // Create user
        let user_id = Uuid::new_v4();
        let email = user_info.email.clone().unwrap_or_else(|| format!("{}@{}.oauth", user_info.provider_user_id, provider));
        let now = Utc::now();
        
        sqlx::query!(
            r#"
            INSERT INTO users (id, email, email_verified, created_at, updated_at, settings)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            user_id,
            email,
            user_info.email_verified.unwrap_or(false),
            now,
            now,
            serde_json::json!({})
        )
        .execute(&mut *tx)
        .await?;
        
        // Create OAuth account
        self.create_oauth_account_in_tx(&mut tx, user_id, provider, user_info, tokens).await?;
        
        tx.commit().await?;
        
        // Return created user
        self.get_user_by_id(user_id).await
    }
    
    pub async fn link_oauth_account_to_user(&self, user_id: Uuid, provider: OAuthProviderType, user_info: OAuthUserInfo, tokens: OAuthTokens) -> Result<()> {
        let mut tx = self.db_pool.begin().await?;
        
        // Check if OAuth account already exists for this provider
        let existing = sqlx::query!(
            "SELECT id FROM oauth_accounts WHERE user_id = $1 AND provider = $2",
            user_id,
            provider.to_string()
        )
        .fetch_optional(&mut *tx)
        .await?;
        
        if existing.is_some() {
            return Err(AppError::InvalidFieldValue {
                field: "provider".to_string(),
                message: format!("OAuth account for {} already linked to this user", provider),
            });
        }
        
        // Create OAuth account
        self.create_oauth_account_in_tx(&mut tx, user_id, provider, user_info, tokens).await?;
        
        tx.commit().await?;
        Ok(())
    }
    
    async fn create_oauth_account_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: Uuid,
        provider: OAuthProviderType,
        user_info: OAuthUserInfo,
        tokens: OAuthTokens,
    ) -> Result<()> {
        // Encrypt tokens
        let access_token_encrypted = self.oauth_encryption.encrypt(&tokens.access_token).await?;
        let refresh_token_encrypted = if let Some(refresh_token) = &tokens.refresh_token {
            Some(self.oauth_encryption.encrypt(refresh_token).await?)
        } else {
            None
        };
        
        let token_expires_at = tokens.expires_in.map(|expires_in| {
            Utc::now() + Duration::seconds(expires_in)
        });
        
        sqlx::query!(
            r#"
            INSERT INTO oauth_accounts (
                user_id, provider, provider_user_id, email, display_name, avatar_url,
                access_token_encrypted, refresh_token_encrypted, token_expires_at,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            user_id,
            provider.to_string(),
            user_info.provider_user_id,
            user_info.email,
            user_info.display_name,
            user_info.avatar_url,
            access_token_encrypted,
            refresh_token_encrypted,
            token_expires_at,
            Utc::now(),
            Utc::now()
        )
        .execute(&mut **tx)
        .await?;
        
        Ok(())
    }
}
```

### 4. Token Management and Refresh

#### Automatic Token Refresh Implementation

```rust
impl AuthService {
    pub async fn refresh_oauth_tokens(&self, user_id: Uuid, provider_type: OAuthProviderType) -> Result<()> {
        // Get OAuth account
        let oauth_account = sqlx::query!(
            r#"
            SELECT id, refresh_token_encrypted, token_expires_at
            FROM oauth_accounts
            WHERE user_id = $1 AND provider = $2
            "#,
            user_id,
            provider_type.to_string()
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: format!("OAuth account for provider {}", provider_type),
        })?;
        
        // Check if refresh token exists
        let refresh_token_encrypted = oauth_account.refresh_token_encrypted
            .ok_or_else(|| AppError::InvalidFieldValue {
                field: "refresh_token".to_string(),
                message: "No refresh token available for this OAuth account".to_string(),
            })?;
        
        // Decrypt refresh token
        let refresh_token = self.oauth_encryption.decrypt(&refresh_token_encrypted).await?;
        
        // Get OAuth provider
        let oauth_provider = self.oauth_providers.get(&provider_type)
            .ok_or_else(|| AppError::InvalidFieldValue {
                field: "provider".to_string(),
                message: format!("OAuth provider {} not configured", provider_type),
            })?;
        
        // Refresh tokens
        let new_tokens = oauth_provider.refresh_token(&refresh_token).await?;
        
        // Encrypt new tokens
        let new_access_token_encrypted = self.oauth_encryption.encrypt(&new_tokens.access_token).await?;
        let new_refresh_token_encrypted = if let Some(new_refresh_token) = &new_tokens.refresh_token {
            Some(self.oauth_encryption.encrypt(new_refresh_token).await?)
        } else {
            Some(refresh_token_encrypted) // Keep existing refresh token if new one not provided
        };
        
        let new_token_expires_at = new_tokens.expires_in.map(|expires_in| {
            Utc::now() + Duration::seconds(expires_in)
        });
        
        // Update tokens in database
        sqlx::query!(
            r#"
            UPDATE oauth_accounts
            SET access_token_encrypted = $1,
                refresh_token_encrypted = $2,
                token_expires_at = $3,
                updated_at = $4
            WHERE id = $5
            "#,
            new_access_token_encrypted,
            new_refresh_token_encrypted,
            new_token_expires_at,
            Utc::now(),
            oauth_account.id
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_valid_oauth_token(&self, user_id: Uuid, provider_type: OAuthProviderType) -> Result<String> {
        let oauth_account = sqlx::query!(
            r#"
            SELECT access_token_encrypted, token_expires_at
            FROM oauth_accounts
            WHERE user_id = $1 AND provider = $2
            "#,
            user_id,
            provider_type.to_string()
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: format!("OAuth account for provider {}", provider_type),
        })?;
        
        // Check if token is expired
        if let Some(expires_at) = oauth_account.token_expires_at {
            if Utc::now() > expires_at - Duration::minutes(5) { // Refresh 5 minutes before expiry
                self.refresh_oauth_tokens(user_id, provider_type).await?;
                
                // Get updated token
                let updated_account = sqlx::query!(
                    "SELECT access_token_encrypted FROM oauth_accounts WHERE user_id = $1 AND provider = $2",
                    user_id,
                    provider_type.to_string()
                )
                .fetch_one(&self.db_pool)
                .await?;
                
                return self.oauth_encryption.decrypt(&updated_account.access_token_encrypted).await;
            }
        }
        
        // Decrypt and return current token
        self.oauth_encryption.decrypt(&oauth_account.access_token_encrypted).await
    }
}
```

## Data Models

### Enhanced OAuth Response Models

```rust
// Google OAuth API responses
#[derive(Debug, Deserialize)]
pub struct GoogleTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub token_type: String,
    pub scope: Option<String>,
    pub id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GoogleUserResponse {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub locale: Option<String>,
}

// Apple OAuth API responses
#[derive(Debug, Deserialize)]
pub struct AppleTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub token_type: String,
    pub id_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AppleClientSecretClaims {
    pub iss: String, // Team ID
    pub iat: i64,    // Issued at
    pub exp: i64,    // Expires at
    pub aud: String, // Audience
    pub sub: String, // Subject (Client ID)
}

// GitHub OAuth API responses
#[derive(Debug, Deserialize)]
pub struct GitHubTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubUserResponse {
    pub id: u64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubEmailResponse {
    pub email: String,
    pub primary: bool,
    pub verified: bool,
}
```

## Error Handling

### OAuth-Specific Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    #[error("OAuth provider error: {provider} - {message}")]
    ProviderError { provider: String, message: String },
    
    #[error("Invalid OAuth state parameter")]
    InvalidState,
    
    #[error("OAuth token expired and refresh failed: {reason}")]
    TokenRefreshFailed { reason: String },
    
    #[error("OAuth account already linked to different user")]
    AccountAlreadyLinked,
    
    #[error("OAuth provider not configured: {provider}")]
    ProviderNotConfigured { provider: String },
    
    #[error("Token encryption error: {0}")]
    TokenEncryptionError(#[from] aes_gcm::Error),
    
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    
    #[error("HTTP request error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("URL parsing error: {0}")]
    UrlError(#[from] url::ParseError),
}
```

### Error Recovery Strategies

1. **Token Refresh Failures**: Prompt user to re-authenticate
2. **Provider API Errors**: Retry with exponential backoff
3. **State Validation Failures**: Log security event and reject request
4. **Account Linking Conflicts**: Provide user with merge options
5. **Encryption Errors**: Rotate keys and re-encrypt tokens

## Testing Strategy

### Unit Tests for OAuth Methods

```rust
#[cfg(test)]
mod oauth_tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    
    #[tokio::test]
    async fn test_complete_oauth_flow_new_user() {
        let mock_server = MockServer::start().await;
        
        // Mock Google token exchange
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({
                    "access_token": "test_access_token",
                    "refresh_token": "test_refresh_token",
                    "expires_in": 3600,
                    "token_type": "Bearer"
                })))
            .mount(&mock_server)
            .await;
        
        // Mock Google user info
        Mock::given(method("GET"))
            .and(path("/oauth2/v2/userinfo"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({
                    "id": "123456789",
                    "email": "test@gmail.com",
                    "verified_email": true,
                    "name": "Test User"
                })))
            .mount(&mock_server)
            .await;
        
        let auth_service = create_test_auth_service(&mock_server.uri()).await;
        
        let result = auth_service.complete_oauth_flow(
            OAuthProviderType::Google,
            "test_code".to_string(),
            "test_state".to_string(),
            "http://localhost:3000/callback".to_string(),
        ).await;
        
        assert!(result.is_ok());
        let token_pair = result.unwrap();
        assert!(!token_pair.access_token.is_empty());
        assert!(!token_pair.refresh_token.is_empty());
    }
    
    #[tokio::test]
    async fn test_oauth_token_refresh() {
        // Test automatic token refresh when tokens are near expiry
    }
    
    #[tokio::test]
    async fn test_oauth_account_linking() {
        // Test linking OAuth account to existing user
    }
}
```

This design provides a comprehensive plan for completing the OAuth functionality by replacing all placeholder implementations with real, working OAuth integration.