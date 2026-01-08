use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use kiro_backend::models::spotify::{ActionType, EntityType};
use kiro_backend::models::{
    ActionBatch, ActionBatchStatus, ActionItem, ActionItemStatus, AggressivenessLevel,
    BatchSummary, Connection, ConnectionStatus, EnforcementOptions, EnforcementPlan,
    ExecuteBatchRequest, PlannedAction, StreamingProvider,
};
use kiro_backend::services::{
    SpotifyEnforcementService, SpotifyLibraryService, SpotifyService, TokenVaultService,
};

#[tokio::test]
async fn test_enforcement_batch_creation() -> Result<()> {
    // Create a mock enforcement plan
    let plan = EnforcementPlan {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        provider: StreamingProvider::Spotify,
        options: EnforcementOptions {
            aggressiveness: AggressivenessLevel::Moderate,
            block_collaborations: true,
            block_featuring: true,
            block_songwriter_only: false,
            dry_run: true,
        },
        actions: vec![PlannedAction {
            id: Uuid::new_v4(),
            action_type: ActionType::RemoveLikedSong,
            entity_type: EntityType::Track,
            entity_id: "test_track_id".to_string(),
            metadata: json!({
                "track_name": "Test Track",
                "artist_name": "Test Artist"
            }),
        }],
        summary: json!({
            "total_actions": 1,
            "liked_songs_to_remove": 1
        }),
        created_at: chrono::Utc::now(),
    };

    // Test that we can create action items from planned actions
    let batch_id = Uuid::new_v4();
    let action_item = ActionItem::new(
        batch_id,
        plan.actions[0].entity_type.to_string(),
        plan.actions[0].entity_id.clone(),
        plan.actions[0].action_type.to_string(),
        Some(plan.actions[0].metadata.clone()),
    );

    assert_eq!(action_item.batch_id, batch_id);
    assert_eq!(action_item.entity_type, "track");
    assert_eq!(action_item.entity_id, "test_track_id");
    assert_eq!(action_item.action, "remove_liked_song");
    assert!(matches!(action_item.status, ActionItemStatus::Pending));

    Ok(())
}

#[tokio::test]
async fn test_batch_summary_creation() -> Result<()> {
    let mut summary = BatchSummary::default();

    assert_eq!(summary.total_actions, 0);
    assert_eq!(summary.completed_actions, 0);
    assert_eq!(summary.failed_actions, 0);
    assert_eq!(summary.execution_time_ms, 0);

    // Test updating summary
    summary.total_actions = 10;
    summary.completed_actions = 8;
    summary.failed_actions = 2;
    summary.execution_time_ms = 5000;

    assert_eq!(summary.total_actions, 10);
    assert_eq!(summary.completed_actions, 8);
    assert_eq!(summary.failed_actions, 2);

    Ok(())
}

#[tokio::test]
async fn test_action_item_state_transitions() -> Result<()> {
    let batch_id = Uuid::new_v4();
    let mut action = ActionItem::new(
        batch_id,
        "track".to_string(),
        "test_track_id".to_string(),
        "remove_liked_song".to_string(),
        Some(json!({"liked_at": "2023-01-01T00:00:00Z"})),
    );

    // Test initial state
    assert!(matches!(action.status, ActionItemStatus::Pending));
    assert!(action.can_rollback()); // Has before_state

    // Test completion
    action.mark_completed(json!({"removed_at": "2023-01-01T01:00:00Z"}));
    assert!(matches!(action.status, ActionItemStatus::Completed));
    assert!(action.after_state.is_some());

    // Test failure
    let mut failed_action = ActionItem::new(
        batch_id,
        "track".to_string(),
        "test_track_id_2".to_string(),
        "remove_liked_song".to_string(),
        None,
    );

    failed_action.mark_failed("API error".to_string());
    assert!(matches!(failed_action.status, ActionItemStatus::Failed));
    assert_eq!(failed_action.error_message, Some("API error".to_string()));
    assert!(!failed_action.can_rollback()); // No before_state

    Ok(())
}

#[tokio::test]
async fn test_execute_batch_request() -> Result<()> {
    let request = ExecuteBatchRequest {
        plan_id: Uuid::new_v4(),
        idempotency_key: Some("test_key_123".to_string()),
        execute_immediately: true,
        batch_size: Some(50),
        rate_limit_buffer_ms: Some(1000),
    };

    assert_eq!(request.idempotency_key, Some("test_key_123".to_string()));
    assert!(request.execute_immediately);
    assert_eq!(request.batch_size, Some(50));

    Ok(())
}

#[tokio::test]
async fn test_action_type_display() -> Result<()> {
    assert_eq!(ActionType::RemoveLikedSong.to_string(), "remove_liked_song");
    assert_eq!(
        ActionType::RemovePlaylistTrack.to_string(),
        "remove_playlist_track"
    );
    assert_eq!(ActionType::UnfollowArtist.to_string(), "unfollow_artist");
    assert_eq!(
        ActionType::RemoveSavedAlbum.to_string(),
        "remove_saved_album"
    );
    assert_eq!(ActionType::SkipTrack.to_string(), "skip_track");

    Ok(())
}

#[tokio::test]
async fn test_entity_type_display() -> Result<()> {
    assert_eq!(EntityType::Track.to_string(), "track");
    assert_eq!(EntityType::Artist.to_string(), "artist");
    assert_eq!(EntityType::Album.to_string(), "album");
    assert_eq!(EntityType::Playlist.to_string(), "playlist");

    Ok(())
}
