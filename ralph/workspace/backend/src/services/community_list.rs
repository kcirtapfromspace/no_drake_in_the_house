use crate::models::*;
use crate::services::EntityResolutionService;
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct CommunityListService {
    db_pool: PgPool,
    entity_service: Arc<EntityResolutionService>,
}

impl CommunityListService {
    pub fn new(db_pool: PgPool, entity_service: Arc<EntityResolutionService>) -> Self {
        Self {
            db_pool,
            entity_service,
        }
    }

    /// Create a new community list
    pub async fn create_community_list(
        &self,
        owner_user_id: Uuid,
        request: CreateCommunityListRequest,
    ) -> Result<CommunityListResponse> {
        // Validate criteria for neutrality
        self.validate_neutral_criteria(&request.criteria)?;

        let list_id = Uuid::new_v4();
        let now = Utc::now();
        let visibility = request.visibility.unwrap_or_else(|| "public".to_string());

        // Insert community list
        sqlx::query(
            r#"
            INSERT INTO community_lists (
                id, owner_user_id, name, description, criteria, 
                governance_url, update_cadence, version, visibility, 
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            list_id,
            owner_user_id,
            request.name,
            request.description,
            request.criteria,
            request.governance_url,
            request.update_cadence,
            1, // Initial version
            visibility,
            now,
            now
        )
        .execute(&self.db_pool)
        .await?;

        self.get_community_list_by_id(list_id, Some(owner_user_id)).await
    }

    /// Get community list by ID
    pub async fn get_community_list_by_id(
        &self,
        list_id: Uuid,
        requesting_user_id: Option<Uuid>,
    ) -> Result<CommunityListResponse> {
        let row = sqlx::query(
            r#"
            SELECT 
                cl.id, cl.owner_user_id, cl.name, cl.description, cl.criteria,
                cl.governance_url, cl.update_cadence, cl.version, cl.visibility,
                cl.created_at, cl.updated_at,
                u.email as owner_email,
                COUNT(DISTINCT cli.artist_id) as total_artists,
                COUNT(DISTINCT uls.user_id) as subscriber_count
            FROM community_lists cl
            JOIN users u ON cl.owner_user_id = u.id
            LEFT JOIN community_list_items cli ON cl.id = cli.list_id
            LEFT JOIN user_list_subscriptions uls ON cl.id = uls.list_id
            WHERE cl.id = $1 AND (cl.visibility = 'public' OR cl.owner_user_id = $2)
            GROUP BY cl.id, u.email
            "#,
            list_id,
            requesting_user_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        let row = row.ok_or_else(|| anyhow!("Community list not found or not accessible"))?;

        // Check if user is subscribed
        let (is_subscribed, subscription_details) = if let Some(user_id) = requesting_user_id {
            let subscription = sqlx::query(
                "SELECT version_pinned, auto_update, created_at FROM user_list_subscriptions WHERE user_id = $1 AND list_id = $2",
                user_id,
                list_id
            )
            .fetch_optional(&self.db_pool)
            .await?;

            if let Some(sub) = subscription {
                (
                    true,
                    Some(SubscriptionDetails {
                        version_pinned: sub.version_pinned,
                        auto_update: sub.auto_update,
                        subscribed_at: sub.created_at,
                    }),
                )
            } else {
                (false, None)
            }
        } else {
            (false, None)
        };

        Ok(CommunityListResponse {
            id: row.id,
            owner: UserInfo {
                id: row.owner_user_id,
                email: self.mask_email(&row.owner_email),
            },
            name: row.name,
            description: row.description,
            criteria: row.criteria,
            governance_url: row.governance_url,
            update_cadence: row.update_cadence,
            version: row.version,
            visibility: row.visibility,
            total_artists: row.total_artists.unwrap_or(0) as usize,
            subscriber_count: row.subscriber_count.unwrap_or(0) as usize,
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_subscribed,
            subscription_details,
        })
    }

    /// Get community list with artists
    pub async fn get_community_list_with_artists(
        &self,
        list_id: Uuid,
        requesting_user_id: Option<Uuid>,
    ) -> Result<CommunityListWithArtists> {
        let list = self.get_community_list_by_id(list_id, requesting_user_id).await?;

        let artists = sqlx::query(
            r#"
            SELECT 
                cli.artist_id,
                cli.rationale_link,
                cli.added_at,
                a.canonical_name,
                a.external_ids,
                a.metadata
            FROM community_list_items cli
            JOIN artists a ON cli.artist_id = a.id
            WHERE cli.list_id = $1
            ORDER BY cli.added_at DESC
            "#,
            list_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        let artist_entries: Vec<CommunityListArtistEntry> = artists
            .into_iter()
            .map(|row| {
                let metadata: serde_json::Value = row.metadata.unwrap_or_else(|| json!({}));
                let external_ids: serde_json::Value = row.external_ids.unwrap_or_else(|| json!({}));

                let image_url = metadata.get("image_url").and_then(|v| v.as_str()).map(String::from);
                let provider_badges = self.create_provider_badges(&external_ids, &metadata);

                CommunityListArtistEntry {
                    artist_id: row.artist_id,
                    artist_name: row.canonical_name,
                    image_url,
                    provider_badges,
                    rationale_link: row.rationale_link,
                    added_at: row.added_at,
                }
            })
            .collect();

        Ok(CommunityListWithArtists {
            list,
            artists: artist_entries,
        })
    }

    /// Browse community lists directory
    pub async fn browse_community_lists(
        &self,
        query: CommunityListQuery,
    ) -> Result<CommunityListDirectory> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(20).min(100); // Cap at 100
        let offset = (page - 1) * per_page;

        let sort_by = query.sort_by.as_deref().unwrap_or("updated_at");
        let sort_order = query.sort_order.as_deref().unwrap_or("desc");

        // Build dynamic query based on filters
        let mut where_conditions = vec!["cl.visibility = 'public'".to_string()];
        let mut query_params: Vec<String> = Vec::new();

        if let Some(search) = &query.search {
            where_conditions.push(format!("(cl.name ILIKE ${})", query_params.len() + 1));
            query_params.push(format!("%{}%", search));
        }

        if let Some(criteria_filter) = &query.criteria_filter {
            where_conditions.push(format!("cl.criteria ILIKE ${}", query_params.len() + 1));
            query_params.push(format!("%{}%", criteria_filter));
        }

        let where_clause = where_conditions.join(" AND ");
        let order_clause = format!("ORDER BY cl.{} {}", sort_by, sort_order);

        let query_str = format!(
            r#"
            SELECT 
                cl.id, cl.name, cl.description, cl.criteria, cl.version,
                cl.update_cadence, cl.created_at, cl.updated_at,
                u.email as owner_email,
                COUNT(DISTINCT cli.artist_id) as total_artists,
                COUNT(DISTINCT uls.user_id) as subscriber_count
            FROM community_lists cl
            JOIN users u ON cl.owner_user_id = u.id
            LEFT JOIN community_list_items cli ON cl.id = cli.list_id
            LEFT JOIN user_list_subscriptions uls ON cl.id = uls.list_id
            WHERE {}
            GROUP BY cl.id, u.email
            {}
            LIMIT {} OFFSET {}
            "#,
            where_clause, order_clause, per_page, offset
        );

        // For now, use a simplified query without dynamic parameters
        // In a production system, you'd want to use a query builder
        let rows = sqlx::query(
            r#"
            SELECT 
                cl.id, cl.name, cl.description, cl.criteria, cl.version,
                cl.update_cadence, cl.created_at, cl.updated_at,
                u.email as owner_email,
                COUNT(DISTINCT cli.artist_id) as total_artists,
                COUNT(DISTINCT uls.user_id) as subscriber_count
            FROM community_lists cl
            JOIN users u ON cl.owner_user_id = u.id
            LEFT JOIN community_list_items cli ON cl.id = cli.list_id
            LEFT JOIN user_list_subscriptions uls ON cl.id = uls.list_id
            WHERE cl.visibility = 'public'
            GROUP BY cl.id, u.email
            ORDER BY cl.updated_at DESC
            LIMIT $1 OFFSET $2
            "#,
            per_page as i64,
            offset as i64
        )
        .fetch_all(&self.db_pool)
        .await?;

        let lists: Vec<CommunityListSummary> = rows
            .into_iter()
            .map(|row| CommunityListSummary {
                id: row.id,
                name: row.name,
                description: row.description,
                criteria: row.criteria,
                owner_email: self.mask_email(&row.owner_email),
                total_artists: row.total_artists.unwrap_or(0) as usize,
                subscriber_count: row.subscriber_count.unwrap_or(0) as usize,
                version: row.version,
                update_cadence: row.update_cadence,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        // Get total count for pagination
        let total_count = sqlx::query_scalar(
            "SELECT COUNT(*) FROM community_lists WHERE visibility = 'public'"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0) as usize;

        Ok(CommunityListDirectory {
            lists,
            total: total_count,
            page,
            per_page,
        })
    }

    /// Subscribe to a community list
    pub async fn subscribe_to_community_list(
        &self,
        user_id: Uuid,
        list_id: Uuid,
        request: SubscribeToCommunityListRequest,
    ) -> Result<SubscriptionDetails> {
        // Check if list exists and is accessible
        let list = self.get_community_list_by_id(list_id, Some(user_id)).await?;

        // Check if already subscribed
        let existing = sqlx::query(
            "SELECT 1 FROM user_list_subscriptions WHERE user_id = $1 AND list_id = $2",
            user_id,
            list_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if existing.is_some() {
            return Err(anyhow!("Already subscribed to this community list"));
        }

        let auto_update = request.auto_update.unwrap_or(true);
        let version_pinned = request.version_pinned.unwrap_or(list.version);
        let now = Utc::now();

        // Create subscription
        sqlx::query(
            "INSERT INTO user_list_subscriptions (user_id, list_id, version_pinned, auto_update, created_at) VALUES ($1, $2, $3, $4, $5)",
            user_id,
            list_id,
            version_pinned,
            auto_update,
            now
        )
        .execute(&self.db_pool)
        .await?;

        Ok(SubscriptionDetails {
            version_pinned: Some(version_pinned),
            auto_update,
            subscribed_at: now,
        })
    }

    /// Unsubscribe from a community list
    pub async fn unsubscribe_from_community_list(
        &self,
        user_id: Uuid,
        list_id: Uuid,
    ) -> Result<()> {
        let result = sqlx::query(
            "DELETE FROM user_list_subscriptions WHERE user_id = $1 AND list_id = $2",
            user_id,
            list_id
        )
        .execute(&self.db_pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("Not subscribed to this community list"));
        }

        Ok(())
    }

    /// Update subscription settings
    pub async fn update_subscription(
        &self,
        user_id: Uuid,
        list_id: Uuid,
        request: UpdateSubscriptionRequest,
    ) -> Result<SubscriptionDetails> {
        // Check if subscribed
        let existing = sqlx::query(
            "SELECT version_pinned, auto_update, created_at FROM user_list_subscriptions WHERE user_id = $1 AND list_id = $2",
            user_id,
            list_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        let existing = existing.ok_or_else(|| anyhow!("Not subscribed to this community list"))?;

        // Update subscription
        if let Some(version_pinned) = request.version_pinned {
            sqlx::query(
                "UPDATE user_list_subscriptions SET version_pinned = $3 WHERE user_id = $1 AND list_id = $2",
                user_id,
                list_id,
                version_pinned
            )
            .execute(&self.db_pool)
            .await?;
        }

        if let Some(auto_update) = request.auto_update {
            sqlx::query(
                "UPDATE user_list_subscriptions SET auto_update = $3 WHERE user_id = $1 AND list_id = $2",
                user_id,
                list_id,
                auto_update
            )
            .execute(&self.db_pool)
            .await?;
        }

        // Return updated subscription details
        let updated = sqlx::query(
            "SELECT version_pinned, auto_update, created_at FROM user_list_subscriptions WHERE user_id = $1 AND list_id = $2",
            user_id,
            list_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(SubscriptionDetails {
            version_pinned: updated.version_pinned,
            auto_update: updated.auto_update,
            subscribed_at: updated.created_at,
        })
    }

    /// Get subscription impact preview
    pub async fn get_subscription_impact_preview(
        &self,
        user_id: Uuid,
        list_id: Uuid,
    ) -> Result<SubscriptionImpactPreview> {
        let list = self.get_community_list_by_id(list_id, Some(user_id)).await?;

        // Get all artists in the community list
        let community_artists = sqlx::query(
            r#"
            SELECT 
                cli.artist_id,
                a.canonical_name,
                a.external_ids,
                a.metadata,
                cli.rationale_link,
                cli.added_at
            FROM community_list_items cli
            JOIN artists a ON cli.artist_id = a.id
            WHERE cli.list_id = $1
            "#,
            list_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        // Get user's current DNP list
        let user_dnp_artists: Vec<Uuid> = sqlx::query_scalar(
            "SELECT artist_id FROM user_artist_blocks WHERE user_id = $1",
            user_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        let user_dnp_set: std::collections::HashSet<Uuid> = user_dnp_artists.into_iter().collect();

        // Calculate impact
        let total_artists_in_list = community_artists.len();
        let mut new_artists_for_user = 0;
        let mut already_blocked_artists = 0;
        let mut sample_new_artists = Vec::new();

        for artist in &community_artists {
            if user_dnp_set.contains(&artist.artist_id) {
                already_blocked_artists += 1;
            } else {
                new_artists_for_user += 1;
                if sample_new_artists.len() < 10 {
                    let metadata: serde_json::Value = artist.metadata.clone().unwrap_or_else(|| json!({}));
                    let external_ids: serde_json::Value = artist.external_ids.clone().unwrap_or_else(|| json!({}));

                    let image_url = metadata.get("image_url").and_then(|v| v.as_str()).map(String::from);
                    let provider_badges = self.create_provider_badges(&external_ids, &metadata);

                    sample_new_artists.push(CommunityListArtistEntry {
                        artist_id: artist.artist_id,
                        artist_name: artist.canonical_name.clone(),
                        image_url,
                        provider_badges,
                        rationale_link: artist.rationale_link.clone(),
                        added_at: artist.added_at,
                    });
                }
            }
        }

        // For now, return placeholder provider impact
        // In a full implementation, this would analyze user's library
        let impact_by_provider = vec![
            ProviderImpact {
                provider: "spotify".to_string(),
                estimated_tracks_affected: new_artists_for_user * 3, // Rough estimate
                estimated_playlists_affected: new_artists_for_user / 5,
                estimated_follows_affected: new_artists_for_user / 10,
            },
        ];

        Ok(SubscriptionImpactPreview {
            list_id,
            list_name: list.name,
            version: list.version,
            total_artists_in_list,
            new_artists_for_user,
            already_blocked_artists,
            impact_by_provider,
            sample_new_artists,
        })
    }

    /// Get user's subscriptions
    pub async fn get_user_subscriptions(&self, user_id: Uuid) -> Result<Vec<CommunityListResponse>> {
        let subscriptions = sqlx::query(
            r#"
            SELECT 
                cl.id, cl.owner_user_id, cl.name, cl.description, cl.criteria,
                cl.governance_url, cl.update_cadence, cl.version, cl.visibility,
                cl.created_at, cl.updated_at,
                u.email as owner_email,
                uls.version_pinned, uls.auto_update, uls.created_at as subscribed_at,
                COUNT(DISTINCT cli.artist_id) as total_artists,
                COUNT(DISTINCT uls2.user_id) as subscriber_count
            FROM user_list_subscriptions uls
            JOIN community_lists cl ON uls.list_id = cl.id
            JOIN users u ON cl.owner_user_id = u.id
            LEFT JOIN community_list_items cli ON cl.id = cli.list_id
            LEFT JOIN user_list_subscriptions uls2 ON cl.id = uls2.list_id
            WHERE uls.user_id = $1
            GROUP BY cl.id, u.email, uls.version_pinned, uls.auto_update, uls.created_at
            ORDER BY uls.created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut results = Vec::new();
        for row in subscriptions {
            results.push(CommunityListResponse {
                id: row.id,
                owner: UserInfo {
                    id: row.owner_user_id,
                    email: self.mask_email(&row.owner_email),
                },
                name: row.name,
                description: row.description,
                criteria: row.criteria,
                governance_url: row.governance_url,
                update_cadence: row.update_cadence,
                version: row.version,
                visibility: row.visibility,
                total_artists: row.total_artists.unwrap_or(0) as usize,
                subscriber_count: row.subscriber_count.unwrap_or(0) as usize,
                created_at: row.created_at,
                updated_at: row.updated_at,
                is_subscribed: true,
                subscription_details: Some(SubscriptionDetails {
                    version_pinned: row.version_pinned,
                    auto_update: row.auto_update,
                    subscribed_at: row.subscribed_at,
                }),
            });
        }

        Ok(results)
    }

    /// Add artist to community list (for list owners)
    pub async fn add_artist_to_community_list(
        &self,
        owner_user_id: Uuid,
        list_id: Uuid,
        request: AddArtistToCommunityListRequest,
    ) -> Result<CommunityListArtistEntry> {
        // Verify ownership
        let list = sqlx::query(
            "SELECT owner_user_id FROM community_lists WHERE id = $1",
            list_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        let list = list.ok_or_else(|| anyhow!("Community list not found"))?;
        if list.owner_user_id != owner_user_id {
            return Err(anyhow!("Not authorized to modify this community list"));
        }

        // Resolve artist
        let artist = self.resolve_artist_from_query(&request.artist_query).await?;

        // Check if artist is already in the list
        let existing = sqlx::query(
            "SELECT 1 FROM community_list_items WHERE list_id = $1 AND artist_id = $2",
            list_id,
            artist.id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if existing.is_some() {
            return Err(anyhow!("Artist is already in this community list"));
        }

        let now = Utc::now();

        // Add artist to community list
        sqlx::query(
            "INSERT INTO community_list_items (list_id, artist_id, rationale_link, added_at) VALUES ($1, $2, $3, $4)",
            list_id,
            artist.id,
            request.rationale_link,
            now
        )
        .execute(&self.db_pool)
        .await?;

        // Update list version and timestamp
        sqlx::query(
            "UPDATE community_lists SET version = version + 1, updated_at = $2 WHERE id = $1",
            list_id,
            now
        )
        .execute(&self.db_pool)
        .await?;

        // Return the added artist entry
        let metadata_json = json!({
            "image_url": artist.metadata.image_url,
            "popularity": artist.metadata.popularity,
            "verified": artist.metadata.verified
        });
        let external_ids_json = json!({
            "spotify": artist.external_ids.spotify,
            "apple": artist.external_ids.apple,
            "youtube": artist.external_ids.youtube,
            "tidal": artist.external_ids.tidal
        });

        Ok(CommunityListArtistEntry {
            artist_id: artist.id,
            artist_name: artist.canonical_name,
            image_url: artist.metadata.image_url,
            provider_badges: self.create_provider_badges(&external_ids_json, &metadata_json),
            rationale_link: request.rationale_link,
            added_at: now,
        })
    }

    /// Remove artist from community list (for list owners)
    pub async fn remove_artist_from_community_list(
        &self,
        owner_user_id: Uuid,
        list_id: Uuid,
        artist_id: Uuid,
    ) -> Result<()> {
        // Verify ownership
        let list = sqlx::query(
            "SELECT owner_user_id FROM community_lists WHERE id = $1",
            list_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        let list = list.ok_or_else(|| anyhow!("Community list not found"))?;
        if list.owner_user_id != owner_user_id {
            return Err(anyhow!("Not authorized to modify this community list"));
        }

        // Remove artist from community list
        let result = sqlx::query(
            "DELETE FROM community_list_items WHERE list_id = $1 AND artist_id = $2",
            list_id,
            artist_id
        )
        .execute(&self.db_pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("Artist not found in community list"));
        }

        // Update list version and timestamp
        sqlx::query(
            "UPDATE community_lists SET version = version + 1, updated_at = $2 WHERE id = $1",
            list_id,
            Utc::now()
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    // Private helper methods

    async fn resolve_artist_from_query(&self, query: &str) -> Result<Artist> {
        let search_query = ArtistSearchQuery::new(query.to_string()).with_limit(1);
        let results = self.entity_service.resolve_artist(&search_query).await?;
        
        if results.is_empty() {
            return Err(anyhow!("Artist not found: {}", query));
        }

        Ok(results[0].artist.clone())
    }

    fn validate_neutral_criteria(&self, criteria: &str) -> Result<()> {
        let prohibited_patterns = [
            r"\b(accused|alleged|guilty)\b",
            r"\b(criminal|illegal|lawsuit)\b",
            r"\b(bad|evil|terrible)\s+(person|artist)\b",
        ];

        for pattern in &prohibited_patterns {
            let regex = regex::Regex::new(pattern)?;
            if regex.is_match(criteria) {
                return Err(anyhow!("Criteria must be neutral and factual. Avoid judgmental language."));
            }
        }

        Ok(())
    }

    fn mask_email(&self, email: &str) -> String {
        if let Some(at_pos) = email.find('@') {
            let (local, domain) = email.split_at(at_pos);
            if local.len() <= 2 {
                format!("{}@{}", "*".repeat(local.len()), domain)
            } else {
                format!("{}{}@{}", &local[..2], "*".repeat(local.len() - 2), domain)
            }
        } else {
            "*".repeat(email.len())
        }
    }

    fn create_provider_badges(&self, external_ids: &serde_json::Value, metadata: &serde_json::Value) -> Vec<ProviderBadge> {
        let mut badges = Vec::new();

        if let Some(_) = external_ids.get("spotify").and_then(|v| v.as_str()) {
            badges.push(ProviderBadge {
                provider: "spotify".to_string(),
                verified: metadata.get("verified").and_then(|v| v.as_bool()).unwrap_or(false),
                follower_count: metadata.get("follower_count").and_then(|v| v.as_u64()),
            });
        }

        if let Some(_) = external_ids.get("apple").and_then(|v| v.as_str()) {
            badges.push(ProviderBadge {
                provider: "apple".to_string(),
                verified: false,
                follower_count: None,
            });
        }

        if let Some(_) = external_ids.get("youtube").and_then(|v| v.as_str()) {
            badges.push(ProviderBadge {
                provider: "youtube".to_string(),
                verified: metadata.get("verified").and_then(|v| v.as_bool()).unwrap_or(false),
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
}