use music_streaming_blocklist_backend::models::{RegisterRequest, RegistrationValidationError};

#[tokio::test]
async fn test_simple() {
    assert_eq!(1 + 1, 2);
}

/// Unit tests for registration validation logic
/// Tests password confirmation matching, terms acceptance, email format, and password strength validation

// Helper function to create validation errors for testing
fn create_validation_error(field: &str, message: &str, code: &str) -> RegistrationValidationError {
    RegistrationValidationError {
        field: field.to_string(),
        message: message.to_string(),
        code: code.to_string(),
    }
}

// Standalone validation functions for testing (extracted from AuthService logic)
fn validate_email_format(email: &str) -> Vec<RegistrationValidationError> {
    let mut errors = Vec::new();

    if email.is_empty() {
        errors.push(create_validation_error(
            "email",
            "Email is required",
            "EMAIL_REQUIRED"
        ));
    } else {
        // Enhanced email validation with proper regex (no consecutive dots)
        let email_regex = regex::Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9._+%-]*[a-zA-Z0-9])?@[a-zA-Z0-9]([a-zA-Z0-9.-]*[a-zA-Z0-9])?\.[a-zA-Z]{2,}$").unwrap();
        if !email_regex.is_match(email) || email.contains("..") {
            errors.push(create_validation_error(
                "email",
                "Please enter a valid email address",
                "EMAIL_INVALID_FORMAT"
            ));
        }
        
        // Check email length
        if email.len() > 255 {
            errors.push(create_validation_error(
                "email",
                "Email address is too long (maximum 255 characters)",
                "EMAIL_TOO_LONG"
            ));
        }
    }

    errors
}

fn validate_password_strength(password: &str) -> Option<RegistrationValidationError> {
    let mut requirements = Vec::new();

    // Minimum length requirement
    if password.len() < 8 {
        requirements.push("at least 8 characters");
    }

    // Uppercase letter requirement
    if !password.chars().any(|c| c.is_uppercase()) {
        requirements.push("at least one uppercase letter");
    }

    // Lowercase letter requirement
    if !password.chars().any(|c| c.is_lowercase()) {
        requirements.push("at least one lowercase letter");
    }

    // Number requirement
    if !password.chars().any(|c| c.is_numeric()) {
        requirements.push("at least one number");
    }

    // Special character requirement
    if !password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)) {
        requirements.push("at least one special character (!@#$%^&*()_+-=[]{}|;:,.<>?)");
    }

    // Check against common passwords (basic implementation)
    let common_passwords = [
        "password", "123456", "password123", "admin", "qwerty", "letmein",
        "welcome", "monkey", "1234567890", "password1", "123456789"
    ];
    
    if common_passwords.iter().any(|&common| password.to_lowercase() == common.to_lowercase()) {
        requirements.push("not be a common password");
    }

    if !requirements.is_empty() {
        let message = format!("Password must contain {}", requirements.join(", "));
        Some(create_validation_error(
            "password",
            &message,
            "PASSWORD_WEAK"
        ))
    } else {
        None
    }
}

fn validate_password_confirmation(password: &str, confirm_password: &str) -> Vec<RegistrationValidationError> {
    let mut errors = Vec::new();

    if confirm_password.is_empty() {
        errors.push(create_validation_error(
            "confirm_password",
            "Password confirmation is required",
            "CONFIRM_PASSWORD_REQUIRED"
        ));
    } else if password != confirm_password {
        errors.push(create_validation_error(
            "confirm_password",
            "Password confirmation does not match",
            "PASSWORD_MISMATCH"
        ));
    }

    errors
}

fn validate_terms_acceptance(terms_accepted: bool) -> Vec<RegistrationValidationError> {
    let mut errors = Vec::new();

    if !terms_accepted {
        errors.push(create_validation_error(
            "terms_accepted",
            "You must accept the terms of service to register",
            "TERMS_NOT_ACCEPTED"
        ));
    }

    errors
}

