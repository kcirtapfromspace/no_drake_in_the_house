//! Credits Sync Service
//!
//! Fetches song credits from Apple Music API and stores them in the database.
//! Apple Music provides comprehensive credits including producers, writers, engineers, etc.

use anyhow::{Context, Result};
use chrono::{Datelike, NaiveDate, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

/// Credit role mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CreditRole {
    PrimaryArtist,
    FeaturedArtist,
    Producer,
    Writer,
    Composer,
    Lyricist,
    Arranger,
    Mixer,
    MasteringEngineer,
    RecordingEngineer,
    BackgroundVocalist,
    Instrumentalist,
    Remixer,
    SampleCredit,
    Other,
}

impl CreditRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            CreditRole::PrimaryArtist => "primary_artist",
            CreditRole::FeaturedArtist => "featured_artist",
            CreditRole::Producer => "producer",
            CreditRole::Writer => "writer",
            CreditRole::Composer => "composer",
            CreditRole::Lyricist => "lyricist",
            CreditRole::Arranger => "arranger",
            CreditRole::Mixer => "mixer",
            CreditRole::MasteringEngineer => "mastering_engineer",
            CreditRole::RecordingEngineer => "recording_engineer",
            CreditRole::BackgroundVocalist => "background_vocalist",
            CreditRole::Instrumentalist => "instrumentalist",
            CreditRole::Remixer => "remixer",
            CreditRole::SampleCredit => "sample_credit",
            CreditRole::Other => "other",
        }
    }

    pub fn from_apple_role(role: &str) -> Self {
        let role_lower = role.to_lowercase();
        if role_lower.contains("producer") || role_lower.contains("produced by") {
            CreditRole::Producer
        } else if role_lower.contains("writer") || role_lower.contains("written by") || role_lower.contains("songwriter") {
            CreditRole::Writer
        } else if role_lower.contains("composer") || role_lower.contains("composed by") {
            CreditRole::Composer
        } else if role_lower.contains("lyricist") || role_lower.contains("lyrics by") {
            CreditRole::Lyricist
        } else if role_lower.contains("mixer") || role_lower.contains("mixed by") {
            CreditRole::Mixer
        } else if role_lower.contains("master") {
            CreditRole::MasteringEngineer
        } else if role_lower.contains("engineer") || role_lower.contains("recorded by") {
            CreditRole::RecordingEngineer
        } else if role_lower.contains("vocal") || role_lower.contains("backing") {
            CreditRole::BackgroundVocalist
        } else if role_lower.contains("guitar") || role_lower.contains("bass") || role_lower.contains("drums")
            || role_lower.contains("keyboard") || role_lower.contains("piano") || role_lower.contains("instrument") {
            CreditRole::Instrumentalist
        } else if role_lower.contains("remix") {
            CreditRole::Remixer
        } else if role_lower.contains("sample") || role_lower.contains("contains") {
            CreditRole::SampleCredit
        } else if role_lower.contains("featuring") || role_lower.contains("feat") {
            CreditRole::FeaturedArtist
        } else if role_lower.contains("arranger") || role_lower.contains("arranged by") {
            CreditRole::Arranger
        } else {
            CreditRole::Other
        }
    }
}

