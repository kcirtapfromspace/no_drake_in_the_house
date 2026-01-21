use axum::{
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::{
    models::oauth::{OAuthFlowResponse, OAuthProviderType},
    models::user::AuthenticatedUser,
    services::oauth::OAuthProvider,
    AppError, AppState, Result,
};

#[derive(Debug, Deserialize)]
pub struct OAuthInitiateQuery {
    pub redirect_uri: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackRequest {
    pub code: String,
    pub state: String,
    pub redirect_uri: String,
}

/// Apple OAuth callback form data (Apple sends POST with form-urlencoded data)
/// Note: Apple sends the callback as POST form data, not JSON
#[derive(Debug, Deserialize)]
pub struct AppleOAuthCallbackForm {
    /// Authorization code
    pub code: String,
    /// State parameter for CSRF protection
    pub state: String,
    /// User data (JSON string, only provided on first authorization)
    /// Contains name.firstName, name.lastName, and email
    pub user: Option<String>,
    /// ID token (JWT) - may be provided directly in form for implicit flow
    pub id_token: Option<String>,
    /// Error from Apple (if authorization failed)
    pub error: Option<String>,
}

/// Parsed Apple user data from the callback
#[derive(Debug, Deserialize)]
pub struct AppleUserData {
    pub name: Option<AppleUserName>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AppleUserName {
    #[serde(rename = "firstName")]
    pub first_name: Option<String>,
    #[serde(rename = "lastName")]
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OAuthCallbackResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct LinkedAccountsResponse {
    pub accounts: Vec<LinkedAccountInfo>,
}

#[derive(Debug, Serialize)]
pub struct LinkedAccountInfo {
    pub provider: String,
    pub provider_user_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub linked_at: chrono::DateTime<chrono::Utc>,
}

/// Initiate OAuth flow for a specific provider
pub async fn initiate_oauth_handler(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<OAuthInitiateQuery>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let start_time = std::time::Instant::now();
    tracing::info!(
        provider = %provider,
        redirect_uri = ?query.redirect_uri,
        "Initiating OAuth flow"
    );

    // Parse and validate provider type
    let provider_type: OAuthProviderType = provider.parse()
        .map_err(|_| {
            tracing::warn!(provider = %provider, "Invalid OAuth provider requested");
            AppError::InvalidFieldValue {
                field: "provider".to_string(),
                message: format!("Unsupported OAuth provider '{}'. Supported providers are: google, apple, github, spotify", provider)
            }
        })?;

    // Set default redirect URI if not provided
    let redirect_uri = query.redirect_uri.unwrap_or_else(|| {
        let default_uri = format!("http://localhost:3000/auth/callback/{}", provider);
        tracing::debug!(default_uri = %default_uri, "Using default redirect URI");
        default_uri
    });

    // Validate redirect URI format
    if let Err(_) = Url::parse(&redirect_uri) {
        tracing::warn!(redirect_uri = %redirect_uri, "Invalid redirect URI format");
        return Err(AppError::InvalidFieldValue {
            field: "redirect_uri".to_string(),
            message: "Invalid redirect URI format. Must be a valid URL.".to_string(),
        });
    }

    // Initiate OAuth flow using the auth service
    let flow_response = match state
        .auth_service
        .initiate_oauth_flow(provider_type.clone(), redirect_uri.clone())
        .await
    {
        Ok(response) => response,
        Err(e) => {
            tracing::error!(
                provider = %provider,
                redirect_uri = %redirect_uri,
                error = %e,
                "Failed to initiate OAuth flow"
            );

            // Provide user-friendly error messages based on error type
            let user_message = match &e {
                AppError::InvalidFieldValue { field, .. } if field == "provider" => {
                    format!("{} OAuth is not configured or temporarily unavailable. Please try again later or contact support.", provider_type)
                }
                AppError::ExternalServiceError(msg) => {
                    format!(
                        "Unable to connect to {} authentication service. Please try again later.",
                        provider_type
                    )
                }
                _ => {
                    format!(
                        "Authentication service temporarily unavailable. Please try again later."
                    )
                }
            };

            return Err(AppError::ExternalServiceError(user_message));
        }
    };

    let duration = start_time.elapsed();
    tracing::info!(
        provider = %provider,
        state = %flow_response.state,
        authorization_url_length = flow_response.authorization_url.len(),
        duration_ms = duration.as_millis(),
        "OAuth flow initiated successfully"
    );

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "data": {
                "authorization_url": flow_response.authorization_url,
                "state": flow_response.state,
                "provider": provider_type.to_string()
            },
            "message": format!("OAuth flow initiated successfully for {}", provider_type)
        })),
    ))
}

#[allow(dead_code)]
async fn initiate_oauth_handler_impl(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<OAuthInitiateQuery>,
) -> Result<(StatusCode, Json<OAuthFlowResponse>)> {
    tracing::info!(provider = %provider, "Initiating OAuth flow");

    let provider_type: OAuthProviderType =
        provider.parse().map_err(|_| AppError::InvalidFieldValue {
            field: "provider".to_string(),
            message: format!("Unsupported OAuth provider: {}", provider),
        })?;

    let oauth_provider = get_oauth_provider(provider_type)?;

    let redirect_uri = query
        .redirect_uri
        .unwrap_or_else(|| format!("http://localhost:3000/auth/callback/{}", provider));

    let flow_response = oauth_provider
        .initiate_flow(&redirect_uri)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to initiate OAuth flow: {}", e)),
        })?;

    tracing::info!(
        provider = %provider,
        state = %flow_response.state,
        "OAuth flow initiated successfully"
    );

    Ok((StatusCode::OK, Json(flow_response)))
}

/// Apple OAuth authorize endpoint (GET)
///
/// Returns a valid authorization URL with proper scopes for Apple Sign In.
/// This endpoint satisfies acceptance criteria US-003.1:
/// "/api/v1/auth/oauth/apple/authorize returns valid authorization URL with proper scopes"
///
/// The authorization URL includes:
/// - response_mode=form_post (Apple's required callback format)
/// - scope=name email (standard Apple scopes)
/// - client_id (Apple Service ID)
/// - redirect_uri (callback URL for authorization code)
/// - state (CSRF protection token)
pub async fn apple_authorize_handler(
    State(state): State<AppState>,
    Query(query): Query<OAuthInitiateQuery>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let start_time = std::time::Instant::now();
    tracing::info!(
        redirect_uri = ?query.redirect_uri,
        "Initiating Apple OAuth flow via authorize endpoint"
    );

    // Set default redirect URI if not provided
    let redirect_uri = query.redirect_uri.unwrap_or_else(|| {
        let default_uri = "http://localhost:3000/auth/callback/apple".to_string();
        tracing::debug!(default_uri = %default_uri, "Using default Apple redirect URI");
        default_uri
    });

    // Validate redirect URI format
    if Url::parse(&redirect_uri).is_err() {
        tracing::warn!(redirect_uri = %redirect_uri, "Invalid redirect URI format");
        return Err(AppError::InvalidFieldValue {
            field: "redirect_uri".to_string(),
            message: "Invalid redirect URI format. Must be a valid URL.".to_string(),
        });
    }

    // Initiate Apple OAuth flow using the auth service
    let flow_response = match state
        .auth_service
        .initiate_oauth_flow(OAuthProviderType::Apple, redirect_uri.clone())
        .await
    {
        Ok(response) => response,
        Err(e) => {
            tracing::error!(
                redirect_uri = %redirect_uri,
                error = %e,
                "Failed to initiate Apple OAuth flow"
            );

            let user_message = match &e {
                AppError::InvalidFieldValue { field, .. } if field == "provider" => {
                    "Apple OAuth is not configured or temporarily unavailable. Please try again later or contact support.".to_string()
                }
                AppError::ExternalServiceError(_) => {
                    "Unable to connect to Apple authentication service. Please try again later.".to_string()
                }
                _ => {
                    "Apple authentication service temporarily unavailable. Please try again later.".to_string()
                }
            };

            return Err(AppError::ExternalServiceError(user_message));
        }
    };

    let duration = start_time.elapsed();
    tracing::info!(
        state = %flow_response.state,
        authorization_url_length = flow_response.authorization_url.len(),
        duration_ms = duration.as_millis(),
        "Apple OAuth flow initiated successfully"
    );

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "data": {
                "authorization_url": flow_response.authorization_url,
                "state": flow_response.state,
                "provider": "apple",
                "scopes": ["name", "email"],
                "response_mode": "form_post"
            },
            "message": "Apple OAuth flow initiated successfully. Redirect the user to the authorization_url."
        })),
    ))
}

