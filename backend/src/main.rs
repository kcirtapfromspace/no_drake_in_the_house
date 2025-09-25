use axum::{
    extract::Extension,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router, Server,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod database;
mod models;
mod handlers;
mod services;

use config::Config;
use database::{Database, DatabasePool};

#[derive(Clone)]
pub struct AppState {
    pub db: DatabasePool,
    pub config: Arc<Config>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "music_streaming_blocklist_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Arc::new(Config::from_env()?);
    
    // Initialize database
    let db = Database::new(&config.database_url).await?;
    db.migrate().await?;

    let state = AppState {
        db: db.pool(),
        config: config.clone(),
    };

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/users", post(handlers::users::create_user))
        .route("/api/v1/artists/search", get(handlers::artists::search_artists))
        .route("/api/v1/dnp", get(handlers::dnp::get_user_dnp_list))
        .route("/api/v1/dnp", post(handlers::dnp::add_artist_to_dnp))
        .layer(Extension(state))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    tracing::info!("Server listening on {}", config.server_address);
    
    Server::bind(&config.server_address.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}