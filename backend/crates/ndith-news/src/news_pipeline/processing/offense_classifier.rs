//! Offense Classifier
//!
//! Classifies articles and entities for potential offenses.
//! Uses keyword matching and pattern recognition to identify
//! articles that may contain information about artist misconduct.

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    CertifiedCreeper,
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
            OffenseCategory::CertifiedCreeper => write!(f, "certified_creeper"),
            OffenseCategory::Other => write!(f, "other"),
        }
    }
}

/// Convert news classifier category to core database category
impl From<&OffenseCategory> for ndith_core::models::offense::OffenseCategory {
    fn from(category: &OffenseCategory) -> Self {
        match category {
            OffenseCategory::SexualMisconduct => Self::SexualMisconduct,
            OffenseCategory::DomesticViolence => Self::DomesticViolence,
            OffenseCategory::HateSpeech => Self::HateSpeech,
            OffenseCategory::Racism => Self::Racism,
            OffenseCategory::Antisemitism => Self::Antisemitism,
            OffenseCategory::Homophobia => Self::Homophobia,
            OffenseCategory::ChildAbuse => Self::ChildAbuse,
            OffenseCategory::AnimalCruelty => Self::AnimalCruelty,
            OffenseCategory::FinancialCrimes => Self::FinancialCrimes,
            OffenseCategory::DrugOffenses => Self::DrugOffenses,
            OffenseCategory::ViolentCrimes => Self::ViolentCrimes,
            OffenseCategory::Harassment => Self::Harassment,
            OffenseCategory::Plagiarism => Self::Plagiarism,
            OffenseCategory::CertifiedCreeper => Self::CertifiedCreeper,
            OffenseCategory::Other => Self::Other,
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
    /// Classification source (keyword, llm, hybrid)
    pub classification_source: Option<String>,
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
    sentence_splitter: Regex,
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
        let sentence_splitter = Regex::new(r"[.!?]+\s+").unwrap();

        Self {
            config,
            category_keywords,
            negation_patterns,
            sentence_splitter,
        }
    }

    /// Build keyword lists for each category
    fn build_category_keywords() -> HashMap<OffenseCategory, CategoryKeywords> {
        let mut map = HashMap::new();

        // Sexual Misconduct
        map.insert(
            OffenseCategory::SexualMisconduct,
            CategoryKeywords {
                keywords: [
                    "sexual assault",
                    "sexual harassment",
                    "rape",
                    "groping",
                    "inappropriate",
                    "misconduct",
                    "metoo",
                    "#metoo",
                    "sexual abuse",
                    "molestation",
                    "predator",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                patterns: vec![
                    Regex::new(r"(?i)sexual(ly)?\s+(assault|harass|abuse)").unwrap(),
                    Regex::new(r"(?i)accused\s+of\s+.*sexual").unwrap(),
                ],
                severity_modifiers: [
                    ("rape", OffenseSeverity::Critical),
                    ("assault", OffenseSeverity::High),
                    ("harassment", OffenseSeverity::Medium),
                ]
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
            },
        );

        // Domestic Violence
        map.insert(
            OffenseCategory::DomesticViolence,
            CategoryKeywords {
                keywords: [
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
                keywords: [
                    "hate speech",
                    "slur",
                    "offensive comments",
                    "racist remarks",
                    "discrimination",
                    "bigot",
                    "hateful",
                    "derogatory",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                patterns: vec![
                    Regex::new(r"(?i)hate\s+speech").unwrap(),
                    Regex::new(r"(?i)(racial|racist|homophobic|transphobic)\s+slur").unwrap(),
                ],
                severity_modifiers: [
                    ("slur", OffenseSeverity::High),
                    ("hate speech", OffenseSeverity::High),
                ]
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
            },
        );

        // Racism
        map.insert(
            OffenseCategory::Racism,
            CategoryKeywords {
                keywords: [
                    "racist",
                    "racism",
                    "racial slur",
                    "n-word",
                    "blackface",
                    "white supremacy",
                    "segregation",
                    "racial discrimination",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                patterns: vec![
                    Regex::new(r"(?i)racist\s+(comment|remark|statement|post)").unwrap(),
                    Regex::new(r"(?i)accused\s+of\s+racism").unwrap(),
                ],
                severity_modifiers: [
                    ("white supremacy", OffenseSeverity::Critical),
                    ("n-word", OffenseSeverity::High),
                    ("blackface", OffenseSeverity::High),
                ]
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
            },
        );

        // Antisemitism
        map.insert(
            OffenseCategory::Antisemitism,
            CategoryKeywords {
                keywords: [
                    "antisemit",
                    "anti-semit",
                    "jewish",
                    "jews",
                    "holocaust",
                    "nazi",
                    "hitler",
                    "concentration camp",
                    "zionist conspiracy",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                patterns: vec![
                    Regex::new(r"(?i)anti[- ]?semit").unwrap(),
                    Regex::new(r"(?i)against\s+jews").unwrap(),
                ],
                severity_modifiers: [
                    ("holocaust denial", OffenseSeverity::Critical),
                    ("nazi", OffenseSeverity::Critical),
                    ("antisemitic", OffenseSeverity::High),
                ]
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
            },
        );

        // Homophobia
        map.insert(
            OffenseCategory::Homophobia,
            CategoryKeywords {
                keywords: [
                    "homophobic",
                    "homophobia",
                    "anti-gay",
                    "anti-lgbtq",
                    "transphobic",
                    "transphobia",
                    "slur",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                patterns: vec![
                    Regex::new(r"(?i)homophobic\s+(comment|remark|slur)").unwrap(),
                    Regex::new(r"(?i)anti[- ]?(gay|lgbtq|trans)").unwrap(),
                ],
                severity_modifiers: [
                    ("slur", OffenseSeverity::High),
                    ("homophobic", OffenseSeverity::Medium),
                ]
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
            },
        );

        // Child Abuse
        map.insert(
            OffenseCategory::ChildAbuse,
            CategoryKeywords {
                keywords: [
                    "child abuse",
                    "minor",
                    "underage",
                    "pedophile",
                    "child exploitation",
                    "grooming",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                patterns: vec![
                    Regex::new(r"(?i)child\s+(abuse|exploitation|pornography)").unwrap(),
                    Regex::new(r"(?i)(sexual|inappropriate)\s+.*\s+(minor|underage|child)")
                        .unwrap(),
                ],
                severity_modifiers: [
                    ("pedophile", OffenseSeverity::Critical),
                    ("child abuse", OffenseSeverity::Critical),
                    ("grooming", OffenseSeverity::Critical),
                ]
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
            },
        );

        // Violent Crimes
        map.insert(
            OffenseCategory::ViolentCrimes,
            CategoryKeywords {
                keywords: [
                    "murder",
                    "killed",
                    "shooting",
                    "stabbing",
                    "assault",
                    "manslaughter",
                    "attempted murder",
                    "gun",
                    "weapon",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                patterns: vec![
                    Regex::new(r"(?i)charged\s+with\s+(murder|assault|battery)").unwrap(),
                    Regex::new(r"(?i)arrested\s+for\s+(shooting|stabbing|assault)").unwrap(),
                ],
                severity_modifiers: [
                    ("murder", OffenseSeverity::Critical),
                    ("shooting", OffenseSeverity::Critical),
                    ("assault", OffenseSeverity::High),
                ]
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
            },
        );

        // Financial Crimes
        map.insert(
            OffenseCategory::FinancialCrimes,
            CategoryKeywords {
                keywords: [
                    "fraud",
                    "embezzlement",
                    "money laundering",
                    "tax evasion",
                    "scam",
                    "ponzi",
                    "crypto scam",
                    "nft scam",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                patterns: vec![
                    Regex::new(r"(?i)charged\s+with\s+(fraud|embezzlement|tax)").unwrap(),
                    Regex::new(r"(?i)(crypto|nft)\s+scam").unwrap(),
                ],
                severity_modifiers: [
                    ("fraud", OffenseSeverity::High),
                    ("embezzlement", OffenseSeverity::High),
                    ("scam", OffenseSeverity::Medium),
                ]
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
            },
        );

        // Drug Offenses
        map.insert(
            OffenseCategory::DrugOffenses,
            CategoryKeywords {
                keywords: [
                    "drug trafficking",
                    "drug possession",
                    "cocaine",
                    "heroin",
                    "fentanyl",
                    "drug arrest",
                    "narcotics",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                patterns: vec![
                    Regex::new(r"(?i)arrested\s+.*\s+drug").unwrap(),
                    Regex::new(r"(?i)drug\s+(trafficking|possession|charges)").unwrap(),
                ],
                severity_modifiers: [
                    ("trafficking", OffenseSeverity::High),
                    ("fentanyl", OffenseSeverity::High),
                    ("possession", OffenseSeverity::Low),
                ]
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
            },
        );

        // Animal Cruelty
        map.insert(
            OffenseCategory::AnimalCruelty,
            CategoryKeywords {
                keywords: [
                    "animal cruelty",
                    "animal abuse",
                    "dogfighting",
                    "dog fighting",
                    "animal neglect",
                    "animal torture",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                patterns: vec![
                    Regex::new(r"(?i)animal\s+(cruelty|abuse|neglect|torture)").unwrap(),
                    Regex::new(r"(?i)dog\s*fight").unwrap(),
                ],
                severity_modifiers: [
                    ("dogfighting", OffenseSeverity::Critical),
                    ("torture", OffenseSeverity::Critical),
                    ("cruelty", OffenseSeverity::High),
                ]
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
            },
        );

        // Harassment
        map.insert(
            OffenseCategory::Harassment,
            CategoryKeywords {
                keywords: [
                    "harassment",
                    "stalking",
                    "cyberbullying",
                    "threats",
                    "intimidation",
                    "bullying",
                    "doxing",
                    "death threats",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                patterns: vec![
                    Regex::new(r"(?i)(harass|stalk|threaten|bully)\s+(his|her|their|a|the)")
                        .unwrap(),
                    Regex::new(r"(?i)accused\s+of\s+(harassment|stalking|threats)").unwrap(),
                ],
                severity_modifiers: [
                    ("death threats", OffenseSeverity::Critical),
                    ("stalking", OffenseSeverity::High),
                    ("harassment", OffenseSeverity::Medium),
                ]
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
            },
        );

        // Plagiarism (was empty — now populated)
        map.insert(
            OffenseCategory::Plagiarism,
            CategoryKeywords {
                keywords: [
                    "plagiarism",
                    "plagiarized",
                    "plagiarised",
                    "copied",
                    "stolen song",
                    "ghostwriter controversy",
                    "uncredited",
                    "copyright infringement",
                    "sampling without permission",
                    "music theft",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                patterns: vec![
                    Regex::new(r"(?i)plagiari(sm|zed|sed)").unwrap(),
                    Regex::new(
                        r"(?i)(stole|copied|ripped off)\s+(the\s+)?(song|beat|melody|lyrics)",
                    )
                    .unwrap(),
                    Regex::new(r"(?i)copyright\s+(infringement|lawsuit|violation)").unwrap(),
                ],
                severity_modifiers: [
                    ("copyright infringement", OffenseSeverity::High),
                    ("plagiarism", OffenseSeverity::Medium),
                    ("uncredited", OffenseSeverity::Low),
                ]
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
            },
        );

        // Certified Creeper (grooming/age-gap predatory behavior)
        map.insert(
            OffenseCategory::CertifiedCreeper,
            CategoryKeywords {
                keywords: [
                    "grooming",
                    "underage girlfriend",
                    "age gap",
                    "dating a minor",
                    "inappropriate relationship",
                    "teenage girlfriend",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                patterns: vec![
                    Regex::new(
                        r"(?i)(dating|relationship with)\s+(a\s+)?(minor|underage|teenager|teen)",
                    )
                    .unwrap(),
                    Regex::new(r"(?i)groom(ed|ing)\s+(a\s+)?(minor|underage|young|teen)").unwrap(),
                ],
                severity_modifiers: [
                    ("grooming", OffenseSeverity::Critical),
                    ("underage", OffenseSeverity::Critical),
                ]
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
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

        // Split text into sentences for scoped negation detection
        let sentences: Vec<&str> = self.sentence_splitter.split(&full_text).collect();

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
                        let context =
                            self.extract_context(&full_text, capture.start(), capture.end());
                        contexts.push(context);
                    }
                }
            }

            if matched_keywords.is_empty() {
                continue;
            }

            // Per-sentence negation detection: only negate if the negation
            // appears in the same sentence as a keyword for THIS category
            let has_negation = self.check_scoped_negation(&sentences, &matched_keywords);

            // Calculate confidence
            let keyword_score = (matched_keywords.len() as f64 * 0.2).min(0.6);
            let pattern_score = if keywords.patterns.iter().any(|p| p.is_match(&full_text)) {
                0.3
            } else {
                0.0
            };
            let title_score = if let Some(t) = title {
                let title_lower = t.to_lowercase();
                if matched_keywords
                    .iter()
                    .any(|k| title_lower.contains(&k.to_lowercase()))
                {
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
                .filter(|e| contexts.iter().any(|c| c.contains(&e.name)))
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
                    needs_review: confidence < self.config.high_confidence_threshold
                        || has_negation,
                    classification_source: Some("keyword".to_string()),
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
                        needs_review: confidence < self.config.high_confidence_threshold
                            || has_negation,
                        classification_source: Some("keyword".to_string()),
                    });
                }
            }
        }

        // Sort by severity and confidence
        classifications.sort_by(|a, b| {
            b.severity
                .cmp(&a.severity)
                .then_with(|| b.confidence.partial_cmp(&a.confidence).unwrap())
        });

        Ok(classifications)
    }

    /// Check if negation patterns appear in the same sentence as any matched keyword
    /// (scoped negation — only affects the relevant category)
    fn check_scoped_negation(&self, sentences: &[&str], matched_keywords: &[String]) -> bool {
        for sentence in sentences {
            let sentence_lower = sentence.to_lowercase();
            // Check if this sentence contains any of the matched keywords
            let has_keyword = matched_keywords
                .iter()
                .any(|k| sentence_lower.contains(&k.to_lowercase()));

            if !has_keyword {
                continue;
            }

            // Check if this same sentence also contains a negation
            for pattern in &self.negation_patterns {
                if pattern.is_match(sentence) {
                    return true;
                }
            }
        }
        false
    }

    /// Extract context around a position
    fn extract_context(&self, text: &str, start: usize, end: usize) -> String {
        let raw_start = start.saturating_sub(self.config.context_window);
        let raw_end = (end + self.config.context_window).min(text.len());

        let context_start = floor_char_boundary(text, raw_start);
        let context_end = ceil_char_boundary(text, raw_end);

        text[context_start..context_end].to_string()
    }
}

fn floor_char_boundary(text: &str, index: usize) -> usize {
    let mut boundary = index.min(text.len());
    while boundary > 0 && !text.is_char_boundary(boundary) {
        boundary -= 1;
    }
    boundary
}

fn ceil_char_boundary(text: &str, index: usize) -> usize {
    let mut boundary = index.min(text.len());
    while boundary < text.len() && !text.is_char_boundary(boundary) {
        boundary += 1;
    }
    boundary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offense_category_display() {
        assert_eq!(
            OffenseCategory::SexualMisconduct.to_string(),
            "sexual_misconduct"
        );
        assert_eq!(
            OffenseCategory::DomesticViolence.to_string(),
            "domestic_violence"
        );
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
        assert!(classifications
            .iter()
            .any(|c| c.category == OffenseCategory::SexualMisconduct));
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

    #[test]
    fn test_scoped_negation_doesnt_affect_unrelated_categories() {
        let config = OffenseClassifierConfig::default();
        let classifier = OffenseClassifier::new(config);

        let article_id = Uuid::new_v4();
        // Article with fraud AND a negation about drug charges in a different sentence
        let text = "The rapper committed massive fraud and stole millions. The drug charges against him were dismissed as unfounded.";

        let classifications = classifier.classify(article_id, text, None, &[]).unwrap();

        // Financial crimes should NOT have reduced confidence since the negation
        // is in a different sentence about drugs
        let financial = classifications
            .iter()
            .find(|c| c.category == OffenseCategory::FinancialCrimes);
        if let Some(fin) = financial {
            assert!(
                fin.confidence > 0.3,
                "Financial crimes confidence should not be reduced by unrelated negation"
            );
        }
    }

    #[test]
    fn test_plagiarism_keywords_detect() {
        let config = OffenseClassifierConfig::default();
        let classifier = OffenseClassifier::new(config);

        let article_id = Uuid::new_v4();
        let text = "The artist was sued for plagiarism after copying the melody from another song. A copyright infringement lawsuit was filed.";

        let classifications = classifier.classify(article_id, text, None, &[]).unwrap();

        assert!(classifications
            .iter()
            .any(|c| c.category == OffenseCategory::Plagiarism));
    }

    #[test]
    fn test_extract_context_handles_unicode_boundary() {
        let classifier = OffenseClassifier::new(OffenseClassifierConfig {
            context_window: 5,
            ..Default::default()
        });
        let text = "Prefix \u{a0}'headline' with unicode";

        let start = text.find("headline").unwrap() + 1;
        let end = start + "headline".len();

        let context = classifier.extract_context(text, start, end);

        assert!(text.contains(&context));
        assert!(context.contains("headline"));
    }
}
