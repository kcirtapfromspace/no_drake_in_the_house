# Technology Stack

## Backend (Rust)

### Core Framework
- **Axum**: Web framework for HTTP API
- **SQLx**: Async PostgreSQL driver with compile-time query checking
- **Tokio**: Async runtime

### Key Libraries
- **Authentication**: `jsonwebtoken`, `bcrypt`, `oauth2`, `totp-lite`
- **Encryption**: `aes-gcm`, `ring` for token vault security
- **Database**: `sqlx` with PostgreSQL features
- **HTTP Client**: `reqwest` for external API calls
- **Serialization**: `serde` with JSON support
- **Error Handling**: `thiserror`, `anyhow`

### Database Stack
- **PostgreSQL**: Primary operational database
- **Redis**: Session storage and caching
- **DuckDB**: Analytics (embedded)

## Frontend (Svelte + TypeScript)

### Core Framework
- **Svelte 4**: Component framework
- **TypeScript**: Type safety
- **Rollup**: Build system

### Styling & UI
- **Tailwind CSS**: Utility-first CSS framework
- **PostCSS**: CSS processing

### State Management
- **Svelte Stores**: Reactive state management
- Custom stores for auth, DNP lists, community lists, enforcement

## Development Tools

### Common Commands

```bash
# Setup
make setup              # Initialize development environment
make dev               # Start development (shows commands to run)

# Backend
cd backend
cargo run              # Start backend server (localhost:3000)
cargo test             # Run tests
cargo fmt              # Format code
cargo clippy           # Lint code
sqlx migrate run       # Run database migrations
sqlx migrate add <name> # Create new migration

# Frontend  
cd frontend
npm run dev            # Start dev server (localhost:5000)
npm run build          # Production build
npm run check          # Type checking

# Database
make migrate           # Run migrations
make reset-db          # Reset database (destroys data)

# Infrastructure
docker-compose up -d postgres redis  # Start databases
docker-compose down -v                # Stop and clean
```

### Code Quality
- **Rust**: Use `cargo fmt` and `cargo clippy` before commits
- **TypeScript**: Configured with strict type checking
- **Database**: SQLx compile-time query verification

### Environment Setup
- Copy `backend/.env.example` to `backend/.env`
- Ensure Docker is running for databases
- Node.js 18+ and Rust 1.75+ required