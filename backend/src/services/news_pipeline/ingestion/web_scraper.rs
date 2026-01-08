//! Web Scraper
//!
//! Scrapes web pages for article content when RSS feeds don't provide full text.
//! Uses the scraper crate for HTML parsing.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::{Client, Url};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

use super::rss_fetcher::FetchedArticle;

/// Web scraper configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebScraperConfig {
    /// User agent for HTTP requests
    pub user_agent: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Delay between requests to same domain (milliseconds)
    pub crawl_delay_ms: u64,
    /// Maximum content length to extract (characters)
    pub max_content_length: usize,
    /// CSS selectors for different sites
    pub site_selectors: HashMap<String, SiteSelectors>,
}

/// CSS selectors for a specific site
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteSelectors {
    /// Selector for article title
    pub title: String,
    /// Selector for article content
    pub content: String,
    /// Selector for author (optional)
    pub author: Option<String>,
    /// Selector for publish date (optional)
    pub date: Option<String>,
    /// Selector for article image (optional)
    pub image: Option<String>,
}

impl Default for WebScraperConfig {
    fn default() -> Self {
        let mut site_selectors = HashMap::new();

        // Default selectors for common sites
        site_selectors.insert(
            "pitchfork.com".to_string(),
            SiteSelectors {
                title: "h1.SplitScreenContentHeaderHed-lcUSuI".to_string(),
                content: "div.body__inner-container".to_string(),
                author: Some("a.byline__name-link".to_string()),
                date: Some("time[datetime]".to_string()),
                image: Some("img.ResponsiveImageContainer-eybHBd".to_string()),
            },
        );

        site_selectors.insert(
            "rollingstone.com".to_string(),
            SiteSelectors {
                title: "h1.c-title".to_string(),
                content: "div.c-content".to_string(),
                author: Some("a.c-byline__link".to_string()),
                date: Some("time.c-timestamp".to_string()),
                image: Some("img.c-figure__image".to_string()),
            },
        );

        site_selectors.insert(
            "billboard.com".to_string(),
            SiteSelectors {
                title: "h1.c-title".to_string(),
                content: "div.a-content".to_string(),
                author: Some("span.c-byline".to_string()),
                date: Some("time".to_string()),
                image: Some("img.a-image".to_string()),
            },
        );

        // Generic fallback selectors
        site_selectors.insert(
            "default".to_string(),
            SiteSelectors {
                title: "h1, article h1, .article-title, .post-title".to_string(),
                content: "article, .article-content, .post-content, .entry-content, main"
                    .to_string(),
                author: Some(".author, .byline, [rel='author']".to_string()),
                date: Some("time[datetime], .date, .published".to_string()),
                image: Some(
                    "article img, .featured-image img, meta[property='og:image']".to_string(),
                ),
            },
        );

        Self {
            user_agent: "NoDrakeNewsScraper/1.0 (music blocklist app)".to_string(),
            timeout_seconds: 30,
            crawl_delay_ms: 1000,
            max_content_length: 50000,
            site_selectors,
        }
    }
}

/// Web scraper
pub struct WebScraper {
    config: WebScraperConfig,
    client: Client,
    last_request: std::sync::Arc<tokio::sync::RwLock<HashMap<String, std::time::Instant>>>,
}

/// Scraped article content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedContent {
    pub url: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub author: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub image_url: Option<String>,
    pub scraped_at: DateTime<Utc>,
}

