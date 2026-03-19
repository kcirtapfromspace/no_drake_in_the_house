use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use std::env;
use tower_http::cors::CorsLayer;
use tracing::{debug, warn};

/// Create CORS layer with environment-specific configuration
pub fn create_cors_layer() -> CorsLayer {
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

    match environment.as_str() {
        "development" | "dev" => create_development_cors(),
        "staging" => create_staging_cors(),
        "production" | "prod" => create_production_cors(),
        _ => {
            warn!(
                "Unknown environment '{}', using development CORS settings",
                environment
            );
            create_development_cors()
        }
    }
}

/// Development CORS configuration - permissive for local development
fn create_development_cors() -> CorsLayer {
    debug!("Configuring CORS for development environment - allowing any localhost origin");

    // In development, allow any localhost/127.0.0.1 origin (Vite uses random ports)
    CorsLayer::new()
        .allow_origin(tower_http::cors::AllowOrigin::predicate(
            |origin: &HeaderValue, _request_parts: &axum::http::request::Parts| {
                if let Ok(origin_str) = origin.to_str() {
                    let is_localhost = origin_str.starts_with("http://localhost:")
                        || origin_str.starts_with("http://127.0.0.1:")
                        || origin_str == "http://localhost"
                        || origin_str == "http://127.0.0.1";
                    if is_localhost {
                        debug!("Allowing development origin: {}", origin_str);
                    }
                    is_localhost
                } else {
                    false
                }
            },
        ))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            ACCEPT,
            AUTHORIZATION,
            CONTENT_TYPE,
            "x-requested-with".parse().unwrap(),
            "x-correlation-id".parse().unwrap(),
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600)) // 1 hour
}

/// Staging CORS configuration - more restrictive than development
fn create_staging_cors() -> CorsLayer {
    debug!("Configuring CORS for staging environment");

    let allowed_origins = get_allowed_origins_from_env().unwrap_or_else(|| {
        vec![
            "https://staging.nodrakeinthe.house".to_string(),
            "https://staging-app.nodrakeinthe.house".to_string(),
        ]
    });

    debug!("Staging CORS allowed origins: {:?}", allowed_origins);

    CorsLayer::new()
        .allow_origin(
            allowed_origins
                .iter()
                .filter_map(|origin| origin.parse::<HeaderValue>().ok())
                .collect::<Vec<_>>(),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            ACCEPT,
            AUTHORIZATION,
            CONTENT_TYPE,
            "x-requested-with".parse().unwrap(),
            "x-correlation-id".parse().unwrap(),
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(1800)) // 30 minutes
}

/// Production CORS configuration - most restrictive
fn create_production_cors() -> CorsLayer {
    debug!("Configuring CORS for production environment");

    let allowed_origins = get_allowed_origins_from_env().unwrap_or_else(|| {
        vec![
            "https://nodrakeinthe.house".to_string(),
            "https://app.nodrakeinthe.house".to_string(),
        ]
    });

    debug!("Production CORS allowed origins: {:?}", allowed_origins);

    CorsLayer::new()
        .allow_origin(
            allowed_origins
                .iter()
                .filter_map(|origin| origin.parse::<HeaderValue>().ok())
                .collect::<Vec<_>>(),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            ACCEPT,
            AUTHORIZATION,
            CONTENT_TYPE,
            "x-correlation-id".parse().unwrap(),
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(600)) // 10 minutes
}

/// Get allowed origins from environment variable
fn get_allowed_origins_from_env() -> Option<Vec<String>> {
    env::var("CORS_ALLOWED_ORIGINS").ok().map(|origins| {
        origins
            .split(',')
            .map(|origin| origin.trim().to_string())
            .filter(|origin| !origin.is_empty())
            .collect()
    })
}

