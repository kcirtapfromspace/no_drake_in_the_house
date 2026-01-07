# Implementation Plan

- [x] 1. Re-enable Core OAuth Service Methods
- [x] 1.1 Implement initiate_oauth_flow method
  - Replace "temporarily disabled" placeholder with real OAuth flow initiation
  - Add secure state parameter generation and storage
  - Integrate with OAuth provider instances to get authorization URLs
  - Add proper error handling for provider configuration issues
  - _Requirements: 1.1, 1.2, 2.1, 2.6_

- [x] 1.2 Implement complete_oauth_flow method
  - Replace "temporarily disabled" placeholder with real OAuth completion logic
  - Add state parameter validation for CSRF protection
  - Implement token exchange with OAuth providers
  - Add user creation and account linking logic
  - Generate JWT tokens for authenticated users
  - _Requirements: 1.2, 1.3, 1.4, 2.2, 2.3_

- [x] 1.3 Implement find_user_by_oauth_account method
  - Replace "temporarily disabled" placeholder with real database query
  - Add proper SQL query to join users and oauth_accounts tables
  - Include OAuth account loading for complete user profile
  - Add error handling for database connection issues
  - _Requirements: 1.5, 3.1, 3.6_

- [x] 1.4 Implement load_oauth_accounts method
  - Replace "temporarily disabled" placeholder with real database query
  - Add token decryption for OAuth account access
  - Include proper error handling for encryption failures
  - Add efficient database queries with proper indexing
  - _Requirements: 3.1, 3.7, 4.1_

- [x] 2. Complete OAuth Provider Implementations
- [x] 2.1 Implement Google OAuth provider with real API integration
  - Replace mock responses with actual Google OAuth2 API calls
  - Add proper authorization URL generation with required scopes
  - Implement token exchange using Google's token endpoint
  - Add user info retrieval from Google's userinfo API
  - Include proper error handling for Google API responses
  - _Requirements: 2.1, 2.2, 2.4, 2.6_

- [x] 2.2 Implement Apple OAuth provider with real API integration
  - Replace mock responses with actual Apple Sign In API calls
  - Add JWT-based client secret generation for Apple authentication
  - Implement token exchange using Apple's token endpoint
  - Add ID token verification and user info extraction
  - Include proper error handling for Apple-specific requirements
  - _Requirements: 2.1, 2.2, 2.4, 2.6_

- [x] 2.3 Implement GitHub OAuth provider with real API integration
  - Replace mock responses with actual GitHub OAuth API calls
  - Add proper authorization URL generation with user:email scope
  - Implement token exchange using GitHub's access_token endpoint
  - Add user info and email retrieval from GitHub API
  - Include proper error handling for GitHub API responses
  - _Requirements: 2.1, 2.2, 2.4, 2.6_

- [x] 3. Implement Database Operations for OAuth Accounts
- [x] 3.1 Implement create_user_with_oauth_account method
  - Replace "temporarily disabled" placeholder with real user creation
  - Add database transaction for atomic user and OAuth account creation
  - Include proper token encryption before database storage
  - Add error handling for database constraint violations
  - _Requirements: 1.4, 3.1, 3.2, 4.1_

- [x] 3.2 Implement link_oauth_account_to_user method
  - Replace "temporarily disabled" placeholder with real account linking
  - Add validation to prevent duplicate OAuth accounts for same provider
  - Include proper token encryption and database storage
  - Add error handling for account linking conflicts
  - _Requirements: 1.5, 3.3, 4.1, 6.2_

- [x] 3.3 Implement unlink_oauth_account method
  - Replace "temporarily disabled" placeholder with real account unlinking
  - Add database deletion of OAuth account records
  - Include token cleanup and revocation where possible
  - Add validation to ensure user retains access after unlinking
  - _Requirements: 1.7, 6.3, 6.5_

- [x] 4. Implement Token Management and Security
- [x] 4.1 Implement refresh_oauth_tokens method
  - Replace "temporarily disabled" placeholder with real token refresh
  - Add automatic token refresh when tokens are near expiry
  - Include proper error handling for refresh token failures
  - Add database updates for refreshed tokens with encryption
  - _Requirements: 1.6, 4.2, 4.3, 4.4_

- [x] 4.2 Implement get_valid_oauth_token method
  - Create new method to get valid tokens with automatic refresh
  - Add token expiry checking and automatic refresh triggering
  - Include proper token decryption for API usage
  - Add error handling for token validation failures
  - _Requirements: 4.2, 4.3, 4.6_

