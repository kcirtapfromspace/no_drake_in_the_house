// Services module - include working services only
pub mod auth_simple;
pub mod auth;
pub mod monitoring;
pub mod rate_limiting_middleware;
pub mod audit_logging;
pub mod dnp_list;
pub mod user;

pub mod stubs;

// Temporarily disabled services due to SQLx compilation issues
// These need to be fixed to use the correct SQLx syntax
// pub mod entity_resolution;
// pub mod external_apis;
// pub mod token_vault;
// pub mod token_vault_background;
// pub mod spotify;
// pub mod spotify_library;
// pub mod spotify_enforcement;
// pub mod apple_music;
// pub mod apple_music_library;
// pub mod community_list;
// pub mod rate_limiting;
// pub mod job_queue;
// pub mod enforcement_job_handler;
// pub mod audit;
// pub mod content_moderation;
// pub mod analytics;

pub use auth_simple::AuthService as SimpleAuthService;
pub use auth::AuthService;
pub use monitoring::*;
pub use rate_limiting_middleware::*;
pub use audit_logging::*;
pub use dnp_list::DnpListService;
pub use user::UserService;

// Export stub services for tests
pub use stubs::*;