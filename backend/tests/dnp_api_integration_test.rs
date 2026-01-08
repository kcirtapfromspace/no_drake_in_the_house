use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use music_streaming_blocklist_backend::{
    create_router, AppState, AuditLoggingService, AuthService, DnpListService, RateLimitService,
    UserService,
};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

async fn create_test_app(pool: PgPool) -> axum::Router {
    let redis_url = "redis://localhost:6379";

    let auth_service = Arc::new(AuthService::new(pool.clone()));
    let rate_limiter = Arc::new(RateLimitService::new(redis_url).unwrap());
    let audit_logger = Arc::new(AuditLoggingService::new(pool.clone()));
    let dnp_service = Arc::new(DnpListService::new(pool.clone()));
    let user_service = Arc::new(UserService::new(pool.clone()));

    let app_state = AppState {
        db_pool: pool,
        auth_service,
        rate_limiter,
        audit_logger,
        dnp_service,
        user_service,
    };

    create_router(app_state)
}

#[sqlx::test]
async fn test_dnp_search_artists_endpoint(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;

    // Create test artists
    let artist1_id = Uuid::new_v4();
    let artist2_id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO artists (id, canonical_name, external_ids, metadata) 
        VALUES ($1, $2, $3, $4), ($5, $6, $7, $8)
        "#,
        artist1_id,
        "Test Search Artist One",
        serde_json::json!({"spotify": "test1_spotify_id"}),
        serde_json::json!({"image_url": "https://example.com/test1.jpg", "popularity": 85}),
        artist2_id,
        "Test Search Artist Two",
        serde_json::json!({"spotify": "test2_spotify_id", "apple": "test2_apple_id"}),
        serde_json::json!({"image_url": "https://example.com/test2.jpg", "popularity": 92, "verified": true})
    )
    .execute(&pool)
    .await
    .unwrap();

    // Test search endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/dnp/search?q=Test%20Search%20Artist&limit=10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let search_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(search_response["total"], 2);
    assert!(search_response["artists"].is_array());
    assert_eq!(search_response["artists"].as_array().unwrap().len(), 2);
}

#[sqlx::test]
async fn test_dnp_add_artist_endpoint(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;

    // Create test user
    let user_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)",
        user_id,
        "test@example.com",
        "hashed_password"
    )
    .execute(&pool)
    .await
    .unwrap();

    // Test add artist endpoint
    let request_body = json!({
        "user_id": user_id.to_string(),
        "artist_name": "New Test Artist",
        "tags": ["test", "api"],
        "note": "Added via API test"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/dnp/add")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let dnp_entry: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(dnp_entry["artist_name"], "New Test Artist");
    assert_eq!(dnp_entry["tags"], json!(["test", "api"]));
    assert_eq!(dnp_entry["note"], "Added via API test");
}

#[sqlx::test]
async fn test_dnp_get_list_endpoint(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;

    // Create test user and artist
    let user_id = Uuid::new_v4();
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
        "List Test Artist",
        serde_json::json!({"spotify": "list_test_spotify_id"}),
        serde_json::json!({"image_url": "https://example.com/list_test.jpg"})
    )
    .execute(&pool)
    .await
    .unwrap();

    // Add artist to DNP list
    sqlx::query!(
        "INSERT INTO user_artist_blocks (user_id, artist_id, tags, note) VALUES ($1, $2, $3, $4)",
        user_id,
        artist_id,
        &vec!["list".to_string(), "test".to_string()],
        Some("List test note".to_string())
    )
    .execute(&pool)
    .await
    .unwrap();

    // Test get list endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("/api/v1/dnp/list?user_id={}", user_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let dnp_list: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(dnp_list["total"], 1);
    assert_eq!(dnp_list["entries"][0]["artist_name"], "List Test Artist");
    assert_eq!(dnp_list["entries"][0]["tags"], json!(["list", "test"]));
}

#[sqlx::test]
async fn test_dnp_remove_artist_endpoint(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;

    // Create test user and artist
    let user_id = Uuid::new_v4();
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
        "Remove Test Artist",
        serde_json::json!({"spotify": "remove_test_spotify_id"})
    )
    .execute(&pool)
    .await
    .unwrap();

    // Add artist to DNP list first
    sqlx::query!(
        "INSERT INTO user_artist_blocks (user_id, artist_id, tags, note) VALUES ($1, $2, $3, $4)",
        user_id,
        artist_id,
        &vec!["remove".to_string(), "test".to_string()],
        Some("Remove test note".to_string())
    )
    .execute(&pool)
    .await
    .unwrap();

    // Test remove artist endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(&format!(
                    "/api/v1/dnp/remove/{}?user_id={}",
                    artist_id, user_id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["message"], "Artist removed from DNP list");

    // Verify artist is removed from database
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM user_artist_blocks WHERE user_id = $1 AND artist_id = $2",
        user_id,
        artist_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(count, Some(0));
}

#[sqlx::test]
async fn test_dnp_update_entry_endpoint(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;

    // Create test user and artist
    let user_id = Uuid::new_v4();
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
        "Update Test Artist",
        serde_json::json!({"spotify": "update_test_spotify_id"}),
        serde_json::json!({"image_url": "https://example.com/update_test.jpg"})
    )
    .execute(&pool)
    .await
    .unwrap();

    // Add artist to DNP list first
    sqlx::query!(
        "INSERT INTO user_artist_blocks (user_id, artist_id, tags, note) VALUES ($1, $2, $3, $4)",
        user_id,
        artist_id,
        &vec!["old".to_string(), "tags".to_string()],
        Some("Old note".to_string())
    )
    .execute(&pool)
    .await
    .unwrap();

    // Test update entry endpoint
    let request_body = json!({
        "user_id": user_id.to_string(),
        "tags": ["updated", "new"],
        "note": "Updated note via API"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(&format!("/api/v1/dnp/update/{}", artist_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let dnp_entry: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(dnp_entry["artist_name"], "Update Test Artist");
    assert_eq!(dnp_entry["tags"], json!(["updated", "new"]));
    assert_eq!(dnp_entry["note"], "Updated note via API");
}

#[sqlx::test]
async fn test_dnp_export_endpoint(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;

    // Create test user and artist
    let user_id = Uuid::new_v4();
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

    // Test JSON export endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&format!(
                    "/api/v1/dnp/export?user_id={}&format=json",
                    user_id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/json"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let export_data: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(export_data["total_entries"], 1);
    assert_eq!(
        export_data["entries"][0]["artist_name"],
        "Export Test Artist"
    );
}

#[sqlx::test]
async fn test_dnp_import_endpoint(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;

    // Create test user
    let user_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)",
        user_id,
        "test@example.com",
        "hashed_password"
    )
    .execute(&pool)
    .await
    .unwrap();

    // Test import endpoint
    let import_data = json!([
        {
            "artist_name": "Import Test Artist 1",
            "tags": ["imported", "test"],
            "note": "Imported via API"
        },
        {
            "artist_name": "Import Test Artist 2",
            "tags": ["imported"],
            "note": null
        }
    ]);

    let request_body = json!({
        "user_id": user_id.to_string(),
        "format": "json",
        "data": import_data.to_string(),
        "overwrite_existing": false
    });

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/dnp/import")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let import_result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(import_result["total_processed"], 2);
    assert_eq!(import_result["successful"], 2);
    assert_eq!(import_result["failed"], 0);
}
