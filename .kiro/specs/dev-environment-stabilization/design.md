# Design Document

## Overview

This design establishes a robust, reliable development environment for the "No Drake in the House" music streaming blocklist manager using Minikube with Tilt orchestration. The solution addresses critical issues in the current setup including missing database tables, port conflicts, UI rendering problems, and broken automation scripts.

The architecture follows a cloud-native approach using Kubernetes for container orchestration, with Tilt providing fast development feedback loops through live updates and automated dependency management. This eliminates the unreliable Docker Compose setup and provides a production-like environment for local development.

## Architecture

### Development Environment Stack

```mermaid
graph TB
    subgraph "Local Development Machine"
        DEV[Developer]
        TILT[Tilt CLI]
        KUBECTL[kubectl]
        MINIKUBE[Minikube Cluster]
    end
    
    subgraph "Minikube Cluster (kiro-dev namespace)"
        subgraph "Database Layer"
            POSTGRES[PostgreSQL Pod]
            REDIS[Redis Pod]
        end
        
        subgraph "Application Layer"
            BACKEND[Backend Pod<br/>Rust/Axum API]
            FRONTEND[Frontend Pod<br/>Svelte/Nginx]
        end
        
        subgraph "Jobs"
            MIGRATE[Migration Job<br/>Auto-run]
            SEED[Seed Data Job<br/>Manual trigger]
        end
    end
    
    DEV --> TILT
    TILT --> KUBECTL
    KUBECTL --> MINIKUBE
    
    MIGRATE --> POSTGRES
    SEED --> POSTGRES
    BACKEND --> POSTGRES
    BACKEND --> REDIS
    FRONTEND --> BACKEND
    
    TILT -.->|Port Forward| FRONTEND
    TILT -.->|Port Forward| BACKEND
    TILT -.->|Port Forward| POSTGRES
    TILT -.->|Port Forward| REDIS
```

### Service Dependencies and Startup Order

```mermaid
graph TD
    A[Minikube Start] --> B[Namespace Creation]
    B --> C[PostgreSQL Deployment]
    B --> D[Redis Deployment]
    C --> E[Database Migration Job<br/>Auto-run]
    E --> G[Backend Deployment]
    D --> G
    G --> H[Frontend Deployment]
    
    E -.->|Manual Trigger| F[Seed Data Job<br/>Optional]
    F -.-> G
    
    style E fill:#e1f5fe
    style F fill:#fff9c4
    style G fill:#f3e5f5
    style H fill:#fff3e0
```

## Components and Interfaces

### Tilt Configuration

**Primary Tiltfile** (`./Tiltfile`)
- **Purpose**: Orchestrates the entire development environment
- **Responsibilities**:
  - Detects Kubernetes platform (Minikube, Kind, K3s, Docker Desktop)
  - Builds optimized Docker images with live updates
  - Manages service dependencies and startup order
  - Provides manual triggers for common development tasks
  - Configures port forwarding for local access

**Key Features**:
- Live code updates without full rebuilds
- Dependency-aware service startup
- Manual triggers for database operations, testing, and health checks
- Platform-agnostic configuration with automatic detection

### Kubernetes Manifests

**Namespace Configuration**
```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: kiro-dev
```

**Database Services**
- **PostgreSQL**: Persistent storage with development credentials
- **Redis**: Session and cache storage
- **Configuration**: Environment-specific settings via ConfigMaps and Secrets

**Application Services**
- **Backend**: Rust API server with health checks and readiness probes
- **Frontend**: Svelte application served via Nginx with development optimizations

### Database Migration System

**Migration Job Architecture**
```mermaid
sequenceDiagram
    participant T as Tilt
    participant K as Kubernetes
    participant MJ as Migration Job
    participant PG as PostgreSQL
    participant B as Backend
    
    T->>K: Deploy PostgreSQL
    K->>PG: Start PostgreSQL Pod
    PG->>K: Ready Signal
    T->>K: Create Migration Job
    K->>MJ: Start Migration Pod
    MJ->>PG: Run sqlx migrate
    PG->>MJ: Migration Success
    MJ->>K: Job Complete
    T->>K: Deploy Backend
    K->>B: Start Backend Pod
    B->>PG: Connect to Database
```

**Migration Job Specification**:
- **Image**: Same as backend (contains sqlx CLI and migrations)
- **Command**: `sqlx migrate run --database-url $DATABASE_URL`
- **Dependencies**: PostgreSQL must be ready
- **Restart Policy**: OnFailure (retry on temporary failures)
- **Completion**: Job must complete before backend starts

### Docker Build Optimization

**Backend Build Strategy**
- **Multi-stage build**: Chef → Planner → Builder → Development
- **Dependency caching**: cargo-chef for optimal layer caching
- **Live updates**: Source code sync without full rebuilds
- **SQLx offline mode**: Compile-time query verification without database connection

**Frontend Build Strategy**
- **Node.js optimization**: Aggressive npm cache mounting
- **Development builds**: Fast builds with source maps
- **Live updates**: Source sync with automatic rebuilds
- **Nginx serving**: Production-like serving with development features

### Configuration Management

**Environment Variables**
```yaml
# Backend Configuration
DATABASE_URL: postgres://kiro:password@postgres:5432/kiro
REDIS_URL: redis://redis:6379
JWT_SECRET: dev_jwt_secret_change_in_production
RUST_LOG: debug
# CORS allows Tilt port-forwarded frontend access
CORS_ALLOWED_ORIGINS: http://localhost:5000,http://localhost:5173,http://localhost:3000

# Frontend Configuration
# VITE_API_URL uses Tilt port-forwarded backend
VITE_API_URL: http://localhost:3000
VITE_ENVIRONMENT: development
NODE_ENV: development
```

