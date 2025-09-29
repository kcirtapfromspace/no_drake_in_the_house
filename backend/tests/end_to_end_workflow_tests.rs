use music_streaming_blocklist_backend::*;
use std::sync::Arc;
use uuid::Uuid;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, header, query_param};
use serde_json::json;

/// End-to-end integration tests that verify complete user workflows
/// These tests simulate real user journeys from authentication to enforcement

#[tokio::test]
async fn test_complete_user_onboarding_workflow() {
    // This test simulates a complete user journey:
    // 1. User registration
    // 2. Service connection (Spotify)
    // 3. DNP list creation
    // 4. Library scanning
    // 5. Enforcement planning
    // 6. Enforcement execution
    
    let mock_server = MockServer::start().await;
    setup_spotify_mocks(&mock_server).await;
    
    // Initialize services
    let token_vault = Arc::new(TokenVaultService::new());
    let auth_service = Arc::new(AuthService::new());
    let dnp_service = Arc::new(DnpListService::new());
    let entity_service = Arc::new(EntityResolutionService::new());
    
    let mut spotify_config = SpotifyConfig::default();
    spotify_config.api_base_url = format!("{}/v1", mock_server.uri());
    spotify_config.token_url = format!("{}/api/token", mock_server.uri());
    
    let spotify_service = Arc::new(SpotifyService::new(spotify_config, token_vault.clone()).unwrap());
    
    // Step 1: User Registration
    let registration_request = UserRegistrationRequest {
        email: "test@example.com".to_string(),
        password: "secure_password123".to_string(),
        display_name: Some("Test User".to_string()),
    };
    
    let user = auth_service.register_user(registration_request).await.unwrap();
    assert_eq!(user.email, "test@example.com");
    
    // Step 2: Service Connection (Spotify OAuth)
    let auth_url_response = spotify_service.get_auth_url().await.unwrap();
    assert!(auth_url_response.auth_url.contains("accounts.spotify.com"));
    
    let callback_request = SpotifyCallbackRequest {
        code: "test_auth_code".to_string(),
        state: auth_url_response.state,
    };
    
    let connection = spotify_service.handle_oauth_callback(user.id, callback_request).await.unwrap();
    assert_eq!(connection.provider, StreamingProvider::Spotify);
    
    // Step 3: DNP List Creation
    let drake_search = ArtistSearchQuery::new("Drake".to_string()).with_limit(5);
    let drake_results = entity_service.resolve_artist(&drake_search).await.unwrap();
    assert!(!drake_results.is_empty());
    
    let drake_artist = &drake_results[0].artist;
    
    let add_request = AddArtistToDnpRequest {
        artist_id: drake_artist.id,
        tags: vec!["test".to_string()],
        note: Some("Test blocking Drake".to_string()),
    };
    
    let dnp_entry = dnp_service.add_artist_to_dnp(user.id, add_request).await.unwrap();
    assert_eq!(dnp_entry.artist_id, drake_artist.id);
    
    // Step 4: Library Scanning
    let library_service = SpotifyLibraryService::new(spotify_service.clone());
    let library_scan = library_service.scan_user_library(&connection).await.unwrap();
    
    assert!(!library_scan.liked_songs.is_empty());
    assert!(!library_scan.playlists.is_empty());
    
    // Verify Drake content was found
    let drake_content = library_scan.liked_songs.iter()
        .any(|track| track.artists.iter().any(|artist| artist.name.contains("Drake")));
    assert!(drake_content, "Should find Drake content in library");
    
    // Step 5: Enforcement Planning
    let planning_service = EnforcementPlanningService::new(
        entity_service.clone(),
        dnp_service.clone(),
    );
    
    let planning_request = EnforcementPlanningRequest {
        user_id: user.id,
        provider: StreamingProvider::Spotify,
        options: EnforcementOptions {
            aggressiveness: AggressivenessLevel::Moderate,
            block_collaborations: true,
            block_featuring: true,
            dry_run: false,
        },
    };
    
    let enforcement_plan = planning_service.create_enforcement_plan(planning_request).await.unwrap();
    
    // Verify plan contains Drake-related actions
    assert!(!enforcement_plan.actions.is_empty());
    let drake_actions = enforcement_plan.actions.iter()
        .filter(|action| action.artist_name.contains("Drake"))
        .count();
    assert!(drake_actions > 0, "Should have actions for Drake content");
    
    // Step 6: Enforcement Execution
    let enforcement_service = SpotifyEnforcementService::new(spotify_service.clone());
    let execution_result = enforcement_service.execute_enforcement(&connection, enforcement_plan).await.unwrap();
    
    // Verify execution was successful
    assert!(execution_result.successful_actions > 0);
    assert_eq!(execution_result.failed_actions, 0);
    
    // Step 7: Verify DNP List Updated
    let updated_dnp_list = dnp_service.get_user_dnp_list(user.id).await.unwrap();
    assert_eq!(updated_dnp_list.len(), 1);
    assert_eq!(updated_dnp_list[0].artist_id, drake_artist.id);
    
    println!("Complete user onboarding workflow test passed!");
}

