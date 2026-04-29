//! Convex HTTP Client
//!
//! Thin HTTP client for calling Convex mutations from the Rust backend.
//! Used by the research pipeline to persist articles, entities, classifications,
//! offense records, and research quality scores directly into Convex.
//!
//! The client reads `CONVEX_URL` from the environment and calls the public
//! Convex HTTP API (`POST /api/mutation`). Retries transient failures up to
//! 3 times with exponential backoff.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Maximum number of retry attempts for transient errors.
const MAX_RETRIES: u32 = 3;

/// Base delay for exponential backoff (doubles each attempt).
const BASE_BACKOFF: Duration = Duration::from_millis(500);

/// HTTP request timeout.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Raw JSON envelope returned by the Convex HTTP API.
#[derive(Debug, Deserialize)]
#[serde(tag = "status")]
enum ConvexRawResponse {
    #[serde(rename = "success")]
    Success { value: serde_json::Value },
    #[serde(rename = "error")]
    Error {
        #[serde(rename = "errorMessage")]
        error_message: String,
    },
}

// ---------------------------------------------------------------------------
// Typed argument structs for each mutation
// ---------------------------------------------------------------------------

/// Arguments for `newsIngestion:ingestArticle`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestArticleArgs {
    pub legacy_key: String,
    pub url: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_generated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Entity within a batch article.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchEntityArgs {
    pub legacy_key: String,
    pub entity_name: String,
    pub entity_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Classification within a batch article.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchClassificationArgs {
    pub legacy_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<String>,
    pub category: String,
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub human_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// A single article within a batch ingest call.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchArticleArgs {
    pub legacy_key: String,
    pub url: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_generated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entities: Option<Vec<BatchEntityArgs>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classifications: Option<Vec<BatchClassificationArgs>>,
}

/// Arguments for `newsIngestion:ingestEntities`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestEntitiesArgs {
    pub article_id: String,
    pub entities: Vec<BatchEntityArgs>,
}

/// Arguments for `newsIngestion:ingestClassification`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestClassificationArgs {
    pub legacy_key: String,
    pub article_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<String>,
    pub category: String,
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub human_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Arguments for `newsIngestion:createOffenseFromResearch`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOffenseArgs {
    pub artist_id: String,
    pub category: String,
    pub severity: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub confidence: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_article_url: Option<String>,
}

/// Arguments for `newsIngestion:linkOffenseEvidence`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkEvidenceArgs {
    pub offense_id: String,
    pub source_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credibility_score: Option<f64>,
}

/// Arguments for `newsIngestion:updateArtistResearchQuality`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResearchQualityArgs {
    pub artist_id: String,
    pub quality_score: f64,
    pub sources_searched: Vec<String>,
    pub research_iterations: f64,
}

// ---------------------------------------------------------------------------
// Common response types
// ---------------------------------------------------------------------------

/// Upsert response returned by most ingestion mutations.
#[derive(Debug, Clone, Deserialize)]
pub struct UpsertResponse {
    pub id: String,
    pub upserted: String,
}

/// Batch ingest response returned by `batchIngestArticles`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchIngestResponse {
    pub articles_created: u32,
    pub articles_updated: u32,
    pub entities_inserted: u32,
    pub classifications_inserted: u32,
    pub total_articles: u32,
}

/// Entity ingest response returned by `ingestEntities`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityIngestResponse {
    pub article_id: String,
    pub inserted: u32,
    pub updated: u32,
    pub total: u32,
}

/// Research quality update response.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdatedResponse {
    pub id: String,
    pub updated: bool,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// HTTP client for calling Convex mutations.
///
/// This struct is `Clone + Send + Sync` and safe to share across async tasks.
#[derive(Clone)]
pub struct ConvexClient {
    /// The base URL of the Convex deployment (e.g. `https://scrupulous-emu-861.convex.cloud`).
    base_url: String,
    /// Reusable reqwest HTTP client.
    http: Client,
}

/// The request body sent to `POST /api/mutation`.
#[derive(Serialize)]
struct MutationRequest<'a> {
    path: &'a str,
    args: serde_json::Value,
}

