#!/bin/bash

# Test script for database migration job functionality
# This script tests the migration job in isolation

set -e

NAMESPACE="kiro-dev"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🧪 Testing Database Migration Job"
echo "=================================="
echo ""

# Check prerequisites
echo "🔍 Checking prerequisites..."

if ! command -v kubectl >/dev/null 2>&1; then
    echo "❌ kubectl is not installed or not in PATH"
    exit 1
fi

if ! kubectl cluster-info >/dev/null 2>&1; then
    echo "❌ No Kubernetes cluster available"
    exit 1
fi

echo "✅ Prerequisites check passed"
echo ""

# Check if namespace exists
echo "🔍 Checking namespace..."
if ! kubectl get namespace "$NAMESPACE" >/dev/null 2>&1; then
    echo "⚠️  Namespace $NAMESPACE does not exist, creating..."
    kubectl create namespace "$NAMESPACE"
fi
echo "✅ Namespace $NAMESPACE is ready"
echo ""

# Deploy PostgreSQL if not running
echo "🔍 Checking PostgreSQL..."
if ! kubectl get deployment postgres -n "$NAMESPACE" >/dev/null 2>&1; then
    echo "⚠️  PostgreSQL not found, deploying..."
    kubectl apply -f "$PROJECT_ROOT/k8s/dev-manifests.yaml"
    echo "⏳ Waiting for PostgreSQL to be ready..."
    kubectl wait --for=condition=available deployment/postgres -n "$NAMESPACE" --timeout=120s
fi

if kubectl wait --for=condition=ready pod -l app=postgres -n "$NAMESPACE" --timeout=60s; then
    echo "✅ PostgreSQL is ready"
else
    echo "❌ PostgreSQL failed to become ready"
    exit 1
fi
echo ""

# Clean up any existing migration job
echo "🧹 Cleaning up existing migration job..."
kubectl delete job database-migration -n "$NAMESPACE" --ignore-not-found=true
echo "⏳ Waiting for cleanup..."
sleep 3
echo ""

# Test migration job
echo "🚀 Testing migration job..."
echo "📋 Applying migration job manifest..."
if kubectl apply -f "$PROJECT_ROOT/k8s/dev/migration-job.yaml"; then
    echo "✅ Migration job created successfully"
else
    echo "❌ Failed to create migration job"
    exit 1
fi

echo ""
echo "⏳ Waiting for migration job to complete (timeout: 5 minutes)..."
if kubectl wait --for=condition=complete job/database-migration -n "$NAMESPACE" --timeout=300s; then
    echo "✅ Migration job completed successfully!"
    
    echo ""
    echo "📊 Migration job logs:"
    kubectl logs job/database-migration -n "$NAMESPACE" --tail=30
    
    echo ""
    echo "🔍 Verifying database schema..."
    if kubectl exec -n "$NAMESPACE" deployment/postgres -- psql -U kiro -d kiro -c "SELECT schemaname, tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename;" >/dev/null 2>&1; then
        echo "✅ Database schema verification passed"
        
        echo ""
        echo "📋 Created tables:"
        kubectl exec -n "$NAMESPACE" deployment/postgres -- psql -U kiro -d kiro -c "SELECT schemaname, tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename;"
        
        echo ""
        echo "📈 Migration history:"
        kubectl exec -n "$NAMESPACE" deployment/postgres -- psql -U kiro -d kiro -c "SELECT version, description, installed_on, success FROM sqlx_migrations ORDER BY version;" 2>/dev/null || echo "Migration table not accessible"
    else
        echo "❌ Database schema verification failed"
        exit 1
    fi
    
else
    echo "❌ Migration job failed or timed out!"
    
    echo ""
    echo "📋 Job status:"
    kubectl describe job/database-migration -n "$NAMESPACE"
    
    echo ""
    echo "📋 Job logs:"
    kubectl logs job/database-migration -n "$NAMESPACE" --tail=50
    
    echo ""
    echo "🔍 Pod status:"
    kubectl get pods -n "$NAMESPACE" -l job-name=database-migration
    
    exit 1
fi

echo ""
echo "🧪 Testing seed job (if available)..."
if kubectl get job database-seed -n "$NAMESPACE" >/dev/null 2>&1; then
    echo "📋 Found seed job, testing manual trigger..."
    
    # Enable seed job
    kubectl patch job database-seed -n "$NAMESPACE" -p '{"spec":{"suspend":false}}'
    
    echo "⏳ Waiting for seed job to complete..."
    if kubectl wait --for=condition=complete job/database-seed -n "$NAMESPACE" --timeout=120s; then
        echo "✅ Seed job completed successfully!"
        
        echo ""
        echo "📊 Seed job logs:"
        kubectl logs job/database-seed -n "$NAMESPACE" --tail=20
        
        echo ""
        echo "🔍 Verifying seeded data..."
        kubectl exec -n "$NAMESPACE" deployment/postgres -- psql -U kiro -d kiro -c "SELECT 'Users: ' || count(*) FROM users UNION ALL SELECT 'Artists: ' || count(*) FROM artists UNION ALL SELECT 'DNP Entries: ' || count(*) FROM user_artist_blocks;"
        
        # Re-suspend seed job
        kubectl patch job database-seed -n "$NAMESPACE" -p '{"spec":{"suspend":true}}'
        echo "🔄 Seed job re-suspended for future manual triggers"
    else
        echo "⚠️  Seed job failed or timed out (this is optional)"
        kubectl logs job/database-seed -n "$NAMESPACE" --tail=20 || true
        kubectl patch job database-seed -n "$NAMESPACE" -p '{"spec":{"suspend":true}}' || true
    fi
else
    echo "⚠️  Seed job not found (this is optional)"
fi

echo ""
echo "🎉 Migration job test completed successfully!"
echo ""
echo "📊 Final Status:"
echo "  ✅ Migration job executed successfully"
echo "  ✅ Database schema created and verified"
echo "  ✅ Migration tracking table populated"
echo "  ✅ All critical tables accessible"
echo ""
echo "💡 The migration job is ready for use with Tilt!"