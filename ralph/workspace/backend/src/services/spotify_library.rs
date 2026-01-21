use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{
    Connection, SpotifyLibrary, SpotifySavedTrack, SpotifyPlaylist, SpotifyFollowedArtist,
    SpotifyAlbum, SpotifyTrack, SpotifyArtist, FeaturedArtistDetection, DetectionMethod,
    EnforcementPlan, EnforcementOptions, EnforcementImpact, PlannedAction, ActionType,
    EntityType, BlockReason, AffectedTrack, LibraryImpact, PlaylistImpact, FollowingImpact,
    AlbumImpact, PlaylistModification, Artist,
};
use crate::services::SpotifyService;

/// Service for analyzing Spotify libraries and planning enforcement
pub struct SpotifyLibraryService {
    spotify_service: Arc<SpotifyService>,
    featuring_patterns: Vec<Regex>,
}

impl SpotifyLibraryService {
    pub fn new(spotify_service: Arc<SpotifyService>) -> Self {
        // Compile regex patterns for detecting featured artists in track titles
        let featuring_patterns = vec![
            Regex::new(r"(?i)\bfeat\.?\s+(.+?)(?:\s*[\(\[]|$)").unwrap(),
            Regex::new(r"(?i)\bft\.?\s+(.+?)(?:\s*[\(\[]|$)").unwrap(),
            Regex::new(r"(?i)\bfeaturing\s+(.+?)(?:\s*[\(\[]|$)").unwrap(),
            Regex::new(r"(?i)\bwith\s+(.+?)(?:\s*[\(\[]|$)").unwrap(),
            Regex::new(r"(?i)\b&\s+(.+?)(?:\s*[\(\[]|$)").unwrap(),
            Regex::new(r"(?i)\bx\s+(.+?)(?:\s*[\(\[]|$)").unwrap(),
        ];

        Self {
            spotify_service,
            featuring_patterns,
        }
    }

    /// Scan user's complete Spotify library
    pub async fn scan_library(&self, connection: &Connection) -> Result<SpotifyLibrary> {
        let user_profile = self.get_user_profile(connection).await?;
        
        // Scan all library components concurrently
        let (liked_songs, playlists, followed_artists, saved_albums) = tokio::try_join!(
            self.scan_liked_songs(connection),
            self.scan_playlists(connection),
            self.scan_followed_artists(connection),
            self.scan_saved_albums(connection)
        )?;

        Ok(SpotifyLibrary {
            user_id: connection.user_id,
            spotify_user_id: user_profile.id,
            liked_songs,
            playlists,
            followed_artists,
            saved_albums,
            scanned_at: Utc::now(),
        })
    }

    /// Get user profile information
    async fn get_user_profile(&self, connection: &Connection) -> Result<crate::services::spotify::SpotifyUserProfile> {
        let response = self
            .spotify_service
            .make_api_request(connection, "GET", "https://api.spotify.com/v1/me", None)
            .await?;

        if response.status().is_success() {
            let profile = response.json().await?;
            Ok(profile)
        } else {
            Err(anyhow!("Failed to get user profile: {}", response.status()))
        }
    }

    /// Scan user's liked songs (saved tracks)
    async fn scan_liked_songs(&self, connection: &Connection) -> Result<Vec<SpotifySavedTrack>> {
        let mut liked_songs = Vec::new();
        let mut offset = 0;
        let limit = 50;

        loop {
            let url = format!(
                "https://api.spotify.com/v1/me/tracks?limit={}&offset={}",
                limit, offset
            );

            let response = self
                .spotify_service
                .make_api_request(connection, "GET", &url, None)
                .await?;

            if !response.status().is_success() {
                return Err(anyhow!("Failed to fetch liked songs: {}", response.status()));
            }

            let data: Value = response.json().await?;
            let empty_vec = vec![];
            let items = data["items"].as_array().unwrap_or(&empty_vec);

            if items.is_empty() {
                break;
            }

            for item in items {
                if let Ok(saved_track) = serde_json::from_value::<SpotifySavedTrack>(item.clone()) {
                    liked_songs.push(saved_track);
                }
            }

            offset += limit;

            // Check if we've reached the end
            if items.len() < limit {
                break;
            }
        }

        Ok(liked_songs)
    }

