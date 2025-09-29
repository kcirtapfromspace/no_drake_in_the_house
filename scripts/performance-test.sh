#!/bin/bash

# Performance testing script for No Drake in the House
# This script runs comprehensive performance tests across the entire stack

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BACKEND_URL="http://localhost:3000"
FRONTEND_URL="http://localhost:5000"
TEST_DURATION="30s"
CONCURRENT_USERS="50"
RESULTS_DIR="performance-results"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install missing tools
install_tools() {
    print_status "Checking for required performance testing tools..."
    
    # Check for wrk
    if ! command_exists wrk; then
        print_warning "wrk not found. Installing..."
        if [[ "$OSTYPE" == "darwin"* ]]; then
            brew install wrk
        elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
            sudo apt-get update && sudo apt-get install -y wrk
        else
            print_error "Please install wrk manually: https://github.com/wg/wrk"
            exit 1
        fi
    fi
    
    # Check for artillery
    if ! command_exists artillery; then
        print_warning "artillery not found. Installing..."
        npm install -g artillery
    fi
    
    # Check for lighthouse
    if ! command_exists lighthouse; then
        print_warning "lighthouse not found. Installing..."
        npm install -g lighthouse
    fi
    
    print_success "All required tools are available"
}

# Function to setup test environment
setup_environment() {
    print_status "Setting up test environment..."
    
    # Create results directory
    mkdir -p "$RESULTS_DIR"
    
    # Start services if not running
    if ! curl -s "$BACKEND_URL/health" >/dev/null 2>&1; then
        print_status "Starting development environment..."
        make dev
        
        # Wait for services to be ready
        print_status "Waiting for services to be ready..."
        for i in {1..60}; do
            if curl -s "$BACKEND_URL/health" >/dev/null 2>&1; then
                break
            fi
            if [ $i -eq 60 ]; then
                print_error "Backend failed to start within 60 seconds"
                exit 1
            fi
            sleep 1
        done
        
        # Additional wait for full initialization
        sleep 10
    fi
    
    print_success "Test environment ready"
}

# Function to run backend performance tests
test_backend_performance() {
    print_status "Running backend performance tests..."
    
    # Health check endpoint test
    print_status "Testing health check endpoint..."
    wrk -t4 -c20 -d"$TEST_DURATION" --latency "$BACKEND_URL/health" > "$RESULTS_DIR/health-check.txt" 2>&1
    
    # Create test user for authenticated endpoints
    print_status "Creating test user..."
    TEST_EMAIL="perf_test_$(date +%s)@example.com"
    TEST_PASSWORD="secure_password123"
    
    curl -s -X POST "$BACKEND_URL/api/v1/auth/register" \
        -H "Content-Type: application/json" \
        -d "{\"email\":\"$TEST_EMAIL\",\"password\":\"$TEST_PASSWORD\"}" \
        > /dev/null || true
    
    # Get auth token
    AUTH_RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/v1/auth/login" \
        -H "Content-Type: application/json" \
        -d "{\"email\":\"$TEST_EMAIL\",\"password\":\"$TEST_PASSWORD\"}")
    
    if echo "$AUTH_RESPONSE" | grep -q "access_token"; then
        ACCESS_TOKEN=$(echo "$AUTH_RESPONSE" | jq -r '.access_token')
        
        # Test authenticated endpoints
        print_status "Testing authenticated endpoints..."
        
        # Create artillery config for authenticated tests
        cat > "$RESULTS_DIR/artillery-auth.yml" << EOF
config:
  target: '$BACKEND_URL'
  phases:
    - duration: 30
      arrivalRate: 5
  defaults:
    headers:
      Authorization: 'Bearer $ACCESS_TOKEN'
      Content-Type: 'application/json'
scenarios:
  - name: "User profile"
    requests:
      - get:
          url: "/api/v1/users/profile"
  - name: "Artist search"
    requests:
      - get:
          url: "/api/v1/artists/search?q=test&limit=10"
  - name: "DNP list"
    requests:
      - get:
          url: "/api/v1/dnp?limit=20"
EOF
        
        artillery run "$RESULTS_DIR/artillery-auth.yml" > "$RESULTS_DIR/authenticated-endpoints.txt" 2>&1
    else
        print_warning "Could not authenticate test user, skipping authenticated endpoint tests"
    fi
    
    # Run Rust benchmarks
    print_status "Running Rust benchmarks..."
    cd backend
    if cargo bench > "../$RESULTS_DIR/rust-benchmarks.txt" 2>&1; then
        print_success "Rust benchmarks completed"
    else
        print_warning "Rust benchmarks failed or not available"
    fi
    cd ..
    
    print_success "Backend performance tests completed"
}

