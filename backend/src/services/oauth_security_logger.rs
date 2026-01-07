//! OAuth security logging and monitoring service

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{warn, error, info};
use uuid::Uuid;

use crate::error::oauth::{OAuthError, ClientInfo};
use crate::models::oauth::OAuthProviderType;

/// OAuth security event
#[derive(Debug, Clone)]
pub struct OAuthSecurityEvent {
    pub event_id: Uuid,
    pub provider: OAuthProviderType,
    pub event_type: SecurityEventType,
    pub severity: SecuritySeverity,
    pub description: String,
    pub client_info: Option<ClientInfo>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user_id: Option<Uuid>,
    pub session_id: Option<String>,
}

/// Types of OAuth security events
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SecurityEventType {
    StateValidationFailure,
    CsrfAttackDetected,
    InvalidTokenUsage,
    SuspiciousClientBehavior,
    RateLimitExceeded,
    UnauthorizedAccess,
    ConfigurationError,
    ProviderError,
}

/// Security event severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// OAuth security logger service
pub struct OAuthSecurityLogger {
    events: Arc<RwLock<Vec<OAuthSecurityEvent>>>,
    max_events: usize,
    alert_thresholds: HashMap<SecurityEventType, u32>,
}

impl OAuthSecurityLogger {
    /// Create a new OAuth security logger
    pub fn new() -> Self {
        let mut alert_thresholds = HashMap::new();
        alert_thresholds.insert(SecurityEventType::StateValidationFailure, 5);
        alert_thresholds.insert(SecurityEventType::CsrfAttackDetected, 1);
        alert_thresholds.insert(SecurityEventType::InvalidTokenUsage, 10);
        alert_thresholds.insert(SecurityEventType::SuspiciousClientBehavior, 3);
        alert_thresholds.insert(SecurityEventType::RateLimitExceeded, 20);
        alert_thresholds.insert(SecurityEventType::UnauthorizedAccess, 5);

        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            max_events: 10000, // Keep last 10k events in memory
            alert_thresholds,
        }
    }

    /// Log an OAuth security event
    pub async fn log_security_event(&self, event: OAuthSecurityEvent) {
        let severity = event.severity.clone();
        let event_type = event.event_type.clone();
        let provider = event.provider;
        let description = event.description.clone();

        // Log to structured logging
        match severity {
            SecuritySeverity::Critical => {
                error!(
                    event_id = %event.event_id,
                    provider = %provider,
                    event_type = ?event_type,
                    severity = ?severity,
                    description = %description,
                    client_info = ?event.client_info,
                    user_id = ?event.user_id,
                    "Critical OAuth security event"
                );
            }
            SecuritySeverity::High => {
                error!(
                    event_id = %event.event_id,
                    provider = %provider,
                    event_type = ?event_type,
                    severity = ?severity,
                    description = %description,
                    client_info = ?event.client_info,
                    user_id = ?event.user_id,
                    "High severity OAuth security event"
                );
            }
            SecuritySeverity::Medium => {
                warn!(
                    event_id = %event.event_id,
                    provider = %provider,
                    event_type = ?event_type,
                    severity = ?severity,
                    description = %description,
                    client_info = ?event.client_info,
                    user_id = ?event.user_id,
                    "Medium severity OAuth security event"
                );
            }
            SecuritySeverity::Low => {
                info!(
                    event_id = %event.event_id,
                    provider = %provider,
                    event_type = ?event_type,
                    severity = ?severity,
                    description = %description,
                    client_info = ?event.client_info,
                    user_id = ?event.user_id,
                    "Low severity OAuth security event"
                );
            }
        }

        // Store event in memory
        let mut events = self.events.write().await;
        let event_clone = OAuthSecurityEvent {
            event_id: event.event_id,
            event_type: event.event_type,
            provider: event.provider,
            severity: event.severity,
            description: event.description,
            user_id: event.user_id,
            client_info: event.client_info,
            timestamp: event.timestamp,
            session_id: event.session_id,
        };
        events.push(event_clone);

        // Trim events if we exceed max capacity
        if events.len() > self.max_events {
            let excess = events.len() - self.max_events;
            events.drain(0..excess);
        }

        // Check for alert thresholds
        self.check_alert_thresholds(&events, &event_type, provider).await;
    }

    /// Log OAuth error as security event
    pub async fn log_oauth_error(&self, oauth_error: &OAuthError, client_info: Option<ClientInfo>, user_id: Option<Uuid>) {
        let (event_type, severity, description) = match oauth_error {
            OAuthError::StateValidationFailed { reason, .. } => (
                SecurityEventType::StateValidationFailure,
                SecuritySeverity::High,
                format!("OAuth state validation failed: {}", reason),
            ),
            OAuthError::CsrfAttackDetected { provider, expected_state, received_state, .. } => (
                SecurityEventType::CsrfAttackDetected,
                SecuritySeverity::Critical,
                format!("CSRF attack detected for {}: expected '{}', received '{}'", provider, expected_state, received_state),
            ),
            OAuthError::SecurityViolation { provider, violation, details, .. } => (
                SecurityEventType::SuspiciousClientBehavior,
                SecuritySeverity::High,
                format!("Security violation for {}: {:?} - {}", provider, violation, details),
            ),
            OAuthError::InvalidToken { provider, token_type, reason } => (
                SecurityEventType::InvalidTokenUsage,
                SecuritySeverity::Medium,
                format!("Invalid {} token for {}: {}", token_type, provider, reason),
            ),
            OAuthError::RateLimitExceeded { provider, limit_type, .. } => (
                SecurityEventType::RateLimitExceeded,
                SecuritySeverity::Medium,
                format!("Rate limit exceeded for {} ({})", provider, limit_type),
            ),
            OAuthError::InvalidConfiguration { provider, reason, .. } => (
                SecurityEventType::ConfigurationError,
                SecuritySeverity::High,
                format!("OAuth configuration error for {}: {}", provider, reason),
            ),
            OAuthError::ProviderError { provider, error_code, message, .. } => (
                SecurityEventType::ProviderError,
                SecuritySeverity::Low,
                format!("Provider error for {} ({}): {}", provider, error_code, message),
            ),
            _ => return, // Don't log other types of errors as security events
        };

        let provider = oauth_error.get_provider().cloned().unwrap_or(OAuthProviderType::Google);

        let event = OAuthSecurityEvent {
            event_id: Uuid::new_v4(),
            provider,
            event_type,
            severity,
            description,
            client_info,
            timestamp: chrono::Utc::now(),
            user_id,
            session_id: None, // Could be extracted from client_info if available
        };

        self.log_security_event(event).await;
    }

    /// Check if alert thresholds are exceeded
    async fn check_alert_thresholds(&self, events: &[OAuthSecurityEvent], event_type: &SecurityEventType, provider: OAuthProviderType) {
        if let Some(&threshold) = self.alert_thresholds.get(event_type) {
            let recent_events = events.iter()
                .filter(|e| {
                    e.provider == provider &&
                    std::mem::discriminant(&e.event_type) == std::mem::discriminant(event_type) &&
                    e.timestamp > chrono::Utc::now() - chrono::Duration::hours(1)
                })
                .count() as u32;

            if recent_events >= threshold {
                error!(
                    provider = %provider,
                    event_type = ?event_type,
                    count = recent_events,
                    threshold = threshold,
                    "OAuth security alert threshold exceeded"
                );

                // In a production system, this would trigger alerts to security teams
                // For now, we just log the alert
            }
        }
    }

    /// Get recent security events for a provider
    pub async fn get_recent_events(&self, provider: Option<OAuthProviderType>, hours: i64) -> Vec<OAuthSecurityEvent> {
        let events = self.events.read().await;
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(hours);

        events.iter()
            .filter(|e| {
                e.timestamp > cutoff &&
                (provider.is_none() || Some(e.provider) == provider)
            })
            .cloned()
            .collect()
    }

    /// Get security event statistics
    pub async fn get_security_stats(&self, provider: Option<OAuthProviderType>, hours: i64) -> SecurityStats {
        let events = self.get_recent_events(provider, hours).await;
        
        let mut stats = SecurityStats {
            total_events: events.len(),
            events_by_type: HashMap::new(),
            events_by_severity: HashMap::new(),
            events_by_provider: HashMap::new(),
        };

        for event in events {
            *stats.events_by_type.entry(format!("{:?}", event.event_type)).or_insert(0) += 1;
            *stats.events_by_severity.entry(format!("{:?}", event.severity)).or_insert(0) += 1;
            *stats.events_by_provider.entry(event.provider.to_string()).or_insert(0) += 1;
        }

        stats
    }

    /// Clear old events (cleanup task)
    pub async fn cleanup_old_events(&self, max_age_hours: i64) {
        let mut events = self.events.write().await;
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(max_age_hours);
        
        events.retain(|e| e.timestamp > cutoff);
        
        info!(
            remaining_events = events.len(),
            max_age_hours = max_age_hours,
            "Cleaned up old OAuth security events"
        );
    }
}