    /// Scan user's playlists
    async fn scan_playlists(&self, connection: &Connection) -> Result<Vec<SpotifyPlaylist>> {
        let mut playlists = Vec::new();
        let mut offset = 0;
        let limit = 50;

        loop {
            let url = format!(
                "https://api.spotify.com/v1/me/playlists?limit={}&offset={}",
                limit, offset
            );

            let response = self
                .spotify_service
                .make_api_request(connection, "GET", &url, None)
                .await?;

            if !response.status().is_success() {
                return Err(anyhow!("Failed to fetch playlists: {}", response.status()));
            }

            let data: Value = response.json().await?;
            let empty_vec = vec![];
            let items = data["items"].as_array().unwrap_or(&empty_vec);

            if items.is_empty() {
                break;
            }

            for item in items {
                if let Ok(mut playlist) = serde_json::from_value::<SpotifyPlaylist>(item.clone()) {
                    // Fetch full playlist details including tracks
                    playlist = self.fetch_playlist_details(connection, &playlist.id).await?;
                    playlists.push(playlist);
                }
            }

            offset += limit;

            if items.len() < limit {
                break;
            }
        }

        Ok(playlists)
    }

    /// Fetch detailed playlist information including tracks
    async fn fetch_playlist_details(&self, connection: &Connection, playlist_id: &str) -> Result<SpotifyPlaylist> {
        let url = format!(
            "https://api.spotify.com/v1/playlists/{}?fields=id,name,description,owner,public,collaborative,tracks.total,tracks.items(added_at,added_by,is_local,track(id,name,artists,album,duration_ms,explicit,popularity,preview_url,external_urls,is_local,is_playable)),external_urls,images,snapshot_id",
            playlist_id
        );

        let response = self
            .spotify_service
            .make_api_request(connection, "GET", &url, None)
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch playlist details: {}", response.status()));
        }

