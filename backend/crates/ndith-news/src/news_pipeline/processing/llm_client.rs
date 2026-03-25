//! Claude LLM Client
//!
//! Thin wrapper around the Anthropic Messages API for offense classification,
//! entity extraction, and evidence synthesis. Uses reqwest (already a dependency).
//!
//! **Cost safety**: All calls go through a BudgetGuard. The kill switch defaults
//! to ON — no API calls will be made unless explicitly enabled and budgeted.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};

// ---------------------------------------------------------------------------
// Budget guard — physically prevents API calls when limits are exceeded
// ---------------------------------------------------------------------------

/// Budget configuration for LLM calls.
/// Defaults are maximally restrictive: kill switch ON, all caps at zero.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmBudgetConfig {
    /// Kill switch — ON by default, rejects ALL calls
    pub kill_switch: bool,
    /// Max calls per day (0 = unlimited once kill_switch is off)
    pub daily_call_limit: u64,
    /// Max estimated spend per day in USD (0.0 = unlimited once kill_switch is off)
    pub daily_cost_cap_usd: f64,
    /// Max estimated spend per month in USD (0.0 = unlimited once kill_switch is off)
    pub monthly_cost_cap_usd: f64,
    /// Path to persist usage counters (survives restarts)
    pub usage_file: Option<PathBuf>,
}

impl Default for LlmBudgetConfig {
    fn default() -> Self {
        Self {
            kill_switch: true,
            daily_call_limit: 0,
            daily_cost_cap_usd: 0.0,
            monthly_cost_cap_usd: 0.0,
            usage_file: None,
        }
    }
}

impl LlmBudgetConfig {
    /// Build config from environment variables, falling back to defaults.
    pub fn from_env() -> Self {
        let kill_switch = std::env::var("LLM_KILL_SWITCH")
            .map(|v| v != "false" && v != "0")
            .unwrap_or(true);

        let daily_call_limit = std::env::var("LLM_DAILY_CALL_LIMIT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        let daily_cost_cap_usd = std::env::var("LLM_DAILY_COST_CAP")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0);

        let monthly_cost_cap_usd = std::env::var("LLM_MONTHLY_COST_CAP")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0);

        let usage_file = std::env::var("LLM_USAGE_FILE").ok().map(PathBuf::from);

        Self {
            kill_switch,
            daily_call_limit,
            daily_cost_cap_usd,
            monthly_cost_cap_usd,
            usage_file,
        }
    }
}

/// Persisted usage counters (JSON file).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct UsageCounters {
    /// ISO-8601 date string for the current day (UTC)
    daily_date: String,
    daily_calls: u64,
    daily_cost_usd: f64,
    /// ISO-8601 month string "YYYY-MM"
    monthly_month: String,
    monthly_cost_usd: f64,
}

/// Budget guard that wraps every API call.
pub struct BudgetGuard {
    config: LlmBudgetConfig,
    counters: Mutex<UsageCounters>,
}

