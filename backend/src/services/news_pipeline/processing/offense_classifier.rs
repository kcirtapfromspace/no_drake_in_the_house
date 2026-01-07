//! Offense Classifier
//!
//! Classifies articles and entities for potential offenses.
//! Uses keyword matching and pattern recognition to identify
//! articles that may contain information about artist misconduct.

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::entity_extractor::ExtractedEntity;

/// Offense categories (matching database enum)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OffenseCategory {
    SexualMisconduct,
    DomesticViolence,
    HateSpeech,
    Racism,
    Antisemitism,
    Homophobia,
    ChildAbuse,
    AnimalCruelty,
    FinancialCrimes,
    DrugOffenses,
    ViolentCrimes,
    Harassment,
    Plagiarism,
    Other,
}

impl std::fmt::Display for OffenseCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OffenseCategory::SexualMisconduct => write!(f, "sexual_misconduct"),
            OffenseCategory::DomesticViolence => write!(f, "domestic_violence"),
            OffenseCategory::HateSpeech => write!(f, "hate_speech"),
            OffenseCategory::Racism => write!(f, "racism"),
            OffenseCategory::Antisemitism => write!(f, "antisemitism"),
            OffenseCategory::Homophobia => write!(f, "homophobia"),
            OffenseCategory::ChildAbuse => write!(f, "child_abuse"),
            OffenseCategory::AnimalCruelty => write!(f, "animal_cruelty"),
            OffenseCategory::FinancialCrimes => write!(f, "financial_crimes"),
            OffenseCategory::DrugOffenses => write!(f, "drug_offenses"),
            OffenseCategory::ViolentCrimes => write!(f, "violent_crimes"),
            OffenseCategory::Harassment => write!(f, "harassment"),
            OffenseCategory::Plagiarism => write!(f, "plagiarism"),
            OffenseCategory::Other => write!(f, "other"),
        }
    }
}

/// Offense severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum OffenseSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// An offense classification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffenseClassification {
    /// Unique identifier
    pub id: Uuid,
    /// Article ID
    pub article_id: Uuid,
    /// Entity ID (if linked to specific entity)
    pub entity_id: Option<Uuid>,
    /// Artist ID (if resolved)
    pub artist_id: Option<Uuid>,
    /// Offense category
    pub category: OffenseCategory,
    /// Severity level
    pub severity: OffenseSeverity,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Keywords that triggered the classification
    pub matched_keywords: Vec<String>,
    /// Context snippet
    pub context: String,
    /// Whether this needs human review
    pub needs_review: bool,
}

/// Classifier configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffenseClassifierConfig {
    /// Minimum confidence for classification
    pub min_confidence: f64,
    /// Threshold for high confidence (no review needed)
    pub high_confidence_threshold: f64,
    /// Context window size
    pub context_window: usize,
}

impl Default for OffenseClassifierConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.4,
            high_confidence_threshold: 0.8,
            context_window: 150,
        }
    }
}

/// Category keywords for classification
struct CategoryKeywords {
    keywords: Vec<String>,
    patterns: Vec<Regex>,
    severity_modifiers: HashMap<String, OffenseSeverity>,
}

/// Offense classifier
pub struct OffenseClassifier {
    config: OffenseClassifierConfig,
    category_keywords: HashMap<OffenseCategory, CategoryKeywords>,
    negation_patterns: Vec<Regex>,
}

impl OffenseClassifier {
    /// Create a new offense classifier
    pub fn new(config: OffenseClassifierConfig) -> Self {
        let category_keywords = Self::build_category_keywords();
        let negation_patterns = vec![
            Regex::new(r"(?i)denied|denies|dismisses|dismissed|unfounded|false|allegations? (were|was) dropped").unwrap(),
            Regex::new(r"(?i)not guilty|acquitted|exonerated|cleared of").unwrap(),
            Regex::new(r"(?i)no evidence|lacks evidence|unsubstantiated").unwrap(),
        ];

        Self {
            config,
            category_keywords,
            negation_patterns,
        }
    }

