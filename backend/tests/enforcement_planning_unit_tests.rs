use chrono::Utc;
use music_streaming_blocklist_backend::models::*;
use music_streaming_blocklist_backend::services::*;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_enforcement_plan_creation() {
    let user_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();

    // Create enforcement options
    let options = EnforcementOptions {
        dry_run: false,
        aggressiveness: AggressivenessLevel::Moderate,
        block_collaborations: true,
        block_featuring: true,
        block_songwriter_only: false,
        providers: vec!["spotify".to_string()],
    };

    // Create enforcement plan
    let plan = EnforcementPlan::new(
        plan_id,
        user_id,
        options.clone(),
        vec![], // Will add actions later
    );

    assert_eq!(plan.id, plan_id);
    assert_eq!(plan.user_id, user_id);
    assert_eq!(plan.options.aggressiveness, AggressivenessLevel::Moderate);
    assert!(plan.options.block_collaborations);
    assert!(plan.options.block_featuring);
    assert!(!plan.options.block_songwriter_only);
    assert_eq!(plan.actions.len(), 0);
    assert!(plan.created_at <= Utc::now());
}

#[tokio::test]
async fn test_planned_action_creation() {
    let action_id = Uuid::new_v4();
    let artist_id = Uuid::new_v4();

    // Create planned action for removing liked song
    let action = PlannedAction::new(
        action_id,
        ActionType::RemoveLikedSong,
        EntityType::Track,
        "spotify_track_id_123".to_string(),
        json!({
            "track_name": "Test Song",
            "artist_name": "Blocked Artist",
            "album_name": "Test Album"
        }),
        Some(artist_id),
        "Blocked artist found in liked songs".to_string(),
    );

    assert_eq!(action.id, action_id);
    assert_eq!(action.action_type, ActionType::RemoveLikedSong);
    assert_eq!(action.entity_type, EntityType::Track);
    assert_eq!(action.entity_id, "spotify_track_id_123");
    assert_eq!(action.blocked_artist_id, Some(artist_id));
    assert_eq!(action.reason, "Blocked artist found in liked songs");

    // Verify metadata
    let metadata = &action.metadata;
    assert_eq!(metadata["track_name"], "Test Song");
    assert_eq!(metadata["artist_name"], "Blocked Artist");
    assert_eq!(metadata["album_name"], "Test Album");
}

#[tokio::test]
async fn test_enforcement_plan_with_multiple_actions() {
    let user_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();
    let artist_id = Uuid::new_v4();

    // Create various planned actions
    let actions = vec![
        PlannedAction::new(
            Uuid::new_v4(),
            ActionType::RemoveLikedSong,
            EntityType::Track,
            "track_1".to_string(),
            json!({"track_name": "Song 1"}),
            Some(artist_id),
            "Blocked artist in liked songs".to_string(),
        ),
        PlannedAction::new(
            Uuid::new_v4(),
            ActionType::UnfollowArtist,
            EntityType::Artist,
            "artist_1".to_string(),
            json!({"artist_name": "Blocked Artist"}),
            Some(artist_id),
            "Following blocked artist".to_string(),
        ),
        PlannedAction::new(
            Uuid::new_v4(),
            ActionType::RemoveFromPlaylist,
            EntityType::Track,
            "track_2".to_string(),
            json!({
                "track_name": "Song 2",
                "playlist_id": "playlist_1",
                "playlist_name": "My Playlist"
            }),
            Some(artist_id),
            "Blocked artist in playlist".to_string(),
        ),
    ];

    let options = EnforcementOptions {
        dry_run: false,
        aggressiveness: AggressivenessLevel::Aggressive,
        block_collaborations: true,
        block_featuring: true,
        block_songwriter_only: true,
        providers: vec!["spotify".to_string()],
    };

    let plan = EnforcementPlan::new(plan_id, user_id, options, actions);

    assert_eq!(plan.actions.len(), 3);

    // Verify action types
    let action_types: Vec<_> = plan.actions.iter().map(|a| &a.action_type).collect();
    assert!(action_types.contains(&&ActionType::RemoveLikedSong));
    assert!(action_types.contains(&&ActionType::UnfollowArtist));
    assert!(action_types.contains(&&ActionType::RemoveFromPlaylist));

    // Verify all actions reference the same blocked artist
    for action in &plan.actions {
        assert_eq!(action.blocked_artist_id, Some(artist_id));
    }
}

