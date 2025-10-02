# Implementation Plan

- [x] 1. Enhanced Development Environment Setup
  - Create optimized Dockerfiles for development with live updates
  - Implement multi-platform Kubernetes detection in Tiltfile
  - Add comprehensive manual triggers for database, testing, and monitoring
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7_

- [x] 2. OAuth Provider Infrastructure
- [x] 2.1 Create OAuth provider trait and base implementations
  - Define OAuthProvider trait with common methods for all providers
  - Implement base OAuth flow structures and error handling
  - Create OAuth token encryption/decryption utilities using AES-GCM
  - Write unit tests for OAuth base functionality
  - _Requirements: 2.1, 2.2, 2.3, 2.6, 6.1, 6.2, 6.3, 6.4_

- [x] 2.2 Implement Google OAuth provider
  - Create GoogleOAuthProvider struct with OAuth2 flow implementation
  - Implement authorization URL generation and token exchange
  - Add user info retrieval and token refresh functionality
  - Write integration tests with mocked Google OAuth endpoints
  - _Requirements: 2.1, 2.2, 2.3, 2.6, 6.1, 6.2, 6.3_

- [x] 2.3 Implement Apple Sign In provider
  - Create AppleOAuthProvider with JWT-based client secret generation
  - Implement Apple ID token verification and user info extraction
  - Handle Apple's unique OAuth flow requirements and privacy features
  - Write tests for Apple-specific authentication flows
  - _Requirements: 2.1, 2.2, 2.3, 2.6, 6.1, 6.2, 6.3_

- [x] 2.4 Implement GitHub OAuth provider
  - Create GitHubOAuthProvider with standard OAuth2 flow
  - Implement GitHub user info retrieval and email access
  - Add GitHub-specific scope handling and permissions
  - Write integration tests for GitHub OAuth flows
  - _Requirements: 2.1, 2.2, 2.3, 2.6, 6.1, 6.2, 6.3_

- [x] 3. Database Schema Extensions for Social Auth
- [x] 3.1 Create OAuth accounts migration
  - Add oauth_accounts table with encrypted token storage
  - Create indexes for efficient provider and user lookups
  - Add account_merges table for audit trail
  - Write migration rollback procedures
  - _Requirements: 2.4, 2.5, 6.1, 6.2_

- [x] 3.2 Extend user model for OAuth integration
  - Update User struct to include OAuth account relationships
  - Implement OAuth account linking and unlinking methods
  - Add user profile merging capabilities
  - Write unit tests for user model extensions
  - _Requirements: 2.4, 2.5, 6.1_

- [x] 4. Enhanced Authentication Service
- [x] 4.1 Extend AuthService with OAuth capabilities
  - Add OAuth flow initiation and completion methods
  - Implement secure state parameter generation and validation
  - Create account linking and merging functionality
  - Write comprehensive unit tests for OAuth authentication flows
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 6.1, 6.2, 6.3, 6.4, 6.5, 6.6_

- [x] 4.2 Implement token management and refresh
  - Create automatic OAuth token refresh mechanisms
  - Implement token expiration handling and user notifications
  - Add token revocation and cleanup procedures
  - Write tests for token lifecycle management
  - _Requirements: 2.6, 6.5, 6.6_

- [x] 5. Frontend OAuth Integration
- [x] 5.1 Create OAuth login components
  - Build social login buttons for Google, Apple, and GitHub
  - Implement OAuth callback handling and error display
  - Create account linking interface for existing users
  - Write component tests for OAuth UI flows
  - _Requirements: 2.1, 2.5, 2.6_

- [x] 5.2 Update authentication store and routing
  - Extend auth store to handle OAuth flows and multiple providers
  - Update routing to handle OAuth callbacks and error states
  - Implement user profile display with linked accounts
  - Write integration tests for frontend OAuth flows
  - _Requirements: 2.1, 2.4, 2.5_

- [ ] 6. Music Service Connection Enhancement
- [ ] 6.1 Create unified music service manager
  - Implement MusicServiceManager with multi-platform support
  - Create service connection flow with improved UX
  - Add unified library statistics aggregation
  - Write unit tests for service management functionality
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7_

- [ ] 6.2 Enhance service connection UI
  - Create improved service connection interface with status display
  - Implement connection health monitoring and error handling
  - Add unified library statistics dashboard
  - Write component tests for service connection UI
  - _Requirements: 3.1, 3.2, 3.3, 3.7_

