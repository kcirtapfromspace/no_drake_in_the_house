//! NewsAPI Client
//!
//! Client for the NewsAPI.org service to fetch aggregated news articles.
//! Free tier: 100 requests/day, 1 month old articles
//! Paid tier: Higher limits, real-time articles

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::rss_fetcher::FetchedArticle;

/// NewsAPI base URL
const NEWSAPI_BASE: &str = "https://newsapi.org/v2";

/// NewsAPI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsApiConfig {
    /// API key
    pub api_key: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum articles per request
    pub page_size: u32,
    /// Daily request limit (for quota tracking)
    pub daily_limit: u32,
}

impl Default for NewsApiConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            timeout_seconds: 30,
            page_size: 100,
            daily_limit: 100, // Free tier limit
        }
    }
}

/// Quota tracking state
struct QuotaState {
    requests_today: u32,
    day_start: chrono::NaiveDate,
}

/// NewsAPI client
pub struct NewsApiClient {
    config: NewsApiConfig,
    client: Client,
    quota: Arc<RwLock<QuotaState>>,
}

/// NewsAPI article response
#[derive(Debug, Deserialize)]
struct NewsApiResponse {
    status: String,
    #[serde(rename = "totalResults")]
    total_results: Option<u32>,
    articles: Option<Vec<NewsApiArticle>>,
    code: Option<String>,
    message: Option<String>,
}

/// NewsAPI article structure
#[derive(Debug, Deserialize)]
struct NewsApiArticle {
    source: NewsApiSource,
    author: Option<String>,
    title: String,
    description: Option<String>,
    url: String,
    #[serde(rename = "urlToImage")]
    url_to_image: Option<String>,
    #[serde(rename = "publishedAt")]
    published_at: Option<String>,
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NewsApiSource {
    id: Option<String>,
    name: String,
}

/// Search parameters for NewsAPI
#[derive(Debug, Clone, Serialize)]
pub struct NewsApiSearchParams {
    /// Search query (keywords or phrases)
    pub query: String,
    /// Search in title, description, or content
    pub search_in: Option<String>,
    /// Comma-separated source IDs
    pub sources: Option<String>,
    /// 2-letter country code
    pub country: Option<String>,
    /// Category (entertainment, general, etc.)
    pub category: Option<String>,
    /// Language code
    pub language: Option<String>,
    /// Sort by (relevancy, popularity, publishedAt)
    pub sort_by: Option<String>,
    /// Oldest article date
    pub from: Option<DateTime<Utc>>,
    /// Newest article date
    pub to: Option<DateTime<Utc>>,
    /// Page number
    pub page: Option<u32>,
}

impl Default for NewsApiSearchParams {
    fn default() -> Self {
        Self {
            query: String::new(),
            search_in: None,
            sources: None,
            country: None,
            category: None,
            language: Some("en".to_string()),
            sort_by: Some("publishedAt".to_string()),
            from: None,
            to: None,
            page: Some(1),
        }
    }
}

impl NewsApiClient {
    /// Create a new NewsAPI client
    pub fn new(config: NewsApiConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            quota: Arc::new(RwLock::new(QuotaState {
                requests_today: 0,
                day_start: Utc::now().date_naive(),
            })),
        }
    }

    /// Check and update quota
    async fn check_quota(&self) -> Result<()> {
        let mut quota = self.quota.write().await;

        // Reset if new day
        let today = Utc::now().date_naive();
        if quota.day_start != today {
            quota.requests_today = 0;
            quota.day_start = today;
        }

        // Check if we have quota remaining
        if quota.requests_today >= self.config.daily_limit {
            return Err(anyhow::anyhow!(
                "NewsAPI daily quota exceeded ({}/{})",
                quota.requests_today,
                self.config.daily_limit
            ));
        }

        quota.requests_today += 1;
        Ok(())
    }

    /// Get remaining quota
    pub async fn remaining_quota(&self) -> u32 {
        let quota = self.quota.read().await;
        let today = Utc::now().date_naive();

        if quota.day_start != today {
            self.config.daily_limit
        } else {
            self.config.daily_limit.saturating_sub(quota.requests_today)
        }
    }

