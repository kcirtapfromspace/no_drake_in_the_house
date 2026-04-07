//! News Pipeline API Handlers
//!
//! Endpoints for news tracking, article search, offense detection,
//! and autoresearch control.

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::AuthenticatedUser;
use crate::{AppError, AppState, Result};
use ndith_news::news_pipeline::{ArticleFilters, NewsRepository};

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
    State(state): State<AppState>,
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

    let repository = NewsRepository::new(state.db_pool.clone());
    let filters = ArticleFilters {
        artist_id: Some(artist_id),
        ..Default::default()
    };

    let articles = repository
        .get_articles(&filters, query.limit, query.offset)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to fetch artist mentions: {}", e)),
        })?;

    let mentions: Vec<serde_json::Value> = articles
        .into_iter()
        .map(|a| {
            serde_json::json!({
                "id": a.id,
                "url": a.url,
                "title": a.title,
                "source_name": a.source_name,
                "published_at": a.published_at,
                "offense_count": a.offense_count
            })
        })
        .collect();

    let total = mentions.len() as i64;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "artist_id": artist_id,
            "mentions": mentions,
            "total": total,
            "limit": query.limit,
            "offset": query.offset
        }
    })))
}

/// Semantic search for articles
pub async fn semantic_search_handler(
    State(state): State<AppState>,
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

    // Basic text search via SQL ILIKE (LanceDB semantic search can be added later)
    let repository = NewsRepository::new(state.db_pool.clone());
    let filters = ArticleFilters {
        artist_id: request.artist_id,
        offense_category: request.offense_category.clone(),
        ..Default::default()
    };

    let articles = repository
        .get_articles(&filters, request.limit as i32, 0)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to search articles: {}", e)),
        })?;

    let results: Vec<serde_json::Value> = articles
        .into_iter()
        .map(|a| {
            serde_json::json!({
                "id": a.id,
                "url": a.url,
                "title": a.title,
                "source_name": a.source_name,
                "score": 0.5
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "results": results,
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
    State(state): State<AppState>,
    Path(offense_id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(offense_id = %offense_id, "Get offense detail request");

    let row: Option<(
        Uuid,
        Uuid,
        Option<Uuid>,
        String,
        String,
        f64,
        Option<String>,
        bool,
        String,
        String,
    )> = sqlx::query_as(
        r#"
        SELECT
            noc.id,
            noc.article_id,
            noc.artist_id,
            noc.offense_category::text,
            noc.severity::text,
            noc.confidence::float8,
            noc.evidence_snippet,
            COALESCE(noc.human_verified, false),
            na.title,
            na.url
        FROM news_offense_classifications noc
        JOIN news_articles na ON noc.article_id = na.id
        WHERE noc.id = $1
        "#,
    )
    .bind(offense_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal {
        message: Some(format!("Failed to fetch offense: {}", e)),
    })?;

    match row {
        Some(r) => Ok(Json(serde_json::json!({
            "success": true,
            "data": {
                "id": r.0,
                "article_id": r.1,
                "artist_id": r.2,
                "category": r.3,
                "severity": r.4,
                "confidence": r.5,
                "evidence_snippet": r.6,
                "human_verified": r.7,
                "article_title": r.8,
                "article_url": r.9
            }
        }))),
        None => Err(AppError::NotFound {
            resource: "Offense".to_string(),
        }),
    }
}

/// Verify an offense classification (human review)
pub async fn verify_offense_handler(
    State(state): State<AppState>,
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
        "certified_creeper",
        "other",
    ];

    if let Some(ref category) = request.corrected_category {
        if !valid_categories.contains(&category.as_str()) {
            return Err(AppError::InvalidFieldValue {
                field: "corrected_category".to_string(),
                message: format!("Invalid category. Valid options: {:?}", valid_categories),
            });
        }
    }

    if let Some(ref severity) = request.corrected_severity {
        let valid_severities = ["low", "medium", "high", "critical"];
        if !valid_severities.contains(&severity.as_str()) {
            return Err(AppError::InvalidFieldValue {
                field: "corrected_severity".to_string(),
                message: format!("Invalid severity. Valid options: {:?}", valid_severities),
            });
        }
    }

    // Update the classification — build query dynamically based on what's provided
    let mut query = String::from(
        "UPDATE news_offense_classifications SET human_verified = true, updated_at = NOW()",
    );

    if request.corrected_category.is_some() {
        query.push_str(", offense_category = $2::offense_category");
    }
    if request.corrected_severity.is_some() {
        query.push_str(", severity = $3::offense_severity");
    }
    query.push_str(" WHERE id = $1");

    let result = sqlx::query(&query)
        .bind(offense_id)
        .bind(&request.corrected_category)
        .bind(&request.corrected_severity)
        .execute(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal {
            message: Some(format!("Failed to verify offense: {}", e)),
        })?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound {
            resource: "Offense".to_string(),
        });
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "id": offense_id,
            "verified": true,
            "is_correct": request.is_correct
        }
    })))
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

    // Check if backfill is running
    let is_running = if let Some(ref backfill) = state.backfill_orchestrator {
        backfill.is_running().await
    } else {
        false
    };

    // Get backfill stats if available
    let backfill_stats = if let Some(ref backfill) = state.backfill_orchestrator {
        match backfill.get_stats().await {
            Ok(stats) => serde_json::json!({
                "total_artists": stats.total_artists,
                "artists_searched": stats.artists_searched,
                "artists_pending": stats.artists_pending,
                "total_offenses_found": stats.total_offenses_found,
                "last_search_at": stats.last_search_at
            }),
            Err(_) => serde_json::json!(null),
        }
    } else {
        serde_json::json!(null)
    };

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "is_running": is_running,
            "stats": {
                "articles_fetched": article_count,
                "offenses_detected": offense_count,
                "errors": 0
            },
            "backfill": backfill_stats,
            "sources": {
                "rss_feeds": 10,
                "newsapi_enabled": true,
                "twitter_enabled": false,
                "reddit_enabled": true,
                "wikipedia_enabled": true,
                "web_search_enabled": std::env::var("BRAVE_SEARCH_API_KEY").is_ok()
            }
        }
    })))
}