#[tokio::test]
async fn test_community_list_subscription_workflow() {
    // Test complete community list workflow:
    // 1. Create community list
    // 2. Subscribe to community list
    // 3. Apply community list to library
    // 4. Handle community list updates
    
    let mock_server = MockServer::start().await;
    setup_spotify_mocks(&mock_server).await;
    
    // Initialize services
    let auth_service = Arc::new(AuthService::new());
    let community_service = Arc::new(CommunityListService::new());
    let dnp_service = Arc::new(DnpListService::new());
    let entity_service = Arc::new(EntityResolutionService::new());
    
    // Create two users: curator and subscriber
    let curator = auth_service.register_user(UserRegistrationRequest {
        email: "curator@example.com".to_string(),
        password: "password123".to_string(),
        display_name: Some("Curator".to_string()),
    }).await.unwrap();
    
    let subscriber = auth_service.register_user(UserRegistrationRequest {
        email: "subscriber@example.com".to_string(),
        password: "password123".to_string(),
        display_name: Some("Subscriber".to_string()),
    }).await.unwrap();
    
    // Step 1: Create Community List
    let create_request = CreateCommunityListRequest {
        name: "Problematic Artists".to_string(),
        description: "Artists with documented issues".to_string(),
        criteria: "Artists with verified legal issues or documented harmful behavior".to_string(),
        governance_url: Some("https://example.com/governance".to_string()),
        update_cadence: "monthly".to_string(),
        visibility: CommunityListVisibility::Public,
    };
    
    let community_list = community_service.create_list(curator.id, create_request).await.unwrap();
    assert_eq!(community_list.name, "Problematic Artists");
    assert_eq!(community_list.owner_user_id, curator.id);
    
    // Add artists to community list
    let drake_search = ArtistSearchQuery::new("Drake".to_string()).with_limit(1);
    let drake_results = entity_service.resolve_artist(&drake_search).await.unwrap();
    let drake_artist = &drake_results[0].artist;
    
    let add_item_request = AddCommunityListItemRequest {
        artist_id: drake_artist.id,
        rationale_link: Some("https://example.com/evidence".to_string()),
    };
    
    community_service.add_item_to_list(curator.id, community_list.id, add_item_request).await.unwrap();
    
    // Step 2: Subscribe to Community List
    let subscription_request = SubscribeToCommunityListRequest {
        list_id: community_list.id,
        version_pinned: None, // Auto-update
        auto_update: true,
    };
    
    let subscription = community_service.subscribe_to_list(subscriber.id, subscription_request).await.unwrap();
    assert_eq!(subscription.list_id, community_list.id);
    assert_eq!(subscription.user_id, subscriber.id);
    
    // Step 3: Apply Community List to Library
    let token_vault = Arc::new(TokenVaultService::new());
    let mut spotify_config = SpotifyConfig::default();
    spotify_config.api_base_url = format!("{}/v1", mock_server.uri());
    
    let spotify_service = Arc::new(SpotifyService::new(spotify_config, token_vault.clone()).unwrap());
    let connection = create_mock_spotify_connection(subscriber.id, &token_vault).await;
    
    // Create enforcement plan that includes community list
    let planning_service = EnforcementPlanningService::new(
        entity_service.clone(),
        dnp_service.clone(),
    );
    
    let planning_request = EnforcementPlanningRequest {
        user_id: subscriber.id,
        provider: StreamingProvider::Spotify,
        options: EnforcementOptions::default(),
    };
    
    let enforcement_plan = planning_service.create_enforcement_plan(planning_request).await.unwrap();
    
    // Verify plan includes community list artists
    let community_actions = enforcement_plan.actions.iter()
        .filter(|action| action.artist_name.contains("Drake"))
        .count();
    assert!(community_actions > 0, "Should include community list artists");
    
    // Step 4: Handle Community List Updates
    let kanye_search = ArtistSearchQuery::new("Kanye West".to_string()).with_limit(1);
    let kanye_results = entity_service.resolve_artist(&kanye_search).await.unwrap();
    let kanye_artist = &kanye_results[0].artist;
    
    let add_kanye_request = AddCommunityListItemRequest {
        artist_id: kanye_artist.id,
        rationale_link: Some("https://example.com/kanye-evidence".to_string()),
    };
    
    // Curator adds new artist to community list
    community_service.add_item_to_list(curator.id, community_list.id, add_kanye_request).await.unwrap();
    
    // Verify subscriber gets notified of update
    let notifications = community_service.get_user_notifications(subscriber.id).await.unwrap();
    let update_notification = notifications.iter()
        .find(|n| n.notification_type == NotificationType::CommunityListUpdate)
        .unwrap();
    
    assert_eq!(update_notification.list_id, Some(community_list.id));
    assert!(update_notification.message.contains("Kanye West"));
    
    println!("Community list subscription workflow test passed!");
}

