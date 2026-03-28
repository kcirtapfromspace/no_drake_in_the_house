#!/bin/bash
# One-time setup: local Docker registry + minikube with insecure-registry trust
#
# What this does:
#   1. Starts a standalone registry:2 container on localhost:5001
#   2. Recreates minikube with --insecure-registry to trust the local registry
#   3. Creates the ndith-dev namespace
#   4. Verifies connectivity: host → registry, minikube → registry
#
# After this, run ./scripts/tilt-build.sh to pre-warm the cargo cache,
# then 'tilt up' to start the dev environment.
set -euo pipefail

REGISTRY_NAME="registry"
REGISTRY_PORT=5001
NAMESPACE="ndith-dev"

echo "=========================================="
echo "  Local Registry + Minikube Setup"
echo "=========================================="

# ---- Step 1: Start or ensure local registry ----
echo ""
echo "[1/4] Setting up local Docker registry on port ${REGISTRY_PORT}..."

if docker inspect "${REGISTRY_NAME}" >/dev/null 2>&1; then
    state=$(docker inspect -f '{{.State.Running}}' "${REGISTRY_NAME}" 2>/dev/null)
    if [ "$state" = "true" ]; then
        echo "  Registry '${REGISTRY_NAME}' is already running on port ${REGISTRY_PORT}."
    else
        echo "  Registry container exists but is stopped. Starting..."
        docker start "${REGISTRY_NAME}"
        echo "  Registry started."
    fi
else
    echo "  Creating registry container..."
    docker run -d \
        --name "${REGISTRY_NAME}" \
        --restart always \
        -p "${REGISTRY_PORT}:5000" \
        -v ndith-registry-data:/var/lib/registry \
        registry:2
    echo "  Registry created and running."
fi

# Verify host can reach registry
echo "  Verifying host → registry connectivity..."
if curl -sf http://localhost:${REGISTRY_PORT}/v2/ >/dev/null 2>&1; then
    echo "  Host → registry: OK"
else
    echo "  ERROR: Cannot reach registry at localhost:${REGISTRY_PORT}"
    echo "  Check: docker logs ${REGISTRY_NAME}"
    exit 1
fi

# ---- Step 2: Recreate minikube with insecure-registry ----
echo ""
echo "[2/4] Setting up minikube with insecure-registry..."

# Check if minikube exists and get current config
NEEDS_RECREATE=false
if minikube status >/dev/null 2>&1; then
    # Check if insecure-registry is already configured
    if minikube ssh -- "grep -q 'host.minikube.internal:${REGISTRY_PORT}' /etc/docker/daemon.json 2>/dev/null"; then
        echo "  Minikube already configured with insecure-registry. Skipping recreate."
    else
        echo "  Minikube exists but lacks insecure-registry config."
        NEEDS_RECREATE=true
    fi
else
    NEEDS_RECREATE=true
fi

if [ "$NEEDS_RECREATE" = true ]; then
    # Capture current driver and memory settings if minikube exists
    DRIVER="docker"
    MEMORY="4096"
    CPUS="4"
    if minikube status >/dev/null 2>&1; then
        DRIVER=$(minikube profile list -o json 2>/dev/null | python3 -c "import sys,json; profiles=json.load(sys.stdin).get('valid',[]);print(profiles[0]['Config']['Driver'] if profiles else 'docker')" 2>/dev/null || echo "docker")
        echo "  Current driver: ${DRIVER}"
        echo "  Deleting existing minikube to reconfigure..."
        minikube delete
    fi

    echo "  Creating minikube with insecure-registry (driver=${DRIVER}, memory=${MEMORY}, cpus=${CPUS})..."
    minikube start \
        --driver="${DRIVER}" \
        --memory="${MEMORY}" \
        --cpus="${CPUS}" \
        --insecure-registry="host.minikube.internal:${REGISTRY_PORT}" \
        --addons=default-storageclass,storage-provisioner
    echo "  Minikube created."
fi

# ---- Step 3: Create namespace ----
echo ""
echo "[3/4] Ensuring namespace '${NAMESPACE}' exists..."
if kubectl get namespace "${NAMESPACE}" >/dev/null 2>&1; then
    echo "  Namespace '${NAMESPACE}' already exists."
else
    kubectl create namespace "${NAMESPACE}"
    echo "  Namespace '${NAMESPACE}' created."
fi

# ---- Step 4: Verify minikube → registry connectivity ----
echo ""
echo "[4/4] Verifying minikube → registry connectivity..."
if minikube ssh -- "curl -sf http://host.minikube.internal:${REGISTRY_PORT}/v2/" >/dev/null 2>&1; then
    echo "  Minikube → registry: OK"
else
    echo "  WARNING: Minikube cannot reach registry at host.minikube.internal:${REGISTRY_PORT}"
    echo "  This may work once pods start (host.minikube.internal resolves inside k8s)."
    echo "  If images fail to pull, check: minikube ssh -- curl http://host.minikube.internal:${REGISTRY_PORT}/v2/"
fi

echo ""
echo "=========================================="
echo "  Setup complete!"
echo "=========================================="
echo ""
echo "Next steps:"
echo "  1. ./scripts/tilt-build.sh   # Pre-warm cargo cache (first time only)"
echo "  2. tilt up                    # Start dev environment"
echo ""
echo "Registry: localhost:${REGISTRY_PORT}"
echo "Minikube: $(minikube ip 2>/dev/null || echo 'starting...')"
