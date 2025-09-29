#!/bin/bash

# Comprehensive test runner script for the backend
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
TEST_TYPE="all"
VERBOSE=false
COVERAGE=false
PARALLEL=false
CLEAN=false

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

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -t, --type TYPE     Test type: unit, integration, performance, all (default: all)"
    echo "  -v, --verbose       Enable verbose output"
    echo "  -c, --coverage      Generate coverage report"
    echo "  -p, --parallel      Run tests in parallel (where safe)"
    echo "  --clean             Clean test artifacts before running"
    echo "  -h, --help          Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 --type unit --verbose"
    echo "  $0 --coverage"
    echo "  $0 --type integration --clean"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--type)
            TEST_TYPE="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -c|--coverage)
            COVERAGE=true
            shift
            ;;
        -p|--parallel)
            PARALLEL=true
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Validate test type
case $TEST_TYPE in
    unit|integration|performance|all)
        ;;
    *)
        print_error "Invalid test type: $TEST_TYPE"
        print_error "Valid types: unit, integration, performance, all"
        exit 1
        ;;
esac

# Set up environment
export RUST_ENV=test
export RUST_LOG=debug
export RUST_BACKTRACE=1

# Clean artifacts if requested
if [ "$CLEAN" = true ]; then
    print_status "Cleaning test artifacts..."
    cargo clean
    rm -rf target/coverage
    rm -f lcov.info
fi

# Check if required tools are installed
check_dependencies() {
    print_status "Checking dependencies..."
    
    if ! command -v cargo &> /dev/null; then
        print_error "cargo is not installed"
        exit 1
    fi
    
    if [ "$COVERAGE" = true ] && ! command -v cargo-tarpaulin &> /dev/null; then
        print_warning "cargo-tarpaulin not found, installing..."
        cargo install cargo-tarpaulin
    fi
    
    # Check if Docker is running (needed for testcontainers)
    if ! docker info &> /dev/null; then
        print_error "Docker is not running. Please start Docker for integration tests."
        exit 1
    fi
}

# Run unit tests
run_unit_tests() {
    print_status "Running unit tests..."
    
    local cmd="cargo test --lib"
    
    if [ "$VERBOSE" = true ]; then
        cmd="$cmd -- --nocapture"
    fi
    
    if [ "$PARALLEL" = false ]; then
        cmd="$cmd -- --test-threads=1"
    fi
    
    # Run unit tests with specific pattern
    cmd="$cmd unit::"
    
    print_status "Executing: $cmd"
    eval $cmd
}

# Run integration tests
run_integration_tests() {
    print_status "Running integration tests..."
    
    # Start test database if needed
    print_status "Starting test database..."
    docker-compose -f docker-compose.test.yml up -d postgres redis
    
    # Wait for database to be ready
    print_status "Waiting for database to be ready..."
    sleep 5
    
    local cmd="cargo test --test '*integration*'"
    
    if [ "$VERBOSE" = true ]; then
        cmd="$cmd -- --nocapture"
    fi
    
    # Integration tests should run serially to avoid database conflicts
    cmd="$cmd -- --test-threads=1"
    
    print_status "Executing: $cmd"
    eval $cmd
    
    # Clean up test database
    print_status "Cleaning up test database..."
    docker-compose -f docker-compose.test.yml down -v
}

# Run performance tests
run_performance_tests() {
    print_status "Running performance tests..."
    
    # Start test database
    print_status "Starting test database for performance tests..."
    docker-compose -f docker-compose.test.yml up -d postgres redis
    
    # Wait for database
    sleep 5
    
    local cmd="cargo test --release performance::"
    
    if [ "$VERBOSE" = true ]; then
        cmd="$cmd -- --nocapture"
    fi
    
    cmd="$cmd -- --test-threads=1"
    
    print_status "Executing: $cmd"
    eval $cmd
    
    # Clean up
    docker-compose -f docker-compose.test.yml down -v
}

# Generate coverage report
generate_coverage() {
    print_status "Generating coverage report..."
    
    # Start test services
    docker-compose -f docker-compose.test.yml up -d postgres redis
    sleep 5
    
    cargo tarpaulin \
        --verbose \
        --all-features \
        --workspace \
        --timeout 120 \
        --out Html \
        --out Lcov \
        --output-dir target/coverage \
        --exclude-files 'tests/*' \
        --exclude-files 'target/*' \
        --exclude-files 'migrations/*'
    
    # Clean up
    docker-compose -f docker-compose.test.yml down -v
    
    print_success "Coverage report generated in target/coverage/"
}

# Main execution
main() {
    print_status "Starting test execution..."
    print_status "Test type: $TEST_TYPE"
    print_status "Verbose: $VERBOSE"
    print_status "Coverage: $COVERAGE"
    print_status "Parallel: $PARALLEL"
    
    check_dependencies
    
    # Record start time
    start_time=$(date +%s)
    
    case $TEST_TYPE in
        unit)
            run_unit_tests
            ;;
        integration)
            run_integration_tests
            ;;
        performance)
            run_performance_tests
            ;;
        all)
            run_unit_tests
            run_integration_tests
            run_performance_tests
            ;;
    esac
    
    # Generate coverage if requested
    if [ "$COVERAGE" = true ]; then
        generate_coverage
    fi
    
    # Calculate execution time
    end_time=$(date +%s)
    execution_time=$((end_time - start_time))
    
    print_success "All tests completed successfully!"
    print_success "Total execution time: ${execution_time} seconds"
    
    if [ "$COVERAGE" = true ]; then
        print_success "Coverage report available at: target/coverage/tarpaulin-report.html"
    fi
}

# Run main function
main