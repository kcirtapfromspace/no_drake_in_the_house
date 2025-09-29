#!/bin/bash
# Simple build test to verify Docker optimization works

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Enable BuildKit for consistent testing
export DOCKER_BUILDKIT=1

echo -e "${BLUE}🧪 Simple Docker Build Test${NC}"
echo "=================================="

# Test backend dependency caching
echo -e "\n${BLUE}📦 Testing Backend Dependency Caching${NC}"
echo "--------------------------------------"

echo -e "${YELLOW}🏗️  Building backend dependencies (chef stage)...${NC}"
start_time=$(date +%s)
if docker build --target chef -t kiro-test:backend-chef -f backend/Dockerfile.fast backend/ > /dev/null 2>&1; then
    end_time=$(date +%s)
    chef_time=$((end_time - start_time))
    echo -e "${GREEN}✅ Chef stage: ${chef_time}s${NC}"
else
    echo -e "${RED}❌ Chef stage failed${NC}"
    chef_time=999
fi

echo -e "${YELLOW}🏗️  Building backend planner...${NC}"
start_time=$(date +%s)
if docker build --target planner -t kiro-test:backend-planner -f backend/Dockerfile.fast backend/ > /dev/null 2>&1; then
    end_time=$(date +%s)
    planner_time=$((end_time - start_time))
    echo -e "${GREEN}✅ Planner stage: ${planner_time}s${NC}"
else
    echo -e "${RED}❌ Planner stage failed${NC}"
    planner_time=999
fi

# Test frontend dependency caching
echo -e "\n${BLUE}🎨 Testing Frontend Dependency Caching${NC}"
echo "---------------------------------------"

echo -e "${YELLOW}🏗️  Building frontend dependencies...${NC}"
start_time=$(date +%s)
if docker build --target builder -t kiro-test:frontend-builder -f frontend/Dockerfile.fast frontend/ > /dev/null 2>&1; then
    end_time=$(date +%s)
    frontend_time=$((end_time - start_time))
    echo -e "${GREEN}✅ Frontend build: ${frontend_time}s${NC}"
else
    echo -e "${RED}❌ Frontend build failed${NC}"
    frontend_time=999
fi

# Summary
echo -e "\n${BLUE}📊 Build Performance Summary${NC}"
echo "============================"
echo "Backend Chef:    ${chef_time}s"
echo "Backend Planner: ${planner_time}s"
echo "Frontend Build:  ${frontend_time}s"

# Cleanup
echo -e "\n${YELLOW}🧹 Cleaning up test images...${NC}"
docker rmi kiro-test:backend-chef kiro-test:backend-planner kiro-test:frontend-builder > /dev/null 2>&1 || true

if [ "$chef_time" -lt 60 ] && [ "$planner_time" -lt 60 ] && [ "$frontend_time" -lt 120 ]; then
    echo -e "\n${GREEN}✅ Build optimization is working! All stages completed in reasonable time.${NC}"
    exit 0
else
    echo -e "\n${YELLOW}⚠️  Some builds took longer than expected. Check Docker BuildKit and cache configuration.${NC}"
    exit 1
fi