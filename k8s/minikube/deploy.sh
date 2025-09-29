#!/bin/bash

set -e

echo "🚀 Deploying Music Blocklist Manager to Minikube"

# Check if minikube is running
if ! minikube status > /dev/null 2>&1; then
    echo "❌ Minikube is not running. Starting minikube..."
    minikube start --driver=docker --memory=4096 --cpus=2
    echo "✅ Minikube started"
else
    echo "✅ Minikube is already running"
fi

# Enable required addons
echo "📦 Enabling minikube addons..."
minikube addons enable ingress
minikube addons enable metrics-server

# Build Docker images
echo "🔨 Building Docker images..."
./k8s/minikube/build-images.sh

# Apply Kubernetes manifests
echo "📋 Applying Kubernetes manifests..."

# Create namespace first
kubectl apply -f k8s/minikube/namespace.yaml

# Apply configurations
kubectl apply -f k8s/minikube/configmap.yaml
kubectl apply -f k8s/minikube/secrets.yaml

# Deploy databases
echo "🗄️  Deploying databases..."
kubectl apply -f k8s/minikube/postgres-deployment.yaml
kubectl apply -f k8s/minikube/redis-deployment.yaml

# Wait for databases to be ready
echo "⏳ Waiting for databases to be ready..."
kubectl wait --for=condition=available --timeout=300s deployment/postgres -n music-blocklist-dev
kubectl wait --for=condition=available --timeout=300s deployment/redis -n music-blocklist-dev

# Run database migrations
echo "🔄 Running database migrations..."
kubectl apply -f k8s/minikube/database-migration-job.yaml
kubectl wait --for=condition=complete --timeout=300s job/database-migration -n music-blocklist-dev

# Deploy application
echo "🚀 Deploying application..."
kubectl apply -f k8s/minikube/api-deployment.yaml
kubectl apply -f k8s/minikube/frontend-deployment.yaml

# Wait for deployments
echo "⏳ Waiting for application to be ready..."
kubectl wait --for=condition=available --timeout=300s deployment/music-blocklist-api -n music-blocklist-dev
kubectl wait --for=condition=available --timeout=300s deployment/music-blocklist-frontend -n music-blocklist-dev

# Get service URLs
echo ""
echo "🎉 Deployment complete!"
echo ""
echo "📊 Service URLs:"
echo "Frontend: http://$(minikube ip):30081"
echo "API: http://$(minikube ip):30080"
echo "Metrics: http://$(minikube ip):30090"
echo ""
echo "🔍 Useful commands:"
echo "kubectl get pods -n music-blocklist-dev"
echo "kubectl logs -f deployment/music-blocklist-api -n music-blocklist-dev"
echo "kubectl logs -f deployment/music-blocklist-frontend -n music-blocklist-dev"
echo "minikube dashboard"
echo ""
echo "🛑 To stop: minikube stop"
echo "🗑️  To clean up: kubectl delete namespace music-blocklist-dev"