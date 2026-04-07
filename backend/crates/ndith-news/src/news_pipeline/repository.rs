//! News Repository
//!
//! Persistence layer for news articles, entities, and offenses.
//! Write operations go to Convex via HTTP mutations.
//! Read operations still use PostgreSQL (will be migrated separately).

use super::ingestion::FetchedArticle;
use super::processing::{ExtractedEntity, OffenseClassification};
use crate::convex_client::{
    BatchArticleArgs, BatchClassificationArgs, BatchEntityArgs, ConvexClient,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for persisting news data.
///
/// Writes go to Convex via `ConvexClient`.
/// Reads still hit PostgreSQL (migrated separately).
pub struct NewsRepository {
    convex: Option<ConvexClient>,
    /// PostgreSQL pool kept for read queries only
    db_pool: Option<PgPool>,
}

/// Article summary for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleSummary {
    pub id: Uuid,
    pub title: String,
    pub url: String,
    pub excerpt: Option<String>,
    pub source_name: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub entity_count: i64,
    pub offense_count: i64,
    pub processing_status: Option<String>,
}

/// Filters for querying articles
#[derive(Debug, Clone, Default)]
pub struct ArticleFilters {
    pub source_id: Option<Uuid>,
    pub artist_id: Option<Uuid>,
    pub offense_category: Option<String>,
    pub processing_status: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub search_query: Option<String>,
}

impl NewsRepository {
    /// Create a new news repository backed by Convex (writes) and PostgreSQL (reads).
    pub fn new(convex: ConvexClient, db_pool: PgPool) -> Self {
        Self {
            convex: Some(convex),
            db_pool: Some(db_pool),
        }
    }

    /// Create a Convex-only repository (no PostgreSQL reads).
    pub fn convex_only(convex: ConvexClient) -> Self {
        Self {
            convex: Some(convex),
            db_pool: None,
        }
    }

    /// Create a read-only repository backed by PostgreSQL.
    ///
    /// Only read queries are available; write operations will return errors.
    /// Used by API handlers that only need to query existing data.
    pub fn read_only(db_pool: PgPool) -> Self {
        Self {
            convex: None,
            db_pool: Some(db_pool),
        }
    }

    /// Get a reference to the Convex client, if configured.
    pub fn convex_client(&self) -> Option<&ConvexClient> {
        self.convex.as_ref()
    }

    // ---------------------------------------------------------------
    // Write operations — Convex
    // ---------------------------------------------------------------

