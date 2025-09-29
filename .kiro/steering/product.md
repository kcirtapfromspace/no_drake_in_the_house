---
inclusion: always
---

# Product Guidelines: No Drake in the House

## Product Overview

A multi-platform music streaming blocklist management system that provides centralized control for users to avoid unpalatable artists across streaming services.

## Core Domain Concepts

### DNP (Do Not Play) Lists
- **Personal Lists**: User-owned blocklists with full CRUD operations
- **Community Lists**: Shared, subscribable blocklists with moderation
- **Enforcement Actions**: Automated removal of blocked content from user libraries

### Entity Resolution
- Artists must be matched across platforms using external IDs (Spotify ID, Apple Music ID, etc.)
- Handle artist name variations and aliases through fuzzy matching
- Maintain canonical artist records with platform-specific mappings

### Platform Adapters
- Each streaming service requires its own adapter implementation
- OAuth flows must be service-specific but follow consistent patterns
- Library scanning and enforcement must be platform-aware

## Development Principles

### Security Requirements
- All external tokens MUST be encrypted at rest using AES-GCM
- User authentication requires JWT with optional 2FA
- All API endpoints require authentication except public health checks
- Audit logging required for all user actions and system operations

### Data Consistency
- Use database transactions for multi-table operations
- Implement optimistic locking for concurrent list modifications
- Maintain referential integrity between users, artists, and lists

### API Design Patterns
- RESTful endpoints with consistent HTTP status codes
- Use structured error responses with error codes and messages
- Implement pagination for list endpoints (limit/offset)
- Version API endpoints when breaking changes are needed

### Code Organization
- Business logic belongs in services layer, not handlers
- Models should only contain data structures and basic validation
- Use dependency injection for service dependencies
- Separate read and write operations where performance matters

### Testing Standards
- Integration tests for all API endpoints
- Unit tests for business logic in services
- Mock external API calls using wiremock
- Use database transactions in tests for isolation

### ML/Recommendation Features
- All automated suggestions MUST include source citations
- Implement human-in-the-loop validation for high-impact recommendations
- User feedback (accept/reject/mute) must be captured for model improvement
- Transparency: users must be able to see why recommendations were made

## Implementation Guidelines

### Error Handling
- Use structured error types with context
- Log errors with correlation IDs for tracing
- Return user-friendly error messages in API responses
- Implement circuit breakers for external service calls

### Performance Considerations
- Use connection pooling for database and Redis
- Implement caching for frequently accessed data (artist lookups, community lists)
- Batch operations when processing large libraries
- Use async/await patterns throughout the codebase

### Deployment Standards
- Use proper environment configuration (no hardcoded values)
- Implement health checks for all services
- Use structured logging with correlation IDs
- Follow 12-factor app principles

## Feature Development Rules

### New Platform Integration
- Create platform-specific models in `models/{platform}.rs`
- Implement OAuth flow in `services/{platform}.rs`
- Add enforcement logic in `services/{platform}_enforcement.rs`
- Update entity resolution to handle platform-specific artist IDs

### Community Features
- All community list operations require moderation capabilities
- Implement subscription management with notification preferences
- Track contribution metrics for community engagement
- Ensure scalable list sharing and discovery

### Enforcement Engine
- Always preview changes before execution
- Implement rollback capabilities for enforcement actions
- Track all enforcement operations in audit logs
- Support dry-run mode for testing enforcement logic