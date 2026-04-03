use axum::{extract::State, response::Json};

use crate::AppState;

/// GET /.well-known/openid-configuration
pub async fn openid_configuration(State(state): State<AppState>) -> Json<serde_json::Value> {
    let issuer = state.auth_service.jwt_issuer();
    Json(serde_json::json!({
        "issuer": issuer,
        "jwks_uri": format!("{}/.well-known/jwks.json", issuer),
        "id_token_signing_alg_values_supported": ["RS256"],
        "subject_types_supported": ["public"],
        "response_types_supported": ["id_token"],
    }))
}

/// GET /.well-known/jwks.json
pub async fn jwks(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(state.auth_service.jwks_response())
}
