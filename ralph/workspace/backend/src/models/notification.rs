use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Notification type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    CommunityListUpdate,
    EnforcementComplete,
    SecurityAlert,
    SystemMaintenance,
}

/// User notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub read: bool,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

impl Notification {
    pub fn new(
        user_id: Uuid,
        notification_type: NotificationType,
        title: String,
        message: String,
        data: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            notification_type,
            title,
            message,
            data,
            read: false,
            created_at: Utc::now(),
            read_at: None,
        }
    }

    pub fn mark_as_read(&mut self) {
        self.read = true;
        self.read_at = Some(Utc::now());
    }
}
