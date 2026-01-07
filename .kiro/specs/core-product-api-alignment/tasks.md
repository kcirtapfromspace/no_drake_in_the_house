# Implementation Plan

- [x] 1. Create centralized API client and error handling
  - Create shared API client with consistent request/response handling
  - Implement standardized error handling with retry logic
  - Add authentication token management and automatic refresh
  - Create response type definitions and validation
  - _Requirements: 1.1, 1.4, 5.1, 5.2, 5.3_

- [ ] 2. Fix connections store to use generic OAuth endpoints
  - [x] 2.1 Update connections store to use /api/v1/auth/oauth endpoints
    - Replace Spotify-specific endpoints with generic OAuth provider endpoints
    - Update initiateSpotifyAuth to use /api/v1/auth/oauth/spotify/link
    - Update handleSpotifyCallback to use /api/v1/auth/oauth/spotify/link-callback
    - Update disconnectSpotify to use /api/v1/auth/oauth/spotify/unlink
    - _Requirements: 1.2, 3.1, 3.2, 3.4_

  - [ ] 2.2 Update connections fetching to use OAuth accounts endpoint
    - Change fetchConnections to call /api/v1/auth/oauth/accounts
    - Update ServiceConnection interface to match backend OAuth account model
    - Add proper error handling for connection failures
    - Update connection status mapping and health checking
    - _Requirements: 1.3, 3.3, 3.5_

- [ ] 3. Verify and fix DNP management endpoints
  - [ ] 3.1 Test DNP list endpoints and fix any issues
    - Verify /api/v1/dnp/list GET endpoint works correctly
    - Test artist search endpoint /api/v1/dnp/search with query parameters
    - Ensure proper error handling for empty results and invalid queries
    - Add loading states and error recovery for DNP operations
    - _Requirements: 2.1, 2.2, 2.5_

  - [ ] 3.2 Fix DNP modification endpoints
    - Update addArtist to use correct POST endpoint format
    - Fix removeArtist to use proper DELETE endpoint with artist ID
    - Update updateEntry to use PUT endpoint with correct payload
    - Add optimistic updates for better user experience
    - _Requirements: 2.2, 2.3, 2.4_

- [ ] 4. Implement enforcement planning endpoints (backend)
  - [ ] 4.1 Create enforcement preview endpoint
    - Add /api/v1/enforcement/preview endpoint to backend router
    - Implement handler to analyze connected services and DNP list
    - Return preview showing what will be removed from each service
    - Add proper error handling for disconnected or invalid services
    - _Requirements: 4.1, 4.2_

  - [ ] 4.2 Create enforcement execution endpoint
    - Add /api/v1/enforcement/execute endpoint for running enforcement
    - Implement background job processing for long-running operations
    - Add progress tracking and real-time updates via WebSocket or polling
    - Create detailed execution results with success/failure breakdown
    - _Requirements: 4.3, 4.4, 4.5_

- [ ] 5. Update frontend stores to use new API client
  - [x] 5.1 Refactor auth store to use centralized API client
    - Update login, register, and refresh token calls
    - Add proper error handling and user feedback
    - Implement automatic token refresh on 401 responses
    - Add loading states for all authentication operations
    - _Requirements: 5.1, 5.2, 6.1, 6.2_

  - [x] 5.2 Update DNP store to use new API client and error handling
    - Refactor all DNP operations to use centralized API client
    - Add proper loading states and error recovery
    - Implement optimistic updates for add/remove operations
    - Add retry logic for failed operations
    - _Requirements: 2.5, 6.1, 6.2, 6.3, 6.4_

  - [ ] 5.3 Update connections store to use new API client
    - Refactor OAuth operations to use centralized client
    - Add proper error handling for OAuth failures
    - Implement connection health monitoring
    - Add user-friendly error messages for connection issues
    - _Requirements: 3.5, 6.1, 6.2, 6.3, 6.4_

- [ ] 6. Implement enforcement planning frontend
  - [ ] 6.1 Create enforcement preview component
    - Build UI to display enforcement preview with service breakdown
    - Show counts of items to be removed per service
    - Add confirmation dialog before executing enforcement
    - Display warnings for potentially destructive actions
    - _Requirements: 4.1, 4.2, 6.4_

  - [ ] 6.2 Create enforcement execution component
    - Build progress tracking UI with real-time updates
    - Display detailed results after enforcement completion
    - Add ability to view logs and retry failed operations
    - Implement undo functionality where supported by service APIs
    - _Requirements: 4.3, 4.4, 4.5, 6.2, 6.4_

- [ ] 7. Add comprehensive error handling and user feedback
  - [ ] 7.1 Implement global error handling
    - Create error boundary components for React-like error catching
    - Add toast notifications for success/error messages
    - Implement offline detection and appropriate UI states
    - Add retry mechanisms with exponential backoff
    - _Requirements: 5.4, 5.5, 6.3, 6.5_

  - [ ] 7.2 Add loading states and user feedback
    - Implement skeleton loading states for all major components
    - Add progress indicators for long-running operations
    - Create empty states with helpful guidance for new users
    - Add confirmation dialogs for destructive actions
    - _Requirements: 6.1, 6.2, 6.4, 6.5_

- [ ] 8. Test and validate core functionality
  - [x] 8.1 Test complete user workflows
    - Test user registration and login flow
    - Test Spotify connection and OAuth flow
    - Test adding artists to DNP list and searching
    - Test enforcement preview and execution (when implemented)
    - _Requirements: All requirements - integration testing_

  - [ ] 8.2 Test error scenarios and edge cases
    - Test behavior when backend is unavailable
    - Test OAuth failures and token expiration
    - Test invalid API responses and malformed data
    - Test network timeouts and retry logic
    - _Requirements: 5.4, 5.5, 6.3, 6.5_

- [ ] 9. Documentation and deployment preparation
  - [ ] 9.1 Update API documentation
    - Document all endpoint changes and new endpoints
    - Create API client usage examples
    - Document error codes and response formats
    - Add troubleshooting guide for common issues
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

  - [ ] 9.2 Prepare deployment configuration
    - Update environment variables for API endpoints
    - Configure CORS settings for frontend-backend communication
    - Set up proper error logging and monitoring
    - Test deployment in staging environment
    - _Requirements: All requirements - deployment readiness_