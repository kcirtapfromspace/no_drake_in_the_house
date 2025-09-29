use music_streaming_blocklist_backend::models::*;
use fake::{Fake, Faker};
use serde_json::json;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Test data factory for creating consistent test fixtures
pub struct TestFixtures;

impl TestFixtures {
    /// Create a test user with realistic data
    pub fn create_user() -> User {
        User {
            id: Uuid::new_v4(),
            email: format!("test_{}@example.com", Uuid::new_v4()),
            password_hash: Some("$2b$12$test_hash_for_testing_purposes".to_string()),
            totp_secret: None,
            totp_enabled: false,
            email_verified: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            settings: None,
        }
    }
    
    /// Create a test user with specific email
    pub fn create_user_with_email(email: &str) -> User {
        User {
            id: Uuid::new_v4(),
            email: email.to_string(),
            password_hash: Some("$2b$12$test_hash_for_testing_purposes".to_string()),
            totp_secret: None,
            totp_enabled: false,
            email_verified: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            settings: None,
        }
    }
    
    /// Create a test user with 2FA enabled
    pub fn create_user_with_2fa() -> User {
        User {
            id: Uuid::new_v4(),
            email: format!("2fa_test_{}@example.com", Uuid::new_v4()),
            password_hash: Some("$2b$12$test_hash_for_testing_purposes".to_string()),
            totp_secret: Some("JBSWY3DPEHPK3PXP".to_string()), // Base32 encoded test secret
            totp_enabled: true,
            email_verified: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            settings: None,
        }
    }
    
    /// Create a test artist with realistic data
    pub fn create_artist() -> Artist {
        let artist_name: String = fake::faker::name::en::Name().fake();
        
        Artist {
            id: Uuid::new_v4(),
            canonical_name: artist_name,
            external_ids: json!({
                "spotify": format!("spotify_{}", Uuid::new_v4()),
                "apple_music": format!("apple_{}", Uuid::new_v4()),
                "youtube_music": format!("youtube_{}", Uuid::new_v4())
            }),
            metadata: json!({
                "genres": ["rock", "pop", "alternative"],
                "image": "https://example.com/artist-image.jpg",
                "popularity": fake::faker::number::en::NumberWithinRange(0..100).fake::<u8>(),
                "followers": fake::faker::number::en::NumberWithinRange(1000..1000000).fake::<u32>()
            }),
            created_at: Utc::now(),
        }
    }
    
    /// Create a test artist with specific name
    pub fn create_artist_with_name(name: &str) -> Artist {
        Artist {
            id: Uuid::new_v4(),
            canonical_name: name.to_string(),
            external_ids: json!({
                "spotify": format!("spotify_{}", name.replace(" ", "_").to_lowercase()),
                "apple_music": format!("apple_{}", name.replace(" ", "_").to_lowercase())
            }),
            metadata: json!({
                "genres": ["test"],
                "image": "https://example.com/test-artist.jpg"
            }),
            created_at: Utc::now(),
        }
    }
    
    /// Create a DNP entry
    pub fn create_dnp_entry(user_id: Uuid, artist_id: Uuid) -> DnpEntry {
        DnpEntry {
            user_id,
            artist_id,
            tags: Some(vec!["test".to_string(), "rock".to_string()]),
            note: Some("Test DNP entry note".to_string()),
            created_at: Utc::now(),
        }
    }
    
    /// Create a DNP entry with specific tags and note
    pub fn create_dnp_entry_with_details(
        user_id: Uuid, 
        artist_id: Uuid, 
        tags: Vec<String>, 
        note: Option<String>
    ) -> DnpEntry {
        DnpEntry {
            user_id,
            artist_id,
            tags: if tags.is_empty() { None } else { Some(tags) },
            note,
            created_at: Utc::now(),
        }
    }
    
