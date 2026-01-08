use music_streaming_blocklist_backend::models::*;
use music_streaming_blocklist_backend::services::*;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

#[sqlx::test]
async fn test_create_community_list(pool: PgPool) {
    let entity_service = Arc::new(EntityResolutionService::new(pool.clone()));
    let community_service = CommunityListService::new(pool.clone(), entity_service);

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

    // Test creating community list
    let request = CreateCommunityListRequest {
        name: "Test Community List".to_string(),
        description: Some("A test community list".to_string()),
        criteria: "Artists who have been involved in specific controversies".to_string(),
        governance_url: Some("https://example.com/governance".to_string()),
        update_cadence: "monthly".to_string(),
        visibility: CommunityListVisibility::Public,
    };

    let result = community_service
        .create_community_list(user_id, request)
        .await;
    assert!(result.is_ok());

    let list = result.unwrap();
    assert_eq!(list.name, "Test Community List");
    assert_eq!(list.description, Some("A test community list".to_string()));
    assert_eq!(
        list.criteria,
        "Artists who have been involved in specific controversies"
    );
    assert_eq!(list.version, 1);
    assert_eq!(list.visibility, "public");
    assert_eq!(list.owner.id, user_id);
}

#[sqlx::test]
async fn test_create_community_list_with_invalid_criteria(pool: PgPool) {
    let entity_service = Arc::new(EntityResolutionService::new(pool.clone()));
    let community_service = CommunityListService::new(pool.clone(), entity_service);

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

    // Test creating community list with invalid criteria
    let request = CreateCommunityListRequest {
        name: "Invalid List".to_string(),
        description: None,
        criteria: "Artists who are accused of bad behavior".to_string(), // Contains prohibited language
        governance_url: None,
        update_cadence: "as-needed".to_string(),
        visibility: CommunityListVisibility::Private,
    };

    let result = community_service
        .create_community_list(user_id, request)
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("neutral"));
}

#[sqlx::test]
async fn test_browse_community_lists(pool: PgPool) {
    let entity_service = Arc::new(EntityResolutionService::new(pool.clone()));
    let community_service = CommunityListService::new(pool.clone(), entity_service);

    // Create test users
    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3), ($4, $5, $6)",
        user1_id,
        "user1@example.com",
        "hashed_password",
        user2_id,
        "user2@example.com",
        "hashed_password"
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create test community lists
    let list1_id = Uuid::new_v4();
    let list2_id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO community_lists (
            id, owner_user_id, name, description, criteria, 
            update_cadence, version, visibility, created_at, updated_at
        ) VALUES 
        ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW()),
        ($9, $10, $11, $12, $13, $14, $15, $16, NOW(), NOW())
        "#,
        list1_id,
        user1_id,
        "Public List 1",
        Some("First public list"),
        "Neutral criteria 1",
        "weekly",
        1,
        "public",
        list2_id,
        user2_id,
        "Public List 2",
        Some("Second public list"),
        "Neutral criteria 2",
        "monthly",
        1,
        "public"
    )
    .execute(&pool)
    .await
    .unwrap();

    // Test browsing community lists
    let query = CommunityListQuery {
        search: None,
        criteria_filter: None,
        owner_filter: None,
        sort_by: None,
        sort_order: None,
        page: None,
        per_page: None,
    };

    let result = community_service.browse_community_lists(query).await;
    assert!(result.is_ok());

    let directory = result.unwrap();
    assert_eq!(directory.lists.len(), 2);
    assert_eq!(directory.total, 2);
    assert_eq!(directory.page, 1);
    assert_eq!(directory.per_page, 20);

    // Check that email is masked
    assert!(directory.lists[0].owner_email.contains("*"));
}

#[sqlx::test]
async fn test_subscribe_to_community_list(pool: PgPool) {
    let entity_service = Arc::new(EntityResolutionService::new(pool.clone()));
    let community_service = CommunityListService::new(pool.clone(), entity_service);

    // Create test users
    let owner_id = Uuid::new_v4();
    let subscriber_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3), ($4, $5, $6)",
        owner_id,
        "owner@example.com",
        "hashed_password",
        subscriber_id,
        "subscriber@example.com",
        "hashed_password"
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create test community list
    let list_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO community_lists (
            id, owner_user_id, name, description, criteria, 
            update_cadence, version, visibility, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
        "#,
        list_id,
        owner_id,
        "Subscription Test List",
        Some("Test list for subscription"),
        "Neutral criteria for testing",
        "weekly",
        1,
        "public"
    )
    .execute(&pool)
    .await
    .unwrap();

    // Test subscribing to community list
    let request = SubscribeToCommunityListRequest {
        version_pinned: Some(1),
        auto_update: Some(false),
    };

    let result = community_service
        .subscribe_to_community_list(subscriber_id, list_id, request)
        .await;
    assert!(result.is_ok());

    let subscription = result.unwrap();
    assert_eq!(subscription.version_pinned, Some(1));
    assert_eq!(subscription.auto_update, false);

    // Test that duplicate subscription fails
    let duplicate_request = SubscribeToCommunityListRequest {
        version_pinned: None,
        auto_update: None,
    };

    let duplicate_result = community_service
        .subscribe_to_community_list(subscriber_id, list_id, duplicate_request)
        .await;
    assert!(duplicate_result.is_err());
    assert!(duplicate_result
        .unwrap_err()
        .to_string()
        .contains("already subscribed"));
}

