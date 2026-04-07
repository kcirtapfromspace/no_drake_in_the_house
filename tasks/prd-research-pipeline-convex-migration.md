# PRD: Research Pipeline — PostgreSQL to Convex Migration

## Introduction

The artist research and evidence pipeline discovers offense records for artists in users' music libraries. It currently writes results (articles, entities, offense classifications, evidence) to PostgreSQL via the Rust `ndith-news` backend. The app has migrated to Convex as its primary data store, but the research pipeline still targets PostgreSQL — meaning research results never reach the Convex tables that power the Taste Grade, Worst Offenders, and Playlist Sanitizer features.

This PRD covers adding a thin Convex HTTP writer to the Rust backend (keeping the existing pipeline intact), fixing the bridge issues (auth, ID mapping) between Convex and Rust, dropping PostgreSQL as the data store for research data, and making the async research flow work end-to-end.

## Goals

- Research results (articles, entities, classifications) written to Convex via HTTP mutations
- Evidence finder in Convex can successfully trigger research on the Rust backend
- Offense pipeline processes research results into taste grades and offender data
- PostgreSQL tables for research data dropped (Convex is sole store)
- External research sources (Wikipedia, Brave, NewsAPI) active based on available API keys
- Async flow: Convex triggers research, Rust queues and processes, results flow back to Convex

## User Stories

### US-001: Convex HTTP client in Rust backend
**Description:** As the Rust backend, I need an HTTP client that can call Convex mutations so that research results are persisted in Convex instead of PostgreSQL.

**Acceptance Criteria:**
- [ ] New module `convex_client.rs` in `ndith-news` crate
- [ ] Reads `CONVEX_URL` env var for the deployment URL
- [ ] Calls Convex HTTP API (`POST /api/mutation`) with function path and arguments
- [ ] Handles errors (network, 4xx, 5xx) with retries (up to 3 attempts)
- [ ] No auth required for Convex mutations called via internal actions
- [ ] Typecheck/clippy passes

### US-002: Write articles, entities, classifications to Convex
**Description:** As the news pipeline orchestrator, I need to write processed articles to Convex so that the offense pipeline can promote them to artist offenses.

**Acceptance Criteria:**
- [ ] `repository.rs` gains a `ConvexRepository` implementation alongside the existing `PostgresRepository`
- [ ] `ConvexRepository` calls `batchIngestArticles` mutation for efficiency (single call per batch)
- [ ] Falls back to individual `ingestArticle` + `ingestEntities` + `ingestClassification` if batch fails
- [ ] Article deduplication by URL still works (Convex mutation handles upsert by legacyKey)
- [ ] Entity and classification records include artist IDs resolved from Convex artist names
- [ ] All fields expected by `newsIngestion.ts` mutations are provided
- [ ] PostgreSQL write path removed — only Convex writes remain
- [ ] Typecheck/clippy passes

### US-003: Write offense records and evidence to Convex
**Description:** As the offense creator, I need to write auto-detected offenses and linked evidence to Convex so they appear in the user's taste grade.

**Acceptance Criteria:**
- [ ] New Convex mutation `createOffenseFromResearch` in `newsIngestion.ts` — accepts artistId, category, severity, title, description, confidence, sourceArticleUrl
- [ ] Deduplicates: same artist + category within 30 days = update, not create
- [ ] New Convex mutation `linkOffenseEvidence` — accepts offenseId, sourceUrl, title, excerpt, credibilityScore
- [ ] `offense_creator.rs` calls these Convex mutations instead of PostgreSQL inserts
- [ ] After creating offenses, triggers `rebuildOffendingArtistIndex` via Convex scheduler
- [ ] Typecheck passes (Rust + Convex)

### US-004: Write research quality scores to Convex
**Description:** As the auto-researcher, I need to record research quality metadata in Convex so the pipeline can prioritize under-researched artists.

**Acceptance Criteria:**
- [ ] New field on Convex `artists` table: `researchQualityScore` (number), `sourcesSearched` (array of strings), `researchIterations` (number)
- [ ] New Convex mutation `updateArtistResearchQuality` — updates these fields on the artist record
- [ ] `autoresearch.rs` calls this mutation after completing research for an artist
- [ ] `_filterArtistsNeedingInvestigation` in `evidenceFinder.ts` uses `researchQualityScore` for prioritization (lowest score first)
- [ ] Typecheck passes (Rust + Convex)