    /// Create a user registration request
    pub fn create_register_request() -> CreateUserRequest {
        CreateUserRequest {
            email: format!("register_test_{}@example.com", Uuid::new_v4()),
            password: "SecureTestPassword123!".to_string(),
        }
    }
    
    /// Create a login request
    pub fn create_login_request(email: &str) -> LoginRequest {
        LoginRequest {
            email: email.to_string(),
            password: "SecureTestPassword123!".to_string(),
            totp_code: None,
        }
    }
    
    /// Create a login request with 2FA code
    pub fn create_login_request_with_2fa(email: &str, totp_code: &str) -> LoginRequest {
        LoginRequest {
            email: email.to_string(),
            password: "SecureTestPassword123!".to_string(),
            totp_code: Some(totp_code.to_string()),
        }
    }
    
    /// Create an add to DNP request
    pub fn create_add_to_dnp_request(artist_id: Uuid) -> AddToDnpRequest {
        AddToDnpRequest {
            artist_id,
            tags: Some(vec!["test".to_string(), "annoying".to_string()]),
            note: Some("Test note for DNP entry".to_string()),
        }
    }
    
    /// Create user settings
    pub fn create_user_settings() -> UserSettings {
        UserSettings {
            theme: Some("dark".to_string()),
            notifications_enabled: Some(true),
            auto_enforcement: Some(false),
            preferred_platforms: Some(vec!["spotify".to_string(), "apple_music".to_string()]),
            privacy_settings: Some(json!({
                "profile_visibility": "private",
                "share_dnp_stats": false,
                "allow_community_suggestions": true
            })),
        }
    }
    
    /// Create an audit log entry
    pub fn create_audit_entry(user_id: Uuid, action: &str) -> AuditLogEntry {
        AuditLogEntry {
            id: Uuid::new_v4(),
            user_id: Some(user_id),
            action: action.to_string(),
            resource_type: "user".to_string(),
            resource_id: Some(user_id.to_string()),
            ip_address: Some(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
            user_agent: Some("Test User Agent".to_string()),
            created_at: Utc::now(),
        }
    }
}

/// Batch fixture creation for performance testing
pub struct BatchFixtures;

impl BatchFixtures {
    /// Create multiple users for testing
    pub fn create_users(count: usize) -> Vec<User> {
        (0..count).map(|_| TestFixtures::create_user()).collect()
    }
    
    /// Create multiple artists for testing
    pub fn create_artists(count: usize) -> Vec<Artist> {
        (0..count).map(|i| {
            TestFixtures::create_artist_with_name(&format!("Test Artist {}", i))
        }).collect()
    }
    
    /// Create multiple DNP entries for a user
    pub fn create_dnp_entries(user_id: Uuid, artist_ids: &[Uuid]) -> Vec<DnpEntry> {
        artist_ids.iter().enumerate().map(|(i, &artist_id)| {
            TestFixtures::create_dnp_entry_with_details(
                user_id,
                artist_id,
                vec![format!("tag{}", i)],
                Some(format!("Note for entry {}", i))
            )
        }).collect()
    }
}

/// Scenario-based fixtures for integration testing
pub struct ScenarioFixtures;

impl ScenarioFixtures {
    /// Create a complete user scenario with DNP list
    pub async fn create_user_with_dnp_list(
        db: &sqlx::PgPool,
        artist_count: usize
    ) -> Result<(User, Vec<Artist>, Vec<DnpEntry>), sqlx::Error> {
        // Create user
        let user = TestFixtures::create_user();
        sqlx::query!(
            "INSERT INTO users (id, email, password_hash, email_verified, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)",
            user.id,
            user.email,
            user.password_hash,
            user.email_verified,
            user.created_at,
            user.updated_at
        )
        .execute(db)
        .await?;
        
        // Create artists
        let mut artists = Vec::new();
        for i in 0..artist_count {
            let artist = TestFixtures::create_artist_with_name(&format!("Scenario Artist {}", i));
            sqlx::query!(
                "INSERT INTO artists (id, canonical_name, external_ids, metadata, created_at) VALUES ($1, $2, $3, $4, $5)",
                artist.id,
                artist.canonical_name,
                artist.external_ids,
                artist.metadata,
                artist.created_at
            )
            .execute(db)
            .await?;
            artists.push(artist);
        }
        
        // Create DNP entries
        let mut dnp_entries = Vec::new();
        for (i, artist) in artists.iter().enumerate() {
            let entry = TestFixtures::create_dnp_entry_with_details(
                user.id,
                artist.id,
                vec![format!("scenario_tag_{}", i)],
                Some(format!("Scenario note {}", i))
            );
            
            sqlx::query!(
                "INSERT INTO user_artist_blocks (user_id, artist_id, tags, note, created_at) VALUES ($1, $2, $3, $4, $5)",
                entry.user_id,
                entry.artist_id,
                entry.tags.as_deref(),
                entry.note,
                entry.created_at
            )
            .execute(db)
            .await?;
            
            dnp_entries.push(entry);
        }
        
        Ok((user, artists, dnp_entries))
    }
    
