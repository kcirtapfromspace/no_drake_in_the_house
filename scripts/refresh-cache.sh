#!/bin/bash
# Cache cleanup and refresh script for stale builds

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔄 Docker Cache Refresh Utility${NC}"
echo "=================================="

# Function to show cache usage
show_cache_usage() {
    echo -e "\n${BLUE}📊 Current Cache Usage${NC}"
    echo "----------------------"
    
    # Show Docker system usage
    docker system df 2>/dev/null || echo "Unable to get Docker system usage"
    
    echo ""
    echo "Kiro Cache Images:"
    docker images --filter "reference=kiro-cache:*" --format "table {{.Repository}}:{{.Tag}}\t{{.Size}}\t{{.CreatedSince}}" 2>/dev/null || echo "No kiro-cache images found"
}

# Function to clean specific cache types
clean_cache_type() {
    local cache_type=$1
    
    case $cache_type in
        "build")
            echo -e "${YELLOW}🧹 Cleaning build cache...${NC}"
            docker builder prune -f
            ;;
        "images")
            echo -e "${YELLOW}🧹 Cleaning unused images...${NC}"
            docker image prune -f
            ;;
        "kiro")
            echo -e "${YELLOW}🧹 Cleaning Kiro cache images...${NC}"
            docker images --filter "reference=kiro-cache:*" -q | xargs -r docker rmi -f
            ;;
        "all")
            echo -e "${YELLOW}🧹 Cleaning all Docker cache...${NC}"
            docker system prune -af --volumes
            ;;
        *)
            echo -e "${RED}❌ Unknown cache type: $cache_type${NC}"
            return 1
            ;;
    esac
}

# Function to refresh cache
refresh_cache() {
    echo -e "${BLUE}🔄 Refreshing Docker cache...${NC}"
    
    # Clean old cache
    clean_cache_type "kiro"
    clean_cache_type "build"
    
    # Rebuild cache
    echo -e "${BLUE}🏗️  Rebuilding cache...${NC}"
    ./scripts/warm-cache.sh
}

# Parse command line arguments
case "${1:-help}" in
    "show"|"status")
        show_cache_usage
        ;;
    "clean")
        cache_type="${2:-build}"
        clean_cache_type "$cache_type"
        echo -e "${GREEN}✅ Cache cleanup complete${NC}"
        ;;
    "refresh")
        refresh_cache
        echo -e "${GREEN}✅ Cache refresh complete${NC}"
        ;;
    "help"|*)
        echo "Usage: $0 [command] [options]"
        echo ""
        echo "Commands:"
        echo "  show              Show current cache usage"
        echo "  clean [type]      Clean cache (types: build, images, kiro, all)"
        echo "  refresh           Clean and rebuild cache"
        echo "  help              Show this help message"
        echo ""
        echo "Examples:"
        echo "  $0 show           # Show cache status"
        echo "  $0 clean build    # Clean build cache only"
        echo "  $0 clean all      # Clean all Docker cache"
        echo "  $0 refresh        # Full cache refresh"
        ;;
esac