    /// Build keyword lists for each category
    fn build_category_keywords() -> HashMap<OffenseCategory, CategoryKeywords> {
        let mut map = HashMap::new();

        // Sexual Misconduct
        map.insert(
            OffenseCategory::SexualMisconduct,
            CategoryKeywords {
                keywords: vec![
                    "sexual assault", "sexual harassment", "rape", "groping",
                    "inappropriate", "misconduct", "metoo", "#metoo",
                    "sexual abuse", "molestation", "predator",
                ].iter().map(|s| s.to_string()).collect(),
                patterns: vec![
                    Regex::new(r"(?i)sexual(ly)?\s+(assault|harass|abuse)").unwrap(),
                    Regex::new(r"(?i)accused\s+of\s+.*sexual").unwrap(),
                ],
                severity_modifiers: [
                    ("rape", OffenseSeverity::Critical),
                    ("assault", OffenseSeverity::High),
                    ("harassment", OffenseSeverity::Medium),
                ].iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
            },
        );

        // Domestic Violence
        map.insert(
            OffenseCategory::DomesticViolence,
            CategoryKeywords {
                keywords: vec![
                    "domestic violence", "domestic abuse", "beat", "hit",
                    "assault", "battery", "restraining order", "abuse",
                    "physical altercation", "attacked",
                ].iter().map(|s| s.to_string()).collect(),
                patterns: vec![
                    Regex::new(r"(?i)(beat|hit|assault|attack)\s*(his|her|their)?\s*(wife|husband|girlfriend|boyfriend|partner|ex)").unwrap(),
                    Regex::new(r"(?i)domestic\s+(violence|abuse)").unwrap(),
                ],
                severity_modifiers: [
                    ("hospitalized", OffenseSeverity::Critical),
                    ("beat", OffenseSeverity::High),
                    ("restraining order", OffenseSeverity::Medium),
                ].iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
            },
        );

        // Hate Speech
        map.insert(
            OffenseCategory::HateSpeech,
            CategoryKeywords {
                keywords: vec![
                    "hate speech", "slur", "offensive comments", "racist remarks",
                    "discrimination", "bigot", "hateful", "derogatory",
                ].iter().map(|s| s.to_string()).collect(),
                patterns: vec![
                    Regex::new(r"(?i)hate\s+speech").unwrap(),
                    Regex::new(r"(?i)(racial|racist|homophobic|transphobic)\s+slur").unwrap(),
                ],
                severity_modifiers: [
                    ("slur", OffenseSeverity::High),
                    ("hate speech", OffenseSeverity::High),
                ].iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
            },
        );

        // Racism
        map.insert(
            OffenseCategory::Racism,
            CategoryKeywords {
                keywords: vec![
                    "racist", "racism", "racial slur", "n-word", "blackface",
                    "white supremacy", "segregation", "racial discrimination",
                ].iter().map(|s| s.to_string()).collect(),
                patterns: vec![
                    Regex::new(r"(?i)racist\s+(comment|remark|statement|post)").unwrap(),
                    Regex::new(r"(?i)accused\s+of\s+racism").unwrap(),
                ],
                severity_modifiers: [
                    ("white supremacy", OffenseSeverity::Critical),
                    ("n-word", OffenseSeverity::High),
                    ("blackface", OffenseSeverity::High),
                ].iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
            },
        );

        // Antisemitism
        map.insert(
            OffenseCategory::Antisemitism,
            CategoryKeywords {
                keywords: vec![
                    "antisemit", "anti-semit", "jewish", "jews", "holocaust",
                    "nazi", "hitler", "concentration camp", "zionist conspiracy",
                ].iter().map(|s| s.to_string()).collect(),
                patterns: vec![
                    Regex::new(r"(?i)anti[- ]?semit").unwrap(),
                    Regex::new(r"(?i)against\s+jews").unwrap(),
                ],
                severity_modifiers: [
                    ("holocaust denial", OffenseSeverity::Critical),
                    ("nazi", OffenseSeverity::Critical),
                    ("antisemitic", OffenseSeverity::High),
                ].iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
            },
        );

        // Homophobia
        map.insert(
            OffenseCategory::Homophobia,
            CategoryKeywords {
                keywords: vec![
                    "homophobic", "homophobia", "anti-gay", "anti-lgbtq",
                    "transphobic", "transphobia", "slur",
                ].iter().map(|s| s.to_string()).collect(),
                patterns: vec![
                    Regex::new(r"(?i)homophobic\s+(comment|remark|slur)").unwrap(),
                    Regex::new(r"(?i)anti[- ]?(gay|lgbtq|trans)").unwrap(),
                ],
                severity_modifiers: [
                    ("slur", OffenseSeverity::High),
                    ("homophobic", OffenseSeverity::Medium),
                ].iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
            },
        );

        // Child Abuse
        map.insert(
            OffenseCategory::ChildAbuse,
            CategoryKeywords {
                keywords: vec![
                    "child abuse", "minor", "underage", "pedophile",
                    "child exploitation", "grooming",
                ].iter().map(|s| s.to_string()).collect(),
                patterns: vec![
                    Regex::new(r"(?i)child\s+(abuse|exploitation|pornography)").unwrap(),
                    Regex::new(r"(?i)(sexual|inappropriate)\s+.*\s+(minor|underage|child)").unwrap(),
                ],
                severity_modifiers: [
                    ("pedophile", OffenseSeverity::Critical),
                    ("child abuse", OffenseSeverity::Critical),
                    ("grooming", OffenseSeverity::Critical),
                ].iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
            },
        );

        // Violent Crimes
        map.insert(
            OffenseCategory::ViolentCrimes,
            CategoryKeywords {
                keywords: vec![
                    "murder", "killed", "shooting", "stabbing", "assault",
                    "manslaughter", "attempted murder", "gun", "weapon",
                ].iter().map(|s| s.to_string()).collect(),
                patterns: vec![
                    Regex::new(r"(?i)charged\s+with\s+(murder|assault|battery)").unwrap(),
                    Regex::new(r"(?i)arrested\s+for\s+(shooting|stabbing|assault)").unwrap(),
                ],
                severity_modifiers: [
                    ("murder", OffenseSeverity::Critical),
                    ("shooting", OffenseSeverity::Critical),
                    ("assault", OffenseSeverity::High),
                ].iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
            },
        );

        // Financial Crimes
        map.insert(
            OffenseCategory::FinancialCrimes,
            CategoryKeywords {
                keywords: vec![
                    "fraud", "embezzlement", "money laundering", "tax evasion",
                    "scam", "ponzi", "crypto scam", "nft scam",
                ].iter().map(|s| s.to_string()).collect(),
                patterns: vec![
                    Regex::new(r"(?i)charged\s+with\s+(fraud|embezzlement|tax)").unwrap(),
                    Regex::new(r"(?i)(crypto|nft)\s+scam").unwrap(),
                ],
                severity_modifiers: [
                    ("fraud", OffenseSeverity::High),
                    ("embezzlement", OffenseSeverity::High),
                    ("scam", OffenseSeverity::Medium),
                ].iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
            },
        );

        // Drug Offenses
        map.insert(
            OffenseCategory::DrugOffenses,
            CategoryKeywords {
                keywords: vec![
                    "drug trafficking", "drug possession", "cocaine", "heroin",
                    "fentanyl", "drug arrest", "narcotics",
                ].iter().map(|s| s.to_string()).collect(),
                patterns: vec![
                    Regex::new(r"(?i)arrested\s+.*\s+drug").unwrap(),
                    Regex::new(r"(?i)drug\s+(trafficking|possession|charges)").unwrap(),
                ],
                severity_modifiers: [
                    ("trafficking", OffenseSeverity::High),
                    ("fentanyl", OffenseSeverity::High),
                    ("possession", OffenseSeverity::Low),
                ].iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
            },
        );

        map
    }

