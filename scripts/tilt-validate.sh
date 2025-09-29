#!/bin/bash
# Tilt Configuration Validation Script
# Validates Tiltfile configuration and dependencies

set -e

echo "🔍 Tilt Configuration Validator"
echo "==============================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track validation results
VALIDATION_PASSED=true

# Function to check requirement
check_requirement() {
    local name=$1
    local command=$2
    local error_msg=$3
    
    echo -n "Checking $name... "
    
    if eval "$command" > /dev/null 2>&1; then
        echo -e "${GREEN}✅${NC}"
    else
        echo -e "${RED}❌${NC}"
        echo -e "  ${RED}Error: $error_msg${NC}"
        VALIDATION_PASSED=false
    fi
}

# Function to check file exists
check_file() {
    local file=$1
    local description=$2
    
    echo -n "Checking $description... "
    
    if [ -f "$file" ]; then
        echo -e "${GREEN}✅${NC}"
    else
        echo -e "${RED}❌${NC}"
        echo -e "  ${RED}Error: $file not found${NC}"
        VALIDATION_PASSED=false
    fi
}

echo ""
echo -e "${BLUE}🔧 Checking Prerequisites${NC}"
echo "-------------------------"

# Check required tools
check_requirement "Docker" "docker --version" "Docker is required for container builds"
check_requirement "kubectl" "kubectl version --client" "kubectl is required for Kubernetes operations"
check_requirement "Tilt" "tilt version" "Tilt is required for development workflow"

# Check optional tools
echo -n "Checking minikube... "
if command -v minikube &> /dev/null; then
    echo -e "${GREEN}✅${NC}"
    
    # Check if minikube is running
    echo -n "Checking minikube status... "
    if minikube status > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Running${NC}"
    else
        echo -e "${YELLOW}⚠️  Not running${NC}"
        echo -e "  ${YELLOW}Note: Start minikube with 'minikube start'${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Not found${NC}"
    echo -e "  ${YELLOW}Note: minikube recommended for local Kubernetes${NC}"
fi

echo ""
echo -e "${BLUE}📁 Checking Required Files${NC}"
echo "--------------------------"

# Check Tiltfile and related files
check_file "Tiltfile" "Tiltfile"
check_file "k8s/dev-manifests.yaml" "Kubernetes manifests"
check_file "backend/Dockerfile.fast" "Backend fast Dockerfile"
check_file "frontend/Dockerfile.fast" "Frontend fast Dockerfile"

# Check backend files
check_file "backend/Cargo.toml" "Backend Cargo.toml"
check_file "backend/src/main.rs" "Backend main.rs"

# Check frontend files
check_file "frontend/package.json" "Frontend package.json"
check_file "frontend/src/main.ts" "Frontend main.ts"

echo ""
echo -e "${BLUE}🔍 Validating Tiltfile Syntax${NC}"
echo "-----------------------------"

echo -n "Validating Tiltfile syntax... "
# Try to parse the Tiltfile by doing a dry run
if tilt ci --dry-run > /dev/null 2>&1; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${YELLOW}⚠️  Cannot validate syntax (requires Kubernetes cluster)${NC}"
    echo -e "  ${YELLOW}Note: Tiltfile syntax will be validated when 'tilt up' is run${NC}"
fi

echo ""
echo -e "${BLUE}🐳 Checking Docker Configuration${NC}"
echo "--------------------------------"

echo -n "Checking Docker daemon... "
if docker info > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Running${NC}"
    
    # Check BuildKit support
    echo -n "Checking BuildKit support... "
    if docker buildx version > /dev/null 2>&1; then
        echo -e "${GREEN}✅${NC}"
    else
        echo -e "${YELLOW}⚠️  Not available${NC}"
        echo -e "  ${YELLOW}Note: BuildKit provides better caching performance${NC}"
    fi
else
    echo -e "${RED}❌${NC}"
    echo -e "  ${RED}Error: Docker daemon not running${NC}"
    VALIDATION_PASSED=false
fi

echo ""
echo -e "${BLUE}☸️  Checking Kubernetes Configuration${NC}"
echo "------------------------------------"

echo -n "Checking kubectl context... "
if kubectl config current-context > /dev/null 2>&1; then
    current_context=$(kubectl config current-context)
    echo -e "${GREEN}✅ $current_context${NC}"
    
    # Check if we can connect to the cluster
    echo -n "Checking cluster connectivity... "
    if kubectl cluster-info > /dev/null 2>&1; then
        echo -e "${GREEN}✅${NC}"
    else
        echo -e "${RED}❌${NC}"
        echo -e "  ${RED}Error: Cannot connect to Kubernetes cluster${NC}"
        VALIDATION_PASSED=false
    fi
else
    echo -e "${RED}❌${NC}"
    echo -e "  ${RED}Error: No kubectl context configured${NC}"
    VALIDATION_PASSED=false
fi

echo ""
echo -e "${BLUE}🎯 Checking Tilt Extensions${NC}"
echo "---------------------------"

# Check if required Tilt extensions are available
echo -n "Checking restart_process extension... "
if tilt dump extensions 2>/dev/null | grep -q "restart_process" || echo 'load("ext://restart_process", "docker_build_with_restart")' | tilt validate --stdin > /dev/null 2>&1; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${YELLOW}⚠️  May not be available${NC}"
    echo -e "  ${YELLOW}Note: Extension will be downloaded automatically when needed${NC}"
fi

echo -n "Checking helm_resource extension... "
if tilt dump extensions 2>/dev/null | grep -q "helm_resource" || echo 'load("ext://helm_resource", "helm_resource")' | tilt validate --stdin > /dev/null 2>&1; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${YELLOW}⚠️  May not be available${NC}"
    echo -e "  ${YELLOW}Note: Extension will be downloaded automatically when needed${NC}"
fi

echo ""
echo "================================="

if [ "$VALIDATION_PASSED" = true ]; then
    echo -e "${GREEN}✅ All validations passed!${NC}"
    echo ""
    echo "🚀 Ready to start development with:"
    echo "   tilt up"
    echo ""
    echo "💡 Quick start tips:"
    echo "   • Run 'make warm-cache' for faster initial builds"
    echo "   • Use Tilt UI for real-time monitoring"
    echo "   • Check manual triggers for development workflow"
    exit 0
else
    echo -e "${RED}❌ Validation failed!${NC}"
    echo ""
    echo "🔧 Please fix the issues above before running Tilt"
    echo ""
    echo "📚 Common solutions:"
    echo "   • Install missing tools (Docker, kubectl, Tilt)"
    echo "   • Start Docker daemon"
    echo "   • Configure kubectl context"
    echo "   • Start minikube if using local Kubernetes"
    exit 1
fi