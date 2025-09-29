#!/bin/bash
set -euo pipefail

# Production deployment script
# Usage: ./scripts/deploy.sh [environment] [component] [image-tag]

ENVIRONMENT=${1:-production}
COMPONENT=${2:-all}
IMAGE_TAG=${3:-latest}
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Load environment-specific configuration
case $ENVIRONMENT in
    "production")
        NAMESPACE="kiro-production"
        REGISTRY="ghcr.io"
        ;;
    "staging")
        NAMESPACE="kiro-staging"
        REGISTRY="ghcr.io"
        ;;
    *)
        echo "Error: Unknown environment '$ENVIRONMENT'"
        echo "Usage: $0 [production|staging] [all|api|worker|frontend] [image-tag]"
        exit 1
        ;;
esac

echo "🚀 Starting deployment to $ENVIRONMENT environment..."
echo "📦 Component: $COMPONENT"
echo "🏷️  Image tag: $IMAGE_TAG"

# Check prerequisites
if ! command -v kubectl &> /dev/null; then
    echo "Error: kubectl is not installed or not in PATH"
    exit 1
fi

if ! command -v helm &> /dev/null; then
    echo "Warning: helm is not installed, skipping Helm operations"
fi

# Check cluster connectivity
if ! kubectl cluster-info &> /dev/null; then
    echo "Error: Cannot connect to Kubernetes cluster"
    exit 1
fi

# Check if namespace exists
if ! kubectl get namespace "$NAMESPACE" &> /dev/null; then
    echo "Error: Namespace '$NAMESPACE' does not exist"
    exit 1
fi

# Pre-deployment checks
echo "🔍 Running pre-deployment checks..."

# Check if required secrets exist
REQUIRED_SECRETS=("kiro-secrets" "grafana-secrets")
for secret in "${REQUIRED_SECRETS[@]}"; do
    if ! kubectl get secret "$secret" -n "$NAMESPACE" &> /dev/null; then
        echo "Error: Required secret '$secret' not found in namespace '$NAMESPACE'"
        exit 1
    fi
done

# Check resource quotas
echo "📊 Checking resource quotas..."
kubectl describe resourcequota kiro-production-quota -n "$NAMESPACE" || echo "Warning: Resource quota not found"

# Backup current deployment state
echo "💾 Creating deployment backup..."
BACKUP_DIR="/tmp/kiro-deployment-backup-$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"

if [ "$COMPONENT" = "all" ] || [ "$COMPONENT" = "api" ]; then
    kubectl get deployment kiro-api -n "$NAMESPACE" -o yaml > "$BACKUP_DIR/kiro-api-deployment.yaml" 2>/dev/null || true
fi

if [ "$COMPONENT" = "all" ] || [ "$COMPONENT" = "worker" ]; then
    kubectl get deployment kiro-worker -n "$NAMESPACE" -o yaml > "$BACKUP_DIR/kiro-worker-deployment.yaml" 2>/dev/null || true
fi

echo "📁 Backup saved to: $BACKUP_DIR"

# Function to deploy a component
deploy_component() {
    local comp=$1
    local image_name="${REGISTRY}/kiro/${comp}:${IMAGE_TAG}"
    
    echo "🔄 Deploying $comp with image: $image_name"
    
    # Update image in deployment
    if [ "$comp" = "api" ]; then
        kubectl set image deployment/kiro-api api="$image_name" -n "$NAMESPACE"
    elif [ "$comp" = "worker" ]; then
        kubectl set image deployment/kiro-worker worker="$image_name" -n "$NAMESPACE"
    fi
    
    # Wait for rollout to complete
    echo "⏳ Waiting for $comp rollout to complete..."
    if ! kubectl rollout status deployment/kiro-$comp -n "$NAMESPACE" --timeout=600s; then
        echo "❌ Rollout failed for $comp"
        echo "🔄 Rolling back..."
        kubectl rollout undo deployment/kiro-$comp -n "$NAMESPACE"
        kubectl rollout status deployment/kiro-$comp -n "$NAMESPACE" --timeout=300s
        return 1
    fi
    
    echo "✅ $comp deployed successfully"
}

# Run database migrations if deploying API
if [ "$COMPONENT" = "all" ] || [ "$COMPONENT" = "api" ]; then
    echo "🗄️  Running database migrations..."
    
    # Apply migration ConfigMap
    kubectl apply -f "$PROJECT_ROOT/k8s/database-migrations.yaml"
    
    # Run migration job
    cat <<EOF | kubectl apply -f -
apiVersion: batch/v1
kind: Job
metadata:
  name: kiro-migration-$(date +%s)
  namespace: $NAMESPACE
  labels:
    app: kiro-migration
