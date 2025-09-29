# Kubernetes Development Environment

This document describes how to set up and use the Kubernetes development environment for the No Drake in the House platform.

## Prerequisites

### Required Tools

1. **Docker Desktop** (with Kubernetes enabled) or **Minikube**
   ```bash
   # Docker Desktop: Enable Kubernetes in settings
   # OR install Minikube
   brew install minikube
   minikube start
   ```

2. **kubectl** - Kubernetes CLI
   ```bash
   brew install kubectl
   ```

3. **Helm** - Kubernetes package manager
   ```bash
   brew install helm
   ```

4. **Skaffold** - Development workflow tool
   ```bash
   brew install skaffold
   ```

5. **Tilt** (Optional) - Alternative development tool with better UI
   ```bash
   brew install tilt
   ```

### Verify Installation

```bash
# Check all tools are installed
kubectl version --client
helm version
skaffold version
docker version
```

## Quick Start

### Option 1: Using Skaffold (Recommended)

```bash
# Start the complete development environment
make k8s-dev

# This will:
# - Build Docker images
# - Deploy to Kubernetes using Helm
# - Set up port forwarding
# - Watch for file changes and rebuild/redeploy automatically
```

### Option 2: Using Tilt (Alternative with better UI)

```bash
# Start Tilt development environment
tilt up

# Open Tilt UI in browser (usually opens automatically)
# Press 'space' in terminal to open UI
```

### Option 3: Manual Deployment

```bash
# Build images
make k8s-build

# Deploy using Helm
make k8s-deploy

# Set up port forwarding (in separate terminal)
make k8s-port-forward
```

## Service Access

Once deployed, services are available at:

- **Backend API**: http://localhost:3000
- **Frontend**: http://localhost:5000
- **PostgreSQL**: localhost:5432 (user: kiro, password: password, db: kiro)
- **Redis**: localhost:6379

## Development Workflow

### File Watching and Hot Reloading

Both Skaffold and Tilt support automatic rebuilding when files change:

**Backend (Rust)**:
- Changes to `backend/src/**/*.rs` trigger cargo build
- Changes to `backend/Cargo.toml` trigger full rebuild
- Container is restarted automatically

**Frontend (Svelte)**:
- Changes to `frontend/src/**/*` trigger npm build
- Changes to `frontend/package.json` trigger full rebuild
- Container is restarted automatically

### Making Code Changes

1. Edit files in `backend/src/` or `frontend/src/`
2. Skaffold/Tilt automatically detects changes
3. Images are rebuilt and pods are updated
4. Services restart with new code

### Debugging

#### View Logs
```bash
# All pods
kubectl logs -f -l app.kubernetes.io/instance=kiro -n kiro-dev

# Specific service
kubectl logs -f deployment/kiro-backend -n kiro-dev
kubectl logs -f deployment/kiro-frontend -n kiro-dev
```

#### Access Pod Shell
```bash
# Backend pod
kubectl exec -it deployment/kiro-backend -n kiro-dev -- /bin/bash

# Frontend pod
kubectl exec -it deployment/kiro-frontend -n kiro-dev -- /bin/sh

# Database pod
kubectl exec -it deployment/kiro-postgresql -n kiro-dev -- psql -U kiro -d kiro
```

#### Port Forward Individual Services
```bash
# Backend only
kubectl port-forward -n kiro-dev service/kiro-backend 3000:3000

# Database only
kubectl port-forward -n kiro-dev service/kiro-postgresql 5432:5432
```

## Configuration

### Environment Variables

Development-specific environment variables are configured in `helm/values-dev.yaml`:

```yaml
backend:
  env:
    DATABASE_URL: "postgres://kiro:password@kiro-postgresql:5432/kiro"
    REDIS_URL: "redis://kiro-redis-master:6379"
    JWT_SECRET: "dev_jwt_secret_change_in_production"
    RUST_LOG: "debug"
```

### Resource Limits

Development resource limits are optimized for local development:

```yaml
backend:
  resources:
    requests:
      memory: "256Mi"
      cpu: "100m"
    limits:
      memory: "512Mi"
      cpu: "500m"
```

### Persistent Storage

In development mode, persistence is disabled for faster cleanup:

