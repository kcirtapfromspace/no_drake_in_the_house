use crate::{
    models::{AddToDnpRequest, AuthenticatedUser, UpdateDnpEntryRequest},
    services::catalog_sync::Platform,
    AppError, AppState, Result,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// === TRACK BLOCKING TYPES ===

#[derive(Debug, Deserialize)]
pub struct AddTrackBlockRequest {
    pub artist_id: Uuid,
    pub track_id: String,
    pub track_title: String,
    pub track_role: String,
}

#[derive(Debug, Deserialize)]
pub struct BatchTrackBlockRequest {
    pub artist_id: Uuid,
    pub track_ids: Vec<String>,
    pub action: BatchAction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchAction {
    Block,
    Unblock,
}

#[derive(Debug, Serialize)]
pub struct TrackBlockEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub artist_id: Uuid,
    pub track_id: String,
    pub track_title: String,
    pub track_role: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
    #[serde(default = "default_limit")]
    limit: i32,
    /// Search Apple Music catalog in real-time (slower but comprehensive)
    #[serde(default)]
    catalog_search: bool,
}

fn default_limit() -> i32 {
    20
}

/// Search for artists - searches local DB first, then Apple Music catalog if enabled
pub async fn search_artists_handler(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        user_id = %user.id,
        query = %query.q,
        limit = query.limit,
        catalog_search = query.catalog_search,
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

    // First search local database
    let mut search_response = state
        .dnp_service
        .search_artists(&query.q, Some(query.limit as usize))
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, user_id = %user.id, "Artist search failed");
            AppError::Internal {
                message: Some(e.to_string()),
            }
        })?;

    let local_count = search_response.artists.len();

    // If local results are empty or insufficient, search Apple Music catalog
    if (search_response.artists.is_empty() || query.catalog_search)
        && local_count < query.limit as usize
    {
        tracing::info!(
            user_id = %user.id,
            local_results = local_count,
            "Searching Apple Music catalog for additional results"
        );

        // Search all platforms via orchestrator
        match state
            .catalog_sync
            .search_artist_all_platforms(&query.q, query.limit as u32)
            .await
        {
            Ok(platform_results) => {
                let mut catalog_artists = Vec::new();
                let mut persisted_ids = Vec::new();

                // Process results from each platform (prioritize Apple Music)
                for platform in [Platform::AppleMusic, Platform::Spotify, Platform::Deezer] {
                    if let Some(artists) = platform_results.get(&platform) {
                        for platform_artist in artists {
                            // Persist to database and get artist_id
                            match state
                                .catalog_sync
                                .persist_platform_artist(platform_artist, None)
                                .await
                            {
                                Ok(artist_id) => {
                                    if !persisted_ids.contains(&artist_id) {
                                        persisted_ids.push(artist_id);

                                        // Check for offenses
                                        let offense_count: i64 = sqlx::query_scalar(
                                            "SELECT COUNT(*) FROM artist_offenses WHERE artist_id = $1 AND status = 'verified'"
                                        )
                                        .bind(artist_id)
                                        .fetch_one(&state.db_pool)
                                        .await
                                        .unwrap_or(0);

                                        catalog_artists.push(serde_json::json!({
                                            "id": artist_id,
                                            "canonical_name": platform_artist.name,
                                            "image_url": platform_artist.image_url,
                                            "genres": platform_artist.genres,
                                            "popularity": platform_artist.popularity,
                                            "provider_badges": [{
                                                "provider": format!("{:?}", platform).to_lowercase(),
                                                "verified": true
                                            }],
                                            "source": "catalog",
                                            "offense_count": offense_count,
                                            "has_offenses": offense_count > 0
                                        }));
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(error = %e, "Failed to persist platform artist");
                                }
                            }

                            if catalog_artists.len() >= (query.limit as usize - local_count) {
                                break;
                            }
                        }
                    }
                    if catalog_artists.len() >= (query.limit as usize - local_count) {
                        break;
                    }
                }

                // Merge local and catalog results
                let total = local_count + catalog_artists.len();

                tracing::info!(
                    user_id = %user.id,
                    local_results = local_count,
                    catalog_results = catalog_artists.len(),
                    total = total,
                    "Catalog search completed"
                );

                // Enrich local results with offense info
                let mut enriched_local: Vec<serde_json::Value> = Vec::new();
                for artist in &search_response.artists {
                    let offense_count: i64 = sqlx::query_scalar(
                        "SELECT COUNT(*) FROM artist_offenses WHERE artist_id = $1 AND status = 'verified'"
                    )
                    .bind(artist.id)
                    .fetch_one(&state.db_pool)
                    .await
                    .unwrap_or(0);

                    enriched_local.push(serde_json::json!({
                        "id": artist.id,
                        "canonical_name": artist.canonical_name,
                        "image_url": artist.image_url,
                        "genres": artist.genres,
                        "popularity": artist.popularity,
                        "provider_badges": artist.provider_badges,
                        "source": "local",
                        "offense_count": offense_count,
                        "has_offenses": offense_count > 0
                    }));
                }

                // Combine results - local first, then catalog
                let mut combined: Vec<serde_json::Value> = enriched_local;
                combined.extend(catalog_artists);

                return Ok(Json(serde_json::json!({
                    "success": true,
                    "data": {
                        "artists": combined,
                        "total": total,
                        "sources": {
                            "local": local_count,
                            "catalog": total - local_count
                        }
                    }
                })));
            }
            Err(e) => {
                tracing::warn!(error = %e, "Catalog search failed, returning local results only");
            }
        }
    }

    // Enrich local results with offense info and graph connections
    let mut enriched: Vec<serde_json::Value> = Vec::new();
    for artist in &search_response.artists {
        let offense_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM artist_offenses WHERE artist_id = $1 AND status = 'verified'",
        )
        .bind(artist.id)
        .fetch_one(&state.db_pool)
        .await
        .unwrap_or(0);

        let has_offenses = offense_count > 0;

        // For artists with offenses, fetch their collaborators
        let collaborators = if has_offenses {
            fetch_collaborators_with_risk(&state.db_pool, artist.id).await
        } else {
            vec![]
        };

        enriched.push(serde_json::json!({
            "id": artist.id,
            "canonical_name": artist.canonical_name,
            "image_url": artist.image_url,
            "genres": artist.genres,
            "popularity": artist.popularity,
            "provider_badges": artist.provider_badges,
            "source": "local",
            "offense_count": offense_count,
            "has_offenses": has_offenses,
            "collaborators": collaborators,
            "collaborator_count": collaborators.len()
        }));
    }

    tracing::info!(
        user_id = %user.id,
        results_count = enriched.len(),
        total = search_response.total,
        "Artist search completed"
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "artists": enriched,
            "total": search_response.total,
            "sources": {
                "local": search_response.total,
                "catalog": 0
            }
        }
    })))
}