impl BudgetGuard {
    pub fn new(config: LlmBudgetConfig) -> Self {
        let counters = if let Some(ref path) = config.usage_file {
            std::fs::read_to_string(path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            UsageCounters::default()
        };

        Self {
            config,
            counters: Mutex::new(counters),
        }
    }

    /// Check whether we're allowed to make a call. Returns Err if blocked.
    pub async fn check(&self) -> Result<()> {
        if self.config.kill_switch {
            return Err(anyhow::anyhow!(
                "LLM kill switch is ON — all API calls are blocked. \
                 Set LLM_KILL_SWITCH=false to enable."
            ));
        }

        let mut c = self.counters.lock().await;
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let month = chrono::Utc::now().format("%Y-%m").to_string();

        // Reset daily counters if the day rolled over
        if c.daily_date != today {
            c.daily_date = today;
            c.daily_calls = 0;
            c.daily_cost_usd = 0.0;
        }

        // Reset monthly counters if the month rolled over
        if c.monthly_month != month {
            c.monthly_month = month;
            c.monthly_cost_usd = 0.0;
        }

        // Enforce daily call limit
        if self.config.daily_call_limit > 0 && c.daily_calls >= self.config.daily_call_limit {
            return Err(anyhow::anyhow!(
                "LLM daily call limit reached ({}/{})",
                c.daily_calls,
                self.config.daily_call_limit
            ));
        }

        // Enforce daily cost cap
        if self.config.daily_cost_cap_usd > 0.0
            && c.daily_cost_usd >= self.config.daily_cost_cap_usd
        {
            return Err(anyhow::anyhow!(
                "LLM daily cost cap reached (${:.4}/${:.2})",
                c.daily_cost_usd,
                self.config.daily_cost_cap_usd
            ));
        }

        // Enforce monthly cost cap
        if self.config.monthly_cost_cap_usd > 0.0
            && c.monthly_cost_usd >= self.config.monthly_cost_cap_usd
        {
            return Err(anyhow::anyhow!(
                "LLM monthly cost cap reached (${:.4}/${:.2})",
                c.monthly_cost_usd,
                self.config.monthly_cost_cap_usd
            ));
        }

        Ok(())
    }

    /// Record a completed call's cost.
    pub async fn record(&self, model: &ClaudeModel, input_tokens: u64, output_tokens: u64) {
        let cost = model.estimate_cost(input_tokens, output_tokens);

        let mut c = self.counters.lock().await;
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let month = chrono::Utc::now().format("%Y-%m").to_string();

        // Reset daily counters if the day rolled over
        if c.daily_date != today {
            c.daily_date = today;
            c.daily_calls = 0;
            c.daily_cost_usd = 0.0;
        }

        // Reset monthly counters if the month rolled over
        if c.monthly_month != month {
            c.monthly_month = month;
            c.monthly_cost_usd = 0.0;
        }

        c.daily_calls += 1;
        c.daily_cost_usd += cost;
        c.monthly_cost_usd += cost;

        tracing::info!(
            model = ?model,
            input_tokens,
            output_tokens,
            cost_usd = cost,
            daily_total = c.daily_cost_usd,
            monthly_total = c.monthly_cost_usd,
            "LLM API call recorded"
        );

        // Persist
        if let Some(ref path) = self.config.usage_file {
            if let Ok(json) = serde_json::to_string_pretty(&*c) {
                let _ = std::fs::write(path, json);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Claude model tiers
// ---------------------------------------------------------------------------

/// Claude model tiers for cost/quality tradeoff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClaudeModel {
    /// Fast, cheap — use for high-volume classification and entity extraction
    Haiku,
    /// Balanced — use for research planning and evidence synthesis
    Sonnet,
}

impl ClaudeModel {
    fn model_id(&self) -> &'static str {
        match self {
            ClaudeModel::Haiku => "claude-haiku-4-5-20251001",
            ClaudeModel::Sonnet => "claude-sonnet-4-5-20241022",
        }
    }

    /// Estimate cost in USD given token counts.
    /// Haiku: $1 / MTok input, $5 / MTok output
    /// Sonnet: $3 / MTok input, $15 / MTok output
    fn estimate_cost(&self, input_tokens: u64, output_tokens: u64) -> f64 {
        let (input_price, output_price) = match self {
            ClaudeModel::Haiku => (1.0, 5.0),
            ClaudeModel::Sonnet => (3.0, 15.0),
        };
        (input_tokens as f64 * input_price + output_tokens as f64 * output_price) / 1_000_000.0
    }
}

// ---------------------------------------------------------------------------
// Data types (kept for backward compatibility — other modules may reference)
// ---------------------------------------------------------------------------

/// LLM classification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmClassification {
    pub categories: Vec<LlmCategoryResult>,
    pub subject_role: SubjectRole,
    pub temporal_info: Option<TemporalInfo>,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCategoryResult {
    pub category: String,
    pub confidence: f64,
    pub severity: String,
    pub evidence_snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubjectRole {
    Perpetrator,
    Victim,
    Witness,
    Unrelated,
    Unclear,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalInfo {
    pub timeframe: String,
    pub is_historical: bool,
    pub approximate_date: Option<String>,
}

/// LLM entity extraction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmEntity {
    pub name: String,
    pub entity_type: String,
    pub confidence: f64,
    pub aliases: Vec<String>,
    pub context: String,
}

/// Evidence summary from cross-source synthesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSummary {
    pub summary: String,
    pub key_facts: Vec<String>,
    pub source_agreement: f64,
    pub gaps: Vec<String>,
    pub recommended_actions: Vec<String>,
}

/// Research plan from Claude for autoresearch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchPlan {
    pub priority_action: String,
    pub search_queries: Vec<String>,
    pub target_sources: Vec<String>,
    pub reasoning: String,
    pub expected_improvement: String,
}

// ---------------------------------------------------------------------------
// API usage tracking
// ---------------------------------------------------------------------------

/// API usage tracking
#[derive(Debug, Clone, Default)]
pub struct ApiUsageStats {
    pub total_calls: u64,
    pub haiku_calls: u64,
    pub sonnet_calls: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub cache_hits: u64,
    pub errors: u64,
    pub estimated_cost_usd: f64,
}

// ---------------------------------------------------------------------------
// Client config & client
// ---------------------------------------------------------------------------

/// Claude API client configuration
#[derive(Debug, Clone)]
pub struct ClaudeClientConfig {
    pub api_key: String,
    pub max_tokens: u32,
    pub timeout_secs: u64,
    pub max_retries: u32,
    pub budget: LlmBudgetConfig,
}

impl Default for ClaudeClientConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            max_tokens: 1024,
            timeout_secs: 30,
            max_retries: 2,
            budget: LlmBudgetConfig::default(),
        }
    }
}