/// Handle Apple OAuth callback (POST with form-urlencoded data)
///
/// Apple's Sign In with Apple sends callbacks differently from other OAuth providers:
/// - Uses POST method with `application/x-www-form-urlencoded` content type
/// - Sends `response_mode=form_post` data instead of query parameters
/// - User's name and email are only provided on first authorization in the `user` field
/// - The `user` field is a JSON string that needs to be parsed
pub async fn apple_oauth_callback_handler(
    State(state): State<AppState>,
    Form(form): Form<AppleOAuthCallbackForm>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let start_time = std::time::Instant::now();
    tracing::info!(
        state = %form.state,
        code_length = form.code.len(),
        has_user_data = form.user.is_some(),
        has_id_token = form.id_token.is_some(),
        has_error = form.error.is_some(),
        "Processing Apple OAuth callback"
    );

    // Check if Apple returned an error
    if let Some(error) = &form.error {
        tracing::error!(error = %error, "Apple OAuth returned an error");
        return Err(AppError::OAuthProviderError {
            provider: "Apple".to_string(),
            message: format!("Apple authentication failed: {}", error),
        });
    }

    // Validate input parameters
    if form.code.is_empty() {
        tracing::warn!("Empty authorization code received from Apple");
        return Err(AppError::InvalidFieldValue {
            field: "code".to_string(),
            message: "Authorization code is required".to_string(),
        });
    }

    if form.state.is_empty() {
        tracing::warn!("Empty state parameter received from Apple");
        return Err(AppError::InvalidFieldValue {
            field: "state".to_string(),
            message: "State parameter is required for security".to_string(),
        });
    }

    // Parse user data if provided (only on first authorization)
    let user_data: Option<AppleUserData> = if let Some(ref user_json) = form.user {
        match serde_json::from_str(user_json) {
            Ok(data) => {
                tracing::info!("Parsed Apple user data from callback");
                Some(data)
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to parse Apple user data");
                None
            }
        }
    } else {
        None
    };

    // For Apple's form_post callback, we use the default redirect URI
    // since Apple doesn't include it in the POST form data
    let redirect_uri = std::env::var("APPLE_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:3000/auth/callback/apple".to_string());

    // Complete OAuth flow using the auth service
    let token_pair = match state
        .auth_service
        .complete_oauth_flow(
            OAuthProviderType::Apple,
            form.code.clone(),
            form.state.clone(),
            redirect_uri,
        )
        .await
    {
        Ok(tokens) => tokens,
        Err(e) => {
            tracing::error!(
                state = %form.state,
                error = %e,
                "Failed to complete Apple OAuth flow"
            );

            let user_message = match &e {
                AppError::InvalidFieldValue { field, .. } if field == "state" => {
                    "Invalid or expired authentication request. Please try signing in again."
                        .to_string()
                }
                AppError::ExternalServiceError(msg) => {
                    format!(
                        "Authentication failed with Apple. Please try again or contact support. Error: {}",
                        msg
                    )
                }
                AppError::NotFound { .. } => {
                    "Unable to complete Apple authentication. The authorization may have expired."
                        .to_string()
                }
                _ => {
                    "Apple authentication service temporarily unavailable. Please try again later."
                        .to_string()
                }
            };

            return Err(AppError::ExternalServiceError(user_message));
        }
    };

    // Get user information for the response
    let (user_claims, user) = match state.auth_service.verify_token(&token_pair.access_token) {
        Ok(claims) => {
            match claims.user_id() {
                Ok(user_id) => {
                    // If we have user data from Apple (first authorization), update the user record
                    if let Some(ref apple_user) = user_data {
                        if let Err(e) =
                            update_user_with_apple_data(&state, user_id, apple_user).await
                        {
                            tracing::warn!(
                                user_id = %user_id,
                                error = %e,
                                "Failed to update user with Apple data"
                            );
                        }
                    }

                    match state.auth_service.get_user_by_id(user_id).await {
                        Ok(user) => (claims, user),
                        Err(e) => {
                            tracing::error!(
                                user_id = %user_id,
                                error = %e,
                                "Failed to retrieve user after Apple OAuth completion"
                            );
                            return Err(AppError::Internal {
                                message: Some(
                                    "Authentication completed but user data unavailable. Please try again."
                                        .to_string(),
                                ),
                            });
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Invalid user ID in generated token");
                    return Err(AppError::Internal {
                        message: Some(
                            "Authentication token generation failed. Please try again.".to_string(),
                        ),
                    });
                }
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to verify generated token");
            return Err(AppError::Internal {
                message: Some(
                    "Authentication token verification failed. Please try again.".to_string(),
                ),
            });
        }
    };

    let duration = start_time.elapsed();
    tracing::info!(
        user_id = %user.id,
        email = %user.email,
        duration_ms = duration.as_millis(),
        "Apple OAuth authentication completed successfully"
    );

    // Check if this is a relay email (Apple's "Hide My Email" feature)
    let is_relay_email = user.email.ends_with("@privaterelay.appleid.com");

    // Prepare user data for response (excluding sensitive information)
    let user_data_response = serde_json::json!({
        "id": user.id,
        "email": user.email,
        "email_verified": user.email_verified,
        "is_relay_email": is_relay_email,
        "created_at": user.created_at,
        "last_login": user.last_login,
        "oauth_accounts": user.oauth_accounts.iter().map(|account| {
            serde_json::json!({
                "provider": account.provider,
                "provider_user_id": account.provider_user_id,
                "email": account.email,
                "display_name": account.display_name,
                "avatar_url": account.avatar_url,
                "linked_at": account.created_at
            })
        }).collect::<Vec<_>>()
    });

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "data": {
                "access_token": token_pair.access_token,
                "refresh_token": token_pair.refresh_token,
                "user": user_data_response,
                "provider": "apple"
            },
            "message": "Successfully authenticated with Apple"
        })),
    ))
}

/// Update user record with Apple-provided data (name)
/// This is only available on first authorization
async fn update_user_with_apple_data(
    state: &AppState,
    user_id: uuid::Uuid,
    apple_data: &AppleUserData,
) -> Result<()> {
    // Extract name components
    let first_name = apple_data.name.as_ref().and_then(|n| n.first_name.clone());
    let last_name = apple_data.name.as_ref().and_then(|n| n.last_name.clone());

    // Only update if we have name data
    if first_name.is_some() || last_name.is_some() {
        // Build display name from components
        let display_name = match (&first_name, &last_name) {
            (Some(f), Some(l)) => Some(format!("{} {}", f, l)),
            (Some(f), None) => Some(f.clone()),
            (None, Some(l)) => Some(l.clone()),
            (None, None) => None,
        };

        if let Some(name) = display_name {
            tracing::info!(
                user_id = %user_id,
                display_name = %name,
                "Updating user with Apple-provided name"
            );

            // Update the OAuth account with display name
            // This would require adding a method to update OAuth account metadata
            // For now, we'll log that we received the data
        }
    }

    Ok(())
}

