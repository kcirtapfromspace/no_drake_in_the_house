# Implementation Plan

- [x] 1. Set up project foundation and core data models
  - Create Svelte project with TypeScript, Tailwind CSS, and Rollup for optimized builds
  - Set up Rust backend with Axum web framework and SQLx for database operations
  - Configure PostgreSQL and DuckDB databases with Docker Compose for development
  - Implement core database schema with SQLx migrations (users, artists, connections, action_batches, action_items)
  - Create database connection pools and basic CRUD operations in Rust
  - _Requirements: 2.1, 2.2, 7.1_

- [x] 2. Implement entity resolution service
  - [x] 2.1 Create high-performance artist entity resolution with external ID mapping
    - Build Rust Artist struct with canonical_artist_id and external_ids using serde for JSON serialization
    - Implement concurrent artist search and disambiguation using tokio for async processing
    - Create ML-based artist alias handling with confidence scoring using parallel processing
    - Write comprehensive unit tests for entity resolution edge cases with mock data
    - _Requirements: 1.1, 1.5_

  - [x] 2.2 Integrate MusicBrainz and ISNI for canonical artist identification
    - Implement high-performance MusicBrainz API client using reqwest with connection pooling
    - Add ISNI integration for authoritative artist identification with async processing
    - Create circuit breaker fallback strategies when external services are unavailable
    - Write integration tests with mock external API responses using wiremock
    - _Requirements: 1.1, 1.5_

- [ ] 3. Build authentication and user management system
  - [x] 3.1 Implement user registration and authentication in Rust
    - Create user registration with email/password and OAuth (Google, Apple) using oauth2 crate
    - Implement JWT token generation and validation with refresh token rotation using jsonwebtoken
    - Add 2FA support using TOTP (Time-based One-Time Password) with totp-lite crate
    - Write authentication middleware for Axum route protection with tower middleware
    - _Requirements: 2.1, 2.2, 7.1_

  - [x] 3.2 Create secure token vault service
    - Implement KMS-based envelope encryption for storing provider tokens
    - Build token vault service with automatic key rotation
    - Create secure token storage and retrieval methods
    - Add token health checking and automatic refresh capabilities
    - _Requirements: 2.2, 2.4, 7.1, 7.2_

- [ ] 4. Develop Spotify integration adapter
  - [ ] 4.1 Implement Spotify OAuth and connection management
    - Create Spotify OAuth 2.0 flow with PKCE for secure authorization
    - Implement connection storage with encrypted token management
    - Add connection health monitoring and automatic token refresh
    - Build Spotify API client with rate limiting and error handling
    - _Requirements: 2.1, 2.2, 2.4_

  - [ ] 4.2 Build Spotify library analysis and planning service
    - Implement library scanning (liked songs, playlists, followed artists)
    - Create featured artist detection from track metadata
    - Build enforcement planning with dry-run impact calculation
    - Add collaboration and featuring detection logic
    - _Requirements: 3.1, 3.2, 5.2_

  - [ ] 4.3 Create Spotify enforcement execution engine
    - Implement batch operations for library modifications (remove liked songs, unfollow artists)
    - Build playlist scrubbing with delta removal to minimize API calls
    - Add idempotent operation handling to prevent duplicate actions
    - Create detailed action logging and rollback capabilities
    - _Requirements: 3.2, 3.3, 3.4, 8.1, 8.5_

