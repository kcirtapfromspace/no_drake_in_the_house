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
            warn!("Unknown environment '{}', using development CORS settings", environment);
            create_development_cors()
        }
    }
}

/// Development CORS configuration - permissive for local development
fn create_development_cors() -> CorsLayer {
    debug!("Configuring CORS for development environment");
    
    let allowed_origins = get_allowed_origins_from_env()
        .unwrap_or_else(|| vec![
            "http://localhost:3000".to_string(),
            "http://localhost:5000".to_string(),
            "http://localhost:8080".to_string(),
            "http://localhost:53136".to_string(),
            "http://127.0.0.1:3000".to_string(),
            "http://127.0.0.1:5000".to_string(),
            "http://127.0.0.1:8080".to_string(),
            "http://127.0.0.1:53136".to_string(),
        ]);

    debug!("Development CORS allowed origins: {:?}", allowed_origins);

    CorsLayer::new()
        .allow_origin(
            allowed_origins
                .iter()
                .filter_map(|origin| origin.parse::<HeaderValue>().ok())
                .collect::<Vec<_>>()
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
        .max_age(std::time::Duration::from_secs(3600)) // 1 hour
}

/// Staging CORS configuration - more restrictive than development
fn create_staging_cors() -> CorsLayer {
    debug!("Configuring CORS for staging environment");
    
    let allowed_origins = get_allowed_origins_from_env()
        .unwrap_or_else(|| vec![
            "https://staging.nodrakeinthe.house".to_string(),
            "https://staging-app.nodrakeinthe.house".to_string(),
        ]);

    debug!("Staging CORS allowed origins: {:?}", allowed_origins);

    CorsLayer::new()
        .allow_origin(
            allowed_origins
                .iter()
                .filter_map(|origin| origin.parse::<HeaderValue>().ok())
                .collect::<Vec<_>>()
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
    
    let allowed_origins = get_allowed_origins_from_env()
        .unwrap_or_else(|| vec![
            "https://nodrakeinthe.house".to_string(),
            "https://app.nodrakeinthe.house".to_string(),
        ]);

    debug!("Production CORS allowed origins: {:?}", allowed_origins);

    CorsLayer::new()
        .allow_origin(
            allowed_origins
                .iter()
                .filter_map(|origin| origin.parse::<HeaderValue>().ok())
                .collect::<Vec<_>>()
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
    env::var("CORS_ALLOWED_ORIGINS")
        .ok()
        .map(|origins| {
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
                        return Err(format!(
                            "Production CORS origin must use HTTPS: {}",
                            origin
                        ));
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
                        return Err(format!(
                            "Invalid CORS origin protocol: {}",
                            origin
                        ));
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
    use std::env;

    #[test]
    fn test_cors_origins_parsing() {
        // Test parsing of comma-separated origins
        env::set_var("CORS_ALLOWED_ORIGINS", "https://example.com,https://app.example.com");
        
        let origins = get_allowed_origins_from_env().unwrap();
        assert_eq!(origins.len(), 2);
        assert!(origins.contains(&"https://example.com".to_string()));
        assert!(origins.contains(&"https://app.example.com".to_string()));
        
        env::remove_var("CORS_ALLOWED_ORIGINS");
    }

    #[test]
    fn test_cors_origins_with_spaces() {
        // Test parsing with spaces around commas
        env::set_var("CORS_ALLOWED_ORIGINS", " https://example.com , https://app.example.com ");
        
        let origins = get_allowed_origins_from_env().unwrap();
        assert_eq!(origins.len(), 2);
        assert!(origins.contains(&"https://example.com".to_string()));
        assert!(origins.contains(&"https://app.example.com".to_string()));
        
        env::remove_var("CORS_ALLOWED_ORIGINS");
    }

    #[test]
    fn test_cors_validation_production() {
        env::set_var("ENVIRONMENT", "production");
        env::set_var("CORS_ALLOWED_ORIGINS", "https://example.com,https://app.example.com");
        
        assert!(validate_cors_config().is_ok());
        
        // Test invalid HTTP in production
        env::set_var("CORS_ALLOWED_ORIGINS", "http://example.com");
        assert!(validate_cors_config().is_err());
        
        // Test localhost in production
        env::set_var("CORS_ALLOWED_ORIGINS", "https://localhost:3000");
        assert!(validate_cors_config().is_err());
        
        env::remove_var("ENVIRONMENT");
        env::remove_var("CORS_ALLOWED_ORIGINS");
    }

    #[test]
    fn test_cors_validation_development() {
        env::set_var("ENVIRONMENT", "development");
        env::set_var("CORS_ALLOWED_ORIGINS", "http://localhost:3000,http://127.0.0.1:5000");
        
        assert!(validate_cors_config().is_ok());
        
        env::remove_var("ENVIRONMENT");
        env::remove_var("CORS_ALLOWED_ORIGINS");
    }

    #[test]
    fn test_cors_no_origins_env() {
        env::remove_var("CORS_ALLOWED_ORIGINS");
        
        let origins = get_allowed_origins_from_env();
        assert!(origins.is_none());
    }

    #[test]
    fn test_cors_empty_origins_env() {
        env::set_var("CORS_ALLOWED_ORIGINS", "");
        
        let origins = get_allowed_origins_from_env().unwrap();
        assert!(origins.is_empty());
        
        env::remove_var("CORS_ALLOWED_ORIGINS");
    }
}