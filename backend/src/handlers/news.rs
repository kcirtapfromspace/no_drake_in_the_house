//! News Pipeline API Handlers
//!
//! Endpoints for news tracking, article search, and offense detection.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::AuthenticatedUser;
use crate::services::news_pipeline::{ArticleFilters, NewsRepository};
use crate::{AppError, AppState, Result};

/// Query parameters for listing articles
#[derive(Debug, Deserialize)]
pub struct ListArticlesQuery {
    /// Filter by artist ID
    pub artist_id: Option<Uuid>,
    /// Filter by source type (rss, newsapi, twitter, reddit)
    pub source_type: Option<String>,
    /// Filter by offense category
    pub offense_category: Option<String>,
    /// Only show articles with detected offenses
    #[serde(default)]
    pub has_offense: Option<bool>,
    /// Limit results
    #[serde(default = "default_limit")]
    pub limit: i32,
    /// Offset for pagination
    #[serde(default)]
    pub offset: i32,
    /// Sort by (published_at, fetched_at, relevance)
    #[serde(default = "default_sort")]
    pub sort: String,
    /// Sort order (asc, desc)
    #[serde(default = "default_order")]
    pub order: String,
}

fn default_limit() -> i32 {
    20
}

fn default_sort() -> String {
    "published_at".to_string()
}

fn default_order() -> String {
    "desc".to_string()
}

/// Semantic search request
#[derive(Debug, Deserialize)]
pub struct SemanticSearchRequest {
    /// Search query
    pub query: String,
    /// Filter by artist ID
    pub artist_id: Option<Uuid>,
    /// Filter by offense category
    pub offense_category: Option<String>,
    /// Number of results
    #[serde(default = "default_search_limit")]
    pub limit: usize,
    /// Minimum similarity score (0.0-1.0)
    #[serde(default = "default_min_score")]
    pub min_score: f32,
}

fn default_search_limit() -> usize {
    10
}

fn default_min_score() -> f32 {
    0.5
}

/// Request to verify an offense classification
#[derive(Debug, Deserialize)]
pub struct VerifyOffenseRequest {
    /// Is the classification correct?
    pub is_correct: bool,
    /// Optional correction if incorrect
    pub corrected_category: Option<String>,
    /// Optional correction for severity
    pub corrected_severity: Option<String>,
    /// Optional notes
    pub notes: Option<String>,
}

/// Response for article list
#[derive(Debug, Serialize)]
pub struct ArticleListResponse {
    pub success: bool,
    pub data: ArticleListData,
}

#[derive(Debug, Serialize)]
pub struct ArticleListData {
    pub articles: Vec<ArticleSummary>,
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Serialize)]
pub struct ArticleSummary {
    pub id: Uuid,
    pub url: String,
    pub title: String,
    pub source_type: String,
    pub source_name: Option<String>,
    pub published_at: Option<String>,
    pub fetched_at: String,
    pub entity_count: usize,
    pub offense_count: usize,
    pub top_offense: Option<String>,
}

/// Response for article detail
#[derive(Debug, Serialize)]
pub struct ArticleDetailResponse {
    pub success: bool,
    pub data: ArticleDetail,
}

#[derive(Debug, Serialize)]
pub struct ArticleDetail {
    pub id: Uuid,
    pub url: String,
    pub title: String,
    pub content: Option<String>,
    pub source: ArticleSource,
    pub published_at: Option<String>,
    pub fetched_at: String,
    pub authors: Vec<String>,
    pub image_url: Option<String>,
    pub entities: Vec<EntityInfo>,
    pub offenses: Vec<OffenseInfo>,
}

