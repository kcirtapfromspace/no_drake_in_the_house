# PRD: Platform Hardening

## Introduction

Platform Hardening closes security gaps, removes placeholder analytics, and
restores incomplete core features in the backend and API contract tests.
The focus is backend correctness, authorization, and reliable metrics.

## Goals

- Enforce role and ownership authorization on sensitive endpoints.
- Replace placeholder analytics with accurate, verifiable metrics.
- Restore or explicitly gate incomplete platform features.
- Improve test coverage for API contract behavior.

## User Stories

### US-001: Enforce authorization on sensitive endpoints
**Description:** As a platform owner, I want sensitive analytics and
moderation endpoints to enforce role and ownership checks so that data is not
exposed or actions are not performed by unauthorized users.

**Acceptance Criteria:**
- [ ] Community list impact endpoints restrict access to list owners or
      public lists.
- [ ] Offense verification requires a moderator role.
- [ ] Unauthorized requests return 403 with a consistent error shape.
- [ ] Typecheck passes.
- [ ] Tests pass.

### US-002: Replace ad-hoc admin checks with role permissions
**Description:** As a platform owner, I want admin-only endpoints to use a
real role or permission check so that admin access is not based on email
patterns.

**Acceptance Criteria:**
- [ ] Admin-only analytics endpoints check a role or permission source of
      truth.
- [ ] Non-admin requests return 403 with a consistent error shape.
- [ ] Typecheck passes.
- [ ] Tests pass.

### US-003: Implement LanceDB client operations
**Description:** As a platform owner, I want vector DB operations to
actually insert, search, and delete embeddings so features using semantic
search behave correctly.

**Acceptance Criteria:**
- [ ] News and artist embedding insert operations persist records.
- [ ] Similarity search returns results for seeded embeddings.
- [ ] Delete operations remove embeddings by ID.
- [ ] Dimension mismatch returns a clear error.
- [ ] Typecheck passes.
- [ ] Tests pass.

### US-004: Complete enforcement analytics metrics
**Description:** As a product owner, I want enforcement analytics to return
real metrics so reports are trustworthy.

**Acceptance Criteria:**
- [ ] Provider breakdown includes most common operations.
- [ ] Operation breakdown includes avg duration computed from stored timing.
- [ ] Recent failures include retry count from stored data.
- [ ] Content breakdown includes scanned totals and filter rate.
- [ ] Blocked artist stats include content types blocked.
- [ ] Typecheck passes.
- [ ] Tests pass.

### US-005: Complete community list analytics metrics
**Description:** As a product owner, I want community list analytics to
reflect actual impact and growth metrics so list owners get real insights.

**Acceptance Criteria:**
- [ ] Content filtered by community lists is tracked and returned.
- [ ] Subscriber satisfaction and impact metrics are computed from stored
      signals.
- [ ] Growth metrics include subscriber growth rate and engagement score.
- [ ] Typecheck passes.
- [ ] Tests pass.

### US-006: Complete system performance analytics
**Description:** As a platform owner, I want the system performance
analytics to report real health and performance metrics.

**Acceptance Criteria:**
- [ ] Overall health aggregates configured health signals.
- [ ] API performance metrics are populated from available telemetry.
- [ ] Enforcement performance metrics are populated from stored data.
- [ ] System resource and alert tracking is reported if data exists.
- [ ] Typecheck passes.
- [ ] Tests pass.

### US-007: Populate user profile correctness fields
**Description:** As a product owner, I want user profile responses to
reflect real email verification status, OAuth accounts, and last login so
clients show accurate data.

**Acceptance Criteria:**
- [ ] Email verification status is sourced from stored data.
- [ ] OAuth accounts are loaded from persistence and returned.
- [ ] Last login is populated from a stored source of truth.
- [ ] Typecheck passes.
- [ ] Tests pass.

### US-008: Restore or explicitly gate account merge
**Description:** As a platform owner, I want account merge to either work
or return a stable, explicit "not available" response that clients can
handle.

**Acceptance Criteria:**
- [ ] Account merge requests return real functionality or a stable
      "feature unavailable" error code and message.
- [ ] API contract is documented for the chosen behavior.
- [ ] Typecheck passes.
- [ ] Tests pass.

### US-009: Strengthen contract tests for API parsing
**Description:** As an engineer, I want contract tests to verify rate limit
and pagination parsing so regressions are caught.

**Acceptance Criteria:**
- [ ] Rate limit parsing is asserted in tests with expected state changes.
- [ ] Pagination parsing is asserted for supported formats.
- [ ] Typecheck passes.
- [ ] Tests pass.

## Functional Requirements

- FR-1: Add role and ownership checks for community list impact analytics.
- FR-2: Require moderator role for offense verification.
- FR-3: Replace admin access checks with role/permission checks.
- FR-4: Implement LanceDB insert, search, and delete operations.
- FR-5: Populate enforcement analytics fields with computed values.
- FR-6: Populate community list analytics with real metrics.
- FR-7: Populate system performance analytics with real telemetry data.
- FR-8: Populate user profile correctness fields from stored sources.
- FR-9: Provide a stable account merge behavior and document it.
- FR-10: Add explicit contract test assertions for rate limits and
  pagination parsing.

## Non-Goals (Out of Scope)

- No new UI or frontend changes.
- No redesign of analytics dashboards.
- No new external provider integrations.
- No data model overhauls beyond what is needed for correctness.

## Technical Considerations

- Reuse existing role/permission infrastructure if present.
- Keep error shapes consistent across auth failures.
- Derive metrics from existing tables before introducing new storage.
- Add telemetry wiring only where data already exists.

## Success Metrics

- All listed sensitive endpoints enforce authorization.
- Analytics responses contain no placeholder metrics in the listed areas.
- Contract tests assert rate limit and pagination parsing behavior.
- All tests pass in CI for backend and contract tests.

## Open Questions

- What is the source of truth for roles and permissions today?
- Which telemetry sources are already available for performance metrics?
- Should account merge be fully restored or feature-gated for now?
