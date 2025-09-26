use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// User's personal DNP (Do Not Play) list entry
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserArtistBlock {
    pub user_id: Uuid,
    pub artist_id: Uuid,
    pub tags: Vec<String>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Request to add an artist to DNP list
#[derive(Debug, Deserialize)]
pub struct AddArtistToDnpRequest {
    pub artist_query: String, // Artist name or provider URL
    pub provider: Option<String>, // "spotify", "apple", "youtube", "tidal"
    pub tags: Option<Vec<String>>,
    pub note: Option<String>,
}

/// Request to update DNP list entry
#[derive(Debug, Deserialize)]
pub struct UpdateDnpEntryRequest {
    pub tags: Option<Vec<String>>,
    pub note: Option<String>,
}

/// Request for bulk import
#[derive(Debug, Deserialize)]
pub struct BulkImportRequest {
    pub format: ImportFormat,
    pub data: String, // CSV or JSON data
    pub overwrite_existing: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportFormat {
    Csv,
    Json,
}

/// Bulk import entry
#[derive(Debug, Deserialize, Serialize)]
pub struct ImportEntry {
    pub artist_name: String,
    pub provider_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub note: Option<String>,
}

/// CSV import entry (tags as semicolon-separated string)
#[derive(Debug, Deserialize)]
pub struct CsvImportEntry {
    pub artist_name: String,
    pub provider_url: Option<String>,
    pub tags: Option<String>, // Semicolon-separated tags
    pub note: Option<String>,
}

/// DNP list export format
#[derive(Debug, Serialize)]
pub struct DnpListExport {
    pub exported_at: DateTime<Utc>,
    pub total_entries: usize,
    pub entries: Vec<DnpExportEntry>,
}

#[derive(Debug, Serialize)]
pub struct DnpExportEntry {
    pub artist_name: String,
    pub external_ids: serde_json::Value,
    pub tags: Vec<String>,
    pub note: Option<String>,
    pub added_at: DateTime<Utc>,
}

/// Response for artist search with provider badges
#[derive(Debug, Serialize)]
pub struct ArtistSearchResponse {
    pub artists: Vec<ArtistSearchResult>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct ArtistSearchResult {
    pub id: Uuid,
    pub canonical_name: String,
    pub image_url: Option<String>,
    pub provider_badges: Vec<ProviderBadge>,
    pub popularity: Option<u32>,
    pub genres: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ProviderBadge {
    pub provider: String,
    pub verified: bool,
    pub follower_count: Option<u64>,
}

/// DNP list with artist details
#[derive(Debug, Serialize)]
pub struct DnpListResponse {
    pub entries: Vec<DnpListEntry>,
    pub total: usize,
    pub tags: Vec<String>, // All unique tags used
}

#[derive(Debug, Serialize)]
pub struct DnpListEntry {
    pub artist_id: Uuid,
    pub artist_name: String,
    pub image_url: Option<String>,
    pub provider_badges: Vec<ProviderBadge>,
    pub tags: Vec<String>,
    pub note: Option<String>,
    pub added_at: DateTime<Utc>,
}

/// Bulk operation result
#[derive(Debug, Serialize)]
pub struct BulkOperationResult {
    pub total_processed: usize,
    pub successful: usize,
    pub failed: usize,
    pub errors: Vec<BulkOperationError>,
}

#[derive(Debug, Serialize)]
pub struct BulkOperationError {
    pub entry_index: usize,
    pub artist_name: String,
    pub error: String,
}