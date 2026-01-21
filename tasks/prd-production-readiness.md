# PRD: Production Readiness - No Drake in the House

## Introduction

This PRD defines the work required to transition No Drake in the House from a sophisticated prototype to a production-ready MVP. An executive technical assessment identified critical blocking issues: stubbed OAuth/token refresh, in-memory token vault with mock KMS, placeholder enforcement execution, and Redis scalability anti-patterns.

The product solves a real customer pain point: users want to permanently avoid specific artists across their fragmented music streaming ecosystem without manually managing each platform separately. The architecture is sound, but core integrations remain stubbed. This PRD covers Phases 0-2 of the completion roadmap.

**Timeline:** 8-12 weeks (thorough completion)
**Target Scale:** 100-1,000 users at launch
**Platform Scope:** All four platforms (Spotify, Apple Music, YouTube Music, Tidal)

## Goals

- Enable real OAuth authentication flows for all supported streaming providers
- Implement persistent token storage with pluggable KMS (HashiCorp Vault as primary)
- Complete token refresh lifecycle so enforcement doesn't fail silently
- Replace placeholder enforcement with real streaming provider API calls
- Fix Redis scalability issues (KEYS anti-pattern) before launch
- Implement real system telemetry and observability
- Achieve 95%+ OAuth flow completion rate in staging
- Ensure median API latency < 250ms for key endpoints
- Zero placeholder responses in production code paths

## User Stories

### Phase 0: OAuth & SQLx Foundation

#### US-001: Re-enable Backend OAuth Routes
**Description:** As a user, I want to authenticate with my streaming providers so that the app can manage my library on my behalf.

**Acceptance Criteria:**
- [ ] OAuth routes in `backend/src/handlers/` are uncommented and functional
- [ ] SQLx offline cache is regenerated with all OAuth queries
- [ ] `cargo sqlx prepare` runs successfully
- [ ] OAuth endpoints return proper HTTP status codes (not 501 Not Implemented)
- [ ] Typecheck passes (`cargo check`)
- [ ] Unit tests pass for OAuth handlers

#### US-002: Implement Google OAuth End-to-End Flow
**Description:** As a user, I want to sign in with Google so that I can use my existing account.

**Acceptance Criteria:**
- [ ] `/api/v1/auth/oauth/google/authorize` returns valid authorization URL
- [ ] `/api/v1/auth/oauth/google/callback` exchanges code for tokens
- [ ] User account is created or linked on successful OAuth
- [ ] Tokens are encrypted before storage (using existing encryption service)
- [ ] Error states return appropriate error codes (see `error.rs` OAuth mappings)
- [ ] Integration test with mocked Google API passes
- [ ] Typecheck passes

#### US-003: Implement Apple OAuth End-to-End Flow
**Description:** As a user, I want to sign in with Apple so that I can use my Apple ID.

**Acceptance Criteria:**
- [ ] `/api/v1/auth/oauth/apple/authorize` returns valid authorization URL with proper scopes
- [ ] `/api/v1/auth/oauth/apple/callback` handles Apple's POST callback format
- [ ] Apple's identity token (JWT) is validated correctly
- [ ] User account is created or linked on successful OAuth
- [ ] Handles Apple's "hide my email" relay addresses
- [ ] Integration test with mocked Apple API passes
- [ ] Typecheck passes

#### US-004: Implement GitHub OAuth End-to-End Flow
**Description:** As a user, I want to sign in with GitHub so that I can use my developer account.

**Acceptance Criteria:**
- [ ] `/api/v1/auth/oauth/github/authorize` returns valid authorization URL
- [ ] `/api/v1/auth/oauth/github/callback` exchanges code for tokens
- [ ] User email is fetched from GitHub API (handling private email case)
- [ ] User account is created or linked on successful OAuth
- [ ] Integration test with mocked GitHub API passes
- [ ] Typecheck passes

#### US-005: Implement Spotify OAuth for Provider Connection
**Description:** As a user, I want to connect my Spotify account so that the app can enforce my DNP list on Spotify.

**Acceptance Criteria:**
- [ ] `/api/v1/connections/spotify/authorize` returns authorization URL with required scopes
- [ ] Required scopes include: `user-library-read`, `user-library-modify`, `playlist-read-private`, `playlist-modify-private`, `user-follow-read`, `user-follow-modify`
- [ ] `/api/v1/connections/spotify/callback` stores encrypted tokens in database
- [ ] Connection record created with `status: Active`
- [ ] User can see connected Spotify account in profile
- [ ] Integration test passes
- [ ] Typecheck passes

### Phase 1: Token Vault & Lifecycle