/// Apple Music API responses
#[derive(Debug, Deserialize)]
struct AppleMusicResponse<T> {
    data: Vec<T>,
    next: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AppleMusicAlbum {
    id: String,
    attributes: AppleMusicAlbumAttributes,
    relationships: Option<AppleMusicAlbumRelationships>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppleMusicAlbumAttributes {
    name: String,
    artist_name: String,
    release_date: Option<String>,
    track_count: Option<i32>,
    record_label: Option<String>,
    upc: Option<String>,
    genre_names: Option<Vec<String>>,
    artwork: Option<AppleMusicArtwork>,
}

#[derive(Debug, Deserialize)]
struct AppleMusicAlbumRelationships {
    tracks: Option<AppleMusicTracksRelationship>,
}

#[derive(Debug, Deserialize)]
struct AppleMusicTracksRelationship {
    data: Vec<AppleMusicTrackRef>,
}

#[derive(Debug, Deserialize)]
struct AppleMusicTrackRef {
    id: String,
}

#[derive(Debug, Deserialize)]
struct AppleMusicTrack {
    id: String,
    attributes: AppleMusicTrackAttributes,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppleMusicTrackAttributes {
    name: String,
    artist_name: String,
    album_name: Option<String>,
    track_number: Option<i32>,
    disc_number: Option<i32>,
    duration_in_millis: Option<i64>,
    isrc: Option<String>,
    composer_name: Option<String>,
    #[serde(default)]
    has_lyrics: bool,
    preview_url: Option<String>,
    // Credits are in the "meta" field when requested with include=credits
    #[serde(default)]
    content_rating: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AppleMusicArtwork {
    url: String,
}

#[derive(Debug, Deserialize)]
struct AppleMusicSongCredits {
    data: Vec<AppleMusicCreditItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppleMusicCreditItem {
    role_name: String,
    artist_name: String,
    #[serde(default)]
    artist_id: Option<String>,
}

/// Credits sync service
pub struct CreditsSyncService {
    db_pool: PgPool,
    client: Client,
    team_id: String,
    key_id: String,
    private_key: String,
    storefront: String,
}

impl CreditsSyncService {
    pub fn new(
        db_pool: PgPool,
        team_id: String,
        key_id: String,
        private_key: String,
        storefront: String,
    ) -> Self {
        Self {
            db_pool,
            client: Client::new(),
            team_id,
            key_id,
            private_key,
            storefront,
        }
    }

    /// Generate Apple Music JWT token
    fn generate_token(&self) -> Result<String> {
        use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
        use serde_json::json;

        let now = Utc::now().timestamp();
        let exp = now + 3600; // 1 hour

        let claims = json!({
            "iss": self.team_id,
            "iat": now,
            "exp": exp
        });

        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some(self.key_id.clone());

        let key = EncodingKey::from_ec_pem(self.private_key.as_bytes())
            .context("Failed to parse Apple Music private key")?;

        encode(&header, &claims, &key).context("Failed to generate JWT")
    }

    /// Sync credits for an artist
    pub async fn sync_artist_credits(&self, artist_id: Uuid) -> Result<SyncStats> {
        let mut stats = SyncStats::default();

        // Get artist's Apple Music ID
        let apple_id: Option<String> = sqlx::query_scalar(
            "SELECT platform_id FROM artist_platform_ids WHERE artist_id = $1 AND platform = 'apple_music'"
        )
        .bind(artist_id)
        .fetch_optional(&self.db_pool)
        .await?;

        let apple_artist_id = match apple_id {
            Some(id) => id,
            None => {
                tracing::warn!(artist_id = %artist_id, "No Apple Music ID found for artist");
                return Ok(stats);
            }
        };

        // Create sync run record
        let run_id = self.create_sync_run(artist_id).await?;

        // Fetch and process albums
        match self.fetch_and_process_albums(&apple_artist_id, artist_id, &mut stats).await {
            Ok(_) => {
                self.complete_sync_run(run_id, &stats, None).await?;
            }
            Err(e) => {
                self.complete_sync_run(run_id, &stats, Some(&e.to_string())).await?;
                return Err(e);
            }
        }

        Ok(stats)
    }

    async fn create_sync_run(&self, artist_id: Uuid) -> Result<Uuid> {
        let run_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO credits_sync_runs (artist_id, platform, status, started_at)
            VALUES ($1, 'apple_music', 'running', NOW())
            RETURNING id
            "#
        )
        .bind(artist_id)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(run_id)
    }

    async fn complete_sync_run(&self, run_id: Uuid, stats: &SyncStats, error: Option<&str>) -> Result<()> {
        let status = if error.is_some() { "failed" } else { "completed" };

        sqlx::query(
            r#"
            UPDATE credits_sync_runs
            SET status = $2, albums_processed = $3, tracks_processed = $4,
                credits_added = $5, completed_at = NOW(),
                error_log = CASE WHEN $6 IS NOT NULL THEN jsonb_build_array($6) ELSE '[]'::jsonb END
            WHERE id = $1
            "#
        )
        .bind(run_id)
        .bind(status)
        .bind(stats.albums_processed as i32)
        .bind(stats.tracks_processed as i32)
        .bind(stats.credits_added as i32)
        .bind(error)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn fetch_and_process_albums(
        &self,
        apple_artist_id: &str,
        artist_id: Uuid,
        stats: &mut SyncStats,
    ) -> Result<()> {
        let token = self.generate_token()?;
        let base_url = format!(
            "https://api.music.apple.com/v1/catalog/{}/artists/{}/albums",
            self.storefront, apple_artist_id
        );

        let mut offset = 0;
        let limit = 25;

        loop {
            let url = format!("{}?limit={}&offset={}", base_url, limit, offset);

            let response = self.client
                .get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await
                .context("Failed to fetch albums from Apple Music")?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                tracing::error!(status = %status, body = %body, "Apple Music API error");
                return Err(anyhow::anyhow!("Apple Music API error: {} - {}", status, body));
            }

            let albums: AppleMusicResponse<AppleMusicAlbum> = response.json().await
                .context("Failed to parse albums response")?;

            if albums.data.is_empty() {
                break;
            }

            for album in albums.data {
                if let Err(e) = self.process_album(&album, artist_id, stats).await {
                    tracing::warn!(album_id = %album.id, error = %e, "Failed to process album");
                    stats.errors += 1;
                }
                stats.albums_processed += 1;
            }

            if albums.next.is_none() {
                break;
            }
            offset += limit;

            // Rate limiting
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }

        Ok(())
    }

    async fn process_album(
        &self,
        album: &AppleMusicAlbum,
        artist_id: Uuid,
        stats: &mut SyncStats,
    ) -> Result<()> {
        let attrs = &album.attributes;

        // Parse release date
        let release_date = attrs.release_date.as_ref().and_then(|d| {
            NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()
        });
        let release_year = release_date.map(|d| d.year());

        // Get cover art URL
        let cover_url = attrs.artwork.as_ref().map(|a| {
            a.url.replace("{w}", "600").replace("{h}", "600")
        });

        // Insert or update album
        let album_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO albums (title, release_date, release_year, album_type, total_tracks,
                               label, upc, cover_art_url, apple_music_id, genres)
            VALUES ($1, $2, $3, 'album', $4, $5, $6, $7, $8, $9)
            ON CONFLICT (apple_music_id) DO UPDATE SET
                title = EXCLUDED.title,
                updated_at = NOW()
            RETURNING id
            "#
        )
        .bind(&attrs.name)
        .bind(release_date)
        .bind(release_year)
        .bind(attrs.track_count)
        .bind(&attrs.record_label)
        .bind(&attrs.upc)
        .bind(&cover_url)
        .bind(&album.id)
        .bind(&attrs.genre_names.clone().unwrap_or_default())
        .fetch_one(&self.db_pool)
        .await?;

        // Link album to artist
        sqlx::query(
            r#"
            INSERT INTO album_artists (album_id, artist_id, is_primary)
            VALUES ($1, $2, true)
            ON CONFLICT DO NOTHING
            "#
        )
        .bind(album_id)
        .bind(artist_id)
        .execute(&self.db_pool)
        .await?;

        // Fetch tracks with credits
        self.fetch_album_tracks(&album.id, album_id, artist_id, stats).await?;

        Ok(())
    }

    async fn fetch_album_tracks(
        &self,
        apple_album_id: &str,
        album_id: Uuid,
        artist_id: Uuid,
        stats: &mut SyncStats,
    ) -> Result<()> {
        let token = self.generate_token()?;
        let url = format!(
            "https://api.music.apple.com/v1/catalog/{}/albums/{}/tracks",
            self.storefront, apple_album_id
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(());
        }

        let tracks: AppleMusicResponse<AppleMusicTrack> = response.json().await?;

        for track in tracks.data {
            if let Err(e) = self.process_track(&track, album_id, artist_id, stats).await {
                tracing::warn!(track_id = %track.id, error = %e, "Failed to process track");
                stats.errors += 1;
            }
            stats.tracks_processed += 1;

            // Rate limit
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }

    async fn process_track(
        &self,
        track: &AppleMusicTrack,
        album_id: Uuid,
        artist_id: Uuid,
        stats: &mut SyncStats,
    ) -> Result<()> {
        let attrs = &track.attributes;
        let is_explicit = attrs.content_rating.as_ref().map(|r| r == "explicit").unwrap_or(false);

        // Insert track
        let track_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO tracks (title, album_id, track_number, disc_number, duration_ms,
                               explicit, isrc, preview_url, apple_music_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (apple_music_id) DO UPDATE SET
                title = EXCLUDED.title,
                updated_at = NOW()
            RETURNING id
            "#
        )
        .bind(&attrs.name)
        .bind(album_id)
        .bind(attrs.track_number)
        .bind(attrs.disc_number)
        .bind(attrs.duration_in_millis.map(|d| d as i32))
        .bind(is_explicit)
        .bind(&attrs.isrc)
        .bind(&attrs.preview_url)
        .bind(&track.id)
        .fetch_one(&self.db_pool)
        .await?;

        // Add primary artist credit
        self.add_credit(track_id, Some(artist_id), &attrs.artist_name, CreditRole::PrimaryArtist, None, stats).await?;

        // Add composer credit if available
        if let Some(ref composer) = attrs.composer_name {
            if !composer.is_empty() && composer != &attrs.artist_name {
                self.add_credit(track_id, None, composer, CreditRole::Composer, None, stats).await?;
            }
        }

        // Fetch detailed credits for this track
        self.fetch_track_credits(&track.id, track_id, stats).await?;

        Ok(())
    }

    async fn fetch_track_credits(
        &self,
        apple_track_id: &str,
        track_id: Uuid,
        stats: &mut SyncStats,
    ) -> Result<()> {
        let token = self.generate_token()?;

        // Apple Music credits endpoint
        let url = format!(
            "https://api.music.apple.com/v1/catalog/{}/songs/{}?include=credits",
            self.storefront, apple_track_id
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(());
        }

        // Parse the response - credits might be in relationships
        let body: serde_json::Value = response.json().await?;

        // Try to extract credits from the response
        if let Some(data) = body.get("data").and_then(|d| d.as_array()) {
            if let Some(song) = data.first() {
                // Check for credits in relationships
                if let Some(relationships) = song.get("relationships") {
                    if let Some(credits) = relationships.get("credits") {
                        if let Some(credit_data) = credits.get("data").and_then(|d| d.as_array()) {
                            for credit in credit_data {
                                if let (Some(role), Some(name)) = (
                                    credit.get("attributes").and_then(|a| a.get("roleName")).and_then(|r| r.as_str()),
                                    credit.get("attributes").and_then(|a| a.get("artistName")).and_then(|n| n.as_str())
                                ) {
                                    let credit_role = CreditRole::from_apple_role(role);
                                    self.add_credit(track_id, None, name, credit_role, Some(role), stats).await?;
                                }
                            }
                        }
                    }
                }

                // Also check "meta" for composer info
                if let Some(meta) = song.get("meta") {
                    if let Some(composers) = meta.get("composers").and_then(|c| c.as_array()) {
                        for composer in composers {
                            if let Some(name) = composer.as_str() {
                                self.add_credit(track_id, None, name, CreditRole::Composer, None, stats).await?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn add_credit(
        &self,
        track_id: Uuid,
        artist_id: Option<Uuid>,
        name: &str,
        role: CreditRole,
        role_detail: Option<&str>,
        stats: &mut SyncStats,
    ) -> Result<()> {
        // Try to match credited name to an existing artist
        let matched_artist_id = if artist_id.is_some() {
            artist_id
        } else {
            self.find_artist_by_name(name).await?
        };

        let result = sqlx::query(
            r#"
            INSERT INTO track_credits (track_id, artist_id, credited_name, role, role_detail, source_platform)
            VALUES ($1, $2, $3, $4::credit_role, $5, 'apple_music')
            ON CONFLICT (track_id, artist_id, credited_name, role) DO NOTHING
            "#
        )
        .bind(track_id)
        .bind(matched_artist_id)
        .bind(name)
        .bind(role.as_str())
        .bind(role_detail)
        .execute(&self.db_pool)
        .await?;

        if result.rows_affected() > 0 {
            stats.credits_added += 1;

            // Update collaboration if we have two artists
            if let Some(credited_artist) = matched_artist_id {
                // Get primary artist for this track
                if let Ok(Some(primary_artist)) = self.get_track_primary_artist(track_id).await {
                    if primary_artist != credited_artist {
                        self.update_collaboration(primary_artist, credited_artist, &role, track_id).await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn find_artist_by_name(&self, name: &str) -> Result<Option<Uuid>> {
        let artist_id: Option<Uuid> = sqlx::query_scalar(
            r#"
            SELECT id FROM artists
            WHERE LOWER(canonical_name) = LOWER($1)
            OR aliases::text ILIKE '%' || $1 || '%'
            LIMIT 1
            "#
        )
        .bind(name)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(artist_id)
    }

    async fn get_track_primary_artist(&self, track_id: Uuid) -> Result<Option<Uuid>> {
        let artist_id: Option<Uuid> = sqlx::query_scalar(
            r#"
            SELECT artist_id FROM track_credits
            WHERE track_id = $1 AND role = 'primary_artist'
            LIMIT 1
            "#
        )
        .bind(track_id)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(artist_id)
    }

    async fn update_collaboration(
        &self,
        artist1: Uuid,
        artist2: Uuid,
        role: &CreditRole,
        track_id: Uuid,
    ) -> Result<()> {
        let (a1, a2) = if artist1 < artist2 { (artist1, artist2) } else { (artist2, artist1) };

        let collab_type = match role {
            CreditRole::FeaturedArtist => "featured",
            CreditRole::Producer => "producer",
            CreditRole::Writer | CreditRole::Composer | CreditRole::Lyricist => "writer",
            CreditRole::SampleCredit => "sample",
            _ => "other",
        };

        sqlx::query(
            r#"
            INSERT INTO artist_collaborations (artist_id_1, artist_id_2, collaboration_type, track_count, sample_track_ids)
            VALUES ($1, $2, $3, 1, ARRAY[$4])
            ON CONFLICT (artist_id_1, artist_id_2, collaboration_type) DO UPDATE SET
                track_count = artist_collaborations.track_count + 1,
                sample_track_ids = array_append(
                    artist_collaborations.sample_track_ids[1:5],
                    $4
                ),
                updated_at = NOW()
            "#
        )
        .bind(a1)
        .bind(a2)
        .bind(collab_type)
        .bind(track_id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Sync credits for all artists in the database
    pub async fn sync_all_artists(&self) -> Result<SyncStats> {
        let mut total_stats = SyncStats::default();

        let artists: Vec<Uuid> = sqlx::query_scalar(
            "SELECT DISTINCT artist_id FROM artist_platform_ids WHERE platform = 'apple_music'"
        )
        .fetch_all(&self.db_pool)
        .await?;

        tracing::info!(count = artists.len(), "Starting credits sync for all artists");

        for artist_id in artists {
            match self.sync_artist_credits(artist_id).await {
                Ok(stats) => {
                    total_stats.merge(&stats);
                    tracing::info!(
                        artist_id = %artist_id,
                        albums = stats.albums_processed,
                        tracks = stats.tracks_processed,
                        credits = stats.credits_added,
                        "Completed credits sync for artist"
                    );
                }
                Err(e) => {
                    tracing::error!(artist_id = %artist_id, error = %e, "Failed to sync credits");
                    total_stats.errors += 1;
                }
            }

            // Delay between artists to avoid rate limiting
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        Ok(total_stats)
    }
}

#[derive(Debug, Default)]
pub struct SyncStats {
    pub albums_processed: usize,
    pub tracks_processed: usize,
    pub credits_added: usize,
    pub errors: usize,
}

impl SyncStats {
    pub fn merge(&mut self, other: &SyncStats) {
        self.albums_processed += other.albums_processed;
        self.tracks_processed += other.tracks_processed;
        self.credits_added += other.credits_added;
        self.errors += other.errors;
    }
}
