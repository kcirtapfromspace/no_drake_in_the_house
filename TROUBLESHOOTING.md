# Troubleshooting Guide

This guide covers common issues you might encounter while developing No Drake in the House and their solutions.

## 🚀 Quick Fixes

### TL;DR - Common Solutions

```bash
# Reset everything and start fresh
make clean && make setup && make dev

# Fix database issues
make reset-db

# Fix dependency issues
cd backend && cargo clean && cargo build
cd frontend && rm -rf node_modules && npm install

# Fix pre-commit issues
pre-commit clean && pre-commit install --install-hooks
```

## 🐳 Docker & Development Environment

### Docker Compose Issues

#### Problem: `docker compose` command not found
```bash
Error: docker compose command not found
```

**Solution:**
```bash
# Check Docker version (should be 20.10.13+ for compose v2)
docker --version

# If using older Docker, use docker-compose (with hyphen)
docker-compose --version

# Update Docker Desktop or install Docker Compose v2
# macOS: Update Docker Desktop
# Linux: https://docs.docker.com/compose/install/
```

#### Problem: Port already in use
```bash
Error: bind: address already in use
```

**Solution:**
```bash
# Find what's using the port
lsof -i :3000  # Backend port
lsof -i :5000  # Frontend port
lsof -i :5432  # PostgreSQL port
lsof -i :6379  # Redis port

# Kill the process or use different ports
kill -9 <PID>

# Or stop all Docker containers
docker stop $(docker ps -aq)
```

#### Problem: Database connection refused
```bash
Error: Connection refused (os error 61)
```

**Solution:**
```bash
# Check if PostgreSQL is running
docker compose ps postgres

# Restart PostgreSQL
docker compose restart postgres

# Check logs
docker compose logs postgres

# Reset database if corrupted
make reset-db
```

#### Problem: Permission denied on volumes
```bash
Error: Permission denied
```

**Solution:**
```bash
# Fix Docker volume permissions (Linux)
sudo chown -R $USER:$USER .

# Or reset Docker volumes
docker compose down -v
docker volume prune -f
```

## 🦀 Rust Backend Issues

### Compilation Errors

#### Problem: SQLx compile-time verification fails
```bash
Error: error occurred while checking query
```

**Solution:**
```bash
# Set DATABASE_URL for SQLx
export DATABASE_URL="postgres://kiro:kiro_dev_password@localhost:5432/kiro_dev"

# Or use offline mode
export SQLX_OFFLINE=true

# Prepare SQLx queries (when database is available)
cd backend
cargo sqlx prepare

# Run migrations first
sqlx migrate run
```

#### Problem: Cargo build fails with dependency errors
```bash
Error: failed to resolve dependencies
```

**Solution:**
```bash
cd backend

# Clean and rebuild
cargo clean
cargo update
cargo build

# Check Rust version (should be 1.75+)
rustc --version

# Update Rust if needed
rustup update
```

#### Problem: Missing system dependencies
```bash
Error: linker `cc` not found
```

**Solution:**
```bash
# macOS
xcode-select --install

# Ubuntu/Debian
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# Fedora/RHEL
sudo dnf install gcc openssl-devel
```

### Runtime Errors

#### Problem: Database migration errors
```bash
Error: migration 001 failed
```

**Solution:**
```bash
# Check database connection
psql postgres://kiro:kiro_dev_password@localhost:5432/kiro_dev

# Reset database and re-run migrations
make reset-db
cd backend && sqlx migrate run

# Check migration files for syntax errors
ls backend/migrations/
```

#### Problem: JWT secret not set
```bash
Error: JWT_SECRET environment variable not set
```

**Solution:**
```bash
# Check .env file exists
ls backend/.env

# Copy from example if missing
cp backend/.env.example backend/.env

# Generate a secure JWT secret
openssl rand -base64 32
# Add to backend/.env: JWT_SECRET=<generated_secret>
```

