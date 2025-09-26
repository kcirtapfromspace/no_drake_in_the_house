use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use music_streaming_blocklist_backend::models::*;
use music_streaming_blocklist_backend::services::*;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

async fn create_test_app(pool: PgPool) -> Router {
    // Initialize services
    let entity_service = Arc::new(EntityResolutionService::new(pool.clone()));
    let auth_service = Arc::new(AuthService::new(pool.clone()));
    let token_vault_service = Arc::new(TokenVaultService::new(pool.clone()));
    let external_api_service = Arc::new(ExternalApiService::new());
    let spotify_service = Arc::new(SpotifyService::new(
        pool.clone(),
        token_vault_service.clone(),
        external_api_service.clone(),
    ));
    let spotify_library_service = Arc::new(SpotifyLibraryService::new(
        pool.clone(),
        spotify_service.clone(),
        entity_service.clone(),
    ));
    let spotify_enforcement_service = Arc::new(SpotifyEnforcementService::new(
        pool.clone(),
        spotify_service.clone(),
        spotify_library_service.clone(),
    ));
    let dnp_list_service = Arc::new(DnpListService::new(
        pool.clone(),
        entity_service.clone(),
    ));

    let app_state = AppState {
        auth_service: auth_service.clone(),
        token_vault_service,
        external_api_service,
        entity_service,
        spotify_service,
        spotify_library_service,
        spotify_enforcement_service,
        dnp_list_service,
        db_pool: pool,
    };

    // Create router with DNP list routes
    Router::new()
        .route("/api/v1/dnp/list", axum::routing::get(get_dnp_list_handler))
        .route("/api/v1/dnp/artists", axum::routing::post(add_artist_to_dnp_handler))
        .route("/api/v1/dnp/artists/:artist_id", axum::routing::delete(remove_artist_from_dnp_handler))
        .route("/api/v1/dnp/artists/:artist_id", axum::routing::put(update_dnp_entry_handler))
        .route("/api/v1/dnp/search", axum::routing::get(search_artists_for_dnp_handler))
        .route("/api/v1/dnp/import", axum::routing::post(bulk_import_dnp_handler))
        .route("/api/v1/dnp/export", axum::routing::get(export_dnp_list_handler))
        .with_state(app_state)
}

// Mock handlers for testing (simplified versions without auth middleware)
async fn get_dnp_list_handler(
    axum::extract::State(app_state): axum::extract::State<AppState>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    // For testing, use a fixed user ID
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    
    match app_state.dnp_list_service.get_user_dnp_list(user_id).await {
        Ok(dnp_list) => Ok(axum::Json(json!({
            "success": true,
            "data": dnp_list,
            "message": "DNP list retrieved successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get DNP list: {}", e)
            })),
        )),
    }
}

async fn add_artist_to_dnp_handler(
    axum::extract::State(app_state): axum::extract::State<AppState>,
    axum::Json(request): axum::Json<AddArtistToDnpRequest>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    
    match app_state.dnp_list_service.add_artist_to_dnp_list(user_id, request).await {
        Ok(entry) => Ok(axum::Json(json!({
            "success": true,
            "data": entry,
            "message": "Artist added to DNP list successfully"
        }))),
        Err(e) => {
            let status = if e.to_string().contains("already in your DNP list") {
                StatusCode::CONFLICT
            } else if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status,
                axum::Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to add artist to DNP list: {}", e)
                })),
            ))
        }
    }
}

async fn remove_artist_from_dnp_handler(
    axum::extract::State(app_state): axum::extract::State<AppState>,
    axum::extract::Path(artist_id): axum::extract::Path<Uuid>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    
    match app_state.dnp_list_service.remove_artist_from_dnp_list(user_id, artist_id).await {
        Ok(_) => Ok(axum::Json(json!({
            "success": true,
            "data": null,
            "message": "Artist removed from DNP list successfully"
        }))),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status,
                axum::Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to remove artist from DNP list: {}", e)
                })),
            ))
        }
    }
}

async fn update_dnp_entry_handler(
    axum::extract::State(app_state): axum::extract::State<AppState>,
    axum::extract::Path(artist_id): axum::extract::Path<Uuid>,
    axum::Json(request): axum::Json<UpdateDnpEntryRequest>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    
    match app_state.dnp_list_service.update_dnp_entry(user_id, artist_id, request).await {
        Ok(entry) => Ok(axum::Json(json!({
            "success": true,
            "data": entry,
            "message": "DNP entry updated successfully"
        }))),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status,
                axum::Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to update DNP entry: {}", e)
                })),
            ))
        }
    }
}

#[derive(serde::Deserialize)]
struct SearchArtistsQuery {
    q: String,
    limit: Option<usize>,
}

async fn search_artists_for_dnp_handler(
    axum::extract::State(app_state): axum::extract::State<AppState>,
    axum::extract::Query(query): axum::extract::Query<SearchArtistsQuery>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    match app_state.dnp_list_service.search_artists(&query.q, query.limit).await {
        Ok(results) => Ok(axum::Json(json!({
            "success": true,
            "data": results,
            "message": "Artist search completed successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({
                "success": false,
                "data": null,
                "message": format!("Artist search failed: {}", e)
            })),
        )),
    }
}

