use music_streaming_blocklist_backend::*;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

#[tokio::test]
async fn test_enforcement_batch_creation() {
    // Test creating an action batch
    let batch = ActionBatch::new(
        Uuid::new_v4(),
        "spotify".to_string(),
        "test_key_123".to_string(),
        false,
        json!({"test": "options"}),
    );

    assert_eq!(batch.provider, "spotify");
    assert_eq!(batch.idempotency_key, "test_key_123");
    assert!(!batch.dry_run);
    assert_eq!(batch.status, ActionBatchStatus::Pending);
}

#[tokio::test]
async fn test_action_item_lifecycle() {
    let batch_id = Uuid::new_v4();
    let mut action = ActionItem::new(
        batch_id,
        "track".to_string(),
        "spotify_track_123".to_string(),
        "remove_liked_song".to_string(),
        Some(json!({"track_name": "Test Song"})),
    );

    assert_eq!(action.batch_id, batch_id);
    assert_eq!(action.entity_type, "track");
    assert_eq!(action.entity_id, "spotify_track_123");
    assert_eq!(action.action, "remove_liked_song");
    assert_eq!(action.status, ActionItemStatus::Pending);
    assert!(action.idempotency_key.is_some());

    // Test marking as completed
    action.mark_completed(json!({"completed_at": Utc::now()}));
    assert_eq!(action.status, ActionItemStatus::Completed);
    assert!(action.after_state.is_some());

    // Test rollback capability
    assert!(action.can_rollback());

    // Test marking as failed
    let mut failed_action = ActionItem::new(
        batch_id,
        "artist".to_string(),
        "spotify_artist_456".to_string(),
        "unfollow_artist".to_string(),
        None,
    );

    failed_action.mark_failed("API error".to_string());
    assert_eq!(failed_action.status, ActionItemStatus::Failed);
    assert_eq!(failed_action.error_message, Some("API error".to_string()));
    assert!(!failed_action.can_rollback()); // No before_state
}

#[tokio::test]
async fn test_batch_summary_calculation() {
    let mut summary = BatchSummary::default();
    
    summary.total_actions = 100;
    summary.completed_actions = 85;
    summary.failed_actions = 10;
    summary.skipped_actions = 5;
    summary.execution_time_ms = 45000; // 45 seconds
    summary.api_calls_made = 25;

    assert_eq!(summary.total_actions, 100);
    assert_eq!(summary.completed_actions + summary.failed_actions + summary.skipped_actions, 100);
}

#[tokio::test]
async fn test_enforcement_plan_action_grouping() {
    let user_id = Uuid::new_v4();
    let mut plan = EnforcementPlan::new(
        user_id,
        "spotify".to_string(),
        EnforcementOptions::default(),
        vec![Uuid::new_v4()],
    );

    // Add different types of actions
    plan.add_action(PlannedAction {
        id: Uuid::new_v4(),
        action_type: ActionType::RemoveLikedSong,
        entity_type: EntityType::Track,
        entity_id: "track1".to_string(),
        entity_name: "Song 1".to_string(),
        reason: BlockReason::ExactMatch,
        confidence: 1.0,
        estimated_duration_ms: 500,
        dependencies: Vec::new(),
        metadata: json!({}),
    });

    plan.add_action(PlannedAction {
        id: Uuid::new_v4(),
        action_type: ActionType::UnfollowArtist,
        entity_type: EntityType::Artist,
        entity_id: "artist1".to_string(),
        entity_name: "Artist 1".to_string(),
        reason: BlockReason::ExactMatch,
        confidence: 1.0,
        estimated_duration_ms: 300,
        dependencies: Vec::new(),
        metadata: json!({}),
    });

    plan.add_action(PlannedAction {
        id: Uuid::new_v4(),
        action_type: ActionType::RemoveLikedSong,
        entity_type: EntityType::Track,
        entity_id: "track2".to_string(),
        entity_name: "Song 2".to_string(),
        reason: BlockReason::Featuring,
        confidence: 0.9,
        estimated_duration_ms: 500,
        dependencies: Vec::new(),
        metadata: json!({}),
    });

    // Test action grouping
    let liked_song_actions = plan.get_actions_by_type(ActionType::RemoveLikedSong);
    let unfollow_actions = plan.get_actions_by_type(ActionType::UnfollowArtist);

    assert_eq!(liked_song_actions.len(), 2);
    assert_eq!(unfollow_actions.len(), 1);
    assert_eq!(plan.estimated_duration_seconds, 1); // (500 + 300 + 500) / 1000 = 1.3 -> 1
}

