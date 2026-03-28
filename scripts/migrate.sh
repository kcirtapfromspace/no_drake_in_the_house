#!/bin/bash
set -euo pipefail

# Database migration script for production
# Usage: ./scripts/migrate.sh [environment]

ENVIRONMENT=${1:-production}
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Load environment-specific configuration
case $ENVIRONMENT in
    "production")
        NAMESPACE="ndith-production"
        ;;
    "staging")
        NAMESPACE="ndith-staging"
        ;;
    *)
        echo "Error: Unknown environment '$ENVIRONMENT'"
        echo "Usage: $0 [production|staging]"
        exit 1
        ;;
esac

echo "🚀 Starting database migration for $ENVIRONMENT environment..."

# Check if kubectl is available and configured
if ! command -v kubectl &> /dev/null; then
    echo "Error: kubectl is not installed or not in PATH"
    exit 1
fi

# Check if we can access the cluster
if ! kubectl cluster-info &> /dev/null; then
    echo "Error: Cannot connect to Kubernetes cluster"
    exit 1
fi

# Check if namespace exists
if ! kubectl get namespace "$NAMESPACE" &> /dev/null; then
    echo "Error: Namespace '$NAMESPACE' does not exist"
    exit 1
fi

# Get database connection details from secrets
echo "📋 Retrieving database connection details..."
DATABASE_URL=$(kubectl get secret ndith-secrets -n "$NAMESPACE" -o jsonpath='{.data.DATABASE_URL}' | base64 -d)

if [ -z "$DATABASE_URL" ]; then
    echo "Error: Could not retrieve DATABASE_URL from secrets"
    exit 1
fi

# Create a temporary migration pod
echo "🔧 Creating migration job..."
cat <<EOF | kubectl apply -f -
apiVersion: batch/v1
kind: Job
metadata:
  name: ndith-migration-$(date +%s)
  namespace: $NAMESPACE
  labels:
    app: ndith-migration
spec:
  ttlSecondsAfterFinished: 300
  template:
    spec:
      restartPolicy: Never
      containers:
      - name: migration
        image: ghcr.io/ndith/api:latest
        command: ["sqlx", "migrate", "run"]
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: ndith-secrets
              key: DATABASE_URL
        volumeMounts:
        - name: migrations
          mountPath: /app/migrations
      volumes:
      - name: migrations
        configMap:
          name: ndith-migrations
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 2000
EOF

# Wait for migration to complete
echo "⏳ Waiting for migration to complete..."
JOB_NAME=$(kubectl get jobs -n "$NAMESPACE" -l app=ndith-migration --sort-by=.metadata.creationTimestamp -o jsonpath='{.items[-1].metadata.name}')

if ! kubectl wait --for=condition=complete job/"$JOB_NAME" -n "$NAMESPACE" --timeout=300s; then
    echo "❌ Migration failed or timed out"
    echo "📋 Job logs:"
    kubectl logs job/"$JOB_NAME" -n "$NAMESPACE"
    exit 1
fi

echo "✅ Database migration completed successfully!"

# Verify migration status
echo "🔍 Verifying migration status..."
kubectl logs job/"$JOB_NAME" -n "$NAMESPACE" | tail -10

echo "🎉 Migration process completed for $ENVIRONMENT environment"