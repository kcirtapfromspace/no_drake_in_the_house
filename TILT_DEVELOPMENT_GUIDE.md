# Enhanced Tilt Development Guide

This guide covers the enhanced Tiltfile configuration for optimal Kubernetes development experience with the Kiro platform.

## 🚀 Quick Start

### Prerequisites
- Docker Desktop or Docker Engine
- kubectl configured with a Kubernetes cluster
- Tilt installed (`brew install tilt-dev/tap/tilt`)
- minikube (recommended for local development)

### One-Command Setup
```bash
make tilt-dev-guide
```

This opens an interactive menu to guide you through the setup process.

### Manual Setup
```bash
# 1. Validate configuration
make tilt-validate-enhanced

# 2. Warm Docker cache (optional but recommended)
make tilt-warm-cache

# 3. Start Tilt
tilt up
```

## 🎯 Enhanced Features

### 1. Optimized Resource Dependencies
The enhanced Tiltfile configures services with proper dependency chains:

```
PostgreSQL & Redis (foundational)
    ↓
Backend (depends on databases)
    ↓
Frontend (depends on backend)
```

This ensures services start in the correct order and wait for dependencies to be ready.

### 2. Live Updates for Fast Iteration
- **Backend**: Source code changes trigger incremental builds within 10 seconds
- **Frontend**: Hot reloading with automatic rebuild on source changes
- **Migrations**: Automatic sync without full container rebuild

### 3. Manual Triggers for Development Workflow

#### Database Management
- `db-migrate` - Run database migrations
- `db-reset` - Reset database (destroys all data)

#### Testing
- `backend-tests` - Run all backend tests
- `backend-unit-tests` - Run backend unit tests only
- `backend-integration-tests` - Run backend integration tests
- `frontend-tests` - Run frontend tests
- `all-tests` - Run complete test suite

#### Monitoring & Health
- `health-check` - Check all service health
- `service-status` - Detailed status report with resource usage

#### Performance & Optimization
- `warm-cache` - Pre-build Docker layers for faster builds
- `build-performance-test` - Measure build time improvements

#### Workflow Automation
- `dev-setup` - Complete environment initialization
- `cleanup-resources` - Clean up old resources

### 4. Real-Time Monitoring
- Live log streaming in Tilt UI
- Service health status indicators
- Resource usage monitoring
- Build time tracking

### 5. Performance Optimizations
- Multi-stage Docker builds with cargo-chef caching
- BuildKit inline caching
- Optimized dependency management
- Parallel build execution (up to 6 concurrent builds)

## 📊 Service Endpoints

Once Tilt is running, services are available at:

- **Backend API**: http://localhost:3000
- **Frontend**: http://localhost:5000
- **PostgreSQL**: localhost:5432 (user: kiro, pass: password, db: kiro)
- **Redis**: localhost:6379
- **Tilt UI**: http://localhost:10350

## 💡 Development Workflow

### Daily Development Cycle

1. **Start Environment**
   ```bash
   tilt up
   ```

2. **Initialize Services**
   - Wait for all services to show "Ready" in Tilt UI
   - Run `dev-setup` trigger for complete initialization

3. **Development Loop**
   - Edit code in `backend/src/` or `frontend/src/`
   - Changes auto-reload within 10 seconds
   - Use `health-check` trigger to verify services
   - Run specific tests with test triggers

4. **Database Changes**
   - Add migration files to `backend/migrations/`
   - Run `db-migrate` trigger to apply changes

5. **Testing**
   - Use specific test triggers for faster feedback
   - Run `all-tests` before committing changes

### Debugging Workflow

1. **Check Service Status**
   - Open Tilt UI at http://localhost:10350
   - Check service logs in real-time
   - Use `service-status` trigger for detailed diagnostics

2. **Common Issues**
   - Services not starting: Check Tilt UI error messages
   - Slow builds: Run `warm-cache` trigger
   - Database issues: Run `db-migrate` or `db-reset`

3. **Emergency Reset**
   ```bash
   tilt down
   make cleanup-resources
   tilt up
   ```

## 🎮 Manual Triggers Reference

### Setup & Initialization
| Trigger | Description | Dependencies |
|---------|-------------|--------------|
| `dev-setup` | Complete environment setup | backend, postgres, redis |

### Database Operations
| Trigger | Description | Dependencies |
|---------|-------------|--------------|
| `db-migrate` | Run database migrations | backend, postgres |
| `db-reset` | Reset database (destroys data) | postgres |

### Testing
| Trigger | Description | Dependencies |
|---------|-------------|--------------|
| `backend-tests` | All backend tests | - |
| `backend-unit-tests` | Backend unit tests only | - |
| `backend-integration-tests` | Backend integration tests | backend, postgres, redis |
| `frontend-tests` | Frontend tests | - |
| `all-tests` | Complete test suite | backend, postgres, redis |