        let playlist: SpotifyPlaylist = response.json().await?;
        Ok(playlist)
    }

    /// Scan user's followed artists
    async fn scan_followed_artists(&self, connection: &Connection) -> Result<Vec<SpotifyFollowedArtist>> {
        let mut followed_artists = Vec::new();
        let mut after: Option<String> = None;
        let limit = 50;

        loop {
            let mut url = format!(
                "https://api.spotify.com/v1/me/following?type=artist&limit={}",
                limit
            );

            if let Some(ref after_cursor) = after {
                url.push_str(&format!("&after={}", after_cursor));
            }

            let response = self
                .spotify_service
                .make_api_request(connection, "GET", &url, None)
                .await?;

            if !response.status().is_success() {
                return Err(anyhow!("Failed to fetch followed artists: {}", response.status()));
            }

            let data: Value = response.json().await?;
            let empty_vec = vec![];
            let artists = data["artists"]["items"].as_array().unwrap_or(&empty_vec);

            if artists.is_empty() {
                break;
            }

            for artist_data in artists {
                if let Ok(artist) = serde_json::from_value::<SpotifyArtist>(artist_data.clone()) {
                    followed_artists.push(SpotifyFollowedArtist {
                        artist,
                        followed_at: None, // Spotify doesn't provide follow date
                    });
                }
            }

            // Get cursor for next page
            if let Some(cursors) = data["artists"]["cursors"].as_object() {
                after = cursors["after"].as_str().map(|s| s.to_string());
            } else {
                break;
            }

            if artists.len() < limit {
                break;
            }
        }

        Ok(followed_artists)
    }

    /// Scan user's saved albums
    async fn scan_saved_albums(&self, connection: &Connection) -> Result<Vec<SpotifyAlbum>> {
        let mut saved_albums = Vec::new();
        let mut offset = 0;
        let limit = 50;

        loop {
            let url = format!(
                "https://api.spotify.com/v1/me/albums?limit={}&offset={}",
                limit, offset
            );

            let response = self
                .spotify_service
                .make_api_request(connection, "GET", &url, None)
                .await?;

            if !response.status().is_success() {
                return Err(anyhow!("Failed to fetch saved albums: {}", response.status()));
            }

            let data: Value = response.json().await?;
            let empty_vec = vec![];
            let items = data["items"].as_array().unwrap_or(&empty_vec);

            if items.is_empty() {
                break;
            }

            for item in items {
                if let Some(album_data) = item["album"].as_object() {
                    if let Ok(album) = serde_json::from_value::<SpotifyAlbum>(Value::Object(album_data.clone())) {
                        saved_albums.push(album);
                    }
                }
            }

            offset += limit;

            if items.len() < limit {
                break;
            }
        }

        Ok(saved_albums)
    }

    /// Detect featured artists and collaborations in a track
    pub fn detect_featured_artists(&self, track: &SpotifyTrack) -> FeaturedArtistDetection {
        let mut featured_artists = Vec::new();
        let mut collaboration_artists = Vec::new();
        let mut detection_method = DetectionMethod::ArtistArray;
        let mut confidence = 0.0;

        // Method 1: Check track title for featuring patterns
        for pattern in &self.featuring_patterns {
            if let Some(captures) = pattern.captures(&track.name) {
                if let Some(featured_text) = captures.get(1) {
                    // Try to match featured text with actual artists
                    let featured_names = self.parse_featured_names(featured_text.as_str());
                    for name in featured_names {
                        if let Some(artist) = self.find_artist_by_name(&track.artists, &name) {
                            featured_artists.push(artist.id.clone());
                        }
                    }
                    detection_method = DetectionMethod::TrackTitle;
                    confidence = 0.9;
                    break;
                }
            }
        }

        // Method 2: Multiple artists in array (collaboration detection)
        if track.artists.len() > 1 {
            let primary_artist = &track.artists[0];
            for artist in &track.artists[1..] {
                collaboration_artists.push(artist.id.clone());
            }
            
            if confidence < 0.7 {
                detection_method = DetectionMethod::ArtistArray;
                confidence = 0.7;
            }
        }

        // Method 3: Check album artist vs track artists
        if track.album.artists.len() == 1 && track.artists.len() > 1 {
            let album_artist_id = &track.album.artists[0].id;
            for artist in &track.artists {
                if artist.id != *album_artist_id {
                    featured_artists.push(artist.id.clone());
                }
            }
            
            if confidence < 0.6 {
                detection_method = DetectionMethod::AlbumArtist;
                confidence = 0.6;
            }
        }

        // Remove duplicates and primary artist
        let primary_artists: Vec<String> = if !track.artists.is_empty() {
            vec![track.artists[0].id.clone()]
        } else {
            Vec::new()
        };

        featured_artists.retain(|id| !primary_artists.contains(id));
        collaboration_artists.retain(|id| !primary_artists.contains(id) && !featured_artists.contains(id));

        FeaturedArtistDetection {
            track_id: track.id.clone(),
            track_name: track.name.clone(),
            primary_artists,
            featured_artists,
            collaboration_artists,
            detection_method,
            confidence,
        }
    }

    /// Parse featured artist names from text
    fn parse_featured_names(&self, text: &str) -> Vec<String> {
        // Split on common separators and clean up
        text.split(&[',', '&', '+'][..])
            .flat_map(|s| s.split(" and "))
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    /// Find artist by name in a list of artists
    fn find_artist_by_name<'a>(&self, artists: &'a [SpotifyArtist], name: &str) -> Option<&'a SpotifyArtist> {
        artists.iter().find(|artist| {
            artist.name.to_lowercase().contains(&name.to_lowercase()) ||
            name.to_lowercase().contains(&artist.name.to_lowercase())
        })
    }

    /// Create enforcement plan with dry-run impact calculation
    pub async fn create_enforcement_plan(
        &self,
        connection: &Connection,
        dnp_artists: Vec<Artist>,
        options: EnforcementOptions,
    ) -> Result<EnforcementPlan> {
        // Scan library if not provided
        let library = self.scan_library(connection).await?;
        
        // Convert DNP artists to a lookup map by external IDs
        let dnp_lookup = self.create_dnp_lookup(&dnp_artists);
        
        let mut plan = EnforcementPlan::new(
            connection.user_id,
            "spotify".to_string(),
            options.clone(),
            dnp_artists.iter().map(|a| a.id).collect(),
        );

        // Analyze impact and create actions
        plan.impact = self.analyze_enforcement_impact(&library, &dnp_lookup, &options).await?;
        plan.actions = self.create_planned_actions(&library, &dnp_lookup, &options).await?;

        // Calculate estimated duration
        plan.estimated_duration_seconds = plan.actions.iter()
            .map(|action| action.estimated_duration_ms / 1000)
            .sum();

        Ok(plan)
    }

    /// Create DNP artist lookup map by Spotify IDs
    fn create_dnp_lookup(&self, dnp_artists: &[Artist]) -> HashMap<String, Uuid> {
        let mut lookup = HashMap::new();
        
        for artist in dnp_artists {
            if let Some(spotify_id) = &artist.external_ids.spotify {
                lookup.insert(spotify_id.clone(), artist.id);
            }
        }
        
        lookup
    }

    /// Analyze enforcement impact on library
    async fn analyze_enforcement_impact(
        &self,
        library: &SpotifyLibrary,
        dnp_lookup: &HashMap<String, Uuid>,
        options: &EnforcementOptions,
    ) -> Result<EnforcementImpact> {
        let mut impact = EnforcementImpact::default();

        // Analyze liked songs impact
        impact.liked_songs = self.analyze_liked_songs_impact(&library.liked_songs, dnp_lookup, options);

        // Analyze playlists impact
        impact.playlists = self.analyze_playlists_impact(&library.playlists, dnp_lookup, options).await;

        // Analyze followed artists impact
        impact.followed_artists = self.analyze_followed_artists_impact(&library.followed_artists, dnp_lookup);

        // Analyze saved albums impact
        impact.saved_albums = self.analyze_saved_albums_impact(&library.saved_albums, dnp_lookup, options);

        // Calculate totals
        impact.total_items_affected = impact.liked_songs.tracks_to_remove
            + impact.playlists.tracks_to_remove
            + impact.followed_artists.artists_to_unfollow
            + impact.saved_albums.albums_to_remove;

        // Estimate time saved (assuming 3.5 minutes average track length)
        let total_tracks_removed = impact.liked_songs.tracks_to_remove + impact.playlists.tracks_to_remove;
        impact.estimated_time_saved_hours = (total_tracks_removed as f64 * 3.5) / 60.0;

        Ok(impact)
    }

    /// Analyze impact on liked songs
    fn analyze_liked_songs_impact(
        &self,
        liked_songs: &[SpotifySavedTrack],
        dnp_lookup: &HashMap<String, Uuid>,
        options: &EnforcementOptions,
    ) -> LibraryImpact {
        let mut impact = LibraryImpact {
            total_tracks: liked_songs.len() as u32,
            ..Default::default()
        };

        for saved_track in liked_songs {
            let detection = self.detect_featured_artists(&saved_track.track);
            let block_result = self.should_block_track(&detection, dnp_lookup, options);
            
            if block_result.should_block {
                impact.tracks_to_remove += 1;
                
                match block_result.reason {
                    BlockReason::ExactMatch => impact.exact_matches += 1,
                    BlockReason::Collaboration => impact.collaborations_found += 1,
                    BlockReason::Featuring => impact.featuring_found += 1,
                    _ => {}
                }
            }
        }

        impact
    }

    /// Analyze impact on playlists
    async fn analyze_playlists_impact(
        &self,
        playlists: &[SpotifyPlaylist],
        dnp_lookup: &HashMap<String, Uuid>,
        options: &EnforcementOptions,
    ) -> PlaylistImpact {
        let mut impact = PlaylistImpact {
            total_playlists: playlists.len() as u32,
            ..Default::default()
        };

        for playlist in playlists {
            let mut playlist_modification = PlaylistModification {
                playlist_id: playlist.id.clone(),
                playlist_name: playlist.name.clone(),
                is_user_owned: true, // Assume user-owned for now
                is_collaborative: playlist.collaborative,
                total_tracks: playlist.tracks.total,
                tracks_to_remove: 0,
                affected_tracks: Vec::new(),
            };

            if let Some(ref tracks) = playlist.tracks.items {
                impact.total_tracks += tracks.len() as u32;
                
                for playlist_track in tracks {
                    if let Some(ref track) = playlist_track.track {
                        let detection = self.detect_featured_artists(track);
                        let block_result = self.should_block_track(&detection, dnp_lookup, options);
                        
                        if block_result.should_block {
                            playlist_modification.tracks_to_remove += 1;
                            playlist_modification.affected_tracks.push(AffectedTrack {
                                track_id: track.id.clone(),
                                track_name: track.name.clone(),
                                artist_names: track.artists.iter().map(|a| a.name.clone()).collect(),
                                blocked_artist_ids: block_result.blocked_artist_ids,
                                reason: block_result.reason.clone(),
                                confidence: detection.confidence,
                            });
                        }
                    }
                }
            }

            if playlist_modification.tracks_to_remove > 0 {
                impact.playlists_to_modify += 1;
                impact.tracks_to_remove += playlist_modification.tracks_to_remove;
                
                if playlist_modification.is_user_owned {
                    impact.user_playlists_affected += 1;
                }
                if playlist_modification.is_collaborative {
                    impact.collaborative_playlists_affected += 1;
                }
                
                impact.playlist_details.push(playlist_modification);
            }
        }

        impact
    }

    /// Analyze impact on followed artists
    fn analyze_followed_artists_impact(
        &self,
        followed_artists: &[SpotifyFollowedArtist],
        dnp_lookup: &HashMap<String, Uuid>,
    ) -> FollowingImpact {
        let mut impact = FollowingImpact {
            total_followed: followed_artists.len() as u32,
            ..Default::default()
        };

        for followed_artist in followed_artists {
            if dnp_lookup.contains_key(&followed_artist.artist.id) {
                impact.artists_to_unfollow += 1;
                impact.exact_matches += 1;
            }
        }

        impact
    }

    /// Analyze impact on saved albums
    fn analyze_saved_albums_impact(
        &self,
        saved_albums: &[SpotifyAlbum],
        dnp_lookup: &HashMap<String, Uuid>,
        options: &EnforcementOptions,
    ) -> AlbumImpact {
        let mut impact = AlbumImpact {
            total_albums: saved_albums.len() as u32,
            ..Default::default()
        };

        for album in saved_albums {
            let mut should_remove = false;
            let mut is_collaboration = false;

            // Check if any album artist is in DNP list
            for artist in &album.artists {
                if dnp_lookup.contains_key(&artist.id) {
                    should_remove = true;
                    if album.artists.len() > 1 {
                        is_collaboration = true;
                    }
                    break;
                }
            }

            // For collaborative albums, check aggressiveness level
            if should_remove && is_collaboration && !options.block_collaborations {
                should_remove = false;
            }

            if should_remove {
                impact.albums_to_remove += 1;
                if is_collaboration {
                    impact.collaboration_albums += 1;
                } else {
                    impact.exact_matches += 1;
                }
            }
        }

        impact
    }

    /// Determine if a track should be blocked based on detection and options
    fn should_block_track(
        &self,
        detection: &FeaturedArtistDetection,
        dnp_lookup: &HashMap<String, Uuid>,
        options: &EnforcementOptions,
    ) -> BlockResult {
        let mut blocked_artist_ids = Vec::new();
        let mut reason = BlockReason::ExactMatch;
        let mut should_block = false;

        // Check primary artists (always block if in DNP)
        for artist_id in &detection.primary_artists {
            if dnp_lookup.contains_key(artist_id) {
                blocked_artist_ids.push(artist_id.clone());
                should_block = true;
                reason = BlockReason::ExactMatch;
            }
        }

        // Check featured artists
        if !should_block && options.block_featuring {
            for artist_id in &detection.featured_artists {
                if dnp_lookup.contains_key(artist_id) {
                    blocked_artist_ids.push(artist_id.clone());
                    should_block = true;
                    reason = BlockReason::Featuring;
                }
            }
        }

        // Check collaboration artists
        if !should_block && options.block_collaborations {
            for artist_id in &detection.collaboration_artists {
                if dnp_lookup.contains_key(artist_id) {
                    blocked_artist_ids.push(artist_id.clone());
                    should_block = true;
                    reason = BlockReason::Collaboration;
                }
            }
        }

        // Apply aggressiveness level
        match options.aggressiveness {
            crate::models::AggressivenessLevel::Conservative => {
                // Only block exact matches
                if reason != BlockReason::ExactMatch {
                    should_block = false;
                }
            }
            crate::models::AggressivenessLevel::Moderate => {
                // Block exact matches and high-confidence detections
                if detection.confidence < 0.7 && reason != BlockReason::ExactMatch {
                    should_block = false;
                }
            }
            crate::models::AggressivenessLevel::Aggressive => {
                // Block all detections
            }
        }

        BlockResult {
            should_block,
            reason,
            blocked_artist_ids,
        }
    }

    /// Create planned actions for enforcement
    async fn create_planned_actions(
        &self,
        library: &SpotifyLibrary,
        dnp_lookup: &HashMap<String, Uuid>,
        options: &EnforcementOptions,
    ) -> Result<Vec<PlannedAction>> {
        let mut actions = Vec::new();

        // Create actions for liked songs
        for saved_track in &library.liked_songs {
            let detection = self.detect_featured_artists(&saved_track.track);
            let block_result = self.should_block_track(&detection, dnp_lookup, options);
            
            if block_result.should_block {
                actions.push(PlannedAction {
                    id: Uuid::new_v4(),
                    action_type: ActionType::RemoveLikedSong,
                    entity_type: EntityType::Track,
                    entity_id: saved_track.track.id.clone(),
                    entity_name: saved_track.track.name.clone(),
                    reason: block_result.reason,
                    confidence: detection.confidence,
                    estimated_duration_ms: 500, // 500ms per API call
                    dependencies: Vec::new(),
                    metadata: serde_json::json!({
                        "artists": saved_track.track.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>(),
                        "blocked_artist_ids": block_result.blocked_artist_ids
                    }),
                });
            }
        }

        // Create actions for playlist tracks
        for playlist in &library.playlists {
            if let Some(ref tracks) = playlist.tracks.items {
                for playlist_track in tracks {
                    if let Some(ref track) = playlist_track.track {
                        let detection = self.detect_featured_artists(track);
                        let block_result = self.should_block_track(&detection, dnp_lookup, options);
                        
                        if block_result.should_block {
                            // Skip user playlists if preserve_user_playlists is true
                            if options.preserve_user_playlists && playlist.owner.id == library.spotify_user_id {
                                continue;
                            }

                            actions.push(PlannedAction {
                                id: Uuid::new_v4(),
                                action_type: ActionType::RemovePlaylistTrack,
                                entity_type: EntityType::Track,
                                entity_id: track.id.clone(),
                                entity_name: track.name.clone(),
                                reason: block_result.reason,
                                confidence: detection.confidence,
                                estimated_duration_ms: 750, // 750ms per playlist track removal
                                dependencies: Vec::new(),
                                metadata: serde_json::json!({
                                    "playlist_id": playlist.id,
                                    "playlist_name": playlist.name,
                                    "artists": track.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>(),
                                    "blocked_artist_ids": block_result.blocked_artist_ids
                                }),
                            });
                        }
                    }
                }
            }
        }

        // Create actions for followed artists
        for followed_artist in &library.followed_artists {
            if dnp_lookup.contains_key(&followed_artist.artist.id) {
                actions.push(PlannedAction {
                    id: Uuid::new_v4(),
                    action_type: ActionType::UnfollowArtist,
                    entity_type: EntityType::Artist,
                    entity_id: followed_artist.artist.id.clone(),
                    entity_name: followed_artist.artist.name.clone(),
                    reason: BlockReason::ExactMatch,
                    confidence: 1.0,
                    estimated_duration_ms: 300, // 300ms per unfollow
                    dependencies: Vec::new(),
                    metadata: serde_json::json!({
                        "artist_id": followed_artist.artist.id
                    }),
                });
            }
        }

        // Create actions for saved albums
        for album in &library.saved_albums {
            let mut should_remove = false;
            let mut blocked_artist_ids = Vec::new();
            let mut reason = BlockReason::ExactMatch;

            for artist in &album.artists {
                if dnp_lookup.contains_key(&artist.id) {
                    should_remove = true;
                    blocked_artist_ids.push(artist.id.clone());
                    if album.artists.len() > 1 {
                        reason = BlockReason::Collaboration;
                    }
                    break;
                }
            }

            if should_remove && (reason != BlockReason::Collaboration || options.block_collaborations) {
                actions.push(PlannedAction {
                    id: Uuid::new_v4(),
                    action_type: ActionType::RemoveSavedAlbum,
                    entity_type: EntityType::Album,
                    entity_id: album.id.clone(),
                    entity_name: album.name.clone(),
                    reason,
                    confidence: 1.0,
                    estimated_duration_ms: 400, // 400ms per album removal
                    dependencies: Vec::new(),
                    metadata: serde_json::json!({
                        "artists": album.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>(),
                        "blocked_artist_ids": blocked_artist_ids
                    }),
                });
            }
        }

        Ok(actions)
    }
}

