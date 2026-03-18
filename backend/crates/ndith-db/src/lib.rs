//! ndith-db: PostgreSQL and Redis connection management, health checks, and recovery.

pub mod database;
pub mod health;
pub mod recovery;

// Re-export commonly used types
pub use database::{
    create_pool, create_redis_pool, health_check as db_health_check, redis_health_check,
    run_migrations, seed_test_data, DatabaseConfig, DatabaseHealthStatus, RedisConfiguration,
    RedisHealthStatus,
};
pub use health::{
    liveness_check, readiness_check, HealthCheckConfig, HealthCheckResponse, HealthChecker,
    HealthStatus, ServiceHealthInfo, SystemInfo,
};
pub use recovery::{
    retry_database_operation, retry_redis_operation, with_circuit_breaker,
    with_graceful_degradation, CircuitBreaker, CircuitBreakerState, RetryConfig,
};