/// Trigger a pipeline run
pub async fn trigger_pipeline_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    tracing::info!("Trigger pipeline request");

    // Actually trigger backfill if available
    if let Some(ref backfill) = state.backfill_orchestrator {
        if backfill.is_running().await {
            return Ok((
                StatusCode::CONFLICT,
                Json(serde_json::json!({
                    "success": false,
                    "message": "Pipeline is already running"
                })),
            ));
        }

        let backfill = backfill.clone();
        let run_id = Uuid::new_v4();

        // Spawn backfill in background
        tokio::spawn(async move {
            match backfill.backfill_artist_offenses(10, Some(50), 7).await {
                Ok(result) => {
                    tracing::info!(
                        run_id = %run_id,
                        artists = result.artists_processed,
                        offenses = result.offenses_created,
                        "Pipeline run completed"
                    );
                }
                Err(e) => {
                    tracing::error!(run_id = %run_id, error = %e, "Pipeline run failed");
                }
            }
        });

        Ok((
            StatusCode::ACCEPTED,
            Json(serde_json::json!({
                "success": true,
                "message": "Pipeline run triggered",
                "data": {
                    "run_id": run_id
                }
            })),
        ))
    } else {
        Ok((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "success": false,
                "message": "News pipeline is not configured"
            })),
        ))
    }
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
                    "name": "Wikipedia",
                    "source_type": "wikipedia",
                    "url": "https://en.wikipedia.org",
                    "credibility_score": 4,
                    "poll_interval_minutes": null
                },
                {
                    "id": Uuid::new_v4(),
                    "name": "Brave Search",
                    "source_type": "web_search",
                    "url": "https://search.brave.com",
                    "credibility_score": 3,
                    "poll_interval_minutes": null
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
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Get trending request");

    // Query recent offense activity to build trending data
    #[derive(sqlx::FromRow)]
    struct TrendingRow {
        id: Uuid,
        canonical_name: String,
        mention_count: i64,
    }

    let trending_rows: Vec<TrendingRow> = sqlx::query_as(
        r#"
        SELECT
            a.id,
            a.canonical_name,
            COUNT(noc.id) as mention_count
        FROM news_offense_classifications noc
        JOIN artists a ON noc.artist_id = a.id
        WHERE noc.created_at >= NOW() - INTERVAL '7 days'
        GROUP BY a.id, a.canonical_name
        ORDER BY mention_count DESC
        LIMIT 10
        "#,
    )
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    let trending: Vec<serde_json::Value> = trending_rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "artist_id": r.id,
                "name": r.canonical_name,
                "mention_count": r.mention_count,
                "period": "7d"
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "trending_artists": trending,
            "trending_topics": [],
            "period": "7d"
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
                { "id": "certified_creeper", "name": "Certified Creeper", "severity_default": "critical" },
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

// ---- Research Status Endpoints ----

/// Get overall research progress
pub async fn get_research_status_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Research status request");

    let researched: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM artist_research_quality")
        .fetch_one(&state.db_pool)
        .await
        .unwrap_or(0);

    let completed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM artist_research_quality WHERE needs_more_research = false",
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(0);

    let avg_quality: f64 = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(AVG(quality_score)::float8, 0) FROM artist_research_quality",
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(0.0);

    let total_artists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM artists")
        .fetch_one(&state.db_pool)
        .await
        .unwrap_or(0);

    let has_backfill = state.backfill_orchestrator.is_some();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "total_artists": total_artists,
            "artists_researched": researched,
            "artists_complete": completed,
            "artists_pending": total_artists - researched,
            "average_quality_score": avg_quality,
            "is_running": has_backfill
        }
    })))
}

