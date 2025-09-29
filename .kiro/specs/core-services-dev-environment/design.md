# Design Document

## Overview

This design focuses on establishing a robust foundation for the No Drake in the House platform with core backend services and a complete operational development environment. The architecture emphasizes developer productivity, system reliability, and scalability while keeping the initial scope manageable.

The system follows a modular monolith approach that can evolve into microservices as needed, with clear service boundaries and well-defined interfaces. The development environment supports both local development with Docker Compose and Kubernetes development with Skaffold for production-like testing.

## Architecture

### High-Level System Architecture

```mermaid
graph TB
    subgraph "Development Environment"
        LocalDev[Local Development<br/>Docker Compose]
        K8sDev[Kubernetes Development<br/>Skaffold + Tilt]
    end
    
    subgraph "Frontend Layer"
        WebApp[Web Application<br/>Svelte + TypeScript]
    end
    
    subgraph "API Layer"
        Gateway[API Gateway<br/>Rate Limiting + CORS]
        API[REST API<br/>Rust (Axum)]
    end
    
    subgraph "Core Services"
        Auth[Authentication Service<br/>JWT + 2FA]
        DNP[DNP List Service<br/>Artist Management]
        User[User Service<br/>Profile Management]
    end
    
    subgraph "Data Layer"
        PostgreSQL[(PostgreSQL<br/>Primary Database)]
        Redis[(Redis<br/>Sessions + Cache)]
        DuckDB[(DuckDB<br/>Analytics)]
    end
    
    subgraph "Infrastructure"
        Monitoring[Monitoring<br/>Prometheus + Grafana]
        Logging[Logging<br/>Structured JSON]
        Health[Health Checks<br/>Kubernetes Probes]
    end
    
    WebApp --> Gateway
    Gateway --> API
    API --> Auth
    API --> DNP
    API --> User
    Auth --> PostgreSQL
    Auth --> Redis
    DNP --> PostgreSQL
    User --> PostgreSQL
    API --> DuckDB
    API --> Monitoring
    API --> Logging
    API --> Health
```

### Development Environment Architecture

```mermaid
graph LR
    subgraph "Local Development"
        Dev[Developer]
        Make[Makefile Commands]
        Docker[Docker Compose]
        HotReload[Hot Reloading]
    end
    
    subgraph "Kubernetes Development"
        Skaffold[Skaffold]
        Tilt[Tilt Dashboard]
        K8sCluster[Local K8s Cluster]
        Helm[Helm Charts]
    end
    
    subgraph "CI/CD Pipeline"
        GitHub[GitHub Actions]
        Tests[Automated Tests]
        Build[Docker Build]
        Deploy[K8s Deploy]
    end
    
    Dev --> Make
    Make --> Docker
    Make --> Skaffold
    Docker --> HotReload
    Skaffold --> Tilt
    Skaffold --> K8sCluster
    K8sCluster --> Helm
    GitHub --> Tests
    Tests --> Build
    Build --> Deploy
```

## Components and Interfaces

### Core API Service (Rust)

The main API service is built as a modular monolith using Axum, with clear service boundaries that can be extracted into separate services later.

```rust
// Main application structure
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: RedisPool,
    pub duck_db: Arc<DuckDbConnection>,
    pub auth_service: Arc<AuthService>,
    pub dnp_service: Arc<DnpService>,
    pub user_service: Arc<UserService>,
    pub config: Arc<Config>,
}

// Service trait for consistent interface
#[async_trait]
pub trait Service {
    type Error: Into<AppError>;
    
    async fn health_check(&self) -> Result<HealthStatus, Self::Error>;
}

// Main router setup
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .nest("/api/v1/auth", auth_routes())
        .nest("/api/v1/users", user_routes())
        .nest("/api/v1/dnp", dnp_routes())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(RateLimitLayer::new())
        .with_state(state)
}
```

### Authentication Service

Handles user registration, login, JWT token management, and 2FA.

