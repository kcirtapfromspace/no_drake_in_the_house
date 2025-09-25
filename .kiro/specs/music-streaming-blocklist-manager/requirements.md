# Requirements Document

## Introduction

No Drake in the House is a music streaming blocklist management platform that provides users with centralized control to avoid specific artists across multiple streaming platforms including Spotify, Apple Music, Tidal, and YouTube Music. The system empowers listeners to maintain personal Do-Not-Play (DNP) lists, subscribe to community-curated blocklists, and apply these preferences across major streaming platforms using official APIs where available, with graceful fallbacks for platforms with limited API support.

The platform maintains strict neutrality by not making claims about individuals, instead focusing on user-defined preferences and community-curated lists with clear provenance and governance.

## Requirements

### Requirement 1

**User Story:** As a listener, I want to create and manage a personal Do-Not-Play list of artists, so that I can avoid hearing music from artists I don't want to support.

#### Acceptance Criteria

1. WHEN a user searches for an artist THEN the system SHALL display matching artists with photos and provider badges for accurate identification
2. WHEN a user selects an artist to add to their DNP list THEN the system SHALL add the artist using canonical provider IDs to avoid name collision issues
3. WHEN a user wants to bulk import artists THEN the system SHALL support CSV/JSON import with artist names and provider URLs
4. WHEN a user adds or removes artists from their DNP list THEN the system SHALL allow them to add private tags and notes for organization
5. IF a user attempts to add a duplicate artist THEN the system SHALL handle collision detection and canonicalization via provider IDs

### Requirement 2

**User Story:** As a listener, I want to connect my streaming service accounts, so that I can apply my blocklist preferences across multiple platforms.

#### Acceptance Criteria

1. WHEN a user initiates account connection THEN the system SHALL support OAuth flows for Spotify, Apple Music, YouTube Music, and Tidal
2. WHEN connecting accounts THEN the system SHALL request only minimum necessary scopes for read/write library operations
3. WHEN storing authentication tokens THEN the system SHALL encrypt all provider tokens using KMS with key rotation
4. WHEN a user wants to disconnect a service THEN the system SHALL allow per-service token revocation
5. IF authentication tokens expire THEN the system SHALL automatically refresh tokens using stored refresh tokens

### Requirement 3

**User Story:** As a listener, I want to preview and execute blocklist enforcement across my connected streaming services, so that I can see what changes will be made before they happen.

#### Acceptance Criteria

1. WHEN a user requests to apply their DNP list THEN the system SHALL generate a dry-run preview showing counts by service (playlists, liked songs, follows, recommendations)
2. WHEN a user executes the enforcement plan THEN the system SHALL perform actions per service with progress UI and idempotency to prevent duplicate operations
3. WHEN enforcement is complete THEN the system SHALL provide a detailed report of all actions taken with timestamps and entity details
4. WHEN a user wants to undo changes THEN the system SHALL support per-item undo and bulk rollback per batch where platform APIs allow
5. IF an enforcement action fails THEN the system SHALL log the error and continue with remaining actions without stopping the entire process

### Requirement 4

**User Story:** As a listener, I want to use a browser extension to hide blocked artists on streaming service web interfaces, so that I can avoid seeing unwanted content while browsing.

#### Acceptance Criteria

1. WHEN browsing Spotify or YouTube Music web interfaces THEN the extension SHALL hide tiles and disable play buttons for blocked artists
2. WHEN a blocked track starts playing THEN the extension SHALL auto-skip the track and show an unobtrusive toast with the reason
3. WHEN a user wants to override the block temporarily THEN the extension SHALL provide an "override once" option in the context menu
4. WHEN the extension detects blocked content THEN it SHALL display subtle badges indicating "Hidden by No Drake in the House" without being disruptive
5. IF the user wants to modify their DNP list THEN the extension SHALL provide context menu options to add/remove artists directly

### Requirement 5

**User Story:** As a listener, I want to subscribe to community-curated blocklists, so that I can benefit from shared curation efforts based on specific criteria.

#### Acceptance Criteria

1. WHEN browsing community lists THEN the system SHALL display list criteria, owner information, and governance details
2. WHEN a user wants to subscribe to a list THEN the system SHALL show a preview of what changes the list would make to their library
3. WHEN subscribing to a list THEN the system SHALL allow users to pin to a specific version or enable auto-updates
4. WHEN a subscribed list updates THEN the system SHALL notify users with a diff preview of changes
5. IF a user disagrees with list content THEN the system SHALL provide an appeals process with structured forms and audit logs

### Requirement 6

**User Story:** As a curator, I want to create and maintain community blocklists with clear criteria and governance, so that I can share curated lists with transparent processes.

#### Acceptance Criteria

1. WHEN creating a community list THEN the system SHALL require declaration of criteria, update process, and governance model
2. WHEN adding items to a list THEN the system SHALL support optional evidence links and rationale without editorial commentary
3. WHEN publishing list updates THEN the system SHALL create immutable versions for reproducible results
4. WHEN list content is disputed THEN the system SHALL provide a moderation queue with SLA and structured appeals process
5. IF inappropriate content is reported THEN the system SHALL have spam/harassment reporting flows with clear escalation paths

### Requirement 7

**User Story:** As a user, I want my data to be secure and private, so that I can trust the platform with my streaming service credentials and preferences.

#### Acceptance Criteria

1. WHEN storing user data THEN the system SHALL encrypt all provider tokens at rest using KMS encryption
2. WHEN processing user libraries THEN the system SHALL minimize PII storage and use pseudonymous IDs for telemetry
3. WHEN users request data export THEN the system SHALL provide GDPR/CCPA compliant data export in portable formats
4. WHEN users want to delete their account THEN the system SHALL provide complete data deletion capabilities
5. IF security incidents occur THEN the system SHALL have SOC2-friendly logging and access review controls

### Requirement 8

**User Story:** As a user, I want the system to be reliable and performant, so that I can efficiently manage large music libraries without long wait times.

#### Acceptance Criteria

1. WHEN applying a 1,000-artist DNP list to a 10,000-track library THEN the system SHALL complete processing within 3 minutes for Spotify given API rate limits
2. WHEN API calls fail THEN the system SHALL implement retries with exponential backoff and circuit breakers
3. WHEN the system experiences high load THEN it SHALL maintain 99.9% monthly uptime for core functionality
4. WHEN processing large batches THEN the system SHALL use parallelism and batching to optimize performance within rate limits
5. IF operations are interrupted THEN the system SHALL support resumable batches to continue from the last successful operation