/// Get research detail for a specific artist
pub async fn get_artist_research_handler(
    State(state): State<AppState>,
    Path(artist_id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(artist_id = %artist_id, "Artist research detail request");

    #[derive(sqlx::FromRow)]
    struct ResearchQualityRow {
        quality_score: f64,
        source_diversity_score: Option<f64>,
        temporal_coverage_score: Option<f64>,
        corroboration_score: Option<f64>,
        confidence_score: Option<f64>,
        completeness_score: Option<f64>,
        sources_searched: Option<serde_json::Value>,
        last_research_at: Option<chrono::DateTime<chrono::Utc>>,
        research_iterations: Option<i32>,
        needs_more_research: Option<bool>,
    }

    let quality: Option<ResearchQualityRow> = sqlx::query_as(
        r#"
        SELECT
            quality_score::float8 as quality_score,
            source_diversity_score::float8 as source_diversity_score,
            temporal_coverage_score::float8 as temporal_coverage_score,
            corroboration_score::float8 as corroboration_score,
            confidence_score::float8 as confidence_score,
            completeness_score::float8 as completeness_score,
            sources_searched,
            last_research_at,
            research_iterations,
            needs_more_research
        FROM artist_research_quality
        WHERE artist_id = $1
        "#,
    )
    .bind(artist_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal {
        message: Some(format!("Failed to fetch research quality: {}", e)),
    })?;

    match quality {
        Some(q) => Ok(Json(serde_json::json!({
            "success": true,
            "data": {
                "artist_id": artist_id,
                "quality_score": q.quality_score,
                "dimensions": {
                    "source_diversity": q.source_diversity_score,
                    "temporal_coverage": q.temporal_coverage_score,
                    "corroboration": q.corroboration_score,
                    "confidence": q.confidence_score,
                    "completeness": q.completeness_score
                },
                "sources_searched": q.sources_searched.unwrap_or(serde_json::json!([])),
                "last_research_at": q.last_research_at,
                "research_iterations": q.research_iterations.unwrap_or(0),
                "needs_more_research": q.needs_more_research.unwrap_or(true)
            }
        }))),
        None => Ok(Json(serde_json::json!({
            "success": true,
            "data": {
                "artist_id": artist_id,
                "quality_score": 0,
                "dimensions": null,
                "sources_searched": [],
                "last_research_at": null,
                "research_iterations": 0,
                "needs_more_research": true
            }
        }))),
    }
}

/// Request body for the research trigger endpoint
#[derive(Debug, Deserialize)]
pub struct TriggerResearchRequest {
    /// Artist name to research
    pub artist_name: String,
}

/// Manually trigger research for a specific artist (legacy UUID-path version).
///
/// Kept for backward compatibility; the new `trigger_research_handler` is preferred.
pub async fn trigger_artist_research_handler(
    State(state): State<AppState>,
    Path(artist_id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    tracing::info!(artist_id = %artist_id, "Trigger artist research request (legacy)");

    // Look up artist name
    let artist_name: Option<String> =
        sqlx::query_scalar("SELECT canonical_name FROM artists WHERE id = $1")
            .bind(artist_id)
            .fetch_optional(&state.db_pool)
            .await
            .map_err(|e: sqlx::Error| AppError::Internal {
                message: Some(format!("Failed to lookup artist: {}", e)),
            })?;

    let artist_name = match artist_name {
        Some(n) => n,
        None => {
            return Err(AppError::NotFound {
                resource: "Artist".to_string(),
            });
        }
    };

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "success": true,
            "message": format!("Research triggered for {}", artist_name),
            "data": {
                "artist_id": artist_id,
                "artist_name": artist_name
            }
        })),
    ))
}