impl ConvexClient {
    /// Create a new client by reading `CONVEX_URL` from the environment.
    ///
    /// Returns an error if the variable is not set.
    pub fn from_env() -> Result<Self> {
        let url = std::env::var("CONVEX_URL").context("CONVEX_URL environment variable not set")?;
        Ok(Self::new(url))
    }

    /// Create a new client with an explicit deployment URL.
    pub fn new(base_url: String) -> Self {
        let base_url = base_url.trim_end_matches('/').to_string();

        let http = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("Failed to build HTTP client");

        Self { base_url, http }
    }

    // -------------------------------------------------------------------
    // Generic mutation caller
    // -------------------------------------------------------------------

    /// Call a Convex mutation by function path and return the deserialized value.
    ///
    /// Retries transient errors (network failures, 5xx responses) up to
    /// [`MAX_RETRIES`] times with exponential backoff. Non-transient errors
    /// (4xx, Convex application errors) are returned immediately.
    pub async fn call_mutation<A: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        args: &A,
    ) -> Result<R> {
        let args_value =
            serde_json::to_value(args).context("Failed to serialize mutation arguments")?;

        let body = MutationRequest {
            path,
            args: args_value,
        };

        let url = format!("{}/api/mutation", self.base_url);
        let mut last_err: Option<anyhow::Error> = None;

        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                let backoff = BASE_BACKOFF * 2u32.pow(attempt - 1);
                tracing::warn!(
                    path,
                    attempt,
                    backoff_ms = backoff.as_millis() as u64,
                    "Retrying Convex mutation"
                );
                tokio::time::sleep(backoff).await;
            }

            let response = match self
                .http
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    // Network errors are transient — retry
                    tracing::warn!(
                        path,
                        attempt,
                        error = %e,
                        "Convex mutation network error"
                    );
                    last_err = Some(e.into());
                    continue;
                }
            };

            let status = response.status();

            // 5xx → transient, retry
            if status.is_server_error() {
                let body_text = response.text().await.unwrap_or_default();
                tracing::warn!(
                    path,
                    attempt,
                    status = status.as_u16(),
                    body = %&body_text[..body_text.len().min(300)],
                    "Convex mutation server error"
                );
                last_err = Some(anyhow::anyhow!(
                    "Convex server error {}: {}",
                    status,
                    &body_text[..body_text.len().min(300)]
                ));
                continue;
            }

            // 4xx → non-transient, fail immediately
            if status.is_client_error() {
                let body_text = response.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!(
                    "Convex client error {} calling {}: {}",
                    status,
                    path,
                    &body_text[..body_text.len().min(500)]
                ));
            }

            // Parse the Convex response envelope
            let raw: ConvexRawResponse = response
                .json()
                .await
                .context("Failed to parse Convex response JSON")?;

            return match raw {
                ConvexRawResponse::Success { value } => serde_json::from_value(value.clone())
                    .with_context(|| {
                        format!(
                            "Failed to deserialize Convex response for {}: {}",
                            path, value
                        )
                    }),
                ConvexRawResponse::Error { error_message } => Err(anyhow::anyhow!(
                    "Convex mutation {} returned error: {}",
                    path,
                    error_message
                )),
            };
        }

        Err(last_err.unwrap_or_else(|| {
            anyhow::anyhow!(
                "Convex mutation {} failed after {} retries",
                path,
                MAX_RETRIES
            )
        }))
    }

    // -------------------------------------------------------------------
    // Typed helper methods
    // -------------------------------------------------------------------

    /// Ingest a single article into Convex.
    ///
    /// Calls `newsIngestion:ingestArticle`. Upserts by `legacyKey`.
    pub async fn ingest_article(&self, args: &IngestArticleArgs) -> Result<UpsertResponse> {
        self.call_mutation("newsIngestion:ingestArticle", args)
            .await
    }

    /// Ingest entities for an article.
    ///
    /// Calls `newsIngestion:ingestEntities`.
    pub async fn ingest_entities(&self, args: &IngestEntitiesArgs) -> Result<EntityIngestResponse> {
        self.call_mutation("newsIngestion:ingestEntities", args)
            .await
    }

    /// Ingest a single offense classification.
    ///
    /// Calls `newsIngestion:ingestClassification`. Upserts by `legacyKey`.
    pub async fn ingest_classification(
        &self,
        args: &IngestClassificationArgs,
    ) -> Result<UpsertResponse> {
        self.call_mutation("newsIngestion:ingestClassification", args)
            .await
    }

    /// Batch-ingest articles (with optional entities and classifications).
    ///
    /// Calls `newsIngestion:batchIngestArticles`. Preferred over individual
    /// calls for efficiency.
    pub async fn batch_ingest_articles(
        &self,
        articles: &[BatchArticleArgs],
    ) -> Result<BatchIngestResponse> {
        #[derive(Serialize)]
        struct BatchArgs<'a> {
            articles: &'a [BatchArticleArgs],
        }
        self.call_mutation("newsIngestion:batchIngestArticles", &BatchArgs { articles })
            .await
    }

    /// Create an offense from research results.
    ///
    /// Calls `newsIngestion:createOffenseFromResearch`. Deduplicates by
    /// artist + category within 30 days.
    pub async fn create_offense_from_research(
        &self,
        args: &CreateOffenseArgs,
    ) -> Result<UpsertResponse> {
        self.call_mutation("newsIngestion:createOffenseFromResearch", args)
            .await
    }

    /// Link evidence to an existing offense.
    ///
    /// Calls `newsIngestion:linkOffenseEvidence`. Deduplicates by
    /// offense ID + source URL.
    pub async fn link_offense_evidence(&self, args: &LinkEvidenceArgs) -> Result<UpsertResponse> {
        self.call_mutation("newsIngestion:linkOffenseEvidence", args)
            .await
    }

    /// Update research quality score for an artist.
    ///
    /// Calls `newsIngestion:updateArtistResearchQuality`.
    pub async fn update_artist_research_quality(
        &self,
        args: &UpdateResearchQualityArgs,
    ) -> Result<UpdatedResponse> {
        self.call_mutation("newsIngestion:updateArtistResearchQuality", args)
            .await
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_is_clone_send_sync() {
        fn assert_clone_send_sync<T: Clone + Send + Sync>() {}
        assert_clone_send_sync::<ConvexClient>();
    }

    #[test]
    fn test_new_trims_trailing_slash() {
        let client = ConvexClient::new("https://example.convex.cloud/".to_string());
        assert_eq!(client.base_url, "https://example.convex.cloud");
    }

    #[test]
    fn test_new_no_trailing_slash() {
        let client = ConvexClient::new("https://example.convex.cloud".to_string());
        assert_eq!(client.base_url, "https://example.convex.cloud");
    }

    #[test]
    fn test_mutation_request_serialization() {
        let body = MutationRequest {
            path: "newsIngestion:ingestArticle",
            args: serde_json::json!({
                "legacyKey": "abc-123",
                "url": "https://example.com/article",
                "title": "Test Article"
            }),
        };
        let json = serde_json::to_value(&body).unwrap();
        assert_eq!(json["path"], "newsIngestion:ingestArticle");
        assert_eq!(json["args"]["legacyKey"], "abc-123");
    }

    #[test]
    fn test_success_response_deserialization() {
        let json = r#"{"status":"success","value":{"id":"abc123","upserted":"created"}}"#;
        let raw: ConvexRawResponse = serde_json::from_str(json).unwrap();
        match raw {
            ConvexRawResponse::Success { value } => {
                let resp: UpsertResponse = serde_json::from_value(value).unwrap();
                assert_eq!(resp.id, "abc123");
                assert_eq!(resp.upserted, "created");
            }
            ConvexRawResponse::Error { .. } => panic!("Expected success"),
        }
    }

    #[test]
    fn test_error_response_deserialization() {
        let json = r#"{"status":"error","errorMessage":"Article not found."}"#;
        let raw: ConvexRawResponse = serde_json::from_str(json).unwrap();
        match raw {
            ConvexRawResponse::Error { error_message } => {
                assert_eq!(error_message, "Article not found.");
            }
            ConvexRawResponse::Success { .. } => panic!("Expected error"),
        }
    }

    #[test]
    fn test_ingest_article_args_serialization() {
        let args = IngestArticleArgs {
            legacy_key: "key-1".to_string(),
            url: "https://example.com".to_string(),
            title: "Test".to_string(),
            content: Some("Body text".to_string()),
            summary: None,
            published_at: None,
            processing_status: Some("completed".to_string()),
            embedding_generated: None,
            source_id: None,
            raw_data: None,
            metadata: None,
        };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["legacyKey"], "key-1");
        assert_eq!(json["url"], "https://example.com");
        assert_eq!(json["content"], "Body text");
        assert_eq!(json["processingStatus"], "completed");
        // Optional None fields should be absent
        assert!(json.get("summary").is_none());
        assert!(json.get("embeddingGenerated").is_none());
    }

    #[test]
    fn test_batch_article_args_serialization() {
        let args = BatchArticleArgs {
            legacy_key: "batch-1".to_string(),
            url: "https://example.com".to_string(),
            title: "Batch".to_string(),
            content: None,
            summary: None,
            published_at: None,
            processing_status: None,
            embedding_generated: None,
            source_id: None,
            raw_data: None,
            metadata: None,
            entities: Some(vec![BatchEntityArgs {
                legacy_key: "ent-1".to_string(),
                entity_name: "Drake".to_string(),
                entity_type: "person".to_string(),
                artist_id: None,
                confidence: Some(0.95),
                metadata: None,
            }]),
            classifications: None,
        };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["legacyKey"], "batch-1");
        let entities = json["entities"].as_array().unwrap();
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0]["entityName"], "Drake");
        assert_eq!(entities[0]["confidence"], 0.95);
    }

    #[test]
    fn test_batch_ingest_response_deserialization() {
        let json = serde_json::json!({
            "articlesCreated": 3,
            "articlesUpdated": 1,
            "entitiesInserted": 5,
            "classificationsInserted": 2,
            "totalArticles": 4
        });
        let resp: BatchIngestResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.articles_created, 3);
        assert_eq!(resp.articles_updated, 1);
        assert_eq!(resp.entities_inserted, 5);
        assert_eq!(resp.classifications_inserted, 2);
        assert_eq!(resp.total_articles, 4);
    }

    #[test]
    fn test_create_offense_args_serialization() {
        let args = CreateOffenseArgs {
            artist_id: "artist_123".to_string(),
            category: "domestic_violence".to_string(),
            severity: "high".to_string(),
            title: "Arrested for assault".to_string(),
            description: Some("Details here".to_string()),
            confidence: 0.85,
            source_article_url: Some("https://example.com/article".to_string()),
        };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["artistId"], "artist_123");
        assert_eq!(json["category"], "domestic_violence");
        assert_eq!(json["confidence"], 0.85);
        assert_eq!(json["sourceArticleUrl"], "https://example.com/article");
    }

    #[test]
    fn test_link_evidence_args_serialization() {
        let args = LinkEvidenceArgs {
            offense_id: "offense_456".to_string(),
            source_url: "https://news.example.com".to_string(),
            title: Some("Breaking News".to_string()),
            excerpt: None,
            credibility_score: Some(0.9),
        };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["offenseId"], "offense_456");
        assert_eq!(json["sourceUrl"], "https://news.example.com");
        assert_eq!(json["credibilityScore"], 0.9);
        assert!(json.get("excerpt").is_none());
    }

    #[test]
    fn test_update_research_quality_args_serialization() {
        let args = UpdateResearchQualityArgs {
            artist_id: "artist_789".to_string(),
            quality_score: 72.5,
            sources_searched: vec![
                "wikipedia".to_string(),
                "brave_search".to_string(),
                "newsapi".to_string(),
            ],
            research_iterations: 1.0,
        };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["artistId"], "artist_789");
        assert_eq!(json["qualityScore"], 72.5);
        assert_eq!(json["researchIterations"], 1.0);
        let sources = json["sourcesSearched"].as_array().unwrap();
        assert_eq!(sources.len(), 3);
    }
}