- [ ] 5. Implement DNP list management system
  - [ ] 5.1 Create personal DNP list CRUD operations
    - Build DNP list creation, modification, and deletion functionality
    - Implement artist search with provider badge display for accurate selection
    - Add bulk import/export functionality for CSV and JSON formats
    - Create tagging and note system for DNP list organization
    - _Requirements: 1.1, 1.2, 1.4_

  - [ ] 5.2 Build community list subscription system
    - Implement community list creation with governance requirements
    - Create list subscription and version pinning functionality
    - Build preview system showing impact before applying community lists
    - Add notification system for community list updates with diff previews
    - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [ ] 6. Develop web application frontend
  - [ ] 6.1 Create user dashboard and DNP management interface with Svelte
    - Build responsive Svelte components for dashboard showing connected services and DNP list status
    - Implement artist search and selection interface with provider badges using Svelte stores
    - Create DNP list management UI with tagging and bulk operations using reactive updates
    - Add service connection management with OAuth flow integration
    - _Requirements: 1.1, 1.2, 1.4, 2.1, 2.4_

  - [ ] 6.2 Build enforcement planning and execution interface with Svelte
    - Create dry-run preview interface using Svelte's reactive derived stores for impact calculation
    - Implement enforcement execution UI with real-time progress tracking using Svelte stores
    - Build action history and undo interface with per-item rollback using Svelte transitions
    - Add settings interface for aggressiveness levels and collaboration blocking with two-way binding
    - _Requirements: 3.1, 3.2, 3.3, 3.4_

  - [ ] 6.3 Implement community list browsing and subscription interface with Svelte
    - Build community list directory with search and filtering using Svelte's reactive filtering
    - Create list detail view showing criteria, governance, and sample impact with Svelte components
    - Implement subscription management with version control using Svelte stores
    - Add appeals and moderation interface for list disputes with form handling
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 6.4, 6.5_

- [ ] 7. Create browser extension for web interface blocking
  - [ ] 7.1 Build extension core architecture and content scripts
    - Create Manifest v3 browser extension with content scripts for major streaming services
    - Implement MutationObserver for resilient DOM monitoring
    - Build shadow DOM isolation for extension UI components
    - Create secure communication between content scripts and background service worker
    - _Requirements: 4.1, 4.4_

  - [ ] 7.2 Implement content filtering and auto-skip functionality
    - Build artist detection using multiple strategies (data attributes, ARIA labels, text content)
    - Implement content hiding with subtle "Hidden by Kiro" badges
    - Create auto-skip functionality for blocked tracks with media event hooks
    - Add override controls with "play once" and "add/remove from DNP" options
    - _Requirements: 4.1, 4.2, 4.3, 4.5_

  - [ ] 7.3 Optimize extension performance with bloom filters
    - Implement bloom filter for O(1) DNP list lookups in extension
    - Create signed, cached DNP filter updates from server
    - Add offline functionality when server is unavailable
    - Build telemetry for selector drift detection and performance monitoring
    - _Requirements: 4.1, 4.4, 8.2_

- [ ] 8. Implement rate limiting and job processing system
  - [ ] 8.1 Create rate-limit aware batching system
    - Build rate limiting framework using provider API response headers
    - Implement optimal batching strategies grouped by playlist and operation type
    - Create resumable batch processing with checkpoint recovery
    - Add exponential backoff and circuit breaker patterns for API failures
    - _Requirements: 8.1, 8.2, 8.5_

  - [ ] 8.2 Build background job processing with Redis queues
    - Implement Redis-based job queue for enforcement operations
    - Create worker processes for asynchronous enforcement execution
    - Add job progress tracking and user notification system
    - Build job retry logic with dead letter queue for failed operations
    - _Requirements: 3.2, 8.1, 8.2_

- [ ] 9. Add Apple Music integration adapter
  - [ ] 9.1 Implement Apple Music MusicKit integration
    - Create MusicKit JS authentication with user token management
    - Build token broker service for developer token rotation
    - Implement Apple Music API client with limited write capabilities
    - Add library scanning and modification capabilities where supported
    - _Requirements: 2.1, 2.2, 3.1, 3.2_

  - [ ] 9.2 Create Apple Music web extension support
    - Extend browser extension to support Apple Music web interface
    - Implement content filtering with Apple Music-specific selectors
    - Add graceful degradation messaging for unsupported operations
    - Create capability matrix documentation for Apple Music limitations
    - _Requirements: 4.1, 4.4_

