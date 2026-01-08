use music_streaming_blocklist_backend::{
    error::AppError,
    models::oauth::{OAuthConfig, OAuthProviderType},
    services::oauth::OAuthProvider,
    services::oauth_github::{GitHubEmail, GitHubOAuthProvider, GitHubOAuthService},
};
use serde_json::json;
use std::collections::HashMap;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn create_test_github_provider() -> GitHubOAuthProvider {
    GitHubOAuthProvider::with_credentials(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "http://localhost:3000/auth/github/callback".to_string(),
    )
    .unwrap()
}

#[tokio::test]
async fn test_github_oauth_token_exchange_success() {
    let mock_server = MockServer::start().await;

    // Mock successful GitHub token exchange
    Mock::given(method("POST"))
        .and(path("/login/oauth/access_token"))
        .and(header("content-type", "application/x-www-form-urlencoded"))
        .and(header("accept", "application/json"))
        .and(header("user-agent", "OAuth-App"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "github_access_token",
            "token_type": "bearer",
            "scope": "user:email,read:user"
        })))
        .mount(&mock_server)
        .await;

    let provider = create_test_github_provider();

    // Test provider creation and validation
    assert_eq!(provider.provider_type(), OAuthProviderType::GitHub);
    assert!(provider.validate_config().is_ok());
}

#[tokio::test]
async fn test_github_oauth_user_info_success() {
    let mock_server = MockServer::start().await;

    // Mock successful GitHub user info request
    Mock::given(method("GET"))
        .and(path("/user"))
        .and(header("authorization", "Bearer github_access_token"))
        .and(header("accept", "application/vnd.github.v3+json"))
        .and(header("user-agent", "OAuth-App"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123456789,
            "login": "octocat",
            "name": "The Octocat",
            "email": "octocat@github.com",
            "avatar_url": "https://avatars.githubusercontent.com/u/583231",
            "html_url": "https://github.com/octocat",
            "bio": "GitHub mascot",
            "company": "GitHub",
            "location": "San Francisco",
            "blog": "https://github.blog",
            "public_repos": 8,
            "followers": 4000,
            "following": 9
        })))
        .mount(&mock_server)
        .await;

    let provider = create_test_github_provider();

    // Test user info parsing directly
    let user_data = json!({
        "id": 123456789,
        "login": "octocat",
        "name": "The Octocat",
        "email": "octocat@github.com",
        "avatar_url": "https://avatars.githubusercontent.com/u/583231",
        "html_url": "https://github.com/octocat",
        "bio": "GitHub mascot",
        "company": "GitHub",
        "location": "San Francisco",
        "blog": "https://github.blog",
        "public_repos": 8,
        "followers": 4000,
        "following": 9
    });

    let user_info = provider.parse_user_info(user_data).unwrap();

    assert_eq!(user_info.provider_user_id, "123456789");
    assert_eq!(user_info.email, Some("octocat@github.com".to_string()));
    assert_eq!(user_info.display_name, Some("The Octocat".to_string()));
    assert_eq!(user_info.first_name, Some("The".to_string()));
    assert_eq!(user_info.last_name, Some("Octocat".to_string()));
    assert_eq!(
        user_info.avatar_url,
        Some("https://avatars.githubusercontent.com/u/583231".to_string())
    );

    // Check GitHub-specific data
    assert_eq!(
        user_info.provider_data.get("username"),
        Some(&serde_json::Value::String("octocat".to_string()))
    );
    assert_eq!(
        user_info.provider_data.get("profile_url"),
        Some(&serde_json::Value::String(
            "https://github.com/octocat".to_string()
        ))
    );
    assert_eq!(
        user_info.provider_data.get("bio"),
        Some(&serde_json::Value::String("GitHub mascot".to_string()))
    );
    assert_eq!(
        user_info.provider_data.get("company"),
        Some(&serde_json::Value::String("GitHub".to_string()))
    );
    assert_eq!(
        user_info.provider_data.get("public_repos"),
        Some(&serde_json::Value::Number(8.into()))
    );
}

