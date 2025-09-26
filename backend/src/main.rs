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
    routing::{get, post},
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

    let app_state = AppState {
        auth_service: auth_service.clone(),
        entity_service: entity_service.clone(),
        token_vault_service: token_vault_service.clone(),
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