#[tokio::test]
async fn test_multi_platform_enforcement_workflow() {
    // Test enforcement across multiple platforms
    let spotify_mock = MockServer::start().await;
    let apple_mock = MockServer::start().await;
    
    setup_spotify_mocks(&spotify_mock).await;
    setup_apple_music_mocks(&apple_mock).await;
    
    // Initialize services
    let token_vault = Arc::new(TokenVaultService::new());
    let auth_service = Arc::new(AuthService::new());
    let dnp_service = Arc::new(DnpListService::new());
    let entity_service = Arc::new(EntityResolutionService::new());
    
    // Set up Spotify service
    let mut spotify_config = SpotifyConfig::default();
    spotify_config.api_base_url = format!("{}/v1", spotify_mock.uri());
    let spotify_service = Arc::new(SpotifyService::new(spotify_config, token_vault.clone()).unwrap());
    
    // Set up Apple Music service
    let mut apple_config = AppleMusicConfig::default();
    apple_config.api_base_url = format!("{}/v1", apple_mock.uri());
    let apple_service = Arc::new(AppleMusicService::new(apple_config, token_vault.clone()).unwrap());
    
    // Create user and connections
    let user = auth_service.register_user(UserRegistrationRequest {
        email: "multiplatform@example.com".to_string(),
        password: "password123".to_string(),
        display_name: Some("Multi Platform User".to_string()),
    }).await.unwrap();
    
    let spotify_connection = create_mock_spotify_connection(user.id, &token_vault).await;
    let apple_connection = create_mock_apple_music_connection(user.id, &token_vault).await;
    
    // Add artist to DNP list
    let drake_search = ArtistSearchQuery::new("Drake".to_string()).with_limit(1);
    let drake_results = entity_service.resolve_artist(&drake_search).await.unwrap();
    let drake_artist = &drake_results[0].artist;
    
    let add_request = AddArtistToDnpRequest {
        artist_id: drake_artist.id,
        tags: vec!["multi-platform-test".to_string()],
        note: Some("Testing multi-platform enforcement".to_string()),
    };
    
    dnp_service.add_artist_to_dnp(user.id, add_request).await.unwrap();
    
    // Create enforcement plans for both platforms
    let planning_service = EnforcementPlanningService::new(
        entity_service.clone(),
        dnp_service.clone(),
    );
    
    let spotify_plan = planning_service.create_enforcement_plan(EnforcementPlanningRequest {
        user_id: user.id,
        provider: StreamingProvider::Spotify,
        options: EnforcementOptions::default(),
    }).await.unwrap();
    
    let apple_plan = planning_service.create_enforcement_plan(EnforcementPlanningRequest {
        user_id: user.id,
        provider: StreamingProvider::AppleMusic,
        options: EnforcementOptions::default(),
    }).await.unwrap();
    
    // Execute enforcement on both platforms
    let spotify_enforcement = SpotifyEnforcementService::new(spotify_service);
    let apple_enforcement = AppleMusicEnforcementService::new(apple_service);
    
    let spotify_result = spotify_enforcement.execute_enforcement(&spotify_connection, spotify_plan).await.unwrap();
    let apple_result = apple_enforcement.execute_enforcement(&apple_connection, apple_plan).await.unwrap();
    
    // Verify both platforms were processed
    assert!(spotify_result.successful_actions > 0);
    assert!(apple_result.successful_actions > 0);
    
    // Verify platform-specific capabilities were respected
    assert!(spotify_result.action_results.iter().any(|r| r.action_type == ActionType::RemoveLikedSong));
    assert!(apple_result.action_results.iter().any(|r| r.action_type == ActionType::RemoveLibrarySong));
    
    println!("Multi-platform enforcement workflow test passed!");
}

