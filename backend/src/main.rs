mod models;
mod services;
mod middleware;

use std::sync::Arc;
use chrono::Utc;
use models::*;
use services::*;
use middleware::*;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::{from_fn, from_fn_with_state},
    response::Json,
    routing::{delete, get, post},
    Extension,
    Router,
};
use serde_json::json;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// Application state
#[derive(Clone)]
struct AppState {
    auth_service: Arc<AuthService>,
    entity_service: Arc<EntityResolutionService>,
    token_vault_service: Arc<TokenVaultService>,
    spotify_service: Arc<SpotifyService>,
    spotify_library_service: Arc<SpotifyLibraryService>,
    spotify_enforcement_service: Arc<SpotifyEnforcementService>,
    db_pool: sqlx::PgPool,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "music_streaming_blocklist_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    println!("Starting Music Streaming Blocklist Manager Backend with Authentication");
    
    // Initialize services
    let entity_service = Arc::new({
        let service = EntityResolutionService::new().with_confidence_threshold(0.7);
        
        // Add some test artists
        let artist1 = Artist::with_external_ids(
            "The Beatles".to_string(),
            ExternalIds::new().with_spotify("4V8Sr092TqfHkfAA5fXXqG".to_string()),
        );
        
        let mut artist2 = Artist::new("Drake".to_string());
        artist2.external_ids.spotify = Some("3TVXtAsR1Inumwj472S9r4".to_string());
        artist2.add_alias(ArtistAlias::new("Aubrey Graham".to_string(), "real_name".to_string(), 0.9));
        
        service.add_artist(artist1).await.unwrap();
        service.add_artist(artist2).await.unwrap();
        
        service
    });

    // Initialize authentication service
    let auth_service = Arc::new(AuthService::new());
    
    // Initialize token vault service
    let token_vault_service = Arc::new(TokenVaultService::new());

    // Initialize Spotify service
    let spotify_config = SpotifyConfig::default();
    let spotify_service = Arc::new(SpotifyService::new(spotify_config, token_vault_service.clone()).unwrap());

    // Initialize database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/music_streaming_blocklist".to_string());
    let db_pool = sqlx::PgPool::connect(&database_url).await
        .expect("Failed to connect to database");

    // Initialize Spotify library service
    let spotify_library_service = Arc::new(SpotifyLibraryService::new(spotify_service.clone()));

    // Initialize Spotify enforcement service
    let spotify_enforcement_service = Arc::new(SpotifyEnforcementService::new(
        spotify_service.clone(),
        spotify_library_service.clone(),
        db_pool.clone(),
    ));

    let app_state = AppState {
        auth_service: auth_service.clone(),
        entity_service: entity_service.clone(),
        token_vault_service: token_vault_service.clone(),
        spotify_service: spotify_service.clone(),
        spotify_library_service: spotify_library_service.clone(),
        spotify_enforcement_service: spotify_enforcement_service.clone(),
        db_pool,
    };

    // Build protected routes
    let protected_routes = Router::new()
        .route("/auth/profile", get(profile_handler))
        .route("/auth/totp/setup", post(totp_setup_handler))
        .route("/auth/totp/enable", post(totp_enable_handler))
        .route("/auth/logout", post(logout_handler))
        .route("/api/v1/artists/resolve", get(resolve_artists_handler))
        .route("/api/v1/artists/search", get(search_artists_handler))
        .route("/api/v1/connections", get(get_connections_handler))
        .route("/api/v1/connections", post(store_connection_handler))
        .route("/api/v1/spotify/auth", get(spotify_auth_handler))
        .route("/api/v1/spotify/callback", post(spotify_callback_handler))
        .route("/api/v1/spotify/connection", get(spotify_connection_handler))
        .route("/api/v1/spotify/connection", delete(spotify_disconnect_handler))
        .route("/api/v1/spotify/health", get(spotify_health_handler))
        .route("/api/v1/spotify/library/scan", post(spotify_scan_library_handler))
        .route("/api/v1/spotify/library/plan", post(spotify_create_plan_handler))
        .route("/api/v1/spotify/enforcement/execute", post(spotify_execute_enforcement_handler))
        .route("/api/v1/spotify/enforcement/progress/:batch_id", get(spotify_enforcement_progress_handler))
        .route("/api/v1/spotify/enforcement/rollback", post(spotify_rollback_enforcement_handler))
        .route_layer(from_fn_with_state(
            auth_service.clone(),
            auth_middleware,
        ));

