//! OAuth-specific error types and handling

use crate::models::oauth::OAuthProviderType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

/// Comprehensive OAuth error types for different failure scenarios
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum OAuthError {
    // Provider-specific errors
    #[error("OAuth provider error: {provider} - {error_code}: {message}")]
    ProviderError {
        provider: OAuthProviderType,
        error_code: String,
        message: String,
        details: Option<serde_json::Value>,
    },

    // Configuration errors
    #[error("OAuth provider {provider} is not configured: {reason}")]
    ProviderNotConfigured {
        provider: OAuthProviderType,
        reason: String,
        missing_variables: Vec<String>,
    },

    #[error("OAuth provider {provider} configuration is invalid: {reason}")]
    InvalidConfiguration {
        provider: OAuthProviderType,
        reason: String,
        validation_errors: Vec<String>,
    },

    // Flow and state errors
    #[error("OAuth state validation failed: {reason}")]
    StateValidationFailed {
        reason: String,
        expected_provider: Option<OAuthProviderType>,
        received_provider: Option<OAuthProviderType>,
    },

    #[error("OAuth authorization code is invalid or expired")]
    InvalidAuthorizationCode {
        provider: OAuthProviderType,
        error_code: Option<String>,
        error_description: Option<String>,
    },

    #[error("OAuth redirect URI mismatch")]
    RedirectUriMismatch {
        provider: OAuthProviderType,
        expected: String,
        received: String,
    },

    // Token-related errors
    #[error("OAuth token exchange failed: {reason}")]
    TokenExchangeFailed {
        provider: OAuthProviderType,
        reason: String,
        error_code: Option<String>,
        retry_after: Option<u64>,
    },

    #[error("OAuth token refresh failed: {reason}")]
    TokenRefreshFailed {
        provider: OAuthProviderType,
        reason: String,
        requires_reauth: bool,
    },

    #[error("OAuth token is invalid or expired")]
    InvalidToken {
        provider: OAuthProviderType,
        token_type: TokenType,
        reason: String,
    },

    #[error("OAuth token encryption/decryption failed: {reason}")]
    TokenEncryptionFailed {
        reason: String,
        operation: EncryptionOperation,
    },

    // Account management errors
    #[error("OAuth account linking failed: {reason}")]
    AccountLinkingFailed {
        provider: OAuthProviderType,
        reason: String,
        conflict_type: Option<AccountConflictType>,
    },

    #[error("OAuth account unlinking failed: {reason}")]
    AccountUnlinkingFailed {
        provider: OAuthProviderType,
        reason: String,
        safety_check: Option<String>,
    },

    #[error("OAuth account merge conflict: {reason}")]
    AccountMergeConflict {
        provider: OAuthProviderType,
        reason: String,
        conflicting_accounts: Vec<String>,
    },

    // Provider health and availability errors
    #[error("OAuth provider {provider} is temporarily unavailable")]
    ProviderUnavailable {
        provider: OAuthProviderType,
        reason: String,
        estimated_recovery: Option<chrono::DateTime<chrono::Utc>>,
        retry_after: Option<u64>,
    },

    #[error("OAuth provider {provider} rate limit exceeded")]
    RateLimitExceeded {
        provider: OAuthProviderType,
        retry_after: u64,
        limit_type: RateLimitType,
    },

    // User info and scope errors
    #[error("Failed to retrieve user information from {provider}: {reason}")]
    UserInfoRetrievalFailed {
        provider: OAuthProviderType,
        reason: String,
        missing_scopes: Vec<String>,
    },

    #[error("Insufficient OAuth scopes for {provider}")]
    InsufficientScopes {
        provider: OAuthProviderType,
        required_scopes: Vec<String>,
        granted_scopes: Vec<String>,
    },

    // Security-related errors
    #[error("OAuth security violation detected: {violation}")]
    SecurityViolation {
        provider: OAuthProviderType,
        violation: SecurityViolationType,
        details: String,
        client_info: Option<ClientInfo>,
    },

    #[error("OAuth CSRF attack detected")]
    CsrfAttackDetected {
        provider: OAuthProviderType,
        expected_state: String,
        received_state: String,
        client_info: Option<ClientInfo>,
    },

    // Network and connectivity errors
    #[error("Network error communicating with {provider}: {reason}")]
    NetworkError {
        provider: OAuthProviderType,
        reason: String,
        is_transient: bool,
        retry_count: u32,
    },

    #[error("OAuth provider {provider} API timeout")]
    ApiTimeout {
        provider: OAuthProviderType,
        operation: String,
        timeout_duration: std::time::Duration,
    },
}

