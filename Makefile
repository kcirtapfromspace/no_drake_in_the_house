.PHONY: setup dev test clean reset-db k8s-dev help

# Default target
help:
	@echo "Available commands:"
	@echo ""
	@echo "🚀 Development:"
	@echo "  setup              - Initialize development environment"
	@echo "  dev                - Start development environment"
	@echo "  clean              - Clean up containers and volumes"
	@echo "  reset-db           - Reset database (destroys all data)"
	@echo "  migrate            - Run database migrations"
	@echo ""
	@echo "☸️  Kubernetes:"
	@echo "  k8s-validate       - Validate Kubernetes development setup"
	@echo "  k8s-dev            - Start Kubernetes development environment (Skaffold)"
	@echo "  k8s-build          - Build Docker images for Kubernetes"
	@echo "  k8s-deploy         - Deploy to Kubernetes using Helm"
	@echo "  k8s-clean          - Clean up Kubernetes resources"
	@echo "  k8s-port-forward   - Set up port forwarding for local access"
	@echo "  k8s-status         - Check Kubernetes deployment status"
	@echo ""
	@echo "🎯 Tilt + Minikube (Recommended K8s Development):"
	@echo "  tilt-setup         - Setup minikube and Tilt environment"
	@echo "  tilt-warm-cache    - Pre-build Docker layers for faster builds"
	@echo "  tilt-validate      - Validate Tilt configuration"
	@echo "  tilt-test-build    - Test Docker builds before using Tilt"
	@echo "  tilt-up            - Start Tilt development environment"
	@echo "  tilt-down          - Stop Tilt and clean up resources"
	@echo "  tilt-clean         - Clean up and restart Tilt"
	@echo ""
	@echo "🧪 Testing:"
	@echo "  test               - Run all tests (backend + frontend)"
	@echo "  test-backend       - Run all backend tests"
	@echo "  test-backend-unit  - Run backend unit tests only"
	@echo "  test-backend-integration - Run backend integration tests"
	@echo "  test-backend-performance - Run backend performance tests"
	@echo "  test-backend-coverage    - Generate backend test coverage"
	@echo "  test-frontend      - Run frontend tests"
	@echo "  test-frontend-watch - Run frontend tests in watch mode"
	@echo "  test-frontend-ui   - Open frontend test UI"
	@echo "  test-setup         - Set up test environment"
	@echo "  test-cleanup       - Clean up test environment"
	@echo "  test-watch         - Run tests continuously"
	@echo ""
	@echo "🔍 Monitoring:"
	@echo "  logs               - Show logs from all services"
	@echo "  status             - Show status of all services"
	@echo ""
	@echo "🛠️  Development Tools:"
	@echo "  setup-pre-commit   - Install and configure pre-commit hooks"
	@echo "  lint               - Run linting on all code"
	@echo "  format             - Format all code"
	@echo ""
	@echo "⚡ Performance:"
	@echo "  perf-test          - Run comprehensive performance tests"
	@echo "  perf-backend       - Run backend benchmarks only"
	@echo "  perf-frontend      - Run frontend performance tests only"
	@echo "  perf-load          - Run load tests on API endpoints"
	@echo ""
	@echo "🐳 Docker Build Optimization:"
	@echo "  warm-cache         - Pre-build Docker layers for faster builds"
	@echo "  test-build-perf    - Test Docker build performance improvements"
	@echo "  clean-cache        - Clean Docker build cache"
	@echo "  refresh-cache      - Clean and rebuild Docker cache"
	@echo "  cache-status       - Show Docker cache usage"

# Development setup
setup:
	@echo "🚀 Setting up development environment..."
	@echo "Checking Docker..."
	@docker --version || (echo "❌ Docker not found. Please install Docker first." && exit 1)
	@echo "Checking Docker Compose..."
	@docker compose version || (echo "❌ Docker Compose not found. Please install Docker Compose first." && exit 1)
	@echo "Pulling Docker images..."
	docker compose pull postgres redis
	@echo "Creating environment files..."
	@if [ ! -f backend/.env ]; then cp backend/.env.example backend/.env; echo "Created backend/.env"; fi
	@if [ ! -f frontend/.env ]; then cp frontend/.env.example frontend/.env; echo "Created frontend/.env"; fi
	@echo "Building development containers..."
	docker compose build
	@echo ""
	@echo "✅ Setup complete! Run 'make dev' to start development servers"
	@echo ""
	@echo "Development workflow:"
	@echo "  1. Run 'make dev' to start databases"
	@echo "  2. In separate terminals:"
	@echo "     - Backend:  cd backend && cargo run"
	@echo "     - Frontend: cd frontend && npm run dev"
	@echo ""
	@echo "Or run 'docker compose up' to start all services in containers"

