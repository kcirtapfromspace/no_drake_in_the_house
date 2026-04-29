//! Web Search Client
//!
//! Searches for artist controversies using Firecrawl Search API.
//! Replaces Brave Search with Firecrawl's /v2/search endpoint which
//! provides web + news results and optional page scraping.
//!
//! Free tier: 500 credits (2 credits per 10 results).
//! Falls back to BRAVE_SEARCH_API_KEY if FIRECRAWL_API_KEY is not set.

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::FetchedArticle;

/// Web search client configuration
#[derive(Debug, Clone)]
pub struct WebSearchConfig {
    pub api_key: String,
    pub max_results_per_query: usize,
}

impl Default for WebSearchConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            max_results_per_query: 10,
        }
    }
}

/// Web search client using Firecrawl Search API
pub struct WebSearchClient {
    http: Client,
    config: WebSearchConfig,
}

// ── Firecrawl API types ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct FirecrawlSearchRequest {
    query: String,
    limit: usize,
    sources: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tbs: Option<String>,
    #[serde(rename = "scrapeOptions", skip_serializing_if = "Option::is_none")]
    scrape_options: Option<FirecrawlScrapeOptions>,
}

#[derive(Debug, Serialize)]
struct FirecrawlScrapeOptions {
    formats: Vec<String>,
    #[serde(rename = "onlyMainContent")]
    only_main_content: bool,
}

#[derive(Debug, Deserialize)]
struct FirecrawlSearchResponse {
    success: bool,
    data: Option<FirecrawlSearchData>,
    warning: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FirecrawlSearchData {
    #[serde(default)]
    web: Vec<FirecrawlWebResult>,
    #[serde(default)]
    news: Vec<FirecrawlNewsResult>,
}

#[derive(Debug, Deserialize)]
struct FirecrawlWebResult {
    url: String,
    title: String,
    description: Option<String>,
    markdown: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FirecrawlNewsResult {
    url: String,
    title: String,
    snippet: Option<String>,
    date: Option<String>,
    markdown: Option<String>,
}

impl WebSearchClient {
    /// Create from environment variable FIRECRAWL_API_KEY (falls back to BRAVE_SEARCH_API_KEY)
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("FIRECRAWL_API_KEY")
            .or_else(|_| std::env::var("BRAVE_SEARCH_API_KEY"))
            .context(
                "Neither FIRECRAWL_API_KEY nor BRAVE_SEARCH_API_KEY environment variable is set",
            )?;

        Ok(Self::new(WebSearchConfig {
            api_key,
            ..Default::default()
        }))
    }

    pub fn new(config: WebSearchConfig) -> Self {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self { http, config }
    }