#[sqlx::test]
async fn test_get_subscription_impact_preview(pool: PgPool) {
    let entity_service = Arc::new(EntityResolutionService::new(pool.clone()));
    let community_service = CommunityListService::new(pool.clone(), entity_service);

    // Create test users
    let owner_id = Uuid::new_v4();
    let subscriber_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3), ($4, $5, $6)",
        owner_id,
        "owner@example.com",
        "hashed_password",
        subscriber_id,
        "subscriber@example.com",
        "hashed_password"
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create test community list
    let list_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO community_lists (
            id, owner_user_id, name, description, criteria, 
            update_cadence, version, visibility, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
        "#,
        list_id,
        owner_id,
        "Impact Test List",
        Some("Test list for impact preview"),
        "Neutral criteria for testing",
        "weekly",
        1,
        "public"
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create test artists
    let artist1_id = Uuid::new_v4();
    let artist2_id = Uuid::new_v4();
    let artist3_id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO artists (id, canonical_name, external_ids, metadata) 
        VALUES 
        ($1, $2, $3, $4),
        ($5, $6, $7, $8),
        ($9, $10, $11, $12)
        "#,
        artist1_id,
        "Impact Artist 1",
        serde_json::json!({"spotify": "impact1_spotify_id"}),
        serde_json::json!({"image_url": "https://example.com/impact1.jpg"}),
        artist2_id,
        "Impact Artist 2",
        serde_json::json!({"spotify": "impact2_spotify_id"}),
        serde_json::json!({"image_url": "https://example.com/impact2.jpg"}),
        artist3_id,
        "Impact Artist 3",
        serde_json::json!({"spotify": "impact3_spotify_id"}),
        serde_json::json!({"image_url": "https://example.com/impact3.jpg"})
    )
    .execute(&pool)
    .await
    .unwrap();

    // Add artists to community list
    sqlx::query!(
        r#"
        INSERT INTO community_list_items (list_id, artist_id, added_at) 
        VALUES ($1, $2, NOW()), ($1, $3, NOW()), ($1, $4, NOW())
        "#,
        list_id,
        artist1_id,
        artist2_id,
        artist3_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Add one artist to user's existing DNP list
    sqlx::query!(
        "INSERT INTO user_artist_blocks (user_id, artist_id, tags, note) VALUES ($1, $2, $3, $4)",
        subscriber_id,
        artist1_id,
        &vec!["existing".to_string()],
        Some("Already blocked".to_string())
    )
    .execute(&pool)
    .await
    .unwrap();

    // Test getting subscription impact preview
    let result = community_service
        .get_subscription_impact_preview(subscriber_id, list_id)
        .await;
    assert!(result.is_ok());

    let impact = result.unwrap();
    assert_eq!(impact.list_name, "Impact Test List");
    assert_eq!(impact.total_artists_in_list, 3);
    assert_eq!(impact.new_artists_for_user, 2); // 2 new artists (artist2 and artist3)
    assert_eq!(impact.already_blocked_artists, 1); // 1 already blocked (artist1)
    assert_eq!(impact.sample_new_artists.len(), 2);
    assert!(impact.impact_by_provider.len() > 0);
}

#[sqlx::test]
async fn test_add_artist_to_community_list(pool: PgPool) {
    let entity_service = Arc::new(EntityResolutionService::new(pool.clone()));
    let community_service = CommunityListService::new(pool.clone(), entity_service);

    // Create test user
    let owner_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)",
        owner_id,
        "owner@example.com",
        "hashed_password"
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create test community list
    let list_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO community_lists (
            id, owner_user_id, name, description, criteria, 
            update_cadence, version, visibility, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
        "#,
        list_id,
        owner_id,
        "Artist Addition Test List",
        Some("Test list for adding artists"),
        "Neutral criteria for testing",
        "weekly",
        1,
        "public"
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create test artist
    let artist_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO artists (id, canonical_name, external_ids, metadata) 
        VALUES ($1, $2, $3, $4)
        "#,
        artist_id,
        "Community List Artist",
        serde_json::json!({"spotify": "community_artist_spotify_id"}),
        serde_json::json!({"image_url": "https://example.com/community_artist.jpg"})
    )
    .execute(&pool)
    .await
    .unwrap();

    // Test adding artist to community list
    let request = AddArtistToCommunityListRequest {
        artist_query: "Community List Artist".to_string(),
        rationale_link: Some("https://example.com/rationale".to_string()),
    };

    let result = community_service
        .add_artist_to_community_list(owner_id, list_id, request)
        .await;
    assert!(result.is_ok());

    let artist_entry = result.unwrap();
    assert_eq!(artist_entry.artist_name, "Community List Artist");
    assert_eq!(
        artist_entry.rationale_link,
        Some("https://example.com/rationale".to_string())
    );
    assert!(artist_entry.provider_badges.len() > 0);

    // Verify list version was incremented
    let updated_list = community_service
        .get_community_list_by_id(list_id, Some(owner_id))
        .await
        .unwrap();
    assert_eq!(updated_list.version, 2);
    assert_eq!(updated_list.total_artists, 1);
}

