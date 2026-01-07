//! Catalog Sync Module
//!
//! Multi-platform artist catalog synchronization for:
//! - Spotify
//! - Apple Music
//! - Tidal
//! - YouTube Music
//! - Deezer
//!
//! Features:
//! - Rate-limited API access per platform
//! - Cross-platform identity resolution
//! - Incremental and full sync support
//! - Checkpoint-based resumable syncs

pub mod traits;
pub mod orchestrator;
pub mod identity_resolver;
pub mod spotify;
pub mod apple_music;
pub mod tidal;
pub mod youtube_music;
pub mod deezer;

pub use traits::*;
pub use orchestrator::*;
pub use identity_resolver::*;
pub use spotify::SpotifySyncWorker;
pub use apple_music::AppleMusicSyncWorker;
pub use tidal::TidalSyncWorker;
pub use youtube_music::YouTubeMusicSyncWorker;
pub use deezer::DeezerSyncWorker;