/// Get user's DNP list
pub async fn get_dnp_list_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(user_id = %user.id, "DNP list request");

    let dnp_list = state.dnp_service.get_dnp_list(user.id).await.map_err(|e| {
        tracing::warn!(error = %e, user_id = %user.id, "Failed to get DNP list");
        AppError::Internal {
            message: Some(e.to_string()),
        }
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

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "success": true,
            "data": entry,
            "message": "Artist added to DNP list successfully"
        })),
    ))
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
            "At least one field (tags or note) must be provided for update".to_string(),
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

/// Query parameters for blocked content
#[derive(Deserialize)]
pub struct BlockedContentQuery {
    /// Only block primary artist credits (ignore composer, writer, producer credits)
    #[serde(default)]
    pub primary_only: bool,
    /// Filter by specific credit roles (comma-separated: primary_artist,composer,producer,featured_artist)
    pub roles: Option<String>,
    /// Limit results
    #[serde(default = "default_content_limit")]
    pub limit: i32,
    /// Offset for pagination
    #[serde(default)]
    pub offset: i32,
}

fn default_content_limit() -> i32 {
    100
}

/// Get all tracks that should be blocked based on user's DNP list
/// This includes tracks where blocked artists have ANY credit (primary, composer, producer, etc.)
/// Use ?primary_only=true to only block tracks where artist is the primary performer
pub async fn get_blocked_tracks_handler(
    State(state): State<AppState>,
    Query(query): Query<BlockedContentQuery>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        user_id = %user.id,
        primary_only = query.primary_only,
        "Blocked tracks request"
    );

    // Parse roles filter - if primary_only is set, override roles
    let roles_filter: Option<Vec<String>> = if query.primary_only {
        Some(vec![
            "primary_artist".to_string(),
            "featured_artist".to_string(),
        ])
    } else {
        query
            .roles
            .map(|r| r.split(',').map(|s| s.trim().to_string()).collect())
    };

    // Query tracks blocked via DNP artists and their credits
    let blocked_tracks = sqlx::query_as::<
        _,
        (
            Uuid,           // track_id
            String,         // track_title
            Option<Uuid>,   // album_id
            Option<String>, // album_title
            Option<i32>,    // duration_ms
            Option<String>, // isrc
            String,         // blocked_artist_name
            Uuid,           // blocked_artist_id
            String,         // credit_role
            String,         // platform_ids (JSON)
        ),
    >(
        r#"
        WITH blocked_artists AS (
            SELECT uab.artist_id, a.canonical_name
            FROM user_artist_blocks uab
            JOIN artists a ON a.id = uab.artist_id
            WHERE uab.user_id = $1
        )
        SELECT DISTINCT ON (t.id)
            t.id AS track_id,
            t.title AS track_title,
            t.album_id,
            al.title AS album_title,
            t.duration_ms,
            t.isrc,
            ba.canonical_name AS blocked_artist_name,
            ba.artist_id AS blocked_artist_id,
            tc.role::text AS credit_role,
            jsonb_build_object(
                'apple_music_id', t.apple_music_id,
                'spotify_id', t.spotify_id,
                'deezer_id', t.deezer_id
            )::text AS platform_ids
        FROM tracks t
        JOIN track_credits tc ON tc.track_id = t.id
        JOIN blocked_artists ba ON (
            tc.artist_id = ba.artist_id
            OR tc.credited_name ILIKE '%' || ba.canonical_name || '%'
        )
        LEFT JOIN albums al ON t.album_id = al.id
        WHERE ($2::text[] IS NULL OR tc.role::text = ANY($2))
        ORDER BY t.id, tc.role
        LIMIT $3 OFFSET $4
    "#,
    )
    .bind(user.id)
    .bind(roles_filter.as_ref().map(|r| r.as_slice()))
    .bind(query.limit)
    .bind(query.offset)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to query blocked tracks");
        AppError::Internal {
            message: Some(format!("Database error: {}", e)),
        }
    })?;

    // Get total count
    let total: i64 = sqlx::query_scalar(
        r#"
        WITH blocked_artists AS (
            SELECT uab.artist_id, a.canonical_name
            FROM user_artist_blocks uab
            JOIN artists a ON a.id = uab.artist_id
            WHERE uab.user_id = $1
        )
        SELECT COUNT(DISTINCT t.id)
        FROM tracks t
        JOIN track_credits tc ON tc.track_id = t.id
        JOIN blocked_artists ba ON (
            tc.artist_id = ba.artist_id
            OR tc.credited_name ILIKE '%' || ba.canonical_name || '%'
        )
    "#,
    )
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(0);

    let tracks: Vec<serde_json::Value> = blocked_tracks
        .into_iter()
        .map(|(track_id, title, album_id, album_title, duration_ms, isrc, artist_name, artist_id, role, platform_ids)| {
            serde_json::json!({
                "track_id": track_id,
                "title": title,
                "album_id": album_id,
                "album_title": album_title,
                "duration_ms": duration_ms,
                "isrc": isrc,
                "blocked_because": {
                    "artist_name": artist_name,
                    "artist_id": artist_id,
                    "credit_role": role
                },
                "platform_ids": serde_json::from_str::<serde_json::Value>(&platform_ids).unwrap_or_default()
            })
        })
        .collect();

    tracing::info!(
        user_id = %user.id,
        blocked_tracks_count = tracks.len(),
        total = total,
        "Blocked tracks retrieved"
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "tracks": tracks,
            "total": total,
            "limit": query.limit,
            "offset": query.offset
        }
    })))
}

