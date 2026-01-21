use crate::error::{AppError, Result};
use crate::models::oauth::OAuthProviderType;
use std::collections::HashMap;
use std::env;
use tracing::{error, info, warn};

/// OAuth provider configuration validation results
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OAuthProviderValidation {
    pub provider: OAuthProviderType,
    pub is_configured: bool,
    pub is_valid: bool,
    pub missing_variables: Vec<String>,
    pub validation_errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// OAuth configuration validator
pub struct OAuthConfigValidator {
    validation_results: HashMap<OAuthProviderType, OAuthProviderValidation>,
}

impl OAuthConfigValidator {
    /// Create a new OAuth configuration validator
    pub fn new() -> Self {
        Self {
            validation_results: HashMap::new(),
        }
    }

    /// Validate all OAuth provider configurations on startup
    pub fn validate_all_providers(&mut self) -> Result<()> {
        info!("🔍 Validating OAuth provider configurations...");

        // Validate each provider
        self.validate_google_config();
        self.validate_apple_config();
        self.validate_github_config();
        self.validate_spotify_config();

        // Log summary
        self.log_validation_summary();

        // Check if at least one provider is configured
        let configured_providers: Vec<_> = self
            .validation_results
            .values()
            .filter(|v| v.is_configured && v.is_valid)
            .collect();

        if configured_providers.is_empty() {
            let dev_mode =
                env::var("OAUTH_DEV_MODE").unwrap_or_else(|_| "false".to_string()) == "true";
            if dev_mode {
                warn!("⚠️  No OAuth providers configured, but OAUTH_DEV_MODE is enabled. Demo providers will be used.");
            } else {
                warn!("⚠️  No OAuth providers are properly configured. Social authentication will be unavailable.");
                warn!("💡 Set environment variables for OAuth providers or enable OAUTH_DEV_MODE=true for development.");
            }
        } else {
            info!(
                "✅ {} OAuth provider(s) configured successfully",
                configured_providers.len()
            );
        }

        Ok(())
    }

    /// Get validation results for all providers
    pub fn get_validation_results(&self) -> &HashMap<OAuthProviderType, OAuthProviderValidation> {
        &self.validation_results
    }

    /// Get validation result for a specific provider
    pub fn get_provider_validation(
        &self,
        provider: &OAuthProviderType,
    ) -> Option<&OAuthProviderValidation> {
        self.validation_results.get(provider)
    }

    /// Check if a provider is properly configured and valid
    pub fn is_provider_available(&self, provider: &OAuthProviderType) -> bool {
        self.validation_results
            .get(provider)
            .map(|v| v.is_configured && v.is_valid)
            .unwrap_or(false)
    }

    /// Get list of available (configured and valid) providers
    pub fn get_available_providers(&self) -> Vec<OAuthProviderType> {
        self.validation_results
            .iter()
            .filter(|(_, validation)| validation.is_configured && validation.is_valid)
            .map(|(provider, _)| provider.clone())
            .collect()
    }

    /// Validate Google OAuth configuration
    fn validate_google_config(&mut self) {
        let mut validation = OAuthProviderValidation {
            provider: OAuthProviderType::Google,
            is_configured: false,
            is_valid: false,
            missing_variables: Vec::new(),
            validation_errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Check required environment variables
        let client_id = env::var("GOOGLE_CLIENT_ID");
        let client_secret = env::var("GOOGLE_CLIENT_SECRET");
        let redirect_uri = env::var("GOOGLE_REDIRECT_URI");

        // Track missing variables
        if client_id.is_err() {
            validation
                .missing_variables
                .push("GOOGLE_CLIENT_ID".to_string());
        }
        if client_secret.is_err() {
            validation
                .missing_variables
                .push("GOOGLE_CLIENT_SECRET".to_string());
        }
        if redirect_uri.is_err() {
            validation
                .missing_variables
                .push("GOOGLE_REDIRECT_URI".to_string());
        }

        // If all variables are present, validate their values
        if let (Ok(client_id), Ok(client_secret), Ok(redirect_uri)) =
            (client_id, client_secret, redirect_uri)
        {
            validation.is_configured = true;

            // Validate client ID format (Google client IDs end with .apps.googleusercontent.com)
            if !client_id.ends_with(".apps.googleusercontent.com")
                && !client_id.starts_with("demo-")
            {
                validation.validation_errors.push(
                    "GOOGLE_CLIENT_ID should end with '.apps.googleusercontent.com'".to_string(),
                );
            }

            // Validate client secret is not empty
            if client_secret.trim().is_empty() {
                validation
                    .validation_errors
                    .push("GOOGLE_CLIENT_SECRET cannot be empty".to_string());
            }

            // Validate redirect URI format
            if !redirect_uri.starts_with("http://") && !redirect_uri.starts_with("https://") {
                validation
                    .validation_errors
                    .push("GOOGLE_REDIRECT_URI must be a valid HTTP/HTTPS URL".to_string());
            }

            // Check for development/demo configuration
            if client_id.starts_with("demo-") || client_secret.starts_with("demo-") {
                validation.warnings.push(
                    "Using demo Google OAuth configuration - not suitable for production"
                        .to_string(),
                );
            }

            // Validate redirect URI points to callback endpoint
            if !redirect_uri.contains("/auth/callback") && !redirect_uri.contains("/oauth/callback")
            {
                validation.warnings.push(
                    "GOOGLE_REDIRECT_URI should point to OAuth callback endpoint (e.g., /auth/callback/google)".to_string()
                );
            }

            // Configuration is valid if no validation errors
            validation.is_valid = validation.validation_errors.is_empty();
        }

        self.validation_results
            .insert(OAuthProviderType::Google, validation);
    }

    /// Validate Apple OAuth configuration
    fn validate_apple_config(&mut self) {
        let mut validation = OAuthProviderValidation {
            provider: OAuthProviderType::Apple,
            is_configured: false,
            is_valid: false,
            missing_variables: Vec::new(),
            validation_errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Check required environment variables
        let client_id = env::var("APPLE_CLIENT_ID");
        let team_id = env::var("APPLE_TEAM_ID");
        let key_id = env::var("APPLE_KEY_ID");
        let private_key = env::var("APPLE_PRIVATE_KEY");
        let redirect_uri = env::var("APPLE_REDIRECT_URI");

        // Track missing variables
        if client_id.is_err() {
            validation
                .missing_variables
                .push("APPLE_CLIENT_ID".to_string());
        }
        if team_id.is_err() {
            validation
                .missing_variables
                .push("APPLE_TEAM_ID".to_string());
        }
        if key_id.is_err() {
            validation
                .missing_variables
                .push("APPLE_KEY_ID".to_string());
        }
        if private_key.is_err() {
            validation
                .missing_variables
                .push("APPLE_PRIVATE_KEY".to_string());
        }
        if redirect_uri.is_err() {
            validation
                .missing_variables
                .push("APPLE_REDIRECT_URI".to_string());
        }

        // If all variables are present, validate their values
        if let (Ok(client_id), Ok(team_id), Ok(key_id), Ok(private_key), Ok(redirect_uri)) =
            (client_id, team_id, key_id, private_key, redirect_uri)
        {
            validation.is_configured = true;

            // Validate client ID format (Apple client IDs are typically bundle identifiers)
            if !client_id.contains('.') && !client_id.starts_with("demo-") {
                validation.validation_errors.push(
                    "APPLE_CLIENT_ID should be a bundle identifier (e.g., com.example.app)"
                        .to_string(),
                );
            }

            // Validate team ID format (10 characters alphanumeric)
            if team_id.len() != 10 || !team_id.chars().all(|c| c.is_alphanumeric()) {
                if !team_id.starts_with("demo-") {
                    validation.validation_errors.push(
                        "APPLE_TEAM_ID should be a 10-character alphanumeric string".to_string(),
                    );
                }
            }

            // Validate key ID format (10 characters alphanumeric)
            if key_id.len() != 10 || !key_id.chars().all(|c| c.is_alphanumeric()) {
                if !key_id.starts_with("demo-") {
                    validation.validation_errors.push(
                        "APPLE_KEY_ID should be a 10-character alphanumeric string".to_string(),
                    );
                }
            }

            // Validate private key format (should be PEM format)
            if !private_key.contains("-----BEGIN PRIVATE KEY-----")
                && !private_key.starts_with("demo-")
            {
                validation
                    .validation_errors
                    .push("APPLE_PRIVATE_KEY should be in PEM format".to_string());
            }

            // Validate redirect URI format
            if !redirect_uri.starts_with("http://") && !redirect_uri.starts_with("https://") {
                validation
                    .validation_errors
                    .push("APPLE_REDIRECT_URI must be a valid HTTP/HTTPS URL".to_string());
            }

            // Check for development/demo configuration
            if client_id.starts_with("demo-") || team_id.starts_with("demo-") {
                validation.warnings.push(
                    "Using demo Apple OAuth configuration - not suitable for production"
                        .to_string(),
                );
            }

            // Apple Sign In requires HTTPS in production
            if redirect_uri.starts_with("http://") && !redirect_uri.contains("localhost") {
                validation.warnings.push(
                    "Apple Sign In requires HTTPS URLs in production (HTTP only allowed for localhost)".to_string()
                );
            }

            // Configuration is valid if no validation errors
            validation.is_valid = validation.validation_errors.is_empty();
        }

        self.validation_results
            .insert(OAuthProviderType::Apple, validation);
    }

    /// Validate GitHub OAuth configuration
    fn validate_github_config(&mut self) {
        let mut validation = OAuthProviderValidation {
            provider: OAuthProviderType::GitHub,
            is_configured: false,
            is_valid: false,
            missing_variables: Vec::new(),
            validation_errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Check required environment variables
        let client_id = env::var("GITHUB_CLIENT_ID");
        let client_secret = env::var("GITHUB_CLIENT_SECRET");
        let redirect_uri = env::var("GITHUB_REDIRECT_URI");

        // Track missing variables
        if client_id.is_err() {
            validation
                .missing_variables
                .push("GITHUB_CLIENT_ID".to_string());
        }
        if client_secret.is_err() {
            validation
                .missing_variables
                .push("GITHUB_CLIENT_SECRET".to_string());
        }
        if redirect_uri.is_err() {
            validation
                .missing_variables
                .push("GITHUB_REDIRECT_URI".to_string());
        }

        // If all variables are present, validate their values
        if let (Ok(client_id), Ok(client_secret), Ok(redirect_uri)) =
            (client_id, client_secret, redirect_uri)
        {
            validation.is_configured = true;

            // Validate client ID format (GitHub client IDs are typically hex strings)
            if client_id.len() < 16 && !client_id.starts_with("demo-") {
                validation.validation_errors.push(
                    "GITHUB_CLIENT_ID appears to be too short (should be at least 16 characters)"
                        .to_string(),
                );
            }

            // Validate client secret is not empty
            if client_secret.trim().is_empty() {
                validation
                    .validation_errors
                    .push("GITHUB_CLIENT_SECRET cannot be empty".to_string());
            }

            // Validate client secret format (GitHub client secrets are typically 40 characters)
            if client_secret.len() < 32 && !client_secret.starts_with("demo-") {
                validation.validation_errors.push(
                    "GITHUB_CLIENT_SECRET appears to be too short (should be at least 32 characters)".to_string()
                );
            }

            // Validate redirect URI format
            if !redirect_uri.starts_with("http://") && !redirect_uri.starts_with("https://") {
                validation
                    .validation_errors
                    .push("GITHUB_REDIRECT_URI must be a valid HTTP/HTTPS URL".to_string());
            }

            // Check for development/demo configuration
            if client_id.starts_with("demo-") || client_secret.starts_with("demo-") {
                validation.warnings.push(
                    "Using demo GitHub OAuth configuration - not suitable for production"
                        .to_string(),
                );
            }

            // Validate redirect URI points to callback endpoint
            if !redirect_uri.contains("/auth/callback") && !redirect_uri.contains("/oauth/callback")
            {
                validation.warnings.push(
                    "GITHUB_REDIRECT_URI should point to OAuth callback endpoint (e.g., /auth/callback/github)".to_string()
                );
            }

            // Configuration is valid if no validation errors
            validation.is_valid = validation.validation_errors.is_empty();
        }

        self.validation_results
            .insert(OAuthProviderType::GitHub, validation);
    }

    /// Validate Spotify OAuth configuration
    fn validate_spotify_config(&mut self) {
        let mut validation = OAuthProviderValidation {
            provider: OAuthProviderType::Spotify,
            is_configured: false,
            is_valid: false,
            missing_variables: Vec::new(),
            validation_errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Check required environment variables
        let client_id = env::var("SPOTIFY_CLIENT_ID");
        let client_secret = env::var("SPOTIFY_CLIENT_SECRET");

        // Check if all required variables are present
        if client_id.is_err() {
            validation
                .missing_variables
                .push("SPOTIFY_CLIENT_ID".to_string());
        }
        if client_secret.is_err() {
            validation
                .missing_variables
                .push("SPOTIFY_CLIENT_SECRET".to_string());
        }

        // If any required variables are missing, mark as not configured
        if !validation.missing_variables.is_empty() {
            self.validation_results
                .insert(OAuthProviderType::Spotify, validation);
            return;
        }

        // Mark as configured since all required variables are present
        validation.is_configured = true;

        // Validate configuration values
        if let (Ok(client_id), Ok(client_secret)) = (client_id, client_secret) {
            // Validate client ID format (Spotify client IDs are typically 32 characters)
            if client_id.len() != 32 && !client_id.starts_with("demo-") {
                validation
                    .validation_errors
                    .push("SPOTIFY_CLIENT_ID should be exactly 32 characters".to_string());
            }

            // Validate client secret format (Spotify client secrets are typically 32 characters)
            if client_secret.len() != 32 && !client_secret.starts_with("demo-") {
                validation
                    .validation_errors
                    .push("SPOTIFY_CLIENT_SECRET should be exactly 32 characters".to_string());
            }

            // Check for development/demo configuration
            if client_id.starts_with("demo-") || client_secret.starts_with("demo-") {
                validation.warnings.push(
                    "Using demo Spotify OAuth configuration - not suitable for production"
                        .to_string(),
                );
            }

            // Configuration is valid if no validation errors
            validation.is_valid = validation.validation_errors.is_empty();
        }

        self.validation_results
            .insert(OAuthProviderType::Spotify, validation);
    }

    /// Log validation summary
    fn log_validation_summary(&self) {
        info!("📋 OAuth Configuration Validation Summary:");

        for (provider, validation) in &self.validation_results {
            if validation.is_configured && validation.is_valid {
                info!("  ✅ {}: Configured and valid", provider);
                for warning in &validation.warnings {
                    warn!("    ⚠️  {}", warning);
                }
            } else if validation.is_configured {
                error!("  ❌ {}: Configured but invalid", provider);
                for error in &validation.validation_errors {
                    error!("    🚫 {}", error);
                }
            } else {
                info!("  ⏭️  {}: Not configured", provider);
                if !validation.missing_variables.is_empty() {
                    info!(
                        "    📝 Missing variables: {}",
                        validation.missing_variables.join(", ")
                    );
                }
            }
        }
    }

    /// Get configuration guidance for a provider
    pub fn get_configuration_guidance(&self, provider: &OAuthProviderType) -> String {
        match provider {
            OAuthProviderType::Google => "To configure Google OAuth:\n\
                1. Go to Google Cloud Console (https://console.cloud.google.com/)\n\
                2. Create or select a project\n\
                3. Enable the Google+ API\n\
                4. Create OAuth 2.0 credentials\n\
                5. Set environment variables:\n\
                   - GOOGLE_CLIENT_ID=your_client_id.apps.googleusercontent.com\n\
                   - GOOGLE_CLIENT_SECRET=your_client_secret\n\
                   - GOOGLE_REDIRECT_URI=https://yourdomain.com/auth/callback/google"
                .to_string(),
            OAuthProviderType::Apple => "To configure Apple Sign In:\n\
                1. Go to Apple Developer Portal (https://developer.apple.com/)\n\
                2. Create an App ID with Sign In with Apple capability\n\
                3. Create a Services ID for web authentication\n\
                4. Create a private key for Sign In with Apple\n\
                5. Set environment variables:\n\
                   - APPLE_CLIENT_ID=your.bundle.identifier\n\
                   - APPLE_TEAM_ID=your_team_id\n\
                   - APPLE_KEY_ID=your_key_id\n\
                   - APPLE_PRIVATE_KEY=your_private_key_pem\n\
                   - APPLE_REDIRECT_URI=https://yourdomain.com/auth/callback/apple"
                .to_string(),
            OAuthProviderType::GitHub => "To configure GitHub OAuth:\n\
                1. Go to GitHub Settings > Developer settings > OAuth Apps\n\
                2. Create a new OAuth App\n\
                3. Set the authorization callback URL\n\
                4. Set environment variables:\n\
                   - GITHUB_CLIENT_ID=your_client_id\n\
                   - GITHUB_CLIENT_SECRET=your_client_secret\n\
                   - GITHUB_REDIRECT_URI=https://yourdomain.com/auth/callback/github"
                .to_string(),
            OAuthProviderType::Spotify => "To configure Spotify OAuth:\n\
                1. Go to Spotify Developer Dashboard (https://developer.spotify.com/dashboard)\n\
                2. Create a new application\n\
                3. Set the redirect URI\n\
                4. Set environment variables:\n\
                   - SPOTIFY_CLIENT_ID=your_client_id\n\
                   - SPOTIFY_CLIENT_SECRET=your_client_secret\n\
                   - SPOTIFY_REDIRECT_URI=https://yourdomain.com/auth/callback/spotify"
                .to_string(),
            OAuthProviderType::YouTubeMusic => "To configure YouTube Music OAuth:\n\
                1. Go to Google Cloud Console (https://console.cloud.google.com/)\n\
                2. Create or select a project\n\
                3. Enable the YouTube Data API v3\n\
                4. Create OAuth 2.0 credentials\n\
                5. Set environment variables:\n\
                   - YOUTUBE_MUSIC_CLIENT_ID=your_client_id.apps.googleusercontent.com\n\
                   - YOUTUBE_MUSIC_CLIENT_SECRET=your_client_secret\n\
                   - YOUTUBE_MUSIC_REDIRECT_URI=https://yourdomain.com/auth/callback/youtube-music"
                .to_string(),
            OAuthProviderType::Tidal => "To configure Tidal OAuth:\n\
                1. Go to TIDAL Developer Portal (https://developer.tidal.com/)\n\
                2. Create a new application\n\
                3. Set the redirect URI\n\
                4. Set environment variables:\n\
                   - TIDAL_CLIENT_ID=your_client_id\n\
                   - TIDAL_CLIENT_SECRET=your_client_secret\n\
                   - TIDAL_REDIRECT_URI=https://yourdomain.com/auth/callback/tidal"
                .to_string(),
        }
    }

    /// Validate environment variables are properly set
    pub fn validate_environment_security(&self) -> Result<()> {
        let mut security_issues = Vec::new();

        // Check for insecure configurations in production
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        let is_production = environment.to_lowercase() == "production";

        if is_production {
            // Check for demo configurations in production
            for (provider, validation) in &self.validation_results {
                if validation.is_configured {
                    for warning in &validation.warnings {
                        if warning.contains("demo") {
                            security_issues.push(format!(
                                "{}: Demo configuration detected in production environment",
                                provider
                            ));
                        }
                    }
                }
            }

            // Check for HTTP redirect URIs in production
            let redirect_uris = [
                ("GOOGLE_REDIRECT_URI", env::var("GOOGLE_REDIRECT_URI")),
                ("APPLE_REDIRECT_URI", env::var("APPLE_REDIRECT_URI")),
                ("GITHUB_REDIRECT_URI", env::var("GITHUB_REDIRECT_URI")),
            ];

            for (var_name, uri_result) in redirect_uris {
                if let Ok(uri) = uri_result {
                    if uri.starts_with("http://") && !uri.contains("localhost") {
                        security_issues.push(format!(
                            "{}: HTTP redirect URI not allowed in production (use HTTPS)",
                            var_name
                        ));
                    }
                }
            }
        }

        if !security_issues.is_empty() {
            error!("🚨 OAuth Security Issues Detected:");
            for issue in &security_issues {
                error!("  🔒 {}", issue);
            }
            return Err(AppError::InvalidFieldValue {
                field: "oauth_configuration".to_string(),
                message: format!("Security issues found: {}", security_issues.join("; ")),
            });
        }

        Ok(())
    }
}

impl Default for OAuthConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_oauth_config_validator_creation() {
        let validator = OAuthConfigValidator::new();
        assert!(validator.validation_results.is_empty());
    }

    #[test]
    fn test_google_config_validation_missing_vars() {
        // Clear environment variables
        env::remove_var("GOOGLE_CLIENT_ID");
        env::remove_var("GOOGLE_CLIENT_SECRET");
        env::remove_var("GOOGLE_REDIRECT_URI");

        let mut validator = OAuthConfigValidator::new();
        validator.validate_google_config();

        let validation = validator
            .get_provider_validation(&OAuthProviderType::Google)
            .unwrap();
        assert!(!validation.is_configured);
        assert!(!validation.is_valid);
        assert_eq!(validation.missing_variables.len(), 3);
    }

    #[test]
    fn test_google_config_validation_valid() {
        // Set valid environment variables
        env::set_var("GOOGLE_CLIENT_ID", "test.apps.googleusercontent.com");
        env::set_var("GOOGLE_CLIENT_SECRET", "test_secret");
        env::set_var(
            "GOOGLE_REDIRECT_URI",
            "https://example.com/auth/callback/google",
        );

        let mut validator = OAuthConfigValidator::new();
        validator.validate_google_config();

        let validation = validator
            .get_provider_validation(&OAuthProviderType::Google)
            .unwrap();
        assert!(validation.is_configured);
        assert!(validation.is_valid);
        assert!(validation.validation_errors.is_empty());

        // Clean up
        env::remove_var("GOOGLE_CLIENT_ID");
        env::remove_var("GOOGLE_CLIENT_SECRET");
        env::remove_var("GOOGLE_REDIRECT_URI");
    }

    #[test]
    fn test_github_config_validation_invalid_format() {
        // Set invalid environment variables
        env::set_var("GITHUB_CLIENT_ID", "short");
        env::set_var("GITHUB_CLIENT_SECRET", "");
        env::set_var("GITHUB_REDIRECT_URI", "invalid-url");

        let mut validator = OAuthConfigValidator::new();
        validator.validate_github_config();

        let validation = validator
            .get_provider_validation(&OAuthProviderType::GitHub)
            .unwrap();
        assert!(validation.is_configured);
        assert!(!validation.is_valid);
        assert!(!validation.validation_errors.is_empty());

        // Clean up
        env::remove_var("GITHUB_CLIENT_ID");
        env::remove_var("GITHUB_CLIENT_SECRET");
        env::remove_var("GITHUB_REDIRECT_URI");
    }

    #[test]
    fn test_configuration_guidance() {
        let validator = OAuthConfigValidator::new();

        let google_guidance = validator.get_configuration_guidance(&OAuthProviderType::Google);
        assert!(google_guidance.contains("Google Cloud Console"));

        let apple_guidance = validator.get_configuration_guidance(&OAuthProviderType::Apple);
        assert!(apple_guidance.contains("Apple Developer Portal"));

        let github_guidance = validator.get_configuration_guidance(&OAuthProviderType::GitHub);
        assert!(github_guidance.contains("GitHub Settings"));
    }
}