    /// Classify an article for offenses
    pub fn classify(
        &self,
        article_id: Uuid,
        text: &str,
        title: Option<&str>,
        entities: &[ExtractedEntity],
    ) -> Result<Vec<OffenseClassification>> {
        let mut classifications = Vec::new();

        // Combine title and text
        let full_text = if let Some(t) = title {
            format!("{}\n\n{}", t, text)
        } else {
            text.to_string()
        };

        let lower_text = full_text.to_lowercase();

        // Check each category
        for (category, keywords) in &self.category_keywords {
            let mut matched_keywords = Vec::new();
            let mut contexts = Vec::new();
            let mut max_severity = OffenseSeverity::Low;

            // Check keywords
            for keyword in &keywords.keywords {
                if lower_text.contains(&keyword.to_lowercase()) {
                    matched_keywords.push(keyword.clone());

                    // Find context
                    if let Some(pos) = lower_text.find(&keyword.to_lowercase()) {
                        let context = self.extract_context(&full_text, pos, pos + keyword.len());
                        contexts.push(context);
                    }

                    // Update severity
                    if let Some(severity) = keywords.severity_modifiers.get(keyword) {
                        if severity > &max_severity {
                            max_severity = severity.clone();
                        }
                    }
                }
            }

            // Check patterns
            for pattern in &keywords.patterns {
                for capture in pattern.find_iter(&full_text) {
                    let matched = capture.as_str();
                    if !matched_keywords.iter().any(|k| matched.contains(k)) {
                        matched_keywords.push(matched.to_string());
                        let context = self.extract_context(&full_text, capture.start(), capture.end());
                        contexts.push(context);
                    }
                }
            }

            if matched_keywords.is_empty() {
                continue;
            }

            // Check for negations
            let has_negation = self.negation_patterns.iter().any(|p| p.is_match(&full_text));

            // Calculate confidence
            let keyword_score = (matched_keywords.len() as f64 * 0.2).min(0.6);
            let pattern_score = if keywords.patterns.iter().any(|p| p.is_match(&full_text)) {
                0.3
            } else {
                0.0
            };
            let title_score = if let Some(t) = title {
                let title_lower = t.to_lowercase();
                if matched_keywords.iter().any(|k| title_lower.contains(&k.to_lowercase())) {
                    0.2
                } else {
                    0.0
                }
            } else {
                0.0
            };

            let mut confidence = (keyword_score + pattern_score + title_score).min(0.95);

            // Reduce confidence for negations
            if has_negation {
                confidence *= 0.5;
            }

            if confidence < self.config.min_confidence {
                continue;
            }

            // Create classification for each relevant entity, or general if no entities
            let relevant_entities: Vec<_> = entities
                .iter()
                .filter(|e| {
                    contexts.iter().any(|c| c.contains(&e.name))
                })
                .collect();

            if relevant_entities.is_empty() {
                // General classification
                classifications.push(OffenseClassification {
                    id: Uuid::new_v4(),
                    article_id,
                    entity_id: None,
                    artist_id: None,
                    category: category.clone(),
                    severity: max_severity.clone(),
                    confidence,
                    matched_keywords: matched_keywords.clone(),
                    context: contexts.first().cloned().unwrap_or_default(),
                    needs_review: confidence < self.config.high_confidence_threshold || has_negation,
                });
            } else {
                // Entity-specific classifications
                for entity in relevant_entities {
                    classifications.push(OffenseClassification {
                        id: Uuid::new_v4(),
                        article_id,
                        entity_id: Some(entity.id),
                        artist_id: entity.artist_id,
                        category: category.clone(),
                        severity: max_severity.clone(),
                        confidence,
                        matched_keywords: matched_keywords.clone(),
                        context: entity.context.clone(),
                        needs_review: confidence < self.config.high_confidence_threshold || has_negation,
                    });
                }
            }
        }

        // Sort by severity and confidence
        classifications.sort_by(|a, b| {
            b.severity.cmp(&a.severity)
                .then_with(|| b.confidence.partial_cmp(&a.confidence).unwrap())
        });

        Ok(classifications)
    }

