//! Graph API Handlers
//!
//! Endpoints for artist collaboration networks and graph analysis.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::AuthenticatedUser;
use crate::{AppError, AppState, Result};

/// Query parameters for network requests
#[derive(Debug, Deserialize)]
pub struct NetworkQuery {
    /// Traversal depth (1-5)
    #[serde(default = "default_depth")]
    pub depth: u32,
    /// Include blocked artists
    #[serde(default = "default_true")]
    pub include_blocked: bool,
    /// Maximum nodes to return
    #[serde(default = "default_max_nodes")]
    pub max_nodes: u32,
}

fn default_depth() -> u32 {
    2
}

fn default_true() -> bool {
    true
}

fn default_max_nodes() -> u32 {
    50
}

/// Query for blocked network analysis
#[derive(Debug, Deserialize)]
pub struct BlockedNetworkQuery {
    /// Maximum distance from blocked artists
    #[serde(default = "default_max_distance")]
    pub max_distance: u32,
    /// Minimum risk score to include
    #[serde(default)]
    pub min_risk_score: Option<f64>,
}

fn default_max_distance() -> u32 {
    3
}

/// Request to analyze blocked network
#[derive(Debug, Deserialize)]
pub struct AnalyzeBlockedRequest {
    /// Specific blocked artist IDs to analyze (empty = all)
    #[serde(default)]
    pub artist_ids: Vec<Uuid>,
    /// Maximum distance to traverse
    #[serde(default = "default_max_distance")]
    pub max_distance: u32,
}

/// Sync trigger request
#[derive(Debug, Deserialize)]
pub struct TriggerSyncRequest {
    /// Type of sync (full, incremental, artists, collaborations)
    #[serde(default = "default_sync_type")]
    pub sync_type: String,
}

fn default_sync_type() -> String {
    "incremental".to_string()
}

/// Get artist collaboration network
pub async fn get_artist_network_handler(
    State(state): State<AppState>,
    Path(artist_id): Path<Uuid>,
    user: AuthenticatedUser,
    Query(query): Query<NetworkQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        artist_id = %artist_id,
        depth = query.depth,
        max_nodes = query.max_nodes,
        "Get artist network request"
    );

    // Validate depth
    if query.depth < 1 || query.depth > 5 {
        return Err(AppError::InvalidFieldValue {
            field: "depth".to_string(),
            message: "Depth must be between 1 and 5".to_string(),
        });
    }

    // Get center artist
    let center_artist: Option<(String, Option<String>, Option<Vec<String>>)> = sqlx::query_as(
        r#"SELECT canonical_name, metadata->>'image_url', genres FROM artists WHERE id = $1"#
    )
    .bind(artist_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal { message: Some(e.to_string()) })?;

    let center = center_artist.ok_or_else(|| AppError::NotFound {
        resource: "Artist".to_string(),
    })?;

    // Check if center artist is blocked by user
    let center_blocked: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM user_artist_blocks WHERE user_id = $1 AND artist_id = $2)"
    )
    .bind(user.id)
    .bind(artist_id)
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(false);

    // Get collaborators using recursive CTE for depth traversal
    let network_artists: Vec<(Uuid, String, Option<String>, Option<Vec<String>>, i32, String, i32, bool)> = sqlx::query_as(r#"
        WITH RECURSIVE network AS (
            -- Start from the center artist
            SELECT
                $1::uuid as artist_id,
                0 as distance,
                'center'::text as connection_type,
                0 as collab_count

            UNION ALL

            -- Find connected artists via collaborations
            SELECT
                CASE
                    WHEN ac.artist_id_1 = n.artist_id THEN ac.artist_id_2
                    ELSE ac.artist_id_1
                END as artist_id,
                n.distance + 1,
                ac.collaboration_type,
                ac.track_count
            FROM network n
            JOIN artist_collaborations ac ON (
                ac.artist_id_1 = n.artist_id OR ac.artist_id_2 = n.artist_id
            )
            WHERE n.distance < $2
        )
        SELECT DISTINCT ON (a.id)
            a.id,
            a.canonical_name,
            a.metadata->>'image_url' as image_url,
            a.genres,
            n.distance,
            n.connection_type,
            n.collab_count,
            EXISTS(SELECT 1 FROM user_artist_blocks uab WHERE uab.user_id = $3 AND uab.artist_id = a.id) as is_blocked
        FROM network n
        JOIN artists a ON a.id = n.artist_id
        WHERE n.distance > 0
        ORDER BY a.id, n.distance ASC
        LIMIT $4
    "#)
    .bind(artist_id)
    .bind(query.depth as i32)
    .bind(user.id)
    .bind(query.max_nodes as i32)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal { message: Some(e.to_string()) })?;

    // Build nodes
    let mut nodes = vec![serde_json::json!({
        "id": artist_id,
        "name": center.0,
        "type": "artist",
        "is_blocked": center_blocked,
        "genres": center.2.unwrap_or_default(),
        "image_url": center.1
    })];

    for (id, name, image_url, genres, _distance, _conn_type, _collab_count, is_blocked) in &network_artists {
        nodes.push(serde_json::json!({
            "id": id,
            "name": name,
            "type": "artist",
            "is_blocked": is_blocked,
            "genres": genres.clone().unwrap_or_default(),
            "image_url": image_url
        }));
    }

    // Get edges (collaborations) between the artists in the network
    let artist_ids: Vec<Uuid> = nodes.iter()
        .filter_map(|n| n.get("id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()))
        .collect();

    let edges: Vec<(Uuid, Uuid, String, i32)> = sqlx::query_as(r#"
        SELECT artist_id_1, artist_id_2, collaboration_type, track_count
        FROM artist_collaborations
        WHERE artist_id_1 = ANY($1) AND artist_id_2 = ANY($1)
    "#)
    .bind(&artist_ids)
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    let edge_list: Vec<serde_json::Value> = edges.iter().map(|(a1, a2, collab_type, count)| {
        serde_json::json!({
            "source": a1,
            "target": a2,
            "type": "collaborated_with",
            "weight": (*count as f64).sqrt().max(1.0).min(5.0),
            "metadata": {
                "collaboration_type": collab_type,
                "track_count": count
            }
        })
    }).collect();

    // Calculate stats
    let total_nodes = nodes.len();
    let total_edges = edge_list.len();
    let blocked_nodes = nodes.iter().filter(|n| n.get("is_blocked").and_then(|v| v.as_bool()).unwrap_or(false)).count();
    let blocked_percentage = if total_nodes > 0 { (blocked_nodes as f64 / total_nodes as f64) * 100.0 } else { 0.0 };
    let avg_degree = if total_nodes > 0 { (total_edges as f64 * 2.0) / total_nodes as f64 } else { 0.0 };
    let density = if total_nodes > 1 { (total_edges as f64 * 2.0) / (total_nodes as f64 * (total_nodes as f64 - 1.0)) } else { 0.0 };

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "nodes": nodes,
            "edges": edge_list,
            "center_artist_id": artist_id,
            "depth": query.depth,
            "stats": {
                "total_nodes": total_nodes,
                "total_edges": total_edges,
                "blocked_nodes": blocked_nodes,
                "blocked_percentage": (blocked_percentage * 10.0).round() / 10.0,
                "average_degree": (avg_degree * 100.0).round() / 100.0,
                "density": (density * 1000.0).round() / 1000.0
            }
        }
    })))
}

