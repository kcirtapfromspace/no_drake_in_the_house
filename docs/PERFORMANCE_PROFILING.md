# Performance Profiling Guide

This guide covers performance profiling tools and techniques for the No Drake in the House platform.

## 🎯 Overview

Performance profiling helps identify bottlenecks and optimize the application. This guide covers:

- **Backend profiling** (Rust/Axum API)
- **Frontend profiling** (Svelte application)
- **Database profiling** (PostgreSQL queries)
- **Infrastructure monitoring** (Docker, Kubernetes)

## 🦀 Backend Performance Profiling

### Rust Profiling Tools

#### 1. Built-in Benchmarking

```bash
# Add to backend/Cargo.toml
[dev-dependencies]
criterion = "0.5"

# Create benchmark file: backend/benches/api_benchmarks.rs
```

Example benchmark:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kiro_backend::services::auth::AuthService;

fn auth_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("user_login", |b| {
        b.to_async(&rt).iter(|| async {
            // Benchmark login performance
            black_box(auth_service.login(login_request).await)
        })
    });
}

criterion_group!(benches, auth_benchmark);
criterion_main!(benches);
```

Run benchmarks:
```bash
cd backend
cargo bench
```

#### 2. CPU Profiling with perf

```bash
# Install perf (Linux)
sudo apt install linux-perf

# Profile the backend
cd backend
cargo build --release
perf record --call-graph=dwarf ./target/release/kiro-backend
perf report

# Generate flamegraph
cargo install flamegraph
cargo flamegraph --bin kiro-backend
```

#### 3. Memory Profiling with Valgrind

```bash
# Install valgrind
sudo apt install valgrind

# Profile memory usage
cd backend
cargo build
valgrind --tool=memcheck --leak-check=full ./target/debug/kiro-backend

# Profile cache performance
valgrind --tool=cachegrind ./target/debug/kiro-backend
```

#### 4. Async Profiling with tokio-console

Add to `backend/Cargo.toml`:
```toml
[dependencies]
console-subscriber = "0.2"
tokio = { version = "1.0", features = ["full", "tracing"] }
```

Add to `backend/src/main.rs`:
```rust
#[tokio::main]
async fn main() {
    // Enable tokio-console
    console_subscriber::init();
    
    // Your application code
}
```

Run with console:
```bash
# Terminal 1: Start backend with console
cd backend
RUSTFLAGS="--cfg tokio_unstable" cargo run

# Terminal 2: Connect console
cargo install --locked tokio-console
tokio-console
```

### HTTP Load Testing

#### 1. Using wrk

```bash
# Install wrk
brew install wrk  # macOS
sudo apt install wrk  # Ubuntu

# Basic load test
wrk -t12 -c400 -d30s http://localhost:3000/health

# Test authentication endpoint
wrk -t4 -c100 -d10s -s scripts/auth_test.lua http://localhost:3000/api/v1/auth/login
```

Create `scripts/auth_test.lua`:
```lua
wrk.method = "POST"
wrk.body = '{"email": "test@example.com", "password": "testpass123"}'
wrk.headers["Content-Type"] = "application/json"
```

#### 2. Using Apache Bench (ab)

```bash
# Install ab
sudo apt install apache2-utils

# Simple load test
ab -n 1000 -c 10 http://localhost:3000/health

# POST request test
ab -n 100 -c 5 -p auth_payload.json -T application/json http://localhost:3000/api/v1/auth/login
```

#### 3. Using Artillery

```bash
# Install Artillery
npm install -g artillery

# Create test configuration
cat > artillery-config.yml << EOF
config:
  target: 'http://localhost:3000'
  phases:
    - duration: 60
      arrivalRate: 10
scenarios:
  - name: "Health check"
    requests:
      - get:
          url: "/health"
  - name: "Authentication flow"
    requests:
      - post:
          url: "/api/v1/auth/login"
          json:
            email: "test@example.com"
            password: "testpass123"
EOF