/// Security statistics
#[derive(Debug, Clone)]
pub struct SecurityStats {
    pub total_events: usize,
    pub events_by_type: HashMap<String, u32>,
    pub events_by_severity: HashMap<String, u32>,
    pub events_by_provider: HashMap<String, u32>,
}

impl Default for OAuthSecurityLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_event_logging() {
        let logger = OAuthSecurityLogger::new();
        
        let event = OAuthSecurityEvent {
            event_id: Uuid::new_v4(),
            provider: OAuthProviderType::Google,
            event_type: SecurityEventType::StateValidationFailure,
            severity: SecuritySeverity::High,
            description: "Test security event".to_string(),
            client_info: None,
            timestamp: chrono::Utc::now(),
            user_id: None,
            session_id: None,
        };

        logger.log_security_event(event).await;

        let recent_events = logger.get_recent_events(Some(OAuthProviderType::Google), 1).await;
        assert_eq!(recent_events.len(), 1);
        assert_eq!(recent_events[0].description, "Test security event");
    }

    #[tokio::test]
    async fn test_oauth_error_logging() {
        let logger = OAuthSecurityLogger::new();
        
        let oauth_error = OAuthError::StateValidationFailed {
            reason: "Invalid state parameter".to_string(),
            expected_provider: Some(OAuthProviderType::Google),
            received_provider: None,
        };

        logger.log_oauth_error(&oauth_error, None, None).await;

        let recent_events = logger.get_recent_events(None, 1).await;
        assert_eq!(recent_events.len(), 1);
        assert!(matches!(recent_events[0].event_type, SecurityEventType::StateValidationFailure));
        assert!(matches!(recent_events[0].severity, SecuritySeverity::High));
    }

    #[tokio::test]
    async fn test_security_stats() {
        let logger = OAuthSecurityLogger::new();
        
        // Log multiple events
        for i in 0..3 {
            let event = OAuthSecurityEvent {
                event_id: Uuid::new_v4(),
                provider: OAuthProviderType::Google,
                event_type: SecurityEventType::StateValidationFailure,
                severity: SecuritySeverity::High,
                description: format!("Test event {}", i),
                client_info: None,
                timestamp: chrono::Utc::now(),
                user_id: None,
                session_id: None,
            };
            logger.log_security_event(event).await;
        }

        let stats = logger.get_security_stats(Some(OAuthProviderType::Google), 1).await;
        assert_eq!(stats.total_events, 3);
        assert_eq!(stats.events_by_provider.get("Google"), Some(&3));
    }

    #[tokio::test]
    async fn test_event_cleanup() {
        let logger = OAuthSecurityLogger::new();
        
        // Create an old event
        let old_event = OAuthSecurityEvent {
            event_id: Uuid::new_v4(),
            provider: OAuthProviderType::Google,
            event_type: SecurityEventType::StateValidationFailure,
            severity: SecuritySeverity::High,
            description: "Old event".to_string(),
            client_info: None,
            timestamp: chrono::Utc::now() - chrono::Duration::hours(25), // 25 hours ago
            user_id: None,
            session_id: None,
        };

        logger.log_security_event(old_event).await;
        
        // Verify event exists
        let events_before = logger.get_recent_events(None, 48).await;
        assert_eq!(events_before.len(), 1);

        // Cleanup events older than 24 hours
        logger.cleanup_old_events(24).await;

        // Verify event was removed
        let events_after = logger.get_recent_events(None, 48).await;
        assert_eq!(events_after.len(), 0);
    }
}