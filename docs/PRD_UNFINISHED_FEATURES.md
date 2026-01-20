# PRD: Unfinished Features Completion Plan

## Summary
This PRD documents unfinished features identified in the current codebase and defines a plan to complete them. The focus is to ship core functionality with clear user value, remove placeholder/stub implementations, and harden security and observability before scaling.

## Goals
- Remove placeholder or stubbed implementations that block real usage.
- Restore disabled backend functionality (notably OAuth).
- Deliver complete enforcement, analytics, and monitoring workflows.
- Establish a clear release path with measurable outcomes.

## Non-Goals
- Redesign the product UX or UI themes.
- Expand to new streaming providers beyond the current scope.
- Overhaul the database schema except where required for completion.

## Unfinished Features (Current Findings)
1. Backend OAuth is disabled due to SQLx cache constraints.
2. Token vault and provider token lifecycle are stubbed or partial.
3. Analytics endpoints include placeholder data and missing authorization.
4. Graph service endpoints return placeholder responses.
5. Monitoring metrics and system resource telemetry are placeholders.
6. LanceDB client integration is marked TODO.
7. Enforcement services contain placeholder rollback and counts.
8. Moderation and review flows lack role checks and real logic.
9. Job queue and background processing are stubbed.

## Product Requirements

### 1) Backend OAuth Re-Enablement
- Problem: Frontend OAuth is complete; backend routes and queries are disabled.
- Requirements:
  - Re-enable OAuth routes and handlers.
  - Regenerate SQLx cache and fix offline build issues.
  - End-to-end OAuth flow for Google, Apple, GitHub.
- Acceptance Criteria:
  - OAuth login works end-to-end with a real provider.
  - Token encryption and account linking persists in DB.
  - API returns correct error states when providers fail.

### 2) Token Vault and Provider Token Lifecycle
- Problem: Token vault and refresh behavior are placeholders.
- Requirements:
  - Implement token refresh for supported providers.
  - Implement connection health checks and error codes.
  - Implement real connection deletion and revocation flows.
- Acceptance Criteria:
  - Expired tokens refresh automatically or surface a user action.
  - Connection health checks show accurate status.
  - Revocation clears tokens and updates state.

### 3) Analytics Completion
- Problem: Analytics endpoints return placeholder data and lack access controls.
- Requirements:
  - Implement time-series analytics and comparative analytics.
  - Add role-based access for admin/system views.
  - Add export support (CSV/JSON).
- Acceptance Criteria:
  - Time-series endpoints return real data for the requested range.
  - Comparative analytics apply privacy protection.
  - Admin endpoints are gated and audited.

### 4) Graph Service Completion
- Problem: Graph handlers return placeholder responses.
- Requirements:
  - Implement real graph queries (artist nodes, edges, relationships).
  - Implement collaboration and sync flows where applicable.
- Acceptance Criteria:
  - Graph endpoints return actual graph entities.
  - Sync endpoints operate on real data and return health status.

### 5) Monitoring and System Telemetry
- Problem: Monitoring service returns placeholders and cannot access disk metrics.
- Requirements:
  - Implement service-level metrics collection.
  - Implement system resource metrics (CPU, RAM, disk).
  - Expose metrics in Prometheus-friendly format.
- Acceptance Criteria:
  - /metrics surfaces real values for CPU, memory, disk.
  - Dashboardable latency and error metrics exist for key endpoints.

### 6) LanceDB Client Integration
- Problem: LanceDB client implementation is TODO.
- Requirements:
  - Implement vector store interface for artist similarity search.
  - Wire into entity resolution where designed.
- Acceptance Criteria:
  - Vector search APIs return results in staging.
  - Existing resolution pipeline can query LanceDB.

### 7) Enforcement and Provider-Specific Gaps
- Problem: Enforcement rollback and counts are placeholders.
- Requirements:
  - Implement Spotify enforcement rollback behavior.
  - Replace placeholder enforcement counts and estimates.
  - Implement missing Apple Music playlist analysis logic.
- Acceptance Criteria:
  - Enforcement rollback executes a real provider action.
  - Counts in the UI match backend results.

### 8) Moderation and Appeals
- Problem: Role checks and review logic are TODO.
- Requirements:
  - Implement moderator and reviewer role checks.
  - Implement appeal review flows with audit logging.
- Acceptance Criteria:
  - Only moderators can verify offenses.
  - Appeals are persisted with review status and audit records.

### 9) Job Queue and Background Processing
- Problem: Job queue services are stubbed.
- Requirements:
  - Implement a job queue for enforcement and sync tasks.
  - Add retry policies and failure handling.
- Acceptance Criteria:
  - Background jobs are durable and observable.
  - Failed jobs are retried and surfaced in logs.

## Milestones
1. Phase 0: OAuth backend re-enable + SQLx cache fix.
2. Phase 1: Token vault lifecycle + enforcement rollback.
3. Phase 2: Analytics completion + monitoring telemetry.
4. Phase 3: Graph service + LanceDB integration.
5. Phase 4: Moderation/appeals + job queue hardening.

## Success Metrics
- OAuth flow completion rate > 95% in staging.
- Enforcement success rate reported with real counts.
- Median API latency < 250ms for key endpoints.
- 0 placeholder responses in production paths.

## Dependencies and Risks
- SQLx offline cache regeneration requires live DB access.
- Provider API limits and auth requirements may delay integration.
- Analytics and monitoring require data volume to validate.

## Open Questions
- Which provider integrations are in scope for launch?
- What is the target role model for moderators and reviewers?
- What telemetry stack (Prometheus/Grafana) is the source of truth?
