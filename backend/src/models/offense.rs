use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Offense category types matching database enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "offense_category", rename_all = "snake_case")]
pub enum OffenseCategory {
    DomesticViolence,
    SexualMisconduct,
    SexualAssault,
    ChildAbuse,
    HateSpeech,
    Racism,
    Homophobia,
    Antisemitism,
    ViolentCrime,
    DrugTrafficking,
    Fraud,
    AnimalAbuse,
    Other,
}

impl OffenseCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::DomesticViolence => "Domestic Violence",
            Self::SexualMisconduct => "Sexual Misconduct",
            Self::SexualAssault => "Sexual Assault",
            Self::ChildAbuse => "Child Abuse",
            Self::HateSpeech => "Hate Speech",
            Self::Racism => "Racism",
            Self::Homophobia => "Homophobia",
            Self::Antisemitism => "Antisemitism",
            Self::ViolentCrime => "Violent Crime",
            Self::DrugTrafficking => "Drug Trafficking",
            Self::Fraud => "Fraud",
            Self::AnimalAbuse => "Animal Abuse",
            Self::Other => "Other",
        }
    }
}

/// Evidence verification status matching database enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "evidence_status", rename_all = "snake_case")]
pub enum EvidenceStatus {
    Pending,
    Verified,
    Disputed,
    Rejected,
}

/// Offense severity levels matching database enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "offense_severity", rename_all = "snake_case")]
pub enum OffenseSeverity {
    Minor,
    Moderate,
    Severe,
    Egregious,
}

impl OffenseSeverity {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Minor => "Controversial statements",
            Self::Moderate => "Arrests, allegations",
            Self::Severe => "Convictions, proven abuse",
            Self::Egregious => "Multiple severe offenses, ongoing patterns",
        }
    }
}

/// Artist offense record from database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ArtistOffense {
    pub id: Uuid,
    pub artist_id: Uuid,
    pub category: OffenseCategory,
    pub severity: OffenseSeverity,
    pub title: String,
    pub description: String,
    pub incident_date: Option<NaiveDate>,
    pub incident_date_approximate: bool,
    pub arrested: bool,
    pub charged: bool,
    pub convicted: bool,
    pub settled: bool,
    pub status: EvidenceStatus,
    pub verified_at: Option<DateTime<Utc>>,
    pub verified_by: Option<Uuid>,
    pub submitted_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Evidence link for an offense
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OffenseEvidence {
    pub id: Uuid,
    pub offense_id: Uuid,
    pub url: String,
    pub source_name: Option<String>,
    pub source_type: Option<String>,
    pub title: Option<String>,
    pub excerpt: Option<String>,
    pub published_date: Option<NaiveDate>,
    pub archived_url: Option<String>,
    pub is_primary_source: bool,
    pub credibility_score: Option<i32>,
    pub submitted_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Request to create a new offense
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOffenseRequest {
    pub artist_id: Uuid,
    pub category: OffenseCategory,
    pub severity: OffenseSeverity,
    pub title: String,
    pub description: String,
    pub incident_date: Option<NaiveDate>,
    pub incident_date_approximate: Option<bool>,
    pub arrested: Option<bool>,
    pub charged: Option<bool>,
    pub convicted: Option<bool>,
    pub settled: Option<bool>,
}

/// Request to add evidence to an offense
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddEvidenceRequest {
    pub offense_id: Uuid,
    pub url: String,
    pub source_name: Option<String>,
    pub source_type: Option<String>,
    pub title: Option<String>,
    pub excerpt: Option<String>,
    pub published_date: Option<NaiveDate>,
    pub is_primary_source: Option<bool>,
    pub credibility_score: Option<i32>,
}

/// Offense with evidence and artist info for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffenseWithEvidence {
    pub offense: ArtistOffense,
    pub evidence: Vec<OffenseEvidence>,
    pub artist_name: String,
}

/// Flagged artist summary for library scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlaggedArtist {
    pub id: Uuid,
    pub name: String,
    pub track_count: i32,
    pub severity: OffenseSeverity,
    pub offenses: Vec<OffenseSummary>,
}

/// Offense summary for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffenseSummary {
    pub category: OffenseCategory,
    pub title: String,
    pub date: String,
    pub evidence_count: i32,
}

/// Library scan results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryScanResponse {
    pub total_tracks: i32,
    pub total_artists: i32,
    pub flagged_artists: Vec<FlaggedArtist>,
    pub flagged_tracks: i32,
}

/// User's library track from streaming service
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserLibraryTrack {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_track_id: String,
    pub track_name: Option<String>,
    pub album_name: Option<String>,
    pub artist_id: Option<Uuid>,
    pub artist_name: Option<String>,
    pub source_type: Option<String>,
    pub playlist_name: Option<String>,
    pub added_at: Option<DateTime<Utc>>,
    pub last_synced: DateTime<Utc>,
}

/// Request to add tracks to user library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportLibraryRequest {
    pub provider: String,
    pub tracks: Vec<ImportTrack>,
}

/// Single track to import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportTrack {
    pub provider_track_id: String,
    pub track_name: String,
    pub album_name: Option<String>,
    pub artist_name: String,
    pub source_type: Option<String>,
    pub playlist_name: Option<String>,
    pub added_at: Option<DateTime<Utc>>,
}