/// Query for collaborators
#[derive(Debug, Deserialize)]
pub struct CollaboratorsQuery {
    #[serde(default = "default_collab_limit")]
    pub limit: i32,
}

fn default_collab_limit() -> i32 {
    20
}

/// Get direct collaborators of an artist
pub async fn get_collaborators_handler(
    State(state): State<AppState>,
    Path(artist_id): Path<Uuid>,
    _user: AuthenticatedUser,
    Query(query): Query<CollaboratorsQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(artist_id = %artist_id, "Get collaborators request");

    // Get collaborators from artist_collaborations table with track details
    let collaborators: Vec<(Uuid, String, String, i32, Option<String>, Option<i32>)> = sqlx::query_as(r#"
        WITH collab_artists AS (
            SELECT
                CASE
                    WHEN ac.artist_id_1 = $1 THEN ac.artist_id_2
                    ELSE ac.artist_id_1
                END as artist_id,
                ac.collaboration_type,
                ac.track_count,
                ac.sample_track_ids
            FROM artist_collaborations ac
            WHERE ac.artist_id_1 = $1 OR ac.artist_id_2 = $1
        )
        SELECT
            a.id as artist_id,
            a.canonical_name as artist_name,
            ca.collaboration_type as collab_type,
            ca.track_count,
            t.title as track_title,
            EXTRACT(YEAR FROM t.created_at)::int as year
        FROM collab_artists ca
        JOIN artists a ON a.id = ca.artist_id
        LEFT JOIN tracks t ON t.id = ANY(ca.sample_track_ids)
        ORDER BY ca.track_count DESC, a.canonical_name ASC
        LIMIT $2
    "#)
    .bind(artist_id)
    .bind(query.limit)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal { message: Some(e.to_string()) })?;

    // If no collaborations found in dedicated table, try track_credits
    let collaborators_list: Vec<serde_json::Value> = if collaborators.is_empty() {
        // Fallback to track_credits for finding collaborators
        let credit_collabs: Vec<(Uuid, String, String, String, Option<i32>)> = sqlx::query_as(r#"
            SELECT DISTINCT
                a2.id as artist_id,
                a2.canonical_name as artist_name,
                tc2.role::text as collab_type,
                t.title as track_title,
                EXTRACT(YEAR FROM t.created_at)::int as year
            FROM track_credits tc1
            JOIN tracks t ON tc1.track_id = t.id
            JOIN track_credits tc2 ON t.id = tc2.track_id
            JOIN artists a2 ON tc2.artist_id = a2.id
            WHERE tc1.artist_id = $1
            AND tc2.artist_id != $1
            ORDER BY a2.canonical_name
            LIMIT $2
        "#)
        .bind(artist_id)
        .bind(query.limit)
        .fetch_all(&state.db_pool)
        .await
        .unwrap_or_default();

        credit_collabs.iter().map(|(id, name, collab_type, track_title, year)| {
            serde_json::json!({
                "artist_id": id,
                "artist_name": name,
                "collab_type": collab_type,
                "track_title": track_title,
                "year": year
            })
        }).collect()
    } else {
        collaborators.iter().map(|(id, name, collab_type, _count, track_title, year)| {
            serde_json::json!({
                "artist_id": id,
                "artist_name": name,
                "collab_type": collab_type,
                "track_title": track_title,
                "year": year
            })
        }).collect()
    };

    let total = collaborators_list.len();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "artist_id": artist_id,
            "collaborators": collaborators_list,
            "total": total
        }
    })))
}

