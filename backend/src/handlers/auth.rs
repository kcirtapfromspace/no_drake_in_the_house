use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use std::sync::Arc;
use crate::{
    AppState, AuthService, Result, AppError,
    models::{RegisterRequest, LoginRequest, AuthResponse, RefreshTokenRequest, TotpSetupRequest, TotpVerifyRequest},
};

/// Register a new user
pub async fn register_handler(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>)> {
    tracing::info!(
        email = %request.email, 
        terms_accepted = %request.terms_accepted,
        "User registration attempt"
    );
    
    let response = state.auth_service.register(request).await
        .map_err(|e| {
            // Enhanced error logging with structured information
            match &e {
                AppError::RegistrationValidationError { errors } => {
                    tracing::warn!(
                        validation_errors = ?errors,
                        "Registration failed due to validation errors"
                    );
                }
                AppError::EmailAlreadyRegistered => {
                    tracing::warn!("Registration failed: email already registered");
                }
                AppError::PasswordMismatch => {
                    tracing::warn!("Registration failed: password confirmation mismatch");
                }
                AppError::TermsNotAccepted => {
                    tracing::warn!("Registration failed: terms not accepted");
                }
                AppError::WeakPassword { requirements } => {
                    tracing::warn!(
                        password_requirements = ?requirements,
                        "Registration failed: weak password"
                    );
                }
                AppError::RateLimitExceeded { retry_after } => {
                    tracing::warn!(
                        retry_after = ?retry_after,
                        "Registration failed: rate limit exceeded"
                    );
                }
                _ => {
                    tracing::error!(error = %e, "Registration failed with unexpected error");
                }
            }
            e
        })?;
    
    // Check if auto-login was successful (tokens are not empty)
    if response.access_token.is_empty() || response.refresh_token.is_empty() {
        tracing::info!(
            user_id = %response.user.id, 
            email = %response.user.email,
            "User registered successfully, auto-login disabled"
        );
    } else {
        tracing::info!(
            user_id = %response.user.id, 
            email = %response.user.email,
            "User registered successfully with auto-login"
        );
    }
    
    Ok((StatusCode::CREATED, Json(response)))
}

/// Login user
pub async fn login_handler(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    tracing::info!(email = %request.email, "User login attempt");
    
    let response = state.auth_service.login(request).await
        .map_err(|e| {
            tracing::warn!(error = %e, "Login failed");
            e
        })?;
    
    tracing::info!(user_id = %response.user.id, "User logged in successfully");
    
    Ok(Json(response))
}

/// Refresh access token
pub async fn refresh_token_handler(
    State(state): State<AppState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<AuthResponse>> {
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
        user: crate::models::UserProfile {
            id: user.id,
            email: user.email,
            email_verified: user.email_verified,
            totp_enabled: user.totp_enabled,
            created_at: user.created_at,
            updated_at: user.updated_at,
            last_login: user.last_login,
            settings: user.settings,
        },
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
    };
    
    tracing::info!(user_id = %user_id, "Token refreshed successfully");
    
    Ok(Json(response))
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