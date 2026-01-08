use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::net::IpAddr;
use uuid::Uuid;

/// Audit log entry for SOC2 compliance
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub actor_user_id: Option<Uuid>,
    pub action: String,
    pub subject_type: String,
    pub subject_id: String,
    pub before_state: Option<serde_json::Value>,
    pub after_state: Option<serde_json::Value>,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Request to create an audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuditLogRequest {
    pub actor_user_id: Option<Uuid>,
    pub action: String,
    pub subject_type: String,
    pub subject_id: String,
    pub before_state: Option<serde_json::Value>,
    pub after_state: Option<serde_json::Value>,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
}

/// Audit log query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogQuery {
    pub user_id: Option<Uuid>,
    pub action: Option<String>,
    pub subject_type: Option<String>,
    pub subject_id: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Audit log response with pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogResponse {
    pub entries: Vec<AuditLogEntry>,
    pub total_count: i64,
    pub has_more: bool,
}

/// Security event types for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    LoginSuccess,
    LoginFailure,
    PasswordChange,
    TotpEnabled,
    TotpDisabled,
    TokenRefresh,
    AccountLocked,
    SuspiciousActivity,
    DataExport,
    DataDeletion,
    PermissionEscalation,
}

/// Security monitoring event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub event_type: SecurityEventType,
    pub user_id: Option<Uuid>,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub details: serde_json::Value,
    pub severity: SecuritySeverity,
    pub timestamp: DateTime<Utc>,
}

/// Security event severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Access review entry for SOC2 compliance
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccessReviewEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub reviewer_id: Option<Uuid>,
    pub access_level: String,
    pub permissions: Vec<String>,
    pub last_login: Option<DateTime<Utc>>,
    pub status: AccessStatus,
    pub review_date: DateTime<Utc>,
    pub next_review_date: DateTime<Utc>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Access status for users
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "access_status", rename_all = "lowercase")]
pub enum AccessStatus {
    Active,
    Inactive,
    Suspended,
    PendingReview,
}

/// GDPR/CCPA data export request
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DataExportRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub request_type: DataRequestType,
    pub status: DataRequestStatus,
    pub requested_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub export_url: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub verification_token: Option<String>,
}

/// Data request types
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "data_request_type", rename_all = "lowercase")]
pub enum DataRequestType {
    Export,
    Deletion,
}

/// Data request status
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "data_request_status", rename_all = "lowercase")]
pub enum DataRequestStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Expired,
}

/// User data export structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDataExport {
    pub user_profile: UserProfile,
    pub dnp_lists: Vec<AuditDnpListExport>,
    pub community_subscriptions: Vec<CommunitySubscriptionExport>,
    pub action_history: Vec<ActionHistoryExport>,
    pub connections: Vec<ConnectionExport>,
    pub export_metadata: ExportMetadata,
}

/// User profile for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub settings: serde_json::Value,
}

/// DNP list export structure for audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditDnpListExport {
    pub artist_name: String,
    pub tags: Vec<String>,
    pub note: Option<String>,
    pub added_at: DateTime<Utc>,
}

/// Community subscription export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunitySubscriptionExport {
    pub list_name: String,
    pub list_description: Option<String>,
    pub subscribed_at: DateTime<Utc>,
    pub version_pinned: Option<i32>,
    pub auto_update: bool,
}

/// Action history export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionHistoryExport {
    pub action_type: String,
    pub provider: String,
    pub executed_at: DateTime<Utc>,
    pub summary: serde_json::Value,
}

/// Connection export (without sensitive tokens)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionExport {
    pub provider: String,
    pub connected_at: DateTime<Utc>,
    pub status: String,
    pub scopes: Vec<String>,
}

/// Export metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub exported_at: DateTime<Utc>,
    pub export_version: String,
    pub total_records: i64,
    pub data_retention_policy: String,
}
