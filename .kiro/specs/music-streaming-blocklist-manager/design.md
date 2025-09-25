# Design Document

## Overview

Kiro is a multi-platform music streaming blocklist management system that provides centralized control for users to avoid specific artists across streaming services. The system follows a microservices architecture with a web application frontend, browser extension, API backend, worker services, and integration adapters for each streaming platform.

The design emphasizes user privacy, platform neutrality, and graceful degradation when platform APIs have limitations. The system uses a capability-based approach where each streaming service adapter declares what enforcement actions it can perform (hard blocks, soft hiding, library purging, etc.).

## Architecture

### High-Level System Architecture

```mermaid
graph TB
    subgraph "Client Layer"
        WebApp[Web Application<br/>Svelte + TypeScript]
        Extension[Browser Extension<br/>Manifest v3]
        Mobile[Mobile Assist<br/>iOS Shortcuts/Android]
    end
    
    subgraph "API Gateway"
        Gateway[API Gateway<br/>Rate Limiting + Auth]
    end
    
    subgraph "Application Layer"
        API[REST/GraphQL API<br/>Rust (Axum/Actix)]
        Auth[Authentication Service<br/>OAuth 2.0/OIDC]
        Policy[Policy Engine<br/>Rule Evaluation]
    end
    
    subgraph "Worker Layer"
        Queue[Job Queue<br/>Redis/BullMQ]
        Workers[Background Workers<br/>Enforcement Execution]
    end
    
    subgraph "Integration Layer"
        Spotify[Spotify Adapter]
        Apple[Apple Music Adapter]
        YouTube[YouTube Music Adapter]
        Tidal[Tidal Adapter]
    end
    
    subgraph "Data Layer"
        DB[(DuckDB<br/>Analytics + Reporting)]
        MainDB[(PostgreSQL<br/>User Data + Metadata)]
        Cache[(Redis<br/>Sessions + Cache)]
        Vault[Token Vault<br/>KMS Encrypted]
        Storage[Object Storage<br/>Exports + Reports]
    end
    
    WebApp --> Gateway
    Extension --> Gateway
    Mobile --> Gateway
    Gateway --> API
    API --> Auth
    API --> Policy
    API --> Queue
    Queue --> Workers
    Workers --> Spotify
    Workers --> Apple
    Workers --> YouTube
    Workers --> Tidal
    API --> MainDB
    API --> DB
    API --> Cache
    Auth --> Vault
    Workers --> Storage
    Workers --> DB
```

### Service Adapter Pattern

Each streaming service implements a common interface with capability declarations:

```typescript
interface StreamingServiceAdapter {
  capabilities: ServiceCapabilities;
  authenticate(credentials: OAuthCredentials): Promise<Connection>;
  planEnforcement(dnpList: Artist[], connection: Connection): Promise<EnforcementPlan>;
  executeEnforcement(plan: EnforcementPlan): Promise<EnforcementResult>;
  rollbackActions(batch: ActionBatch): Promise<RollbackResult>;
}

interface ServiceCapabilities {
  LIBRARY_PURGE: 'SUPPORTED' | 'LIMITED' | 'UNSUPPORTED';
  PLAYLIST_SCRUB: 'SUPPORTED' | 'LIMITED' | 'UNSUPPORTED';
  ARTIST_BLOCK: 'SUPPORTED' | 'LIMITED' | 'UNSUPPORTED';
  RECOMMENDATION_FILTER: 'SUPPORTED' | 'LIMITED' | 'UNSUPPORTED';
  RADIO_SEED_FILTER: 'SUPPORTED' | 'LIMITED' | 'UNSUPPORTED';
  AUTOPLAY_SKIP: 'SUPPORTED' | 'LIMITED' | 'UNSUPPORTED';
  WEB_OVERLAY: 'SUPPORTED' | 'LIMITED' | 'UNSUPPORTED';
  FEATURED_ARTIST_DETECTION: 'SUPPORTED' | 'LIMITED' | 'UNSUPPORTED';
}
```

## Components and Interfaces

### Frontend Components

**Web Application (Svelte + TypeScript)**
- Lightning-fast development with Svelte's compile-time optimizations and built-in dev server
- Minimal runtime overhead with Svelte's compile-to-vanilla-JS approach
- User dashboard for DNP list management with reactive state management
- Service connection management with OAuth flows
- Community list browsing and subscription interface
- Enforcement planning and execution with real-time updates
- Action history and undo capabilities with Svelte stores
- Settings and preferences management