/// Get all albums that should be blocked based on user's DNP list
pub async fn get_blocked_albums_handler(
    State(state): State<AppState>,
    Query(query): Query<BlockedContentQuery>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        user_id = %user.id,
        "Blocked albums request"
    );

    // Query albums by blocked artists
    let blocked_albums = sqlx::query_as::<
        _,
        (
            Uuid,           // album_id
            String,         // album_title
            Option<String>, // album_type
            Option<i32>,    // total_tracks
            Option<String>, // release_date
            Option<String>, // label
            String,         // blocked_artist_name
            Uuid,           // blocked_artist_id
            String,         // platform_ids (JSON)
        ),
    >(
        r#"
        WITH blocked_artists AS (
            SELECT uab.artist_id, a.canonical_name
            FROM user_artist_blocks uab
            JOIN artists a ON a.id = uab.artist_id
            WHERE uab.user_id = $1
        )
        SELECT DISTINCT ON (al.id)
            al.id AS album_id,
            al.title AS album_title,
            al.album_type,
            al.total_tracks,
            al.release_date::text,
            al.label,
            ba.canonical_name AS blocked_artist_name,
            ba.artist_id AS blocked_artist_id,
            jsonb_build_object(
                'apple_music_id', al.apple_music_id,
                'spotify_id', al.spotify_id,
                'deezer_id', al.deezer_id
            )::text AS platform_ids
        FROM albums al
        JOIN album_artists aa ON aa.album_id = al.id
        JOIN blocked_artists ba ON aa.artist_id = ba.artist_id
        ORDER BY al.id, al.release_date DESC
        LIMIT $2 OFFSET $3
    "#,
    )
    .bind(user.id)
    .bind(query.limit)
    .bind(query.offset)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to query blocked albums");
        AppError::Internal {
            message: Some(format!("Database error: {}", e)),
        }
    })?;

    // Get total count
    let total: i64 = sqlx::query_scalar(
        r#"
        WITH blocked_artists AS (
            SELECT uab.artist_id
            FROM user_artist_blocks uab
            WHERE uab.user_id = $1
        )
        SELECT COUNT(DISTINCT al.id)
        FROM albums al
        JOIN album_artists aa ON aa.album_id = al.id
        JOIN blocked_artists ba ON aa.artist_id = ba.artist_id
    "#,
    )
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(0);

    let albums: Vec<serde_json::Value> = blocked_albums
        .into_iter()
        .map(|(album_id, title, album_type, total_tracks, release_date, label, artist_name, artist_id, platform_ids)| {
            serde_json::json!({
                "album_id": album_id,
                "title": title,
                "album_type": album_type,
                "total_tracks": total_tracks,
                "release_date": release_date,
                "label": label,
                "blocked_because": {
                    "artist_name": artist_name,
                    "artist_id": artist_id
                },
                "platform_ids": serde_json::from_str::<serde_json::Value>(&platform_ids).unwrap_or_default()
            })
        })
        .collect();

    tracing::info!(
        user_id = %user.id,
        blocked_albums_count = albums.len(),
        total = total,
        "Blocked albums retrieved"
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "albums": albums,
            "total": total,
            "limit": query.limit,
            "offset": query.offset
        }
    })))
}

