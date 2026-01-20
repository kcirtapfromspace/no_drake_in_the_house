//! Comprehensive error handling for the application

pub mod oauth;

use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;
use validator::ValidationErrors;

pub use oauth::{
    AccountConflictType, ClientInfo, EncryptionOperation, OAuthError, RateLimitType,
    SecurityViolationType, TokenType,
};

/// Error response structure for consistent API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub error_code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub correlation_id: String,
    pub timestamp: String,
}

/// Main application error type with comprehensive error handling
#[derive(Debug, Error)]
pub enum AppError {
    // Authentication errors
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Authentication token required")]
    TokenRequired,

    #[error("Authentication token expired")]
    TokenExpired,

    #[error("Authentication token invalid")]
    TokenInvalid,

    #[error("Two-factor authentication required")]
    TwoFactorRequired,

    #[error("Two-factor authentication code invalid")]
    TwoFactorInvalid,

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    // OAuth errors - comprehensive OAuth error handling
    #[error("OAuth error: {0}")]
    OAuth(#[from] OAuthError),

    // Legacy OAuth errors (deprecated - use OAuth(OAuthError) instead)
    #[error("OAuth provider error: {provider} - {message}")]
    OAuthProviderError { provider: String, message: String },

    #[error("OAuth state validation failed")]
    OAuthStateValidationFailed,

    #[error("OAuth account linking failed: {reason}")]
    OAuthAccountLinkingFailed { reason: String },

    #[error("OAuth account merge conflict: {reason}")]
    OAuthAccountMergeConflict { reason: String },

    #[error("Account merge unavailable: {reason}")]
    AccountMergeUnavailable { reason: String },

    // Encryption errors
    #[error("Token encryption error: {0}")]
    EncryptionError(String),

    // Validation errors
    #[error("Request validation failed")]
    ValidationFailed(ValidationErrors),

    #[error("Invalid request format")]
    InvalidRequestFormat(String),

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid field value: {field}")]
    InvalidFieldValue { field: String, message: String },

    // Registration-specific validation errors
    #[error("Registration validation failed")]
    RegistrationValidationError {
        errors: Vec<crate::models::RegistrationValidationError>,
    },

    #[error("Password confirmation does not match")]
    PasswordMismatch,

    #[error("Terms of service must be accepted")]
    TermsNotAccepted,

    #[error("Email already registered")]
    EmailAlreadyRegistered,

    #[error("Password does not meet security requirements")]
    WeakPassword { requirements: Vec<String> },

    // Resource errors
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },

    #[error("Resource already exists: {resource}")]
    AlreadyExists { resource: String },

    #[error("Resource conflict: {message}")]
    Conflict { message: String },

    // Rate limiting
    #[error("Rate limit exceeded")]
    RateLimitExceeded { retry_after: Option<u64> },

    // External service errors
    #[error("External service unavailable: {service}")]
    ExternalServiceUnavailable { service: String },

    #[error("External service error: {0}")]
    ExternalServiceError(String),

    // Database errors
    #[error("Database connection failed")]
    DatabaseConnectionFailed,

    #[error("Database query failed")]
    DatabaseQueryFailed(sqlx::Error),

    #[error("Database transaction failed")]
    DatabaseTransactionFailed,

    #[error("Database constraint violation")]
    DatabaseConstraintViolation(String),

    // Redis errors
    #[error("Redis connection failed")]
    RedisConnectionFailed,

    #[error("Redis operation failed")]
    RedisOperationFailed(String),

    // Business logic errors
    #[error("Business rule violation: {rule}")]
    BusinessRuleViolation { rule: String },

    #[error("Operation not allowed: {reason}")]
    OperationNotAllowed { reason: String },

    // System errors
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Internal server error")]
    Internal { message: Option<String> },

    #[error("Service unavailable")]
    ServiceUnavailable,

    // JSON parsing errors
    #[error("JSON parsing error")]
    JsonParsingError(#[from] JsonRejection),
}

impl AppError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            // 400 Bad Request
            AppError::ValidationFailed(_)
            | AppError::InvalidRequestFormat(_)
            | AppError::MissingField { .. }
            | AppError::InvalidFieldValue { .. }
            | AppError::JsonParsingError(_)
            | AppError::RegistrationValidationError { .. }
            | AppError::PasswordMismatch
            | AppError::TermsNotAccepted
            | AppError::WeakPassword { .. } => StatusCode::BAD_REQUEST,

