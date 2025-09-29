# Authentication API

## Overview

The authentication system supports multiple methods including OAuth 2.0 with external providers and traditional email/password authentication with optional 2FA.

## Endpoints

### POST /v1/auth/register

Register a new user account.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "secure_password_123",
  "confirm_password": "secure_password_123",
  "terms_accepted": true
}
```

**Response (201 Created):**
```json
{
  "user": {
    "id": "user_123456",
    "email": "user@example.com",
    "created_at": "2024-01-15T10:30:00Z",
    "email_verified": false
  },
  "tokens": {
    "access_token": "eyJhbGciOiJIUzI1NiIs...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
    "expires_in": 3600,
    "token_type": "Bearer"
  }
}
```

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
    "id": "user_123456",
    "email": "user@example.com",
    "two_factor_enabled": true
  },
  "tokens": {
    "access_token": "eyJhbGciOiJIUzI1NiIs...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
    "expires_in": 3600,
    "token_type": "Bearer"
  }
}
```

### POST /v1/auth/oauth/initiate

Start OAuth flow with external provider.

**Request Body:**
```json
{
  "provider": "google",  // "google", "apple"
  "redirect_uri": "https://app.nodrakeinthe.house/auth/callback"
}
```

**Response (200 OK):**
```json
{
  "authorization_url": "https://accounts.google.com/oauth/authorize?client_id=...",
  "state": "random_state_string",
  "code_verifier": "code_challenge_verifier"  // For PKCE
}
```

### POST /v1/auth/oauth/callback

Complete OAuth flow with authorization code.

**Request Body:**
```json
{
  "provider": "google",
  "code": "authorization_code_from_provider",
  "state": "random_state_string",
  "code_verifier": "code_challenge_verifier"
}
```

**Response (200 OK):**
```json
{
  "user": {
    "id": "user_123456",
    "email": "user@gmail.com",
    "provider": "google",
    "provider_id": "google_user_id"
  },
  "tokens": {
    "access_token": "eyJhbGciOiJIUzI1NiIs...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
    "expires_in": 3600,
    "token_type": "Bearer"
  }
}
```

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
  "tokens": {
    "access_token": "eyJhbGciOiJIUzI1NiIs...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
    "expires_in": 3600,
    "token_type": "Bearer"
  }
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
    "id": "user_123456",
    "email": "user@example.com",
    "two_factor_enabled": true,
    "email_verified": true,
    "created_at": "2024-01-15T10:30:00Z",
    "settings": {
      "enforcement_aggressiveness": "moderate",
      "auto_enforcement": false,
      "notification_preferences": {
        "email": true,
        "push": false
      }
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
  "secret": "JBSWY3DPEHPK3PXP",
  "qr_code": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA...",
  "backup_codes": [
    "12345678",
    "87654321",
    "11223344"
  ]
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
  "totp_code": "123456"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "backup_codes": [
    "12345678",
    "87654321",
    "11223344"
  ]
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
  "password": "current_password",
  "totp_code": "123456"
}
```

**Response (200 OK):**
```json
{
  "success": true
}
```

## Password Management

### POST /v1/auth/password/forgot

Request password reset email.

**Request Body:**
```json
{
  "email": "user@example.com"
}
```

**Response (200 OK):**
```json
{
  "message": "If an account with this email exists, a reset link has been sent."
}
```

### POST /v1/auth/password/reset

Reset password with token from email.

**Request Body:**
```json
{
  "token": "password_reset_token",
  "password": "new_secure_password",
  "confirm_password": "new_secure_password"
}
```

**Response (200 OK):**
```json
{
  "message": "Password has been reset successfully."
}
```

### POST /v1/auth/password/change

Change password for authenticated user.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Request Body:**
```json
{
  "current_password": "old_password",
  "new_password": "new_secure_password",
  "confirm_password": "new_secure_password"
}
```

**Response (200 OK):**
```json
{
  "message": "Password changed successfully."
}
```

## Security Considerations

### JWT Token Structure

Access tokens contain the following claims:
```json
{
  "sub": "user_123456",
  "email": "user@example.com",
  "iat": 1640995200,
  "exp": 1640998800,
  "scope": "read:profile write:dnp_lists"
}
```

### Token Rotation

- Access tokens expire after 1 hour
- Refresh tokens expire after 30 days
- Refresh tokens are rotated on each use
- Old refresh tokens are invalidated immediately

### Rate Limiting

Authentication endpoints have strict rate limits:
- Login attempts: 5 per minute per IP
- Registration: 3 per minute per IP
- Password reset: 1 per minute per email
- 2FA verification: 3 per minute per user

### Security Headers

All authentication responses include security headers:
```
Strict-Transport-Security: max-age=31536000; includeSubDomains
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
```

## Error Responses

### 400 Bad Request
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Password must be at least 8 characters",
    "details": {
      "field": "password",
      "constraint": "min_length"
    }
  }
}
```

### 401 Unauthorized
```json
{
  "error": {
    "code": "INVALID_CREDENTIALS",
    "message": "Email or password is incorrect"
  }
}
```

### 429 Too Many Requests
```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many login attempts. Try again in 60 seconds.",
    "retry_after": 60
  }
}
```