### US-005: Fix service auth bypass for research endpoint
**Description:** As the Convex evidence finder, I need to call the Rust backend's research endpoint without JWT auth so that research can be triggered.

**Acceptance Criteria:**
- [ ] Research trigger endpoint (`POST /api/v1/news/research/artists/{id}/trigger`) skips JWT auth middleware
- [ ] Uses a simple shared secret header instead: `X-Service-Key: {NDITH_SERVICE_KEY}`
- [ ] `NDITH_SERVICE_KEY` env var added to both Convex and Rust backend (render.yaml)
- [ ] Evidence finder sends the service key header instead of JWT Bearer token
- [ ] Endpoint still validates the key (rejects requests without it)
- [ ] Typecheck passes (Rust + Convex)

### US-006: Fix artist ID mapping (Convex ID → artist name)
**Description:** As the evidence finder, I need to pass artist names (not Convex document IDs) to the Rust backend so the research endpoint can look up and research the artist.

**Acceptance Criteria:**
- [ ] Evidence finder resolves artist name from Convex `artists` table before calling backend
- [ ] New internal query `_getArtistName` in `evidenceFinder.ts` — returns `{ name, canonicalName }` for an artist ID
- [ ] Research trigger endpoint accepts artist name in request body (not UUID in URL path)
- [ ] Endpoint signature changes to `POST /api/v1/news/research/trigger` with body `{ "artist_name": "Drake" }`
- [ ] Response includes `offenses_detected` count so evidence finder can track progress
- [ ] Typecheck passes (Rust + Convex)

### US-007: Make research trigger actually queue research
**Description:** As the research trigger handler, I need to actually queue and execute research instead of returning a stub response.

**Acceptance Criteria:**
- [ ] Handler calls `ArtistResearcher::research_artist()` with the resolved artist name
- [ ] Research runs asynchronously (spawned as a background task via `tokio::spawn`)
- [ ] Handler returns immediately with `202 Accepted` and a `research_id`
- [ ] Research results written to Convex via the ConvexRepository (US-002)
- [ ] Response body includes `{ "success": true, "research_id": "...", "status": "queued" }`
- [ ] Evidence finder treats 202 as success and moves to next artist
- [ ] Typecheck/clippy passes

### US-008: Add CONVEX_URL to Rust backend configuration
**Description:** As the Rust backend, I need the Convex deployment URL to make HTTP calls to Convex mutations.

**Acceptance Criteria:**
- [ ] `CONVEX_URL` env var added to `render.yaml` for ndith-news service
- [ ] `CONVEX_URL` env var added to `render.yaml` for ndith-backend service
- [ ] ConvexClient reads from this env var on initialization
- [ ] Startup fails with clear error if `CONVEX_URL` is not set
- [ ] Value matches the dev deployment: `https://scrupulous-emu-861.convex.cloud`

### US-009: Drop PostgreSQL research tables
**Description:** As the system, I need to remove the PostgreSQL research tables since Convex is the sole data store.

**Acceptance Criteria:**
- [ ] Migration removes: `news_articles`, `news_article_entities`, `news_offense_classifications`, `news_fetch_log`, `artist_research_quality`
- [ ] Migration does NOT remove non-research tables (users, connections, etc. if still used)
- [ ] All PostgreSQL read queries in news handlers replaced with Convex queries (or handlers removed)
- [ ] No remaining `sqlx::query` calls referencing dropped tables
- [ ] Backend compiles and starts without these tables

### US-010: End-to-end research pipeline test
**Description:** As a developer, I need to verify the full pipeline works from trigger to taste grade.

**Acceptance Criteria:**
- [ ] Trigger research for a known artist (e.g., "R. Kelly") via evidence finder
- [ ] Research runs on Rust backend, fetches from Wikipedia/Brave/NewsAPI
- [ ] Articles, entities, classifications written to Convex
- [ ] `promoteClassifications` cron promotes high-confidence classifications to offenses
- [ ] `rebuildOffendingArtistIndex` updates the offender index
- [ ] `recomputeUserOffenseSummary` computes the user's taste grade
- [ ] Taste Grade section on sync dashboard shows a non-zero score
- [ ] Worst Offenders section shows the flagged artist

