# Registration Flow Setup and Troubleshooting Guide

## Overview
This guide helps resolve the 409 Conflict error and ensures proper communication between the Svelte frontend and Rust backend for the enhanced registration flow.

## Configuration Requirements

### For Tilt/Minikube Development

The configuration is handled in the Kubernetes manifests (`k8s/dev-manifests.yaml`):

**Frontend Container Environment:**
```yaml
env:
- name: VITE_API_URL
  value: http://localhost:3000  # Port-forwarded backend URL
- name: VITE_API_VERSION
  value: v1
```

**Backend Container Environment:**
```yaml
env:
- name: CORS_ALLOWED_ORIGINS
  value: http://localhost:5000,http://localhost:5173,http://localhost:3000
- name: AUTO_LOGIN_ENABLED
  value: "true"
```

### For Local Development (Non-Kubernetes)

#### Frontend Configuration (`frontend/.env`)
```env
# API Configuration - CRITICAL: Must point to backend
VITE_API_URL=http://localhost:3000
VITE_API_VERSION=v1

# Other configuration...
NODE_ENV=development
VITE_ENVIRONMENT=development
```

### Backend Configuration (`backend/.env`)
```env
# Environment Configuration
ENVIRONMENT=development

# Database Configuration
DATABASE_URL=postgres://kiro:kiro_dev_password@localhost:5432/kiro_dev

# Server Configuration
SERVER_ADDRESS=0.0.0.0:3000

# CORS Configuration - CRITICAL: Must include frontend origin
CORS_ALLOWED_ORIGINS=http://localhost:5000,http://localhost:5173,http://localhost:3000

# Auto-login Configuration
AUTO_LOGIN_ENABLED=true

# Other configuration...
JWT_SECRET=dev-secret-key-not-for-production-this-should-be-at-least-32-characters-long
REDIS_URL=redis://localhost:6379
```

## Common Issues and Solutions

### 1. 409 Conflict Error - Wrong API URL
**Problem**: Frontend sends requests to `localhost:5000` instead of `localhost:3000`

**Solution**: 
- Ensure `VITE_API_URL=http://localhost:3000` in `frontend/.env`
- Restart the frontend dev server after changing environment variables
- Clear browser cache and localStorage

### 2. CORS Errors
**Problem**: Browser blocks requests due to CORS policy

**Solution**:
- Ensure `CORS_ALLOWED_ORIGINS` includes your frontend URL in `backend/.env`
- Common frontend ports: `5000`, `5173` (Vite), `3000`
- Restart the backend server after changing CORS configuration

### 3. Email Already Exists (409 from backend)
**Problem**: Legitimate 409 error when email is already registered

**Solution**:
- Use a different email address for testing
- Or clear the database: `DELETE FROM users WHERE email = 'test@example.com';`

### 4. Tilt/Minikube Specific Issues

**Problem**: Frontend still sends requests to localhost:5000 instead of localhost:3000

**Root Cause**: The frontend container environment variable `VITE_API_URL` is set to the internal Kubernetes service URL (`http://backend:3000`) instead of the port-forwarded URL.

**Solution**:
1. Update `k8s/dev-manifests.yaml` frontend environment:
   ```yaml
   - name: VITE_API_URL
     value: http://localhost:3000  # NOT http://backend:3000
   ```
2. Restart Tilt: `tilt down && tilt up`
3. Wait for frontend pod to rebuild and restart

**Problem**: CORS errors in Tilt/Minikube setup

**Solution**:
1. Ensure backend has CORS configuration in `k8s/dev-manifests.yaml`:
   ```yaml
   - name: CORS_ALLOWED_ORIGINS
     value: http://localhost:5000,http://localhost:5173,http://localhost:3000
   ```
2. Restart backend pod in Tilt

**Problem**: Services not accessible via port forwarding

**Solution**:
1. Check Tilt UI for port forwarding status
2. Verify Minikube is running: `minikube status`
3. Check kubectl context: `kubectl config current-context`
4. Restart port forwarding: `tilt down && tilt up`

