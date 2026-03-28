#!/bin/bash
# Pre-build images for Tilt/Minikube development
# Run this BEFORE 'tilt up' to pre-warm the cargo cache and push images to the
# local registry so the first tilt up is fast.
#
# Prerequisites:
#   ./scripts/setup-registry.sh   (one-time: starts registry + configures minikube)
set -euo pipefail

REGISTRY="localhost:5001"
BACKEND_IMAGE="${REGISTRY}/ndith_backend"
FRONTEND_IMAGE="${REGISTRY}/ndith_frontend"

echo "=========================================="
echo "  Pre-building images for Tilt"
echo "=========================================="

# Verify registry is running
echo ""
echo "Checking registry at ${REGISTRY}..."
if ! curl -sf http://${REGISTRY}/v2/ >/dev/null 2>&1; then
    echo "ERROR: Registry not running at ${REGISTRY}"
    echo "Run ./scripts/setup-registry.sh first."
    exit 1
fi
echo "  Registry: OK"

# ---- Backend: Build in Docker, extract binary, build runtime image ----
echo ""
echo "[1/2] Building backend (may take 10-17 min first time, ~2-5 min warm)..."

# Step 1: Build using Dockerfile.dev builder target (cargo cache in BuildKit layers)
echo "  Building Rust binary in Docker..."
DOCKER_BUILDKIT=1 docker build \
    -t ndith-backend-builder \
    -f ./backend/Dockerfile.dev \
    --target builder \
    ./backend

# Step 2: Extract compiled binary
echo "  Extracting binary..."
mkdir -p ./backend/.build-output
docker rm -f ndith-extract >/dev/null 2>&1 || true
docker create --name ndith-extract ndith-backend-builder
docker cp ndith-extract:/tmp/backend ./backend/.build-output/backend
docker rm ndith-extract

# Step 3: Build minimal runtime image
echo "  Building runtime image..."
docker build -t "${BACKEND_IMAGE}:latest" -f ./backend/Dockerfile.runtime ./backend

# Step 4: Push to registry
echo "  Pushing to registry..."
docker push "${BACKEND_IMAGE}:latest"

echo "  Backend: DONE"

# ---- Frontend: Build on host, build runtime image ----
echo ""
echo "[2/2] Building frontend..."

# Step 1: Build on host
echo "  Running npm build..."
(cd frontend && npm run build)

# Step 2: Build minimal runtime image
echo "  Building runtime image..."
docker build -t "${FRONTEND_IMAGE}:latest" -f ./frontend/Dockerfile.runtime ./frontend

# Step 3: Push to registry
echo "  Pushing to registry..."
docker push "${FRONTEND_IMAGE}:latest"

echo "  Frontend: DONE"

echo ""
echo "=========================================="
echo "  Pre-build complete!"
echo "=========================================="
echo ""
echo "Images pushed to registry at ${REGISTRY}:"
echo "  ${BACKEND_IMAGE}:latest"
echo "  ${FRONTEND_IMAGE}:latest"
echo ""
echo "Run 'tilt up' to start the environment."