#### US-006: Implement Pluggable KMS Provider Interface
**Description:** As a developer, I need a KMS abstraction so that we can use different key management solutions.

**Acceptance Criteria:**
- [ ] Create `KmsProvider` trait in `backend/src/services/kms/mod.rs`
- [ ] Trait defines: `generate_data_key()`, `decrypt_data_key()`, `rotate_key()`
- [ ] Implement `MockKmsProvider` for development/testing (existing code)
- [ ] Implement `VaultKmsProvider` for HashiCorp Vault
- [ ] KMS provider is configurable via environment variable `KMS_PROVIDER`
- [ ] Existing `TokenVaultService` uses the trait instead of concrete `MockKmsService`
- [ ] All existing token vault tests pass
- [ ] Typecheck passes

#### US-007: Implement HashiCorp Vault KMS Provider
**Description:** As an operator, I need real KMS integration so that encryption keys are securely managed in production.

**Acceptance Criteria:**
- [ ] `VaultKmsProvider` connects to Vault server via `VAULT_ADDR` env var
- [ ] Authenticates using AppRole or Token auth method
- [ ] Uses Vault Transit secrets engine for envelope encryption
- [ ] `generate_data_key()` creates and returns encrypted DEK
- [ ] `decrypt_data_key()` decrypts DEK using Vault
- [ ] Handles Vault unavailability gracefully (returns error, doesn't panic)
- [ ] Connection pooling/reuse for Vault client
- [ ] Integration test with Vault dev server passes
- [ ] Typecheck passes

#### US-008: Migrate Token Vault to Persistent Storage
**Description:** As a user, I need my provider connections to persist across service restarts so that I don't have to reconnect constantly.

**Acceptance Criteria:**
- [ ] Token vault reads/writes to PostgreSQL `connections` table (not in-memory DashMap)
- [ ] Encrypted tokens stored in `access_token_encrypted` and `refresh_token_encrypted` columns
- [ ] Data key ID stored with connection for decryption lookup
- [ ] Connection retrieval uses database queries with proper indexing
- [ ] In-memory cache (optional) with TTL for performance
- [ ] Service restart does not lose connection data
- [ ] Migration script if schema changes needed
- [ ] All token vault tests updated and passing
- [ ] Typecheck passes

#### US-009: Implement Spotify Token Refresh
**Description:** As a user, I need my Spotify tokens to refresh automatically so that enforcement continues working.

**Acceptance Criteria:**
- [ ] `TokenVaultService::refresh_token()` calls Spotify's `/api/token` endpoint
- [ ] Uses stored refresh token to obtain new access token
- [ ] Updates `access_token_encrypted`, `refresh_token_encrypted`, `expires_at` in database
- [ ] Handles refresh token rotation (Spotify may return new refresh token)
- [ ] Returns `TokenRefreshResult` with `success: true` and new expiry
- [ ] On failure, updates connection status to `NeedsReauth`
- [ ] Logs refresh attempts with correlation ID
- [ ] Integration test with mocked Spotify API passes
- [ ] Typecheck passes

#### US-010: Implement Apple Music Token Refresh
**Description:** As a user, I need my Apple Music tokens to refresh automatically.

**Acceptance Criteria:**
- [ ] Implement Apple Music token refresh flow (developer token + user token)
- [ ] Handle Apple's 6-month user token expiry
- [ ] Update connection record on successful refresh
- [ ] On failure, update connection status to `NeedsReauth`
- [ ] Integration test passes
- [ ] Typecheck passes

#### US-011: Implement Proactive Token Refresh Background Job
**Description:** As a system, I need to refresh tokens before they expire so that users don't experience service interruptions.

**Acceptance Criteria:**
- [ ] Background job runs every hour (configurable)
- [ ] Queries connections expiring within 24 hours
- [ ] Refreshes tokens in batches with rate limiting
- [ ] Uses `TokenRefreshJobHandler` in job queue
- [ ] Failed refreshes are retried with exponential backoff
- [ ] After max retries, connection marked `NeedsReauth` and user notified
- [ ] Job progress tracked in job queue system
- [ ] Metrics emitted: `tokens_refreshed_total`, `token_refresh_failures_total`
- [ ] Typecheck passes

#### US-012: Implement Connection Health Check Endpoint
**Description:** As a user, I want to see if my provider connections are healthy so that I know if enforcement will work.

**Acceptance Criteria:**
- [ ] `GET /api/v1/connections` returns list with health status for each
- [ ] Health status includes: `active`, `expiring_soon`, `needs_reauth`, `error`
- [ ] `expiring_soon` = expires within 24 hours
- [ ] Response includes `last_used_at` and `expires_at` timestamps
- [ ] Frontend displays connection health badges
- [ ] Typecheck passes
- [ ] Verify in browser (frontend)

### Phase 1B: Enforcement Completion

#### US-013: Implement Real Spotify Library Scan
**Description:** As a user, I want the app to scan my actual Spotify library so that enforcement plans are based on real data.

**Acceptance Criteria:**
- [ ] `SpotifyService::scan_library()` fetches real liked songs via Spotify API
- [ ] Fetches saved albums, followed artists, and user playlists
- [ ] Handles pagination (Spotify returns max 50 items per request)
- [ ] Respects Spotify rate limits (429 responses trigger backoff)
- [ ] Returns structured `LibraryScanResult` with counts and items
- [ ] Scan progress is trackable via job queue
- [ ] Integration test with mocked Spotify API passes
- [ ] Typecheck passes

#### US-014: Implement Real Spotify Enforcement Execution
**Description:** As a user, I want enforcement to actually remove blocked artists from my Spotify library.

**Acceptance Criteria:**
- [ ] `execute_action_batch()` in `enforcement_job_handler.rs` calls real Spotify APIs
- [ ] `remove_liked_song` action calls `DELETE /v1/me/tracks`
- [ ] `unfollow_artist` action calls `DELETE /v1/me/following`
- [ ] `remove_playlist_track` action calls `DELETE /v1/playlists/{id}/tracks`
- [ ] Each action records `before_state` and `after_state` for rollback
- [ ] Failed actions don't stop batch; errors collected and reported
- [ ] Rate limiting respected with automatic backoff
- [ ] Batch results stored in `action_items` table
- [ ] Integration test passes
- [ ] Typecheck passes

#### US-015: Implement Enforcement Rollback
**Description:** As a user, I want to undo an enforcement batch if I made a mistake.

**Acceptance Criteria:**
- [ ] `POST /api/v1/enforcement/batches/{id}/rollback` triggers rollback job
- [ ] Rollback reads `before_state` from `action_items` table
- [ ] Re-adds removed liked songs via `PUT /v1/me/tracks`
- [ ] Re-follows unfollowed artists via `PUT /v1/me/following`
- [ ] Re-adds removed playlist tracks (best effort - playlist may have changed)
- [ ] Rollback batch created with reference to original batch
- [ ] Progress tracked via job queue
- [ ] Returns actual `actions_rolled_back` count (not hardcoded 0)
- [ ] Integration test passes
- [ ] Typecheck passes

#### US-016: Implement Apple Music Enforcement
**Description:** As a user, I want enforcement to work on my Apple Music library.

**Acceptance Criteria:**
- [ ] Library scan fetches Apple Music library via MusicKit API
- [ ] Enforcement removes songs from library via API
- [ ] Handles Apple Music's different API structure
- [ ] Rate limiting respected
- [ ] Rollback supported
- [ ] Integration test passes
- [ ] Typecheck passes

#### US-017: Implement YouTube Music Enforcement
**Description:** As a user, I want enforcement to work on my YouTube Music library.

**Acceptance Criteria:**
- [ ] OAuth flow for YouTube/Google account connection
- [ ] Library scan via YouTube Data API
- [ ] Enforcement removes liked videos/songs
- [ ] Handles YouTube Music's playlist structure
- [ ] Rate limiting respected
- [ ] Integration test passes
- [ ] Typecheck passes

#### US-018: Implement Tidal Enforcement
**Description:** As a user, I want enforcement to work on my Tidal library.

**Acceptance Criteria:**
- [ ] OAuth flow for Tidal account connection
- [ ] Library scan via Tidal API
- [ ] Enforcement removes favorites and playlist items
- [ ] Rate limiting respected
- [ ] Integration test passes
- [ ] Typecheck passes

### Phase 1C: Job Queue Scalability

#### US-019: Replace Redis KEYS with SCAN
**Description:** As a system, I need efficient Redis queries so that the job queue doesn't block at scale.

**Acceptance Criteria:**
- [ ] `get_user_jobs()` in `job_queue.rs` uses `SCAN` instead of `KEYS`
- [ ] `cleanup_jobs()` uses `SCAN` with cursor iteration
- [ ] Batch size configurable (default 100)
- [ ] No use of `KEYS *` pattern anywhere in codebase
- [ ] Performance test: 10,000 jobs doesn't block Redis
- [ ] Existing job queue tests pass
- [ ] Typecheck passes

#### US-020: Add Job Index by User ID
**Description:** As a system, I need efficient job lookups by user so that the dashboard loads quickly.

**Acceptance Criteria:**
- [ ] Add Redis sorted set `user:{user_id}:jobs` indexing job IDs
- [ ] Jobs added to user index on enqueue
- [ ] Jobs removed from user index on cleanup
- [ ] `get_user_jobs()` queries user index instead of scanning all jobs
- [ ] Index maintained atomically with job operations
- [ ] Typecheck passes

#### US-021: Implement Data Key Cache with LRU Eviction
**Description:** As a system, I need bounded memory usage for the data key cache.

**Acceptance Criteria:**
- [ ] Replace unbounded `DashMap` in `TokenVaultService` with LRU cache
- [ ] Default capacity: 10,000 keys
- [ ] Capacity configurable via `DATA_KEY_CACHE_SIZE` env var
- [ ] Cache miss triggers KMS fetch and caches result
- [ ] Metrics: `data_key_cache_hits`, `data_key_cache_misses`
- [ ] Existing token vault tests pass
- [ ] Typecheck passes

### Phase 2: Observability & Analytics

#### US-022: Implement Real System Metrics
**Description:** As an operator, I need real system metrics so that I can monitor service health.

**Acceptance Criteria:**
- [ ] `/metrics` returns real CPU usage (via `sysinfo` crate)
- [ ] Returns real memory usage (heap and RSS)
- [ ] Returns disk usage for data directory
- [ ] Returns active database connection count
- [ ] Returns Redis connection pool stats
- [ ] Returns job queue depth by job type
- [ ] Metrics in Prometheus format
- [ ] No placeholder values in metrics output
- [ ] Typecheck passes

#### US-023: Implement Request Latency Metrics
**Description:** As an operator, I need latency metrics so that I can identify slow endpoints.

**Acceptance Criteria:**
- [ ] Middleware records latency for all HTTP requests
- [ ] Histogram buckets: 10ms, 50ms, 100ms, 250ms, 500ms, 1000ms, 5000ms
- [ ] Labels: `method`, `path`, `status_code`
- [ ] P50, P90, P99 calculable from histogram
- [ ] Metrics available at `/metrics`
- [ ] Typecheck passes

#### US-024: Implement Enforcement Analytics
**Description:** As a user, I want to see statistics about my enforcement history.

**Acceptance Criteria:**
- [ ] `GET /api/v1/analytics/enforcement` returns real data (not placeholders)
- [ ] Returns: total batches, total actions, success rate, actions by type
- [ ] Time-series data for last 30 days
- [ ] Filtered by provider
- [ ] Proper authorization (users see only their data)
- [ ] Response time < 500ms
- [ ] Typecheck passes

#### US-025: Implement User Activity Dashboard Data
**Description:** As a user, I want to see my activity summary on the dashboard.

**Acceptance Criteria:**
- [ ] `GET /api/v1/analytics/summary` returns user activity summary
- [ ] Includes: DNP list size, connected providers, recent enforcement count
- [ ] Includes: last enforcement date, next scheduled scan
- [ ] Cached for 5 minutes to reduce database load
- [ ] Typecheck passes
- [ ] Verify in browser (frontend)

### Phase 2B: Error Recovery & Resilience

#### US-026: Implement Circuit Breaker for Provider APIs
**Description:** As a system, I need circuit breakers so that provider outages don't cascade.

**Acceptance Criteria:**
- [ ] Circuit breaker wraps all provider API calls
- [ ] Opens after 5 consecutive failures within 1 minute
- [ ] Half-open state allows 1 test request every 30 seconds
- [ ] Closes after 3 successful requests in half-open
- [ ] Metrics: `circuit_breaker_state`, `circuit_breaker_trips_total`
- [ ] When open, returns `ProviderUnavailable` error immediately
- [ ] Typecheck passes

#### US-027: Implement Graceful Degradation for Token Refresh Failures
**Description:** As a user, I need clear feedback when my provider connection needs attention.

**Acceptance Criteria:**
- [ ] After token refresh failure, connection status = `NeedsReauth`
- [ ] User receives in-app notification (not just silent failure)
- [ ] Dashboard shows warning banner for unhealthy connections
- [ ] Enforcement jobs skip providers with `NeedsReauth` status (with explanation)
- [ ] Email notification sent after 24 hours of `NeedsReauth` (if email enabled)
- [ ] Typecheck passes
- [ ] Verify in browser (frontend)

## Functional Requirements

### OAuth & Authentication
- FR-1: The system must support OAuth 2.0 flows for Google, Apple, GitHub (identity) and Spotify, Apple Music, YouTube, Tidal (provider connections)
- FR-2: All OAuth state parameters must be validated to prevent CSRF attacks
- FR-3: OAuth tokens must be encrypted at rest using envelope encryption
- FR-4: Failed OAuth flows must return appropriate error codes per `error.rs` mappings

### Token Management
- FR-5: The system must store tokens in PostgreSQL, not in-memory
- FR-6: The system must support pluggable KMS with HashiCorp Vault as primary implementation
- FR-7: Access tokens must be refreshed automatically before expiration
- FR-8: Token refresh failures must update connection status and notify users
- FR-9: Data encryption keys must be cached with LRU eviction (max 10,000 keys)

### Enforcement
- FR-10: Enforcement must call real streaming provider APIs (no simulated delays)
- FR-11: Each enforcement action must record before/after state for rollback
- FR-12: Enforcement rollback must restore previous state via provider APIs
- FR-13: Enforcement must respect provider rate limits with automatic backoff
- FR-14: Failed actions must not halt batch execution; errors collected and reported

### Scalability
- FR-15: Redis operations must use SCAN, not KEYS, for iteration
- FR-16: Job lookups by user must use indexed queries (O(1) not O(n))
- FR-17: System must handle 1,000 concurrent users without degradation

### Observability
- FR-18: Metrics endpoint must return real system values (CPU, memory, disk)
- FR-19: All HTTP requests must record latency histograms
- FR-20: Analytics endpoints must return real data, not placeholders

## Non-Goals

- UI/UX redesign or theme changes
- New streaming provider integrations beyond the four specified
- Mobile app development
- Real-time collaborative features
- Machine learning for artist recommendations
- Social features (sharing lists publicly)
- Payment/subscription system integration
- GDPR data export automation (manual process acceptable for MVP)

## Technical Considerations

### Dependencies
- HashiCorp Vault server (or dev mode for staging)
- PostgreSQL 14+ with `uuid-ossp` extension
- Redis 6+ with persistence enabled
- Provider API credentials for all four platforms

### Database Changes
- Add `data_key_id` column to `connections` table if not present
- Add index on `connections(user_id, provider)`
- Add index on `action_items(batch_id)`

### Configuration
New environment variables required:
```
KMS_PROVIDER=vault|mock
VAULT_ADDR=https://vault.example.com
VAULT_TOKEN=<token> or VAULT_ROLE_ID + VAULT_SECRET_ID
DATA_KEY_CACHE_SIZE=10000
TOKEN_REFRESH_INTERVAL_HOURS=1
CIRCUIT_BREAKER_THRESHOLD=5
```

### Testing Strategy
- Unit tests for all new services with mocked dependencies
- Integration tests using WireMock for provider APIs
- Integration tests using Vault dev server for KMS
- Load tests for job queue with 10,000 jobs
- End-to-end OAuth flow tests in staging environment

### Rollout Strategy
1. Deploy KMS integration to staging with mock provider
2. Switch to Vault in staging, validate encryption/decryption
3. Enable real provider OAuth in staging
4. Load test with synthetic users
5. Gradual rollout to production with feature flags

## Success Metrics

- OAuth flow completion rate > 95% in staging
- Token refresh success rate > 99%
- Zero placeholder responses in production paths
- Median API latency < 250ms for enforcement endpoints
- P99 API latency < 2000ms
- Job queue can process 100 jobs/minute sustained
- Circuit breaker prevents cascade failures (tested via chaos engineering)
- User-reported "enforcement stopped working" tickets < 5% of users

## Open Questions

1. Should we implement webhook notifications for enforcement completion, or is polling acceptable?
2. What is the SLA expectation for token refresh? (Current design: refresh within 1 hour of expiry)
3. Should rollback be time-limited? (e.g., only available for 7 days after enforcement)
4. How should we handle provider API deprecations? (YouTube frequently changes APIs)
5. Should we add Spotify's "don't play this artist" feature detection to avoid duplicate blocking?

## Appendix: File Locations

Key files to modify:

**OAuth:**
- `backend/src/handlers/oauth_*.rs` - OAuth route handlers
- `backend/src/services/oauth*.rs` - OAuth service implementations
- `backend/src/services/auth.rs` - Authentication service

**Token Vault:**
- `backend/src/services/token_vault.rs` - Main token vault (lines 81-84, 229-248)
- `backend/src/services/kms/` - New KMS provider directory

**Enforcement:**
- `backend/src/services/enforcement_job_handler.rs` - Job handler (lines 232-263, 304-372)
- `backend/src/services/spotify_enforcement.rs` - Spotify enforcement service

**Job Queue:**
- `backend/src/services/job_queue.rs` - Job queue (lines 301-302 KEYS pattern)

**Metrics:**
- `backend/src/services/monitoring.rs` - System metrics
- `backend/src/middleware/metrics.rs` - Request metrics
