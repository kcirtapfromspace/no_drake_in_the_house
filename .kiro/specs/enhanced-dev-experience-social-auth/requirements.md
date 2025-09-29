# Requirements Document

## Introduction

This feature enhances the developer experience by streamlining local development setup with modern tooling (Tiltfile, minikube, k3s) and improves user onboarding through social authentication providers (Google, Apple, GitHub). Additionally, it introduces ML-generated blocklists that can automatically categorize artists into evidence-based block groups like "violent", "bad vibes", etc.

## Requirements

### Requirement 1: Streamlined Developer Experience

**User Story:** As a developer, I want a one-command development environment setup so that I can quickly start contributing to the project without complex configuration.

#### Acceptance Criteria

1. WHEN a developer runs `tilt up` THEN the system SHALL automatically start all required services (backend, frontend, database, Redis) in a local Kubernetes environment
2. WHEN services are starting THEN Tilt SHALL provide real-time logs and status updates for all components
3. WHEN code changes are made THEN the system SHALL automatically rebuild and redeploy affected services within 30 seconds
4. IF any service fails to start THEN Tilt SHALL display clear error messages with troubleshooting guidance
5. WHEN using minikube or k3s THEN the system SHALL automatically configure ingress routing for local development
6. WHEN the development environment is running THEN developers SHALL be able to access the frontend at a predictable local URL

### Requirement 2: Social Authentication Integration

**User Story:** As a user, I want to sign up and log in using my existing social accounts (Google, Apple, GitHub, etc) so that I don't need to create and remember another password.

#### Acceptance Criteria

1. WHEN a user visits the login page THEN the system SHALL display options for Google, Apple, and GitHub authentication
2. WHEN a user clicks a social login button THEN the system SHALL redirect to the appropriate OAuth provider
3. WHEN OAuth flow completes successfully THEN the system SHALL create or update the user account with profile information
4. WHEN a user has multiple linked accounts THEN the system SHALL merge them into a single user profile
5. IF OAuth fails THEN the system SHALL display a user-friendly error message and fallback options
6. WHEN a user logs in via social auth THEN the system SHALL issue a JWT token with appropriate expiration
7. WHEN user profile information changes on the social provider THEN the system SHALL update local profile data on next login

### Requirement 3: Streamlined Music Service Linking

**User Story:** As a user, I want to easily connect my streaming music accounts (Spotify, Apple Music, youtube, tidal,etc) so that I can manage my blocklists across platforms.

#### Acceptance Criteria

1. WHEN a user accesses service connections THEN the system SHALL display available streaming platforms with connection status
2. WHEN a user clicks "Connect Spotify" THEN the system SHALL initiate Spotify OAuth flow with appropriate scopes
3. WHEN OAuth completes THEN the system SHALL securely store encrypted tokens in the token vault
4. WHEN tokens expire THEN the system SHALL automatically refresh them using stored refresh tokens
5. IF token refresh fails THEN the system SHALL notify the user to re-authenticate
6. WHEN a user disconnects a service THEN the system SHALL revoke tokens and remove stored credentials
7. WHEN multiple services are connected THEN the system SHALL display unified library statistics

### Requirement 4: ML-Generated Evidence-Based Blocklists

**User Story:** As a user, I want access to ML-generated blocklists based on evidence and categorized by themes so that I can quickly block artists that don't align with my values.

#### Acceptance Criteria

1. WHEN the ML system analyzes artist data THEN it SHALL categorize artists into evidence-based groups (violent, explicit, controversial, etc.)
2. WHEN generating categories THEN the system SHALL provide source citations and evidence for each classification
3. WHEN a user browses ML blocklists THEN they SHALL see category descriptions, artist counts, and confidence scores
4. WHEN a user subscribes to an ML blocklist THEN they SHALL be able to review and approve/reject individual artists
5. WHEN new artists are added to ML categories THEN subscribed users SHALL receive notifications with opt-out options
6. IF a user disputes an ML classification THEN they SHALL be able to provide feedback that improves the model
7. WHEN ML recommendations are displayed THEN the system SHALL show transparency information about why artists were categorized
8. WHEN users provide feedback THEN the system SHALL track accept/reject/mute rates for model improvement

### Requirement 5: Enhanced Development Tooling

**User Story:** As a developer, I want comprehensive development tooling support so that I can work efficiently across different local Kubernetes distributions.

#### Acceptance Criteria

1. WHEN using minikube THEN the Tiltfile SHALL automatically detect and configure minikube-specific settings
2. WHEN using k3s THEN the system SHALL configure appropriate ingress and load balancer settings
3. WHEN using kind THEN the system SHALL set up port forwarding and cluster networking correctly
4. WHEN switching between Kubernetes distributions THEN the Tiltfile SHALL adapt configuration automatically
5. WHEN services are unhealthy THEN the development environment SHALL provide debugging tools and logs
6. WHEN running integration tests THEN the system SHALL provide isolated test environments
7. WHEN debugging issues THEN developers SHALL have access to service logs, metrics, and tracing information

### Requirement 6: OAuth Provider Management

**User Story:** As a system administrator, I want to manage OAuth provider configurations so that I can enable/disable social login options and update credentials.

#### Acceptance Criteria

1. WHEN configuring OAuth providers THEN the system SHALL support environment-based configuration for client IDs and secrets
2. WHEN an OAuth provider is disabled THEN existing users SHALL still be able to access their accounts but new registrations SHALL be blocked
3. WHEN OAuth credentials are updated THEN the system SHALL validate them before applying changes
4. WHEN OAuth flows fail THEN the system SHALL log detailed error information for debugging
5. IF rate limits are exceeded THEN the system SHALL implement exponential backoff and user-friendly error messages
6. WHEN users revoke access on the provider side THEN the system SHALL handle authorization errors gracefully