**Browser Extension (Manifest v3)**
- Content scripts for Spotify Web, YouTube Music, Apple Music web
- Background service worker for cross-tab coordination
- Popup interface for quick DNP list modifications
- DOM manipulation for hiding blocked content
- Auto-skip functionality with user override options

### Backend Services

**Authentication Service**
- OAuth 2.0/OpenID Connect implementation
- Multi-provider support (Google, Apple, email/password)
- JWT token management with refresh rotation
- 2FA support (TOTP)
- Session management and security controls

**Policy Engine**
- DNP list evaluation and rule processing
- Community list subscription resolution
- Capability-aware enforcement planning
- Conflict resolution for overlapping rules
- Aggressiveness level application

**Enforcement Workers**
- Asynchronous job processing for platform operations
- Idempotent operation handling
- Progress tracking and user notifications
- Error handling and retry logic
- Batch operation management

### Integration Adapters

**Entity Resolution Service (Rust)**
- High-performance multi-authority artist disambiguation using Spotify, Apple Music, MusicBrainz, and ISNI IDs
- Concurrent track-level ISRC and album-level UPC matching with async processing
- Featured artist and collaboration detection from track metadata using parallel processing
- ML-based confidence scoring for artist aliases and name variants
- Canonical entity mapping with fallback strategies and caching

**Spotify Adapter**
- OAuth 2.0 with PKCE for user authorization
- Library management (liked songs, saved albums) with batch operations
- Playlist modification with delta removal to minimize API calls
- Artist following/unfollowing with featured artist detection
- Rate-limit aware batching using 429 response headers
- Web extension integration for client-side blocking and auto-skip

**Apple Music Adapter**
- MusicKit JS with user token + developer token rotation
- Token broker service for short-lived user token management
- Apple Music API for library operations (limited write capabilities)
- Featured artist detection from track artist arrays
- Web overlay for visual blocking with MutationObserver resilience

**YouTube Music Adapter**
- Web-based approach due to limited API access (ToS compliant)
- Extension-based content hiding with semantic selectors (ARIA/role)
- Auto-skip functionality with media event hooks
- User data export/import workflows for manual synchronization
- Shadow DOM-safe content filtering strategies

**Tidal Adapter**
- Best-effort implementation based on available APIs
- Web extension support with bloom filter for O(1) DNP lookups
- Preview-only mode with optional user-initiated scripts
- Graceful degradation messaging when API access is limited

## Data Models

### Core Entities