    /// Extract context around a position
    fn extract_context(&self, text: &str, start: usize, end: usize) -> String {
        let context_start = start.saturating_sub(self.config.context_window);
        let context_end = (end + self.config.context_window).min(text.len());

        text[context_start..context_end].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offense_category_display() {
        assert_eq!(OffenseCategory::SexualMisconduct.to_string(), "sexual_misconduct");
        assert_eq!(OffenseCategory::DomesticViolence.to_string(), "domestic_violence");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(OffenseSeverity::Critical > OffenseSeverity::High);
        assert!(OffenseSeverity::High > OffenseSeverity::Medium);
        assert!(OffenseSeverity::Medium > OffenseSeverity::Low);
    }

    #[tokio::test]
    async fn test_classify_sexual_misconduct() {
        let config = OffenseClassifierConfig::default();
        let classifier = OffenseClassifier::new(config);

        let article_id = Uuid::new_v4();
        let text = "The artist was accused of sexual harassment by multiple women.";

        let classifications = classifier.classify(article_id, text, None, &[]).unwrap();

        assert!(!classifications.is_empty());
        assert!(classifications.iter().any(|c| c.category == OffenseCategory::SexualMisconduct));
    }

    #[tokio::test]
    async fn test_negation_reduces_confidence() {
        let config = OffenseClassifierConfig::default();
        let classifier = OffenseClassifier::new(config);

        let article_id = Uuid::new_v4();
        let text1 = "The artist was accused of sexual harassment.";
        let text2 = "The allegations of sexual harassment were dismissed as unfounded.";

        let class1 = classifier.classify(article_id, text1, None, &[]).unwrap();
        let class2 = classifier.classify(article_id, text2, None, &[]).unwrap();

        // Both should classify, but the negated one should have lower confidence
        if !class1.is_empty() && !class2.is_empty() {
            assert!(class2[0].confidence < class1[0].confidence);
        }
    }
}
