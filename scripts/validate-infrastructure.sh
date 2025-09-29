#!/bin/bash
set -euo pipefail

# Infrastructure validation script for production deployment
# Usage: ./scripts/validate-infrastructure.sh [environment]

ENVIRONMENT=${1:-production}
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Load environment-specific configuration
case $ENVIRONMENT in
    "production")
        NAMESPACE="kiro-production"
        ;;
    "staging")
        NAMESPACE="kiro-staging"
        ;;
    *)
        echo "Error: Unknown environment '$ENVIRONMENT'"
        echo "Usage: $0 [production|staging]"
        exit 1
        ;;
esac

echo "🔍 Validating infrastructure for $ENVIRONMENT environment..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
PASSED=0
FAILED=0
WARNINGS=0

# Function to print test results
print_result() {
    local test_name=$1
    local status=$2
    local message=$3
    
    case $status in
        "PASS")
            echo -e "✅ ${GREEN}PASS${NC}: $test_name"
            ((PASSED++))
            ;;
        "FAIL")
            echo -e "❌ ${RED}FAIL${NC}: $test_name - $message"
            ((FAILED++))
            ;;
        "WARN")
            echo -e "⚠️  ${YELLOW}WARN${NC}: $test_name - $message"
            ((WARNINGS++))
            ;;
    esac
}

# Check prerequisites
echo "📋 Checking prerequisites..."

# Check kubectl
if command -v kubectl &> /dev/null; then
    print_result "kubectl installed" "PASS"
else
    print_result "kubectl installed" "FAIL" "kubectl not found in PATH"
fi

# Check cluster connectivity
if kubectl cluster-info &> /dev/null; then
    print_result "Kubernetes cluster connectivity" "PASS"
else
    print_result "Kubernetes cluster connectivity" "FAIL" "Cannot connect to cluster"
fi

# Check namespace
if kubectl get namespace "$NAMESPACE" &> /dev/null; then
    print_result "Namespace exists" "PASS"
else
    print_result "Namespace exists" "FAIL" "Namespace '$NAMESPACE' not found"
fi

echo ""
echo "🔐 Checking secrets and configuration..."

# Check required secrets
REQUIRED_SECRETS=("kiro-secrets" "grafana-secrets")
for secret in "${REQUIRED_SECRETS[@]}"; do
    if kubectl get secret "$secret" -n "$NAMESPACE" &> /dev/null; then
        print_result "Secret $secret exists" "PASS"
        
        # Check secret keys
        case $secret in
            "kiro-secrets")
                REQUIRED_KEYS=("DATABASE_URL" "REDIS_URL" "JWT_SECRET" "KMS_KEY_ID")
                for key in "${REQUIRED_KEYS[@]}"; do
                    if kubectl get secret "$secret" -n "$NAMESPACE" -o jsonpath="{.data.$key}" &> /dev/null; then
                        print_result "Secret key $secret.$key" "PASS"
                    else
                        print_result "Secret key $secret.$key" "FAIL" "Key not found in secret"
                    fi
                done
                ;;
        esac
    else
        print_result "Secret $secret exists" "FAIL" "Secret not found"
    fi
done

# Check ConfigMaps
REQUIRED_CONFIGMAPS=("kiro-config" "kiro-migrations")
for cm in "${REQUIRED_CONFIGMAPS[@]}"; do
    if kubectl get configmap "$cm" -n "$NAMESPACE" &> /dev/null; then
        print_result "ConfigMap $cm exists" "PASS"
    else
        print_result "ConfigMap $cm exists" "FAIL" "ConfigMap not found"
    fi
done

echo ""
echo "🚀 Checking deployments and services..."

# Check deployments
DEPLOYMENTS=("kiro-api" "kiro-worker" "prometheus" "grafana")
for deployment in "${DEPLOYMENTS[@]}"; do
    if kubectl get deployment "$deployment" -n "$NAMESPACE" &> /dev/null; then
        print_result "Deployment $deployment exists" "PASS"
        
        # Check deployment status
        READY_REPLICAS=$(kubectl get deployment "$deployment" -n "$NAMESPACE" -o jsonpath='{.status.readyReplicas}' 2>/dev/null || echo "0")
        DESIRED_REPLICAS=$(kubectl get deployment "$deployment" -n "$NAMESPACE" -o jsonpath='{.spec.replicas}' 2>/dev/null || echo "0")
        
        if [ "$READY_REPLICAS" = "$DESIRED_REPLICAS" ] && [ "$READY_REPLICAS" != "0" ]; then
            print_result "Deployment $deployment ready" "PASS"
        else
            print_result "Deployment $deployment ready" "FAIL" "$READY_REPLICAS/$DESIRED_REPLICAS replicas ready"
        fi
    else
        print_result "Deployment $deployment exists" "FAIL" "Deployment not found"
    fi
