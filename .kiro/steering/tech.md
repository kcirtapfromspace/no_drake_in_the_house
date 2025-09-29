---
inclusion: always
---

# Technology Stack & Development Guidelines

## Backend (Rust)

### Core Stack
- **Axum**: Web framework - use for all HTTP handlers and routing
- **SQLx**: Database driver with compile-time query verification - ALWAYS use `sqlx::query!` macro
- **Tokio**: Async runtime - use `#[tokio::main]` and async/await throughout

### Required Libraries & Patterns
- **Authentication**: `jsonwebtoken` for JWT, `bcrypt` for passwords, `oauth2` for external services
- **Encryption**: `aes-gcm` + `ring` for token vault - encrypt ALL external API tokens
- **Error Handling**: Use `thiserror` for custom errors, `anyhow` for error context
- **Serialization**: `serde` with `#[derive(Serialize, Deserialize)]` on all API types
- **HTTP Client**: `reqwest` with connection pooling for external APIs

### Database Patterns
- **PostgreSQL**: Primary database - use transactions for multi-table operations
- **Redis**: Session storage and caching - prefix keys by service
- **Migrations**: Use `sqlx migrate` - never modify existing migrations
- **Queries**: Use `sqlx::query!` for compile-time verification, avoid raw SQL strings

## Frontend (Svelte + TypeScript)

### Core Framework
- **Svelte 4**: Component framework - use `<script lang="ts">` for all components
- **TypeScript**: Strict mode enabled - define interfaces for all API responses
- **Rollup**: Build system - configured for development and production

### Styling & State
- **Tailwind CSS**: Use utility classes, avoid custom CSS when possible
- **Svelte Stores**: Use for reactive state - create typed stores in `src/lib/stores/`
- **Component Props**: Always type component props with TypeScript interfaces

## Development Workflow

### Essential Commands
```bash
# Initial setup
make setup                    # One-time environment setup
docker-compose up -d postgres redis  # Start databases

# Backend development
cd backend
cargo run                     # Start API server (localhost:3000)
cargo test                    # Run all tests
cargo fmt && cargo clippy     # Format and lint (required before commits)
sqlx migrate run              # Apply database migrations

# Frontend development
cd frontend
npm run dev                   # Start dev server (localhost:5000)
npm run check                 # TypeScript type checking
npm run build                 # Production build

# Database operations
make migrate                  # Run migrations
sqlx migrate add <name>       # Create new migration
```

### Code Quality Standards
- **Rust**: Run `cargo fmt` and `cargo clippy` before every commit
- **TypeScript**: Enable strict mode, fix all type errors
- **Database**: Use SQLx compile-time query checking - queries must be valid at build time
- **Testing**: Write integration tests for all API endpoints, unit tests for business logic

### Architecture Patterns
- **Services Layer**: Business logic goes in `src/services/` - handlers should be thin
- **Models**: Data structures in `src/models/` - use for database and API types
- **Middleware**: Cross-cutting concerns in `src/middleware/` (auth, CORS, logging)
- **Error Handling**: Use `Result<T, E>` throughout, convert to HTTP responses in handlers

### Environment & Dependencies
- **Rust**: 1.75+ required, use stable toolchain
- **Node.js**: 18+ required for frontend development
- **Docker**: Required for PostgreSQL and Redis in development
- **Environment**: Copy `backend/.env.example` to `backend/.env` and configure

### Security Requirements
- Encrypt all external API tokens using `aes-gcm`
- Use JWT for authentication with proper expiration
- Validate all inputs at API boundaries
- Use parameterized queries (SQLx handles this automatically)