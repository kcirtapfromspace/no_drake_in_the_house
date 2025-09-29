#!/bin/bash
# Build performance testing script to measure optimization gains

set -euo pipefail

# Enable BuildKit for consistent testing
export DOCKER_BUILDKIT=1
export BUILDKIT_PROGRESS=plain

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to measure build time
measure_build() {
    local dockerfile=$1
    local context=$2
    local tag=$3
    local description=$4
    
    echo -e "${BLUE}🧪 Testing: $description${NC}"
    
    local start_time=$(date +%s)
    
    if docker build -f "$dockerfile" -t "$tag" "$context" > /dev/null 2>&1; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        echo -e "${GREEN}✅ Success: ${duration}s${NC}"
        return $duration
    else
        echo -e "${RED}❌ Failed${NC}"
        return 999
    fi
}

# Function to clean build cache
clean_cache() {
    echo -e "${YELLOW}🧹 Cleaning build cache...${NC}"
    docker builder prune -f > /dev/null 2>&1 || true
    docker system prune -f > /dev/null 2>&1 || true
}

echo -e "${BLUE}🧪 Docker Build Performance Testing${NC}"
echo "=================================================="

# Test backend builds
echo -e "\n${BLUE}📦 Backend Build Performance${NC}"
echo "----------------------------------"

# Cold cache test - standard Dockerfile
clean_cache
backend_standard_cold=$(measure_build "backend/Dockerfile" "backend" "test-backend-standard" "Backend Standard (Cold Cache)")

# Warm cache test - standard Dockerfile  
backend_standard_warm=$(measure_build "backend/Dockerfile" "backend" "test-backend-standard" "Backend Standard (Warm Cache)")

# Cold cache test - fast Dockerfile
clean_cache
backend_fast_cold=$(measure_build "backend/Dockerfile.fast" "backend" "test-backend-fast" "Backend Fast (Cold Cache)")

# Warm cache test - fast Dockerfile
backend_fast_warm=$(measure_build "backend/Dockerfile.fast" "backend" "test-backend-fast" "Backend Fast (Warm Cache)")

# Test frontend builds
echo -e "\n${BLUE}🎨 Frontend Build Performance${NC}"
echo "----------------------------------"

# Cold cache test - standard Dockerfile
clean_cache
frontend_standard_cold=$(measure_build "frontend/Dockerfile" "frontend" "test-frontend-standard" "Frontend Standard (Cold Cache)")

# Warm cache test - standard Dockerfile
frontend_standard_warm=$(measure_build "frontend/Dockerfile" "frontend" "test-frontend-standard" "Frontend Standard (Warm Cache)")

# Cold cache test - fast Dockerfile
clean_cache
frontend_fast_cold=$(measure_build "frontend/Dockerfile.fast" "frontend" "test-frontend-fast" "Frontend Fast (Cold Cache)")

# Warm cache test - fast Dockerfile
frontend_fast_warm=$(measure_build "frontend/Dockerfile.fast" "frontend" "test-frontend-fast" "Frontend Fast (Warm Cache)")

# Calculate improvements
echo -e "\n${BLUE}📊 Performance Summary${NC}"
echo "=================================================="

calculate_improvement() {
    local old=$1
    local new=$2
    if [ "$old" -gt 0 ] && [ "$new" -gt 0 ] && [ "$old" -ne 999 ] && [ "$new" -ne 999 ]; then
        local improvement=$(( (old - new) * 100 / old ))
        echo "${improvement}%"
    else
        echo "N/A"
    fi
}

echo "Backend Improvements:"
echo "  Cold Cache: $(calculate_improvement $backend_standard_cold $backend_fast_cold) faster"
echo "  Warm Cache: $(calculate_improvement $backend_standard_warm $backend_fast_warm) faster"

echo ""
echo "Frontend Improvements:"
echo "  Cold Cache: $(calculate_improvement $frontend_standard_cold $frontend_fast_cold) faster"
echo "  Warm Cache: $(calculate_improvement $frontend_standard_warm $frontend_fast_warm) faster"

echo ""
echo "Build Times Summary:"
echo "  Backend Standard: ${backend_standard_cold}s (cold) → ${backend_standard_warm}s (warm)"
echo "  Backend Fast:     ${backend_fast_cold}s (cold) → ${backend_fast_warm}s (warm)"
echo "  Frontend Standard: ${frontend_standard_cold}s (cold) → ${frontend_standard_warm}s (warm)"
echo "  Frontend Fast:     ${frontend_fast_cold}s (cold) → ${frontend_fast_warm}s (warm)"

# Cleanup test images
echo -e "\n${YELLOW}🧹 Cleaning up test images...${NC}"
docker rmi test-backend-standard test-backend-fast test-frontend-standard test-frontend-fast > /dev/null 2>&1 || true

echo -e "\n${GREEN}✅ Build performance testing complete!${NC}"

# Check if we achieved the 80% improvement target
if [ "$backend_fast_warm" -lt "$((backend_standard_warm * 20 / 100))" ] 2>/dev/null; then
    echo -e "${GREEN}🎯 Target achieved: Backend builds are >80% faster with warm cache!${NC}"
else
    echo -e "${YELLOW}⚠️  Backend warm cache improvement may be less than 80% target${NC}"
fi