            // 401 Unauthorized
            AppError::InvalidCredentials
            | AppError::TokenRequired
            | AppError::TokenExpired
            | AppError::TokenInvalid
            | AppError::TwoFactorRequired
            | AppError::TwoFactorInvalid
            | AppError::OAuthStateValidationFailed => StatusCode::UNAUTHORIZED,

            // OAuth-specific status codes
            AppError::OAuth(oauth_error) => match oauth_error {
                OAuthError::StateValidationFailed { .. }
                | OAuthError::InvalidToken { .. }
                | OAuthError::SecurityViolation { .. }
                | OAuthError::CsrfAttackDetected { .. } => StatusCode::UNAUTHORIZED,

                OAuthError::InvalidConfiguration { .. }
                | OAuthError::ProviderNotConfigured { .. }
                | OAuthError::InvalidAuthorizationCode { .. }
                | OAuthError::RedirectUriMismatch { .. }
                | OAuthError::InsufficientScopes { .. } => StatusCode::BAD_REQUEST,

                OAuthError::AccountLinkingFailed { .. }
                | OAuthError::AccountUnlinkingFailed { .. }
                | OAuthError::AccountMergeConflict { .. } => StatusCode::CONFLICT,

                OAuthError::RateLimitExceeded { .. } => StatusCode::TOO_MANY_REQUESTS,

                OAuthError::ProviderUnavailable { .. }
                | OAuthError::NetworkError { .. }
                | OAuthError::ApiTimeout { .. } => StatusCode::SERVICE_UNAVAILABLE,

                OAuthError::ProviderError { .. }
                | OAuthError::TokenExchangeFailed { .. }
                | OAuthError::TokenRefreshFailed { .. }
                | OAuthError::UserInfoRetrievalFailed { .. } => StatusCode::BAD_GATEWAY,

                OAuthError::TokenEncryptionFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            },

            // 403 Forbidden
            AppError::InsufficientPermissions => StatusCode::FORBIDDEN,

            // 404 Not Found
            AppError::NotFound { .. } => StatusCode::NOT_FOUND,

            // 409 Conflict
            AppError::AlreadyExists { .. }
            | AppError::Conflict { .. }
            | AppError::DatabaseConstraintViolation(_)
            | AppError::EmailAlreadyRegistered => StatusCode::CONFLICT,

            // 422 Unprocessable Entity
            AppError::BusinessRuleViolation { .. }
            | AppError::OperationNotAllowed { .. }
            | AppError::OAuthAccountLinkingFailed { .. }
            | AppError::OAuthAccountMergeConflict { .. } => StatusCode::UNPROCESSABLE_ENTITY,

            // 429 Too Many Requests
            AppError::RateLimitExceeded { .. } => StatusCode::TOO_MANY_REQUESTS,

            // 502 Bad Gateway
            AppError::ExternalServiceUnavailable { .. }
            | AppError::ExternalServiceError(_)
            | AppError::OAuthProviderError { .. } => StatusCode::BAD_GATEWAY,

            // 503 Service Unavailable
            AppError::ServiceUnavailable
            | AppError::AccountMergeUnavailable { .. }
            | AppError::DatabaseConnectionFailed
            | AppError::RedisConnectionFailed => StatusCode::SERVICE_UNAVAILABLE,