# Run load test
artillery run artillery-config.yml
```

### Database Query Profiling

#### 1. PostgreSQL Query Analysis

```sql
-- Enable query logging
ALTER SYSTEM SET log_statement = 'all';
ALTER SYSTEM SET log_min_duration_statement = 100; -- Log queries > 100ms
SELECT pg_reload_conf();

-- Analyze slow queries
SELECT query, mean_exec_time, calls, total_exec_time
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;

-- Explain query performance
EXPLAIN (ANALYZE, BUFFERS) 
SELECT * FROM users WHERE email = 'test@example.com';
```

#### 2. SQLx Query Profiling

Add to `backend/src/main.rs`:
```rust
use sqlx::postgres::PgPoolOptions;
use tracing::{info, instrument};

#[instrument]
async fn create_db_pool() -> sqlx::PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .expect("Failed to create pool");
    
    info!("Database pool created with {} connections", pool.size());
    pool
}
```

### Application Metrics

#### 1. Prometheus Metrics

The backend already includes Prometheus metrics. View them:
```bash
curl http://localhost:3000/metrics
```

Key metrics to monitor:
- `http_requests_total` - Request count by endpoint
- `http_request_duration_seconds` - Request latency
- `database_connections_active` - Active DB connections
- `auth_attempts_total` - Authentication attempts

#### 2. Custom Metrics

Add custom metrics in `backend/src/metrics.rs`:
```rust
use prometheus::{Counter, Histogram, register_counter, register_histogram};

lazy_static! {
    static ref DNP_OPERATIONS: Counter = register_counter!(
        "dnp_operations_total",
        "Total DNP list operations"
    ).unwrap();
    
    static ref QUERY_DURATION: Histogram = register_histogram!(
        "database_query_duration_seconds",
        "Database query duration"
    ).unwrap();
}

// Use in service methods
pub async fn add_to_dnp_list(&self, request: AddToDnpRequest) -> Result<DnpEntry> {
    let _timer = QUERY_DURATION.start_timer();
    DNP_OPERATIONS.inc();
    
    // Your implementation
}
```

## ⚛️ Frontend Performance Profiling

### Browser DevTools

#### 1. Performance Tab

1. Open Chrome DevTools (F12)
2. Go to **Performance** tab
3. Click **Record** 🔴
4. Interact with the application
5. Click **Stop** ⏹️
6. Analyze the timeline

Key metrics to look for:
- **FCP** (First Contentful Paint)
- **LCP** (Largest Contentful Paint)
- **FID** (First Input Delay)
- **CLS** (Cumulative Layout Shift)

#### 2. Lighthouse Audit

1. Open DevTools
2. Go to **Lighthouse** tab
3. Select **Performance** category
4. Click **Generate report**

Or use CLI:
```bash
npm install -g lighthouse
lighthouse http://localhost:5000 --output html --output-path ./lighthouse-report.html
```

#### 3. Memory Profiling

1. DevTools > **Memory** tab
2. Select **Heap snapshot**
3. Take snapshots before/after operations
4. Compare to find memory leaks

### Svelte Performance Tools

#### 1. Svelte DevTools

Install browser extension:
- [Chrome](https://chrome.google.com/webstore/detail/svelte-devtools/ckolcbmkjpjmangdbmnkpjigpkddpogn)
- [Firefox](https://addons.mozilla.org/en-US/firefox/addon/svelte-devtools/)

Features:
- Component tree inspection
- Props and state monitoring
- Performance profiling

#### 2. Bundle Analysis

```bash
cd frontend

# Install bundle analyzer
npm install --save-dev rollup-plugin-analyzer

# Add to rollup.config.js
import { analyzer } from 'rollup-plugin-analyzer'

export default {
  plugins: [
    // ... other plugins
    analyzer({ summaryOnly: true })
  ]
}

# Build and analyze
npm run build
```

#### 3. Vite Performance

```bash
cd frontend

# Build with analysis
npm run build -- --analyze

# Preview production build
npm run preview
```

### Web Vitals Monitoring

Add to `frontend/src/main.ts`:
```typescript
import { getCLS, getFID, getFCP, getLCP, getTTFB } from 'web-vitals';

function sendToAnalytics(metric: any) {
  console.log('Web Vital:', metric);
  // Send to your analytics service
}