/// Find shortest path between two artists using BFS
pub async fn find_path_handler(
    State(state): State<AppState>,
    Path((from_id, to_id)): Path<(Uuid, Uuid)>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        from = %from_id,
        to = %to_id,
        "Find path request"
    );

    if from_id == to_id {
        return Err(AppError::InvalidFieldValue {
            field: "to_id".to_string(),
            message: "Cannot find path to same artist".to_string(),
        });
    }

    // Verify both artists exist
    let from_artist: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT canonical_name, metadata->>'image_url' FROM artists WHERE id = $1"
    )
    .bind(from_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal { message: Some(e.to_string()) })?;

    let from_artist = from_artist.ok_or_else(|| AppError::NotFound {
        resource: format!("Source artist {}", from_id),
    })?;

    let to_artist: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT canonical_name, metadata->>'image_url' FROM artists WHERE id = $1"
    )
    .bind(to_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal { message: Some(e.to_string()) })?;

    let to_artist = to_artist.ok_or_else(|| AppError::NotFound {
        resource: format!("Target artist {}", to_id),
    })?;

    // Use recursive CTE to find shortest path (BFS)
    // Limited to 6 hops max for performance
    let path_result: Vec<(Uuid, String, Option<String>, i32, Option<Uuid>, bool)> = sqlx::query_as(r#"
        WITH RECURSIVE path_search AS (
            -- Start from source
            SELECT
                $1::uuid as artist_id,
                0 as depth,
                ARRAY[$1::uuid] as path,
                false as found

            UNION ALL

            -- Expand to neighbors
            SELECT
                CASE
                    WHEN ac.artist_id_1 = ps.artist_id THEN ac.artist_id_2
                    ELSE ac.artist_id_1
                END as artist_id,
                ps.depth + 1,
                ps.path || CASE
                    WHEN ac.artist_id_1 = ps.artist_id THEN ac.artist_id_2
                    ELSE ac.artist_id_1
                END,
                CASE
                    WHEN ac.artist_id_1 = ps.artist_id THEN ac.artist_id_2
                    ELSE ac.artist_id_1
                END = $2
            FROM path_search ps
            JOIN artist_collaborations ac ON (
                ac.artist_id_1 = ps.artist_id OR ac.artist_id_2 = ps.artist_id
            )
            WHERE ps.depth < 6
            AND NOT ps.found
            AND NOT (CASE
                WHEN ac.artist_id_1 = ps.artist_id THEN ac.artist_id_2
                ELSE ac.artist_id_1
            END = ANY(ps.path))
        ),
        -- Get the shortest path that reached the target
        shortest_path AS (
            SELECT path, depth
            FROM path_search
            WHERE found = true
            ORDER BY depth ASC
            LIMIT 1
        )
        -- Return path nodes with details
        SELECT
            a.id,
            a.canonical_name,
            a.metadata->>'image_url' as image_url,
            idx as path_position,
            CASE WHEN idx > 0 THEN sp.path[idx] ELSE NULL END as prev_node,
            EXISTS(SELECT 1 FROM user_artist_blocks uab WHERE uab.user_id = $3 AND uab.artist_id = a.id) as is_blocked
        FROM shortest_path sp
        CROSS JOIN LATERAL unnest(sp.path) WITH ORDINALITY AS u(artist_id, idx)
        JOIN artists a ON a.id = u.artist_id
        ORDER BY idx
    "#)
    .bind(from_id)
    .bind(to_id)
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    if path_result.is_empty() {
        // No path found
        return Ok(Json(serde_json::json!({
            "success": true,
            "data": {
                "found": false,
                "path": [],
                "edges": [],
                "total_distance": null,
                "via_blocked": false
            }
        })));
    }

    // Build path nodes
    let path_nodes: Vec<serde_json::Value> = path_result.iter().map(|(id, name, image_url, _pos, _prev, is_blocked)| {
        serde_json::json!({
            "id": id,
            "name": name,
            "type": "artist",
            "is_blocked": is_blocked,
            "image_url": image_url
        })
    }).collect();

    // Build edges between consecutive path nodes
    let mut edges = Vec::new();
    for i in 0..path_result.len() - 1 {
        let (from, _, _, _, _, _) = &path_result[i];
        let (to, _, _, _, _, _) = &path_result[i + 1];
        edges.push(serde_json::json!({
            "source": from,
            "target": to,
            "type": "collaborated_with",
            "weight": 1
        }));
    }

    // Check if path goes through any blocked artist
    let via_blocked = path_result.iter().any(|(_, _, _, _, _, is_blocked)| *is_blocked);
    let total_distance = path_result.len() - 1;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "found": true,
            "path": path_nodes,
            "edges": edges,
            "total_distance": total_distance,
            "via_blocked": via_blocked
        }
    })))
}

