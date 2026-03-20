use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::spotify::BlockReason;

/// Grade letter for playlist cleanliness
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GradeLetter {
    A, // 90-100
    B, // 80-89
    C, // 70-79
    D, // 60-69
    F, // 0-59
}

impl GradeLetter {
    pub fn from_score(score: u32) -> Self {
        match score {
            90..=100 => GradeLetter::A,
            80..=89 => GradeLetter::B,
            70..=79 => GradeLetter::C,
            60..=69 => GradeLetter::D,
            _ => GradeLetter::F,
        }
    }
}

impl std::fmt::Display for GradeLetter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GradeLetter::A => write!(f, "A"),
            GradeLetter::B => write!(f, "B"),
            GradeLetter::C => write!(f, "C"),
            GradeLetter::D => write!(f, "D"),
            GradeLetter::F => write!(f, "F"),
        }
    }
}

/// Summary of blocked tracks by a single artist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedArtistBreakdown {
    pub artist_id: String,
    pub artist_name: String,
    pub track_count: u32,
    pub block_reason: BlockReason,
}

/// Detail about a single blocked track in a playlist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedTrackDetail {
    pub track_id: String,
    pub track_name: String,
    pub artist_id: String,
    pub artist_name: String,
    pub all_artist_names: Vec<String>,
    pub block_reason: BlockReason,
    pub position: u32,
    pub duration_ms: u32,
}

/// Grade result for a playlist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistGrade {
    pub playlist_id: String,
    pub playlist_name: String,
    pub total_tracks: u32,
    pub clean_tracks: u32,
    pub blocked_tracks: u32,
    pub cleanliness_score: u32,
    pub grade_letter: GradeLetter,
    pub artist_breakdown: Vec<BlockedArtistBreakdown>,
    pub blocked_track_details: Vec<BlockedTrackDetail>,
}

impl PlaylistGrade {
    pub fn compute_score(total: u32, blocked: u32) -> u32 {
        if total == 0 {
            return 100;
        }
        let clean = total.saturating_sub(blocked);
        ((clean as f64 / total as f64) * 100.0).round() as u32
    }
}

/// A candidate replacement track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplacementTrack {
    pub track_id: String,
    pub track_name: String,
    pub artist_name: String,
    pub artist_id: String,
    pub album_name: String,
    pub popularity: u32,
    pub preview_url: Option<String>,
    pub duration_ms: u32,
    pub spotify_uri: String,
}

/// Replacement suggestion for a single blocked track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplacementSuggestion {
    pub original_track_id: String,
    pub original_track_name: String,
    pub original_artist_name: String,
    pub candidates: Vec<ReplacementTrack>,
}

/// Status of a sanitization plan
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SanitizationStatus {
    Draft,
    Confirmed,
    Publishing,
    Published,
    Failed,
}

impl std::fmt::Display for SanitizationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SanitizationStatus::Draft => write!(f, "draft"),
            SanitizationStatus::Confirmed => write!(f, "confirmed"),
            SanitizationStatus::Publishing => write!(f, "publishing"),
            SanitizationStatus::Published => write!(f, "published"),
            SanitizationStatus::Failed => write!(f, "failed"),
        }
    }
}

impl SanitizationStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "draft" => SanitizationStatus::Draft,
            "confirmed" => SanitizationStatus::Confirmed,
            "publishing" => SanitizationStatus::Publishing,
            "published" => SanitizationStatus::Published,
            "failed" => SanitizationStatus::Failed,
            _ => SanitizationStatus::Draft,
        }
    }
}

/// Full sanitization plan persisted to DB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizationPlan {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub source_playlist_id: String,
    pub source_playlist_name: String,
    pub target_playlist_name: Option<String>,
    pub grade: PlaylistGrade,
    pub replacements: Option<Vec<ReplacementSuggestion>>,
    pub selected_replacements: Option<std::collections::HashMap<String, String>>,
    pub publish_result: Option<PublishResult>,
    pub status: SanitizationStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

/// Result of publishing a sanitized playlist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResult {
    pub new_playlist_id: String,
    pub new_playlist_url: String,
    pub tracks_kept: u32,
    pub tracks_replaced: u32,
    pub tracks_removed: u32,
    pub total_tracks: u32,
}

/// Request to grade a playlist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradePlaylistRequest {
    pub playlist_id: String,
}

/// Request to suggest replacements (grade + suggest)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestReplacementsRequest {
    pub playlist_id: String,
}

/// Request to confirm a plan with user selections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmPlanRequest {
    pub target_playlist_name: String,
    /// Map of original_track_id -> chosen replacement_track_id (or "skip" to drop)
    pub selected_replacements: std::collections::HashMap<String, String>,
}

/// Response from grading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeResponse {
    pub grade: PlaylistGrade,
}

/// Response from suggesting replacements (creates a draft plan)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestResponse {
    pub plan_id: Uuid,
    pub grade: PlaylistGrade,
    pub replacements: Vec<ReplacementSuggestion>,
}

/// Response from publishing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResponse {
    pub plan_id: Uuid,
    pub result: PublishResult,
}