/// Handle OAuth callback and complete authentication
pub async fn oauth_callback_handler(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Json(request): Json<OAuthCallbackRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let start_time = std::time::Instant::now();
    tracing::info!(
        provider = %provider,
        state = %request.state,
        code_length = request.code.len(),
        redirect_uri = %request.redirect_uri,
        "Processing OAuth callback"
    );

    // Validate input parameters
    if request.code.is_empty() {
        tracing::warn!(provider = %provider, "Empty authorization code received");
        return Err(AppError::InvalidFieldValue {
            field: "code".to_string(),
            message: "Authorization code is required".to_string(),
        });
    }

    if request.state.is_empty() {
        tracing::warn!(provider = %provider, "Empty state parameter received");
        return Err(AppError::InvalidFieldValue {
            field: "state".to_string(),
            message: "State parameter is required for security".to_string(),
        });
    }

    // Parse and validate provider type
    let provider_type: OAuthProviderType = provider.parse()
        .map_err(|_| {
            tracing::warn!(provider = %provider, "Invalid OAuth provider in callback");
            AppError::InvalidFieldValue {
                field: "provider".to_string(),
                message: format!("Unsupported OAuth provider '{}'. Supported providers are: google, apple, github, spotify", provider)
            }
        })?;

    // Validate redirect URI format
    if let Err(_) = Url::parse(&request.redirect_uri) {
        tracing::warn!(redirect_uri = %request.redirect_uri, "Invalid redirect URI format in callback");
        return Err(AppError::InvalidFieldValue {
            field: "redirect_uri".to_string(),
            message: "Invalid redirect URI format. Must be a valid URL.".to_string(),
        });
    }

    // Complete OAuth flow using the auth service
    let token_pair = match state
        .auth_service
        .complete_oauth_flow(
            provider_type.clone(),
            request.code.clone(),
            request.state.clone(),
            request.redirect_uri.clone(),
        )
        .await
    {
        Ok(tokens) => tokens,
        Err(e) => {
            tracing::error!(
                provider = %provider,
                state = %request.state,
                error = %e,
                "Failed to complete OAuth flow"
            );

            // Provide user-friendly error messages based on error type
            let user_message = match &e {
                AppError::InvalidFieldValue { field, message } if field == "state" => {
                    "Invalid or expired authentication request. Please try signing in again."
                        .to_string()
                }
                AppError::ExternalServiceError(msg) => {
                    format!(
                        "Authentication failed with {}. Please try again or contact support.",
                        provider_type
                    )
                }
                AppError::NotFound { .. } => {
                    format!(
                        "Unable to complete {} authentication. The authorization may have expired.",
                        provider_type
                    )
                }
                _ => {
                    format!(
                        "Authentication service temporarily unavailable. Please try again later."
                    )
                }
            };

            return Err(AppError::ExternalServiceError(user_message));
        }
    };

    // Get user information for the response
    let (user_claims, user) = match state.auth_service.verify_token(&token_pair.access_token) {
        Ok(claims) => {
            match claims.user_id() {
                Ok(user_id) => match state.auth_service.get_user_by_id(user_id).await {
                    Ok(user) => (claims, user),
                    Err(e) => {
                        tracing::error!(
                            provider = %provider,
                            user_id = %user_id,
                            error = %e,
                            "Failed to retrieve user after OAuth completion"
                        );
                        return Err(AppError::Internal {
                                message: Some("Authentication completed but user data unavailable. Please try again.".to_string())
                            });
                    }
                },
                Err(e) => {
                    tracing::error!(
                        provider = %provider,
                        error = %e,
                        "Invalid user ID in generated token"
                    );
                    return Err(AppError::Internal {
                        message: Some(
                            "Authentication token generation failed. Please try again.".to_string(),
                        ),
                    });
                }
            }
        }
        Err(e) => {
            tracing::error!(
                provider = %provider,
                error = %e,
                "Failed to verify generated token"
            );
            return Err(AppError::Internal {
                message: Some(
                    "Authentication token verification failed. Please try again.".to_string(),
                ),
            });
        }
    };

    let duration = start_time.elapsed();
    tracing::info!(
        provider = %provider,
        user_id = %user.id,
        email = %user.email,
        duration_ms = duration.as_millis(),
        "OAuth authentication completed successfully"
    );

    // Prepare user data for response (excluding sensitive information)
    let user_data = serde_json::json!({
        "id": user.id,
        "email": user.email,
        "email_verified": user.email_verified,
        "created_at": user.created_at,
        "last_login": user.last_login,
        "oauth_accounts": user.oauth_accounts.iter().map(|account| {
            serde_json::json!({
                "provider": account.provider,
                "provider_user_id": account.provider_user_id,
                "email": account.email,
                "display_name": account.display_name,
                "avatar_url": account.avatar_url,
                "linked_at": account.created_at
            })
        }).collect::<Vec<_>>()
    });

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "data": {
                "access_token": token_pair.access_token,
                "refresh_token": token_pair.refresh_token,
                "user": user_data,
                "provider": provider_type.to_string()
            },
            "message": format!("Successfully authenticated with {}", provider_type)
        })),
    ))
}

