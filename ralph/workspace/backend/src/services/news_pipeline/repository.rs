//! News Repository
//!
//! Database persistence layer for news articles, entities, and offenses.

use super::ingestion::FetchedArticle;
use super::processing::{ExtractedEntity, OffenseClassification};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for persisting news data to the database
pub struct NewsRepository {
    db_pool: PgPool,
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
    /// Create a new news repository
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Check if an article URL already exists
    pub async fn article_exists(&self, url: &str) -> Result<bool> {
        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM news_articles WHERE url = $1) as "exists!""#,
            url
        )
        .fetch_one(&self.db_pool)
        .await
        .context("Failed to check article existence")?;

        Ok(exists)
    }

    /// Insert a processed article with entities and offenses
    pub async fn insert_article(
        &self,
        article: &FetchedArticle,
        entities: &[ExtractedEntity],
        offenses: &[OffenseClassification],
    ) -> Result<Uuid> {
        // Insert the article
        let article_id = sqlx::query_scalar!(
            r#"
            INSERT INTO news_articles (
                id, source_id, url, title, content, excerpt,
                author, published_at, fetched_at, image_url,
                word_count, processing_status, processed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'completed', NOW())
            ON CONFLICT (url) DO UPDATE SET
                title = EXCLUDED.title,
                content = EXCLUDED.content,
                processing_status = 'completed',
                processed_at = NOW()
            RETURNING id
            "#,
            article.id,
            article.source_id,
            &article.url,
            &article.title,
            article.content.as_deref(),
            article.content.as_ref().map(|c| &c[..c.len().min(500)]), // Excerpt
            article.authors.first().map(|s| s.as_str()),
            article.published_at,
            article.fetched_at,
            article.image_url.as_deref(),
            article
                .content
                .as_ref()
                .map(|c| c.split_whitespace().count() as i32)
        )
        .fetch_one(&self.db_pool)
        .await
        .context("Failed to insert article")?;

        // Insert entities
        for entity in entities {
            self.insert_entity(article_id, entity).await?;
        }

        // Insert offenses
        for offense in offenses {
            self.insert_offense(article_id, offense).await?;
        }

        tracing::debug!(
            article_id = %article_id,
            url = %article.url,
            entities = entities.len(),
            offenses = offenses.len(),
            "Persisted article with entities and offenses"
        );

        Ok(article_id)
    }

    /// Insert an extracted entity
    async fn insert_entity(&self, article_id: Uuid, entity: &ExtractedEntity) -> Result<Uuid> {
        let entity_type = format!("{:?}", entity.entity_type).to_lowercase();

        let entity_id = sqlx::query_scalar!(
            r#"
            INSERT INTO news_article_entities (
                id, article_id, artist_id, entity_name, entity_type,
                confidence, context_snippet, position_start, position_end
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT DO NOTHING
            RETURNING id
            "#,
            entity.id,
            article_id,
            entity.artist_id,
            &entity.name,
            entity_type,
            entity.confidence as f32,
            &entity.context,
            entity.position.0 as i32,
            entity.position.1 as i32
        )
        .fetch_optional(&self.db_pool)
        .await
        .context("Failed to insert entity")?;

        Ok(entity_id.unwrap_or(entity.id))
    }

    /// Insert an offense classification
    async fn insert_offense(
        &self,
        article_id: Uuid,
        offense: &OffenseClassification,
    ) -> Result<Uuid> {
        let category = format!("{:?}", offense.category).to_lowercase();
        let severity = format!("{:?}", offense.severity).to_lowercase();

        // Use raw query to avoid sqlx enum type mapping issues
        let offense_id: Option<Uuid> = sqlx::query_scalar(
            r#"
            INSERT INTO news_offense_classifications (
                id, article_id, entity_id, artist_id,
                offense_category, severity, confidence,
                evidence_snippet, keywords_matched
            )
            VALUES (
                $1, $2, $3, $4,
                $5::offense_category, $6::offense_severity, $7,
                $8, $9
            )
            ON CONFLICT DO NOTHING
            RETURNING id
            "#,
        )
        .bind(offense.id)
        .bind(article_id)
        .bind(offense.entity_id)
        .bind(offense.artist_id)
        .bind(&category)
        .bind(&severity)
        .bind(offense.confidence as f32)
        .bind(&offense.context)
        .bind(&offense.matched_keywords)
        .fetch_optional(&self.db_pool)
        .await
        .context("Failed to insert offense")?;

        Ok(offense_id.unwrap_or(offense.id))
    }

    /// Get articles with pagination and filters
    pub async fn get_articles(
        &self,
        filters: &ArticleFilters,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<ArticleSummary>> {
        // Build dynamic query based on filters
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
        .fetch_all(&self.db_pool)
        .await
        .context("Failed to fetch articles")?;

        Ok(articles)
    }

    /// Get article by ID with full details
    pub async fn get_article_by_id(&self, article_id: Uuid) -> Result<Option<ArticleSummary>> {
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
        .fetch_optional(&self.db_pool)
        .await
        .context("Failed to fetch article")?;

        Ok(article)
    }

    /// Get recent offenses
    pub async fn get_recent_offenses(&self, limit: i32) -> Result<Vec<serde_json::Value>> {
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
        .fetch_all(&self.db_pool)
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

    /// Get article count
    pub async fn get_article_count(&self) -> Result<i64> {
        let count = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM news_articles"#)
            .fetch_one(&self.db_pool)
            .await
            .context("Failed to count articles")?;

        Ok(count)
    }

    /// Get offense count
    pub async fn get_offense_count(&self) -> Result<i64> {
        let count =
            sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM news_offense_classifications"#)
                .fetch_one(&self.db_pool)
                .await
                .context("Failed to count offenses")?;

        Ok(count)
    }

    /// Update fetch log for a source
    pub async fn log_fetch(
        &self,
        source_id: Uuid,
        articles_found: i32,
        articles_new: i32,
        status: &str,
        error: Option<&str>,
        duration_ms: i32,
    ) -> Result<()> {
        // Use raw query to avoid sqlx type inference issues with interval multiplication
        sqlx::query(
            r#"
            INSERT INTO news_fetch_log (
                source_id, fetch_started_at, fetch_completed_at,
                status, articles_found, articles_new, error_message, response_time_ms
            )
            VALUES ($1, NOW() - make_interval(secs => $6::double precision / 1000.0), NOW(), $2, $3, $4, $5, $6)
            "#,
        )
        .bind(source_id)
        .bind(status)
        .bind(articles_found)
        .bind(articles_new)
        .bind(error)
        .bind(duration_ms)
        .execute(&self.db_pool)
        .await
        .context("Failed to log fetch")?;

        Ok(())
    }
}
