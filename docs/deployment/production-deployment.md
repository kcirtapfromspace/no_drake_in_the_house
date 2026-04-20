# Production Deployment Guide

This document outlines the comprehensive production deployment process for the NDITH music streaming blocklist manager.

## Prerequisites

- Kubernetes cluster (1.25+) with RBAC enabled
- kubectl configured with cluster access
- Helm 3.x (optional, for package management)
- Docker registry access (GitHub Container Registry)
- Domain name and SSL certificates (Let's Encrypt recommended)
- PostgreSQL 15+ and Redis 7+ instances (managed or self-hosted)
- AWS S3 bucket for backups (optional)
- Monitoring stack (Prometheus/Grafana)

## Infrastructure Requirements

### Minimum Resource Requirements

- **API Pods**: 3 replicas, 512Mi memory, 250m CPU each
- **Worker Pods**: 2 replicas, 256Mi memory, 100m CPU each
- **Database**: PostgreSQL with 20GB storage, 2 CPU, 4GB RAM
- **Redis**: 1GB memory, persistent storage
- **Monitoring**: Prometheus (50GB storage), Grafana (10GB storage)

### Network Requirements

- Ingress controller (nginx recommended)
- Network policies for pod-to-pod communication
- External access to streaming service APIs (Spotify, Apple Music, etc.)
- DNS resolution for custom domains

## Deployment Steps

### 1. Prepare Infrastructure

```bash
# Create namespace and basic resources
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/rbac.yaml
kubectl apply -f k8s/resource-quotas.yaml
kubectl apply -f k8s/network-policies.yaml
kubectl apply -f k8s/pod-security.yaml

# Verify namespace creation
kubectl get namespace ndith-production
```

### 2. Configure Secrets

Update the secrets file with actual production values:

```bash
# Edit secrets with base64 encoded values
kubectl apply -f k8s/secrets.yaml

# Verify secrets
kubectl get secrets -n ndith-production
```

**Required Secrets:**
- `DATABASE_URL`: PostgreSQL connection string
- `REDIS_URL`: Redis connection string
- `JWT_SECRET`: Strong random key for JWT signing
- `KMS_KEY_ID`: AWS KMS key for token encryption
- `SPOTIFY_CLIENT_SECRET`: Spotify API credentials
- `APPLE_MUSIC_TEAM_ID`: Apple Developer Team ID
- `APPLE_MUSIC_KEY_ID`: Apple Music key identifier
- `APPLE_MUSIC_PRIVATE_KEY`: Apple Music API key
- `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY`: For S3 backups

### 3. Deploy Database Infrastructure

```bash
# Apply database migrations ConfigMap
kubectl apply -f k8s/database-migrations.yaml

# Run initial database setup
./scripts/migrate.sh production

# Verify migration completion
kubectl logs -l app=ndith-migration -n ndith-production
```

### 4. Deploy Core Application

```bash
# Deploy configuration
kubectl apply -f k8s/configmap.yaml

# Deploy API and Worker services
kubectl apply -f k8s/api-deployment.yaml
kubectl apply -f k8s/worker-deployment.yaml

# Deploy services and networking
kubectl apply -f k8s/services.yaml
kubectl apply -f k8s/ingress.yaml

# Wait for deployments to be ready
kubectl rollout status deployment/ndith-api -n ndith-production --timeout=300s
kubectl rollout status deployment/ndith-worker -n ndith-production --timeout=300s
```

### 5. Deploy Monitoring Stack

```bash
# Deploy Prometheus with RBAC
kubectl apply -f k8s/monitoring/rbac.yaml
kubectl apply -f k8s/monitoring/prometheus.yaml
kubectl apply -f k8s/monitoring/alerting-rules.yaml

# Deploy Grafana with dashboards
kubectl apply -f k8s/monitoring/grafana.yaml
kubectl apply -f k8s/monitoring/grafana-dashboards.yaml

# Deploy logging infrastructure
kubectl apply -f k8s/logging.yaml

# Verify monitoring stack
kubectl get pods -l app=prometheus -n ndith-production
kubectl get pods -l app=grafana -n ndith-production
```

### 6. Configure Autoscaling and Backup

```bash
# Deploy Horizontal Pod Autoscaler
kubectl apply -f k8s/hpa.yaml

# Deploy backup CronJobs
kubectl apply -f k8s/backup-cronjob.yaml

# Deploy health check system
kubectl apply -f k8s/health-checks.yaml

# Verify HPA status
kubectl get hpa -n ndith-production
```

### 7. SSL and Domain Configuration

```bash
# Install cert-manager (if not already installed)
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml

# Create ClusterIssuer for Let's Encrypt
cat <<EOF | kubectl apply -f -
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: admin@ndith.house
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
EOF

# Update ingress with your domain
kubectl patch ingress ndith-ingress -n ndith-production -p '{"spec":{"rules":[{"host":"api.yourdomain.com","http":{"paths":[{"path":"/","pathType":"Prefix","backend":{"service":{"name":"ndith-api-service","port":{"number":80}}}}]}}]}}'
```

## Verification and Testing

### 1. Health Checks

```bash
# Check pod status
kubectl get pods -n ndith-production -o wide

# Check services
kubectl get svc -n ndith-production

# Check ingress and SSL
kubectl get ingress -n ndith-production
kubectl describe ingress ndith-ingress -n ndith-production

# Test health endpoints
curl -f https://api.yourdomain.com/health
curl -f https://api.yourdomain.com/health/ready
curl -f https://api.yourdomain.com/metrics
```

### 2. Load Testing

```bash
# Install k6 for load testing
kubectl run k6-load-test --image=grafana/k6:latest --rm -it --restart=Never -- run - <<EOF
import http from 'k6/http';
import { check } from 'k6';

export let options = {
  stages: [
    { duration: '2m', target: 10 },
    { duration: '5m', target: 10 },
    { duration: '2m', target: 0 },
  ],
};

export default function() {
  let response = http.get('https://api.yourdomain.com/health');
  check(response, {
    'status is 200': (r) => r.status === 200,
    'response time < 500ms': (r) => r.timings.duration < 500,
  });
}
EOF
```

### 3. Monitoring Verification

```bash
# Port forward to access Grafana locally
kubectl port-forward svc/grafana 3000:3000 -n ndith-production

# Access Grafana at http://localhost:3000
# Default credentials: admin / (check grafana-secrets)

# Verify Prometheus targets
kubectl port-forward svc/prometheus 9090:9090 -n ndith-production
# Access Prometheus at http://localhost:9090
```

## Production Operations

### Deployment Updates

Use the automated deployment script:

```bash
# Deploy specific component with new image
./scripts/deploy.sh production api v1.2.3

# Deploy all components
./scripts/deploy.sh production all latest

# Check deployment status
kubectl rollout status deployment/ndith-api -n ndith-production
```

### Backup and Recovery

```bash
# Manual full backup
./scripts/backup.sh production full

# Manual incremental backup
./scripts/backup.sh production incremental

# List available backups
aws s3 ls s3://ndith-backups-prod/database/

# Restore from backup (example)
kubectl run postgres-restore --image=postgres:15-alpine --rm -it --restart=Never -- \
  pg_restore --verbose --clean --no-acl --no-owner -h $DB_HOST -U $DB_USER -d ndith_prod /path/to/backup.dump
```

### Scaling Operations

```bash
# Manual scaling
kubectl scale deployment/ndith-api --replicas=5 -n ndith-production
kubectl scale deployment/ndith-worker --replicas=4 -n ndith-production

# Check HPA status
kubectl get hpa -n ndith-production

# Update HPA limits
kubectl patch hpa ndith-api-hpa -n ndith-production -p '{"spec":{"maxReplicas":15}}'
```

### Log Management

```bash
# View application logs
kubectl logs -f deployment/ndith-api -n ndith-production --tail=100

# View worker logs
kubectl logs -f deployment/ndith-worker -n ndith-production --tail=100

# View logs from specific time
kubectl logs deployment/ndith-api -n ndith-production --since=1h

# Export logs for analysis
kubectl logs deployment/ndith-api -n ndith-production --since=24h > api-logs.txt
```

## Monitoring and Alerting

### Key Metrics to Monitor

1. **Application Metrics**:
   - Request rate and latency (95th percentile < 1s)
   - Error rate (< 1%)
   - Job queue size and processing rate
   - Database connection pool usage

2. **Infrastructure Metrics**:
   - CPU and memory usage
   - Disk space and I/O
   - Network throughput
   - Pod restart count

3. **Business Metrics**:
   - Enforcement success rate (> 95%)
   - External API rate limit usage
   - User authentication success rate
   - Token vault operations

### Alert Thresholds

- **Critical**: API down, database unreachable, error rate > 5%
- **Warning**: High latency, memory usage > 80%, job queue backlog
- **Info**: Deployment updates, scaling events, backup completion

### Grafana Dashboards

Access pre-configured dashboards:
- **NDITH Overview**: Application performance and health
- **NDITH Enforcement**: Enforcement job metrics and external API status
- **NDITH Infrastructure**: Kubernetes cluster and resource usage

## Security Considerations

### Network Security

- Network policies restrict pod-to-pod communication
- Ingress controller with rate limiting enabled
- TLS 1.2+ enforced for all external connections
- Regular security scanning with Trivy and Semgrep

### Data Security

- All secrets encrypted at rest with KMS
- Database connections use TLS
- Audit logging for all user actions
- Regular backup encryption verification

### Access Control

- RBAC configured with least privilege principle
- Service accounts with minimal required permissions
- Pod security standards enforced (restricted profile)
- Regular access reviews and key rotation

## Troubleshooting

### Common Issues

1. **Pod CrashLoopBackOff**:
   ```bash
   kubectl describe pod <pod-name> -n ndith-production
   kubectl logs <pod-name> -n ndith-production --previous
   ```

2. **Database Connection Issues**:
   ```bash
   # Test database connectivity
   kubectl run db-test --image=postgres:15-alpine --rm -it --restart=Never -- \
     psql $DATABASE_URL -c "SELECT version();"
   ```

3. **High Memory Usage**:
   ```bash
   # Check memory usage by pod
   kubectl top pods -n ndith-production --sort-by=memory
   
   # Get detailed resource usage
   kubectl describe node <node-name>
   ```

4. **SSL Certificate Issues**:
   ```bash
   # Check certificate status
   kubectl get certificate -n ndith-production
   kubectl describe certificate ndith-api-tls -n ndith-production
   
   # Force certificate renewal
   kubectl delete certificate ndith-api-tls -n ndith-production
   ```

### Emergency Procedures

1. **Complete Service Outage**:
   ```bash
   # Check cluster status
   kubectl cluster-info
   kubectl get nodes
   
   # Check critical pods
   kubectl get pods -n ndith-production
   kubectl get pods -n kube-system
   
   # Restart deployments if needed
   kubectl rollout restart deployment/ndith-api -n ndith-production
   ```

2. **Database Recovery**:
   ```bash
   # Restore from latest backup
   ./scripts/backup.sh production restore latest
   
   # Run health checks
   kubectl exec -it deployment/ndith-api -n ndith-production -- \
     curl -f http://localhost:3000/health/db
   ```

3. **Rollback Deployment**:
   ```bash
   # Rollback to previous version
   kubectl rollout undo deployment/ndith-api -n ndith-production
   kubectl rollout undo deployment/ndith-worker -n ndith-production
   
   # Verify rollback
   kubectl rollout status deployment/ndith-api -n ndith-production
   ```

## Maintenance Windows

### Planned Maintenance

1. **Monthly Security Updates**:
   - Update base images and dependencies
   - Apply Kubernetes cluster updates
   - Rotate secrets and certificates

2. **Quarterly Reviews**:
   - Resource usage analysis and optimization
   - Security audit and penetration testing
   - Disaster recovery testing

3. **Annual Tasks**:
   - Complete infrastructure review
   - Backup and recovery procedure validation
   - Performance benchmarking and capacity planning

### Maintenance Checklist

- [ ] Notify users of maintenance window
- [ ] Create full system backup
- [ ] Update monitoring alerts
- [ ] Perform updates in staging first
- [ ] Execute deployment with rollback plan
- [ ] Verify all health checks pass
- [ ] Monitor system for 24 hours post-deployment
- [ ] Document any issues and resolutions

## Support and Escalation

### On-Call Procedures

1. **Severity 1 (Critical)**: Complete service outage
   - Response time: 15 minutes
   - Escalation: Immediate to senior engineer

2. **Severity 2 (High)**: Degraded performance
   - Response time: 1 hour
   - Escalation: Within 2 hours if unresolved

3. **Severity 3 (Medium)**: Minor issues
   - Response time: 4 hours
   - Escalation: Next business day

### Contact Information

- **Primary On-Call**: [Your team's contact]
- **Secondary Escalation**: [Senior engineer contact]
- **Infrastructure Team**: [Platform team contact]
- **Security Team**: [Security team contact]

## Useful Commands Reference

```bash
# Quick status check
kubectl get pods,svc,ingress -n ndith-production

# Resource usage
kubectl top pods -n ndith-production
kubectl top nodes

# Logs and debugging
kubectl logs -f deployment/ndith-api -n ndith-production
kubectl exec -it deployment/ndith-api -n ndith-production -- /bin/sh

# Scaling and updates
kubectl scale deployment/ndith-api --replicas=5 -n ndith-production
kubectl set image deployment/ndith-api api=ghcr.io/ndith/api:v1.2.3 -n ndith-production

# Backup and restore
./scripts/backup.sh production full
./scripts/deploy.sh production all latest

# Port forwarding for debugging
kubectl port-forward svc/ndith-api-service 3000:80 -n ndith-production
kubectl port-forward svc/grafana 3000:3000 -n ndith-production
```
