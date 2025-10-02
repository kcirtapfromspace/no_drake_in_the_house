use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use std::sync::Arc;
use std::time::Instant;
use serde_json;
use crate::{
    AppState, AuthService, Result, AppError,
    models::{RegisterRequest, LoginRequest, AuthResponse, RefreshTokenRequest, TotpSetupRequest, TotpVerifyRequest},
    services::registration_monitoring::RegistrationMonitoringService,
};

/// Register a new user with monitoring
pub async fn register_handler(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let start_time = Instant::now();
    
    // Create monitoring service for this request
    let monitoring_service = RegistrationMonitoringService::new()
        .map_err(|e| AppError::Internal { 
            message: Some(format!("Failed to create monitoring service: {}", e)) 
        })?;
    
    // Record registration attempt
    monitoring_service.record_registration_attempt();
    
    // Log structured registration event
    monitoring_service.log_registration_event(
        "registration_attempt",
        None,
        &request.email,
        None,
        None,
        Some(serde_json::json!({
            "terms_accepted": request.terms_accepted,
            "user_agent": "unknown", // Would extract from headers in real implementation
        })),
    );
    
    tracing::info!(
        email = %request.email, 
        terms_accepted = %request.terms_accepted,
        "User registration attempt"
    );
    
    let response = state.auth_service.register(request.clone()).await
        .map_err(|e| {
            let duration = start_time.elapsed();
            
            // Record failure in monitoring
            let error_type = match &e {
                AppError::RegistrationValidationError { .. } => "validation_error",
                AppError::EmailAlreadyRegistered => "email_duplicate",
                AppError::PasswordMismatch => "password_mismatch",
                AppError::TermsNotAccepted => "terms_not_accepted",
                AppError::WeakPassword { .. } => "weak_password",
                AppError::RateLimitExceeded { .. } => "rate_limit_exceeded",
                _ => "unknown_error",
            };
            
            monitoring_service.record_registration_failure(error_type);
            
            // Log structured failure event
            monitoring_service.log_registration_event(
                "registration_failure",
                None,
                &request.email,
                Some(duration.as_millis() as f64),
                Some(&e.to_string()),
                Some(serde_json::json!({
                    "error_type": error_type,
                })),
            );
            
            // Enhanced error logging with structured information
            match &e {
                AppError::RegistrationValidationError { errors } => {
                    monitoring_service.record_validation_failure(duration);
                    tracing::warn!(
                        validation_errors = ?errors,
                        duration_ms = duration.as_millis(),
                        "Registration failed due to validation errors"
                    );
                }
                AppError::EmailAlreadyRegistered => {
                    monitoring_service.record_email_duplicate();
                    tracing::warn!(
                        duration_ms = duration.as_millis(),
                        "Registration failed: email already registered"
                    );
                }
                AppError::PasswordMismatch => {
                    tracing::warn!(
                        duration_ms = duration.as_millis(),
                        "Registration failed: password confirmation mismatch"
                    );
                }
                AppError::TermsNotAccepted => {
                    tracing::warn!(
                        duration_ms = duration.as_millis(),
                        "Registration failed: terms not accepted"
                    );
                }
                AppError::WeakPassword { requirements } => {
                    tracing::warn!(
                        password_requirements = ?requirements,
                        duration_ms = duration.as_millis(),
                        "Registration failed: weak password"
                    );
                }
                AppError::RateLimitExceeded { retry_after } => {
                    tracing::warn!(
                        retry_after = ?retry_after,
                        duration_ms = duration.as_millis(),
                        "Registration failed: rate limit exceeded"
                    );
                }
                _ => {
                    tracing::error!(
                        error = %e,
                        duration_ms = duration.as_millis(),
                        "Registration failed with unexpected error"
                    );
                }
            }
            e
        })?;
    
    let duration = start_time.elapsed();
    
    // Record successful registration
    monitoring_service.record_registration_success(duration);
    
    // Log structured success event
    monitoring_service.log_registration_event(
        "registration_success",
        Some(response.user.id),
        &response.user.email,
        Some(duration.as_millis() as f64),
        None,
        Some(serde_json::json!({
            "auto_login": !response.access_token.is_empty(),
            "email_verified": response.user.email_verified,
        })),
    );
    
    // Check if auto-login was successful (tokens are not empty)
    if response.access_token.is_empty() || response.refresh_token.is_empty() {
        tracing::info!(
            user_id = %response.user.id, 
            email = %response.user.email,
            duration_ms = duration.as_millis(),
            "User registered successfully, auto-login disabled"
        );
    } else {
        tracing::info!(
            user_id = %response.user.id, 
            email = %response.user.email,
            duration_ms = duration.as_millis(),
            "User registered successfully with auto-login"
        );
    }
    
    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "success": true,
        "data": response,
        "message": "Registration successful"
    }))))
}

/// Login user
pub async fn login_handler(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(email = %request.email, "User login attempt");
    
    let response = state.auth_service.login(request).await
        .map_err(|e| {
            tracing::warn!(error = %e, "Login failed");
            e
        })?;
    
    tracing::info!(user_id = %response.user.id, "User logged in successfully");
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": response,
        "message": "Login successful"
    })))
}

