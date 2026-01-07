# Requirements Document

## Introduction

The core product functionality is well-implemented but has API endpoint mismatches between the frontend and backend. The frontend expects Spotify-specific endpoints while the backend implements a more generic OAuth system. This spec will align the frontend and backend to restore full core functionality including DNP list management, service connections, and enforcement planning.

## Requirements

### Requirement 1

**User Story:** As a developer, I want the frontend and backend APIs to be properly aligned, so that the core product functionality works seamlessly.

#### Acceptance Criteria

1. WHEN the frontend calls DNP list endpoints THEN the backend SHALL respond with the correct data format
2. WHEN the frontend initiates OAuth connections THEN the backend SHALL handle the generic OAuth flow for any provider
3. WHEN the frontend checks connection status THEN the backend SHALL return standardized connection information
4. WHEN API calls fail THEN both frontend and backend SHALL use consistent error response formats
5. IF endpoints don't exist THEN the system SHALL provide clear error messages indicating missing functionality

### Requirement 2

**User Story:** As a user, I want to manage my DNP list through the web interface, so that I can add, remove, and organize blocked artists.

#### Acceptance Criteria

1. WHEN I search for artists THEN the system SHALL display matching results with provider badges and metadata
2. WHEN I add an artist to my DNP list THEN the system SHALL save the entry with optional tags and notes
3. WHEN I view my DNP list THEN the system SHALL display all blocked artists with filtering and search capabilities
4. WHEN I remove an artist THEN the system SHALL update the list immediately and confirm the action
5. IF the backend is unavailable THEN the frontend SHALL display appropriate error messages and retry options

### Requirement 3

**User Story:** As a user, I want to connect my Spotify account, so that I can apply my DNP list to my Spotify library.

#### Acceptance Criteria

1. WHEN I click "Connect Spotify" THEN the system SHALL initiate the OAuth flow using the generic backend endpoints
2. WHEN I complete OAuth authorization THEN the system SHALL store my connection securely and display connection status
3. WHEN I view my connections THEN the system SHALL show the status, permissions, and health of each connected service
4. WHEN I disconnect a service THEN the system SHALL revoke tokens and update the connection status
5. IF a connection expires THEN the system SHALL provide options to refresh or reconnect

### Requirement 4

**User Story:** As a user, I want to plan and execute enforcement actions, so that I can remove blocked artists from my connected streaming services.

#### Acceptance Criteria

1. WHEN I have a DNP list and connected services THEN the system SHALL allow me to preview enforcement actions
2. WHEN I request an enforcement preview THEN the system SHALL show what will be removed from each service
3. WHEN I execute enforcement THEN the system SHALL perform the actions and provide detailed progress feedback
4. WHEN enforcement completes THEN the system SHALL display a summary of all actions taken
5. IF enforcement fails partially THEN the system SHALL show which actions succeeded and which failed with reasons

### Requirement 5

**User Story:** As a developer, I want consistent error handling and API responses, so that the frontend can provide meaningful feedback to users.

#### Acceptance Criteria

1. WHEN API calls succeed THEN responses SHALL follow the format: `{success: true, data: any, message?: string}`
2. WHEN API calls fail THEN responses SHALL follow the format: `{success: false, message: string, error_code?: string}`
3. WHEN authentication fails THEN the system SHALL return 401 status with clear error messages
4. WHEN validation fails THEN the system SHALL return 400 status with field-specific error details
5. IF server errors occur THEN the system SHALL return 500 status with generic error messages (no sensitive data)

### Requirement 6

**User Story:** As a user, I want the application to work reliably with proper loading states, so that I understand what's happening when I interact with the system.

#### Acceptance Criteria

1. WHEN data is loading THEN the UI SHALL display appropriate loading indicators
2. WHEN operations are in progress THEN buttons SHALL be disabled with loading states
3. WHEN errors occur THEN the UI SHALL display user-friendly error messages with retry options
4. WHEN operations complete successfully THEN the UI SHALL provide confirmation feedback
5. IF the backend is unreachable THEN the UI SHALL indicate offline status and suggest troubleshooting steps