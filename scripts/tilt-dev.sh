#!/bin/bash
# Enhanced Tilt Development Workflow Script
# Provides guided setup and management for Tilt development environment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
NAMESPACE="ndith-dev"
TILT_UI_PORT="10350"

# Function to print colored output
print_header() {
    echo -e "${BLUE}$1${NC}"
    echo "$(echo "$1" | sed 's/./=/g')"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${CYAN}ℹ️  $1${NC}"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to wait for user input
wait_for_user() {
    echo ""
    read -p "Press Enter to continue..."
    echo ""
}

# Function to check prerequisites
check_prerequisites() {
    print_header "🔍 Checking Prerequisites"
    
    local all_good=true
    
    # Check Docker
    if command_exists docker && docker info >/dev/null 2>&1; then
        print_success "Docker is running"
    else
        print_error "Docker is not running or not installed"
        all_good=false
    fi
    
    # Check kubectl
    if command_exists kubectl; then
        print_success "kubectl is available"
    else
        print_error "kubectl is not installed"
        all_good=false
    fi
    
    # Check Tilt
    if command_exists tilt; then
        print_success "Tilt is available"
    else
        print_error "Tilt is not installed"
        echo "Install with: brew install tilt-dev/tap/tilt"
        all_good=false
    fi
    
    # Check Kubernetes cluster
    if kubectl cluster-info >/dev/null 2>&1; then
        local context=$(kubectl config current-context)
        print_success "Kubernetes cluster accessible ($context)"
    else
        print_warning "Kubernetes cluster not accessible"
        echo "Consider starting minikube: minikube start"
        all_good=false
    fi
    
    if [ "$all_good" = false ]; then
        print_error "Prerequisites not met. Please fix the issues above."
        exit 1
    fi
    
    echo ""
}

# Function to validate Tiltfile
validate_tiltfile() {
    print_header "📋 Validating Tiltfile Configuration"
    
    if tilt validate >/dev/null 2>&1; then
        print_success "Tiltfile syntax is valid"
    else
        print_error "Tiltfile validation failed"
        echo "Run 'tilt validate' for details"
        exit 1
    fi
    
    echo ""
}

# Function to warm cache
warm_cache() {
    print_header "🔥 Warming Docker Build Cache"
    
    print_info "This will pre-build Docker layers for faster development..."
    
    # Warm backend cache
    echo "Building backend base layers..."
    docker build --target chef -t ndith/backend:chef ./backend/ >/dev/null 2>&1 || true
    docker build --target planner -t ndith/backend:planner ./backend/ >/dev/null 2>&1 || true
    
    # Warm frontend cache
    echo "Building frontend base layers..."
    docker build --target builder -t ndith/frontend:builder ./frontend/ >/dev/null 2>&1 || true
    
    print_success "Cache warming completed"
    echo ""
}

# Function to start Tilt
start_tilt() {
    print_header "🚀 Starting Tilt Development Environment"
    
    print_info "Starting Tilt with enhanced configuration..."
    print_info "Tilt UI will be available at: http://localhost:$TILT_UI_PORT"
    
    echo ""
    echo "🎯 What happens next:"
    echo "  1. Tilt will build and deploy all services"
    echo "  2. Services will be available with port forwarding"
    echo "  3. Live updates will enable fast iteration"
    echo "  4. Manual triggers will be available for testing and maintenance"
    echo ""
    
    wait_for_user
    
    # Start Tilt
    tilt up
}

# Function to show development workflow
show_workflow() {
    print_header "💡 Development Workflow Guide"
    
    echo "🔄 Daily Development Workflow:"
    echo ""
    echo "1. 📊 Check Service Status:"
    echo "   • Open Tilt UI: http://localhost:$TILT_UI_PORT"
    echo "   • Wait for all services to show 'Ready'"
    echo "   • Run 'dev-setup' trigger for initialization"
    echo ""
    echo "2. 🧪 Development Cycle:"
    echo "   • Edit code in backend/src/ or frontend/src/"
    echo "   • Changes auto-reload within 10 seconds"
    echo "   • Use 'health-check' trigger to verify services"
    echo "   • Run specific tests with test triggers"
    echo ""
    echo "3. 🗄️  Database Management:"
    echo "   • Run 'db-migrate' for schema changes"
    echo "   • Use 'db-reset' to start fresh (destroys data)"
    echo ""
    echo "4. 🔍 Debugging:"
    echo "   • Check real-time logs in Tilt UI"
    echo "   • Use 'service-status' for detailed diagnostics"
    echo "   • Access services directly via port forwards"
    echo ""
    echo "5. 🧹 Maintenance:"
    echo "   • Run 'cleanup-resources' to clean up"
    echo "   • Use 'warm-cache' for build optimization"
    echo ""
    echo "📡 Service Endpoints:"
    echo "   • Backend API: http://localhost:3000"
    echo "   • Frontend: http://localhost:5000"
    echo "   • PostgreSQL: localhost:5432"
    echo "   • Redis: localhost:6379"
    echo ""
}

# Function to show manual triggers
show_triggers() {
    print_header "🎮 Available Manual Triggers"
    
    echo "📊 Monitoring & Health:"
    echo "   • health-check         - Check all service health"
    echo "   • service-status       - Detailed status report"
    echo ""
    echo "🗄️  Database Operations:"
    echo "   • db-migrate          - Run database migrations"
    echo "   • db-reset            - Reset database (destroys data)"
    echo ""
    echo "🧪 Testing:"
    echo "   • backend-tests       - Run all backend tests"
    echo "   • backend-unit-tests  - Backend unit tests only"
    echo "   • backend-integration-tests - Backend integration tests"
    echo "   • frontend-tests      - Run frontend tests"
    echo "   • all-tests          - Complete test suite"
    echo ""
    echo "⚡ Performance & Optimization:"
    echo "   • warm-cache         - Pre-build Docker layers"
    echo "   • build-performance-test - Measure build times"
    echo ""
    echo "🛠️  Workflow Automation:"
    echo "   • dev-setup          - Complete environment setup"
    echo "   • cleanup-resources  - Clean up old resources"
    echo ""
    echo "💡 Tip: Click trigger names in Tilt UI to execute them"
    echo ""
}

# Function to show troubleshooting tips
show_troubleshooting() {
    print_header "🔧 Troubleshooting Guide"
    
    echo "🚨 Common Issues & Solutions:"
    echo ""
    echo "❌ Services not starting:"
    echo "   • Check Tilt UI for error messages"
    echo "   • Run 'service-status' trigger for diagnostics"
    echo "   • Verify Kubernetes cluster is running"
    echo "   • Try 'cleanup-resources' and restart"
    echo ""
    echo "❌ Slow builds:"
    echo "   • Run 'warm-cache' trigger"
    echo "   • Check Docker daemon has sufficient resources"
    echo "   • Use 'build-performance-test' to measure improvements"
    echo ""
    echo "❌ Database connection issues:"
    echo "   • Wait for PostgreSQL to be 'Ready' in Tilt UI"
    echo "   • Run 'db-migrate' trigger"
    echo "   • Check port forwarding is working (localhost:5432)"
    echo ""
    echo "❌ Live updates not working:"
    echo "   • Check file paths in Tiltfile live_update rules"
    echo "   • Verify file changes are being detected"
    echo "   • Try manual rebuild in Tilt UI"
    echo ""
    echo "❌ Port forwarding issues:"
    echo "   • Check Tilt UI port forward status"
    echo "   • Verify no other services using same ports"
    echo "   • Restart Tilt if needed"
    echo ""
    echo "🆘 Emergency Reset:"
    echo "   1. tilt down"
    echo "   2. Run 'cleanup-resources' trigger"
    echo "   3. docker system prune -f"
    echo "   4. tilt up"
    echo ""
}

# Main menu
show_menu() {
    clear
    echo -e "${CYAN}"
    echo "🚀 NDITH Tilt Development Environment Manager"
    echo "============================================"
    echo -e "${NC}"
    echo ""
    echo "Choose an option:"
    echo ""
    echo "1) 🏁 Quick Start (recommended for first time)"
    echo "2) 🚀 Start Tilt Development Environment"
    echo "3) 🔍 Validate Configuration"
    echo "4) 🔥 Warm Docker Cache"
    echo "5) 💡 Show Development Workflow"
    echo "6) 🎮 Show Manual Triggers"
    echo "7) 🔧 Troubleshooting Guide"
    echo "8) 📊 Performance Test"
    echo "9) 🚪 Exit"
    echo ""
    read -p "Enter your choice (1-9): " choice
    echo ""
}

# Quick start function
quick_start() {
    print_header "🏁 Quick Start Setup"
    
    echo "This will:"
    echo "  1. Check all prerequisites"
    echo "  2. Validate Tiltfile configuration"
    echo "  3. Warm Docker build cache"
    echo "  4. Start Tilt development environment"
    echo ""
    
    wait_for_user
    
    check_prerequisites
    validate_tiltfile
    warm_cache
    show_workflow
    start_tilt
}

# Main script logic
main() {
    while true; do
        show_menu
        
        case $choice in
            1)
                quick_start
                ;;
            2)
                check_prerequisites
                start_tilt
                ;;
            3)
                validate_tiltfile
                print_success "Configuration is valid!"
                wait_for_user
                ;;
            4)
                warm_cache
                wait_for_user
                ;;
            5)
                show_workflow
                wait_for_user
                ;;
            6)
                show_triggers
                wait_for_user
                ;;
            7)
                show_troubleshooting
                wait_for_user
                ;;
            8)
                if [ -f "scripts/tilt-performance-test.sh" ]; then
                    ./scripts/tilt-performance-test.sh
                else
                    print_error "Performance test script not found"
                fi
                wait_for_user
                ;;
            9)
                echo "👋 Happy coding!"
                exit 0
                ;;
            *)
                print_error "Invalid option. Please choose 1-9."
                wait_for_user
                ;;
        esac
    done
}

# Run main function
main