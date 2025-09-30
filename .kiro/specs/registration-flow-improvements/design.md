# Design Document

## Overview

This design addresses the registration flow improvements for No Drake in the House by implementing server-side validation for password confirmation and terms acceptance, aligning API documentation with implementation, adding auto-login functionality, and enhancing error handling. The solution maintains backward compatibility while adding the missing features described in the API documentation.

## Architecture

The registration flow improvements will be implemented across three main layers:

1. **API Layer** - Enhanced request/response models and validation
2. **Service Layer** - Extended business logic for validation and auto-login
3. **Frontend Layer** - Improved form handling and user experience

### Current vs. Proposed Flow

**Current Flow:**
```
Frontend Form → RegisterRequest{email, password} → AuthService → Database → AuthResponse
```

**Proposed Flow:**
```
Frontend Form → EnhancedRegisterRequest{email, password, confirm_password, terms_accepted} 
→ Enhanced AuthService (with validation) → Database → AuthResponse (with auto-login)
```

## Components and Interfaces

### 1. Enhanced Registration Request Model

**File:** `backend/src/models/user.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub confirm_password: String,
    pub terms_accepted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationValidationError {
    pub field: String,
    pub message: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationErrorResponse {
    pub errors: Vec<RegistrationValidationError>,
    pub message: String,
}
```

### 2. Enhanced Authentication Service

**File:** `backend/src/services/auth.rs`

The `AuthService::register` method will be enhanced to:
- Validate password confirmation matches
- Validate terms acceptance
- Provide detailed field-specific error messages
- Automatically generate and return tokens (auto-login)

```rust
impl AuthService {
    pub async fn register(&self, request: RegisterRequest) -> Result<AuthResponse> {
        // 1. Validate all fields with detailed error collection
        let mut validation_errors = Vec::new();
        
        // 2. Email validation (enhanced)
        // 3. Password validation (enhanced with strength checks)
        // 4. Password confirmation validation (NEW)
        // 5. Terms acceptance validation (NEW)
        
        // 6. If validation fails, return structured errors
        // 7. If validation passes, create user and auto-login
    }
    
    fn validate_registration_request(&self, request: &RegisterRequest) -> Vec<RegistrationValidationError> {
        // Comprehensive validation logic
    }
    
    fn validate_password_strength(&self, password: &str) -> Option<RegistrationValidationError> {
        // Enhanced password validation
    }
}
```

### 3. Enhanced Registration Handler

**File:** `backend/src/handlers/auth.rs`

```rust
pub async fn register_handler(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>)> {
    // Enhanced error handling with field-specific messages
    // Rate limiting implementation
    // Structured logging for security events
}
```

### 4. Frontend Registration Form Enhancement

**File:** `frontend/src/lib/components/Register.svelte`

Enhanced form with:
- Real-time validation feedback
- Password strength indicator
- Terms acceptance checkbox
- Improved error display
- Loading states and success handling

### 5. Frontend Authentication Store Enhancement

**File:** `frontend/src/lib/stores/auth.ts`

Enhanced registration action with:
- Auto-login after successful registration
- Structured error handling
- Token storage and state management

## Data Models

### Registration Request Validation

```typescript
interface RegistrationValidation {
  email: {
    required: boolean;
    format: 'email';
    maxLength: 255;
  };
  password: {
    required: boolean;
    minLength: 8;
    requireUppercase: boolean;
    requireLowercase: boolean;
    requireNumber: boolean;
    requireSpecialChar: boolean;
  };
  confirmPassword: {
    required: boolean;
    mustMatch: 'password';
  };
  termsAccepted: {
    required: boolean;
    mustBeTrue: boolean;
  };
}
```

### Enhanced Error Response Structure

```json
{
  "error": {
    "code": "REGISTRATION_VALIDATION_ERROR",
    "message": "Registration validation failed",
    "details": {
      "errors": [
        {
          "field": "confirm_password",
          "message": "Password confirmation does not match",
          "code": "PASSWORD_MISMATCH"
        },
        {
          "field": "terms_accepted",
          "message": "You must accept the terms of service",
          "code": "TERMS_NOT_ACCEPTED"
        }
      ]
    }
  }
}
```

## Error Handling

### Server-Side Error Categories

1. **Validation Errors** - Field-specific validation failures
2. **Business Logic Errors** - Email already exists, etc.
3. **System Errors** - Database failures, internal errors
4. **Security Errors** - Rate limiting, suspicious activity

### Error Response Strategy