```sql
-- User management
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    settings JSONB DEFAULT '{}'
);

-- Service connections with health tracking
CREATE TABLE connections (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,
    provider_user_id VARCHAR(255),
    scopes TEXT[],
    access_token_encrypted TEXT,
    refresh_token_encrypted TEXT,
    token_version INTEGER DEFAULT 1,
    expires_at TIMESTAMP,
    status VARCHAR(20) DEFAULT 'active',
    last_health_check TIMESTAMP,
    error_code TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(user_id, provider)
);

-- Rate limiting state per provider
CREATE TABLE provider_rate_state (
    provider VARCHAR(50) PRIMARY KEY,
    remaining INTEGER DEFAULT 0,
    reset_at TIMESTAMP,
    window_size INTEGER DEFAULT 3600,
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Artist catalog with disambiguation
CREATE TABLE artists (
    id UUID PRIMARY KEY,
    canonical_name VARCHAR(255) NOT NULL,
    canonical_artist_id UUID REFERENCES artists(id), -- self-ref for aliases
    external_ids JSONB DEFAULT '{}', -- {spotify: "id", apple: "id", musicbrainz: "id", isni: "id"}
    metadata JSONB DEFAULT '{}', -- {image: "url", genres: ["string"], isrc: ["string"], upc: ["string"]}
    aliases JSONB DEFAULT '{}', -- {name: "string", source: "string", confidence: float}
    created_at TIMESTAMP DEFAULT NOW()
);

-- Indexes for fast lookups
CREATE INDEX idx_artists_external_ids_spotify ON artists USING GIN ((external_ids->>'spotify'));
CREATE INDEX idx_artists_external_ids_apple ON artists USING GIN ((external_ids->>'apple'));
CREATE INDEX idx_artists_external_ids_musicbrainz ON artists USING GIN ((external_ids->>'musicbrainz'));
CREATE INDEX idx_artists_canonical ON artists(canonical_artist_id) WHERE canonical_artist_id IS NOT NULL;

-- User DNP lists
CREATE TABLE user_artist_blocks (
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    artist_id UUID REFERENCES artists(id) ON DELETE CASCADE,
    tags TEXT[],
    note TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    PRIMARY KEY (user_id, artist_id)
);

-- Community lists with governance
CREATE TABLE community_lists (
    id UUID PRIMARY KEY,
    owner_user_id UUID REFERENCES users(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    criteria TEXT NOT NULL, -- Required neutral criteria
    governance_url TEXT, -- Link to governance process
    update_cadence TEXT, -- "weekly", "monthly", "as-needed"
    version INTEGER DEFAULT 1,
    visibility VARCHAR(20) DEFAULT 'public',
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Audit log for SOC2 compliance
CREATE TABLE audit_log (
    id UUID PRIMARY KEY,
    actor_user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    subject_type VARCHAR(50) NOT NULL,
    subject_id VARCHAR(255) NOT NULL,
    before_state JSONB,
    after_state JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE community_list_items (
    list_id UUID REFERENCES community_lists(id) ON DELETE CASCADE,
    artist_id UUID REFERENCES artists(id) ON DELETE CASCADE,
    rationale_link TEXT,
    added_at TIMESTAMP DEFAULT NOW(),
    PRIMARY KEY (list_id, artist_id)
);

-- List subscriptions
CREATE TABLE user_list_subscriptions (
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    list_id UUID REFERENCES community_lists(id) ON DELETE CASCADE,
    version_pinned INTEGER,
    auto_update BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    PRIMARY KEY (user_id, list_id)
);

-- Action tracking with idempotency
CREATE TABLE action_batches (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,
    idempotency_key TEXT UNIQUE,
    dry_run BOOLEAN DEFAULT false,
    status VARCHAR(20) DEFAULT 'pending',
    options JSONB DEFAULT '{}', -- {block_collabs: true, block_featuring: true, aggressiveness: "moderate"}
    summary JSONB DEFAULT '{}',
    created_at TIMESTAMP DEFAULT NOW(),
    completed_at TIMESTAMP
);

CREATE TABLE action_items (
    id UUID PRIMARY KEY,
    batch_id UUID REFERENCES action_batches(id) ON DELETE CASCADE,
    entity_type VARCHAR(50) NOT NULL,
    entity_id VARCHAR(255) NOT NULL,
    action VARCHAR(50) NOT NULL,
    idempotency_key TEXT,
    before_state JSONB,
    after_state JSONB,
    status VARCHAR(20) DEFAULT 'pending',
    error_message TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(batch_id, entity_type, entity_id, action, idempotency_key)
);

-- Indexes for fast lookups
CREATE INDEX idx_action_items_batch ON action_items(batch_id);
CREATE INDEX idx_action_items_provider_entity ON action_items(entity_type, entity_id);
CREATE INDEX idx_connections_user_provider ON connections(user_id, provider);
CREATE INDEX idx_connections_status ON connections(status);
```

### API Schema Examples

**DNP List Management**
```json
{
  "addArtist": {
    "input": {
      "query": "string", // artist name or provider URL
      "provider": "spotify|apple|youtube|tidal",
      "tags": ["string"],
      "note": "string"
    },
    "output": {
      "artist": {
        "id": "uuid",
        "canonicalName": "string",
        "externalIds": {"spotify": "id", "apple": "id"},
        "metadata": {"image": "url", "genres": ["string"]}
      }
    }
  }
}
```

