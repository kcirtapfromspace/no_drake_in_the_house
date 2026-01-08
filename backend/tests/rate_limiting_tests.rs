use music_streaming_blocklist_backend::*;
use std::sync::Arc;
use tokio_test;
use uuid::Uuid;

#[tokio::test]
async fn test_rate_limiting_basic_functionality() {
    // This test requires Redis to be running
    // Skip if Redis is not available
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let rate_limiter = match RateLimitingService::new(&redis_url) {
        Ok(service) => Arc::new(service),
        Err(_) => {
            println!("Skipping rate limiting test - Redis not available");
            return;
        }
    };

    // Test basic rate limit check
    let can_proceed = rate_limiter.can_proceed("spotify").await.unwrap();
    assert!(can_proceed, "Should be able to proceed initially");

    // Test successful request recording
    let response = RateLimitResponse {
        requests_remaining: Some(99),
        reset_at: None,
        retry_after_seconds: None,
        rate_limit_hit: false,
    };

    rate_limiter
        .record_success("spotify", response)
        .await
        .unwrap();

    // Test optimal batch size calculation
    let batch_size = rate_limiter
        .get_optimal_batch_size("spotify", "remove_tracks")
        .await
        .unwrap();
    assert!(batch_size > 0, "Batch size should be greater than 0");
    assert!(batch_size <= 50, "Batch size should not exceed maximum");
}

#[tokio::test]
async fn test_circuit_breaker_functionality() {
    let mut breaker = CircuitBreakerState::default();

    // Initially closed
    assert_eq!(breaker.state, CircuitState::Closed);
    assert!(breaker.should_allow_request());

    // Record failures to open circuit
    for _ in 0..5 {
        breaker.record_failure();
    }
    assert_eq!(breaker.state, CircuitState::Open);

    // Should not allow requests when open
    assert!(!breaker.should_allow_request());

    // Transition to half-open
    breaker.transition_to_half_open();
    assert_eq!(breaker.state, CircuitState::HalfOpen);
    assert!(breaker.should_allow_request());

    // Record successes to close circuit
    for _ in 0..3 {
        breaker.record_success();
    }
    assert_eq!(breaker.state, CircuitState::Closed);
}

#[tokio::test]
async fn test_batch_checkpoint_functionality() {
    let batch_id = Uuid::new_v4();
    let mut checkpoint = BatchCheckpoint::new(
        batch_id,
        "spotify".to_string(),
        "remove_tracks".to_string(),
        100,
    );

    assert_eq!(checkpoint.progress_percentage(), 0.0);
    assert!(!checkpoint.is_complete());

    checkpoint.update_progress(
        50,
        5,
        55,
        Some("track_123".to_string()),
        serde_json::json!({"current_playlist": "playlist_456"}),
    );

    assert_eq!(checkpoint.progress_percentage(), 55.0);
    assert!(!checkpoint.is_complete());

    checkpoint.update_progress(
        90,
        10,
        100,
        Some("track_999".to_string()),
        serde_json::json!({"completed": true}),
    );

    assert_eq!(checkpoint.progress_percentage(), 100.0);
    assert!(checkpoint.is_complete());
}

#[tokio::test]
async fn test_optimal_batching() {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let rate_limiter = match RateLimitingService::new(&redis_url) {
        Ok(service) => Arc::new(service),
        Err(_) => {
            println!("Skipping batching test - Redis not available");
            return;
        }
    };

    // Test creating optimal batches
    let items: Vec<String> = (0..100).map(|i| format!("item_{}", i)).collect();
    let batches = rate_limiter
        .create_optimal_batches("spotify", "remove_tracks", items)
        .await
        .unwrap();

    assert!(!batches.is_empty(), "Should create at least one batch");

    // Verify batch sizes are reasonable
    for batch in &batches {
        assert!(batch.len() <= 50, "Batch size should not exceed maximum");
        assert!(!batch.is_empty(), "Batch should not be empty");
    }

    // Verify all items are included
    let total_items: usize = batches.iter().map(|b| b.len()).sum();
    assert_eq!(total_items, 100, "All items should be included in batches");
}

#[tokio::test]
async fn test_rate_limit_config_builder() {
    let config = RateLimitConfigBuilder::new("test_provider".to_string())
        .requests_per_window(200)
        .window_duration_seconds(120)
        .burst_allowance(30)
        .backoff_multiplier(1.8)
        .max_backoff_seconds(180)
        .build();

    assert_eq!(config.provider, "test_provider");
    assert_eq!(config.requests_per_window, 200);
    assert_eq!(config.window_duration_seconds, 120);
    assert_eq!(config.burst_allowance, 30);
    assert_eq!(config.backoff_multiplier, 1.8);
    assert_eq!(config.max_backoff_seconds, 180);
}

#[tokio::test]
async fn test_rate_limited_request() {
    let request = RateLimitedRequest::new(
        "spotify".to_string(),
        "remove_tracks".to_string(),
        serde_json::json!({"track_ids": ["123", "456"]}),
    )
    .with_priority(RequestPriority::High)
    .with_max_retries(5);

    assert_eq!(request.provider, "spotify");
    assert_eq!(request.operation_type, "remove_tracks");
    assert_eq!(request.priority, RequestPriority::High);
    assert_eq!(request.max_retries, 5);
    assert_eq!(request.retry_count, 0);
    assert!(request.can_retry());

    let mut request_mut = request;
    request_mut.increment_retry();
    assert_eq!(request_mut.retry_count, 1);
    assert!(request_mut.can_retry());

    // Test retry limit
    for _ in 0..4 {
        request_mut.increment_retry();
    }
    assert_eq!(request_mut.retry_count, 5);
    assert!(!request_mut.can_retry());
}
