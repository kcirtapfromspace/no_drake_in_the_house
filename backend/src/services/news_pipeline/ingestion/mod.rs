//! News Ingestion Module
//!
//! Handles fetching news from various sources:
//! - RSS feeds from music news sites
//! - NewsAPI for aggregated news
//! - Twitter/X for social media mentions
//! - Reddit for community discussions
//! - Web scraping for additional coverage

pub mod rss_fetcher;
pub mod newsapi_client;
pub mod twitter_monitor;
pub mod reddit_monitor;
pub mod web_scraper;

pub use rss_fetcher::{RssFetcher, RssFetcherConfig, FetchedArticle};
pub use newsapi_client::{NewsApiClient, NewsApiConfig};
pub use twitter_monitor::{TwitterMonitor, TwitterConfig};
pub use reddit_monitor::{RedditMonitor, RedditConfig};
pub use web_scraper::{WebScraper, WebScraperConfig};
