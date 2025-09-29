# Troubleshooting Guide: Development Environment

This guide covers common startup and connectivity issues for both Docker Compose and Kubernetes development environments.

## Quick Health Check Commands

### Docker Compose Environment
```bash
# Check service status
docker-compose ps

# Check service health
curl http://localhost:3000/health  # Backend
curl http://localhost:5001/health  # Frontend

# View logs
docker-compose logs backend
docker-compose logs frontend
```

### Kubernetes Environment
```bash
# Check pod status
kubectl get pods -n kiro-dev

# Check service health (with port forwarding)
kubectl port-forward -n kiro-dev deployment/backend 3000:3000 &
curl http://localhost:3000/health
kill %1

# View logs
kubectl logs -n kiro-dev deployment/backend
kubectl logs -n kiro-dev deployment/frontend
```

## Common Issues and Solutions

### 1. Backend Compilation Errors

**Symptoms:**
- Docker build fails with Rust compilation errors
- Backend container exits immediately

**Solutions:**
```bash
# Check compilation locally first
cd backend
cargo check
cargo build

# Fix common issues:
# - Update dependencies: cargo update
# - Clear cache: cargo clean
# - Check SQLx offline mode: export SQLX_OFFLINE=true
```

### 2. Database Connection Issues

**Symptoms:**
- Backend logs show "connection refused" or "database not found"
- Health check shows database as unhealthy

**Docker Compose Solutions:**
```bash
# Ensure databases are running
docker-compose up -d postgres redis

# Check database connectivity
docker-compose exec postgres pg_isready -U kiro -d kiro_dev
docker-compose exec redis redis-cli ping

# Reset database if corrupted
make reset-db
# or
docker-compose down -v
docker-compose up -d postgres redis
```

**Kubernetes Solutions:**
```bash
# Check database pods
kubectl get pods -n kiro-dev -l app=postgres
kubectl get pods -n kiro-dev -l app=redis

# Test connectivity from backend pod
kubectl exec -n kiro-dev deployment/backend -- pg_isready -h postgres -U kiro -d kiro
kubectl exec -n kiro-dev deployment/backend -- redis-cli -h redis ping

# Check service DNS resolution
kubectl run -n kiro-dev test-pod --image=alpine --rm -it --restart=Never -- nslookup postgres
```

### 3. Port Conflicts

**Symptoms:**
- "Port already in use" errors
- Services fail to start

**Solutions:**
```bash
# Find processes using ports
lsof -i :3000  # Backend
lsof -i :5000  # Frontend (Docker Compose)
lsof -i :5001  # Frontend (Docker Compose mapped)
lsof -i :5432  # PostgreSQL
lsof -i :6379  # Redis

# Kill conflicting processes
kill -9 <PID>

# Or use different ports in docker-compose.yml
```

### 4. Frontend Nginx Configuration Issues

**Symptoms:**
- Frontend container crashes with nginx errors
- "host not found in upstream" errors

**Docker Compose Solutions:**
- Ensure backend service name matches nginx configuration
- Check `frontend/nginx-dev.conf` uses correct service names

**Kubernetes Solutions:**
```bash
# Check service names and endpoints
kubectl get services -n kiro-dev
kubectl get endpoints -n kiro-dev

# Verify nginx configuration in container
kubectl exec -n kiro-dev deployment/frontend -- cat /etc/nginx/conf.d/default.conf

# Check if backend service is accessible
kubectl exec -n kiro-dev deployment/frontend -- nslookup backend
```

### 5. Image Pull Issues (Kubernetes)

**Symptoms:**
- `ErrImageNeverPull` or `ImagePullBackOff` errors
- Pods stuck in pending state

**Solutions:**
```bash
# For minikube, load images manually
minikube image load kiro/backend
minikube image load kiro/frontend:v3

# For other Kubernetes setups, ensure images are built
docker build -t kiro/backend backend/
docker build -f frontend/Dockerfile.k8s -t kiro/frontend frontend/

# Check image pull policy in manifests (should be "Never" for local images)
```

### 6. Service Discovery Issues

**Symptoms:**
- Services can't communicate with each other
- DNS resolution failures

**Docker Compose Solutions:**
```bash
# Ensure all services are on the same network
docker network ls
docker-compose ps

# Check service names in docker-compose.yml match application configuration
```

**Kubernetes Solutions:**
```bash
# Check service and endpoint configuration
kubectl get services -n kiro-dev
kubectl get endpoints -n kiro-dev

# Test DNS resolution between pods
kubectl exec -n kiro-dev deployment/backend -- nslookup postgres
kubectl exec -n kiro-dev deployment/frontend -- nslookup backend

# Check network policies (if any)
kubectl get networkpolicies -n kiro-dev
```

