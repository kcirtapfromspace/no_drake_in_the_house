use axum::{extract::Extension, http::StatusCode, response::Json};
use uuid::Uuid;

use crate::{
    models::{AddArtistToDnpRequest, UserArtistBlock},
    services::dnp_service,
    AppState,
};

// TODO: Extract user_id from JWT token in real implementation
const MOCK_USER_ID: &str = "550e8400-e29b-41d4-a716-446655440000";

pub async fn get_user_dnp_list(
    Extension(state): Extension<AppState>,
) -> Result<Json<Vec<UserArtistBlock>>, StatusCode> {
    let user_id = Uuid::parse_str(MOCK_USER_ID).unwrap();
    
    match dnp_service::get_user_dnp_list(&state.db, user_id).await {
        Ok(blocks) => Ok(Json(blocks)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn add_artist_to_dnp(
    Extension(state): Extension<AppState>,
    Json(request): Json<AddArtistToDnpRequest>,
) -> Result<Json<UserArtistBlock>, StatusCode> {
    let user_id = Uuid::parse_str(MOCK_USER_ID).unwrap();
    
    match dnp_service::add_artist_to_dnp(&state.db, user_id, request).await {
        Ok(block) => Ok(Json(block)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}