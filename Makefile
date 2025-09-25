.PHONY: help setup dev clean test build

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

setup: ## Set up the development environment
	@echo "Setting up development environment..."
	docker-compose up -d postgres redis
	@echo "Waiting for databases to be ready..."
	sleep 5
	cd backend && cargo install sqlx-cli --no-default-features --features postgres
	cd backend && sqlx migrate run
	cd frontend && npm install
	@echo "Setup complete!"

dev: ## Start development servers
	@echo "Starting development servers..."
	docker-compose up -d postgres redis
	@echo "Backend will start on http://localhost:3000"
	@echo "Frontend will start on http://localhost:5000"
	@echo ""
	@echo "Run these commands in separate terminals:"
	@echo "  cd backend && cargo run"
	@echo "  cd frontend && npm run dev"

clean: ## Clean up development environment
	docker-compose down -v
	cd backend && cargo clean
	cd frontend && rm -rf node_modules

test: ## Run all tests
	cd backend && cargo test
	cd frontend && npm test

build: ## Build for production
	cd backend && cargo build --release
	cd frontend && npm run build

migrate: ## Run database migrations
	cd backend && sqlx migrate run

reset-db: ## Reset database (WARNING: destroys all data)
	docker-compose down postgres
	docker volume rm music-streaming-blocklist-manager_postgres_data || true
	docker-compose up -d postgres
	sleep 5
	cd backend && sqlx migrate run