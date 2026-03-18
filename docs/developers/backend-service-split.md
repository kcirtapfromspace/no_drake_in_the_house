# Backend Service Split

The backend is moving in two separate stages:

1. Package split for parallel CI and faster ownership boundaries.
2. Deployable service split for true microservices.

## Current State

On `main`, production still deploys a single backend web service plus the frontend. See `render.yaml`.

The backend workspace now separates source into these Rust packages:

- `music-streaming-blocklist-backend`: the API binary and route wiring
- `ndith-core`: shared models, config, validation, error types
- `ndith-db`: database bootstrap, health, recovery
- `ndith-services`: auth, sync, enforcement, provider integrations, token vault, monitoring
- `ndith-news`: news ingestion and offense creation
- `ndith-analytics`: analytics and graph backends

This gives CI package-level parallelism, but it does not create separate deployable services by itself.

## Why GitHub Actions Used To Look Monolithic

Before the workspace split, CI ran a single backend job with `cargo test --all-features`. That produced one backend lane in GitHub Actions even though the code had logical subsystems.

With the workspace split, CI can run package- and feature-level jobs independently:

- `api`
- `api-render`
- `core`
- `db`
- `services`
- `news`
- `analytics`
- `analytics-graph-kuzu`
- `analytics-graph-ladybugdb`
- `api-graph-kuzu`
- `api-graph-ladybugdb`

## Why You Still Do Not Have True Microservices

You only have true microservices when each runtime has its own deployable binary, environment contract, and service definition.

Right now Render still has one backend service:

- `ndith-backend`
- `ndith-frontend`

That means analytics, graph, auth, sync, and news are still linked into one backend deploy path even if the code is split into workspace crates.

## Target Service Split

The realistic next split is:

### 1. API Service

Owns:

- HTTP routes
- auth
- user/session state
- provider connections
- enforcement planning requests

Suggested binary:

- `src/bin/api.rs`

### 2. Analytics Service

Owns:

- DuckDB-backed analytics
- reporting
- revenue and category calculations

Suggested binary:

- `src/bin/analytics.rs`

### 3. Graph Service

Owns:

- graph queries
- Kuzu or LadybugDB adapter
- collaboration/network traversal

Suggested binary:

- `src/bin/graph.rs`

### 4. Ingestion / News Worker

Owns:

- RSS/news crawling
- embedding generation
- offense candidate creation
- scheduled backfills

Suggested binary:

- `src/bin/news-worker.rs`

## What Has To Change Before That Split Is Real

- Move route handlers out of the API binary for analytics and graph calls and replace them with service clients.
- Define per-service storage ownership and remove direct cross-service database assumptions.
- Add separate Render services for each binary.
- Add contract tests between API and analytics/graph/news workers.
- Split environment variables so heavy services do not inherit API-only secrets.
- Decide whether graph stays sync request/response or moves to async jobs.

## Practical Recommendation

Do the migration in this order:

1. Keep the workspace split and matrix CI.
2. Keep production on one API binary with `render-api` for stability.
3. Extract analytics into its own binary and service first.
4. Extract graph next, because its native dependencies are the next major build boundary.
5. Move ingestion/news into worker-style execution last.
