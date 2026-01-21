use crate::error::AppError;
use crate::models::*;
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

// Stub services are defined below and exported by the services module

// Stub implementations for services that are not yet fully implemented
// These allow tests to compile and run while the full implementations are being developed

#[derive(Clone)]
pub struct TokenVaultService {
    // Stub implementation
}

impl TokenVaultService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn store_token(&self, request: StoreTokenRequest) -> Result<Connection, AppError> {
        // Return a mock connection
        Ok(Connection {
            id: Uuid::new_v4(),
            user_id: request.user_id,
            provider: request.provider,
            provider_user_id: request.provider_user_id,
            scopes: request.scopes,
            access_token_encrypted: None,
            refresh_token_encrypted: None,
            token_version: 1,
            expires_at: request.expires_at,
            status: ConnectionStatus::Active,
            last_health_check: Some(chrono::Utc::now()),
            error_code: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    pub async fn get_token(
        &self,
        _user_id: Uuid,
        _provider: &str,
    ) -> Result<Option<String>, AppError> {
        Ok(Some("mock_token".to_string()))
    }

    pub async fn get_decrypted_token(
        &self,
        _connection_id: Uuid,
    ) -> Result<DecryptedToken, AppError> {
        Ok(DecryptedToken {
            access_token: "mock_access_token".to_string(),
            refresh_token: Some("mock_refresh_token".to_string()),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
            scopes: vec!["library.read".to_string()],
        })
    }

    pub async fn get_user_connections(&self, user_id: Uuid) -> Result<Vec<Connection>, AppError> {
        Ok(vec![Connection {
            id: Uuid::new_v4(),
            user_id,
            provider: StreamingProvider::AppleMusic,
            provider_user_id: "mock_user".to_string(),
            scopes: vec!["library.read".to_string()],
            access_token_encrypted: None,
            refresh_token_encrypted: None,
            token_version: 1,
            expires_at: None,
            status: ConnectionStatus::Active,
            last_health_check: Some(chrono::Utc::now()),
            error_code: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }])
    }

    pub async fn delete_connection(&self, _connection_id: Uuid) -> Result<(), AppError> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct EntityResolutionService {
    _pool: PgPool,
}

impl EntityResolutionService {
    pub fn new(pool: PgPool) -> Self {
        Self { _pool: pool }
    }

    pub async fn resolve_artist(
        &self,
        _query: &str,
        _provider: Option<&str>,
    ) -> Result<Option<Artist>, AppError> {
        // Return None for stub implementation
        Ok(None)
    }
}

#[derive(Clone)]
pub struct DnpListService {
    _pool: PgPool,
    _entity_service: Arc<EntityResolutionService>,
}

impl DnpListService {
    pub fn new(pool: PgPool, entity_service: Arc<EntityResolutionService>) -> Self {
        Self {
            _pool: pool,
            _entity_service: entity_service,
        }
    }

    pub async fn add_artist_to_dnp_list(
        &self,
        _user_id: Uuid,
        _request: AddArtistToDnpRequest,
    ) -> Result<UserArtistBlock, AppError> {
        Ok(UserArtistBlock {
            user_id: _user_id,
            artist_id: Uuid::new_v4(),
            tags: vec!["test".to_string()],
            note: Some("Mock entry".to_string()),
            created_at: chrono::Utc::now(),
        })
    }

    pub async fn get_user_dnp_list(
        &self,
        _user_id: Uuid,
    ) -> Result<Vec<UserArtistBlock>, AppError> {
        Ok(vec![])
    }

    pub async fn remove_artist_from_dnp_list(
        &self,
        _user_id: Uuid,
        _artist_id: Uuid,
    ) -> Result<(), AppError> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct CommunityListService {
    _pool: PgPool,
    _entity_service: Arc<EntityResolutionService>,
}

impl CommunityListService {
    pub fn new(pool: PgPool, entity_service: Arc<EntityResolutionService>) -> Self {
        Self {
            _pool: pool,
            _entity_service: entity_service,
        }
    }

    pub async fn create_community_list(
        &self,
        _request: CreateCommunityListRequest,
    ) -> Result<CommunityList, AppError> {
        Ok(CommunityList {
            id: Uuid::new_v4(),
            owner_user_id: Uuid::new_v4(),
            name: "Mock List".to_string(),
            description: Some("Mock description".to_string()),
            criteria: "Mock criteria".to_string(),
            visibility: "public".to_string(),
            governance_url: None,
            update_cadence: "weekly".to_string(),
            version: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
}

#[derive(Clone)]
pub struct SpotifyConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub api_base_url: String,
    pub token_url: String,
}

impl Default for SpotifyConfig {
    fn default() -> Self {
        Self {
            client_id: "mock_client_id".to_string(),
            client_secret: "mock_client_secret".to_string(),
            redirect_uri: "http://localhost:3000/callback".to_string(),
            api_base_url: "https://api.spotify.com/v1".to_string(),
            token_url: "https://accounts.spotify.com/api/token".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct SpotifyService {
    _config: SpotifyConfig,
    _token_vault: Arc<TokenVaultService>,
}

impl SpotifyService {
    pub fn new(
        config: SpotifyConfig,
        token_vault: Arc<TokenVaultService>,
    ) -> Result<Self, AppError> {
        Ok(Self {
            _config: config,
            _token_vault: token_vault,
        })
    }

    pub async fn get_auth_url(&self) -> Result<SpotifyAuthUrlResponse, AppError> {
        Ok(SpotifyAuthUrlResponse {
            auth_url: "https://accounts.spotify.com/authorize?mock=true".to_string(),
            state: "mock_state".to_string(),
        })
    }
}

#[derive(Clone)]
pub struct SpotifyLibraryService {
    _spotify_service: SpotifyService,
}

impl SpotifyLibraryService {
    pub fn new(spotify_service: SpotifyService) -> Self {
        Self {
            _spotify_service: spotify_service,
        }
    }

    pub async fn scan_library(&self, _user_id: Uuid) -> Result<LibraryScanResult, AppError> {
        Ok(LibraryScanResult {
            total_tracks: 100,
            blocked_tracks: 5,
            scan_duration_ms: 1000,
        })
    }
}

#[derive(Clone)]
pub struct SpotifyEnforcementService {
    _spotify_service: SpotifyService,
}

impl SpotifyEnforcementService {
    pub fn new(spotify_service: SpotifyService) -> Self {
        Self {
            _spotify_service: spotify_service,
        }
    }

    pub async fn execute_enforcement(
        &self,
        _user_id: Uuid,
        _plan: EnforcementPlan,
    ) -> Result<EnforcementResult, AppError> {
        Ok(EnforcementResult {
            tracks_removed: 5,
            playlists_modified: 2,
            execution_time_ms: 500,
        })
    }
}

#[derive(Clone)]
pub struct EnforcementPlanningService {
    _dnp_service: Arc<DnpListService>,
    _entity_service: Arc<EntityResolutionService>,
}

impl EnforcementPlanningService {
    pub fn new(
        dnp_service: Arc<DnpListService>,
        entity_service: Arc<EntityResolutionService>,
    ) -> Self {
        Self {
            _dnp_service: dnp_service,
            _entity_service: entity_service,
        }
    }

    pub async fn create_enforcement_plan(
        &self,
        _user_id: Uuid,
        _library_scan: LibraryScanResult,
    ) -> Result<EnforcementPlan, AppError> {
        Ok(EnforcementPlan {
            user_id: _user_id,
            tracks_to_remove: vec![],
            playlists_to_modify: vec![],
            estimated_duration_ms: 1000,
        })
    }
}

#[derive(Clone)]
pub struct AppleMusicConfig {
    pub team_id: String,
    pub key_id: String,
    pub private_key: String,
}

impl Default for AppleMusicConfig {
    fn default() -> Self {
        Self {
            team_id: "mock_team_id".to_string(),
            key_id: "mock_key_id".to_string(),
            private_key: "mock_private_key".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct AppleMusicService {
    _config: AppleMusicConfig,
    _token_vault: Arc<TokenVaultService>,
}

impl AppleMusicService {
    pub fn new(
        config: AppleMusicConfig,
        token_vault: Arc<TokenVaultService>,
    ) -> Result<Self, AppError> {
        Ok(Self {
            _config: config,
            _token_vault: token_vault,
        })
    }
}

#[derive(Clone)]
pub struct AppleMusicEnforcementService {
    _apple_service: AppleMusicService,
}

impl AppleMusicEnforcementService {
    pub fn new(apple_service: AppleMusicService) -> Self {
        Self {
            _apple_service: apple_service,
        }
    }
}

// Job queue related stubs
#[derive(Clone, Debug, PartialEq)]
pub enum JobType {
    EnforcementExecution,
    BatchRollback,
    TokenRefresh,
    LibraryScan,
    CommunityListUpdate,
    HealthCheck,
}

#[derive(Clone, Debug, PartialEq)]
pub enum JobPriority {
    Critical,
    High,
    Normal,
    Low,
}

#[derive(Clone, Debug, PartialEq)]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Retrying,
    DeadLetter,
}

#[derive(Clone)]
pub struct Job {
    pub id: Uuid,
    pub job_type: JobType,
    pub priority: JobPriority,
    pub payload: Value,
    pub user_id: Option<Uuid>,
    pub provider: Option<String>,
    pub status: JobStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub scheduled_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub error_message: Option<String>,
    pub progress: Option<Value>,
}

#[derive(Clone)]
pub struct RateLimitingService {
    // Stub implementation
}

impl RateLimitingService {
    pub fn new(_redis_url: &str) -> Self {
        Self {}
    }
}

#[derive(Clone)]
pub struct JobQueueService {
    _redis_url: String,
    _rate_limiter: RateLimitingService,
}

impl JobQueueService {
    pub fn new(redis_url: &str, rate_limiter: RateLimitingService) -> Self {
        Self {
            _redis_url: redis_url.to_string(),
            _rate_limiter: rate_limiter,
        }
    }

    pub async fn enqueue_job(&self, _job: Job) -> Result<(), AppError> {
        Ok(())
    }

    pub async fn get_jobs_by_priority(&self) -> Result<Vec<Job>, AppError> {
        Ok(vec![])
    }

    pub async fn get_jobs_by_type(&self, _job_types: Vec<JobType>) -> Result<Vec<Job>, AppError> {
        Ok(vec![])
    }

    pub async fn get_jobs_by_status(
        &self,
        _statuses: Vec<JobStatus>,
    ) -> Result<Vec<Job>, AppError> {
        Ok(vec![])
    }
}

// Additional stub services needed by tests
#[derive(Clone)]
pub struct ExternalApiService {
    // Stub implementation
}

impl ExternalApiService {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone)]
pub struct WorkerConfig {
    pub job_types: Vec<JobType>,
    pub max_concurrent_jobs: usize,
    pub poll_interval_ms: u64,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            job_types: vec![JobType::EnforcementExecution],
            max_concurrent_jobs: 4,
            poll_interval_ms: 1000,
        }
    }
}

pub trait JobHandler {
    async fn handle(&self, job: &Job) -> anyhow::Result<serde_json::Value>;
    fn job_type(&self) -> JobType;
}

#[derive(Clone)]
pub struct JobProgress {
    pub job_id: Uuid,
    pub progress_percent: f32,
    pub status_message: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
