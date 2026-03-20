//! Wikipedia Client
//!
//! Fetches artist Wikipedia pages and extracts controversy/legal sections.
//! Uses the MediaWiki REST API (no rate limit issues at our scale).

use anyhow::{Context, Result};
use chrono::Utc;
use reqwest::Client;
use scraper::{Html, Selector};
use uuid::Uuid;

use super::FetchedArticle;

/// Wikipedia client for fetching artist controversy data
pub struct WikipediaClient {
    http: Client,
}

/// Sections of interest from Wikipedia
const SECTIONS_OF_INTEREST: &[&str] = &[
    "controversies",
    "controversy",
    "legal issues",
    "legal",
    "personal life",
    "legal problems",
    "legal troubles",
    "criminal history",
    "arrests",
    "incidents",
    "allegations",
    "lawsuits",
    "criticism",
];

impl WikipediaClient {
    pub fn new() -> Self {
        let http = Client::builder()
            .user_agent("NdithBot/1.0 (offense-research-tool)")
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("Failed to build HTTP client");

        Self { http }
    }

    /// Fetch controversy-related content for an artist from Wikipedia
    pub async fn fetch_artist_controversies(
        &self,
        artist_name: &str,
    ) -> Result<Vec<FetchedArticle>> {
        let title = self.search_title(artist_name).await?;
        let title = match title {
            Some(t) => t,
            None => {
                tracing::debug!(artist = artist_name, "No Wikipedia article found");
                return Ok(Vec::new());
            }
        };

        let html = self.fetch_page_html(&title).await?;
        let sections = self.extract_relevant_sections(&html);

        if sections.is_empty() {
            tracing::debug!(
                artist = artist_name,
                title = title,
                "No controversy sections found"
            );
            return Ok(Vec::new());
        }

        let mut articles = Vec::new();
        for (section_name, content) in sections {
            if content.trim().is_empty() {
                continue;
            }

            articles.push(FetchedArticle {
                id: Uuid::new_v4(),
                source_id: Uuid::nil(),
                url: format!(
                    "https://en.wikipedia.org/wiki/{}",
                    urlencoding::encode(&title)
                ),
                title: format!("{} - {} (Wikipedia)", artist_name, section_name),
                content: Some(content),
                published_at: None,
                fetched_at: Utc::now(),
                authors: vec!["Wikipedia".to_string()],
                categories: vec!["wikipedia".to_string(), section_name.to_lowercase()],
                image_url: None,
            });
        }

        tracing::info!(
            artist = artist_name,
            sections = articles.len(),
            "Extracted Wikipedia controversy sections"
        );

        Ok(articles)
    }

    /// Search for the Wikipedia article title for an artist
    async fn search_title(&self, artist_name: &str) -> Result<Option<String>> {
        let url = format!(
            "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&srnamespace=0&srlimit=3&format=json",
            urlencoding::encode(artist_name)
        );

        let response: serde_json::Value = self
            .http
            .get(&url)
            .send()
            .await
            .context("Wikipedia search request failed")?
            .json()
            .await
            .context("Failed to parse Wikipedia search response")?;

        let results = response["query"]["search"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        // Find the best match — prefer exact match or musician/singer articles
        for result in &results {
            let title = result["title"].as_str().unwrap_or_default();
            let snippet = result["snippet"]
                .as_str()
                .unwrap_or_default()
                .to_lowercase();

            if snippet.contains("musician")
                || snippet.contains("singer")
                || snippet.contains("rapper")
                || snippet.contains("artist")
                || snippet.contains("band")
            {
                return Ok(Some(title.to_string()));
            }
        }

        // Fall back to first result
        Ok(results
            .first()
            .and_then(|r| r["title"].as_str())
            .map(|s| s.to_string()))
    }

    /// Fetch the HTML content of a Wikipedia page
    async fn fetch_page_html(&self, title: &str) -> Result<String> {
        let url = format!(
            "https://en.wikipedia.org/api/rest_v1/page/html/{}",
            urlencoding::encode(title)
        );

        let response = self
            .http
            .get(&url)
            .send()
            .await
            .context("Wikipedia page fetch failed")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Wikipedia returned status {}",
                response.status()
            ));
        }

        response
            .text()
            .await
            .context("Failed to read Wikipedia response body")
    }

    /// Extract relevant sections (Controversies, Legal issues, etc.) from HTML
    fn extract_relevant_sections(&self, html: &str) -> Vec<(String, String)> {
        let document = Html::parse_document(html);
        let mut sections = Vec::new();

        // Try to find section headers (h2, h3) that match our sections of interest
        let heading_selector = Selector::parse("h2, h3").expect("Invalid heading selector");

        let headings: Vec<_> = document.select(&heading_selector).collect();

        for heading in &headings {
            let heading_text = heading.text().collect::<String>().trim().to_string();
            let heading_lower = heading_text.to_lowercase();

            // Remove "[edit]" suffix
            let clean_heading = heading_lower
                .replace("[edit]", "")
                .replace("[edit source]", "")
                .trim()
                .to_string();

            let is_relevant = SECTIONS_OF_INTEREST
                .iter()
                .any(|s| clean_heading.contains(s));

            if !is_relevant {
                continue;
            }

            // Collect text from paragraphs following this heading.
            // Simple approach: gather <p> elements from the document
            // (a more robust implementation would traverse siblings directly).
            let mut content_parts = Vec::new();
            let p_selector = Selector::parse("p").expect("Invalid p selector");

            for elem in document.select(&p_selector) {
                let elem_text: String = elem.text().collect();
                let trimmed = elem_text.trim().to_string();

                if !trimmed.is_empty() && trimmed.len() > 20 {
                    content_parts.push(trimmed);
                }

                if content_parts.len() >= 10 {
                    break;
                }
            }

            if content_parts.is_empty() {
                continue;
            }

            content_parts.truncate(5);

            if !content_parts.is_empty() {
                sections.push((heading_text, content_parts.join("\n\n")));
            }
        }

        sections
    }
}

impl Default for WikipediaClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sections_of_interest() {
        assert!(SECTIONS_OF_INTEREST.contains(&"controversies"));
        assert!(SECTIONS_OF_INTEREST.contains(&"legal issues"));
    }

    #[test]
    fn test_extract_relevant_sections() {
        let client = WikipediaClient::new();
        let html = r#"
        <html><body>
        <h2>Early life</h2>
        <p>Some early life info.</p>
        <h2>Controversies</h2>
        <p>This artist was involved in several controversies.</p>
        <p>In 2020, they were arrested for assault.</p>
        <h2>Discography</h2>
        <p>Albums list.</p>
        </body></html>
        "#;

        let sections = client.extract_relevant_sections(html);
        assert_eq!(sections.len(), 1);
        assert!(sections[0].0.contains("Controversies"));
        assert!(sections[0].1.contains("arrested"));
    }
}
