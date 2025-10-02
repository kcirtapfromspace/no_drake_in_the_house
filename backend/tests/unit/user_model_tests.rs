#[cfg(test)]
mod user_model_tests {
    use music_streaming_blocklist_backend::models::{
        user::*,
        oauth::{OAuthAccount, OAuthProviderType},
    };
    use uuid::Uuid;
    use chrono::Utc;

    fn create_test_oauth_account(provider: OAuthProviderType, user_id: Uuid) -> OAuthAccount {
        OAuthAccount::new(
            user_id,
            provider.clone(),
            format!("provider_user_123_{}", provider),
            Some(format!("test@{}.com", provider)),
            Some(format!("Test User {}", provider)),
            Some(format!("https://avatar.{}.com/test", provider)),
            vec![1, 2, 3, 4], // Mock encrypted token
            Some(vec![5, 6, 7, 8]), // Mock encrypted refresh token
            Some(Utc::now() + chrono::Duration::hours(1)),
        )
    }

    #[test]
    fn test_user_new() {
        let email = "test@example.com".to_string();
        let password_hash = Some("hashed_password".to_string());
        
        let user = User::new(email.clone(), password_hash.clone());
        
        assert_eq!(user.email, email);
        assert_eq!(user.password_hash, password_hash);
        assert!(!user.email_verified);
        assert!(!user.totp_enabled);
        assert!(user.oauth_accounts.is_empty());
        assert!(user.last_login.is_none());
    }

    #[test]
    fn test_user_with_oauth_account() {
        let email = "test@google.com".to_string();
        let user_id = Uuid::new_v4();
        let oauth_account = create_test_oauth_account(OAuthProviderType::Google, user_id);
        
        let user = User::with_oauth_account(email.clone(), oauth_account.clone());
        
        assert_eq!(user.email, email);
        assert!(user.password_hash.is_none());
        assert!(user.email_verified); // Should be verified since OAuth account has email
        assert_eq!(user.oauth_accounts.len(), 1);
        assert_eq!(user.oauth_accounts[0].provider, OAuthProviderType::Google);
    }

    #[test]
    fn test_add_oauth_account() {
        let mut user = User::new("test@example.com".to_string(), None);
        let oauth_account = create_test_oauth_account(OAuthProviderType::Google, user.id);
        
        user.add_oauth_account(oauth_account.clone());
        
        assert_eq!(user.oauth_accounts.len(), 1);
        assert_eq!(user.oauth_accounts[0].provider, OAuthProviderType::Google);
    }

    #[test]
    fn test_add_oauth_account_replaces_existing() {
        let mut user = User::new("test@example.com".to_string(), None);
        let oauth_account1 = create_test_oauth_account(OAuthProviderType::Google, user.id);
        let mut oauth_account2 = create_test_oauth_account(OAuthProviderType::Google, user.id);
        oauth_account2.display_name = Some("Updated Name".to_string());
        
        user.add_oauth_account(oauth_account1);
        assert_eq!(user.oauth_accounts.len(), 1);
        
        user.add_oauth_account(oauth_account2);
        assert_eq!(user.oauth_accounts.len(), 1); // Should still be 1
        assert_eq!(user.oauth_accounts[0].display_name, Some("Updated Name".to_string()));
    }

    #[test]
    fn test_remove_oauth_account() {
        let mut user = User::new("test@example.com".to_string(), None);
        let google_account = create_test_oauth_account(OAuthProviderType::Google, user.id);
        let github_account = create_test_oauth_account(OAuthProviderType::GitHub, user.id);
        
        user.add_oauth_account(google_account);
        user.add_oauth_account(github_account);
        assert_eq!(user.oauth_accounts.len(), 2);
        
        let removed = user.remove_oauth_account(&OAuthProviderType::Google);
        assert!(removed);
        assert_eq!(user.oauth_accounts.len(), 1);
        assert_eq!(user.oauth_accounts[0].provider, OAuthProviderType::GitHub);
        
        // Try to remove non-existent provider
        let removed = user.remove_oauth_account(&OAuthProviderType::Apple);
        assert!(!removed);
        assert_eq!(user.oauth_accounts.len(), 1);
    }

