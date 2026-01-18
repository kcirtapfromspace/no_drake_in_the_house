# Product Requirements Document: Feature Completion

**Project:** No Drake in the House
**Version:** 1.0
**Date:** January 2026
**Status:** Draft

---

## Executive Summary

This PRD outlines the requirements for completing all partially implemented and planned features in the No Drake in the House platform. Based on a comprehensive codebase evaluation, the platform is approximately 75-80% complete with strong core functionality but notable gaps in analytics, vector search, mobile applications, and testing coverage.

### Current State Assessment

| Category | Status | Completion |
|----------|--------|------------|
| Core DNP Management | ✅ Complete | 100% |
| Community Lists | ✅ Complete | 100% |
| Authentication & Security | ✅ Complete | 95% |
| Platform Integrations | ✅ Complete | 90% |
| Analytics & Insights | ⚠️ Partial | 40% |
| Vector Search (LanceDB) | ❌ Stub Only | 5% |
| Mobile Applications | ⚠️ Skeleton | 15% |
| End-to-End Testing | ❌ Not Started | 0% |

---

## Table of Contents

1. [Priority 1: Vector Search Implementation (LanceDB)](#priority-1-vector-search-implementation-lancedb)
2. [Priority 2: Analytics Service Completion](#priority-2-analytics-service-completion)
3. [Priority 3: OAuth & Authentication Enhancements](#priority-3-oauth--authentication-enhancements)
4. [Priority 4: Enforcement Service Completion](#priority-4-enforcement-service-completion)
5. [Priority 5: Mobile Application Development](#priority-5-mobile-application-development)
6. [Priority 6: End-to-End Testing Suite](#priority-6-end-to-end-testing-suite)
7. [Priority 7: Service Consolidation & Technical Debt](#priority-7-service-consolidation--technical-debt)
8. [Implementation Roadmap](#implementation-roadmap)
9. [Success Metrics](#success-metrics)

---

## Priority 1: Vector Search Implementation (LanceDB)

### Overview

The LanceDB client (`backend/src/services/databases/lancedb_client.rs`) is currently a stub implementation. All core functions have placeholder logic marked with TODO comments. This feature is critical for semantic search capabilities across news articles and artist similarity recommendations.

### Current State

```rust
// Current stub example from lancedb_client.rs
pub async fn insert_news_embedding(&self, ...) -> Result<(), LanceDbError> {
    // TODO: Implement with proper lancedb 0.23 API
    // Validates input but doesn't actually insert
    Ok(())
}
```

### Requirements

#### 1.1 News Embedding Storage

**Description:** Implement full vector storage for news article embeddings to enable semantic search and similarity matching.

**Functional Requirements:**
- FR-1.1.1: Insert news embeddings with associated metadata (article_id, title, timestamp, source)
- FR-1.1.2: Support batch insertion of up to 1000 embeddings per operation
- FR-1.1.3: Automatically handle schema creation on first insert
- FR-1.1.4: Support embedding dimensions of 384, 768, and 1536 (configurable)

**Technical Requirements:**
- TR-1.1.1: Use LanceDB 0.23+ async API
- TR-1.1.2: Implement connection pooling with max 10 concurrent connections
- TR-1.1.3: Add retry logic with exponential backoff for transient failures
- TR-1.1.4: Implement proper error mapping to `LanceDbError` enum

**Acceptance Criteria:**
- [ ] Embeddings persist across database restarts
- [ ] Insert latency < 50ms for single embedding, < 500ms for batch of 100
- [ ] Duplicate article_id handling (update existing or reject)
- [ ] Unit tests with at least 90% code coverage

#### 1.2 News Similarity Search

**Description:** Implement semantic search to find similar news articles based on embedding similarity.

**Functional Requirements:**
- FR-1.2.1: Search by embedding vector with configurable top-k results (default: 10)
- FR-1.2.2: Support minimum similarity threshold filtering
- FR-1.2.3: Return results with similarity scores and metadata
- FR-1.2.4: Support filtering by date range, source, and category

**Technical Requirements:**
- TR-1.2.1: Implement ANN (Approximate Nearest Neighbor) search using IVF-PQ index
- TR-1.2.2: Search latency < 100ms for 1M vectors
- TR-1.2.3: Support both L2 distance and cosine similarity metrics

**Acceptance Criteria:**
- [ ] Returns relevant results for known test queries
- [ ] Performance benchmarks pass for target dataset sizes
- [ ] Edge cases handled (empty database, no matches, invalid embeddings)

#### 1.3 Artist Embedding Storage

**Description:** Store artist embeddings for music taste profiling and similarity recommendations.

**Functional Requirements:**
- FR-1.3.1: Insert artist embeddings with metadata (artist_id, name, genres, popularity)
- FR-1.3.2: Support incremental updates as artist data changes
- FR-1.3.3: Link embeddings to MusicBrainz IDs when available

**Technical Requirements:**
- TR-1.3.1: Separate table/collection from news embeddings
- TR-1.3.2: Support embedding versioning for model updates
- TR-1.3.3: Implement soft delete for removed artists

**Acceptance Criteria:**
- [ ] Artist embeddings queryable within 24 hours of artist addition
- [ ] Embedding updates don't cause search index corruption
- [ ] Integration with existing artist catalog service

#### 1.4 Artist Similarity Search

**Description:** Find similar artists based on embedding similarity for "at-risk" artist identification.

**Functional Requirements:**
- FR-1.4.1: Search for artists similar to blocked artists
- FR-1.4.2: Support batch queries (find similar to N artists)
- FR-1.4.3: Filter out already-blocked artists from results
- FR-1.4.4: Return similarity explanation factors

**Technical Requirements:**
- TR-1.4.1: Hybrid search combining vector similarity and metadata filters
- TR-1.4.2: Cache frequent query results with 1-hour TTL
- TR-1.4.3: Support for negative examples (dissimilar artists to exclude)

**Acceptance Criteria:**
- [ ] Similarity recommendations align with genre/style expectations
- [ ] Performance: < 200ms for single artist, < 1s for batch of 10
- [ ] Integration tests with graph service for collaboration data

#### 1.5 Embedding Deletion

**Description:** Implement proper cleanup of embeddings when news articles or artists are removed.

**Functional Requirements:**
- FR-1.5.1: Delete single embedding by ID
- FR-1.5.2: Batch delete multiple embeddings
- FR-1.5.3: Delete all embeddings for a given source/category

**Technical Requirements:**
- TR-1.5.1: Maintain index consistency after deletions
- TR-1.5.2: Support soft delete with configurable retention period
- TR-1.5.3: Implement vacuum/compaction for reclaiming space

**Acceptance Criteria:**
- [ ] Deleted embeddings not returned in search results
- [ ] No orphaned data after cascade deletions
- [ ] Storage space reclaimed within 24 hours of deletion

### Files to Modify

| File | Changes Required |
|------|------------------|
| `backend/src/services/databases/lancedb_client.rs` | Full implementation of all stub methods |
| `backend/src/services/databases/mod.rs` | Export updated client |
| `backend/src/services/news_pipeline/ingestion.rs` | Integrate embedding storage |
| `backend/src/services/graph_service/mod.rs` | Integrate artist similarity |
| `backend/Cargo.toml` | Verify lancedb dependency version |

---

## Priority 2: Analytics Service Completion

### Overview

The analytics service (`backend/src/services/analytics.rs`) has numerous unimplemented metrics and stub calculations. These gaps affect dashboard accuracy and user insights.

### Current State

Multiple TODO comments and placeholder implementations:
- Line 394: `most_common_operations` not calculated
- Line 445: `avg_duration_ms` not calculated
- Line 484: `retry_count` not tracked
- Lines 541-543: `total_items_scanned`, `filter_rate` not tracked
- Line 578: `content_types_blocked` not implemented
- Line 599: ML-based recommendations not implemented
- Lines 662-672: Community list analytics stubs
- Lines 690-776: System health tracking stubs

### Requirements

#### 2.1 Operation Analytics

**Description:** Implement comprehensive tracking of user operations and API usage patterns.

**Functional Requirements:**
- FR-2.1.1: Track all API endpoint calls with timing data
- FR-2.1.2: Calculate most common operations per user/globally
- FR-2.1.3: Compute average, P50, P95, P99 operation durations
- FR-2.1.4: Track operation success/failure rates

**Technical Requirements:**
- TR-2.1.1: Store metrics in DuckDB for OLAP queries
- TR-2.1.2: Aggregate metrics at 1-minute, 1-hour, 1-day granularities
- TR-2.1.3: Implement sliding window calculations for real-time stats
- TR-2.1.4: Memory-efficient streaming aggregation

**Acceptance Criteria:**
- [ ] Dashboard shows accurate operation counts within 1-minute lag
- [ ] Historical queries performant for up to 1 year of data
- [ ] Resource usage < 100MB RAM for aggregation buffers

#### 2.2 Sync Performance Metrics

**Description:** Track catalog synchronization performance across all platforms.

**Functional Requirements:**
- FR-2.2.1: Track total items scanned per sync operation
- FR-2.2.2: Calculate filter rate (blocked/total items)
- FR-2.2.3: Track retry counts and failure reasons
- FR-2.2.4: Monitor sync duration trends over time

**Technical Requirements:**
- TR-2.2.1: Per-platform and aggregate statistics
- TR-2.2.2: Real-time sync progress streaming via WebSocket
- TR-2.2.3: Anomaly detection for sync performance degradation

**Acceptance Criteria:**
- [ ] Sync metrics visible within 30 seconds of sync completion
- [ ] Historical comparison with previous syncs available
- [ ] Alerts trigger on > 50% performance degradation

#### 2.3 Content Blocking Analytics

**Description:** Analyze patterns in content blocking across users and categories.

**Functional Requirements:**
- FR-2.3.1: Track content types blocked (artists, albums, tracks)
- FR-2.3.2: Aggregate blocking reasons/categories
- FR-2.3.3: Calculate blocking velocity (blocks per time period)
- FR-2.3.4: Identify trending blocked content

**Technical Requirements:**
- TR-2.3.1: Privacy-preserving aggregation (no individual user exposure)
- TR-2.3.2: Differential privacy for public statistics
- TR-2.3.3: Real-time trending calculation with decay factor

**Acceptance Criteria:**
- [ ] Category breakdown accurate within 5% margin
- [ ] Trending updates within 5 minutes of blocking activity
- [ ] Privacy compliance verified (no PII in aggregate stats)

#### 2.4 ML-Based Recommendations

**Description:** Implement machine learning-based recommendations for DNP list curation.

**Functional Requirements:**
- FR-2.4.1: Suggest artists similar to already-blocked artists
- FR-2.4.2: Recommend community lists based on user preferences
- FR-2.4.3: Predict user interest in news-discovered offenses
- FR-2.4.4: Personalized notification prioritization

**Technical Requirements:**
- TR-2.4.1: Integration with LanceDB vector search (Priority 1)
- TR-2.4.2: Collaborative filtering using user block patterns
- TR-2.4.3: Content-based filtering using artist metadata
- TR-2.4.4: Online learning for preference updates

**Acceptance Criteria:**
- [ ] Recommendation relevance > 60% (user accepts recommendation)
- [ ] Cold-start handling for new users
- [ ] Model retraining pipeline operational

#### 2.5 Community List Analytics

**Description:** Track community list performance and subscriber engagement.

**Functional Requirements:**
- FR-2.5.1: Track subscriber counts and growth rates
- FR-2.5.2: Calculate subscriber satisfaction (retention, unsubscribes)
- FR-2.5.3: Measure list impact (artists blocked, enforcement actions)
- FR-2.5.4: Track curator activity and response times

**Technical Requirements:**
- TR-2.5.1: Time-series storage for growth tracking
- TR-2.5.2: Cohort analysis for retention metrics
- TR-2.5.3: A/B testing framework for list recommendations

**Acceptance Criteria:**
- [ ] Subscriber metrics update in real-time
- [ ] Satisfaction score correlates with actual retention
- [ ] Impact metrics accurate within 1% error margin

#### 2.6 System Health Monitoring

**Description:** Implement comprehensive system health tracking and alerting.

**Functional Requirements:**
- FR-2.6.1: Track database connection pool utilization
- FR-2.6.2: Monitor Redis cache hit rates
- FR-2.6.3: Track external API response times (Spotify, Apple Music, etc.)
- FR-2.6.4: Monitor background job queue depths

**Technical Requirements:**
- TR-2.6.1: Prometheus metrics export
- TR-2.6.2: Grafana dashboard templates
- TR-2.6.3: AlertManager integration for notifications
- TR-2.6.4: Health check aggregation endpoint

**Acceptance Criteria:**
- [ ] All metrics visible in Grafana dashboard
- [ ] Alerts fire within 1 minute of threshold breach
- [ ] Historical health data retained for 30 days

### Files to Modify

| File | Changes Required |
|------|------------------|
| `backend/src/services/analytics.rs` | Implement all TODO items |
| `backend/src/services/analytics_service/` | Additional analytics modules |
| `backend/src/handlers/analytics.rs` | Update handlers for new metrics |
| `backend/src/monitoring.rs` | Prometheus metric registration |
| `k8s/grafana/` | Dashboard configurations |

---

## Priority 3: OAuth & Authentication Enhancements

### Overview

Authentication is largely complete but has gaps in error handling and advanced OAuth features.

### Current State

- Line 795 in `auth.rs`: OAuth error logging marked as TODO
- PKCE (Proof Key for Code Exchange) not implemented
- Some session functions disabled pending SQLx cache updates

### Requirements

#### 3.1 OAuth Error Handling Enhancement

**Description:** Implement comprehensive OAuth error logging and user feedback.

**Functional Requirements:**
- FR-3.1.1: Log all OAuth failures with error codes and context
- FR-3.1.2: Provide user-friendly error messages for common failures
- FR-3.1.3: Track OAuth failure patterns for debugging
- FR-3.1.4: Implement automatic retry for transient OAuth errors

**Technical Requirements:**
- TR-3.1.1: Structured logging with correlation IDs
- TR-3.1.2: Integration with audit logging service
- TR-3.1.3: Error categorization (user error, provider error, system error)
- TR-3.1.4: PII scrubbing from error logs

**Acceptance Criteria:**
- [ ] All OAuth errors logged with sufficient context for debugging
- [ ] User sees actionable error message, not technical details
- [ ] Error rates trackable in monitoring dashboard

#### 3.2 PKCE Implementation

**Description:** Implement PKCE for enhanced OAuth security, especially for mobile clients.

**Functional Requirements:**
- FR-3.2.1: Generate code_verifier and code_challenge for auth requests
- FR-3.2.2: Validate code_verifier on token exchange
- FR-3.2.3: Support S256 challenge method
- FR-3.2.4: Backwards compatible with non-PKCE flows

**Technical Requirements:**
- TR-3.2.1: Cryptographically secure random code_verifier generation
- TR-3.2.2: SHA-256 hashing for code_challenge
- TR-3.2.3: PKCE state storage in Redis with 10-minute TTL
- TR-3.2.4: Per-provider PKCE configuration

**Acceptance Criteria:**
- [ ] PKCE flow works with Spotify, Apple Music, Google
- [ ] Mobile apps use PKCE by default
- [ ] Security audit passes for PKCE implementation

#### 3.3 Session Management Improvements

**Description:** Enable disabled session functions and improve session handling.

**Functional Requirements:**
- FR-3.3.1: Re-enable session retrieval functions
- FR-3.3.2: Implement session listing for users (view active sessions)
- FR-3.3.3: Allow users to revoke specific sessions
- FR-3.3.4: Automatic session cleanup for expired sessions

**Technical Requirements:**
- TR-3.3.1: Update SQLx queries for session operations
- TR-3.3.2: Session metadata (device, location, last active)
- TR-3.3.3: Rate limiting on session revocation
- TR-3.3.4: Background job for session cleanup

**Acceptance Criteria:**
- [ ] Users can view all active sessions
- [ ] Session revocation takes effect within 1 minute
- [ ] No orphaned session data after cleanup

### Files to Modify

| File | Changes Required |
|------|------------------|
| `backend/src/services/auth.rs` | Error handling, PKCE, session functions |
| `backend/src/services/oauth.rs` | PKCE integration |
| `backend/src/handlers/auth.rs` | Session listing/revocation endpoints |
| `backend/src/models/session.rs` | Session metadata model |

---

## Priority 4: Enforcement Service Completion

### Overview

Enforcement services for Spotify and Apple Music are functional but have incomplete features.

### Current State

- Line 356 in `spotify_enforcement.rs`: Rollback disabled pending full service enablement
- Line 94 in `offense.rs`: Moderator role check TODO

### Requirements

#### 4.1 Spotify Enforcement Rollback

**Description:** Enable full rollback functionality for Spotify enforcement actions.

**Functional Requirements:**
- FR-4.1.1: Rollback individual track/artist blocks
- FR-4.1.2: Rollback all blocks from a sync session
- FR-4.1.3: Rollback blocks from specific community list
- FR-4.1.4: Preview rollback impact before execution

**Technical Requirements:**
- TR-4.1.1: Transaction logging for all enforcement actions
- TR-4.1.2: Idempotent rollback operations
- TR-4.1.3: Partial rollback support (continue on individual failures)
- TR-4.1.4: Rate limiting to respect Spotify API limits

**Acceptance Criteria:**
- [ ] Full rollback restores library to pre-enforcement state
- [ ] Partial failures don't corrupt enforcement state
- [ ] Rollback audit trail maintained

#### 4.2 Moderator Verification System

**Description:** Implement role-based access control for offense verification.

**Functional Requirements:**
- FR-4.2.1: Moderator role assignment by admins
- FR-4.2.2: Only moderators can verify submitted offenses
- FR-4.2.3: Verification workflow (pending -> verified/rejected)
- FR-4.2.4: Moderator activity logging

**Technical Requirements:**
- TR-4.2.1: Role column in users table
- TR-4.2.2: Middleware for role-based route protection
- TR-4.2.3: Audit log integration for all moderator actions
- TR-4.2.4: Rate limiting for verification actions

**Acceptance Criteria:**
- [ ] Non-moderators cannot access verification endpoints
- [ ] All verification actions logged with moderator identity
- [ ] Admin can promote/demote moderators

#### 4.3 Apple Music Enforcement Parity

**Description:** Ensure Apple Music enforcement matches Spotify functionality.

**Functional Requirements:**
- FR-4.3.1: Full library scan for blocked content
- FR-4.3.2: Skip track enforcement
- FR-4.3.3: Playlist content hiding
- FR-4.3.4: Rollback support

**Technical Requirements:**
- TR-4.3.1: Apple Music API integration for library access
- TR-4.3.2: State tracking in `apple_music_enforcement_state` table
- TR-4.3.3: Credit-based usage tracking

**Acceptance Criteria:**
- [ ] Feature parity with Spotify enforcement
- [ ] Enforcement state survives app restarts
- [ ] Credit usage accurately tracked

### Files to Modify

| File | Changes Required |
|------|------------------|
| `backend/src/handlers/spotify_enforcement.rs` | Enable rollback |
| `backend/src/handlers/offense.rs` | Add moderator check |
| `backend/src/services/apple_music_enforcement.rs` | Parity features |
| `backend/src/middleware/` | Role-based access middleware |
| `migrations/` | Role column migration if needed |

---

## Priority 5: Mobile Application Development

### Overview

Mobile applications (iOS and Android) exist as skeleton implementations with some platform-specific integrations but lack core functionality.

### Current State

- iOS: Widget, Siri integration, shortcuts structure present
- Android: Tasker integration present
- Core app functionality status unclear (likely placeholder)

### Requirements

#### 5.1 iOS Application

**Description:** Complete iOS application with full feature parity to web.

**Functional Requirements:**
- FR-5.1.1: User authentication (login, register, 2FA)
- FR-5.1.2: DNP list management (view, add, remove, search)
- FR-5.1.3: Community list browsing and subscription
- FR-5.1.4: Real-time enforcement status
- FR-5.1.5: Push notifications for updates
- FR-5.1.6: Widget showing quick stats
- FR-5.1.7: Siri shortcuts for common actions

**Technical Requirements:**
- TR-5.1.1: SwiftUI for UI implementation
- TR-5.1.2: Keychain for secure credential storage
- TR-5.1.3: Background app refresh for sync
- TR-5.1.4: PKCE OAuth flow
- TR-5.1.5: iOS 15+ minimum deployment target

**Acceptance Criteria:**
- [ ] App Store review guidelines compliance
- [ ] Crash-free rate > 99.5%
- [ ] App launch time < 2 seconds
- [ ] Offline mode for viewing DNP list

#### 5.2 Android Application

**Description:** Complete Android application with full feature parity to web.

**Functional Requirements:**
- FR-5.2.1: User authentication (login, register, 2FA)
- FR-5.2.2: DNP list management (view, add, remove, search)
- FR-5.2.3: Community list browsing and subscription
- FR-5.2.4: Real-time enforcement status
- FR-5.2.5: Push notifications for updates
- FR-5.2.6: Home screen widget
- FR-5.2.7: Tasker integration for automation

**Technical Requirements:**
- TR-5.2.1: Kotlin with Jetpack Compose
- TR-5.2.2: EncryptedSharedPreferences for credentials
- TR-5.2.3: WorkManager for background sync
- TR-5.2.4: PKCE OAuth flow
- TR-5.2.5: Android 8.0+ (API 26) minimum

**Acceptance Criteria:**
- [ ] Play Store policy compliance
- [ ] ANR rate < 0.1%
- [ ] Battery optimization compliant
- [ ] Material Design 3 guidelines followed

#### 5.3 Shared Mobile Infrastructure

**Description:** Common infrastructure for both mobile platforms.

**Functional Requirements:**
- FR-5.3.1: Unified API client with proper error handling
- FR-5.3.2: Consistent push notification handling
- FR-5.3.3: Analytics event tracking
- FR-5.3.4: Feature flags for gradual rollout

**Technical Requirements:**
- TR-5.3.1: Firebase Cloud Messaging for push
- TR-5.3.2: Certificate pinning for API calls
- TR-5.3.3: Crash reporting integration (Crashlytics/Sentry)
- TR-5.3.4: Remote configuration for feature flags

**Acceptance Criteria:**
- [ ] Push notifications delivered within 5 seconds
- [ ] API errors gracefully handled with retry
- [ ] All crashes reported with stack traces

### Files to Modify

| Location | Changes Required |
|----------|------------------|
| `mobile/ios/` | Complete SwiftUI implementation |
| `mobile/android/` | Complete Kotlin/Compose implementation |
| `backend/src/handlers/` | Push notification endpoints |
| `backend/src/services/` | FCM integration service |

---

## Priority 6: End-to-End Testing Suite

### Overview

End-to-end tests are documented as planned but not implemented. This is critical for ensuring feature reliability across the full stack.

### Requirements

#### 6.1 Test Infrastructure

**Description:** Set up E2E testing infrastructure using Playwright or Cypress.

**Functional Requirements:**
- FR-6.1.1: Automated browser testing for frontend flows
- FR-6.1.2: API integration testing
- FR-6.1.3: Mobile app testing (Detox/Appium)
- FR-6.1.4: Visual regression testing

**Technical Requirements:**
- TR-6.1.1: Playwright for web E2E tests
- TR-6.1.2: Docker-compose test environment
- TR-6.1.3: CI/CD integration (GitHub Actions)
- TR-6.1.4: Test data seeding and cleanup

**Acceptance Criteria:**
- [ ] Tests run in < 15 minutes
- [ ] Parallelization for faster execution
- [ ] Screenshots/videos on failure

#### 6.2 Critical User Flows

**Description:** Test all critical user journeys end-to-end.

**Test Scenarios:**
- TS-6.2.1: User registration and verification
- TS-6.2.2: User login with 2FA
- TS-6.2.3: OAuth connection (Spotify, Apple Music)
- TS-6.2.4: Add artist to DNP list via search
- TS-6.2.5: Subscribe to community list
- TS-6.2.6: Trigger and verify enforcement sync
- TS-6.2.7: View analytics dashboard
- TS-6.2.8: Manage account settings

**Acceptance Criteria:**
- [ ] All critical flows have test coverage
- [ ] Tests are deterministic (no flakiness)
- [ ] Tests run on every PR

#### 6.3 Load Testing

**Description:** Performance and load testing for scalability validation.

**Test Scenarios:**
- TS-6.3.1: 1000 concurrent users viewing dashboard
- TS-6.3.2: 100 concurrent sync operations
- TS-6.3.3: 10,000 requests/second to search endpoint
- TS-6.3.4: Database connection pool exhaustion

**Technical Requirements:**
- TR-6.3.1: k6 for load testing
- TR-6.3.2: Grafana dashboards for test results
- TR-6.3.3: Automated regression detection

**Acceptance Criteria:**
- [ ] P99 latency < 500ms under load
- [ ] No errors under expected load
- [ ] Graceful degradation under overload

### Files to Create

| Location | Purpose |
|----------|---------|
| `e2e/` | E2E test directory |
| `e2e/playwright.config.ts` | Playwright configuration |
| `e2e/tests/` | Test files |
| `e2e/fixtures/` | Test data fixtures |
| `.github/workflows/e2e.yml` | CI workflow |

---

## Priority 7: Service Consolidation & Technical Debt

### Overview

The codebase has multiple service implementations that need consolidation and various technical debt items.

### Requirements

#### 7.1 Auth Service Consolidation

**Description:** Consolidate multiple auth service implementations.

**Current Files:**
- `backend/src/services/auth.rs` (main implementation)
- `backend/src/services/auth_simple.rs` (simplified version)
- `backend/src/services/auth_stub.rs` (test stub)

**Requirements:**
- FR-7.1.1: Single production auth service
- FR-7.1.2: Clear test mocking strategy
- FR-7.1.3: Document when to use each variant
- FR-7.1.4: Remove unused code paths

**Acceptance Criteria:**
- [ ] Single source of truth for auth logic
- [ ] Test stubs clearly separated
- [ ] No duplicate business logic

#### 7.2 Handler Test Completion

**Description:** Complete test implementations for handlers.

**Current Issues:**
- `registration_health.rs` line 194: `todo!("Implement mock AppState for testing")`
- Various handlers missing test coverage

**Requirements:**
- FR-7.2.1: Mock AppState implementation for testing
- FR-7.2.2: Test coverage > 80% for all handlers
- FR-7.2.3: Integration tests with testcontainers

**Acceptance Criteria:**
- [ ] All handlers have unit tests
- [ ] Integration tests for critical paths
- [ ] No `todo!()` macros in test code

#### 7.3 Code Cleanup

**Description:** Remove deprecated code and improve code quality.

**Tasks:**
- Remove unused imports and dead code
- Update deprecated dependency usages
- Standardize error handling patterns
- Add missing documentation

**Acceptance Criteria:**
- [ ] `cargo clippy` passes with no warnings
- [ ] All public APIs documented
- [ ] No `#[allow(dead_code)]` without justification

---

## Implementation Roadmap

### Phase 1: Foundation (Weeks 1-4)

| Priority | Feature | Effort | Dependencies |
|----------|---------|--------|--------------|
| P1 | LanceDB Implementation | Large | None |
| P3 | OAuth Error Handling | Small | None |
| P4 | Moderator Role Check | Small | None |
| P7.2 | Handler Test Completion | Medium | None |

### Phase 2: Analytics (Weeks 5-8)

| Priority | Feature | Effort | Dependencies |
|----------|---------|--------|--------------|
| P2.1-2.3 | Core Analytics Metrics | Large | P1 (partial) |
| P2.6 | System Health Monitoring | Medium | None |
| P4.1 | Spotify Rollback | Medium | None |

### Phase 3: Advanced Features (Weeks 9-12)

| Priority | Feature | Effort | Dependencies |
|----------|---------|--------|--------------|
| P2.4 | ML Recommendations | Large | P1, P2.1-2.3 |
| P2.5 | Community List Analytics | Medium | P2.1-2.3 |
| P3.2 | PKCE Implementation | Medium | None |
| P4.3 | Apple Music Parity | Medium | None |

### Phase 4: Mobile & Testing (Weeks 13-20)

| Priority | Feature | Effort | Dependencies |
|----------|---------|--------|--------------|
| P5.1 | iOS Application | X-Large | P3.2 |
| P5.2 | Android Application | X-Large | P3.2 |
| P6 | E2E Testing Suite | Large | All above |

### Phase 5: Polish (Weeks 21-24)

| Priority | Feature | Effort | Dependencies |
|----------|---------|--------|--------------|
| P7.1 | Auth Consolidation | Medium | None |
| P7.3 | Code Cleanup | Medium | All above |
| - | Documentation Updates | Small | All above |

---

## Success Metrics

### Technical Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Test Coverage (Backend) | ~70% | > 85% |
| Test Coverage (Frontend) | ~60% | > 80% |
| E2E Test Coverage | 0% | > 90% critical paths |
| API Latency (P99) | Unknown | < 200ms |
| Error Rate | Unknown | < 0.1% |
| Code Coverage (LanceDB) | 0% | > 90% |

### Product Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Analytics Completion | 40% | 100% |
| Mobile App Stores | Not published | Published |
| Feature Parity (Mobile) | 15% | 100% |
| Vector Search Functionality | 5% | 100% |

### Quality Metrics

| Metric | Target |
|--------|--------|
| Clippy Warnings | 0 |
| TODO Comments (non-test) | 0 |
| Duplicate Code | < 3% |
| Documentation Coverage | > 80% public APIs |

---

## Appendix A: Current TODO Locations

### Backend TODOs

| File | Line | Description |
|------|------|-------------|
| `services/databases/lancedb_client.rs` | Multiple | All vector DB operations |
| `services/analytics.rs` | 394 | `most_common_operations` |
| `services/analytics.rs` | 445 | `avg_duration_ms` |
| `services/analytics.rs` | 484 | `retry_count` |
| `services/analytics.rs` | 541-543 | `total_items_scanned`, `filter_rate` |
| `services/analytics.rs` | 578 | `content_types_blocked` |
| `services/analytics.rs` | 599 | ML recommendations |
| `services/analytics.rs` | 662 | Community list filtering |
| `services/analytics.rs` | 667 | Subscriber satisfaction |
| `services/analytics.rs` | 672 | Detailed impact metrics |
| `services/analytics.rs` | 690-776 | System health tracking |
| `services/auth.rs` | 795 | OAuth error logging |
| `handlers/offense.rs` | 94 | Moderator role check |
| `handlers/spotify_enforcement.rs` | 356 | Enable rollback |
| `handlers/registration_health.rs` | 194 | Mock AppState for tests |

---

## Appendix B: API Endpoint Inventory

### Fully Implemented
- `/api/v1/auth/*` - Authentication endpoints
- `/api/v1/dnp/*` - DNP list management
- `/api/v1/artists/*` - Artist search and management
- `/api/v1/community/*` - Community lists
- `/api/v1/sync/*` - Platform synchronization
- `/api/v1/graph/*` - Network analysis

### Partially Implemented
- `/api/v1/analytics/*` - Analytics (metrics incomplete)
- `/api/v1/enforcement/*` - Enforcement (rollback disabled)

### Not Implemented
- `/api/v1/admin/*` - Admin panel endpoints
- `/api/v1/notifications/*` - Push notification management

---

## Appendix C: Database Migration Status

All 32 migrations applied successfully. No pending schema changes required for existing features.

New migrations needed for:
- User roles (moderator functionality)
- Push notification tokens (mobile)
- Session metadata (device info)

---

*Document Version: 1.0*
*Last Updated: January 2026*
*Author: Claude Code Analysis*
