use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::{
    Connection, AppleMusicLibrary, AppleMusicLibraryTrack, AppleMusicLibraryAlbum,
    AppleMusicLibraryPlaylist, AppleMusicEnforcementResult, AppleMusicEnforcementOptions,
    Artist, FeaturedArtistDetection, DetectionMethod,
};
use crate::services::{AppleMusicService, EntityResolutionService};

/// Apple Music library analysis and scanning service
pub struct AppleMusicLibraryService {
    apple_music_service: Arc<AppleMusicService>,
    entity_resolver: Arc<EntityResolutionService>,
}

/// Blocked content item for export
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlockedContentItem {
    pub item_type: String, // "track", "album", "playlist"
    pub item_id: String,
    pub item_name: String,
    pub artist_name: String,
    pub blocked_artist_ids: Vec<String>,
    pub reason: String,
    pub confidence: f64,
    pub manual_action_required: String,
}

/// Library analysis summary
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LibraryAnalysisSummary {
    pub user_id: Uuid,
    pub total_tracks_scanned: u32,
    pub total_albums_scanned: u32,
    pub total_playlists_scanned: u32,
    pub blocked_tracks: Vec<BlockedContentItem>,
    pub blocked_albums: Vec<BlockedContentItem>,
    pub blocked_playlists: Vec<BlockedContentItem>,
    pub analysis_completed_at: DateTime<Utc>,
    pub limitations: Vec<String>,
}

impl AppleMusicLibraryService {
    pub fn new(
        apple_music_service: Arc<AppleMusicService>,
        entity_resolver: Arc<EntityResolutionService>,
    ) -> Self {
        Self {
            apple_music_service,
            entity_resolver,
        }
    }

    /// Perform comprehensive library analysis for blocked content
    pub async fn analyze_library(
        &self,
        connection: &Connection,
        dnp_artists: &[Artist],
        options: AppleMusicEnforcementOptions,
    ) -> Result<LibraryAnalysisSummary> {
        let mut summary = LibraryAnalysisSummary {
            user_id: connection.user_id,
            total_tracks_scanned: 0,
            total_albums_scanned: 0,
            total_playlists_scanned: 0,
            blocked_tracks: Vec::new(),
            blocked_albums: Vec::new(),
            blocked_playlists: Vec::new(),
            analysis_completed_at: Utc::now(),
            limitations: vec![
                "Apple Music API has limited write capabilities".to_string(),
                "Manual action required for content removal".to_string(),
            ],
        };

        // Create DNP artist lookup map for efficient matching
        let dnp_lookup = self.create_dnp_lookup_map(dnp_artists);

        if options.scan_library {
            // Scan library tracks
            let library_tracks = self.apple_music_service.get_library_tracks(connection).await?;
            summary.total_tracks_scanned = library_tracks.len() as u32;
            
            for track in library_tracks {
                if let Some(blocked_item) = self.analyze_track(&track, &dnp_lookup).await? {
                    summary.blocked_tracks.push(blocked_item);
                }
            }

            // Scan library albums
            let library_albums = self.apple_music_service.get_library_albums(connection).await?;
            summary.total_albums_scanned = library_albums.len() as u32;
            
            for album in library_albums {
                if let Some(blocked_item) = self.analyze_album(&album, &dnp_lookup).await? {
                    summary.blocked_albums.push(blocked_item);
                }
            }

            // Scan library playlists (limited analysis)
            if options.scan_playlists {
                let library_playlists = self.apple_music_service.get_library_playlists(connection).await?;
                summary.total_playlists_scanned = library_playlists.len() as u32;
                
                for playlist in library_playlists {
                    if let Some(blocked_item) = self.analyze_playlist(&playlist, &dnp_lookup).await? {
                        summary.blocked_playlists.push(blocked_item);
                    }
                }
                
                summary.limitations.push(
                    "Playlist analysis is limited - individual track analysis requires additional API calls".to_string()
                );
            }
        }

        summary.analysis_completed_at = Utc::now();
        Ok(summary)
    }