#[tokio::test]
async fn test_idempotency_key_generation() {
    let batch_id = Uuid::new_v4();
    let action1 = ActionItem::new(
        batch_id,
        "track".to_string(),
        "same_track_id".to_string(),
        "remove_liked_song".to_string(),
        None,
    );

    let action2 = ActionItem::new(
        batch_id,
        "track".to_string(),
        "same_track_id".to_string(),
        "remove_liked_song".to_string(),
        None,
    );

    // Same parameters should generate same idempotency key
    assert_eq!(action1.idempotency_key, action2.idempotency_key);

    let action3 = ActionItem::new(
        batch_id,
        "track".to_string(),
        "different_track_id".to_string(),
        "remove_liked_song".to_string(),
        None,
    );

    // Different entity_id should generate different idempotency key
    assert_ne!(action1.idempotency_key, action3.idempotency_key);
}

#[tokio::test]
async fn test_batch_error_handling() {
    let error = BatchError {
        action_id: Uuid::new_v4(),
        entity_type: "track".to_string(),
        entity_id: "track_123".to_string(),
        error_code: "RATE_LIMIT_EXCEEDED".to_string(),
        error_message: "Too many requests".to_string(),
        retry_count: 3,
        is_recoverable: true,
    };

    assert_eq!(error.error_code, "RATE_LIMIT_EXCEEDED");
    assert!(error.is_recoverable);
    assert_eq!(error.retry_count, 3);
}

#[tokio::test]
async fn test_rollback_info_structure() {
    let rollback_info = RollbackInfo {
        rollback_batch_id: Uuid::new_v4(),
        rollback_actions: vec![],
        rollback_summary: BatchSummary::default(),
        partial_rollback: false,
        rollback_errors: vec![],
    };

    assert!(!rollback_info.partial_rollback);
    assert_eq!(rollback_info.rollback_actions.len(), 0);
    assert_eq!(rollback_info.rollback_errors.len(), 0);
}

#[tokio::test]
async fn test_enforcement_options_defaults() {
    let options = EnforcementOptions::default();
    
    assert_eq!(options.aggressiveness, AggressivenessLevel::Moderate);
    assert!(options.block_collaborations);
    assert!(options.block_featuring);
    assert!(!options.block_songwriter_only);
    assert!(!options.preserve_user_playlists);
    assert!(options.dry_run);
}

#[tokio::test]
async fn test_batch_progress_calculation() {
    let batch_id = Uuid::new_v4();
    let progress = BatchProgress {
        batch_id,
        total_actions: 100,
        completed_actions: 75,
        failed_actions: 5,
        current_action: Some("track:remove_liked_song".to_string()),
        estimated_remaining_ms: 15000, // 15 seconds
        rate_limit_status: RateLimitStatus {
            requests_remaining: 50,
            reset_time: Utc::now() + chrono::Duration::seconds(3600),
            current_delay_ms: 0,
        },
    };

    assert_eq!(progress.total_actions, 100);
    assert_eq!(progress.completed_actions, 75);
    assert_eq!(progress.failed_actions, 5);
    
    // Calculate remaining actions
    let remaining = progress.total_actions - progress.completed_actions - progress.failed_actions;
    assert_eq!(remaining, 20);
    
    assert!(progress.current_action.is_some());
    assert_eq!(progress.rate_limit_status.requests_remaining, 50);
}