    // Build the application router
    let app = Router::new()
        // Public routes
        .route("/health", get(health_handler))
        .route("/auth/register", post(register_handler))
        .route("/auth/login", post(login_handler))
        .route("/auth/refresh", post(refresh_token_handler))
        // Merge protected routes
        .merge(protected_routes)
        // Add global middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                )
                .layer(from_fn(auth_rate_limit_middleware)),
        )
        .with_state(app_state);

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://0.0.0.0:3000");
    println!("Health check: http://0.0.0.0:3000/health");
    println!("Authentication endpoints:");
    println!("  POST /auth/register");
    println!("  POST /auth/login");
    println!("  POST /auth/refresh");
    println!("  GET  /auth/profile (requires auth)");
    println!("  POST /auth/totp/setup (requires auth)");
    println!("  POST /auth/totp/enable (requires auth)");
    println!("  POST /auth/logout (requires auth)");
    println!("API endpoints (require auth):");
    println!("  GET  /api/v1/artists/resolve");
    println!("  GET  /api/v1/artists/search");
    println!("  GET  /api/v1/connections");
    println!("  POST /api/v1/connections");
    println!("Spotify endpoints (require auth):");
    println!("  GET    /api/v1/spotify/auth");
    println!("  POST   /api/v1/spotify/callback");
    println!("  GET    /api/v1/spotify/connection");
    println!("  DELETE /api/v1/spotify/connection");
    println!("  GET    /api/v1/spotify/health");
    println!("  POST   /api/v1/spotify/library/scan");
    println!("  POST   /api/v1/spotify/library/plan");
    println!("  POST   /api/v1/spotify/enforcement/execute");
    println!("  GET    /api/v1/spotify/enforcement/progress/:batch_id");
    println!("  POST   /api/v1/spotify/enforcement/rollback");

    axum::serve(listener, app).await.unwrap();
}

// Handler functions

async fn health_handler() -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "data": {
            "status": "healthy",
            "version": env!("CARGO_PKG_VERSION"),
            "timestamp": Utc::now().to_rfc3339()
        },
        "message": "Service is healthy"
    }))
}

async fn register_handler(
    State(app_state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.auth_service.register_user(request).await {
        Ok(user) => Ok(Json(json!({
            "success": true,
            "data": {
                "id": user.id.to_string(),
                "email": user.email,
                "email_verified": user.email_verified,
                "totp_enabled": user.totp_enabled,
                "created_at": user.created_at.to_rfc3339()
            },
            "message": "User registered successfully"
        }))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Registration failed: {}", e)
            })),
        )),
    }
}

async fn login_handler(
    State(app_state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.auth_service.login_user(request).await {
        Ok(token_pair) => Ok(Json(json!({
            "success": true,
            "data": token_pair,
            "message": "Login successful"
        }))),
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Login failed: {}", e)
            })),
        )),
    }
}

async fn refresh_token_handler(
    State(app_state): State<AppState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.auth_service.refresh_token(&request.refresh_token).await {
        Ok(token_pair) => Ok(Json(json!({
            "success": true,
            "data": token_pair,
            "message": "Token refreshed successfully"
        }))),
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Token refresh failed: {}", e)
            })),
        )),
    }
}

async fn profile_handler(
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {

    Ok(Json(json!({
        "success": true,
        "data": {
            "id": user.id.to_string(),
            "email": user.email,
            "email_verified": user.email_verified,
            "totp_enabled": user.totp_enabled,
            "created_at": user.created_at.to_rfc3339(),
            "last_login": user.last_login.map(|dt| dt.to_rfc3339())
        },
        "message": "Profile retrieved successfully"
    })))
}

async fn totp_setup_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {

    match app_state.auth_service.setup_totp(user.id).await {
        Ok(setup_response) => Ok(Json(json!({
            "success": true,
            "data": setup_response,
            "message": "TOTP setup initiated"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("TOTP setup failed: {}", e)
            })),
        )),
    }
}