# Function to run frontend performance tests
test_frontend_performance() {
    print_status "Running frontend performance tests..."
    
    # Check if frontend is accessible
    if ! curl -s "$FRONTEND_URL" >/dev/null 2>&1; then
        print_warning "Frontend not accessible at $FRONTEND_URL, skipping frontend tests"
        return
    fi
    
    # Lighthouse performance audit
    print_status "Running Lighthouse performance audit..."
    lighthouse "$FRONTEND_URL" \
        --output json \
        --output html \
        --output-path "$RESULTS_DIR/lighthouse" \
        --chrome-flags="--headless --no-sandbox --disable-dev-shm-usage" \
        --quiet || print_warning "Lighthouse audit failed"
    
    # Frontend load test
    print_status "Running frontend load test..."
    wrk -t2 -c10 -d15s "$FRONTEND_URL" > "$RESULTS_DIR/frontend-load.txt" 2>&1
    
    print_success "Frontend performance tests completed"
}

# Function to run database performance tests
test_database_performance() {
    print_status "Running database performance tests..."
    
    # Check if we can connect to the database
    if docker compose exec -T postgres pg_isready -U kiro -d kiro_dev >/dev/null 2>&1; then
        # Run database performance queries
        cat > "$RESULTS_DIR/db-performance.sql" << 'EOF'
-- Database performance analysis
\timing on

-- Test query performance
EXPLAIN (ANALYZE, BUFFERS) SELECT * FROM users LIMIT 100;
EXPLAIN (ANALYZE, BUFFERS) SELECT * FROM artists WHERE canonical_name ILIKE '%test%' LIMIT 20;

-- Check index usage
SELECT 
    schemaname,
    tablename,
    indexname,
    idx_scan,
    idx_tup_read,
    idx_tup_fetch
FROM pg_stat_user_indexes
ORDER BY idx_scan DESC;

-- Check table statistics
SELECT 
    schemaname,
    tablename,
    n_tup_ins,
    n_tup_upd,
    n_tup_del,
    n_live_tup,
    n_dead_tup
FROM pg_stat_user_tables;

-- Reset statistics for clean measurement
SELECT pg_stat_reset();
EOF
        
        docker compose exec -T postgres psql -U kiro -d kiro_dev -f - < "$RESULTS_DIR/db-performance.sql" > "$RESULTS_DIR/database-performance.txt" 2>&1
        
        print_success "Database performance tests completed"
    else
        print_warning "Could not connect to database, skipping database tests"
    fi
}

# Function to run infrastructure performance tests
test_infrastructure_performance() {
    print_status "Running infrastructure performance tests..."
    
    # Docker container stats
    print_status "Collecting Docker container statistics..."
    docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}\t{{.BlockIO}}" > "$RESULTS_DIR/docker-stats.txt"
    
    # System resource usage
    print_status "Collecting system resource usage..."
    {
        echo "=== CPU Information ==="
        if command_exists lscpu; then
            lscpu
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            sysctl -n machdep.cpu.brand_string
            sysctl -n hw.ncpu
        fi
        
        echo -e "\n=== Memory Information ==="
        if command_exists free; then
            free -h
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            vm_stat
        fi
        
        echo -e "\n=== Disk Usage ==="
        df -h
        
        echo -e "\n=== Load Average ==="
        uptime
    } > "$RESULTS_DIR/system-info.txt"
    
    print_success "Infrastructure performance tests completed"
}

