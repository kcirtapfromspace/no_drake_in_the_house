use axum::{extract::{Query, State}, http::StatusCode, response::Json};

use crate::{
    models::{ArtistResponse, SearchArtistRequest},
    services::artist_service,
    AppState,
};

pub async fn search_artists(
    State(state): State<AppState>,
    Query(request): Query<SearchArtistRequest>,
) -> Result<Json<Vec<ArtistResponse>>, StatusCode> {
    match artist_service::search_artists(&state.db, request).await {
        Ok(artists) => Ok(Json(artists.into_iter().map(|a| a.into()).collect())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}