**Enforcement Planning**
```json
{
  "planEnforcement": {
    "input": {
      "providers": ["spotify", "apple"],
      "options": {
        "aggressiveness": "conservative|moderate|aggressive",
        "blockCollabs": true,
        "blockFeaturing": true,
        "blockSongwriterOnly": false
      },
      "dryRun": true
    },
    "output": {
      "planId": "uuid",
      "idempotencyKey": "string",
      "impact": {
        "spotify": {
          "likedSongs": {"toRemove": 45, "collabsFound": 12},
          "playlists": {"toScrub": 12, "tracksToRemove": 127, "featuringFound": 23},
          "following": {"toUnfollow": 3},
          "radioSeeds": {"toFilter": 8}
        }
      },
      "capabilities": {
        "spotify": {
          "LIBRARY_PURGE": "SUPPORTED",
          "FEATURED_ARTIST_DETECTION": "SUPPORTED",
          "RADIO_SEED_FILTER": "LIMITED"
        }
      },
      "estimatedDuration": "180s",
      "resumable": true
    }
  }
}
```

## Error Handling

### Error Categories and Responses

**Authentication Errors**
- Token expiration: Automatic refresh with fallback to re-authentication
- Invalid scopes: Clear messaging about required permissions
- Rate limiting: Exponential backoff with user-visible wait times

**Platform API Errors**
- Service unavailable: Graceful degradation to available capabilities
- Rate limit exceeded: Queue operations with progress updates
- Invalid operations: Skip with detailed logging and user notification

**Data Consistency Errors**
- Duplicate operations: Idempotency keys prevent double-execution
- Concurrent modifications: Optimistic locking with conflict resolution
- Rollback failures: Best-effort cleanup with manual intervention options

### Circuit Breaker Pattern

```typescript
class ServiceCircuitBreaker {
  private failureCount = 0;
  private lastFailureTime?: Date;
  private state: 'CLOSED' | 'OPEN' | 'HALF_OPEN' = 'CLOSED';
  
  async execute<T>(operation: () => Promise<T>): Promise<T> {
    if (this.state === 'OPEN') {
      if (this.shouldAttemptReset()) {
        this.state = 'HALF_OPEN';
      } else {
        throw new Error('Circuit breaker is OPEN');
      }
    }
    
    try {
      const result = await operation();
      this.onSuccess();
      return result;
    } catch (error) {
      this.onFailure();
      throw error;
    }
  }
}
```

## Testing Strategy

### Unit Testing
- Service adapter implementations with mocked API responses
- Policy engine rule evaluation with various DNP list configurations
- Data model validation and constraint testing
- Authentication flow testing with mock OAuth providers

### Integration Testing
- End-to-end API workflows with test streaming service accounts
- Database transaction testing with concurrent operations
- Queue processing with job failure and retry scenarios
- Browser extension testing across different streaming service UIs

### Performance Testing
- Load testing with simulated large libraries (10k+ tracks)
- Rate limit compliance testing with actual API endpoints
- Concurrent user simulation for enforcement operations
- Memory and CPU profiling for worker processes

### Security Testing
- OAuth flow security with PKCE validation
- Token encryption/decryption verification
- SQL injection and XSS vulnerability scanning
- Access control testing for multi-tenant isolation

## Deployment and Infrastructure

### Container Architecture
```dockerfile
# Rust API Service
FROM rust:1.75-alpine AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM alpine:latest
RUN apk --no-cache add ca-certificates
WORKDIR /root/
COPY --from=builder /app/target/release/kiro-api ./
EXPOSE 3000
CMD ["./kiro-api"]
```

```dockerfile
# Svelte Frontend
FROM node:18-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

### Kubernetes Deployment
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: kiro-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: kiro-api
  template:
    metadata:
      labels:
        app: kiro-api
    spec:
      containers:
      - name: api
        image: kiro/api:latest
        ports:
        - containerPort: 3000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: kiro-secrets
              key: database-url
```

### Monitoring and Observability
- OpenTelemetry for distributed tracing
- Prometheus metrics for performance monitoring
- Structured logging with correlation IDs
- Health checks for all services
- Alerting for critical failures and performance degradation
## Sec
urity Architecture