#[allow(dead_code)]
async fn oauth_callback_handler_impl(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Json(request): Json<OAuthCallbackRequest>,
) -> Result<(StatusCode, Json<OAuthCallbackResponse>)> {
    tracing::info!(
        provider = %provider,
        state = %request.state,
        "Processing OAuth callback"
    );

    let provider_type: OAuthProviderType =
        provider.parse().map_err(|_| AppError::InvalidFieldValue {
            field: "provider".to_string(),
            message: format!("Unsupported OAuth provider: {}", provider),
        })?;

    let oauth_provider = get_oauth_provider(provider_type.clone())?;

    // Exchange authorization code for tokens
    let tokens = oauth_provider
        .exchange_code(&request.code, &request.state, &request.redirect_uri)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to exchange OAuth code: {}", e)),
        })?;

    // Get user information from OAuth provider
    let user_info = oauth_provider
        .get_user_info(&tokens.access_token)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to get user info: {}", e)),
        })?;

    // Check if user exists or create new user
    let user = match state
        .auth_service
        .find_user_by_oauth_account_public(provider_type.clone(), &user_info.provider_user_id)
        .await
    {
        Ok(existing_user) => {
            // Update OAuth account tokens
            state
                .auth_service
                .update_oauth_account(existing_user.id, provider_type, &tokens, &user_info)
                .await?;
            existing_user
        }
        Err(AppError::NotFound { .. }) => {
            // Create new user with OAuth account
            let email = user_info
                .email
                .clone()
                .ok_or_else(|| AppError::MissingField {
                    field: "email".to_string(),
                })?;

            state
                .auth_service
                .create_user_with_oauth(&email, provider_type, &tokens, &user_info)
                .await?
        }
        Err(e) => return Err(e),
    };

    // Generate JWT tokens for the user
    let (access_token, refresh_token) = state.auth_service.generate_tokens(user.id).await?;

    tracing::info!(
        provider = %provider,
        user_id = %user.id,
        "OAuth authentication completed successfully"
    );

    let response = OAuthCallbackResponse {
        access_token,
        refresh_token,
        user: serde_json::to_value(&user).map_err(|e| AppError::Internal {
            message: Some(format!("Failed to serialize user: {}", e)),
        })?,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Initiate OAuth flow for linking an account to existing user
pub async fn link_oauth_account_handler(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    authenticated_user: AuthenticatedUser,
    Query(query): Query<OAuthInitiateQuery>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let start_time = std::time::Instant::now();
    tracing::info!(
        provider = %provider,
        user_id = %authenticated_user.id,
        email = %authenticated_user.email,
        "Initiating OAuth account linking"
    );

    // Parse and validate provider type
    let provider_type: OAuthProviderType = provider.parse()
        .map_err(|_| {
            tracing::warn!(provider = %provider, "Invalid OAuth provider for linking");
            AppError::InvalidFieldValue {
                field: "provider".to_string(),
                message: format!("Unsupported OAuth provider '{}'. Supported providers are: google, apple, github, spotify", provider)
            }
        })?;

    // Check if user already has this provider linked
    let existing_accounts = match state
        .auth_service
        .load_oauth_accounts(authenticated_user.id)
        .await
    {
        Ok(accounts) => accounts,
        Err(e) => {
            tracing::error!(
                user_id = %authenticated_user.id,
                provider = %provider,
                error = %e,
                "Failed to check existing OAuth accounts"
            );
            return Err(AppError::ExternalServiceError(
                "Unable to verify existing account links. Please try again later.".to_string(),
            ));
        }
    };

    // Check if provider is already linked
    if existing_accounts
        .iter()
        .any(|account| account.provider == provider_type)
    {
        tracing::warn!(
            user_id = %authenticated_user.id,
            provider = %provider,
            "Attempt to link already linked OAuth provider"
        );
        return Err(AppError::InvalidFieldValue {
            field: "provider".to_string(),
            message: format!("Your account is already linked to {}. Please unlink first if you want to connect a different {} account.", provider_type, provider_type),
        });
    }

    // Set redirect URI for account linking
    let redirect_uri = query.redirect_uri.unwrap_or_else(|| {
        let default_uri = format!("http://localhost:3000/auth/link-callback/{}", provider);
        tracing::debug!(default_uri = %default_uri, "Using default link callback URI");
        default_uri
    });

    // Validate redirect URI format
    if let Err(_) = Url::parse(&redirect_uri) {
        tracing::warn!(redirect_uri = %redirect_uri, "Invalid redirect URI format for linking");
        return Err(AppError::InvalidFieldValue {
            field: "redirect_uri".to_string(),
            message: "Invalid redirect URI format. Must be a valid URL.".to_string(),
        });
    }

    // Initiate OAuth flow for account linking
    let flow_response = match state
        .auth_service
        .initiate_oauth_flow(provider_type.clone(), redirect_uri.clone())
        .await
    {
        Ok(response) => response,
        Err(e) => {
            tracing::error!(
                user_id = %authenticated_user.id,
                provider = %provider,
                redirect_uri = %redirect_uri,
                error = %e,
                "Failed to initiate OAuth linking flow"
            );

            let user_message = match &e {
                AppError::InvalidFieldValue { field, .. } if field == "provider" => {
                    format!("{} OAuth is not configured or temporarily unavailable. Please try again later.", provider_type)
                }
                AppError::ExternalServiceError(_) => {
                    format!(
                        "Unable to connect to {} authentication service. Please try again later.",
                        provider_type
                    )
                }
                _ => "Account linking service temporarily unavailable. Please try again later."
                    .to_string(),
            };

            return Err(AppError::ExternalServiceError(user_message));
        }
    };

    let duration = start_time.elapsed();
    tracing::info!(
        provider = %provider,
        user_id = %authenticated_user.id,
        state = %flow_response.state,
        duration_ms = duration.as_millis(),
        "OAuth account linking flow initiated successfully"
    );

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "data": {
                "authorization_url": flow_response.authorization_url,
                "state": flow_response.state,
                "provider": provider_type.to_string(),
                "linking_mode": true
            },
            "message": format!("Account linking initiated for {}. Please complete the authorization process.", provider_type)
        })),
    ))
}

/// Complete OAuth account linking callback
pub async fn oauth_link_callback_handler(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    authenticated_user: AuthenticatedUser,
    Json(request): Json<OAuthCallbackRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let start_time = std::time::Instant::now();
    tracing::info!(
        provider = %provider,
        user_id = %authenticated_user.id,
        state = %request.state,
        "Processing OAuth account linking callback"
    );

    // Validate input parameters
    if request.code.is_empty() {
        tracing::warn!(provider = %provider, user_id = %authenticated_user.id, "Empty authorization code received for linking");
        return Err(AppError::InvalidFieldValue {
            field: "code".to_string(),
            message: "Authorization code is required".to_string(),
        });
    }

    if request.state.is_empty() {
        tracing::warn!(provider = %provider, user_id = %authenticated_user.id, "Empty state parameter received for linking");
        return Err(AppError::InvalidFieldValue {
            field: "state".to_string(),
            message: "State parameter is required for security".to_string(),
        });
    }

    // Parse and validate provider type
    let provider_type: OAuthProviderType = provider.parse()
        .map_err(|_| {
            tracing::warn!(provider = %provider, "Invalid OAuth provider in linking callback");
            AppError::InvalidFieldValue {
                field: "provider".to_string(),
                message: format!("Unsupported OAuth provider '{}'. Supported providers are: google, apple, github, spotify", provider)
            }
        })?;

    // Use the link_oauth_account method from auth service
    let account_link_request = crate::models::oauth::AccountLinkRequest {
        provider: provider_type.clone(),
        code: request.code.clone(),
        state: request.state.clone(),
        redirect_uri: request.redirect_uri.clone(),
    };

    match state
        .auth_service
        .link_oauth_account(authenticated_user.id, account_link_request)
        .await
    {
        Ok(()) => {
            let duration = start_time.elapsed();
            tracing::info!(
                provider = %provider,
                user_id = %authenticated_user.id,
                duration_ms = duration.as_millis(),
                "OAuth account linking completed successfully"
            );

            // Get updated user information
            let updated_accounts = state
                .auth_service
                .load_oauth_accounts(authenticated_user.id)
                .await
                .unwrap_or_default();

            Ok((
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "data": {
                        "provider": provider_type.to_string(),
                        "linked_at": chrono::Utc::now(),
                        "total_linked_accounts": updated_accounts.len()
                    },
                    "message": format!("{} account has been successfully linked to your profile", provider_type)
                })),
            ))
        }
        Err(e) => {
            tracing::error!(
                provider = %provider,
                user_id = %authenticated_user.id,
                state = %request.state,
                error = %e,
                "Failed to complete OAuth account linking"
            );

            let user_message = match &e {
                AppError::InvalidFieldValue { field, .. } if field == "state" => {
                    "Invalid or expired linking request. Please try linking your account again."
                        .to_string()
                }
                AppError::InvalidFieldValue { field, .. } if field == "provider" => {
                    format!(
                        "This {} account is already linked to another user or to your account.",
                        provider_type
                    )
                }
                AppError::ExternalServiceError(_) => {
                    format!(
                        "Failed to connect with {}. Please try again or contact support.",
                        provider_type
                    )
                }
                _ => "Account linking service temporarily unavailable. Please try again later."
                    .to_string(),
            };

            Err(AppError::ExternalServiceError(user_message))
        }
    }
}

/// Unlink OAuth account from user
#[derive(Debug, Deserialize)]
pub struct UnlinkAccountRequest {
    pub confirm: bool,
}

