# Music Streaming Blocklist Manager

A multi-platform music streaming blocklist management system that provides centralized control for users to avoid specific artists across streaming services.

## Architecture

- **Frontend**: Svelte + TypeScript + Tailwind CSS
- **Backend**: Rust + Axum + SQLx
- **Databases**: PostgreSQL (operational data) + DuckDB (analytics)
- **Infrastructure**: Docker Compose for development

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Node.js 18+ (for frontend)
- Rust 1.75+ (for backend)

### Development Setup

1. **Start the databases:**
   ```bash
   docker-compose up -d postgres redis
   ```

2. **Set up the backend:**
   ```bash
   cd backend
   cp .env.example .env
   # Edit .env with your configuration
   
   # Install SQLx CLI for migrations
   cargo install sqlx-cli --no-default-features --features postgres
   
   # Run migrations
   sqlx migrate run
   
   # Start the backend
   cargo run
   ```

3. **Set up the frontend:**
   ```bash
   cd frontend
   npm install
   npm run dev
   ```

4. **Access the application:**
   - Frontend: http://localhost:5000
   - Backend API: http://localhost:3000
   - Health check: http://localhost:3000/health

## API Endpoints

### Core Endpoints

- `GET /health` - Health check
- `POST /api/v1/users` - Create user
- `GET /api/v1/artists/search` - Search artists
- `GET /api/v1/dnp` - Get user's DNP list
- `POST /api/v1/dnp` - Add artist to DNP list

## Database Schema

The system uses PostgreSQL for operational data with the following core tables:

- `users` - User accounts and settings
- `artists` - Artist catalog with external ID mapping
- `connections` - Streaming service connections
- `user_artist_blocks` - Personal DNP lists
- `community_lists` - Community-curated blocklists
- `action_batches` - Enforcement operation tracking
- `action_items` - Individual enforcement actions
- `audit_log` - SOC2 compliance logging

## Development

### Running Tests

```bash
# Backend tests
cd backend
cargo test

# Frontend tests (when implemented)
cd frontend
npm test
```

### Database Migrations

```bash
cd backend
sqlx migrate add <migration_name>
sqlx migrate run
```

### Code Style

- Rust: `cargo fmt` and `cargo clippy`
- Frontend: Prettier and ESLint (configured in package.json)

## Requirements Addressed

This foundation addresses the following requirements:

- **2.1**: User account management and service connections
- **2.2**: Secure token storage infrastructure (encrypted fields ready)
- **7.1**: Security-first database design with audit logging

## Next Steps

With the foundation in place, the next tasks will implement:

1. Entity resolution service with external API integration
2. Spotify adapter with OAuth flow
3. Web extension for content filtering
4. Community list management
5. Enforcement planning and execution