```rust
pub struct AuthService {
    db: PgPool,
    redis: RedisPool,
    jwt_secret: String,
    totp_issuer: String,
}

impl AuthService {
    pub async fn register(&self, request: RegisterRequest) -> Result<AuthResponse, AuthError> {
        // Hash password with bcrypt (12 rounds minimum)
        let password_hash = bcrypt::hash(&request.password, 12)?;
        
        // Create user record
        let user = sqlx::query_as!(
            User,
            "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING *",
            request.email,
            password_hash
        )
        .fetch_one(&self.db)
        .await?;
        
        // Generate JWT tokens
        let (access_token, refresh_token) = self.generate_tokens(&user).await?;
        
        Ok(AuthResponse {
            user: user.into(),
            access_token,
            refresh_token,
        })
    }
    
    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse, AuthError> {
        // Rate limiting check
        self.check_rate_limit(&request.email).await?;
        
        // Verify credentials
        let user = self.verify_credentials(&request.email, &request.password).await?;
        
        // Check 2FA if enabled
        if user.totp_secret.is_some() {
            self.verify_totp(&user, request.totp_code.ok_or(AuthError::TotpRequired)?).await?;
        }
        
        // Generate tokens
        let (access_token, refresh_token) = self.generate_tokens(&user).await?;
        
        Ok(AuthResponse {
            user: user.into(),
            access_token,
            refresh_token,
        })
    }
    
    pub async fn setup_2fa(&self, user_id: Uuid) -> Result<TotpSetupResponse, AuthError> {
        let secret = totp_lite::totp_custom::<Sha1>(30, 6, &generate_secret(), 0);
        let qr_code_url = format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}",
            self.totp_issuer,
            user_id,
            secret,
            self.totp_issuer
        );
        
        // Store temporary secret (not activated until verified)
        self.store_temp_totp_secret(user_id, &secret).await?;
        
        Ok(TotpSetupResponse {
            secret,
            qr_code_url,
        })
    }
}
```

### DNP List Service

Manages personal Do-Not-Play lists with artist search and management.

```rust
pub struct DnpService {
    db: PgPool,
    duck_db: Arc<DuckDbConnection>,
}

impl DnpService {
    pub async fn search_artists(&self, query: &str, limit: i32) -> Result<Vec<Artist>, DnpError> {
        // Search in local artist database
        let artists = sqlx::query_as!(
            Artist,
            r#"
            SELECT id, canonical_name, external_ids, metadata, created_at
            FROM artists 
            WHERE canonical_name ILIKE $1 
               OR aliases ? $2
            ORDER BY 
                CASE WHEN canonical_name ILIKE $3 THEN 1 ELSE 2 END,
                canonical_name
            LIMIT $4
            "#,
            format!("%{}%", query),
            query,
            format!("{}%", query),
            limit
        )
        .fetch_all(&self.db)
        .await?;
        
        Ok(artists)
    }
    
    pub async fn add_to_dnp_list(
        &self, 
        user_id: Uuid, 
        artist_id: Uuid, 
        tags: Option<Vec<String>>,
        note: Option<String>
    ) -> Result<DnpEntry, DnpError> {
        // Check for duplicates
        let existing = sqlx::query!(
            "SELECT 1 FROM user_artist_blocks WHERE user_id = $1 AND artist_id = $2",
            user_id,
            artist_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        if existing.is_some() {
            return Err(DnpError::DuplicateEntry);
        }
        
        // Add to DNP list
        let entry = sqlx::query_as!(
            DnpEntry,
            r#"
            INSERT INTO user_artist_blocks (user_id, artist_id, tags, note)
            VALUES ($1, $2, $3, $4)
            RETURNING user_id, artist_id, tags, note, created_at
            "#,
            user_id,
            artist_id,
            tags.as_deref(),
            note
        )
        .fetch_one(&self.db)
        .await?;
        
        // Update analytics
        self.update_dnp_analytics(user_id).await?;
        
        Ok(entry)
    }
    
    pub async fn get_dnp_list(&self, user_id: Uuid) -> Result<Vec<DnpEntryWithArtist>, DnpError> {
        let entries = sqlx::query_as!(
            DnpEntryWithArtist,
            r#"
            SELECT 
                b.user_id,
                b.artist_id,
                b.tags,
                b.note,
                b.created_at,
                a.canonical_name,
                a.external_ids,
                a.metadata
            FROM user_artist_blocks b
            JOIN artists a ON b.artist_id = a.id
            WHERE b.user_id = $1
            ORDER BY b.created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.db)
        .await?;
        
        Ok(entries)
    }
}
```

### User Service

Manages user profiles and preferences.

```rust
pub struct UserService {
    db: PgPool,
}

impl UserService {
    pub async fn get_profile(&self, user_id: Uuid) -> Result<UserProfile, UserError> {
        let user = sqlx::query_as!(
            UserProfile,
            "SELECT id, email, created_at, updated_at, settings FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(UserError::NotFound)?;
        
        Ok(user)
    }
    
    pub async fn update_settings(
        &self, 
        user_id: Uuid, 
        settings: UserSettings
    ) -> Result<UserProfile, UserError> {
        let user = sqlx::query_as!(
            UserProfile,
            r#"
            UPDATE users 
            SET settings = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, email, created_at, updated_at, settings
            "#,
            user_id,
            serde_json::to_value(settings)?
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(user)
    }
}
```

## Data Models

### Database Schema

