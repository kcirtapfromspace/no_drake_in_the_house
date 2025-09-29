# Enhanced Tiltfile for optimal Kubernetes development experience
# Optimized for fast feedback, comprehensive monitoring, and developer productivity

# Load Tilt extensions
load('ext://restart_process', 'docker_build_with_restart')

# =============================================================================
# CONFIGURATION & PERFORMANCE SETTINGS
# =============================================================================

# Development namespace
namespace = "kiro-dev"
k8s_namespace(namespace)

# Performance optimizations for fastest feedback loop
update_settings(
    max_parallel_updates=6,
    k8s_upsert_timeout_secs=90,
    suppress_unused_image_warnings=None
)

# Advanced Docker build optimization
docker_prune_settings(
    num_builds=5,
    keep_recent=3
)

# =============================================================================
# MULTI-PLATFORM KUBERNETES DETECTION
# =============================================================================

def detect_k8s_platform():
    # Get current context - using a simpler approach for Starlark compatibility
    context = str(local('kubectl config current-context', quiet=True)).strip()
    
    if 'minikube' in context:
        return {
            'platform': 'minikube',
            'registry': '',
            'load_balancer': False,
            'docker_env': True
        }
    elif 'k3s' in context or 'k3d' in context:
        return {
            'platform': 'k3s',
            'registry': 'localhost:5000/',
            'load_balancer': True,
            'docker_env': False
        }
    elif 'kind' in context:
        return {
            'platform': 'kind',
            'registry': '',
            'load_balancer': False,
            'docker_env': False
        }
    elif 'docker-desktop' in context:
        return {
            'platform': 'docker-desktop',
            'registry': '',
            'load_balancer': True,
            'docker_env': False
        }
    else:
        return {
            'platform': 'generic',
            'registry': '',
            'load_balancer': False,
            'docker_env': False
        }

# Detect platform and configure accordingly
k8s_config = detect_k8s_platform()
print("🔍 Detected Kubernetes platform: " + k8s_config['platform'])

# =============================================================================
# OPTIMIZED DOCKER BUILDS WITH LIVE UPDATES
# =============================================================================

# Backend build with live updates and optimized caching
backend_image = k8s_config['registry'] + 'kiro/backend'
docker_build(
    backend_image,
    context='./backend',
    dockerfile='./backend/Dockerfile.dev',
    only=[
        './src',
        './Cargo.toml',
        './Cargo.lock',
        './migrations',
        './.sqlx'
    ],
    live_update=[
        sync('./backend/src', '/app/src'),
        sync('./backend/migrations', '/app/migrations'),
        run('cargo build --bin music-streaming-blocklist-backend', trigger=['./backend/src'])
    ],
    build_args={
        'BUILDKIT_INLINE_CACHE': '1'
    }
)

# Frontend build with live updates and hot reloading
frontend_image = k8s_config['registry'] + 'kiro/frontend'
docker_build(
    frontend_image,
    context='./frontend',
    dockerfile='./frontend/Dockerfile.fast',
    only=[
        './src',
        './public',
        './package.json',
        './package-lock.json',
        './rollup.config.js',
        './tsconfig.json',
        './nginx.conf',
        './nginx-dev.conf'
    ],
    live_update=[
        sync('./frontend/src', '/app/src'),
        sync('./frontend/public', '/app/public'),
        run('VITE_API_URL= VITE_ENVIRONMENT=development npm run build:dev', trigger=['./frontend/src'])
    ],
    build_args={
        'BUILDKIT_INLINE_CACHE': '1',
        'NODE_ENV': 'development'
    }
)

# =============================================================================
# KUBERNETES RESOURCE DEPLOYMENT
# =============================================================================

# Deploy Kubernetes manifests
k8s_yaml('./k8s/dev-manifests.yaml')

# =============================================================================
# RESOURCE CONFIGURATION WITH OPTIMIZED DEPENDENCIES
# =============================================================================

# Database resources (foundational layer)
k8s_resource('postgres',
    port_forwards=['5432:5432'] if not k8s_config['load_balancer'] else [],
    labels=['database', 'foundation'],
    resource_deps=[],
    auto_init=True,
    pod_readiness='wait'
)

k8s_resource('redis',
    port_forwards=['6379:6379'] if not k8s_config['load_balancer'] else [],
    labels=['cache', 'foundation'],
    resource_deps=[],
    auto_init=True,
    pod_readiness='wait'
)

# Backend service (depends on databases)
k8s_resource('backend',
    port_forwards=['3000:3000'] if not k8s_config['load_balancer'] else [],
    labels=['api', 'core'],
    resource_deps=['postgres', 'redis'],
    auto_init=True,
    pod_readiness='wait'
)

# Frontend service (depends on backend)
k8s_resource('frontend',
    port_forwards=['5000:5000'] if not k8s_config['load_balancer'] else [],
    labels=['web', 'ui'],
    resource_deps=['backend'],
    auto_init=True,
    pod_readiness='wait'
)

# =============================================================================
# MANUAL TRIGGER COMMANDS FOR DEVELOPMENT WORKFLOW
# =============================================================================

# Database management triggers
local_resource(
    'db-migrate',
    cmd='kubectl exec -n ' + namespace + ' deployment/backend -- sh -c "cd /app && export SQLX_OFFLINE=true && ./backend migrate"',
    resource_deps=['backend', 'postgres'],
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['database', 'setup']
)