/// Claude API client
pub struct ClaudeClient {
    http: Client,
    config: ClaudeClientConfig,
    cache: Arc<RwLock<HashMap<String, String>>>,
    usage: Arc<Mutex<ApiUsageStats>>,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    budget_guard: Arc<BudgetGuard>,
}

struct RateLimiter {
    last_call: std::time::Instant,
    min_interval: Duration,
}

impl RateLimiter {
    fn new(requests_per_minute: u32) -> Self {
        Self {
            last_call: std::time::Instant::now() - Duration::from_secs(60),
            min_interval: Duration::from_secs_f64(60.0 / requests_per_minute as f64),
        }
    }

    async fn wait(&mut self) {
        let elapsed = self.last_call.elapsed();
        if elapsed < self.min_interval {
            tokio::time::sleep(self.min_interval - elapsed).await;
        }
        self.last_call = std::time::Instant::now();
    }
}

/// Anthropic Messages API request
#[derive(Debug, Serialize)]
struct MessagesRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    system: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

/// Anthropic Messages API response
#[derive(Debug, Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: u64,
    output_tokens: u64,
}

impl ClaudeClient {
    /// Create a new Claude client from environment
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .context("ANTHROPIC_API_KEY environment variable not set")?;

        Ok(Self::new(ClaudeClientConfig {
            api_key,
            budget: LlmBudgetConfig::from_env(),
            ..Default::default()
        }))
    }

    /// Create a new Claude client with config
    pub fn new(config: ClaudeClientConfig) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to build HTTP client");

        let budget_guard = Arc::new(BudgetGuard::new(config.budget.clone()));

        Self {
            http,
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            usage: Arc::new(Mutex::new(ApiUsageStats::default())),
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(50))),
            budget_guard,
        }
    }

    /// Classify an article using Claude
    pub async fn classify_article(
        &self,
        text: &str,
        title: &str,
        entities: &[String],
    ) -> Result<LlmClassification> {
        let cache_key = format!("classify:{}", sha256_short(text));

        // Check cache
        if let Some(cached) = self.get_cached(&cache_key).await {
            let mut usage = self.usage.lock().await;
            usage.cache_hits += 1;
            return serde_json::from_str(&cached).context("Failed to parse cached classification");
        }

        let entity_list = if entities.is_empty() {
            "No specific entities identified".to_string()
        } else {
            entities.join(", ")
        };

        let system = r#"You are an offense classifier for a music industry accountability tool. Analyze articles about musicians/artists and classify any offenses.

Respond ONLY with valid JSON in this exact format:
{
  "categories": [{"category": "string", "confidence": 0.0-1.0, "severity": "low|medium|high|critical", "evidence_snippet": "string"}],
  "subject_role": "perpetrator|victim|witness|unrelated|unclear",
  "temporal_info": {"timeframe": "string", "is_historical": true/false, "approximate_date": "YYYY-MM or null"},
  "reasoning": "string"
}

Valid categories: sexual_misconduct, domestic_violence, hate_speech, racism, antisemitism, homophobia, child_abuse, animal_cruelty, financial_crimes, drug_offenses, violent_crimes, harassment, plagiarism, certified_creeper, other

CRITICAL: Distinguish between the artist being the PERPETRATOR vs VICTIM. "Artist X was attacked" means X is a victim, not a perpetrator."#;

        let prompt = format!(
            "Title: {}\n\nEntities mentioned: {}\n\nArticle text:\n{}",
            title,
            entity_list,
            &text[..text.len().min(3000)]
        );

        let response = self.call(ClaudeModel::Haiku, system, &prompt).await?;
        let classification: LlmClassification =
            serde_json::from_str(&response).context("Failed to parse LLM classification")?;

        // Cache result
        self.set_cached(&cache_key, &response).await;

        Ok(classification)
    }

    /// Extract entities using Claude when regex fails
    pub async fn extract_entities(&self, text: &str) -> Result<Vec<LlmEntity>> {
        let system = r#"You are a named entity recognition system specialized in music industry figures. Extract all musician/artist names from the text.

Respond ONLY with valid JSON array:
[{"name": "string", "entity_type": "artist|band|label|producer", "confidence": 0.0-1.0, "aliases": ["string"], "context": "surrounding text snippet"}]

Handle ALL name formats: all-caps (DMX, ASAP Rocky), names with numbers (21 Savage, 6ix9ine), hyphenated (Jay-Z), single-word (Drake, Rihanna), stage names, and legal names."#;

        let prompt = format!(
            "Extract all musician/artist entities from this text:\n\n{}",
            &text[..text.len().min(3000)]
        );

        let response = self.call(ClaudeModel::Haiku, system, &prompt).await?;
        let entities: Vec<LlmEntity> =
            serde_json::from_str(&response).context("Failed to parse LLM entities")?;

        Ok(entities)
    }

    /// Synthesize evidence from multiple sources
    pub async fn summarize_evidence(
        &self,
        artist_name: &str,
        sources: &[String],
    ) -> Result<EvidenceSummary> {
        let system = r#"You are an evidence synthesis system. Analyze multiple source summaries about a musician and create a unified assessment.

Respond ONLY with valid JSON:
{
  "summary": "Concise factual summary",
  "key_facts": ["fact1", "fact2"],
  "source_agreement": 0.0-1.0,
  "gaps": ["What information is missing"],
  "recommended_actions": ["What to search next"]
}"#;

        let sources_text = sources
            .iter()
            .enumerate()
            .map(|(i, s)| format!("Source {}:\n{}", i + 1, s))
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        let prompt = format!(
            "Artist: {}\n\nSummarize and cross-reference these sources:\n\n{}",
            artist_name, sources_text
        );

        let response = self.call(ClaudeModel::Sonnet, system, &prompt).await?;
        serde_json::from_str(&response).context("Failed to parse evidence summary")
    }

    /// Plan next research action for autoresearch loop
    pub async fn plan_research(
        &self,
        artist_name: &str,
        current_scores: &str,
        sources_searched: &[String],
        existing_offenses: &[String],
    ) -> Result<ResearchPlan> {
        let system = r#"You are a research planning agent. Given an artist's current research quality scores and what has already been searched, recommend the single most impactful research action.

Respond ONLY with valid JSON:
{
  "priority_action": "Description of what to do next",
  "search_queries": ["query1", "query2"],
  "target_sources": ["wikipedia|brave_search|newsapi|reddit"],
  "reasoning": "Why this action will most improve the research quality",
  "expected_improvement": "Which score dimension this targets"
}"#;

        let searched = if sources_searched.is_empty() {
            "None yet".to_string()
        } else {
            sources_searched.join(", ")
        };

        let offenses = if existing_offenses.is_empty() {
            "None found yet".to_string()
        } else {
            existing_offenses.join("; ")
        };

        let prompt = format!(
            "Artist: {}\n\nCurrent quality scores:\n{}\n\nSources already searched: {}\n\nOffenses found so far: {}\n\nWhat single research action would most improve the overall quality score?",
            artist_name, current_scores, searched, offenses
        );

        let response = self.call(ClaudeModel::Sonnet, system, &prompt).await?;
        serde_json::from_str(&response).context("Failed to parse research plan")
    }

    /// Make an API call to Claude.
    /// Budget is checked BEFORE the HTTP request — never after.
    async fn call(&self, model: ClaudeModel, system: &str, prompt: &str) -> Result<String> {
        // ── Budget check (before anything else) ──
        self.budget_guard.check().await?;

        // Rate limit
        {
            let mut limiter = self.rate_limiter.lock().await;
            limiter.wait().await;
        }

        let request = MessagesRequest {
            model: model.model_id().to_string(),
            max_tokens: self.config.max_tokens,
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            system: Some(system.to_string()),
        };

        let mut last_error = None;
        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                let backoff = Duration::from_millis(500 * 2_u64.pow(attempt - 1));
                tokio::time::sleep(backoff).await;
            }

            match self.do_request(&request, &model).await {
                Ok(text) => return Ok(text),
                Err(e) => {
                    tracing::warn!(
                        attempt = attempt + 1,
                        max = self.config.max_retries + 1,
                        error = %e,
                        "Claude API call failed"
                    );
                    last_error = Some(e);
                }
            }
        }

        // Track error
        {
            let mut usage = self.usage.lock().await;
            usage.errors += 1;
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown error")))
    }

    async fn do_request(&self, request: &MessagesRequest, model: &ClaudeModel) -> Result<String> {
        let api_url = "https://api.anthropic.com/v1/messages";

        let mut response = None;
        for attempt in 0..=3u32 {
            let resp = self
                .http
                .post(api_url)
                .header("x-api-key", &self.config.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(request)
                .send()
                .await
                .context("Failed to send request to Claude API")?;

            if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                if attempt == 3 {
                    return Err(anyhow::anyhow!("Claude API rate limited after 4 attempts"));
                }
                let wait = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(2u64.pow(attempt + 1))
                    .min(60);
                tracing::warn!(
                    "Rate limited on Claude API, retry {}/3 after {}s",
                    attempt + 1,
                    wait
                );
                tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
                continue;
            }

            response = Some(resp);
            break;
        }

        let response =
            response.ok_or_else(|| anyhow::anyhow!("Claude API: no response after retries"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Claude API error {}: {}",
                status,
                &body[..body.len().min(500)]
            ));
        }

        let body: MessagesResponse = response
            .json()
            .await
            .context("Failed to parse Claude API response")?;

        // Track usage & record budget
        if let Some(usage) = &body.usage {
            let cost = model.estimate_cost(usage.input_tokens, usage.output_tokens);

            let mut stats = self.usage.lock().await;
            stats.total_calls += 1;
            match model {
                ClaudeModel::Haiku => stats.haiku_calls += 1,
                ClaudeModel::Sonnet => stats.sonnet_calls += 1,
            }
            stats.total_input_tokens += usage.input_tokens;
            stats.total_output_tokens += usage.output_tokens;
            stats.estimated_cost_usd += cost;

            // Record in budget guard (persists to disk)
            self.budget_guard
                .record(model, usage.input_tokens, usage.output_tokens)
                .await;
        }

        let text = body
            .content
            .into_iter()
            .filter_map(|c| c.text)
            .collect::<Vec<_>>()
            .join("");

        if text.is_empty() {
            return Err(anyhow::anyhow!("Empty response from Claude API"));
        }

        // Extract JSON from response (handle markdown code blocks)
        let json_text = extract_json(&text);

        Ok(json_text)
    }

    async fn get_cached(&self, key: &str) -> Option<String> {
        let cache = self.cache.read().await;
        cache.get(key).cloned()
    }

    async fn set_cached(&self, key: &str, value: &str) {
        let mut cache = self.cache.write().await;
        // Simple LRU: clear if too large
        if cache.len() > 10000 {
            cache.clear();
        }
        cache.insert(key.to_string(), value.to_string());
    }

    /// Get API usage statistics
    pub async fn get_usage(&self) -> ApiUsageStats {
        self.usage.lock().await.clone()
    }
}

