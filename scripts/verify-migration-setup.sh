#!/bin/bash

# Verification script for database migration job setup
# This script tests the migration job configuration without running the full Tilt environment

set -e

NAMESPACE="kiro-dev"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🔍 Verifying database migration job setup..."
echo "Project root: $PROJECT_ROOT"
echo "Namespace: $NAMESPACE"
echo ""

# Check if kubectl is available
if ! command -v kubectl &> /dev/null; then
    echo "❌ kubectl is not installed or not in PATH"
    exit 1
fi

# Check if we have a Kubernetes context
if ! kubectl config current-context &> /dev/null; then
    echo "❌ No Kubernetes context configured"
    exit 1
fi

echo "✓ kubectl is available"
echo "✓ Kubernetes context: $(kubectl config current-context)"
echo ""

# Check if namespace exists
if ! kubectl get namespace "$NAMESPACE" &> /dev/null; then
    echo "⚠ Namespace $NAMESPACE does not exist, creating it..."
    kubectl create namespace "$NAMESPACE"
    echo "✓ Namespace $NAMESPACE created"
else
    echo "✓ Namespace $NAMESPACE exists"
fi

# Validate migration job YAML
echo ""
echo "🔍 Validating migration job YAML..."
if kubectl apply --dry-run=client -f "$PROJECT_ROOT/k8s/dev/migration-job.yaml" &> /dev/null; then
    echo "✓ Migration job YAML is valid"
else
    echo "❌ Migration job YAML validation failed"
    kubectl apply --dry-run=client -f "$PROJECT_ROOT/k8s/dev/migration-job.yaml"
    exit 1
fi

# Check if migration files exist
echo ""
echo "🔍 Checking migration files..."
MIGRATION_DIR="$PROJECT_ROOT/backend/migrations"
if [ ! -d "$MIGRATION_DIR" ]; then
    echo "❌ Migration directory not found: $MIGRATION_DIR"
    exit 1
fi

migration_count=$(find "$MIGRATION_DIR" -name "*.sql" | wc -l)
if [ "$migration_count" -eq 0 ]; then
    echo "❌ No migration files found in $MIGRATION_DIR"
    exit 1
fi

echo "✓ Found $migration_count migration files:"
find "$MIGRATION_DIR" -name "*.sql" | sort | while read -r file; do
    echo "  - $(basename "$file")"
done

# Check if backend Dockerfile exists and includes migrations
echo ""
echo "🔍 Checking backend Docker configuration..."
BACKEND_DOCKERFILE="$PROJECT_ROOT/backend/Dockerfile.dev"
if [ ! -f "$BACKEND_DOCKERFILE" ]; then
    echo "❌ Backend Dockerfile not found: $BACKEND_DOCKERFILE"
    exit 1
fi

if grep -q "COPY migrations" "$BACKEND_DOCKERFILE"; then
    echo "✓ Backend Dockerfile includes migration files"
else
    echo "❌ Backend Dockerfile does not copy migration files"
    exit 1
fi

# Validate dev-manifests YAML
echo ""
echo "🔍 Validating dev-manifests YAML..."
if kubectl apply --dry-run=client -f "$PROJECT_ROOT/k8s/dev-manifests.yaml" &> /dev/null; then
    echo "✓ Dev-manifests YAML is valid"
else
    echo "❌ Dev-manifests YAML validation failed"
    kubectl apply --dry-run=client -f "$PROJECT_ROOT/k8s/dev-manifests.yaml"
    exit 1
fi

# Check Tiltfile syntax
echo ""
echo "🔍 Checking Tiltfile syntax..."
TILTFILE="$PROJECT_ROOT/Tiltfile"
if [ ! -f "$TILTFILE" ]; then
    echo "❌ Tiltfile not found: $TILTFILE"
    exit 1
fi

# Basic syntax check - look for common issues
if grep -q "k8s_yaml.*migration-job" "$TILTFILE"; then
    echo "✓ Tiltfile includes migration job"
else
    echo "❌ Tiltfile does not reference migration job"
    exit 1
fi

if grep -q "database-migration" "$TILTFILE"; then
    echo "✓ Tiltfile includes migration resource configuration"
else
    echo "❌ Tiltfile does not configure migration resource"
    exit 1
fi

echo ""
echo "🎉 Migration setup verification completed successfully!"
echo ""
echo "Next steps:"
echo "1. Run 'tilt up' to start the development environment"
echo "2. Watch for the database-migration job to complete"
echo "3. Verify backend starts after migration completion"
echo "4. Use 'db-seed' trigger to load test data if needed"
echo ""
echo "Troubleshooting commands:"
echo "  kubectl logs job/database-migration -n $NAMESPACE"
echo "  kubectl describe job database-migration -n $NAMESPACE"
echo "  kubectl get pods -n $NAMESPACE"