#### Problem: Redis connection errors
```bash
Error: Connection refused (redis)
```

**Solution:**
```bash
# Check Redis is running
docker compose ps redis

# Test Redis connection
docker compose exec redis redis-cli ping

# Restart Redis
docker compose restart redis

# Check Redis logs
docker compose logs redis
```

## ⚛️ Frontend Issues

### Node.js & npm Issues

#### Problem: Node version incompatibility
```bash
Error: Unsupported engine
```

**Solution:**
```bash
# Check Node version (should be 18+)
node --version

# Install correct version using nvm
nvm install 18
nvm use 18

# Or update Node.js directly
# macOS: brew upgrade node
# Windows: Download from nodejs.org
```

#### Problem: npm install fails
```bash
Error: EACCES permission denied
```

**Solution:**
```bash
cd frontend

# Clear npm cache
npm cache clean --force

# Remove node_modules and reinstall
rm -rf node_modules package-lock.json
npm install

# Fix npm permissions (if needed)
sudo chown -R $(whoami) ~/.npm
```

#### Problem: Svelte compilation errors
```bash
Error: Unexpected token
```

**Solution:**
```bash
cd frontend

# Check TypeScript configuration
npm run check

# Clear Svelte cache
rm -rf .svelte-kit
npm run build

# Check for syntax errors in .svelte files
# Look for missing closing tags, incorrect TypeScript syntax
```

### Development Server Issues

#### Problem: Frontend can't connect to backend
```bash
Error: Network Error / CORS error
```

**Solution:**
```bash
# Check backend is running
curl http://localhost:3000/health

# Check CORS configuration in backend
# Verify VITE_API_URL in frontend/.env
cat frontend/.env

# Should be: VITE_API_URL=http://localhost:3000
```

#### Problem: Hot reloading not working
```bash
Files change but browser doesn't update
```

**Solution:**
```bash
# Restart dev server
cd frontend
npm run dev

# Check file watchers (Linux)
echo fs.inotify.max_user_watches=524288 | sudo tee -a /etc/sysctl.conf
sudo sysctl -p

# Clear browser cache
# Open DevTools > Application > Storage > Clear storage
```

## ☸️ Kubernetes Issues

### Cluster Setup

#### Problem: kubectl not found
```bash
Error: kubectl command not found
```

**Solution:**
```bash
# macOS
brew install kubectl

# Linux
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl

# Windows
# Download from https://kubernetes.io/docs/tasks/tools/install-kubectl-windows/
```

#### Problem: No Kubernetes cluster available
```bash
Error: connection refused - no cluster
```

**Solution:**
```bash
# Enable Kubernetes in Docker Desktop
# Docker Desktop > Settings > Kubernetes > Enable

# Or install minikube
brew install minikube  # macOS
minikube start

# Or use kind
brew install kind  # macOS
kind create cluster
```

#### Problem: Skaffold not found
```bash
Error: skaffold command not found
```

**Solution:**
```bash
# macOS
brew install skaffold

# Linux
curl -Lo skaffold https://storage.googleapis.com/skaffold/releases/latest/skaffold-linux-amd64
sudo install skaffold /usr/local/bin/

# Windows
# Download from https://github.com/GoogleContainerTools/skaffold/releases
```

#### Problem: Tilt not found
```bash
Error: tilt command not found
```

**Solution:**
```bash
# macOS
brew install tilt-dev/tap/tilt

# Linux
curl -fsSL https://raw.githubusercontent.com/tilt-dev/tilt/master/scripts/install.sh | bash

# Windows
# Download from https://github.com/tilt-dev/tilt/releases
```

### Deployment Issues

#### Problem: Helm chart deployment fails
```bash
Error: failed to install chart
```

**Solution:**
```bash
# Check Helm is installed
helm version

# Add required repositories
helm repo add bitnami https://charts.bitnami.com/bitnami
helm repo update

# Check chart syntax
helm lint ./helm

# Debug deployment
helm install kiro ./helm --dry-run --debug
```

