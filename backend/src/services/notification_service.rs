//! Notification Service (US-027)
//!
//! Handles sending in-app notifications to users for various events,
//! including connection status changes and enforcement issues.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::notification::{Notification, NotificationType};
use crate::models::token_vault::StreamingProvider;

/// Service for managing user notifications
pub struct NotificationService {
    db_pool: Option<PgPool>,
}

impl NotificationService {
    /// Create a new notification service with database persistence
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool: Some(db_pool),
        }
    }

    /// Create a notification service without database (for testing)
    pub fn new_in_memory() -> Self {
        Self { db_pool: None }
    }

    /// Send a notification when a connection needs re-authentication
    ///
    /// This is triggered after token refresh failures exceed max retries (US-027).
    pub async fn notify_connection_needs_reauth(
        &self,
        user_id: Uuid,
        provider: &StreamingProvider,
        reason: &str,
    ) -> Result<Notification> {
        let provider_name = provider.to_string();
        let provider_display = match provider {
            StreamingProvider::Spotify => "Spotify",
            StreamingProvider::Apple => "Apple",
            StreamingProvider::AppleMusic => "Apple Music",
            StreamingProvider::YouTubeMusic => "YouTube Music",
            StreamingProvider::Tidal => "Tidal",
        };

        let notification = Notification::new(
            user_id,
            NotificationType::ConnectionNeedsReauth,
            format!("{} Connection Needs Attention", provider_display),
            format!(
                "Your {} connection needs to be re-authenticated. Please reconnect your account to continue using enforcement features. Reason: {}",
                provider_display, reason
            ),
            Some(json!({
                "provider": provider_name,
                "reason": reason,
                "action_required": "reconnect",
                "reconnect_url": format!("/settings/connections/{}/reconnect", provider_name),
            })),
        );

        self.save_notification(&notification).await?;

        tracing::info!(
            user_id = %user_id,
            provider = %provider_name,
            reason = %reason,
            "Sent ConnectionNeedsReauth notification"
        );

        Ok(notification)
    }

    /// Send a notification when enforcement is skipped due to connection issues
    ///
    /// This notifies users why their enforcement job wasn't executed (US-027).
    pub async fn notify_enforcement_skipped(
        &self,
        user_id: Uuid,
        provider: &StreamingProvider,
        reason: &str,
    ) -> Result<Notification> {
        let provider_name = provider.to_string();
        let provider_display = match provider {
            StreamingProvider::Spotify => "Spotify",
            StreamingProvider::Apple => "Apple",
            StreamingProvider::AppleMusic => "Apple Music",
            StreamingProvider::YouTubeMusic => "YouTube Music",
            StreamingProvider::Tidal => "Tidal",
        };

        let notification = Notification::new(
            user_id,
            NotificationType::EnforcementSkipped,
            format!("{} Enforcement Skipped", provider_display),
            format!(
                "Your {} enforcement was skipped because your connection needs attention. {}. Please reconnect your account to resume enforcement.",
                provider_display, reason
            ),
            Some(json!({
                "provider": provider_name,
                "reason": reason,
                "action_required": "reconnect",
                "reconnect_url": format!("/settings/connections/{}/reconnect", provider_name),
            })),
        );

        self.save_notification(&notification).await?;

        tracing::info!(
            user_id = %user_id,
            provider = %provider_name,
            reason = %reason,
            "Sent EnforcementSkipped notification"
        );

        Ok(notification)
    }

    /// Save a notification to the database
    async fn save_notification(&self, notification: &Notification) -> Result<()> {
        if let Some(pool) = &self.db_pool {
            sqlx::query(
                r#"
                INSERT INTO notifications (
                    id, user_id, notification_type, title, message,
                    data, read, created_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(notification.id)
            .bind(notification.user_id)
            .bind(serde_json::to_string(&notification.notification_type)?)
            .bind(&notification.title)
            .bind(&notification.message)
            .bind(&notification.data)
            .bind(notification.read)
            .bind(notification.created_at)
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    /// Get unread notifications for a user
    pub async fn get_unread_notifications(&self, user_id: Uuid) -> Result<Vec<Notification>> {
        if let Some(pool) = &self.db_pool {
            let rows = sqlx::query_as::<_, NotificationRow>(
                r#"
                SELECT id, user_id, notification_type, title, message, data, read, created_at, read_at
                FROM notifications
                WHERE user_id = $1 AND read = false
                ORDER BY created_at DESC
                LIMIT 50
                "#,
            )
            .bind(user_id)
            .fetch_all(pool)
            .await?;

            Ok(rows.into_iter().map(|r| r.into()).collect())
        } else {
            Ok(Vec::new())
        }
    }

    /// Mark a notification as read
    pub async fn mark_as_read(&self, notification_id: Uuid) -> Result<()> {
        if let Some(pool) = &self.db_pool {
            sqlx::query(
                r#"
                UPDATE notifications
                SET read = true, read_at = $1
                WHERE id = $2
                "#,
            )
            .bind(Utc::now())
            .bind(notification_id)
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    /// Get count of unread notifications by type for a user
    pub async fn get_unread_count_by_type(
        &self,
        user_id: Uuid,
        notification_type: &NotificationType,
    ) -> Result<i64> {
        if let Some(pool) = &self.db_pool {
            let type_str = serde_json::to_string(notification_type)?;
            let row: (i64,) = sqlx::query_as(
                r#"
                SELECT COUNT(*)
                FROM notifications
                WHERE user_id = $1 AND notification_type = $2 AND read = false
                "#,
            )
            .bind(user_id)
            .bind(type_str)
            .fetch_one(pool)
            .await?;

            Ok(row.0)
        } else {
            Ok(0)
        }
    }
}

/// Database row for notifications
#[derive(Debug, sqlx::FromRow)]
struct NotificationRow {
    id: Uuid,
    user_id: Uuid,
    notification_type: String,
    title: String,
    message: String,
    data: Option<serde_json::Value>,
    read: bool,
    created_at: DateTime<Utc>,
    read_at: Option<DateTime<Utc>>,
}

impl From<NotificationRow> for Notification {
    fn from(row: NotificationRow) -> Self {
        let notification_type: NotificationType = serde_json::from_str(&row.notification_type)
            .unwrap_or(NotificationType::SystemMaintenance);

        Notification {
            id: row.id,
            user_id: row.user_id,
            notification_type,
            title: row.title,
            message: row.message,
            data: row.data,
            read: row.read,
            created_at: row.created_at,
            read_at: row.read_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_notification_service_in_memory() {
        let service = NotificationService::new_in_memory();
        let user_id = Uuid::new_v4();

        let notification = service
            .notify_connection_needs_reauth(
                user_id,
                &StreamingProvider::Spotify,
                "Token refresh failed after max retries",
            )
            .await
            .unwrap();

        assert_eq!(notification.user_id, user_id);
        assert_eq!(
            notification.notification_type,
            NotificationType::ConnectionNeedsReauth
        );
        assert!(notification.title.contains("Spotify"));
        assert!(notification.message.contains("re-authenticated"));
    }

    #[tokio::test]
    async fn test_enforcement_skipped_notification() {
        let service = NotificationService::new_in_memory();
        let user_id = Uuid::new_v4();

        let notification = service
            .notify_enforcement_skipped(
                user_id,
                &StreamingProvider::AppleMusic,
                "Connection status is NeedsReauth",
            )
            .await
            .unwrap();

        assert_eq!(notification.user_id, user_id);
        assert_eq!(
            notification.notification_type,
            NotificationType::EnforcementSkipped
        );
        assert!(notification.title.contains("Apple Music"));
        assert!(notification.message.contains("skipped"));
    }
}
