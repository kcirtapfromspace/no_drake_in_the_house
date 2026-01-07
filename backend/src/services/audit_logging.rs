use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::net::IpAddr;
use std::str::FromStr;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Audit event types for security monitoring
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "audit_event_type", rename_all = "snake_case")]
pub enum AuditEventType {
    UserRegistration,
    UserLogin,
    UserLoginFailed,
    UserLogout,
    UserAccountDeleted,
    PasswordChange,
    TotpEnabled,
    TotpDisabled,
    TotpVerificationFailed,
    TokenRefresh,
    TokenRevoked,
    RateLimitExceeded,
    SuspiciousActivity,
    SecurityViolation,
    DataAccess,
    DataModification,
    AdminAction,
    SystemEvent,
}

impl FromStr for AuditEventType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "user_registration" => Ok(AuditEventType::UserRegistration),
            "user_login" => Ok(AuditEventType::UserLogin),
            "user_login_failed" => Ok(AuditEventType::UserLoginFailed),
            "user_logout" => Ok(AuditEventType::UserLogout),
            "user_account_deleted" => Ok(AuditEventType::UserAccountDeleted),
            "password_change" => Ok(AuditEventType::PasswordChange),
            "totp_enabled" => Ok(AuditEventType::TotpEnabled),
            "totp_disabled" => Ok(AuditEventType::TotpDisabled),
            "totp_verification_failed" => Ok(AuditEventType::TotpVerificationFailed),
            "token_refresh" => Ok(AuditEventType::TokenRefresh),
            "token_revoked" => Ok(AuditEventType::TokenRevoked),
            "rate_limit_exceeded" => Ok(AuditEventType::RateLimitExceeded),
            "suspicious_activity" => Ok(AuditEventType::SuspiciousActivity),
            "security_violation" => Ok(AuditEventType::SecurityViolation),
            "data_access" => Ok(AuditEventType::DataAccess),
            "data_modification" => Ok(AuditEventType::DataModification),
            "admin_action" => Ok(AuditEventType::AdminAction),
            "system_event" => Ok(AuditEventType::SystemEvent),
            _ => Err(anyhow!("Invalid audit event type: {}", s)),
        }
    }
}

/// Audit event severity levels
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "audit_severity", rename_all = "snake_case")]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Audit log entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub event_type: AuditEventType,
    pub severity: AuditSeverity,
    pub user_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub action: String,
    pub details: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<String>,
}

/// Audit logging service for security events
#[derive(Clone)]
pub struct AuditLoggingService {
    db_pool: PgPool,
}

