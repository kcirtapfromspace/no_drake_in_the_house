//! Offense Category Revenue Analytics
//!
//! Simulates and tracks streaming revenue distribution across offense categories.
//! Provides insights into how much listener money flows to artists with various
//! types of documented offenses.

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

/// Offense categories matching the artist_offenses table
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OffenseCategory {
    SexualMisconduct,
    DomesticViolence,
    ChildAbuse,
    HateSpeech,
    Racism,
    Antisemitism,
    Homophobia,
    ViolentCrime,
    DrugTrafficking,
    Fraud,
    AnimalAbuse,
    CertifiedCreeper,
    Other,
}

impl OffenseCategory {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().replace("-", "_").as_str() {
            "sexual_misconduct" => Self::SexualMisconduct,
            "domestic_violence" => Self::DomesticViolence,
            "child_abuse" => Self::ChildAbuse,
            "hate_speech" => Self::HateSpeech,
            "racism" => Self::Racism,
            "antisemitism" => Self::Antisemitism,
            "homophobia" => Self::Homophobia,
            "violent_crime" => Self::ViolentCrime,
            "drug_trafficking" => Self::DrugTrafficking,
            "fraud" => Self::Fraud,
            "animal_abuse" => Self::AnimalAbuse,
            "certified_creeper" => Self::CertifiedCreeper,
            _ => Self::Other,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SexualMisconduct => "sexual_misconduct",
            Self::DomesticViolence => "domestic_violence",
            Self::ChildAbuse => "child_abuse",
            Self::HateSpeech => "hate_speech",
            Self::Racism => "racism",
            Self::Antisemitism => "antisemitism",
            Self::Homophobia => "homophobia",
            Self::ViolentCrime => "violent_crime",
            Self::DrugTrafficking => "drug_trafficking",
            Self::Fraud => "fraud",
            Self::AnimalAbuse => "animal_abuse",
            Self::CertifiedCreeper => "certified_creeper",
            Self::Other => "other",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::SexualMisconduct => "Sexual Misconduct",
            Self::DomesticViolence => "Domestic Violence",
            Self::ChildAbuse => "Child Abuse",
            Self::HateSpeech => "Hate Speech",
            Self::Racism => "Racism",
            Self::Antisemitism => "Antisemitism",
            Self::Homophobia => "Homophobia",
            Self::ViolentCrime => "Violent Crime",
            Self::DrugTrafficking => "Drug Trafficking",
            Self::Fraud => "Fraud",
            Self::AnimalAbuse => "Animal Abuse",
            Self::CertifiedCreeper => "Certified Creeper",
            Self::Other => "Other",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::SexualMisconduct,
            Self::DomesticViolence,
            Self::ChildAbuse,
            Self::HateSpeech,
            Self::Racism,
            Self::Antisemitism,
            Self::Homophobia,
            Self::ViolentCrime,
            Self::DrugTrafficking,
            Self::Fraud,
            Self::AnimalAbuse,
            Self::CertifiedCreeper,
        ]
    }
}

/// Revenue breakdown for an offense category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRevenue {
    pub category: String,
    pub display_name: String,
    pub artist_count: i32,
    pub offense_count: i32,
    pub simulated_streams: i64,
    pub simulated_revenue: Decimal,
    pub percentage_of_total: f64,
    pub top_artists: Vec<CategoryArtistRevenue>,
}

/// Artist revenue within a category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryArtistRevenue {
    pub artist_id: Uuid,
    pub artist_name: String,
    pub offense_count: i32,
    pub severity: String,
    pub simulated_streams: i64,
    pub simulated_revenue: Decimal,
}

/// Global revenue distribution across categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalCategoryRevenue {
    pub period: String,
    pub total_simulated_revenue: Decimal,
    pub total_artists_with_offenses: i32,
    pub clean_artist_revenue: Decimal,
    pub problematic_artist_revenue: Decimal,
    pub problematic_percentage: f64,
    pub by_category: Vec<CategoryRevenue>,
    pub generated_at: DateTime<Utc>,
}