/// Extract JSON from a response that may be wrapped in markdown code blocks
fn extract_json(text: &str) -> String {
    let trimmed = text.trim();

    // Try to extract from ```json ... ``` blocks
    if let Some(start) = trimmed.find("```json") {
        let json_start = start + 7;
        if let Some(end) = trimmed[json_start..].find("```") {
            return trimmed[json_start..json_start + end].trim().to_string();
        }
    }

    // Try to extract from ``` ... ``` blocks
    if let Some(start) = trimmed.find("```") {
        let json_start = start + 3;
        // Skip optional language identifier on first line
        let actual_start = trimmed[json_start..]
            .find('\n')
            .map(|n| json_start + n + 1)
            .unwrap_or(json_start);
        if let Some(end) = trimmed[actual_start..].find("```") {
            return trimmed[actual_start..actual_start + end].trim().to_string();
        }
    }

    // Already plain JSON
    trimmed.to_string()
}

/// Quick hash for cache keys
fn sha256_short(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_plain() {
        let input = r#"{"key": "value"}"#;
        assert_eq!(extract_json(input), r#"{"key": "value"}"#);
    }

    #[test]
    fn test_extract_json_code_block() {
        let input = "```json\n{\"key\": \"value\"}\n```";
        assert_eq!(extract_json(input), r#"{"key": "value"}"#);
    }

    #[test]
    fn test_sha256_short_deterministic() {
        assert_eq!(sha256_short("hello"), sha256_short("hello"));
        assert_ne!(sha256_short("hello"), sha256_short("world"));
    }

    #[test]
    fn test_model_ids() {
        assert!(ClaudeModel::Haiku.model_id().contains("haiku"));
        assert!(ClaudeModel::Sonnet.model_id().contains("sonnet"));
    }

    #[test]
    fn test_budget_config_defaults() {
        let config = LlmBudgetConfig::default();
        assert!(config.kill_switch);
        assert_eq!(config.daily_call_limit, 0);
        assert_eq!(config.daily_cost_cap_usd, 0.0);
        assert_eq!(config.monthly_cost_cap_usd, 0.0);
    }

    #[tokio::test]
    async fn test_kill_switch_blocks_calls() {
        let guard = BudgetGuard::new(LlmBudgetConfig::default());
        assert!(guard.check().await.is_err());
    }

    #[tokio::test]
    async fn test_budget_allows_when_kill_switch_off() {
        let guard = BudgetGuard::new(LlmBudgetConfig {
            kill_switch: false,
            ..Default::default()
        });
        assert!(guard.check().await.is_ok());
    }

    #[tokio::test]
    async fn test_daily_cost_cap_enforced() {
        let guard = BudgetGuard::new(LlmBudgetConfig {
            kill_switch: false,
            daily_cost_cap_usd: 0.01,
            ..Default::default()
        });

        // Record some cost
        guard.record(&ClaudeModel::Sonnet, 100_000, 10_000).await;
        // Sonnet: 100k * $3/M + 10k * $15/M = $0.30 + $0.15 = $0.45 — over the $0.01 cap
        assert!(guard.check().await.is_err());
    }

    #[test]
    fn test_cost_estimation() {
        // Haiku: 1000 input, 500 output → $0.001 + $0.0025 = $0.0035
        let cost = ClaudeModel::Haiku.estimate_cost(1000, 500);
        assert!((cost - 0.0035).abs() < 1e-9);

        // Sonnet: 1000 input, 500 output → $0.003 + $0.0075 = $0.0105
        let cost = ClaudeModel::Sonnet.estimate_cost(1000, 500);
        assert!((cost - 0.0105).abs() < 1e-9);
    }
}