## Enhanced Registration Flow Features

### Backend Validation
- ✅ Password confirmation matching
- ✅ Terms acceptance validation
- ✅ Enhanced email format validation with regex
- ✅ Password strength requirements (8+ chars, uppercase, lowercase, number, special char)
- ✅ Common password detection
- ✅ Structured error responses with field-specific errors

### Frontend Features
- ✅ Real-time validation feedback
- ✅ Password strength indicator
- ✅ Terms acceptance checkbox
- ✅ Field-specific error display
- ✅ Auto-login after successful registration
- ✅ Loading states and submission prevention

### Auto-Login Flow
1. User submits valid registration form
2. Backend validates all fields
3. Backend creates user account
4. Backend automatically generates JWT tokens
5. Backend returns `AuthResponse` with tokens and user profile
6. Frontend stores tokens and updates authentication state
7. Frontend redirects to dashboard

## API Request/Response Format

### Registration Request
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123!",
  "confirm_password": "SecurePassword123!",
  "terms_accepted": true
}
```

### Success Response (200)
```json
{
  "user": {
    "id": "uuid",
    "email": "user@example.com",
    "email_verified": false,
    "totp_enabled": false,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z",
    "settings": {
      "two_factor_enabled": false,
      "email_notifications": true,
      "privacy_mode": false
    }
  },
  "access_token": "jwt_access_token",
  "refresh_token": "jwt_refresh_token"
}
```

### Validation Error Response (400)
```json
{
  "error": "Registration validation failed",
  "errors": [
    {
      "field": "password",
      "message": "Password must contain at least 8 characters, at least one uppercase letter, at least one number",
      "code": "PASSWORD_WEAK"
    },
    {
      "field": "confirm_password", 
      "message": "Password confirmation does not match",
      "code": "PASSWORD_MISMATCH"
    }
  ]
}
```

### Email Already Exists Response (409)
```json
{
  "error": "An account with this email address already exists",
  "code": "EMAIL_ALREADY_REGISTERED"
}
```

## Testing the Setup

### For Tilt/Minikube Development

```bash
# Start Tilt (this will build and deploy everything)
tilt up

# Wait for all services to be ready in Tilt UI
# Services will be available at:
# - Frontend: http://localhost:5000
# - Backend: http://localhost:3000
# - PostgreSQL: localhost:5432
# - Redis: localhost:6379
```

### For Local Development (Non-Kubernetes)

```bash
# Terminal 1: Start PostgreSQL and Redis
docker-compose up -d postgres redis

# Terminal 2: Start backend
cd backend
cargo run

# Terminal 3: Start frontend  
cd frontend
npm run dev
```

### 2. Test Registration
1. Open `http://localhost:5000` in browser
2. Navigate to registration form
3. Fill out form with valid data
4. Check browser network tab - requests should go to `localhost:3000`
5. Successful registration should auto-login and redirect

### 3. Verify Configuration
- Backend logs should show CORS configuration validation
- Frontend should make requests to `http://localhost:3000/api/v1/auth/register`
- No CORS errors in browser console
- Auto-login should work and redirect to dashboard

## Troubleshooting Commands

### Check Environment Variables
```bash
# Frontend
cd frontend && npm run dev -- --debug

# Backend  
cd backend && cargo run
```

### Clear Test Data
```sql
-- Connect to PostgreSQL
psql -h localhost -U kiro -d kiro_dev

-- Clear test users
DELETE FROM users WHERE email LIKE '%test%' OR email LIKE '%example%';
```

### Reset Configuration
```bash
# Restart both servers after config changes
# Frontend
cd frontend && npm run dev

# Backend
cd backend && cargo run
```

## Security Notes

- The current configuration is for development only
- Production should use HTTPS origins only
- JWT secrets should be properly generated for production
- Rate limiting is implemented for registration endpoint
- All passwords are hashed with bcrypt (cost 12)
- Audit logging tracks all registration attempts