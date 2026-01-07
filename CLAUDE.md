# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

No Drake in the House is a multi-platform music streaming blocklist management system. Users can create personal Do-Not-Play (DNP) lists and subscribe to community-curated blocklists that are enforced across streaming platforms.

## Architecture

- **Backend**: Rust with Axum web framework, SQLx for database access, Tokio async runtime
- **Frontend**: Svelte 4 + TypeScript + Tailwind CSS
- **Databases**: PostgreSQL (primary), Redis (sessions/cache), DuckDB (analytics)
- **Infrastructure**: Docker Compose for dev, Kubernetes + Helm for production

## Development Commands

```bash
# Initial setup
make setup

# Start databases (PostgreSQL + Redis)
make dev

# Run backend (in separate terminal)
cd backend && cargo run

# Run frontend (in separate terminal)
cd frontend && npm run dev

# Run all tests
make test

# Backend tests only
cd backend && cargo test
cd backend && cargo test test_name  # specific test

# Frontend tests only
cd frontend && npm test -- --run

# Linting and formatting
cd backend && cargo fmt && cargo clippy
cd frontend && npm run lint

# Database migrations
cd backend && sqlx migrate run
cd backend && sqlx migrate add <name>  # new migration
```

### Kubernetes/Tilt Development

```bash
# One-time setup
./scripts/dev-setup.sh

# Start Tilt environment (recommended for K8s dev)
tilt up

# Or use Skaffold
make k8s-dev
```

## Service Endpoints

- Frontend: http://localhost:5000
- Backend API: http://localhost:3000
- Health check: http://localhost:3000/health
- Metrics: http://localhost:3000/metrics

## Backend Structure

- `src/main.rs` - Entry point and route configuration
- `src/handlers/` - HTTP request handlers
- `src/services/` - Business logic layer
- `src/models/` - Data structures and database models
- `src/middleware/` - HTTP middleware (auth, rate limiting, etc.)
- `src/error.rs` - Error types and handling
- `migrations/` - SQLx database migrations

## Frontend Structure

- `src/App.svelte` - Root component
- `src/lib/components/` - Reusable UI components
- `src/lib/stores/` - Svelte stores for state management
- `src/lib/utils/` - Utility functions

## API Routes

Authentication: `/api/v1/auth/{register,login,refresh,logout,setup-2fa,verify-2fa}`
DNP Lists: `/api/v1/dnp` (GET/POST), `/api/v1/dnp/{artist_id}` (DELETE)
Artists: `/api/v1/artists/search`
Users: `/api/v1/users/{profile,settings}` (GET/PUT)

## Database

PostgreSQL with tables: `users`, `artists`, `user_artist_blocks`, `user_sessions`, `rate_limits`, `audit_log`

- TOTP secrets are encrypted at rest (AES-GCM)
- All user actions logged to audit_log
- Rate limiting backed by Redis with database fallback

## Testing

- Backend uses testcontainers for isolated PostgreSQL/Redis instances
- Integration tests: `cargo test --test integration_tests`
- Uses factories for test data generation and transactions for test isolation