/// Result of block decision
struct BlockResult {
    should_block: bool,
    reason: BlockReason,
    blocked_artist_ids: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SpotifyTrack, SpotifyArtist, SpotifyAlbum};
    use std::collections::HashMap;

    fn create_test_track(name: &str, artists: Vec<(&str, &str)>) -> SpotifyTrack {
        SpotifyTrack {
            id: "test_track".to_string(),
            name: name.to_string(),
            artists: artists.into_iter().map(|(id, name)| SpotifyArtist {
                id: id.to_string(),
                name: name.to_string(),
                external_urls: HashMap::new(),
                href: None,
                uri: format!("spotify:artist:{}", id),
                genres: None,
                images: None,
                popularity: None,
                followers: None,
            }).collect(),
            album: SpotifyAlbum {
                id: "test_album".to_string(),
                name: "Test Album".to_string(),
                artists: vec![],
                album_type: "album".to_string(),
                total_tracks: 1,
                external_urls: HashMap::new(),
                images: vec![],
                release_date: "2023-01-01".to_string(),
                release_date_precision: "day".to_string(),
            },
            duration_ms: 180000,
            explicit: false,
            popularity: Some(50),
            preview_url: None,
            external_urls: HashMap::new(),
            is_local: false,
            is_playable: Some(true),
        }
    }

    #[tokio::test]
    async fn test_featured_artist_detection_from_title() {
        let spotify_service = Arc::new(SpotifyService::new(
            crate::services::spotify::SpotifyConfig::default(),
            Arc::new(crate::services::TokenVaultService::new()),
        ).unwrap());
        
        let service = SpotifyLibraryService::new(spotify_service);
        
        let track = create_test_track(
            "Song Title (feat. Featured Artist)",
            vec![("artist1", "Main Artist"), ("artist2", "Featured Artist")]
        );
        
        let detection = service.detect_featured_artists(&track);
        
        assert_eq!(detection.detection_method, DetectionMethod::TrackTitle);
        assert!(detection.confidence > 0.8);
        assert_eq!(detection.featured_artists.len(), 1);
        assert_eq!(detection.featured_artists[0], "artist2");
    }

    #[tokio::test]
    async fn test_collaboration_detection() {
        let spotify_service = Arc::new(SpotifyService::new(
            crate::services::spotify::SpotifyConfig::default(),
            Arc::new(crate::services::TokenVaultService::new()),
        ).unwrap());
        
        let service = SpotifyLibraryService::new(spotify_service);
        
        let track = create_test_track(
            "Collaboration Song",
            vec![("artist1", "Artist One"), ("artist2", "Artist Two")]
        );
        
        let detection = service.detect_featured_artists(&track);
        
        assert_eq!(detection.collaboration_artists.len(), 1);
        assert_eq!(detection.collaboration_artists[0], "artist2");
        assert_eq!(detection.primary_artists[0], "artist1");
    }
}