    #[test]
    fn test_get_oauth_account() {
        let mut user = User::new("test@example.com".to_string(), None);
        let oauth_account = create_test_oauth_account(OAuthProviderType::Google, user.id);
        
        user.add_oauth_account(oauth_account.clone());
        
        let found_account = user.get_oauth_account(&OAuthProviderType::Google);
        assert!(found_account.is_some());
        assert_eq!(found_account.unwrap().provider, OAuthProviderType::Google);
        
        let not_found = user.get_oauth_account(&OAuthProviderType::Apple);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_has_oauth_account() {
        let mut user = User::new("test@example.com".to_string(), None);
        let oauth_account = create_test_oauth_account(OAuthProviderType::Google, user.id);
        
        assert!(!user.has_oauth_account(&OAuthProviderType::Google));
        
        user.add_oauth_account(oauth_account);
        
        assert!(user.has_oauth_account(&OAuthProviderType::Google));
        assert!(!user.has_oauth_account(&OAuthProviderType::Apple));
    }

    #[test]
    fn test_is_oauth_only() {
        // User with password is not OAuth-only
        let mut user = User::new("test@example.com".to_string(), Some("password".to_string()));
        assert!(!user.is_oauth_only());
        
        // User without password but no OAuth accounts is not OAuth-only
        user.password_hash = None;
        assert!(!user.is_oauth_only());
        
        // User without password but with OAuth accounts is OAuth-only
        let oauth_account = create_test_oauth_account(OAuthProviderType::Google, user.id);
        user.add_oauth_account(oauth_account);
        assert!(user.is_oauth_only());
    }

    #[test]
    fn test_linked_providers() {
        let mut user = User::new("test@example.com".to_string(), None);
        
        assert!(user.linked_providers().is_empty());
        
        let google_account = create_test_oauth_account(OAuthProviderType::Google, user.id);
        let github_account = create_test_oauth_account(OAuthProviderType::GitHub, user.id);
        
        user.add_oauth_account(google_account);
        user.add_oauth_account(github_account);
        
        let providers = user.linked_providers();
        assert_eq!(providers.len(), 2);
        assert!(providers.contains(&OAuthProviderType::Google));
        assert!(providers.contains(&OAuthProviderType::GitHub));
    }

    #[test]
    fn test_to_profile() {
        let mut user = User::new("test@example.com".to_string(), Some("password".to_string()));
        user.email_verified = true;
        user.enable_totp("secret".to_string());
        
        let oauth_account = create_test_oauth_account(OAuthProviderType::Google, user.id);
        user.add_oauth_account(oauth_account);
        
        let profile = user.to_profile();
        
        assert_eq!(profile.id, user.id);
        assert_eq!(profile.email, user.email);
        assert!(profile.email_verified);
        assert!(profile.totp_enabled);
        assert_eq!(profile.oauth_accounts.len(), 1);
        assert_eq!(profile.oauth_accounts[0].provider, OAuthProviderType::Google);
        
        // Ensure sensitive data is not included
        // (UserProfile doesn't have password_hash or totp_secret fields)
    }

    #[test]
    fn test_merge_from() {
        let mut primary_user = User::new("primary@example.com".to_string(), Some("password".to_string()));
        let mut secondary_user = User::new("secondary@example.com".to_string(), None);
        
        // Add OAuth accounts to secondary user
        let google_account = create_test_oauth_account(OAuthProviderType::Google, secondary_user.id);
        let github_account = create_test_oauth_account(OAuthProviderType::GitHub, secondary_user.id);
        secondary_user.add_oauth_account(google_account);
        secondary_user.add_oauth_account(github_account);
        secondary_user.email_verified = true;
        
        // Merge secondary into primary
        let result = primary_user.merge_from(&secondary_user);
        assert!(result.is_ok());
        
        // Check that OAuth accounts were merged
        assert_eq!(primary_user.oauth_accounts.len(), 2);
        assert!(primary_user.has_oauth_account(&OAuthProviderType::Google));
        assert!(primary_user.has_oauth_account(&OAuthProviderType::GitHub));
        
        // Check that email verification was updated
        assert!(primary_user.email_verified);
        
        // Check that user IDs were updated in OAuth accounts
        for account in &primary_user.oauth_accounts {
            assert_eq!(account.user_id, primary_user.id);
        }
    }

    #[test]
    fn test_merge_from_no_duplicate_providers() {
        let mut primary_user = User::new("primary@example.com".to_string(), Some("password".to_string()));
        let mut secondary_user = User::new("secondary@example.com".to_string(), None);
        
        // Both users have Google accounts
        let primary_google = create_test_oauth_account(OAuthProviderType::Google, primary_user.id);
        let secondary_google = create_test_oauth_account(OAuthProviderType::Google, secondary_user.id);
        
        primary_user.add_oauth_account(primary_google);
        secondary_user.add_oauth_account(secondary_google);
        
        // Merge should not create duplicates
        let result = primary_user.merge_from(&secondary_user);
        assert!(result.is_ok());
        
        assert_eq!(primary_user.oauth_accounts.len(), 1);
        assert_eq!(primary_user.oauth_accounts[0].user_id, primary_user.id);
    }

    #[test]
    fn test_oauth_account_info_from_oauth_account() {
        let user_id = Uuid::new_v4();
        let oauth_account = create_test_oauth_account(OAuthProviderType::Google, user_id);
        
        let account_info = OAuthAccountInfo::from_oauth_account(&oauth_account);
        
        assert_eq!(account_info.provider, oauth_account.provider);
        assert_eq!(account_info.provider_user_id, oauth_account.provider_user_id);
        assert_eq!(account_info.email, oauth_account.email);
        assert_eq!(account_info.display_name, oauth_account.display_name);
        assert_eq!(account_info.avatar_url, oauth_account.avatar_url);
        assert_eq!(account_info.connected_at, oauth_account.created_at);
        assert!(account_info.last_used_at.is_none());
    }

    #[test]
    fn test_enable_disable_totp() {
        let mut user = User::new("test@example.com".to_string(), Some("password".to_string()));
        
        assert!(!user.totp_enabled);
        assert!(!user.settings.two_factor_enabled);
        assert!(user.totp_secret.is_none());
        
        user.enable_totp("secret123".to_string());
        
        assert!(user.totp_enabled);
        assert!(user.settings.two_factor_enabled);
        assert_eq!(user.totp_secret, Some("secret123".to_string()));
        
        user.disable_totp();
        
        assert!(!user.totp_enabled);
        assert!(!user.settings.two_factor_enabled);
        assert!(user.totp_secret.is_none());
    }

    #[test]
    fn test_update_last_login() {
        let mut user = User::new("test@example.com".to_string(), Some("password".to_string()));
        let initial_updated_at = user.updated_at;
        
        assert!(user.last_login.is_none());
        
        // Sleep a bit to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        user.update_last_login();
        
        assert!(user.last_login.is_some());
        assert!(user.updated_at > initial_updated_at);
        assert_eq!(user.last_login.unwrap(), user.updated_at);
    }
}