getCLS(sendToAnalytics);
getFID(sendToAnalytics);
getFCP(sendToAnalytics);
getLCP(sendToAnalytics);
getTTFB(sendToAnalytics);
```

## 🗄️ Database Performance

### PostgreSQL Monitoring

#### 1. Built-in Statistics

```sql
-- Connection statistics
SELECT * FROM pg_stat_database WHERE datname = 'kiro_dev';

-- Table statistics
SELECT * FROM pg_stat_user_tables;

-- Index usage
SELECT 
    schemaname,
    tablename,
    indexname,
    idx_scan,
    idx_tup_read,
    idx_tup_fetch
FROM pg_stat_user_indexes
ORDER BY idx_scan DESC;

-- Lock monitoring
SELECT * FROM pg_locks WHERE NOT granted;
```

#### 2. Query Performance

```sql
-- Enable pg_stat_statements
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- Top slow queries
SELECT 
    query,
    calls,
    total_exec_time,
    mean_exec_time,
    stddev_exec_time,
    rows
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;

-- Reset statistics
SELECT pg_stat_statements_reset();
```

#### 3. Index Analysis

```sql
-- Missing indexes (queries doing seq scans)
SELECT 
    schemaname,
    tablename,
    seq_scan,
    seq_tup_read,
    idx_scan,
    seq_tup_read / seq_scan as avg_seq_read
FROM pg_stat_user_tables
WHERE seq_scan > 0
ORDER BY seq_tup_read DESC;

-- Unused indexes
SELECT 
    schemaname,
    tablename,
    indexname,
    idx_scan
FROM pg_stat_user_indexes
WHERE idx_scan = 0;
```

### Redis Performance

```bash
# Redis CLI monitoring
redis-cli monitor

# Redis statistics
redis-cli info stats

# Memory usage
redis-cli info memory

# Slow log
redis-cli slowlog get 10
```

## 🐳 Infrastructure Monitoring

### Docker Performance

#### 1. Container Stats

```bash
# Real-time stats
docker stats

# Specific container
docker stats kiro-backend

# Export stats to file
docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}" > docker-stats.txt
```

#### 2. Container Resource Usage

```bash
# Inspect container resources
docker inspect kiro-backend | jq '.[0].HostConfig.Memory'

# Check container logs for OOM kills
dmesg | grep -i "killed process"

# Monitor disk usage
docker system df
```

### Kubernetes Monitoring

#### 1. Resource Usage

```bash
# Node resource usage
kubectl top nodes

# Pod resource usage
kubectl top pods -n kiro-dev

# Describe pod for resource limits
kubectl describe pod <pod-name> -n kiro-dev
```

#### 2. Metrics Server

```bash
# Install metrics server (if not present)
kubectl apply -f https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml

# Check metrics
kubectl get --raw /apis/metrics.k8s.io/v1beta1/nodes
kubectl get --raw /apis/metrics.k8s.io/v1beta1/pods
```

#### 3. Prometheus & Grafana

Deploy monitoring stack:
```bash
# Add Prometheus Helm repo
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update

# Install Prometheus + Grafana
helm install monitoring prometheus-community/kube-prometheus-stack \
  --namespace monitoring \
  --create-namespace

# Port forward to access
kubectl port-forward -n monitoring svc/monitoring-grafana 3000:80
kubectl port-forward -n monitoring svc/monitoring-kube-prometheus-prometheus 9090:9090
```

## 📊 Performance Testing Scripts

### Automated Performance Tests

Create `scripts/performance-test.sh`:
```bash
#!/bin/bash

set -euo pipefail

echo "🚀 Starting performance test suite..."

# Start services
make dev

# Wait for services to be ready
sleep 10

# Backend load test
echo "📊 Testing backend performance..."
wrk -t4 -c50 -d30s --latency http://localhost:3000/health > backend-perf.txt

# Frontend lighthouse test
echo "🔍 Testing frontend performance..."
lighthouse http://localhost:5000 \
  --output json \
  --output-path frontend-perf.json \
  --chrome-flags="--headless"

