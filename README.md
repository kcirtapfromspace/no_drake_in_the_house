# No Drake in the House

A multi-platform music streaming blocklist management system that provides centralized control for users to avoid specific artists across streaming services.

## 🎵 Overview

No Drake in the House allows users to:
- Create and manage personal Do-Not-Play (DNP) lists
- Subscribe to community-curated blocklists
- Automatically enforce blocklists across multiple streaming platforms
- Track and audit all enforcement actions

## 🏗️ Architecture

- **Frontend**: Svelte 4 + TypeScript + Tailwind CSS
- **Backend**: Rust + Axum + SQLx + Tokio
- **Databases**: PostgreSQL (operational) + Redis (sessions/cache) + DuckDB (analytics)
- **Infrastructure**: Docker Compose (dev) + Kubernetes (production)
- **CI/CD**: GitHub Actions + Helm charts

## 🚀 Quick Start

### Prerequisites

- **Docker & Docker Compose** (for databases)
- **Node.js 18+** (for frontend development)
- **Rust 1.75+** (for backend development)
- **Make** (for development commands)

### 🎯 Recommended: Ultra-Fast Tilt + Minikube Development

```bash
# Complete setup (one-time, ~5 minutes)
chmod +x scripts/dev-setup.sh && ./scripts/dev-setup.sh

# Start development environment
tilt up
```

**⚡ Optimized for Speed:**
- **Sub-30s Rust rebuilds** (vs 2-5 minutes normally)
- **Instant frontend updates** with live reloading
- **Docker layer caching** with cargo-chef optimization
- **Live updates** - code changes deploy in seconds
- **Full Kubernetes** - production-like environment

**🔧 Manual Setup (if preferred):**
```bash
make tilt-setup          # Setup minikube + Tilt
make tilt-warm-cache     # Pre-build Docker layers (saves 5+ min later)
tilt up                  # Start development
```

### 🐳 Alternative: Local Development

```bash
make setup && make dev
```

Then in separate terminals:
```bash
# Backend (Rust API server)
cd backend && cargo run

# Frontend (Svelte dev server)  
cd frontend && npm run dev
```

**Access Points:**
- 🌐 **Frontend**: http://localhost:5000
- 🔧 **Backend API**: http://localhost:3000
- ❤️ **Health Check**: http://localhost:3000/health
- 📊 **Metrics**: http://localhost:3000/metrics

## 📖 Development Guide

### Available Commands

```bash
# Setup & Development
make setup              # Initialize development environment
make dev               # Start development (shows next steps)
make test              # Run all tests
make clean             # Clean up containers and volumes
make reset-db          # Reset database (destroys data!)

# Backend Development
cd backend
cargo run              # Start API server (localhost:3000)
cargo test             # Run Rust tests
cargo fmt              # Format code
cargo clippy           # Lint code
sqlx migrate run       # Run database migrations
sqlx migrate add <name> # Create new migration

# Frontend Development
cd frontend
npm run dev            # Start dev server (localhost:5000)
npm run build          # Production build
npm run check          # TypeScript checking
npm test               # Run tests

# Kubernetes Development
make k8s-dev           # Start with Skaffold (requires local K8s)
```

### Project Structure

```
├── backend/           # Rust API server
│   ├── src/
│   │   ├── main.rs           # Entry point & routes
│   │   ├── models/           # Data structures
│   │   ├── services/         # Business logic
│   │   └── middleware/       # HTTP middleware
│   ├── tests/               # Integration & unit tests
│   └── migrations/          # Database schema
├── frontend/          # Svelte web application
│   ├── src/
│   │   ├── lib/components/  # UI components
│   │   ├── lib/stores/      # State management
│   │   └── lib/utils/       # Utilities
├── extension/         # Browser extension
├── mobile/           # Mobile integrations
├── k8s/              # Kubernetes manifests
├── helm/             # Helm charts
└── docker-compose.yml # Development infrastructure
```

## 🔧 API Documentation

### Authentication Endpoints

```http
POST /api/v1/auth/register
POST /api/v1/auth/login
POST /api/v1/auth/refresh
POST /api/v1/auth/logout
POST /api/v1/auth/setup-2fa
POST /api/v1/auth/verify-2fa
```

### DNP List Management

