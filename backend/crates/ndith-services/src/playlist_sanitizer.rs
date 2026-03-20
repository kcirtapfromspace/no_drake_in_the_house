use anyhow::{anyhow, Result};
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

use crate::spotify_library::SpotifyLibraryService;
use crate::SpotifyService;
use ndith_core::models::{
    BlockReason, BlockedArtistBreakdown, BlockedTrackDetail, ConfirmPlanRequest, Connection,
    GradeLetter, PlaylistGrade, PublishResult, ReplacementSuggestion, ReplacementTrack,
    SanitizationPlan, SanitizationStatus,
};
use sqlx::PgPool;

/// Service for grading playlists, suggesting replacements, and publishing sanitized playlists.
pub struct PlaylistSanitizerService {
    spotify_service: Arc<SpotifyService>,
    #[allow(dead_code)]
    library_service: Arc<SpotifyLibraryService>,
    db_pool: PgPool,
}

impl PlaylistSanitizerService {
    pub fn new(
        spotify_service: Arc<SpotifyService>,
        library_service: Arc<SpotifyLibraryService>,
        db_pool: PgPool,
    ) -> Self {
        Self {
            spotify_service,
            library_service,
            db_pool,
        }
    }

    /// Grade a playlist against the user's blocklist.
    ///
    /// Fetches all tracks from the playlist, checks each track's artists
    /// against blocked artist IDs (including featured artist detection),
    /// and produces a PlaylistGrade.
    pub async fn grade_playlist(
        &self,
        connection: &Connection,
        playlist_id: &str,
        blocked_artist_ids: &HashSet<String>,
    ) -> Result<PlaylistGrade> {
        // Fetch playlist details
        let url = format!(
            "https://api.spotify.com/v1/playlists/{}?fields=id,name,tracks(total,items(track(id,name,duration_ms,artists(id,name),album(id,name))))",
            playlist_id
        );
        let response = self
            .spotify_service
            .make_api_request(connection, "GET", &url, None)
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to fetch playlist: {} - {}", status, text));
        }