#### Problem: Pods stuck in Pending state
```bash
Status: Pending
```

**Solution:**
```bash
# Check pod events
kubectl describe pod <pod-name> -n kiro-dev

# Check node resources
kubectl top nodes

# Check if images can be pulled
kubectl get events -n kiro-dev --sort-by='.lastTimestamp'

# Common fixes:
# 1. Insufficient resources - reduce resource requests
# 2. Image pull errors - check image names/tags
# 3. PVC issues - check storage class
```

### Tilt Issues

#### Problem: Tilt build failures
```bash
Error: build failed
```

**Solution:**
```bash
# Validate Tilt configuration
make tilt-validate

# Check Tiltfile syntax
tilt validate

# Clean and restart
make tilt-clean

# Check Docker daemon is running
docker ps

# Check Kubernetes cluster
kubectl cluster-info
```

#### Problem: Tilt live updates not working
```bash
Changes not reflected in running containers
```

**Solution:**
```bash
# Check if fast_build is enabled in Tiltfile
# Verify file paths in live_update sync rules

# Manually trigger rebuild
# In Tilt UI: click the refresh button for the resource

# Check container logs in Tilt UI
# Look for build errors or sync failures

# Restart Tilt with clean slate
tilt down
make tilt-clean
```

#### Problem: Tilt port forwards not working
```bash
Cannot access services on localhost
```

**Solution:**
```bash
# Check Tilt UI for port forward status
# Verify services are running in Kubernetes

kubectl get pods -n kiro-dev
kubectl get services -n kiro-dev

# Manually set up port forwards
kubectl port-forward -n kiro-dev svc/kiro-backend 3000:3000
kubectl port-forward -n kiro-dev svc/kiro-frontend 5000:80

# Check for port conflicts
lsof -i :3000
lsof -i :5000
```

## 🧪 Testing Issues

### Backend Tests

#### Problem: Tests fail with database errors
```bash
Error: database connection failed in tests
```

**Solution:**
```bash
# Start test database
docker compose -f backend/docker-compose.test.yml up -d

# Run tests with proper environment
cd backend
DATABASE_URL="postgres://test:test@localhost:5433/test" cargo test

# Or use the test script
./scripts/run_tests.sh --type integration
```

#### Problem: SQLx offline mode issues
```bash
Error: query data not found
```

**Solution:**
```bash
cd backend

# Regenerate SQLx query data
export DATABASE_URL="postgres://kiro:kiro_dev_password@localhost:5432/kiro_dev"
cargo sqlx prepare

# Commit the generated .sqlx/ directory
git add .sqlx/
git commit -m "Update SQLx query data"
```

### Frontend Tests

#### Problem: Vitest configuration errors
```bash
Error: Failed to resolve config
```

**Solution:**
```bash
cd frontend

# Check Vitest configuration
cat vitest.config.ts

# Reinstall dependencies
rm -rf node_modules
npm install

# Run tests with verbose output
npm test -- --reporter=verbose
```

## 🔒 Security & Authentication

### JWT Issues

#### Problem: Token validation fails
```bash
Error: Invalid token signature
```

**Solution:**
```bash
# Check JWT_SECRET is consistent
grep JWT_SECRET backend/.env

# Regenerate secret if needed
openssl rand -base64 32

# Clear browser storage
# DevTools > Application > Storage > Clear storage
```

#### Problem: 2FA setup fails
```bash
Error: Invalid TOTP code
```

**Solution:**
```bash
# Check system time is synchronized
date

# Verify TOTP secret generation
# Use a different authenticator app to test

# Check TOTP window configuration in backend
# Default is 30-second window with 1-step tolerance
```

## 🔍 Performance Issues

### Slow Database Queries

#### Problem: Queries taking too long
```bash
Slow query detected
```