async fn totp_enable_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    Json(totp_request): Json<TotpSetupRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {

    match app_state.auth_service.enable_totp(user.id, &totp_request.totp_code).await {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "data": null,
            "message": "TOTP enabled successfully"
        }))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("TOTP enable failed: {}", e)
            })),
        )),
    }
}

async fn logout_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {

    match app_state.auth_service.revoke_all_sessions(user.id).await {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "data": null,
            "message": "Logged out successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Logout failed: {}", e)
            })),
        )),
    }
}

async fn resolve_artists_handler(
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let query = ArtistSearchQuery::new("Beatles".to_string()).with_limit(5);
    
    match app_state.entity_service.resolve_artist(&query).await {
        Ok(results) => Ok(Json(json!({
            "success": true,
            "data": results,
            "message": "Artists resolved successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Resolution failed: {}", e)
            })),
        )),
    }
}

async fn search_artists_handler() -> Json<serde_json::Value> {
    let artists = vec!["Artist 1".to_string(), "Artist 2".to_string()];
    Json(json!({
        "success": true,
        "data": artists,
        "message": "Artists retrieved successfully"
    }))
}

async fn get_connections_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {

    let connections = app_state.token_vault_service.get_user_connections(user.id).await;
    
    Ok(Json(json!({
        "success": true,
        "data": connections,
        "message": "Connections retrieved successfully"
    })))
}

async fn store_connection_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    Json(store_request): Json<StoreTokenRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {

    // Ensure the request is for the authenticated user
    if store_request.user_id != user.id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "success": false,
                "data": null,
                "message": "Cannot store tokens for another user"
            })),
        ));
    }

    match app_state.token_vault_service.store_token(store_request).await {
        Ok(connection) => Ok(Json(json!({
            "success": true,
            "data": connection,
            "message": "Connection stored successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to store connection: {}", e)
            })),
        )),
    }
}

// Spotify handler functions

async fn spotify_auth_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.spotify_service.get_auth_url().await {
        Ok(auth_response) => Ok(Json(json!({
            "success": true,
            "data": auth_response,
            "message": "Spotify authorization URL generated"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to generate auth URL: {}", e)
            })),
        )),
    }
}

async fn spotify_callback_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    Json(callback_request): Json<SpotifyCallbackRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.spotify_service.handle_callback(user.id, callback_request).await {
        Ok(connection) => Ok(Json(json!({
            "success": true,
            "data": {
                "connection_id": connection.id,
                "provider": connection.provider,
                "provider_user_id": connection.provider_user_id,
                "scopes": connection.scopes,
                "status": connection.status,
                "created_at": connection.created_at.to_rfc3339()
            },
            "message": "Spotify connection established successfully"
        }))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Spotify connection failed: {}", e)
            })),
        )),
    }
}

async fn spotify_connection_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.spotify_service.get_user_connection(user.id).await {
        Ok(Some(connection)) => Ok(Json(json!({
            "success": true,
            "data": {
                "connection_id": connection.id,
                "provider": connection.provider,
                "provider_user_id": connection.provider_user_id,
                "scopes": connection.scopes,
                "status": connection.status,
                "expires_at": connection.expires_at.map(|dt| dt.to_rfc3339()),
                "last_health_check": connection.last_health_check.map(|dt| dt.to_rfc3339()),
                "created_at": connection.created_at.to_rfc3339()
            },
            "message": "Spotify connection retrieved"
        }))),
        Ok(None) => Ok(Json(json!({
            "success": true,
            "data": null,
            "message": "No Spotify connection found"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get connection: {}", e)
            })),
        )),
    }
}

async fn spotify_disconnect_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.spotify_service.disconnect_user(user.id).await {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "data": null,
            "message": "Spotify connection disconnected successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to disconnect: {}", e)
            })),
        )),
    }
}

