use crate::models::{AuthenticatedUser, Claims, User};
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

// FromRequestParts impls for AuthenticatedUser and Claims are in ndith-core
// (where the types are defined) to satisfy the orphan rule.

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

    // Check if user has admin access (via role or scope)
    if !claims.has_admin_access() {
        tracing::warn!(
            user_id = %user_id,
            role = ?claims.role,
            scopes = ?claims.scopes,
            "Unauthorized admin access attempt"
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "success": false,
                "error_code": "AUTH_INSUFFICIENT_PERMISSIONS",
                "message": "Admin access required"
            })),
        ));
    }

    // Add user and claims to request extensions
    request.extensions_mut().insert(claims);
    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}

/// Moderator-only authentication middleware
/// Requires moderator or admin role
pub async fn moderator_auth_middleware(
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
                    "error_code": "AUTH_TOKEN_REQUIRED",
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
                    "error_code": "AUTH_TOKEN_INVALID",
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
                    "error_code": "AUTH_TOKEN_INVALID",
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
                    "error_code": "AUTH_USER_NOT_FOUND",
                    "message": "User not found"
                })),
            ))
        }
    };

    // Check if user has moderator access (via role or scope)
    if !claims.has_moderator_access() {
        tracing::warn!(
            user_id = %user_id,
            role = ?claims.role,
            scopes = ?claims.scopes,
            "Unauthorized moderator access attempt"
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "success": false,
                "error_code": "AUTH_INSUFFICIENT_PERMISSIONS",
                "message": "Moderator access required"
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

/// Extract the authenticated user's ID from a validated auth extractor.
pub fn authenticated_user_id(user: &AuthenticatedUser) -> Uuid {
    user.id
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn authenticated_user_id_returns_authenticated_identity() {
        let user_id = Uuid::new_v4();
        let user = AuthenticatedUser {
            id: user_id,
            email: "launch-test@example.com".to_string(),
        };

        assert_eq!(authenticated_user_id(&user), user_id);
    }
}