#[tokio::test]
async fn test_action_type_display() {
    assert_eq!(ActionType::RemoveLikedSong.to_string(), "remove_liked_song");
    assert_eq!(ActionType::RemovePlaylistTrack.to_string(), "remove_playlist_track");
    assert_eq!(ActionType::UnfollowArtist.to_string(), "unfollow_artist");
    assert_eq!(ActionType::RemoveSavedAlbum.to_string(), "remove_saved_album");
    assert_eq!(ActionType::SkipTrack.to_string(), "skip_track");
}

#[tokio::test]
async fn test_batch_status_transitions() {
    let mut batch = ActionBatch::new(
        Uuid::new_v4(),
        "spotify".to_string(),
        "test_key".to_string(),
        false,
        json!({}),
    );

    // Initial state
    assert_eq!(batch.status, ActionBatchStatus::Pending);
    assert!(batch.completed_at.is_none());

    // Mark as completed with no failures
    let summary = BatchSummary {
        total_actions: 10,
        completed_actions: 10,
        failed_actions: 0,
        skipped_actions: 0,
        execution_time_ms: 5000,
        api_calls_made: 5,
        rate_limit_delays_ms: 0,
        errors: Vec::new(),
    };

    batch.mark_completed(summary);
    assert_eq!(batch.status, ActionBatchStatus::Completed);
    assert!(batch.completed_at.is_some());

    // Test partial completion
    let mut batch2 = ActionBatch::new(
        Uuid::new_v4(),
        "spotify".to_string(),
        "test_key_2".to_string(),
        false,
        json!({}),
    );

    let partial_summary = BatchSummary {
        total_actions: 10,
        completed_actions: 7,
        failed_actions: 3,
        skipped_actions: 0,
        execution_time_ms: 5000,
        api_calls_made: 5,
        rate_limit_delays_ms: 1000,
        errors: Vec::new(),
    };

    batch2.mark_completed(partial_summary);
    assert_eq!(batch2.status, ActionBatchStatus::PartiallyCompleted);

    // Test failure
    let mut batch3 = ActionBatch::new(
        Uuid::new_v4(),
        "spotify".to_string(),
        "test_key_3".to_string(),
        false,
        json!({}),
    );

    batch3.mark_failed("Critical error occurred".to_string());
    assert_eq!(batch3.status, ActionBatchStatus::Failed);
    assert!(batch3.completed_at.is_some());
}

#[tokio::test]
async fn test_execute_batch_request_validation() {
    let request = ExecuteBatchRequest {
        plan_id: Uuid::new_v4(),
        idempotency_key: Some("custom_key_123".to_string()),
        execute_immediately: true,
        batch_size: Some(25),
        rate_limit_buffer_ms: Some(1000),
    };

    assert!(request.execute_immediately);
    assert_eq!(request.batch_size, Some(25));
    assert_eq!(request.rate_limit_buffer_ms, Some(1000));
    assert_eq!(request.idempotency_key, Some("custom_key_123".to_string()));
}

#[tokio::test]
async fn test_rollback_request_validation() {
    let batch_id = Uuid::new_v4();
    let action_ids = vec![Uuid::new_v4(), Uuid::new_v4()];
    
    let request = RollbackBatchRequest {
        batch_id,
        action_ids: Some(action_ids.clone()),
        reason: "User requested rollback".to_string(),
    };

    assert_eq!(request.batch_id, batch_id);
    assert_eq!(request.action_ids, Some(action_ids));
    assert_eq!(request.reason, "User requested rollback");

    // Test full batch rollback
    let full_rollback = RollbackBatchRequest {
        batch_id,
        action_ids: None,
        reason: "Full rollback needed".to_string(),
    };

    assert!(full_rollback.action_ids.is_none());
}