/// Refresh access token
pub async fn refresh_token_handler(
    State(state): State<AppState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Token refresh attempt");
    
    // Get token pair from service
    let token_pair = state.auth_service.refresh_token(&request.refresh_token).await
        .map_err(|e| {
            tracing::warn!(error = %e, "Token refresh failed");
            AppError::InvalidCredentials
        })?;
    
    // Get user info from the new access token
    let claims = state.auth_service.verify_token(&token_pair.access_token)
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to verify newly generated token");
            AppError::Internal { message: Some("Token generation error".to_string()) }
        })?;
    
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|e| {
            tracing::error!(error = %e, "Invalid user ID in token claims");
            AppError::Internal { message: Some("Invalid token claims".to_string()) }
        })?;
    
    let user = state.auth_service.get_user(user_id).await
        .map_err(|e| {
            tracing::warn!(error = %e, "User not found during token refresh");
            AppError::NotFound { resource: "User".to_string() }
        })?;
    
    let response = AuthResponse {
        user: user.to_profile(),
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
    };
    
    tracing::info!(user_id = %user_id, "Token refreshed successfully");
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": response,
        "message": "Token refreshed successfully"
    })))
}

/// Setup 2FA for user
pub async fn setup_2fa_handler(
    State(state): State<AppState>,
    user: crate::models::AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(user_id = %user.id, "2FA setup attempt");
    
    let response = state.auth_service.setup_2fa(user.id).await
        .map_err(|e| {
            tracing::warn!(error = %e, user_id = %user.id, "2FA setup failed");
            match e.to_string().as_str() {
                s if s.contains("already enabled") => AppError::Conflict { 
                    message: "2FA is already enabled for this user".to_string() 
                },
                _ => AppError::Internal { message: Some(e.to_string()) }
            }
        })?;
    
    tracing::info!(user_id = %user.id, "2FA setup completed");
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "secret": response.secret,
            "qr_code_url": response.qr_code_url,
            "backup_codes": response.backup_codes
        }
    })))
}

/// Verify and enable 2FA
pub async fn verify_2fa_handler(
    State(state): State<AppState>,
    user: crate::models::AuthenticatedUser,
    Json(request): Json<TotpVerifyRequest>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(user_id = %user.id, "2FA verification attempt");
    
    // Validate TOTP code format
    if request.totp_code.len() != 6 || !request.totp_code.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::InvalidFieldValue { 
            field: "totp_code".to_string(), 
            message: "TOTP code must be 6 digits".to_string() 
        });
    }
    
    let success = state.auth_service.verify_and_enable_2fa(user.id, &request.totp_code).await
        .map_err(|e| {
            tracing::warn!(error = %e, user_id = %user.id, "2FA verification failed");
            AppError::InvalidFieldValue { 
                field: "totp_code".to_string(), 
                message: "Invalid TOTP code".to_string() 
            }
        })?;
    
    if success {
        tracing::info!(user_id = %user.id, "2FA enabled successfully");
        Ok(Json(serde_json::json!({
            "success": true,
            "message": "2FA enabled successfully"
        })))
    } else {
        Err(AppError::InvalidFieldValue { 
            field: "totp_code".to_string(), 
            message: "Invalid TOTP code".to_string() 
        })
    }
}

/// Disable 2FA for user
pub async fn disable_2fa_handler(
    State(state): State<AppState>,
    user: crate::models::AuthenticatedUser,
    Json(request): Json<TotpVerifyRequest>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(user_id = %user.id, "2FA disable attempt");
    
    // Validate TOTP code format
    if request.totp_code.len() != 6 || !request.totp_code.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::InvalidFieldValue { 
            field: "totp_code".to_string(), 
            message: "TOTP code must be 6 digits".to_string() 
        });
    }
    
    let success = state.auth_service.disable_2fa(user.id, &request.totp_code).await
        .map_err(|e| {
            tracing::warn!(error = %e, user_id = %user.id, "2FA disable failed");
            AppError::InvalidFieldValue { 
                field: "totp_code".to_string(), 
                message: "Invalid TOTP code".to_string() 
            }
        })?;
    
    if success {
        tracing::info!(user_id = %user.id, "2FA disabled successfully");
        Ok(Json(serde_json::json!({
            "success": true,
            "message": "2FA disabled successfully"
        })))
    } else {
        Err(AppError::InvalidFieldValue { 
            field: "totp_code".to_string(), 
            message: "Invalid TOTP code".to_string() 
        })
    }
}

/// Logout user
pub async fn logout_handler(
    State(state): State<AppState>,
    user: crate::models::AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(user_id = %user.id, "User logout attempt");
    
    // For now, we'll just log the logout since we don't have refresh token in the request
    // In a full implementation, we would:
    // 1. Get the refresh token from the request
    // 2. Invalidate the refresh token in the database
    // 3. Optionally revoke all sessions for the user
    
    // Log the logout event
    tracing::info!(user_id = %user.id, "User logged out successfully");
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Logged out successfully"
    })))
}