/// Artist discography with simulated revenue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistDiscographyRevenue {
    pub artist_id: Uuid,
    pub artist_name: String,
    pub offenses: Vec<ArtistOffenseSummary>,
    pub total_albums: i32,
    pub total_tracks: i32,
    pub simulated_monthly_streams: i64,
    pub simulated_monthly_revenue: Decimal,
    pub simulated_yearly_revenue: Decimal,
    pub albums: Vec<AlbumRevenue>,
}

/// Summary of an artist's offense
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistOffenseSummary {
    pub category: String,
    pub severity: String,
    pub title: String,
    pub date: Option<NaiveDate>,
}

/// Album revenue simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumRevenue {
    pub album_id: Option<Uuid>,
    pub title: String,
    pub release_year: Option<i32>,
    pub track_count: i32,
    pub simulated_monthly_streams: i64,
    pub simulated_monthly_revenue: Decimal,
}

/// Simulation parameters for revenue estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationParams {
    /// Base monthly streams for an average artist
    pub base_monthly_streams: i64,
    /// Multiplier for popular artists (based on offense notoriety)
    pub popularity_multiplier: f64,
    /// Average payout rate per stream (weighted across platforms)
    pub avg_payout_rate: Decimal,
    /// Album age decay factor (older albums get fewer streams)
    pub age_decay_factor: f64,
}

impl Default for SimulationParams {
    fn default() -> Self {
        Self {
            base_monthly_streams: 100_000,       // 100K base streams/month
            popularity_multiplier: 5.0,          // Up to 5x for notorious artists
            avg_payout_rate: Decimal::new(4, 3), // $0.004 per stream avg
            age_decay_factor: 0.9,               // 10% decay per year
        }
    }
}

/// Category revenue analytics service
pub struct CategoryRevenueService {
    pool: PgPool,
    params: SimulationParams,
}