# Start development environment
dev:
	@echo "🔧 Starting development environment..."
	@echo "Starting PostgreSQL and Redis..."
	docker compose up -d postgres redis
	@echo "Waiting for services to be ready..."
	@for i in $$(seq 1 30); do \
		if docker compose exec postgres pg_isready -U kiro -d kiro_dev >/dev/null 2>&1; then break; fi; \
		if [ $$i -eq 30 ]; then echo "❌ PostgreSQL failed to start" && exit 1; fi; \
		sleep 1; \
	done
	@for i in $$(seq 1 30); do \
		if docker compose exec redis redis-cli ping 2>/dev/null | grep -q PONG; then break; fi; \
		if [ $$i -eq 30 ]; then echo "❌ Redis failed to start" && exit 1; fi; \
		sleep 1; \
	done
	@echo ""
	@echo "✅ Services ready!"
	@echo ""
	@echo "🎯 Next steps:"
	@echo "  Backend:  cd backend && cargo run"
	@echo "  Frontend: cd frontend && npm run dev"
	@echo ""
	@echo "📊 Service URLs:"
	@echo "  Backend API:    http://localhost:3000"
	@echo "  Frontend:       http://localhost:5000"
	@echo "  PostgreSQL:     localhost:5432 (user: kiro, db: kiro_dev)"
	@echo "  Redis:          localhost:6379"
	@echo ""
	@echo "🔍 Useful commands:"
	@echo "  make logs       - View service logs"
	@echo "  make status     - Check service status"
	@echo "  make reset-db   - Reset database"

# Run tests
test:
	@echo "🧪 Running all tests..."
	@$(MAKE) test-backend
	@$(MAKE) test-frontend

# Backend testing commands
test-backend:
	@echo "🦀 Running backend tests..."
	@if [ -d "backend" ]; then \
		cd backend && ./scripts/run_tests.sh --type all; \
	else \
		echo "⚠️  Backend directory not found, skipping backend tests"; \
	fi

test-backend-unit:
	@echo "🧪 Running backend unit tests..."
	@if [ -d "backend" ]; then \
		cd backend && ./scripts/run_tests.sh --type unit --verbose; \
	else \
		echo "⚠️  Backend directory not found"; \
	fi

test-backend-integration:
	@echo "🔗 Running backend integration tests..."
	@if [ -d "backend" ]; then \
		cd backend && ./scripts/run_tests.sh --type integration --verbose; \
	else \
		echo "⚠️  Backend directory not found"; \
	fi

test-backend-performance:
	@echo "⚡ Running backend performance tests..."
	@if [ -d "backend" ]; then \
		cd backend && ./scripts/run_tests.sh --type performance --verbose; \
	else \
		echo "⚠️  Backend directory not found"; \
	fi

test-backend-coverage:
	@echo "📊 Generating backend test coverage..."
	@if [ -d "backend" ]; then \
		cd backend && ./scripts/run_tests.sh --coverage; \
	else \
		echo "⚠️  Backend directory not found"; \
	fi

# Frontend testing commands
test-frontend:
	@echo "⚛️  Running frontend tests..."
	@if [ -d "frontend" ] && [ -f "frontend/package.json" ]; then \
		cd frontend && npm test -- --run; \
	else \
		echo "⚠️  Frontend not set up yet, skipping frontend tests"; \
	fi

test-frontend-watch:
	@echo "👀 Running frontend tests in watch mode..."
	@if [ -d "frontend" ] && [ -f "frontend/package.json" ]; then \
		cd frontend && npm run test:watch; \
	else \
		echo "⚠️  Frontend not set up yet"; \
	fi

test-frontend-ui:
	@echo "🎨 Opening frontend test UI..."
	@if [ -d "frontend" ] && [ -f "frontend/package.json" ]; then \
		cd frontend && npm run test:ui; \
	else \
		echo "⚠️  Frontend not set up yet"; \
	fi

