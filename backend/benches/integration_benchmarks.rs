use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use kiro_backend::{
    create_app_state, create_router,
    models::{
        auth::{LoginRequest, RegisterRequest},
        dnp_list::AddToDnpRequest,
    },
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

async fn setup_test_app() -> axum::Router {
    let state = create_app_state().await.expect("Failed to create app state");
    create_router(state)
}

fn auth_benchmarks(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("user_registration", |b| {
        b.to_async(&rt).iter(|| async {
            let app = setup_test_app().await;
            
            let register_request = RegisterRequest {
                email: format!("test{}@example.com", rand::random::<u32>()),
                password: "secure_password123".to_string(),
            };
            
            let request = Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&register_request).unwrap()))
                .unwrap();
            
            let response = black_box(app.oneshot(request).await.unwrap());
            assert_eq!(response.status(), StatusCode::CREATED);
        })
    });
    
    c.bench_function("user_login", |b| {
        b.to_async(&rt).iter(|| async {
            let app = setup_test_app().await;
            
            // First register a user
            let email = format!("login_test{}@example.com", rand::random::<u32>());
            let register_request = RegisterRequest {
                email: email.clone(),
                password: "secure_password123".to_string(),
            };
            
            let register_req = Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&register_request).unwrap()))
                .unwrap();
            
            app.clone().oneshot(register_req).await.unwrap();
            
            // Now benchmark login
            let login_request = LoginRequest {
                email,
                password: "secure_password123".to_string(),
                totp_code: None,
            };
            
            let login_req = Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&login_request).unwrap()))
                .unwrap();
            
            let response = black_box(app.oneshot(login_req).await.unwrap());
            assert_eq!(response.status(), StatusCode::OK);
        })
    });
}

fn dnp_benchmarks(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("artist_search", |b| {
        b.to_async(&rt).iter(|| async {
            let app = setup_test_app().await;
            
            // Create authenticated request
            let request = Request::builder()
                .method("GET")
                .uri("/api/v1/artists/search?q=test&limit=20")
                .header("authorization", "Bearer test_token") // Mock token for benchmark
                .body(Body::empty())
                .unwrap();
            
            let response = black_box(app.oneshot(request).await.unwrap());
            // Note: This will return 401 without proper auth, but we're benchmarking the routing
        })
    });
    
    c.bench_function("dnp_list_retrieval", |b| {
        b.to_async(&rt).iter(|| async {
            let app = setup_test_app().await;
            
            let request = Request::builder()
                .method("GET")
                .uri("/api/v1/dnp?limit=50")
                .header("authorization", "Bearer test_token")
                .body(Body::empty())
                .unwrap();
            
            let response = black_box(app.oneshot(request).await.unwrap());
        })
    });
}

fn database_benchmarks(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("database_operations");
    group.measurement_time(Duration::from_secs(10));
    
    for batch_size in [1, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("batch_user_creation", batch_size),
            batch_size,
            |b, &batch_size| {
                b.to_async(&rt).iter(|| async {
                    let app = setup_test_app().await;
                    
                    for i in 0..batch_size {
                        let register_request = RegisterRequest {
                            email: format!("batch_test_{}_{i}@example.com", rand::random::<u32>()),
                            password: "secure_password123".to_string(),
                        };
                        
                        let request = Request::builder()
                            .method("POST")
                            .uri("/api/v1/auth/register")
                            .header("content-type", "application/json")
                            .body(Body::from(serde_json::to_string(&register_request).unwrap()))
                            .unwrap();
                        
                        black_box(app.clone().oneshot(request).await.unwrap());
                    }
                })
            },
        );
    }
    group.finish();
}

fn health_check_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("health_check", |b| {
        b.to_async(&rt).iter(|| async {
            let app = setup_test_app().await;
            
            let request = Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap();
            
            let response = black_box(app.oneshot(request).await.unwrap());
            assert_eq!(response.status(), StatusCode::OK);
        })
    });
}

fn concurrent_requests_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrent_requests");
    group.measurement_time(Duration::from_secs(15));
    
    for concurrency in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_health_checks", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let app = setup_test_app().await;
                    
                    let mut handles = Vec::new();
                    
                    for _ in 0..concurrency {
                        let app_clone = app.clone();
                        let handle = tokio::spawn(async move {
                            let request = Request::builder()
                                .method("GET")
                                .uri("/health")
                                .body(Body::empty())
                                .unwrap();
                            
                            app_clone.oneshot(request).await.unwrap()
                        });
                        handles.push(handle);
                    }
                    
                    for handle in handles {
                        let response = black_box(handle.await.unwrap());
                        assert_eq!(response.status(), StatusCode::OK);
                    }
                })
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    auth_benchmarks,
    dnp_benchmarks,
    database_benchmarks,
    health_check_benchmark,
    concurrent_requests_benchmark
);
criterion_main!(benches);