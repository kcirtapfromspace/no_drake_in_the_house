//! Twitter/X Monitor
//!
//! Monitors Twitter/X for artist mentions and news.
//! Uses Twitter API v2 for searching tweets.
//! Rate limit: 450 requests/15 minutes (standard), 300 tweets/15 min (search)

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::rss_fetcher::FetchedArticle;

/// Twitter API v2 base URL
const TWITTER_API_BASE: &str = "https://api.twitter.com/2";

/// Twitter monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterConfig {
    /// Bearer token for API authentication
    pub bearer_token: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum tweets per search
    pub max_results: u32,
    /// Minimum followers for accounts to consider
    pub min_followers: u32,
    /// Accounts to monitor (usernames)
    pub monitored_accounts: Vec<String>,
    /// Keywords to search for
    pub search_keywords: Vec<String>,
}

impl Default for TwitterConfig {
    fn default() -> Self {
        Self {
            bearer_token: String::new(),
            timeout_seconds: 30,
            max_results: 100,
            min_followers: 10000,
            monitored_accounts: vec![
                "pitlocker".to_string(),
                "billboard".to_string(),
                "rollingstone".to_string(),
                "complex".to_string(),
                "xxl".to_string(),
                "hloic".to_string(),
            ],
            search_keywords: vec![
                "musician controversy".to_string(),
                "artist scandal".to_string(),
                "rapper arrested".to_string(),
                "singer accused".to_string(),
            ],
        }
    }
}

/// Rate limiter state
struct RateLimiterState {
    requests_in_window: u32,
    window_start: std::time::Instant,
}

/// Twitter monitor
pub struct TwitterMonitor {
    config: TwitterConfig,
    client: Client,
    rate_limiter: Arc<RwLock<RateLimiterState>>,
    seen_tweet_ids: Arc<RwLock<HashSet<String>>>,
}

/// Twitter API tweet response
#[derive(Debug, Deserialize)]
struct TwitterSearchResponse {
    data: Option<Vec<Tweet>>,
    includes: Option<TweetIncludes>,
    meta: Option<TwitterMeta>,
}

#[derive(Debug, Deserialize)]
struct Tweet {
    id: String,
    text: String,
    author_id: Option<String>,
    created_at: Option<String>,
    entities: Option<TweetEntities>,
    public_metrics: Option<TweetMetrics>,
}

#[derive(Debug, Deserialize)]
struct TweetIncludes {
    users: Option<Vec<TwitterUser>>,
}

#[derive(Debug, Deserialize)]
struct TwitterUser {
    id: String,
    username: String,
    name: String,
    public_metrics: Option<UserMetrics>,
    verified: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct UserMetrics {
    followers_count: u32,
    following_count: u32,
    tweet_count: u32,
}

#[derive(Debug, Deserialize)]
struct TweetEntities {
    urls: Option<Vec<TweetUrl>>,
    mentions: Option<Vec<TweetMention>>,
    hashtags: Option<Vec<TweetHashtag>>,
}

#[derive(Debug, Deserialize)]
struct TweetUrl {
    expanded_url: Option<String>,
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TweetMention {
    username: String,
}

#[derive(Debug, Deserialize)]
struct TweetHashtag {
    tag: String,
}

#[derive(Debug, Deserialize)]
struct TweetMetrics {
    retweet_count: u32,
    reply_count: u32,
    like_count: u32,
    quote_count: u32,
}

#[derive(Debug, Deserialize)]
struct TwitterMeta {
    result_count: u32,
    next_token: Option<String>,
}

/// A processed tweet for our system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedTweet {
    pub id: Uuid,
    pub tweet_id: String,
    pub text: String,
    pub author_username: String,
    pub author_name: String,
    pub author_followers: u32,
    pub author_verified: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub retweet_count: u32,
    pub like_count: u32,
    pub linked_urls: Vec<String>,
    pub hashtags: Vec<String>,
    pub mentions: Vec<String>,
}

impl TwitterMonitor {
    /// Create a new Twitter monitor
    pub fn new(config: TwitterConfig) -> Self {
        let client = Client::builder()
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
            seen_tweet_ids: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Check and update rate limit (450 requests per 15 minutes)
    async fn check_rate_limit(&self) -> Result<()> {
        let mut state = self.rate_limiter.write().await;

        // Reset if window expired (15 minutes)
        if state.window_start.elapsed().as_secs() >= 900 {
            state.requests_in_window = 0;
            state.window_start = std::time::Instant::now();
        }

        if state.requests_in_window >= 450 {
            let wait_time = 900 - state.window_start.elapsed().as_secs();
            return Err(anyhow::anyhow!(
                "Twitter rate limit exceeded, try again in {} seconds",
                wait_time
            ));
        }

        state.requests_in_window += 1;
        Ok(())
    }

    /// Search for tweets
    pub async fn search(&self, query: &str) -> Result<Vec<ProcessedTweet>> {
        if self.config.bearer_token.is_empty() {
            return Err(anyhow::anyhow!("Twitter bearer token not configured"));
        }

        self.check_rate_limit().await?;

        let url = format!(
            "{}/tweets/search/recent?query={}&max_results={}&tweet.fields=created_at,public_metrics,entities,author_id&expansions=author_id&user.fields=username,name,public_metrics,verified",
            TWITTER_API_BASE,
            urlencoding::encode(query),
            self.config.max_results.min(100)
        );

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.config.bearer_token)
            .send()
            .await
            .context("Twitter API request failed")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Twitter API error: {} - {}", status, body));
        }