    /// Insert a processed article with entities and offenses into Convex.
    ///
    /// Uses `batchIngestArticles` for a single-call write of the article
    /// plus all its entities and classifications.
    ///
    /// Returns an error if no Convex client is configured (read-only mode).
    pub async fn insert_article(
        &self,
        article: &FetchedArticle,
        entities: &[ExtractedEntity],
        offenses: &[OffenseClassification],
    ) -> Result<Uuid> {
        let convex = self
            .convex
            .as_ref()
            .context("Convex client not configured — cannot write articles")?;
        let batch_entities: Vec<BatchEntityArgs> = entities
            .iter()
            .map(|e| {
                let entity_type = format!("{:?}", e.entity_type).to_lowercase();
                BatchEntityArgs {
                    legacy_key: e.id.to_string(),
                    entity_name: e.name.clone(),
                    entity_type,
                    artist_id: e.artist_id.map(|id| id.to_string()),
                    confidence: Some(e.confidence),
                    metadata: None,
                }
            })
            .collect();

        let batch_classifications: Vec<BatchClassificationArgs> = offenses
            .iter()
            .map(|o| {
                let category = o.category.to_string();
                let severity = format!("{:?}", o.severity).to_lowercase();
                BatchClassificationArgs {
                    legacy_key: o.id.to_string(),
                    entity_id: o.entity_id.map(|id| id.to_string()),
                    artist_id: o.artist_id.map(|id| id.to_string()),
                    category,
                    severity,
                    confidence: Some(o.confidence),
                    human_verified: None,
                    metadata: Some(serde_json::json!({
                        "matchedKeywords": o.matched_keywords,
                        "context": o.context,
                        "needsReview": o.needs_review,
                        "classificationSource": o.classification_source,
                    })),
                }
            })
            .collect();

        let batch_article = BatchArticleArgs {
            legacy_key: article.id.to_string(),
            url: article.url.clone(),
            title: article.title.clone(),
            content: article.content.clone(),
            summary: article
                .content
                .as_ref()
                .map(|c| c[..c.len().min(500)].to_string()),
            published_at: article.published_at.map(|d| d.to_rfc3339()),
            processing_status: Some("completed".to_string()),
            embedding_generated: None,
            source_id: None,
            raw_data: None,
            metadata: Some(serde_json::json!({
                "authors": article.authors,
                "categories": article.categories,
            })),
            entities: if batch_entities.is_empty() {
                None
            } else {
                Some(batch_entities)
            },
            classifications: if batch_classifications.is_empty() {
                None
            } else {
                Some(batch_classifications)
            },
        };

        let response = convex
            .batch_ingest_articles(&[batch_article])
            .await
            .context("Failed to batch-ingest article into Convex")?;

        tracing::debug!(
            article_id = %article.id,
            url = %article.url,
            entities = entities.len(),
            offenses = offenses.len(),
            created = response.articles_created,
            updated = response.articles_updated,
            "Persisted article via Convex batch ingest"
        );

        Ok(article.id)
    }

    /// Log a fetch operation.
    ///
    /// Previously wrote to `news_fetch_log` in PostgreSQL. Now emits a
    /// structured log line instead (no Convex mutation for fetch logs yet).
    pub async fn log_fetch(
        &self,
        source_id: Uuid,
        articles_found: i32,
        articles_new: i32,
        status: &str,
        error: Option<&str>,
        duration_ms: i32,
    ) -> Result<()> {
        tracing::info!(
            source_id = %source_id,
            articles_found,
            articles_new,
            status,
            error = error.unwrap_or("none"),
            duration_ms,
            "Fetch log"
        );
        Ok(())
    }

    // ---------------------------------------------------------------
    // Read operations — PostgreSQL (migrated separately)
    // ---------------------------------------------------------------