### Token Vault Service
```typescript
class TokenVaultService {
  private kmsClient: KMSClient;
  private dataKeyCache: Map<string, DataKey> = new Map();
  
  async encryptToken(token: string, userId: string): Promise<string> {
    const dataKey = await this.getOrCreateDataKey(userId);
    const encrypted = await this.encrypt(token, dataKey);
    return this.envelopeEncrypt(encrypted, dataKey.keyId);
  }
  
  async decryptToken(encryptedToken: string, userId: string): Promise<string> {
    const { payload, keyId } = this.envelopeDecrypt(encryptedToken);
    const dataKey = await this.getDataKey(keyId);
    return this.decrypt(payload, dataKey);
  }
  
  private async rotateDataKeys(): Promise<void> {
    // Proactive key rotation every 30 days
    for (const [userId, dataKey] of this.dataKeyCache) {
      if (this.shouldRotateKey(dataKey)) {
        await this.createNewDataKey(userId);
        await this.reencryptUserTokens(userId);
      }
    }
  }
}
```

### Rate Limiting and Batching
```typescript
class RateLimitAwareBatcher {
  private rateLimits: Map<string, RateLimit> = new Map();
  
  async executeBatch(provider: string, operations: Operation[]): Promise<BatchResult> {
    const rateLimit = await this.getRateLimit(provider);
    const batches = this.createOptimalBatches(operations, rateLimit);
    
    const results: OperationResult[] = [];
    for (const batch of batches) {
      await this.waitForRateLimit(provider);
      const batchResult = await this.executeBatchWithRetry(batch);
      results.push(...batchResult.items);
      
      // Update rate limit from response headers
      this.updateRateLimit(provider, batchResult.headers);
    }
    
    return { items: results, resumeToken: this.createResumeToken(operations, results) };
  }
  
  private createOptimalBatches(operations: Operation[], rateLimit: RateLimit): Operation[][] {
    // Group by playlist to minimize API calls
    const playlistOps = operations.filter(op => op.type === 'playlist_track_remove');
    const playlistGroups = this.groupBy(playlistOps, op => op.playlistId);
    
    // Create delta removal batches
    return playlistGroups.map(group => this.createDeltaBatch(group));
  }
}
```

### Extension Security and Resilience
```typescript
class ExtensionContentFilter {
  private observer: MutationObserver;
  private dnpBloomFilter: BloomFilter;
  private shadowRoot: ShadowRoot;
  
  constructor() {
    this.shadowRoot = this.createIsolatedShadowRoot();
    this.setupMutationObserver();
    this.loadDNPFilter();
  }
  
  private setupMutationObserver(): void {
    this.observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        if (mutation.type === 'childList') {
          this.scanForBlockedContent(mutation.addedNodes);
        }
      }
    });
    
    this.observer.observe(document.body, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ['data-testid', 'aria-label', 'role']
    });
  }
  
  private scanForBlockedContent(nodes: NodeList): void {
    for (const node of nodes) {
      if (node.nodeType === Node.ELEMENT_NODE) {
        const element = node as Element;
        const artistInfo = this.extractArtistInfo(element);
        
        if (artistInfo && this.isBlocked(artistInfo)) {
          this.hideElement(element);
          this.addOverrideControls(element, artistInfo);
        }
      }
    }
  }
  
  private extractArtistInfo(element: Element): ArtistInfo | null {
    // Use multiple strategies for resilient artist detection
    const strategies = [
      () => this.extractFromDataAttributes(element),
      () => this.extractFromAriaLabels(element),
      () => this.extractFromTextContent(element),
      () => this.extractFromLinks(element)
    ];
    
    for (const strategy of strategies) {
      const result = strategy();
      if (result) return result;
    }
    
    return null;
  }
}
```

## Deployment Strategy

### Modular Monolith Approach
```yaml
# Initial deployment as modular monolith
apiVersion: apps/v1
kind: Deployment
metadata:
  name: kiro-api
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: api
        image: kiro/api:latest
        env:
        - name: MODULE_ENABLED_AUTH
          value: "true"
        - name: MODULE_ENABLED_POLICY
          value: "true"
        - name: MODULE_ENABLED_ENFORCEMENT
          value: "false" # Separate worker deployment
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"

---
# Dedicated worker deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: kiro-workers
spec:
  replicas: 2
  template:
    spec:
      containers:
      - name: worker
        image: kiro/worker:latest
        env:
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: kiro-secrets
              key: redis-url
        resources:
          requests:
            memory: "256Mi"
            cpu: "100m"
```