### Monitoring
| Trigger | Description | Dependencies |
|---------|-------------|--------------|
| `health-check` | Check all service health | all services |
| `service-status` | Detailed status report | all services |

### Performance
| Trigger | Description | Dependencies |
|---------|-------------|--------------|
| `warm-cache` | Pre-build Docker layers | - |
| `build-performance-test` | Measure build times | - |

### Maintenance
| Trigger | Description | Dependencies |
|---------|-------------|--------------|
| `cleanup-resources` | Clean up old resources | - |

## ⚡ Performance Features

### Build Time Optimization
- **Cargo-chef caching**: Rust dependencies cached separately from source code
- **Multi-stage builds**: Optimized layer caching
- **BuildKit features**: Inline caching and parallel builds
- **Live updates**: Source changes don't require full rebuilds

### Expected Performance
- **Cold build**: 2-5 minutes (first time)
- **Warm build**: 30-60 seconds (with cache)
- **Live updates**: 5-10 seconds (source changes only)
- **Test execution**: 10-30 seconds (depending on test type)

### Performance Testing
```bash
# Test build performance
make tilt-performance-test

# Warm cache for faster builds
make tilt-warm-cache

# Monitor build times
# Check Tilt UI for real-time build metrics
```

## 🔧 Configuration

### Environment Variables
The Tiltfile uses these key configurations:

```python
# Performance settings
update_settings(
    max_parallel_updates=6,           # Parallel build limit
    k8s_upsert_timeout_secs=90,      # Kubernetes operation timeout
    suppress_unused_image_warnings=None
)

# Docker optimization
docker_prune_settings(
    num_builds=5,                     # Keep build cache
    keep_recent=3                     # Keep recent builds
)
```

### Customization
To customize the Tiltfile for your needs:

1. **Adjust resource dependencies**: Modify `resource_deps` arrays
2. **Change port forwards**: Update `port_forwards` configurations
3. **Add custom triggers**: Create new `local_resource` blocks
4. **Modify build settings**: Adjust `docker_build` configurations

### Development vs Production
The enhanced Tiltfile is optimized for development:
- Uses fast Dockerfiles (`Dockerfile.fast`)
- Enables live updates and hot reloading
- Includes debug symbols and verbose logging
- Provides extensive manual triggers for testing

For production deployment, use the Helm charts or Skaffold configuration.

## 🚨 Troubleshooting

### Common Issues

#### Services Not Starting
```bash
# Check Tilt UI for error messages
# Run diagnostics
make tilt-validate-enhanced

# Check Kubernetes cluster
kubectl cluster-info
```

#### Slow Builds
```bash
# Warm Docker cache
make tilt-warm-cache

# Test build performance
make tilt-performance-test

# Check Docker resources
docker system df
```

#### Database Connection Issues
```bash
# Check PostgreSQL status in Tilt UI
# Run database setup
# Click 'db-migrate' trigger in Tilt UI

# Manual check
kubectl exec -n kiro-dev deployment/postgres -- pg_isready -U kiro -d kiro
```

#### Live Updates Not Working
1. Check file paths in Tiltfile `live_update` rules
2. Verify file changes are being detected
3. Try manual rebuild in Tilt UI
4. Check container logs for sync errors

### Emergency Recovery
```bash
# Complete reset
tilt down
make cleanup-resources
docker system prune -f
tilt up
```

### Getting Help
1. Check Tilt UI logs and error messages
2. Run `make tilt-validate-enhanced` for configuration issues
3. Use `service-status` trigger for detailed diagnostics
4. Check this guide's troubleshooting section
5. Consult Tilt documentation: https://docs.tilt.dev/

## 📚 Additional Resources

- [Tilt Documentation](https://docs.tilt.dev/)
- [Kubernetes Development Best Practices](https://kubernetes.io/docs/concepts/)
- [Docker Build Optimization](https://docs.docker.com/develop/dev-best-practices/)
- [Cargo Chef for Rust Caching](https://github.com/LukeMathWalker/cargo-chef)

## 🎯 Tips for Maximum Productivity

1. **Use the Tilt UI**: Real-time monitoring and trigger execution
2. **Warm cache regularly**: Run `warm-cache` trigger for faster builds
3. **Use specific test triggers**: Faster feedback than running all tests
4. **Monitor resource usage**: Check `service-status` for performance insights
5. **Keep Tilt running**: Live updates provide the fastest development cycle
6. **Use manual triggers**: Avoid command-line context switching
7. **Check logs in real-time**: Tilt UI provides better log viewing than kubectl