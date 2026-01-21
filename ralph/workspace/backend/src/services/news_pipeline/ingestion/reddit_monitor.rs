//! Reddit Monitor
//!
//! Monitors Reddit for artist mentions and news discussions.
//! Uses Reddit's JSON API (no authentication required for read-only).
//! Rate limit: 60 requests/minute

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::rss_fetcher::FetchedArticle;

/// Reddit API base URL (JSON endpoints)
const REDDIT_BASE: &str = "https://www.reddit.com";

/// Reddit monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditConfig {
    /// User agent (required by Reddit)
    pub user_agent: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum posts per subreddit
    pub max_posts: u32,
    /// Subreddits to monitor
    pub subreddits: Vec<String>,
    /// Minimum upvotes for posts to consider
    pub min_upvotes: i32,
}

impl Default for RedditConfig {
    fn default() -> Self {
        Self {
            user_agent: "NoDrakeNewsBot/1.0 (music blocklist app)".to_string(),
            timeout_seconds: 30,
            max_posts: 25,
            subreddits: vec![
                "hiphopheads".to_string(),
                "Music".to_string(),
                "popheads".to_string(),
                "indieheads".to_string(),
                "rnb".to_string(),
                "rap".to_string(),
                "entertainment".to_string(),
                "Celebs".to_string(),
            ],
            min_upvotes: 100,
        }
    }
}

/// Rate limiter state
struct RateLimiterState {
    requests_in_window: u32,
    window_start: std::time::Instant,
}

/// Reddit monitor
pub struct RedditMonitor {
    config: RedditConfig,
    client: Client,
    rate_limiter: Arc<RwLock<RateLimiterState>>,
    seen_post_ids: Arc<RwLock<HashSet<String>>>,
}

/// Reddit API listing response
#[derive(Debug, Deserialize)]
struct RedditListing {
    data: RedditListingData,
}

#[derive(Debug, Deserialize)]
struct RedditListingData {
    children: Vec<RedditChild>,
    after: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RedditChild {
    data: RedditPost,
}

#[derive(Debug, Deserialize)]
struct RedditPost {
    id: String,
    title: String,
    selftext: Option<String>,
    author: String,
    subreddit: String,
    permalink: String,
    url: String,
    created_utc: f64,
    score: i32,
    num_comments: i32,
    upvote_ratio: Option<f64>,
    is_self: bool,
    link_flair_text: Option<String>,
    over_18: bool,
}

/// A processed Reddit post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedRedditPost {
    pub id: Uuid,
    pub post_id: String,
    pub title: String,
    pub content: Option<String>,
    pub author: String,
    pub subreddit: String,
    pub url: String,
    pub permalink: String,
    pub created_at: DateTime<Utc>,
    pub score: i32,
    pub num_comments: i32,
    pub upvote_ratio: f64,
    pub flair: Option<String>,
    pub is_external_link: bool,
}