#[sqlx::test]
async fn test_unauthorized_community_list_modification(pool: PgPool) {
    let entity_service = Arc::new(EntityResolutionService::new(pool.clone()));
    let community_service = CommunityListService::new(pool.clone(), entity_service);

    // Create test users
    let owner_id = Uuid::new_v4();
    let other_user_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3), ($4, $5, $6)",
        owner_id,
        "owner@example.com",
        "hashed_password",
        other_user_id,
        "other@example.com",
        "hashed_password"
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create test community list
    let list_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO community_lists (
            id, owner_user_id, name, description, criteria, 
            update_cadence, version, visibility, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
        "#,
        list_id,
        owner_id,
        "Authorization Test List",
        Some("Test list for authorization"),
        "Neutral criteria for testing",
        "weekly",
        1,
        "public"
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
        "Unauthorized Test Artist",
        serde_json::json!({"spotify": "unauthorized_spotify_id"})
    )
    .execute(&pool)
    .await
    .unwrap();

    // Test that non-owner cannot add artist
    let request = AddArtistToCommunityListRequest {
        artist_query: "Unauthorized Test Artist".to_string(),
        rationale_link: None,
    };

    let result = community_service
        .add_artist_to_community_list(other_user_id, list_id, request)
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not authorized"));

    // Test that non-owner cannot remove artist (first add one as owner)
    let add_request = AddArtistToCommunityListRequest {
        artist_query: "Unauthorized Test Artist".to_string(),
        rationale_link: None,
    };
    community_service
        .add_artist_to_community_list(owner_id, list_id, add_request)
        .await
        .unwrap();

    let remove_result = community_service
        .remove_artist_from_community_list(other_user_id, list_id, artist_id)
        .await;
    assert!(remove_result.is_err());
    assert!(remove_result
        .unwrap_err()
        .to_string()
        .contains("not authorized"));
}

#[sqlx::test]
async fn test_get_user_subscriptions(pool: PgPool) {
    let entity_service = Arc::new(EntityResolutionService::new(pool.clone()));
    let community_service = CommunityListService::new(pool.clone(), entity_service);

    // Create test users
    let owner_id = Uuid::new_v4();
    let subscriber_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3), ($4, $5, $6)",
        owner_id,
        "owner@example.com",
        "hashed_password",
        subscriber_id,
        "subscriber@example.com",
        "hashed_password"
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create test community lists
    let list1_id = Uuid::new_v4();
    let list2_id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO community_lists (
            id, owner_user_id, name, description, criteria, 
            update_cadence, version, visibility, created_at, updated_at
        ) VALUES 
        ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW()),
        ($9, $10, $11, $12, $13, $14, $15, $16, NOW(), NOW())
        "#,
        list1_id,
        owner_id,
        "Subscription List 1",
        Some("First subscription list"),
        "Neutral criteria 1",
        "weekly",
        1,
        "public",
        list2_id,
        owner_id,
        "Subscription List 2",
        Some("Second subscription list"),
        "Neutral criteria 2",
        "monthly",
        1,
        "public"
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create subscriptions
    sqlx::query!(
        r#"
        INSERT INTO user_list_subscriptions (user_id, list_id, version_pinned, auto_update, created_at) 
        VALUES ($1, $2, $3, $4, NOW()), ($1, $5, $6, $7, NOW())
        "#,
        subscriber_id,
        list1_id,
        Some(1),
        true,
        list2_id,
        None::<i32>,
        false
    )
    .execute(&pool)
    .await
    .unwrap();

    // Test getting user subscriptions
    let result = community_service
        .get_user_subscriptions(subscriber_id)
        .await;
    assert!(result.is_ok());

    let subscriptions = result.unwrap();
    assert_eq!(subscriptions.len(), 2);

    // Check subscription details
    let sub1 = subscriptions
        .iter()
        .find(|s| s.name == "Subscription List 1")
        .unwrap();
    assert_eq!(sub1.is_subscribed, true);
    assert!(sub1.subscription_details.is_some());
    assert_eq!(
        sub1.subscription_details.as_ref().unwrap().version_pinned,
        Some(1)
    );
    assert_eq!(
        sub1.subscription_details.as_ref().unwrap().auto_update,
        true
    );

    let sub2 = subscriptions
        .iter()
        .find(|s| s.name == "Subscription List 2")
        .unwrap();
    assert_eq!(sub2.is_subscribed, true);
    assert!(sub2.subscription_details.is_some());
    assert_eq!(
        sub2.subscription_details.as_ref().unwrap().version_pinned,
        None
    );
    assert_eq!(
        sub2.subscription_details.as_ref().unwrap().auto_update,
        false
    );
}
