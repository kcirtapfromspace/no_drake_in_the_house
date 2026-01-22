use crate::error::{AppError, Result};
use crate::models::UserSettings;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

// Re-export the model UserProfile
pub use crate::models::UserProfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserProfileRequest {
    pub email: Option<String>,
    pub settings: Option<UserSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDataExport {
    pub profile: UserProfile,
    pub dnp_lists: Vec<DnpListExport>,
    pub community_list_subscriptions: Vec<CommunityListSubscriptionExport>,
    pub audit_log: Vec<AuditLogExport>,
    pub exported_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnpListExport {
    pub artist_name: String,
    pub tags: Option<Vec<String>>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityListSubscriptionExport {
    pub list_name: String,
    pub list_description: Option<String>,
    pub subscribed_at: DateTime<Utc>,
    pub auto_update: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogExport {
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDeletionRequest {
    pub confirmation_email: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDeletionResult {
    pub deleted_at: DateTime<Utc>,
    pub data_retention_days: u32,
    pub cleanup_summary: HashMap<String, u32>,
}

pub struct UserService {
    db_pool: PgPool,
}

impl UserService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Get user profile by ID
    pub async fn get_profile(&self, user_id: Uuid) -> Result<UserProfile> {
        // Use dynamic query to avoid SQLx offline mode issues
        let user: (
            Uuid,
            String,
            Option<bool>,
            Option<bool>,
            Option<DateTime<Utc>>,
            Option<DateTime<Utc>>,
            Option<DateTime<Utc>>,
            Option<serde_json::Value>,
        ) = sqlx::query_as(
            r#"
                SELECT
                    id,
                    email,
                    email_verified,
                    totp_enabled,
                    last_login,
                    created_at,
                    updated_at,
                    settings
                FROM users
                WHERE id = $1
                "#,
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(AppError::DatabaseQueryFailed)?
        .ok_or_else(|| AppError::NotFound {
            resource: "User not found".to_string(),
        })?;

        let (
            id,
            email,
            email_verified,
            totp_enabled,
            last_login,
            created_at,
            updated_at,
            settings_json,
        ) = user;

        // Parse settings from JSONB
        let settings: UserSettings =
            serde_json::from_value(settings_json.unwrap_or_else(|| serde_json::json!({})))
                .unwrap_or_default();

        // Fetch OAuth accounts from database
        let oauth_accounts = self.get_oauth_accounts(user_id).await?;

        Ok(UserProfile {
            id,
            email,
            email_verified: email_verified.unwrap_or(false),
            totp_enabled: totp_enabled.unwrap_or(false),
            oauth_accounts,
            created_at: created_at.unwrap_or_else(Utc::now),
            updated_at: updated_at.unwrap_or_else(Utc::now),
            last_login,
            settings,
        })
    }

    /// Get OAuth accounts for a user
    async fn get_oauth_accounts(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<crate::models::OAuthAccountInfo>> {
        use crate::models::oauth::OAuthProviderType;
        use std::str::FromStr;

        // Use dynamic query to avoid SQLx offline mode issues
        let accounts: Vec<(
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<DateTime<Utc>>,
            Option<DateTime<Utc>>,
        )> = sqlx::query_as(
            r#"
                SELECT
                    provider,
                    provider_user_id,
                    email,
                    display_name,
                    avatar_url,
                    created_at,
                    last_used_at
                FROM oauth_accounts
                WHERE user_id = $1
                ORDER BY created_at DESC
                "#,
        )
        .bind(user_id)
        .fetch_all(&self.db_pool)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

        Ok(accounts
            .into_iter()
            .map(
                |(
                    provider,
                    provider_user_id,
                    email,
                    display_name,
                    avatar_url,
                    created_at,
                    last_used_at,
                )| {
                    crate::models::OAuthAccountInfo {
                        provider: OAuthProviderType::from_str(&provider)
                            .unwrap_or(OAuthProviderType::Google),
                        provider_user_id,
                        email,
                        display_name,
                        avatar_url,
                        connected_at: created_at.unwrap_or_else(Utc::now),
                        last_used_at,
                    }
                },
            )
            .collect())
    }

    /// Update user profile
    pub async fn update_profile(
        &self,
        user_id: Uuid,
        request: UpdateUserProfileRequest,
    ) -> Result<UserProfile> {
        let mut tx = self
            .db_pool
            .begin()
            .await
            .map_err(AppError::DatabaseQueryFailed)?;

        // Update email if provided
        if let Some(email) = &request.email {
            sqlx::query!(
                "UPDATE users SET email = $1, updated_at = NOW() WHERE id = $2",
                email,
                user_id
            )
            .execute(&mut *tx)
            .await
            .map_err(AppError::DatabaseQueryFailed)?;
        }

        // Update settings if provided
        if let Some(settings) = &request.settings {
            let settings_json = serde_json::to_value(settings).map_err(|e| {
                AppError::InvalidRequestFormat(format!("Invalid settings format: {}", e))
            })?;

            sqlx::query!(
                "UPDATE users SET settings = $1, updated_at = NOW() WHERE id = $2",
                settings_json,
                user_id
            )
            .execute(&mut *tx)
            .await
            .map_err(AppError::DatabaseQueryFailed)?;
        }

        tx.commit().await.map_err(AppError::DatabaseQueryFailed)?;

        // Return updated profile
        self.get_profile(user_id).await
    }

    /// Export all user data for GDPR compliance
    pub async fn export_user_data(&self, user_id: Uuid) -> Result<UserDataExport> {
        // Get user profile
        let profile = self.get_profile(user_id).await?;

        // Get DNP list data
        let dnp_lists = self.export_dnp_data(user_id).await?;

        // Get community list subscriptions
        let community_subscriptions = self.export_community_subscriptions(user_id).await?;

        // Get audit log data (limited to user's own actions)
        let audit_log = self.export_audit_data(user_id).await?;

        Ok(UserDataExport {
            profile,
            dnp_lists,
            community_list_subscriptions: community_subscriptions,
            audit_log,
            exported_at: Utc::now(),
        })
    }

    /// Delete user account and all associated data
    pub async fn delete_account(
        &self,
        user_id: Uuid,
        request: AccountDeletionRequest,
    ) -> Result<AccountDeletionResult> {
        // Verify the confirmation email matches the user's email
        let user = sqlx::query!("SELECT email FROM users WHERE id = $1", user_id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(AppError::DatabaseQueryFailed)?
            .ok_or_else(|| AppError::NotFound {
                resource: "User not found".to_string(),
            })?;

        if user.email != request.confirmation_email {
            return Err(AppError::InvalidRequestFormat(
                "Email confirmation does not match".to_string(),
            ));
        }

        let mut tx = self
            .db_pool
            .begin()
            .await
            .map_err(AppError::DatabaseQueryFailed)?;

        let mut cleanup_summary = HashMap::new();

        // Delete user's DNP list entries
        let dnp_deleted =
            sqlx::query!("DELETE FROM user_artist_blocks WHERE user_id = $1", user_id)
                .execute(&mut *tx)
                .await
                .map_err(AppError::DatabaseQueryFailed)?
                .rows_affected();
        cleanup_summary.insert("dnp_entries".to_string(), dnp_deleted as u32);

        // Delete community list subscriptions
        let subscriptions_deleted = sqlx::query!(
            "DELETE FROM user_list_subscriptions WHERE user_id = $1",
            user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::DatabaseQueryFailed)?
        .rows_affected();
        cleanup_summary.insert(
            "community_subscriptions".to_string(),
            subscriptions_deleted as u32,
        );

        // Delete action batches and items
        let actions_deleted =
            sqlx::query!("DELETE FROM action_batches WHERE user_id = $1", user_id)
                .execute(&mut *tx)
                .await
                .map_err(AppError::DatabaseQueryFailed)?
                .rows_affected();
        cleanup_summary.insert("action_batches".to_string(), actions_deleted as u32);

        // Delete service connections
        let connections_deleted =
            sqlx::query!("DELETE FROM connections WHERE user_id = $1", user_id)
                .execute(&mut *tx)
                .await
                .map_err(AppError::DatabaseQueryFailed)?
                .rows_affected();
        cleanup_summary.insert(
            "service_connections".to_string(),
            connections_deleted as u32,
        );

        // Update community lists owned by user to be orphaned or transfer ownership
        let owned_lists_updated = sqlx::query!(
            "UPDATE community_lists SET owner_user_id = NULL WHERE owner_user_id = $1",
            user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::DatabaseQueryFailed)?
        .rows_affected();
        cleanup_summary.insert(
            "owned_lists_orphaned".to_string(),
            owned_lists_updated as u32,
        );

        // Anonymize audit log entries (keep for compliance but remove PII)
        let audit_anonymized = sqlx::query!(
            r#"
            UPDATE audit_log 
            SET user_id = NULL, 
                ip_address = NULL, 
                user_agent = 'ANONYMIZED'
            WHERE user_id = $1
            "#,
            user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::DatabaseQueryFailed)?
        .rows_affected();
        cleanup_summary.insert(
            "audit_entries_anonymized".to_string(),
            audit_anonymized as u32,
        );

        // Finally, delete the user account
        sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
            .execute(&mut *tx)
            .await
            .map_err(AppError::DatabaseQueryFailed)?;

        tx.commit().await.map_err(AppError::DatabaseQueryFailed)?;

        Ok(AccountDeletionResult {
            deleted_at: Utc::now(),
            data_retention_days: 90, // Audit logs retained for 90 days for compliance
            cleanup_summary,
        })
    }

    /// Export DNP list data for user
    async fn export_dnp_data(&self, user_id: Uuid) -> Result<Vec<DnpListExport>> {
        let entries = sqlx::query!(
            r#"
            SELECT 
                a.canonical_name,
                b.tags,
                b.note,
                b.created_at
            FROM user_artist_blocks b
            JOIN artists a ON b.artist_id = a.id
            WHERE b.user_id = $1
            ORDER BY b.created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

        Ok(entries
            .into_iter()
            .map(|row| DnpListExport {
                artist_name: row.canonical_name,
                tags: row.tags,
                note: row.note,
                created_at: row.created_at.unwrap_or_else(|| Utc::now()),
            })
            .collect())
    }

    /// Export community list subscriptions for user
    async fn export_community_subscriptions(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<CommunityListSubscriptionExport>> {
        let subscriptions = sqlx::query!(
            r#"
            SELECT 
                cl.name,
                cl.description,
                s.created_at,
                s.auto_update
            FROM user_list_subscriptions s
            JOIN community_lists cl ON s.list_id = cl.id
            WHERE s.user_id = $1
            ORDER BY s.created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

        Ok(subscriptions
            .into_iter()
            .map(|row| CommunityListSubscriptionExport {
                list_name: row.name,
                list_description: row.description,
                subscribed_at: row.created_at.unwrap_or_else(|| Utc::now()),
                auto_update: row.auto_update.unwrap_or(true),
            })
            .collect())
    }

    /// Export audit log data for user (limited to their own actions)
    async fn export_audit_data(&self, user_id: Uuid) -> Result<Vec<AuditLogExport>> {
        let audit_entries = sqlx::query!(
            r#"
            SELECT 
                action,
                old_subject_type,
                timestamp
            FROM audit_log
            WHERE user_id = $1
            ORDER BY timestamp DESC
            LIMIT 1000
            "#,
            user_id
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

        Ok(audit_entries
            .into_iter()
            .map(|row| AuditLogExport {
                event_type: row.action,
                timestamp: row.timestamp.unwrap_or_else(|| Utc::now()),
                details: row.old_subject_type,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_profile_creation() {
        let profile = UserProfile {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            email_verified: false,
            totp_enabled: false,
            oauth_accounts: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: None,
            settings: UserSettings::default(),
        };

        assert_eq!(profile.email, "test@example.com");
        assert!(!profile.totp_enabled);
    }

    #[test]
    fn test_update_request_creation() {
        let request = UpdateUserProfileRequest {
            email: Some("new@example.com".to_string()),
            settings: Some(UserSettings {
                two_factor_enabled: true,
                email_notifications: false,
                privacy_mode: true,
            }),
        };

        assert!(request.email.is_some());
        assert!(request.settings.is_some());
    }
}
