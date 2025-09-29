#!/bin/bash
# Tilt Performance Testing Script
# Tests build times and optimization effectiveness

set -e

echo "🏁 Tilt Performance Testing Suite"
echo "=================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to measure time
measure_time() {
    local start_time=$(date +%s.%N)
    "$@"
    local end_time=$(date +%s.%N)
    local duration=$(echo "$end_time - $start_time" | bc -l)
    echo "$duration"
}

# Function to format time
format_time() {
    local time=$1
    printf "%.2f seconds" "$time"
}

echo ""
echo -e "${BLUE}📊 Testing Docker Build Performance${NC}"
echo "-----------------------------------"

# Test cold build (no cache)
echo -e "${YELLOW}🧊 Testing cold build performance...${NC}"
docker system prune -f > /dev/null 2>&1

echo "Building backend (cold)..."
backend_cold_time=$(measure_time docker build -t kiro/backend:perf-test ./backend/ > /dev/null 2>&1)

echo "Building frontend (cold)..."
frontend_cold_time=$(measure_time docker build -t kiro/frontend:perf-test ./frontend/ > /dev/null 2>&1)

# Test warm build (with cache)
echo -e "${YELLOW}🔥 Testing warm build performance...${NC}"

echo "Building backend (warm)..."
backend_warm_time=$(measure_time docker build -t kiro/backend:perf-test ./backend/ > /dev/null 2>&1)

echo "Building frontend (warm)..."
frontend_warm_time=$(measure_time docker build -t kiro/frontend:perf-test ./frontend/ > /dev/null 2>&1)

# Calculate improvements
backend_improvement=$(echo "scale=1; ($backend_cold_time - $backend_warm_time) / $backend_cold_time * 100" | bc -l)
frontend_improvement=$(echo "scale=1; ($frontend_cold_time - $frontend_warm_time) / $frontend_cold_time * 100" | bc -l)

echo ""
echo -e "${GREEN}📈 Performance Results${NC}"
echo "====================="
echo ""
echo "Backend Build Times:"
echo "  Cold build: $(format_time $backend_cold_time)"
echo "  Warm build: $(format_time $backend_warm_time)"
echo "  Improvement: ${backend_improvement}%"
echo ""
echo "Frontend Build Times:"
echo "  Cold build: $(format_time $frontend_cold_time)"
echo "  Warm build: $(format_time $frontend_warm_time)"
echo "  Improvement: ${frontend_improvement}%"
echo ""

# Test Tilt startup time if Tilt is available
if command -v tilt &> /dev/null; then
    echo -e "${BLUE}🚀 Testing Tilt Startup Performance${NC}"
    echo "-----------------------------------"
    
    # Stop any running Tilt
    tilt down > /dev/null 2>&1 || true
    
    echo "Measuring Tilt startup time..."
    tilt_start_time=$(measure_time timeout 300 tilt up --stream=false > /dev/null 2>&1 || true)
    
    echo "Tilt startup time: $(format_time $tilt_start_time)"
    
    # Clean up
    tilt down > /dev/null 2>&1 || true
else
    echo -e "${YELLOW}⚠️  Tilt not found - skipping Tilt performance tests${NC}"
fi

echo ""
echo -e "${GREEN}✅ Performance testing completed!${NC}"
echo ""
echo "💡 Optimization Tips:"
echo "  • Run 'make warm-cache' before development sessions"
echo "  • Use live updates in Tilt for fastest iteration"
echo "  • Keep Docker daemon running for better cache performance"
echo "  • Consider using BuildKit for advanced caching features"