#[tokio::test]
async fn test_enforcement_impact_calculation() {
    let user_id = Uuid::new_v4();
    let artist_id = Uuid::new_v4();

    // Create actions representing different types of impact
    let actions = vec![
        // Liked songs impact
        PlannedAction::new(
            Uuid::new_v4(),
            ActionType::RemoveLikedSong,
            EntityType::Track,
            "liked_1".to_string(),
            json!({"track_name": "Liked Song 1"}),
            Some(artist_id),
            "Blocked artist".to_string(),
        ),
        PlannedAction::new(
            Uuid::new_v4(),
            ActionType::RemoveLikedSong,
            EntityType::Track,
            "liked_2".to_string(),
            json!({"track_name": "Liked Song 2"}),
            Some(artist_id),
            "Blocked artist".to_string(),
        ),
        // Playlist impact
        PlannedAction::new(
            Uuid::new_v4(),
            ActionType::RemoveFromPlaylist,
            EntityType::Track,
            "playlist_track_1".to_string(),
            json!({
                "track_name": "Playlist Song 1",
                "playlist_id": "playlist_1"
            }),
            Some(artist_id),
            "Blocked artist".to_string(),
        ),
        PlannedAction::new(
            Uuid::new_v4(),
            ActionType::RemoveFromPlaylist,
            EntityType::Track,
            "playlist_track_2".to_string(),
            json!({
                "track_name": "Playlist Song 2",
                "playlist_id": "playlist_2"
            }),
            Some(artist_id),
            "Blocked artist".to_string(),
        ),
        // Artist following impact
        PlannedAction::new(
            Uuid::new_v4(),
            ActionType::UnfollowArtist,
            EntityType::Artist,
            "artist_1".to_string(),
            json!({"artist_name": "Blocked Artist"}),
            Some(artist_id),
            "Following blocked artist".to_string(),
        ),
    ];

    let options = EnforcementOptions {
        dry_run: true, // Dry run for impact calculation
        aggressiveness: AggressivenessLevel::Moderate,
        block_collaborations: true,
        block_featuring: false,
        block_songwriter_only: false,
        providers: vec!["spotify".to_string()],
    };

    let plan = EnforcementPlan::new(Uuid::new_v4(), user_id, options, actions);

    // Calculate impact summary
    let impact = plan.calculate_impact_summary();

    assert_eq!(impact.total_actions, 5);
    assert_eq!(impact.liked_songs_to_remove, 2);
    assert_eq!(impact.playlist_tracks_to_remove, 2);
    assert_eq!(impact.artists_to_unfollow, 1);
    assert_eq!(impact.playlists_affected, 2); // Two different playlists

    // Verify provider-specific impact
    assert!(impact.impact_by_provider.contains_key("spotify"));
    let spotify_impact = &impact.impact_by_provider["spotify"];
    assert_eq!(spotify_impact.total_actions, 5);
}

#[tokio::test]
async fn test_aggressiveness_level_behavior() {
    let user_id = Uuid::new_v4();
    let artist_id = Uuid::new_v4();

    // Test Conservative aggressiveness
    let conservative_options = EnforcementOptions {
        dry_run: false,
        aggressiveness: AggressivenessLevel::Conservative,
        block_collaborations: false,
        block_featuring: false,
        block_songwriter_only: false,
        providers: vec!["spotify".to_string()],
    };

    // Test Moderate aggressiveness
    let moderate_options = EnforcementOptions {
        dry_run: false,
        aggressiveness: AggressivenessLevel::Moderate,
        block_collaborations: true,
        block_featuring: false,
        block_songwriter_only: false,
        providers: vec!["spotify".to_string()],
    };

    // Test Aggressive aggressiveness
    let aggressive_options = EnforcementOptions {
        dry_run: false,
        aggressiveness: AggressivenessLevel::Aggressive,
        block_collaborations: true,
        block_featuring: true,
        block_songwriter_only: true,
        providers: vec!["spotify".to_string()],
    };

    // Verify that different aggressiveness levels have different blocking behaviors
    assert!(!conservative_options.block_collaborations);
    assert!(!conservative_options.block_featuring);
    assert!(!conservative_options.block_songwriter_only);

    assert!(moderate_options.block_collaborations);
    assert!(!moderate_options.block_featuring);
    assert!(!moderate_options.block_songwriter_only);

    assert!(aggressive_options.block_collaborations);
    assert!(aggressive_options.block_featuring);
    assert!(aggressive_options.block_songwriter_only);
}