#[derive(Debug, Serialize)]
pub struct ArticleSource {
    pub id: Uuid,
    pub name: String,
    pub source_type: String,
    pub credibility_score: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct EntityInfo {
    pub id: Uuid,
    pub name: String,
    pub entity_type: String,
    pub confidence: f64,
    pub artist_id: Option<Uuid>,
    pub context: String,
}

#[derive(Debug, Serialize)]
pub struct OffenseInfo {
    pub id: Uuid,
    pub category: String,
    pub severity: String,
    pub confidence: f64,
    pub human_verified: bool,
    pub evidence_snippet: Option<String>,
}

/// List articles with filtering
pub async fn list_articles_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Query(query): Query<ListArticlesQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        artist_id = ?query.artist_id,
        source_type = ?query.source_type,
        offense_category = ?query.offense_category,
        has_offense = ?query.has_offense,
        limit = query.limit,
        offset = query.offset,
        "List articles request"
    );

    // Validate limit
    if query.limit < 1 || query.limit > 100 {
        return Err(AppError::InvalidFieldValue {
            field: "limit".to_string(),
            message: "Limit must be between 1 and 100".to_string(),
        });
    }

    // Query database
    let repository = NewsRepository::new(state.db_pool.clone());
    let filters = ArticleFilters {
        artist_id: query.artist_id,
        offense_category: query.offense_category.clone(),
        ..Default::default()
    };

    let articles = repository
        .get_articles(&filters, query.limit, query.offset)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to fetch articles: {}", e)),
        })?;

    let total = repository
        .get_article_count()
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to count articles: {}", e)),
        })?;

    // Convert to response format
    let article_summaries: Vec<serde_json::Value> = articles
        .into_iter()
        .map(|a| {
            serde_json::json!({
                "id": a.id,
                "url": a.url,
                "title": a.title,
                "source_type": "rss",
                "source_name": a.source_name,
                "published_at": a.published_at,
                "fetched_at": a.published_at,
                "entity_count": a.entity_count,
                "offense_count": a.offense_count,
                "top_offense": null
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "articles": article_summaries,
            "total": total,
            "limit": query.limit,
            "offset": query.offset
        }
    })))
}

/// Get article detail with entities and offenses
pub async fn get_article_handler(
    State(state): State<AppState>,
    Path(article_id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(article_id = %article_id, "Get article detail request");

    let repository = NewsRepository::new(state.db_pool.clone());
    let article = repository
        .get_article_by_id(article_id)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to fetch article: {}", e)),
        })?;

    match article {
        Some(a) => Ok(Json(serde_json::json!({
            "success": true,
            "data": {
                "id": a.id,
                "url": a.url,
                "title": a.title,
                "excerpt": a.excerpt,
                "source_name": a.source_name,
                "published_at": a.published_at,
                "entity_count": a.entity_count,
                "offense_count": a.offense_count,
                "processing_status": a.processing_status
            }
        }))),
        None => Err(AppError::NotFound {
            resource: "Article".to_string(),
        }),
    }
}

/// Get news mentions for a specific artist
pub async fn get_artist_mentions_handler(
    State(_state): State<AppState>,
    Path(artist_id): Path<Uuid>,
    _user: AuthenticatedUser,
    Query(query): Query<ListArticlesQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        artist_id = %artist_id,
        limit = query.limit,
        offset = query.offset,
        "Get artist mentions request"
    );

    // Return empty list for now
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "artist_id": artist_id,
            "mentions": [],
            "total": 0,
            "limit": query.limit,
            "offset": query.offset
        }
    })))
}

/// Semantic search for articles
pub async fn semantic_search_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Json(request): Json<SemanticSearchRequest>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        query = %request.query,
        artist_id = ?request.artist_id,
        limit = request.limit,
        min_score = request.min_score,
        "Semantic search request"
    );

    if request.query.trim().is_empty() {
        return Err(AppError::InvalidFieldValue {
            field: "query".to_string(),
            message: "Search query cannot be empty".to_string(),
        });
    }

    if request.query.len() > 500 {
        return Err(AppError::InvalidFieldValue {
            field: "query".to_string(),
            message: "Search query too long (max 500 characters)".to_string(),
        });
    }

    // Return empty results for now (requires LanceDB integration)
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "results": [],
            "query": request.query,
            "limit": request.limit
        }
    })))
}