- [x] 4.3 Implement OAuth token encryption utilities
  - Ensure OAuth token encryption service is properly configured
  - Add key rotation capabilities for enhanced security
  - Include proper error handling for encryption/decryption failures
  - Add performance optimization for frequent token operations
  - _Requirements: 4.1, 4.6, 3.7_

- [x] 5. Enhance OAuth Handlers with Real Functionality
- [x] 5.1 Update initiate_oauth_handler to use real service methods
  - Remove any remaining placeholder responses in OAuth handlers
  - Ensure proper integration with re-enabled AuthService methods
  - Add comprehensive error handling and user-friendly error messages
  - Include proper logging for OAuth flow tracking
  - _Requirements: 2.1, 5.1, 5.3_

- [x] 5.2 Update oauth_callback_handler to use real service methods
  - Remove any remaining placeholder responses in callback handler
  - Ensure proper integration with complete_oauth_flow method
  - Add comprehensive error handling for OAuth callback failures
  - Include proper user response formatting with complete user data
  - _Requirements: 2.2, 2.3, 5.1, 5.4_

- [x] 5.3 Update account management handlers
  - Implement get_linked_accounts_handler with real database queries
  - Add link_oauth_account_handler with proper validation
  - Implement unlink_oauth_account_handler with confirmation
  - Include proper error handling and user feedback
  - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [x] 6. Add OAuth Provider Configuration and Validation
- [x] 6.1 Implement OAuth provider configuration validation
  - Add validation for OAuth client IDs and secrets on startup
  - Include environment variable validation for required OAuth credentials
  - Add graceful handling when OAuth providers are not configured
  - Include proper error messages for configuration issues
  - _Requirements: 2.4, 2.5, 5.2_

- [x] 6.2 Add OAuth provider health checks
  - Implement health check endpoints for OAuth provider connectivity
  - Add monitoring for OAuth provider API availability
  - Include rate limit tracking and exponential backoff
  - Add alerting for OAuth provider service disruptions
  - _Requirements: 2.5, 5.5, 5.6_

- [x] 7. Implement Enhanced Error Handling
- [x] 7.1 Add OAuth-specific error types and handling
  - Create comprehensive OAuth error types for different failure scenarios
  - Add proper error mapping from provider APIs to internal errors
  - Include user-friendly error messages for common OAuth issues
  - Add error logging and monitoring for OAuth failures
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.7_

- [x] 7.2 Implement OAuth error recovery strategies
  - Add automatic retry logic for transient OAuth provider errors
  - Include graceful fallback when OAuth providers are unavailable
  - Add user guidance for resolving OAuth authentication issues
  - Include proper security logging for OAuth-related security events
  - _Requirements: 5.2, 5.5, 5.6, 5.7_

- [x] 8. Add Account Management Features
- [x] 8.1 Implement account merging functionality
  - Replace "temporarily disabled" merge_accounts method with real implementation
  - Add guided workflow for merging duplicate accounts
  - Include proper data migration and audit trail
  - Add validation to prevent data loss during account merges
  - _Requirements: 6.4, 6.6_

- [x] 8.2 Implement OAuth account status monitoring
  - Add monitoring for OAuth token expiry and refresh status
  - Include user notifications for authentication issues
  - Add dashboard display of OAuth account connection health
  - Include proactive token refresh scheduling
  - _Requirements: 6.1, 6.7_

- [x] 9. Add Comprehensive Testing
- [x] 9.1 Create unit tests for OAuth service methods
  - Write tests for all re-enabled OAuth service methods
  - Add mock OAuth provider responses for testing
  - Include edge case testing for error scenarios
  - Add performance testing for token encryption/decryption
  - _Requirements: All requirements - comprehensive testing_

- [x] 9.2 Create integration tests for OAuth flows
  - Write end-to-end tests for complete OAuth authentication flows
  - Add tests for account linking and unlinking scenarios
  - Include tests for token refresh and expiry handling
  - Add security testing for state parameter validation
  - _Requirements: All requirements - integration testing_

- [ ] 10. Update Documentation and Configuration
- [ ] 10.1 Update OAuth configuration documentation
  - Document required environment variables for OAuth providers
  - Add setup instructions for Google, Apple, and GitHub OAuth apps
  - Include troubleshooting guide for common OAuth configuration issues
  - Add security best practices for OAuth implementation
  - _Requirements: 2.4, 6.1_

- [ ] 10.2 Update API documentation for OAuth endpoints
  - Document all OAuth API endpoints with request/response examples
  - Add error code documentation for OAuth-specific errors
  - Include OAuth flow diagrams and sequence documentation
  - Add rate limiting and security considerations documentation
  - _Requirements: 5.1, 5.2, 5.3, 5.4_