    /// Search for controversies about an artist
    pub async fn search_artist_controversies(
        &self,
        artist_name: &str,
    ) -> Result<Vec<FetchedArticle>> {
        let queries = Self::build_search_queries(artist_name);
        let mut all_articles = Vec::new();
        let mut seen_urls = std::collections::HashSet::new();

        for query in queries {
            match self.search(&query).await {
                Ok(results) => {
                    for result in results {
                        if seen_urls.insert(result.url.clone()) {
                            all_articles.push(result);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(query = %query, error = %e, "Firecrawl search failed");
                }
            }

            // Respect free-tier rate limit (5 req/min)
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        tracing::info!(
            artist = artist_name,
            results = all_articles.len(),
            "Web search complete"
        );

        Ok(all_articles)
    }

    /// Build search queries — broader than before to catch more controversy types
    fn build_search_queries(artist_name: &str) -> Vec<String> {
        vec![
            // Criminal / legal
            format!(
                "\"{}\" arrested OR charged OR convicted OR indicted OR lawsuit OR legal",
                artist_name
            ),
            // Misconduct / abuse
            format!(
                "\"{}\" allegations OR accused OR assault OR abuse OR harassment OR misconduct OR scandal",
                artist_name
            ),
            // Broader controversy signals
            format!(
                "\"{}\" controversy OR problematic OR canceled OR backlash OR apology OR investigation",
                artist_name
            ),
        ]
    }

    /// Execute a single search query via Firecrawl
    async fn search(&self, query: &str) -> Result<Vec<FetchedArticle>> {
        let api_url = "https://api.firecrawl.dev/v2/search";

        let request_body = FirecrawlSearchRequest {
            query: query.to_string(),
            limit: self.config.max_results_per_query,
            sources: vec!["web".to_string(), "news".to_string()],
            tbs: Some("qdr:y".to_string()), // Last year
            scrape_options: None,           // Don't scrape to save credits
        };

        let mut final_response = None;
        for attempt in 0..=3u32 {
            let response = self
                .http
                .post(api_url)
                .header("Authorization", format!("Bearer {}", &self.config.api_key))
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await
                .context("Firecrawl Search API request failed")?;

            if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                if attempt == 3 {
                    return Err(anyhow::anyhow!("Firecrawl rate limited after 4 attempts"));
                }
                let wait = response
                    .headers()
                    .get("retry-after")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(2u64.pow(attempt + 1))
                    .min(60);
                tracing::warn!(
                    "Rate limited on Firecrawl, retry {}/3 after {}s",
                    attempt + 1,
                    wait
                );
                tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
                continue;
            }

            final_response = Some(response);
            break;
        }

        let response = final_response
            .ok_or_else(|| anyhow::anyhow!("Firecrawl: no response after retries"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Firecrawl API error {}: {}",
                status,
                &body[..body.len().min(300)]
            ));
        }

        let search_response: FirecrawlSearchResponse = response
            .json()
            .await
            .context("Failed to parse Firecrawl response")?;

        if !search_response.success {
            return Err(anyhow::anyhow!(
                "Firecrawl search unsuccessful: {}",
                search_response.error.unwrap_or_default()
            ));
        }

        if let Some(warning) = &search_response.warning {
            tracing::warn!(warning = %warning, "Firecrawl search warning");
        }

        let data = search_response.data.unwrap_or(FirecrawlSearchData {
            web: Vec::new(),
            news: Vec::new(),
        });

        let mut articles: Vec<FetchedArticle> = Vec::new();

        // Map web results
        for r in data.web {
            articles.push(FetchedArticle {
                id: Uuid::new_v4(),
                source_id: Uuid::nil(),
                url: r.url,
                title: r.title,
                content: r.markdown.or(r.description),
                published_at: None,
                fetched_at: Utc::now(),
                authors: Vec::new(),
                categories: vec!["web_search".to_string()],
                image_url: None,
            });
        }

        // Map news results (these have dates and are higher signal)
        for r in data.news {
            let published_at = r.date.as_deref().and_then(parse_loose_date);
            articles.push(FetchedArticle {
                id: Uuid::new_v4(),
                source_id: Uuid::nil(),
                url: r.url,
                title: r.title,
                content: r.markdown.or(r.snippet),
                published_at,
                fetched_at: Utc::now(),
                authors: Vec::new(),
                categories: vec!["news_search".to_string()],
                image_url: None,
            });
        }

        Ok(articles)
    }
}

/// Best-effort parsing of date strings from news results
fn parse_loose_date(s: &str) -> Option<DateTime<Utc>> {
    // Try ISO 8601 first
    if let Ok(dt) = s.parse::<DateTime<Utc>>() {
        return Some(dt);
    }
    // Try date-only formats
    for fmt in &["%Y-%m-%d", "%b %d, %Y", "%B %d, %Y", "%m/%d/%Y"] {
        if let Ok(nd) =
            NaiveDateTime::parse_from_str(&format!("{} 00:00:00", s), &format!("{} %H:%M:%S", fmt))
        {
            return Some(nd.and_utc());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_search_queries() {
        let queries = WebSearchClient::build_search_queries("Drake");
        assert_eq!(queries.len(), 3);
        assert!(queries[0].contains("Drake"));
        assert!(queries[0].contains("arrested"));
        assert!(queries[2].contains("controversy"));
    }

    #[test]
    fn test_default_config() {
        let config = WebSearchConfig::default();
        assert_eq!(config.max_results_per_query, 10);
        assert!(config.api_key.is_empty());
    }

    #[test]
    fn test_parse_loose_date() {
        assert!(parse_loose_date("2024-05-07").is_some());
        assert!(parse_loose_date("May 7, 2024").is_some());
        assert!(parse_loose_date("garbage").is_none());
    }
}
