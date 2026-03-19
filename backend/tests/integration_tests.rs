use music_streaming_blocklist_backend::services::{
    CircuitBreakerConfig, CircuitBreakerService, EntityResolutionService, ExternalApiService,
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

fn lazy_test_pool() -> PgPool {
    PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy("postgres://test:test@localhost:5432/test_db")
        .expect("lazy postgres pool should be constructible for smoke tests")
}

#[test]
fn external_api_service_stub_constructs() {
    let _service = ExternalApiService::new();
}

#[tokio::test]
async fn entity_resolution_stub_returns_none_for_unresolved_artist() {
    let service = EntityResolutionService::new(lazy_test_pool());

    let result = service
        .resolve_artist("Nonexistent Artist", Some("spotify"))
        .await
        .expect("stub entity resolution should not fail");

    assert!(result.is_none());
}

#[tokio::test]
async fn circuit_breaker_opens_after_threshold_and_resets() {
    let breaker = CircuitBreakerService::with_config(CircuitBreakerConfig {
        failure_threshold: 1,
        failure_window_seconds: 60,
        open_timeout_seconds: 30,
        half_open_success_threshold: 1,
        half_open_test_interval_seconds: 1,
    });

    assert!(breaker.can_proceed("spotify").await);

    breaker.record_failure("spotify").await;
    assert!(!breaker.can_proceed("spotify").await);

    breaker.reset("spotify").await;
    assert!(breaker.can_proceed("spotify").await);
}

#[tokio::test]
async fn circuit_breaker_closes_after_successful_execution() {
    let breaker = CircuitBreakerService::with_config(CircuitBreakerConfig {
        failure_threshold: 2,
        failure_window_seconds: 60,
        open_timeout_seconds: 30,
        half_open_success_threshold: 1,
        half_open_test_interval_seconds: 1,
    });

    let result = breaker
        .execute("spotify", || async {
            tokio::time::sleep(Duration::from_millis(5)).await;
            Ok::<_, music_streaming_blocklist_backend::AppError>("ok")
        })
        .await
        .expect("successful operation should pass through circuit breaker");

    assert_eq!(result, "ok");
}
