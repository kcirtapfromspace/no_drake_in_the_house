use crate::{
    models::AuthenticatedUser,
    services::user::{AccountDeletionRequest, UpdateUserProfileRequest},
    AppState, Result,
};
use axum::{
    extract::{Json as ExtractJson, State},
    response::Json,
};

/// Get user profile
pub async fn get_profile_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    let profile = state.user_service.get_profile(user.id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": profile
    })))
}

/// Update user profile
pub async fn update_profile_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    ExtractJson(request): ExtractJson<UpdateUserProfileRequest>,
) -> Result<Json<serde_json::Value>> {
    let updated_profile = state.user_service.update_profile(user.id, request).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": updated_profile
    })))
}

/// Export user data
pub async fn export_data_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    let export_data = state.user_service.export_user_data(user.id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": export_data
    })))
}

/// Delete user account
pub async fn delete_account_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    ExtractJson(request): ExtractJson<AccountDeletionRequest>,
) -> Result<Json<serde_json::Value>> {
    let deletion_result = state.user_service.delete_account(user.id, request).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": deletion_result
    })))
}
