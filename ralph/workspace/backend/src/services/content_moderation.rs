use crate::models::*;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

/// Content moderation service for community lists
#[derive(Clone)]
pub struct ContentModerationService {
    db_pool: PgPool,
    prohibited_patterns: Vec<regex::Regex>,
    audit_service: std::sync::Arc<AuditService>,
}

/// Content moderation result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModerationResult {
    pub is_approved: bool,
    pub violations: Vec<ContentViolation>,
    pub suggested_changes: Vec<String>,
    pub confidence_score: f64,
}

/// Content violation details
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContentViolation {
    pub violation_type: ViolationType,
    pub description: String,
    pub severity: ViolationSeverity,
    pub matched_text: Option<String>,
    pub suggested_replacement: Option<String>,
}

/// Types of content violations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ViolationType {
    PersonalAttack,
    UnsubstantiatedClaim,
    OffensiveLanguage,
    NonNeutralLanguage,
    MissingEvidence,
    PolicyViolation,
}

/// Severity levels for violations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Moderation queue entry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct ModerationQueueEntry {
    pub id: Uuid,
    pub list_id: Uuid,
    pub submitter_id: Uuid,
    pub content_type: String, // "list_creation", "list_update", "artist_addition"
    pub content_data: serde_json::Value,
    pub status: ModerationStatus,
    pub priority: ModerationPriority,
    pub assigned_moderator_id: Option<Uuid>,
    pub submitted_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_notes: Option<String>,
    pub auto_moderation_result: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Moderation status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "moderation_status", rename_all = "lowercase")]
pub enum ModerationStatus {
    Pending,
    UnderReview,
    Approved,
    Rejected,
    RequiresChanges,
}

/// Moderation priority
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "moderation_priority", rename_all = "lowercase")]
pub enum ModerationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

/// Appeal request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct AppealRequest {
    pub id: Uuid,
    pub moderation_entry_id: Uuid,
    pub appellant_id: Uuid,
    pub appeal_reason: String,
    pub additional_evidence: Option<serde_json::Value>,
    pub status: AppealStatus,
    pub assigned_reviewer_id: Option<Uuid>,
    pub submitted_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub resolution: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Appeal status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "appeal_status", rename_all = "lowercase")]
pub enum AppealStatus {
    Pending,
    UnderReview,
    Upheld,
    Denied,
    Escalated,
}

/// Moderation statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModerationStats {
    pub total_pending: i64,
    pub total_under_review: i64,
    pub total_approved_today: i64,
    pub total_rejected_today: i64,
    pub average_review_time_hours: f64,
    pub appeals_pending: i64,
    pub auto_approval_rate: f64,
}

impl ContentModerationService {
    pub fn new(db_pool: PgPool, audit_service: std::sync::Arc<AuditService>) -> Self {
        let prohibited_patterns = vec![
            // Personal attacks and character judgments
            regex::Regex::new(r"\b(evil|bad|terrible|awful|horrible)\s+(person|artist|individual)\b").unwrap(),
            regex::Regex::new(r"\b(scum|trash|garbage|waste)\b").unwrap(),
            
            // Unsubstantiated legal claims
            regex::Regex::new(r"\b(guilty|convicted|criminal|illegal|lawsuit|sued)\b").unwrap(),
            regex::Regex::new(r"\b(accused|alleged|charged)\s+with\b").unwrap(),
            
            // Offensive language
            regex::Regex::new(r"\b(hate|despise|loathe)\b").unwrap(),
            
            // Non-neutral language
            regex::Regex::new(r"\b(obviously|clearly|definitely)\s+(bad|wrong|evil)\b").unwrap(),
            regex::Regex::new(r"\b(everyone knows|it's obvious)\b").unwrap(),
        ];

        Self {
            db_pool,
            prohibited_patterns,
            audit_service,
        }
    }