```yaml
postgresql:
  primary:
    persistence:
      enabled: false  # Ephemeral storage for dev

redis:
  master:
    persistence:
      enabled: false  # Ephemeral storage for dev
```

## Health Checks

### Application Health

```bash
# Check all services
make k8s-status

# Manual health check
curl http://localhost:3000/health
```

### Kubernetes Health

```bash
# Pod status
kubectl get pods -n kiro-dev

# Service endpoints
kubectl get endpoints -n kiro-dev

# Events (for troubleshooting)
kubectl get events -n kiro-dev --sort-by='.lastTimestamp'
```

## Testing in Kubernetes

### Running Tests

```bash
# Backend tests (from local machine)
cd backend && cargo test

# Frontend tests (from local machine)
cd frontend && npm test

# Or run tests inside pods
kubectl exec -it deployment/kiro-backend -n kiro-dev -- cargo test
```

### Integration Testing

The Kubernetes environment provides a production-like setup for integration testing:

1. **Database Integration**: Real PostgreSQL instance
2. **Redis Integration**: Real Redis instance
3. **Network Policies**: Service-to-service communication
4. **Resource Constraints**: Memory and CPU limits

## Troubleshooting

### Common Issues

#### Pods Not Starting
```bash
# Check pod status
kubectl describe pod -l app.kubernetes.io/instance=kiro -n kiro-dev

# Check events
kubectl get events -n kiro-dev --sort-by='.lastTimestamp'
```

#### Image Pull Errors
```bash
# Ensure images are built locally
make k8s-build

# Check image pull policy in values-dev.yaml
# Should be: pullPolicy: Never
```

#### Port Forward Failures
```bash
# Check if services are running
kubectl get services -n kiro-dev

# Check if pods are ready
kubectl get pods -n kiro-dev

# Try different local ports
kubectl port-forward -n kiro-dev service/kiro-backend 3001:3000
```

#### Database Connection Issues
```bash
# Check PostgreSQL pod
kubectl logs -f deployment/kiro-postgresql -n kiro-dev

# Test connection from backend pod
kubectl exec -it deployment/kiro-backend -n kiro-dev -- \
  psql postgres://kiro:password@kiro-postgresql:5432/kiro -c "SELECT 1;"
```

### Reset Environment

```bash
# Complete cleanup and restart
make k8s-clean
make k8s-dev
```

### Debug Mode

Enable debug mode for more verbose logging:

```bash
# With Skaffold
skaffold dev --profile=debug

# With Tilt
tilt up -- --debug=true
```

## Production Differences

The development environment differs from production in several ways:

| Aspect | Development | Production |
|--------|-------------|------------|
| Image Pull Policy | Never (local images) | Always (registry) |
| Persistence | Disabled (ephemeral) | Enabled (persistent volumes) |
| Resource Limits | Low (for local dev) | Higher (for performance) |
| Logging Level | Debug | Info/Warn |
| Secrets | Hardcoded values | Kubernetes secrets |
| Ingress | Simple nginx | Production ingress controller |

## Advanced Usage

### Custom Helm Values

Create your own values file for specific configurations:

```bash
# Create custom values
cp helm/values-dev.yaml helm/values-local.yaml

# Edit as needed
vim helm/values-local.yaml

# Deploy with custom values
helm upgrade --install kiro ./helm \
  --values ./helm/values-local.yaml \
  --namespace kiro-dev
```

### Multiple Environments

Deploy multiple environments for testing:

```bash
# Deploy to different namespace
helm upgrade --install kiro-feature ./helm \
  --values ./helm/values-dev.yaml \
  --namespace kiro-feature \
  --create-namespace
```

### Performance Profiling

Enable performance monitoring:

```bash
# Add performance monitoring to values
# Then access metrics at http://localhost:3000/metrics
```

## Next Steps

After setting up the Kubernetes development environment:

1. **Run the test suite**: `make test`
2. **Check service health**: `make k8s-status`
3. **Start developing**: Edit files and watch automatic rebuilds
4. **Monitor logs**: Use Tilt UI or kubectl logs
5. **Deploy changes**: Automatic with file watching enabled

For production deployment, see the production deployment documentation.