### Service Level Objectives (SLOs)
```yaml
slos:
  api_availability:
    target: 99.9%
    measurement_window: "30d"
    
  enforcement_success_rate:
    target: 99.0%
    conditions:
      - operation_count: "<=1000"
      - duration: "<=300s"
      - provider: "spotify"
    
  api_latency:
    target: "300ms"
    percentile: 95
    endpoints: ["/v1/artists/search", "/v1/dnp/*"]
    
  plan_generation_time:
    target: "30s"
    conditions:
      - dnp_list_size: "<=1000"
      - library_size: "<=10000"
```

## Legal and Compliance Safeguards

### Content Policy Framework
```typescript
interface CommunityListPolicy {
  requireNeutralCriteria: boolean; // Must be true
  allowReasonDisplay: boolean; // Default false
  requireGovernanceUrl: boolean; // Must be true
  moderationQueue: boolean; // Must be true
  appealProcess: boolean; // Must be true
}

class ContentModerationService {
  validateListCriteria(criteria: string): ValidationResult {
    const prohibitedPatterns = [
      /\b(accused|alleged|guilty)\b/i,
      /\b(criminal|illegal|lawsuit)\b/i,
      /\b(bad|evil|terrible)\s+(person|artist)\b/i
    ];
    
    for (const pattern of prohibitedPatterns) {
      if (pattern.test(criteria)) {
        return {
          valid: false,
          reason: "Criteria must be neutral and factual"
        };
      }
    }
    
    return { valid: true };
  }
  
  generateNeutralDescription(userInput: string): string {
    // Transform user input into neutral, factual language
    return `User-defined preference list based on: ${this.sanitizeInput(userInput)}`;
  }
}
```

### Privacy Controls
```typescript
class PrivacyController {
  async exportUserData(userId: string): Promise<UserDataExport> {
    return {
      profile: await this.getUserProfile(userId),
      dnpLists: await this.getUserDNPLists(userId),
      subscriptions: await this.getUserSubscriptions(userId),
      actionHistory: await this.getUserActionHistory(userId),
      // Exclude: raw tokens, IP addresses, detailed logs
    };
  }
  
  async deleteUserData(userId: string): Promise<DeletionResult> {
    const operations = [
      () => this.deleteUserProfile(userId),
      () => this.deleteUserConnections(userId), // Revokes tokens
      () => this.deleteUserDNPLists(userId),
      () => this.deleteUserSubscriptions(userId),
      () => this.anonymizeAuditLogs(userId),
      () => this.deleteUserActionHistory(userId)
    ];
    
    const results = await Promise.allSettled(operations.map(op => op()));
    return this.summarizeDeletionResults(results);
  }
}
```

## Updated Delivery Plan

### Milestone 0 - Foundation & Entity Resolution (3-4 weeks)
- Entity resolution service with MusicBrainz/ISNI integration
- Spotify adapter with featured artist detection
- Basic web extension with resilient selectors
- Rate limiting framework and token vault service

### Milestone 1 - Core MVP (6-8 weeks)
- Complete Spotify enforcement with batch operations
- Web application with dry-run/execute workflows
- Browser extension with auto-skip and override controls
- Community lists with governance framework (private beta)
- Audit logging and basic observability

### Milestone 2 - Multi-Platform Beta (4-6 weeks)
- Apple Music adapter with token broker service
- YouTube Music extension hardening
- Tidal best-effort implementation
- Mobile assist via iOS Shortcuts/Android intents
- Performance optimization and SLO monitoring

### Milestone 3 - Production Hardening (3-4 weeks)
- SOC2-friendly security controls and logging
- Appeals process and content moderation tools
- Accessibility audit (WCAG 2.2 AA compliance)
- Load testing and capacity planning
- Legal review and ToS finalization
##
 Technology Stack Rationale

### Svelte Frontend
**Why Svelte over React/other frameworks:**
- **Compile-time optimizations**: Svelte compiles to vanilla JavaScript with no runtime overhead
- **Built-in dev server**: Fast development with Svelte's native tooling
- **Reactive by design**: Built-in reactivity without complex state management libraries
- **Smaller bundles**: No framework runtime, only your compiled component code
- **Better performance**: No virtual DOM overhead, direct DOM updates
- **Excellent TypeScript support**: First-class TypeScript integration
- **Simple mental model**: Easy to learn and reason about component behavior
- **Zero configuration**: Works out of the box without complex build setup

