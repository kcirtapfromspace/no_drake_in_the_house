use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Artist {
    pub id: Uuid,
    pub canonical_name: String,
    pub canonical_artist_id: Option<Uuid>,
    pub external_ids: serde_json::Value,
    pub metadata: serde_json::Value,
    pub aliases: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SearchArtistRequest {
    pub query: String,
    pub provider: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ArtistResponse {
    pub id: Uuid,
    pub canonical_name: String,
    pub external_ids: serde_json::Value,
    pub metadata: serde_json::Value,
}

impl From<Artist> for ArtistResponse {
    fn from(artist: Artist) -> Self {
        ArtistResponse {
            id: artist.id,
            canonical_name: artist.canonical_name,
            external_ids: artist.external_ids,
            metadata: artist.metadata,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserArtistBlock {
    pub user_id: Uuid,
    pub artist_id: Uuid,
    pub tags: Vec<String>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AddArtistToDnpRequest {
    pub artist_id: Uuid,
    pub tags: Option<Vec<String>>,
    pub note: Option<String>,
}