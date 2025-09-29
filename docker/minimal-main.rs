use axum::{
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde_json::{json, Value};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Music Blocklist Manager API...");

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/ready", get(health));

    // Run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🌐 Binding to {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("✅ Server listening on {}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn root() -> Json<Value> {
    Json(json!({
        "message": "Music Blocklist Manager API",
        "version": "0.1.0",
        "status": "running"
    }))
}

async fn health() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}