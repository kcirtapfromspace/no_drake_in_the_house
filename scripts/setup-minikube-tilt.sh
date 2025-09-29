#!/bin/bash

# Setup script for Tilt + Minikube development
# This script ensures minikube is configured properly for Tilt development

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

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    local missing_tools=()
    
    if ! command_exists minikube; then
        missing_tools+=("minikube")
    fi
    
    if ! command_exists kubectl; then
        missing_tools+=("kubectl")
    fi
    
    if ! command_exists tilt; then
        missing_tools+=("tilt")
    fi
    
    if ! command_exists docker; then
        missing_tools+=("docker")
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        print_error "Missing required tools: ${missing_tools[*]}"
        echo ""
        echo "Installation instructions:"
        echo "  minikube: https://minikube.sigs.k8s.io/docs/start/"
        echo "  kubectl: https://kubernetes.io/docs/tasks/tools/"
        echo "  tilt: https://docs.tilt.dev/install.html"
        echo "  docker: https://docs.docker.com/get-docker/"
        exit 1
    fi
    
    print_success "All prerequisites are installed"
}

# Setup minikube
setup_minikube() {
    print_status "Setting up minikube..."
    
    # Check if minikube is running
    if minikube status >/dev/null 2>&1; then
        print_success "Minikube is already running"
    else
        print_status "Starting minikube..."
        minikube start --driver=docker --memory=4096 --cpus=2
        print_success "Minikube started"
    fi
    
    # Configure Docker environment to use minikube's Docker daemon
    print_status "Configuring Docker environment for minikube..."
    eval $(minikube docker-env)
    
    # Verify minikube is accessible
    if kubectl cluster-info >/dev/null 2>&1; then
        print_success "Minikube cluster is accessible"
    else
        print_error "Cannot access minikube cluster"
        exit 1
    fi
}

# Setup namespace
setup_namespace() {
    print_status "Setting up kiro-dev namespace..."
    
    if kubectl get namespace kiro-dev >/dev/null 2>&1; then
        print_success "Namespace kiro-dev already exists"
    else
        kubectl create namespace kiro-dev
        print_success "Created namespace kiro-dev"
    fi
}

# Verify Tilt configuration
verify_tilt() {
    print_status "Verifying Tilt configuration..."
    
    if [[ ! -f "Tiltfile" ]]; then
        print_error "Tiltfile not found in current directory"
        exit 1
    fi
    
    print_success "Tiltfile found"
}

# Main function
main() {
    echo "🎯 Setting up Minikube + Tilt Development Environment"
    echo ""
    
    # Check if we're in the project root
    if [[ ! -f "Tiltfile" ]] || [[ ! -d "backend" ]] || [[ ! -d "frontend" ]]; then
        print_error "Please run this script from the project root directory"
        exit 1
    fi
    
    check_prerequisites
    setup_minikube
    setup_namespace
    verify_tilt
    
    echo ""
    print_success "Setup complete! 🎉"
    echo ""
    echo "🔥 Recommended: Warm the Docker cache for faster builds"
    echo "  Run: make tilt-warm-cache (takes 2-3 minutes, saves 5+ minutes later)"
    echo ""
    echo "🚀 Next steps:"
    echo "  1. Run 'make tilt-warm-cache' for faster builds (recommended)"
    echo "  2. Run 'tilt up' to start the development environment"
    echo "  3. Open the Tilt UI in your browser (should open automatically)"
    echo "  4. Wait for all services to be ready (green in Tilt UI)"
    echo "  5. Run the 'db-migrate' trigger to set up the database"
    echo "  6. Start coding! Changes will auto-reload in seconds"
    echo ""
    echo "📡 Services will be available at:"
    echo "  - Backend: http://localhost:3000"
    echo "  - Frontend: http://localhost:5000"
    echo "  - PostgreSQL: localhost:5432"
    echo "  - Redis: localhost:6379"
    echo ""
    echo "🛠️  Useful commands:"
    echo "  tilt up          # Start development environment"
    echo "  tilt down        # Stop and clean up"
    echo "  minikube status  # Check minikube status"
    echo "  kubectl get pods -n kiro-dev  # Check pod status"
}

# Run main function
main "$@"