        let api_response: TwitterSearchResponse = response
            .json()
            .await
            .context("Failed to parse Twitter response")?;

        let users: std::collections::HashMap<String, &TwitterUser> = api_response
            .includes
            .as_ref()
            .and_then(|i| i.users.as_ref())
            .map(|users| users.iter().map(|u| (u.id.clone(), u)).collect())
            .unwrap_or_default();

        let mut tweets = Vec::new();
        let mut seen = self.seen_tweet_ids.write().await;

        for tweet in api_response.data.unwrap_or_default() {
            // Skip if we've seen this tweet
            if seen.contains(&tweet.id) {
                continue;
            }
            seen.insert(tweet.id.clone());

            let author = tweet
                .author_id
                .as_ref()
                .and_then(|id| users.get(id).copied());

            let (author_username, author_name, author_followers, author_verified) = author
                .map(|u| {
                    (
                        u.username.clone(),
                        u.name.clone(),
                        u.public_metrics.as_ref().map(|m| m.followers_count).unwrap_or(0),
                        u.verified.unwrap_or(false),
                    )
                })
                .unwrap_or_else(|| ("unknown".to_string(), "Unknown".to_string(), 0, false));

            // Skip low-follower accounts
            if author_followers < self.config.min_followers {
                continue;
            }

            let created_at = tweet
                .created_at
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc));

            let metrics = tweet.public_metrics.as_ref();

            let entities = tweet.entities.as_ref();
            let linked_urls: Vec<String> = entities
                .and_then(|e| e.urls.as_ref())
                .map(|urls| {
                    urls.iter()
                        .filter_map(|u| u.expanded_url.clone())
                        .collect()
                })
                .unwrap_or_default();

            let hashtags: Vec<String> = entities
                .and_then(|e| e.hashtags.as_ref())
                .map(|tags| tags.iter().map(|t| t.tag.clone()).collect())
                .unwrap_or_default();

            let mentions: Vec<String> = entities
                .and_then(|e| e.mentions.as_ref())
                .map(|m| m.iter().map(|u| u.username.clone()).collect())
                .unwrap_or_default();

            tweets.push(ProcessedTweet {
                id: Uuid::new_v4(),
                tweet_id: tweet.id,
                text: tweet.text,
                author_username,
                author_name,
                author_followers,
                author_verified,
                created_at,
                retweet_count: metrics.map(|m| m.retweet_count).unwrap_or(0),
                like_count: metrics.map(|m| m.like_count).unwrap_or(0),
                linked_urls,
                hashtags,
                mentions,
            });
        }

        Ok(tweets)
    }

    /// Search for music-related tweets
    pub async fn search_music_news(&self) -> Result<Vec<ProcessedTweet>> {
        let mut all_tweets = Vec::new();

        for keyword in &self.config.search_keywords {
            match self.search(keyword).await {
                Ok(tweets) => all_tweets.extend(tweets),
                Err(e) => {
                    tracing::warn!(keyword = %keyword, error = %e, "Twitter search failed");
                }
            }
        }

        Ok(all_tweets)
    }

    /// Search for tweets about a specific artist
    pub async fn search_artist(&self, artist_name: &str) -> Result<Vec<ProcessedTweet>> {
        let query = format!("\"{}\" -is:retweet lang:en", artist_name);
        self.search(&query).await
    }

    /// Convert tweets to FetchedArticle format for unified processing
    pub fn tweets_to_articles(&self, tweets: Vec<ProcessedTweet>) -> Vec<FetchedArticle> {
        tweets
            .into_iter()
            .map(|tweet| FetchedArticle {
                id: tweet.id,
                source_id: Uuid::nil(),
                url: format!("https://twitter.com/{}/status/{}", tweet.author_username, tweet.tweet_id),
                title: format!("Tweet by @{}", tweet.author_username),
                content: Some(tweet.text),
                published_at: tweet.created_at,
                fetched_at: Utc::now(),
                authors: vec![tweet.author_name],
                categories: tweet.hashtags,
                image_url: None,
            })
            .collect()
    }

    /// Check if the monitor is configured
    pub fn is_configured(&self) -> bool {
        !self.config.bearer_token.is_empty()
    }

    /// Clean up old seen tweet IDs
    pub async fn cleanup_seen_tweets(&self, max_size: usize) {
        let mut seen = self.seen_tweet_ids.write().await;
        if seen.len() > max_size {
            // Keep only the most recent (this is a simple approach)
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
        let config = TwitterConfig::default();
        assert!(!config.monitored_accounts.is_empty());
        assert!(!config.search_keywords.is_empty());
    }

    #[tokio::test]
    async fn test_monitor_without_token() {
        let config = TwitterConfig::default();
        let monitor = TwitterMonitor::new(config);

        assert!(!monitor.is_configured());

        let result = monitor.search("test").await;
        assert!(result.is_err());
    }
}