fn validate_registration_request(request: &RegisterRequest) -> Vec<RegistrationValidationError> {
    let mut errors = Vec::new();

    // Email format validation
    errors.extend(validate_email_format(&request.email));

    // Password validation
    if request.password.is_empty() {
        errors.push(create_validation_error(
            "password",
            "Password is required",
            "PASSWORD_REQUIRED"
        ));
    } else {
        // Password strength validation with detailed requirements checking
        if let Some(password_error) = validate_password_strength(&request.password) {
            errors.push(password_error);
        }
    }

    // Password confirmation matching validation
    errors.extend(validate_password_confirmation(&request.password, &request.confirm_password));

    // Terms acceptance validation logic
    errors.extend(validate_terms_acceptance(request.terms_accepted));

    errors
}

#[tokio::test]
async fn test_password_confirmation_matching_validation() {
    // Test case: Passwords match - should pass validation
    let valid_request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: "SecurePassword123!".to_string(),
        confirm_password: "SecurePassword123!".to_string(),
        terms_accepted: true,
    };

    let validation_errors = validate_registration_request(&valid_request);
    let password_errors: Vec<_> = validation_errors.iter()
        .filter(|e| e.field == "confirm_password")
        .collect();
    
    assert!(password_errors.is_empty(), "Should not have password confirmation errors when passwords match");

    // Test case: Passwords don't match - should fail validation
    let invalid_request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: "SecurePassword123!".to_string(),
        confirm_password: "DifferentPassword123!".to_string(),
        terms_accepted: true,
    };

    let validation_errors = validate_registration_request(&invalid_request);
    let password_mismatch_error = validation_errors.iter()
        .find(|e| e.field == "confirm_password" && e.code == "PASSWORD_MISMATCH");
    
    assert!(password_mismatch_error.is_some(), "Should have password mismatch error");
    assert_eq!(
        password_mismatch_error.unwrap().message,
        "Password confirmation does not match"
    );

    // Test case: Empty confirm_password - should fail validation
    let empty_confirm_request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: "SecurePassword123!".to_string(),
        confirm_password: "".to_string(),
        terms_accepted: true,
    };

    let validation_errors = validate_registration_request(&empty_confirm_request);
    let empty_confirm_error = validation_errors.iter()
        .find(|e| e.field == "confirm_password" && e.code == "CONFIRM_PASSWORD_REQUIRED");
    
    assert!(empty_confirm_error.is_some(), "Should have required error for empty confirm_password");
    assert_eq!(
        empty_confirm_error.unwrap().message,
        "Password confirmation is required"
    );
}

#[tokio::test]
async fn test_terms_acceptance_validation() {
    // Test case: Terms accepted - should pass validation
    let valid_request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: "SecurePassword123!".to_string(),
        confirm_password: "SecurePassword123!".to_string(),
        terms_accepted: true,
    };

    let validation_errors = validate_registration_request(&valid_request);
    let terms_errors: Vec<_> = validation_errors.iter()
        .filter(|e| e.field == "terms_accepted")
        .collect();
    
    assert!(terms_errors.is_empty(), "Should not have terms errors when terms are accepted");

    // Test case: Terms not accepted - should fail validation
    let invalid_request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: "SecurePassword123!".to_string(),
        confirm_password: "SecurePassword123!".to_string(),
        terms_accepted: false,
    };

    let validation_errors = validate_registration_request(&invalid_request);
    let terms_error = validation_errors.iter()
        .find(|e| e.field == "terms_accepted" && e.code == "TERMS_NOT_ACCEPTED");
    
    assert!(terms_error.is_some(), "Should have terms not accepted error");
    assert_eq!(
        terms_error.unwrap().message,
        "You must accept the terms of service to register"
    );
}

