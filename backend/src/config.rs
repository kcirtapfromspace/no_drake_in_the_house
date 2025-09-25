use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub server_address: String,
    pub jwt_secret: String,
    pub duckdb_path: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/music_blocklist".to_string());
        
        let server_address = env::var("SERVER_ADDRESS")
            .unwrap_or_else(|_| "0.0.0.0:3000".to_string());
        
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-secret-key".to_string());
        
        let duckdb_path = env::var("DUCKDB_PATH")
            .unwrap_or_else(|_| "./data/analytics.duckdb".to_string());

        Ok(Config {
            database_url,
            server_address,
            jwt_secret,
            duckdb_path,
        })
    }
}