//! Offense Creator Service
//!
//! Automatically creates artist_offenses records from news_offense_classifications.
//! Handles deduplication, confidence thresholds, and evidence linking.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::processing::OffenseClassification;

/// Minimum confidence threshold for auto-creating offenses
const CONFIDENCE_THRESHOLD: f64 = 0.7;

/// Service for creating offense records from news detections
pub struct OffenseCreator {
    db_pool: PgPool,
}

/// Result of processing a classification
#[derive(Debug)]
pub struct OffenseCreationResult {
    /// Whether an offense was created
    pub created: bool,
    /// The offense ID (existing or new)
    pub offense_id: Option<Uuid>,
    /// Whether evidence was linked
    pub evidence_linked: bool,
    /// Reason if not created
    pub reason: Option<String>,
}

impl OffenseCreator {
    /// Create a new offense creator
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Process a news offense classification and potentially create an artist offense
    pub async fn process_classification(
        &self,
        classification: &OffenseClassification,
        article_id: Uuid,
        article_title: &str,
        article_url: &str,
        published_at: Option<DateTime<Utc>>,
    ) -> Result<OffenseCreationResult> {
        // 1. Check confidence threshold
        if classification.confidence < CONFIDENCE_THRESHOLD {
            return Ok(OffenseCreationResult {
                created: false,
                offense_id: None,
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
                    evidence_linked: false,
                    reason: Some("No artist ID associated with classification".to_string()),
                });
            }
        };

        // 3. Check for duplicate offense (same artist + category + similar timeframe)
        let incident_date = published_at.map(|d| d.date_naive());
        if let Some(existing_id) = self
            .find_duplicate_offense(artist_id, &classification.category, incident_date)
            .await?
        {
            // Link evidence to existing offense
            let evidence_linked = self
                .link_evidence(existing_id, article_url, article_title, &classification.context)
                .await
                .is_ok();

            return Ok(OffenseCreationResult {
                created: false,
                offense_id: Some(existing_id),
                evidence_linked,
                reason: Some("Duplicate offense - linked as additional evidence".to_string()),
            });
        }

        // 4. Create the offense
        let category = classification.category.to_string(); // Uses Display impl
        let severity = severity_to_db(&classification.severity);