impl CategoryRevenueService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            params: SimulationParams::default(),
        }
    }

    pub fn with_params(pool: PgPool, params: SimulationParams) -> Self {
        Self { pool, params }
    }

    /// Get global revenue distribution across offense categories
    pub async fn get_global_category_revenue(&self) -> Result<GlobalCategoryRevenue> {
        let mut categories = Vec::new();
        let mut total_problematic_revenue = Decimal::ZERO;

        // Get revenue for each offense category
        for category in OffenseCategory::all() {
            match self.get_category_revenue(category, 5).await {
                Ok(cat_revenue) => {
                    total_problematic_revenue += cat_revenue.simulated_revenue;
                    categories.push(cat_revenue);
                }
                Err(e) => {
                    tracing::warn!("Failed to get revenue for {}: {}", category.as_str(), e);
                }
            }
        }

        // Sort by revenue descending
        categories.sort_by(|a, b| b.simulated_revenue.cmp(&a.simulated_revenue));

        // Get clean artist count and simulate their revenue
        let clean_stats = self.get_clean_artist_stats().await?;
        let clean_revenue = self
            .simulate_clean_artist_revenue(clean_stats.artist_count)
            .await;

        let total_revenue = total_problematic_revenue + clean_revenue;
        let problematic_percentage = if total_revenue > Decimal::ZERO {
            (total_problematic_revenue / total_revenue * Decimal::from(100))
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        // Calculate percentages for each category
        for cat in &mut categories {
            if total_revenue > Decimal::ZERO {
                cat.percentage_of_total = (cat.simulated_revenue / total_revenue
                    * Decimal::from(100))
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0);
            }
        }

        let total_artists_with_offenses: i32 = categories.iter().map(|c| c.artist_count).sum();

        Ok(GlobalCategoryRevenue {
            period: "monthly_simulation".to_string(),
            total_simulated_revenue: total_revenue,
            total_artists_with_offenses,
            clean_artist_revenue: clean_revenue,
            problematic_artist_revenue: total_problematic_revenue,
            problematic_percentage,
            by_category: categories,
            generated_at: Utc::now(),
        })
    }

    /// Get revenue breakdown for a specific offense category
    pub async fn get_category_revenue(
        &self,
        category: OffenseCategory,
        top_n: i32,
    ) -> Result<CategoryRevenue> {
        // Get artists with this offense category
        let artists = sqlx::query_as::<_, CategoryArtistRow>(
            r#"
            SELECT
                a.id as artist_id,
                a.canonical_name as artist_name,
                COUNT(ao.id)::integer as offense_count,
                MAX(ao.severity::text) as max_severity
            FROM artists a
            JOIN artist_offenses ao ON ao.artist_id = a.id
            WHERE ao.category::text = $1
            GROUP BY a.id, a.canonical_name
            ORDER BY offense_count DESC
            LIMIT 100
            "#,
        )
        .bind(category.as_str())
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch category artists")?;

        let artist_count = artists.len() as i32;

        // Get total offense count for this category
        let offense_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM artist_offenses WHERE category::text = $1")
                .bind(category.as_str())
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

        // Simulate revenue for each artist
        let mut total_streams: i64 = 0;
        let mut total_revenue = Decimal::ZERO;
        let mut top_artists = Vec::new();

        for (idx, artist) in artists.iter().enumerate() {
            let popularity_boost =
                self.calculate_popularity_boost(artist.offense_count, &artist.max_severity);
            let artist_streams =
                (self.params.base_monthly_streams as f64 * popularity_boost) as i64;
            let artist_revenue = self.params.avg_payout_rate * Decimal::from(artist_streams);

            total_streams += artist_streams;
            total_revenue += artist_revenue;

            if idx < top_n as usize {
                top_artists.push(CategoryArtistRevenue {
                    artist_id: artist.artist_id,
                    artist_name: artist.artist_name.clone(),
                    offense_count: artist.offense_count,
                    severity: artist
                        .max_severity
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string()),
                    simulated_streams: artist_streams,
                    simulated_revenue: artist_revenue,
                });
            }
        }

        Ok(CategoryRevenue {
            category: category.as_str().to_string(),
            display_name: category.display_name().to_string(),
            artist_count,
            offense_count: offense_count as i32,
            simulated_streams: total_streams,
            simulated_revenue: total_revenue,
            percentage_of_total: 0.0, // Will be calculated by caller
            top_artists,
        })
    }

    /// Get artist discography with simulated revenue
    pub async fn get_artist_discography_revenue(
        &self,
        artist_id: Uuid,
    ) -> Result<ArtistDiscographyRevenue> {
        // Get artist info
        let artist_name: String =
            sqlx::query_scalar("SELECT canonical_name FROM artists WHERE id = $1")
                .bind(artist_id)
                .fetch_one(&self.pool)
                .await
                .context("Artist not found")?;

        // Get artist offenses
        let offenses = sqlx::query_as::<_, OffenseRow>(
            r#"
            SELECT
                category::text as category,
                severity::text as severity,
                title,
                incident_date as date
            FROM artist_offenses
            WHERE artist_id = $1
            ORDER BY incident_date DESC NULLS LAST
            "#,
        )
        .bind(artist_id)
        .fetch_all(&self.pool)
        .await?;

        let offense_summaries: Vec<ArtistOffenseSummary> = offenses
            .into_iter()
            .map(|o| ArtistOffenseSummary {
                category: o.category,
                severity: o.severity,
                title: o.title,
                date: o.date,
            })
            .collect();

        // Get albums (from catalog if available)
        let albums = sqlx::query_as::<_, AlbumRow>(
            r#"
            SELECT
                al.id as album_id,
                al.title,
                EXTRACT(YEAR FROM al.release_date)::integer as release_year,
                al.total_tracks as track_count
            FROM albums al
            WHERE al.artist_id = $1
            ORDER BY al.release_date DESC NULLS LAST
            "#,
        )
        .bind(artist_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        // Calculate popularity boost based on offenses
        let max_severity = offense_summaries
            .iter()
            .map(|o| &o.severity)
            .max()
            .map(|s| s.as_str())
            .unwrap_or("unknown");
        let popularity_boost = self.calculate_popularity_boost(
            offense_summaries.len() as i32,
            &Some(max_severity.to_string()),
        );

        // Simulate album revenues
        let current_year = Utc::now().year();
        let mut album_revenues = Vec::new();
        let mut total_tracks = 0i32;

        if albums.is_empty() {
            // Create simulated discography
            album_revenues = self.simulate_discography(artist_id, popularity_boost).await;
            total_tracks = album_revenues.iter().map(|a| a.track_count).sum();
        } else {
            for album in &albums {
                let age_years = album
                    .release_year
                    .map(|y| (current_year - y).max(0))
                    .unwrap_or(5) as f64;
                let age_factor = self.params.age_decay_factor.powf(age_years);
                let track_count = album.track_count.unwrap_or(10);

                let album_streams = (self.params.base_monthly_streams as f64
                    * popularity_boost
                    * age_factor
                    * (track_count as f64 / 12.0)) as i64;
                let album_revenue = self.params.avg_payout_rate * Decimal::from(album_streams);

                total_tracks += track_count;
                album_revenues.push(AlbumRevenue {
                    album_id: album.album_id,
                    title: album.title.clone(),
                    release_year: album.release_year,
                    track_count,
                    simulated_monthly_streams: album_streams,
                    simulated_monthly_revenue: album_revenue,
                });
            }
        }

        let total_monthly_streams: i64 = album_revenues
            .iter()
            .map(|a| a.simulated_monthly_streams)
            .sum();
        let total_monthly_revenue: Decimal = album_revenues
            .iter()
            .map(|a| a.simulated_monthly_revenue)
            .sum();
        let total_yearly_revenue = total_monthly_revenue * Decimal::from(12);

        Ok(ArtistDiscographyRevenue {
            artist_id,
            artist_name,
            offenses: offense_summaries,
            total_albums: album_revenues.len() as i32,
            total_tracks,
            simulated_monthly_streams: total_monthly_streams,
            simulated_monthly_revenue: total_monthly_revenue,
            simulated_yearly_revenue: total_yearly_revenue,
            albums: album_revenues,
        })
    }

    /// Get revenue by category for a specific user's listening
    pub async fn get_user_category_exposure(
        &self,
        user_id: Uuid,
        days: i32,
    ) -> Result<Vec<CategoryRevenue>> {
        let rows = sqlx::query_as::<_, UserCategoryRow>(
            r#"
            SELECT
                ao.category::text as category,
                COUNT(DISTINCT ao.artist_id)::integer as artist_count,
                COUNT(ao.id)::integer as offense_count,
                COALESCE(SUM(pc.play_count), 0)::bigint as total_streams,
                COALESCE(SUM(pc.estimated_revenue), 0) as total_revenue
            FROM artist_offenses ao
            JOIN user_artist_playcounts pc ON pc.artist_id = ao.artist_id
            WHERE pc.user_id = $1
            AND pc.period_start >= CURRENT_DATE - $2::integer
            GROUP BY ao.category
            ORDER BY SUM(pc.estimated_revenue) DESC
            "#,
        )
        .bind(user_id)
        .bind(days)
        .fetch_all(&self.pool)
        .await?;

        let total_revenue: Decimal = rows.iter().map(|r| r.total_revenue).sum();

        Ok(rows
            .into_iter()
            .map(|r| {
                let category = OffenseCategory::from_str(&r.category);
                let percentage = if total_revenue > Decimal::ZERO {
                    (r.total_revenue / total_revenue * Decimal::from(100))
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0)
                } else {
                    0.0
                };

                CategoryRevenue {
                    category: r.category,
                    display_name: category.display_name().to_string(),
                    artist_count: r.artist_count,
                    offense_count: r.offense_count,
                    simulated_streams: r.total_streams,
                    simulated_revenue: r.total_revenue,
                    percentage_of_total: percentage,
                    top_artists: vec![], // Could be populated with additional query
                }
            })
            .collect())
    }

    // Helper methods

    fn calculate_popularity_boost(&self, offense_count: i32, severity: &Option<String>) -> f64 {
        let base_boost = 1.0;
        let offense_boost = (offense_count as f64 * 0.2).min(1.0); // Up to 1.0 extra

        let severity_boost = match severity.as_deref() {
            Some("egregious") => 2.0,
            Some("severe") => 1.5,
            Some("moderate") => 1.0,
            Some("minor") => 0.5,
            _ => 0.5,
        };

        (base_boost + offense_boost + severity_boost).min(self.params.popularity_multiplier)
    }

    async fn get_clean_artist_stats(&self) -> Result<CleanArtistStats> {
        let row = sqlx::query_as::<_, CleanStatsRow>(
            r#"
            SELECT
                COUNT(DISTINCT a.id)::integer as artist_count
            FROM artists a
            LEFT JOIN artist_offenses ao ON ao.artist_id = a.id
            WHERE ao.id IS NULL
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(CleanArtistStats {
            artist_count: row.artist_count,
        })
    }

    async fn simulate_clean_artist_revenue(&self, artist_count: i32) -> Decimal {
        // Clean artists get base streams without notoriety boost
        let streams_per_artist = self.params.base_monthly_streams;
        let total_streams = streams_per_artist * artist_count as i64;
        self.params.avg_payout_rate * Decimal::from(total_streams)
    }

    async fn simulate_discography(
        &self,
        _artist_id: Uuid,
        popularity_boost: f64,
    ) -> Vec<AlbumRevenue> {
        // Generate realistic simulated discography
        let album_count = 3 + (popularity_boost * 2.0) as i32;
        let current_year = Utc::now().year();

        (0..album_count.min(10))
            .map(|i| {
                let release_year = current_year - i - 1;
                let age_factor = self.params.age_decay_factor.powf(i as f64);
                let track_count = 10 + (i % 5);

                let album_streams = (self.params.base_monthly_streams as f64
                    * popularity_boost
                    * age_factor
                    * (track_count as f64 / 12.0)) as i64;
                let album_revenue = self.params.avg_payout_rate * Decimal::from(album_streams);

                AlbumRevenue {
                    album_id: None,
                    title: format!("Album {} (Simulated)", i + 1),
                    release_year: Some(release_year),
                    track_count,
                    simulated_monthly_streams: album_streams,
                    simulated_monthly_revenue: album_revenue,
                }
            })
            .collect()
    }
}

