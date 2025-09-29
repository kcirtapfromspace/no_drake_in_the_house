#!/bin/bash

set -e

echo "Building Docker images for minikube..."

# Set minikube docker environment
eval $(minikube docker-env)

# Build backend API image
echo "Building backend API image..."
docker build -f docker/api.Dockerfile -t music-blocklist/api:dev .

# Build frontend image
echo "Building frontend image..."
docker build -f docker/frontend.Dockerfile -t music-blocklist/frontend:dev .

# Build migration image
echo "Building migration image..."
docker build -f docker/migration.Dockerfile -t music-blocklist/migration:dev .

echo "Docker images built successfully!"
echo "Available images:"
docker images | grep music-blocklist