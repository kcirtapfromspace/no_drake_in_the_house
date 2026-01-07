# Design Document

## Overview

This design addresses the API alignment issues between the frontend and backend to restore full core product functionality. The backend implements a robust generic OAuth system and comprehensive DNP management, while the frontend expects more specific endpoints. We'll update the frontend to work with the existing backend architecture while ensuring all core features function properly.

## Architecture

### API Endpoint Mapping

The backend provides these key endpoint patterns:
- **Authentication**: `/api/v1/auth/*` (login, register, refresh)
- **OAuth Management**: `/api/v1/auth/oauth/:provider/*` (link, unlink, accounts)
- **DNP Management**: `/api/v1/dnp/*` (list, search, add, remove, update)
- **User Management**: `/api/v1/users/*` (profile, export, delete)

The frontend currently expects:
- **Spotify-specific**: `/api/v1/spotify/*` (auth, callback, health)
- **Connections**: `/api/v1/connections` (generic connections endpoint)

### Data Flow Architecture

```
Frontend Stores → API Client Layer → Backend Handlers → Services → Database
     ↓                    ↓               ↓            ↓         ↓
  Reactive UI ← Response Mapping ← JSON Response ← Business Logic ← Data
```

## Components and Interfaces

### 1. API Client Abstraction Layer

Create a centralized API client that handles:
- Authentication token management
- Request/response formatting
- Error handling and retry logic
- Endpoint URL construction

```typescript
interface ApiClient {
  get<T>(endpoint: string): Promise<ApiResponse<T>>;
  post<T>(endpoint: string, data?: any): Promise<ApiResponse<T>>;
  put<T>(endpoint: string, data?: any): Promise<ApiResponse<T>>;
  delete<T>(endpoint: string): Promise<ApiResponse<T>>;
}

interface ApiResponse<T> {
  success: boolean;
  data?: T;
  message?: string;
  error_code?: string;
}
```

### 2. Connection Management Service

Update the connections store to use the generic OAuth endpoints:

```typescript
interface ConnectionService {
  // Maps to /api/v1/auth/oauth/accounts
  fetchConnections(): Promise<ServiceConnection[]>;
  
  // Maps to /api/v1/auth/oauth/:provider/link
  initiateConnection(provider: string): Promise<{auth_url: string}>;
  
  // Maps to /api/v1/auth/oauth/:provider/link-callback
  handleCallback(provider: string, code: string, state: string): Promise<void>;
  
  // Maps to /api/v1/auth/oauth/:provider/unlink
  disconnectService(provider: string): Promise<void>;
}
```

### 3. DNP Management Service

The DNP endpoints are correctly aligned, but we'll ensure proper error handling:

```typescript
interface DnpService {
  // Maps to /api/v1/dnp/list
  fetchDnpList(): Promise<DnpEntry[]>;
  
  // Maps to /api/v1/dnp/search
  searchArtists(query: string, limit?: number): Promise<Artist[]>;
  
  // Maps to /api/v1/dnp/list (POST)
  addArtist(request: AddToDnpRequest): Promise<DnpEntry>;
  
  // Maps to /api/v1/dnp/list/:artist_id (DELETE)
  removeArtist(artistId: string): Promise<void>;
  
  // Maps to /api/v1/dnp/list/:artist_id (PUT)
  updateEntry(artistId: string, update: UpdateDnpEntryRequest): Promise<DnpEntry>;
}
```

### 4. Enforcement Planning Service

Create new endpoints for enforcement functionality:

```typescript
interface EnforcementService {
  // New endpoint: /api/v1/enforcement/preview
  previewEnforcement(providers: string[]): Promise<EnforcementPreview>;
  
  // New endpoint: /api/v1/enforcement/execute
  executeEnforcement(plan: EnforcementPlan): Promise<EnforcementResult>;
  
  // New endpoint: /api/v1/enforcement/history
  getEnforcementHistory(): Promise<EnforcementAction[]>;
}
```

## Data Models

### Standardized API Response Format

All API responses will follow this format:

```typescript
interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  message?: string;
  error_code?: string;
  timestamp?: string;
}
```

### Service Connection Model

```typescript
interface ServiceConnection {
  id: string;
  provider: 'spotify' | 'apple' | 'youtube';
  provider_user_id?: string;
  scopes: string[];
  status: 'active' | 'expired' | 'error' | 'pending';
  expires_at?: string;
  last_health_check?: string;
  created_at: string;
  updated_at: string;
  error_code?: string;
  error_message?: string;
}
```

### DNP Entry Model

```typescript
interface DnpEntry {
  artist: Artist;
  tags: string[];
  note?: string;
  created_at: string;
  updated_at: string;
}

interface Artist {
  id: string;
  canonical_name: string;
  external_ids: {
    spotify?: string;
    apple?: string;
    musicbrainz?: string;
    isni?: string;
  };
  metadata: {
    image?: string;
    genres?: string[];
    popularity?: number;
  };
}
```

## Error Handling

### Frontend Error Handling Strategy

1. **Network Errors**: Retry with exponential backoff
2. **Authentication Errors**: Redirect to login or refresh token
3. **Validation Errors**: Display field-specific messages
4. **Server Errors**: Show generic error with retry option
5. **Offline State**: Cache data and sync when reconnected

### Error Response Format

```typescript
interface ErrorResponse {
  success: false;
  message: string;
  error_code?: string;
  field_errors?: Record<string, string>;
  retry_after?: number;
}
```

## Testing Strategy

### Unit Testing
- Test each store action independently
- Mock API responses for different scenarios
- Test error handling and retry logic
- Validate data transformations

### Integration Testing
- Test complete user flows (login → connect → add DNP → enforce)
- Test OAuth flows with mock providers
- Test error scenarios and recovery
- Test offline/online state transitions

### End-to-End Testing
- Test full application workflows
- Test with real backend services
- Test cross-browser compatibility
- Test responsive design on different devices

## Performance Considerations

### Frontend Optimizations
- Implement request deduplication
- Cache frequently accessed data
- Use optimistic updates for better UX
- Implement virtual scrolling for large lists

### Backend Optimizations
- Use database connection pooling
- Implement query optimization
- Add response caching where appropriate
- Use batch operations for bulk actions

## Security Considerations

### Token Management
- Store tokens securely in httpOnly cookies or secure storage
- Implement automatic token refresh
- Clear tokens on logout or expiration
- Use CSRF protection for state-changing operations

### API Security
- Validate all inputs on both frontend and backend
- Implement rate limiting
- Use HTTPS for all communications
- Sanitize error messages to prevent information leakage