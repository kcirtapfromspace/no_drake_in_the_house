//! MusicBrainz Bulk Import Service
//!
//! Imports artist metadata from MusicBrainz for broader catalog coverage.
//! Rate limit: 1 request/second (requires proper User-Agent)

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

use super::traits::{Platform, PlatformArtist};

/// MusicBrainz API base URL
const MUSICBRAINZ_API_BASE: &str = "https://musicbrainz.org/ws/2";

/// User agent required by MusicBrainz API
const USER_AGENT: &str = "NoDrakeInTheHouse/1.0 (https://github.com/no-drake-in-the-house)";

/// MusicBrainz artist import service
pub struct MusicBrainzImporter {
    client: Client,
    db_pool: PgPool,
    /// Rate limiter - last request time
    last_request: Arc<RwLock<Option<std::time::Instant>>>,
}

/// Import statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MusicBrainzImportStats {
    pub artists_fetched: usize,
    pub artists_imported: usize,
    pub artists_skipped: usize,
    pub errors: usize,
}

/// MusicBrainz artist response
#[derive(Debug, Deserialize)]
struct MusicBrainzSearchResponse {
    artists: Vec<MusicBrainzArtist>,
    count: i64,
    offset: i64,
}

/// MusicBrainz artist
#[derive(Debug, Clone, Deserialize)]
pub struct MusicBrainzArtist {
    pub id: String,
    pub name: String,
    #[serde(rename = "sort-name")]
    pub sort_name: Option<String>,
    pub country: Option<String>,
    pub disambiguation: Option<String>,
    #[serde(rename = "type")]
    pub artist_type: Option<String>,
    #[serde(rename = "life-span")]
    pub life_span: Option<MusicBrainzLifeSpan>,
    pub tags: Option<Vec<MusicBrainzTag>>,
    pub aliases: Option<Vec<MusicBrainzAlias>>,
    #[serde(rename = "isnis")]
    pub isnis: Option<Vec<String>>,
    pub score: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MusicBrainzLifeSpan {
    pub begin: Option<String>,
    pub end: Option<String>,
    pub ended: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MusicBrainzTag {
    pub name: String,
    pub count: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MusicBrainzAlias {
    pub name: String,
    #[serde(rename = "sort-name")]
    pub sort_name: Option<String>,
    #[serde(rename = "type")]
    pub alias_type: Option<String>,
    pub primary: Option<bool>,
}

impl MusicBrainzImporter {
    /// Create a new MusicBrainz importer
    pub fn new(db_pool: PgPool) -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            db_pool,
            last_request: Arc::new(RwLock::new(None)),
        }
    }

    /// Wait for rate limit (1 request/second)
    async fn wait_for_rate_limit(&self) {
        let mut last = self.last_request.write().await;
        if let Some(last_time) = *last {
            let elapsed = last_time.elapsed();
            if elapsed < Duration::from_millis(1100) {
                // 1.1 seconds to be safe
                sleep(Duration::from_millis(1100) - elapsed).await;
            }
        }
        *last = Some(std::time::Instant::now());
    }

    /// Fetch artists from MusicBrainz with search query
    pub async fn search_artists(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<MusicBrainzArtist>> {
        self.wait_for_rate_limit().await;

        let url = format!(
            "{}/artist?query={}&limit={}&offset={}&fmt=json",
            MUSICBRAINZ_API_BASE,
            urlencoding::encode(query),
            limit.min(100),
            offset
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("MusicBrainz API request failed")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "MusicBrainz API error: {} - {}",
                status,
                body
            ));
        }

        let result: MusicBrainzSearchResponse = response
            .json()
            .await
            .context("Failed to parse MusicBrainz response")?;

        Ok(result.artists)
    }

