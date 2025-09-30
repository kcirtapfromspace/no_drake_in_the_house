# Requirements Document

## Introduction

The current development environment for the "No Drake in the House" music streaming blocklist manager is broken and unusable. Multiple critical issues prevent developers from running the application locally, including missing database tables, port conflicts, UI rendering problems, and broken setup scripts. This feature will establish a stable, reliable development environment using Minikube with Tilt orchestration, eliminating Docker Compose dependencies and providing a consistent developer experience.

## Requirements

### Requirement 1

**User Story:** As a developer, I want a working local development environment, so that I can develop and test features without encountering setup failures.

#### Acceptance Criteria

1. WHEN a developer runs the setup process THEN the environment SHALL start successfully without manual intervention
2. WHEN the backend service starts THEN all required database tables SHALL exist and be accessible
3. WHEN the frontend loads THEN it SHALL display properly without UI rendering issues
4. WHEN a user attempts to register an account THEN the process SHALL complete successfully without 500 errors
5. IF the environment is torn down and restarted THEN it SHALL return to a working state consistently

### Requirement 2

**User Story:** As a developer, I want database migrations to run automatically, so that I don't encounter "relation does not exist" errors.

#### Acceptance Criteria

1. WHEN the backend pod starts THEN database migrations SHALL run via a separate Kubernetes Job before the service accepts requests
2. WHEN migrations complete THEN all required tables (users, artists, user_artist_blocks) SHALL exist
3. IF migrations fail THEN the system SHALL provide clear error messages and prevent the backend from starting
4. WHEN the database is reset THEN migrations SHALL re-run successfully on the next startup
5. WHEN migrations complete successfully THEN any required seed data SHALL be loaded automatically

### Requirement 3

**User Story:** As a developer, I want port conflicts resolved through proper configuration, so that services start without "Address already in use" errors.

#### Acceptance Criteria

1. WHEN services start THEN they SHALL use configurable ports defined in environment variables
2. WHEN a local port is already in use THEN Tilt SHALL provide clear guidance on changing port-forward configuration
3. WHEN multiple developers run the environment simultaneously THEN each SHALL use isolated Kubernetes namespaces
4. IF a service fails to bind to a port THEN it SHALL provide clear error messages with resolution steps
5. WHEN port-forwarding conflicts occur THEN developers SHALL be able to modify local ports without affecting the cluster

### Requirement 4

**User Story:** As a developer, I want the frontend UI to render properly, so that I can interact with all interface elements effectively.

#### Acceptance Criteria

1. WHEN the registration form loads THEN password validation icons SHALL be appropriately sized and positioned
2. WHEN form validation occurs THEN feedback SHALL be clearly visible and not obstruct other UI elements
3. WHEN the application loads THEN all components SHALL render with proper styling and layout
4. IF UI elements overlap or are oversized THEN they SHALL be corrected to maintain usability

### Requirement 5

**User Story:** As a developer, I want a Tilt-based development workflow, so that I have fast, reliable builds and deployments.

#### Acceptance Criteria

1. WHEN I run `tilt up` THEN all services SHALL build, deploy, and become ready automatically
2. WHEN I make code changes THEN Tilt SHALL rebuild and redeploy only the affected services quickly
3. WHEN services are ready THEN they SHALL be accessible via port forwarding on predictable local ports
4. IF a service fails to start THEN Tilt SHALL provide clear logs and status information
5. WHEN I run `tilt down` THEN all Kubernetes resources and port-forwards SHALL be cleaned up properly

### Requirement 6

**User Story:** As a developer, I want working user registration, so that I can test authentication flows end-to-end.

#### Acceptance Criteria

1. WHEN I submit a valid registration form THEN a new user account SHALL be created successfully
2. WHEN registration completes THEN I SHALL receive appropriate success feedback
3. WHEN I attempt to register with invalid data THEN I SHALL receive clear validation errors
4. WHEN I attempt to register with a duplicate email THEN I SHALL receive a specific "email already exists" error
5. IF the backend is unavailable THEN the frontend SHALL display appropriate error messages
6. WHEN registration succeeds THEN I SHALL be able to log in with the created credentials

### Requirement 7

**User Story:** As a developer, I want reliable service dependencies, so that Postgres and Redis are always available when the backend starts.

#### Acceptance Criteria

1. WHEN the backend starts THEN Postgres SHALL be ready and accepting connections
2. WHEN the backend starts THEN Redis SHALL be ready and accepting connections
3. IF dependencies are not ready THEN the backend SHALL wait or retry until they become available
4. WHEN dependency services restart THEN the backend SHALL reconnect automatically
5. IF a dependency fails permanently THEN the system SHALL provide clear error messages

### Requirement 8

**User Story:** As a developer, I want clear documentation and setup instructions, so that new team members can get started quickly.

#### Acceptance Criteria

1. WHEN a new developer joins THEN they SHALL be able to set up the environment using documented steps
2. WHEN setup instructions are followed THEN the environment SHALL work on the first attempt
3. WHEN troubleshooting is needed THEN documentation SHALL provide solutions for common issues
4. IF prerequisites are missing (Rust toolchain, Node.js, kubectl, minikube, tilt) THEN the setup process SHALL detect and report them clearly
5. WHEN the environment is working THEN developers SHALL know how to access all services and tools

### Requirement 9

**User Story:** As a developer, I want proper configuration and secrets management, so that services can connect to dependencies securely.

#### Acceptance Criteria

1. WHEN services start THEN all required environment variables (DATABASE_URL, REDIS_URL, JWT_SECRET, VITE_API_URL) SHALL be available
2. WHEN configuration changes THEN services SHALL reload or restart automatically
3. WHEN sensitive data is stored THEN it SHALL use Kubernetes Secrets rather than plain ConfigMaps
4. WHEN the frontend makes API requests THEN CORS SHALL be properly configured to allow the requests
5. IF configuration is missing or invalid THEN services SHALL fail fast with clear error messages