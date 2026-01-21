//! Validated request models with comprehensive validation

use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

/// Validated user registration request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedCreateUserRequest {
    #[validate(custom = "validate_email")]
    pub email: String,
    
    #[validate(custom = "validate_password")]
    pub password: String,
}

/// Validated user login request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedLoginRequest {
    #[validate(custom = "validate_email")]
    pub email: String,
    
    #[validate(length(min = 1, max = 128, message = "Password is required"))]
    pub password: String,
    
    #[validate(custom = "validate_totp_code")]
    pub totp_code: Option<String>,
}

/// Validated TOTP setup request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedTotpSetupRequest {
    #[validate(custom = "validate_uuid_string")]
    pub user_id: String,
}

/// Validated TOTP enable request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedTotpEnableRequest {
    #[validate(custom = "validate_uuid_string")]
    pub user_id: String,
    
    #[validate(custom = "validate_totp_code")]
    pub totp_code: String,
}

/// Validated TOTP disable request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedTotpDisableRequest {
    #[validate(custom = "validate_uuid_string")]
    pub user_id: String,
    
    #[validate(custom = "validate_totp_code")]
    pub totp_code: String,
}

/// Validated artist search request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedArtistSearchRequest {
    #[validate(length(min = 1, max = 255, message = "Search query must be between 1 and 255 characters"))]
    pub query: String,
    
    #[validate(custom = "validate_pagination_limit")]
    pub limit: Option<i32>,
    
    #[validate(custom = "validate_pagination_offset")]
    pub offset: Option<i32>,
}

/// Validated add artist to DNP list request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedAddToDnpRequest {
    #[validate(custom = "validate_uuid_string")]
    pub user_id: String,
    
    #[validate(custom = "validate_artist_name")]
    pub artist_name: String,
    
    #[validate(custom = "validate_tags")]
    pub tags: Option<Vec<String>>,
    
    #[validate(length(max = 1000, message = "Note cannot exceed 1000 characters"))]
    pub note: Option<String>,
}

/// Validated update DNP entry request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedUpdateDnpEntryRequest {
    #[validate(custom = "validate_uuid_string")]
    pub user_id: String,
    
    #[validate(custom = "validate_uuid_string")]
    pub artist_id: String,
    
    #[validate(custom = "validate_tags")]
    pub tags: Option<Vec<String>>,
    
    #[validate(length(max = 1000, message = "Note cannot exceed 1000 characters"))]
    pub note: Option<String>,
}

/// Validated user profile update request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedUpdateUserProfileRequest {
    #[validate(custom = "validate_uuid_string")]
    pub user_id: String,
    
    #[validate(custom = "validate_email")]
    pub email: Option<String>,
    
    pub settings: Option<ValidatedUserSettings>,
}

/// Validated user settings
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedUserSettings {
    pub two_factor_enabled: Option<bool>,
    pub email_notifications: Option<bool>,
    pub privacy_mode: Option<bool>,
    
    #[validate(length(max = 100, message = "Timezone cannot exceed 100 characters"))]
    pub timezone: Option<String>,
    
    #[validate(length(max = 10, message = "Language code cannot exceed 10 characters"))]
    pub language: Option<String>,
}

/// Validated bulk import request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedBulkImportRequest {
    #[validate(custom = "validate_uuid_string")]
    pub user_id: String,
    
    pub format: ImportFormat,
    
    #[validate(length(min = 1, max = 10485760, message = "Data must be between 1 byte and 10MB"))] // 10MB limit
    pub data: String,
    
    pub overwrite_existing: Option<bool>,
}

/// Import format enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportFormat {
    Json,
    Csv,
}

/// Validated pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedPaginationParams {
    #[validate(custom = "validate_pagination_limit")]
    pub limit: Option<i32>,
    
    #[validate(custom = "validate_pagination_offset")]
    pub offset: Option<i32>,
}

/// Validated account deletion request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedAccountDeletionRequest {
    #[validate(custom = "validate_uuid_string")]
    pub user_id: String,
    
    #[validate(custom = "validate_email")]
    pub confirmation_email: String,
    
    #[validate(length(max = 500, message = "Reason cannot exceed 500 characters"))]
    pub reason: Option<String>,
}