- [ ] 10. Implement YouTube Music integration
  - [ ] 10.1 Build YouTube Music web extension support
    - Create YouTube Music content filtering with ToS-compliant approach
    - Implement auto-skip functionality for YouTube Music web player
    - Add user data export/import workflows for manual synchronization
    - Build preview-only mode with clear capability limitations
    - _Requirements: 4.1, 4.2, 4.4_

  - [ ] 10.2 Add YouTube Music recommendation filtering
    - Implement recommendation hiding based on blocked artists
    - Create "Not Interested" automation where permitted by ToS
    - Add radio seed filtering to reduce blocked artist recommendations
    - Build user education about YouTube Music limitations
    - _Requirements: 4.1, 4.4_

- [ ] 11. Create comprehensive testing suite
  - [ ] 11.1 Implement unit tests for core services
    - Write unit tests for entity resolution with mock external APIs
    - Create tests for authentication flows and token management
    - Build tests for DNP list operations and community list subscriptions
    - Add tests for enforcement planning and execution logic
    - _Requirements: All requirements - testing coverage_

  - [ ] 11.2 Build integration tests with provider sandboxes
    - Create integration tests using Spotify test accounts and sandbox APIs
    - Implement contract tests for external API changes
    - Build end-to-end tests for complete enforcement workflows
    - Add browser extension tests with DOM fixtures for selector resilience
    - _Requirements: All requirements - integration testing_

- [ ] 12. Implement security and compliance features
  - [ ] 12.1 Add audit logging and SOC2 compliance controls
    - Implement comprehensive audit logging for all user actions
    - Create access review and security monitoring capabilities
    - Build GDPR/CCPA compliant data export and deletion functionality
    - Add security headers and vulnerability scanning integration
    - _Requirements: 7.1, 7.2, 7.3, 7.4_

  - [ ] 12.2 Create content moderation and appeals system
    - Build community list moderation queue with structured appeals process
    - Implement content policy validation for neutral language requirements
    - Create automated detection for prohibited content patterns
    - Add moderation dashboard for list governance and dispute resolution
    - _Requirements: 6.1, 6.4, 6.5_

- [ ] 13. Build monitoring and observability system
  - [ ] 13.1 Implement application monitoring and alerting
    - Create structured logging with correlation IDs for distributed tracing
    - Implement Prometheus metrics for performance and success rate monitoring
    - Build health checks for all services and external API dependencies
    - Add alerting for critical failures and SLO violations
    - _Requirements: 8.3, 8.4_

  - [ ] 13.2 Create user-facing analytics and reporting
    - Build enforcement success reporting with detailed action summaries
    - Implement usage analytics for DNP list effectiveness
    - Create community list impact reporting for curators
    - Add performance dashboards for system health and capacity planning
    - _Requirements: 3.3, 3.4_

- [ ] 14. Implement mobile assistance features
  - [ ] 14.1 Create iOS Shortcuts integration
    - Build iOS Shortcuts for quick DNP list modifications
    - Implement skip automation using iOS media controls where possible
    - Create Siri integration for voice-based DNP list management
    - Add iOS widget for quick access to enforcement status
    - _Requirements: 4.5_

  - [ ] 14.2 Add Android intents and automation support
    - Create Android intents for DNP list management
    - Implement Tasker integration for automated enforcement triggers
    - Build Android quick settings tile for enforcement status
    - Add notification-based controls for skip and block actions
    - _Requirements: 4.5_

- [ ] 15. Finalize production deployment and documentation
  - [ ] 15.1 Create production deployment infrastructure
    - Set up Kubernetes deployment manifests with proper resource limits
    - Implement CI/CD pipeline with automated testing and security scanning
    - Create database migration and backup strategies
    - Build monitoring and logging infrastructure for production
    - _Requirements: 8.3, 8.4_

  - [ ] 15.2 Complete documentation and user onboarding
    - Write comprehensive API documentation with capability matrices
    - Create user guides for each streaming service integration
    - Build developer documentation for extension and mobile integration
    - Implement in-app onboarding flow with guided setup
    - _Requirements: All requirements - documentation and usability_