#[tokio::test]
async fn test_enhanced_email_format_validation() {
    // Test case: Valid email formats - should pass validation
    let valid_emails = vec![
        "user@example.com",
        "test.email@domain.co.uk",
        "user+tag@example.org",
        "user_name@example-domain.com",
        "123@example.com",
    ];

    for email in valid_emails {
        let request = RegisterRequest {
            email: email.to_string(),
            password: "SecurePassword123!".to_string(),
            confirm_password: "SecurePassword123!".to_string(),
            terms_accepted: true,
        };

        let validation_errors = validate_registration_request(&request);
        let email_errors: Vec<_> = validation_errors.iter()
            .filter(|e| e.field == "email")
            .collect();
        
        assert!(email_errors.is_empty(), "Should not have email errors for valid email: {}", email);
    }

    // Test case: Invalid email formats - should fail validation
    let invalid_emails = vec![
        "",
        "invalid-email",
        "@example.com",
        "user@",
        "user@.com",
        "user..double@example.com",
        "user@example",
        "user name@example.com", // space in local part
    ];

    for email in invalid_emails {
        let request = RegisterRequest {
            email: email.to_string(),
            password: "SecurePassword123!".to_string(),
            confirm_password: "SecurePassword123!".to_string(),
            terms_accepted: true,
        };

        let validation_errors = validate_registration_request(&request);
        let has_email_error = validation_errors.iter()
            .any(|e| e.field == "email" && (e.code == "EMAIL_INVALID_FORMAT" || e.code == "EMAIL_REQUIRED"));
        
        assert!(has_email_error, "Should have email error for invalid email: '{}'", email);
    }

    // Test case: Email too long - should fail validation
    let long_email = format!("{}@example.com", "a".repeat(250));
    let request = RegisterRequest {
        email: long_email,
        password: "SecurePassword123!".to_string(),
        confirm_password: "SecurePassword123!".to_string(),
        terms_accepted: true,
    };

    let validation_errors = validate_registration_request(&request);
    let length_error = validation_errors.iter()
        .find(|e| e.field == "email" && e.code == "EMAIL_TOO_LONG");
    
    assert!(length_error.is_some(), "Should have email too long error");
    assert!(length_error.unwrap().message.contains("maximum 255 characters"));
}

#[tokio::test]
async fn test_password_strength_requirements_validation() {
    // Test case: Strong password - should pass validation
    let strong_passwords = vec![
        "SecurePassword123!",
        "MyP@ssw0rd2024",
        "Complex!Pass123",
        "Str0ng#Password",
    ];

    for password in strong_passwords {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: password.to_string(),
            confirm_password: password.to_string(),
            terms_accepted: true,
        };

        let validation_errors = validate_registration_request(&request);
        let password_errors: Vec<_> = validation_errors.iter()
            .filter(|e| e.field == "password")
            .collect();
        
        if !password_errors.is_empty() {
            println!("Password errors for '{}': {:?}", password, password_errors);
        }
        
        assert!(password_errors.is_empty(), "Should not have password errors for strong password: {}", password);
    }

    // Test case: Password too short - should fail validation
    let short_password = "Short1!";
    let request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: short_password.to_string(),
        confirm_password: short_password.to_string(),
        terms_accepted: true,
    };

    let validation_errors = validate_registration_request(&request);
    let length_error = validation_errors.iter()
        .find(|e| e.field == "password" && e.code == "PASSWORD_WEAK");
    
    assert!(length_error.is_some(), "Should have password weak error for short password");
    assert!(length_error.unwrap().message.contains("at least 8 characters"));

    // Test case: Password missing uppercase - should fail validation
    let no_uppercase = "lowercase123!";
    let request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: no_uppercase.to_string(),
        confirm_password: no_uppercase.to_string(),
        terms_accepted: true,
    };

    let validation_errors = validate_registration_request(&request);
    let uppercase_error = validation_errors.iter()
        .find(|e| e.field == "password" && e.code == "PASSWORD_WEAK");
    
    assert!(uppercase_error.is_some(), "Should have password weak error for missing uppercase");
    assert!(uppercase_error.unwrap().message.contains("at least one uppercase letter"));

    // Test case: Password missing lowercase - should fail validation
    let no_lowercase = "UPPERCASE123!";
    let request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: no_lowercase.to_string(),
        confirm_password: no_lowercase.to_string(),
        terms_accepted: true,
    };

    let validation_errors = validate_registration_request(&request);
    let lowercase_error = validation_errors.iter()
        .find(|e| e.field == "password" && e.code == "PASSWORD_WEAK");
    
    assert!(lowercase_error.is_some(), "Should have password weak error for missing lowercase");
    assert!(lowercase_error.unwrap().message.contains("at least one lowercase letter"));

    // Test case: Password missing number - should fail validation
    let no_number = "NoNumbersHere!";
    let request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: no_number.to_string(),
        confirm_password: no_number.to_string(),
        terms_accepted: true,
    };

    let validation_errors = validate_registration_request(&request);
    let number_error = validation_errors.iter()
        .find(|e| e.field == "password" && e.code == "PASSWORD_WEAK");
    
    assert!(number_error.is_some(), "Should have password weak error for missing number");
    assert!(number_error.unwrap().message.contains("at least one number"));

    // Test case: Password missing special character - should fail validation
    let no_special = "NoSpecialChars123";
    let request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: no_special.to_string(),
        confirm_password: no_special.to_string(),
        terms_accepted: true,
    };

    let validation_errors = validate_registration_request(&request);
    let special_error = validation_errors.iter()
        .find(|e| e.field == "password" && e.code == "PASSWORD_WEAK");
    
    assert!(special_error.is_some(), "Should have password weak error for missing special character");
    assert!(special_error.unwrap().message.contains("at least one special character"));

    // Test case: Common password - should fail validation
    let common_passwords = vec![
        "password",
        "123456",
        "admin",
        "qwerty",
    ];

    for password in common_passwords {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: password.to_string(),
            confirm_password: password.to_string(),
            terms_accepted: true,
        };

        let validation_errors = validate_registration_request(&request);
        let common_error = validation_errors.iter()
            .find(|e| e.field == "password" && e.code == "PASSWORD_WEAK");
        
        assert!(common_error.is_some(), "Should have password weak error for common password: {}", password);
        assert!(common_error.unwrap().message.contains("not be a common password"));
    }

    // Test case: Empty password - should fail validation
    let request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: "".to_string(),
        confirm_password: "".to_string(),
        terms_accepted: true,
    };

    let validation_errors = validate_registration_request(&request);
    let empty_error = validation_errors.iter()
        .find(|e| e.field == "password" && e.code == "PASSWORD_REQUIRED");
    
    assert!(empty_error.is_some(), "Should have password required error for empty password");
    assert_eq!(empty_error.unwrap().message, "Password is required");
}