/// Validated password reset request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedPasswordResetRequest {
    #[validate(custom = "validate_email")]
    pub email: String,
}

/// Validated password reset confirmation request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedPasswordResetConfirmRequest {
    #[validate(length(min = 1, message = "Reset token is required"))]
    pub reset_token: String,
    
    #[validate(custom = "validate_password")]
    pub new_password: String,
}

/// Validated change password request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedChangePasswordRequest {
    #[validate(custom = "validate_uuid_string")]
    pub user_id: String,
    
    #[validate(length(min = 1, message = "Current password is required"))]
    pub current_password: String,
    
    #[validate(custom = "validate_password")]
    pub new_password: String,
}

// Custom validation functions for complex types

fn validate_import_format(format: &ImportFormat) -> Result<(), ValidationError> {
    match format {
        ImportFormat::Json | ImportFormat::Csv => Ok(()),
    }
}

// Conversion implementations from validated to regular models

impl From<ValidatedCreateUserRequest> for crate::models::CreateUserRequest {
    fn from(validated: ValidatedCreateUserRequest) -> Self {
        Self {
            email: validated.email,
            password: validated.password,
        }
    }
}

impl From<ValidatedLoginRequest> for crate::models::LoginRequest {
    fn from(validated: ValidatedLoginRequest) -> Self {
        Self {
            email: validated.email,
            password: validated.password,
            totp_code: validated.totp_code,
        }
    }
}

impl From<ValidatedUserSettings> for crate::models::UserSettings {
    fn from(validated: ValidatedUserSettings) -> Self {
        Self {
            two_factor_enabled: validated.two_factor_enabled.unwrap_or(false),
            email_notifications: validated.email_notifications.unwrap_or(true),
            privacy_mode: validated.privacy_mode.unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_validated_create_user_request() {
        // Valid request
        let valid_request = ValidatedCreateUserRequest {
            email: "test@example.com".to_string(),
            password: "SecurePass123!".to_string(),
        };
        assert!(valid_request.validate().is_ok());

        // Invalid email
        let invalid_email = ValidatedCreateUserRequest {
            email: "invalid-email".to_string(),
            password: "SecurePass123!".to_string(),
        };
        assert!(invalid_email.validate().is_err());

        // Invalid password
        let invalid_password = ValidatedCreateUserRequest {
            email: "test@example.com".to_string(),
            password: "weak".to_string(),
        };
        assert!(invalid_password.validate().is_err());
    }

    #[test]
    fn test_validated_login_request() {
        // Valid request without TOTP
        let valid_request = ValidatedLoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            totp_code: None,
        };
        assert!(valid_request.validate().is_ok());

        // Valid request with TOTP
        let valid_with_totp = ValidatedLoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            totp_code: Some("123456".to_string()),
        };
        assert!(valid_with_totp.validate().is_ok());

        // Invalid TOTP code
        let invalid_totp = ValidatedLoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            totp_code: Some("12345".to_string()), // Too short
        };
        assert!(invalid_totp.validate().is_err());
    }

    #[test]
    fn test_validated_add_to_dnp_request() {
        // Valid request
        let valid_request = ValidatedAddToDnpRequest {
            user_id: Uuid::new_v4().to_string(),
            artist_name: "Test Artist".to_string(),
            tags: Some(vec!["rock".to_string(), "classic".to_string()]),
            note: Some("Great artist but not my style".to_string()),
        };
        assert!(valid_request.validate().is_ok());

        // Invalid artist name
        let invalid_artist = ValidatedAddToDnpRequest {
            user_id: Uuid::new_v4().to_string(),
            artist_name: "".to_string(), // Empty
            tags: None,
            note: None,
        };
        assert!(invalid_artist.validate().is_err());

        // Invalid tags
        let invalid_tags = ValidatedAddToDnpRequest {
            user_id: Uuid::new_v4().to_string(),
            artist_name: "Test Artist".to_string(),
            tags: Some(vec!["".to_string()]), // Empty tag
            note: None,
        };
        assert!(invalid_tags.validate().is_err());
    }
}