            // 500 Internal Server Error
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get the error code for this error
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::InvalidCredentials => "AUTH_INVALID_CREDENTIALS",
            AppError::TokenRequired => "AUTH_TOKEN_REQUIRED",
            AppError::TokenExpired => "AUTH_TOKEN_EXPIRED",
            AppError::TokenInvalid => "AUTH_TOKEN_INVALID",
            AppError::TwoFactorRequired => "AUTH_2FA_REQUIRED",
            AppError::TwoFactorInvalid => "AUTH_2FA_INVALID",
            AppError::InsufficientPermissions => "AUTH_INSUFFICIENT_PERMISSIONS",
            AppError::OAuth(oauth_error) => match oauth_error.error_type().as_str() {
                "provider_error" => "OAUTH_PROVIDER_ERROR",
                "provider_not_configured" => "OAUTH_PROVIDER_NOT_CONFIGURED",
                "invalid_configuration" => "OAUTH_INVALID_CONFIGURATION",
                "state_validation_failed" => "OAUTH_STATE_VALIDATION_FAILED",
                "invalid_authorization_code" => "OAUTH_INVALID_AUTHORIZATION_CODE",
                "redirect_uri_mismatch" => "OAUTH_REDIRECT_URI_MISMATCH",
                "token_exchange_failed" => "OAUTH_TOKEN_EXCHANGE_FAILED",
                "token_refresh_failed" => "OAUTH_TOKEN_REFRESH_FAILED",
                "invalid_token" => "OAUTH_INVALID_TOKEN",
                "token_encryption_failed" => "OAUTH_TOKEN_ENCRYPTION_FAILED",
                "account_linking_failed" => "OAUTH_ACCOUNT_LINKING_FAILED",
                "account_unlinking_failed" => "OAUTH_ACCOUNT_UNLINKING_FAILED",
                "account_merge_conflict" => "OAUTH_ACCOUNT_MERGE_CONFLICT",
                "provider_unavailable" => "OAUTH_PROVIDER_UNAVAILABLE",
                "rate_limit_exceeded" => "OAUTH_RATE_LIMIT_EXCEEDED",
                "user_info_retrieval_failed" => "OAUTH_USER_INFO_RETRIEVAL_FAILED",
                "insufficient_scopes" => "OAUTH_INSUFFICIENT_SCOPES",
                "security_violation" => "OAUTH_SECURITY_VIOLATION",
                "csrf_attack_detected" => "OAUTH_CSRF_ATTACK_DETECTED",
                "network_error" => "OAUTH_NETWORK_ERROR",
                "api_timeout" => "OAUTH_API_TIMEOUT",
                _ => "OAUTH_UNKNOWN_ERROR",
            },
            AppError::OAuthProviderError { .. } => "OAUTH_PROVIDER_ERROR",
            AppError::OAuthStateValidationFailed => "OAUTH_STATE_VALIDATION_FAILED",
            AppError::OAuthAccountLinkingFailed { .. } => "OAUTH_ACCOUNT_LINKING_FAILED",
            AppError::OAuthAccountMergeConflict { .. } => "OAUTH_ACCOUNT_MERGE_CONFLICT",
            AppError::AccountMergeUnavailable { .. } => "ACCOUNT_MERGE_UNAVAILABLE",
            AppError::EncryptionError(_) => "ENCRYPTION_ERROR",
            AppError::ValidationFailed(_) => "VALIDATION_FAILED",
            AppError::InvalidRequestFormat(_) => "INVALID_REQUEST_FORMAT",
            AppError::MissingField { .. } => "MISSING_FIELD",
            AppError::InvalidFieldValue { .. } => "INVALID_FIELD_VALUE",
            AppError::RegistrationValidationError { .. } => "REGISTRATION_VALIDATION_ERROR",
            AppError::PasswordMismatch => "PASSWORD_MISMATCH",
            AppError::TermsNotAccepted => "TERMS_NOT_ACCEPTED",
            AppError::EmailAlreadyRegistered => "EMAIL_ALREADY_REGISTERED",
            AppError::WeakPassword { .. } => "WEAK_PASSWORD",
            AppError::NotFound { .. } => "RESOURCE_NOT_FOUND",
            AppError::AlreadyExists { .. } => "RESOURCE_ALREADY_EXISTS",
            AppError::Conflict { .. } => "RESOURCE_CONFLICT",
            AppError::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            AppError::ExternalServiceUnavailable { .. } => "EXTERNAL_SERVICE_UNAVAILABLE",
            AppError::ExternalServiceError { .. } => "EXTERNAL_SERVICE_ERROR",
            AppError::DatabaseConnectionFailed => "DATABASE_CONNECTION_FAILED",
            AppError::DatabaseQueryFailed(_) => "DATABASE_QUERY_FAILED",
            AppError::DatabaseTransactionFailed => "DATABASE_TRANSACTION_FAILED",
            AppError::DatabaseConstraintViolation(_) => "DATABASE_CONSTRAINT_VIOLATION",
            AppError::RedisConnectionFailed => "REDIS_CONNECTION_FAILED",
            AppError::RedisOperationFailed(_) => "REDIS_OPERATION_FAILED",
            AppError::BusinessRuleViolation { .. } => "BUSINESS_RULE_VIOLATION",
            AppError::OperationNotAllowed { .. } => "OPERATION_NOT_ALLOWED",
            AppError::ConfigurationError { .. } => "CONFIGURATION_ERROR",
            AppError::Internal { .. } => "INTERNAL_SERVER_ERROR",
            AppError::ServiceUnavailable => "SERVICE_UNAVAILABLE",
            AppError::JsonParsingError(_) => "JSON_PARSING_ERROR",
        }
    }

    /// Get user-friendly message for this error
    pub fn user_message(&self) -> String {
        match self {
            AppError::InvalidCredentials => "Invalid email or password".to_string(),
            AppError::TokenRequired => "Authentication required".to_string(),
            AppError::TokenExpired => "Session expired, please log in again".to_string(),
            AppError::TokenInvalid => "Invalid authentication token".to_string(),
            AppError::TwoFactorRequired => "Two-factor authentication code required".to_string(),
            AppError::TwoFactorInvalid => "Invalid two-factor authentication code".to_string(),
            AppError::InsufficientPermissions => {
                "You don't have permission to perform this action".to_string()
            }
            AppError::ValidationFailed(_) => "Please check your input and try again".to_string(),
            AppError::InvalidRequestFormat(msg) => format!("Invalid request format: {}", msg),
            AppError::MissingField { field } => format!("Missing required field: {}", field),
            AppError::InvalidFieldValue { field, message } => {
                format!("Invalid value for {}: {}", field, message)
            }
            AppError::RegistrationValidationError { .. } => {
                "Registration validation failed. Please check your input and try again.".to_string()
            }
            AppError::PasswordMismatch => "Password confirmation does not match".to_string(),
            AppError::TermsNotAccepted => {
                "You must accept the terms of service to register".to_string()
            }
            AppError::EmailAlreadyRegistered => {
                "An account with this email address already exists".to_string()
            }
            AppError::WeakPassword { .. } => {
                "Password does not meet security requirements".to_string()
            }
            AppError::NotFound { resource } => format!("{} not found", resource),
            AppError::AlreadyExists { resource } => format!("{} already exists", resource),
            AppError::Conflict { message } => message.clone(),
            AppError::RateLimitExceeded { .. } => {
                "Too many requests, please try again later".to_string()
            }
            AppError::ExternalServiceUnavailable { service } => {
                format!("{} is currently unavailable", service)
            }
            AppError::ExternalServiceError(msg) => format!("External service error: {}", msg),
            AppError::OAuth(oauth_error) => oauth_error.user_message(),
            AppError::OAuthProviderError { provider, .. } => {
                format!("Authentication with {} failed", provider)
            }
            AppError::OAuthStateValidationFailed => {
                "OAuth authentication failed due to invalid state".to_string()
            }
            AppError::OAuthAccountLinkingFailed { reason } => {
                format!("Account linking failed: {}", reason)
            }
            AppError::OAuthAccountMergeConflict { reason } => {
                format!("Account merge conflict: {}", reason)
            }
            AppError::AccountMergeUnavailable { .. } => {
                "Account merge is currently unavailable. Please try again later.".to_string()
            }
            AppError::EncryptionError(_) => "Token encryption/decryption failed".to_string(),
            AppError::BusinessRuleViolation { rule } => {
                format!("Business rule violation: {}", rule)
            }
            AppError::OperationNotAllowed { reason } => {
                format!("Operation not allowed: {}", reason)
            }
            _ => "An unexpected error occurred".to_string(),
        }
    }

    /// Get error details for debugging
    pub fn error_details(&self) -> Option<serde_json::Value> {
        match self {
            AppError::ValidationFailed(errors) => {
                let mut details = HashMap::new();
                for (field, field_errors) in errors.field_errors() {
                    let messages: Vec<String> = field_errors
                        .iter()
                        .map(|e| {
                            e.message
                                .as_ref()
                                .map(|m| m.to_string())
                                .unwrap_or_else(|| "Invalid value".to_string())
                        })
                        .collect();
                    details.insert(field.to_string(), messages);
                }
                Some(json!(details))
            }
            AppError::RegistrationValidationError { errors } => Some(json!({
                "validation_errors": errors
            })),
            AppError::WeakPassword { requirements } => Some(json!({
                "password_requirements": requirements
            })),
            AppError::RateLimitExceeded { retry_after } => Some(json!({
                "retry_after_seconds": retry_after
            })),
            AppError::DatabaseQueryFailed(e) => Some(json!({
                "database_error": e.to_string()
            })),
            AppError::ExternalServiceError(message) => Some(json!({
                "service_message": message
            })),
            AppError::OAuth(oauth_error) => {
                let details = oauth_error.error_details();
                if details.is_empty() {
                    None
                } else {
                    Some(serde_json::to_value(details).unwrap_or_default())
                }
            }
            AppError::OAuthProviderError { provider, message } => Some(json!({
                "provider": provider,
                "provider_message": message
            })),
            AppError::OAuthAccountLinkingFailed { reason } => Some(json!({
                "linking_failure_reason": reason
            })),
            AppError::OAuthAccountMergeConflict { reason } => Some(json!({
                "merge_conflict_reason": reason
            })),
            AppError::AccountMergeUnavailable { reason } => Some(json!({
                "merge_unavailable_reason": reason
            })),
            AppError::EncryptionError(message) => Some(json!({
                "encryption_error": message
            })),
            _ => None,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let correlation_id = Uuid::new_v4().to_string();
        let status = self.status_code();
        let error_code = self.error_code();
        let user_message = self.user_message();
        let details = self.error_details();

        // Log error with correlation ID for debugging
        match &self {
            AppError::Internal { .. }
            | AppError::DatabaseConnectionFailed
            | AppError::DatabaseQueryFailed(_)
            | AppError::DatabaseTransactionFailed
            | AppError::RedisConnectionFailed
            | AppError::RedisOperationFailed(_)
            | AppError::ConfigurationError { .. }
            | AppError::ServiceUnavailable
            | AppError::AccountMergeUnavailable { .. } => {
                tracing::error!(
                    correlation_id = %correlation_id,
                    error_code = %error_code,
                    error = %self,
                    "Server error occurred"
                );
            }
            AppError::ExternalServiceError(_)
            | AppError::ExternalServiceUnavailable { .. }
            | AppError::OAuthProviderError { .. } => {
                tracing::warn!(
                    correlation_id = %correlation_id,
                    error_code = %error_code,
                    error = %self,
                    "External service error"
                );
            }
            AppError::OAuth(oauth_error) => {
                match oauth_error {
                    // Security-related OAuth errors should be logged as warnings with details
                    OAuthError::SecurityViolation { .. }
                    | OAuthError::CsrfAttackDetected { .. } => {
                        tracing::warn!(
                            correlation_id = %correlation_id,
                            error_code = %error_code,
                            error = %self,
                            oauth_details = ?oauth_error.error_details(),
                            "OAuth security violation detected"
                        );
                    }
                    // Configuration and provider errors
                    OAuthError::ProviderNotConfigured { .. }
                    | OAuthError::InvalidConfiguration { .. } => {
                        tracing::error!(
                            correlation_id = %correlation_id,
                            error_code = %error_code,
                            error = %self,
                            "OAuth configuration error"
                        );
                    }
                    // Network and availability errors
                    OAuthError::ProviderUnavailable { .. }
                    | OAuthError::NetworkError { .. }
                    | OAuthError::ApiTimeout { .. } => {
                        tracing::warn!(
                            correlation_id = %correlation_id,
                            error_code = %error_code,
                            error = %self,
                            "OAuth provider availability issue"
                        );
                    }
                    // Token encryption errors are internal server errors
                    OAuthError::TokenEncryptionFailed { .. } => {
                        tracing::error!(
                            correlation_id = %correlation_id,
                            error_code = %error_code,
                            error = %self,
                            "OAuth token encryption error"
                        );
                    }
                    // Other OAuth errors are client-related
                    _ => {
                        tracing::info!(
                            correlation_id = %correlation_id,
                            error_code = %error_code,
                            error = %self,
                            oauth_provider = ?oauth_error.get_provider(),
                            "OAuth client error"
                        );
                    }
                }
            }
            _ => {
                tracing::info!(
                    correlation_id = %correlation_id,
                    error_code = %error_code,
                    error = %self,
                    "Client error occurred"
                );
            }
        }

        let error_response = ErrorResponse {
            error: self.to_string(),
            error_code: error_code.to_string(),
            message: user_message,
            details,
            correlation_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        (status, Json(error_response)).into_response()
    }
}

// Conversion implementations for common error types
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::Database(db_err) => {
                if db_err.is_unique_violation() {
                    AppError::DatabaseConstraintViolation("Unique constraint violation".to_string())
                } else if db_err.is_foreign_key_violation() {
                    AppError::DatabaseConstraintViolation(
                        "Foreign key constraint violation".to_string(),
                    )
                } else if db_err.is_check_violation() {
                    AppError::DatabaseConstraintViolation("Check constraint violation".to_string())
                } else {
                    AppError::DatabaseQueryFailed(err)
                }
            }
            sqlx::Error::PoolTimedOut => AppError::DatabaseConnectionFailed,
            sqlx::Error::PoolClosed => AppError::DatabaseConnectionFailed,
            _ => AppError::DatabaseQueryFailed(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_merge_unavailable_contract_is_stable() {
        let reason = "maintenance window".to_string();
        let error = AppError::AccountMergeUnavailable {
            reason: reason.clone(),
        };

        assert_eq!(error.status_code(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(error.error_code(), "ACCOUNT_MERGE_UNAVAILABLE");
        assert_eq!(
            error.user_message(),
            "Account merge is currently unavailable. Please try again later."
        );

        let details = error.error_details().expect("merge unavailable details");
        assert_eq!(
            details
                .get("merge_unavailable_reason")
                .and_then(|value| value.as_str()),
            Some(reason.as_str())
        );
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        match err.kind() {
            redis::ErrorKind::IoError => AppError::RedisConnectionFailed,
            redis::ErrorKind::AuthenticationFailed => AppError::RedisConnectionFailed,
            _ => AppError::RedisOperationFailed(err.to_string()),
        }
    }
}

impl From<deadpool::managed::PoolError<redis::RedisError>> for AppError {
    fn from(err: deadpool::managed::PoolError<redis::RedisError>) -> Self {
        match err {
            deadpool::managed::PoolError::Backend(redis_err) => AppError::from(redis_err),
            deadpool::managed::PoolError::Timeout(_) => AppError::RedisConnectionFailed,
            deadpool::managed::PoolError::Closed => AppError::RedisConnectionFailed,
            _ => AppError::RedisConnectionFailed,
        }
    }
}

impl From<ValidationErrors> for AppError {
    fn from(err: ValidationErrors) -> Self {
        AppError::ValidationFailed(err)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal {
            message: Some(err.to_string()),
        }
    }
}

impl From<axum::http::StatusCode> for AppError {
    fn from(status: axum::http::StatusCode) -> Self {
        match status {
            axum::http::StatusCode::BAD_REQUEST => {
                AppError::InvalidRequestFormat("Bad request".to_string())
            }
            axum::http::StatusCode::UNAUTHORIZED => AppError::InvalidCredentials,
            axum::http::StatusCode::FORBIDDEN => AppError::InsufficientPermissions,
            axum::http::StatusCode::NOT_FOUND => AppError::NotFound {
                resource: "Resource".to_string(),
            },
            axum::http::StatusCode::CONFLICT => AppError::Conflict {
                message: "Conflict".to_string(),
            },
            axum::http::StatusCode::TOO_MANY_REQUESTS => {
                AppError::RateLimitExceeded { retry_after: None }
            }
            axum::http::StatusCode::INTERNAL_SERVER_ERROR => AppError::Internal { message: None },
            axum::http::StatusCode::SERVICE_UNAVAILABLE => AppError::ServiceUnavailable,
            _ => AppError::Internal {
                message: Some(format!("HTTP status: {}", status)),
            },
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::InvalidRequestFormat(format!("JSON error: {}", err))
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        AppError::Internal {
            message: Some(format!("Password hashing error: {}", err)),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
            jsonwebtoken::errors::ErrorKind::InvalidToken => AppError::TokenInvalid,
            _ => AppError::Internal {
                message: Some(format!("JWT error: {}", err)),
            },
        }
    }
}

impl From<uuid::Error> for AppError {
    fn from(err: uuid::Error) -> Self {
        AppError::Internal {
            message: Some(format!("UUID parsing error: {}", err)),
        }
    }
}

/// Result type alias for application errors
pub type Result<T> = std::result::Result<T, AppError>;