#[tokio::test]
async fn test_error_recovery_workflow() {
    // Test system behavior when things go wrong
    let mock_server = MockServer::start().await;
    
    // Set up intermittent failures
    Mock::given(method("GET"))
        .and(path("/v1/me/tracks"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": {
                "status": 500,
                "message": "Internal server error"
            }
        })))
        .up_to_n_times(2)
        .mount(&mock_server)
        .await;
    
    // Then succeed
    Mock::given(method("GET"))
        .and(path("/v1/me/tracks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [
                {
                    "track": {
                        "id": "track1",
                        "name": "Test Song",
                        "artists": [{"id": "artist1", "name": "Drake"}]
                    }
                }
            ],
            "total": 1
        })))
        .mount(&mock_server)
        .await;
    
    let token_vault = Arc::new(TokenVaultService::new());
    let mut config = SpotifyConfig::default();
    config.api_base_url = format!("{}/v1", mock_server.uri());
    
    let spotify_service = SpotifyService::new(config, token_vault.clone()).unwrap();
    let user_id = Uuid::new_v4();
    let connection = create_mock_spotify_connection(user_id, &token_vault).await;
    
    // This should retry and eventually succeed
    let library_service = SpotifyLibraryService::new(spotify_service);
    let result = library_service.scan_user_library(&connection).await;
    
    // Should succeed after retries
    assert!(result.is_ok());
    let library_scan = result.unwrap();
    assert_eq!(library_scan.liked_songs.len(), 1);
    
    println!("Error recovery workflow test passed!");
}

#[tokio::test]
async fn test_concurrent_user_workflow() {
    // Test system behavior with multiple concurrent users
    let mock_server = MockServer::start().await;
    setup_spotify_mocks(&mock_server).await;
    
    let token_vault = Arc::new(TokenVaultService::new());
    let auth_service = Arc::new(AuthService::new());
    let dnp_service = Arc::new(DnpListService::new());
    
    let mut spotify_config = SpotifyConfig::default();
    spotify_config.api_base_url = format!("{}/v1", mock_server.uri());
    let spotify_service = Arc::new(SpotifyService::new(spotify_config, token_vault.clone()).unwrap());
    
    // Create multiple users concurrently
    let user_tasks: Vec<_> = (0..5).map(|i| {
        let auth_service = auth_service.clone();
        let dnp_service = dnp_service.clone();
        let spotify_service = spotify_service.clone();
        let token_vault = token_vault.clone();
        
        tokio::spawn(async move {
            // Register user
            let user = auth_service.register_user(UserRegistrationRequest {
                email: format!("user{}@example.com", i),
                password: "password123".to_string(),
                display_name: Some(format!("User {}", i)),
            }).await.unwrap();
            
            // Create connection
            let connection = create_mock_spotify_connection(user.id, &token_vault).await;
            
            // Add to DNP list
            let add_request = AddArtistToDnpRequest {
                artist_id: Uuid::new_v4(), // Mock artist ID
                tags: vec![format!("user-{}", i)],
                note: Some(format!("User {} test", i)),
            };
            
            dnp_service.add_artist_to_dnp(user.id, add_request).await.unwrap();
            
            // Scan library
            let library_service = SpotifyLibraryService::new(spotify_service);
            let _library_scan = library_service.scan_user_library(&connection).await.unwrap();
            
            user.id
        })
    }).collect();
    
    // Wait for all users to complete
    let user_ids: Vec<_> = futures::future::join_all(user_tasks).await
        .into_iter()
        .map(|result| result.unwrap())
        .collect();
    
    // Verify all users were processed
    assert_eq!(user_ids.len(), 5);
    
    // Verify each user has their own DNP list
    for user_id in user_ids {
        let dnp_list = dnp_service.get_user_dnp_list(user_id).await.unwrap();
        assert_eq!(dnp_list.len(), 1);
    }
    
    println!("Concurrent user workflow test passed!");
}