/// Get revenue impact of blocking - estimates revenue diverted from blocked artists
pub async fn get_revenue_impact_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(user_id = %user.id, "Revenue impact request");

    // Get blocked artist stats with estimated revenue
    let artist_stats = sqlx::query_as::<_, (Uuid, String, Option<i64>, Option<i64>, Option<rust_decimal::Decimal>)>(r#"
        SELECT
            a.id,
            a.canonical_name,
            ass.monthly_streams,
            ass.total_streams,
            ass.estimated_monthly_revenue
        FROM user_artist_blocks uab
        JOIN artists a ON a.id = uab.artist_id
        LEFT JOIN artist_streaming_stats ass ON a.id = ass.artist_id AND ass.platform = 'apple_music'
        WHERE uab.user_id = $1
        ORDER BY ass.estimated_monthly_revenue DESC NULLS LAST
    "#)
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    // Get platform rates for reference
    let rates = sqlx::query_as::<_, (String, rust_decimal::Decimal)>(
        "SELECT platform, rate_per_stream FROM platform_streaming_rates ORDER BY rate_per_stream DESC"
    )
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    // Calculate totals
    let total_monthly_streams: i64 = artist_stats
        .iter()
        .filter_map(|(_, _, monthly, _, _)| *monthly)
        .sum();

    let total_revenue: rust_decimal::Decimal = artist_stats
        .iter()
        .filter_map(|(_, _, _, _, rev)| *rev)
        .sum();

    // Format artist breakdown
    let artists: Vec<serde_json::Value> = artist_stats
        .iter()
        .map(|(id, name, monthly, total, rev)| {
            serde_json::json!({
                "artist_id": id,
                "name": name,
                "monthly_streams": monthly,
                "total_streams": total,
                "estimated_monthly_revenue": rev.map(|r| format!("${:.2}", r))
            })
        })
        .collect();

    // Format rates
    let platform_rates: Vec<serde_json::Value> = rates
        .iter()
        .map(|(platform, rate)| {
            serde_json::json!({
                "platform": platform,
                "rate_per_stream": format!("${:.4}", rate)
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "summary": {
                "blocked_artists": artists.len(),
                "total_monthly_streams_blocked": total_monthly_streams,
                "estimated_monthly_revenue_diverted": format!("${:.2}", total_revenue),
                "note": "Revenue estimates based on average platform payouts. Actual revenue varies by contract."
            },
            "artists": artists,
            "platform_rates": platform_rates,
            "methodology": {
                "description": "Revenue estimated using industry-average per-stream rates",
                "apple_music_rate": "$0.01/stream",
                "spotify_rate": "$0.0035/stream",
                "sources": ["Apple Music for Artists", "Spotify for Artists", "Industry reports"]
            }
        }
    })))
}

