//! Web Search Client
//!
//! Searches for artist controversies using Brave Search API.
//! Free tier: 2000 queries/month. Returns URLs that can be scraped.

use anyhow::{Context, Result};
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;
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

/// Web search client using Brave Search API
pub struct WebSearchClient {
    http: Client,
    config: WebSearchConfig,
}

#[derive(Debug, Deserialize)]
struct BraveSearchResponse {
    web: Option<BraveWebResults>,
}

#[derive(Debug, Deserialize)]
struct BraveWebResults {
    results: Vec<BraveWebResult>,
}

#[derive(Debug, Deserialize)]
struct BraveWebResult {
    url: String,
    title: String,
    description: Option<String>,
    #[allow(dead_code)]
    page_age: Option<String>,
}

impl WebSearchClient {
    /// Create from environment variable BRAVE_SEARCH_API_KEY
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("BRAVE_SEARCH_API_KEY")
            .context("BRAVE_SEARCH_API_KEY environment variable not set")?;

        Ok(Self::new(WebSearchConfig {
            api_key,
            ..Default::default()
        }))
    }

    pub fn new(config: WebSearchConfig) -> Self {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(15))
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
                    tracing::warn!(query = %query, error = %e, "Brave search failed");
                }
            }

            // Small delay between queries to be respectful
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }

        tracing::info!(
            artist = artist_name,
            results = all_articles.len(),
            "Web search complete"
        );

        Ok(all_articles)
    }

    /// Build multiple search queries to maximize coverage
    fn build_search_queries(artist_name: &str) -> Vec<String> {
        vec![
            format!(
                "\"{}\" controversy OR arrested OR accused OR charged",
                artist_name
            ),
            format!(
                "\"{}\" lawsuit OR convicted OR allegations OR scandal",
                artist_name
            ),
            format!(
                "\"{}\" domestic violence OR assault OR harassment",
                artist_name
            ),
        ]
    }

    /// Execute a single search query
    async fn search(&self, query: &str) -> Result<Vec<FetchedArticle>> {
        let api_url = "https://api.search.brave.com/res/v1/web/search";
        let count_str = self.config.max_results_per_query.to_string();

        let mut final_response = None;
        for attempt in 0..=3u32 {
            let response = self
                .http
                .get(api_url)
                .header("X-Subscription-Token", &self.config.api_key)
                .header("Accept", "application/json")
                .query(&[
                    ("q", query),
                    ("count", count_str.as_str()),
                    ("text_decorations", "false"),
                ])
                .send()
                .await
                .context("Brave Search API request failed")?;

            if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                if attempt == 3 {
                    return Err(anyhow::anyhow!(
                        "Brave Search rate limited after 4 attempts"
                    ));
                }
                let wait = response
                    .headers()
                    .get("retry-after")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(2u64.pow(attempt + 1))
                    .min(60);
                tracing::warn!(
                    "Rate limited on Brave Search, retry {}/3 after {}s",
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
            .ok_or_else(|| anyhow::anyhow!("Brave Search: no response after retries"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Brave Search API error {}: {}",
                status,
                &body[..body.len().min(300)]
            ));
        }

        let search_response: BraveSearchResponse = response
            .json()
            .await
            .context("Failed to parse Brave Search response")?;

        let results = search_response.web.map(|w| w.results).unwrap_or_default();

        let articles: Vec<FetchedArticle> = results
            .into_iter()
            .map(|r| FetchedArticle {
                id: Uuid::new_v4(),
                source_id: Uuid::nil(),
                url: r.url,
                title: r.title,
                content: r.description,
                published_at: None,
                fetched_at: Utc::now(),
                authors: Vec::new(),
                categories: vec!["web_search".to_string()],
                image_url: None,
            })
            .collect();

        Ok(articles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_search_queries() {
        let queries = WebSearchClient::build_search_queries("Drake");
        assert_eq!(queries.len(), 3);
        assert!(queries[0].contains("Drake"));
        assert!(queries[0].contains("controversy"));
    }

    #[test]
    fn test_default_config() {
        let config = WebSearchConfig::default();
        assert_eq!(config.max_results_per_query, 10);
        assert!(config.api_key.is_empty());
    }
}
