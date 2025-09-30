# Authentication API

## Overview

The authentication system supports multiple methods including OAuth 2.0 with external providers and traditional email/password authentication with optional 2FA.

## Endpoints

### POST /v1/auth/register

Register a new user account with enhanced validation and automatic login.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "secure_password_123",
  "confirm_password": "secure_password_123",
  "terms_accepted": true
}
```

**Field Requirements:**
- `email`: Valid email address format, maximum 255 characters
- `password`: Minimum 8 characters, must include uppercase, lowercase, number, and special character
- `confirm_password`: Must exactly match the password field
- `terms_accepted`: Must be `true` to proceed with registration

**Response (201 Created):**
```json
{
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "email_verified": false,
    "totp_enabled": false,
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z",
    "last_login": null,
    "settings": {
      "two_factor_enabled": false,
      "email_notifications": true,
      "privacy_mode": false
    }
  },
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
}
```

**Note:** The registration endpoint automatically logs in the user upon successful registration, returning access and refresh tokens. This eliminates the need for a separate login step after registration.

**Validation Error Codes:**
- `PASSWORD_MISMATCH`: Password confirmation does not match
- `TERMS_NOT_ACCEPTED`: Terms of service must be accepted
- `EMAIL_ALREADY_REGISTERED`: Email address is already in use
- `WEAK_PASSWORD`: Password does not meet security requirements
- `INVALID_EMAIL_FORMAT`: Email format is invalid
- `PASSWORD_TOO_SHORT`: Password must be at least 8 characters

### POST /v1/auth/login

Authenticate with email and password.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "secure_password_123",
  "totp_code": "123456"  // Optional, required if 2FA enabled
}
```

**Response (200 OK):**
```json
{
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "email_verified": true,
    "totp_enabled": true,
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z",
    "last_login": "2024-01-15T11:30:00Z",
    "settings": {
      "two_factor_enabled": true,
      "email_notifications": true,
      "privacy_mode": false
    }
  },
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
}
```

### OAuth Authentication (Future Implementation)

OAuth authentication with external providers (Google, Apple) is planned for future implementation. Currently, only email/password authentication is supported.

For Spotify integration, OAuth is handled through the service connection endpoints rather than general authentication. See the service connections documentation for details on connecting Spotify accounts.

### POST /v1/auth/refresh

Refresh access token using refresh token.

**Request Body:**
```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
}
```

**Response (200 OK):**
```json
{
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "email_verified": true,
    "totp_enabled": false,
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z",
    "last_login": "2024-01-15T11:30:00Z",
    "settings": {
      "two_factor_enabled": false,
      "email_notifications": true,
      "privacy_mode": false
    }
  },
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
}
```

### POST /v1/auth/logout

Invalidate current session and tokens.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (204 No Content)**

### GET /v1/auth/me

Get current user information.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
{
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "email_verified": true,
    "totp_enabled": true,
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z",
    "last_login": "2024-01-15T11:30:00Z",
    "settings": {
      "two_factor_enabled": true,
      "email_notifications": true,
      "privacy_mode": false
    }
  }
}
```

## Two-Factor Authentication (2FA)

### POST /v1/auth/2fa/setup

Enable 2FA for the current user.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "secret": "JBSWY3DPEHPK3PXP",
    "qr_code_url": "otpauth://totp/NodrakeInTheHouse:user@example.com?secret=JBSWY3DPEHPK3PXP&issuer=NodrakeInTheHouse",
    "backup_codes": [
      "12345678",
      "87654321",
      "11223344"
    ]
  }
}
```

### POST /v1/auth/2fa/verify

Verify and activate 2FA setup.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Request Body:**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "totp_code": "123456"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "message": "2FA enabled successfully"
}
```

### POST /v1/auth/2fa/disable

Disable 2FA for the current user.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Request Body:**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "totp_code": "123456"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "message": "2FA disabled successfully"
}
```

## Password Management (Future Implementation)

Password reset and change functionality is planned for future implementation. The backend service layer includes password reset logic, but the API endpoints are not yet exposed.

