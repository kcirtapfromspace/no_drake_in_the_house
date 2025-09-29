# Development Environment Setup

This document provides comprehensive instructions for setting up and working with the No Drake in the House development environment.

## Prerequisites

Before starting, ensure you have the following installed:

- **Docker** (version 20.10 or later)
- **Docker Compose** (version 2.0 or later)
- **Make** (for running development commands)
- **Git** (for version control)

### Optional (for native development):
- **Rust** (1.75 or later) with Cargo
- **Node.js** (18 or later) with npm
- **PostgreSQL** client tools (for database access)

## Quick Start

1. **Clone the repository:**
   ```bash
   git clone <repository-url>
   cd no-drake-in-the-house
   ```

2. **Run the setup script:**
   ```bash
   make setup
   ```

3. **Start the development environment:**
   ```bash
   make dev
   ```

4. **Start the application services:**
   
   In separate terminals:
   ```bash
   # Backend (Terminal 1)
   cd backend && cargo run
   
   # Frontend (Terminal 2)
   cd frontend && npm install && npm run dev
   ```

## Development Workflow

### Available Make Commands

| Command | Description |
|---------|-------------|
| `make setup` | Initialize development environment |
| `make dev` | Start databases and show next steps |
| `make test` | Run all tests |
| `make clean` | Clean up containers and volumes |
| `make reset-db` | Reset database (destroys all data) |
| `make migrate` | Run database migrations |
| `make logs` | Show logs from all services |
| `make status` | Show status of all services |
| `make k8s-dev` | Start Kubernetes development environment |

### Service URLs

When running, the following services will be available:

- **Backend API:** http://localhost:3000
- **Frontend:** http://localhost:5000
- **PostgreSQL:** localhost:5432 (user: `kiro`, db: `kiro_dev`)
- **Redis:** localhost:6379

### Environment Configuration

Environment variables are managed through `.env` files:

- `backend/.env` - Backend configuration
- `frontend/.env` - Frontend configuration

These files are created from `.env.example` templates during setup.

## Development Modes

### 1. Container-based Development (Recommended)

Run everything in Docker containers with hot reloading:

```bash
# Start all services
docker compose up

# Or start specific services
docker compose up postgres redis
docker compose up backend frontend
```

### 2. Hybrid Development

Run databases in containers, applications natively:

```bash
# Start databases
make dev

# Run backend natively
cd backend && cargo run

# Run frontend natively  
cd frontend && npm run dev
```

### 3. Native Development

Run everything natively (requires local PostgreSQL and Redis):

```bash
# Update .env files to point to local services
# Then run applications directly
cd backend && cargo run
cd frontend && npm run dev
```

## Database Management

### Migrations

Database migrations are managed using SQLx:

```bash
# Run migrations
make migrate

# Create new migration
cd backend && sqlx migrate add <migration_name>

# Reset database (destroys all data)
make reset-db
```

### Database Access

```bash
# Connect to database via Docker
make db-shell

# Or use any PostgreSQL client
psql -h localhost -U kiro -d kiro_dev
```

## Testing

### Running Tests

```bash
# Run all tests
make test

# Run backend tests only
cd backend && cargo test

# Run frontend tests only
cd frontend && npm test
```

### Test Database

Tests use a separate test database that's automatically created and cleaned up.

## Debugging

### Viewing Logs

```bash
# All services
make logs

# Specific service
docker compose logs -f backend
docker compose logs -f frontend
docker compose logs -f postgres
```

### Service Health

```bash
# Check service status
make status

# Manual health checks
curl http://localhost:3000/health
```

### Container Access

```bash
# Backend container shell
make backend-shell

# Frontend container shell  
make frontend-shell

# Database shell
make db-shell

# Redis shell
make redis-shell
```

## Troubleshooting

### Common Issues

1. **Port conflicts:**
   - Ensure ports 3000, 5000, 5432, and 6379 are available
   - Stop conflicting services or modify port mappings in `docker-compose.yml`

2. **Docker daemon not running:**
   ```bash
   # Start Docker Desktop or Docker daemon
   sudo systemctl start docker  # Linux
   ```

3. **Permission issues:**
   ```bash
   # Fix file permissions
   sudo chown -R $USER:$USER .
   ```

4. **Database connection issues:**
   ```bash
   # Reset database
   make reset-db
   
   # Check database status
   make status
   ```

5. **Hot reloading not working:**
   - Ensure file watching is enabled in your IDE
   - Check volume mounts in `docker-compose.yml`
   - Restart the affected service

### Clean Slate Reset

If you encounter persistent issues:

```bash
# Complete cleanup
make clean
docker system prune -a
make setup
```

## Performance Tips

1. **Use Docker volumes for dependencies:**
   - Cargo cache and node_modules are stored in Docker volumes
   - This speeds up rebuilds and container restarts

2. **Selective service startup:**
   ```bash
   # Only start what you need
   docker compose up -d postgres redis
   ```

3. **Native development for faster iteration:**
   - Run applications natively when doing intensive development
   - Use containers for dependencies (databases, etc.)

## IDE Configuration

### VS Code

Recommended extensions:
- Rust Analyzer
- Svelte for VS Code
- Docker
- PostgreSQL

### Environment Variables

Both backend and frontend support environment-based configuration. See `.env.example` files for available options.

## Security Notes

- Default credentials are for development only
- Change all secrets before production deployment
- Environment files contain sensitive information and are gitignored

## Next Steps

After setup, you can:

1. Explore the API at http://localhost:3000/health
2. Access the frontend at http://localhost:5000
3. Review the project structure in the main README
4. Start implementing features according to the task list

For production deployment, see the deployment documentation.