- [ ] 7. ML Classification Infrastructure
- [ ] 7.1 Create ML model client interface
  - Define MLModelClient trait for artist classification
  - Implement classification data structures and evidence models
  - Create batch processing capabilities for efficient classification
  - Write unit tests for ML client interface
  - _Requirements: 4.1, 4.2, 4.3, 4.7_

- [ ] 7.2 Implement evidence storage and management
  - Create EvidenceStore for managing classification evidence
  - Implement evidence validation and source tracking
  - Add evidence aggregation and confidence scoring
  - Write tests for evidence storage and retrieval
  - _Requirements: 4.1, 4.2, 4.7_

- [ ] 8. ML Blocklist Generation System
- [ ] 8.1 Create classification categories and criteria
  - Implement BlocklistCategory enum with violence, explicit, controversial types
  - Create ClassificationCriteria for configurable classification rules
  - Add confidence threshold and evidence requirement management
  - Write unit tests for classification logic
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [ ] 8.2 Implement ML blocklist service
  - Create MLBlocklistService for automated list generation
  - Implement subscription management with user preferences
  - Add pending addition review and approval workflows
  - Write integration tests for ML blocklist functionality
  - _Requirements: 4.1, 4.4, 4.5, 4.6_

- [ ] 9. Transparency and Feedback System
- [ ] 9.1 Create classification explanation service
  - Implement TransparencyService for classification explanations
  - Create detailed evidence presentation with source citations
  - Add similar artist suggestions and confidence breakdowns
  - Write unit tests for transparency features
  - _Requirements: 4.2, 4.6, 4.7_

- [ ] 9.2 Implement user feedback collection
  - Create UserFeedback system for accept/reject/dispute actions
  - Implement evidence dispute handling and resolution
  - Add feedback aggregation for model improvement
  - Write tests for feedback collection and processing
  - _Requirements: 4.6, 4.7_

- [ ] 10. ML Blocklist Frontend Interface
- [ ] 10.1 Create ML blocklist browser and subscription UI
  - Build interface for browsing available ML-generated blocklists
  - Implement subscription management with preference settings
  - Create category filtering and search functionality
  - Write component tests for ML blocklist UI
  - _Requirements: 4.3, 4.4, 4.5_

- [ ] 10.2 Implement transparency and feedback UI
  - Create classification explanation display with evidence
  - Build user feedback interface for approving/rejecting classifications
  - Implement evidence dispute submission and tracking
  - Write integration tests for transparency UI
  - _Requirements: 4.2, 4.6, 4.7_

- [ ] 11. Enhanced API Endpoints
- [ ] 11.1 Create OAuth authentication endpoints
  - Implement OAuth flow initiation and callback endpoints
  - Add account linking and unlinking API endpoints
  - Create OAuth provider management endpoints
  - Write API integration tests for OAuth endpoints
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 6.1, 6.2, 6.3, 6.4_

- [ ] 11.2 Create ML blocklist API endpoints
  - Implement ML blocklist browsing and subscription endpoints
  - Add classification explanation and evidence endpoints
  - Create user feedback submission and tracking endpoints
  - Write API integration tests for ML functionality
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7_

- [ ] 12. Security and Performance Enhancements
- [ ] 12.1 Implement enhanced token security
  - Add OAuth token encryption with AES-GCM in token vault
  - Implement secure token rotation and cleanup procedures
  - Add rate limiting for OAuth endpoints and ML classification
  - Write security tests for token handling and encryption
  - _Requirements: 2.3, 2.6, 6.1, 6.2, 6.5, 6.6_

- [ ] 12.2 Add performance monitoring and optimization
  - Implement caching for ML classifications and evidence
  - Add performance metrics for OAuth flows and ML processing
  - Create batch processing optimization for large-scale classification
  - Write performance tests and benchmarks
  - _Requirements: 4.1, 5.5, 5.6, 5.7_

- [ ] 13. Integration Testing and Documentation
- [ ] 13.1 Create comprehensive integration tests
  - Write end-to-end tests for complete OAuth flows
  - Implement ML classification pipeline integration tests
  - Add music service connection integration tests
  - Create performance and load testing for ML features
  - _Requirements: All requirements - comprehensive testing_

- [ ] 13.2 Update documentation and deployment
  - Update API documentation with new OAuth and ML endpoints
  - Create developer setup guide for enhanced development environment
  - Add user guides for social authentication and ML blocklists
  - Update deployment configurations for new features
  - _Requirements: 1.6, 5.7, 6.6_