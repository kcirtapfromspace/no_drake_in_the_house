#!/bin/bash
# Build and push a Render-compatible backend image to GitHub Container Registry.
#
# Defaults:
#   IMAGE_REPO=ghcr.io/kcirtapfromspace/ndith-backend
#   IMAGE_TAG=<git-short-sha>
#   IMAGE_PLATFORM=linux/amd64
#   PUBLISH_LATEST=true
#
# Usage:
#   ./scripts/publish-render-backend-image.sh
#   ./scripts/publish-render-backend-image.sh --tag 20260316-1
#   ./scripts/publish-render-backend-image.sh --repo ghcr.io/myorg/my-backend --no-latest

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

IMAGE_REPO="${IMAGE_REPO:-ghcr.io/kcirtapfromspace/ndith-backend}"
IMAGE_TAG="${IMAGE_TAG:-}"
IMAGE_PLATFORM="${IMAGE_PLATFORM:-linux/amd64}"
BUILDER_NAME="${BUILDER_NAME:-local-multiarch}"
PUBLISH_LATEST="${PUBLISH_LATEST:-true}"
BACKEND_CARGO_FEATURES="${BACKEND_CARGO_FEATURES:-}"
BACKEND_CARGO_NO_DEFAULT_FEATURES="${BACKEND_CARGO_NO_DEFAULT_FEATURES:-false}"

usage() {
  cat <<'EOF'
Usage: ./scripts/publish-render-backend-image.sh [options]

Options:
  --repo <image-repo>     Full image repository (default: ghcr.io/kcirtapfromspace/ndith-backend)
  --tag <tag>             Image tag (default: git short SHA)
  --platform <platform>   Build platform (default: linux/amd64)
  --builder <name>        buildx builder to use (default: local-multiarch)
  --features <features>   Cargo features to pass to the backend image build
  --no-default-features   Disable backend crate default features during image build
  --no-latest             Do not also publish :latest
  --help                  Show this help
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      IMAGE_REPO="${2:?missing value for --repo}"
      shift 2
      ;;
    --tag)
      IMAGE_TAG="${2:?missing value for --tag}"
      shift 2
      ;;
    --platform)
      IMAGE_PLATFORM="${2:?missing value for --platform}"
      shift 2
      ;;
    --builder)
      BUILDER_NAME="${2:?missing value for --builder}"
      shift 2
      ;;
    --features)
      BACKEND_CARGO_FEATURES="${2:?missing value for --features}"
      shift 2
      ;;
    --no-default-features)
      BACKEND_CARGO_NO_DEFAULT_FEATURES=true
      shift
      ;;
    --no-latest)
      PUBLISH_LATEST=false
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ -z "${IMAGE_TAG}" ]]; then
  IMAGE_TAG="$(git rev-parse --short HEAD)"
fi

IMAGE_REF="${IMAGE_REPO}:${IMAGE_TAG}"
BUILD_ARGS=(
  --builder "${BUILDER_NAME}"
  --platform "${IMAGE_PLATFORM}"
  --file ./backend/Dockerfile
  --tag "${IMAGE_REF}"
  --build-arg "BACKEND_CARGO_FEATURES=${BACKEND_CARGO_FEATURES}"
  --build-arg "BACKEND_CARGO_NO_DEFAULT_FEATURES=${BACKEND_CARGO_NO_DEFAULT_FEATURES}"
)

if [[ "${PUBLISH_LATEST}" == "true" ]]; then
  BUILD_ARGS+=(--tag "${IMAGE_REPO}:latest")
fi

echo "=========================================="
echo "  Render Backend Image Publish"
echo "=========================================="
echo "repo=${IMAGE_REPO}"
echo "tag=${IMAGE_TAG}"
echo "platform=${IMAGE_PLATFORM}"
echo "builder=${BUILDER_NAME}"
echo "publish_latest=${PUBLISH_LATEST}"
echo "cargo_features=${BACKEND_CARGO_FEATURES:-<none>}"
echo "cargo_no_default_features=${BACKEND_CARGO_NO_DEFAULT_FEATURES}"

docker buildx inspect "${BUILDER_NAME}" >/dev/null

DOCKER_BUILDKIT=1 docker buildx build \
  "${BUILD_ARGS[@]}" \
  --push \
  ./backend

echo ""
echo "Published:"
echo "  ${IMAGE_REF}"
if [[ "${PUBLISH_LATEST}" == "true" ]]; then
  echo "  ${IMAGE_REPO}:latest"
fi