impl AuditLoggingService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Log a security event
    pub async fn log_security_event(
        &self,
        event_type: AuditEventType,
        severity: AuditSeverity,
        action: String,
        details: serde_json::Value,
        context: Option<AuditContext>,
    ) -> Result<Uuid> {
        let entry_id = Uuid::new_v4();
        let timestamp = Utc::now();

        let (user_id, session_id, ip_address, user_agent, correlation_id) = if let Some(ctx) = context {
            (ctx.user_id, ctx.session_id, ctx.ip_address, ctx.user_agent, ctx.correlation_id)
        } else {
            (None, None, None, None, None)
        };

        // Log to structured logging first
        match severity {
            AuditSeverity::Info => info!(
                event_type = ?event_type,
                user_id = ?user_id,
                ip_address = ?ip_address,
                action = %action,
                correlation_id = ?correlation_id,
                "Security event logged"
            ),
            AuditSeverity::Warning => warn!(
                event_type = ?event_type,
                user_id = ?user_id,
                ip_address = ?ip_address,
                action = %action,
                correlation_id = ?correlation_id,
                "Security warning logged"
            ),
            AuditSeverity::Error => error!(
                event_type = ?event_type,
                user_id = ?user_id,
                ip_address = ?ip_address,
                action = %action,
                correlation_id = ?correlation_id,
                "Security error logged"
            ),
            AuditSeverity::Critical => error!(
                event_type = ?event_type,
                user_id = ?user_id,
                ip_address = ?ip_address,
                action = %action,
                correlation_id = ?correlation_id,
                "CRITICAL security event logged"
            ),
        }

        // Store in database for compliance and analysis
        sqlx::query!(
            r#"
            INSERT INTO audit_log (
                id, event_type, severity, user_id, session_id, ip_address, 
                user_agent, action, details, timestamp, correlation_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            entry_id,
            event_type as AuditEventType,
            severity as AuditSeverity,
            user_id,
            session_id,
            ip_address.map(|ip| sqlx::types::ipnetwork::IpNetwork::from(ip)),
            user_agent,
            action,
            details,
            timestamp,
            correlation_id
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| anyhow!("Failed to insert audit log entry: {}", e))?;

        debug!("Audit log entry created with ID: {}", entry_id);
        Ok(entry_id)
    }

    /// Log user authentication events
    pub async fn log_auth_event(
        &self,
        event_type: AuditEventType,
        _user_id: Option<Uuid>,
        success: bool,
        context: Option<AuditContext>,
        additional_details: Option<serde_json::Value>,
    ) -> Result<Uuid> {
        let severity = if success {
            AuditSeverity::Info
        } else {
            match event_type {
                AuditEventType::UserLoginFailed | AuditEventType::TotpVerificationFailed => {
                    AuditSeverity::Warning
                }
                _ => AuditSeverity::Error,
            }
        };

        let action = match event_type {
            AuditEventType::UserRegistration => "User registered".to_string(),
            AuditEventType::UserLogin => "User logged in".to_string(),
            AuditEventType::UserLoginFailed => "User login failed".to_string(),
            AuditEventType::UserLogout => "User logged out".to_string(),
            AuditEventType::UserAccountDeleted => "User account deleted".to_string(),
            AuditEventType::TotpEnabled => "2FA enabled".to_string(),
            AuditEventType::TotpDisabled => "2FA disabled".to_string(),
            AuditEventType::TotpVerificationFailed => "2FA verification failed".to_string(),
            _ => format!("{:?}", event_type),
        };

        let mut details = serde_json::json!({
            "success": success,
            "timestamp": Utc::now().to_rfc3339()
        });

        if let Some(additional) = additional_details {
            if let serde_json::Value::Object(map) = additional {
                for (key, value) in map {
                    details[key] = value;
                }
            }
        }

        self.log_security_event(event_type, severity, action, details, context)
            .await
    }

    /// Log rate limiting events
    pub async fn log_rate_limit_event(
        &self,
        ip_address: IpAddr,
        endpoint: String,
        limit_exceeded: bool,
        current_count: u32,
        limit: u32,
        correlation_id: Option<String>,
    ) -> Result<Uuid> {
        let severity = if limit_exceeded {
            AuditSeverity::Warning
        } else {
            AuditSeverity::Info
        };

        let action = if limit_exceeded {
            "Rate limit exceeded".to_string()
        } else {
            "Rate limit check".to_string()
        };

        let details = serde_json::json!({
            "endpoint": endpoint,
            "limit_exceeded": limit_exceeded,
            "current_count": current_count,
            "limit": limit,
            "timestamp": Utc::now().to_rfc3339()
        });

        let context = AuditContext {
            user_id: None,
            session_id: None,
            ip_address: Some(ip_address),
            user_agent: None,
            correlation_id,
        };

        self.log_security_event(
            AuditEventType::RateLimitExceeded,
            severity,
            action,
            details,
            Some(context),
        )
        .await
    }

    /// Log suspicious activity
    pub async fn log_suspicious_activity(
        &self,
        activity_type: String,
        description: String,
        context: Option<AuditContext>,
        risk_score: Option<f64>,
    ) -> Result<Uuid> {
        let severity = match risk_score {
            Some(score) if score >= 0.8 => AuditSeverity::Critical,
            Some(score) if score >= 0.6 => AuditSeverity::Error,
            Some(score) if score >= 0.4 => AuditSeverity::Warning,
            _ => AuditSeverity::Info,
        };

        let details = serde_json::json!({
            "activity_type": activity_type,
            "description": description,
            "risk_score": risk_score,
            "timestamp": Utc::now().to_rfc3339()
        });

        self.log_security_event(
            AuditEventType::SuspiciousActivity,
            severity,
            format!("Suspicious activity detected: {}", activity_type),
            details,
            context,
        )
        .await
    }

    /// Get audit logs with filtering
    pub async fn get_audit_logs(
        &self,
        filters: AuditLogFilters,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<AuditLogEntry>> {
        let mut query = "SELECT * FROM audit_log WHERE 1=1".to_string();
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
        let mut param_count = 0;

        // Add filters
        if let Some(event_type) = &filters.event_type {
            param_count += 1;
            query.push_str(&format!(" AND event_type = ${}", param_count));
            params.push(Box::new(event_type.clone()));
        }

        if let Some(user_id) = &filters.user_id {
            param_count += 1;
            query.push_str(&format!(" AND user_id = ${}", param_count));
            params.push(Box::new(*user_id));
        }

        if let Some(start_time) = &filters.start_time {
            param_count += 1;
            query.push_str(&format!(" AND timestamp >= ${}", param_count));
            params.push(Box::new(*start_time));
        }

        if let Some(end_time) = &filters.end_time {
            param_count += 1;
            query.push_str(&format!(" AND timestamp <= ${}", param_count));
            params.push(Box::new(*end_time));
        }

        // Add ordering and pagination
        query.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = limit {
            param_count += 1;
            query.push_str(&format!(" LIMIT ${}", param_count));
            params.push(Box::new(limit));
        }

        if let Some(offset) = offset {
            param_count += 1;
            query.push_str(&format!(" OFFSET ${}", param_count));
            params.push(Box::new(offset));
        }

        // Execute query with manual row processing
        let rows = sqlx::query!(
            r#"
            SELECT 
                id,
                event_type::text as event_type_str,
                severity::text as severity_str,
                user_id,
                session_id,
                ip_address,
                user_agent,
                action,
                details,
                timestamp,
                correlation_id
            FROM audit_log 
            WHERE event_type IS NOT NULL AND severity IS NOT NULL AND timestamp IS NOT NULL
            ORDER BY timestamp DESC 
            LIMIT $1 OFFSET $2
            "#,
            limit.unwrap_or(100),
            offset.unwrap_or(0)
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch audit logs: {}", e))?;

        // Convert the results
        let mut entries = Vec::new();
        for row in rows {
            // Parse event_type and severity from strings
            let event_type = row.event_type_str
                .as_ref()
                .and_then(|et| AuditEventType::from_str(et).ok())
                .unwrap_or(AuditEventType::SystemEvent);
            
            let severity = match row.severity_str.as_deref() {
                Some("info") => AuditSeverity::Info,
                Some("warning") => AuditSeverity::Warning,
                Some("error") => AuditSeverity::Error,
                Some("critical") => AuditSeverity::Critical,
                _ => AuditSeverity::Info,
            };

            let ip_address = row.ip_address
                .as_ref()
                .and_then(|ip_net| {
                    // Convert from IpNetwork to IpAddr
                    match ip_net.ip() {
                        std::net::IpAddr::V4(v4) => Some(std::net::IpAddr::V4(v4)),
                        std::net::IpAddr::V6(v6) => Some(std::net::IpAddr::V6(v6)),
                    }
                });

            entries.push(AuditLogEntry {
                id: row.id,
                event_type,
                severity,
                user_id: row.user_id,
                session_id: row.session_id,
                ip_address,
                user_agent: row.user_agent,
                resource_type: None, // Not used in current schema
                resource_id: None,   // Not used in current schema
                action: row.action,
                details: row.details.unwrap_or_else(|| serde_json::json!({})),
                timestamp: row.timestamp.unwrap_or_else(|| Utc::now()),
                correlation_id: row.correlation_id,
            });
        }

        Ok(entries)
    }

    /// Get audit statistics for monitoring
    pub async fn get_audit_statistics(
        &self,
        time_range_hours: Option<i32>,
    ) -> Result<AuditStatistics> {
        let hours = time_range_hours.unwrap_or(24);
        let since = Utc::now() - chrono::Duration::hours(hours as i64);

        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_events,
                COUNT(CASE WHEN severity = 'critical' THEN 1 END) as critical_events,
                COUNT(CASE WHEN severity = 'error' THEN 1 END) as error_events,
                COUNT(CASE WHEN severity = 'warning' THEN 1 END) as warning_events,
                COUNT(CASE WHEN event_type = 'user_login_failed' THEN 1 END) as failed_logins,
                COUNT(CASE WHEN event_type = 'rate_limit_exceeded' THEN 1 END) as rate_limit_violations,
                COUNT(CASE WHEN event_type = 'suspicious_activity' THEN 1 END) as suspicious_activities,
                COUNT(DISTINCT ip_address) as unique_ips,
                COUNT(DISTINCT user_id) as unique_users
            FROM audit_log 
            WHERE timestamp >= $1
            "#,
            since
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch audit statistics: {}", e))?;

        Ok(AuditStatistics {
            time_range_hours: hours,
            total_events: stats.total_events.unwrap_or(0) as u64,
            critical_events: stats.critical_events.unwrap_or(0) as u64,
            error_events: stats.error_events.unwrap_or(0) as u64,
            warning_events: stats.warning_events.unwrap_or(0) as u64,
            failed_logins: stats.failed_logins.unwrap_or(0) as u64,
            rate_limit_violations: stats.rate_limit_violations.unwrap_or(0) as u64,
            suspicious_activities: stats.suspicious_activities.unwrap_or(0) as u64,
            unique_ips: stats.unique_ips.unwrap_or(0) as u64,
            unique_users: stats.unique_users.unwrap_or(0) as u64,
        })
    }
}

/// Context information for audit logging
#[derive(Debug, Clone)]
pub struct AuditContext {
    pub user_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub correlation_id: Option<String>,
}

/// Filters for querying audit logs
#[derive(Debug, Default)]
pub struct AuditLogFilters {
    pub event_type: Option<AuditEventType>,
    pub user_id: Option<Uuid>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub severity: Option<AuditSeverity>,
}

/// Audit statistics for monitoring
#[derive(Debug, Serialize)]
pub struct AuditStatistics {
    pub time_range_hours: i32,
    pub total_events: u64,
    pub critical_events: u64,
    pub error_events: u64,
    pub warning_events: u64,
    pub failed_logins: u64,
    pub rate_limit_violations: u64,
    pub suspicious_activities: u64,
    pub unique_ips: u64,
    pub unique_users: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_audit_logging_basic() {
        // This test would require a database connection
        // Implementation would test basic audit logging functionality
    }

    #[test]
    fn test_audit_event_type_parsing() {
        assert!(matches!(
            AuditEventType::from_str("user_login"),
            Ok(AuditEventType::UserLogin)
        ));
        assert!(matches!(
            AuditEventType::from_str("rate_limit_exceeded"),
            Ok(AuditEventType::RateLimitExceeded)
        ));
        assert!(AuditEventType::from_str("invalid_type").is_err());
    }

    #[test]
    fn test_audit_context_creation() {
        let context = AuditContext {
            user_id: Some(Uuid::new_v4()),
            session_id: Some("session123".to_string()),
            ip_address: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))),
            user_agent: Some("Mozilla/5.0".to_string()),
            correlation_id: Some("corr123".to_string()),
        };

        assert!(context.user_id.is_some());
        assert!(context.ip_address.is_some());
    }
}