# Test environment management
test-setup:
	@echo "🔧 Setting up test environment..."
	@echo "Starting test databases..."
	docker compose -f backend/docker-compose.test.yml up -d
	@echo "Waiting for test services to be ready..."
	@sleep 10
	@echo "✅ Test environment ready"

test-cleanup:
	@echo "🧹 Cleaning up test environment..."
	docker compose -f backend/docker-compose.test.yml down -v
	@echo "✅ Test cleanup complete"

# Continuous testing
test-watch:
	@echo "👀 Starting continuous testing..."
	@echo "This will run tests whenever files change..."
	@$(MAKE) test-backend-unit &
	@$(MAKE) test-frontend-watch

# Clean up
clean:
	@echo "🧹 Cleaning up development environment..."
	docker compose down -v --remove-orphans
	docker system prune -f
	@echo "✅ Cleanup complete"

# Reset database
reset-db:
	@echo "⚠️  This will destroy all data in the database!"
	@read -p "Are you sure? (y/N): " confirm && [ "$$confirm" = "y" ] || exit 1
	@echo "🔄 Resetting database..."
	docker compose stop postgres
	docker compose rm -f postgres
	docker volume rm $$(docker volume ls -q | grep postgres) 2>/dev/null || true
	docker compose up -d postgres
	@echo "Waiting for PostgreSQL to be ready..."
	@for i in $$(seq 1 30); do \
		if docker compose exec postgres pg_isready -U kiro -d kiro_dev >/dev/null 2>&1; then break; fi; \
		if [ $$i -eq 30 ]; then echo "❌ PostgreSQL failed to start" && exit 1; fi; \
		sleep 1; \
	done
	@echo "✅ Database reset complete"

# Validate Kubernetes setup
k8s-validate:
	@echo "🔍 Validating Kubernetes development setup..."
	@./scripts/validate-k8s-setup.sh

# Kubernetes development
k8s-dev:
	@echo "☸️  Starting Kubernetes development environment..."
	@command -v skaffold >/dev/null 2>&1 || (echo "❌ Skaffold not found. Please install Skaffold first." && exit 1)
	@command -v kubectl >/dev/null 2>&1 || (echo "❌ kubectl not found. Please install kubectl first." && exit 1)
	@command -v helm >/dev/null 2>&1 || (echo "❌ Helm not found. Please install Helm first." && exit 1)
	@echo "Adding Bitnami Helm repository..."
	helm repo add bitnami https://charts.bitnami.com/bitnami || true
	helm repo update
	@echo "Starting Skaffold development..."
	skaffold dev --port-forward

# Build Docker images for Kubernetes
k8s-build:
	@echo "🐳 Building Docker images for Kubernetes..."
	docker build -t kiro/backend:latest -f backend/Dockerfile.dev backend/
	docker build -t kiro/frontend:latest -f frontend/Dockerfile.dev frontend/

# Deploy to Kubernetes using Helm
k8s-deploy:
	@echo "🚀 Deploying to Kubernetes using Helm..."
	@command -v helm >/dev/null 2>&1 || (echo "❌ Helm not found. Please install Helm first." && exit 1)
	helm repo add bitnami https://charts.bitnami.com/bitnami || true
	helm repo update
	helm upgrade --install kiro ./helm \
		--values ./helm/values-dev.yaml \
		--namespace kiro-dev \
		--create-namespace \
		--wait

# Clean up Kubernetes resources
k8s-clean:
	@echo "🧹 Cleaning up Kubernetes resources..."
	helm uninstall kiro --namespace kiro-dev || true
	kubectl delete namespace kiro-dev || true

# Port forward services for local access
k8s-port-forward:
	@echo "🔌 Setting up port forwarding..."
	@echo "Backend will be available at http://localhost:3000"
	@echo "Frontend will be available at http://localhost:5000"
	@echo "PostgreSQL will be available at localhost:5432"
	@echo "Redis will be available at localhost:6379"
	@echo "Press Ctrl+C to stop port forwarding"
	kubectl port-forward -n kiro-dev service/kiro-backend 3000:3000 &
	kubectl port-forward -n kiro-dev service/kiro-frontend 5000:80 &
	kubectl port-forward -n kiro-dev service/kiro-postgresql 5432:5432 &
	kubectl port-forward -n kiro-dev service/kiro-redis-master 6379:6379 &
	wait