done

# Check services
SERVICES=("kiro-api-service" "kiro-worker-service" "prometheus" "grafana")
for service in "${SERVICES[@]}"; do
    if kubectl get service "$service" -n "$NAMESPACE" &> /dev/null; then
        print_result "Service $service exists" "PASS"
        
        # Check service endpoints
        ENDPOINTS=$(kubectl get endpoints "$service" -n "$NAMESPACE" -o jsonpath='{.subsets[*].addresses[*].ip}' 2>/dev/null || echo "")
        if [ -n "$ENDPOINTS" ]; then
            print_result "Service $service has endpoints" "PASS"
        else
            print_result "Service $service has endpoints" "WARN" "No endpoints found"
        fi
    else
        print_result "Service $service exists" "FAIL" "Service not found"
    fi
done

echo ""
echo "🌐 Checking networking and ingress..."

# Check ingress
if kubectl get ingress kiro-ingress -n "$NAMESPACE" &> /dev/null; then
    print_result "Ingress exists" "PASS"
    
    # Check ingress status
    INGRESS_IP=$(kubectl get ingress kiro-ingress -n "$NAMESPACE" -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || echo "")
    if [ -n "$INGRESS_IP" ]; then
        print_result "Ingress has external IP" "PASS"
    else
        print_result "Ingress has external IP" "WARN" "No external IP assigned yet"
    fi
else
    print_result "Ingress exists" "FAIL" "Ingress not found"
fi

# Check network policies
NETWORK_POLICIES=("kiro-api-network-policy" "kiro-worker-network-policy")
for policy in "${NETWORK_POLICIES[@]}"; do
    if kubectl get networkpolicy "$policy" -n "$NAMESPACE" &> /dev/null; then
        print_result "NetworkPolicy $policy exists" "PASS"
    else
        print_result "NetworkPolicy $policy exists" "WARN" "NetworkPolicy not found"
    fi
done

echo ""
echo "📊 Checking monitoring and observability..."

# Check Prometheus
if kubectl get pod -l app=prometheus -n "$NAMESPACE" &> /dev/null; then
    print_result "Prometheus pods exist" "PASS"
    
    # Check Prometheus health
    if kubectl exec -n "$NAMESPACE" deployment/prometheus -- wget -q -O- http://localhost:9090/-/healthy &> /dev/null; then
        print_result "Prometheus health check" "PASS"
    else
        print_result "Prometheus health check" "FAIL" "Health endpoint not responding"
    fi
else
    print_result "Prometheus pods exist" "FAIL" "No Prometheus pods found"
fi

# Check Grafana
if kubectl get pod -l app=grafana -n "$NAMESPACE" &> /dev/null; then
    print_result "Grafana pods exist" "PASS"
    
    # Check Grafana health
    if kubectl exec -n "$NAMESPACE" deployment/grafana -- wget -q -O- http://localhost:3000/api/health &> /dev/null; then
        print_result "Grafana health check" "PASS"
    else
        print_result "Grafana health check" "FAIL" "Health endpoint not responding"
    fi
else
    print_result "Grafana pods exist" "FAIL" "No Grafana pods found"
fi

echo ""
echo "🔄 Checking autoscaling and resource management..."

# Check HPA
HPAS=("kiro-api-hpa" "kiro-worker-hpa")
for hpa in "${HPAS[@]}"; do
    if kubectl get hpa "$hpa" -n "$NAMESPACE" &> /dev/null; then
        print_result "HPA $hpa exists" "PASS"
        
        # Check HPA status
        HPA_STATUS=$(kubectl get hpa "$hpa" -n "$NAMESPACE" -o jsonpath='{.status.conditions[?(@.type=="AbleToScale")].status}' 2>/dev/null || echo "")
        if [ "$HPA_STATUS" = "True" ]; then
            print_result "HPA $hpa able to scale" "PASS"
        else
            print_result "HPA $hpa able to scale" "WARN" "HPA may not be able to scale"
        fi
    else
        print_result "HPA $hpa exists" "WARN" "HPA not found"
    fi
done