/// Validate CORS configuration at startup
pub fn validate_cors_config() -> Result<(), String> {
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

    match environment.as_str() {
        "production" | "prod" => {
            // In production, ensure we have explicit allowed origins
            if let Some(origins) = get_allowed_origins_from_env() {
                for origin in &origins {
                    if !origin.starts_with("https://") {
                        return Err(format!("Production CORS origin must use HTTPS: {}", origin));
                    }
                    if origin.contains("localhost") || origin.contains("127.0.0.1") {
                        return Err(format!(
                            "Production CORS should not allow localhost: {}",
                            origin
                        ));
                    }
                }
                debug!("Production CORS configuration validated successfully");
            } else {
                warn!("No CORS_ALLOWED_ORIGINS set for production, using defaults");
            }
        }
        "staging" => {
            // In staging, prefer HTTPS but allow HTTP for testing
            if let Some(origins) = get_allowed_origins_from_env() {
                for origin in &origins {
                    if !origin.starts_with("https://") && !origin.starts_with("http://") {
                        return Err(format!("Invalid CORS origin protocol: {}", origin));
                    }
                }
                debug!("Staging CORS configuration validated successfully");
            }
        }
        "development" | "dev" => {
            // Development is more permissive
            debug!("Development CORS configuration - validation skipped");
        }
        _ => {
            warn!("Unknown environment '{}' for CORS validation", environment);
        }
    }

    Ok(())
}

/// CORS preflight handler for complex requests
pub async fn cors_preflight_handler() -> axum::response::Response {
    axum::response::Response::builder()
        .status(200)
        .body(axum::body::Body::empty())
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env,
        sync::{Mutex, MutexGuard, OnceLock},
    };

    fn env_lock() -> MutexGuard<'static, ()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env lock poisoned")
    }

    struct EnvVarGuard {
        key: &'static str,
        original: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let original = env::var(key).ok();
            env::set_var(key, value);
            Self { key, original }
        }

        fn remove(key: &'static str) -> Self {
            let original = env::var(key).ok();
            env::remove_var(key);
            Self { key, original }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(original) = &self.original {
                env::set_var(self.key, original);
            } else {
                env::remove_var(self.key);
            }
        }
    }

    #[test]
    fn test_cors_origins_parsing() {
        let _env_lock = env_lock();
        // Test parsing of comma-separated origins
        let _origins_guard = EnvVarGuard::set(
            "CORS_ALLOWED_ORIGINS",
            "https://example.com,https://app.example.com",
        );

        let origins = get_allowed_origins_from_env().unwrap();
        assert_eq!(origins.len(), 2);
        assert!(origins.contains(&"https://example.com".to_string()));
        assert!(origins.contains(&"https://app.example.com".to_string()));
    }

    #[test]
    fn test_cors_origins_with_spaces() {
        let _env_lock = env_lock();
        // Test parsing with spaces around commas
        let _origins_guard = EnvVarGuard::set(
            "CORS_ALLOWED_ORIGINS",
            " https://example.com , https://app.example.com ",
        );

        let origins = get_allowed_origins_from_env().unwrap();
        assert_eq!(origins.len(), 2);
        assert!(origins.contains(&"https://example.com".to_string()));
        assert!(origins.contains(&"https://app.example.com".to_string()));
    }

    #[test]
    fn test_cors_validation_production() {
        let _env_lock = env_lock();
        let _environment_guard = EnvVarGuard::set("ENVIRONMENT", "production");
        let _origins_guard = EnvVarGuard::set(
            "CORS_ALLOWED_ORIGINS",
            "https://example.com,https://app.example.com",
        );

        assert!(validate_cors_config().is_ok());

        // Test invalid HTTP in production
        env::set_var("CORS_ALLOWED_ORIGINS", "http://example.com");
        assert!(validate_cors_config().is_err());

        // Test localhost in production
        env::set_var("CORS_ALLOWED_ORIGINS", "https://localhost:3000");
        assert!(validate_cors_config().is_err());
    }

    #[test]
    fn test_cors_validation_development() {
        let _env_lock = env_lock();
        let _environment_guard = EnvVarGuard::set("ENVIRONMENT", "development");
        let _origins_guard = EnvVarGuard::set(
            "CORS_ALLOWED_ORIGINS",
            "http://localhost:3000,http://127.0.0.1:5000",
        );

        assert!(validate_cors_config().is_ok());
    }

    #[test]
    fn test_cors_no_origins_env() {
        let _env_lock = env_lock();
        let _origins_guard = EnvVarGuard::remove("CORS_ALLOWED_ORIGINS");

        let origins = get_allowed_origins_from_env();
        assert!(origins.is_none());
    }

    #[test]
    fn test_cors_empty_origins_env() {
        let _env_lock = env_lock();
        let _origins_guard = EnvVarGuard::set("CORS_ALLOWED_ORIGINS", "");

        let origins = get_allowed_origins_from_env().unwrap();
        assert!(origins.is_empty());
    }
}