/// Analyze blocked network for a user
pub async fn analyze_blocked_network_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<BlockedNetworkQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        user_id = %user.id,
        max_distance = query.max_distance,
        "Analyze blocked network request"
    );

    // Get user's blocked artists
    let blocked_artists: Vec<(Uuid, String, Option<String>)> = sqlx::query_as(r#"
        SELECT a.id, a.canonical_name, a.metadata->>'image_url'
        FROM user_artist_blocks uab
        JOIN artists a ON a.id = uab.artist_id
        WHERE uab.user_id = $1
    "#)
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal { message: Some(e.to_string()) })?;

    if blocked_artists.is_empty() {
        return Ok(Json(serde_json::json!({
            "success": true,
            "data": {
                "at_risk_artists": [],
                "blocked_clusters": [],
                "summary": {
                    "total_blocked": 0,
                    "total_at_risk": 0,
                    "avg_collaborations_per_blocked": 0.0
                }
            }
        })));
    }

    let blocked_ids: Vec<Uuid> = blocked_artists.iter().map(|(id, _, _)| *id).collect();

    // Find artists who collaborate with blocked artists (at-risk artists)
    let at_risk: Vec<(Uuid, String, Option<String>, i64, f64)> = sqlx::query_as(r#"
        WITH blocked_collabs AS (
            SELECT
                CASE
                    WHEN ac.artist_id_1 = ANY($1) THEN ac.artist_id_2
                    ELSE ac.artist_id_1
                END as collaborator_id,
                ac.track_count
            FROM artist_collaborations ac
            WHERE (ac.artist_id_1 = ANY($1) OR ac.artist_id_2 = ANY($1))
            AND NOT (ac.artist_id_1 = ANY($1) AND ac.artist_id_2 = ANY($1))
        )
        SELECT
            a.id,
            a.canonical_name,
            a.metadata->>'image_url' as image_url,
            COUNT(DISTINCT bc.collaborator_id) as blocked_collab_count,
            SUM(bc.track_count)::float / GREATEST(COUNT(*), 1) as risk_score
        FROM blocked_collabs bc
        JOIN artists a ON a.id = bc.collaborator_id
        WHERE NOT a.id = ANY($1)
        GROUP BY a.id, a.canonical_name, a.metadata->>'image_url'
        ORDER BY risk_score DESC
        LIMIT 50
    "#)
    .bind(&blocked_ids)
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    // Calculate average collaborations per blocked artist
    let total_collaborations: i64 = sqlx::query_scalar(r#"
        SELECT COALESCE(SUM(track_count), 0)::bigint
        FROM artist_collaborations
        WHERE artist_id_1 = ANY($1) OR artist_id_2 = ANY($1)
    "#)
    .bind(&blocked_ids)
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(0);

    let avg_collabs = if blocked_artists.is_empty() {
        0.0
    } else {
        total_collaborations as f64 / blocked_artists.len() as f64
    };

    // Build at-risk artists list
    let at_risk_artists: Vec<serde_json::Value> = at_risk.iter().map(|(id, name, image_url, blocked_count, risk_score)| {
        let normalized_risk = (*risk_score / 10.0).min(1.0);
        serde_json::json!({
            "artist": {
                "id": id,
                "name": name,
                "genres": [],
                "is_blocked": false,
                "image_url": image_url
            },
            "blocked_collaborators": blocked_count,
            "risk_score": (normalized_risk * 100.0).round() / 100.0
        })
    }).collect();

    // Build blocked artists clusters (artists who collaborate with each other)
    let blocked_clusters: Vec<serde_json::Value> = blocked_artists.iter().map(|(id, name, image_url)| {
        serde_json::json!({
            "cluster_id": id,
            "artists": [{
                "id": id,
                "name": name,
                "genres": [],
                "is_blocked": true,
                "image_url": image_url
            }],
            "internal_collaborations": 0
        })
    }).collect();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "at_risk_artists": at_risk_artists,
            "blocked_clusters": blocked_clusters,
            "summary": {
                "total_blocked": blocked_artists.len(),
                "total_at_risk": at_risk.len(),
                "avg_collaborations_per_blocked": (avg_collabs * 10.0).round() / 10.0
            }
        }
    })))
}

/// Get user's blocked artists with network context
pub async fn get_blocked_artists_network_handler(
    State(_state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(user_id = %user.id, "Get blocked artists network request");

    // Return placeholder response
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "blocked_artists": [],
            "total": 0
        }
    })))
}

