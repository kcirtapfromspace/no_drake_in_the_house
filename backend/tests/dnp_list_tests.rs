use music_streaming_blocklist_backend::models::*;
use music_streaming_blocklist_backend::services::*;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

#[sqlx::test]
async fn test_add_artist_to_dnp_list(pool: PgPool) {
    let entity_service = Arc::new(EntityResolutionService::new(pool.clone()));
    let dnp_service = DnpListService::new(pool.clone(), entity_service);
    
    // Create a test user
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

    // Create a test artist
    let artist_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO artists (id, canonical_name, external_ids, metadata) 
        VALUES ($1, $2, $3, $4)
        "#,
        artist_id,
        "Test Artist",
        serde_json::json!({"spotify": "test_spotify_id"}),
        serde_json::json!({"image_url": "https://example.com/image.jpg"})
    )
    .execute(&pool)
    .await
    .unwrap();

    // Test adding artist to DNP list
    let request = AddArtistToDnpRequest {
        artist_query: "Test Artist".to_string(),
        provider: Some("spotify".to_string()),
        tags: Some(vec!["test".to_string(), "example".to_string()]),
        note: Some("Test note".to_string()),
    };

    let result = dnp_service.add_artist_to_dnp_list(user_id, request).await;
    assert!(result.is_ok());
    
    let entry = result.unwrap();
    assert_eq!(entry.artist_name, "Test Artist");
    assert_eq!(entry.tags, vec!["test", "example"]);
    assert_eq!(entry.note, Some("Test note".to_string()));
}

#[sqlx::test]
async fn test_remove_artist_from_dnp_list(pool: PgPool) {
    let entity_service = Arc::new(EntityResolutionService::new(pool.clone()));
    let dnp_service = DnpListService::new(pool.clone(), entity_service);
    
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
        "Test Artist",
        serde_json::json!({"spotify": "test_spotify_id"})
    )
    .execute(&pool)
    .await
    .unwrap();

    // Add artist to DNP list first
    sqlx::query!(
        "INSERT INTO user_artist_blocks (user_id, artist_id, tags, note) VALUES ($1, $2, $3, $4)",
        user_id,
        artist_id,
        &vec!["test".to_string()],
        Some("Test note".to_string())
    )
    .execute(&pool)
    .await
    .unwrap();

    // Test removing artist from DNP list
    let result = dnp_service.remove_artist_from_dnp_list(user_id, artist_id).await;
    assert!(result.is_ok());

    // Verify artist is removed
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
async fn test_update_dnp_entry(pool: PgPool) {
    let entity_service = Arc::new(EntityResolutionService::new(pool.clone()));
    let dnp_service = DnpListService::new(pool.clone(), entity_service);
    
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
        "Test Artist",
        serde_json::json!({"spotify": "test_spotify_id"}),
        serde_json::json!({"image_url": "https://example.com/image.jpg"})
    )
    .execute(&pool)
    .await
    .unwrap();

    // Add artist to DNP list first
    sqlx::query!(
        "INSERT INTO user_artist_blocks (user_id, artist_id, tags, note) VALUES ($1, $2, $3, $4)",
        user_id,
        artist_id,
        &vec!["old_tag".to_string()],
        Some("Old note".to_string())
    )
    .execute(&pool)
    .await
    .unwrap();

    // Test updating DNP entry
    let update_request = UpdateDnpEntryRequest {
        tags: Some(vec!["new_tag".to_string(), "updated".to_string()]),
        note: Some("Updated note".to_string()),
    };

    let result = dnp_service.update_dnp_entry(user_id, artist_id, update_request).await;
    assert!(result.is_ok());
    
    let entry = result.unwrap();
    assert_eq!(entry.tags, vec!["new_tag", "updated"]);
    assert_eq!(entry.note, Some("Updated note".to_string()));
}