impl RedditMonitor {
    /// Create a new Reddit monitor
    pub fn new(config: RedditConfig) -> Self {
        let client = Client::builder()
            .user_agent(&config.user_agent)
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            rate_limiter: Arc::new(RwLock::new(RateLimiterState {
                requests_in_window: 0,
                window_start: std::time::Instant::now(),
            })),
            seen_post_ids: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Check and update rate limit (60 requests per minute)
    async fn check_rate_limit(&self) -> Result<()> {
        let mut state = self.rate_limiter.write().await;

        // Reset if window expired (60 seconds)
        if state.window_start.elapsed().as_secs() >= 60 {
            state.requests_in_window = 0;
            state.window_start = std::time::Instant::now();
        }

        if state.requests_in_window >= 60 {
            let wait_time = 60 - state.window_start.elapsed().as_secs();
            return Err(anyhow::anyhow!(
                "Reddit rate limit exceeded, try again in {} seconds",
                wait_time
            ));
        }

        state.requests_in_window += 1;
        Ok(())
    }

    /// Fetch hot posts from a subreddit
    pub async fn fetch_subreddit(&self, subreddit: &str) -> Result<Vec<ProcessedRedditPost>> {
        self.check_rate_limit().await?;

        let url = format!(
            "{}/r/{}/hot.json?limit={}",
            REDDIT_BASE, subreddit, self.config.max_posts
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context(format!("Failed to fetch r/{}", subreddit))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(anyhow::anyhow!(
                "Reddit API error: {} for r/{}",
                status,
                subreddit
            ));
        }

        let listing: RedditListing = response
            .json()
            .await
            .context("Failed to parse Reddit response")?;

        let mut posts = Vec::new();
        let mut seen = self.seen_post_ids.write().await;

        for child in listing.data.children {
            let post = child.data;

            // Skip if we've seen this post
            if seen.contains(&post.id) {
                continue;
            }

            // Skip NSFW posts
            if post.over_18 {
                continue;
            }

            // Skip low-score posts
            if post.score < self.config.min_upvotes {
                continue;
            }

            seen.insert(post.id.clone());

            let created_at =
                DateTime::from_timestamp(post.created_utc as i64, 0).unwrap_or_else(|| Utc::now());

            posts.push(ProcessedRedditPost {
                id: Uuid::new_v4(),
                post_id: post.id,
                title: post.title,
                content: post.selftext.filter(|s| !s.is_empty()),
                author: post.author,
                subreddit: post.subreddit,
                url: post.url,
                permalink: format!("{}{}", REDDIT_BASE, post.permalink),
                created_at,
                score: post.score,
                num_comments: post.num_comments,
                upvote_ratio: post.upvote_ratio.unwrap_or(0.0),
                flair: post.link_flair_text,
                is_external_link: !post.is_self,
            });
        }

        Ok(posts)
    }

    /// Search Reddit for a query
    pub async fn search(
        &self,
        query: &str,
        subreddit: Option<&str>,
    ) -> Result<Vec<ProcessedRedditPost>> {
        self.check_rate_limit().await?;

        let url = if let Some(sub) = subreddit {
            format!(
                "{}/r/{}/search.json?q={}&restrict_sr=1&limit={}&sort=new",
                REDDIT_BASE,
                sub,
                urlencoding::encode(query),
                self.config.max_posts
            )
        } else {
            format!(
                "{}/search.json?q={}&limit={}&sort=new",
                REDDIT_BASE,
                urlencoding::encode(query),
                self.config.max_posts
            )
        };

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Reddit search failed")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Reddit search error: {}",
                response.status()
            ));
        }

        let listing: RedditListing = response
            .json()
            .await
            .context("Failed to parse Reddit search response")?;

        let mut posts = Vec::new();
        let mut seen = self.seen_post_ids.write().await;

        for child in listing.data.children {
            let post = child.data;

            if seen.contains(&post.id) || post.over_18 {
                continue;
            }

            seen.insert(post.id.clone());

            let created_at =
                DateTime::from_timestamp(post.created_utc as i64, 0).unwrap_or_else(|| Utc::now());

            posts.push(ProcessedRedditPost {
                id: Uuid::new_v4(),
                post_id: post.id,
                title: post.title,
                content: post.selftext.filter(|s| !s.is_empty()),
                author: post.author,
                subreddit: post.subreddit,
                url: post.url,
                permalink: format!("{}{}", REDDIT_BASE, post.permalink),
                created_at,
                score: post.score,
                num_comments: post.num_comments,
                upvote_ratio: post.upvote_ratio.unwrap_or(0.0),
                flair: post.link_flair_text,
                is_external_link: !post.is_self,
            });
        }

        Ok(posts)
    }

    /// Fetch posts from all configured subreddits
    pub async fn fetch_all_subreddits(&self) -> Result<Vec<ProcessedRedditPost>> {
        let mut all_posts = Vec::new();

        for subreddit in &self.config.subreddits.clone() {
            match self.fetch_subreddit(subreddit).await {
                Ok(posts) => {
                    tracing::info!(
                        subreddit = %subreddit,
                        posts_count = posts.len(),
                        "Fetched Reddit posts"
                    );
                    all_posts.extend(posts);
                }
                Err(e) => {
                    tracing::warn!(
                        subreddit = %subreddit,
                        error = %e,
                        "Failed to fetch subreddit"
                    );
                }
            }
        }

        Ok(all_posts)
    }

    /// Search for artist-related posts
    pub async fn search_artist(&self, artist_name: &str) -> Result<Vec<ProcessedRedditPost>> {
        // Search in music-related subreddits
        let music_subs = ["hiphopheads", "Music", "popheads"];
        let mut all_posts = Vec::new();

        for sub in music_subs {
            match self.search(artist_name, Some(sub)).await {
                Ok(posts) => all_posts.extend(posts),
                Err(e) => {
                    tracing::warn!(
                        subreddit = %sub,
                        artist = %artist_name,
                        error = %e,
                        "Artist search failed"
                    );
                }
            }
        }

        Ok(all_posts)
    }

    /// Convert Reddit posts to FetchedArticle format
    pub fn posts_to_articles(&self, posts: Vec<ProcessedRedditPost>) -> Vec<FetchedArticle> {
        posts
            .into_iter()
            .map(|post| FetchedArticle {
                id: post.id,
                source_id: Uuid::nil(),
                url: post.permalink,
                title: post.title,
                content: post.content,
                published_at: Some(post.created_at),
                fetched_at: Utc::now(),
                authors: vec![format!("u/{}", post.author)],
                categories: vec![format!("r/{}", post.subreddit)],
                image_url: None,
            })
            .collect()
    }

    /// Clean up old seen post IDs
    pub async fn cleanup_seen_posts(&self, max_size: usize) {
        let mut seen = self.seen_post_ids.write().await;
        if seen.len() > max_size {
            let to_remove = seen.len() - max_size;
            let ids_to_remove: Vec<_> = seen.iter().take(to_remove).cloned().collect();
            for id in ids_to_remove {
                seen.remove(&id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RedditConfig::default();
        assert!(!config.subreddits.is_empty());
        assert!(config.subreddits.contains(&"hiphopheads".to_string()));
    }

    #[tokio::test]
    async fn test_monitor_creation() {
        let config = RedditConfig::default();
        let monitor = RedditMonitor::new(config);
        assert!(monitor.seen_post_ids.read().await.is_empty());
    }
}