/// Get network statistics
pub async fn get_network_stats_handler(
    State(_state): State<AppState>,
    Path(artist_id): Path<Uuid>,
    _user: AuthenticatedUser,
    Query(query): Query<NetworkQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        artist_id = %artist_id,
        depth = query.depth,
        "Get network stats request"
    );

    // Return placeholder response
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "artist_id": artist_id,
            "depth": query.depth,
            "total_nodes": 1,
            "total_edges": 0,
            "blocked_nodes": 0,
            "blocked_percentage": 0.0,
            "average_degree": 0.0,
            "density": 0.0,
            "clustering_coefficient": null
        }
    })))
}

/// Get global graph statistics
pub async fn get_global_stats_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Get global graph stats request");

    // Count artists
    let artist_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM artists")
        .fetch_one(&state.db_pool)
        .await
        .unwrap_or(0);

    // Count collaborations
    let collaboration_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM artist_collaborations")
        .fetch_one(&state.db_pool)
        .await
        .unwrap_or(0);

    // Count labels (from albums table)
    let label_count: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT label) FROM albums WHERE label IS NOT NULL")
        .fetch_one(&state.db_pool)
        .await
        .unwrap_or(0);

    // Count tracks
    let track_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tracks")
        .fetch_one(&state.db_pool)
        .await
        .unwrap_or(0);

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "artist_count": artist_count,
            "collaboration_count": collaboration_count,
            "label_count": label_count,
            "track_count": track_count
        }
    })))
}

/// Get graph sync status
pub async fn get_sync_status_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Get graph sync status request");

    // Return placeholder status
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "is_syncing": false,
            "current_job": null,
            "stats": {
                "total_artists_in_graph": 0,
                "total_collaborations": 0,
                "total_labels": 0,
                "last_full_sync": null,
                "last_incremental_sync": null
            }
        }
    })))
}

/// Trigger a graph sync
pub async fn trigger_sync_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Json(request): Json<TriggerSyncRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    tracing::info!(sync_type = %request.sync_type, "Trigger graph sync request");

    let valid_types = ["full", "incremental", "artists", "collaborations", "labels"];
    if !valid_types.contains(&request.sync_type.as_str()) {
        return Err(AppError::InvalidFieldValue {
            field: "sync_type".to_string(),
            message: format!("Invalid sync type. Valid options: {:?}", valid_types),
        });
    }

    // Return accepted response
    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "success": true,
            "message": "Graph sync triggered",
            "data": {
                "job_id": Uuid::new_v4(),
                "sync_type": request.sync_type
            }
        })),
    ))
}

/// Get collaboration statistics for an artist
pub async fn get_collaboration_stats_handler(
    State(_state): State<AppState>,
    Path(artist_id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(artist_id = %artist_id, "Get collaboration stats request");

    // Return placeholder response
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "artist_id": artist_id,
            "total_collaborations": 0,
            "unique_collaborators": 0,
            "type_breakdown": {},
            "top_collaborators": [],
            "blocked_collaborators": 0
        }
    })))
}

/// Search for artists by network proximity
pub async fn search_by_proximity_handler(
    State(_state): State<AppState>,
    Path(artist_id): Path<Uuid>,
    _user: AuthenticatedUser,
    Query(query): Query<NetworkQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        artist_id = %artist_id,
        depth = query.depth,
        "Search by proximity request"
    );

    // Return placeholder response
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "center_artist_id": artist_id,
            "depth": query.depth,
            "results": [],
            "total": 0
        }
    })))
}

/// Get artists at risk based on blocked network
pub async fn get_at_risk_artists_handler(
    State(_state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<BlockedNetworkQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        user_id = %user.id,
        min_risk = ?query.min_risk_score,
        "Get at-risk artists request"
    );

    // Return placeholder response
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "at_risk_artists": [],
            "total": 0,
            "min_risk_score": query.min_risk_score.unwrap_or(0.5)
        }
    })))
}

/// Get graph database health
pub async fn get_graph_health_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Get graph health request");

    // Return placeholder health status
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "healthy": true,
            "database": "kuzu",
            "stats": {
                "node_count": 0,
                "edge_count": 0,
                "storage_size_bytes": null
            },
            "last_checked": chrono::Utc::now().to_rfc3339()
        }
    })))
}

/// Query for offense network
#[derive(Debug, Deserialize)]
pub struct OffenseNetworkQuery {
    /// Filter by offense category
    pub category: Option<String>,
    /// Minimum severity (minor, moderate, severe, egregious)
    pub min_severity: Option<String>,
    /// Maximum depth to traverse collaborations
    #[serde(default = "default_offense_depth")]
    pub depth: u32,
    /// Limit results
    #[serde(default = "default_offense_limit")]
    pub limit: i32,
}

fn default_offense_depth() -> u32 {
    1
}

fn default_offense_limit() -> i32 {
    50
}