// Integration test for the complete enforcement flow
#[tokio::test]
async fn test_enforcement_flow_integration() {
    // This test would require a database connection and mock Spotify service
    // For now, we'll test the data structures and logic flow
    
    let user_id = Uuid::new_v4();
    let dnp_artist_id = Uuid::new_v4();
    
    // Create enforcement plan
    let mut plan = EnforcementPlan::new(
        user_id,
        "spotify".to_string(),
        EnforcementOptions {
            aggressiveness: AggressivenessLevel::Moderate,
            block_collaborations: true,
            block_featuring: true,
            block_songwriter_only: false,
            preserve_user_playlists: false,
            dry_run: false,
        },
        vec![dnp_artist_id],
    );

    // Add some planned actions
    plan.add_action(PlannedAction {
        id: Uuid::new_v4(),
        action_type: ActionType::RemoveLikedSong,
        entity_type: EntityType::Track,
        entity_id: "track_1".to_string(),
        entity_name: "Blocked Song 1".to_string(),
        reason: BlockReason::ExactMatch,
        confidence: 1.0,
        estimated_duration_ms: 500,
        dependencies: Vec::new(),
        metadata: json!({
            "artist_name": "Blocked Artist",
            "album_name": "Some Album"
        }),
    });

    plan.add_action(PlannedAction {
        id: Uuid::new_v4(),
        action_type: ActionType::UnfollowArtist,
        entity_type: EntityType::Artist,
        entity_id: dnp_artist_id.to_string(),
        entity_name: "Blocked Artist".to_string(),
        reason: BlockReason::ExactMatch,
        confidence: 1.0,
        estimated_duration_ms: 300,
        dependencies: Vec::new(),
        metadata: json!({
            "follower_count": 1000000
        }),
    });

    // Verify plan structure
    assert_eq!(plan.actions.len(), 2);
    assert_eq!(plan.dnp_artists.len(), 1);
    assert!(!plan.options.dry_run);
    
    // Test action grouping
    let liked_song_actions = plan.get_actions_by_type(ActionType::RemoveLikedSong);
    let unfollow_actions = plan.get_actions_by_type(ActionType::UnfollowArtist);
    
    assert_eq!(liked_song_actions.len(), 1);
    assert_eq!(unfollow_actions.len(), 1);

    // Create batch from plan
    let batch = ActionBatch::new(
        plan.user_id,
        plan.provider.clone(),
        plan.idempotency_key.clone(),
        plan.options.dry_run,
        serde_json::to_value(&plan.options).unwrap(),
    );

    assert_eq!(batch.user_id, user_id);
    assert_eq!(batch.provider, "spotify");
    assert!(!batch.dry_run);

    // Create action items from planned actions
    let mut action_items = Vec::new();
    for planned_action in &plan.actions {
        let action_item = ActionItem::new(
            batch.id,
            planned_action.entity_type.to_string(),
            planned_action.entity_id.clone(),
            planned_action.action_type.to_string(),
            Some(planned_action.metadata.clone()),
        );
        action_items.push(action_item);
    }

    assert_eq!(action_items.len(), 2);
    
    // Simulate execution results
    let mut completed_actions = Vec::new();
    let mut failed_actions = Vec::new();
    
    for mut action in action_items {
        // Simulate successful execution
        action.mark_completed(json!({
            "executed_at": Utc::now(),
            "api_response": "success"
        }));
        completed_actions.push(action);
    }

    // Create execution result
    let result = BatchExecutionResult {
        batch_id: batch.id,
        status: ActionBatchStatus::Completed,
        summary: BatchSummary {
            total_actions: 2,
            completed_actions: 2,
            failed_actions: 0,
            skipped_actions: 0,
            execution_time_ms: 1500,
            api_calls_made: 2,
            rate_limit_delays_ms: 0,
            errors: Vec::new(),
        },
        completed_actions,
        failed_actions,
        rollback_info: None,
    };

    assert_eq!(result.completed_actions.len(), 2);
    assert_eq!(result.failed_actions.len(), 0);
    assert_eq!(result.summary.completed_actions, 2);
    assert!(result.rollback_info.is_none());
}