**Planned Features:**
- Password reset via email
- Password change for authenticated users
- Secure token-based password reset flow

For now, users who forget their passwords will need to contact support or create a new account.

## Security Considerations

### JWT Token Structure

Access tokens contain the following claims:
```json
{
  "sub": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "exp": 1640998800,
  "iat": 1640995200,
  "jti": "550e8400-e29b-41d4-a716-446655440001",
  "token_type": "Access",
  "scopes": ["read", "write"]
}
```

Refresh tokens have a similar structure but with different token_type and scopes:
```json
{
  "sub": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "exp": 1643587200,
  "iat": 1640995200,
  "jti": "550e8400-e29b-41d4-a716-446655440002",
  "token_type": "Refresh",
  "scopes": ["refresh"]
}
```

### Token Management

- Access tokens expire after 24 hours (configurable)
- Refresh tokens expire after 30 days (configurable)
- Each token has a unique JWT ID (jti) for tracking and revocation
- Token families are used for refresh token rotation
- Revoked tokens are tracked in the database for security

### Rate Limiting

Authentication endpoints have strict rate limits to prevent abuse:

**Registration Endpoint:**
- 3 attempts per minute per IP address
- Failed registration attempts: 5 per 15 minutes per IP address
- Rate limiting is enforced using Redis-based storage

**Login Endpoint:**
- 5 attempts per minute per IP address
- Additional rate limiting may apply for repeated failed attempts

**Password Reset:**
- 1 request per minute per email address

**2FA Verification:**
- 3 attempts per minute per user

When rate limits are exceeded, the API returns a 429 status code with retry timing information in the response details.

### Security Headers

All authentication responses include security headers:
```
Strict-Transport-Security: max-age=31536000; includeSubDomains
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
```

## Error Responses

All error responses follow a consistent structure with detailed error information and correlation IDs for debugging.

### Registration Validation Errors (400 Bad Request)

**Password Confirmation Mismatch:**
```json
{
  "error": "Password confirmation does not match",
  "error_code": "PASSWORD_MISMATCH",
  "message": "Password confirmation does not match",
  "details": null,
  "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

**Terms Not Accepted:**
```json
{
  "error": "Terms of service must be accepted",
  "error_code": "TERMS_NOT_ACCEPTED",
  "message": "You must accept the terms of service to register",
  "details": null,
  "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

**Multiple Validation Errors:**
```json
{
  "error": "Registration validation failed",
  "error_code": "REGISTRATION_VALIDATION_ERROR",
  "message": "Registration validation failed. Please check your input and try again.",
  "details": {
    "validation_errors": [
      {
        "field": "password",
        "message": "Password must be at least 8 characters long",
        "code": "PASSWORD_TOO_SHORT"
      },
      {
        "field": "email",
        "message": "Invalid email format",
        "code": "INVALID_EMAIL_FORMAT"
      }
    ]
  },
  "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

**Weak Password:**
```json
{
  "error": "Password does not meet security requirements",
  "error_code": "WEAK_PASSWORD",
  "message": "Password does not meet security requirements",
  "details": {
    "password_requirements": [
      "Must contain at least one uppercase letter",
      "Must contain at least one number",
      "Must contain at least one special character"
    ]
  },
  "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

### Authentication Errors (401 Unauthorized)
```json
{
  "error": "Invalid credentials",
  "error_code": "AUTH_INVALID_CREDENTIALS",
  "message": "Invalid email or password",
  "details": null,
  "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

### Conflict Errors (409 Conflict)
```json
{
  "error": "Email already registered",
  "error_code": "EMAIL_ALREADY_REGISTERED",
  "message": "An account with this email address already exists",
  "details": null,
  "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

### Rate Limiting (429 Too Many Requests)
```json
{
  "error": "Rate limit exceeded",
  "error_code": "RATE_LIMIT_EXCEEDED",
  "message": "Too many requests, please try again later",
  "details": {
    "retry_after_seconds": 60
  },
  "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2024-01-15T10:30:00Z"
}
```