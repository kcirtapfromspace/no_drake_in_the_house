// Test library for comprehensive testing infrastructure
// This module provides common testing utilities and organizes all test modules

pub mod common;
pub mod fixtures;
pub mod integration;
pub mod unit;

// Re-export commonly used testing utilities
pub use common::*;
pub use fixtures::*;

// Test configuration and setup
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize test environment (call once per test process)
pub fn init_test_env() {
    INIT.call_once(|| {
        // Set test environment variables
        std::env::set_var("RUST_ENV", "test");
        std::env::set_var("RUST_LOG", "debug");

        // Initialize tracing for tests
        tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter("debug")
            .try_init()
            .ok(); // Ignore error if already initialized
    });
}

/// Test categories for organizing test execution
pub mod categories {
    /// Unit tests - fast, isolated tests
    pub const UNIT: &str = "unit";

    /// Integration tests - tests with database/external dependencies
    pub const INTEGRATION: &str = "integration";

    /// Performance tests - tests that measure performance
    pub const PERFORMANCE: &str = "performance";

    /// End-to-end tests - full workflow tests
    pub const E2E: &str = "e2e";
}

/// Test utilities for common operations
pub mod utils {
    use super::*;
    use std::time::Duration;

    /// Wait for a condition to be true with timeout
    pub async fn wait_for_condition<F, Fut>(
        mut condition: F,
        timeout: Duration,
        check_interval: Duration,
    ) -> bool
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            if condition().await {
                return true;
            }
            tokio::time::sleep(check_interval).await;
        }

        false
    }

    /// Retry an async operation with exponential backoff
    pub async fn retry_with_backoff<F, Fut, T, E>(
        mut operation: F,
        max_retries: usize,
        initial_delay: Duration,
    ) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        let mut delay = initial_delay;

        for attempt in 0..=max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt == max_retries {
                        return Err(e);
                    }
                    tokio::time::sleep(delay).await;
                    delay *= 2; // Exponential backoff
                }
            }
        }

        unreachable!()
    }
}

/// Macros for common test patterns
#[macro_export]
macro_rules! assert_error_contains {
    ($result:expr, $expected:expr) => {
        match $result {
            Ok(_) => panic!("Expected error but got Ok"),
            Err(e) => assert!(
                e.to_string().contains($expected),
                "Error '{}' does not contain '{}'",
                e.to_string(),
                $expected
            ),
        }
    };
}

#[macro_export]
macro_rules! assert_performance {
    ($duration:expr, $max_ms:expr) => {
        assert!(
            $duration.as_millis() <= $max_ms as u128,
            "Operation took {}ms, expected <= {}ms",
            $duration.as_millis(),
            $max_ms
        );
    };
}

#[macro_export]
macro_rules! test_with_db {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        #[serial_test::serial]
        async fn $test_name() {
            $crate::init_test_env();
            let db = $crate::TestDatabase::new().await;
            $test_body(db).await;
        }
    };
}

/// Test result aggregation for reporting
#[derive(Debug, Default)]
pub struct TestResults {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration: Duration,
}

impl TestResults {
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.passed as f64) / (self.total as f64) * 100.0
        }
    }

    pub fn is_passing(&self) -> bool {
        self.failed == 0
    }
}

/// Test configuration for different environments
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub database_url: String,
    pub redis_url: String,
    pub timeout: Duration,
    pub parallel_tests: bool,
    pub cleanup_after_tests: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            database_url: std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
                "postgres://test_user:test_password@localhost:5432/test_db".to_string()
            }),
            redis_url: std::env::var("TEST_REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            timeout: Duration::from_secs(30),
            parallel_tests: false, // Use serial by default for database tests
            cleanup_after_tests: true,
        }
    }
}

/// Test environment setup and teardown
pub struct TestEnvironment {
    pub config: TestConfig,
}

impl TestEnvironment {
    pub fn new() -> Self {
        Self {
            config: TestConfig::default(),
        }
    }

    pub async fn setup(&self) -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();

        // Additional setup can be added here
        // - Start test containers
        // - Initialize test databases
        // - Set up mock services

        Ok(())
    }

    pub async fn teardown(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.cleanup_after_tests {
            // Cleanup operations
            // - Stop test containers
            // - Clean up test data
            // - Reset mock services
        }

        Ok(())
    }
}