async fn bulk_import_dnp_handler(
    axum::extract::State(app_state): axum::extract::State<AppState>,
    axum::Json(request): axum::Json<BulkImportRequest>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    
    match app_state.dnp_list_service.bulk_import(user_id, request).await {
        Ok(result) => Ok(axum::Json(json!({
            "success": true,
            "data": result,
            "message": "Bulk import completed"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({
                "success": false,
                "data": null,
                "message": format!("Bulk import failed: {}", e)
            })),
        )),
    }
}

#[derive(serde::Deserialize)]
struct ExportQuery {
    format: Option<String>,
}

async fn export_dnp_list_handler(
    axum::extract::State(app_state): axum::extract::State<AppState>,
    axum::extract::Query(query): axum::extract::Query<ExportQuery>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    
    let format = match query.format.as_deref() {
        Some("csv") => ImportFormat::Csv,
        Some("json") | None => ImportFormat::Json,
        Some(other) => {
            return Err((
                StatusCode::BAD_REQUEST,
                axum::Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Unsupported export format: {}", other)
                })),
            ));
        }
    };

    match app_state.dnp_list_service.export_dnp_list(user_id, format).await {
        Ok(data) => Ok(axum::Json(json!({
            "success": true,
            "data": {
                "content": data,
                "format": match format {
                    ImportFormat::Csv => "csv",
                    ImportFormat::Json => "json",
                }
            },
            "message": "DNP list exported successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({
                "success": false,
                "data": null,
                "message": format!("Export failed: {}", e)
            })),
        )),
    }
}

// App state structure for testing
#[derive(Clone)]
struct AppState {
    auth_service: Arc<AuthService>,
    token_vault_service: Arc<TokenVaultService>,
    external_api_service: Arc<ExternalApiService>,
    entity_service: Arc<EntityResolutionService>,
    spotify_service: Arc<SpotifyService>,
    spotify_library_service: Arc<SpotifyLibraryService>,
    spotify_enforcement_service: Arc<SpotifyEnforcementService>,
    dnp_list_service: Arc<DnpListService>,
    db_pool: PgPool,
}

#[sqlx::test]
async fn test_get_empty_dnp_list_api(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;
    
    // Create test user
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    sqlx::query!(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)",
        user_id,
        "test@example.com",
        "hashed_password"
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/dnp/list")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["total"], 0);
    assert_eq!(json["data"]["entries"].as_array().unwrap().len(), 0);
}

#[sqlx::test]
async fn test_add_artist_to_dnp_list_api(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;
    
    // Create test user and artist
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let artist_id = Uuid::new_v4();
    
    sqlx::query!(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)",
        user_id,
        "test@example.com",
        "hashed_password"
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        r#"
        INSERT INTO artists (id, canonical_name, external_ids, metadata) 
        VALUES ($1, $2, $3, $4)
        "#,
        artist_id,
        "API Test Artist",
        serde_json::json!({"spotify": "api_test_spotify_id"}),
        serde_json::json!({"image_url": "https://example.com/api_test.jpg"})
    )
    .execute(&pool)
    .await
    .unwrap();

    let request_body = json!({
        "artist_query": "API Test Artist",
        "provider": "spotify",
        "tags": ["api", "test"],
        "note": "Added via API test"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dnp/artists")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["artist_name"], "API Test Artist");
    assert_eq!(json["data"]["tags"], json!(["api", "test"]));
    assert_eq!(json["data"]["note"], "Added via API test");
}

#[sqlx::test]
async fn test_bulk_import_json_api(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;
    
    // Create test user and artists
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let artist1_id = Uuid::new_v4();
    let artist2_id = Uuid::new_v4();
    
    sqlx::query!(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)",
        user_id,
        "test@example.com",
        "hashed_password"
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        r#"
        INSERT INTO artists (id, canonical_name, external_ids) 
        VALUES ($1, $2, $3), ($4, $5, $6)
        "#,
        artist1_id,
        "Bulk Import Artist 1",
        serde_json::json!({"spotify": "bulk1_spotify_id"}),
        artist2_id,
        "Bulk Import Artist 2",
        serde_json::json!({"spotify": "bulk2_spotify_id"})
    )
    .execute(&pool)
    .await
    .unwrap();

    let import_data = json!([
        {
            "artist_name": "Bulk Import Artist 1",
            "tags": ["bulk", "import"],
            "note": "Bulk imported"
        },
        {
            "artist_name": "Bulk Import Artist 2",
            "tags": ["bulk"],
            "note": null
        }
    ]);

    let request_body = json!({
        "format": "json",
        "data": import_data.to_string(),
        "overwrite_existing": false
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dnp/import")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["total_processed"], 2);
    assert_eq!(json["data"]["successful"], 2);
    assert_eq!(json["data"]["failed"], 0);
}

#[sqlx::test]
async fn test_export_dnp_list_json_api(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;
    
    // Create test user and artist
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let artist_id = Uuid::new_v4();
    
    sqlx::query!(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)",
        user_id,
        "test@example.com",
        "hashed_password"
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        r#"
        INSERT INTO artists (id, canonical_name, external_ids) 
        VALUES ($1, $2, $3)
        "#,
        artist_id,
        "Export Test Artist",
        serde_json::json!({"spotify": "export_test_spotify_id"})
    )
    .execute(&pool)
    .await
    .unwrap();

    // Add artist to DNP list
    sqlx::query!(
        "INSERT INTO user_artist_blocks (user_id, artist_id, tags, note) VALUES ($1, $2, $3, $4)",
        user_id,
        artist_id,
        &vec!["export".to_string(), "test".to_string()],
        Some("Export test note".to_string())
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/dnp/export?format=json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["format"], "json");
    
    let export_content: DnpListExport = serde_json::from_str(json["data"]["content"].as_str().unwrap()).unwrap();
    assert_eq!(export_content.total_entries, 1);
    assert_eq!(export_content.entries[0].artist_name, "Export Test Artist");
}