# Frontend API Connection Fixes

## Issues Identified

The frontend was experiencing 403 Forbidden errors when trying to access backend API endpoints. The main issues were:

### 1. **Incorrect API URL Configuration**
- **Problem**: Frontend was making requests to `localhost:5000` (frontend server) instead of `localhost:3000` (backend server)
- **Root Cause**: Rollup configuration wasn't loading environment variables from `.env` file properly
- **Symptoms**: 
  ```
  GET http://localhost:5000/api/v1/users/profile 403 (Forbidden)
  POST http://localhost:5000/api/v1/auth/logout 403 (Forbidden)
  ```

### 2. **Missing Logout Endpoint**
- **Problem**: Backend didn't have a logout endpoint defined
- **Root Cause**: Logout handler existed but wasn't added to the route configuration
- **Symptoms**: 404 errors when trying to logout

### 3. **Environment Variable Loading**
- **Problem**: `VITE_API_URL` was defaulting to empty string, causing fallback to `window.location.origin`
- **Root Cause**: Rollup replace plugin wasn't reading `.env` file

## Fixes Applied

### 1. **Fixed Rollup Configuration** (`frontend/rollup.config.js`)

Added environment variable loading from `.env` file:

```javascript
// Load environment variables from .env file
function loadEnv() {
  try {
    const envPath = pathResolve('.env');
    const envFile = readFileSync(envPath, 'utf8');
    const envVars = {};
    
    envFile.split('\n').forEach(line => {
      const [key, ...valueParts] = line.split('=');
      if (key && valueParts.length > 0) {
        const value = valueParts.join('=').trim();
        envVars[key.trim()] = value;
      }
    });
    
    return envVars;
  } catch (error) {
    console.warn('Could not load .env file:', error.message);
    return {};
  }
}
```

Updated replace plugin to use loaded environment variables:

```javascript
'import.meta.env.VITE_API_URL': JSON.stringify(
  process.env.VITE_API_URL || envVars.VITE_API_URL || 'http://localhost:3000'
),
```

### 2. **Fixed Config Fallback** (`frontend/src/lib/utils/config.ts`)

Changed the API URL fallback to always use the backend server:

```typescript
// Before
apiUrl: import.meta.env.VITE_API_URL || (typeof window !== 'undefined' ? window.location.origin : 'http://localhost:3000'),

// After  
apiUrl: import.meta.env.VITE_API_URL || 'http://localhost:3000',
```

### 3. **Added Logout Handler** (`backend/src/handlers/auth.rs`)

```rust
/// Logout user
pub async fn logout_handler(
    State(state): State<AppState>,
    user: crate::models::AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(user_id = %user.id, "User logout attempt");
    
    // Log the logout event
    tracing::info!(user_id = %user.id, "User logged out successfully");
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Logged out successfully"
    })))
}
```

### 4. **Added Logout Route** (`backend/src/lib.rs`)

Added to protected routes:

```rust
// Auth routes (protected)
.route("/auth/logout", post(handlers::auth::logout_handler))
```

## Environment Configuration

The `.env` file should contain:

```env
# API Configuration
VITE_API_URL=http://localhost:3000
VITE_API_VERSION=v1
```

## Expected Behavior After Fixes

1. **Frontend requests go to correct backend URL**: `http://localhost:3000`
2. **Logout endpoint works**: `POST /api/v1/auth/logout` returns 200 OK
3. **Profile endpoint works**: `GET /api/v1/users/profile` returns user data
4. **Authentication flows properly**: JWT tokens are sent and validated correctly

## Testing the Fixes

To test the fixes:

1. **Rebuild the frontend**:
   ```bash
   cd frontend
   npm run build
   npm run dev
   ```

2. **Start the backend**:
   ```bash
   cd backend
   cargo run
   ```

3. **Test API endpoints**:
   - Register a new user
   - Login with the user
   - Access profile endpoint
   - Logout successfully

## Additional Notes

- The CORS configuration already allows `localhost:3000` and `localhost:5000` origins
- Authentication middleware is working correctly
- The issue was purely with frontend API URL configuration
- All backend endpoints are properly protected with authentication middleware

## Future Improvements

1. **Better Error Handling**: Add more specific error messages for API connection issues
2. **Environment Detection**: Automatically detect backend URL in development
3. **Health Checks**: Add frontend health checks to verify backend connectivity
4. **Token Refresh**: Implement automatic token refresh on 401 errors