    /// Create a user with 2FA setup scenario
    pub async fn create_2fa_user_scenario(
        db: &sqlx::PgPool
    ) -> Result<(User, String), sqlx::Error> {
        let user = TestFixtures::create_user_with_2fa();
        let totp_secret = user.totp_secret.clone().unwrap();
        
        sqlx::query!(
            "INSERT INTO users (id, email, password_hash, totp_secret, totp_enabled, email_verified, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            user.id,
            user.email,
            user.password_hash,
            user.totp_secret,
            user.totp_enabled,
            user.email_verified,
            user.created_at,
            user.updated_at
        )
        .execute(db)
        .await?;
        
        Ok((user, totp_secret))
    }
    
    /// Create multiple users with varying DNP list sizes
    pub async fn create_performance_test_scenario(
        db: &sqlx::PgPool,
        user_count: usize,
        max_dnp_entries: usize
    ) -> Result<Vec<(User, Vec<DnpEntry>)>, sqlx::Error> {
        let mut results = Vec::new();
        
        // Create a pool of artists to use
        let artists = BatchFixtures::create_artists(max_dnp_entries * 2);
        for artist in &artists {
            sqlx::query!(
                "INSERT INTO artists (id, canonical_name, external_ids, metadata, created_at) VALUES ($1, $2, $3, $4, $5)",
                artist.id,
                artist.canonical_name,
                artist.external_ids,
                artist.metadata,
                artist.created_at
            )
            .execute(db)
            .await?;
        }
        
        // Create users with varying DNP list sizes
        for i in 0..user_count {
            let user = TestFixtures::create_user();
            sqlx::query!(
                "INSERT INTO users (id, email, password_hash, email_verified, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)",
                user.id,
                user.email,
                user.password_hash,
                user.email_verified,
                user.created_at,
                user.updated_at
            )
            .execute(db)
            .await?;
            
            // Create varying number of DNP entries (0 to max_dnp_entries)
            let entry_count = (i * max_dnp_entries) / user_count;
            let mut user_entries = Vec::new();
            
            for j in 0..entry_count {
                let artist = &artists[j % artists.len()];
                let entry = TestFixtures::create_dnp_entry_with_details(
                    user.id,
                    artist.id,
                    vec![format!("perf_tag_{}", j)],
                    Some(format!("Performance test entry {}", j))
                );
                
                sqlx::query!(
                    "INSERT INTO user_artist_blocks (user_id, artist_id, tags, note, created_at) VALUES ($1, $2, $3, $4, $5)",
                    entry.user_id,
                    entry.artist_id,
                    entry.tags.as_deref(),
                    entry.note,
                    entry.created_at
                )
                .execute(db)
                .await?;
                
                user_entries.push(entry);
            }
            
            results.push((user, user_entries));
        }
        
        Ok(results)
    }
}