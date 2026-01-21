use crate::error::AppError;
use crate::models::*;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

// Stub services are defined below and exported by the services module

// Stub implementations for services that are not yet fully implemented
// These allow tests to compile and run while the full implementations are being developed
//
// NOTE (US-008): TokenVaultService, AppleMusicConfig, and AppleMusicService
// stubs have been REMOVED in favor of the real PostgreSQL-backed implementations.
// See:
// - services/token_vault.rs (TokenVaultService with PostgreSQL persistence)
// - services/apple_music.rs (AppleMusicService)
// These are exported from services/mod.rs directly.

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

// SpotifyConfig, SpotifyService, and SpotifyLibraryService are now in the real spotify.rs/spotify_library.rs modules
// SpotifyEnforcementService stub removed - real implementation available
// AppleMusicConfig and AppleMusicService stubs removed - real implementations in apple_music.rs
// AppleMusicEnforcementService stub removed - real implementation in apple_music_enforcement.rs

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
            id: Uuid::new_v4(),
            user_id: _user_id,
            provider: "spotify".to_string(),
            options: EnforcementOptions::default(),
            dnp_artists: vec![],
            impact: EnforcementImpact {
                liked_songs: LibraryImpact {
                    total_tracks: 0,
                    tracks_to_remove: 0,
                    collaborations_found: 0,
                    featuring_found: 0,
                    exact_matches: 0,
                },
                playlists: PlaylistImpact {
                    total_playlists: 0,
                    playlists_to_modify: 0,
                    total_tracks: 0,
                    tracks_to_remove: 0,
                    user_playlists_affected: 0,
                    collaborative_playlists_affected: 0,
                    playlist_details: vec![],
                },
                followed_artists: FollowingImpact {
                    total_followed: 0,
                    artists_to_unfollow: 0,
                    exact_matches: 0,
                },
                saved_albums: AlbumImpact {
                    total_albums: 0,
                    albums_to_remove: 0,
                    exact_matches: 0,
                    collaboration_albums: 0,
                },
                total_items_affected: 0,
                estimated_time_saved_hours: 0.0,
            },
            actions: vec![],
            estimated_duration_seconds: 1,
            created_at: chrono::Utc::now(),
            idempotency_key: Uuid::new_v4().to_string(),
        })
    }
}

// Job queue related types (JobType, JobPriority, JobStatus, Job, JobProgress, WorkerConfig, JobHandler)
// are now in the real job_queue.rs module

// RateLimitingService is now in the real rate_limiting.rs module

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