```sql
-- Users table with encrypted fields
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    totp_secret TEXT, -- Encrypted TOTP secret
    email_verified BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    settings JSONB DEFAULT '{}'::jsonb
);

-- Artists catalog
CREATE TABLE artists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    canonical_name VARCHAR(255) NOT NULL,
    external_ids JSONB DEFAULT '{}'::jsonb, -- {spotify: "id", apple: "id", etc}
    metadata JSONB DEFAULT '{}'::jsonb, -- {image: "url", genres: ["rock"], etc}
    aliases JSONB DEFAULT '{}'::jsonb, -- Alternative names and spellings
    created_at TIMESTAMP DEFAULT NOW()
);

-- User DNP lists
CREATE TABLE user_artist_blocks (
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    artist_id UUID REFERENCES artists(id) ON DELETE CASCADE,
    tags TEXT[], -- User-defined tags for organization
    note TEXT, -- Optional user note
    created_at TIMESTAMP DEFAULT NOW(),
    PRIMARY KEY (user_id, artist_id)
);

-- Session management
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    refresh_token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    last_used_at TIMESTAMP DEFAULT NOW(),
    user_agent TEXT,
    ip_address INET
);

-- Rate limiting
CREATE TABLE rate_limits (
    key VARCHAR(255) PRIMARY KEY,
    count INTEGER DEFAULT 0,
    window_start TIMESTAMP DEFAULT NOW(),
    expires_at TIMESTAMP NOT NULL
);

-- Audit logging for security
CREATE TABLE audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id VARCHAR(255),
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_artists_canonical_name ON artists(canonical_name);
CREATE INDEX idx_artists_external_ids_gin ON artists USING GIN (external_ids);
CREATE INDEX idx_user_artist_blocks_user_id ON user_artist_blocks(user_id);
CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_expires_at ON user_sessions(expires_at);
CREATE INDEX idx_rate_limits_expires_at ON rate_limits(expires_at);
CREATE INDEX idx_audit_log_user_id ON audit_log(user_id);
CREATE INDEX idx_audit_log_created_at ON audit_log(created_at);
```

### API Models

```rust
// Authentication models
#[derive(Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub totp_code: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: UserProfile,
    pub access_token: String,
    pub refresh_token: String,
}

// DNP models
#[derive(Serialize, Deserialize)]
pub struct Artist {
    pub id: Uuid,
    pub canonical_name: String,
    pub external_ids: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct AddToDnpRequest {
    pub artist_id: Uuid,
    pub tags: Option<Vec<String>>,
    pub note: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DnpEntry {
    pub user_id: Uuid,
    pub artist_id: Uuid,
    pub tags: Option<Vec<String>>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}
```

## Error Handling

### Error Types and Responses

```rust
// Centralized error handling
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),
    
    #[error("DNP service error: {0}")]
    Dnp(#[from] DnpError),
    
    #[error("User service error: {0}")]
    User(#[from] UserError),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Rate limit exceeded")]
    RateLimit,
    
    #[error("Internal server error")]
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Auth(AuthError::InvalidCredentials) => {
                (StatusCode::UNAUTHORIZED, "Invalid credentials")
            }
            AppError::Auth(AuthError::TotpRequired) => {
                (StatusCode::UNAUTHORIZED, "2FA code required")
            }
            AppError::Dnp(DnpError::DuplicateEntry) => {
                (StatusCode::CONFLICT, "Artist already in DNP list")
            }
            AppError::Validation(msg) => {
                (StatusCode::BAD_REQUEST, msg.as_str())
            }
            AppError::RateLimit => {
                (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded")
            }
            _ => {
                tracing::error!("Internal server error: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };
        
        let body = Json(serde_json::json!({
            "error": error_message,
            "timestamp": Utc::now().to_rfc3339()
        }));
        
        (status, body).into_response()
    }
}
```

### Health Checks

```rust
#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub services: HashMap<String, ServiceHealth>,
}

#[derive(Serialize)]
pub struct ServiceHealth {
    pub status: String,
    pub response_time_ms: u64,
    pub details: Option<serde_json::Value>,
}

pub async fn health_check(State(state): State<AppState>) -> Result<Json<HealthStatus>, AppError> {
    let start = Instant::now();
    let mut services = HashMap::new();
    
    // Check database
    let db_start = Instant::now();
    let db_result = sqlx::query("SELECT 1").fetch_one(&state.db).await;
    services.insert("database".to_string(), ServiceHealth {
        status: if db_result.is_ok() { "healthy" } else { "unhealthy" }.to_string(),
        response_time_ms: db_start.elapsed().as_millis() as u64,
        details: db_result.err().map(|e| serde_json::json!({"error": e.to_string()})),
    });
    
    // Check Redis
    let redis_start = Instant::now();
    let mut redis_conn = state.redis.get().await.map_err(|_| AppError::Internal)?;
    let redis_result: Result<String, _> = redis::cmd("PING").query_async(&mut redis_conn).await;
    services.insert("redis".to_string(), ServiceHealth {
        status: if redis_result.is_ok() { "healthy" } else { "unhealthy" }.to_string(),
        response_time_ms: redis_start.elapsed().as_millis() as u64,
        details: redis_result.err().map(|e| serde_json::json!({"error": e.to_string()})),
    });
    
    let overall_status = if services.values().all(|s| s.status == "healthy") {
        "healthy"
    } else {
        "unhealthy"
    };
    
    Ok(Json(HealthStatus {
        status: overall_status.to_string(),
        timestamp: Utc::now(),
        services,
    }))
}
```

