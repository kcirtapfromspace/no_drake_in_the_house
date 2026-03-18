//! News Ingestion Module
//!
//! Handles fetching news from various sources:
//! - RSS feeds from music news sites
//! - NewsAPI for aggregated news
//! - Twitter/X for social media mentions
//! - Reddit for community discussions
//! - Web scraping for additional coverage

pub mod newsapi_client;
pub mod reddit_monitor;
pub mod rss_fetcher;
pub mod twitter_monitor;
pub mod web_scraper;

pub use newsapi_client::{NewsApiClient, NewsApiConfig};
pub use reddit_monitor::{RedditConfig, RedditMonitor};
pub use rss_fetcher::{FetchedArticle, RssFetcher, RssFetcherConfig};
pub use twitter_monitor::{TwitterConfig, TwitterMonitor};
pub use web_scraper::{WebScraper, WebScraperConfig};