/// Get artist analytics page data
pub async fn get_artist_analytics_handler(
    State(state): State<AppState>,
    Path(artist_id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(artist_id = %artist_id, "Artist analytics request");

    // Get artist info
    let artist = sqlx::query_as::<_, (String,)>("SELECT canonical_name FROM artists WHERE id = $1")
        .bind(artist_id)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(e.to_string()),
        })?
        .ok_or_else(|| AppError::NotFound {
            resource: "Artist".to_string(),
        })?;

    // Get streaming stats per platform
    let stats = sqlx::query_as::<
        _,
        (
            String,
            Option<i64>,
            Option<i64>,
            Option<i32>,
            Option<rust_decimal::Decimal>,
        ),
    >(
        r#"
        SELECT
            platform,
            monthly_streams,
            total_streams,
            popularity_score,
            estimated_monthly_revenue
        FROM artist_streaming_stats
        WHERE artist_id = $1
        ORDER BY estimated_monthly_revenue DESC NULLS LAST
    "#,
    )
    .bind(artist_id)
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    // Get track count and top tracks
    let track_count: i64 = sqlx::query_scalar(r#"
        SELECT COUNT(DISTINCT t.id)
        FROM tracks t
        JOIN track_credits tc ON tc.track_id = t.id
        WHERE tc.artist_id = $1 OR tc.credited_name ILIKE '%' || (SELECT canonical_name FROM artists WHERE id = $1) || '%'
    "#)
    .bind(artist_id)
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(0);

    let album_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(DISTINCT al.id)
        FROM albums al
        JOIN album_artists aa ON aa.album_id = al.id
        WHERE aa.artist_id = $1
    "#,
    )
    .bind(artist_id)
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(0);

    // Get collaborator count
    let collaborator_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(DISTINCT
            CASE
                WHEN tc2.artist_id != $1 THEN tc2.artist_id
            END
        )
        FROM track_credits tc1
        JOIN track_credits tc2 ON tc1.track_id = tc2.track_id
        WHERE tc1.artist_id = $1 AND tc2.artist_id IS NOT NULL AND tc2.artist_id != $1
    "#,
    )
    .bind(artist_id)
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(0);

    // Format platform stats
    let platform_stats: Vec<serde_json::Value> = stats
        .iter()
        .map(|(platform, monthly, total, pop, rev)| {
            serde_json::json!({
                "platform": platform,
                "monthly_streams": monthly,
                "total_streams": total,
                "popularity_score": pop,
                "estimated_monthly_revenue": rev.map(|r| format!("${:.2}", r))
            })
        })
        .collect();

    // Calculate totals
    let total_monthly: i64 = stats.iter().filter_map(|(_, m, _, _, _)| *m).sum();
    let total_all_time: i64 = stats.iter().filter_map(|(_, _, t, _, _)| *t).sum();
    let total_revenue: rust_decimal::Decimal = stats.iter().filter_map(|(_, _, _, _, r)| *r).sum();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "artist": {
                "id": artist_id,
                "name": artist.0,
                "track_count": track_count,
                "album_count": album_count,
                "collaborator_count": collaborator_count
            },
            "streaming": {
                "total_monthly_streams": total_monthly,
                "total_all_time_streams": total_all_time,
                "estimated_monthly_revenue": format!("${:.2}", total_revenue),
                "platforms": platform_stats
            },
            "data_freshness": "Stats are estimates based on available public data"
        }
    })))
}