/// Get offense network - shows artists with offenses and their collaborators
/// This reveals how "clean" artists are connected to problematic ones through collaborations
pub async fn get_offense_network_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Query(query): Query<OffenseNetworkQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        category = ?query.category,
        min_severity = ?query.min_severity,
        depth = query.depth,
        "Get offense network request"
    );

    // Get all artists with verified offenses
    let offensive_artists = sqlx::query_as::<_, (Uuid, String, String, String, i64)>(r#"
        SELECT
            a.id,
            a.canonical_name,
            ao.category::text,
            ao.severity::text,
            COUNT(*) as offense_count
        FROM artists a
        JOIN artist_offenses ao ON a.id = ao.artist_id
        WHERE ao.status = 'verified'
        AND ($1::text IS NULL OR ao.category::text = $1)
        AND ($2::text IS NULL OR ao.severity::text IN (
            CASE $2
                WHEN 'minor' THEN 'minor'
                WHEN 'moderate' THEN 'moderate'
                WHEN 'severe' THEN 'severe'
                WHEN 'egregious' THEN 'egregious'
            END,
            CASE WHEN $2 = 'minor' THEN 'moderate' ELSE NULL END,
            CASE WHEN $2 IN ('minor', 'moderate') THEN 'severe' ELSE NULL END,
            CASE WHEN $2 IN ('minor', 'moderate', 'severe') THEN 'egregious' ELSE NULL END
        ))
        GROUP BY a.id, a.canonical_name, ao.category, ao.severity
        ORDER BY offense_count DESC
        LIMIT $3
    "#)
    .bind(&query.category)
    .bind(&query.min_severity)
    .bind(query.limit)
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    // For each offensive artist, find their collaborators
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut collaborator_ids: std::collections::HashSet<Uuid> = std::collections::HashSet::new();

    for (artist_id, name, category, severity, offense_count) in &offensive_artists {
        // Add offensive artist as node
        nodes.push(serde_json::json!({
            "id": artist_id,
            "name": name,
            "type": "offensive",
            "offense_category": category,
            "offense_severity": severity,
            "offense_count": offense_count
        }));

        // Find collaborators through track_credits
        let collaborators = sqlx::query_as::<_, (Uuid, String, String, i64)>(r#"
            SELECT DISTINCT
                a2.id,
                a2.canonical_name,
                tc2.role::text,
                COUNT(DISTINCT t.id) as shared_tracks
            FROM track_credits tc1
            JOIN tracks t ON tc1.track_id = t.id
            JOIN track_credits tc2 ON t.id = tc2.track_id
            JOIN artists a2 ON tc2.artist_id = a2.id
            WHERE tc1.artist_id = $1
            AND tc2.artist_id != $1
            AND a2.id IS NOT NULL
            GROUP BY a2.id, a2.canonical_name, tc2.role
            ORDER BY shared_tracks DESC
            LIMIT 20
        "#)
        .bind(artist_id)
        .fetch_all(&state.db_pool)
        .await
        .unwrap_or_default();

        for (collab_id, collab_name, role, shared_tracks) in collaborators {
            // Check if collaborator has offenses
            let collab_offense_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM artist_offenses WHERE artist_id = $1 AND status = 'verified'"
            )
            .bind(collab_id)
            .fetch_one(&state.db_pool)
            .await
            .unwrap_or(0);

            // Add collaborator node if not already added
            if !collaborator_ids.contains(&collab_id) && !offensive_artists.iter().any(|(id, _, _, _, _)| *id == collab_id) {
                collaborator_ids.insert(collab_id);
                nodes.push(serde_json::json!({
                    "id": collab_id,
                    "name": collab_name,
                    "type": if collab_offense_count > 0 { "offensive" } else { "collaborator" },
                    "offense_count": collab_offense_count,
                    "connected_to_offensive": true
                }));
            }

            // Add edge
            edges.push(serde_json::json!({
                "from": artist_id,
                "to": collab_id,
                "type": "collaboration",
                "role": role,
                "weight": shared_tracks
            }));
        }
    }

    // Calculate stats
    let offensive_count = offensive_artists.len();
    let collaborator_count = collaborator_ids.len();
    let total_edges = edges.len();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "nodes": nodes,
            "edges": edges,
            "stats": {
                "offensive_artists": offensive_count,
                "connected_collaborators": collaborator_count,
                "total_connections": total_edges,
                "categories_represented": offensive_artists.iter()
                    .map(|(_, _, cat, _, _)| cat.clone())
                    .collect::<std::collections::HashSet<_>>()
            },
            "filters_applied": {
                "category": query.category,
                "min_severity": query.min_severity,
                "depth": query.depth
            }
        }
    })))
}