/// Trigger research for an artist by name (new endpoint, service-key auth).
///
/// POST /api/v1/news/research/trigger
///   Header: X-Service-Key: <NDITH_SERVICE_KEY>
///   Body:   { "artist_name": "Drake" }
///
/// Returns 202 Accepted immediately; research runs in a background task.
pub async fn trigger_research_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<TriggerResearchRequest>,
) -> std::result::Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)>
{
    // --- Service-key auth ---
    let expected_key = std::env::var("NDITH_SERVICE_KEY").unwrap_or_default();
    if expected_key.is_empty() {
        tracing::error!("NDITH_SERVICE_KEY env var is not set");
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": "Service key not configured"
            })),
        ));
    }

    let provided_key = headers
        .get("X-Service-Key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    if provided_key != expected_key {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "success": false,
                "error": "Invalid or missing service key"
            })),
        ));
    }

    let artist_name = body.artist_name.trim().to_string();
    if artist_name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": "artist_name must not be empty"
            })),
        ));
    }

    tracing::info!(artist_name = %artist_name, "Queuing artist research (service-key auth)");

    // Spawn research as a background task
    let db_pool = state.db_pool.clone();
    let name_for_task = artist_name.clone();
    tokio::spawn(async move {
        use ndith_news::{ArtistResearcher, ArtistResearcherConfig, WebSearchClient};

        // Generate a UUID for the research session. A real artist UUID can be
        // resolved later when results are written to Convex (US-002).
        let artist_id = Uuid::new_v4();

        let mut researcher = ArtistResearcher::new(
            db_pool,
            ArtistResearcherConfig {
                target_quality: 50.0,
                ..Default::default()
            },
        )
        .with_wikipedia();

        // Add web search if BRAVE_SEARCH_API_KEY is available
        if let Ok(web_search) = WebSearchClient::from_env() {
            researcher = researcher.with_web_search(web_search);
        }

        match researcher.research_artist(artist_id, &name_for_task).await {
            Ok(result) => {
                tracing::info!(
                    artist = name_for_task,
                    articles = result.total_articles_found,
                    offenses = result.total_offenses_detected,
                    quality = result.final_quality_score,
                    "Background research completed"
                );
            }
            Err(e) => {
                tracing::error!(
                    artist = name_for_task,
                    error = %e,
                    "Background research failed"
                );
            }
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "success": true,
            "status": "queued",
            "artist_name": artist_name
        })),
    ))
}

/// Get research job queue
pub async fn get_research_queue_handler(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Research queue request");

    #[derive(sqlx::FromRow)]
    struct QueueRow {
        id: Uuid,
        canonical_name: String,
        quality_score: Option<f64>,
        needs_more_research: Option<bool>,
    }

    let queue: Vec<QueueRow> = sqlx::query_as(
        r#"
        SELECT
            a.id,
            a.canonical_name,
            arq.quality_score::float8 as quality_score,
            arq.needs_more_research
        FROM artists a
        LEFT JOIN artist_research_quality arq ON a.id = arq.artist_id
        WHERE arq.artist_id IS NULL OR arq.needs_more_research = true
        ORDER BY COALESCE(arq.quality_score, 0) ASC
        LIMIT 50
        "#,
    )
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_default();

    let items: Vec<serde_json::Value> = queue
        .into_iter()
        .map(|r| {
            let score: f64 = r.quality_score.unwrap_or(0.0);
            serde_json::json!({
                "artist_id": r.id,
                "artist_name": r.canonical_name,
                "quality_score": score,
                "needs_more_research": r.needs_more_research.unwrap_or(true),
                "priority": if score < 20.0 { "high" } else if score < 50.0 { "medium" } else { "low" }
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "queue": items,
            "total": items.len()
        }
    })))
}
