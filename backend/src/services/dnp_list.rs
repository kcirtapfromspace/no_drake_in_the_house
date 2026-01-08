use crate::models::*;
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;

use uuid::Uuid;

pub struct DnpListService {
    db_pool: PgPool,
}

impl DnpListService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Add an artist to user's DNP list by artist ID
    pub async fn add_artist_to_dnp_list(
        &self,
        user_id: Uuid,
        artist_id: Uuid,
        tags: Option<Vec<String>>,
        note: Option<String>,
    ) -> Result<DnpListEntry> {
        // Check if artist exists
        let artist_exists = sqlx::query!("SELECT id FROM artists WHERE id = $1", artist_id)
            .fetch_optional(&self.db_pool)
            .await?;

        if artist_exists.is_none() {
            return Err(anyhow!("Artist not found"));
        }

        // Check if artist is already in DNP list
        let existing = sqlx::query!(
            "SELECT user_id FROM user_artist_blocks WHERE user_id = $1 AND artist_id = $2",
            user_id,
            artist_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if existing.is_some() {
            return Err(anyhow!("Artist is already in your DNP list"));
        }

        // Add to DNP list
        let tags = tags.unwrap_or_default();
        sqlx::query!(
            "INSERT INTO user_artist_blocks (user_id, artist_id, tags, note) VALUES ($1, $2, $3, $4)",
            user_id,
            artist_id,
            &tags,
            note
        )
        .execute(&self.db_pool)
        .await?;

        // Return the created entry
        self.get_dnp_entry(user_id, artist_id).await
    }

    /// Remove an artist from user's DNP list
    pub async fn remove_artist_from_dnp_list(&self, user_id: Uuid, artist_id: Uuid) -> Result<()> {
        let result = sqlx::query!(
            "DELETE FROM user_artist_blocks WHERE user_id = $1 AND artist_id = $2",
            user_id,
            artist_id
        )
        .execute(&self.db_pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("Artist not found in DNP list"));
        }

        Ok(())
    }

    /// Update DNP list entry
    pub async fn update_dnp_entry(
        &self,
        user_id: Uuid,
        artist_id: Uuid,
        request: UpdateDnpEntryRequest,
    ) -> Result<DnpListEntry> {
        // Check if entry exists
        let existing = sqlx::query!(
            "SELECT user_id FROM user_artist_blocks WHERE user_id = $1 AND artist_id = $2",
            user_id,
            artist_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if existing.is_none() {
            return Err(anyhow!("Artist not found in DNP list"));
        }

        // Update the entry
        if let Some(tags) = &request.tags {
            sqlx::query!(
                "UPDATE user_artist_blocks SET tags = $3 WHERE user_id = $1 AND artist_id = $2",
                user_id,
                artist_id,
                tags
            )
            .execute(&self.db_pool)
            .await?;
        }

        if let Some(note) = &request.note {
            sqlx::query!(
                "UPDATE user_artist_blocks SET note = $3 WHERE user_id = $1 AND artist_id = $2",
                user_id,
                artist_id,
                note
            )
            .execute(&self.db_pool)
            .await?;
        }

        self.get_dnp_entry(user_id, artist_id).await
    }

    /// Get user's complete DNP list
    pub async fn get_user_dnp_list(&self, user_id: Uuid) -> Result<DnpListResponse> {
        let entries = sqlx::query!(
            r#"
            SELECT 
                uab.artist_id,
                uab.tags,
                uab.note,
                uab.created_at,
                a.canonical_name,
                a.external_ids,
                a.metadata
            FROM user_artist_blocks uab
            JOIN artists a ON uab.artist_id = a.id
            WHERE uab.user_id = $1
            ORDER BY uab.created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut dnp_entries = Vec::new();
        let mut all_tags = std::collections::HashSet::new();

        for row in entries {
            // Parse metadata for image and provider badges
            let metadata: serde_json::Value = row.metadata.unwrap_or_else(|| json!({}));
            let external_ids: serde_json::Value = row.external_ids.unwrap_or_else(|| json!({}));

            let image_url = metadata
                .get("image_url")
                .and_then(|v| v.as_str())
                .map(String::from);
            let provider_badges = self.create_provider_badges(&external_ids, &metadata);

            // Collect tags
            let tags = row.tags.unwrap_or_default();
            for tag in &tags {
                all_tags.insert(tag.clone());
            }

            dnp_entries.push(DnpListEntry {
                artist_id: row.artist_id,
                artist_name: row.canonical_name,
                image_url,
                provider_badges,
                tags,
                note: row.note,
                added_at: row.created_at.unwrap_or_else(|| Utc::now()),
            });
        }

        Ok(DnpListResponse {
            total: dnp_entries.len(),
            entries: dnp_entries,
            tags: all_tags.into_iter().collect(),
        })
    }

    /// Search for artists with fuzzy matching
    pub async fn search_artists(
        &self,
        query: &str,
        limit: Option<usize>,
    ) -> Result<ArtistSearchResponse> {
        let limit = limit.unwrap_or(10) as i64;

        // Use fuzzy matching with ILIKE and similarity scoring
        let artists = sqlx::query!(
            r#"
            SELECT 
                id,
                canonical_name,
                external_ids,
                metadata
            FROM artists 
            WHERE 
                canonical_name ILIKE $1 
                OR canonical_name ILIKE $2
                OR aliases::text ILIKE $1
            ORDER BY 
                CASE 
                    WHEN canonical_name ILIKE $2 THEN 1
                    WHEN canonical_name ILIKE $1 THEN 2
                    ELSE 3
                END,
                canonical_name
            LIMIT $3
            "#,
            format!("%{}%", query),
            format!("{}%", query),
            limit
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut search_results = Vec::new();
        for row in artists {
            let metadata: serde_json::Value = row.metadata.unwrap_or_else(|| json!({}));
            let external_ids: serde_json::Value = row.external_ids.unwrap_or_else(|| json!({}));

            let image_url = metadata
                .get("image_url")
                .and_then(|v| v.as_str())
                .map(String::from);
            let popularity = metadata
                .get("popularity")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32);
            let genres = metadata
                .get("genres")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            search_results.push(ArtistSearchResult {
                id: row.id,
                canonical_name: row.canonical_name,
                image_url,
                provider_badges: self.create_provider_badges(&external_ids, &metadata),
                popularity,
                genres,
            });
        }

        Ok(ArtistSearchResponse {
            total: search_results.len(),
            artists: search_results,
        })
    }

    /// Bulk import artists to DNP list
    pub async fn bulk_import(
        &self,
        user_id: Uuid,
        request: BulkImportRequest,
    ) -> Result<BulkOperationResult> {
        let entries = match request.format {
            ImportFormat::Csv => self.parse_csv_import(&request.data)?,
            ImportFormat::Json => self.parse_json_import(&request.data)?,
        };

        let mut successful = 0;
        let mut errors = Vec::new();

        for (index, entry) in entries.iter().enumerate() {
            match self
                .import_single_entry(user_id, entry, request.overwrite_existing.unwrap_or(false))
                .await
            {
                Ok(_) => successful += 1,
                Err(e) => {
                    errors.push(BulkOperationError {
                        entry_index: index,
                        artist_name: entry.artist_name.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }

        Ok(BulkOperationResult {
            total_processed: entries.len(),
            successful,
            failed: errors.len(),
            errors,
        })
    }

    /// Export user's DNP list
    pub async fn export_dnp_list(&self, user_id: Uuid, format: ImportFormat) -> Result<String> {
        // Get detailed DNP list with external IDs
        let entries = sqlx::query!(
            r#"
            SELECT 
                uab.artist_id,
                uab.tags,
                uab.note,
                uab.created_at,
                a.canonical_name,
                a.external_ids,
                a.metadata
            FROM user_artist_blocks uab
            JOIN artists a ON uab.artist_id = a.id
            WHERE uab.user_id = $1
            ORDER BY uab.created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        let export_entries: Vec<DnpExportEntry> = entries
            .into_iter()
            .map(|row| {
                let external_ids = row.external_ids.unwrap_or_else(|| json!({}));

                DnpExportEntry {
                    artist_name: row.canonical_name,
                    external_ids,
                    tags: row.tags.unwrap_or_default(),
                    note: row.note,
                    added_at: row.created_at.unwrap_or_else(|| Utc::now()),
                }
            })
            .collect();

        let export = DnpListExport {
            exported_at: Utc::now(),
            total_entries: export_entries.len(),
            entries: export_entries,
        };

        match format {
            ImportFormat::Json => Ok(serde_json::to_string_pretty(&export)?),
            ImportFormat::Csv => self.export_to_csv(&export),
        }
    }

    // Private helper methods

    /// Create or find an artist by name
    pub async fn create_or_find_artist(
        &self,
        name: &str,
        external_ids: Option<serde_json::Value>,
    ) -> Result<Uuid> {
        // First try to find existing artist by name
        if let Some(existing) = sqlx::query!(
            "SELECT id FROM artists WHERE canonical_name ILIKE $1 LIMIT 1",
            name
        )
        .fetch_optional(&self.db_pool)
        .await?
        {
            return Ok(existing.id);
        }

        // Create new artist if not found
        let artist_id = Uuid::new_v4();
        let external_ids = external_ids.unwrap_or_else(|| json!({}));
        let metadata = json!({});
        let aliases = json!({});

        sqlx::query!(
            r#"
            INSERT INTO artists (id, canonical_name, external_ids, metadata, aliases)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            artist_id,
            name,
            external_ids,
            metadata,
            aliases
        )
        .execute(&self.db_pool)
        .await?;

        Ok(artist_id)
    }

    async fn get_dnp_entry(&self, user_id: Uuid, artist_id: Uuid) -> Result<DnpListEntry> {
        let row = sqlx::query!(
            r#"
            SELECT 
                uab.tags,
                uab.note,
                uab.created_at,
                a.canonical_name,
                a.external_ids,
                a.metadata
            FROM user_artist_blocks uab
            JOIN artists a ON uab.artist_id = a.id
            WHERE uab.user_id = $1 AND uab.artist_id = $2
            "#,
            user_id,
            artist_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        let metadata: serde_json::Value = row.metadata.unwrap_or_else(|| json!({}));
        let external_ids: serde_json::Value = row.external_ids.unwrap_or_else(|| json!({}));

        let image_url = metadata
            .get("image_url")
            .and_then(|v| v.as_str())
            .map(String::from);
        let provider_badges = self.create_provider_badges(&external_ids, &metadata);

        Ok(DnpListEntry {
            artist_id,
            artist_name: row.canonical_name,
            image_url,
            provider_badges,
            tags: row.tags.unwrap_or_default(),
            note: row.note,
            added_at: row.created_at.unwrap_or_else(|| Utc::now()),
        })
    }

    fn create_provider_badges(
        &self,
        external_ids: &serde_json::Value,
        metadata: &serde_json::Value,
    ) -> Vec<ProviderBadge> {
        let mut badges = Vec::new();

        if let Some(_) = external_ids.get("spotify").and_then(|v| v.as_str()) {
            badges.push(ProviderBadge {
                provider: "spotify".to_string(),
                verified: metadata
                    .get("verified")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                follower_count: metadata.get("follower_count").and_then(|v| v.as_u64()),
            });
        }

        if let Some(_) = external_ids.get("apple").and_then(|v| v.as_str()) {
            badges.push(ProviderBadge {
                provider: "apple".to_string(),
                verified: false, // Apple Music doesn't have verification badges
                follower_count: None,
            });
        }

        if let Some(_) = external_ids.get("youtube").and_then(|v| v.as_str()) {
            badges.push(ProviderBadge {
                provider: "youtube".to_string(),
                verified: metadata
                    .get("verified")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                follower_count: metadata.get("follower_count").and_then(|v| v.as_u64()),
            });
        }

        if let Some(_) = external_ids.get("tidal").and_then(|v| v.as_str()) {
            badges.push(ProviderBadge {
                provider: "tidal".to_string(),
                verified: false,
                follower_count: None,
            });
        }

        badges
    }

    fn parse_csv_import(&self, data: &str) -> Result<Vec<ImportEntry>> {
        let mut entries = Vec::new();
        let mut reader = csv::Reader::from_reader(data.as_bytes());

        for result in reader.deserialize() {
            let entry: CsvImportEntry = result?;
            entries.push(ImportEntry {
                artist_name: entry.artist_name,
                provider_url: entry.provider_url,
                tags: entry
                    .tags
                    .map(|t| t.split(';').map(|s| s.trim().to_string()).collect()),
                note: entry.note,
            });
        }

        Ok(entries)
    }

    fn parse_json_import(&self, data: &str) -> Result<Vec<ImportEntry>> {
        let entries: Vec<ImportEntry> = serde_json::from_str(data)?;
        Ok(entries)
    }

    async fn import_single_entry(
        &self,
        user_id: Uuid,
        entry: &ImportEntry,
        overwrite: bool,
    ) -> Result<()> {
        // Create or find artist
        let artist_id = self.create_or_find_artist(&entry.artist_name, None).await?;

        // Check if already exists
        let existing = sqlx::query!(
            "SELECT user_id FROM user_artist_blocks WHERE user_id = $1 AND artist_id = $2",
            user_id,
            artist_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if existing.is_some() && !overwrite {
            return Err(anyhow!("Artist already in DNP list"));
        }

        if existing.is_some() && overwrite {
            // Update existing entry
            sqlx::query!(
                "UPDATE user_artist_blocks SET tags = $3, note = $4 WHERE user_id = $1 AND artist_id = $2",
                user_id,
                artist_id,
                &entry.tags.clone().unwrap_or_default(),
                entry.note
            )
            .execute(&self.db_pool)
            .await?;
        } else {
            // Insert new entry
            sqlx::query!(
                "INSERT INTO user_artist_blocks (user_id, artist_id, tags, note) VALUES ($1, $2, $3, $4)",
                user_id,
                artist_id,
                &entry.tags.clone().unwrap_or_default(),
                entry.note
            )
            .execute(&self.db_pool)
            .await?;
        }

        Ok(())
    }

    fn export_to_csv(&self, export: &DnpListExport) -> Result<String> {
        let mut writer = csv::Writer::from_writer(Vec::new());

        // Write header
        writer.write_record(&[
            "artist_name",
            "spotify_id",
            "apple_id",
            "tags",
            "note",
            "added_at",
        ])?;

        // Write data
        for entry in &export.entries {
            let spotify_id = entry
                .external_ids
                .get("spotify")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let apple_id = entry
                .external_ids
                .get("apple")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let tags = entry.tags.join(";");
            let note = entry.note.as_deref().unwrap_or("");

            writer.write_record(&[
                &entry.artist_name,
                spotify_id,
                apple_id,
                &tags,
                note,
                &entry.added_at.to_rfc3339(),
            ])?;
        }

        let data = writer.into_inner()?;
        Ok(String::from_utf8(data)?)
    }

    pub async fn get_dnp_list(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<crate::models::DnpEntryWithArtist>> {
        use crate::models::DnpEntryWithArtist;

        let entries = sqlx::query!(
            r#"
            SELECT 
                b.user_id,
                b.artist_id,
                b.tags,
                b.note,
                b.created_at,
                a.canonical_name,
                a.external_ids,
                a.metadata
            FROM user_artist_blocks b
            JOIN artists a ON b.artist_id = a.id
            WHERE b.user_id = $1
            ORDER BY b.created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut result = Vec::new();
        for row in entries {
            result.push(DnpEntryWithArtist {
                user_id: row.user_id,
                artist_id: row.artist_id,
                tags: row.tags,
                note: row.note,
                created_at: row.created_at.unwrap_or_else(|| Utc::now()),
                canonical_name: row.canonical_name,
                external_ids: row.external_ids.unwrap_or_else(|| json!({})),
                metadata: row.metadata.unwrap_or_else(|| json!({})),
            });
        }

        Ok(result)
    }

    pub async fn add_to_dnp_list(
        &self,
        user_id: Uuid,
        artist_id: Uuid,
        tags: Option<Vec<String>>,
        note: Option<String>,
    ) -> Result<crate::models::DnpEntry> {
        use crate::models::DnpEntry;

        // Check for duplicates
        let existing = sqlx::query!(
            "SELECT 1 as exists FROM user_artist_blocks WHERE user_id = $1 AND artist_id = $2",
            user_id,
            artist_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if existing.is_some() {
            return Err(anyhow!("Artist already in DNP list"));
        }

        // Add to DNP list
        let entry = sqlx::query!(
            r#"
            INSERT INTO user_artist_blocks (user_id, artist_id, tags, note)
            VALUES ($1, $2, $3, $4)
            RETURNING user_id, artist_id, tags, note, created_at
            "#,
            user_id,
            artist_id,
            tags.as_deref(),
            note
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(DnpEntry {
            user_id: entry.user_id,
            artist_id: entry.artist_id,
            tags: entry.tags,
            note: entry.note,
            created_at: entry.created_at.unwrap_or_else(|| chrono::Utc::now()),
        })
    }

    pub async fn remove_from_dnp_list(&self, user_id: Uuid, artist_id: Uuid) -> Result<()> {
        let result = sqlx::query!(
            "DELETE FROM user_artist_blocks WHERE user_id = $1 AND artist_id = $2",
            user_id,
            artist_id
        )
        .execute(&self.db_pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("Artist not found in DNP list"));
        }

        Ok(())
    }
}
