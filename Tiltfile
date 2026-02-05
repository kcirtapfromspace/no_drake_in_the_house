# Tiltfile for Minikube development with local registry
#
# One-time setup:
#   ./scripts/setup-registry.sh    # registry + minikube with insecure-registry
#   ./scripts/tilt-build.sh        # pre-warm cargo cache (first time only)
#
# Then:
#   tilt up

# =============================================================================
# CONFIGURATION
# =============================================================================

namespace = "kiro-dev"
k8s_namespace(namespace)

update_settings(
    max_parallel_updates=3,
    k8s_upsert_timeout_secs=900,
    suppress_unused_image_warnings=None
)

# Disable Docker prune to preserve build cache (Rust builds are very slow)
docker_prune_settings(disable=True)

# =============================================================================
# MINIKUBE DETECTION
# =============================================================================

context = str(local('kubectl config current-context', quiet=True)).strip()
if 'minikube' not in context:
    fail("Expected minikube context, got: " + context + ". Run: kubectl config use-context minikube")

# =============================================================================
# LOCAL REGISTRY
# =============================================================================

# Push images to localhost:5001, pull from host.minikube.internal:5001 inside cluster
default_registry('localhost:5001', host_from_cluster='host.minikube.internal:5001')

# =============================================================================
# SHARED: Strip minikube Docker env vars so builds use HOST Docker daemon
# (where BuildKit cache lives). Tilt injects DOCKER_HOST etc. for minikube.
# =============================================================================

_strip_mk = 'env -u DOCKER_HOST -u DOCKER_TLS_VERIFY -u DOCKER_CERT_PATH -u MINIKUBE_ACTIVE_DOCKERD '

# =============================================================================
# FRONTEND BUILD
# =============================================================================

# Build on host, package into nginx, push to registry
custom_build(
    'kiro/frontend',
    'set -e && ' +
    'cd frontend && npm run build && cd .. && ' +
    _strip_mk + 'docker build -t $EXPECTED_REF -f ./frontend/Dockerfile.runtime ./frontend && ' +
    _strip_mk + 'docker push $EXPECTED_REF',
    deps=['./frontend/src', './frontend/public/index.html', './frontend/public/global.css', './frontend/public/favicon.svg', './frontend/package.json', './frontend/rollup.config.js', './frontend/tsconfig.json'],
    skips_local_docker=True,
    live_update=[],
)

# =============================================================================
# BACKEND BUILD
# =============================================================================

# Build in Docker (Dockerfile.dev builder target), extract binary to .build-output/,
# package into minimal runtime image (Dockerfile.runtime), push to registry.
custom_build(
    'kiro/backend',
    'set -e && ' +
    # Step 1: Build in Docker using Dockerfile.dev builder target (BuildKit cache)
    _strip_mk + 'DOCKER_BUILDKIT=1 docker build ' +
        '-t kiro-backend-builder ' +
        '-f ./backend/Dockerfile.dev ' +
        '--target builder ' +
        './backend && ' +
    # Step 2: Extract compiled binary to .build-output/ (small context for runtime image)
    'mkdir -p ./backend/.build-output && ' +
    _strip_mk + 'docker rm -f kiro-extract >/dev/null 2>&1 || true && ' +
    _strip_mk + 'docker create --name kiro-extract kiro-backend-builder && ' +
    _strip_mk + 'docker cp kiro-extract:/tmp/backend ./backend/.build-output/backend && ' +
    _strip_mk + 'docker rm kiro-extract && ' +
    # Step 3: Build minimal runtime image
    _strip_mk + 'docker build -t $EXPECTED_REF -f ./backend/Dockerfile.runtime ./backend && ' +
    # Step 4: Push to registry
    _strip_mk + 'docker push $EXPECTED_REF',
    deps=['./backend/src', './backend/Cargo.toml', './backend/Cargo.lock', './backend/migrations', './backend/.sqlx', './backend/.cargo'],
    skips_local_docker=True,
    live_update=[],
)

# =============================================================================
# KUBERNETES RESOURCES
# =============================================================================

k8s_yaml('./k8s/dev-manifests.yaml')

# =============================================================================
# RESOURCE CONFIGURATION
# =============================================================================

k8s_resource('postgres',
    port_forwards=['15432:5432'],
    labels=['database'],
    resource_deps=[],
    auto_init=True,
    pod_readiness='wait'
)

k8s_resource('redis',
    port_forwards=['16379:6379'],
    labels=['cache'],
    resource_deps=[],
    auto_init=True,
    pod_readiness='wait'
)

k8s_resource('backend',
    port_forwards=['3000:3000'],
    labels=['api'],
    resource_deps=['postgres', 'redis'],
    auto_init=True,
    pod_readiness='wait'
)

k8s_resource('frontend',
    port_forwards=['5000:5000'],
    labels=['web'],
    resource_deps=['backend'],
    auto_init=True,
    pod_readiness='wait'
)

# =============================================================================
# MANUAL TRIGGERS
# =============================================================================

local_resource(
    'db-migrate',
    cmd='kubectl rollout restart deployment/backend -n ' + namespace + ' && kubectl rollout status deployment/backend -n ' + namespace + ' --timeout=120s',
    resource_deps=['backend'],
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['ops']
)

local_resource(
    'db-reset',
    cmd='kubectl exec -n ' + namespace + ' deployment/postgres -- psql -U kiro -d postgres -c "DROP DATABASE IF EXISTS kiro; CREATE DATABASE kiro;"',
    resource_deps=['postgres'],
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['ops']
)

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
# STARTUP INFO
# =============================================================================

print("=" * 60)
print("  No Drake in the House - Minikube Dev Environment")
print("=" * 60)
print("")
print("Services (after pods are ready):")
print("  Backend API:  http://localhost:3000")
print("  Frontend:     http://localhost:5000")
print("  PostgreSQL:   localhost:15432 (user: kiro, pass: password, db: kiro)")
print("  Redis:        localhost:16379")
print("")
print("Registry:       localhost:5001")
print("")
print("First time? Run:")
print("  1. ./scripts/setup-registry.sh   (one-time)")
print("  2. ./scripts/tilt-build.sh       (pre-warm cargo cache)")
print("")
print("Manual triggers: db-migrate, db-reset, backend-tests, frontend-tests")
print("=" * 60)