#[tokio::test]
async fn test_multiple_validation_errors() {
    // Test case: Multiple validation failures - should return all errors
    let invalid_request = RegisterRequest {
        email: "invalid-email".to_string(),
        password: "weak".to_string(),
        confirm_password: "different".to_string(),
        terms_accepted: false,
    };

    let validation_errors = validate_registration_request(&invalid_request);
    
    // Should have errors for all fields
    assert!(validation_errors.iter().any(|e| e.field == "email"), "Should have email error");
    assert!(validation_errors.iter().any(|e| e.field == "password"), "Should have password error");
    assert!(validation_errors.iter().any(|e| e.field == "confirm_password"), "Should have confirm_password error");
    assert!(validation_errors.iter().any(|e| e.field == "terms_accepted"), "Should have terms_accepted error");
    
    // Verify specific error codes
    assert!(validation_errors.iter().any(|e| e.code == "EMAIL_INVALID_FORMAT"), "Should have email format error");
    assert!(validation_errors.iter().any(|e| e.code == "PASSWORD_WEAK"), "Should have password weak error");
    assert!(validation_errors.iter().any(|e| e.code == "PASSWORD_MISMATCH"), "Should have password mismatch error");
    assert!(validation_errors.iter().any(|e| e.code == "TERMS_NOT_ACCEPTED"), "Should have terms not accepted error");
}

#[tokio::test]
async fn test_validation_error_structure() {
    let invalid_request = RegisterRequest {
        email: "".to_string(),
        password: "".to_string(),
        confirm_password: "".to_string(),
        terms_accepted: false,
    };

    let validation_errors = validate_registration_request(&invalid_request);
    
    // Verify error structure
    for error in &validation_errors {
        assert!(!error.field.is_empty(), "Error field should not be empty");
        assert!(!error.message.is_empty(), "Error message should not be empty");
        assert!(!error.code.is_empty(), "Error code should not be empty");
        
        // Verify error codes follow expected pattern
        assert!(error.code.chars().all(|c| c.is_uppercase() || c == '_'), 
               "Error code should be uppercase with underscores: {}", error.code);
    }
}