    /// Search for news articles
    pub async fn search(&self, params: NewsApiSearchParams) -> Result<Vec<FetchedArticle>> {
        if self.config.api_key.is_empty() {
            return Err(anyhow::anyhow!("NewsAPI key not configured"));
        }

        self.check_quota().await?;

        let mut url = format!("{}/everything", NEWSAPI_BASE);
        let mut query_params = vec![
            ("q", params.query.clone()),
            ("pageSize", self.config.page_size.to_string()),
            ("apiKey", self.config.api_key.clone()),
        ];

        if let Some(search_in) = &params.search_in {
            query_params.push(("searchIn", search_in.clone()));
        }
        if let Some(sources) = &params.sources {
            query_params.push(("sources", sources.clone()));
        }
        if let Some(language) = &params.language {
            query_params.push(("language", language.clone()));
        }
        if let Some(sort_by) = &params.sort_by {
            query_params.push(("sortBy", sort_by.clone()));
        }
        if let Some(from) = params.from {
            query_params.push(("from", from.format("%Y-%m-%dT%H:%M:%S").to_string()));
        }
        if let Some(to) = params.to {
            query_params.push(("to", to.format("%Y-%m-%dT%H:%M:%S").to_string()));
        }
        if let Some(page) = params.page {
            query_params.push(("page", page.to_string()));
        }

        let response = self
            .client
            .get(&url)
            .query(&query_params)
            .send()
            .await
            .context("NewsAPI request failed")?;

        let api_response: NewsApiResponse = response
            .json()
            .await
            .context("Failed to parse NewsAPI response")?;

        if api_response.status != "ok" {
            return Err(anyhow::anyhow!(
                "NewsAPI error: {} - {}",
                api_response.code.unwrap_or_default(),
                api_response.message.unwrap_or_default()
            ));
        }

        let articles = api_response
            .articles
            .unwrap_or_default()
            .into_iter()
            .map(|a| self.convert_article(a))
            .collect();

        Ok(articles)
    }

    /// Search for top headlines
    pub async fn top_headlines(
        &self,
        category: Option<&str>,
        country: Option<&str>,
    ) -> Result<Vec<FetchedArticle>> {
        if self.config.api_key.is_empty() {
            return Err(anyhow::anyhow!("NewsAPI key not configured"));
        }

        self.check_quota().await?;

        let mut query_params = vec![
            ("apiKey", self.config.api_key.clone()),
            ("pageSize", self.config.page_size.to_string()),
        ];

        if let Some(cat) = category {
            query_params.push(("category", cat.to_string()));
        }
        if let Some(country_code) = country {
            query_params.push(("country", country_code.to_string()));
        } else {
            query_params.push(("country", "us".to_string()));
        }

        let response = self
            .client
            .get(&format!("{}/top-headlines", NEWSAPI_BASE))
            .query(&query_params)
            .send()
            .await
            .context("NewsAPI request failed")?;

        let api_response: NewsApiResponse = response
            .json()
            .await
            .context("Failed to parse NewsAPI response")?;

        if api_response.status != "ok" {
            return Err(anyhow::anyhow!(
                "NewsAPI error: {} - {}",
                api_response.code.unwrap_or_default(),
                api_response.message.unwrap_or_default()
            ));
        }

        let articles = api_response
            .articles
            .unwrap_or_default()
            .into_iter()
            .map(|a| self.convert_article(a))
            .collect();

        Ok(articles)
    }

    /// Search for music-related news
    pub async fn search_music_news(&self, keywords: Option<&[&str]>) -> Result<Vec<FetchedArticle>> {
        let default_keywords = ["music artist", "musician", "singer", "rapper", "band"];
        let search_terms = keywords.unwrap_or(&default_keywords);

        let query = search_terms.join(" OR ");

        let params = NewsApiSearchParams {
            query,
            category: Some("entertainment".to_string()),
            sort_by: Some("publishedAt".to_string()),
            language: Some("en".to_string()),
            from: Some(Utc::now() - chrono::Duration::days(7)),
            ..Default::default()
        };

        self.search(params).await
    }

    /// Search for artist-specific news
    pub async fn search_artist_news(&self, artist_name: &str) -> Result<Vec<FetchedArticle>> {
        let params = NewsApiSearchParams {
            query: format!("\"{}\"", artist_name),
            sort_by: Some("publishedAt".to_string()),
            language: Some("en".to_string()),
            from: Some(Utc::now() - chrono::Duration::days(30)),
            ..Default::default()
        };

        self.search(params).await
    }

    /// Convert NewsAPI article to our format
    fn convert_article(&self, article: NewsApiArticle) -> FetchedArticle {
        let published_at = article
            .published_at
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        FetchedArticle {
            id: Uuid::new_v4(),
            source_id: Uuid::nil(), // NewsAPI sources don't have UUIDs
            url: article.url,
            title: article.title,
            content: article.content.or(article.description),
            published_at,
            fetched_at: Utc::now(),
            authors: article.author.map(|a| vec![a]).unwrap_or_default(),
            categories: vec![article.source.name],
            image_url: article.url_to_image,
        }
    }

    /// Check if the client is configured
    pub fn is_configured(&self) -> bool {
        !self.config.api_key.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NewsApiConfig::default();
        assert_eq!(config.daily_limit, 100);
        assert!(config.api_key.is_empty());
    }

    #[test]
    fn test_search_params_default() {
        let params = NewsApiSearchParams::default();
        assert_eq!(params.language, Some("en".to_string()));
        assert_eq!(params.sort_by, Some("publishedAt".to_string()));
    }

    #[tokio::test]
    async fn test_client_without_key() {
        let config = NewsApiConfig::default();
        let client = NewsApiClient::new(config);

        assert!(!client.is_configured());

        let result = client.search(NewsApiSearchParams::default()).await;
        assert!(result.is_err());
    }
}