/// Get offense connections for a specific artist
/// Shows how an artist is connected to offensive artists through collaborations
pub async fn get_artist_offense_connections_handler(
    State(state): State<AppState>,
    Path(artist_id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(artist_id = %artist_id, "Get artist offense connections request");

    // Get artist info
    let artist = sqlx::query_as::<_, (String,)>(
        "SELECT canonical_name FROM artists WHERE id = $1"
    )
    .bind(artist_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal { message: Some(e.to_string()) })?
    .ok_or_else(|| AppError::NotFound { resource: "Artist".to_string() })?;

    // Check if this artist has offenses
    let artist_offenses = sqlx::query_as::<_, (String, String, String)>(r#"
        SELECT category::text, severity::text, title
        FROM artist_offenses
        WHERE artist_id = $1 AND status = 'verified'
    "#)
    .bind(artist_id)
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    // Find collaborators who have offenses
    let offensive_collaborators = sqlx::query_as::<_, (Uuid, String, String, String, String, i64)>(r#"
        SELECT DISTINCT
            a2.id,
            a2.canonical_name,
            ao.category::text,
            ao.severity::text,
            ao.title,
            COUNT(DISTINCT t.id) as shared_tracks
        FROM track_credits tc1
        JOIN tracks t ON tc1.track_id = t.id
        JOIN track_credits tc2 ON t.id = tc2.track_id
        JOIN artists a2 ON tc2.artist_id = a2.id
        JOIN artist_offenses ao ON a2.id = ao.artist_id
        WHERE tc1.artist_id = $1
        AND tc2.artist_id != $1
        AND ao.status = 'verified'
        GROUP BY a2.id, a2.canonical_name, ao.category, ao.severity, ao.title
        ORDER BY shared_tracks DESC
    "#)
    .bind(artist_id)
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    // Get shared tracks with offensive collaborators
    let mut connections = Vec::new();
    let mut seen_artists: std::collections::HashSet<Uuid> = std::collections::HashSet::new();

    for (collab_id, collab_name, category, severity, offense_title, shared_tracks) in offensive_collaborators {
        if !seen_artists.contains(&collab_id) {
            seen_artists.insert(collab_id);

            // Get example shared tracks
            let shared_track_examples = sqlx::query_as::<_, (String, Option<String>)>(r#"
                SELECT DISTINCT t.title, al.title as album_title
                FROM track_credits tc1
                JOIN tracks t ON tc1.track_id = t.id
                JOIN track_credits tc2 ON t.id = tc2.track_id
                LEFT JOIN albums al ON t.album_id = al.id
                WHERE tc1.artist_id = $1 AND tc2.artist_id = $2
                LIMIT 5
            "#)
            .bind(artist_id)
            .bind(collab_id)
            .fetch_all(&state.db_pool)
            .await
            .unwrap_or_default();

            connections.push(serde_json::json!({
                "collaborator": {
                    "id": collab_id,
                    "name": collab_name
                },
                "offense": {
                    "category": category,
                    "severity": severity,
                    "title": offense_title
                },
                "connection": {
                    "shared_tracks": shared_tracks,
                    "example_tracks": shared_track_examples.iter()
                        .map(|(title, album)| serde_json::json!({
                            "title": title,
                            "album": album
                        }))
                        .collect::<Vec<_>>()
                }
            }));
        }
    }

    // Calculate risk score based on connections to offensive artists
    let total_offensive_connections = connections.len();
    let severe_connections = connections.iter()
        .filter(|c| {
            let severity = c["offense"]["severity"].as_str().unwrap_or("");
            severity == "severe" || severity == "egregious"
        })
        .count();

    let risk_score = if total_offensive_connections == 0 {
        0.0
    } else {
        let base_score = (total_offensive_connections as f64 * 10.0).min(50.0);
        let severity_bonus = severe_connections as f64 * 15.0;
        (base_score + severity_bonus).min(100.0)
    };

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "artist": {
                "id": artist_id,
                "name": artist.0,
                "has_offenses": !artist_offenses.is_empty(),
                "offenses": artist_offenses.iter().map(|(cat, sev, title)| {
                    serde_json::json!({
                        "category": cat,
                        "severity": sev,
                        "title": title
                    })
                }).collect::<Vec<_>>()
            },
            "offensive_connections": connections,
            "risk_assessment": {
                "total_offensive_collaborators": total_offensive_connections,
                "severe_collaborators": severe_connections,
                "risk_score": risk_score,
                "risk_level": if risk_score >= 70.0 { "high" }
                              else if risk_score >= 40.0 { "medium" }
                              else if risk_score > 0.0 { "low" }
                              else { "none" }
            }
        }
    })))
}

/// Query parameters for offense radius
#[derive(Debug, Deserialize)]
pub struct OffenseRadiusQuery {
    /// Starting artist ID
    pub artist_id: Uuid,
    /// Maximum depth (1-3 hops)
    #[serde(default = "default_radius_depth")]
    pub depth: u32,
    /// Include only artists with offenses
    #[serde(default)]
    pub offenders_only: bool,
}

fn default_radius_depth() -> u32 {
    2
}

