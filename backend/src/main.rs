use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use warp::Filter;

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

async fn health_check() -> Result<impl warp::Reply, warp::Rejection> {
    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    Ok(warp::reply::json(&response))
}

async fn get_users() -> Result<impl warp::Reply, warp::Rejection> {
    let users: Vec<String> = vec!["user1".to_string(), "user2".to_string()];
    let response = ApiResponse {
        data: users,
        message: "Users retrieved successfully".to_string(),
    };
    Ok(warp::reply::json(&response))
}

async fn search_artists() -> Result<impl warp::Reply, warp::Rejection> {
    let artists: Vec<String> = vec!["Artist 1".to_string(), "Artist 2".to_string()];
    let response = ApiResponse {
        data: artists,
        message: "Artists retrieved successfully".to_string(),
    };
    Ok(warp::reply::json(&response))
}

async fn get_dnp_list() -> Result<impl warp::Reply, warp::Rejection> {
    let dnp_list: Vec<String> = vec!["Blocked Artist 1".to_string()];
    let response = ApiResponse {
        data: dnp_list,
        message: "DNP list retrieved successfully".to_string(),
    };
    Ok(warp::reply::json(&response))
}

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();
    
    log::info!("Starting Music Streaming Blocklist Manager Backend");

    // CORS configuration
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]);

    // Health check route
    let health = warp::path("health")
        .and(warp::get())
        .and_then(health_check);

    // API routes
    let users = warp::path!("api" / "v1" / "users")
        .and(warp::get())
        .and_then(get_users);

    let artists = warp::path!("api" / "v1" / "artists" / "search")
        .and(warp::get())
        .and_then(search_artists);

    let dnp_get = warp::path!("api" / "v1" / "dnp")
        .and(warp::get())
        .and_then(get_dnp_list);

    // Combine all routes
    let routes = health
        .or(users)
        .or(artists)
        .or(dnp_get)
        .with(cors)
        .with(warp::log("api"));

    log::info!("Server starting on http://0.0.0.0:3000");
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], 3000))
        .await;
}