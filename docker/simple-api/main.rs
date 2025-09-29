use std::net::SocketAddr;
use axum::{
    extract::Json as ExtractJson,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio;

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
    totp_code: Option<String>,
}

#[derive(Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

#[tokio::main]
async fn main() {
    println!("🎵 Music Blocklist API starting...");
    
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/api/status", get(status))
        // Auth endpoints
        .route("/auth/login", post(auth_login))
        .route("/auth/register", post(auth_register))
        .route("/auth/profile", get(auth_profile))
        .route("/auth/logout", post(auth_logout))
        .route("/auth/refresh", post(auth_refresh));
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Server running on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Json<Value> {
    Json(json!({
        "message": "Music Streaming Blocklist Manager API",
        "version": "1.0.0",
        "status": "running"
    }))
}

async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn status() -> Json<Value> {
    Json(json!({
        "api": "online",
        "database": "connected",
        "redis": "connected",
        "services": {
            "auth": "ready",
            "spotify": "ready",
            "apple_music": "ready"
        }
    }))
}

// Mock authentication endpoints
async fn auth_login(ExtractJson(payload): ExtractJson<LoginRequest>) -> Json<Value> {
    println!("Login attempt for: {}", payload.email);
    
    // Mock successful login for any email/password
    Json(json!({
        "success": true,
        "data": {
            "access_token": "mock_access_token_12345",
            "refresh_token": "mock_refresh_token_67890"
        },
        "message": "Login successful"
    }))
}

async fn auth_register(ExtractJson(payload): ExtractJson<RegisterRequest>) -> Json<Value> {
    println!("Registration attempt for: {}", payload.email);
    
    // Mock successful registration
    Json(json!({
        "success": true,
        "message": "Account created successfully. Please check your email to verify your account."
    }))
}

async fn auth_profile() -> Json<Value> {
    // Mock user profile
    Json(json!({
        "success": true,
        "data": {
            "id": "mock_user_123",
            "email": "demo@musicblocklist.com",
            "email_verified": true,
            "totp_enabled": false,
            "created_at": "2025-09-27T00:00:00Z",
            "last_login": "2025-09-27T05:00:00Z"
        }
    }))
}

async fn auth_logout() -> Json<Value> {
    println!("User logged out");
    
    Json(json!({
        "success": true,
        "message": "Logged out successfully"
    }))
}

async fn auth_refresh(ExtractJson(payload): ExtractJson<RefreshRequest>) -> Json<Value> {
    println!("Token refresh requested");
    
    // Mock successful token refresh
    Json(json!({
        "success": true,
        "data": {
            "access_token": "new_mock_access_token_12345",
            "refresh_token": "new_mock_refresh_token_67890"
        }
    }))
}