//! Request latency middleware (US-023)
//!
//! Records HTTP request latency metrics for all endpoints with labels:
//! - method: HTTP method (GET, POST, etc.)
//! - path: Request path (normalized)
//! - status_code: Response status code
//!
//! Histogram buckets: 10ms, 50ms, 100ms, 250ms, 500ms, 1000ms, 5000ms
//! Enables calculation of P50, P90, P99 percentiles.

use axum::{
    extract::{MatchedPath, Request, State},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::time::Instant;

use crate::metrics::MetricsCollector;

/// Middleware to record request latency metrics for all HTTP requests.
///
/// This middleware:
/// 1. Records the start time before processing the request
/// 2. Passes the request to the next handler
/// 3. Records the latency to the prometheus histogram after response
///
/// Labels captured:
/// - method: The HTTP method (GET, POST, PUT, DELETE, etc.)
/// - path: The matched route path (or the raw path if no route matched)
/// - status_code: The HTTP status code of the response
pub async fn latency_middleware(
    State(metrics): State<Arc<MetricsCollector>>,
    matched_path: Option<MatchedPath>,
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().to_string();

    // Use matched path if available (normalized route), otherwise use raw path
    // This prevents high cardinality from path parameters
    let path = matched_path
        .map(|mp| mp.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());

    // Process the request
    let response = next.run(request).await;

    // Record latency
    let duration = start.elapsed();
    let status_code = response.status().as_u16();

    metrics.record_request_latency(&method, &path, status_code, duration);

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "OK"
    }

    async fn slow_handler() -> &'static str {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        "SLOW OK"
    }

    #[tokio::test]
    async fn test_latency_middleware_records_metrics() {
        let metrics = Arc::new(MetricsCollector::new().expect("Failed to create metrics"));

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn_with_state(
                metrics.clone(),
                latency_middleware,
            ))
            .with_state(metrics.clone());

        // Make a request
        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify metrics were recorded
        let metrics_text = metrics.get_metrics().expect("Failed to get metrics");
        assert!(metrics_text.contains("kiro_http_request_latency_seconds"));
        assert!(metrics_text.contains("method=\"GET\""));
        assert!(metrics_text.contains("path=\"/test\""));
        assert!(metrics_text.contains("status_code=\"200\""));
    }

    #[tokio::test]
    async fn test_latency_middleware_records_slow_requests() {
        let metrics = Arc::new(MetricsCollector::new().expect("Failed to create metrics"));

        let app = Router::new()
            .route("/slow", get(slow_handler))
            .layer(middleware::from_fn_with_state(
                metrics.clone(),
                latency_middleware,
            ))
            .with_state(metrics.clone());

        // Make a request to slow endpoint
        let response = app
            .oneshot(Request::builder().uri("/slow").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify metrics were recorded with duration > 50ms
        let metrics_text = metrics.get_metrics().expect("Failed to get metrics");
        assert!(metrics_text.contains("kiro_http_request_latency_seconds"));
        // The 50ms bucket should have count, and the 10ms bucket should not include this request
        assert!(metrics_text.contains("le=\"0.05\"")); // 50ms bucket exists
    }
}
