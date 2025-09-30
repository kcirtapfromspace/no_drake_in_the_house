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
        run('SQLX_OFFLINE=true cargo build --bin music-streaming-blocklist-backend', trigger=['./backend/src'])
    ],
    build_args={
        'BUILDKIT_INLINE_CACHE': '1',
        'SQLX_OFFLINE': 'true'
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
        run('VITE_API_URL=http://localhost:3000 VITE_ENVIRONMENT=development npm run build:dev', trigger=['./frontend/src'])
    ],
    build_args={
        'BUILDKIT_INLINE_CACHE': '1',
        'NODE_ENV': 'development',
        'VITE_API_URL': 'http://localhost:3000'
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

# Backend service (runs migrations automatically on startup)
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

# Simple database migration trigger - restarts backend which runs migrations automatically
local_resource(
    'db-migrate',
    cmd='kubectl rollout restart deployment/backend -n ' + namespace + ' && kubectl rollout status deployment/backend -n ' + namespace,
    resource_deps=['backend'],
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['database']
)

# Simple database reset
local_resource(
    'db-reset',
    cmd='kubectl exec -n ' + namespace + ' deployment/postgres -- psql -U kiro -d postgres -c "DROP DATABASE IF EXISTS kiro; CREATE DATABASE kiro;"',
    resource_deps=['postgres'],
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['database']
)





# Simple test triggers
local_resource(
    'backend-tests',
    cmd='cd backend && SQLX_OFFLINE=true cargo test',
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['tests']
)

local_resource(
    'frontend-tests',
    cmd='cd frontend && npm test -- --run',
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['tests']
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
print("💡 Quick Start:")
print("  1. Wait for services to show 'Ready' in Tilt UI")
print("  2. Backend runs migrations automatically on startup")
print("  3. Start coding - changes auto-reload!")
print("")
print("🔧 Manual Triggers:")
print("  • db-migrate: Restart backend (runs migrations)")
print("  • db-reset: Reset database")
print("  • health-check: Check all services")
print("  • backend-tests / frontend-tests: Run tests")
print("")
print("=" * 60)