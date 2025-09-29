# CI/CD Pipeline Setup

This document describes the comprehensive CI/CD pipeline for the No Drake in the House platform.

## Overview

The CI/CD pipeline is built using GitHub Actions and provides:

- **Automated Testing**: Unit and integration tests for both backend and frontend
- **Security Scanning**: Vulnerability scanning and dependency auditing
- **Docker Image Building**: Multi-platform container images with proper tagging
- **Automated Deployment**: Staging and production deployments using Kubernetes
- **Release Management**: Automated releases with changelog generation
- **Maintenance Tasks**: Dependency updates and cleanup automation

## Workflows

### 1. CI/CD Pipeline (`ci-cd.yml`)

**Triggers**: Push to `main`/`develop`, Pull Requests

**Jobs**:
- `test-backend`: Rust tests with PostgreSQL and Redis
- `test-frontend`: TypeScript/Svelte tests and build
- `security-scan`: Trivy vulnerability scanning
- `build-images`: Docker image building and registry push
- `deploy-staging`: Automatic staging deployment (develop branch)
- `deploy-production`: Automatic production deployment (main branch)
- `notify`: Pipeline status notifications

### 2. Pull Request Validation (`pr-validation.yml`)

**Triggers**: Pull Request events

**Jobs**:
- `validate-pr`: PR title validation, breaking change detection
- `size-check`: PR size analysis and labeling
- `dependency-review`: Security review of dependency changes

### 3. Release Management (`release.yml`)

**Triggers**: Git tags (`v*`)

**Jobs**:
- `create-release`: GitHub release creation with changelog
- `build-release-images`: Multi-platform release images
- `update-helm-chart`: Helm chart versioning and packaging
- `notify-release`: Release status notifications

### 4. Maintenance (`maintenance.yml`)

**Triggers**: Daily schedule (2 AM UTC), Manual dispatch

**Jobs**:
- `dependency-updates`: Automated dependency update detection
- `security-audit`: Daily security vulnerability scanning
- `cleanup-artifacts`: Cleanup old workflow runs and container images

## Setup Instructions

### 1. Repository Configuration

#### Required Secrets

Configure these secrets in GitHub repository settings:

```bash
# Kubernetes Configuration (base64 encoded)
KUBE_CONFIG_STAGING=<base64-encoded-kubeconfig-for-staging>
KUBE_CONFIG_PRODUCTION=<base64-encoded-kubeconfig-for-production>

# Container Registry (automatically provided by GitHub)
GITHUB_TOKEN=<automatically-provided>
```

#### Environment Configuration

Create environments in GitHub repository settings:

- **staging**: Requires approval from maintainers
- **production**: Requires approval from admin team

### 2. Branch Protection

Configure branch protection for `main`:

```bash
# Using GitHub CLI
gh api repos/:owner/:repo/branches/main/protection \
  --method PUT \
  --field required_status_checks='{"strict":true,"contexts":["test-backend","test-frontend"]}' \
  --field enforce_admins=true \
  --field required_pull_request_reviews='{"required_approving_review_count":2}' \
  --field restrictions=null
```

### 3. Container Registry Setup

The pipeline uses GitHub Container Registry (ghcr.io). Images are automatically pushed to:

- `ghcr.io/your-org/kiro/backend`
- `ghcr.io/your-org/kiro/frontend`

### 4. Kubernetes Cluster Setup

#### Staging Cluster

```bash
# Create namespace
kubectl create namespace kiro-staging

# Install cert-manager for TLS
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml

# Create cluster issuer for staging
kubectl apply -f - <<EOF
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-staging
spec:
  acme:
    server: https://acme-staging-v02.api.letsencrypt.org/directory
    email: admin@nodrakeinthe.house
    privateKeySecretRef:
      name: letsencrypt-staging
    solvers:
    - http01:
        ingress:
          class: nginx
EOF
```

#### Production Cluster

```bash
# Create namespace
kubectl create namespace kiro-production

# Create cluster issuer for production
kubectl apply -f - <<EOF
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: admin@nodrakeinthe.house
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
EOF
```

## Image Tagging Strategy

### Development Images

- `develop-latest`: Latest build from develop branch
- `develop-<sha>`: Specific commit from develop branch
- `pr-<number>`: Pull request builds

### Release Images

- `latest`: Latest stable release
- `v1.2.3`: Specific version tag
- `v1.2`: Minor version tag
- `v1`: Major version tag

## Deployment Strategy

### Staging Environment

- **Trigger**: Push to `develop` branch
- **Strategy**: Rolling update
- **Approval**: Automatic
- **Health Checks**: Basic smoke tests

### Production Environment

- **Trigger**: Push to `main` branch
- **Strategy**: Blue-green deployment
- **Approval**: Manual (admin team)
- **Health Checks**: Comprehensive validation

## Monitoring and Notifications

### Pipeline Status

Monitor pipeline status through:

- GitHub Actions UI
- Commit status checks
- PR comments with validation results

### Failure Notifications

Pipeline failures trigger:

- GitHub issue creation for security vulnerabilities
- PR comments for validation failures
- Workflow run annotations for debugging

## Local Testing

### Test Workflows Locally

Using [act](https://github.com/nektos/act):

```bash
# Install act
brew install act  # macOS
# or
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# Test backend workflow
act -j test-backend

# Test frontend workflow
act -j test-frontend

# Test full CI pipeline
act push
```

### Validate Setup

```bash
# Run validation script
./scripts/validate-cicd.sh

# Check Docker builds
docker build -t kiro/backend ./backend
docker build -t kiro/frontend ./frontend

# Test Helm charts
helm lint ./helm
helm template kiro ./helm --values helm/values-staging.yaml
```

## Troubleshooting

### Common Issues

#### 1. Docker Build Failures

```bash
# Check Dockerfile syntax
docker build --no-cache -t test ./backend

# Debug build context
docker build --progress=plain -t test ./backend
```

#### 2. Kubernetes Deployment Failures

```bash
# Check deployment status
kubectl get pods -n kiro-staging
kubectl describe deployment kiro-backend -n kiro-staging

# Check logs
kubectl logs -f deployment/kiro-backend -n kiro-staging
```

#### 3. Helm Chart Issues

```bash
# Validate chart
helm lint ./helm

# Debug template rendering
helm template kiro ./helm --values helm/values-staging.yaml --debug

# Check release status
helm status kiro-staging -n kiro-staging
```

### Debug Commands

```bash
# Check workflow runs
gh run list --limit 10

# View workflow logs
gh run view <run-id> --log

# Check repository secrets
gh secret list

# Validate branch protection
gh api repos/:owner/:repo/branches/main/protection
```

## Security Considerations

### Container Security

- Images built from minimal base images
- Non-root user execution
- Security scanning with Trivy
- Regular base image updates

### Secrets Management

- Kubernetes secrets for sensitive data
- GitHub secrets for CI/CD credentials
- No secrets in container images or logs

### Network Security

- Network policies in production
- TLS termination at ingress
- Internal service communication encryption

## Performance Optimization

### Build Optimization

- Multi-stage Docker builds
- Layer caching with GitHub Actions cache
- Parallel job execution
- Dependency caching for Rust and Node.js

### Deployment Optimization

- Rolling updates with health checks
- Resource limits and requests
- Horizontal pod autoscaling
- Pod disruption budgets

## Maintenance

### Regular Tasks

- Weekly dependency updates review
- Monthly security audit review
- Quarterly pipeline optimization
- Annual disaster recovery testing

### Automated Maintenance

- Daily security scans
- Weekly artifact cleanup
- Monthly dependency update notifications
- Quarterly performance reports