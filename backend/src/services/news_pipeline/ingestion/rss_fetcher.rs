//! RSS Feed Fetcher
//!
//! Fetches and parses RSS feeds from music news sources.
//! Supports Atom and RSS 2.0 formats via feed-rs crate.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use feed_rs::parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// RSS Fetcher configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RssFetcherConfig {
    /// User agent for HTTP requests
    pub user_agent: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum articles per feed
    pub max_articles_per_feed: usize,
    /// Minimum poll interval in minutes
    pub min_poll_interval_minutes: u32,
}

impl Default for RssFetcherConfig {
    fn default() -> Self {
        Self {
            user_agent: "NoDrakeNewsFetcher/1.0".to_string(),
            timeout_seconds: 30,
            max_articles_per_feed: 50,
            min_poll_interval_minutes: 15,
        }
    }
}

/// A news source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsSource {
    /// Unique identifier
    pub id: Uuid,
    /// Source name
    pub name: String,
    /// RSS feed URL
    pub url: String,
    /// Credibility score (1-5)
    pub credibility_score: u8,
    /// Poll interval in minutes
    pub poll_interval_minutes: u32,
    /// Whether the source is enabled
    pub enabled: bool,
    /// Source category
    pub category: SourceCategory,
}

/// Source categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SourceCategory {
    MusicNews,
    Entertainment,
    Tabloid,
    Industry,
    Blog,
}

/// A fetched article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchedArticle {
    /// Unique identifier
    pub id: Uuid,
    /// Source ID
    pub source_id: Uuid,
    /// Article URL
    pub url: String,
    /// Article title
    pub title: String,
    /// Article content/summary
    pub content: Option<String>,
    /// Publication date
    pub published_at: Option<DateTime<Utc>>,
    /// When the article was fetched
    pub fetched_at: DateTime<Utc>,
    /// Authors
    pub authors: Vec<String>,
    /// Categories/tags
    pub categories: Vec<String>,
    /// Article thumbnail/image
    pub image_url: Option<String>,
}

/// RSS Fetcher state
struct FetcherState {
    /// Last fetch time per source
    last_fetch: HashMap<Uuid, DateTime<Utc>>,
    /// Known article URLs to avoid duplicates
    seen_urls: HashMap<String, DateTime<Utc>>,
}

/// RSS Feed Fetcher
pub struct RssFetcher {
    config: RssFetcherConfig,
    client: Client,
    sources: Arc<RwLock<Vec<NewsSource>>>,
    state: Arc<RwLock<FetcherState>>,
}