# Function to generate performance report
generate_report() {
    print_status "Generating performance report..."
    
    cat > "$RESULTS_DIR/performance-report.md" << EOF
# Performance Test Report

Generated on: $(date)

## Test Configuration

- Backend URL: $BACKEND_URL
- Frontend URL: $FRONTEND_URL
- Test Duration: $TEST_DURATION
- Concurrent Users: $CONCURRENT_USERS

## Test Results Summary

### Backend Performance

EOF
    
    # Add backend results if available
    if [ -f "$RESULTS_DIR/health-check.txt" ]; then
        echo "#### Health Check Endpoint" >> "$RESULTS_DIR/performance-report.md"
        echo '```' >> "$RESULTS_DIR/performance-report.md"
        grep -E "(Requests/sec|Latency|Transfer/sec)" "$RESULTS_DIR/health-check.txt" >> "$RESULTS_DIR/performance-report.md"
        echo '```' >> "$RESULTS_DIR/performance-report.md"
        echo "" >> "$RESULTS_DIR/performance-report.md"
    fi
    
    # Add frontend results if available
    if [ -f "$RESULTS_DIR/lighthouse.report.json" ]; then
        echo "### Frontend Performance" >> "$RESULTS_DIR/performance-report.md"
        echo "" >> "$RESULTS_DIR/performance-report.md"
        
        # Extract key metrics from Lighthouse report
        if command_exists jq; then
            {
                echo "#### Lighthouse Scores"
                echo "- Performance: $(jq -r '.categories.performance.score * 100' "$RESULTS_DIR/lighthouse.report.json")%"
                echo "- Accessibility: $(jq -r '.categories.accessibility.score * 100' "$RESULTS_DIR/lighthouse.report.json")%"
                echo "- Best Practices: $(jq -r '.categories["best-practices"].score * 100' "$RESULTS_DIR/lighthouse.report.json")%"
                echo "- SEO: $(jq -r '.categories.seo.score * 100' "$RESULTS_DIR/lighthouse.report.json")%"
                echo ""
                echo "#### Core Web Vitals"
                echo "- First Contentful Paint: $(jq -r '.audits["first-contentful-paint"].displayValue' "$RESULTS_DIR/lighthouse.report.json")"
                echo "- Largest Contentful Paint: $(jq -r '.audits["largest-contentful-paint"].displayValue' "$RESULTS_DIR/lighthouse.report.json")"
                echo "- Cumulative Layout Shift: $(jq -r '.audits["cumulative-layout-shift"].displayValue' "$RESULTS_DIR/lighthouse.report.json")"
            } >> "$RESULTS_DIR/performance-report.md"
        fi
        echo "" >> "$RESULTS_DIR/performance-report.md"
    fi
    
    # Add infrastructure info
    echo "### Infrastructure" >> "$RESULTS_DIR/performance-report.md"
    echo '```' >> "$RESULTS_DIR/performance-report.md"
    if [ -f "$RESULTS_DIR/docker-stats.txt" ]; then
        cat "$RESULTS_DIR/docker-stats.txt" >> "$RESULTS_DIR/performance-report.md"
    fi
    echo '```' >> "$RESULTS_DIR/performance-report.md"
    
    # Add file list
    echo "" >> "$RESULTS_DIR/performance-report.md"
    echo "## Detailed Results" >> "$RESULTS_DIR/performance-report.md"
    echo "" >> "$RESULTS_DIR/performance-report.md"
    echo "The following files contain detailed test results:" >> "$RESULTS_DIR/performance-report.md"
    echo "" >> "$RESULTS_DIR/performance-report.md"
    
    for file in "$RESULTS_DIR"/*.txt "$RESULTS_DIR"/*.json "$RESULTS_DIR"/*.html; do
        if [ -f "$file" ]; then
            echo "- $(basename "$file")" >> "$RESULTS_DIR/performance-report.md"
        fi
    done
    
    print_success "Performance report generated: $RESULTS_DIR/performance-report.md"
}

# Function to cleanup
cleanup() {
    print_status "Cleaning up temporary files..."
    rm -f "$RESULTS_DIR/artillery-auth.yml" "$RESULTS_DIR/db-performance.sql"
}

# Main execution
main() {
    print_status "Starting No Drake in the House performance test suite..."
    
    # Check if we're in the project root
    if [[ ! -f "Makefile" ]] || [[ ! -d "backend" ]] || [[ ! -d "frontend" ]]; then
        print_error "Please run this script from the project root directory"
        exit 1
    fi
    
    install_tools
    setup_environment
    
    test_backend_performance
    test_frontend_performance
    test_database_performance
    test_infrastructure_performance
    
    generate_report
    cleanup
    
    print_success "Performance testing completed!"
    print_status "Results available in: $RESULTS_DIR/"
    print_status "Summary report: $RESULTS_DIR/performance-report.md"
    
    # Open report if on macOS
    if [[ "$OSTYPE" == "darwin"* ]] && command_exists open; then
        open "$RESULTS_DIR/performance-report.md"
    fi
}

# Handle script interruption
trap cleanup EXIT

# Run main function
main "$@"