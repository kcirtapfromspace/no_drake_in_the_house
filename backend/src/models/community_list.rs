use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Community list with governance information
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CommunityList {
    pub id: Uuid,
    pub owner_user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub criteria: String, // Required neutral criteria
    pub governance_url: Option<String>, // Link to governance process
    pub update_cadence: String, // "weekly", "monthly", "as-needed"
    pub version: i32,
    pub visibility: String, // "public", "private", "unlisted"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Community list item (artist in the list)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CommunityListItem {
    pub list_id: Uuid,
    pub artist_id: Uuid,
    pub rationale_link: Option<String>,
    pub added_at: DateTime<Utc>,
}

/// User subscription to a community list
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserListSubscription {
    pub user_id: Uuid,
    pub list_id: Uuid,
    pub version_pinned: Option<i32>,
    pub auto_update: bool,
    pub created_at: DateTime<Utc>,
}

/// Request to create a community list
#[derive(Debug, Deserialize)]
pub struct CreateCommunityListRequest {
    pub name: String,
    pub description: Option<String>,
    pub criteria: String,
    pub governance_url: Option<String>,
    pub update_cadence: String,
    pub visibility: Option<String>,
}

/// Request to update a community list
#[derive(Debug, Deserialize)]
pub struct UpdateCommunityListRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub criteria: Option<String>,
    pub governance_url: Option<String>,
    pub update_cadence: Option<String>,
    pub visibility: Option<String>,
}

/// Request to add an artist to a community list
#[derive(Debug, Deserialize)]
pub struct AddArtistToCommunityListRequest {
    pub artist_query: String, // Artist name or provider URL
    pub rationale_link: Option<String>,
}

/// Request to subscribe to a community list
#[derive(Debug, Deserialize)]
pub struct SubscribeToCommunityListRequest {
    pub version_pinned: Option<i32>,
    pub auto_update: Option<bool>,
}

/// Request to update subscription settings
#[derive(Debug, Deserialize)]
pub struct UpdateSubscriptionRequest {
    pub version_pinned: Option<i32>,
    pub auto_update: Option<bool>,
}

/// Community list with detailed information
#[derive(Debug, Serialize)]
pub struct CommunityListResponse {
    pub id: Uuid,
    pub owner: UserInfo,
    pub name: String,
    pub description: Option<String>,
    pub criteria: String,
    pub governance_url: Option<String>,
    pub update_cadence: String,
    pub version: i32,
    pub visibility: String,
    pub total_artists: usize,
    pub subscriber_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_subscribed: bool, // For the requesting user
    pub subscription_details: Option<SubscriptionDetails>,
}

/// User information for community list display
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String, // May be masked for privacy
}

/// Subscription details for the requesting user
#[derive(Debug, Serialize)]
pub struct SubscriptionDetails {
    pub version_pinned: Option<i32>,
    pub auto_update: bool,
    pub subscribed_at: DateTime<Utc>,
}

/// Community list directory response
#[derive(Debug, Serialize)]
pub struct CommunityListDirectory {
    pub lists: Vec<CommunityListSummary>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
}

/// Summary information for community list browsing
#[derive(Debug, Serialize)]
pub struct CommunityListSummary {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub criteria: String,
    pub owner_email: String, // Masked for privacy
    pub total_artists: usize,
    pub subscriber_count: usize,
    pub version: i32,
    pub update_cadence: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Community list with artists for detailed view
#[derive(Debug, Serialize)]
pub struct CommunityListWithArtists {
    pub list: CommunityListResponse,
    pub artists: Vec<CommunityListArtistEntry>,
}

/// Artist entry in a community list
#[derive(Debug, Serialize)]
pub struct CommunityListArtistEntry {
    pub artist_id: Uuid,
    pub artist_name: String,
    pub image_url: Option<String>,
    pub provider_badges: Vec<ProviderBadge>,
    pub rationale_link: Option<String>,
    pub added_at: DateTime<Utc>,
}

/// Provider badge for artist identification
#[derive(Debug, Serialize)]
pub struct ProviderBadge {
    pub provider: String,
    pub verified: bool,
    pub follower_count: Option<u64>,
}

/// Subscription impact preview
#[derive(Debug, Serialize)]
pub struct SubscriptionImpactPreview {
    pub list_id: Uuid,
    pub list_name: String,
    pub version: i32,
    pub total_artists_in_list: usize,
    pub new_artists_for_user: usize,
    pub already_blocked_artists: usize,
    pub impact_by_provider: Vec<ProviderImpact>,
    pub sample_new_artists: Vec<CommunityListArtistEntry>, // First 10 new artists
}

/// Impact on a specific provider
#[derive(Debug, Serialize)]
pub struct ProviderImpact {
    pub provider: String,
    pub estimated_tracks_affected: usize,
    pub estimated_playlists_affected: usize,
    pub estimated_follows_affected: usize,
}

/// Community list update notification
#[derive(Debug, Serialize)]
pub struct CommunityListUpdateNotification {
    pub list_id: Uuid,
    pub list_name: String,
    pub old_version: i32,
    pub new_version: i32,
    pub changes: CommunityListChanges,
    pub updated_at: DateTime<Utc>,
}

/// Changes in a community list update
#[derive(Debug, Serialize)]
pub struct CommunityListChanges {
    pub added_artists: Vec<CommunityListArtistEntry>,
    pub removed_artists: Vec<CommunityListArtistEntry>,
    pub modified_artists: Vec<CommunityListArtistEntry>, // Changed rationale links
}

/// Bulk operation result for community lists
#[derive(Debug, Serialize)]
pub struct CommunityListBulkResult {
    pub total_processed: usize,
    pub successful: usize,
    pub failed: usize,
    pub errors: Vec<CommunityListBulkError>,
}

/// Error in bulk community list operations
#[derive(Debug, Serialize)]
pub struct CommunityListBulkError {
    pub entry_index: usize,
    pub artist_name: String,
    pub error: String,
}

/// Query parameters for community list directory
#[derive(Debug, Deserialize)]
pub struct CommunityListQuery {
    pub search: Option<String>,
    pub criteria_filter: Option<String>,
    pub owner_filter: Option<String>,
    pub sort_by: Option<String>, // "name", "created_at", "updated_at", "subscriber_count"
    pub sort_order: Option<String>, // "asc", "desc"
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

/// Community list moderation status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModerationStatus {
    Pending,
    Approved,
    Rejected,
    UnderReview,
}

/// Community list moderation queue entry
#[derive(Debug, Serialize)]
pub struct ModerationQueueEntry {
    pub list_id: Uuid,
    pub list_name: String,
    pub owner_email: String,
    pub status: ModerationStatus,
    pub submitted_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewer_notes: Option<String>,
}

/// Appeal for community list content
#[derive(Debug, Deserialize)]
pub struct CommunityListAppeal {
    pub list_id: Uuid,
    pub artist_id: Option<Uuid>, // None for list-level appeals
    pub reason: String,
    pub evidence: Option<String>,
    pub contact_email: String,
}

/// Appeal response
#[derive(Debug, Serialize)]
pub struct AppealResponse {
    pub appeal_id: Uuid,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
    pub estimated_review_time: String,
}