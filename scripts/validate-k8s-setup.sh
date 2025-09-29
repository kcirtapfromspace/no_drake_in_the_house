#!/bin/bash

# Kubernetes Development Environment Validation Script
# This script validates that all required tools and configurations are in place

set -e

echo "🔍 Validating Kubernetes Development Environment Setup"
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to check if command exists
check_command() {
    if command -v "$1" >/dev/null 2>&1; then
        echo -e "✅ ${GREEN}$1${NC} is installed"
        return 0
    else
        echo -e "❌ ${RED}$1${NC} is not installed"
        return 1
    fi
}

# Function to check version
check_version() {
    local cmd=$1
    local version_flag=${2:-"--version"}
    echo -e "📋 ${YELLOW}$cmd version:${NC}"
    $cmd $version_flag 2>/dev/null | head -1 || echo "  Could not get version"
    echo
}

# Check required tools
echo "🛠️  Checking required tools..."
echo

MISSING_TOOLS=0

if ! check_command "docker"; then
    echo "   Install: https://docs.docker.com/get-docker/"
    MISSING_TOOLS=$((MISSING_TOOLS + 1))
else
    check_version "docker" "--version"
fi

if ! check_command "kubectl"; then
    echo "   Install: brew install kubectl"
    MISSING_TOOLS=$((MISSING_TOOLS + 1))
else
    check_version "kubectl" "version --client --short"
fi

if ! check_command "helm"; then
    echo "   Install: brew install helm"
    MISSING_TOOLS=$((MISSING_TOOLS + 1))
else
    check_version "helm" "version --short"
fi

if ! check_command "skaffold"; then
    echo "   Install: brew install skaffold"
    MISSING_TOOLS=$((MISSING_TOOLS + 1))
else
    check_version "skaffold" "version"
fi

# Optional tools
echo "🔧 Checking optional tools..."
echo

if check_command "tilt"; then
    check_version "tilt" "version"
else
    echo "   Install (optional): brew install tilt"
fi

if check_command "minikube"; then
    check_version "minikube" "version --short"
fi

echo

# Check Kubernetes cluster
echo "☸️  Checking Kubernetes cluster..."
echo

if kubectl cluster-info >/dev/null 2>&1; then
    echo -e "✅ ${GREEN}Kubernetes cluster${NC} is accessible"
    echo -e "📋 ${YELLOW}Cluster info:${NC}"
    kubectl cluster-info | head -2
    echo
    
    # Check cluster nodes
    echo -e "📋 ${YELLOW}Cluster nodes:${NC}"
    kubectl get nodes --no-headers 2>/dev/null | wc -l | xargs echo "  Nodes:"
    
    # Check if we can create resources
    if kubectl auth can-i create pods >/dev/null 2>&1; then
        echo -e "✅ ${GREEN}Pod creation${NC} is allowed"
    else
        echo -e "❌ ${RED}Pod creation${NC} is not allowed"
        MISSING_TOOLS=$((MISSING_TOOLS + 1))
    fi
    
    if kubectl auth can-i create services >/dev/null 2>&1; then
        echo -e "✅ ${GREEN}Service creation${NC} is allowed"
    else
        echo -e "❌ ${RED}Service creation${NC} is not allowed"
        MISSING_TOOLS=$((MISSING_TOOLS + 1))
    fi
    
else
    echo -e "❌ ${RED}Kubernetes cluster${NC} is not accessible"
    echo "   Make sure Docker Desktop has Kubernetes enabled or minikube is running"
    MISSING_TOOLS=$((MISSING_TOOLS + 1))
fi

echo

# Check Docker daemon
echo "🐳 Checking Docker daemon..."
echo

if docker info >/dev/null 2>&1; then
    echo -e "✅ ${GREEN}Docker daemon${NC} is running"
    
    # Check if we can build images
    if docker images >/dev/null 2>&1; then
        echo -e "✅ ${GREEN}Docker image operations${NC} are working"
    else
        echo -e "❌ ${RED}Docker image operations${NC} are not working"
        MISSING_TOOLS=$((MISSING_TOOLS + 1))
    fi
else
    echo -e "❌ ${RED}Docker daemon${NC} is not running"
    echo "   Start Docker Desktop or docker daemon"
    MISSING_TOOLS=$((MISSING_TOOLS + 1))
fi

echo

# Check Helm repositories
echo "📦 Checking Helm repositories..."
echo

if helm repo list 2>/dev/null | grep -q bitnami; then
    echo -e "✅ ${GREEN}Bitnami Helm repository${NC} is configured"
else
    echo -e "⚠️  ${YELLOW}Bitnami Helm repository${NC} is not configured"
    echo "   Run: helm repo add bitnami https://charts.bitnami.com/bitnami"
fi

echo

# Check project files
echo "📁 Checking project files..."
echo

PROJECT_FILES=(
    "helm/Chart.yaml"
    "helm/values.yaml"
    "helm/values-dev.yaml"
    "skaffold.yaml"
    "Tiltfile"
    "backend/Dockerfile.dev"
    "frontend/Dockerfile.dev"
)

for file in "${PROJECT_FILES[@]}"; do
    if [[ -f "$file" ]]; then
        echo -e "✅ ${GREEN}$file${NC} exists"
    else
        echo -e "❌ ${RED}$file${NC} is missing"
        MISSING_TOOLS=$((MISSING_TOOLS + 1))
    fi
done

echo

# Check environment files
echo "⚙️  Checking environment configuration..."
echo

ENV_FILES=(
    "backend/.env"
    "frontend/.env"
)

for file in "${ENV_FILES[@]}"; do
    if [[ -f "$file" ]]; then
        echo -e "✅ ${GREEN}$file${NC} exists"
    else
        echo -e "⚠️  ${YELLOW}$file${NC} is missing (will be created from .env.example)"
    fi
done

echo

# Summary
echo "📊 Validation Summary"
echo "===================="

if [[ $MISSING_TOOLS -eq 0 ]]; then
    echo -e "🎉 ${GREEN}All checks passed!${NC} Your Kubernetes development environment is ready."
    echo
    echo "🚀 Next steps:"
    echo "   1. Run 'make k8s-dev' to start the development environment"
    echo "   2. Or run 'tilt up' for the alternative UI-based workflow"
    echo "   3. Access services at:"
    echo "      - Backend: http://localhost:3000"
    echo "      - Frontend: http://localhost:5000"
    echo
    exit 0
else
    echo -e "⚠️  ${YELLOW}$MISSING_TOOLS issue(s) found.${NC} Please fix the issues above before proceeding."
    echo
    echo "🔧 Common fixes:"
    echo "   - Install missing tools using brew or package manager"
    echo "   - Start Docker Desktop and enable Kubernetes"
    echo "   - Run 'make setup' to create environment files"
    echo
    exit 1
fi