```rust
#[derive(Debug, thiserror::Error)]
pub enum RegistrationError {
    #[error("Validation failed")]
    ValidationError { errors: Vec<RegistrationValidationError> },
    
    #[error("Email already registered")]
    EmailAlreadyExists,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded { retry_after: u64 },
    
    #[error("Internal server error")]
    InternalError { message: Option<String> },
}
```

### Frontend Error Handling

```typescript
interface RegistrationFormState {
  loading: boolean;
  errors: Record<string, string>;
  globalError: string | null;
  success: boolean;
}

// Error display strategy:
// - Field-specific errors shown inline
// - Global errors shown at form level
// - Success state triggers auto-redirect
```

## Testing Strategy

### Backend Testing

1. **Unit Tests** - `backend/tests/auth_service_registration_tests.rs`
   - Password confirmation validation
   - Terms acceptance validation
   - Email format validation
   - Password strength validation
   - Error message accuracy

2. **Integration Tests** - `backend/tests/registration_integration_tests.rs`
   - Complete registration flow
   - Auto-login functionality
   - Rate limiting behavior
   - Database transaction integrity

3. **Security Tests**
   - SQL injection prevention
   - Rate limiting effectiveness
   - Password hashing verification
   - Token generation security

### Frontend Testing

1. **Component Tests** - Registration form validation
2. **Integration Tests** - Complete registration flow
3. **E2E Tests** - User journey from form to dashboard

### Test Scenarios

```rust
#[cfg(test)]
mod registration_tests {
    // Valid registration with auto-login
    async fn test_successful_registration_with_auto_login()
    
    // Password mismatch validation
    async fn test_password_confirmation_mismatch()
    
    // Terms not accepted validation
    async fn test_terms_not_accepted()
    
    // Email already exists handling
    async fn test_duplicate_email_registration()
    
    // Rate limiting behavior
    async fn test_registration_rate_limiting()
    
    // Password strength validation
    async fn test_password_strength_requirements()
}
```

## Security Considerations

### Password Security

- Minimum 8 characters (existing)
- Require uppercase, lowercase, number, special character
- Check against common password lists
- Bcrypt hashing with 12+ rounds (existing)

### Rate Limiting

```rust
// Registration rate limits
const REGISTRATION_RATE_LIMIT: RateLimit = RateLimit {
    requests: 3,
    window: Duration::minutes(1),
    scope: RateLimitScope::IpAddress,
};

// Failed attempt tracking
const FAILED_REGISTRATION_LIMIT: RateLimit = RateLimit {
    requests: 5,
    window: Duration::minutes(15),
    scope: RateLimitScope::IpAddress,
};
```

### Input Validation

- Server-side validation for all fields
- SQL injection prevention (using SQLx parameterized queries)
- XSS prevention (proper serialization)
- CSRF protection (SameSite cookies)

### Audit Logging

```rust
// Security events to log
enum RegistrationSecurityEvent {
    SuccessfulRegistration { user_id: Uuid, ip: String },
    FailedRegistration { email: String, reason: String, ip: String },
    RateLimitExceeded { ip: String, attempts: u32 },
    SuspiciousActivity { ip: String, details: String },
}
```

## Implementation Plan

### Phase 1: Backend Model Updates
1. Update `RegisterRequest` struct with new fields
2. Add validation error types
3. Update API documentation

### Phase 2: Service Layer Enhancement
1. Implement enhanced validation logic
2. Add auto-login functionality
3. Improve error handling and logging

### Phase 3: Handler Updates
1. Update registration handler
2. Add rate limiting middleware
3. Implement structured error responses

### Phase 4: Frontend Enhancement
1. Update registration form component
2. Enhance authentication store
3. Improve error display and user feedback

### Phase 5: Testing and Documentation
1. Comprehensive test coverage
2. Update API documentation
3. Security testing and validation

## Performance Considerations

### Database Operations
- Use database transactions for user creation
- Implement connection pooling (existing)
- Add database indexes for email lookups (existing)

### Validation Performance
- Cache password strength validation rules
- Optimize regex patterns for email validation
- Batch validation operations where possible

### Rate Limiting Storage
- Use Redis for rate limiting counters (existing infrastructure)
- Implement sliding window rate limiting
- Clean up expired rate limit entries

## Monitoring and Observability

### Metrics to Track
- Registration success/failure rates
- Validation error frequencies by field
- Rate limiting trigger frequency
- Auto-login success rates
- Password strength distribution

### Logging Strategy
```rust
// Structured logging for registration events
tracing::info!(
    user_id = %user.id,
    email = %user.email,
    auto_login = true,
    validation_errors = ?validation_errors,
    "User registration completed"
);
```

### Health Checks
- Registration endpoint availability
- Database connectivity for user operations
- Rate limiting service health
- Token generation service health