**Solution:**
```bash
# Check database indexes
psql postgres://kiro:kiro_dev_password@localhost:5432/kiro_dev
\d+ users
\d+ artists

# Analyze query performance
EXPLAIN ANALYZE SELECT * FROM users WHERE email = 'test@example.com';

# Add missing indexes (create migration)
cd backend
sqlx migrate add add_missing_indexes
```

### Memory Issues

#### Problem: High memory usage
```bash
Out of memory errors
```

**Solution:**
```bash
# Check Docker memory limits
docker stats

# Increase Docker memory (Docker Desktop > Settings > Resources)

# Check for memory leaks in Rust code
# Use cargo-profiler or valgrind

# Optimize database connection pools
# Reduce max_connections in backend/.env
```

## 🛠️ Development Tools

### Pre-commit Hooks

#### Problem: Pre-commit hooks fail
```bash
Error: hook failed
```

**Solution:**
```bash
# Update pre-commit hooks
pre-commit autoupdate

# Clean and reinstall
pre-commit clean
pre-commit install --install-hooks

# Skip hooks temporarily (not recommended)
git commit --no-verify

# Fix specific issues:
# - Rust: cargo fmt && cargo clippy --fix
# - Frontend: npm run format && npm run lint:fix
```

### IDE Issues

#### Problem: Rust Analyzer not working
```bash
Rust Analyzer: failed to load workspace
```

**Solution:**
```bash
# Restart Rust Analyzer in VS Code
# Cmd/Ctrl + Shift + P > "Rust Analyzer: Restart Server"

# Check Cargo.toml is valid
cd backend
cargo check

# Clear Rust Analyzer cache
rm -rf target/
```

#### Problem: TypeScript errors in Svelte files
```bash
Cannot find module or its type declarations
```

**Solution:**
```bash
cd frontend

# Regenerate TypeScript definitions
npm run check

# Install missing types
npm install --save-dev @types/node

# Check tsconfig.json configuration
cat tsconfig.json
```

## 📞 Getting Help

### Debugging Steps

1. **Check the logs**:
   ```bash
   make logs
   docker compose logs <service>
   kubectl logs <pod> -n kiro-dev
   ```

2. **Verify service health**:
   ```bash
   make status
   curl http://localhost:3000/health
   ```

3. **Reset environment**:
   ```bash
   make clean && make setup
   ```

4. **Check versions**:
   ```bash
   docker --version
   node --version
   rustc --version
   kubectl version
   ```

### Common Environment Variables

```bash
# Backend (.env)
DATABASE_URL=postgres://kiro:kiro_dev_password@localhost:5432/kiro_dev
REDIS_URL=redis://localhost:6379
JWT_SECRET=your-secret-here
RUST_LOG=debug

# Frontend (.env)
VITE_API_URL=http://localhost:3000
```

### Useful Commands

```bash
# Reset everything
make clean && make setup && make dev

# Check service status
make status

# View logs
make logs

# Run tests
make test

# Format code
make format

# Lint code
make lint

# Database shell
make db-shell

# Redis shell
make redis-shell
```

### When to Ask for Help

If you've tried the solutions above and still have issues:

1. **Check existing issues**: Search GitHub issues for similar problems
2. **Gather information**:
   - Error messages (full stack traces)
   - Environment details (OS, Docker version, etc.)
   - Steps to reproduce
   - What you've already tried

3. **Create a detailed issue** with:
   - Clear problem description
   - Environment information
   - Reproduction steps
   - Expected vs actual behavior
   - Relevant logs/screenshots

### Additional Resources

- **Docker Documentation**: https://docs.docker.com/
- **Rust Book**: https://doc.rust-lang.org/book/
- **Svelte Documentation**: https://svelte.dev/docs
- **Kubernetes Documentation**: https://kubernetes.io/docs/
- **SQLx Documentation**: https://docs.rs/sqlx/
- **Axum Documentation**: https://docs.rs/axum/