    /// Create efficient lookup map for DNP artists
    fn create_dnp_lookup_map<'a>(&self, dnp_artists: &'a [Artist]) -> HashMap<String, &'a Artist> {
        let mut lookup = HashMap::new();
        
        for artist in dnp_artists {
            // Add canonical name
            lookup.insert(artist.canonical_name.to_lowercase(), artist);
            
            // Add external IDs if available
            if let Some(apple_id) = &artist.external_ids.apple {
                lookup.insert(apple_id.to_lowercase(), artist);
            }
            
            // Add aliases if available
            for alias in &artist.aliases {
                lookup.insert(alias.name.to_lowercase(), artist);
            }
        }
        
        lookup
    }

    /// Analyze a library track for blocked content
    async fn analyze_track(
        &self,
        track: &AppleMusicLibraryTrack,
        dnp_lookup: &HashMap<String, &Artist>,
    ) -> Result<Option<BlockedContentItem>> {
        let artist_name = &track.attributes.artist_name;
        
        // Check for direct artist match
        if let Some(blocked_artist) = dnp_lookup.get(&artist_name.to_lowercase()) {
            return Ok(Some(BlockedContentItem {
                item_type: "track".to_string(),
                item_id: track.id.clone(),
                item_name: track.attributes.name.clone(),
                artist_name: artist_name.clone(),
                blocked_artist_ids: vec![blocked_artist.id.to_string()],
                reason: "Primary artist is in DNP list".to_string(),
                confidence: 1.0,
                manual_action_required: "Remove from library manually via Apple Music app".to_string(),
            }));
        }

        // Check for featured artists in track name
        if let Some(featured_detection) = self.detect_featured_artists_in_title(&track.attributes.name, dnp_lookup) {
            return Ok(Some(BlockedContentItem {
                item_type: "track".to_string(),
                item_id: track.id.clone(),
                item_name: track.attributes.name.clone(),
                artist_name: artist_name.clone(),
                blocked_artist_ids: featured_detection.featured_artists,
                reason: "Contains featured artist from DNP list".to_string(),
                confidence: featured_detection.confidence,
                manual_action_required: "Remove from library manually via Apple Music app".to_string(),
            }));
        }

        Ok(None)
    }

    /// Analyze a library album for blocked content
    async fn analyze_album(
        &self,
        album: &AppleMusicLibraryAlbum,
        dnp_lookup: &HashMap<String, &Artist>,
    ) -> Result<Option<BlockedContentItem>> {
        let artist_name = &album.attributes.artist_name;
        
        // Check for direct artist match
        if let Some(blocked_artist) = dnp_lookup.get(&artist_name.to_lowercase()) {
            return Ok(Some(BlockedContentItem {
                item_type: "album".to_string(),
                item_id: album.id.clone(),
                item_name: album.attributes.name.clone(),
                artist_name: artist_name.clone(),
                blocked_artist_ids: vec![blocked_artist.id.to_string()],
                reason: "Album artist is in DNP list".to_string(),
                confidence: 1.0,
                manual_action_required: "Remove from library manually via Apple Music app".to_string(),
            }));
        }

        Ok(None)
    }

    /// Analyze a library playlist for blocked content (limited)
    async fn analyze_playlist(
        &self,
        playlist: &AppleMusicLibraryPlaylist,
        dnp_lookup: &HashMap<String, &Artist>,
    ) -> Result<Option<BlockedContentItem>> {
        let playlist_name = &playlist.attributes.name;
        
        // Basic check if playlist name contains blocked artist names
        for (artist_name, blocked_artist) in dnp_lookup {
            if playlist_name.to_lowercase().contains(artist_name) {
                return Ok(Some(BlockedContentItem {
                    item_type: "playlist".to_string(),
                    item_id: playlist.id.clone(),
                    item_name: playlist_name.clone(),
                    artist_name: artist_name.clone(),
                    blocked_artist_ids: vec![blocked_artist.id.to_string()],
                    reason: "Playlist name contains blocked artist".to_string(),
                    confidence: 0.7, // Lower confidence for name-based matching
                    manual_action_required: "Review playlist contents and remove blocked tracks manually".to_string(),
                }));
            }
        }

        Ok(None)
    }

    /// Detect featured artists in track titles
    fn detect_featured_artists_in_title(
        &self,
        track_title: &str,
        dnp_lookup: &HashMap<String, &Artist>,
    ) -> Option<FeaturedArtistDetection> {
        let title_lower = track_title.to_lowercase();
        
        // Common featuring patterns
        let featuring_patterns = [
            r"feat\.?\s+([^)]+)",
            r"ft\.?\s+([^)]+)",
            r"featuring\s+([^)]+)",
            r"with\s+([^)]+)",
            r"\(feat\.?\s+([^)]+)\)",
            r"\(ft\.?\s+([^)]+)\)",
            r"\(featuring\s+([^)]+)\)",
            r"\(with\s+([^)]+)\)",
        ];

        for pattern in &featuring_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if let Some(captures) = regex.captures(&title_lower) {
                    if let Some(featured_text) = captures.get(1) {
                        let featured_artists_text = featured_text.as_str();
                        
                        // Check if any featured artist is in DNP list
                        for (artist_name, blocked_artist) in dnp_lookup {
                            if featured_artists_text.contains(artist_name) {
                                return Some(FeaturedArtistDetection {
                                    track_id: "unknown".to_string(), // Apple Music doesn't provide track IDs in this context
                                    track_name: track_title.to_string(),
                                    primary_artists: vec![], // Would need additional API call
                                    featured_artists: vec![blocked_artist.id.to_string()],
                                    collaboration_artists: vec![],
                                    detection_method: DetectionMethod::TrackTitle,
                                    confidence: 0.8,
                                });
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Export blocked content analysis to JSON file
    pub async fn export_analysis_to_file(
        &self,
        summary: &LibraryAnalysisSummary,
        export_path: &str,
    ) -> Result<()> {
        let json_content = serde_json::to_string_pretty(summary)
            .map_err(|e| anyhow!("Failed to serialize analysis: {}", e))?;

        tokio::fs::write(export_path, json_content)
            .await
            .map_err(|e| anyhow!("Failed to write export file: {}", e))?;

        tracing::info!("Exported Apple Music analysis to: {}", export_path);
        Ok(())
    }

    /// Generate manual action instructions
    pub fn generate_manual_action_instructions(&self, summary: &LibraryAnalysisSummary) -> String {
        let mut instructions = String::new();
        
        instructions.push_str("# Apple Music Manual Action Instructions\n\n");
        instructions.push_str("Due to Apple Music API limitations, the following content must be removed manually:\n\n");

        if !summary.blocked_tracks.is_empty() {
            instructions.push_str("## Blocked Tracks to Remove:\n");
            for track in &summary.blocked_tracks {
                instructions.push_str(&format!(
                    "- **{}** by {} (Reason: {})\n",
                    track.item_name, track.artist_name, track.reason
                ));
            }
            instructions.push_str("\n");
        }

        if !summary.blocked_albums.is_empty() {
            instructions.push_str("## Blocked Albums to Remove:\n");
            for album in &summary.blocked_albums {
                instructions.push_str(&format!(
                    "- **{}** by {} (Reason: {})\n",
                    album.item_name, album.artist_name, album.reason
                ));
            }
            instructions.push_str("\n");
        }

        if !summary.blocked_playlists.is_empty() {
            instructions.push_str("## Playlists to Review:\n");
            for playlist in &summary.blocked_playlists {
                instructions.push_str(&format!(
                    "- **{}** (Reason: {})\n",
                    playlist.item_name, playlist.reason
                ));
            }
            instructions.push_str("\n");
        }

        instructions.push_str("## How to Remove Content:\n");
        instructions.push_str("1. Open the Apple Music app on your device\n");
        instructions.push_str("2. Navigate to your Library\n");
        instructions.push_str("3. Find the blocked content listed above\n");
        instructions.push_str("4. For tracks/albums: Tap the '...' menu and select 'Remove from Library'\n");
        instructions.push_str("5. For playlists: Review the playlist contents and remove individual blocked tracks\n\n");

        instructions.push_str("## Limitations:\n");
        for limitation in &summary.limitations {
            instructions.push_str(&format!("- {}\n", limitation));
        }

        instructions
    }

    /// Get library statistics
    pub async fn get_library_statistics(&self, connection: &Connection) -> Result<LibraryStatistics> {
        let library = self.apple_music_service.scan_library(connection).await?;
        
        Ok(LibraryStatistics {
            total_tracks: library.library_tracks.len() as u32,
            total_albums: library.library_albums.len() as u32,
            total_playlists: library.library_playlists.len() as u32,
            last_scanned: library.scanned_at,
        })
    }
}

/// Library statistics summary
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LibraryStatistics {
    pub total_tracks: u32,
    pub total_albums: u32,
    pub total_playlists: u32,
    pub last_scanned: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::services::{AppleMusicService, EntityResolutionService, TokenVaultService};
    use crate::models::AppleMusicConfig;

    #[tokio::test]
    async fn test_create_dnp_lookup_map() {
        let token_vault = Arc::new(TokenVaultService::new());
        let apple_music_service = Arc::new(
            AppleMusicService::new(AppleMusicConfig::default(), token_vault).unwrap()
        );
        let entity_resolver = Arc::new(EntityResolutionService::new());
        let service = AppleMusicLibraryService::new(apple_music_service, entity_resolver);

        let mut artist = Artist::new("Test Artist".to_string());
        artist.external_ids.apple = Some("123456".to_string());
        
        let dnp_artists = vec![artist];
        let lookup = service.create_dnp_lookup_map(&dnp_artists);

        assert!(lookup.contains_key("test artist"));
        assert!(lookup.contains_key("123456"));
    }

    #[test]
    fn test_detect_featured_artists_in_title() {
        let token_vault = Arc::new(TokenVaultService::new());
        let apple_music_service = Arc::new(
            AppleMusicService::new(AppleMusicConfig::default(), token_vault).unwrap()
        );
        let entity_resolver = Arc::new(EntityResolutionService::new());
        let service = AppleMusicLibraryService::new(apple_music_service, entity_resolver);

        let mut blocked_artist = Artist::new("Drake".to_string());
        blocked_artist.id = Uuid::new_v4();
        
        let mut dnp_lookup = HashMap::new();
        dnp_lookup.insert("drake".to_string(), &blocked_artist);

        let detection = service.detect_featured_artists_in_title(
            "Song Title (feat. Drake)",
            &dnp_lookup,
        );

        assert!(detection.is_some());
        let detection = detection.unwrap();
        assert_eq!(detection.detection_method, DetectionMethod::TrackTitle);
        assert!(!detection.featured_artists.is_empty());
    }

    #[test]
    fn test_generate_manual_action_instructions() {
        let token_vault = Arc::new(TokenVaultService::new());
        let apple_music_service = Arc::new(
            AppleMusicService::new(AppleMusicConfig::default(), token_vault).unwrap()
        );
        let entity_resolver = Arc::new(EntityResolutionService::new());
        let service = AppleMusicLibraryService::new(apple_music_service, entity_resolver);

        let summary = LibraryAnalysisSummary {
            user_id: Uuid::new_v4(),
            total_tracks_scanned: 100,
            total_albums_scanned: 20,
            total_playlists_scanned: 5,
            blocked_tracks: vec![BlockedContentItem {
                item_type: "track".to_string(),
                item_id: "123".to_string(),
                item_name: "Test Song".to_string(),
                artist_name: "Test Artist".to_string(),
                blocked_artist_ids: vec!["456".to_string()],
                reason: "Primary artist is in DNP list".to_string(),
                confidence: 1.0,
                manual_action_required: "Remove manually".to_string(),
            }],
            blocked_albums: vec![],
            blocked_playlists: vec![],
            analysis_completed_at: Utc::now(),
            limitations: vec!["API limitations".to_string()],
        };

        let instructions = service.generate_manual_action_instructions(&summary);
        
        assert!(instructions.contains("# Apple Music Manual Action Instructions"));
        assert!(instructions.contains("Test Song"));
        assert!(instructions.contains("Test Artist"));
        assert!(instructions.contains("API limitations"));
    }
}