    /// Automatically moderate content using rules and ML
    pub async fn moderate_content(&self, content: &str, context: &str) -> Result<ModerationResult> {
        let mut violations = Vec::new();
        let mut suggested_changes = Vec::new();
        let mut confidence_score = 1.0;

        // Check for prohibited patterns
        for pattern in &self.prohibited_patterns {
            if let Some(matched) = pattern.find(content) {
                let violation_type = self.classify_violation(matched.as_str());
                let severity = self.determine_severity(&violation_type);
                
                violations.push(ContentViolation {
                    violation_type,
                    description: format!("Prohibited pattern detected: {}", matched.as_str()),
                    severity,
                    matched_text: Some(matched.as_str().to_string()),
                    suggested_replacement: self.suggest_replacement(matched.as_str()),
                });
                
                confidence_score -= 0.2;
            }
        }

        // Check for neutral language requirements
        if self.contains_subjective_language(content) {
            violations.push(ContentViolation {
                violation_type: ViolationType::NonNeutralLanguage,
                description: "Content contains subjective or non-neutral language".to_string(),
                severity: ViolationSeverity::Medium,
                matched_text: None,
                suggested_replacement: None,
            });
            suggested_changes.push("Use neutral, factual language without personal opinions".to_string());
            confidence_score -= 0.15;
        }

        // Check for evidence requirements
        if context == "list_creation" && !self.has_governance_info(content) {
            violations.push(ContentViolation {
                violation_type: ViolationType::MissingEvidence,
                description: "Missing governance information or criteria explanation".to_string(),
                severity: ViolationSeverity::High,
                matched_text: None,
                suggested_replacement: None,
            });
            suggested_changes.push("Include clear criteria and governance process".to_string());
            confidence_score -= 0.3;
        }

        let is_approved = violations.is_empty() || violations.iter().all(|v| matches!(v.severity, ViolationSeverity::Low));

        Ok(ModerationResult {
            is_approved,
            violations,
            suggested_changes,
            confidence_score: confidence_score.max(0.0),
        })
    }