// Internal row types

#[derive(Debug, sqlx::FromRow)]
struct CategoryArtistRow {
    artist_id: Uuid,
    artist_name: String,
    offense_count: i32,
    max_severity: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct OffenseRow {
    category: String,
    severity: String,
    title: String,
    date: Option<NaiveDate>,
}

#[derive(Debug, sqlx::FromRow)]
struct AlbumRow {
    album_id: Option<Uuid>,
    title: String,
    release_year: Option<i32>,
    track_count: Option<i32>,
}

#[derive(Debug, sqlx::FromRow)]
struct UserCategoryRow {
    category: String,
    artist_count: i32,
    offense_count: i32,
    total_streams: i64,
    total_revenue: Decimal,
}

#[derive(Debug, sqlx::FromRow)]
struct CleanStatsRow {
    artist_count: i32,
}

struct CleanArtistStats {
    artist_count: i32,
}

trait YearExt {
    fn year(&self) -> i32;
}

impl YearExt for DateTime<Utc> {
    fn year(&self) -> i32 {
        chrono::Datelike::year(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offense_category_from_str() {
        assert_eq!(
            OffenseCategory::from_str("sexual_misconduct"),
            OffenseCategory::SexualMisconduct
        );
        assert_eq!(
            OffenseCategory::from_str("DOMESTIC_VIOLENCE"),
            OffenseCategory::DomesticViolence
        );
        assert_eq!(
            OffenseCategory::from_str("unknown_category"),
            OffenseCategory::Other
        );
    }

    #[test]
    fn test_simulation_params_default() {
        let params = SimulationParams::default();
        assert_eq!(params.base_monthly_streams, 100_000);
        assert_eq!(params.popularity_multiplier, 5.0);
    }
}
