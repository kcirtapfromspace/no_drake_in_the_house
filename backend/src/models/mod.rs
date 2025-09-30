pub mod artist;
pub mod user;
pub mod auth;
pub mod token_vault;
pub mod spotify;
pub mod apple_music;
pub mod action;
pub mod dnp_list;
pub mod community_list;
pub mod rate_limit;
pub mod audit;
pub mod notification;


use serde::Serialize;

/// Provider badge for artist identification (shared between dnp_list and community_list)
#[derive(Debug, Serialize)]
pub struct ProviderBadge {
    pub provider: String,
    pub verified: bool,
    pub follower_count: Option<u64>,
}

// Explicit imports to avoid ambiguity
pub use artist::{Artist, ArtistMetadata, ExternalIds};
pub use user::{
    User, UserSettings, OAuthProvider, OAuthProviderInfo, CreateUserRequest, 
    LoginRequest, OAuthLoginRequest, TokenPair, RefreshTokenRequest, 
    TotpSetupRequest, TotpSetupResponse, TotpEnableRequest, TotpDisableRequest,
    RegisterRequest, AuthResponse, TotpVerifyRequest, AuthenticatedUser,
    TotpStatusResponse, RegistrationValidationError, RegistrationErrorResponse
};
// Use qualified name for UserProfile to avoid conflict with audit::UserProfile
pub use user::UserProfile as UserUserProfile;
// Main UserProfile type alias for backward compatibility
pub type UserProfile = user::UserProfile;
pub use auth::*;
pub use token_vault::*;
pub use spotify::*;
pub use apple_music::*;
pub use action::*;
pub use dnp_list::*;
pub use community_list::*;
pub use rate_limit::*;
pub use audit::{
    AuditLogEntry, CreateAuditLogRequest, AuditLogQuery, AuditLogResponse,
    SecurityEventType, SecurityEvent, SecuritySeverity, AccessReviewEntry,
    AccessStatus, DataExportRequest, DataRequestType, DataRequestStatus,
    UserDataExport, AuditDnpListExport, CommunitySubscriptionExport,
    ActionHistoryExport, ConnectionExport, ExportMetadata
};
// Use qualified name for audit UserProfile
pub use audit::UserProfile as AuditUserProfile;
pub use notification::*;


// Re-export stub types for tests
pub use crate::services::stubs::{Job, JobType, JobPriority, JobStatus, JobProgress, WorkerConfig, JobHandler};

// Additional types needed by tests
use serde::Deserialize;
use uuid::Uuid;

// Type alias for backward compatibility with tests
pub type UserRegistrationRequest = CreateUserRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddArtistToDnpRequest {
    pub artist_id: Uuid,
    pub reason: String,
    pub tags: Vec<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommunityListRequest {
    pub name: String,
    pub description: Option<String>,
    pub criteria: String,
    pub governance_url: Option<String>,
    pub update_cadence: String,
    pub visibility: CommunityListVisibility,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddCommunityListItemRequest {
    pub list_id: Uuid,
    pub artist_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeToCommunityListRequest {
    pub list_id: Uuid,
    pub notification_preferences: NotificationPreferences,
    pub version_pinned: Option<i32>,
    pub auto_update: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementPlanningRequest {
    pub user_id: Uuid,
    pub provider: StreamingProvider,
    pub options: EnforcementOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyCallbackRequest {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistSearchQuery {
    pub query: String,
    pub limit: Option<u32>,
    pub provider: Option<StreamingProvider>,
}

impl ArtistSearchQuery {
    pub fn new(query: String) -> Self {
        Self {
            query,
            limit: None,
            provider: None,
        }
    }
    
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
    
    pub fn with_provider(mut self, provider: StreamingProvider) -> Self {
        self.provider = Some(provider);
        self
    }
}

// StreamingProvider is defined in token_vault.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunityListVisibility {
    Public,
    Private,
    Unlisted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub email_enabled: bool,
    pub push_enabled: bool,
    pub frequency: NotificationFrequency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationFrequency {
    Immediate,
    Daily,
    Weekly,
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryScanResult {
    pub total_tracks: u32,
    pub blocked_tracks: u32,
    pub scan_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementPlan {
    pub user_id: Uuid,
    pub tracks_to_remove: Vec<String>,
    pub playlists_to_modify: Vec<String>,
    pub estimated_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementResult {
    pub tracks_removed: u32,
    pub playlists_modified: u32,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyAuthUrlResponse {
    pub auth_url: String,
    pub state: String,
}