/// Types of OAuth tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenType {
    AccessToken,
    RefreshToken,
    IdToken,
}

/// Encryption operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionOperation {
    Encrypt,
    Decrypt,
    KeyRotation,
}

/// Types of account conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountConflictType {
    AlreadyLinkedToSameUser,
    AlreadyLinkedToDifferentUser,
    EmailConflict,
    ProviderUserIdConflict,
}

/// Types of rate limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitType {
    ApiCalls,
    TokenRequests,
    UserInfoRequests,
    AuthorizationRequests,
}

/// Security violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityViolationType {
    StateParameterMissing,
    StateParameterTampered,
    UnexpectedRedirectUri,
    SuspiciousClientBehavior,
    TokenReplayAttack,
    InvalidSignature,
}

/// Client information for security logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl OAuthError {
    /// Create a provider error with structured details
    pub fn provider_error(
        provider: OAuthProviderType,
        error_code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::ProviderError {
            provider,
            error_code: error_code.into(),
            message: message.into(),
            details: None,
        }
    }

    /// Create a provider error with additional details
    pub fn provider_error_with_details(
        provider: OAuthProviderType,
        error_code: impl Into<String>,
        message: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        Self::ProviderError {
            provider,
            error_code: error_code.into(),
            message: message.into(),
            details: Some(details),
        }
    }

    /// Create a configuration error
    pub fn configuration_error(
        provider: OAuthProviderType,
        reason: impl Into<String>,
        missing_variables: Vec<String>,
    ) -> Self {
        Self::ProviderNotConfigured {
            provider,
            reason: reason.into(),
            missing_variables,
        }
    }

    /// Create a token refresh error
    pub fn token_refresh_failed(
        provider: OAuthProviderType,
        reason: impl Into<String>,
        requires_reauth: bool,
    ) -> Self {
        Self::TokenRefreshFailed {
            provider,
            reason: reason.into(),
            requires_reauth,
        }
    }

    /// Create a security violation error
    pub fn security_violation(
        provider: OAuthProviderType,
        violation: SecurityViolationType,
        details: impl Into<String>,
        client_info: Option<ClientInfo>,
    ) -> Self {
        Self::SecurityViolation {
            provider,
            violation,
            details: details.into(),
            client_info,
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::NetworkError { is_transient, .. } => *is_transient,
            Self::ApiTimeout { .. } => true,
            Self::ProviderUnavailable { .. } => true,
            Self::RateLimitExceeded { .. } => true,
            Self::TokenExchangeFailed { error_code, .. } => {
                // Some token exchange errors are retryable
                matches!(
                    error_code.as_deref(),
                    Some("temporarily_unavailable") | Some("server_error")
                )
            }
            _ => false,
        }
    }

    /// Get retry delay in seconds
    pub fn retry_delay(&self) -> Option<u64> {
        match self {
            Self::RateLimitExceeded { retry_after, .. } => Some(*retry_after),
            Self::ProviderUnavailable { retry_after, .. } => *retry_after,
            Self::TokenExchangeFailed { retry_after, .. } => *retry_after,
            Self::NetworkError { retry_count, .. } => {
                // Exponential backoff: 2^retry_count seconds, max 300 seconds (5 minutes)
                Some(std::cmp::min(2_u64.pow(*retry_count), 300))
            }
            Self::ApiTimeout { .. } => Some(30), // 30 seconds for timeouts
            _ => None,
        }
    }

    /// Check if this error requires user re-authentication
    pub fn requires_reauth(&self) -> bool {
        match self {
            Self::TokenRefreshFailed {
                requires_reauth, ..
            } => *requires_reauth,
            Self::InvalidToken { .. } => true,
            Self::InsufficientScopes { .. } => true,
            Self::InvalidAuthorizationCode { .. } => true,
            _ => false,
        }
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            Self::ProviderError {
                provider, message, ..
            } => {
                format!("Authentication with {} failed: {}", provider, message)
            }
            Self::ProviderNotConfigured { provider, .. } => {
                format!("{} authentication is not available at this time", provider)
            }
            Self::InvalidConfiguration { provider, .. } => {
                format!("{} authentication is temporarily unavailable", provider)
            }
            Self::StateValidationFailed { .. } => {
                "Authentication request is invalid or expired. Please try again.".to_string()
            }
            Self::InvalidAuthorizationCode { provider, .. } => {
                format!("Authentication with {} failed. Please try again.", provider)
            }
            Self::RedirectUriMismatch { provider, .. } => {
                format!(
                    "Authentication with {} failed due to configuration error",
                    provider
                )
            }
            Self::TokenExchangeFailed { provider, .. } => {
                format!(
                    "Failed to complete authentication with {}. Please try again.",
                    provider
                )
            }
            Self::TokenRefreshFailed {
                provider,
                requires_reauth,
                ..
            } => {
                if *requires_reauth {
                    format!(
                        "Your {} authentication has expired. Please sign in again.",
                        provider
                    )
                } else {
                    format!(
                        "Failed to refresh {} authentication. Please try again.",
                        provider
                    )
                }
            }
            Self::InvalidToken { provider, .. } => {
                format!(
                    "Your {} authentication is invalid. Please sign in again.",
                    provider
                )
            }
            Self::TokenEncryptionFailed { .. } => {
                "Authentication data processing failed. Please try again.".to_string()
            }
            Self::AccountLinkingFailed {
                provider, reason, ..
            } => {
                format!("Failed to link {} account: {}", provider, reason)
            }
            Self::AccountUnlinkingFailed {
                provider, reason, ..
            } => {
                format!("Failed to unlink {} account: {}", provider, reason)
            }
            Self::AccountMergeConflict {
                provider, reason, ..
            } => {
                format!("Account merge conflict with {}: {}", provider, reason)
            }
            Self::ProviderUnavailable { provider, .. } => {
                format!(
                    "{} authentication is temporarily unavailable. Please try again later.",
                    provider
                )
            }
            Self::RateLimitExceeded {
                provider,
                retry_after,
                ..
            } => {
                format!(
                    "{} authentication rate limit exceeded. Please try again in {} seconds.",
                    provider, retry_after
                )
            }
            Self::UserInfoRetrievalFailed { provider, .. } => {
                format!(
                    "Failed to retrieve profile information from {}. Please try again.",
                    provider
                )
            }
            Self::InsufficientScopes { provider, .. } => {
                format!("Insufficient permissions granted for {}. Please try again and grant the required permissions.", provider)
            }
            Self::SecurityViolation { .. } => {
                "Authentication request blocked for security reasons. Please try again.".to_string()
            }
            Self::CsrfAttackDetected { .. } => {
                "Authentication request blocked for security reasons. Please try again.".to_string()
            }
            Self::NetworkError { provider, .. } => {
                format!(
                    "Network error connecting to {}. Please check your connection and try again.",
                    provider
                )
            }
            Self::ApiTimeout { provider, .. } => {
                format!("{} authentication timed out. Please try again.", provider)
            }
        }
    }

    /// Get error details for debugging and logging
    pub fn error_details(&self) -> HashMap<String, serde_json::Value> {
        let mut details = HashMap::new();

        match self {
            Self::ProviderError {
                provider,
                error_code,
                details: provider_details,
                ..
            } => {
                details.insert(
                    "provider".to_string(),
                    serde_json::Value::String(provider.to_string()),
                );
                details.insert(
                    "error_code".to_string(),
                    serde_json::Value::String(error_code.clone()),
                );
                if let Some(provider_details) = provider_details {
                    details.insert("provider_details".to_string(), provider_details.clone());
                }
            }
            Self::ProviderNotConfigured {
                provider,
                missing_variables,
                ..
            } => {
                details.insert(
                    "provider".to_string(),
                    serde_json::Value::String(provider.to_string()),
                );
                details.insert(
                    "missing_variables".to_string(),
                    serde_json::Value::Array(
                        missing_variables
                            .iter()
                            .map(|v| serde_json::Value::String(v.clone()))
                            .collect(),
                    ),
                );
            }
            Self::InvalidConfiguration {
                provider,
                validation_errors,
                ..
            } => {
                details.insert(
                    "provider".to_string(),
                    serde_json::Value::String(provider.to_string()),
                );
                details.insert(
                    "validation_errors".to_string(),
                    serde_json::Value::Array(
                        validation_errors
                            .iter()
                            .map(|e| serde_json::Value::String(e.clone()))
                            .collect(),
                    ),
                );
            }
            Self::StateValidationFailed {
                expected_provider,
                received_provider,
                ..
            } => {
                if let Some(expected) = expected_provider {
                    details.insert(
                        "expected_provider".to_string(),
                        serde_json::Value::String(expected.to_string()),
                    );
                }
                if let Some(received) = received_provider {
                    details.insert(
                        "received_provider".to_string(),
                        serde_json::Value::String(received.to_string()),
                    );
                }
            }
            Self::TokenRefreshFailed {
                provider,
                requires_reauth,
                ..
            } => {
                details.insert(
                    "provider".to_string(),
                    serde_json::Value::String(provider.to_string()),
                );
                details.insert(
                    "requires_reauth".to_string(),
                    serde_json::Value::Bool(*requires_reauth),
                );
            }
            Self::RateLimitExceeded {
                provider,
                retry_after,
                limit_type,
            } => {
                details.insert(
                    "provider".to_string(),
                    serde_json::Value::String(provider.to_string()),
                );
                details.insert(
                    "retry_after".to_string(),
                    serde_json::Value::Number((*retry_after).into()),
                );
                details.insert(
                    "limit_type".to_string(),
                    serde_json::to_value(limit_type).unwrap_or_default(),
                );
            }
            Self::InsufficientScopes {
                provider,
                required_scopes,
                granted_scopes,
            } => {
                details.insert(
                    "provider".to_string(),
                    serde_json::Value::String(provider.to_string()),
                );
                details.insert(
                    "required_scopes".to_string(),
                    serde_json::Value::Array(
                        required_scopes
                            .iter()
                            .map(|s| serde_json::Value::String(s.clone()))
                            .collect(),
                    ),
                );
                details.insert(
                    "granted_scopes".to_string(),
                    serde_json::Value::Array(
                        granted_scopes
                            .iter()
                            .map(|s| serde_json::Value::String(s.clone()))
                            .collect(),
                    ),
                );
            }
            Self::SecurityViolation {
                provider,
                violation,
                client_info,
                ..
            } => {
                details.insert(
                    "provider".to_string(),
                    serde_json::Value::String(provider.to_string()),
                );
                details.insert(
                    "violation_type".to_string(),
                    serde_json::to_value(violation).unwrap_or_default(),
                );
                if let Some(client) = client_info {
                    details.insert(
                        "client_info".to_string(),
                        serde_json::to_value(client).unwrap_or_default(),
                    );
                }
            }
            Self::NetworkError {
                provider,
                retry_count,
                is_transient,
                ..
            } => {
                details.insert(
                    "provider".to_string(),
                    serde_json::Value::String(provider.to_string()),
                );
                details.insert(
                    "retry_count".to_string(),
                    serde_json::Value::Number((*retry_count).into()),
                );
                details.insert(
                    "is_transient".to_string(),
                    serde_json::Value::Bool(*is_transient),
                );
            }
            _ => {
                // Add common details for other error types
                if let Some(provider) = self.get_provider() {
                    details.insert(
                        "provider".to_string(),
                        serde_json::Value::String(provider.to_string()),
                    );
                }
            }
        }

        details.insert(
            "error_type".to_string(),
            serde_json::Value::String(self.error_type()),
        );
        details.insert(
            "is_retryable".to_string(),
            serde_json::Value::Bool(self.is_retryable()),
        );
        details.insert(
            "requires_reauth".to_string(),
            serde_json::Value::Bool(self.requires_reauth()),
        );

        if let Some(retry_delay) = self.retry_delay() {
            details.insert(
                "retry_delay_seconds".to_string(),
                serde_json::Value::Number(retry_delay.into()),
            );
        }

        details
    }

    /// Get the OAuth provider associated with this error
    pub fn get_provider(&self) -> Option<&OAuthProviderType> {
        match self {
            Self::ProviderError { provider, .. }
            | Self::ProviderNotConfigured { provider, .. }
            | Self::InvalidConfiguration { provider, .. }
            | Self::InvalidAuthorizationCode { provider, .. }
            | Self::RedirectUriMismatch { provider, .. }
            | Self::TokenExchangeFailed { provider, .. }
            | Self::TokenRefreshFailed { provider, .. }
            | Self::InvalidToken { provider, .. }
            | Self::AccountLinkingFailed { provider, .. }
            | Self::AccountUnlinkingFailed { provider, .. }
            | Self::AccountMergeConflict { provider, .. }
            | Self::ProviderUnavailable { provider, .. }
            | Self::RateLimitExceeded { provider, .. }
            | Self::UserInfoRetrievalFailed { provider, .. }
            | Self::InsufficientScopes { provider, .. }
            | Self::SecurityViolation { provider, .. }
            | Self::CsrfAttackDetected { provider, .. }
            | Self::NetworkError { provider, .. }
            | Self::ApiTimeout { provider, .. } => Some(provider),
            _ => None,
        }
    }

    /// Get a string identifier for the error type
    pub fn error_type(&self) -> String {
        match self {
            Self::ProviderError { .. } => "provider_error".to_string(),
            Self::ProviderNotConfigured { .. } => "provider_not_configured".to_string(),
            Self::InvalidConfiguration { .. } => "invalid_configuration".to_string(),
            Self::StateValidationFailed { .. } => "state_validation_failed".to_string(),
            Self::InvalidAuthorizationCode { .. } => "invalid_authorization_code".to_string(),
            Self::RedirectUriMismatch { .. } => "redirect_uri_mismatch".to_string(),
            Self::TokenExchangeFailed { .. } => "token_exchange_failed".to_string(),
            Self::TokenRefreshFailed { .. } => "token_refresh_failed".to_string(),
            Self::InvalidToken { .. } => "invalid_token".to_string(),
            Self::TokenEncryptionFailed { .. } => "token_encryption_failed".to_string(),
            Self::AccountLinkingFailed { .. } => "account_linking_failed".to_string(),
            Self::AccountUnlinkingFailed { .. } => "account_unlinking_failed".to_string(),
            Self::AccountMergeConflict { .. } => "account_merge_conflict".to_string(),
            Self::ProviderUnavailable { .. } => "provider_unavailable".to_string(),
            Self::RateLimitExceeded { .. } => "rate_limit_exceeded".to_string(),
            Self::UserInfoRetrievalFailed { .. } => "user_info_retrieval_failed".to_string(),
            Self::InsufficientScopes { .. } => "insufficient_scopes".to_string(),
            Self::SecurityViolation { .. } => "security_violation".to_string(),
            Self::CsrfAttackDetected { .. } => "csrf_attack_detected".to_string(),
            Self::NetworkError { .. } => "network_error".to_string(),
            Self::ApiTimeout { .. } => "api_timeout".to_string(),
        }
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::AccessToken => write!(f, "access_token"),
            TokenType::RefreshToken => write!(f, "refresh_token"),
            TokenType::IdToken => write!(f, "id_token"),
        }
    }
}