async fn spotify_health_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.spotify_service.get_user_connection(user.id).await {
        Ok(Some(connection)) => {
            match app_state.spotify_service.check_token_health(&connection).await {
                Ok(health_check) => Ok(Json(json!({
                    "success": true,
                    "data": health_check,
                    "message": "Token health check completed"
                }))),
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "success": false,
                        "data": null,
                        "message": format!("Health check failed: {}", e)
                    })),
                )),
            }
        },
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "data": null,
                "message": "No Spotify connection found"
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get connection: {}", e)
            })),
        )),
    }
}

// Spotify library analysis handlers

async fn spotify_scan_library_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Get user's Spotify connection
    match app_state.spotify_service.get_user_connection(user.id).await {
        Ok(Some(connection)) => {
            match app_state.spotify_library_service.scan_library(&connection).await {
                Ok(library) => Ok(Json(json!({
                    "success": true,
                    "data": {
                        "user_id": library.user_id,
                        "spotify_user_id": library.spotify_user_id,
                        "liked_songs_count": library.liked_songs.len(),
                        "playlists_count": library.playlists.len(),
                        "followed_artists_count": library.followed_artists.len(),
                        "saved_albums_count": library.saved_albums.len(),
                        "scanned_at": library.scanned_at.to_rfc3339(),
                        "summary": {
                            "total_tracks_in_playlists": library.playlists.iter()
                                .map(|p| p.tracks.total)
                                .sum::<u32>(),
                            "total_library_items": library.liked_songs.len() + 
                                library.playlists.len() + 
                                library.followed_artists.len() + 
                                library.saved_albums.len()
                        }
                    },
                    "message": "Library scanned successfully"
                }))),
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "success": false,
                        "data": null,
                        "message": format!("Library scan failed: {}", e)
                    })),
                )),
            }
        },
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "data": null,
                "message": "No Spotify connection found. Please connect your Spotify account first."
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get connection: {}", e)
            })),
        )),
    }
}

#[derive(serde::Deserialize)]
struct CreateEnforcementPlanRequest {
    dnp_artist_ids: Vec<uuid::Uuid>,
    options: Option<EnforcementOptions>,
}

async fn spotify_create_plan_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    Json(request): Json<CreateEnforcementPlanRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Get user's Spotify connection
    match app_state.spotify_service.get_user_connection(user.id).await {
        Ok(Some(connection)) => {
            // Get DNP artists from entity service
            let mut dnp_artists = Vec::new();
            for artist_id in request.dnp_artist_ids {
                match app_state.entity_service.get_artist_by_id(artist_id).await {
                    Ok(Some(artist)) => dnp_artists.push(artist),
                    Ok(None) => {
                        return Err((
                            StatusCode::NOT_FOUND,
                            Json(json!({
                                "success": false,
                                "data": null,
                                "message": format!("Artist with ID {} not found", artist_id)
                            })),
                        ));
                    },
                    Err(e) => {
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({
                                "success": false,
                                "data": null,
                                "message": format!("Failed to get artist {}: {}", artist_id, e)
                            })),
                        ));
                    }
                }
            }

            let options = request.options.unwrap_or_default();

            match app_state.spotify_library_service.create_enforcement_plan(&connection, dnp_artists, options).await {
                Ok(plan) => Ok(Json(json!({
                    "success": true,
                    "data": {
                        "plan_id": plan.id,
                        "user_id": plan.user_id,
                        "provider": plan.provider,
                        "options": plan.options,
                        "dnp_artists_count": plan.dnp_artists.len(),
                        "impact": plan.impact,
                        "actions_count": plan.actions.len(),
                        "estimated_duration_seconds": plan.estimated_duration_seconds,
                        "created_at": plan.created_at.to_rfc3339(),
                        "idempotency_key": plan.idempotency_key,
                        "actions_summary": {
                            "remove_liked_songs": plan.get_actions_by_type(ActionType::RemoveLikedSong).len(),
                            "remove_playlist_tracks": plan.get_actions_by_type(ActionType::RemovePlaylistTrack).len(),
                            "unfollow_artists": plan.get_actions_by_type(ActionType::UnfollowArtist).len(),
                            "remove_saved_albums": plan.get_actions_by_type(ActionType::RemoveSavedAlbum).len(),
                        }
                    },
                    "message": "Enforcement plan created successfully"
                }))),
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "success": false,
                        "data": null,
                        "message": format!("Failed to create enforcement plan: {}", e)
                    })),
                )),
            }
        },
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "data": null,
                "message": "No Spotify connection found. Please connect your Spotify account first."
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get connection: {}", e)
            })),
        )),
    }
}

