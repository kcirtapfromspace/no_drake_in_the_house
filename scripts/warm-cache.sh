#!/bin/bash
# Cache warming script to pre-build Docker layers for faster development

set -euo pipefail

echo "🔥 Warming Docker build cache for faster development..."

# Enable BuildKit for advanced caching
export DOCKER_BUILDKIT=1
export BUILDKIT_PROGRESS=plain

# Function to warm cache for a specific target
warm_target() {
    local dockerfile=$1
    local context=$2
    local target=$3
    local tag=$4
    
    echo "🏗️  Warming cache for $tag (target: $target)..."
    
    # Special handling for backend builds - prepare SQLx if needed
    if [[ "$context" == "backend" && "$target" == "builder" ]]; then
        echo "📝 Preparing SQLx queries for backend build..."
        (cd backend && ./prepare-sqlx.sh) || {
            echo "⚠️  SQLx preparation failed, using offline mode..."
        }
    fi
    
    docker build \
        --target "$target" \
        --tag "ndith-cache:$tag" \
        --file "$dockerfile" \
        "$context" || {
        echo "⚠️  Warning: Failed to warm cache for $tag, continuing..."
    }
}

# Warm backend caches
echo "📦 Warming Rust backend caches..."
warm_target "backend/Dockerfile" "backend" "chef" "backend-chef"
warm_target "backend/Dockerfile" "backend" "planner" "backend-planner"
warm_target "backend/Dockerfile" "backend" "builder" "backend-builder"

# Warm fast backend caches
echo "⚡ Warming fast backend caches..."
warm_target "backend/Dockerfile.fast" "backend" "chef" "backend-fast-chef"
warm_target "backend/Dockerfile.fast" "backend" "planner" "backend-fast-planner"
warm_target "backend/Dockerfile.fast" "backend" "builder" "backend-fast-builder"

# Warm frontend caches
echo "🎨 Warming frontend caches..."
warm_target "frontend/Dockerfile" "frontend" "builder" "frontend-builder"

# Warm fast frontend caches
echo "⚡ Warming fast frontend caches..."
warm_target "frontend/Dockerfile.fast" "frontend" "builder" "frontend-fast-builder"

# Pre-pull base images
echo "📥 Pre-pulling base images..."
docker pull rust:1.82-slim || echo "⚠️  Warning: Failed to pull rust:1.82-slim"
docker pull node:18-alpine || echo "⚠️  Warning: Failed to pull node:18-alpine"
docker pull debian:bookworm-slim || echo "⚠️  Warning: Failed to pull debian:bookworm-slim"
docker pull nginx:alpine || echo "⚠️  Warning: Failed to pull nginx:alpine"

echo "✅ Cache warming complete!"
echo ""
echo "📊 Cache status:"
docker images --filter "reference=ndith-cache:*" --format "table {{.Repository}}:{{.Tag}}\t{{.Size}}\t{{.CreatedSince}}"
echo ""
echo "🚀 Next builds will be significantly faster!"
echo "   - Use 'docker build -f backend/Dockerfile.fast backend/' for fastest backend builds"
echo "   - Use 'docker build -f frontend/Dockerfile.fast frontend/' for fastest frontend builds"