impl fmt::Display for EncryptionOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncryptionOperation::Encrypt => write!(f, "encrypt"),
            EncryptionOperation::Decrypt => write!(f, "decrypt"),
            EncryptionOperation::KeyRotation => write!(f, "key_rotation"),
        }
    }
}

impl fmt::Display for AccountConflictType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccountConflictType::AlreadyLinkedToSameUser => {
                write!(f, "already_linked_to_same_user")
            }
            AccountConflictType::AlreadyLinkedToDifferentUser => {
                write!(f, "already_linked_to_different_user")
            }
            AccountConflictType::EmailConflict => write!(f, "email_conflict"),
            AccountConflictType::ProviderUserIdConflict => write!(f, "provider_user_id_conflict"),
        }
    }
}

impl fmt::Display for RateLimitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RateLimitType::ApiCalls => write!(f, "api_calls"),
            RateLimitType::TokenRequests => write!(f, "token_requests"),
            RateLimitType::UserInfoRequests => write!(f, "user_info_requests"),
            RateLimitType::AuthorizationRequests => write!(f, "authorization_requests"),
        }
    }
}

impl fmt::Display for SecurityViolationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityViolationType::StateParameterMissing => write!(f, "state_parameter_missing"),
            SecurityViolationType::StateParameterTampered => write!(f, "state_parameter_tampered"),
            SecurityViolationType::UnexpectedRedirectUri => write!(f, "unexpected_redirect_uri"),
            SecurityViolationType::SuspiciousClientBehavior => {
                write!(f, "suspicious_client_behavior")
            }
            SecurityViolationType::TokenReplayAttack => write!(f, "token_replay_attack"),
            SecurityViolationType::InvalidSignature => write!(f, "invalid_signature"),
        }
    }
}