        let offense_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO artist_offenses (
                artist_id, category, severity, title, description,
                incident_date, incident_date_approximate, status,
                verification_status, source_classification_id
            )
            VALUES (
                $1, $2::offense_category, $3::offense_severity, $4, $5,
                $6, true, 'active', 'pending', $7
            )
            RETURNING id
            "#,
        )
        .bind(artist_id)
        .bind(&category)
        .bind(&severity)
        .bind(format!("Auto-detected: {}", article_title))
        .bind(&classification.context)
        .bind(incident_date)
        .bind(classification.id)
        .fetch_one(&self.db_pool)
        .await
        .context("Failed to create offense")?;

        tracing::info!(
            offense_id = %offense_id,
            artist_id = %artist_id,
            category = %category,
            confidence = classification.confidence,
            "Created new offense from news detection"
        );

        // 5. Link the article as evidence
        let evidence_linked = self
            .link_evidence(offense_id, article_url, article_title, &classification.context)
            .await
            .is_ok();

        // 6. Trigger trouble score recalculation
        self.trigger_score_recalculation(artist_id, "news_detection")
            .await?;

        Ok(OffenseCreationResult {
            created: true,
            offense_id: Some(offense_id),
            evidence_linked,
            reason: None,
        })
    }

    /// Find a potentially duplicate offense
    async fn find_duplicate_offense(
        &self,
        artist_id: Uuid,
        category: &super::processing::OffenseCategory,
        incident_date: Option<chrono::NaiveDate>,
    ) -> Result<Option<Uuid>> {
        let category_str = format!("{:?}", category).to_lowercase();

        // Look for existing offense with same artist and category within 30 days
        let existing: Option<Uuid> = sqlx::query_scalar(
            r#"
            SELECT id FROM artist_offenses
            WHERE artist_id = $1
              AND category = $2::offense_category
              AND (
                incident_date IS NULL
                OR $3::date IS NULL
                OR ABS(incident_date - $3::date) <= 30
              )
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(artist_id)
        .bind(&category_str)
        .bind(incident_date)
        .fetch_optional(&self.db_pool)
        .await
        .context("Failed to check for duplicate offense")?;

        Ok(existing)
    }

    /// Link an article as evidence for an offense
    async fn link_evidence(
        &self,
        offense_id: Uuid,
        source_url: &str,
        title: &str,
        excerpt: &str,
    ) -> Result<Uuid> {
        // Get source credibility if we have it
        let credibility: i32 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(
                (SELECT credibility_score FROM news_sources ns
                 JOIN news_articles na ON ns.id = na.source_id
                 WHERE na.url = $1),
                3
            )
            "#,
        )
        .bind(source_url)
        .fetch_one(&self.db_pool)
        .await
        .unwrap_or(3);

        let evidence_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO offense_evidence (
                offense_id, source_url, source_name, source_type,
                title, excerpt, credibility_score, is_primary_source
            )
            VALUES ($1, $2, 'News Article', 'news', $3, $4, $5, false)
            ON CONFLICT (offense_id, source_url) DO UPDATE
            SET credibility_score = GREATEST(offense_evidence.credibility_score, $5)
            RETURNING id
            "#,
        )
        .bind(offense_id)
        .bind(source_url)
        .bind(title)
        .bind(&excerpt[..excerpt.len().min(500)])
        .bind(credibility)
        .fetch_one(&self.db_pool)
        .await
        .context("Failed to link evidence")?;

        tracing::debug!(
            evidence_id = %evidence_id,
            offense_id = %offense_id,
            url = %source_url,
            "Linked article as evidence"
        );

        Ok(evidence_id)
    }

    /// Trigger trouble score recalculation for an artist
    async fn trigger_score_recalculation(&self, artist_id: Uuid, trigger: &str) -> Result<()> {
        // Call the database function if it exists
        let result = sqlx::query("SELECT recalculate_trouble_score($1, $2)")
            .bind(artist_id)
            .bind(trigger)
            .execute(&self.db_pool)
            .await;

        match result {
            Ok(_) => {
                tracing::debug!(
                    artist_id = %artist_id,
                    trigger = %trigger,
                    "Triggered trouble score recalculation"
                );
            }
            Err(e) => {
                // Don't fail if function doesn't exist yet
                tracing::warn!(
                    artist_id = %artist_id,
                    error = %e,
                    "Could not trigger trouble score recalculation"
                );
            }
        }

        Ok(())
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
                "Processed article offenses"
            );
        }

        Ok(results)
    }

    /// Backfill: Process existing news_offense_classifications that haven't been converted
    pub async fn backfill_pending_classifications(&self, limit: i32) -> Result<i32> {
        // Find classifications that haven't been processed yet
        let pending = sqlx::query!(
            r#"
            SELECT
                noc.id,
                noc.article_id,
                noc.artist_id,
                noc.offense_category::text as "category!",
                noc.severity::text as "severity!",
                noc.confidence,
                noc.evidence_snippet,
                na.title as article_title,
                na.url as article_url,
                na.published_at
            FROM news_offense_classifications noc
            JOIN news_articles na ON noc.article_id = na.id
            WHERE noc.artist_id IS NOT NULL
              AND noc.confidence >= $1
              AND NOT EXISTS (
                SELECT 1 FROM artist_offenses ao
                WHERE ao.source_classification_id = noc.id
              )
            ORDER BY noc.created_at DESC
            LIMIT $2
            "#,
            CONFIDENCE_THRESHOLD as f32,
            limit as i64
        )
        .fetch_all(&self.db_pool)
        .await
        .context("Failed to fetch pending classifications")?;

        let mut processed = 0;

        for record in pending {
            // Convert to OffenseClassification
            let classification = OffenseClassification {
                id: record.id,
                article_id: record.article_id,
                entity_id: None,
                artist_id: record.artist_id,
                category: parse_category(&record.category),
                severity: parse_severity(&record.severity),
                confidence: record.confidence as f64,
                context: record.evidence_snippet.unwrap_or_default(),
                matched_keywords: vec![],
                needs_review: true, // Backfilled items always need review
            };

            let result = self
                .process_classification(
                    &classification,
                    record.article_id,
                    &record.article_title,
                    &record.article_url,
                    record.published_at,
                )
                .await;

            match result {
                Ok(r) if r.created => processed += 1,
                Ok(_) => {} // Skipped (duplicate or low confidence)
                Err(e) => {
                    tracing::error!(
                        classification_id = %record.id,
                        error = %e,
                        "Failed to backfill classification"
                    );
                }
            }
        }

        tracing::info!(processed = processed, "Backfill completed");
        Ok(processed)
    }
}

/// Parse offense category from string
fn parse_category(s: &str) -> super::processing::OffenseCategory {
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
        _ => OffenseCategory::Other,
    }
}

/// Parse offense severity from string
fn parse_severity(s: &str) -> super::processing::OffenseSeverity {
    use super::processing::OffenseSeverity;
    match s.to_lowercase().as_str() {
        "critical" | "egregious" => OffenseSeverity::Critical,
        "high" | "severe" => OffenseSeverity::High,
        "medium" | "moderate" => OffenseSeverity::Medium,
        "low" | "minor" => OffenseSeverity::Low,
        _ => OffenseSeverity::Low,
    }
}

/// Convert Rust OffenseSeverity to database severity string
fn severity_to_db(severity: &super::processing::OffenseSeverity) -> &'static str {
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
            parse_category("domestic_violence"),
            OffenseCategory::DomesticViolence
        ));
        assert!(matches!(
            parse_category("SEXUAL_MISCONDUCT"),
            OffenseCategory::SexualMisconduct
        ));
        assert!(matches!(parse_category("unknown"), OffenseCategory::Other));
    }

    #[test]
    fn test_parse_severity() {
        use super::super::processing::OffenseSeverity;
        assert!(matches!(
            parse_severity("egregious"),
            OffenseSeverity::Critical
        ));
        assert!(matches!(parse_severity("SEVERE"), OffenseSeverity::High));
        assert!(matches!(parse_severity("unknown"), OffenseSeverity::Low));
    }

    #[test]
    fn test_severity_to_db() {
        use super::super::processing::OffenseSeverity;
        assert_eq!(severity_to_db(&OffenseSeverity::Critical), "egregious");
        assert_eq!(severity_to_db(&OffenseSeverity::High), "severe");
        assert_eq!(severity_to_db(&OffenseSeverity::Medium), "moderate");
        assert_eq!(severity_to_db(&OffenseSeverity::Low), "minor");
    }
}