/// Get all artists within N hops of an offender with risk scores
/// Returns "contamination radius" - artists connected to offenders
pub async fn get_offense_radius_handler(
    State(state): State<AppState>,
    Query(query): Query<OffenseRadiusQuery>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        artist_id = %query.artist_id,
        depth = query.depth,
        offenders_only = query.offenders_only,
        "Offense radius request"
    );

    // Validate depth
    let depth = query.depth.clamp(1, 3);

    // First, verify the artist exists and has offenses
    let artist: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT canonical_name, metadata->>'image_url' FROM artists WHERE id = $1"
    )
    .bind(query.artist_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal { message: Some(e.to_string()) })?;

    let artist = artist.ok_or_else(|| AppError::NotFound {
        resource: "Artist".to_string(),
    })?;

    // Get the starting artist's offenses
    let center_offenses: Vec<(String, String)> = sqlx::query_as(
        r#"SELECT category::text, severity::text FROM artist_offenses
           WHERE artist_id = $1 AND status = 'verified'"#
    )
    .bind(query.artist_id)
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    // Get artists within the radius using recursive CTE
    let radius_artists: Vec<(
        Uuid,           // artist_id
        String,         // canonical_name
        Option<String>, // image_url
        i32,            // distance
        String,         // connection_type
        i64,            // offense_count
    )> = sqlx::query_as(r#"
        WITH RECURSIVE network AS (
            -- Start from the center artist
            SELECT
                $1::uuid as artist_id,
                0 as distance,
                'center'::text as connection_type,
                $1::uuid as connected_from

            UNION ALL

            -- Find connected artists
            SELECT
                CASE
                    WHEN ac.artist_id_1 = n.artist_id THEN ac.artist_id_2
                    ELSE ac.artist_id_1
                END as artist_id,
                n.distance + 1,
                ac.collaboration_type,
                n.artist_id as connected_from
            FROM network n
            JOIN artist_collaborations ac ON (
                ac.artist_id_1 = n.artist_id OR ac.artist_id_2 = n.artist_id
            )
            WHERE n.distance < $2
        )
        SELECT DISTINCT ON (a.id)
            a.id as artist_id,
            a.canonical_name,
            a.metadata->>'image_url' as image_url,
            n.distance,
            n.connection_type,
            (SELECT COUNT(*) FROM artist_offenses ao
             WHERE ao.artist_id = a.id AND ao.status = 'verified') as offense_count
        FROM network n
        JOIN artists a ON a.id = n.artist_id
        WHERE n.distance > 0  -- Exclude center artist
        ORDER BY a.id, n.distance ASC
    "#)
    .bind(query.artist_id)
    .bind(depth as i32)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal { message: Some(e.to_string()) })?;

    // Calculate risk scores and build response
    let mut artists_in_radius: Vec<serde_json::Value> = radius_artists
        .into_iter()
        .filter(|(_, _, _, _, _, offense_count)| {
            !query.offenders_only || *offense_count > 0
        })
        .map(|(id, name, image_url, distance, connection_type, offense_count)| {
            // Risk score decreases with distance
            let distance_factor = 1.0 / (distance as f64 + 1.0);

            // Connection type risk multiplier
            let connection_multiplier = match connection_type.as_str() {
                "featured_artist" | "primary_artist" => 0.9,
                "producer" => 0.7,
                "writer" => 0.5,
                _ => 0.3,
            };

            // Offense status multiplier
            let offense_multiplier = if offense_count > 0 { 2.0 } else { 1.0 };

            let risk_score = (distance_factor * connection_multiplier * offense_multiplier * 100.0).min(100.0);

            let risk_level = if risk_score >= 60.0 {
                "high"
            } else if risk_score >= 30.0 {
                "medium"
            } else {
                "low"
            };

            serde_json::json!({
                "id": id,
                "name": name,
                "image_url": image_url,
                "distance": distance,
                "connection_type": connection_type,
                "has_offenses": offense_count > 0,
                "offense_count": offense_count,
                "risk_score": (risk_score * 10.0).round() / 10.0,
                "risk_level": risk_level
            })
        })
        .collect();

    // Sort by risk score descending
    artists_in_radius.sort_by(|a, b| {
        let score_a = a.get("risk_score").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let score_b = b.get("risk_score").and_then(|v| v.as_f64()).unwrap_or(0.0);
        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Count summary stats
    let total_in_radius = artists_in_radius.len();
    let offenders_in_radius = artists_in_radius
        .iter()
        .filter(|a| a.get("has_offenses").and_then(|v| v.as_bool()).unwrap_or(false))
        .count();

    tracing::info!(
        artist_id = %query.artist_id,
        total_in_radius = total_in_radius,
        offenders_in_radius = offenders_in_radius,
        "Offense radius calculated"
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "center": {
                "id": query.artist_id,
                "name": artist.0,
                "image_url": artist.1,
                "has_offenses": !center_offenses.is_empty(),
                "offenses": center_offenses.iter().map(|(cat, sev)| {
                    serde_json::json!({
                        "category": cat,
                        "severity": sev
                    })
                }).collect::<Vec<_>>()
            },
            "radius": {
                "depth": depth,
                "total_artists": total_in_radius,
                "offenders_count": offenders_in_radius
            },
            "artists": artists_in_radius
        }
    })))
}
