use std::time::Instant;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::{info, warn, error, Span};
use uuid::Uuid;

use crate::services::monitoring::{CorrelationId, MonitoringService, StructuredLogEntry};

/// Header name for correlation ID
pub const CORRELATION_ID_HEADER: &str = "x-correlation-id";

/// Middleware to add correlation IDs and structured logging to requests
pub async fn tracing_middleware(
    State(monitoring_service): State<Arc<MonitoringService>>,
    mut request: Request,
    next: Next,
) -> Response {
    let start_time = Instant::now();
    
    // Extract or generate correlation ID
    let correlation_id = extract_or_generate_correlation_id(request.headers());
    
    // Add correlation ID to request extensions for use in handlers
    request.extensions_mut().insert(correlation_id.clone());
    
    // Create tracing span with correlation ID
    let span = tracing::info_span!(
        "http_request",
        correlation_id = %correlation_id.as_str(),
        method = %request.method(),
        uri = %request.uri(),
    );
    
    let _enter = span.enter();
    
    // Extract request information
    let method = request.method().to_string();
    let uri = request.uri().to_string();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    
    // Process the request
    let response = next.run(request).await;
    
    // Calculate duration
    let duration = start_time.elapsed();
    let status_code = response.status().as_u16();
    
    // Record metrics
    monitoring_service.record_http_request(&method, &uri, status_code, duration);
    
    // Create structured log entry
    let log_entry = StructuredLogEntry {
        timestamp: chrono::Utc::now(),
        correlation_id: correlation_id.as_str().to_string(),
        level: if status_code >= 500 {
            "ERROR".to_string()
        } else if status_code >= 400 {
            "WARN".to_string()
        } else {
            "INFO".to_string()
        },
        service: "api".to_string(),
        operation: "http_request".to_string(),
        user_id: None, // Will be populated by auth middleware if available
        duration_ms: Some(duration.as_millis() as u64),
        status: status_code.to_string(),
        message: format!("{} {} -> {}", method, uri, status_code),
        metadata: {
            let mut metadata = std::collections::HashMap::new();
            metadata.insert("method".to_string(), serde_json::Value::String(method));
            metadata.insert("uri".to_string(), serde_json::Value::String(uri));
            metadata.insert("status_code".to_string(), serde_json::Value::Number(serde_json::Number::from(status_code)));
            metadata.insert("duration_ms".to_string(), serde_json::Value::Number(serde_json::Number::from(duration.as_millis() as u64)));
            metadata.insert("user_agent".to_string(), serde_json::Value::String(user_agent));
            metadata
        },
    };
    
    // Log the structured entry
    monitoring_service.log_structured(log_entry);
    
    // Add correlation ID to response headers
    let mut response = response;
    if let Ok(header_value) = HeaderValue::from_str(correlation_id.as_str()) {
        response.headers_mut().insert(
            HeaderName::from_static(CORRELATION_ID_HEADER),
            header_value,
        );
    }
    
    response
}

/// Extract correlation ID from headers or generate a new one
fn extract_or_generate_correlation_id(headers: &HeaderMap) -> CorrelationId {
    headers
        .get(CORRELATION_ID_HEADER)
        .and_then(|h| h.to_str().ok())
        .map(|s| CorrelationId::from_string(s.to_string()))
        .unwrap_or_else(CorrelationId::new)
}

/// Middleware to add user information to structured logs
pub async fn user_context_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    // This middleware runs after auth middleware, so user should be in extensions
    let user_id = request
        .extensions()
        .get::<crate::models::User>()
        .map(|user| user.id);
    
    // Store user ID in request extensions for logging
    if let Some(user_id) = user_id {
        request.extensions_mut().insert(user_id);
    }
    
    next.run(request).await
}

/// Helper function to get correlation ID from request extensions
pub fn get_correlation_id_from_request(request: &Request) -> Option<CorrelationId> {
    request.extensions().get::<CorrelationId>().cloned()
}

/// Helper function to create a child correlation ID for sub-operations
pub fn create_child_correlation_id(parent: &CorrelationId, operation: &str) -> CorrelationId {
    CorrelationId::from_string(format!("{}.{}.{}", parent.as_str(), operation, Uuid::new_v4()))
}

/// Macro for structured logging with correlation ID
#[macro_export]
macro_rules! log_with_correlation {
    ($level:ident, $correlation_id:expr, $service:expr, $operation:expr, $message:expr) => {
        log_with_correlation!($level, $correlation_id, $service, $operation, $message, std::collections::HashMap::new())
    };
    ($level:ident, $correlation_id:expr, $service:expr, $operation:expr, $message:expr, $metadata:expr) => {
        {
            let entry = $crate::services::monitoring::StructuredLogEntry {
                timestamp: chrono::Utc::now(),
                correlation_id: $correlation_id.as_str().to_string(),
                level: stringify!($level).to_uppercase(),
                service: $service.to_string(),
                operation: $operation.to_string(),
                user_id: None,
                duration_ms: None,
                status: "logged".to_string(),
                message: $message.to_string(),
                metadata: $metadata,
            };
            
            match stringify!($level) {
                "error" => tracing::error!(correlation_id = %$correlation_id.as_str(), "{}", $message),
                "warn" => tracing::warn!(correlation_id = %$correlation_id.as_str(), "{}", $message),
                "info" => tracing::info!(correlation_id = %$correlation_id.as_str(), "{}", $message),
                "debug" => tracing::debug!(correlation_id = %$correlation_id.as_str(), "{}", $message),
                _ => tracing::info!(correlation_id = %$correlation_id.as_str(), "{}", $message),
            }
        }
    };
}

/// Helper struct for timing operations with correlation ID
pub struct OperationTimer {
    correlation_id: CorrelationId,
    service: String,
    operation: String,
    start_time: Instant,
}

impl OperationTimer {
    pub fn new(correlation_id: CorrelationId, service: String, operation: String) -> Self {
        Self {
            correlation_id,
            service,
            operation,
            start_time: Instant::now(),
        }
    }
    
    pub fn finish(self, monitoring_service: &MonitoringService, success: bool, message: String) {
        let duration = self.start_time.elapsed();
        
        let log_entry = StructuredLogEntry {
            timestamp: chrono::Utc::now(),
            correlation_id: self.correlation_id.as_str().to_string(),
            level: if success { "INFO".to_string() } else { "ERROR".to_string() },
            service: self.service,
            operation: self.operation,
            user_id: None,
            duration_ms: Some(duration.as_millis() as u64),
            status: if success { "success".to_string() } else { "failure".to_string() },
            message,
            metadata: {
                let mut metadata = std::collections::HashMap::new();
                metadata.insert("duration_ms".to_string(), serde_json::Value::Number(serde_json::Number::from(duration.as_millis() as u64)));
                metadata.insert("success".to_string(), serde_json::Value::Bool(success));
                metadata
            },
        };
        
        monitoring_service.log_structured(log_entry);
    }
}