# Database performance test
echo "🗄️ Testing database performance..."
cd backend
cargo bench > db-perf.txt

echo "✅ Performance tests complete!"
echo "📋 Results:"
echo "  Backend: backend-perf.txt"
echo "  Frontend: frontend-perf.json"
echo "  Database: db-perf.txt"
```

### Continuous Performance Monitoring

Add to `.github/workflows/performance.yml`:
```yaml
name: Performance Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  performance:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup environment
      run: |
        make setup
        make dev
    
    - name: Run backend benchmarks
      run: |
        cd backend
        cargo bench
    
    - name: Run load tests
      run: |
        # Install wrk
        sudo apt-get update
        sudo apt-get install wrk
        
        # Wait for services
        sleep 30
        
        # Run load test
        wrk -t2 -c10 -d10s http://localhost:3000/health
    
    - name: Lighthouse CI
      uses: treosh/lighthouse-ci-action@v10
      with:
        urls: |
          http://localhost:5000
        uploadArtifacts: true
```

## 🎯 Performance Optimization Tips

### Backend Optimization

1. **Database Connection Pooling**:
   ```rust
   let pool = PgPoolOptions::new()
       .max_connections(10)
       .min_connections(2)
       .acquire_timeout(Duration::from_secs(3))
       .idle_timeout(Duration::from_secs(600))
       .connect(&database_url).await?;
   ```

2. **Async Optimization**:
   ```rust
   // Use join! for concurrent operations
   let (user, settings) = tokio::join!(
       get_user(user_id),
       get_user_settings(user_id)
   );
   ```

3. **Caching**:
   ```rust
   // Redis caching for expensive operations
   let cache_key = format!("user:{}:profile", user_id);
   if let Ok(cached) = redis.get(&cache_key).await {
       return Ok(cached);
   }
   ```

### Frontend Optimization

1. **Code Splitting**:
   ```javascript
   // Lazy load components
   const DnpManager = lazy(() => import('./DnpManager.svelte'));
   ```

2. **Bundle Optimization**:
   ```javascript
   // Tree shaking in rollup.config.js
   export default {
     treeshake: {
       moduleSideEffects: false
     }
   };
   ```

3. **Image Optimization**:
   ```html
   <!-- Use modern formats -->
   <picture>
     <source srcset="image.webp" type="image/webp">
     <img src="image.jpg" alt="Description" loading="lazy">
   </picture>
   ```

### Database Optimization

1. **Indexing Strategy**:
   ```sql
   -- Composite indexes for common queries
   CREATE INDEX idx_user_artist_blocks_user_created 
   ON user_artist_blocks(user_id, created_at DESC);
   
   -- Partial indexes for filtered queries
   CREATE INDEX idx_users_active 
   ON users(email) WHERE active = true;
   ```

2. **Query Optimization**:
   ```sql
   -- Use LIMIT for pagination
   SELECT * FROM artists 
   WHERE canonical_name ILIKE $1 
   ORDER BY canonical_name 
   LIMIT 20 OFFSET $2;
   
   -- Use EXISTS instead of IN for large datasets
   SELECT * FROM users u 
   WHERE EXISTS (
     SELECT 1 FROM user_artist_blocks b 
     WHERE b.user_id = u.id
   );
   ```

## 📈 Monitoring Dashboard

### Grafana Dashboard

Import dashboard configuration:
```json
{
  "dashboard": {
    "title": "No Drake in the House Performance",
    "panels": [
      {
        "title": "HTTP Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])"
          }
        ]
      },
      {
        "title": "Response Time",
        "type": "graph", 
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))"
          }
        ]
      }
    ]
  }
}
```

### Alerting Rules

Create `monitoring/alerts.yml`:
```yaml
groups:
- name: performance
  rules:
  - alert: HighResponseTime
    expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 1
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High response time detected"
      
  - alert: HighErrorRate
    expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
    for: 2m
    labels:
      severity: critical
    annotations:
      summary: "High error rate detected"
```

This comprehensive performance profiling guide provides the tools and techniques needed to identify bottlenecks and optimize the No Drake in the House platform across all layers of the stack.