//! Comprehensive tests for error handling and validation

use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
    response::Response,
    Json,
};
use music_streaming_blocklist_backend::{
    liveness_check, readiness_check, retry_database_operation, retry_redis_operation, AppError,
    ErrorResponse, HealthCheckConfig, HealthChecker, Result, RetryConfig, ValidatedAddToDnpRequest,
    ValidatedCreateUserRequest, ValidatedJson, ValidatedLoginRequest,
};
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;
use validator::Validate;

#[tokio::test]
async fn test_app_error_status_codes() {
    // Test authentication errors
    assert_eq!(
        AppError::InvalidCredentials.status_code(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        AppError::TokenExpired.status_code(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        AppError::TwoFactorRequired.status_code(),
        StatusCode::UNAUTHORIZED
    );

    // Test validation errors
    let validation_errors = validator::ValidationErrors::new();
    assert_eq!(
        AppError::ValidationFailed(validation_errors).status_code(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        AppError::MissingField {
            field: "email".to_string()
        }
        .status_code(),
        StatusCode::BAD_REQUEST
    );

    // Test resource errors
    assert_eq!(
        AppError::NotFound {
            resource: "user".to_string()
        }
        .status_code(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        AppError::AlreadyExists {
            resource: "user".to_string()
        }
        .status_code(),
        StatusCode::CONFLICT
    );

    // Test rate limiting
    assert_eq!(
        AppError::RateLimitExceeded {
            retry_after: Some(60)
        }
        .status_code(),
        StatusCode::TOO_MANY_REQUESTS
    );

    // Test system errors
    assert_eq!(
        AppError::DatabaseConnectionFailed.status_code(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        AppError::Internal { message: None }.status_code(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[tokio::test]
async fn test_app_error_codes() {
    assert_eq!(
        AppError::InvalidCredentials.error_code(),
        "AUTH_INVALID_CREDENTIALS"
    );
    assert_eq!(
        AppError::ValidationFailed(validator::ValidationErrors::new()).error_code(),
        "VALIDATION_FAILED"
    );
    assert_eq!(
        AppError::NotFound {
            resource: "user".to_string()
        }
        .error_code(),
        "RESOURCE_NOT_FOUND"
    );
    assert_eq!(
        AppError::RateLimitExceeded { retry_after: None }.error_code(),
        "RATE_LIMIT_EXCEEDED"
    );
    assert_eq!(
        AppError::DatabaseConnectionFailed.error_code(),
        "DATABASE_CONNECTION_FAILED"
    );
}

#[tokio::test]
async fn test_app_error_user_messages() {
    assert_eq!(
        AppError::InvalidCredentials.user_message(),
        "Invalid email or password"
    );
    assert_eq!(
        AppError::TokenExpired.user_message(),
        "Session expired, please log in again"
    );
    assert_eq!(
        AppError::TwoFactorRequired.user_message(),
        "Two-factor authentication code required"
    );
    assert_eq!(
        AppError::NotFound {
            resource: "User".to_string()
        }
        .user_message(),
        "User not found"
    );
    assert_eq!(
        AppError::RateLimitExceeded {
            retry_after: Some(60)
        }
        .user_message(),
        "Too many requests, please try again later"
    );
}

#[tokio::test]
async fn test_app_error_details() {
    // Test validation error details
    let mut validation_errors = validator::ValidationErrors::new();
    validation_errors.add("email", validator::ValidationError::new("invalid_email"));
    let error = AppError::ValidationFailed(validation_errors);
    let details = error.error_details();
    assert!(details.is_some());

    // Test rate limit error details
    let error = AppError::RateLimitExceeded {
        retry_after: Some(60),
    };
    let details = error.error_details();
    assert!(details.is_some());
    assert_eq!(details.unwrap()["retry_after_seconds"], 60);
}

#[test]
fn test_validated_create_user_request() {
    // Valid request
    let valid_request = ValidatedCreateUserRequest {
        email: "test@example.com".to_string(),
        password: "SecurePass123!".to_string(),
    };
    assert!(valid_request.validate().is_ok());

    // Invalid email
    let invalid_email = ValidatedCreateUserRequest {
        email: "invalid-email".to_string(),
        password: "SecurePass123!".to_string(),
    };
    assert!(invalid_email.validate().is_err());

    // Weak password
    let weak_password = ValidatedCreateUserRequest {
        email: "test@example.com".to_string(),
        password: "weak".to_string(),
    };
    assert!(weak_password.validate().is_err());

    // Password missing uppercase
    let no_uppercase = ValidatedCreateUserRequest {
        email: "test@example.com".to_string(),
        password: "lowercase123!".to_string(),
    };
    assert!(no_uppercase.validate().is_err());

    // Password missing special character
    let no_special = ValidatedCreateUserRequest {
        email: "test@example.com".to_string(),
        password: "NoSpecial123".to_string(),
    };
    assert!(no_special.validate().is_err());
}

#[test]
fn test_validated_login_request() {
    // Valid request without TOTP
    let valid_request = ValidatedLoginRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        totp_code: None,
    };
    assert!(valid_request.validate().is_ok());

    // Valid request with TOTP
    let valid_with_totp = ValidatedLoginRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        totp_code: Some("123456".to_string()),
    };
    assert!(valid_with_totp.validate().is_ok());

    // Invalid TOTP code (too short)
    let invalid_totp = ValidatedLoginRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        totp_code: Some("12345".to_string()),
    };
    assert!(invalid_totp.validate().is_err());

    // Invalid TOTP code (contains letters)
    let invalid_totp_format = ValidatedLoginRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        totp_code: Some("12345a".to_string()),
    };
    assert!(invalid_totp_format.validate().is_err());
}

#[test]
fn test_validated_add_to_dnp_request() {
    // Valid request
    let valid_request = ValidatedAddToDnpRequest {
        user_id: Uuid::new_v4().to_string(),
        artist_name: "Test Artist".to_string(),
        tags: Some(vec!["rock".to_string(), "classic".to_string()]),
        note: Some("Great artist but not my style".to_string()),
    };
    assert!(valid_request.validate().is_ok());

    // Invalid user ID
    let invalid_user_id = ValidatedAddToDnpRequest {
        user_id: "not-a-uuid".to_string(),
        artist_name: "Test Artist".to_string(),
        tags: None,
        note: None,
    };
    assert!(invalid_user_id.validate().is_err());

    // Empty artist name
    let empty_artist = ValidatedAddToDnpRequest {
        user_id: Uuid::new_v4().to_string(),
        artist_name: "".to_string(),
        tags: None,
        note: None,
    };
    assert!(empty_artist.validate().is_err());

    // Artist name with invalid characters
    let invalid_chars = ValidatedAddToDnpRequest {
        user_id: Uuid::new_v4().to_string(),
        artist_name: "Artist<script>alert('xss')</script>".to_string(),
        tags: None,
        note: None,
    };
    assert!(invalid_chars.validate().is_err());

    // Too many tags
    let too_many_tags = ValidatedAddToDnpRequest {
        user_id: Uuid::new_v4().to_string(),
        artist_name: "Test Artist".to_string(),
        tags: Some((0..11).map(|i| format!("tag{}", i)).collect()),
        note: None,
    };
    assert!(too_many_tags.validate().is_err());

    // Empty tag
    let empty_tag = ValidatedAddToDnpRequest {
        user_id: Uuid::new_v4().to_string(),
        artist_name: "Test Artist".to_string(),
        tags: Some(vec!["".to_string()]),
        note: None,
    };
    assert!(empty_tag.validate().is_err());

    // Note too long
    let long_note = ValidatedAddToDnpRequest {
        user_id: Uuid::new_v4().to_string(),
        artist_name: "Test Artist".to_string(),
        tags: None,
        note: Some("a".repeat(1001)),
    };
    assert!(long_note.validate().is_err());
}

#[tokio::test]
async fn test_retry_database_operation_success() {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let config = RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(10),
        backoff_multiplier: 2.0,
    };

    let result = retry_database_operation(
        || {
            let counter = counter_clone.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(AppError::DatabaseConnectionFailed)
                } else {
                    Ok("success")
                }
            }
        },
        config,
        "test_operation",
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_retry_database_operation_failure() {
    let config = RetryConfig {
        max_attempts: 2,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(10),
        backoff_multiplier: 2.0,
    };

    let result = retry_database_operation(
        || async { Err(AppError::DatabaseConnectionFailed) },
        config,
        "test_operation",
    )
    .await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        AppError::DatabaseConnectionFailed
    ));
}

#[tokio::test]
async fn test_retry_redis_operation_success() {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let config = RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(10),
        backoff_multiplier: 2.0,
    };

    let result = retry_redis_operation(
        || {
            let counter = counter_clone.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(AppError::RedisConnectionFailed)
                } else {
                    Ok("success")
                }
            }
        },
        config,
        "test_operation",
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_liveness_check() {
    let result = liveness_check().await;
    assert!(result.is_ok());
}