    /// Fetch popular artists by querying with genre tags
    /// Uses tags like "pop", "rock", "hip hop" etc. and sorts by relevance
    pub async fn fetch_popular_artists(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<MusicBrainzArtist>> {
        // Query for artists with type "Person" or "Group" (not "Other", "Character", etc.)
        let query = "type:person OR type:group";
        self.search_artists(query, limit, offset).await
    }

    /// Fetch artists by genre/tag
    pub async fn fetch_artists_by_tag(
        &self,
        tag: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<MusicBrainzArtist>> {
        let query = format!("tag:{}", tag);
        self.search_artists(&query, limit, offset).await
    }

    /// Fetch artists by country
    pub async fn fetch_artists_by_country(
        &self,
        country_code: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<MusicBrainzArtist>> {
        let query = format!("country:{}", country_code);
        self.search_artists(&query, limit, offset).await
    }

    /// Check if artist already exists in database
    async fn artist_exists(&self, musicbrainz_id: &str, name: &str) -> Result<bool> {
        // Check by MusicBrainz ID in external_ids JSONB
        let exists: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT 1 FROM artists
            WHERE external_ids->>'musicbrainz' = $1
               OR LOWER(canonical_name) = LOWER($2)
            LIMIT 1
            "#,
        )
        .bind(musicbrainz_id)
        .bind(name)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(exists.is_some())
    }

    /// Import a batch of MusicBrainz artists into the database
    pub async fn import_batch(
        &self,
        artists: Vec<MusicBrainzArtist>,
        skip_existing: bool,
    ) -> Result<MusicBrainzImportStats> {
        let mut stats = MusicBrainzImportStats {
            artists_fetched: artists.len(),
            ..Default::default()
        };

        for artist in artists {
            // Skip non-person/group types
            if let Some(ref artist_type) = artist.artist_type {
                if !["Person", "Group"].contains(&artist_type.as_str()) {
                    stats.artists_skipped += 1;
                    continue;
                }
            }

            // Check if already exists
            if skip_existing {
                match self.artist_exists(&artist.id, &artist.name).await {
                    Ok(true) => {
                        stats.artists_skipped += 1;
                        continue;
                    }
                    Ok(false) => {}
                    Err(e) => {
                        tracing::warn!("Failed to check if artist exists: {}", e);
                        stats.errors += 1;
                        continue;
                    }
                }
            }

            // Build external_ids JSONB
            let mut external_ids = serde_json::json!({
                "musicbrainz": artist.id
            });

            // Add ISNI if available
            if let Some(isnis) = &artist.isnis {
                if let Some(first_isni) = isnis.first() {
                    external_ids["isni"] = serde_json::json!(first_isni);
                }
            }

            // Build metadata JSONB
            let mut metadata = serde_json::json!({});
            if let Some(country) = &artist.country {
                metadata["country"] = serde_json::json!(country);
            }
            if let Some(artist_type) = &artist.artist_type {
                metadata["type"] = serde_json::json!(artist_type);
            }
            if let Some(disambiguation) = &artist.disambiguation {
                metadata["disambiguation"] = serde_json::json!(disambiguation);
            }

            // Extract genre tags
            let genres: Vec<String> = artist
                .tags
                .as_ref()
                .map(|tags| {
                    let mut sorted_tags = tags.clone();
                    sorted_tags.sort_by(|a, b| b.count.cmp(&a.count));
                    sorted_tags.into_iter().take(5).map(|t| t.name).collect()
                })
                .unwrap_or_default();

            if !genres.is_empty() {
                metadata["genres"] = serde_json::json!(genres);
            }

            // Build aliases JSONB
            let aliases: Vec<String> = artist
                .aliases
                .as_ref()
                .map(|als| als.iter().map(|a| a.name.clone()).collect())
                .unwrap_or_default();

            let aliases_json = serde_json::json!(aliases);

            // Insert into database
            match sqlx::query(
                r#"
                INSERT INTO artists (canonical_name, external_ids, metadata, aliases)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (canonical_name) DO UPDATE SET
                    external_ids = artists.external_ids || $2,
                    metadata = artists.metadata || $3,
                    aliases = (
                        SELECT jsonb_agg(DISTINCT x)
                        FROM (
                            SELECT jsonb_array_elements(artists.aliases) AS x
                            UNION
                            SELECT jsonb_array_elements($4) AS x
                        ) sub
                    )
                "#,
            )
            .bind(&artist.name)
            .bind(&external_ids)
            .bind(&metadata)
            .bind(&aliases_json)
            .execute(&self.db_pool)
            .await
            {
                Ok(_) => {
                    stats.artists_imported += 1;
                }
                Err(e) => {
                    tracing::warn!("Failed to import artist '{}': {}", artist.name, e);
                    stats.errors += 1;
                }
            }
        }

        Ok(stats)
    }

    /// Bulk import artists from MusicBrainz
    /// Fetches artists by multiple genre tags to get diverse coverage
    pub async fn bulk_import(
        &self,
        target_count: usize,
        skip_existing: bool,
        progress_callback: impl Fn(usize, usize),
    ) -> Result<MusicBrainzImportStats> {
        let mut total_stats = MusicBrainzImportStats::default();
        let mut imported_names: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        // Tags to search for (popular genres)
        let tags = [
            "pop",
            "rock",
            "hip hop",
            "r&b",
            "country",
            "electronic",
            "jazz",
            "classical",
            "metal",
            "indie",
            "folk",
            "soul",
            "blues",
            "punk",
            "reggae",
            "latin",
            "alternative",
            "dance",
            "funk",
            "disco",
        ];

        let artists_per_tag = (target_count / tags.len()).max(100);
        let mut current_count = 0;

        for tag in tags {
            if current_count >= target_count {
                break;
            }

            tracing::info!(
                "Importing '{}' artists... ({}/{})",
                tag,
                current_count,
                target_count
            );

            let mut offset = 0;
            let mut tag_imported = 0;

            while tag_imported < artists_per_tag && current_count < target_count {
                match self.fetch_artists_by_tag(tag, 100, offset).await {
                    Ok(artists) => {
                        if artists.is_empty() {
                            break;
                        }

                        // Filter out already imported artists by name
                        let new_artists: Vec<MusicBrainzArtist> = artists
                            .into_iter()
                            .filter(|a| !imported_names.contains(&a.name.to_lowercase()))
                            .collect();

                        if new_artists.is_empty() {
                            offset += 100;
                            continue;
                        }

                        // Track names
                        for artist in &new_artists {
                            imported_names.insert(artist.name.to_lowercase());
                        }

                        let batch_count = new_artists.len();
                        match self.import_batch(new_artists, skip_existing).await {
                            Ok(stats) => {
                                total_stats.artists_fetched += stats.artists_fetched;
                                total_stats.artists_imported += stats.artists_imported;
                                total_stats.artists_skipped += stats.artists_skipped;
                                total_stats.errors += stats.errors;

                                current_count += stats.artists_imported;
                                tag_imported += stats.artists_imported;
                                progress_callback(current_count, target_count);
                            }
                            Err(e) => {
                                tracing::warn!("Failed to import batch: {}", e);
                                total_stats.errors += batch_count;
                            }
                        }

                        offset += 100;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to fetch '{}' artists: {}", tag, e);
                        break;
                    }
                }
            }
        }

        tracing::info!(
            "MusicBrainz import complete: {} imported, {} skipped, {} errors",
            total_stats.artists_imported,
            total_stats.artists_skipped,
            total_stats.errors
        );

        Ok(total_stats)
    }
}

impl MusicBrainzArtist {
    /// Convert to PlatformArtist
    pub fn to_platform_artist(&self) -> PlatformArtist {
        let mut external_urls = HashMap::new();
        external_urls.insert(
            "musicbrainz".to_string(),
            format!("https://musicbrainz.org/artist/{}", self.id),
        );

        let genres = self
            .tags
            .as_ref()
            .map(|tags| {
                let mut sorted = tags.clone();
                sorted.sort_by(|a, b| b.count.cmp(&a.count));
                sorted.into_iter().take(5).map(|t| t.name).collect()
            })
            .unwrap_or_default();

        let mut metadata: HashMap<String, serde_json::Value> = HashMap::new();
        metadata.insert("musicbrainz_id".to_string(), serde_json::json!(self.id));
        if let Some(country) = &self.country {
            metadata.insert("country".to_string(), serde_json::json!(country));
        }
        if let Some(isnis) = &self.isnis {
            if let Some(isni) = isnis.first() {
                metadata.insert("isni".to_string(), serde_json::json!(isni));
            }
        }

        PlatformArtist {
            platform_id: self.id.clone(),
            platform: Platform::AppleMusic, // Will be resolved during identity matching
            name: self.name.clone(),
            genres,
            popularity: self.score.map(|s| s as u64),
            image_url: None, // MusicBrainz doesn't provide images directly
            external_urls,
            metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_musicbrainz_artist_to_platform_artist() {
        let mb_artist = MusicBrainzArtist {
            id: "test-id".to_string(),
            name: "Test Artist".to_string(),
            sort_name: Some("Artist, Test".to_string()),
            country: Some("US".to_string()),
            disambiguation: None,
            artist_type: Some("Person".to_string()),
            life_span: None,
            tags: Some(vec![
                MusicBrainzTag {
                    name: "pop".to_string(),
                    count: 10,
                },
                MusicBrainzTag {
                    name: "rock".to_string(),
                    count: 5,
                },
            ]),
            aliases: None,
            isnis: Some(vec!["0000000000000001".to_string()]),
            score: Some(95),
        };

        let platform_artist = mb_artist.to_platform_artist();
        assert_eq!(platform_artist.name, "Test Artist");
        assert_eq!(platform_artist.genres, vec!["pop", "rock"]);
        assert_eq!(platform_artist.popularity, Some(95));
    }
}
