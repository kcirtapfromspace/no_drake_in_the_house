mod models;
mod services;

use std::sync::Arc;
use chrono::Utc;
use models::*;
use services::*;

// Simple HTTP server for demo - replace with proper Axum setup in production
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use tokio::runtime::Runtime;

fn main() {
    println!("Starting Music Streaming Blocklist Manager Backend with Authentication");
    
    // Initialize services
    let rt = Runtime::new().unwrap();
    let entity_service = Arc::new(rt.block_on(async {
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
    }));

    // Initialize authentication service
    let auth_service = Arc::new(AuthService::new());
    
    let listener = TcpListener::bind("0.0.0.0:3000").unwrap();
    println!("Server running on http://0.0.0.0:3000");
    println!("Health check: http://0.0.0.0:3000/health");
    println!("Authentication endpoints:");
    println!("  POST /auth/register");
    println!("  POST /auth/login");
    println!("  GET  /auth/profile (requires auth)");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let entity_service = Arc::clone(&entity_service);
        let auth_service = Arc::clone(&auth_service);
        thread::spawn(move || {
            handle_request(stream, entity_service, auth_service);
        });
    }
}

fn handle_request(mut stream: TcpStream, entity_service: Arc<EntityResolutionService>, auth_service: Arc<AuthService>) {
    let mut buffer = [0; 4096];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..]);
    let request_line = request.lines().next().unwrap_or("");
    let headers: Vec<&str> = request.lines().collect();
    
    // Extract body for POST requests
    let body = if let Some(body_start) = request.find("\r\n\r\n") {
        &request[body_start + 4..]
    } else {
        ""
    };

    let (status_line, content) = if request_line.contains("GET /health") {
        let health = serde_json::json!({
            "success": true,
            "data": {
                "status": "healthy",
                "version": env!("CARGO_PKG_VERSION"),
                "timestamp": Utc::now().to_rfc3339()
            },
            "message": "Service is healthy"
        });
        ("HTTP/1.1 200 OK", serde_json::to_string(&health).unwrap())
    } else if request_line.contains("POST /auth/register") {
        handle_register(body, &auth_service)
    } else if request_line.contains("POST /auth/login") {
        handle_login(body, &auth_service)
    } else if request_line.contains("GET /auth/profile") {
        handle_profile(&headers, &auth_service)
    } else if request_line.contains("POST /auth/totp/setup") {
        handle_totp_setup(&headers, &auth_service)
    } else if request_line.contains("GET /api/v1/artists/resolve") {
        handle_resolve_artists(&entity_service)
    } else if request_line.contains("GET /api/v1/artists/search") {
        let artists = vec!["Artist 1".to_string(), "Artist 2".to_string()];
        let response = serde_json::json!({
            "success": true,
            "data": artists,
            "message": "Artists retrieved successfully"
        });
        ("HTTP/1.1 200 OK", serde_json::to_string(&response).unwrap())
    } else {
        ("HTTP/1.1 404 NOT FOUND", "Not Found".to_string())
    };

    let response = format!(
        "{}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        content.len(),
        content
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn handle_register(body: &str, auth_service: &AuthService) -> (&'static str, String) {
    let rt = Runtime::new().unwrap();
    
    match serde_json::from_str::<CreateUserRequest>(body) {
        Ok(request) => {
            match rt.block_on(auth_service.register_user(request)) {
                Ok(user) => {
                    let response = serde_json::json!({
                        "success": true,
                        "data": {
                            "id": user.id.to_string(),
                            "email": user.email,
                            "email_verified": user.email_verified,
                            "totp_enabled": user.totp_enabled,
                            "created_at": user.created_at.to_rfc3339()
                        },
                        "message": "User registered successfully"
                    });
                    ("HTTP/1.1 201 CREATED", serde_json::to_string(&response).unwrap())
                }
                Err(e) => {
                    let error_response = serde_json::json!({
                        "success": false,
                        "data": null,
                        "message": format!("Registration failed: {}", e)
                    });
                    ("HTTP/1.1 400 BAD REQUEST", serde_json::to_string(&error_response).unwrap())
                }
            }
        }
        Err(_) => {
            let error_response = serde_json::json!({
                "success": false,
                "data": null,
                "message": "Invalid request body"
            });
            ("HTTP/1.1 400 BAD REQUEST", serde_json::to_string(&error_response).unwrap())
        }
    }
}

fn handle_login(body: &str, auth_service: &AuthService) -> (&'static str, String) {
    let rt = Runtime::new().unwrap();
    
    match serde_json::from_str::<LoginRequest>(body) {
        Ok(request) => {
            match rt.block_on(auth_service.login_user(request)) {
                Ok(token_pair) => {
                    let response = serde_json::json!({
                        "success": true,
                        "data": token_pair,
                        "message": "Login successful"
                    });
                    ("HTTP/1.1 200 OK", serde_json::to_string(&response).unwrap())
                }
                Err(e) => {
                    let error_response = serde_json::json!({
                        "success": false,
                        "data": null,
                        "message": format!("Login failed: {}", e)
                    });
                    ("HTTP/1.1 401 UNAUTHORIZED", serde_json::to_string(&error_response).unwrap())
                }
            }
        }
        Err(_) => {
            let error_response = serde_json::json!({
                "success": false,
                "data": null,
                "message": "Invalid request body"
            });
            ("HTTP/1.1 400 BAD REQUEST", serde_json::to_string(&error_response).unwrap())
        }
    }
}

fn handle_profile(headers: &[&str], auth_service: &AuthService) -> (&'static str, String) {
    // Extract Authorization header
    let auth_header = headers.iter()
        .find(|h| h.to_lowercase().starts_with("authorization:"))
        .and_then(|h| h.split(':').nth(1))
        .map(|h| h.trim());

    if let Some(auth_value) = auth_header {
        if auth_value.starts_with("Bearer ") {
            let token = &auth_value[7..];
            
            match auth_service.verify_token(token) {
                Ok(claims) => {
                    if let Ok(user_id) = uuid::Uuid::parse_str(&claims.sub) {
                        let rt = Runtime::new().unwrap();
                        match rt.block_on(auth_service.get_user(user_id)) {
                            Ok(user) => {
                                let response = serde_json::json!({
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
                                });
                                return ("HTTP/1.1 200 OK", serde_json::to_string(&response).unwrap());
                            }
                            Err(_) => {}
                        }
                    }
                }
                Err(_) => {}
            }
        }
    }

    let error_response = serde_json::json!({
        "success": false,
        "data": null,
        "message": "Unauthorized"
    });
    ("HTTP/1.1 401 UNAUTHORIZED", serde_json::to_string(&error_response).unwrap())
}

fn handle_totp_setup(headers: &[&str], auth_service: &AuthService) -> (&'static str, String) {
    // Extract Authorization header
    let auth_header = headers.iter()
        .find(|h| h.to_lowercase().starts_with("authorization:"))
        .and_then(|h| h.split(':').nth(1))
        .map(|h| h.trim());

    if let Some(auth_value) = auth_header {
        if auth_value.starts_with("Bearer ") {
            let token = &auth_value[7..];
            
            match auth_service.verify_token(token) {
                Ok(claims) => {
                    if let Ok(user_id) = uuid::Uuid::parse_str(&claims.sub) {
                        let rt = Runtime::new().unwrap();
                        match rt.block_on(auth_service.setup_totp(user_id)) {
                            Ok(setup_response) => {
                                let response = serde_json::json!({
                                    "success": true,
                                    "data": setup_response,
                                    "message": "TOTP setup initiated"
                                });
                                return ("HTTP/1.1 200 OK", serde_json::to_string(&response).unwrap());
                            }
                            Err(_) => {}
                        }
                    }
                }
                Err(_) => {}
            }
        }
    }

    let error_response = serde_json::json!({
        "success": false,
        "data": null,
        "message": "Unauthorized"
    });
    ("HTTP/1.1 401 UNAUTHORIZED", serde_json::to_string(&error_response).unwrap())
}

fn handle_resolve_artists(entity_service: &EntityResolutionService) -> (&'static str, String) {
    let rt = Runtime::new().unwrap();
    let result = rt.block_on(async {
        let query = ArtistSearchQuery::new("Beatles".to_string()).with_limit(5);
        entity_service.resolve_artist(&query).await
    });
    
    match result {
        Ok(results) => {
            let response = serde_json::json!({
                "success": true,
                "data": results,
                "message": "Artists resolved successfully"
            });
            ("HTTP/1.1 200 OK", serde_json::to_string(&response).unwrap())
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "success": false,
                "data": null,
                "message": format!("Resolution failed: {}", e)
            });
            ("HTTP/1.1 500 INTERNAL SERVER ERROR", serde_json::to_string(&error_response).unwrap())
        }
    }
}