/// Get aggregate revenue estimates by offense category
/// Shows how much revenue is being generated by artists in each offense category
pub async fn get_revenue_by_category_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Revenue by offense category request");

    // Get revenue aggregated by offense category
    let category_revenue = sqlx::query_as::<_, (String, i64, Option<i64>, Option<i64>, Option<rust_decimal::Decimal>)>(r#"
        SELECT
            ao.category::text AS category,
            COUNT(DISTINCT a.id) AS artist_count,
            SUM(ass.monthly_streams) AS total_monthly_streams,
            SUM(ass.total_streams) AS total_all_time_streams,
            SUM(ass.estimated_monthly_revenue) AS total_monthly_revenue
        FROM artist_offenses ao
        JOIN artists a ON a.id = ao.artist_id
        LEFT JOIN artist_streaming_stats ass ON a.id = ass.artist_id AND ass.platform = 'apple_music'
        WHERE ao.status = 'verified'
        GROUP BY ao.category
        ORDER BY total_monthly_revenue DESC NULLS LAST
    "#)
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    // Get revenue aggregated by severity level
    let severity_revenue = sqlx::query_as::<_, (String, i64, Option<i64>, Option<rust_decimal::Decimal>)>(r#"
        SELECT
            ao.severity::text AS severity,
            COUNT(DISTINCT a.id) AS artist_count,
            SUM(ass.monthly_streams) AS total_monthly_streams,
            SUM(ass.estimated_monthly_revenue) AS total_monthly_revenue
        FROM artist_offenses ao
        JOIN artists a ON a.id = ao.artist_id
        LEFT JOIN artist_streaming_stats ass ON a.id = ass.artist_id AND ass.platform = 'apple_music'
        WHERE ao.status = 'verified'
        GROUP BY ao.severity
        ORDER BY
            CASE ao.severity
                WHEN 'egregious' THEN 1
                WHEN 'severe' THEN 2
                WHEN 'moderate' THEN 3
                WHEN 'minor' THEN 4
            END
    "#)
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    // Get top offending artists by revenue
    let top_artists = sqlx::query_as::<_, (Uuid, String, String, Option<i64>, Option<rust_decimal::Decimal>)>(r#"
        SELECT DISTINCT ON (a.id)
            a.id,
            a.canonical_name,
            ao.category::text AS primary_offense,
            ass.monthly_streams,
            ass.estimated_monthly_revenue
        FROM artist_offenses ao
        JOIN artists a ON a.id = ao.artist_id
        LEFT JOIN artist_streaming_stats ass ON a.id = ass.artist_id AND ass.platform = 'apple_music'
        WHERE ao.status = 'verified'
        ORDER BY a.id, ass.estimated_monthly_revenue DESC NULLS LAST
    "#)
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    // Sort by revenue and take top 20
    let mut sorted_artists: Vec<_> = top_artists.into_iter().collect();
    sorted_artists.sort_by(|a, b| b.4.cmp(&a.4));
    let top_20: Vec<_> = sorted_artists.into_iter().take(20).collect();

    // Calculate grand totals
    let grand_total_artists: i64 = category_revenue
        .iter()
        .map(|(_, count, _, _, _)| *count)
        .sum();
    let grand_total_streams: i64 = category_revenue
        .iter()
        .filter_map(|(_, _, streams, _, _)| *streams)
        .sum();
    let grand_total_revenue: rust_decimal::Decimal = category_revenue
        .iter()
        .filter_map(|(_, _, _, _, rev)| *rev)
        .sum();

    // Format category breakdown
    let categories: Vec<serde_json::Value> = category_revenue.iter().map(|(cat, count, monthly, total, rev)| {
        serde_json::json!({
            "category": cat,
            "artist_count": count,
            "monthly_streams": monthly,
            "total_streams": total,
            "estimated_monthly_revenue": rev.map(|r| format!("${:.2}", r)).unwrap_or_else(|| "N/A".to_string()),
            "percentage_of_total": if grand_total_revenue > rust_decimal::Decimal::ZERO {
                rev.map(|r| format!("{:.1}%", (r / grand_total_revenue) * rust_decimal::Decimal::from(100)))
                    .unwrap_or_else(|| "0.0%".to_string())
            } else {
                "N/A".to_string()
            }
        })
    }).collect();

    // Format severity breakdown
    let severities: Vec<serde_json::Value> = severity_revenue.iter().map(|(sev, count, monthly, rev)| {
        serde_json::json!({
            "severity": sev,
            "artist_count": count,
            "monthly_streams": monthly,
            "estimated_monthly_revenue": rev.map(|r| format!("${:.2}", r)).unwrap_or_else(|| "N/A".to_string())
        })
    }).collect();

    // Format top artists
    let artists: Vec<serde_json::Value> = top_20.iter().map(|(id, name, offense, streams, rev)| {
        serde_json::json!({
            "artist_id": id,
            "name": name,
            "primary_offense": offense,
            "monthly_streams": streams,
            "estimated_monthly_revenue": rev.map(|r| format!("${:.2}", r)).unwrap_or_else(|| "N/A".to_string())
        })
    }).collect();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "summary": {
                "total_offending_artists": grand_total_artists,
                "total_monthly_streams": grand_total_streams,
                "total_estimated_monthly_revenue": format!("${:.2}", grand_total_revenue),
                "note": "Revenue estimates based on industry-average per-stream rates. Actual figures may vary."
            },
            "by_category": categories,
            "by_severity": severities,
            "top_earning_offenders": artists,
            "methodology": {
                "description": "Revenue aggregated from artists with verified offenses",
                "rate_source": "Industry average: Apple Music ~$0.01/stream, Spotify ~$0.0035/stream",
                "limitations": "Public streaming data may not reflect all revenue sources (touring, merchandise, licensing)"
            }
        }
    })))
}

/// Get comprehensive block summary for a user
pub async fn get_block_summary_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(user_id = %user.id, "Block summary request");

    // Get counts in parallel
    let blocked_artists: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM user_artist_blocks WHERE user_id = $1")
            .bind(user.id)
            .fetch_one(&state.db_pool)
            .await
            .unwrap_or(0);

    let blocked_tracks: i64 = sqlx::query_scalar(
        r#"
        WITH blocked_artists AS (
            SELECT uab.artist_id, a.canonical_name
            FROM user_artist_blocks uab
            JOIN artists a ON a.id = uab.artist_id
            WHERE uab.user_id = $1
        )
        SELECT COUNT(DISTINCT t.id)
        FROM tracks t
        JOIN track_credits tc ON tc.track_id = t.id
        JOIN blocked_artists ba ON (
            tc.artist_id = ba.artist_id
            OR tc.credited_name ILIKE '%' || ba.canonical_name || '%'
        )
    "#,
    )
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(0);

    let blocked_albums: i64 = sqlx::query_scalar(
        r#"
        WITH blocked_artists AS (
            SELECT uab.artist_id
            FROM user_artist_blocks uab
            WHERE uab.user_id = $1
        )
        SELECT COUNT(DISTINCT al.id)
        FROM albums al
        JOIN album_artists aa ON aa.album_id = al.id
        JOIN blocked_artists ba ON aa.artist_id = ba.artist_id
    "#,
    )
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(0);

    // Get breakdown by role
    let role_breakdown = sqlx::query_as::<_, (String, i64)>(
        r#"
        WITH blocked_artists AS (
            SELECT uab.artist_id, a.canonical_name
            FROM user_artist_blocks uab
            JOIN artists a ON a.id = uab.artist_id
            WHERE uab.user_id = $1
        )
        SELECT tc.role::text, COUNT(DISTINCT t.id)
        FROM tracks t
        JOIN track_credits tc ON tc.track_id = t.id
        JOIN blocked_artists ba ON (
            tc.artist_id = ba.artist_id
            OR tc.credited_name ILIKE '%' || ba.canonical_name || '%'
        )
        GROUP BY tc.role
        ORDER BY COUNT(DISTINCT t.id) DESC
    "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    let breakdown: std::collections::HashMap<String, i64> = role_breakdown.into_iter().collect();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "blocked_artists": blocked_artists,
            "blocked_tracks": blocked_tracks,
            "blocked_albums": blocked_albums,
            "tracks_by_role": breakdown
        }
    })))
}

