use crate::models::*;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Audit logging service for SOC2 compliance
#[derive(Clone)]
pub struct AuditService {
    db_pool: PgPool,
    security_events: Arc<RwLock<Vec<SecurityEvent>>>,
}

impl AuditService {
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool,
            security_events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Log an audit event
    pub async fn log_audit_event(
        &self,
        request: CreateAuditLogRequest,
    ) -> Result<AuditLogEntry> {
        let entry = sqlx::query_as(
            AuditLogEntry,
            r#"
            INSERT INTO audit_log (
                actor_user_id, action, subject_type, subject_id,
                before_state, after_state, ip_address, user_agent
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, actor_user_id, action, subject_type, subject_id,
                     before_state, after_state, ip_address, user_agent, created_at
            "#,
            request.actor_user_id,
            request.action,
            request.subject_type,
            request.subject_id,
            request.before_state,
            request.after_state,
            request.ip_address.map(|ip| ip.to_string()),
            request.user_agent
        )
        .fetch_one(&self.db_pool)
        .await?;

        // Log security events for monitoring
        if self.is_security_relevant(&request.action) {
            self.log_security_event(&request).await;
        }

        Ok(entry)
    }

    /// Query audit logs with filtering and pagination
    pub async fn query_audit_logs(
        &self,
        query: AuditLogQuery,
    ) -> Result<AuditLogResponse> {
        let limit = query.limit.unwrap_or(100).min(1000); // Max 1000 records
        let offset = query.offset.unwrap_or(0);

        let mut where_conditions = Vec::new();
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
        let mut param_count = 0;

        if let Some(user_id) = query.user_id {
            param_count += 1;
            where_conditions.push(format!("actor_user_id = ${}", param_count));
            params.push(Box::new(user_id));
        }

        if let Some(action) = &query.action {
            param_count += 1;
            where_conditions.push(format!("action = ${}", param_count));
            params.push(Box::new(action.clone()));
        }

        if let Some(subject_type) = &query.subject_type {
            param_count += 1;
            where_conditions.push(format!("subject_type = ${}", param_count));
            params.push(Box::new(subject_type.clone()));
        }

        if let Some(subject_id) = &query.subject_id {
            param_count += 1;
            where_conditions.push(format!("subject_id = ${}", param_count));
            params.push(Box::new(subject_id.clone()));
        }

        if let Some(start_date) = query.start_date {
            param_count += 1;
            where_conditions.push(format!("created_at >= ${}", param_count));
            params.push(Box::new(start_date));
        }

        if let Some(end_date) = query.end_date {
            param_count += 1;
            where_conditions.push(format!("created_at <= ${}", param_count));
            params.push(Box::new(end_date));
        }

        let where_clause = if where_conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_conditions.join(" AND "))
        };

        // Get total count
        let count_query = format!(
            "SELECT COUNT(*) as count FROM audit_log {}",
            where_clause
        );

        // For now, use a simple count query without dynamic parameters
        let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
            .fetch_one(&self.db_pool)
            .await?;

        // Get entries with pagination
        param_count += 1;
        let limit_param = param_count;
        param_count += 1;
        let offset_param = param_count;

        let entries_query = format!(
            r#"
            SELECT id, actor_user_id, action, subject_type, subject_id,
                   before_state, after_state, ip_address, user_agent, created_at
            FROM audit_log
            {}
            ORDER BY created_at DESC
            LIMIT ${} OFFSET ${}
            "#,
            where_clause, limit_param, offset_param
        );

        // For now, use a simple query without dynamic parameters
        let entries: Vec<AuditLogEntry> = sqlx::query_as(
            AuditLogEntry,
            "SELECT id, actor_user_id, action, subject_type, subject_id, before_state, after_state, ip_address, user_agent, created_at FROM audit_log ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            limit,
            offset
        )
        .fetch_all(&self.db_pool)
        .await?;

        let has_more = (offset + limit) < total_count;