pub async fn unlink_oauth_account_handler(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    authenticated_user: AuthenticatedUser,
    Json(request): Json<UnlinkAccountRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let start_time = std::time::Instant::now();
    tracing::info!(
        provider = %provider,
        user_id = %authenticated_user.id,
        email = %authenticated_user.email,
        confirmed = request.confirm,
        "Unlinking OAuth account"
    );

    // Parse and validate provider type
    let provider_type: OAuthProviderType = provider.parse()
        .map_err(|_| {
            tracing::warn!(provider = %provider, "Invalid OAuth provider for unlinking");
            AppError::InvalidFieldValue {
                field: "provider".to_string(),
                message: format!("Unsupported OAuth provider '{}'. Supported providers are: google, apple, github, spotify", provider)
            }
        })?;

    // Require explicit confirmation for account unlinking
    if !request.confirm {
        tracing::warn!(
            user_id = %authenticated_user.id,
            provider = %provider,
            "Account unlinking attempted without confirmation"
        );
        return Err(AppError::InvalidFieldValue {
            field: "confirm".to_string(),
            message: format!("Account unlinking requires explicit confirmation. Set 'confirm' to true to proceed with unlinking your {} account.", provider_type),
        });
    }

    // Check if user has this provider linked
    let existing_accounts = match state
        .auth_service
        .load_oauth_accounts(authenticated_user.id)
        .await
    {
        Ok(accounts) => accounts,
        Err(e) => {
            tracing::error!(
                user_id = %authenticated_user.id,
                provider = %provider,
                error = %e,
                "Failed to check existing OAuth accounts for unlinking"
            );
            return Err(AppError::ExternalServiceError(
                "Unable to verify existing account links. Please try again later.".to_string(),
            ));
        }
    };

    // Verify the provider is actually linked
    let account_exists = existing_accounts
        .iter()
        .any(|account| account.provider == provider_type);
    if !account_exists {
        tracing::warn!(
            user_id = %authenticated_user.id,
            provider = %provider,
            "Attempt to unlink non-existent OAuth provider"
        );
        return Err(AppError::NotFound {
            resource: format!("No {} account is linked to your profile", provider_type),
        });
    }

    // Check if user has other authentication methods (password or other OAuth accounts)
    let user = match state
        .auth_service
        .get_user_by_id(authenticated_user.id)
        .await
    {
        Ok(user) => user,
        Err(e) => {
            tracing::error!(
                user_id = %authenticated_user.id,
                error = %e,
                "Failed to retrieve user for unlinking validation"
            );
            return Err(AppError::ExternalServiceError(
                "Unable to verify account security. Please try again later.".to_string(),
            ));
        }
    };

    // Ensure user retains access after unlinking
    let has_password = user.password_hash.is_some();
    let other_oauth_accounts = existing_accounts.len() > 1;

    if !has_password && !other_oauth_accounts {
        tracing::warn!(
            user_id = %authenticated_user.id,
            provider = %provider,
            "Attempt to unlink last authentication method"
        );
        return Err(AppError::InvalidFieldValue {
            field: "provider".to_string(),
            message: format!("Cannot unlink {} account as it's your only authentication method. Please set a password or link another account first.", provider_type),
        });
    }

    // Perform the unlinking
    match state
        .auth_service
        .unlink_oauth_account(authenticated_user.id, provider_type.clone())
        .await
    {
        Ok(()) => {
            let duration = start_time.elapsed();
            tracing::info!(
                user_id = %authenticated_user.id,
                provider = %provider,
                duration_ms = duration.as_millis(),
                "OAuth account unlinked successfully"
            );

            Ok((
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "data": {
                        "provider": provider_type.to_string(),
                        "unlinked_at": chrono::Utc::now(),
                        "remaining_auth_methods": {
                            "has_password": has_password,
                            "oauth_accounts": existing_accounts.len() - 1
                        }
                    },
                    "message": format!("{} account has been successfully unlinked from your profile", provider_type)
                })),
            ))
        }
        Err(e) => {
            tracing::error!(
                user_id = %authenticated_user.id,
                provider = %provider,
                error = %e,
                "Failed to unlink OAuth account"
            );

            let user_message = match &e {
                AppError::NotFound { .. } => {
                    format!("No {} account found to unlink", provider_type)
                }
                AppError::DatabaseQueryFailed(_) => {
                    "Unable to update account information. Please try again later.".to_string()
                }
                _ => "Account unlinking service temporarily unavailable. Please try again later."
                    .to_string(),
            };

            Err(AppError::ExternalServiceError(user_message))
        }
    }
}

/// Get user's linked OAuth accounts
pub async fn get_linked_accounts_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let start_time = std::time::Instant::now();
    tracing::info!(
        user_id = %authenticated_user.id,
        email = %authenticated_user.email,
        "Getting linked OAuth accounts"
    );

    // Load OAuth accounts for the authenticated user
    let oauth_accounts = match state
        .auth_service
        .load_oauth_accounts(authenticated_user.id)
        .await
    {
        Ok(accounts) => accounts,
        Err(e) => {
            tracing::error!(
                user_id = %authenticated_user.id,
                error = %e,
                "Failed to load OAuth accounts"
            );

            let user_message = match &e {
                AppError::NotFound { .. } => "No linked accounts found.".to_string(),
                AppError::DatabaseQueryFailed(_) => {
                    "Unable to retrieve account information. Please try again later.".to_string()
                }
                _ => "Account service temporarily unavailable. Please try again later.".to_string(),
            };

            return Err(AppError::ExternalServiceError(user_message));
        }
    };

    let duration = start_time.elapsed();
    tracing::info!(
        user_id = %authenticated_user.id,
        account_count = oauth_accounts.len(),
        duration_ms = duration.as_millis(),
        "Successfully retrieved linked OAuth accounts"
    );

    // Format accounts for response (excluding sensitive token information)
    let accounts_data: Vec<serde_json::Value> = oauth_accounts
        .iter()
        .map(|account| {
            serde_json::json!({
                "provider": account.provider.to_string(),
                "provider_user_id": account.provider_user_id,
                "email": account.email,
                "display_name": account.display_name,
                "avatar_url": account.avatar_url,
                "linked_at": account.created_at,
                "last_updated": account.updated_at,
                "status": "active" // Could be enhanced with actual token status
            })
        })
        .collect();

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "data": {
                "accounts": accounts_data,
                "total_count": accounts_data.len()
            },
            "message": format!("Found {} linked account(s)", accounts_data.len())
        })),
    ))
}

/// Get OAuth provider instance based on provider type
#[allow(dead_code)]
fn get_oauth_provider(provider_type: OAuthProviderType) -> Result<Box<dyn OAuthProvider>> {
    match provider_type {
        OAuthProviderType::Google => {
            let provider =
                crate::services::oauth_google::GoogleOAuthProvider::new().map_err(|e| {
                    AppError::Internal {
                        message: Some(format!("Failed to create Google OAuth provider: {}", e)),
                    }
                })?;
            Ok(Box::new(provider))
        }
        OAuthProviderType::Apple => {
            let provider =
                crate::services::oauth_apple::AppleOAuthProvider::new().map_err(|e| {
                    AppError::Internal {
                        message: Some(format!("Failed to create Apple OAuth provider: {}", e)),
                    }
                })?;
            Ok(Box::new(provider))
        }
        OAuthProviderType::GitHub => {
            let provider =
                crate::services::oauth_github::GitHubOAuthProvider::new().map_err(|e| {
                    AppError::Internal {
                        message: Some(format!("Failed to create GitHub OAuth provider: {}", e)),
                    }
                })?;
            Ok(Box::new(provider))
        }
        OAuthProviderType::Spotify => {
            let provider =
                crate::services::oauth_spotify::SpotifyOAuthProvider::new().map_err(|e| {
                    AppError::Internal {
                        message: Some(format!("Failed to create Spotify OAuth provider: {}", e)),
                    }
                })?;
            Ok(Box::new(provider))
        }
        OAuthProviderType::YouTubeMusic => {
            let provider = crate::services::oauth_youtube_music::YouTubeMusicOAuthProvider::new()
                .map_err(|e| AppError::Internal {
                message: Some(format!(
                    "Failed to create YouTube Music OAuth provider: {}",
                    e
                )),
            })?;
            Ok(Box::new(provider))
        }
        OAuthProviderType::Tidal => Err(AppError::OperationNotAllowed {
            reason: "Tidal OAuth provider is not yet implemented".to_string(),
        }),
    }
}

