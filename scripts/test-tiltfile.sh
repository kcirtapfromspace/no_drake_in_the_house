#!/bin/bash

# Simple Tiltfile syntax test
echo "Testing Tiltfile syntax..."

if command -v tilt >/dev/null 2>&1; then
    # Check if required files exist
    if [[ ! -f "Tiltfile" ]]; then
        echo "❌ Tiltfile not found"
        exit 1
    fi
    
    # Check if k8s manifests exist
    if [[ ! -d "k8s/dev" ]]; then
        echo "❌ k8s/dev directory not found"
        exit 1
    fi
    
    required_files=(
        "k8s/dev/namespace.yaml"
        "k8s/dev/postgres.yaml"
        "k8s/dev/redis.yaml"
        "k8s/dev/backend.yaml"
        "k8s/dev/frontend.yaml"
    )
    
    for file in "${required_files[@]}"; do
        if [[ ! -f "$file" ]]; then
            echo "❌ Required file not found: $file"
            exit 1
        fi
    done
    
    echo "✅ Tiltfile and required manifests found"
    echo "✅ Basic validation passed"
    echo ""
    echo "To test fully, run: tilt up"
    exit 0
else
    echo "⚠️  Tilt not installed, cannot validate syntax"
    echo "Install Tilt from: https://docs.tilt.dev/install.html"
    exit 1
fi