spec:
  ttlSecondsAfterFinished: 300
  template:
    spec:
      restartPolicy: Never
      containers:
      - name: migration
        image: ${REGISTRY}/kiro/api:${IMAGE_TAG}
        command: ["sqlx", "migrate", "run"]
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: kiro-secrets
              key: DATABASE_URL
        volumeMounts:
        - name: migrations
          mountPath: /app/migrations
      volumes:
      - name: migrations
        configMap:
          name: kiro-migrations
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 2000
EOF
    
    # Wait for migration to complete
    MIGRATION_JOB=$(kubectl get jobs -n "$NAMESPACE" -l app=kiro-migration --sort-by=.metadata.creationTimestamp -o jsonpath='{.items[-1].metadata.name}')
    
    if ! kubectl wait --for=condition=complete job/"$MIGRATION_JOB" -n "$NAMESPACE" --timeout=300s; then
        echo "❌ Database migration failed"
        kubectl logs job/"$MIGRATION_JOB" -n "$NAMESPACE"
        exit 1
    fi
    
    echo "✅ Database migrations completed"
fi

# Deploy components
if [ "$COMPONENT" = "all" ]; then
    deploy_component "api"
    deploy_component "worker"
elif [ "$COMPONENT" = "api" ] || [ "$COMPONENT" = "worker" ]; then
    deploy_component "$COMPONENT"
else
    echo "Error: Unknown component '$COMPONENT'"
    echo "Usage: $0 [production|staging] [all|api|worker] [image-tag]"
    exit 1
fi

# Post-deployment verification
echo "🔍 Running post-deployment verification..."

# Health checks
if [ "$COMPONENT" = "all" ] || [ "$COMPONENT" = "api" ]; then
    echo "🏥 Checking API health..."
    kubectl wait --for=condition=ready pod -l app=kiro-api -n "$NAMESPACE" --timeout=120s
    
    # Get API URL and test endpoints
    if kubectl get ingress kiro-ingress -n "$NAMESPACE" &> /dev/null; then
        API_URL=$(kubectl get ingress kiro-ingress -n "$NAMESPACE" -o jsonpath='{.spec.rules[0].host}')
        echo "🌐 Testing API endpoints at https://$API_URL"
        
        # Test health endpoint
        if curl -f -s "https://$API_URL/health" > /dev/null; then
            echo "✅ Health endpoint responding"
        else
            echo "⚠️  Health endpoint not responding"
        fi
        
        # Test ready endpoint
        if curl -f -s "https://$API_URL/ready" > /dev/null; then
            echo "✅ Ready endpoint responding"
        else
            echo "⚠️  Ready endpoint not responding"
        fi
    fi
fi

if [ "$COMPONENT" = "all" ] || [ "$COMPONENT" = "worker" ]; then
    echo "👷 Checking worker health..."
    kubectl wait --for=condition=ready pod -l app=kiro-worker -n "$NAMESPACE" --timeout=120s
fi

# Check metrics endpoints
echo "📊 Checking metrics endpoints..."
kubectl get pods -l app=kiro-api -n "$NAMESPACE" -o name | head -1 | xargs -I {} kubectl port-forward {} 9090:9090 -n "$NAMESPACE" &
PORTFORWARD_PID=$!
sleep 5

if curl -f -s "http://localhost:9090/metrics" > /dev/null; then
    echo "✅ Metrics endpoint responding"
else
    echo "⚠️  Metrics endpoint not responding"
fi

kill $PORTFORWARD_PID 2>/dev/null || true

# Check HPA status
echo "📈 Checking HPA status..."
kubectl get hpa -n "$NAMESPACE" || echo "Warning: HPA not found"

# Check PDB status
echo "🛡️  Checking PodDisruptionBudget status..."
kubectl get pdb -n "$NAMESPACE" || echo "Warning: PDB not found"

# Final status report
echo ""
echo "🎉 Deployment completed successfully!"
echo "📊 Deployment summary:"
echo "   Environment: $ENVIRONMENT"
echo "   Component: $COMPONENT"
echo "   Image tag: $IMAGE_TAG"
echo "   Namespace: $NAMESPACE"
echo "   Backup location: $BACKUP_DIR"

# Show current pod status
echo ""
echo "📋 Current pod status:"
kubectl get pods -n "$NAMESPACE" -l "app in (kiro-api,kiro-worker)" -o wide

echo ""
echo "🔗 Useful commands:"
echo "   View logs: kubectl logs -f deployment/kiro-api -n $NAMESPACE"
echo "   Check status: kubectl get pods -n $NAMESPACE"
echo "   Rollback: kubectl rollout undo deployment/kiro-api -n $NAMESPACE"
echo "   Scale: kubectl scale deployment/kiro-api --replicas=5 -n $NAMESPACE"