# Check Kubernetes deployment status
k8s-status:
	@echo "📊 Checking Kubernetes deployment status..."
	@echo ""
	@echo "Pods:"
	kubectl get pods -n kiro-dev
	@echo ""
	@echo "Services:"
	kubectl get services -n kiro-dev
	@echo ""
	@echo "Ingress:"
	kubectl get ingress -n kiro-dev

# Run database migrations
migrate:
	@echo "🗄️  Running database migrations..."
	@if [ -d "backend" ]; then \
		cd backend && sqlx migrate run; \
	else \
		echo "❌ Backend directory not found"; \
		exit 1; \
	fi

# Show logs
logs:
	@echo "📋 Showing logs from all services..."
	docker compose logs -f

# Show service status
status:
	@echo "📊 Service Status:"
	@docker compose ps
	@echo ""
	@echo "🔍 Health Checks:"
	@echo -n "PostgreSQL: "
	@docker compose exec postgres pg_isready -U kiro -d kiro_dev >/dev/null 2>&1 && echo "✅ Healthy" || echo "❌ Unhealthy"
	@echo -n "Redis: "
	@docker compose exec redis redis-cli ping 2>/dev/null | grep -q PONG && echo "✅ Healthy" || echo "❌ Unhealthy"

# Development helpers
backend-shell:
	@echo "🐚 Opening backend container shell..."
	docker compose exec backend bash

frontend-shell:
	@echo "🐚 Opening frontend container shell..."
	docker compose exec frontend sh

db-shell:
	@echo "🗄️  Opening database shell..."
	docker compose exec postgres psql -U kiro -d kiro_dev

redis-shell:
	@echo "📦 Opening Redis shell..."
	docker compose exec redis redis-cli

# Development tools
setup-pre-commit:
	@echo "🔧 Setting up pre-commit hooks..."
	@command -v pre-commit >/dev/null 2>&1 || (echo "Installing pre-commit..." && pip install pre-commit)
	@echo "Installing git hooks..."
	pre-commit install
	@echo "Installing commit-msg hook..."
	pre-commit install --hook-type commit-msg || true
	@echo "Creating secrets baseline..."
	@command -v detect-secrets >/dev/null 2>&1 || pip install detect-secrets
	@[ -f .secrets.baseline ] || detect-secrets scan --baseline .secrets.baseline
	@echo "Testing pre-commit setup..."
	@pre-commit run --all-files || echo "⚠️  Some checks failed - this is normal on first run"
	@echo "✅ Pre-commit hooks installed successfully!"
	@echo ""
	@echo "📋 Pre-commit will now run automatically on git commit"
	@echo "🔧 Manual commands:"
	@echo "  pre-commit run --all-files  # Run on all files"
	@echo "  pre-commit autoupdate       # Update hook versions"

lint:
	@echo "🔍 Running linting on all code..."
	@echo "Rust linting..."
	@if [ -d "backend" ]; then cd backend && cargo clippy --all-targets --all-features -- -D warnings; fi
	@echo "Frontend linting..."
	@if [ -d "frontend" ] && [ -f "frontend/package.json" ]; then cd frontend && npm run lint; fi
	@echo "✅ Linting complete"

format:
	@echo "🎨 Formatting all code..."
	@echo "Rust formatting..."
	@if [ -d "backend" ]; then cd backend && cargo fmt --all; fi
	@echo "Frontend formatting..."
	@if [ -d "frontend" ] && [ -f "frontend/package.json" ]; then cd frontend && npm run format; fi
	@echo "✅ Formatting complete"

# Performance testing
perf-test:
	@echo "⚡ Running comprehensive performance tests..."
	@chmod +x scripts/performance-test.sh
	@./scripts/performance-test.sh

perf-backend:
	@echo "🦀 Running backend benchmarks..."
	@if [ -d "backend" ]; then \
		cd backend && cargo bench; \
	else \
		echo "❌ Backend directory not found"; \
	fi

perf-frontend:
	@echo "⚛️  Running frontend performance tests..."
	@command -v lighthouse >/dev/null 2>&1 || (echo "Installing lighthouse..." && npm install -g lighthouse)
	@if curl -s http://localhost:5000 >/dev/null 2>&1; then \
		lighthouse http://localhost:5000 --output html --output-path ./lighthouse-report.html; \
		echo "📊 Lighthouse report generated: lighthouse-report.html"; \
	else \
		echo "❌ Frontend not running at http://localhost:5000"; \
		echo "   Run 'make dev' first, then start frontend with 'cd frontend && npm run dev'"; \
	fi