**Secrets Management**
- Development: Plain ConfigMaps for simplicity
- Production: Kubernetes Secrets with proper encryption
- Token encryption: AES-GCM keys stored as secrets

### Port Management and Forwarding

**Port Allocation Strategy**
```yaml
Services:
  PostgreSQL: 5432 (internal) → 5432 (local)
  Redis: 6379 (internal) → 6379 (local)
  Backend: 3000 (internal) → 3000 (local)
  Frontend: 5000 (internal) → 5000 (local)
```

**Conflict Resolution**:
- Configurable local ports via Tilt configuration
- Automatic detection of port conflicts
- Clear error messages with resolution guidance
- Namespace isolation for multi-developer environments

## Data Models

### Database Schema Validation

**Required Tables** (from `001_initial_schema.sql`):
- `users`: User accounts and authentication
- `artists`: Artist catalog with external ID mapping
- `user_artist_blocks`: Personal DNP entries
- `connections`: Service provider connections
- `community_lists`: Shared blocklists
- `action_batches`: Enforcement operation tracking
- `audit_log`: SOC2 compliance logging

**Migration Verification**:
```sql
-- Health check query to verify all tables exist
SELECT 
    (SELECT count(*) FROM users) as user_count,
    (SELECT count(*) FROM artists) as artist_count,
    (SELECT count(*) FROM user_artist_blocks) as dnp_entries_count;
```

### Configuration Schema

**Tilt Configuration Model**:
```python
@dataclass
class K8sConfig:
    platform: str  # minikube, kind, k3s, docker-desktop
    registry: str  # container registry prefix
    load_balancer: bool  # external load balancer support
    docker_env: bool  # docker environment integration
```

## Error Handling

### Database Connection Failures

**Symptoms**: "relation 'users' does not exist" errors
**Root Cause**: Backend starts before migrations complete
**Solution**: 
- Kubernetes Job for migrations with completion dependency
- Backend init containers wait for migration job completion
- Health checks verify table existence before accepting requests

### Port Binding Conflicts

**Symptoms**: "Address already in use" (errno 98)
**Root Cause**: Multiple processes binding to same port
**Solution**:
- Configurable port forwarding in Tilt
- Process detection and cleanup scripts
- Clear error messages with resolution steps
- Namespace isolation for multi-developer setups

### UI Rendering Issues

**Symptoms**: Oversized password validation icons, layout problems
**Root Cause**: CSS conflicts and missing responsive design
**Solution**:
- Tailwind CSS utility classes for consistent sizing
- Component-scoped styling to prevent conflicts
- Responsive design patterns for form elements
- Development-specific CSS debugging tools

### Service Dependency Failures

**Symptoms**: Backend fails to connect to PostgreSQL/Redis
**Root Cause**: Services start in wrong order or dependencies not ready
**Solution**:
- Kubernetes readiness probes for all services
- Init containers for dependency waiting
- Exponential backoff retry logic
- Clear dependency visualization in Tilt UI

## Testing Strategy

### Development Environment Testing

**Automated Health Checks**:
```bash
# Service availability
curl -f http://localhost:3000/health  # Backend health
curl -f http://localhost:5000/        # Frontend availability
kubectl exec deployment/postgres -- pg_isready -U kiro -d kiro
kubectl exec deployment/redis -- redis-cli ping

# Database schema validation
kubectl exec deployment/backend -- ./backend migrate --dry-run
```

**Integration Test Pipeline**:
1. **Environment Setup**: Tilt brings up all services
2. **Migration Verification**: Confirm all tables exist
3. **Service Connectivity**: Test inter-service communication
4. **User Registration Flow**: End-to-end authentication test
5. **API Endpoint Testing**: Comprehensive API validation

### Manual Testing Triggers

**Tilt Manual Resources**:
- `health-check`: Comprehensive service health validation
- `db-migrate`: Manual migration execution (migrations auto-run on startup)
- `db-seed`: Load development seed data (manual only to preserve dev data)
- `db-reset`: Database reset for clean testing
- `backend-tests`: Backend unit and integration tests
- `frontend-tests`: Frontend component and integration tests
- `all-tests`: Complete test suite execution

### Performance Testing

**Load Testing Strategy**:
- Database connection pool stress testing
- Concurrent user registration testing
- API endpoint performance validation
- Frontend rendering performance measurement

**Monitoring Integration**:
- Resource usage tracking (CPU, memory, disk)
- Response time monitoring
- Error rate tracking
- Database query performance analysis

## Implementation Phases

### Phase 1: Core Infrastructure
- Minikube cluster setup and validation
- Basic Kubernetes manifests for all services
- Tilt configuration with service orchestration
- Database migration job implementation

### Phase 2: Service Integration
- Backend and frontend Docker optimization
- Live update configuration for fast development
- Port forwarding and local access setup
- Basic health checks and readiness probes

### Phase 3: Developer Experience
- Manual trigger implementation for common tasks
- Comprehensive error handling and messaging
- Documentation and setup guides
- Performance optimization and caching

### Phase 4: Validation and Hardening
- End-to-end testing automation
- Edge case handling (port conflicts, dependency failures)
- Multi-developer environment testing
- Production readiness validation