local_resource(
    'db-reset',
    cmd='kubectl exec -n ' + namespace + ' deployment/postgres -- sh -c "psql -U kiro -d postgres -c \'DROP DATABASE IF EXISTS kiro;\' && psql -U kiro -d postgres -c \'CREATE DATABASE kiro;\'"',
    resource_deps=['postgres'],
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['database', 'reset']
)

local_resource(
    'db-status',
    cmd='kubectl exec -n ' + namespace + ' deployment/postgres -- psql -U kiro -d kiro -c "\\l"',
    resource_deps=['postgres'],
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['database', 'monitoring']
)

# Testing triggers
local_resource(
    'backend-tests',
    cmd='cd backend && export DATABASE_URL=postgres://kiro:password@localhost:5432/kiro && export REDIS_URL=redis://localhost:6379 && cargo test --release -- --nocapture',
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['tests', 'backend']
)

local_resource(
    'frontend-tests',
    cmd='cd frontend && npm test -- --run',
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['tests', 'frontend']
)

local_resource(
    'all-tests',
    cmd='cd backend && export DATABASE_URL=postgres://kiro:password@localhost:5432/kiro && export REDIS_URL=redis://localhost:6379 && cargo test --release -- --nocapture && cd ../frontend && npm test -- --run',
    resource_deps=['backend', 'postgres', 'redis'],
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['tests', 'all']
)

# Health check and monitoring triggers
local_resource(
    'health-check',
    cmd='curl -f http://localhost:3000/health && curl -f http://localhost:5000/health && kubectl exec -n ' + namespace + ' deployment/postgres -- pg_isready -U kiro -d kiro && kubectl exec -n ' + namespace + ' deployment/redis -- redis-cli ping',
    resource_deps=['backend', 'frontend', 'postgres', 'redis'],
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['monitoring', 'health']
)

local_resource(
    'service-status',
    cmd='kubectl get pods -n ' + namespace + ' -o wide && kubectl get services -n ' + namespace,
    resource_deps=['backend', 'frontend', 'postgres', 'redis'],
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['monitoring', 'status']
)

# Performance and build optimization triggers
local_resource(
    'warm-cache',
    cmd='docker build --target chef -t kiro/backend:chef ./backend/ || true && docker build --target builder -t kiro/frontend:builder ./frontend/ || true',
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['performance', 'cache']
)

# Development workflow automation
local_resource(
    'dev-setup',
    cmd='kubectl wait --for=condition=ready pod -l app=postgres -n ' + namespace + ' --timeout=60s && kubectl wait --for=condition=ready pod -l app=redis -n ' + namespace + ' --timeout=60s && kubectl wait --for=condition=ready pod -l app=backend -n ' + namespace + ' --timeout=120s && kubectl exec -n ' + namespace + ' deployment/backend -- sh -c "cd /app && export SQLX_OFFLINE=true && ./backend migrate"',
    resource_deps=['backend', 'postgres', 'redis'],
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['setup', 'workflow']
)

# Cleanup and maintenance triggers
local_resource(
    'cleanup-resources',
    cmd='docker image prune -f && kubectl delete pods --field-selector=status.phase=Succeeded -n ' + namespace + ' 2>/dev/null || true && kubectl delete pods --field-selector=status.phase=Failed -n ' + namespace + ' 2>/dev/null || true',
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['maintenance', 'cleanup']
)

# =============================================================================
# DEVELOPER EXPERIENCE ENHANCEMENTS
# =============================================================================

# Print comprehensive startup information
print("🚀 Enhanced Kiro Development Environment")
print("=" * 60)
print("")
print("🔍 Platform: " + k8s_config['platform'])
print("📦 Namespace: " + namespace)
print("")
print("📡 Service Endpoints:")
print("  🔗 Backend API:    http://localhost:3000")
print("  🔗 Frontend Web:   http://localhost:5000")
print("  🔗 PostgreSQL:     localhost:5432 (user: kiro, pass: password, db: kiro)")
print("  🔗 Redis:          localhost:6379")
print("")
print("🎮 Manual Triggers Available:")
print("  📊 Monitoring:")
print("    • health-check      - Check all service health")
print("    • service-status    - Detailed status report")
print("")
print("  🗄️  Database:")
print("    • db-migrate        - Run database migrations")
print("    • db-reset          - Reset database (destroys data)")
print("    • db-status         - Database status")
print("")
print("  🧪 Testing:")
print("    • backend-tests     - Run all backend tests")
print("    • frontend-tests    - Run frontend tests")
print("    • all-tests         - Run complete test suite")
print("")
print("  ⚡ Performance:")
print("    • warm-cache        - Pre-build Docker layers")
print("")
print("  🛠️  Workflow:")
print("    • dev-setup         - Complete environment setup")
print("    • cleanup-resources - Clean up old resources")
print("")
print("💡 Quick Start Workflow:")
print("  1. Wait for all services to show 'Ready' in Tilt UI")
print("  2. Run 'dev-setup' trigger for complete initialization")
print("  3. Start coding - changes auto-reload within 10 seconds!")
print("  4. Use 'health-check' to verify everything is working")
print("  5. Run 'all-tests' before committing changes")
print("")
print("=" * 60)