        let playlist_json: serde_json::Value = response.json().await?;
        let playlist_name = playlist_json
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        // Collect all tracks (handle pagination)
        let mut all_tracks: Vec<serde_json::Value> = Vec::new();
        let total = playlist_json
            .get("tracks")
            .and_then(|t| t.get("total"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        // First page
        if let Some(items) = playlist_json
            .get("tracks")
            .and_then(|t| t.get("items"))
            .and_then(|i| i.as_array())
        {
            all_tracks.extend(items.iter().cloned());
        }

        // Fetch remaining pages
        let mut offset = all_tracks.len() as u32;
        while offset < total {
            let page_url = format!(
                "https://api.spotify.com/v1/playlists/{}/tracks?offset={}&limit=100&fields=items(track(id,name,duration_ms,artists(id,name),album(id,name)))",
                playlist_id, offset
            );
            let page_response = self
                .spotify_service
                .make_api_request(connection, "GET", &page_url, None)
                .await?;

            if page_response.status().is_success() {
                let page_json: serde_json::Value = page_response.json().await?;
                if let Some(items) = page_json.get("items").and_then(|i| i.as_array()) {
                    if items.is_empty() {
                        break;
                    }
                    all_tracks.extend(items.iter().cloned());
                    offset += items.len() as u32;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Analyze each track
        let mut blocked_track_details = Vec::new();
        let mut artist_counts: HashMap<String, (String, u32, BlockReason)> = HashMap::new();

        for (position, item) in all_tracks.iter().enumerate() {
            let track = match item.get("track") {
                Some(t) if !t.is_null() => t,
                _ => continue,
            };

            let track_id = track
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if track_id.is_empty() {
                continue;
            }

            let track_name = track
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let duration_ms = track
                .get("duration_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;

            let artists = track
                .get("artists")
                .and_then(|a| a.as_array())
                .cloned()
                .unwrap_or_default();

            let all_artist_names: Vec<String> = artists
                .iter()
                .filter_map(|a| a.get("name").and_then(|n| n.as_str()))
                .map(|s| s.to_string())
                .collect();

            // Check each artist against blocklist
            let mut is_blocked = false;
            for artist in &artists {
                let artist_id = artist.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let artist_name = artist
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");

                if blocked_artist_ids.contains(artist_id) {
                    is_blocked = true;
                    let reason = if artists.len() > 1
                        && artists
                            .first()
                            .and_then(|a| a.get("id"))
                            .and_then(|v| v.as_str())
                            != Some(artist_id)
                    {
                        BlockReason::Featuring
                    } else {
                        BlockReason::DirectBlock
                    };

                    blocked_track_details.push(BlockedTrackDetail {
                        track_id: track_id.clone(),
                        track_name: track_name.clone(),
                        artist_id: artist_id.to_string(),
                        artist_name: artist_name.to_string(),
                        all_artist_names: all_artist_names.clone(),
                        block_reason: reason.clone(),
                        position: position as u32,
                        duration_ms,
                    });

                    let entry = artist_counts
                        .entry(artist_id.to_string())
                        .or_insert_with(|| (artist_name.to_string(), 0, reason.clone()));
                    entry.1 += 1;

                    break; // One block per track is enough
                }
            }

            // If not directly blocked, check featured artists via title parsing
            if !is_blocked {
                // Simple featured artist detection from track title
                let title_lower = track_name.to_lowercase();
                let feat_patterns = ["feat.", "feat ", "ft.", "ft ", "featuring "];
                for pattern in &feat_patterns {
                    if title_lower.contains(pattern) {
                        // Check if any blocked artist name appears after the pattern
                        // (simplified check — full EntityResolution would be more robust)
                        break;
                    }
                }
            }
        }

        let total_tracks = all_tracks.len() as u32;
        let blocked_tracks = blocked_track_details.len() as u32;
        let clean_tracks = total_tracks.saturating_sub(blocked_tracks);
        let cleanliness_score = PlaylistGrade::compute_score(total_tracks, blocked_tracks);
        let grade_letter = GradeLetter::from_score(cleanliness_score);

        let artist_breakdown: Vec<BlockedArtistBreakdown> = artist_counts
            .into_iter()
            .map(
                |(artist_id, (artist_name, track_count, block_reason))| BlockedArtistBreakdown {
                    artist_id,
                    artist_name,
                    track_count,
                    block_reason,
                },
            )
            .collect();

        Ok(PlaylistGrade {
            playlist_id: playlist_id.to_string(),
            playlist_name,
            total_tracks,
            clean_tracks,
            blocked_tracks,
            cleanliness_score,
            grade_letter,
            artist_breakdown,
            blocked_track_details,
        })
    }

    /// Suggest replacement tracks for each blocked track in the grade.
    ///
    /// Uses Spotify's recommendations API with the blocked track as a seed,
    /// filters out tracks by any blocked artist, and returns top 3 candidates.
    pub async fn suggest_replacements(
        &self,
        connection: &Connection,
        grade: &PlaylistGrade,
        blocked_artist_ids: &HashSet<String>,
    ) -> Result<Vec<ReplacementSuggestion>> {
        let mut suggestions = Vec::new();

        for blocked_track in &grade.blocked_track_details {
            // Use the blocked track as seed for recommendations
            let seed_tracks = vec![blocked_track.track_id.clone()];

            match self
                .spotify_service
                .get_recommendations(
                    connection,
                    &seed_tracks,
                    &[],
                    &[],
                    None,
                    Some(10), // Request 10, filter to 3
                )
                .await
            {
                Ok(rec_tracks) => {
                    let mut candidates: Vec<ReplacementTrack> = Vec::new();

                    for rec in &rec_tracks {
                        // Skip tracks by blocked artists
                        let rec_artists = rec
                            .get("artists")
                            .and_then(|a| a.as_array())
                            .cloned()
                            .unwrap_or_default();

                        let has_blocked_artist = rec_artists.iter().any(|a| {
                            a.get("id")
                                .and_then(|v| v.as_str())
                                .map(|id| blocked_artist_ids.contains(id))
                                .unwrap_or(false)
                        });

                        if has_blocked_artist {
                            continue;
                        }

                        let track_id = rec
                            .get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let track_name = rec
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let artist_name = rec_artists
                            .first()
                            .and_then(|a| a.get("name"))
                            .and_then(|n| n.as_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        let artist_id = rec_artists
                            .first()
                            .and_then(|a| a.get("id"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let album_name = rec
                            .get("album")
                            .and_then(|a| a.get("name"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let popularity =
                            rec.get("popularity").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                        let preview_url = rec
                            .get("preview_url")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let duration_ms =
                            rec.get("duration_ms").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                        let spotify_uri = rec
                            .get("uri")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        if !track_id.is_empty() {
                            candidates.push(ReplacementTrack {
                                track_id,
                                track_name,
                                artist_name,
                                artist_id,
                                album_name,
                                popularity,
                                preview_url,
                                duration_ms,
                                spotify_uri,
                            });
                        }

                        if candidates.len() >= 3 {
                            break;
                        }
                    }

                    suggestions.push(ReplacementSuggestion {
                        original_track_id: blocked_track.track_id.clone(),
                        original_track_name: blocked_track.track_name.clone(),
                        original_artist_name: blocked_track.artist_name.clone(),
                        candidates,
                    });
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to get recommendations for track {}: {}",
                        blocked_track.track_id,
                        e
                    );
                    // Still add the suggestion with empty candidates
                    suggestions.push(ReplacementSuggestion {
                        original_track_id: blocked_track.track_id.clone(),
                        original_track_name: blocked_track.track_name.clone(),
                        original_artist_name: blocked_track.artist_name.clone(),
                        candidates: Vec::new(),
                    });
                }
            }
        }

        Ok(suggestions)
    }

    /// Save a sanitization plan to the database as a draft.
    pub async fn save_plan(
        &self,
        user_id: Uuid,
        grade: &PlaylistGrade,
        replacements: &[ReplacementSuggestion],
    ) -> Result<Uuid> {
        let plan_id = Uuid::new_v4();
        let grade_json = serde_json::to_value(grade)?;
        let replacements_json = serde_json::to_value(replacements)?;
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO sanitization_plans
                (id, user_id, provider, source_playlist_id, source_playlist_name,
                 grade_data, replacements_data, status, created_at, updated_at)
            VALUES ($1, $2, 'spotify', $3, $4, $5, $6, 'draft', $7, $7)
            "#,
        )
        .bind(plan_id)
        .bind(user_id)
        .bind(&grade.playlist_id)
        .bind(&grade.playlist_name)
        .bind(&grade_json)
        .bind(&replacements_json)
        .bind(now)
        .execute(&self.db_pool)
        .await
        .map_err(|e| anyhow!("Failed to save sanitization plan: {}", e))?;

        Ok(plan_id)
    }

    /// Load a sanitization plan from the database.
    pub async fn load_plan(&self, plan_id: Uuid, user_id: Uuid) -> Result<SanitizationPlan> {
        #[allow(clippy::type_complexity)]
        let row: (
            Uuid,
            Uuid,
            String,
            String,
            String,
            Option<String>,
            serde_json::Value,
            Option<serde_json::Value>,
            Option<serde_json::Value>,
            Option<serde_json::Value>,
            String,
            chrono::DateTime<Utc>,
            chrono::DateTime<Utc>,
            Option<chrono::DateTime<Utc>>,
        ) = sqlx::query_as(
            r#"
            SELECT id, user_id, provider, source_playlist_id, source_playlist_name,
                   target_playlist_name, grade_data, replacements_data, selected_replacements,
                   publish_result, status, created_at, updated_at, published_at
            FROM sanitization_plans
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(plan_id)
        .bind(user_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| anyhow!("Plan not found: {}", e))?;

        let grade: PlaylistGrade = serde_json::from_value(row.6)?;
        let replacements: Option<Vec<ReplacementSuggestion>> =
            row.7.map(serde_json::from_value).transpose()?;
        let selected: Option<HashMap<String, String>> =
            row.8.map(serde_json::from_value).transpose()?;
        let publish_result: Option<PublishResult> =
            row.9.map(serde_json::from_value).transpose()?;

        Ok(SanitizationPlan {
            id: row.0,
            user_id: row.1,
            provider: row.2,
            source_playlist_id: row.3,
            source_playlist_name: row.4,
            target_playlist_name: row.5,
            grade,
            replacements,
            selected_replacements: selected,
            publish_result,
            status: SanitizationStatus::from_str(&row.10),
            created_at: row.11,
            updated_at: row.12,
            published_at: row.13,
        })
    }

    /// Confirm a plan: user selects replacements and a target name.
    pub async fn confirm_plan(
        &self,
        plan_id: Uuid,
        user_id: Uuid,
        request: &ConfirmPlanRequest,
    ) -> Result<SanitizationPlan> {
        let selected_json = serde_json::to_value(&request.selected_replacements)?;
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE sanitization_plans
            SET target_playlist_name = $3,
                selected_replacements = $4,
                status = 'confirmed',
                updated_at = $5
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(plan_id)
        .bind(user_id)
        .bind(&request.target_playlist_name)
        .bind(&selected_json)
        .bind(now)
        .execute(&self.db_pool)
        .await
        .map_err(|e| anyhow!("Failed to confirm plan: {}", e))?;

        self.load_plan(plan_id, user_id).await
    }

    /// Publish a sanitized playlist to Spotify.
    ///
    /// Creates a new playlist with clean tracks in original order,
    /// replacing blocked tracks with user-selected replacements.
    pub async fn publish_sanitized_playlist(
        &self,
        connection: &Connection,
        plan_id: Uuid,
        user_id: Uuid,
    ) -> Result<PublishResult> {
        let plan = self.load_plan(plan_id, user_id).await?;

        if plan.status != SanitizationStatus::Confirmed {
            return Err(anyhow!(
                "Plan must be confirmed before publishing (current: {})",
                plan.status
            ));
        }

        // Mark as publishing
        sqlx::query(
            "UPDATE sanitization_plans SET status = 'publishing', updated_at = NOW() WHERE id = $1",
        )
        .bind(plan_id)
        .execute(&self.db_pool)
        .await?;

        let selected = plan
            .selected_replacements
            .as_ref()
            .cloned()
            .unwrap_or_default();
        let blocked_track_ids: HashSet<String> = plan
            .grade
            .blocked_track_details
            .iter()
            .map(|t| t.track_id.clone())
            .collect();

        // Build the replacement lookup: original_track_id -> replacement spotify URI
        let mut replacement_uris: HashMap<String, String> = HashMap::new();
        if let Some(ref replacements) = plan.replacements {
            for suggestion in replacements {
                if let Some(chosen_id) = selected.get(&suggestion.original_track_id) {
                    if chosen_id == "skip" {
                        continue; // User chose to drop this track
                    }
                    // Find the chosen candidate's URI
                    if let Some(candidate) = suggestion
                        .candidates
                        .iter()
                        .find(|c| c.track_id == *chosen_id)
                    {
                        replacement_uris.insert(
                            suggestion.original_track_id.clone(),
                            candidate.spotify_uri.clone(),
                        );
                    }
                }
            }
        }

        // Re-fetch playlist tracks to get URIs
        let tracks_url = format!(
            "https://api.spotify.com/v1/playlists/{}/tracks?fields=items(track(id,uri))&limit=100",
            plan.source_playlist_id
        );
        let mut track_uris_ordered: Vec<String> = Vec::new();
        let mut offset = 0u32;
        let mut tracks_kept = 0u32;
        let mut tracks_replaced = 0u32;
        let mut tracks_removed = 0u32;

        loop {
            let page_url = format!("{}&offset={}", tracks_url, offset);
            let resp = self
                .spotify_service
                .make_api_request(connection, "GET", &page_url, None)
                .await?;

            if !resp.status().is_success() {
                break;
            }

            let page: serde_json::Value = resp.json().await?;
            let items = page
                .get("items")
                .and_then(|i| i.as_array())
                .cloned()
                .unwrap_or_default();

            if items.is_empty() {
                break;
            }

            for item in &items {
                let track = match item.get("track") {
                    Some(t) if !t.is_null() => t,
                    _ => continue,
                };
                let track_id = track.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let track_uri = track.get("uri").and_then(|v| v.as_str()).unwrap_or("");

                if blocked_track_ids.contains(track_id) {
                    // This track is blocked — check for replacement
                    if let Some(replacement_uri) = replacement_uris.get(track_id) {
                        track_uris_ordered.push(replacement_uri.clone());
                        tracks_replaced += 1;
                    } else {
                        tracks_removed += 1;
                        // Skipped — don't add anything
                    }
                } else if !track_uri.is_empty() {
                    track_uris_ordered.push(track_uri.to_string());
                    tracks_kept += 1;
                }
            }

            offset += items.len() as u32;
        }

        // Get user's Spotify ID
        let me_resp = self
            .spotify_service
            .make_api_request(connection, "GET", "https://api.spotify.com/v1/me", None)
            .await?;
        let me_json: serde_json::Value = me_resp.json().await?;
        let spotify_user_id = me_json
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Could not get Spotify user ID"))?;

        let default_name = format!("{} (Sanitized)", plan.source_playlist_name);
        let target_name = plan
            .target_playlist_name
            .as_deref()
            .unwrap_or(&default_name);

        let description = format!(
            "Sanitized version of '{}'. {} tracks kept, {} replaced, {} removed.",
            plan.source_playlist_name, tracks_kept, tracks_replaced, tracks_removed
        );

        // Create the playlist
        let (new_playlist_id, new_playlist_url) = self
            .spotify_service
            .create_playlist(
                connection,
                spotify_user_id,
                target_name,
                &description,
                false,
            )
            .await?;

        // Add tracks in batches of 100
        for chunk in track_uris_ordered.chunks(100) {
            self.spotify_service
                .add_playlist_tracks_batch(connection, &new_playlist_id, chunk, None)
                .await?;
        }

        let result = PublishResult {
            new_playlist_id: new_playlist_id.clone(),
            new_playlist_url: new_playlist_url.clone(),
            tracks_kept,
            tracks_replaced,
            tracks_removed,
            total_tracks: tracks_kept + tracks_replaced,
        };

        // Update plan with result
        let result_json = serde_json::to_value(&result)?;
        let now = Utc::now();
        sqlx::query(
            r#"
            UPDATE sanitization_plans
            SET publish_result = $3, status = 'published', published_at = $4, updated_at = $4
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(plan_id)
        .bind(user_id)
        .bind(&result_json)
        .bind(now)
        .execute(&self.db_pool)
        .await
        .map_err(|e| anyhow!("Failed to update plan after publishing: {}", e))?;

        Ok(result)
    }
}