#[sqlx::test]
async fn test_get_user_dnp_list(pool: PgPool) {
    let entity_service = Arc::new(EntityResolutionService::new(pool.clone()));
    let dnp_service = DnpListService::new(pool.clone(), entity_service);
    
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

    // Create test artists
    let artist1_id = Uuid::new_v4();
    let artist2_id = Uuid::new_v4();
    
    sqlx::query!(
        r#"
        INSERT INTO artists (id, canonical_name, external_ids, metadata) 
        VALUES ($1, $2, $3, $4), ($5, $6, $7, $8)
        "#,
        artist1_id,
        "Artist One",
        serde_json::json!({"spotify": "artist1_spotify_id"}),
        serde_json::json!({"image_url": "https://example.com/artist1.jpg"}),
        artist2_id,
        "Artist Two",
        serde_json::json!({"spotify": "artist2_spotify_id", "apple": "artist2_apple_id"}),
        serde_json::json!({"image_url": "https://example.com/artist2.jpg", "verified": true})
    )
    .execute(&pool)
    .await
    .unwrap();

    // Add artists to DNP list
    sqlx::query!(
        r#"
        INSERT INTO user_artist_blocks (user_id, artist_id, tags, note) 
        VALUES ($1, $2, $3, $4), ($1, $5, $6, $7)
        "#,
        user_id,
        artist1_id,
        &vec!["tag1".to_string(), "shared".to_string()],
        Some("Note for artist 1".to_string()),
        artist2_id,
        &vec!["tag2".to_string(), "shared".to_string()],
        None::<String>
    )
    .execute(&pool)
    .await
    .unwrap();

    // Test getting user's DNP list
    let result = dnp_service.get_user_dnp_list(user_id).await;
    assert!(result.is_ok());
    
    let dnp_list = result.unwrap();
    assert_eq!(dnp_list.total, 2);
    assert_eq!(dnp_list.entries.len(), 2);
    
    // Check that all unique tags are collected
    assert!(dnp_list.tags.contains(&"tag1".to_string()));
    assert!(dnp_list.tags.contains(&"tag2".to_string()));
    assert!(dnp_list.tags.contains(&"shared".to_string()));
    
    // Check provider badges
    let artist_with_spotify = dnp_list.entries.iter()
        .find(|e| e.artist_name == "Artist One")
        .unwrap();
    assert_eq!(artist_with_spotify.provider_badges.len(), 1);
    assert_eq!(artist_with_spotify.provider_badges[0].provider, "spotify");
    
    let artist_with_multiple = dnp_list.entries.iter()
        .find(|e| e.artist_name == "Artist Two")
        .unwrap();
    assert_eq!(artist_with_multiple.provider_badges.len(), 2);
}

#[sqlx::test]
async fn test_bulk_import_json(pool: PgPool) {
    let entity_service = Arc::new(EntityResolutionService::new(pool.clone()));
    let dnp_service = DnpListService::new(pool.clone(), entity_service);
    
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

    // Create test artists that will be found during import
    let artist1_id = Uuid::new_v4();
    let artist2_id = Uuid::new_v4();
    
    sqlx::query!(
        r#"
        INSERT INTO artists (id, canonical_name, external_ids) 
        VALUES ($1, $2, $3), ($4, $5, $6)
        "#,
        artist1_id,
        "Import Artist 1",
        serde_json::json!({"spotify": "import1_spotify_id"}),
        artist2_id,
        "Import Artist 2",
        serde_json::json!({"spotify": "import2_spotify_id"})
    )
    .execute(&pool)
    .await
    .unwrap();

    // Test JSON bulk import
    let import_data = serde_json::json!([
        {
            "artist_name": "Import Artist 1",
            "tags": ["imported", "test"],
            "note": "Imported from JSON"
        },
        {
            "artist_name": "Import Artist 2",
            "tags": ["imported"],
            "note": null
        },
        {
            "artist_name": "Nonexistent Artist",
            "tags": ["imported"],
            "note": "This should fail"
        }
    ]);

    let request = BulkImportRequest {
        format: ImportFormat::Json,
        data: import_data.to_string(),
        overwrite_existing: Some(false),
    };

    let result = dnp_service.bulk_import(user_id, request).await;
    assert!(result.is_ok());
    
    let bulk_result = result.unwrap();
    assert_eq!(bulk_result.total_processed, 3);
    assert_eq!(bulk_result.successful, 2);
    assert_eq!(bulk_result.failed, 1);
    assert_eq!(bulk_result.errors.len(), 1);
    assert!(bulk_result.errors[0].artist_name == "Nonexistent Artist");
}