// === TRACK BLOCKING HANDLERS ===

/// Add a track to user's block list
pub async fn add_track_block_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<AddTrackBlockRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    tracing::info!(
        user_id = %user.id,
        artist_id = %request.artist_id,
        track_id = %request.track_id,
        "Add track block request"
    );

    // Check if track is already blocked
    let existing: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM user_track_blocks WHERE user_id = $1 AND track_id = $2")
            .bind(user.id)
            .bind(&request.track_id)
            .fetch_optional(&state.db_pool)
            .await
            .map_err(|e| AppError::Internal {
                message: Some(e.to_string()),
            })?;

    if existing.is_some() {
        return Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": "Track already blocked"
            })),
        ));
    }

    // Insert new track block
    let block_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO user_track_blocks (id, user_id, artist_id, track_id, track_title, track_role, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, NOW())
        "#
    )
    .bind(block_id)
    .bind(user.id)
    .bind(request.artist_id)
    .bind(&request.track_id)
    .bind(&request.track_title)
    .bind(&request.track_role)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal { message: Some(e.to_string()) })?;

    tracing::info!(
        user_id = %user.id,
        track_id = %request.track_id,
        "Track blocked successfully"
    );

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "success": true,
            "data": {
                "id": block_id,
                "track_id": request.track_id,
                "track_title": request.track_title
            },
            "message": "Track blocked successfully"
        })),
    ))
}

/// Remove a track from user's block list
pub async fn remove_track_block_handler(
    State(state): State<AppState>,
    Path(track_id): Path<String>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        user_id = %user.id,
        track_id = %track_id,
        "Remove track block request"
    );

    let result = sqlx::query("DELETE FROM user_track_blocks WHERE user_id = $1 AND track_id = $2")
        .bind(user.id)
        .bind(&track_id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(e.to_string()),
        })?;

    if result.rows_affected() == 0 {
        return Ok(Json(serde_json::json!({
            "success": true,
            "message": "Track was not blocked"
        })));
    }

    tracing::info!(
        user_id = %user.id,
        track_id = %track_id,
        "Track unblocked successfully"
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Track unblocked successfully"
    })))
}

/// Get all blocked tracks for user
pub async fn get_track_blocks_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(user_id = %user.id, "Get track blocks request");

    let blocks = sqlx::query_as::<
        _,
        (
            Uuid,
            Uuid,
            String,
            String,
            String,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        r#"
        SELECT id, artist_id, track_id, track_title, track_role, created_at
        FROM user_track_blocks
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal {
        message: Some(e.to_string()),
    })?;

    let entries: Vec<serde_json::Value> = blocks
        .into_iter()
        .map(
            |(id, artist_id, track_id, track_title, track_role, created_at)| {
                serde_json::json!({
                    "id": id,
                    "artist_id": artist_id,
                    "track_id": track_id,
                    "track_title": track_title,
                    "track_role": track_role,
                    "created_at": created_at.to_rfc3339()
                })
            },
        )
        .collect();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "entries": entries,
            "total": entries.len()
        }
    })))
}

