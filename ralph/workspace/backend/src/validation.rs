//! Request validation utilities and custom validators

use crate::error::{AppError, Result};
use axum::{
    async_trait,
    extract::{FromRequest, Request},
    Json,
};
use serde::de::DeserializeOwned;
use validator::{Validate, ValidationError};

/// Validated JSON extractor that automatically validates incoming requests
#[derive(Debug)]
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection| AppError::InvalidRequestFormat(rejection.to_string()))?;

        value.validate()?;
        Ok(ValidatedJson(value))
    }
}

/// Custom validator for email addresses
pub fn validate_email(email: &str) -> std::result::Result<(), ValidationError> {
    let email_regex = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .map_err(|_| ValidationError::new("invalid_regex"))?;

    if !email_regex.is_match(email) {
        return Err(ValidationError::new("invalid_email"));
    }

    // Additional checks
    if email.len() > 254 {
        return Err(ValidationError::new("email_too_long"));
    }

    if email.contains("..") {
        return Err(ValidationError::new("consecutive_dots"));
    }

    Ok(())
}

/// Custom validator for password strength
pub fn validate_password(password: &str) -> std::result::Result<(), ValidationError> {
    if password.len() < 8 {
        return Err(ValidationError::new("password_too_short"));
    }

    if password.len() > 128 {
        return Err(ValidationError::new("password_too_long"));
    }

    // Check for at least one lowercase letter
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(ValidationError::new("password_missing_lowercase"));
    }

    // Check for at least one uppercase letter
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(ValidationError::new("password_missing_uppercase"));
    }

    // Check for at least one digit
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(ValidationError::new("password_missing_digit"));
    }

    // Check for at least one special character
    if !password
        .chars()
        .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c))
    {
        return Err(ValidationError::new("password_missing_special"));
    }

    Ok(())
}

/// Custom validator for TOTP codes
pub fn validate_totp_code(code: &str) -> std::result::Result<(), ValidationError> {
    if code.len() != 6 {
        return Err(ValidationError::new("totp_invalid_length"));
    }

    if !code.chars().all(|c| c.is_ascii_digit()) {
        return Err(ValidationError::new("totp_invalid_format"));
    }

    Ok(())
}

/// Custom validator for artist names
pub fn validate_artist_name(name: &str) -> std::result::Result<(), ValidationError> {
    if name.trim().is_empty() {
        return Err(ValidationError::new("artist_name_empty"));
    }

    if name.len() > 255 {
        return Err(ValidationError::new("artist_name_too_long"));
    }

    // Check for potentially harmful characters
    if name.contains(['<', '>', '"', '\'', '&']) {
        return Err(ValidationError::new("artist_name_invalid_characters"));
    }

    Ok(())
}

/// Custom validator for tags
pub fn validate_tags(tags: &[String]) -> std::result::Result<(), ValidationError> {
    if tags.len() > 10 {
        return Err(ValidationError::new("too_many_tags"));
    }

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(ValidationError::new("empty_tag"));
        }

        if tag.len() > 50 {
            return Err(ValidationError::new("tag_too_long"));
        }

        // Check for invalid characters
        if tag.contains(['<', '>', '"', '\'', '&']) {
            return Err(ValidationError::new("tag_invalid_characters"));
        }
    }

    Ok(())
}

/// Custom validator for UUIDs
pub fn validate_uuid_string(uuid_str: &str) -> std::result::Result<(), ValidationError> {
    uuid::Uuid::parse_str(uuid_str).map_err(|_| ValidationError::new("invalid_uuid"))?;
    Ok(())
}

/// Custom validator for pagination limits
pub fn validate_pagination_limit(limit: i32) -> std::result::Result<(), ValidationError> {
    if limit < 1 {
        return Err(ValidationError::new("limit_too_small"));
    }

    if limit > 100 {
        return Err(ValidationError::new("limit_too_large"));
    }

    Ok(())
}

/// Custom validator for pagination offsets
pub fn validate_pagination_offset(offset: i32) -> std::result::Result<(), ValidationError> {
    if offset < 0 {
        return Err(ValidationError::new("offset_negative"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email() {
        // Valid emails
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user.name+tag@domain.co.uk").is_ok());

        // Invalid emails
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("test@").is_err());
        assert!(validate_email("test..test@example.com").is_err());
    }

    #[test]
    fn test_validate_password() {
        // Valid password
        assert!(validate_password("SecurePass123!").is_ok());

        // Invalid passwords
        assert!(validate_password("short").is_err()); // Too short
        assert!(validate_password("nouppercase123!").is_err()); // No uppercase
        assert!(validate_password("NOLOWERCASE123!").is_err()); // No lowercase
        assert!(validate_password("NoDigits!").is_err()); // No digits
        assert!(validate_password("NoSpecial123").is_err()); // No special chars
    }

    #[test]
    fn test_validate_totp_code() {
        // Valid TOTP codes
        assert!(validate_totp_code("123456").is_ok());
        assert!(validate_totp_code("000000").is_ok());

        // Invalid TOTP codes
        assert!(validate_totp_code("12345").is_err()); // Too short
        assert!(validate_totp_code("1234567").is_err()); // Too long
        assert!(validate_totp_code("12345a").is_err()); // Contains letter
    }

    #[test]
    fn test_validate_artist_name() {
        // Valid artist names
        assert!(validate_artist_name("The Beatles").is_ok());
        assert!(validate_artist_name("AC/DC").is_ok());

        // Invalid artist names
        assert!(validate_artist_name("").is_err()); // Empty
        assert!(validate_artist_name("   ").is_err()); // Only whitespace
        assert!(validate_artist_name("Artist<script>").is_err()); // Invalid chars
    }

    #[test]
    fn test_validate_tags() {
        // Valid tags
        assert!(validate_tags(&["rock".to_string(), "classic".to_string()]).is_ok());
        assert!(validate_tags(&[]).is_ok()); // Empty is ok

        // Invalid tags
        let too_many_tags: Vec<String> = (0..11).map(|i| format!("tag{}", i)).collect();
        assert!(validate_tags(&too_many_tags).is_err()); // Too many

        assert!(validate_tags(&["".to_string()]).is_err()); // Empty tag
        assert!(validate_tags(&["tag<script>".to_string()]).is_err()); // Invalid chars
    }
}