// Helper functions

async fn setup_spotify_mocks(mock_server: &MockServer) {
    // OAuth token endpoint
    Mock::given(method("POST"))
        .and(path("/api/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "test_token",
            "token_type": "Bearer",
            "expires_in": 3600,
            "refresh_token": "test_refresh_token"
        })))
        .mount(mock_server)
        .await;
    
    // User profile endpoint
    Mock::given(method("GET"))
        .and(path("/v1/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "test_user_123",
            "display_name": "Test User"
        })))
        .mount(mock_server)
        .await;
    
    // Liked songs endpoint
    Mock::given(method("GET"))
        .and(path("/v1/me/tracks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [
                {
                    "track": {
                        "id": "track1",
                        "name": "God's Plan",
                        "artists": [
                            {
                                "id": "3TVXtAsR1Inumwj472S9r4",
                                "name": "Drake"
                            }
                        ],
                        "album": {
                            "id": "album1",
                            "name": "Scorpion"
                        }
                    },
                    "added_at": "2023-01-01T00:00:00Z"
                }
            ],
            "total": 1
        })))
        .mount(mock_server)
        .await;
    
    // Playlists endpoint
    Mock::given(method("GET"))
        .and(path("/v1/me/playlists"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [
                {
                    "id": "playlist1",
                    "name": "My Playlist",
                    "owner": {"id": "test_user_123"},
                    "tracks": {"total": 5}
                }
            ],
            "total": 1
        })))
        .mount(mock_server)
        .await;
    
    // Followed artists endpoint
    Mock::given(method("GET"))
        .and(path("/v1/me/following"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "artists": {
                "items": [
                    {
                        "id": "3TVXtAsR1Inumwj472S9r4",
                        "name": "Drake",
                        "followers": {"total": 50000000}
                    }
                ],
                "total": 1
            }
        })))
        .mount(mock_server)
        .await;
    
    // Enforcement endpoints
    Mock::given(method("DELETE"))
        .and(path("/v1/me/tracks"))
        .respond_with(ResponseTemplate::new(200))
        .mount(mock_server)
        .await;
    
    Mock::given(method("DELETE"))
        .and(path("/v1/me/following"))
        .respond_with(ResponseTemplate::new(204))
        .mount(mock_server)
        .await;
}

async fn setup_apple_music_mocks(mock_server: &MockServer) {
    // Storefront endpoint
    Mock::given(method("GET"))
        .and(path("/v1/me/storefront"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {
                    "id": "us",
                    "type": "storefronts",
                    "attributes": {
                        "name": "United States"
                    }
                }
            ]
        })))
        .mount(mock_server)
        .await;
    
    // Library songs endpoint
    Mock::given(method("GET"))
        .and(path("/v1/me/library/songs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {
                    "id": "i.song1",
                    "type": "library-songs",
                    "attributes": {
                        "name": "God's Plan",
                        "artistName": "Drake",
                        "albumName": "Scorpion"
                    }
                }
            ]
        })))
        .mount(mock_server)
        .await;
}

async fn create_mock_spotify_connection(user_id: Uuid, token_vault: &TokenVaultService) -> Connection {
    let store_request = StoreTokenRequest {
        user_id,
        provider: StreamingProvider::Spotify,
        provider_user_id: "test_user_123".to_string(),
        access_token: "test_token".to_string(),
        refresh_token: Some("test_refresh_token".to_string()),
        scopes: vec!["user-read-private".to_string(), "user-library-read".to_string()],
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
    };
    
    token_vault.store_token(store_request).await.unwrap()
}

async fn create_mock_apple_music_connection(user_id: Uuid, token_vault: &TokenVaultService) -> Connection {
    let store_request = StoreTokenRequest {
        user_id,
        provider: StreamingProvider::AppleMusic,
        provider_user_id: "test_apple_user".to_string(),
        access_token: "test_user_token".to_string(),
        refresh_token: None,
        scopes: vec!["library-read".to_string()],
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
    };
    
    token_vault.store_token(store_request).await.unwrap()
}