/// Batch block/unblock tracks
pub async fn batch_track_blocks_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<BatchTrackBlockRequest>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        user_id = %user.id,
        artist_id = %request.artist_id,
        track_count = request.track_ids.len(),
        action = ?request.action,
        "Batch track block request"
    );

    let mut success_count = 0;
    let mut error_count = 0;

    match request.action {
        BatchAction::Block => {
            for track_id in &request.track_ids {
                let result = sqlx::query(
                    r#"
                    INSERT INTO user_track_blocks (id, user_id, artist_id, track_id, track_title, track_role, created_at)
                    VALUES ($1, $2, $3, $4, '', 'batch', NOW())
                    ON CONFLICT (user_id, track_id) DO NOTHING
                    "#
                )
                .bind(Uuid::new_v4())
                .bind(user.id)
                .bind(request.artist_id)
                .bind(track_id)
                .execute(&state.db_pool)
                .await;

                match result {
                    Ok(_) => success_count += 1,
                    Err(e) => {
                        tracing::warn!(error = %e, track_id = %track_id, "Failed to block track");
                        error_count += 1;
                    }
                }
            }
        }
        BatchAction::Unblock => {
            for track_id in &request.track_ids {
                let result = sqlx::query(
                    "DELETE FROM user_track_blocks WHERE user_id = $1 AND track_id = $2",
                )
                .bind(user.id)
                .bind(track_id)
                .execute(&state.db_pool)
                .await;

                match result {
                    Ok(_) => success_count += 1,
                    Err(e) => {
                        tracing::warn!(error = %e, track_id = %track_id, "Failed to unblock track");
                        error_count += 1;
                    }
                }
            }
        }
    }

    tracing::info!(
        user_id = %user.id,
        success_count = success_count,
        error_count = error_count,
        "Batch track block completed"
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "processed": request.track_ids.len(),
            "successful": success_count,
            "failed": error_count
        }
    })))
}

/// Fetch collaborators for an artist with risk information
/// Returns collaborators with their offense status and connection type
async fn fetch_collaborators_with_risk(
    pool: &sqlx::PgPool,
    artist_id: Uuid,
) -> Vec<serde_json::Value> {
    // Query collaborators from artist_collaborations table
    let collaborators = sqlx::query_as::<
        _,
        (
            Uuid,           // collaborator_id
            String,         // canonical_name
            String,         // collaboration_type
            i32,            // track_count
            Option<String>, // image_url
            i64,            // offense_count
        ),
    >(
        r#"
        SELECT
            CASE
                WHEN ac.artist_id_1 = $1 THEN ac.artist_id_2
                ELSE ac.artist_id_1
            END AS collaborator_id,
            a.canonical_name,
            ac.collaboration_type,
            ac.track_count,
            a.metadata->>'image_url' as image_url,
            (SELECT COUNT(*) FROM artist_offenses ao WHERE ao.artist_id =
                CASE
                    WHEN ac.artist_id_1 = $1 THEN ac.artist_id_2
                    ELSE ac.artist_id_1
                END
                AND ao.status = 'verified'
            ) AS offense_count
        FROM artist_collaborations ac
        JOIN artists a ON a.id = CASE
            WHEN ac.artist_id_1 = $1 THEN ac.artist_id_2
            ELSE ac.artist_id_1
        END
        WHERE ac.artist_id_1 = $1 OR ac.artist_id_2 = $1
        ORDER BY ac.track_count DESC
        LIMIT 10
    "#,
    )
    .bind(artist_id)
    .fetch_all(pool)
    .await;

    match collaborators {
        Ok(collabs) => collabs
            .into_iter()
            .map(|(id, name, collab_type, track_count, image_url, offense_count)| {
                // Calculate risk level based on connection type and track count
                let risk_level = calculate_connection_risk(&collab_type, track_count);

                serde_json::json!({
                    "id": id,
                    "name": name,
                    "image_url": image_url,
                    "connection_type": collab_type,
                    "track_count": track_count,
                    "has_offenses": offense_count > 0,
                    "offense_count": offense_count,
                    "risk_level": risk_level,
                    "risk_reason": format!("{} collaboration on {} tracks", collab_type, track_count)
                })
            })
            .collect(),
        Err(e) => {
            tracing::warn!(error = %e, artist_id = %artist_id, "Failed to fetch collaborators");
            vec![]
        }
    }
}

/// Calculate risk level based on collaboration type and track count
fn calculate_connection_risk(collab_type: &str, track_count: i32) -> String {
    // Risk multipliers based on connection type
    let type_multiplier = match collab_type {
        "featured_artist" => 0.8, // High - direct musical involvement
        "producer" => 0.6,        // Medium-high - creative involvement
        "primary_artist" => 0.9,  // Very high - main collaborator
        "writer" => 0.4,          // Medium - songwriting
        "remixer" => 0.3,         // Lower - derivative work
        _ => 0.2,                 // Low - other connection types
    };

    // Track count multiplier (more tracks = higher risk)
    let track_multiplier = if track_count >= 10 {
        1.0
    } else if track_count >= 5 {
        0.7
    } else if track_count >= 2 {
        0.5
    } else {
        0.3
    };

    let risk_score = type_multiplier * track_multiplier;

    if risk_score >= 0.7 {
        "high".to_string()
    } else if risk_score >= 0.4 {
        "medium".to_string()
    } else {
        "low".to_string()
    }
}