/// Helper function to create client info from HTTP request context
pub fn create_client_info(
    ip_address: Option<String>,
    user_agent: Option<String>,
    request_id: Option<String>,
) -> ClientInfo {
    ClientInfo {
        ip_address,
        user_agent,
        request_id,
        timestamp: chrono::Utc::now(),
    }
}

/// Helper function to parse provider-specific error responses
pub fn parse_provider_error(
    provider: OAuthProviderType,
    status_code: u16,
    response_body: &str,
) -> OAuthError {
    // Try to parse as JSON first
    if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(response_body) {
        let error_code = error_json["error"]
            .as_str()
            .unwrap_or("unknown_error")
            .to_string();
        let error_description = error_json["error_description"]
            .as_str()
            .or_else(|| error_json["message"].as_str())
            .unwrap_or(response_body)
            .to_string();

        // Handle common OAuth error codes
        match error_code.as_str() {
            "invalid_grant" | "bad_verification_code" => OAuthError::InvalidAuthorizationCode {
                provider,
                error_code: Some(error_code),
                error_description: Some(error_description),
            },
            "invalid_client" | "incorrect_client_credentials" => OAuthError::InvalidConfiguration {
                provider,
                reason: "Invalid client credentials".to_string(),
                validation_errors: vec![error_description],
            },
            "redirect_uri_mismatch" => OAuthError::RedirectUriMismatch {
                provider,
                expected: "configured redirect URI".to_string(),
                received: "request redirect URI".to_string(),
            },
            "temporarily_unavailable" | "server_error" => {
                OAuthError::ProviderUnavailable {
                    provider,
                    reason: error_description,
                    estimated_recovery: None,
                    retry_after: Some(60), // Default 1 minute retry
                }
            }
            _ => OAuthError::provider_error_with_details(
                provider,
                error_code,
                error_description,
                error_json,
            ),
        }
    } else {
        // Fallback for non-JSON responses
        let error_code = match status_code {
            400 => "bad_request",
            401 => "unauthorized",
            403 => "forbidden",
            404 => "not_found",
            429 => "rate_limit_exceeded",
            500..=599 => "server_error",
            _ => "unknown_error",
        };

        OAuthError::provider_error(provider, error_code, response_body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_error_creation() {
        let error = OAuthError::provider_error(
            OAuthProviderType::Google,
            "invalid_grant",
            "Authorization code expired",
        );

        assert_eq!(error.get_provider(), Some(&OAuthProviderType::Google));
        assert_eq!(error.error_type(), "provider_error");
        assert!(!error.is_retryable());
        assert!(!error.requires_reauth());
    }

    #[test]
    fn test_oauth_error_retryable() {
        let error = OAuthError::NetworkError {
            provider: OAuthProviderType::GitHub,
            reason: "Connection timeout".to_string(),
            is_transient: true,
            retry_count: 1,
        };

        assert!(error.is_retryable());
        assert_eq!(error.retry_delay(), Some(2)); // 2^1 = 2 seconds
    }

    #[test]
    fn test_oauth_error_requires_reauth() {
        let error = OAuthError::TokenRefreshFailed {
            provider: OAuthProviderType::Apple,
            reason: "Refresh token expired".to_string(),
            requires_reauth: true,
        };

        assert!(error.requires_reauth());
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_oauth_error_user_message() {
        let error = OAuthError::ProviderUnavailable {
            provider: OAuthProviderType::Google,
            reason: "Service maintenance".to_string(),
            estimated_recovery: None,
            retry_after: Some(300),
        };

        let message = error.user_message();
        assert!(message.contains("Google"));
        assert!(message.contains("temporarily unavailable"));
    }

    #[test]
    fn test_oauth_error_details() {
        let error = OAuthError::RateLimitExceeded {
            provider: OAuthProviderType::GitHub,
            retry_after: 60,
            limit_type: RateLimitType::ApiCalls,
        };

        let details = error.error_details();
        assert_eq!(
            details.get("provider").unwrap(),
            &serde_json::Value::String("GitHub".to_string())
        );
        assert_eq!(
            details.get("retry_after").unwrap(),
            &serde_json::Value::Number(60.into())
        );
        assert_eq!(
            details.get("is_retryable").unwrap(),
            &serde_json::Value::Bool(true)
        );
    }

    #[test]
    fn test_parse_provider_error() {
        let json_response = r#"{"error": "invalid_grant", "error_description": "The authorization code is invalid"}"#;
        let error = parse_provider_error(OAuthProviderType::Google, 400, json_response);

        match error {
            OAuthError::InvalidAuthorizationCode {
                provider,
                error_code,
                ..
            } => {
                assert_eq!(provider, OAuthProviderType::Google);
                assert_eq!(error_code, Some("invalid_grant".to_string()));
            }
            _ => panic!("Expected InvalidAuthorizationCode error"),
        }
    }

    #[test]
    fn test_client_info_creation() {
        let client_info = create_client_info(
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
            Some("req-123".to_string()),
        );

        assert_eq!(client_info.ip_address, Some("192.168.1.1".to_string()));
        assert_eq!(client_info.user_agent, Some("Mozilla/5.0".to_string()));
        assert_eq!(client_info.request_id, Some("req-123".to_string()));
    }
}
