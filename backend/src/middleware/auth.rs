use crate::models::{Claims, User};
use crate::services::AuthService;
use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

/// Authentication middleware for Axum routes
pub async fn auth_middleware(
    State(auth_service): State<Arc<AuthService>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(&header[7..])
            } else {
                None
            }
        });

    let token = match auth_header {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // Verify JWT token
    let claims = match auth_service.verify_token(token) {
        Ok(claims) => claims,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Parse user ID from claims
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Get user from auth service
    let user = match auth_service.get_user(user_id).await {
        Ok(user) => user,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Add user and claims to request extensions for use in handlers
    request.extensions_mut().insert(claims);
    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}

/// Optional authentication middleware - doesn't fail if no token provided
pub async fn optional_auth_middleware(
    State(auth_service): State<Arc<AuthService>>,
    mut request: Request,
    next: Next,
) -> Response {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(&header[7..])
            } else {
                None
            }
        });

    if let Some(token) = auth_header {
        // Try to verify token, but don't fail if invalid
        if let Ok(claims) = auth_service.verify_token(token) {
            if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
                if let Ok(user) = auth_service.get_user(user_id).await {
                    // Add user and claims to request extensions
                    request.extensions_mut().insert(claims);
                    request.extensions_mut().insert(user);
                }
            }
        }
    }

    next.run(request).await
}

/// Admin-only authentication middleware
pub async fn admin_auth_middleware(
    State(auth_service): State<Arc<AuthService>>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    // First run regular auth middleware logic
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(&header[7..])
            } else {
                None
            }
        });

    let token = match auth_header {
        Some(token) => token,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "message": "Authorization header required"
                })),
            ))
        }
    };

    let claims = match auth_service.verify_token(token) {
        Ok(claims) => claims,
        Err(_) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "message": "Invalid or expired token"
                })),
            ))
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "message": "Invalid user ID in token"
                })),
            ))
        }
    };

    let user = match auth_service.get_user(user_id).await {
        Ok(user) => user,
        Err(_) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "message": "User not found"
                })),
            ))
        }
    };

    // Check if user has admin scope
    if !claims.scopes.contains(&"admin".to_string()) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "success": false,
                "message": "Admin access required"
            })),
        ));
    }

    // Add user and claims to request extensions
    request.extensions_mut().insert(claims);
    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}

/// Rate limiting middleware for authentication endpoints
pub async fn auth_rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    // Simple rate limiting based on IP address
    // In production, use a proper rate limiting solution like tower-governor
    
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|header| header.to_str().ok())
        .unwrap_or("unknown");

    // For demo purposes, we'll allow all requests
    // In production, implement proper rate limiting logic here
    tracing::debug!("Auth request from IP: {}", client_ip);

    Ok(next.run(request).await)
}

/// Extract user from request extensions (helper for handlers)
pub fn extract_user(request: &Request) -> Option<&User> {
    request.extensions().get::<User>()
}

/// Extract claims from request extensions (helper for handlers)
pub fn extract_claims(request: &Request) -> Option<&Claims> {
    request.extensions().get::<Claims>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CreateUserRequest, LoginRequest};
    use crate::services::AuthService;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
        middleware,
        response::Response,
        routing::get,
        Router,
    };
    use std::sync::Arc;
    use tower::ServiceExt;

    async fn protected_handler() -> &'static str {
        "Protected content"
    }

    async fn create_test_user_and_token() -> (Arc<AuthService>, String) {
        let auth_service = Arc::new(AuthService::new());
        
        // Create test user
        let register_request = CreateUserRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        
        auth_service.register_user(register_request).await.unwrap();
        
        // Login to get token
        let login_request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            totp_code: None,
        };
        
        let token_pair = auth_service.login_user(login_request).await.unwrap();
        
        (auth_service, token_pair.access_token)
    }

    #[tokio::test]
    async fn test_auth_middleware_with_valid_token() {
        let (auth_service, token) = create_test_user_and_token().await;
        
        let app = Router::new()
            .route("/protected", get(protected_handler))
            .route_layer(middleware::from_fn_with_state(
                auth_service.clone(),
                auth_middleware,
            ))
            .with_state(auth_service);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/protected")
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_middleware_without_token() {
        let auth_service = Arc::new(AuthService::new());
        
        let app = Router::new()
            .route("/protected", get(protected_handler))
            .route_layer(middleware::from_fn_with_state(
                auth_service.clone(),
                auth_middleware,
            ))
            .with_state(auth_service);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/protected")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_middleware_with_invalid_token() {
        let auth_service = Arc::new(AuthService::new());
        
        let app = Router::new()
            .route("/protected", get(protected_handler))
            .route_layer(middleware::from_fn_with_state(
                auth_service.clone(),
                auth_middleware,
            ))
            .with_state(auth_service);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/protected")
            .header(AUTHORIZATION, "Bearer invalid_token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_optional_auth_middleware_without_token() {
        let auth_service = Arc::new(AuthService::new());
        
        let app = Router::new()
            .route("/optional", get(protected_handler))
            .route_layer(middleware::from_fn_with_state(
                auth_service.clone(),
                optional_auth_middleware,
            ))
            .with_state(auth_service);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/optional")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}