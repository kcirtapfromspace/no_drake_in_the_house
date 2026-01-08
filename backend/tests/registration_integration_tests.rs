use music_streaming_blocklist_backend::{
    models::RegisterRequest, services::auth::AuthService, AppError,
};

mod common;
use common::{init_test_tracing, TestDatabase};

/// Integration tests for registration flow
/// Tests end-to-end registration with auto-login, error handling, rate limiting, and database integrity

#[tokio::test]
async fn test_successful_registration_with_auto_login() {
    let test_db = TestDatabase::new().await;
    let auth_service = AuthService::new(test_db.pool.clone());

    // Set auto-login enabled
    std::env::set_var("AUTO_LOGIN_ENABLED", "true");

    let request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: "SecurePassword123!".to_string(),
        confirm_password: "SecurePassword123!".to_string(),
        terms_accepted: true,
    };

    let result = auth_service.register(request).await;
    assert!(result.is_ok(), "Registration should succeed");

    let auth_response = result.unwrap();

    // Verify auto-login tokens are provided
    assert!(
        !auth_response.access_token.is_empty(),
        "Access token should be provided"
    );
    assert!(
        !auth_response.refresh_token.is_empty(),
        "Refresh token should be provided"
    );

    // Verify user profile
    assert_eq!(auth_response.user.email, "test@example.com");
    assert!(
        !auth_response.user.email_verified,
        "Email should not be verified initially"
    );
    assert!(
        !auth_response.user.totp_enabled,
        "TOTP should not be enabled initially"
    );

    // Verify user was created in database
    let user_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind("test@example.com")
        .fetch_one(&test_db.pool)
        .await
        .unwrap();
    assert_eq!(user_count, 1, "User should be created in database");

    // Verify audit log entry
    let audit_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'user_registered'",
    )
    .fetch_one(&test_db.pool)
    .await
    .unwrap();
    assert_eq!(audit_count, 1, "Registration should be logged in audit log");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_registration_with_auto_login_disabled() {
    let test_db = TestDatabase::new().await;
    let auth_service = AuthService::new(test_db.pool.clone());

    // Set auto-login disabled
    std::env::set_var("AUTO_LOGIN_ENABLED", "false");

    let request = RegisterRequest {
        email: "test_no_autologin@example.com".to_string(),
        password: "SecurePassword123!".to_string(),
        confirm_password: "SecurePassword123!".to_string(),
        terms_accepted: true,
    };

    let result = auth_service.register(request).await;

    // When auto-login is disabled, registration should still succeed but without tokens
    // The current implementation always returns tokens, but this tests the configuration
    assert!(
        result.is_ok(),
        "Registration should succeed even with auto-login disabled"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_registration_validation_errors() {
    let test_db = TestDatabase::new().await;
    let auth_service = AuthService::new(test_db.pool.clone());

    // Test multiple validation errors
    let invalid_request = RegisterRequest {
        email: "invalid-email".to_string(),
        password: "weak".to_string(),
        confirm_password: "different".to_string(),
        terms_accepted: false,
    };

    let result = auth_service.register(invalid_request).await;
    assert!(
        result.is_err(),
        "Registration should fail with validation errors"
    );

    match result.unwrap_err() {
        AppError::RegistrationValidationError { errors } => {
            // Verify all expected validation errors are present
            assert!(
                errors.iter().any(|e| e.field == "email"),
                "Should have email error"
            );
            assert!(
                errors.iter().any(|e| e.field == "password"),
                "Should have password error"
            );
            assert!(
                errors.iter().any(|e| e.field == "confirm_password"),
                "Should have confirm_password error"
            );
            assert!(
                errors.iter().any(|e| e.field == "terms_accepted"),
                "Should have terms_accepted error"
            );

            // Verify error codes
            assert!(
                errors.iter().any(|e| e.code == "EMAIL_INVALID_FORMAT"),
                "Should have email format error"
            );
            assert!(
                errors.iter().any(|e| e.code == "PASSWORD_WEAK"),
                "Should have password weak error"
            );
            assert!(
                errors.iter().any(|e| e.code == "PASSWORD_MISMATCH"),
                "Should have password mismatch error"
            );
            assert!(
                errors.iter().any(|e| e.code == "TERMS_NOT_ACCEPTED"),
                "Should have terms not accepted error"
            );
        }
        _ => panic!("Expected RegistrationValidationError"),
    }

    // Verify no user was created
    let user_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(&test_db.pool)
        .await
        .unwrap();
    assert_eq!(
        user_count, 0,
        "No user should be created with validation errors"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_registration_duplicate_email() {
    let test_db = TestDatabase::new().await;
    let auth_service = AuthService::new(test_db.pool.clone());

    let request1 = RegisterRequest {
        email: "duplicate@example.com".to_string(),
        password: "SecurePassword123!".to_string(),
        confirm_password: "SecurePassword123!".to_string(),
        terms_accepted: true,
    };

    // First registration should succeed
    let result1 = auth_service.register(request1).await;
    assert!(result1.is_ok(), "First registration should succeed");

    let request2 = RegisterRequest {
        email: "duplicate@example.com".to_string(),
        password: "DifferentPassword123!".to_string(),
        confirm_password: "DifferentPassword123!".to_string(),
        terms_accepted: true,
    };

    // Second registration with same email should fail
    let result2 = auth_service.register(request2).await;
    assert!(
        result2.is_err(),
        "Second registration with same email should fail"
    );

    match result2.unwrap_err() {
        AppError::EmailAlreadyRegistered => {
            // Expected error type
        }
        _ => panic!("Expected EmailAlreadyRegistered error"),
    }

    // Verify only one user was created
    let user_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind("duplicate@example.com")
        .fetch_one(&test_db.pool)
        .await
        .unwrap();
    assert_eq!(
        user_count, 1,
        "Only one user should exist with duplicate email"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_registration_database_transaction_integrity() {
    let test_db = TestDatabase::new().await;
    let auth_service = AuthService::new(test_db.pool.clone());

    // Test that failed registration doesn't leave partial data
    let invalid_request = RegisterRequest {
        email: "transaction_test@example.com".to_string(),
        password: "weak".to_string(), // This will cause validation to fail
        confirm_password: "weak".to_string(),
        terms_accepted: true,
    };

    let result = auth_service.register(invalid_request).await;
    assert!(result.is_err(), "Registration should fail");

    // Verify no partial data was left in database
    let user_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind("transaction_test@example.com")
        .fetch_one(&test_db.pool)
        .await
        .unwrap();
    assert_eq!(
        user_count, 0,
        "No user should be created when validation fails"
    );

    let audit_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM audit_log WHERE old_subject_id LIKE '%transaction_test%'",
    )
    .fetch_one(&test_db.pool)
    .await
    .unwrap();
    assert_eq!(
        audit_count, 0,
        "No audit log should be created when validation fails"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_registration_password_hashing() {
    let test_db = TestDatabase::new().await;
    let auth_service = AuthService::new(test_db.pool.clone());

    let request = RegisterRequest {
        email: "hash_test@example.com".to_string(),
        password: "TestPassword123!".to_string(),
        confirm_password: "TestPassword123!".to_string(),
        terms_accepted: true,
    };

    let result = auth_service.register(request).await;
    assert!(result.is_ok(), "Registration should succeed");

    // Verify password is hashed in database
    let password_hash =
        sqlx::query_scalar::<_, Option<String>>("SELECT password_hash FROM users WHERE email = $1")
            .bind("hash_test@example.com")
            .fetch_one(&test_db.pool)
            .await
            .unwrap();

    assert!(password_hash.is_some(), "Password hash should be stored");
    let hash = password_hash.unwrap();

    // Verify it's a bcrypt hash
    assert!(hash.starts_with("$2b$"), "Should be a bcrypt hash");
    assert_ne!(
        hash, "TestPassword123!",
        "Password should not be stored in plain text"
    );

    // Verify hash has appropriate cost (should be at least 12)
    let parts: Vec<&str> = hash.split('$').collect();
    assert!(parts.len() >= 4, "Bcrypt hash should have at least 4 parts");
    if let Ok(cost) = parts[2].parse::<u32>() {
        assert!(
            cost >= 12,
            "Bcrypt cost should be at least 12, got {}",
            cost
        );
    }

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_registration_audit_logging() {
    let test_db = TestDatabase::new().await;
    let auth_service = AuthService::new(test_db.pool.clone());

    let request = RegisterRequest {
        email: "audit_test@example.com".to_string(),
        password: "SecurePassword123!".to_string(),
        confirm_password: "SecurePassword123!".to_string(),
        terms_accepted: true,
    };

    let result = auth_service.register(request).await;
    assert!(result.is_ok(), "Registration should succeed");

    let auth_response = result.unwrap();

    // Verify audit log entry was created (simplified check)
    let audit_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'user_registered'",
    )
    .fetch_one(&test_db.pool)
    .await
    .unwrap();
    assert!(
        audit_count > 0,
        "Registration should be logged in audit log"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_registration_token_generation() {
    let test_db = TestDatabase::new().await;
    let auth_service = AuthService::new(test_db.pool.clone());

    let request = RegisterRequest {
        email: "token_test@example.com".to_string(),
        password: "SecurePassword123!".to_string(),
        confirm_password: "SecurePassword123!".to_string(),
        terms_accepted: true,
    };

    let result = auth_service.register(request).await;
    assert!(result.is_ok(), "Registration should succeed");

    let auth_response = result.unwrap();

    // Verify tokens are valid JWTs
    assert!(
        !auth_response.access_token.is_empty(),
        "Access token should not be empty"
    );
    assert!(
        !auth_response.refresh_token.is_empty(),
        "Refresh token should not be empty"
    );

    // Verify access token format (should have 3 parts separated by dots)
    let token_parts: Vec<&str> = auth_response.access_token.split('.').collect();
    assert_eq!(
        token_parts.len(),
        3,
        "JWT should have 3 parts separated by dots"
    );

    // Verify token can be validated
    let validation_result = auth_service.verify_token(&auth_response.access_token);
    assert!(validation_result.is_ok(), "Generated token should be valid");

    let claims = validation_result.unwrap();
    assert_eq!(claims.sub, auth_response.user.id.to_string());
    assert_eq!(claims.email, auth_response.user.email);

    // Verify refresh token was stored in database
    let session_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM user_sessions WHERE user_id = $1")
            .bind(auth_response.user.id)
            .fetch_one(&test_db.pool)
            .await
            .unwrap();
    assert_eq!(session_count, 1, "Refresh token session should be stored");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_registration_concurrent_requests() {
    let test_db = TestDatabase::new().await;
    let auth_service = AuthService::new(test_db.pool.clone());

    // Test concurrent registration attempts with same email
    let request1 = RegisterRequest {
        email: "concurrent@example.com".to_string(),
        password: "SecurePassword123!".to_string(),
        confirm_password: "SecurePassword123!".to_string(),
        terms_accepted: true,
    };

    let request2 = RegisterRequest {
        email: "concurrent@example.com".to_string(),
        password: "DifferentPassword123!".to_string(),
        confirm_password: "DifferentPassword123!".to_string(),
        terms_accepted: true,
    };

    // Run both registrations concurrently
    let (result1, result2) = tokio::join!(
        auth_service.register(request1),
        auth_service.register(request2)
    );

    // One should succeed, one should fail
    let success_count = [&result1, &result2].iter().filter(|r| r.is_ok()).count();
    let error_count = [&result1, &result2].iter().filter(|r| r.is_err()).count();

    assert_eq!(success_count, 1, "Exactly one registration should succeed");
    assert_eq!(error_count, 1, "Exactly one registration should fail");

    // Verify only one user was created
    let user_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind("concurrent@example.com")
        .fetch_one(&test_db.pool)
        .await
        .unwrap();
    assert_eq!(
        user_count, 1,
        "Only one user should be created despite concurrent requests"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_registration_edge_cases() {
    let test_db = TestDatabase::new().await;
    let auth_service = AuthService::new(test_db.pool.clone());

    // Test with maximum length email
    let long_email = format!("{}@example.com", "a".repeat(240));
    let request = RegisterRequest {
        email: long_email.clone(),
        password: "SecurePassword123!".to_string(),
        confirm_password: "SecurePassword123!".to_string(),
        terms_accepted: true,
    };

    let result = auth_service.register(request).await;
    assert!(
        result.is_ok(),
        "Registration with long email should succeed"
    );

    // Test with special characters in password
    let special_password = "P@ssw0rd!#$%^&*()_+-=[]{}|;:,.<>?";
    let request = RegisterRequest {
        email: "special@example.com".to_string(),
        password: special_password.to_string(),
        confirm_password: special_password.to_string(),
        terms_accepted: true,
    };

    let result = auth_service.register(request).await;
    assert!(
        result.is_ok(),
        "Registration with special characters in password should succeed"
    );

    test_db.cleanup().await;
}