# Check PodDisruptionBudgets
PDBS=("kiro-api-pdb" "kiro-worker-pdb")
for pdb in "${PDBS[@]}"; do
    if kubectl get pdb "$pdb" -n "$NAMESPACE" &> /dev/null; then
        print_result "PDB $pdb exists" "PASS"
    else
        print_result "PDB $pdb exists" "WARN" "PDB not found"
    fi
done

# Check resource quotas
if kubectl get resourcequota kiro-production-quota -n "$NAMESPACE" &> /dev/null; then
    print_result "ResourceQuota exists" "PASS"
    
    # Check quota usage
    QUOTA_STATUS=$(kubectl describe resourcequota kiro-production-quota -n "$NAMESPACE" | grep -E "(requests.cpu|requests.memory|limits.cpu|limits.memory)" | head -4)
    echo "📊 Resource quota status:"
    echo "$QUOTA_STATUS" | while read -r line; do
        echo "   $line"
    done
else
    print_result "ResourceQuota exists" "WARN" "ResourceQuota not found"
fi

echo ""
echo "💾 Checking backup and recovery..."

# Check backup CronJobs
BACKUP_CRONJOBS=("kiro-database-backup-full" "kiro-database-backup-incremental")
for cronjob in "${BACKUP_CRONJOBS[@]}"; do
    if kubectl get cronjob "$cronjob" -n "$NAMESPACE" &> /dev/null; then
        print_result "CronJob $cronjob exists" "PASS"
        
        # Check last successful run
        LAST_SUCCESS=$(kubectl get cronjob "$cronjob" -n "$NAMESPACE" -o jsonpath='{.status.lastSuccessfulTime}' 2>/dev/null || echo "")
        if [ -n "$LAST_SUCCESS" ]; then
            print_result "CronJob $cronjob last success" "PASS"
        else
            print_result "CronJob $cronjob last success" "WARN" "No successful runs yet"
        fi
    else
        print_result "CronJob $cronjob exists" "WARN" "CronJob not found"
    fi
done

# Check backup storage
if kubectl get pvc backup-storage -n "$NAMESPACE" &> /dev/null; then
    print_result "Backup storage PVC exists" "PASS"
    
    # Check PVC status
    PVC_STATUS=$(kubectl get pvc backup-storage -n "$NAMESPACE" -o jsonpath='{.status.phase}' 2>/dev/null || echo "")
    if [ "$PVC_STATUS" = "Bound" ]; then
        print_result "Backup storage PVC bound" "PASS"
    else
        print_result "Backup storage PVC bound" "FAIL" "PVC status: $PVC_STATUS"
    fi
else
    print_result "Backup storage PVC exists" "WARN" "PVC not found"
fi

echo ""
echo "🏥 Checking health and readiness..."

