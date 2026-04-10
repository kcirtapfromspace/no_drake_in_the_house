//! Offense Creator Service
//!
//! Automatically creates artist_offenses records from news_offense_classifications.
//! Writes offense records and evidence to Convex via HTTP mutations.
//! Deduplication is handled by the Convex `createOffenseFromResearch` mutation
//! (same artist + category within 30 days = update, not create).
//! Score recalculation is handled by Convex cron jobs.

use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::convex_client::{ConvexClient, CreateOffenseArgs, LinkEvidenceArgs, UpsertResponse};

use super::processing::OffenseClassification;

/// Minimum confidence threshold for auto-creating offenses
const CONFIDENCE_THRESHOLD: f64 = 0.7;

/// Strip HTML tags and collapse whitespace from article content.
fn strip_html(s: &str) -> String {
    // Remove HTML tags
    let re = regex::Regex::new(r"<[^>]*>").unwrap();
    let stripped = re.replace_all(s, "");
    // Collapse whitespace
    let ws = regex::Regex::new(r"\s+").unwrap();
    ws.replace_all(&stripped, " ").trim().to_string()
}

/// Service for creating offense records from news detections.
///
/// Writes to Convex via `ConvexClient`. No PostgreSQL dependency.
pub struct OffenseCreator {
    convex: ConvexClient,
}

/// Result of processing a classification
#[derive(Debug)]
pub struct OffenseCreationResult {
    /// Whether an offense was created
    pub created: bool,
    /// The offense ID (existing or new) — Convex document ID string
    pub offense_id: Option<Uuid>,
    /// The Convex document ID returned by the mutation
    pub convex_offense_id: Option<String>,
    /// Whether evidence was linked
    pub evidence_linked: bool,
    /// Reason if not created
    pub reason: Option<String>,
}

impl OffenseCreator {
    /// Create a new offense creator backed by Convex.
    pub fn new(convex: ConvexClient) -> Self {
        Self { convex }
    }

    /// Process a news offense classification and create an artist offense in Convex.
    ///
    /// Deduplication is handled server-side by the Convex mutation
    /// (`createOffenseFromResearch` deduplicates by artist + category within 30 days).
    pub async fn process_classification(
        &self,
        classification: &OffenseClassification,
        _article_id: Uuid,
        article_title: &str,
        article_url: &str,
        _published_at: Option<DateTime<Utc>>,
    ) -> Result<OffenseCreationResult> {
        // 1. Check confidence threshold
        if classification.confidence < CONFIDENCE_THRESHOLD {
            return Ok(OffenseCreationResult {
                created: false,
                offense_id: None,
                convex_offense_id: None,
                evidence_linked: false,
                reason: Some(format!(
                    "Confidence {:.2} below threshold {:.2}",
                    classification.confidence, CONFIDENCE_THRESHOLD
                )),
            });
        }

        // 2. Must have an artist ID
        let artist_id = match classification.artist_id {
            Some(id) => id,
            None => {
                return Ok(OffenseCreationResult {
                    created: false,
                    offense_id: None,
                    convex_offense_id: None,
                    evidence_linked: false,
                    reason: Some("No artist ID associated with classification".to_string()),
                });
            }
        };

        // 3. Create/update offense via Convex mutation (handles dedup server-side)
        let category = classification.category.to_string();
        let severity = format!("{:?}", classification.severity).to_lowercase();

        // Use Convex document ID if available, otherwise fall back to UUID
        let convex_id = classification
            .convex_artist_id
            .clone()
            .unwrap_or_else(|| artist_id.to_string());

        let create_args = CreateOffenseArgs {
            artist_id: convex_id,
            category: category.clone(),
            severity,
            title: format!("Auto-detected: {}", strip_html(article_title)),
            description: Some(strip_html(&classification.context)),
            confidence: classification.confidence,
            source_article_url: Some(article_url.to_string()),
        };

        let response: UpsertResponse = self
            .convex
            .create_offense_from_research(&create_args)
            .await?;

        let created = response.upserted == "created";
        let convex_offense_id = response.id.clone();

        if created {
            tracing::info!(
                convex_offense_id = %convex_offense_id,
                artist_id = %artist_id,
                category = %category,
                confidence = classification.confidence,
                "Created new offense in Convex from news detection"
            );
        } else {
            tracing::debug!(
                convex_offense_id = %convex_offense_id,
                artist_id = %artist_id,
                category = %category,
                "Updated existing offense in Convex (dedup within 30 days)"
            );
        }

        // 4. Link the article as evidence via Convex mutation
        let evidence_linked = self
            .link_evidence(
                &convex_offense_id,
                article_url,
                article_title,
                &classification.context,
            )
            .await
            .is_ok();

        // 5. Score recalculation is handled by Convex cron — no action needed

        Ok(OffenseCreationResult {
            created,
            offense_id: Some(artist_id), // Keep for backward compat (not the real offense UUID)
            convex_offense_id: Some(convex_offense_id),
            evidence_linked,
            reason: if created {
                None
            } else {
                Some("Duplicate offense — updated existing and linked evidence".to_string())
            },
        })
    }