#[test]
fn test_health_check_config() {
    let config = HealthCheckConfig::default();
    assert_eq!(config.timeout, Duration::from_secs(5));
    assert!(config.include_system_info);
    assert!(config.detailed_checks);

    let custom_config = HealthCheckConfig {
        timeout: Duration::from_secs(10),
        include_system_info: false,
        detailed_checks: false,
    };
    assert_eq!(custom_config.timeout, Duration::from_secs(10));
    assert!(!custom_config.include_system_info);
    assert!(!custom_config.detailed_checks);
}

#[test]
fn test_error_response_serialization() {
    let error_response = ErrorResponse {
        error: "Test error".to_string(),
        error_code: "TEST_ERROR".to_string(),
        message: "This is a test error".to_string(),
        details: Some(json!({"field": "value"})),
        correlation_id: Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let serialized = serde_json::to_string(&error_response).unwrap();
    assert!(serialized.contains("Test error"));
    assert!(serialized.contains("TEST_ERROR"));
    assert!(serialized.contains("This is a test error"));
}

#[tokio::test]
async fn test_sqlx_error_conversion() {
    // Test database error conversion
    let db_error = sqlx::Error::PoolTimedOut;
    let app_error: AppError = db_error.into();
    assert!(matches!(app_error, AppError::DatabaseConnectionFailed));
}

#[tokio::test]
async fn test_validation_errors_conversion() {
    let mut validation_errors = validator::ValidationErrors::new();
    validation_errors.add("email", validator::ValidationError::new("invalid_email"));

    let app_error: AppError = validation_errors.into();
    assert!(matches!(app_error, AppError::ValidationFailed(_)));
}

#[tokio::test]
async fn test_anyhow_error_conversion() {
    let anyhow_error = anyhow::anyhow!("Test error");
    let app_error: AppError = anyhow_error.into();
    assert!(matches!(app_error, AppError::Internal { .. }));
}

// Integration test for error response format
#[tokio::test]
async fn test_error_response_format() {
    use axum::response::IntoResponse;

    let error = AppError::InvalidCredentials;
    let response = error.into_response();

    // Check status code
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Check that response contains JSON with expected fields
    let (parts, body) = response.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Parse JSON to verify structure
    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    assert!(json.get("error").is_some());
    assert!(json.get("error_code").is_some());
    assert!(json.get("message").is_some());
    assert!(json.get("correlation_id").is_some());
    assert!(json.get("timestamp").is_some());

    assert_eq!(json["error_code"], "AUTH_INVALID_CREDENTIALS");
    assert_eq!(json["message"], "Invalid email or password");
}