### 7. Environment Variable Issues

**Symptoms:**
- Services start but behave incorrectly
- Configuration-related errors in logs

**Solutions:**
```bash
# Check environment variables in containers
docker-compose exec backend env | grep -E "(DATABASE_URL|REDIS_URL|JWT_SECRET)"
kubectl exec -n kiro-dev deployment/backend -- env | grep -E "(DATABASE_URL|REDIS_URL|JWT_SECRET)"

# Verify configuration matches between environments
# Docker Compose: docker-compose.yml
# Kubernetes: k8s/dev-manifests.yaml
```

### 8. Volume and Persistence Issues

**Symptoms:**
- Data loss between restarts
- Permission denied errors

**Docker Compose Solutions:**
```bash
# Check volume mounts
docker-compose config
docker volume ls

# Fix permission issues
docker-compose exec postgres chown -R postgres:postgres /var/lib/postgresql/data
```

**Kubernetes Solutions:**
```bash
# Check persistent volume claims (if using persistent storage)
kubectl get pvc -n kiro-dev

# For development, we use emptyDir volumes (data is ephemeral)
# This is expected behavior for dev environment
```

## Performance Troubleshooting

### Slow Build Times

**Solutions:**
```bash
# Warm Docker build cache
./scripts/warm-cache.sh

# Use BuildKit for faster builds
export DOCKER_BUILDKIT=1

# Clean up old images and containers
docker system prune -f
```

### High Resource Usage

**Solutions:**
```bash
# Check resource usage
docker stats  # Docker Compose
kubectl top pods -n kiro-dev  # Kubernetes

# Adjust resource limits in configuration files
# Docker Compose: Add resource limits to services
# Kubernetes: Update resource requests/limits in manifests
```

## Environment-Specific Commands

### Docker Compose Development
```bash
# Full environment setup
make setup
make dev

# Individual service management
docker-compose up -d postgres redis
docker-compose up backend frontend

# Cleanup
docker-compose down -v
make clean
```

### Kubernetes Development
```bash
# Deploy to Kubernetes
kubectl apply -f k8s/dev-manifests.yaml

# Port forwarding for local access
kubectl port-forward -n kiro-dev deployment/backend 3000:3000
kubectl port-forward -n kiro-dev deployment/frontend 5000:5000

# Cleanup
kubectl delete namespace kiro-dev
```

### Using Tilt (Advanced Kubernetes Development)
```bash
# Start Tilt development environment
tilt up

# Available manual triggers:
# - health-check: Check all service health
# - db-migrate: Run database migrations
# - dev-setup: Complete environment setup
# - all-tests: Run complete test suite

# Cleanup
tilt down
```

## Verification Checklist

After resolving issues, verify the environment is working:

### ✅ Docker Compose Verification
- [ ] All services show "healthy" status: `docker-compose ps`
- [ ] Backend health check passes: `curl http://localhost:3000/health`
- [ ] Frontend serves content: `curl http://localhost:5001/health`
- [ ] API proxy works: `curl http://localhost:5001/api/health`
- [ ] Database connectivity: Backend logs show successful database connection
- [ ] Redis connectivity: Backend logs show successful Redis connection

### ✅ Kubernetes Verification
- [ ] All pods are running: `kubectl get pods -n kiro-dev`
- [ ] Services are accessible via port forwarding
- [ ] Backend health check passes through port forward
- [ ] Frontend serves content through port forward
- [ ] Service discovery works: DNS resolution between services
- [ ] Logs show no error messages

## Getting Help

If issues persist after following this guide:

1. **Check service logs** for specific error messages
2. **Verify network connectivity** between services
3. **Ensure resource availability** (CPU, memory, disk space)
4. **Review configuration files** for typos or mismatched service names
5. **Test individual components** in isolation
6. **Compare working vs. non-working configurations**

## Useful Debugging Commands

```bash
# Docker debugging
docker inspect <container_name>
docker exec -it <container_name> /bin/sh
docker logs --follow <container_name>

# Kubernetes debugging
kubectl describe pod -n kiro-dev <pod_name>
kubectl exec -n kiro-dev <pod_name> -- /bin/sh
kubectl logs -n kiro-dev <pod_name> --follow

# Network debugging
kubectl run -n kiro-dev debug-pod --image=alpine --rm -it --restart=Never -- /bin/sh
# Inside the pod: nslookup, wget, curl, ping
```

This troubleshooting guide should help resolve most common issues encountered during development environment setup and operation.