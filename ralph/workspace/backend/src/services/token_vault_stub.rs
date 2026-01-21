use anyhow::Result;
use uuid::Uuid;

/// Stub implementation of TokenVaultService for tests
#[derive(Clone)]
pub struct TokenVaultService {
}

impl TokenVaultService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn store_token(&self, _user_id: Uuid, _provider: &str, _token: &str) -> Result<()> {
        // Stub implementation
        Ok(())
    }

    pub async fn get_token(&self, _user_id: Uuid, _provider: &str) -> Result<Option<String>> {
        // Stub implementation
        Ok(None)
    }

    pub async fn delete_token(&self, _user_id: Uuid, _provider: &str) -> Result<()> {
        // Stub implementation
        Ok(())
    }
}