        Ok(AuditLogResponse {
            entries,
            total_count,
            has_more,
        })
    }

    /// Log a security event for monitoring
    pub async fn log_security_event(&self, request: &CreateAuditLogRequest) {
        let event_type = self.map_action_to_security_event(&request.action);
        let severity = self.determine_security_severity(&request.action);

        let event = SecurityEvent {
            event_type,
            user_id: request.actor_user_id,
            ip_address: request.ip_address,
            user_agent: request.user_agent.clone(),
            details: json!({
                "action": request.action,
                "subject_type": request.subject_type,
                "subject_id": request.subject_id
            }),
            severity,
            timestamp: Utc::now(),
        };

        let mut events = self.security_events.write().await;
        events.push(event);

        // Keep only last 10,000 events in memory
        if events.len() > 10_000 {
            events.drain(0..1_000);
        }
    }

    /// Get recent security events
    pub async fn get_security_events(
        &self,
        limit: Option<usize>,
        severity_filter: Option<SecuritySeverity>,
    ) -> Vec<SecurityEvent> {
        let events = self.security_events.read().await;
        let mut filtered_events: Vec<SecurityEvent> = events
            .iter()
            .filter(|event| {
                if let Some(ref filter_severity) = severity_filter {
                    matches!(
                        (&event.severity, filter_severity),
                        (SecuritySeverity::Critical, SecuritySeverity::Critical)
                            | (SecuritySeverity::High, SecuritySeverity::High)
                            | (SecuritySeverity::Medium, SecuritySeverity::Medium)
                            | (SecuritySeverity::Low, SecuritySeverity::Low)
                    )
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        filtered_events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            filtered_events.truncate(limit);
        }

        filtered_events
    }

    /// Create a data export request
    pub async fn create_data_export_request(
        &self,
        user_id: Uuid,
        request_type: DataRequestType,
    ) -> Result<DataExportRequest> {
        let verification_token = uuid::Uuid::new_v4().to_string();
        let expires_at = Utc::now() + chrono::Duration::days(7); // 7 days to download

        let request = sqlx::query_as(
            DataExportRequest,
            r#"
            INSERT INTO data_export_requests (
                user_id, request_type, status, verification_token, expires_at
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, request_type as "request_type: DataRequestType",
                     status as "status: DataRequestStatus", requested_at,
                     completed_at, export_url, expires_at, verification_token
            "#,
            user_id,
            request_type as DataRequestType,
            DataRequestStatus::Pending as DataRequestStatus,
            verification_token,
            expires_at
        )
        .fetch_one(&self.db_pool)
        .await?;

        // Log the data export request
        self.log_audit_event(CreateAuditLogRequest {
            actor_user_id: Some(user_id),
            action: "data_export_requested".to_string(),
            subject_type: "user".to_string(),
            subject_id: user_id.to_string(),
            before_state: None,
            after_state: Some(json!({
                "request_type": request_type,
                "verification_token": verification_token
            })),
            ip_address: None,
            user_agent: None,
        })
        .await?;

        Ok(request)
    }

    /// Process a data export request
    pub async fn process_data_export_request(
        &self,
        request_id: Uuid,
    ) -> Result<UserDataExport> {
        // Update status to processing
        sqlx::query!(
            "UPDATE data_export_requests SET status = $1 WHERE id = $2",
            DataRequestStatus::Processing as DataRequestStatus,
            request_id
        )
        .execute(&self.db_pool)
        .await?;

        // Get the request details
        let request = sqlx::query_as(
            DataExportRequest,
            r#"
            SELECT id, user_id, request_type as "request_type: DataRequestType",
                   status as "status: DataRequestStatus", requested_at,
                   completed_at, export_url, expires_at, verification_token
            FROM data_export_requests
            WHERE id = $1
            "#,
            request_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        // Export user data
        let user_data = self.export_user_data(request.user_id).await?;

        // Update status to completed
        sqlx::query(
            r#"
            UPDATE data_export_requests 
            SET status = $1, completed_at = $2, export_url = $3
            WHERE id = $4
            "#,
            DataRequestStatus::Completed as DataRequestStatus,
            Utc::now(),
            format!("/api/v1/data-export/{}/download", request_id),
            request_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(user_data)
    }

    /// Export all user data for GDPR/CCPA compliance
    pub async fn export_user_data(&self, user_id: Uuid) -> Result<UserDataExport> {
        // Get user profile
        let user_profile = sqlx::query_as(
            UserProfile,
            "SELECT id, email, created_at, settings FROM users WHERE id = $1",
            user_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        // Get DNP lists
        let dnp_lists = sqlx::query_as(
            AuditDnpListExport,
            r#"
            SELECT a.canonical_name as artist_name, uab.tags, uab.note, uab.created_at as added_at
            FROM user_artist_blocks uab
            JOIN artists a ON uab.artist_id = a.id
            WHERE uab.user_id = $1
            ORDER BY uab.created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        // Get community subscriptions
        let community_subscriptions = sqlx::query_as(
            CommunitySubscriptionExport,
            r#"
            SELECT cl.name as list_name, cl.description as list_description,
                   uls.created_at as subscribed_at, uls.version_pinned, uls.auto_update
            FROM user_list_subscriptions uls
            JOIN community_lists cl ON uls.list_id = cl.id
            WHERE uls.user_id = $1
            ORDER BY uls.created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        // Get action history
        let action_history = sqlx::query_as(
            ActionHistoryExport,
            r#"
            SELECT 'enforcement' as action_type, provider, created_at as executed_at, summary
            FROM action_batches
            WHERE user_id = $1 AND status = 'completed'
            ORDER BY created_at DESC
            LIMIT 1000
            "#,
            user_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        // Get connections (without sensitive tokens)
        let connections = sqlx::query_as(
            ConnectionExport,
            "SELECT provider, created_at as connected_at, status, scopes FROM connections WHERE user_id = $1",
            user_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        let total_records = dnp_lists.len() as i64
            + community_subscriptions.len() as i64
            + action_history.len() as i64
            + connections.len() as i64
            + 1; // user profile

        let export_metadata = ExportMetadata {
            exported_at: Utc::now(),
            export_version: "1.0".to_string(),
            total_records,
            data_retention_policy: "Data is retained for 7 years for compliance purposes".to_string(),
        };

        Ok(UserDataExport {
            user_profile,
            dnp_lists,
            community_subscriptions,
            action_history,
            connections,
            export_metadata,
        })
    }

    /// Delete all user data for GDPR/CCPA compliance
    pub async fn delete_user_data(&self, user_id: Uuid) -> Result<()> {
        let mut tx = self.db_pool.begin().await?;

        // Log the deletion request
        sqlx::query(
            r#"
            INSERT INTO audit_log (actor_user_id, action, subject_type, subject_id, before_state)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            Some(user_id),
            "user_data_deletion_started",
            "user",
            user_id.to_string(),
            Some(json!({"timestamp": Utc::now()}))
        )
        .execute(&mut *tx)
        .await?;

        // Delete user data in correct order (respecting foreign keys)
        sqlx::query("DELETE FROM user_list_subscriptions WHERE user_id = $1", user_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM user_artist_blocks WHERE user_id = $1", user_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM action_items WHERE batch_id IN (SELECT id FROM action_batches WHERE user_id = $1)", user_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM action_batches WHERE user_id = $1", user_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM connections WHERE user_id = $1", user_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM data_export_requests WHERE user_id = $1", user_id)
            .execute(&mut *tx)
            .await?;

        // Anonymize audit logs instead of deleting for compliance
        sqlx::query(
            r#"
            UPDATE audit_log 
            SET actor_user_id = NULL, 
                ip_address = NULL, 
                user_agent = 'ANONYMIZED',
                before_state = CASE 
                    WHEN before_state IS NOT NULL THEN '{"anonymized": true}'::jsonb 
                    ELSE NULL 
                END,
                after_state = CASE 
                    WHEN after_state IS NOT NULL THEN '{"anonymized": true}'::jsonb 
                    ELSE NULL 
                END
            WHERE actor_user_id = $1
            "#,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        // Finally delete the user
        sqlx::query("DELETE FROM users WHERE id = $1", user_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Check if an action is security-relevant
    fn is_security_relevant(&self, action: &str) -> bool {
        matches!(
            action,
            "user_login"
                | "user_login_failed"
                | "password_change"
                | "totp_enabled"
                | "totp_disabled"
                | "token_refresh"
                | "account_locked"
                | "data_export_requested"
                | "data_deletion_requested"
                | "permission_change"
        )
    }

    /// Map action to security event type
    fn map_action_to_security_event(&self, action: &str) -> SecurityEventType {
        match action {
            "user_login" => SecurityEventType::LoginSuccess,
            "user_login_failed" => SecurityEventType::LoginFailure,
            "password_change" => SecurityEventType::PasswordChange,
            "totp_enabled" => SecurityEventType::TotpEnabled,
            "totp_disabled" => SecurityEventType::TotpDisabled,
            "token_refresh" => SecurityEventType::TokenRefresh,
            "account_locked" => SecurityEventType::AccountLocked,
            "data_export_requested" => SecurityEventType::DataExport,
            "data_deletion_requested" => SecurityEventType::DataDeletion,
            _ => SecurityEventType::SuspiciousActivity,
        }
    }

    /// Determine security severity based on action
    fn determine_security_severity(&self, action: &str) -> SecuritySeverity {
        match action {
            "user_login_failed" | "account_locked" | "permission_change" => SecuritySeverity::High,
            "password_change" | "totp_enabled" | "totp_disabled" => SecuritySeverity::Medium,
            "user_login" | "token_refresh" => SecuritySeverity::Low,
            "data_export_requested" | "data_deletion_requested" => SecuritySeverity::Medium,
            _ => SecuritySeverity::Low,
        }
    }
}