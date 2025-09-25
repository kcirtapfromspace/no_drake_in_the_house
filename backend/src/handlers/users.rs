use axum::{extract::State, http::StatusCode, response::Json};
use uuid::Uuid;

use crate::{
    models::{CreateUserRequest, UserResponse},
    services::user_service,
    AppState,
};

pub async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
    match user_service::create_user(&state.db, request).await {
        Ok(user) => Ok(Json(user.into())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}