/// Get recent offense detections
pub async fn get_offenses_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Query(query): Query<ListArticlesQuery>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        offense_category = ?query.offense_category,
        artist_id = ?query.artist_id,
        limit = query.limit,
        "Get offenses request"
    );

    let repository = NewsRepository::new(state.db_pool.clone());
    let offenses = repository
        .get_recent_offenses(query.limit)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to fetch offenses: {}", e)),
        })?;

    let total = repository
        .get_offense_count()
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to count offenses: {}", e)),
        })?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "offenses": offenses,
            "total": total,
            "limit": query.limit,
            "offset": query.offset
        }
    })))
}

/// Get a specific offense classification
pub async fn get_offense_handler(
    State(_state): State<AppState>,
    Path(offense_id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(offense_id = %offense_id, "Get offense detail request");

    // Return not found for now
    Err(AppError::NotFound {
        resource: "Offense".to_string(),
    })
}

/// Verify an offense classification (human review)
pub async fn verify_offense_handler(
    State(_state): State<AppState>,
    Path(offense_id): Path<Uuid>,
    _user: AuthenticatedUser,
    Json(request): Json<VerifyOffenseRequest>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(
        offense_id = %offense_id,
        is_correct = request.is_correct,
        corrected_category = ?request.corrected_category,
        "Verify offense request"
    );

    // Validate corrected category if provided
    if let Some(ref category) = request.corrected_category {
        let valid_categories = [
            "sexual_misconduct",
            "domestic_violence",
            "hate_speech",
            "racism",
            "antisemitism",
            "homophobia",
            "child_abuse",
            "animal_cruelty",
            "financial_crimes",
            "drug_offenses",
            "violent_crimes",
            "harassment",
            "plagiarism",
            "other",
        ];
        if !valid_categories.contains(&category.as_str()) {
            return Err(AppError::InvalidFieldValue {
                field: "corrected_category".to_string(),
                message: format!("Invalid category. Valid options: {:?}", valid_categories),
            });
        }
    }

    // Validate corrected severity if provided
    if let Some(ref severity) = request.corrected_severity {
        let valid_severities = ["low", "medium", "high", "critical"];
        if !valid_severities.contains(&severity.as_str()) {
            return Err(AppError::InvalidFieldValue {
                field: "corrected_severity".to_string(),
                message: format!("Invalid severity. Valid options: {:?}", valid_severities),
            });
        }
    }

    // Return not found for now
    Err(AppError::NotFound {
        resource: "Offense".to_string(),
    })
}

/// Get pipeline status and statistics
pub async fn get_pipeline_status_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Pipeline status request");

    let repository = NewsRepository::new(state.db_pool.clone());
    let article_count = repository.get_article_count().await.unwrap_or(0);
    let offense_count = repository.get_offense_count().await.unwrap_or(0);

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "is_running": false,
            "stats": {
                "articles_fetched": article_count,
                "rss_articles": article_count,
                "newsapi_articles": 0,
                "twitter_posts": 0,
                "reddit_posts": 0,
                "articles_scraped": 0,
                "entities_extracted": 0,
                "offenses_detected": offense_count,
                "embeddings_generated": 0,
                "errors": 0,
                "last_run": null,
                "last_run_duration_secs": null
            },
            "sources": {
                "rss_feeds": 10,
                "newsapi_enabled": true,
                "twitter_enabled": false,
                "reddit_enabled": true
            }
        }
    })))
}

/// Trigger a pipeline run
pub async fn trigger_pipeline_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    tracing::info!("Trigger pipeline request");

    // For now, return accepted but don't actually run
    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "success": true,
            "message": "Pipeline run triggered",
            "data": {
                "run_id": Uuid::new_v4()
            }
        })),
    ))
}

