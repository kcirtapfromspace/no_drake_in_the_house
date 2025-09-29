#!/bin/bash

# Tilt validation script
# This script validates that the Tiltfile is properly configured

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

# Validate Tiltfile syntax
validate_tiltfile() {
    print_status "Validating Tiltfile syntax..."
    
    if [[ ! -f "Tiltfile" ]]; then
        print_error "Tiltfile not found in current directory"
        return 1
    fi
    
    # Check Tiltfile syntax with dry run
    if tilt up --dry-run >/dev/null 2>&1; then
        print_success "Tiltfile syntax is valid"
    else
        print_error "Tiltfile syntax validation failed"
        echo "Run 'tilt up' for detailed error information"
        return 1
    fi
}

# Check required files
check_required_files() {
    print_status "Checking required files..."
    
    local required_files=(
        "backend/Dockerfile.dev"
        "frontend/Dockerfile.dev"
        "helm/Chart.yaml"
        "helm/values-dev.yaml"
    )
    
    local missing_files=()
    
    for file in "${required_files[@]}"; do
        if [[ ! -f "$file" ]]; then
            missing_files+=("$file")
        fi
    done
    
    if [ ${#missing_files[@]} -ne 0 ]; then
        print_error "Missing required files:"
        for file in "${missing_files[@]}"; do
            echo "  - $file"
        done
        return 1
    fi
    
    print_success "All required files are present"
}

# Check Dockerfile syntax
check_dockerfiles() {
    print_status "Checking Dockerfile syntax..."
    
    local dockerfiles=(
        "backend/Dockerfile.dev"
        "frontend/Dockerfile.dev"
    )
    
    for dockerfile in "${dockerfiles[@]}"; do
        if docker build --dry-run -f "$dockerfile" "$(dirname "$dockerfile")" >/dev/null 2>&1; then
            print_success "$dockerfile syntax is valid"
        else
            print_warning "$dockerfile may have syntax issues"
        fi
    done
}

# Check Helm chart
check_helm_chart() {
    print_status "Checking Helm chart..."
    
    if helm lint ./helm >/dev/null 2>&1; then
        print_success "Helm chart is valid"
    else
        print_warning "Helm chart has linting issues"
        echo "Run 'helm lint ./helm' for details"
    fi
    
    # Check if we can template the chart
    if helm template kiro ./helm --values ./helm/values-dev.yaml >/dev/null 2>&1; then
        print_success "Helm chart templates successfully"
    else
        print_error "Helm chart templating failed"
        echo "Run 'helm template kiro ./helm --values ./helm/values-dev.yaml' for details"
        return 1
    fi
}

# Check Kubernetes connectivity
check_kubernetes() {
    print_status "Checking Kubernetes connectivity..."
    
    if kubectl cluster-info >/dev/null 2>&1; then
        local context=$(kubectl config current-context)
        print_success "Connected to Kubernetes cluster: $context"
    else
        print_error "Cannot connect to Kubernetes cluster"
        echo "Please ensure you have a Kubernetes cluster running"
        return 1
    fi
}

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    local tools=(
        "tilt:Tilt"
        "kubectl:Kubernetes CLI"
        "helm:Helm"
        "docker:Docker"
    )
    
    local missing_tools=()
    
    for tool_info in "${tools[@]}"; do
        tool="${tool_info%%:*}"
        description="${tool_info##*:}"
        
        if command_exists "$tool"; then
            print_success "$description is installed"
        else
            missing_tools+=("$tool ($description)")
        fi
    done
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        print_error "Missing required tools:"
        for tool in "${missing_tools[@]}"; do
            echo "  - $tool"
        done
        return 1
    fi
}

# Main validation function
main() {
    echo "🔍 Tilt Configuration Validation"
    echo ""
    
    local validation_failed=false
    
    # Run all validations
    check_prerequisites || validation_failed=true
    echo ""
    
    validate_tiltfile || validation_failed=true
    echo ""
    
    check_required_files || validation_failed=true
    echo ""
    
    check_dockerfiles || validation_failed=true
    echo ""
    
    check_helm_chart || validation_failed=true
    echo ""
    
    check_kubernetes || validation_failed=true
    echo ""
    
    # Final result
    if [ "$validation_failed" = true ]; then
        print_error "Tilt validation failed"
        echo ""
        echo "Please fix the issues above before running Tilt"
        exit 1
    else
        print_success "All validations passed!"
        echo ""
        echo "🚀 Your Tilt configuration is ready to use"
        echo "   Run 'make tilt-up' or 'tilt up' to start development"
    fi
}

# Run main function
main "$@"