### Rust Backend
**Why Rust over Node.js/Python:**
- **Performance**: 10-100x faster for CPU-intensive tasks like entity resolution and batch processing
- **Memory safety**: Zero-cost abstractions prevent common security vulnerabilities
- **Concurrency**: Excellent async/await support for handling thousands of concurrent API requests
- **Type safety**: Compile-time guarantees prevent runtime errors in production
- **Ecosystem**: Mature libraries for HTTP (Axum), database (SQLx), and serialization (Serde)

**Rust Service Architecture:**
```rust
// High-performance entity resolution
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/v1/artists/resolve", post(resolve_artists))
        .route("/v1/enforcement/plan", post(plan_enforcement))
        .layer(TraceLayer::new_for_http())
        .with_state(AppState::new().await);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn resolve_artists(
    State(state): State<AppState>,
    Json(request): Json<ResolveRequest>
) -> Result<Json<ResolveResponse>, AppError> {
    let resolved = state.entity_resolver
        .resolve_concurrent(&request.queries)
        .await?;
    
    Ok(Json(ResolveResponse { artists: resolved }))
}
```

### DuckDB for Analytics
**Why DuckDB alongside PostgreSQL:**
- **Analytical queries**: Optimized for OLAP workloads like enforcement impact analysis
- **Columnar storage**: Efficient for aggregating large datasets (user libraries, action histories)
- **In-process**: No separate database server needed, embedded directly in Rust application
- **SQL compatibility**: Standard SQL interface for complex analytics queries
- **Parquet integration**: Direct import/export of user data in portable formats

**DuckDB Use Cases:**
```rust
// Fast analytics queries for enforcement planning
pub struct AnalyticsService {
    duck_conn: Arc<duckdb::Connection>,
}

impl AnalyticsService {
    pub async fn analyze_enforcement_impact(&self, user_id: Uuid, dnp_artists: &[Uuid]) -> Result<ImpactAnalysis> {
        let query = r#"
            SELECT 
                provider,
                COUNT(*) as total_tracks,
                COUNT(CASE WHEN artist_id = ANY($2) THEN 1 END) as blocked_tracks,
                AVG(play_count) as avg_play_count
            FROM user_library_snapshot 
            WHERE user_id = $1 
            GROUP BY provider
        "#;
        
        let results = self.duck_conn
            .prepare(query)?
            .query_map(params![user_id, dnp_artists], |row| {
                Ok(ProviderImpact {
                    provider: row.get(0)?,
                    total_tracks: row.get(1)?,
                    blocked_tracks: row.get(2)?,
                    avg_play_count: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
            
        Ok(ImpactAnalysis { providers: results })
    }
}
```

### Hybrid Data Architecture
**PostgreSQL for OLTP (Operational Data):**
- User accounts, connections, DNP lists
- Real-time operations requiring ACID transactions
- Relational integrity for user data

**DuckDB for OLAP (Analytics):**
- Enforcement impact analysis
- User behavior analytics
- Community list effectiveness metrics
- Historical reporting and trends

**Data Flow:**
```rust
// Periodic sync from PostgreSQL to DuckDB for analytics
pub async fn sync_analytics_data(&self) -> Result<()> {
    // Export recent actions from PostgreSQL
    let recent_actions = sqlx::query_as!(
        ActionItem,
        "SELECT * FROM action_items WHERE created_at > $1",
        self.last_sync_time
    )
    .fetch_all(&self.pg_pool)
    .await?;
    
    // Bulk insert into DuckDB for analytics
    let mut stmt = self.duck_conn.prepare(
        "INSERT INTO action_analytics SELECT * FROM read_json_auto(?)"
    )?;
    
    let json_data = serde_json::to_string(&recent_actions)?;
    stmt.execute([json_data])?;
    
    Ok(())
}
```

### Performance Benefits
**Rust + DuckDB Combination:**
- **Entity resolution**: 50-100ms for complex multi-provider artist disambiguation
- **Enforcement planning**: <5 seconds for 10k track libraries with 1k DNP artists
- **Analytics queries**: Sub-second response for complex aggregations over millions of records
- **Memory efficiency**: Rust's zero-cost abstractions + DuckDB's columnar storage
- **Concurrent processing**: Handle thousands of simultaneous enforcement operations

