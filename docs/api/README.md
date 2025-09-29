# API Documentation

## 📋 OpenAPI Specification

The complete API specification is available in OpenAPI 3.0 format:

- **OpenAPI Spec**: [openapi.yaml](./openapi.yaml)
- **Interactive Documentation**: Import the OpenAPI spec into [Swagger Editor](https://editor.swagger.io/) or [Postman](https://www.postman.com/)

## Overview

The No Drake in the House API provides RESTful endpoints for managing Do-Not-Play (DNP) lists across multiple streaming platforms. The API follows OpenAPI 3.0 specification and supports JSON request/response formats.

## Base URL

- **Production**: `https://api.nodrakeinthe.house`
- **Development**: `http://localhost:3000`

## Authentication

All API endpoints require authentication via JWT tokens with optional 2FA support.

### Headers
```
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

### Quick Start
```bash
# Register a new user
curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "secure_password123"}'

# Login and get token
curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "secure_password123"}'
```

## Platform Capabilities Matrix

Different streaming platforms support different enforcement capabilities. Use this matrix to understand what actions are available for each platform.

| Capability | Spotify | Apple Music | YouTube Music | Tidal |
|------------|---------|-------------|---------------|-------|
| **Library Management** |
| Remove Liked Songs | ✅ SUPPORTED | ✅ SUPPORTED | ❌ UNSUPPORTED | ⚠️ LIMITED |
| Remove Saved Albums | ✅ SUPPORTED | ✅ SUPPORTED | ❌ UNSUPPORTED | ⚠️ LIMITED |
| Unfollow Artists | ✅ SUPPORTED | ✅ SUPPORTED | ❌ UNSUPPORTED | ❌ UNSUPPORTED |
| **Playlist Management** |
| Remove Tracks from Playlists | ✅ SUPPORTED | ⚠️ LIMITED | ❌ UNSUPPORTED | ⚠️ LIMITED |
| Modify Collaborative Playlists | ✅ SUPPORTED | ❌ UNSUPPORTED | ❌ UNSUPPORTED | ❌ UNSUPPORTED |
| **Content Filtering** |
| Web Interface Blocking | ✅ SUPPORTED | ✅ SUPPORTED | ✅ SUPPORTED | ⚠️ LIMITED |
| Auto-Skip Functionality | ✅ SUPPORTED | ⚠️ LIMITED | ✅ SUPPORTED | ⚠️ LIMITED |
| Recommendation Filtering | ⚠️ LIMITED | ❌ UNSUPPORTED | ⚠️ LIMITED | ❌ UNSUPPORTED |
| **Advanced Features** |
| Featured Artist Detection | ✅ SUPPORTED | ✅ SUPPORTED | ⚠️ LIMITED | ⚠️ LIMITED |
| Collaboration Detection | ✅ SUPPORTED | ✅ SUPPORTED | ❌ UNSUPPORTED | ❌ UNSUPPORTED |
| Radio Seed Filtering | ⚠️ LIMITED | ❌ UNSUPPORTED | ❌ UNSUPPORTED | ❌ UNSUPPORTED |

**Legend:**
- ✅ **SUPPORTED**: Full functionality available
- ⚠️ **LIMITED**: Partial functionality or requires manual steps
- ❌ **UNSUPPORTED**: Not available due to platform limitations

## Rate Limits

API endpoints are subject to rate limiting to ensure fair usage:

- **Authentication endpoints**: 10 requests per minute per IP
- **DNP list operations**: 100 requests per minute per user
- **Enforcement operations**: 5 concurrent operations per user
- **Community list browsing**: 200 requests per minute per user

Rate limit headers are included in all responses:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1640995200
```

## Error Handling

The API uses standard HTTP status codes and returns detailed error information:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid artist ID format",
    "details": {
      "field": "artist_id",
      "expected": "UUID format",
      "received": "invalid-id"
    },
    "request_id": "req_123456789"
  }
}
```

### Common Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `AUTHENTICATION_REQUIRED` | 401 | Valid JWT token required |
| `INSUFFICIENT_PERMISSIONS` | 403 | User lacks required permissions |
| `RESOURCE_NOT_FOUND` | 404 | Requested resource does not exist |
| `VALIDATION_ERROR` | 400 | Request validation failed |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests |
| `PLATFORM_ERROR` | 502 | External platform API error |
| `MAINTENANCE_MODE` | 503 | Service temporarily unavailable |

## Pagination

List endpoints support cursor-based pagination:

```json
{
  "data": [...],
  "pagination": {
    "next_cursor": "eyJpZCI6IjEyMyJ9",
    "has_more": true,
    "total_count": 1500
  }
}
```

Query parameters:
- `cursor`: Pagination cursor from previous response
- `limit`: Number of items per page (max 100, default 20)

## Webhooks

The API supports webhooks for real-time notifications:

### Supported Events
- `enforcement.completed` - Enforcement operation finished
- `community_list.updated` - Subscribed community list changed
- `connection.expired` - Platform connection needs refresh

### Webhook Payload
```json
{
  "event": "enforcement.completed",
  "timestamp": "2024-01-15T10:30:00Z",
  "data": {
    "batch_id": "batch_123",
    "user_id": "user_456",
    "status": "completed",
    "summary": {
      "total_actions": 45,
      "successful_actions": 43,
      "failed_actions": 2
    }
  }
}
```

## SDK and Libraries

Official SDKs are available for popular programming languages:

- **JavaScript/TypeScript**: `@nodrakeinthe/api-client`
- **Python**: `nodrakeinthe-api`
- **Go**: `github.com/nodrakeinthe/go-client`
- **Rust**: `nodrakeinthe-api` (crates.io)

## Next Steps

- [Authentication Guide](./authentication.md)
- [DNP List Management](./dnp-lists.md)
- [Community Lists](./community-lists.md)
- [Enforcement Operations](./enforcement.md)
- [Platform Integrations](./platforms/)