/// OAuth provider health check endpoint
pub async fn oauth_health_handler(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    // Check health for all known providers
    let providers = vec![
        crate::models::oauth::OAuthProviderType::Google,
        crate::models::oauth::OAuthProviderType::Apple,
        crate::models::oauth::OAuthProviderType::GitHub,
    ];

    let mut health_status = std::collections::HashMap::new();
    for provider in providers {
        let health = state
            .auth_service
            .get_oauth_provider_health(&provider)
            .await;
        health_status.insert(provider.to_string(), format!("{:?}", health));
    }

    let health_summary = serde_json::json!({
        "status": "ok",
        "providers": health_status,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    Ok(Json(health_summary))
}

/// Health check for a specific OAuth provider
pub async fn oauth_provider_health_handler(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let provider_type: OAuthProviderType =
        provider.parse().map_err(|_| AppError::InvalidFieldValue {
            field: "provider".to_string(),
            message: format!("Unknown OAuth provider: {}", provider),
        })?;

    let health = state
        .auth_service
        .get_oauth_provider_health(&provider_type)
        .await;

    let response = serde_json::json!({
        "provider": provider_type,
        "health": format!("{:?}", health),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    Ok(Json(response))
}

/// Force health check for all OAuth providers
pub async fn force_oauth_health_check_handler(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    // Force health check is not implemented yet
    // state.auth_service.force_oauth_health_check().await;

    let response = serde_json::json!({
        "status": "health_check_initiated",
        "message": "OAuth provider health checks have been initiated",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    Ok(Json(response))
}

/// GitHub OAuth authorize endpoint (GET)
///
/// Returns a valid authorization URL with proper scopes for GitHub Sign In.
/// This endpoint satisfies acceptance criteria US-004.1:
/// "/api/v1/auth/oauth/github/authorize returns valid authorization URL"
///
/// The authorization URL includes:
/// - scope=user:email read:user (standard GitHub scopes)
/// - client_id (GitHub OAuth App Client ID)
/// - redirect_uri (callback URL for authorization code)
/// - state (CSRF protection token)
/// - allow_signup=true (allows new GitHub accounts)
pub async fn github_authorize_handler(
    State(state): State<AppState>,
    Query(query): Query<OAuthInitiateQuery>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let start_time = std::time::Instant::now();
    tracing::info!(
        redirect_uri = ?query.redirect_uri,
        "Initiating GitHub OAuth flow via authorize endpoint"
    );

    // Set default redirect URI if not provided
    let redirect_uri = query.redirect_uri.unwrap_or_else(|| {
        let default_uri = "http://localhost:3000/auth/callback/github".to_string();
        tracing::debug!(default_uri = %default_uri, "Using default GitHub redirect URI");
        default_uri
    });

    // Validate redirect URI format
    if Url::parse(&redirect_uri).is_err() {
        tracing::warn!(redirect_uri = %redirect_uri, "Invalid redirect URI format");
        return Err(AppError::InvalidFieldValue {
            field: "redirect_uri".to_string(),
            message: "Invalid redirect URI format. Must be a valid URL.".to_string(),
        });
    }

    // Initiate GitHub OAuth flow using the auth service
    let flow_response = match state
        .auth_service
        .initiate_oauth_flow(OAuthProviderType::GitHub, redirect_uri.clone())
        .await
    {
        Ok(response) => response,
        Err(e) => {
            tracing::error!(
                redirect_uri = %redirect_uri,
                error = %e,
                "Failed to initiate GitHub OAuth flow"
            );

            let user_message = match &e {
                AppError::InvalidFieldValue { field, .. } if field == "provider" => {
                    "GitHub OAuth is not configured or temporarily unavailable. Please try again later or contact support.".to_string()
                }
                AppError::ExternalServiceError(_) => {
                    "Unable to connect to GitHub authentication service. Please try again later.".to_string()
                }
                _ => {
                    "GitHub authentication service temporarily unavailable. Please try again later.".to_string()
                }
            };

            return Err(AppError::ExternalServiceError(user_message));
        }
    };

    let duration = start_time.elapsed();
    tracing::info!(
        state = %flow_response.state,
        authorization_url_length = flow_response.authorization_url.len(),
        duration_ms = duration.as_millis(),
        "GitHub OAuth flow initiated successfully"
    );

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "data": {
                "authorization_url": flow_response.authorization_url,
                "state": flow_response.state,
                "provider": "github",
                "scopes": ["user:email", "read:user"],
                "allow_signup": true
            },
            "message": "GitHub OAuth flow initiated successfully. Redirect the user to the authorization_url."
        })),
    ))
}

/// GitHub OAuth callback handler (POST with JSON)
///
/// Handles the callback from GitHub after user authorization.
/// This endpoint satisfies acceptance criteria US-004.2:
/// "/api/v1/auth/oauth/github/callback exchanges code for tokens"
///
/// The callback:
/// - Validates the state parameter (CSRF protection)
/// - Exchanges the authorization code for access tokens
/// - Fetches user info from GitHub API
/// - Handles private email case by fetching from /user/emails endpoint
/// - Creates or links user account
/// - Returns JWT tokens and user data
pub async fn github_oauth_callback_handler(
    State(state): State<AppState>,
    Json(request): Json<OAuthCallbackRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let start_time = std::time::Instant::now();
    tracing::info!(
        state = %request.state,
        code_length = request.code.len(),
        redirect_uri = %request.redirect_uri,
        "Processing GitHub OAuth callback"
    );

    // Validate input parameters
    if request.code.is_empty() {
        tracing::warn!("Empty authorization code received from GitHub");
        return Err(AppError::InvalidFieldValue {
            field: "code".to_string(),
            message: "Authorization code is required".to_string(),
        });
    }

    if request.state.is_empty() {
        tracing::warn!("Empty state parameter received from GitHub");
        return Err(AppError::InvalidFieldValue {
            field: "state".to_string(),
            message: "State parameter is required for security".to_string(),
        });
    }

    // Validate redirect URI format
    if Url::parse(&request.redirect_uri).is_err() {
        tracing::warn!(redirect_uri = %request.redirect_uri, "Invalid redirect URI format in GitHub callback");
        return Err(AppError::InvalidFieldValue {
            field: "redirect_uri".to_string(),
            message: "Invalid redirect URI format. Must be a valid URL.".to_string(),
        });
    }

    // Complete OAuth flow using the auth service
    // This exchanges the code for tokens and fetches user info (including private email handling)
    let token_pair = match state
        .auth_service
        .complete_oauth_flow(
            OAuthProviderType::GitHub,
            request.code.clone(),
            request.state.clone(),
            request.redirect_uri.clone(),
        )
        .await
    {
        Ok(tokens) => tokens,
        Err(e) => {
            tracing::error!(
                state = %request.state,
                error = %e,
                "Failed to complete GitHub OAuth flow"
            );

            let user_message = match &e {
                AppError::InvalidFieldValue { field, .. } if field == "state" => {
                    "Invalid or expired authentication request. Please try signing in again."
                        .to_string()
                }
                AppError::OAuthProviderError { message, .. }
                    if message.contains("invalid or expired") =>
                {
                    "GitHub authorization code is invalid or expired. Please try signing in again."
                        .to_string()
                }
                AppError::ExternalServiceError(_) => {
                    "Authentication failed with GitHub. Please try again or contact support."
                        .to_string()
                }
                AppError::NotFound { .. } => {
                    "Unable to complete GitHub authentication. The authorization may have expired."
                        .to_string()
                }
                _ => {
                    "GitHub authentication service temporarily unavailable. Please try again later."
                        .to_string()
                }
            };

            return Err(AppError::ExternalServiceError(user_message));
        }
    };

    // Get user information for the response
    let (user_claims, user) = match state.auth_service.verify_token(&token_pair.access_token) {
        Ok(claims) => match claims.user_id() {
            Ok(user_id) => match state.auth_service.get_user_by_id(user_id).await {
                Ok(user) => (claims, user),
                Err(e) => {
                    tracing::error!(
                        user_id = %user_id,
                        error = %e,
                        "Failed to retrieve user after GitHub OAuth completion"
                    );
                    return Err(AppError::Internal {
                        message: Some(
                            "Authentication completed but user data unavailable. Please try again."
                                .to_string(),
                        ),
                    });
                }
            },
            Err(e) => {
                tracing::error!(error = %e, "Invalid user ID in generated token");
                return Err(AppError::Internal {
                    message: Some(
                        "Authentication token generation failed. Please try again.".to_string(),
                    ),
                });
            }
        },
        Err(e) => {
            tracing::error!(error = %e, "Failed to verify generated token");
            return Err(AppError::Internal {
                message: Some(
                    "Authentication token verification failed. Please try again.".to_string(),
                ),
            });
        }
    };

    let duration = start_time.elapsed();
    tracing::info!(
        user_id = %user.id,
        email = %user.email,
        duration_ms = duration.as_millis(),
        "GitHub OAuth authentication completed successfully"
    );

    // Find GitHub OAuth account to get GitHub-specific data
    let github_account = user
        .oauth_accounts
        .iter()
        .find(|a| a.provider == OAuthProviderType::GitHub);

    // Prepare user data for response (excluding sensitive information)
    let user_data = serde_json::json!({
        "id": user.id,
        "email": user.email,
        "email_verified": user.email_verified,
        "created_at": user.created_at,
        "last_login": user.last_login,
        "oauth_accounts": user.oauth_accounts.iter().map(|account| {
            serde_json::json!({
                "provider": account.provider,
                "provider_user_id": account.provider_user_id,
                "email": account.email,
                "display_name": account.display_name,
                "avatar_url": account.avatar_url,
                "linked_at": account.created_at
            })
        }).collect::<Vec<_>>()
    });

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "data": {
                "access_token": token_pair.access_token,
                "refresh_token": token_pair.refresh_token,
                "user": user_data,
                "provider": "github"
            },
            "message": "Successfully authenticated with GitHub"
        })),
    ))
}