    /// Check if an article URL already exists (PostgreSQL read).
    pub async fn article_exists(&self, url: &str) -> Result<bool> {
        let pool = self
            .db_pool
            .as_ref()
            .context("PostgreSQL pool not available for reads")?;

        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM news_articles WHERE url = $1) as "exists!""#,
            url
        )
        .fetch_one(pool)
        .await
        .context("Failed to check article existence")?;

        Ok(exists)
    }

    /// Get articles with pagination and filters (PostgreSQL read).
    pub async fn get_articles(
        &self,
        _filters: &ArticleFilters,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<ArticleSummary>> {
        let pool = self
            .db_pool
            .as_ref()
            .context("PostgreSQL pool not available for reads")?;

        let articles = sqlx::query_as!(
            ArticleSummary,
            r#"
            SELECT
                a.id,
                a.title,
                a.url,
                a.excerpt,
                s.name as source_name,
                a.published_at,
                COALESCE(e.entity_count, 0) as "entity_count!",
                COALESCE(o.offense_count, 0) as "offense_count!",
                a.processing_status
            FROM news_articles a
            LEFT JOIN news_sources s ON a.source_id = s.id
            LEFT JOIN (
                SELECT article_id, COUNT(*) as entity_count
                FROM news_article_entities
                GROUP BY article_id
            ) e ON a.id = e.article_id
            LEFT JOIN (
                SELECT article_id, COUNT(*) as offense_count
                FROM news_offense_classifications
                GROUP BY article_id
            ) o ON a.id = o.article_id
            WHERE a.processing_status = 'completed'
            ORDER BY a.published_at DESC NULLS LAST, a.fetched_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit as i64,
            offset as i64
        )
        .fetch_all(pool)
        .await
        .context("Failed to fetch articles")?;

        Ok(articles)
    }

    /// Get article by ID with full details (PostgreSQL read).
    pub async fn get_article_by_id(&self, article_id: Uuid) -> Result<Option<ArticleSummary>> {
        let pool = self
            .db_pool
            .as_ref()
            .context("PostgreSQL pool not available for reads")?;

        let article = sqlx::query_as!(
            ArticleSummary,
            r#"
            SELECT
                a.id,
                a.title,
                a.url,
                a.excerpt,
                s.name as source_name,
                a.published_at,
                COALESCE(e.entity_count, 0) as "entity_count!",
                COALESCE(o.offense_count, 0) as "offense_count!",
                a.processing_status
            FROM news_articles a
            LEFT JOIN news_sources s ON a.source_id = s.id
            LEFT JOIN (
                SELECT article_id, COUNT(*) as entity_count
                FROM news_article_entities
                GROUP BY article_id
            ) e ON a.id = e.article_id
            LEFT JOIN (
                SELECT article_id, COUNT(*) as offense_count
                FROM news_offense_classifications
                GROUP BY article_id
            ) o ON a.id = o.article_id
            WHERE a.id = $1
            "#,
            article_id
        )
        .fetch_optional(pool)
        .await
        .context("Failed to fetch article")?;

        Ok(article)
    }

    /// Get recent offenses (PostgreSQL read).
    pub async fn get_recent_offenses(&self, limit: i32) -> Result<Vec<serde_json::Value>> {
        let pool = self
            .db_pool
            .as_ref()
            .context("PostgreSQL pool not available for reads")?;

        let offenses = sqlx::query!(
            r#"
            SELECT
                o.id,
                o.offense_category::text as "category!",
                o.severity::text as "severity!",
                o.confidence,
                o.evidence_snippet,
                o.keywords_matched,
                o.created_at,
                a.title as article_title,
                a.url as article_url,
                e.entity_name,
                art.canonical_name as artist_name
            FROM news_offense_classifications o
            JOIN news_articles a ON o.article_id = a.id
            LEFT JOIN news_article_entities e ON o.entity_id = e.id
            LEFT JOIN artists art ON o.artist_id = art.id
            ORDER BY o.created_at DESC
            LIMIT $1
            "#,
            limit as i64
        )
        .fetch_all(pool)
        .await
        .context("Failed to fetch offenses")?;

        let results: Vec<serde_json::Value> = offenses
            .into_iter()
            .map(|o| {
                serde_json::json!({
                    "id": o.id,
                    "category": o.category,
                    "severity": o.severity,
                    "confidence": o.confidence,
                    "evidence_snippet": o.evidence_snippet,
                    "keywords_matched": o.keywords_matched,
                    "created_at": o.created_at,
                    "article_title": o.article_title,
                    "article_url": o.article_url,
                    "entity_name": o.entity_name,
                    "artist_name": o.artist_name
                })
            })
            .collect();

        Ok(results)
    }

    /// Get article count (PostgreSQL read).
    pub async fn get_article_count(&self) -> Result<i64> {
        let pool = self
            .db_pool
            .as_ref()
            .context("PostgreSQL pool not available for reads")?;

        let count = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM news_articles"#)
            .fetch_one(pool)
            .await
            .context("Failed to count articles")?;

        Ok(count)
    }

    /// Get offense count (PostgreSQL read).
    pub async fn get_offense_count(&self) -> Result<i64> {
        let pool = self
            .db_pool
            .as_ref()
            .context("PostgreSQL pool not available for reads")?;

        let count =
            sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM news_offense_classifications"#)
                .fetch_one(pool)
                .await
                .context("Failed to count offenses")?;

        Ok(count)
    }
}
