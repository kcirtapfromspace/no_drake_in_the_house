use fake::{Fake, Faker};
use music_streaming_blocklist_backend::{
    initialize_database, models::*, services::*, DatabaseConfig,
};
use sqlx::PgPool;
use std::sync::Once;
use std::time::Duration;
use testcontainers::{clients::Cli, images::postgres::Postgres, Container};
use uuid::Uuid;

static INIT: Once = Once::new();

/// Initialize tracing for tests (only once)
pub fn init_test_tracing() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter("debug")
            .init();
    });
}

/// Test database container wrapper
pub struct TestDatabase {
    pub pool: PgPool,
    pub _container: Container<'static, Postgres>,
}

impl TestDatabase {
    pub async fn new() -> Self {
        init_test_tracing();

        let docker = Cli::default();
        let postgres_image = Postgres::default()
            .with_db_name("test_db")
            .with_user("test_user")
            .with_password("test_password");

        let container = docker.run(postgres_image);
        let connection_string = format!(
            "postgres://test_user:test_password@127.0.0.1:{}/test_db",
            container.get_host_port_ipv4(5432)
        );

        let config = DatabaseConfig {
            url: connection_string,
            max_connections: 5,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300),
        };

        let pool = initialize_database(config)
            .await
            .expect("Failed to initialize test database");

        Self {
            pool,
            _container: container,
        }
    }

    /// Create a test user with random data
    pub async fn create_test_user(&self) -> User {
        let auth_service = AuthService::new(self.pool.clone());
        let request = CreateUserRequest {
            email: format!("test_{}@example.com", Uuid::new_v4()),
            password: "test_password_123".to_string(),
        };

        auth_service
            .register_user(request)
            .await
            .expect("Failed to create test user")
    }

    /// Create a test artist with random data
    pub async fn create_test_artist(&self, name: Option<&str>) -> Artist {
        let artist_name = name.unwrap_or(&format!("Test Artist {}", Uuid::new_v4()));

        sqlx::query_as!(
            Artist,
            r#"
            INSERT INTO artists (canonical_name, external_ids, metadata)
            VALUES ($1, $2, $3)
            RETURNING id, canonical_name, external_ids, metadata, created_at
            "#,
            artist_name,
            serde_json::json!({"spotify": format!("spotify_{}", Uuid::new_v4())}),
            serde_json::json!({"genres": ["test"], "image": "https://example.com/image.jpg"})
        )
        .fetch_one(&self.pool)
        .await
        .expect("Failed to create test artist")
    }

    /// Clean up test data (useful for isolated tests)
    pub async fn cleanup(&self) {
        let _ = sqlx::query("DELETE FROM user_artist_blocks")
            .execute(&self.pool)
            .await;
        let _ = sqlx::query("DELETE FROM user_sessions")
            .execute(&self.pool)
            .await;
        let _ = sqlx::query("DELETE FROM audit_log")
            .execute(&self.pool)
            .await;
        let _ = sqlx::query("DELETE FROM artists").execute(&self.pool).await;
        let _ = sqlx::query("DELETE FROM users").execute(&self.pool).await;
    }
}

/// Test data factories using the fake crate
pub struct TestDataFactory;

impl TestDataFactory {
    pub fn create_user_request() -> CreateUserRequest {
        CreateUserRequest {
            email: format!("{}@example.com", Uuid::new_v4()),
            password: "SecurePassword123!".to_string(),
        }
    }

    pub fn create_login_request(email: String) -> LoginRequest {
        LoginRequest {
            email,
            password: "SecurePassword123!".to_string(),
            totp_code: None,
        }
    }

    pub fn create_artist_data() -> (String, serde_json::Value, serde_json::Value) {
        let name: String = fake::faker::name::en::Name().fake();
        let external_ids = serde_json::json!({
            "spotify": format!("spotify_{}", Uuid::new_v4()),
            "apple_music": format!("apple_{}", Uuid::new_v4())
        });
        let metadata = serde_json::json!({
            "genres": ["rock", "pop"],
            "image": "https://example.com/artist.jpg",
            "popularity": fake::faker::number::en::NumberWithinRange(0..100).fake::<u8>()
        });

        (name, external_ids, metadata)
    }
}

/// Assertion helpers for common test patterns
pub struct TestAssertions;

impl TestAssertions {
    pub fn assert_valid_jwt(token: &str) {
        assert!(!token.is_empty());
        assert_eq!(
            token.matches('.').count(),
            2,
            "JWT should have 3 parts separated by dots"
        );
    }

    pub fn assert_valid_uuid(uuid_str: &str) {
        Uuid::parse_str(uuid_str).expect("Should be a valid UUID");
    }

    pub fn assert_bcrypt_hash(hash: &str) {
        assert!(hash.starts_with("$2b$"), "Should be a bcrypt hash");
        let parts: Vec<&str> = hash.split('$').collect();
        assert!(parts.len() >= 4, "Bcrypt hash should have at least 4 parts");

        if let Ok(cost) = parts[2].parse::<u32>() {
            assert!(cost >= 12, "Bcrypt cost should be at least 12");
        }
    }

    pub fn assert_email_format(email: &str) {
        assert!(email.contains('@'), "Email should contain @");
        assert!(email.contains('.'), "Email should contain domain");
    }
}

/// Mock service builder for unit tests
pub struct MockServiceBuilder {
    pub auth_service: Option<AuthService>,
}

impl MockServiceBuilder {
    pub fn new() -> Self {
        Self { auth_service: None }
    }

    pub async fn with_database(mut self, pool: PgPool) -> Self {
        self.auth_service = Some(AuthService::new(pool));
        self
    }

    pub fn build(self) -> MockServices {
        MockServices {
            auth_service: self.auth_service.expect("Auth service must be configured"),
        }
    }
}

pub struct MockServices {
    pub auth_service: AuthService,
}

/// Performance test helpers
pub struct PerformanceTestHelper;

impl PerformanceTestHelper {
    pub async fn measure_async<F, Fut, T>(operation: F) -> (T, Duration)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let start = std::time::Instant::now();
        let result = operation().await;
        let duration = start.elapsed();
        (result, duration)
    }

    pub fn assert_performance_threshold(duration: Duration, threshold_ms: u64) {
        assert!(
            duration.as_millis() <= threshold_ms as u128,
            "Operation took {}ms, expected <= {}ms",
            duration.as_millis(),
            threshold_ms
        );
    }
}