/// Get news sources
pub async fn get_sources_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Get news sources request");

    // Return default sources
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "sources": [
                {
                    "id": Uuid::new_v4(),
                    "name": "Pitchfork",
                    "source_type": "rss",
                    "url": "https://pitchfork.com/rss/news/",
                    "credibility_score": 4,
                    "poll_interval_minutes": 30
                },
                {
                    "id": Uuid::new_v4(),
                    "name": "Rolling Stone",
                    "source_type": "rss",
                    "url": "https://www.rollingstone.com/music/feed/",
                    "credibility_score": 4,
                    "poll_interval_minutes": 30
                },
                {
                    "id": Uuid::new_v4(),
                    "name": "Billboard",
                    "source_type": "rss",
                    "url": "https://www.billboard.com/feed/",
                    "credibility_score": 4,
                    "poll_interval_minutes": 30
                },
                {
                    "id": Uuid::new_v4(),
                    "name": "The Guardian Music",
                    "source_type": "rss",
                    "url": "https://www.theguardian.com/music/rss",
                    "credibility_score": 5,
                    "poll_interval_minutes": 60
                },
                {
                    "id": Uuid::new_v4(),
                    "name": "TMZ",
                    "source_type": "rss",
                    "url": "https://www.tmz.com/rss.xml",
                    "credibility_score": 2,
                    "poll_interval_minutes": 15
                },
                {
                    "id": Uuid::new_v4(),
                    "name": "NewsAPI",
                    "source_type": "newsapi",
                    "url": "https://newsapi.org",
                    "credibility_score": 3,
                    "poll_interval_minutes": 60
                },
                {
                    "id": Uuid::new_v4(),
                    "name": "r/hiphopheads",
                    "source_type": "reddit",
                    "url": "https://reddit.com/r/hiphopheads",
                    "credibility_score": 2,
                    "poll_interval_minutes": 30
                },
                {
                    "id": Uuid::new_v4(),
                    "name": "r/Music",
                    "source_type": "reddit",
                    "url": "https://reddit.com/r/Music",
                    "credibility_score": 2,
                    "poll_interval_minutes": 30
                }
            ]
        }
    })))
}

/// Get trending topics/artists from news
pub async fn get_trending_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Get trending request");

    // Return empty trending for now
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "trending_artists": [],
            "trending_topics": [],
            "period": "24h"
        }
    })))
}

/// Get offense categories
pub async fn get_offense_categories_handler(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Get offense categories request");

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "categories": [
                { "id": "sexual_misconduct", "name": "Sexual Misconduct", "severity_default": "critical" },
                { "id": "domestic_violence", "name": "Domestic Violence", "severity_default": "critical" },
                { "id": "hate_speech", "name": "Hate Speech", "severity_default": "high" },
                { "id": "racism", "name": "Racism", "severity_default": "high" },
                { "id": "antisemitism", "name": "Antisemitism", "severity_default": "high" },
                { "id": "homophobia", "name": "Homophobia", "severity_default": "high" },
                { "id": "child_abuse", "name": "Child Abuse", "severity_default": "critical" },
                { "id": "animal_cruelty", "name": "Animal Cruelty", "severity_default": "high" },
                { "id": "financial_crimes", "name": "Financial Crimes", "severity_default": "medium" },
                { "id": "drug_offenses", "name": "Drug Offenses", "severity_default": "medium" },
                { "id": "violent_crimes", "name": "Violent Crimes", "severity_default": "critical" },
                { "id": "harassment", "name": "Harassment", "severity_default": "medium" },
                { "id": "plagiarism", "name": "Plagiarism", "severity_default": "low" },
                { "id": "other", "name": "Other", "severity_default": "low" }
            ],
            "severities": [
                { "id": "low", "name": "Low", "description": "Minor issue, may not warrant blocking" },
                { "id": "medium", "name": "Medium", "description": "Moderate concern, warrants attention" },
                { "id": "high", "name": "High", "description": "Serious offense, recommend blocking" },
                { "id": "critical", "name": "Critical", "description": "Severe offense, strongly recommend blocking" }
            ]
        }
    })))
}
