mod models;
mod services;

use models::*;
use services::*;
use serde::Serialize;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::Arc;
use tokio::runtime::Runtime;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    data: T,
    message: String,
}

fn handle_request(mut stream: TcpStream, entity_service: Arc<EntityResolutionService>) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..]);
    let request_line = request.lines().next().unwrap_or("");

    let (status_line, content) = if request_line.contains("GET /health") {
        let health = HealthResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        ("HTTP/1.1 200 OK", serde_json::to_string(&health).unwrap())
    } else if request_line.contains("GET /api/v1/users") {
        let users = vec!["user1".to_string(), "user2".to_string()];
        let response = ApiResponse {
            data: users,
            message: "Users retrieved successfully".to_string(),
        };
        ("HTTP/1.1 200 OK", serde_json::to_string(&response).unwrap())
    } else if request_line.contains("GET /api/v1/artists/search") {
        let artists = vec!["Artist 1".to_string(), "Artist 2".to_string()];
        let response = ApiResponse {
            data: artists,
            message: "Artists retrieved successfully".to_string(),
        };
        ("HTTP/1.1 200 OK", serde_json::to_string(&response).unwrap())
    } else if request_line.contains("GET /api/v1/artists/resolve") {
        // Simple test endpoint for entity resolution
        let rt = Runtime::new().unwrap();
        let result = rt.block_on(async {
            let query = ArtistSearchQuery::new("Beatles".to_string()).with_limit(5);
            entity_service.resolve_artist(&query).await
        });
        
        match result {
            Ok(results) => {
                let response = ApiResponse {
                    data: results,
                    message: "Artists resolved successfully".to_string(),
                };
                ("HTTP/1.1 200 OK", serde_json::to_string(&response).unwrap())
            }
            Err(e) => {
                let error_response = ApiResponse {
                    data: Vec::<String>::new(),
                    message: format!("Resolution failed: {}", e),
                };
                ("HTTP/1.1 500 INTERNAL SERVER ERROR", serde_json::to_string(&error_response).unwrap())
            }
        }
    } else if request_line.contains("GET /api/v1/dnp") {
        let dnp_list = vec!["Blocked Artist 1".to_string()];
        let response = ApiResponse {
            data: dnp_list,
            message: "DNP list retrieved successfully".to_string(),
        };
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

fn main() {
    println!("Starting Music Streaming Blocklist Manager Backend");
    
    // Initialize entity resolution service
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
    
    let listener = TcpListener::bind("0.0.0.0:3000").unwrap();
    println!("Server running on http://0.0.0.0:3000");
    println!("Health check: http://0.0.0.0:3000/health");
    println!("Entity resolution: http://0.0.0.0:3000/api/v1/artists/resolve");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let service = Arc::clone(&entity_service);
        thread::spawn(move || {
            handle_request(stream, service);
        });
    }
}