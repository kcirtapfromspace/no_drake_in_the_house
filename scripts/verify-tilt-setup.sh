#!/bin/bash

# Verification script for Tilt/Minikube registration flow setup
# This script checks that all services are properly configured and accessible

set -e

echo "🔍 Verifying Tilt/Minikube Registration Flow Setup"
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to check if a service is accessible
check_service() {
    local name=$1
    local url=$2
    local expected_status=${3:-200}
    
    echo -n "Checking $name ($url)... "
    
    if curl -s -o /dev/null -w "%{http_code}" "$url" | grep -q "$expected_status"; then
        echo -e "${GREEN}✓ OK${NC}"
        return 0
    else
        echo -e "${RED}✗ FAILED${NC}"
        return 1
    fi
}

# Function to check Kubernetes resources
check_k8s_resource() {
    local resource_type=$1
    local resource_name=$2
    local namespace=${3:-kiro-dev}
    
    echo -n "Checking $resource_type/$resource_name in namespace $namespace... "
    
    if kubectl get "$resource_type" "$resource_name" -n "$namespace" >/dev/null 2>&1; then
        local status=$(kubectl get "$resource_type" "$resource_name" -n "$namespace" -o jsonpath='{.status.readyReplicas}' 2>/dev/null || echo "0")
        if [ "$status" -gt 0 ] 2>/dev/null; then
            echo -e "${GREEN}✓ READY${NC}"
            return 0
        else
            echo -e "${YELLOW}⚠ NOT READY${NC}"
            return 1
        fi
    else
        echo -e "${RED}✗ NOT FOUND${NC}"
        return 1
    fi
}

# Check prerequisites
echo -e "\n${YELLOW}1. Checking Prerequisites${NC}"
echo "------------------------"

# Check if kubectl is available
if command -v kubectl >/dev/null 2>&1; then
    echo -e "kubectl: ${GREEN}✓ Available${NC}"
else
    echo -e "kubectl: ${RED}✗ Not found${NC}"
    exit 1
fi

# Check if minikube is running
if minikube status >/dev/null 2>&1; then
    echo -e "minikube: ${GREEN}✓ Running${NC}"
else
    echo -e "minikube: ${RED}✗ Not running${NC}"
    echo "Please start minikube with: minikube start"
    exit 1
fi

# Check current kubectl context
current_context=$(kubectl config current-context)
echo "kubectl context: $current_context"

# Check Kubernetes resources
echo -e "\n${YELLOW}2. Checking Kubernetes Resources${NC}"
echo "--------------------------------"

check_k8s_resource "deployment" "postgres"
check_k8s_resource "deployment" "redis"
check_k8s_resource "deployment" "backend"
check_k8s_resource "deployment" "frontend"

check_k8s_resource "service" "postgres"
check_k8s_resource "service" "redis"
check_k8s_resource "service" "backend"
check_k8s_resource "service" "frontend"

# Check service accessibility
echo -e "\n${YELLOW}3. Checking Service Accessibility${NC}"
echo "---------------------------------"

# Wait a moment for services to be ready
sleep 2

check_service "Backend Health" "http://localhost:3000/health"
check_service "Frontend" "http://localhost:5000" "200"

# Check database connectivity
echo -n "Checking PostgreSQL connectivity... "
if kubectl exec -n kiro-dev deployment/postgres -- pg_isready -U kiro -d kiro >/dev/null 2>&1; then
    echo -e "${GREEN}✓ OK${NC}"
else
    echo -e "${RED}✗ FAILED${NC}"
fi

# Check Redis connectivity
echo -n "Checking Redis connectivity... "
if kubectl exec -n kiro-dev deployment/redis -- redis-cli ping >/dev/null 2>&1; then
    echo -e "${GREEN}✓ OK${NC}"
else
    echo -e "${RED}✗ FAILED${NC}"
fi

# Check API endpoint specifically
echo -e "\n${YELLOW}4. Checking Registration API${NC}"
echo "----------------------------"

# Test the registration endpoint (should return 400 for empty request)
echo -n "Testing registration endpoint... "
response_code=$(curl -s -o /dev/null -w "%{http_code}" -X POST \
    -H "Content-Type: application/json" \
    -d '{}' \
    "http://localhost:3000/api/v1/auth/register")

if [ "$response_code" = "400" ]; then
    echo -e "${GREEN}✓ OK (returns 400 for invalid request as expected)${NC}"
elif [ "$response_code" = "422" ]; then
    echo -e "${GREEN}✓ OK (returns 422 for validation error as expected)${NC}"
else
    echo -e "${RED}✗ FAILED (returned $response_code)${NC}"
fi

# Check frontend configuration
echo -e "\n${YELLOW}5. Checking Frontend Configuration${NC}"
echo "----------------------------------"

# Get the frontend pod's environment variables
echo -n "Checking VITE_API_URL in frontend pod... "
api_url=$(kubectl exec -n kiro-dev deployment/frontend -- printenv VITE_API_URL 2>/dev/null || echo "NOT_SET")

if [ "$api_url" = "http://localhost:3000" ]; then
    echo -e "${GREEN}✓ OK ($api_url)${NC}"
elif [ "$api_url" = "http://backend:3000" ]; then
    echo -e "${RED}✗ INCORRECT ($api_url)${NC}"
    echo -e "  ${YELLOW}Fix: Update k8s/dev-manifests.yaml frontend VITE_API_URL to http://localhost:3000${NC}"
else
    echo -e "${RED}✗ NOT SET or INCORRECT ($api_url)${NC}"
fi

# Check backend CORS configuration
echo -n "Checking CORS configuration in backend pod... "
cors_origins=$(kubectl exec -n kiro-dev deployment/backend -- printenv CORS_ALLOWED_ORIGINS 2>/dev/null || echo "NOT_SET")

if [[ "$cors_origins" == *"localhost:5000"* ]]; then
    echo -e "${GREEN}✓ OK (includes localhost:5000)${NC}"
else
    echo -e "${RED}✗ MISSING localhost:5000 ($cors_origins)${NC}"
    echo -e "  ${YELLOW}Fix: Update k8s/dev-manifests.yaml backend CORS_ALLOWED_ORIGINS${NC}"
fi

# Final summary
echo -e "\n${YELLOW}6. Summary${NC}"
echo "----------"

echo "If all checks pass, the registration flow should work correctly."
echo "If there are failures, please:"
echo "1. Fix the configuration issues mentioned above"
echo "2. Run 'tilt down && tilt up' to restart services"
echo "3. Wait for all pods to be ready"
echo "4. Run this script again to verify"

echo -e "\n${YELLOW}Quick Test:${NC}"
echo "1. Open http://localhost:5000 in your browser"
echo "2. Navigate to the registration form"
echo "3. Check browser network tab - requests should go to localhost:3000"
echo "4. Fill out the form and submit"
echo "5. Should see proper validation or successful registration"

echo -e "\n${GREEN}Setup verification complete!${NC}"