impl RssFetcher {
    /// Create a new RSS fetcher
    pub fn new(config: RssFetcherConfig) -> Self {
        let client = Client::builder()
            .user_agent(&config.user_agent)
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            sources: Arc::new(RwLock::new(Self::default_sources())),
            state: Arc::new(RwLock::new(FetcherState {
                last_fetch: HashMap::new(),
                seen_urls: HashMap::new(),
            })),
        }
    }

    /// Default music news sources
    fn default_sources() -> Vec<NewsSource> {
        vec![
            NewsSource {
                id: Uuid::new_v4(),
                name: "Pitchfork".to_string(),
                url: "https://pitchfork.com/rss/news/".to_string(),
                credibility_score: 4,
                poll_interval_minutes: 30,
                enabled: true,
                category: SourceCategory::MusicNews,
            },
            NewsSource {
                id: Uuid::new_v4(),
                name: "Rolling Stone Music".to_string(),
                url: "https://www.rollingstone.com/music/feed/".to_string(),
                credibility_score: 4,
                poll_interval_minutes: 30,
                enabled: true,
                category: SourceCategory::MusicNews,
            },
            NewsSource {
                id: Uuid::new_v4(),
                name: "Billboard".to_string(),
                url: "https://www.billboard.com/feed/".to_string(),
                credibility_score: 4,
                poll_interval_minutes: 30,
                enabled: true,
                category: SourceCategory::Industry,
            },
            NewsSource {
                id: Uuid::new_v4(),
                name: "NME".to_string(),
                url: "https://www.nme.com/news/music/feed".to_string(),
                credibility_score: 3,
                poll_interval_minutes: 30,
                enabled: true,
                category: SourceCategory::MusicNews,
            },
            NewsSource {
                id: Uuid::new_v4(),
                name: "Consequence of Sound".to_string(),
                url: "https://consequence.net/feed/".to_string(),
                credibility_score: 3,
                poll_interval_minutes: 30,
                enabled: true,
                category: SourceCategory::MusicNews,
            },
            NewsSource {
                id: Uuid::new_v4(),
                name: "Stereogum".to_string(),
                url: "https://www.stereogum.com/feed/".to_string(),
                credibility_score: 3,
                poll_interval_minutes: 30,
                enabled: true,
                category: SourceCategory::MusicNews,
            },
            NewsSource {
                id: Uuid::new_v4(),
                name: "The Guardian Music".to_string(),
                url: "https://www.theguardian.com/music/rss".to_string(),
                credibility_score: 5,
                poll_interval_minutes: 60,
                enabled: true,
                category: SourceCategory::MusicNews,
            },
            NewsSource {
                id: Uuid::new_v4(),
                name: "Complex Music".to_string(),
                url: "https://www.complex.com/music/rss".to_string(),
                credibility_score: 3,
                poll_interval_minutes: 30,
                enabled: true,
                category: SourceCategory::Entertainment,
            },
            NewsSource {
                id: Uuid::new_v4(),
                name: "XXL Magazine".to_string(),
                url: "https://www.xxlmag.com/feed/".to_string(),
                credibility_score: 3,
                poll_interval_minutes: 30,
                enabled: true,
                category: SourceCategory::MusicNews,
            },
            NewsSource {
                id: Uuid::new_v4(),
                name: "HotNewHipHop".to_string(),
                url: "https://www.hotnewhiphop.com/rss/news.xml".to_string(),
                credibility_score: 2,
                poll_interval_minutes: 15,
                enabled: true,
                category: SourceCategory::Blog,
            },
        ]
    }

    /// Add a news source
    pub async fn add_source(&self, source: NewsSource) {
        let mut sources = self.sources.write().await;
        sources.push(source);
    }

    /// Remove a news source
    pub async fn remove_source(&self, source_id: Uuid) -> bool {
        let mut sources = self.sources.write().await;
        if let Some(pos) = sources.iter().position(|s| s.id == source_id) {
            sources.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get all sources
    pub async fn get_sources(&self) -> Vec<NewsSource> {
        self.sources.read().await.clone()
    }

    /// Fetch articles from a single source
    pub async fn fetch_source(&self, source: &NewsSource) -> Result<Vec<FetchedArticle>> {
        tracing::info!(source = %source.name, url = %source.url, "Fetching RSS feed");

        let response = self
            .client
            .get(&source.url)
            .send()
            .await
            .context(format!("Failed to fetch feed from {}", source.name))?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "HTTP error {} fetching {}",
                response.status(),
                source.name
            ));
        }

        let body = response
            .bytes()
            .await
            .context("Failed to read response body")?;

        let feed = parser::parse(&body[..])
            .context(format!("Failed to parse feed from {}", source.name))?;

        let mut articles = Vec::new();
        let mut state = self.state.write().await;

        for entry in feed.entries.iter().take(self.config.max_articles_per_feed) {
            // Get the article URL
            let url = entry
                .links
                .first()
                .map(|l| l.href.clone())
                .or_else(|| Some(entry.id.clone()))
                .unwrap_or_default();

            // Skip if we've seen this URL recently
            if state.seen_urls.contains_key(&url) {
                continue;
            }

            // Extract content
            let content = entry
                .content
                .as_ref()
                .and_then(|c| c.body.clone())
                .or_else(|| entry.summary.as_ref().map(|s| s.content.clone()));

            // Extract authors
            let authors: Vec<String> = entry.authors.iter().map(|a| a.name.clone()).collect();

            // Extract categories
            let categories: Vec<String> = entry.categories.iter().map(|c| c.term.clone()).collect();

            // Extract image
            let image_url = entry
                .media
                .first()
                .and_then(|m| m.content.first())
                .and_then(|c| c.url.as_ref().map(|u| u.to_string()));

            let article = FetchedArticle {
                id: Uuid::new_v4(),
                source_id: source.id,
                url: url.clone(),
                title: entry
                    .title
                    .as_ref()
                    .map(|t| t.content.clone())
                    .unwrap_or_default(),
                content,
                published_at: entry.published.or(entry.updated),
                fetched_at: Utc::now(),
                authors,
                categories,
                image_url,
            };

            // Mark URL as seen
            state.seen_urls.insert(url, Utc::now());
            articles.push(article);
        }

        // Update last fetch time
        state.last_fetch.insert(source.id, Utc::now());

        tracing::info!(
            source = %source.name,
            articles_count = articles.len(),
            "Fetched RSS articles"
        );

        Ok(articles)
    }

    /// Fetch articles from all enabled sources
    pub async fn fetch_all(&self) -> Result<Vec<FetchedArticle>> {
        let sources = self.sources.read().await.clone();
        let mut all_articles = Vec::new();

        for source in sources.iter().filter(|s| s.enabled) {
            // Check if we should poll this source
            if !self.should_poll(&source).await {
                continue;
            }

            match self.fetch_source(source).await {
                Ok(articles) => {
                    all_articles.extend(articles);
                }
                Err(e) => {
                    tracing::warn!(
                        source = %source.name,
                        error = %e,
                        "Failed to fetch RSS feed"
                    );
                }
            }
        }

        Ok(all_articles)
    }

    /// Check if we should poll a source based on interval
    async fn should_poll(&self, source: &NewsSource) -> bool {
        let state = self.state.read().await;

        if let Some(last_fetch) = state.last_fetch.get(&source.id) {
            let interval = chrono::Duration::minutes(source.poll_interval_minutes as i64);
            let next_fetch = *last_fetch + interval;
            Utc::now() >= next_fetch
        } else {
            true // Never fetched before
        }
    }

    /// Clean up old seen URLs (older than 7 days)
    pub async fn cleanup_seen_urls(&self) {
        let cutoff = Utc::now() - chrono::Duration::days(7);
        let mut state = self.state.write().await;
        state.seen_urls.retain(|_, seen_at| *seen_at > cutoff);
    }

    /// Get fetch statistics
    pub async fn get_stats(&self) -> FetcherStats {
        let state = self.state.read().await;
        let sources = self.sources.read().await;

        FetcherStats {
            total_sources: sources.len(),
            enabled_sources: sources.iter().filter(|s| s.enabled).count(),
            seen_urls_count: state.seen_urls.len(),
            last_fetch_times: state.last_fetch.clone(),
        }
    }
}

/// Fetcher statistics
#[derive(Debug, Clone, Serialize)]
pub struct FetcherStats {
    pub total_sources: usize,
    pub enabled_sources: usize,
    pub seen_urls_count: usize,
    pub last_fetch_times: HashMap<Uuid, DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RssFetcherConfig::default();
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_articles_per_feed, 50);
    }

    #[test]
    fn test_default_sources() {
        let sources = RssFetcher::default_sources();
        assert!(!sources.is_empty());
        assert!(sources.iter().any(|s| s.name == "Pitchfork"));
    }

    #[tokio::test]
    async fn test_fetcher_creation() {
        let config = RssFetcherConfig::default();
        let fetcher = RssFetcher::new(config);

        let sources = fetcher.get_sources().await;
        assert!(!sources.is_empty());
    }
}