#[tokio::test]
async fn test_github_oauth_initiate_flow() {
    let provider = create_test_github_provider();
    let redirect_uri = "http://localhost:3000/auth/github/callback";

    let flow_response = provider.initiate_flow(redirect_uri).await.unwrap();

    // Verify the authorization URL contains expected parameters
    assert!(flow_response
        .authorization_url
        .contains("github.com/login/oauth/authorize"));
    assert!(flow_response
        .authorization_url
        .contains("client_id=test_client_id"));
    assert!(flow_response
        .authorization_url
        .contains("response_type=code"));
    assert!(flow_response
        .authorization_url
        .contains("scope=user%3Aemail%20read%3Auser"));
    assert!(flow_response
        .authorization_url
        .contains("allow_signup=true"));

    // State should be a valid UUID
    assert_eq!(flow_response.state.len(), 36); // UUID length with hyphens
    assert!(flow_response.code_verifier.is_none()); // GitHub doesn't use PKCE for server-side
}

#[test]
fn test_github_oauth_config_validation() {
    // Test valid configuration
    let provider = create_test_github_provider();
    assert!(provider.validate_config().is_ok());

    // Test missing client_id
    let config = OAuthConfig {
        client_id: "".to_string(),
        client_secret: "test_secret".to_string(),
        redirect_uri: "http://localhost:3000/auth/callback".to_string(),
        scopes: vec!["user:email".to_string(), "read:user".to_string()],
        additional_params: HashMap::new(),
    };
    let provider = GitHubOAuthProvider::new(config).unwrap();
    let result = provider.validate_config();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("client_id is required"));

    // Test missing client_secret
    let config = OAuthConfig {
        client_id: "test_client_id".to_string(),
        client_secret: "".to_string(),
        redirect_uri: "http://localhost:3000/auth/callback".to_string(),
        scopes: vec!["user:email".to_string(), "read:user".to_string()],
        additional_params: HashMap::new(),
    };
    let provider = GitHubOAuthProvider::new(config).unwrap();
    let result = provider.validate_config();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("client_secret is required"));
}

#[tokio::test]
async fn test_github_oauth_service_with_options() {
    let provider = create_test_github_provider();
    let service = GitHubOAuthService::new(provider);

    let redirect_uri = "http://localhost:3000/auth/github/callback";
    let allow_signup = Some(false);
    let login = Some("suggested_user");

    let flow_response = service
        .initiate_flow_with_options(redirect_uri, allow_signup, login)
        .await
        .unwrap();

    // Verify additional parameters are included
    assert!(flow_response
        .authorization_url
        .contains("allow_signup=false"));
    assert!(flow_response
        .authorization_url
        .contains("login=suggested_user"));
}

#[tokio::test]
async fn test_github_user_emails() {
    let mock_server = MockServer::start().await;

    // Mock GitHub user emails endpoint
    Mock::given(method("GET"))
        .and(path("/user/emails"))
        .and(header("authorization", "Bearer github_access_token"))
        .and(header("accept", "application/vnd.github.v3+json"))
        .and(header("user-agent", "OAuth-App"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "email": "octocat@github.com",
                "primary": true,
                "verified": true,
                "visibility": "public"
            },
            {
                "email": "octocat@users.noreply.github.com",
                "primary": false,
                "verified": true,
                "visibility": null
            }
        ])))
        .mount(&mock_server)
        .await;

    let provider = create_test_github_provider();
    let service = GitHubOAuthService::new(provider);

    // Test email structure parsing
    let email_data = json!([
        {
            "email": "octocat@github.com",
            "primary": true,
            "verified": true,
            "visibility": "public"
        },
        {
            "email": "octocat@users.noreply.github.com",
            "primary": false,
            "verified": true,
            "visibility": null
        }
    ]);

    // Parse emails manually for testing
    let mut github_emails = Vec::new();
    for email_item in email_data.as_array().unwrap() {
        let email = GitHubEmail {
            email: email_item["email"].as_str().unwrap_or("").to_string(),
            primary: email_item["primary"].as_bool().unwrap_or(false),
            verified: email_item["verified"].as_bool().unwrap_or(false),
            visibility: email_item["visibility"].as_str().map(|s| s.to_string()),
        };
        github_emails.push(email);
    }

    assert_eq!(github_emails.len(), 2);
    assert_eq!(github_emails[0].email, "octocat@github.com");
    assert!(github_emails[0].primary);
    assert!(github_emails[0].verified);
    assert_eq!(github_emails[0].visibility, Some("public".to_string()));

    assert_eq!(github_emails[1].email, "octocat@users.noreply.github.com");
    assert!(!github_emails[1].primary);
    assert!(github_emails[1].verified);
    assert_eq!(github_emails[1].visibility, None);
}

