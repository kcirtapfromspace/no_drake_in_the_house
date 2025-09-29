# Development Kubernetes Manifests

This directory contains simple Kubernetes manifests used by Tilt for local development.

## Files

- **`namespace.yaml`** - Creates the `kiro-dev` namespace
- **`postgres.yaml`** - PostgreSQL database deployment and service
- **`redis.yaml`** - Redis cache deployment and service  
- **`backend.yaml`** - Backend API deployment and service
- **`frontend.yaml`** - Frontend web app deployment and service

## Usage

These manifests are automatically deployed by Tilt when you run:

```bash
tilt up
```

## Configuration

The manifests are configured for development with:

- **PostgreSQL**: 
  - Database: `kiro`
  - User: `kiro` 
  - Password: `password`
  - Port: 5432

- **Redis**:
  - No authentication
  - Port: 6379

- **Backend**:
  - Environment variables for database and Redis connections
  - Debug logging enabled
  - Port: 3000

- **Frontend**:
  - Nginx serving static files
  - Port: 5000 (mapped to 80 in service)

## Development Notes

- All services use `emptyDir` volumes (data is lost when pods restart)
- Images are built locally by Tilt and pushed to minikube's Docker daemon
- Services are automatically port-forwarded by Tilt for local access