# Check API health
if kubectl get pod -l app=kiro-api -n "$NAMESPACE" &> /dev/null; then
    API_POD=$(kubectl get pod -l app=kiro-api -n "$NAMESPACE" -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")
    if [ -n "$API_POD" ]; then
        if kubectl exec -n "$NAMESPACE" "$API_POD" -- wget -q -O- http://localhost:3000/health &> /dev/null; then
            print_result "API health endpoint" "PASS"
        else
            print_result "API health endpoint" "FAIL" "Health endpoint not responding"
        fi
        
        if kubectl exec -n "$NAMESPACE" "$API_POD" -- wget -q -O- http://localhost:3000/ready &> /dev/null; then
            print_result "API ready endpoint" "PASS"
        else
            print_result "API ready endpoint" "FAIL" "Ready endpoint not responding"
        fi
    fi
fi

# Check worker health
if kubectl get pod -l app=kiro-worker -n "$NAMESPACE" &> /dev/null; then
    WORKER_POD=$(kubectl get pod -l app=kiro-worker -n "$NAMESPACE" -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")
    if [ -n "$WORKER_POD" ]; then
        if kubectl exec -n "$NAMESPACE" "$WORKER_POD" -- wget -q -O- http://localhost:9090/health &> /dev/null; then
            print_result "Worker health endpoint" "PASS"
        else
            print_result "Worker health endpoint" "FAIL" "Health endpoint not responding"
        fi
    fi
fi

echo ""
echo "🔒 Checking security configuration..."

# Check pod security context
API_PODS=$(kubectl get pods -l app=kiro-api -n "$NAMESPACE" -o jsonpath='{.items[*].metadata.name}' 2>/dev/null || echo "")
for pod in $API_PODS; do
    if [ -n "$pod" ]; then
        RUN_AS_NON_ROOT=$(kubectl get pod "$pod" -n "$NAMESPACE" -o jsonpath='{.spec.securityContext.runAsNonRoot}' 2>/dev/null || echo "")
        if [ "$RUN_AS_NON_ROOT" = "true" ]; then
            print_result "Pod $pod runs as non-root" "PASS"
        else
            print_result "Pod $pod runs as non-root" "WARN" "Pod may be running as root"
        fi
        break # Check only first pod
    fi
done

# Check RBAC
SERVICE_ACCOUNTS=("kiro-api" "kiro-worker" "prometheus")
for sa in "${SERVICE_ACCOUNTS[@]}"; do
    if kubectl get serviceaccount "$sa" -n "$NAMESPACE" &> /dev/null; then
        print_result "ServiceAccount $sa exists" "PASS"
    else
        print_result "ServiceAccount $sa exists" "WARN" "ServiceAccount not found"
    fi
done

echo ""
echo "📈 Performance and capacity checks..."

# Check resource usage
echo "📊 Current resource usage:"
kubectl top pods -n "$NAMESPACE" --no-headers 2>/dev/null | while read -r line; do
    echo "   $line"
done || echo "   Metrics server not available"

# Check node capacity
echo "📊 Node capacity:"
kubectl top nodes --no-headers 2>/dev/null | while read -r line; do
    echo "   $line"
done || echo "   Metrics server not available"

# Check persistent volume usage
echo "📊 Persistent volume usage:"
kubectl get pv -o custom-columns=NAME:.metadata.name,CAPACITY:.spec.capacity.storage,STATUS:.status.phase,CLAIM:.spec.claimRef.name 2>/dev/null | grep -E "(prometheus|grafana|backup)" || echo "   No matching PVs found"

echo ""
echo "🎯 Final validation summary..."

# External connectivity test (if ingress is available)
INGRESS_HOST=$(kubectl get ingress kiro-ingress -n "$NAMESPACE" -o jsonpath='{.spec.rules[0].host}' 2>/dev/null || echo "")
if [ -n "$INGRESS_HOST" ]; then
    if curl -f -s "https://$INGRESS_HOST/health" &> /dev/null; then
        print_result "External API access" "PASS"
    else
        print_result "External API access" "WARN" "Cannot reach API externally"
    fi
fi

# Database connectivity test
if kubectl get secret kiro-secrets -n "$NAMESPACE" &> /dev/null; then
    DATABASE_URL=$(kubectl get secret kiro-secrets -n "$NAMESPACE" -o jsonpath='{.data.DATABASE_URL}' | base64 -d 2>/dev/null || echo "")
    if [ -n "$DATABASE_URL" ]; then
        if kubectl run db-test --image=postgres:15-alpine --rm -i --restart=Never --quiet -- psql "$DATABASE_URL" -c "SELECT 1;" &> /dev/null; then
            print_result "Database connectivity" "PASS"
        else
            print_result "Database connectivity" "FAIL" "Cannot connect to database"
        fi
    fi
fi

# Redis connectivity test
if kubectl get secret kiro-secrets -n "$NAMESPACE" &> /dev/null; then
    REDIS_URL=$(kubectl get secret kiro-secrets -n "$NAMESPACE" -o jsonpath='{.data.REDIS_URL}' | base64 -d 2>/dev/null || echo "")
    if [ -n "$REDIS_URL" ]; then
        if kubectl run redis-test --image=redis:7-alpine --rm -i --restart=Never --quiet -- redis-cli -u "$REDIS_URL" ping &> /dev/null; then
            print_result "Redis connectivity" "PASS"
        else
            print_result "Redis connectivity" "FAIL" "Cannot connect to Redis"
        fi
    fi
fi

echo ""
echo "📋 Validation Results Summary:"
echo "================================"
echo -e "✅ ${GREEN}Passed${NC}: $PASSED"
echo -e "⚠️  ${YELLOW}Warnings${NC}: $WARNINGS"
echo -e "❌ ${RED}Failed${NC}: $FAILED"
echo "================================"

if [ $FAILED -eq 0 ]; then
    if [ $WARNINGS -eq 0 ]; then
        echo -e "🎉 ${GREEN}All checks passed! Infrastructure is ready for production.${NC}"
        exit 0
    else
        echo -e "✅ ${YELLOW}Infrastructure is mostly ready, but please review warnings.${NC}"
        exit 0
    fi
else
    echo -e "❌ ${RED}Infrastructure validation failed. Please fix the issues before proceeding.${NC}"
    exit 1
fi