```http
GET    /api/v1/dnp              # Get user's DNP list
POST   /api/v1/dnp              # Add artist to DNP list
DELETE /api/v1/dnp/{artist_id}  # Remove from DNP list
GET    /api/v1/artists/search   # Search artists
```

### User Profile

```http
GET    /api/v1/users/profile    # Get user profile
PUT    /api/v1/users/profile    # Update profile
GET    /api/v1/users/settings   # Get user settings
PUT    /api/v1/users/settings   # Update settings
```

### System Endpoints

```http
GET /health                     # Health check
GET /metrics                    # Prometheus metrics
```

## 🗄️ Database Schema

### Core Tables

- **`users`** - User accounts with encrypted 2FA secrets
- **`artists`** - Artist catalog with external platform IDs
- **`user_artist_blocks`** - Personal DNP list entries
- **`user_sessions`** - JWT refresh token storage
- **`rate_limits`** - Rate limiting state
- **`audit_log`** - Security and compliance logging

### Key Features

- **Encryption**: TOTP secrets encrypted at rest
- **Audit Trail**: All user actions logged
- **Rate Limiting**: Redis-backed with database fallback
- **Referential Integrity**: Foreign key constraints
- **Performance**: Optimized indexes for common queries

## 🧪 Testing

### Running Tests

```bash
# All tests
make test

# Backend only
cd backend && cargo test

# Frontend only  
cd frontend && npm test

# Integration tests with database
cd backend && cargo test --test integration_tests

# Specific test
cd backend && cargo test test_user_registration
```

### Test Structure

- **Unit Tests**: Service layer business logic
- **Integration Tests**: Full API workflows with test database
- **Component Tests**: Frontend component behavior
- **End-to-End**: Complete user workflows (planned)

### Test Data

Tests use:
- **Testcontainers**: Isolated PostgreSQL instances
- **Factories**: Consistent test data generation
- **Mocks**: External API simulation
- **Transactions**: Test isolation

## 🔒 Security

### Authentication

- **JWT Tokens**: 24-hour access tokens with refresh capability
- **2FA Support**: TOTP with QR code setup
- **Password Security**: bcrypt with 12+ rounds
- **Rate Limiting**: Brute force protection

### Data Protection

- **Encryption**: Sensitive data encrypted at rest (AES-GCM)
- **Audit Logging**: All security events tracked
- **Input Validation**: Comprehensive request validation
- **CORS**: Properly configured for development/production

## 📊 Monitoring & Observability

### Health Checks

```bash
curl http://localhost:3000/health
```

Returns detailed status of all dependencies:
- Database connectivity
- Redis availability  
- Service health metrics

### Metrics

Prometheus metrics available at `/metrics`:
- HTTP request counts and latency
- Database connection pool stats
- Authentication attempt tracking
- DNP operation metrics

### Logging

Structured JSON logging with:
- Correlation IDs for request tracing
- Error context and stack traces
- Security event logging
- Performance metrics

## 🚀 Deployment

### Local Development

```bash
# Docker Compose
docker-compose up -d postgres redis
cd backend && cargo run
cd frontend && npm run dev
```

### Kubernetes Development

```bash
# Requires local Kubernetes (Docker Desktop, minikube, etc.)
make k8s-dev
```

### Production Deployment

```bash
# Build and deploy with Helm
helm upgrade --install kiro ./helm \
  --values ./helm/values-production.yaml \
  --namespace kiro-production
```

## 🛠️ Development Resources

### Documentation

- **[API Documentation](./docs/api/)** - Complete API reference with OpenAPI spec
- **[Troubleshooting Guide](./TROUBLESHOOTING.md)** - Common issues and solutions
- **[Performance Profiling](./docs/PERFORMANCE_PROFILING.md)** - Performance testing and optimization

### Development Tools

```bash
# Setup pre-commit hooks (recommended)
make setup-pre-commit

# Run performance tests
make perf-test

# Code quality
make lint          # Run linters
make format        # Format code
```

## 🤝 Contributing

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Commit** your changes (`git commit -m 'Add amazing feature'`)
4. **Push** to the branch (`git push origin feature/amazing-feature`)
5. **Open** a Pull Request

### Code Standards

- **Rust**: Use `cargo fmt` and `cargo clippy`
- **TypeScript**: Configured with strict type checking
- **Commits**: Conventional commit format preferred
- **Tests**: Required for new features

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.