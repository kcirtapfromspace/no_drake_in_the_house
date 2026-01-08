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

pub mod apple_music;
pub mod deezer;
pub mod identity_resolver;
pub mod orchestrator;
pub mod spotify;
pub mod tidal;
pub mod traits;
pub mod youtube_music;

pub use apple_music::AppleMusicSyncWorker;
pub use deezer::DeezerSyncWorker;
pub use identity_resolver::*;
pub use orchestrator::*;
pub use spotify::SpotifySyncWorker;
pub use tidal::TidalSyncWorker;
pub use traits::*;
pub use youtube_music::YouTubeMusicSyncWorker;