## Functional Requirements

- FR-1: Rust backend must have a `ConvexClient` that calls Convex mutations via HTTP POST to `/api/mutation`
- FR-2: All research pipeline writes (articles, entities, classifications, offenses, evidence, quality scores) must target Convex, not PostgreSQL
- FR-3: Research trigger endpoint must accept artist name in request body and skip JWT auth (use shared service key)
- FR-4: Research must run asynchronously — handler returns 202, research completes in background
- FR-5: Evidence finder must resolve artist name before calling backend and handle 202 responses
- FR-6: New Convex mutations must exist for offense creation, evidence linking, and research quality tracking
- FR-7: PostgreSQL research tables must be dropped via migration
- FR-8: `CONVEX_URL` must be configured in render.yaml for all backend services
- FR-9: Existing Convex ingestion mutations (`ingestArticle`, `ingestEntities`, `ingestClassification`, `batchIngestArticles`) must be used where they exist
- FR-10: Research sources active based on available API keys — no hard failures if a key is missing

## Non-Goals

- No changes to the research algorithm itself (Wikipedia/Brave/NewsAPI pipeline stays as-is)
- No new external data sources added in this migration
- No frontend UI changes (taste grade, offenders, playlist sanitizer UIs remain as-is)
- No migration of non-research PostgreSQL tables (users, connections, etc.)
- No real-time streaming of research progress to the frontend
- No changes to the cron schedule (daily investigation at 03:00 UTC stays)
- No LLM/AI integration for classification (existing keyword-based classifier stays)

## Technical Considerations

### Architecture
```
Convex evidenceFinder          Rust ndith-news              Convex newsIngestion
────────────────────           ───────────────              ────────────────────
investigateLibraryArtists()
  │
  ├── resolve artist name
  │
  ├── POST /research/trigger ──► trigger_handler()
  │   (X-Service-Key header)     │
  │                              ├── tokio::spawn research
  │   ◄── 202 Accepted ─────────┤
  │                              │
  │   (moves to next artist)     ├── Wikipedia fetch
  │                              ├── Brave Search
  │                              ├── NewsAPI fetch
  │                              │
  │                              ├── process articles
  │                              │   ├── EntityExtractor
  │                              │   └── OffenseClassifier
  │                              │
  │                              └── ConvexClient.call() ──► batchIngestArticles()
  │                                                         ingestClassification()
  │                                                         createOffenseFromResearch()
  │                                                         updateArtistResearchQuality()
  │
  ├── (later, via cron)
  │   promoteClassifications()
  │   rebuildOffendingArtistIndex()
  │   recomputeUserOffenseSummary()
  │
  └── Taste Grade updated
```

### Dependencies
- Convex HTTP API for mutations: `POST https://{deployment}.convex.cloud/api/mutation`
- `reqwest` crate already in Rust backend's Cargo.toml
- Free-tier API budgets: Wikipedia (unlimited), Brave Search (2000/month), NewsAPI (100/day)

### Execution Order
1. US-001 (Convex HTTP client) — foundational, everything depends on it
2. US-005 + US-006 (auth bypass + ID mapping) — unblocks the bridge
3. US-008 (CONVEX_URL config) — needed for client to connect
4. US-003 + US-004 (new Convex mutations) — data sinks
5. US-002 (repository migration) — the main write path change
6. US-007 (research trigger) — makes research actually run
7. US-009 (drop PG tables) — cleanup after verification
8. US-010 (E2E test) — final validation

## Success Metrics

- Evidence finder completes investigation of 300+ artists without errors
- At least 1 artist in the user's library gets flagged with offense records
- Taste Grade shows a computed score (not "? 0.00")
- Worst Offenders section populates with flagged artists
- Research completes within the daily cron window (< 2 hours for full library)
- No PostgreSQL research tables remain after migration

## Open Questions

- What Brave Search and NewsAPI keys are currently configured? Are they active?
- Should the `batchIngestArticles` mutation handle offense creation inline, or keep it as a separate step via the promote cron?
- Is there a Convex HTTP API rate limit we need to respect for high-volume writes during research?
- Should we add a research status webhook from Rust → Convex so the evidence finder knows when research completes (vs. just fire-and-forget)?
