#!/bin/bash

# Complete development environment setup script
# This script sets up everything needed for ultra-fast Rust + Tilt development

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# Main setup function
main() {
    echo "🚀 No Drake in the House - Ultra-Fast Development Setup"
    echo ""
    echo "This script will set up:"
    echo "  ✅ Minikube + Tilt environment"
    echo "  ✅ Optimized Docker builds with caching"
    echo "  ✅ Live reloading for Rust and Svelte"
    echo "  ✅ Performance testing tools"
    echo ""
    
    read -p "Continue with setup? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Setup cancelled."
        exit 0
    fi
    
    # Check if we're in the project root
    if [[ ! -f "Tiltfile" ]] || [[ ! -d "backend" ]] || [[ ! -d "frontend" ]]; then
        print_error "Please run this script from the project root directory"
        exit 1
    fi
    
    # Step 1: Setup minikube and Tilt
    print_status "Step 1/3: Setting up Minikube + Tilt..."
    make tilt-setup
    
    # Step 2: Warm up Docker cache
    print_status "Step 2/3: Warming up Docker build cache..."
    print_warning "This may take 2-3 minutes but will save 5+ minutes on every build"
    make tilt-warm-cache
    
    # Step 3: Test performance
    print_status "Step 3/3: Testing build performance..."
    make tilt-test-perf
    
    echo ""
    print_success "🎉 Development environment setup complete!"
    echo ""
    echo "⚡ Your environment is now optimized for:"
    echo "  - Sub-30 second Rust rebuilds (vs 2-5 minutes normally)"
    echo "  - Instant frontend updates"
    echo "  - Live reloading in Kubernetes"
    echo "  - Production-like environment"
    echo ""
    echo "🚀 Start developing:"
    echo "  tilt up    # Start the development environment"
    echo ""
    echo "💡 Pro tips:"
    echo "  - Keep Docker running to maintain cache"
    echo "  - Use the Tilt UI to monitor builds and logs"
    echo "  - Run 'make tilt-warm-cache' if builds get slow again"
    echo "  - Edit code and watch it reload in seconds!"
}

# Run main function
main "$@"