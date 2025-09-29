# Docker Build Optimization Guide

This document describes the advanced Docker build optimizations implemented for the Kiro project to achieve faster development iteration cycles.

## Overview

The optimization strategy focuses on:
- **Cargo-chef** for Rust dependency caching (80%+ build time reduction)
- **BuildKit** features for parallel builds and advanced caching
- **Multi-stage builds** with optimal layer ordering
- **Cache mount strategies** for package managers
- **Separate development and production Dockerfiles**

## Build Performance Targets

- **Cold cache builds**: Complete in under 5 minutes
- **Warm cache builds**: Complete in under 30 seconds for incremental changes
- **Dependency changes**: Rebuild only affected layers
- **Source changes**: Skip dependency rebuilds entirely

## Dockerfile Structure

### Backend Optimization

#### Production Dockerfile (`backend/Dockerfile`)
```dockerfile
# syntax=docker/dockerfile:1.4
FROM rust:1.82-slim as chef
RUN cargo install cargo-chef --locked

FROM chef as planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
# Build dependencies first (heavily cached)
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

# Build application (fast since deps are cached)
COPY . .
RUN --mount=type=cache,target=/app/target,sharing=locked \
    cargo build --release
```

#### Fast Development Dockerfile (`backend/Dockerfile.fast`)
- Uses debug builds for faster compilation
- Optimized for development workflow
- Includes development tools and debugging symbols

### Frontend Optimization

#### Production Dockerfile (`frontend/Dockerfile`)
```dockerfile
# syntax=docker/dockerfile:1.4
FROM node:18-alpine as builder
COPY package*.json ./
RUN --mount=type=cache,target=/root/.npm,sharing=locked \
    npm ci --only=production
```

#### Fast Development Dockerfile (`frontend/Dockerfile.fast`)
- Installs all dependencies (including dev dependencies)
- Uses npm cache mounts for faster installs
- Optimized nginx configuration for development

## Cache Management

### Cache Warming
```bash
# Pre-build all dependency layers
./scripts/warm-cache.sh
```

This script:
- Builds all intermediate stages
- Pre-pulls base images
- Creates reusable cache layers
- Reports cache status

### Performance Testing
```bash
# Measure build performance improvements
./scripts/test-docker-build.sh
```

This script:
- Tests cold vs warm cache builds
- Compares standard vs optimized Dockerfiles
- Calculates improvement percentages
- Validates 80% improvement target

### Cache Maintenance
```bash
# Show cache usage
./scripts/refresh-cache.sh show

# Clean specific cache types
./scripts/refresh-cache.sh clean build
./scripts/refresh-cache.sh clean images
./scripts/refresh-cache.sh clean all

# Full cache refresh
./scripts/refresh-cache.sh refresh
```

## BuildKit Features Used

### Cache Mounts
- `--mount=type=cache,target=/usr/local/cargo/registry` - Cargo package cache
- `--mount=type=cache,target=/root/.npm` - npm package cache
- `--mount=type=cache,target=/app/target` - Rust build cache

### Parallel Builds
- Multi-stage builds run in parallel where possible
- Dependency analysis happens independently of source builds
- Base image pulls happen concurrently

### Advanced Syntax
- `# syntax=docker/dockerfile:1.4` enables latest BuildKit features
- `sharing=locked` prevents cache corruption in concurrent builds

## Development Workflow

### Fast Development Builds
```bash
# Backend (debug build, ~10-30s with warm cache)
docker build -f backend/Dockerfile.fast -t kiro/backend:dev backend/

# Frontend (~5-15s with warm cache)
docker build -f frontend/Dockerfile.fast -t kiro/frontend:dev frontend/
```

### Production Builds
```bash
# Backend (release build, optimized)
docker build -f backend/Dockerfile -t kiro/backend:prod backend/

# Frontend (production optimized)
docker build -f frontend/Dockerfile -t kiro/frontend:prod frontend/
```

## Cache Strategy Details

### Rust Backend Caching

1. **Chef Stage**: Install cargo-chef tool (cached indefinitely)
2. **Planner Stage**: Analyze dependencies (cached until Cargo.toml changes)
3. **Builder Stage**: 
   - Build dependencies (cached until Cargo.lock changes)
   - Build application (only rebuilds when source changes)

### Node.js Frontend Caching

1. **Package Installation**: Cached until package.json changes
2. **Source Build**: Only rebuilds when source files change
3. **Static Assets**: Nginx serves pre-built assets

## Performance Monitoring

### Build Time Metrics
- Track build times for each stage
- Monitor cache hit rates
- Measure improvement over baseline

### Cache Efficiency
- Monitor cache size growth
- Track cache hit/miss ratios
- Identify cache invalidation patterns

## Troubleshooting

### Common Issues

#### Slow Builds Despite Caching
```bash
# Check if BuildKit is enabled
export DOCKER_BUILDKIT=1

# Verify cache mounts are working
docker build --progress=plain -f backend/Dockerfile.fast backend/
```

#### Cache Corruption
```bash
# Clean and rebuild cache
./scripts/refresh-cache.sh refresh
```

#### Out of Disk Space
```bash
# Clean unused cache
./scripts/refresh-cache.sh clean all
```

### Debug Build Performance
```bash
# Enable detailed build output
export BUILDKIT_PROGRESS=plain
docker build --no-cache -f backend/Dockerfile.fast backend/
```

## Best Practices

### Dockerfile Optimization
1. **Layer Ordering**: Most stable layers first (base images, system packages)
2. **Cache Mounts**: Use for package managers and build artifacts
3. **Multi-stage**: Separate dependency builds from application builds
4. **Minimal Context**: Use .dockerignore to reduce build context

### Development Workflow
1. **Warm Cache First**: Run `./scripts/warm-cache.sh` after setup
2. **Use Fast Dockerfiles**: For development iteration
3. **Monitor Performance**: Regular performance testing
4. **Clean Regularly**: Prevent cache bloat

### CI/CD Integration
1. **Cache Persistence**: Use registry cache or build cache volumes
2. **Parallel Builds**: Build frontend and backend concurrently
3. **Layer Reuse**: Share base layers between builds
4. **Performance Monitoring**: Track build times in CI

## Expected Performance Gains

With proper cache warming and optimized Dockerfiles:

- **Backend builds**: 80%+ faster with warm cache
- **Frontend builds**: 70%+ faster with warm cache
- **Development iteration**: Sub-30 second rebuilds
- **Cold builds**: Still 40-50% faster due to better layer ordering

## Integration with Development Tools

### Makefile Integration
```makefile
# Add to Makefile
warm-cache:
	./scripts/warm-cache.sh

test-build-performance:
	./scripts/test-docker-build.sh

clean-cache:
	./scripts/refresh-cache.sh clean all
```

### Tilt Integration
```python
# Use fast Dockerfiles in Tiltfile
docker_build('kiro/backend', 
    context='./backend',
    dockerfile='./backend/Dockerfile.fast')
```

This optimization strategy provides the foundation for rapid development iteration while maintaining production-ready build processes.