### Development Experience
**Svelte Frontend:**
```json
{
  "scripts": {
    "dev": "rollup -c -w",
    "build": "rollup -c",
    "start": "sirv public --no-clear",
    "check": "svelte-check --tsconfig ./tsconfig.json"
  },
  "dependencies": {
    "svelte": "^4.2.0",
    "tailwindcss": "^3.3.0"
  },
  "devDependencies": {
    "@rollup/plugin-commonjs": "^25.0.0",
    "@rollup/plugin-node-resolve": "^15.0.0",
    "@rollup/plugin-typescript": "^11.0.0",
    "rollup": "^4.0.0",
    "rollup-plugin-svelte": "^7.0.0",
    "rollup-plugin-terser": "^7.0.0",
    "svelte-check": "^3.6.0",
    "svelte-preprocess": "^5.0.0",
    "typescript": "^5.0.0",
    "sirv-cli": "^2.0.0"
  }
}
```

**Rust Backend:**
```toml
[dependencies]
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls"] }
duckdb = "0.9"
serde = { version = "1.0", features = ["derive"] }
tracing = "0.1"
```

This technology stack provides:
- **Fast development** with Svelte's built-in tooling and instant compilation
- **100x better performance** for compute-intensive operations with Rust
- **Optimal analytics** with DuckDB's columnar processing
- **Production reliability** with Rust's memory safety and type system
- **Minimal frontend overhead** with Svelte's compile-to-vanilla-JS approach
- **Scalable architecture** that can handle millions of users and enforcement operations
### S
velte Component Architecture
**State Management with Svelte Stores:**
```typescript
// stores/dnp.ts - Reactive DNP list management
import { writable, derived } from 'svelte/store';

export interface Artist {
  id: string;
  canonicalName: string;
  externalIds: Record<string, string>;
  metadata: {
    image?: string;
    genres: string[];
  };
}

export const dnpList = writable<Artist[]>([]);
export const selectedProviders = writable<string[]>(['spotify']);
export const enforcementOptions = writable({
  aggressiveness: 'moderate',
  blockCollabs: true,
  blockFeaturing: true
});

// Derived store for enforcement planning
export const enforcementPlan = derived(
  [dnpList, selectedProviders, enforcementOptions],
  async ([$dnpList, $providers, $options]) => {
    if ($dnpList.length === 0) return null;
    
    const response = await fetch('/api/v1/enforcement/plan', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        providers: $providers,
        options: $options,
        dryRun: true
      })
    });
    
    return response.json();
  }
);
```

**Component Example:**
```svelte
<!-- DNPManager.svelte -->
<script lang="ts">
  import { dnpList, enforcementPlan } from '../stores/dnp';
  import ArtistSearch from './ArtistSearch.svelte';
  import EnforcementPreview from './EnforcementPreview.svelte';
  
  let searchQuery = '';
  let showPreview = false;
  
  async function addArtist(artist: Artist) {
    dnpList.update(list => [...list, artist]);
    searchQuery = '';
  }
  
  function removeArtist(artistId: string) {
    dnpList.update(list => list.filter(a => a.id !== artistId));
  }
</script>

<div class="dnp-manager">
  <h2>Do Not Play List</h2>
  
  <ArtistSearch 
    bind:query={searchQuery}
    on:select={e => addArtist(e.detail)}
  />
  
  <div class="dnp-list">
    {#each $dnpList as artist (artist.id)}
      <div class="artist-item">
        <img src={artist.metadata.image} alt={artist.canonicalName} />
        <span>{artist.canonicalName}</span>
        <button on:click={() => removeArtist(artist.id)}>Remove</button>
      </div>
    {/each}
  </div>
  
  {#if $dnpList.length > 0}
    <button on:click={() => showPreview = true}>
      Preview Enforcement
    </button>
  {/if}
  
  {#if showPreview}
    {#await $enforcementPlan}
      <p>Calculating impact...</p>
    {:then plan}
      <EnforcementPreview {plan} />
    {:catch error}
      <p>Error: {error.message}</p>
    {/await}
  {/if}
</div>
```

This Svelte architecture provides:
- **Reactive state management** with built-in stores
- **Compile-time optimizations** for smaller bundle sizes
- **Type-safe components** with TypeScript integration
- **Efficient updates** with fine-grained reactivity
- **Simple async handling** with await blocks