perf-load:
	@echo "🔥 Running load tests..."
	@command -v wrk >/dev/null 2>&1 || (echo "❌ wrk not found. Install with: brew install wrk (macOS) or apt install wrk (Ubuntu)" && exit 1)
	@if curl -s http://localhost:3000/health >/dev/null 2>&1; then \
		echo "Testing health endpoint..."; \
		wrk -t4 -c20 -d30s --latency http://localhost:3000/health; \
	else \
		echo "❌ Backend not running at http://localhost:3000"; \
		echo "   Run 'make dev' first, then start backend with 'cd backend && cargo run'"; \
	fi

# Tilt development commands
tilt-setup:
	@echo "🎯 Setting up Minikube + Tilt environment..."
	@chmod +x scripts/setup-minikube-tilt.sh
	@./scripts/setup-minikube-tilt.sh

tilt-warm-cache:
	@echo "🔥 Warming up Docker build cache..."
	@chmod +x scripts/warm-cache.sh
	@./scripts/warm-cache.sh

tilt-warm-cache:
	@echo "🔥 Warming Docker cache for faster builds..."
	@chmod +x scripts/warm-cache.sh
	@./scripts/warm-cache.sh

tilt-validate:
	@echo "🔍 Validating Tilt configuration..."
	@chmod +x scripts/validate-tilt.sh
	@./scripts/validate-tilt.sh

tilt-test-build:
	@echo "🐳 Testing Docker builds..."
	@chmod +x scripts/test-docker-build.sh
	@./scripts/test-docker-build.sh

tilt-up:
	@echo "🎯 Starting Tilt development environment..."
	@command -v tilt >/dev/null 2>&1 || (echo "❌ Tilt not found. Install from: https://docs.tilt.dev/install.html" && exit 1)
	@chmod +x scripts/tilt-dev.sh
	@./scripts/tilt-dev.sh

tilt-down:
	@echo "⏹️  Stopping Tilt..."
	@command -v tilt >/dev/null 2>&1 || (echo "❌ Tilt not found" && exit 1)
	tilt down

tilt-clean:
	@echo "🧹 Cleaning up and restarting Tilt..."
	@command -v tilt >/dev/null 2>&1 || (echo "❌ Tilt not found" && exit 1)
	@chmod +x scripts/tilt-dev.sh
	@./scripts/tilt-dev.sh --clean

tilt-validate-enhanced:
	@echo "🔍 Validating enhanced Tilt configuration..."
	@chmod +x scripts/tilt-validate.sh
	@./scripts/tilt-validate.sh

tilt-performance-test:
	@echo "⚡ Running Tilt performance tests..."
	@chmod +x scripts/tilt-performance-test.sh
	@./scripts/tilt-performance-test.sh

tilt-dev-guide:
	@echo "💡 Opening Tilt development guide..."
	@chmod +x scripts/tilt-dev.sh
	@./scripts/tilt-dev.sh

tilt-test-perf:
	@echo "🧪 Testing Docker build performance..."
	@chmod +x scripts/test-docker-build.sh
	@./scripts/test-docker-build.sh

# Docker build optimization commands
warm-cache:
	@echo "🔥 Warming Docker build cache..."
	@chmod +x scripts/warm-cache.sh
	@./scripts/warm-cache.sh

test-build-perf:
	@echo "🧪 Testing Docker build performance..."
	@chmod +x scripts/test-build-simple.sh
	@./scripts/test-build-simple.sh

test-build-perf-full:
	@echo "🧪 Running comprehensive build performance tests..."
	@chmod +x scripts/test-docker-build.sh
	@./scripts/test-docker-build.sh

clean-cache:
	@echo "🧹 Cleaning Docker build cache..."
	@chmod +x scripts/refresh-cache.sh
	@./scripts/refresh-cache.sh clean all

refresh-cache:
	@echo "🔄 Refreshing Docker build cache..."
	@chmod +x scripts/refresh-cache.sh
	@./scripts/refresh-cache.sh refresh

cache-status:
	@echo "📊 Docker cache status..."
	@chmod +x scripts/refresh-cache.sh
	@./scripts/refresh-cache.sh show