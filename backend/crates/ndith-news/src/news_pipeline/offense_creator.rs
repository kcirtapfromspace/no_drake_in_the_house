//! Offense Creator Service
//!
//! Routes legacy news detections into the shared Jev evaluation queue.
//! Only the Convex reviewer workflow can create approved offenses.
//! Queue deduplication is by artist/source/model/policy; keywords never grant approval.
//! Approval updates the artist index atomically and schedules user-score recomputation.

use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::convex_client::{ConvexClient, QueueEvaluationArgs};

use super::processing::OffenseClassification;

/// Strip HTML tags and collapse whitespace from article content.
fn strip_html(s: &str) -> String {
    // Remove HTML tags
    let re = regex::Regex::new(r"<[^>]*>").unwrap();
    let stripped = re.replace_all(s, "");
    // Collapse whitespace
    let ws = regex::Regex::new(r"\s+").unwrap();
    ws.replace_all(&stripped, " ").trim().to_string()
}

/// Compatibility service for routing news detections into evidence evaluation.
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
    /// Create a candidate evaluator backed by Convex.
    pub fn new(convex: ConvexClient) -> Self {
        Self { convex }
    }

    /// Compatibility hook: queue a candidate source without creating an offense.
    pub async fn process_classification(
        &self,
        classification: &OffenseClassification,
        _article_id: Uuid,
        article_title: &str,
        article_url: &str,
        _published_at: Option<DateTime<Utc>>,
    ) -> Result<OffenseCreationResult> {
        let convex_id = match classification.convex_artist_id.clone() {
            Some(id) => id,
            None => {
                return Ok(OffenseCreationResult {
                    created: false,
                    offense_id: None,
                    convex_offense_id: None,
                    evidence_linked: false,
                    reason: Some("Unresolved artist identity".to_string()),
                })
            }
        };
        let response = self
            .convex
            .queue_evaluation(&QueueEvaluationArgs {
                artist_id: convex_id,
                url: article_url.to_string(),
                source_title: strip_html(article_title),
            })
            .await?;
        tracing::info!(evaluation_job_id = %response.job_id, "Queued evidence evaluation; reviewer approval required");
        Ok(OffenseCreationResult {
            created: false,
            offense_id: None,
            convex_offense_id: None,
            evidence_linked: false,
            reason: Some(format!("Evaluation queued: {}", response.job_id)),
        })
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