#[sqlx::test]
async fn test_bulk_import_csv(pool: PgPool) {
    let entity_service = Arc::new(EntityResolutionService::new(pool.clone()));
    let dnp_service = DnpListService::new(pool.clone(), entity_service);
    
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

    // Create test artist
    let artist_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO artists (id, canonical_name, external_ids) 
        VALUES ($1, $2, $3)
        "#,
        artist_id,
        "CSV Artist",
        serde_json::json!({"spotify": "csv_spotify_id"})
    )
    .execute(&pool)
    .await
    .unwrap();

    // Test CSV bulk import
    let csv_data = "artist_name,tags,note\nCSV Artist,\"csv;imported\",\"Imported from CSV\"";

    let request = BulkImportRequest {
        format: ImportFormat::Csv,
        data: csv_data.to_string(),
        overwrite_existing: Some(false),
    };

    let result = dnp_service.bulk_import(user_id, request).await;
    assert!(result.is_ok());
    
    let bulk_result = result.unwrap();
    assert_eq!(bulk_result.successful, 1);
    assert_eq!(bulk_result.failed, 0);
}

#[sqlx::test]
async fn test_export_dnp_list_json(pool: PgPool) {
    let entity_service = Arc::new(EntityResolutionService::new(pool.clone()));
    let dnp_service = DnpListService::new(pool.clone(), entity_service);
    
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
        "Export Artist",
        serde_json::json!({"spotify": "export_spotify_id"})
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

    // Test JSON export
    let result = dnp_service.export_dnp_list(user_id, ImportFormat::Json).await;
    assert!(result.is_ok());
    
    let json_data = result.unwrap();
    let export: DnpListExport = serde_json::from_str(&json_data).unwrap();
    
    assert_eq!(export.total_entries, 1);
    assert_eq!(export.entries[0].artist_name, "Export Artist");
    assert_eq!(export.entries[0].tags, vec!["export", "test"]);
    assert_eq!(export.entries[0].note, Some("Export test note".to_string()));
}

#[sqlx::test]
async fn test_export_dnp_list_csv(pool: PgPool) {
    let entity_service = Arc::new(EntityResolutionService::new(pool.clone()));
    let dnp_service = DnpListService::new(pool.clone(), entity_service);
    
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
        "CSV Export Artist",
        serde_json::json!({"spotify": "csv_export_spotify_id"})
    )
    .execute(&pool)
    .await
    .unwrap();

    // Add artist to DNP list
    sqlx::query!(
        "INSERT INTO user_artist_blocks (user_id, artist_id, tags, note) VALUES ($1, $2, $3, $4)",
        user_id,
        artist_id,
        &vec!["csv".to_string(), "export".to_string()],
        Some("CSV export note".to_string())
    )
    .execute(&pool)
    .await
    .unwrap();

    // Test CSV export
    let result = dnp_service.export_dnp_list(user_id, ImportFormat::Csv).await;
    assert!(result.is_ok());
    
    let csv_data = result.unwrap();
    assert!(csv_data.contains("CSV Export Artist"));
    assert!(csv_data.contains("csv;export"));
    assert!(csv_data.contains("CSV export note"));
}

#[sqlx::test]
async fn test_search_artists(pool: PgPool) {
    let entity_service = Arc::new(EntityResolutionService::new(pool.clone()));
    let dnp_service = DnpListService::new(pool.clone(), entity_service);
    
    // Create test artists
    let artist1_id = Uuid::new_v4();
    let artist2_id = Uuid::new_v4();
    
    sqlx::query!(
        r#"
        INSERT INTO artists (id, canonical_name, external_ids, metadata) 
        VALUES ($1, $2, $3, $4), ($5, $6, $7, $8)
        "#,
        artist1_id,
        "Search Artist One",
        serde_json::json!({"spotify": "search1_spotify_id"}),
        serde_json::json!({"image_url": "https://example.com/search1.jpg", "popularity": 85}),
        artist2_id,
        "Search Artist Two",
        serde_json::json!({"spotify": "search2_spotify_id", "apple": "search2_apple_id"}),
        serde_json::json!({"image_url": "https://example.com/search2.jpg", "popularity": 92, "verified": true})
    )
    .execute(&pool)
    .await
    .unwrap();

    // Test artist search
    let result = dnp_service.search_artists("Search Artist", Some(10)).await;
    assert!(result.is_ok());
    
    let search_response = result.unwrap();
    assert!(search_response.total >= 2);
    
    // Check that results include provider badges
    let artist_with_multiple_providers = search_response.artists.iter()
        .find(|a| a.canonical_name == "Search Artist Two")
        .unwrap();
    
    assert!(artist_with_multiple_providers.provider_badges.len() >= 1);
    assert!(artist_with_multiple_providers.provider_badges.iter()
        .any(|badge| badge.provider == "spotify"));
}