// Spotify enforcement handlers

#[derive(serde::Deserialize)]
struct ExecuteEnforcementRequest {
    plan_id: uuid::Uuid,
    idempotency_key: Option<String>,
    execute_immediately: Option<bool>,
    batch_size: Option<u32>,
    rate_limit_buffer_ms: Option<u64>,
}

async fn spotify_execute_enforcement_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    Json(request): Json<ExecuteEnforcementRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Get user's Spotify connection
    match app_state.spotify_service.get_user_connection(user.id).await {
        Ok(Some(connection)) => {
            // For this implementation, we'll need to store and retrieve enforcement plans
            // For now, we'll create a mock plan based on the request
            // In a real implementation, you'd retrieve the plan from storage
            
            // Create a mock enforcement plan for demonstration
            let mock_plan = EnforcementPlan::new(
                user.id,
                "spotify".to_string(),
                EnforcementOptions::default(),
                vec![], // Empty DNP artists for now
            );

            let execute_request = ExecuteBatchRequest {
                plan_id: request.plan_id,
                idempotency_key: request.idempotency_key,
                execute_immediately: request.execute_immediately.unwrap_or(true),
                batch_size: request.batch_size,
                rate_limit_buffer_ms: request.rate_limit_buffer_ms,
            };

            match app_state.spotify_enforcement_service
                .execute_enforcement_batch(&connection, &mock_plan, execute_request).await {
                Ok(result) => Ok(Json(json!({
                    "success": true,
                    "data": {
                        "batch_id": result.batch_id,
                        "status": result.status,
                        "summary": result.summary,
                        "completed_actions_count": result.completed_actions.len(),
                        "failed_actions_count": result.failed_actions.len(),
                        "rollback_available": result.rollback_info.is_some()
                    },
                    "message": "Enforcement batch executed successfully"
                }))),
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "success": false,
                        "data": null,
                        "message": format!("Enforcement execution failed: {}", e)
                    })),
                )),
            }
        },
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "data": null,
                "message": "No Spotify connection found. Please connect your Spotify account first."
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get connection: {}", e)
            })),
        )),
    }
}

async fn spotify_enforcement_progress_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    axum::extract::Path(batch_id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match app_state.spotify_enforcement_service.get_batch_progress(&batch_id).await {
        Ok(progress) => Ok(Json(json!({
            "success": true,
            "data": progress,
            "message": "Batch progress retrieved successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get batch progress: {}", e)
            })),
        )),
    }
}

async fn spotify_rollback_enforcement_handler(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    Json(request): Json<RollbackBatchRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Get user's Spotify connection
    match app_state.spotify_service.get_user_connection(user.id).await {
        Ok(Some(connection)) => {
            match app_state.spotify_enforcement_service.rollback_batch(&connection, request).await {
                Ok(rollback_info) => Ok(Json(json!({
                    "success": true,
                    "data": {
                        "rollback_batch_id": rollback_info.rollback_batch_id,
                        "rollback_actions_count": rollback_info.rollback_actions.len(),
                        "rollback_summary": rollback_info.rollback_summary,
                        "partial_rollback": rollback_info.partial_rollback,
                        "rollback_errors_count": rollback_info.rollback_errors.len()
                    },
                    "message": "Enforcement batch rolled back successfully"
                }))),
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "success": false,
                        "data": null,
                        "message": format!("Rollback failed: {}", e)
                    })),
                )),
            }
        },
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "data": null,
                "message": "No Spotify connection found. Please connect your Spotify account first."
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to get connection: {}", e)
            })),
        )),
    }
}