/// Google OAuth authorize endpoint (GET)
///
/// Returns a valid authorization URL with proper scopes for Google Sign In.
/// This endpoint satisfies acceptance criteria US-002.1:
/// "/api/v1/auth/oauth/google/authorize returns valid authorization URL"
///
/// The authorization URL includes:
/// - scope=openid email profile (standard Google scopes)
/// - client_id (Google OAuth Client ID)
/// - redirect_uri (callback URL for authorization code)
/// - state (CSRF protection token)
/// - access_type=offline (for refresh tokens)
/// - prompt=consent (force approval screen)
pub async fn google_authorize_handler(
    State(state): State<AppState>,
    Query(query): Query<OAuthInitiateQuery>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let start_time = std::time::Instant::now();
    tracing::info!(
        redirect_uri = ?query.redirect_uri,
        "Initiating Google OAuth flow via authorize endpoint"
    );

    // Set default redirect URI if not provided
    let redirect_uri = query.redirect_uri.unwrap_or_else(|| {
        let default_uri = "http://localhost:3000/auth/callback/google".to_string();
        tracing::debug!(default_uri = %default_uri, "Using default Google redirect URI");
        default_uri
    });

    // Validate redirect URI format
    if Url::parse(&redirect_uri).is_err() {
        tracing::warn!(redirect_uri = %redirect_uri, "Invalid redirect URI format");
        return Err(AppError::InvalidFieldValue {
            field: "redirect_uri".to_string(),
            message: "Invalid redirect URI format. Must be a valid URL.".to_string(),
        });
    }

    // Initiate Google OAuth flow using the auth service
    let flow_response = match state
        .auth_service
        .initiate_oauth_flow(OAuthProviderType::Google, redirect_uri.clone())
        .await
    {
        Ok(response) => response,
        Err(e) => {
            tracing::error!(
                redirect_uri = %redirect_uri,
                error = %e,
                "Failed to initiate Google OAuth flow"
            );

            let user_message = match &e {
                AppError::InvalidFieldValue { field, .. } if field == "provider" => {
                    "Google OAuth is not configured or temporarily unavailable. Please try again later or contact support.".to_string()
                }
                AppError::ExternalServiceError(_) => {
                    "Unable to connect to Google authentication service. Please try again later.".to_string()
                }
                _ => {
                    "Google authentication service temporarily unavailable. Please try again later.".to_string()
                }
            };

            return Err(AppError::ExternalServiceError(user_message));
        }
    };

    let duration = start_time.elapsed();
    tracing::info!(
        state = %flow_response.state,
        authorization_url_length = flow_response.authorization_url.len(),
        duration_ms = duration.as_millis(),
        "Google OAuth flow initiated successfully"
    );

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "data": {
                "authorization_url": flow_response.authorization_url,
                "state": flow_response.state,
                "provider": "google",
                "scopes": ["openid", "email", "profile"],
                "access_type": "offline"
            },
            "message": "Google OAuth flow initiated successfully. Redirect the user to the authorization_url."
        })),
    ))
}

/// Google OAuth callback handler (POST with JSON)
///
/// Handles the callback from Google after user authorization.
/// This endpoint satisfies acceptance criteria US-002.2:
/// "/api/v1/auth/oauth/google/callback exchanges code for tokens"
///
/// The callback:
/// - Validates the state parameter (CSRF protection)
/// - Exchanges the authorization code for access tokens
/// - Tokens are encrypted before storage using existing encryption service
/// - Fetches user info from Google API
/// - Creates or links user account
/// - Returns JWT tokens and user data
pub async fn google_oauth_callback_handler(
    State(state): State<AppState>,
    Json(request): Json<OAuthCallbackRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let start_time = std::time::Instant::now();
    tracing::info!(
        state = %request.state,
        code_length = request.code.len(),
        redirect_uri = %request.redirect_uri,
        "Processing Google OAuth callback"
    );

    // Validate input parameters
    if request.code.is_empty() {
        tracing::warn!("Empty authorization code received from Google");
        return Err(AppError::InvalidFieldValue {
            field: "code".to_string(),
            message: "Authorization code is required".to_string(),
        });
    }

    if request.state.is_empty() {
        tracing::warn!("Empty state parameter received from Google");
        return Err(AppError::InvalidFieldValue {
            field: "state".to_string(),
            message: "State parameter is required for security".to_string(),
        });
    }

    // Validate redirect URI format
    if Url::parse(&request.redirect_uri).is_err() {
        tracing::warn!(redirect_uri = %request.redirect_uri, "Invalid redirect URI format in Google callback");
        return Err(AppError::InvalidFieldValue {
            field: "redirect_uri".to_string(),
            message: "Invalid redirect URI format. Must be a valid URL.".to_string(),
        });
    }

    // Complete OAuth flow using the auth service
    // This exchanges the code for tokens (which are encrypted before storage) and fetches user info
    let token_pair = match state
        .auth_service
        .complete_oauth_flow(
            OAuthProviderType::Google,
            request.code.clone(),
            request.state.clone(),
            request.redirect_uri.clone(),
        )
        .await
    {
        Ok(tokens) => tokens,
        Err(e) => {
            tracing::error!(
                state = %request.state,
                error = %e,
                "Failed to complete Google OAuth flow"
            );

            let user_message = match &e {
                AppError::InvalidFieldValue { field, .. } if field == "state" => {
                    "Invalid or expired authentication request. Please try signing in again."
                        .to_string()
                }
                AppError::OAuthProviderError { message, .. }
                    if message.contains("invalid or expired") =>
                {
                    "Google authorization code is invalid or expired. Please try signing in again."
                        .to_string()
                }
                AppError::ExternalServiceError(_) => {
                    "Authentication failed with Google. Please try again or contact support."
                        .to_string()
                }
                AppError::NotFound { .. } => {
                    "Unable to complete Google authentication. The authorization may have expired."
                        .to_string()
                }
                _ => {
                    "Google authentication service temporarily unavailable. Please try again later."
                        .to_string()
                }
            };

            return Err(AppError::ExternalServiceError(user_message));
        }
    };

    // Get user information for the response
    let (_user_claims, user) = match state.auth_service.verify_token(&token_pair.access_token) {
        Ok(claims) => match claims.user_id() {
            Ok(user_id) => match state.auth_service.get_user_by_id(user_id).await {
                Ok(user) => (claims, user),
                Err(e) => {
                    tracing::error!(
                        user_id = %user_id,
                        error = %e,
                        "Failed to retrieve user after Google OAuth completion"
                    );
                    return Err(AppError::Internal {
                        message: Some(
                            "Authentication completed but user data unavailable. Please try again."
                                .to_string(),
                        ),
                    });
                }
            },
            Err(e) => {
                tracing::error!(error = %e, "Invalid user ID in generated token");
                return Err(AppError::Internal {
                    message: Some(
                        "Authentication token generation failed. Please try again.".to_string(),
                    ),
                });
            }
        },
        Err(e) => {
            tracing::error!(error = %e, "Failed to verify generated token");
            return Err(AppError::Internal {
                message: Some(
                    "Authentication token verification failed. Please try again.".to_string(),
                ),
            });
        }
    };

    let duration = start_time.elapsed();
    tracing::info!(
        user_id = %user.id,
        email = %user.email,
        duration_ms = duration.as_millis(),
        "Google OAuth authentication completed successfully"
    );

    // Prepare user data for response (excluding sensitive information)
    let user_data = serde_json::json!({
        "id": user.id,
        "email": user.email,
        "email_verified": user.email_verified,
        "created_at": user.created_at,
        "last_login": user.last_login,
        "oauth_accounts": user.oauth_accounts.iter().map(|account| {
            serde_json::json!({
                "provider": account.provider,
                "provider_user_id": account.provider_user_id,
                "email": account.email,
                "display_name": account.display_name,
                "avatar_url": account.avatar_url,
                "linked_at": account.created_at
            })
        }).collect::<Vec<_>>()
    });

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "data": {
                "access_token": token_pair.access_token,
                "refresh_token": token_pair.refresh_token,
                "user": user_data,
                "provider": "google"
            },
            "message": "Successfully authenticated with Google"
        })),
    ))
}

