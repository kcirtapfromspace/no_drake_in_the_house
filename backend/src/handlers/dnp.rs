use axum::{
    extract::{State, Query, Path},
    response::Json,
    http::StatusCode,
};
use std::sync::Arc;
use uuid::Uuid;
use serde::Deserialize;
use crate::{
    AppState, Result, AppError,
    models::{AddToDnpRequest, DnpEntryWithArtist, Artist, AuthenticatedUser, UpdateDnpEntryRequest},
};

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
    #[serde(default = "default_limit")]
    limit: i32,
}

fn default_limit() -> i32 {
    20
}

/// Search for artists
pub async fn search_artists_handler(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        user_id = %user.id,
        query = %query.q,
        limit = query.limit,
        "Artist search request"
    );
    
    // Validate search query
    if query.q.trim().is_empty() {
        return Err(AppError::InvalidFieldValue {
            field: "q".to_string(),
            message: "Search query cannot be empty".to_string(),
        });
    }
    
    if query.q.len() > 100 {
        return Err(AppError::InvalidFieldValue {
            field: "q".to_string(),
            message: "Search query too long (max 100 characters)".to_string(),
        });
    }
    
    if query.limit > 50 {
        return Err(AppError::InvalidFieldValue {
            field: "limit".to_string(),
            message: "Limit cannot exceed 50".to_string(),
        });
    }
    
    let search_response = state.dnp_service.search_artists(&query.q, Some(query.limit as usize)).await
        .map_err(|e| {
            tracing::warn!(error = %e, user_id = %user.id, "Artist search failed");
            AppError::Internal { message: Some(e.to_string()) }
        })?;
    
    tracing::info!(
        user_id = %user.id,
        results_count = search_response.artists.len(),
        total = search_response.total,
        "Artist search completed"
    );
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "artists": search_response.artists,
            "total": search_response.total
        }
    })))
}

/// Get user's DNP list
pub async fn get_dnp_list_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(user_id = %user.id, "DNP list request");
    
    let dnp_list = state.dnp_service.get_dnp_list(user.id).await
        .map_err(|e| {
            tracing::warn!(error = %e, user_id = %user.id, "Failed to get DNP list");
            AppError::Internal { message: Some(e.to_string()) }
        })?;
    
    tracing::info!(
        user_id = %user.id,
        entries_count = dnp_list.len(),
        "DNP list retrieved"
    );
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "entries": dnp_list,
            "total": dnp_list.len()
        }
    })))
}

/// Add artist to DNP list
pub async fn add_to_dnp_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<AddToDnpRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    tracing::info!(
        user_id = %user.id,
        artist_id = %request.artist_id,
        "Add to DNP list request"
    );
    
    // Validate tags if provided
    if let Some(ref tags) = request.tags {
        if tags.len() > 10 {
            return Err(AppError::InvalidFieldValue {
                field: "tags".to_string(),
                message: "Maximum 10 tags allowed".to_string(),
            });
        }
        
        for tag in tags {
            if tag.len() > 50 {
                return Err(AppError::InvalidFieldValue {
                    field: "tags".to_string(),
                    message: "Tag length cannot exceed 50 characters".to_string(),
                });
            }
        }
    }
    
    // Validate note if provided
    if let Some(ref note) = request.note {
        if note.len() > 500 {
            return Err(AppError::InvalidFieldValue {
                field: "note".to_string(),
                message: "Note length cannot exceed 500 characters".to_string(),
            });
        }
    }
    
    let entry = state.dnp_service.add_to_dnp_list(
        user.id,
        request.artist_id,
        request.tags,
        request.note,
    ).await
        .map_err(|e| {
            tracing::warn!(error = %e, user_id = %user.id, artist_id = %request.artist_id, "Failed to add to DNP list");
            match e.to_string().as_str() {
                s if s.contains("already in DNP list") || s.contains("already exists") => {
                    AppError::AlreadyExists { resource: "DNP entry".to_string() }
                },
                s if s.contains("not found") => {
                    AppError::NotFound { resource: "Artist".to_string() }
                },
                _ => AppError::Internal { message: Some(e.to_string()) }
            }
        })?;
    
    tracing::info!(
        user_id = %user.id,
        artist_id = %request.artist_id,
        "Artist added to DNP list"
    );
    
    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "success": true,
        "data": entry,
        "message": "Artist added to DNP list successfully"
    }))))
}

/// Remove artist from DNP list
pub async fn remove_from_dnp_handler(
    State(state): State<AppState>,
    Path(artist_id): Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        user_id = %user.id,
        artist_id = %artist_id,
        "Remove from DNP list request"
    );
    
    state.dnp_service.remove_from_dnp_list(user.id, artist_id).await
        .map_err(|e| {
            tracing::warn!(error = %e, user_id = %user.id, artist_id = %artist_id, "Failed to remove from DNP list");
            match e.to_string().as_str() {
                s if s.contains("not found") => {
                    AppError::NotFound { resource: "DNP entry".to_string() }
                },
                _ => AppError::Internal { message: Some(e.to_string()) }
            }
        })?;
    
    tracing::info!(
        user_id = %user.id,
        artist_id = %artist_id,
        "Artist removed from DNP list"
    );
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Artist removed from DNP list successfully"
    })))
}

/// Update DNP entry
pub async fn update_dnp_entry_handler(
    State(state): State<AppState>,
    Path(artist_id): Path<Uuid>,
    user: AuthenticatedUser,
    Json(request): Json<UpdateDnpEntryRequest>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        user_id = %user.id,
        artist_id = %artist_id,
        "Update DNP entry request"
    );
    
    // Validate tags if provided
    if let Some(ref tags) = request.tags {
        if tags.len() > 10 {
            return Err(AppError::InvalidFieldValue {
                field: "tags".to_string(),
                message: "Maximum 10 tags allowed".to_string(),
            });
        }
        
        for tag in tags {
            if tag.len() > 50 {
                return Err(AppError::InvalidFieldValue {
                    field: "tags".to_string(),
                    message: "Tag length cannot exceed 50 characters".to_string(),
                });
            }
        }
    }
    
    // Validate note if provided
    if let Some(ref note) = request.note {
        if note.len() > 500 {
            return Err(AppError::InvalidFieldValue {
                field: "note".to_string(),
                message: "Note length cannot exceed 500 characters".to_string(),
            });
        }
    }
    
    // Check if at least one field is being updated
    if request.tags.is_none() && request.note.is_none() {
        return Err(AppError::InvalidRequestFormat(
            "At least one field (tags or note) must be provided for update".to_string()
        ));
    }
    
    let entry = state.dnp_service.update_dnp_entry(
        user.id,
        artist_id,
        request,
    ).await
        .map_err(|e| {
            tracing::warn!(error = %e, user_id = %user.id, artist_id = %artist_id, "Failed to update DNP entry");
            match e.to_string().as_str() {
                s if s.contains("not found") => {
                    AppError::NotFound { resource: "DNP entry".to_string() }
                },
                _ => AppError::Internal { message: Some(e.to_string()) }
            }
        })?;
    
    tracing::info!(
        user_id = %user.id,
        artist_id = %artist_id,
        "DNP entry updated"
    );
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": entry,
        "message": "DNP entry updated successfully"
    })))
}