    /// Link an article as evidence for an offense via Convex.
    async fn link_evidence(
        &self,
        convex_offense_id: &str,
        source_url: &str,
        title: &str,
        excerpt: &str,
    ) -> Result<UpsertResponse> {
        let clean_excerpt = strip_html(excerpt);
        let args = LinkEvidenceArgs {
            offense_id: convex_offense_id.to_string(),
            source_url: source_url.to_string(),
            title: Some(strip_html(title)),
            excerpt: Some(clean_excerpt[..clean_excerpt.len().min(500)].to_string()),
            credibility_score: Some(3.0), // Default credibility
        };

        let response = self.convex.link_offense_evidence(&args).await?;

        tracing::debug!(
            evidence_id = %response.id,
            offense_id = %convex_offense_id,
            url = %source_url,
            "Linked article as evidence via Convex"
        );

        Ok(response)
    }

    /// Process multiple classifications from a processed article
    pub async fn process_article_offenses(
        &self,
        article_id: Uuid,
        article_title: &str,
        article_url: &str,
        published_at: Option<DateTime<Utc>>,
        classifications: &[OffenseClassification],
    ) -> Result<Vec<OffenseCreationResult>> {
        let mut results = Vec::with_capacity(classifications.len());

        for classification in classifications {
            let result = self
                .process_classification(
                    classification,
                    article_id,
                    article_title,
                    article_url,
                    published_at,
                )
                .await?;
            results.push(result);
        }

        let created_count = results.iter().filter(|r| r.created).count();
        let linked_count = results.iter().filter(|r| r.evidence_linked).count();

        if created_count > 0 || linked_count > 0 {
            tracing::info!(
                article_id = %article_id,
                created = created_count,
                evidence_linked = linked_count,
                total = classifications.len(),
                "Processed article offenses via Convex"
            );
        }

        Ok(results)
    }
}

/// Convert news classifier category to database category string
fn _parse_category(s: &str) -> super::processing::OffenseCategory {
    use super::processing::OffenseCategory;
    match s.to_lowercase().as_str() {
        "domestic_violence" => OffenseCategory::DomesticViolence,
        "sexual_misconduct" => OffenseCategory::SexualMisconduct,
        "hate_speech" => OffenseCategory::HateSpeech,
        "racism" => OffenseCategory::Racism,
        "antisemitism" => OffenseCategory::Antisemitism,
        "financial_crimes" => OffenseCategory::FinancialCrimes,
        "drug_offenses" => OffenseCategory::DrugOffenses,
        "violent_crimes" => OffenseCategory::ViolentCrimes,
        "child_abuse" => OffenseCategory::ChildAbuse,
        "harassment" => OffenseCategory::Harassment,
        "homophobia" => OffenseCategory::Homophobia,
        "animal_cruelty" => OffenseCategory::AnimalCruelty,
        "plagiarism" => OffenseCategory::Plagiarism,
        "certified_creeper" => OffenseCategory::CertifiedCreeper,
        _ => OffenseCategory::Other,
    }
}

/// Convert Rust OffenseSeverity to database severity string
fn _severity_to_db(severity: &super::processing::OffenseSeverity) -> &'static str {
    use super::processing::OffenseSeverity;
    match severity {
        OffenseSeverity::Critical => "egregious",
        OffenseSeverity::High => "severe",
        OffenseSeverity::Medium => "moderate",
        OffenseSeverity::Low => "minor",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_category() {
        use super::super::processing::OffenseCategory;
        assert!(matches!(
            _parse_category("domestic_violence"),
            OffenseCategory::DomesticViolence
        ));
        assert!(matches!(
            _parse_category("SEXUAL_MISCONDUCT"),
            OffenseCategory::SexualMisconduct
        ));
        assert!(matches!(_parse_category("unknown"), OffenseCategory::Other));
    }

    #[test]
    fn test_severity_to_db() {
        use super::super::processing::OffenseSeverity;
        assert_eq!(_severity_to_db(&OffenseSeverity::Critical), "egregious");
        assert_eq!(_severity_to_db(&OffenseSeverity::High), "severe");
        assert_eq!(_severity_to_db(&OffenseSeverity::Medium), "moderate");
        assert_eq!(_severity_to_db(&OffenseSeverity::Low), "minor");
    }
}