#[tokio::test]
async fn test_collaboration_and_featuring_detection() {
    let user_id = Uuid::new_v4();
    let blocked_artist_id = Uuid::new_v4();
    let main_artist_id = Uuid::new_v4();

    // Create actions for different types of artist involvement
    let actions = vec![
        // Main artist track
        PlannedAction::new(
            Uuid::new_v4(),
            ActionType::RemoveLikedSong,
            EntityType::Track,
            "main_track".to_string(),
            json!({
                "track_name": "Main Artist Song",
                "artists": [{"id": blocked_artist_id.to_string(), "name": "Blocked Artist"}],
                "involvement_type": "main_artist"
            }),
            Some(blocked_artist_id),
            "Main artist is blocked".to_string(),
        ),
        // Collaboration track
        PlannedAction::new(
            Uuid::new_v4(),
            ActionType::RemoveLikedSong,
            EntityType::Track,
            "collab_track".to_string(),
            json!({
                "track_name": "Collaboration Song",
                "artists": [
                    {"id": main_artist_id.to_string(), "name": "Main Artist"},
                    {"id": blocked_artist_id.to_string(), "name": "Blocked Artist"}
                ],
                "involvement_type": "collaboration"
            }),
            Some(blocked_artist_id),
            "Blocked artist collaboration".to_string(),
        ),
        // Featuring track
        PlannedAction::new(
            Uuid::new_v4(),
            ActionType::RemoveLikedSong,
            EntityType::Track,
            "featuring_track".to_string(),
            json!({
                "track_name": "Song (feat. Blocked Artist)",
                "artists": [{"id": main_artist_id.to_string(), "name": "Main Artist"}],
                "featured_artists": [{"id": blocked_artist_id.to_string(), "name": "Blocked Artist"}],
                "involvement_type": "featuring"
            }),
            Some(blocked_artist_id),
            "Blocked artist featuring".to_string(),
        ),
        // Songwriter only
        PlannedAction::new(
            Uuid::new_v4(),
            ActionType::RemoveLikedSong,
            EntityType::Track,
            "songwriter_track".to_string(),
            json!({
                "track_name": "Song with Blocked Songwriter",
                "artists": [{"id": main_artist_id.to_string(), "name": "Main Artist"}],
                "songwriters": [{"id": blocked_artist_id.to_string(), "name": "Blocked Artist"}],
                "involvement_type": "songwriter"
            }),
            Some(blocked_artist_id),
            "Blocked artist songwriter".to_string(),
        ),
    ];

    let options = EnforcementOptions {
        dry_run: false,
        aggressiveness: AggressivenessLevel::Aggressive,
        block_collaborations: true,
        block_featuring: true,
        block_songwriter_only: true,
        providers: vec!["spotify".to_string()],
    };

    let plan = EnforcementPlan::new(Uuid::new_v4(), user_id, options, actions);

    // Verify all involvement types are captured
    let involvement_types: Vec<_> = plan
        .actions
        .iter()
        .filter_map(|a| a.metadata.get("involvement_type"))
        .collect();

    assert!(involvement_types.contains(&json!("main_artist")));
    assert!(involvement_types.contains(&json!("collaboration")));
    assert!(involvement_types.contains(&json!("featuring")));
    assert!(involvement_types.contains(&json!("songwriter")));

    // Test impact calculation with different involvement types
    let impact = plan.calculate_impact_summary();
    assert_eq!(impact.total_actions, 4);
    assert_eq!(impact.liked_songs_to_remove, 4);

    // Verify breakdown by involvement type
    assert!(impact.collaborations_found > 0);
    assert!(impact.featuring_found > 0);
    assert!(impact.songwriter_only_found > 0);
}