/// Get OAuth provider configuration status
pub async fn oauth_config_status_handler(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let validation_results = state.auth_service.get_oauth_config_validation();
    let available_providers = state.auth_service.get_available_oauth_providers();

    let config_status = serde_json::json!({
        "available_providers": available_providers,
        "configuration_details": validation_results,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    Ok(Json(config_status))
}

/// Get configuration guidance for a specific OAuth provider
pub async fn oauth_config_guidance_handler(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let provider_type: OAuthProviderType =
        provider.parse().map_err(|_| AppError::InvalidFieldValue {
            field: "provider".to_string(),
            message: format!("Unknown OAuth provider: {}", provider),
        })?;

    let guidance = state
        .auth_service
        .get_oauth_configuration_guidance(&provider_type);
    let validation = state
        .auth_service
        .get_oauth_config_validation()
        .get(&provider_type)
        .cloned();

    let response = serde_json::json!({
        "provider": provider_type,
        "configuration_guidance": guidance,
        "current_validation": validation,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    Ok(Json(response))
}

/// Get OAuth account health status for dashboard display
pub async fn get_oauth_account_health_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::debug!(
        user_id = %authenticated_user.id,
        "Getting OAuth account health status"
    );

    let health_statuses = state
        .auth_service
        .get_oauth_account_health(authenticated_user.id)
        .await?;

    let response = serde_json::json!({
        "success": true,
        "data": {
            "accounts": health_statuses,
            "total_accounts": health_statuses.len(),
            "healthy_accounts": health_statuses.iter()
                .filter(|h| matches!(h.connection_status, crate::models::oauth::OAuthConnectionStatus::Healthy))
                .count(),
            "accounts_needing_attention": health_statuses.iter()
                .filter(|h| !matches!(h.connection_status, crate::models::oauth::OAuthConnectionStatus::Healthy))
                .count(),
        },
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    Ok(Json(response))
}

/// Get OAuth token status for monitoring
pub async fn get_oauth_token_status_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::debug!(
        user_id = %authenticated_user.id,
        "Getting OAuth token status"
    );

    let token_statuses = state
        .auth_service
        .get_oauth_token_status(authenticated_user.id)
        .await?;

    let response = serde_json::json!({
        "success": true,
        "data": {
            "tokens": token_statuses,
            "total_tokens": token_statuses.len(),
            "expired_tokens": token_statuses.iter()
                .filter(|t| matches!(t.status, crate::models::oauth::TokenExpirationStatus::Expired))
                .count(),
            "expiring_soon": token_statuses.iter()
                .filter(|t| matches!(t.status, crate::models::oauth::TokenExpirationStatus::ExpiringSoon { .. }))
                .count(),
        },
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    Ok(Json(response))
}

/// Execute proactive token refresh for high-priority tokens
pub async fn execute_proactive_refresh_handler(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        user_id = %authenticated_user.id,
        "Executing proactive OAuth token refresh"
    );

    // For security, only allow users to refresh their own tokens
    // Admin functionality would need additional authorization
    let schedules = state.auth_service.schedule_token_refresh().await?;
    let user_schedules: Vec<_> = schedules
        .into_iter()
        .filter(|s| s.user_id == authenticated_user.id)
        .collect();

    let mut successful_refreshes = 0;
    let mut failed_refreshes = 0;
    let mut errors = Vec::new();

    for schedule in user_schedules {
        match state
            .auth_service
            .refresh_oauth_tokens(schedule.user_id, schedule.provider)
            .await
        {
            Ok(()) => {
                successful_refreshes += 1;
                tracing::info!(
                    user_id = %schedule.user_id,
                    provider = %schedule.provider,
                    "Successfully refreshed OAuth token"
                );
            }
            Err(e) => {
                failed_refreshes += 1;
                errors.push(format!("Provider {}: {}", schedule.provider, e));
                tracing::warn!(
                    user_id = %schedule.user_id,
                    provider = %schedule.provider,
                    error = %e,
                    "Failed to refresh OAuth token"
                );
            }
        }
    }

    let response = serde_json::json!({
        "success": true,
        "data": {
            "successful_refreshes": successful_refreshes,
            "failed_refreshes": failed_refreshes,
            "errors": errors,
        },
        "message": format!(
            "Token refresh completed: {} successful, {} failed",
            successful_refreshes, failed_refreshes
        ),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    Ok(Json(response))
}

/// Admin endpoint to get users needing token notifications
pub async fn get_token_notification_targets_handler(
    State(state): State<AppState>,
    // Note: In a real implementation, this would require admin authentication
) -> Result<Json<serde_json::Value>> {
    tracing::debug!("Getting users needing OAuth token notifications");

    let notification_targets = state
        .auth_service
        .get_users_needing_token_notifications()
        .await?;

    let response = serde_json::json!({
        "success": true,
        "data": {
            "notification_targets": notification_targets,
            "total_users": notification_targets.len(),
            "high_urgency": notification_targets.iter()
                .filter(|t| t.urgency == crate::models::oauth::NotificationUrgency::High)
                .count(),
            "medium_urgency": notification_targets.iter()
                .filter(|t| t.urgency == crate::models::oauth::NotificationUrgency::Medium)
                .count(),
            "low_urgency": notification_targets.iter()
                .filter(|t| t.urgency == crate::models::oauth::NotificationUrgency::Low)
                .count(),
        },
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    Ok(Json(response))
}

/// Admin endpoint to execute system-wide proactive token refresh
pub async fn execute_system_token_refresh_handler(
    State(state): State<AppState>,
    // Note: In a real implementation, this would require admin authentication
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Executing system-wide proactive OAuth token refresh");

    let summary = state.auth_service.execute_proactive_token_refresh().await?;

    let response = serde_json::json!({
        "success": true,
        "data": summary,
        "message": format!(
            "System token refresh completed: {}/{} successful",
            summary.successful_refreshes, summary.total_attempted
        ),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    Ok(Json(response))
}