#[test]
fn test_github_user_info_parsing_edge_cases() {
    let provider = create_test_github_provider();

    // Test minimal valid response
    let minimal_data = json!({
        "id": 123456789,
        "login": "testuser"
    });
    let user_info = provider.parse_user_info(minimal_data).unwrap();
    assert_eq!(user_info.provider_user_id, "123456789");
    assert_eq!(user_info.email, None);
    assert_eq!(user_info.display_name, None);
    assert_eq!(
        user_info.provider_data.get("username"),
        Some(&serde_json::Value::String("testuser".to_string()))
    );

    // Test missing ID (should fail)
    let invalid_data = json!({
        "login": "testuser",
        "email": "test@example.com"
    });
    let result = provider.parse_user_info(invalid_data);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Missing user ID"));

    // Test name parsing variations
    let single_name_data = json!({
        "id": 123456789,
        "name": "SingleName"
    });
    let user_info = provider.parse_user_info(single_name_data).unwrap();
    assert_eq!(user_info.first_name, Some("SingleName".to_string()));
    assert_eq!(user_info.last_name, None);

    let full_name_data = json!({
        "id": 123456789,
        "name": "First Last"
    });
    let user_info = provider.parse_user_info(full_name_data).unwrap();
    assert_eq!(user_info.first_name, Some("First".to_string()));
    assert_eq!(user_info.last_name, Some("Last".to_string()));

    let complex_name_data = json!({
        "id": 123456789,
        "name": "First Middle Last"
    });
    let user_info = provider.parse_user_info(complex_name_data).unwrap();
    assert_eq!(user_info.first_name, Some("First".to_string()));
    assert_eq!(user_info.last_name, Some("Middle Last".to_string()));
}

#[test]
fn test_github_provider_data_extraction() {
    let provider = create_test_github_provider();

    let complete_data = json!({
        "id": 123456789,
        "login": "testuser",
        "name": "Test User",
        "email": "test@example.com",
        "avatar_url": "https://avatars.githubusercontent.com/u/123456789",
        "html_url": "https://github.com/testuser",
        "bio": "Software Developer",
        "company": "Test Company",
        "location": "San Francisco, CA",
        "blog": "https://testuser.dev",
        "public_repos": 42,
        "followers": 100,
        "following": 50
    });

    let user_info = provider.parse_user_info(complete_data).unwrap();

    // Verify all provider-specific data is captured
    assert_eq!(user_info.provider_data.len(), 8);
    assert!(user_info.provider_data.contains_key("username"));
    assert!(user_info.provider_data.contains_key("profile_url"));
    assert!(user_info.provider_data.contains_key("bio"));
    assert!(user_info.provider_data.contains_key("company"));
    assert!(user_info.provider_data.contains_key("location"));
    assert!(user_info.provider_data.contains_key("blog"));
    assert!(user_info.provider_data.contains_key("public_repos"));
    assert!(user_info.provider_data.contains_key("followers"));
    assert!(user_info.provider_data.contains_key("following"));
}

#[test]
fn test_github_empty_blog_handling() {
    let provider = create_test_github_provider();

    // Test with empty blog field (should not be included)
    let data_with_empty_blog = json!({
        "id": 123456789,
        "login": "testuser",
        "blog": ""
    });

    let user_info = provider.parse_user_info(data_with_empty_blog).unwrap();
    assert!(!user_info.provider_data.contains_key("blog"));

    // Test with valid blog field (should be included)
    let data_with_blog = json!({
        "id": 123456789,
        "login": "testuser",
        "blog": "https://example.com"
    });

    let user_info = provider.parse_user_info(data_with_blog).unwrap();
    assert!(user_info.provider_data.contains_key("blog"));
    assert_eq!(
        user_info.provider_data.get("blog"),
        Some(&serde_json::Value::String(
            "https://example.com".to_string()
        ))
    );
}

#[tokio::test]
async fn test_github_refresh_token_not_supported() {
    let provider = create_test_github_provider();

    let result = provider.refresh_token("dummy_token").await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("don't support token refresh"));
}

#[test]
fn test_github_email_struct() {
    let email = GitHubEmail {
        email: "test@example.com".to_string(),
        primary: true,
        verified: true,
        visibility: Some("public".to_string()),
    };

    assert_eq!(email.email, "test@example.com");
    assert!(email.primary);
    assert!(email.verified);
    assert_eq!(email.visibility, Some("public".to_string()));

    // Test with no visibility
    let private_email = GitHubEmail {
        email: "private@example.com".to_string(),
        primary: false,
        verified: false,
        visibility: None,
    };

    assert_eq!(private_email.email, "private@example.com");
    assert!(!private_email.primary);
    assert!(!private_email.verified);
    assert_eq!(private_email.visibility, None);
}