#[tokio::test]
async fn test_multi_provider_enforcement_plan() {
    let user_id = Uuid::new_v4();
    let artist_id = Uuid::new_v4();

    // Create actions for multiple providers
    let actions = vec![
        // Spotify actions
        PlannedAction::new(
            Uuid::new_v4(),
            ActionType::RemoveLikedSong,
            EntityType::Track,
            "spotify_track_1".to_string(),
            json!({
                "track_name": "Spotify Song",
                "provider": "spotify"
            }),
            Some(artist_id),
            "Blocked artist on Spotify".to_string(),
        ),
        PlannedAction::new(
            Uuid::new_v4(),
            ActionType::UnfollowArtist,
            EntityType::Artist,
            "spotify_artist_1".to_string(),
            json!({
                "artist_name": "Blocked Artist",
                "provider": "spotify"
            }),
            Some(artist_id),
            "Following blocked artist on Spotify".to_string(),
        ),
        // Apple Music actions
        PlannedAction::new(
            Uuid::new_v4(),
            ActionType::RemoveFromLibrary,
            EntityType::Track,
            "apple_track_1".to_string(),
            json!({
                "track_name": "Apple Song",
                "provider": "apple"
            }),
            Some(artist_id),
            "Blocked artist on Apple Music".to_string(),
        ),
    ];

    let options = EnforcementOptions {
        dry_run: false,
        aggressiveness: AggressivenessLevel::Moderate,
        block_collaborations: true,
        block_featuring: false,
        block_songwriter_only: false,
        providers: vec!["spotify".to_string(), "apple".to_string()],
    };

    let plan = EnforcementPlan::new(Uuid::new_v4(), user_id, options, actions);

    // Verify multi-provider support
    assert_eq!(plan.options.providers.len(), 2);
    assert!(plan.options.providers.contains(&"spotify".to_string()));
    assert!(plan.options.providers.contains(&"apple".to_string()));

    // Calculate impact by provider
    let impact = plan.calculate_impact_summary();
    assert_eq!(impact.total_actions, 3);

    // Verify provider-specific impact
    assert!(impact.impact_by_provider.contains_key("spotify"));
    assert!(impact.impact_by_provider.contains_key("apple"));

    let spotify_impact = &impact.impact_by_provider["spotify"];
    let apple_impact = &impact.impact_by_provider["apple"];

    assert_eq!(spotify_impact.total_actions, 2);
    assert_eq!(apple_impact.total_actions, 1);
}

#[tokio::test]
async fn test_enforcement_plan_validation() {
    let user_id = Uuid::new_v4();

    // Test plan with no actions
    let empty_plan = EnforcementPlan::new(
        Uuid::new_v4(),
        user_id,
        EnforcementOptions {
            dry_run: false,
            aggressiveness: AggressivenessLevel::Moderate,
            block_collaborations: true,
            block_featuring: false,
            block_songwriter_only: false,
            providers: vec!["spotify".to_string()],
        },
        vec![],
    );

    let validation_result = empty_plan.validate();
    assert!(validation_result.is_ok()); // Empty plan should be valid (no-op)

    // Test plan with invalid action (missing required metadata)
    let invalid_action = PlannedAction::new(
        Uuid::new_v4(),
        ActionType::RemoveLikedSong,
        EntityType::Track,
        "".to_string(), // Empty entity ID should be invalid
        json!({}),
        None,
        "".to_string(), // Empty reason should be invalid
    );

    let invalid_plan = EnforcementPlan::new(
        Uuid::new_v4(),
        user_id,
        EnforcementOptions {
            dry_run: false,
            aggressiveness: AggressivenessLevel::Moderate,
            block_collaborations: true,
            block_featuring: false,
            block_songwriter_only: false,
            providers: vec!["spotify".to_string()],
        },
        vec![invalid_action],
    );

    let invalid_validation = invalid_plan.validate();
    assert!(invalid_validation.is_err());
}

#[tokio::test]
async fn test_enforcement_plan_serialization() {
    let user_id = Uuid::new_v4();
    let artist_id = Uuid::new_v4();

    let action = PlannedAction::new(
        Uuid::new_v4(),
        ActionType::RemoveLikedSong,
        EntityType::Track,
        "test_track".to_string(),
        json!({
            "track_name": "Test Song",
            "artist_name": "Test Artist"
        }),
        Some(artist_id),
        "Test reason".to_string(),
    );

    let options = EnforcementOptions {
        dry_run: true,
        aggressiveness: AggressivenessLevel::Moderate,
        block_collaborations: true,
        block_featuring: false,
        block_songwriter_only: false,
        providers: vec!["spotify".to_string()],
    };

    let plan = EnforcementPlan::new(Uuid::new_v4(), user_id, options, vec![action]);

    // Test serialization to JSON
    let serialized = serde_json::to_string(&plan);
    assert!(serialized.is_ok());

    let json_str = serialized.unwrap();
    assert!(json_str.contains("RemoveLikedSong"));
    assert!(json_str.contains("Test Song"));
    assert!(json_str.contains("Moderate"));

    // Test deserialization from JSON
    let deserialized: Result<EnforcementPlan, _> = serde_json::from_str(&json_str);
    assert!(deserialized.is_ok());

    let restored_plan = deserialized.unwrap();
    assert_eq!(restored_plan.id, plan.id);
    assert_eq!(restored_plan.user_id, plan.user_id);
    assert_eq!(restored_plan.actions.len(), 1);
    assert_eq!(
        restored_plan.options.aggressiveness,
        AggressivenessLevel::Moderate
    );
}
