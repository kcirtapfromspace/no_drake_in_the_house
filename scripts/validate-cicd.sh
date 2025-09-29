#!/bin/bash

# Validate CI/CD Pipeline Setup
# This script checks that all CI/CD components are properly configured

set -e

echo "🔍 Validating CI/CD Pipeline Setup..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
        exit 1
    fi
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_info() {
    echo -e "ℹ️  $1"
}

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${RED}❌ Not in a git repository${NC}"
    exit 1
fi

print_status 0 "Git repository detected"

# Check GitHub Actions workflows
echo ""
print_info "Checking GitHub Actions workflows..."

WORKFLOWS_DIR=".github/workflows"
if [ ! -d "$WORKFLOWS_DIR" ]; then
    print_status 1 "GitHub Actions workflows directory not found"
fi

# Check required workflow files
REQUIRED_WORKFLOWS=(
    "ci-cd.yml"
    "pr-validation.yml"
    "release.yml"
    "maintenance.yml"
)

for workflow in "${REQUIRED_WORKFLOWS[@]}"; do
    if [ -f "$WORKFLOWS_DIR/$workflow" ]; then
        print_status 0 "Workflow $workflow exists"
    else
        print_status 1 "Workflow $workflow missing"
    fi
done

# Validate workflow syntax
echo ""
print_info "Validating workflow syntax..."

for workflow_file in "$WORKFLOWS_DIR"/*.yml; do
    if [ -f "$workflow_file" ]; then
        # Basic YAML syntax check
        if command -v yq > /dev/null 2>&1; then
            if yq eval '.' "$workflow_file" > /dev/null 2>&1; then
                print_status 0 "$(basename "$workflow_file") syntax is valid"
            else
                print_status 1 "$(basename "$workflow_file") has invalid YAML syntax"
            fi
        else
            print_warning "yq not installed, skipping YAML validation"
        fi
    fi
done

# Check Dockerfiles
echo ""
print_info "Checking Dockerfiles..."

DOCKERFILES=(
    "backend/Dockerfile"
    "frontend/Dockerfile"
)

for dockerfile in "${DOCKERFILES[@]}"; do
    if [ -f "$dockerfile" ]; then
        print_status 0 "Dockerfile $dockerfile exists"
        
        # Basic Dockerfile validation
        if grep -q "FROM" "$dockerfile" && grep -q "COPY\|ADD" "$dockerfile"; then
            print_status 0 "Dockerfile $dockerfile has basic structure"
        else
            print_status 1 "Dockerfile $dockerfile missing basic structure"
        fi
    else
        print_status 1 "Dockerfile $dockerfile missing"
    fi
done

# Check Helm charts
echo ""
print_info "Checking Helm charts..."

HELM_DIR="helm"
if [ ! -d "$HELM_DIR" ]; then
    print_status 1 "Helm charts directory not found"
fi

# Check Chart.yaml
if [ -f "$HELM_DIR/Chart.yaml" ]; then
    print_status 0 "Chart.yaml exists"
else
    print_status 1 "Chart.yaml missing"
fi

# Check values files
VALUES_FILES=(
    "values.yaml"
    "values-staging.yaml"
    "values-production.yaml"
)

for values_file in "${VALUES_FILES[@]}"; do
    if [ -f "$HELM_DIR/$values_file" ]; then
        print_status 0 "Values file $values_file exists"
    else
        print_status 1 "Values file $values_file missing"
    fi
done

# Check if Helm is installed and validate charts
if command -v helm > /dev/null 2>&1; then
    print_info "Validating Helm charts..."
    
    if helm lint "$HELM_DIR" > /dev/null 2>&1; then
        print_status 0 "Helm chart validation passed"
    else
        print_status 1 "Helm chart validation failed"
    fi
else
    print_warning "Helm not installed, skipping chart validation"
fi

# Check environment files
echo ""
print_info "Checking environment configuration..."

ENV_FILES=(
    "backend/.env.example"
    "frontend/.env.example"
)

for env_file in "${ENV_FILES[@]}"; do
    if [ -f "$env_file" ]; then
        print_status 0 "Environment file $env_file exists"
    else
        print_status 1 "Environment file $env_file missing"
    fi
done

# Check scripts
echo ""
print_info "Checking deployment scripts..."

SCRIPTS=(
    "scripts/validate-cicd.sh"
)

for script in "${SCRIPTS[@]}"; do
    if [ -f "$script" ]; then
        print_status 0 "Script $script exists"
        
        # Check if script is executable
        if [ -x "$script" ]; then
            print_status 0 "Script $script is executable"
        else
            print_warning "Script $script is not executable (run: chmod +x $script)"
        fi
    else
        print_status 1 "Script $script missing"
    fi
done

# Check for required secrets documentation
echo ""
print_info "Checking secrets documentation..."

if [ -f "README.md" ]; then
    if grep -q -i "secrets\|environment" "README.md"; then
        print_status 0 "README.md contains secrets/environment documentation"
    else
        print_warning "README.md should document required secrets and environment variables"
    fi
else
    print_warning "README.md not found"
fi

# Check GitHub repository settings (if we can access them)
echo ""
print_info "Repository configuration checks..."

# Check if we're on GitHub
REMOTE_URL=$(git config --get remote.origin.url 2>/dev/null || echo "")
if [[ "$REMOTE_URL" == *"github.com"* ]]; then
    print_status 0 "GitHub repository detected"
    
    # Extract repository info
    if [[ "$REMOTE_URL" =~ github\.com[:/]([^/]+)/([^/]+)(\.git)?$ ]]; then
        REPO_OWNER="${BASH_REMATCH[1]}"
        REPO_NAME="${BASH_REMATCH[2]}"
        print_info "Repository: $REPO_OWNER/$REPO_NAME"
    fi
else
    print_warning "Not a GitHub repository, some features may not work"
fi

# Check branch protection (requires GitHub CLI)
if command -v gh > /dev/null 2>&1; then
    print_info "Checking branch protection..."
    
    if gh api repos/:owner/:repo/branches/main/protection > /dev/null 2>&1; then
        print_status 0 "Main branch protection is enabled"
    else
        print_warning "Main branch protection not configured"
    fi
else
    print_warning "GitHub CLI not installed, skipping branch protection check"
fi

# Summary
echo ""
echo "🎉 CI/CD Pipeline validation completed!"
echo ""
print_info "Next steps:"
echo "  1. Configure repository secrets in GitHub:"
echo "     - KUBE_CONFIG_STAGING (base64 encoded kubeconfig)"
echo "     - KUBE_CONFIG_PRODUCTION (base64 encoded kubeconfig)"
echo "  2. Enable branch protection for main branch"
echo "  3. Configure environments in GitHub (staging, production)"
echo "  4. Test the pipeline by creating a pull request"
echo ""
print_info "To test locally:"
echo "  - Run: act -j test-backend (requires act CLI)"
echo "  - Run: docker build -t test ./backend"
echo "  - Run: docker build -t test ./frontend"