## Testing Strategy

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use testcontainers::{clients::Cli, images::postgres::Postgres, Container};
    
    struct TestContext {
        db: PgPool,
        _container: Container<'static, Postgres>,
    }
    
    impl TestContext {
        async fn new() -> Self {
            let docker = Cli::default();
            let container = docker.run(Postgres::default());
            let connection_string = format!(
                "postgres://postgres:postgres@127.0.0.1:{}/postgres",
                container.get_host_port_ipv4(5432)
            );
            
            let db = PgPool::connect(&connection_string).await.unwrap();
            sqlx::migrate!("./migrations").run(&db).await.unwrap();
            
            Self {
                db,
                _container: container,
            }
        }
    }
    
    #[tokio::test]
    async fn test_user_registration() {
        let ctx = TestContext::new().await;
        let auth_service = AuthService::new(ctx.db.clone(), /* other deps */);
        
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "secure_password123".to_string(),
        };
        
        let response = auth_service.register(request).await.unwrap();
        
        assert_eq!(response.user.email, "test@example.com");
        assert!(!response.access_token.is_empty());
        assert!(!response.refresh_token.is_empty());
    }
    
    #[tokio::test]
    async fn test_dnp_list_management() {
        let ctx = TestContext::new().await;
        let dnp_service = DnpService::new(ctx.db.clone(), /* other deps */);
        
        // Create test user and artist
        let user_id = create_test_user(&ctx.db).await;
        let artist_id = create_test_artist(&ctx.db, "Test Artist").await;
        
        // Add to DNP list
        let entry = dnp_service.add_to_dnp_list(
            user_id,
            artist_id,
            Some(vec!["test".to_string()]),
            Some("Test note".to_string())
        ).await.unwrap();
        
        assert_eq!(entry.user_id, user_id);
        assert_eq!(entry.artist_id, artist_id);
        
        // Test duplicate prevention
        let duplicate_result = dnp_service.add_to_dnp_list(
            user_id,
            artist_id,
            None,
            None
        ).await;
        
        assert!(matches!(duplicate_result, Err(DnpError::DuplicateEntry)));
    }
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_auth_flow_integration() {
    let app = create_test_app().await;
    
    // Register user
    let register_response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/v1/auth/register")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_string(&RegisterRequest {
                    email: "test@example.com".to_string(),
                    password: "secure_password123".to_string(),
                }).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(register_response.status(), StatusCode::CREATED);
    
    let auth_response: AuthResponse = serde_json::from_slice(
        &hyper::body::to_bytes(register_response.into_body()).await.unwrap()
    ).unwrap();
    
    // Use token to access protected endpoint
    let profile_response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/api/v1/users/profile")
                .header(http::header::AUTHORIZATION, format!("Bearer {}", auth_response.access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(profile_response.status(), StatusCode::OK);
}
```

## Development Environment

### Local Development Setup

```yaml
# docker-compose.yml
version: '3.8'
services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: kiro_dev
      POSTGRES_USER: kiro
      POSTGRES_PASSWORD: kiro_dev_password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./scripts/init-db.sql:/docker-entrypoint-initdb.d/init-db.sql
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U kiro -d kiro_dev"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5

  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile.dev
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: postgres://kiro:kiro_dev_password@postgres:5432/kiro_dev
      REDIS_URL: redis://redis:6379
      JWT_SECRET: dev_jwt_secret_change_in_production
      RUST_LOG: debug
    volumes:
      - ./backend:/app
      - cargo_cache:/usr/local/cargo/registry
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    command: cargo watch -x run

  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile.dev
    ports:
      - "5000:5000"
    volumes:
      - ./frontend:/app
      - node_modules:/app/node_modules
    environment:
      VITE_API_URL: http://localhost:3000
    command: npm run dev

volumes:
  postgres_data:
  cargo_cache:
  node_modules:
```

### Makefile Commands

```makefile
# Makefile
.PHONY: setup dev test clean reset-db k8s-dev

# Development setup
setup:
	@echo "Setting up development environment..."
	docker-compose pull
	docker-compose build
	cp .env.example .env
	@echo "Run 'make dev' to start development servers"

# Start development environment
dev:
	@echo "Starting development environment..."
	docker-compose up -d postgres redis
	@echo "Waiting for services to be ready..."
	docker-compose exec postgres pg_isready -U kiro -d kiro_dev
	@echo ""
	@echo "Services ready! Run these commands in separate terminals:"
	@echo "  Backend:  cd backend && cargo run"
	@echo "  Frontend: cd frontend && npm run dev"
	@echo ""
	@echo "Or run 'docker-compose up' to start all services in containers"

# Run tests
test:
	cd backend && cargo test
	cd frontend && npm test

# Clean up
clean:
	docker-compose down -v
	docker system prune -f

# Reset database
reset-db:
	docker-compose stop postgres
	docker-compose rm -f postgres
	docker volume rm kiro_postgres_data || true
	docker-compose up -d postgres
	@echo "Database reset complete"

# Kubernetes development
k8s-dev:
	@echo "Starting Kubernetes development environment..."
	skaffold dev --port-forward
```

### Kubernetes Development

```yaml
# skaffold.yaml
apiVersion: skaffold/v4beta6
kind: Config
metadata:
  name: kiro-dev
build:
  artifacts:
  - image: kiro/backend
    context: backend
    docker:
      dockerfile: Dockerfile
  - image: kiro/frontend
    context: frontend
    docker:
      dockerfile: Dockerfile
  local:
    push: false
deploy:
  helm:
    releases:
    - name: kiro
      chartPath: helm/kiro
      valuesFiles:
      - helm/kiro/values-dev.yaml
      setValues:
        backend.image.tag: kiro/backend
        frontend.image.tag: kiro/frontend
portForward:
- resourceType: service
  resourceName: kiro-backend
  port: 3000
- resourceType: service
  resourceName: kiro-frontend
  port: 80
  localPort: 5000
```

```yaml
# helm/kiro/values-dev.yaml
backend:
  image:
    repository: kiro/backend
    tag: latest
    pullPolicy: Never
  
  env:
    DATABASE_URL: postgres://kiro:password@postgres:5432/kiro
    REDIS_URL: redis://redis:6379
    JWT_SECRET: dev_jwt_secret
    RUST_LOG: debug
  
  resources:
    requests:
      memory: "256Mi"
      cpu: "100m"
    limits:
      memory: "512Mi"
      cpu: "500m"

frontend:
  image:
    repository: kiro/frontend
    tag: latest
    pullPolicy: Never
  
  env:
    VITE_API_URL: http://localhost:3000

postgres:
  enabled: true
  auth:
    username: kiro
    password: password
    database: kiro

redis:
  enabled: true
  auth:
    enabled: false
```

## Docker Build Optimization

### Multi-Stage Caching Strategy

The Docker build optimization focuses on minimizing rebuild times through intelligent layer caching and dependency management.

#### Backend Dockerfile with Cargo-Chef

```dockerfile
# Optimized multi-stage Dockerfile for Rust backend
FROM rust:1.82-slim as chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef as planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
# Install system dependencies (cached layer)
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev libpq-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Build dependencies first (heavily cached layer)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source and build (only rebuilds when source changes)
COPY . .
RUN cargo build --release

# Minimal runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates libpq5 curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/music-streaming-blocklist-backend ./backend
COPY --from=builder /app/migrations ./migrations

RUN useradd -r -s /bin/false -u 1001 kiro && chown -R kiro:kiro /app
USER kiro

EXPOSE 3000
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["./backend"]
```

#### Frontend Dockerfile with Node.js Optimization

```dockerfile
# Optimized multi-stage Dockerfile for Svelte frontend
FROM node:18-alpine as builder
WORKDIR /app

# Copy package files first (cached layer)
COPY package*.json ./
RUN npm ci --only=production

# Copy source and build (only rebuilds when source changes)
COPY . .
RUN npm run build

# Minimal nginx runtime
FROM nginx:alpine
RUN apk add --no-cache curl

COPY --from=builder /app/public /usr/share/nginx/html
COPY nginx-dev.conf /etc/nginx/conf.d/default.conf

RUN mkdir -p /var/cache/nginx /var/log/nginx /var/run && \
    chown -R nginx:nginx /var/cache/nginx /var/log/nginx /var/run /usr/share/nginx/html

USER nginx
EXPOSE 5000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:5000/ || exit 1

CMD ["nginx", "-g", "daemon off;"]
```

### Build Performance Optimization

#### Cache Warming Script

```bash
#!/bin/bash
# scripts/warm-cache.sh - Pre-build Docker layers for faster development

echo "🔥 Warming Docker build cache..."

# Build base layers for backend
echo "Building Rust base layers..."
docker build --target chef -t kiro/backend:chef ./backend/
docker build --target planner -t kiro/backend:planner ./backend/
docker build --target builder -t kiro/backend:builder ./backend/

# Build base layers for frontend  
echo "Building Node.js base layers..."
docker build --target builder -t kiro/frontend:builder ./frontend/

echo "✅ Cache warming complete!"
echo "Next builds will be significantly faster."
```

#### Build Performance Testing

```bash
#!/bin/bash
# scripts/test-docker-build.sh - Measure build performance

echo "🧪 Testing Docker build performance..."

# Test backend build times
echo "Testing backend build (cold cache)..."
docker system prune -f
time docker build -t kiro/backend:test ./backend/

echo "Testing backend build (warm cache)..."
time docker build -t kiro/backend:test ./backend/

# Test frontend build times
echo "Testing frontend build (cold cache)..."
docker system prune -f  
time docker build -t kiro/frontend:test ./frontend/

echo "Testing frontend build (warm cache)..."
time docker build -t kiro/frontend:test ./frontend/

echo "✅ Build performance testing complete!"
```

## Enhanced Tilt Configuration

### Optimized Tiltfile

```python
# Enhanced Tiltfile with performance optimizations and developer experience improvements

# Configuration
namespace = "kiro-dev"
k8s_namespace(namespace)

# Performance settings for faster feedback
update_settings(
    max_parallel_updates=4,
    k8s_upsert_timeout_secs=60,
    suppress_unused_image_warnings=None
)

# Enable BuildKit and optimize Docker builds
docker_prune_settings(num_builds=3, keep_recent=2)

# Backend build with optimized caching
docker_build(
    'kiro/backend',
    context='./backend',
    dockerfile='./backend/Dockerfile.dev',
    # Use BuildKit for faster builds
    build_args={'BUILDKIT_INLINE_CACHE': '1'},
    # Only rebuild when source changes
    only=['./backend/src', './backend/Cargo.toml', './backend/Cargo.lock'],
    # Ignore non-essential files
    ignore=['./backend/target', './backend/.env']
)

# Frontend build with optimized caching
docker_build(
    'kiro/frontend',
    context='./frontend', 
    dockerfile='./frontend/Dockerfile.dev',
    build_args={'BUILDKIT_INLINE_CACHE': '1'},
    only=['./frontend/src', './frontend/package.json', './frontend/package-lock.json'],
    ignore=['./frontend/node_modules', './frontend/public/build']
)

# Deploy Kubernetes manifests
k8s_yaml('./k8s/dev-manifests.yaml')

# Configure resources with dependencies and health checks
k8s_resource('postgres',
    port_forwards=['5432:5432'],
    resource_deps=[],
    readiness_probe=probe(
        http_get=http_get_action(port=5432, path='/'),
        initial_delay_secs=10,
        timeout_secs=5
    )
)

k8s_resource('redis',
    port_forwards=['6379:6379'], 
    resource_deps=[],
    readiness_probe=probe(
        tcp_socket=tcp_socket_action(port=6379),
        initial_delay_secs=5,
        timeout_secs=3
    )
)

k8s_resource('backend',
    port_forwards=['3000:3000'],
    resource_deps=['postgres', 'redis'],
    readiness_probe=probe(
        http_get=http_get_action(port=3000, path='/health'),
        initial_delay_secs=15,
        timeout_secs=10
    )
)

k8s_resource('frontend',
    port_forwards=['5000:5000'],
    resource_deps=['backend'],
    readiness_probe=probe(
        http_get=http_get_action(port=5000, path='/'),
        initial_delay_secs=10,
        timeout_secs=5
    )
)

# Development tools and manual triggers
local_resource(
    'backend-tests',
    cmd='cd backend && cargo test --lib --bins --tests',
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['tests'],
    resource_deps=['backend']
)

local_resource(
    'frontend-tests', 
    cmd='cd frontend && npm test -- --run',
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['tests'],
    resource_deps=['frontend']
)

local_resource(
    'health-check',
    cmd='''
    echo "🔍 Checking service health..."
    curl -f http://localhost:3000/health && echo "✅ Backend healthy" || echo "❌ Backend not ready"
    curl -f http://localhost:5000/ && echo "✅ Frontend healthy" || echo "❌ Frontend not ready"
    ''',
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['monitoring'],
    resource_deps=['backend', 'frontend']
)

local_resource(
    'db-migrate',
    cmd='kubectl exec -n ' + namespace + ' deployment/backend -- ./backend migrate',
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['database'],
    resource_deps=['backend', 'postgres']
)

local_resource(
    'warm-cache',
    cmd='./scripts/warm-cache.sh',
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['optimization']
)

local_resource(
    'test-build-perf',
    cmd='./scripts/test-docker-build.sh',
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
    labels=['optimization']
)

# Development workflow information
print("🚀 Kiro Development Environment - Optimized with Tilt + Minikube")
print("")
print("📡 Services automatically available at:")
print("- Backend API: http://localhost:3000")
print("- Frontend: http://localhost:5000") 
print("- PostgreSQL: localhost:5432 (user: kiro, pass: password, db: kiro)")
print("- Redis: localhost:6379")
print("")
print("🔄 Optimized Multi-Stage Builds:")
print("- Backend: cargo-chef for 80% faster Rust builds")
print("- Frontend: Node.js layer caching for fast rebuilds")
print("- BuildKit enabled for parallel layer processing")
print("- Smart file watching - only rebuilds when source changes")
print("")
print("🎮 Manual triggers available:")
print("- backend-tests: Run backend test suite")
print("- frontend-tests: Run frontend test suite") 
print("- health-check: Verify all services are healthy")
print("- db-migrate: Run database migrations")
print("- warm-cache: Pre-build Docker layers for speed")
print("- test-build-perf: Measure build performance")
print("")
print("💡 Optimized development workflow:")
print("1. Wait for all services to show 'Ready' status")
print("2. Run 'db-migrate' trigger to initialize database")
print("3. Run 'health-check' to verify everything works")
print("4. Start coding - changes auto-rebuild in ~10 seconds!")
print("5. Use manual triggers for testing and debugging")
```

## Frontend-Backend Integration

### API Client Configuration

```typescript
// frontend/src/lib/utils/api.ts
import { browser } from '$app/environment';

const API_BASE_URL = browser 
    ? (import.meta.env.VITE_API_URL || 'http://localhost:3000')
    : 'http://backend:3000'; // Internal service name for SSR

class ApiClient {
    private baseUrl: string;
    private token: string | null = null;

    constructor(baseUrl: string = API_BASE_URL) {
        this.baseUrl = baseUrl;
        
        // Load token from localStorage if available
        if (browser && localStorage.getItem('auth_token')) {
            this.token = localStorage.getItem('auth_token');
        }
    }

    setToken(token: string) {
        this.token = token;
        if (browser) {
            localStorage.setItem('auth_token', token);
        }
    }

    clearToken() {
        this.token = null;
        if (browser) {
            localStorage.removeItem('auth_token');
        }
    }

    private async request<T>(
        endpoint: string, 
        options: RequestInit = {}
    ): Promise<T> {
        const url = `${this.baseUrl}${endpoint}`;
        
        const headers: HeadersInit = {
            'Content-Type': 'application/json',
            ...options.headers,
        };

        if (this.token) {
            headers.Authorization = `Bearer ${this.token}`;
        }

        const response = await fetch(url, {
            ...options,
            headers,
        });

        if (!response.ok) {
            const error = await response.json().catch(() => ({ 
                error: 'Network error' 
            }));
            throw new Error(error.error || `HTTP ${response.status}`);
        }

        return response.json();
    }

    // Authentication endpoints
    async register(email: string, password: string) {
        return this.request<AuthResponse>('/api/v1/auth/register', {
            method: 'POST',
            body: JSON.stringify({ email, password }),
        });
    }

    async login(email: string, password: string, totpCode?: string) {
        return this.request<AuthResponse>('/api/v1/auth/login', {
            method: 'POST',
            body: JSON.stringify({ email, password, totp_code: totpCode }),
        });
    }

    // DNP list endpoints
    async searchArtists(query: string) {
        return this.request<Artist[]>(`/api/v1/dnp/search?q=${encodeURIComponent(query)}`);
    }

    async getDnpList() {
        return this.request<DnpEntry[]>('/api/v1/dnp/list');
    }

    async addToDnpList(artistId: string, tags?: string[], note?: string) {
        return this.request<DnpEntry>('/api/v1/dnp/add', {
            method: 'POST',
            body: JSON.stringify({ artist_id: artistId, tags, note }),
        });
    }

    // Health check
    async healthCheck() {
        return this.request<HealthStatus>('/health');
    }
}

export const apiClient = new ApiClient();

// Type definitions
export interface AuthResponse {
    user: UserProfile;
    access_token: string;
    refresh_token: string;
}

export interface UserProfile {
    id: string;
    email: string;
    created_at: string;
    updated_at: string;
}

export interface Artist {
    id: string;
    canonical_name: string;
    external_ids: Record<string, string>;
    metadata: Record<string, any>;
    created_at: string;
}

export interface DnpEntry {
    user_id: string;
    artist_id: string;
    tags?: string[];
    note?: string;
    created_at: string;
    artist?: Artist;
}

export interface HealthStatus {
    status: string;
    timestamp: string;
    services: Record<string, any>;
}
```

### Environment Configuration

```bash
# backend/.env
DATABASE_URL=postgres://kiro:kiro_dev_password@postgres:5432/kiro_dev
REDIS_URL=redis://redis:6379
JWT_SECRET=dev_jwt_secret_change_in_production
RUST_LOG=debug
ENVIRONMENT=development
```

```bash
# frontend/.env
VITE_API_URL=http://localhost:3000
NODE_ENV=development
```

## Monitoring and Observability

### Structured Logging

```rust
use tracing::{info, error, warn, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kiro=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}

#[instrument(skip(state))]
pub async fn login_handler(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    info!(email = %request.email, "Login attempt");
    
    match state.auth_service.login(request).await {
        Ok(response) => {
            info!(user_id = %response.user.id, "Login successful");
            Ok(Json(response))
        }
        Err(e) => {
            warn!(error = %e, "Login failed");
            Err(e.into())
        }
    }
}
```

### Prometheus Metrics

```rust
use prometheus::{Counter, Histogram, Registry, Encoder, TextEncoder};

#[derive(Clone)]
pub struct Metrics {
    pub http_requests_total: Counter,
    pub http_request_duration: Histogram,
    pub auth_attempts_total: Counter,
    pub dnp_operations_total: Counter,
}

impl Metrics {
    pub fn new() -> Self {
        let http_requests_total = Counter::new(
            "http_requests_total",
            "Total number of HTTP requests"
        ).unwrap();
        
        let http_request_duration = Histogram::new(
            "http_request_duration_seconds",
            "HTTP request duration in seconds"
        ).unwrap();
        
        let auth_attempts_total = Counter::new(
            "auth_attempts_total",
            "Total number of authentication attempts"
        ).unwrap();
        
        let dnp_operations_total = Counter::new(
            "dnp_operations_total",
            "Total number of DNP list operations"
        ).unwrap();
        
        Self {
            http_requests_total,
            http_request_duration,
            auth_attempts_total,
            dnp_operations_total,
        }
    }
    
    pub fn register(&self, registry: &Registry) {
        registry.register(Box::new(self.http_requests_total.clone())).unwrap();
        registry.register(Box::new(self.http_request_duration.clone())).unwrap();
        registry.register(Box::new(self.auth_attempts_total.clone())).unwrap();
        registry.register(Box::new(self.dnp_operations_total.clone())).unwrap();
    }
}

pub async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = state.metrics.registry.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    Response::builder()
        .header("content-type", "text/plain; version=0.0.4")
        .body(Body::from(buffer))
        .unwrap()
}
```

## Security Considerations

### JWT Token Management

```rust
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub exp: usize,  // expiration
    pub iat: usize,  // issued at
    pub jti: String, // JWT ID for revocation
}