    /// Submit content for moderation
    pub async fn submit_for_moderation(
        &self,
        list_id: Uuid,
        submitter_id: Uuid,
        content_type: String,
        content_data: serde_json::Value,
    ) -> Result<ModerationQueueEntry> {
        // Run automatic moderation first
        let content_text = self.extract_text_from_content(&content_data);
        let auto_result = self.moderate_content(&content_text, &content_type).await?;
        
        let priority = if auto_result.violations.iter().any(|v| matches!(v.severity, ViolationSeverity::Critical | ViolationSeverity::High)) {
            ModerationPriority::High
        } else if auto_result.is_approved {
            ModerationPriority::Low
        } else {
            ModerationPriority::Normal
        };

        let status = if auto_result.is_approved && auto_result.confidence_score > 0.8 {
            ModerationStatus::Approved
        } else {
            ModerationStatus::Pending
        };

        let entry = sqlx::query_as(
            ModerationQueueEntry,
            r#"
            INSERT INTO moderation_queue (
                list_id, submitter_id, content_type, content_data, status, priority,
                auto_moderation_result, submitted_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, list_id, submitter_id, content_type, content_data,
                     status as "status: ModerationStatus", priority as "priority: ModerationPriority",
                     assigned_moderator_id, submitted_at, reviewed_at, review_notes,
                     auto_moderation_result, created_at, updated_at
            "#,
            list_id,
            submitter_id,
            content_type,
            content_data,
            status as ModerationStatus,
            priority as ModerationPriority,
            serde_json::to_value(&auto_result)?,
            Utc::now()
        )
        .fetch_one(&self.db_pool)
        .await?;

        // Log the moderation submission
        self.audit_service.log_audit_event(CreateAuditLogRequest {
            actor_user_id: Some(submitter_id),
            action: "content_submitted_for_moderation".to_string(),
            subject_type: "moderation_queue".to_string(),
            subject_id: entry.id.to_string(),
            before_state: None,
            after_state: Some(json!({
                "content_type": content_type,
                "auto_approved": status == ModerationStatus::Approved,
                "priority": priority,
                "confidence_score": auto_result.confidence_score
            })),
            ip_address: None,
            user_agent: None,
        }).await?;

        Ok(entry)
    }

    /// Get moderation queue entries
    pub async fn get_moderation_queue(
        &self,
        status_filter: Option<ModerationStatus>,
        priority_filter: Option<ModerationPriority>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ModerationQueueEntry>> {
        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let mut query = "SELECT * FROM moderation_queue WHERE 1=1".to_string();
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
        let mut param_count = 0;

        if let Some(status) = status_filter {
            param_count += 1;
            query.push_str(&format!(" AND status = ${}", param_count));
            params.push(Box::new(status));
        }

        if let Some(priority) = priority_filter {
            param_count += 1;
            query.push_str(&format!(" AND priority = ${}", param_count));
            params.push(Box::new(priority));
        }

        param_count += 1;
        query.push_str(&format!(" ORDER BY priority DESC, submitted_at ASC LIMIT ${}", param_count));
        params.push(Box::new(limit));

        param_count += 1;
        query.push_str(&format!(" OFFSET ${}", param_count));
        params.push(Box::new(offset));

        // For now, return a simple query without dynamic parameters
        let entries = sqlx::query_as(
            ModerationQueueEntry,
            r#"
            SELECT id, list_id, submitter_id, content_type, content_data,
                   status as "status: ModerationStatus", priority as "priority: ModerationPriority",
                   assigned_moderator_id, submitted_at, reviewed_at, review_notes,
                   auto_moderation_result, created_at, updated_at
            FROM moderation_queue
            ORDER BY priority DESC, submitted_at ASC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(entries)
    }

    /// Review and approve/reject moderation entry
    pub async fn review_content(
        &self,
        entry_id: Uuid,
        moderator_id: Uuid,
        decision: ModerationStatus,
        review_notes: Option<String>,
    ) -> Result<ModerationQueueEntry> {
        let updated_entry = sqlx::query_as(
            ModerationQueueEntry,
            r#"
            UPDATE moderation_queue
            SET status = $1, assigned_moderator_id = $2, reviewed_at = $3, 
                review_notes = $4, updated_at = $5
            WHERE id = $6
            RETURNING id, list_id, submitter_id, content_type, content_data,
                     status as "status: ModerationStatus", priority as "priority: ModerationPriority",
                     assigned_moderator_id, submitted_at, reviewed_at, review_notes,
                     auto_moderation_result, created_at, updated_at
            "#,
            decision as ModerationStatus,
            moderator_id,
            Utc::now(),
            review_notes,
            Utc::now(),
            entry_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        // Log the moderation decision
        self.audit_service.log_audit_event(CreateAuditLogRequest {
            actor_user_id: Some(moderator_id),
            action: "content_moderation_decision".to_string(),
            subject_type: "moderation_queue".to_string(),
            subject_id: entry_id.to_string(),
            before_state: None,
            after_state: Some(json!({
                "decision": decision,
                "review_notes": review_notes
            })),
            ip_address: None,
            user_agent: None,
        }).await?;

        Ok(updated_entry)
    }

    /// Submit an appeal
    pub async fn submit_appeal(
        &self,
        moderation_entry_id: Uuid,
        appellant_id: Uuid,
        appeal_reason: String,
        additional_evidence: Option<serde_json::Value>,
    ) -> Result<AppealRequest> {
        let appeal = sqlx::query_as(
            AppealRequest,
            r#"
            INSERT INTO appeals (
                moderation_entry_id, appellant_id, appeal_reason, additional_evidence,
                status, submitted_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, moderation_entry_id, appellant_id, appeal_reason, additional_evidence,
                     status as "status: AppealStatus", assigned_reviewer_id, submitted_at,
                     reviewed_at, resolution, created_at, updated_at
            "#,
            moderation_entry_id,
            appellant_id,
            appeal_reason,
            additional_evidence,
            AppealStatus::Pending as AppealStatus,
            Utc::now()
        )
        .fetch_one(&self.db_pool)
        .await?;

        // Log the appeal submission
        self.audit_service.log_audit_event(CreateAuditLogRequest {
            actor_user_id: Some(appellant_id),
            action: "appeal_submitted".to_string(),
            subject_type: "appeal".to_string(),
            subject_id: appeal.id.to_string(),
            before_state: None,
            after_state: Some(json!({
                "moderation_entry_id": moderation_entry_id,
                "appeal_reason": appeal_reason
            })),
            ip_address: None,
            user_agent: None,
        }).await?;

        Ok(appeal)
    }

    /// Get moderation statistics
    pub async fn get_moderation_stats(&self) -> Result<ModerationStats> {
        let total_pending: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM moderation_queue WHERE status = 'pending'"
        )
        .fetch_one(&self.db_pool)
        .await?;

        let total_under_review: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM moderation_queue WHERE status = 'under_review'"
        )
        .fetch_one(&self.db_pool)
        .await?;

        let total_approved_today: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM moderation_queue WHERE status = 'approved' AND DATE(reviewed_at) = CURRENT_DATE"
        )
        .fetch_one(&self.db_pool)
        .await?;

        let total_rejected_today: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM moderation_queue WHERE status = 'rejected' AND DATE(reviewed_at) = CURRENT_DATE"
        )
        .fetch_one(&self.db_pool)
        .await?;

        let appeals_pending: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM appeals WHERE status = 'pending'"
        )
        .fetch_one(&self.db_pool)
        .await?;

        // Calculate average review time and auto-approval rate
        let avg_review_time: Option<f64> = sqlx::query_scalar(
            r#"
            SELECT AVG(EXTRACT(EPOCH FROM (reviewed_at - submitted_at)) / 3600.0)
            FROM moderation_queue 
            WHERE reviewed_at IS NOT NULL AND submitted_at > NOW() - INTERVAL '30 days'
            "#
        )
        .fetch_one(&self.db_pool)
        .await?;

        let auto_approval_rate: Option<f64> = sqlx::query_scalar(
            r#"
            SELECT 
                COUNT(CASE WHEN status = 'approved' AND assigned_moderator_id IS NULL THEN 1 END)::float / 
                COUNT(*)::float * 100
            FROM moderation_queue 
            WHERE submitted_at > NOW() - INTERVAL '30 days'
            "#
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(ModerationStats {
            total_pending,
            total_under_review,
            total_approved_today,
            total_rejected_today,
            average_review_time_hours: avg_review_time.unwrap_or(0.0),
            appeals_pending,
            auto_approval_rate: auto_approval_rate.unwrap_or(0.0),
        })
    }

    // Helper methods

    fn classify_violation(&self, matched_text: &str) -> ViolationType {
        if matched_text.contains("evil") || matched_text.contains("bad") || matched_text.contains("terrible") {
            ViolationType::PersonalAttack
        } else if matched_text.contains("guilty") || matched_text.contains("criminal") {
            ViolationType::UnsubstantiatedClaim
        } else if matched_text.contains("hate") || matched_text.contains("despise") {
            ViolationType::OffensiveLanguage
        } else {
            ViolationType::NonNeutralLanguage
        }
    }

    fn determine_severity(&self, violation_type: &ViolationType) -> ViolationSeverity {
        match violation_type {
            ViolationType::PersonalAttack => ViolationSeverity::High,
            ViolationType::UnsubstantiatedClaim => ViolationSeverity::Critical,
            ViolationType::OffensiveLanguage => ViolationSeverity::High,
            ViolationType::NonNeutralLanguage => ViolationSeverity::Medium,
            ViolationType::MissingEvidence => ViolationSeverity::High,
            ViolationType::PolicyViolation => ViolationSeverity::Medium,
        }
    }

    fn suggest_replacement(&self, matched_text: &str) -> Option<String> {
        match matched_text {
            text if text.contains("evil") => Some("controversial".to_string()),
            text if text.contains("bad person") => Some("artist with concerning behavior".to_string()),
            text if text.contains("guilty") => Some("associated with".to_string()),
            _ => None,
        }
    }

    fn contains_subjective_language(&self, content: &str) -> bool {
        let subjective_patterns = [
            r"\b(I think|I believe|in my opinion|obviously|clearly)\b",
            r"\b(amazing|terrible|awesome|horrible)\b",
            r"\b(everyone knows|it's obvious|no doubt)\b",
        ];

        subjective_patterns.iter().any(|pattern| {
            regex::Regex::new(pattern).unwrap().is_match(content)
        })
    }

    fn has_governance_info(&self, content: &str) -> bool {
        let governance_keywords = [
            "criteria", "process", "review", "appeal", "governance", 
            "policy", "guidelines", "moderation", "update"
        ];

        governance_keywords.iter().any(|keyword| {
            content.to_lowercase().contains(keyword)
        })
    }

    fn extract_text_from_content(&self, content_data: &serde_json::Value) -> String {
        // Extract text from various content types
        match content_data {
            serde_json::Value::Object(obj) => {
                let mut text = String::new();
                if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
                    text.push_str(name);
                    text.push(' ');
                }
                if let Some(description) = obj.get("description").and_then(|v| v.as_str()) {
                    text.push_str(description);
                    text.push(' ');
                }
                if let Some(criteria) = obj.get("criteria").and_then(|v| v.as_str()) {
                    text.push_str(criteria);
                }
                text
            }
            serde_json::Value::String(s) => s.clone(),
            _ => content_data.to_string(),
        }
    }
}