impl WebScraper {
    /// Create a new web scraper
    pub fn new(config: WebScraperConfig) -> Self {
        let client = Client::builder()
            .user_agent(&config.user_agent)
            .timeout(Duration::from_secs(config.timeout_seconds))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            last_request: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Extract domain from URL
    fn extract_domain(url: &str) -> Option<String> {
        Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|s| s.to_string()))
    }

    /// Get selectors for a domain
    fn get_selectors(&self, domain: &str) -> &SiteSelectors {
        self.config
            .site_selectors
            .get(domain)
            .or_else(|| self.config.site_selectors.get("default"))
            .expect("Default selectors should always exist")
    }

    /// Respect crawl delay
    async fn respect_crawl_delay(&self, domain: &str) {
        let mut last_requests = self.last_request.write().await;

        if let Some(last) = last_requests.get(domain) {
            let elapsed = last.elapsed();
            let delay = Duration::from_millis(self.config.crawl_delay_ms);
            if elapsed < delay {
                tokio::time::sleep(delay - elapsed).await;
            }
        }

        last_requests.insert(domain.to_string(), std::time::Instant::now());
    }

    /// Scrape a URL
    pub async fn scrape(&self, url: &str) -> Result<ScrapedContent> {
        let domain =
            Self::extract_domain(url).ok_or_else(|| anyhow::anyhow!("Invalid URL: {}", url))?;

        self.respect_crawl_delay(&domain).await;

        tracing::debug!(url = %url, "Scraping web page");

        let response = self
            .client
            .get(url)
            .send()
            .await
            .context(format!("Failed to fetch {}", url))?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "HTTP error {} for {}",
                response.status(),
                url
            ));
        }

        let html = response.text().await.context("Failed to read response")?;
        let document = Html::parse_document(&html);

        let selectors = self.get_selectors(&domain);

        // Extract title
        let title = self.extract_text(&document, &selectors.title);

        // Extract content
        let content = self
            .extract_text(&document, &selectors.content)
            .map(|c| self.clean_content(&c));

        // Extract author
        let author = selectors
            .author
            .as_ref()
            .and_then(|s| self.extract_text(&document, s));

        // Extract date
        let published_at = selectors
            .date
            .as_ref()
            .and_then(|s| self.extract_date(&document, s));

        // Extract image
        let image_url = selectors
            .image
            .as_ref()
            .and_then(|s| self.extract_image(&document, s, url));

        Ok(ScrapedContent {
            url: url.to_string(),
            title,
            content,
            author,
            published_at,
            image_url,
            scraped_at: Utc::now(),
        })
    }

    /// Extract text from document using selector
    fn extract_text(&self, document: &Html, selector_str: &str) -> Option<String> {
        let selector = Selector::parse(selector_str).ok()?;
        let element = document.select(&selector).next()?;

        let text: String = element
            .text()
            .collect::<Vec<_>>()
            .join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    }

    /// Extract date from document
    fn extract_date(&self, document: &Html, selector_str: &str) -> Option<DateTime<Utc>> {
        let selector = Selector::parse(selector_str).ok()?;
        let element = document.select(&selector).next()?;

        // Try datetime attribute first
        if let Some(datetime) = element.value().attr("datetime") {
            if let Ok(dt) = DateTime::parse_from_rfc3339(datetime) {
                return Some(dt.with_timezone(&Utc));
            }
            // Try other common formats
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(datetime, "%Y-%m-%dT%H:%M:%S") {
                return Some(DateTime::from_naive_utc_and_offset(dt, Utc));
            }
        }

        // Try content attribute (for meta tags)
        if let Some(content) = element.value().attr("content") {
            if let Ok(dt) = DateTime::parse_from_rfc3339(content) {
                return Some(dt.with_timezone(&Utc));
            }
        }

        None
    }

    /// Extract image URL from document
    fn extract_image(&self, document: &Html, selector_str: &str, base_url: &str) -> Option<String> {
        let selector = Selector::parse(selector_str).ok()?;
        let element = document.select(&selector).next()?;

        // Try src attribute
        if let Some(src) = element.value().attr("src") {
            return Some(self.resolve_url(src, base_url));
        }

        // Try content attribute (for meta tags)
        if let Some(content) = element.value().attr("content") {
            return Some(self.resolve_url(content, base_url));
        }

        // Try data-src (lazy loading)
        if let Some(data_src) = element.value().attr("data-src") {
            return Some(self.resolve_url(data_src, base_url));
        }

        None
    }

    /// Resolve relative URL to absolute
    fn resolve_url(&self, url: &str, base: &str) -> String {
        if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else if url.starts_with("//") {
            format!("https:{}", url)
        } else if let Ok(base_url) = Url::parse(base) {
            base_url
                .join(url)
                .map(|u| u.to_string())
                .unwrap_or_else(|_| url.to_string())
        } else {
            url.to_string()
        }
    }

    /// Clean extracted content
    fn clean_content(&self, content: &str) -> String {
        let cleaned = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        if cleaned.len() > self.config.max_content_length {
            cleaned[..self.config.max_content_length].to_string()
        } else {
            cleaned
        }
    }

    /// Enrich a FetchedArticle with scraped content
    pub async fn enrich_article(&self, article: &FetchedArticle) -> Result<FetchedArticle> {
        let scraped = self.scrape(&article.url).await?;

        Ok(FetchedArticle {
            id: article.id,
            source_id: article.source_id,
            url: article.url.clone(),
            title: scraped.title.unwrap_or_else(|| article.title.clone()),
            content: scraped.content.or_else(|| article.content.clone()),
            published_at: scraped.published_at.or(article.published_at),
            fetched_at: article.fetched_at,
            authors: scraped
                .author
                .map(|a| vec![a])
                .unwrap_or_else(|| article.authors.clone()),
            categories: article.categories.clone(),
            image_url: scraped.image_url.or_else(|| article.image_url.clone()),
        })
    }

    /// Enrich multiple articles (with rate limiting)
    pub async fn enrich_articles(&self, articles: Vec<FetchedArticle>) -> Vec<FetchedArticle> {
        let mut enriched = Vec::with_capacity(articles.len());

        for article in articles {
            match self.enrich_article(&article).await {
                Ok(enriched_article) => {
                    enriched.push(enriched_article);
                }
                Err(e) => {
                    tracing::warn!(
                        url = %article.url,
                        error = %e,
                        "Failed to scrape article, using original"
                    );
                    enriched.push(article);
                }
            }
        }

        enriched
    }

    /// Add custom selectors for a domain
    pub fn add_site_selectors(&mut self, domain: String, selectors: SiteSelectors) {
        self.config.site_selectors.insert(domain, selectors);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = WebScraperConfig::default();
        assert!(config.site_selectors.contains_key("default"));
        assert!(config.site_selectors.contains_key("pitchfork.com"));
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            WebScraper::extract_domain("https://pitchfork.com/news/test"),
            Some("pitchfork.com".to_string())
        );
        assert_eq!(
            WebScraper::extract_domain("https://www.billboard.com/article"),
            Some("www.billboard.com".to_string())
        );
    }

    #[test]
    fn test_scraper_creation() {
        let config = WebScraperConfig::default();
        let _scraper = WebScraper::new(config);
    }
}