impl AuthService {
    pub async fn generate_tokens(&self, user: &User) -> Result<(String, String), AuthError> {
        let now = Utc::now();
        let access_exp = now + Duration::hours(24);
        let refresh_exp = now + Duration::days(30);
        
        // Access token
        let access_claims = Claims {
            sub: user.id.to_string(),
            exp: access_exp.timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: Uuid::new_v4().to_string(),
        };
        
        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;
        
        // Refresh token (stored in database)
        let refresh_token = Uuid::new_v4().to_string();
        let refresh_token_hash = bcrypt::hash(&refresh_token, 12)?;
        
        sqlx::query!(
            r#"
            INSERT INTO user_sessions (user_id, refresh_token_hash, expires_at)
            VALUES ($1, $2, $3)
            "#,
            user.id,
            refresh_token_hash,
            refresh_exp.naive_utc()
        )
        .execute(&self.db)
        .await?;
        
        Ok((access_token, refresh_token))
    }
    
    pub async fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &validation,
        )?;
        
        Ok(token_data.claims)
    }
}
```

### Rate Limiting

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct RateLimiter {
    limits: Arc<RwLock<HashMap<String, RateLimit>>>,
    redis: RedisPool,
}

#[derive(Debug)]
pub struct RateLimit {
    pub count: u32,
    pub window_start: Instant,
    pub limit: u32,
    pub window_duration: Duration,
}

impl RateLimiter {
    pub async fn check_rate_limit(&self, key: &str, limit: u32, window: Duration) -> Result<bool, AppError> {
        let mut conn = self.redis.get().await?;
        
        let current_count: u32 = redis::cmd("INCR")
            .arg(format!("rate_limit:{}", key))
            .query_async(&mut conn)
            .await?;
        
        if current_count == 1 {
            // Set expiration on first request
            redis::cmd("EXPIRE")
                .arg(format!("rate_limit:{}", key))
                .arg(window.as_secs())
                .query_async::<_, ()>(&mut conn)
                .await?;
        }
        
        Ok(current_count <= limit)
    }
}

// Middleware for rate limiting
pub async fn rate_limit_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|hv| hv.to_str().ok())
        .unwrap_or("unknown");
    
    let rate_limiter = request.extensions().get::<RateLimiter>().unwrap();
    
    if !rate_limiter.check_rate_limit(ip, 100, Duration::from_secs(3600)).await? {
        return Err(AppError::RateLimit);
    }
    
    Ok(next.run(request).await)
}
```

This design provides a solid foundation for the core services and development environment, focusing on developer productivity, system reliability, and security while maintaining the flexibility to evolve into a more complex system as needed.