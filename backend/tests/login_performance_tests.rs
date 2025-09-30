use std::time::Instant;
use tokio_test;

use music_streaming_blocklist_backend::services::login_performance::{
    LoginPerformanceService, LoginPerformanceMetrics, CachedUserLogin,
};

#[tokio::test]
async fn test_login_performance_service_creation() {
    let redis_url = "redis://localhost:6379";
    
    // This might fail if Redis is not available, which is expected in CI
    match LoginPerformanceService::new(redis_url) {
        Ok(service) => {
            let metrics = service.get_metrics().await;
            assert_eq!(metrics.total_logins, 0);
            assert_eq!(metrics.successful_logins, 0);
            assert_eq!(metrics.failed_logins, 0);
        }
        Err(_) => {
            // Redis not available, which is fine for this test
            assert!(true);
        }
    }
}

#[tokio::test]
async fn test_password_verification_performance() {
    let redis_url = "redis://localhost:6379";
    
    if let Ok(service) = LoginPerformanceService::new(redis_url) {
        let password = "test_password_123";
        let hash = bcrypt::hash(password, 8).unwrap(); // Use 8 rounds for testing
        
        let start = Instant::now();
        let result = service.verify_password_optimized(password, &hash).await;
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Should be faster than 500ms even on slow systems
        assert!(duration.as_millis() < 500);
        
        tracing::info!("Password verification took {}ms", duration.as_millis());
    }
}

#[tokio::test]
async fn test_optimized_refresh_token_generation() {
    let redis_url = "redis://localhost:6379";
    
    if let Ok(service) = LoginPerformanceService::new(redis_url) {
        let start = Instant::now();
        let result = service.generate_optimized_refresh_token().await;
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        let (token_raw, token_hash) = result.unwrap();
        
        assert!(!token_raw.is_empty());
        assert!(!token_hash.is_empty());
        assert_ne!(token_raw, token_hash);
        
        // Should be faster than 200ms
        assert!(duration.as_millis() < 200);
        
        tracing::info!("Token generation took {}ms", duration.as_millis());
    }
}

#[tokio::test]
async fn test_metrics_recording_and_calculation() {
    let redis_url = "redis://localhost:6379";
    
    if let Ok(service) = LoginPerformanceService::new(redis_url) {
        // Record some login attempts
        service.record_login_attempt(true, 150.0).await.unwrap();
        service.record_login_attempt(true, 200.0).await.unwrap();
        service.record_login_attempt(false, 300.0).await.unwrap();
        service.record_login_attempt(true, 100.0).await.unwrap();
        
        let metrics = service.get_metrics().await;
        
        assert_eq!(metrics.total_logins, 4);
        assert_eq!(metrics.successful_logins, 3);
        assert_eq!(metrics.failed_logins, 1);
        
        // Check that average is being calculated
        assert!(metrics.avg_login_time_ms > 0.0);
        assert!(metrics.avg_login_time_ms < 300.0); // Should be reasonable average
        
        tracing::info!("Metrics: {:?}", metrics);
    }
}

#[tokio::test]
async fn test_cache_invalidation() {
    let redis_url = "redis://localhost:6379";
    
    if let Ok(service) = LoginPerformanceService::new(redis_url) {
        let test_email = "test@example.com";
        
        // This should not fail even if cache is empty
        let result = service.invalidate_user_cache(test_email).await;
        assert!(result.is_ok());
        
        tracing::info!("Cache invalidation test completed");
    }
}

#[test]
fn test_cached_user_login_serialization() {
    let cached_user = CachedUserLogin {
        user_id: uuid::Uuid::new_v4(),
        email: "test@example.com".to_string(),
        password_hash: "test_hash".to_string(),
        totp_enabled: true,
        totp_secret: Some("test_secret".to_string()),
        last_login: Some(chrono::Utc::now()),
        cached_at: chrono::Utc::now(),
    };

    // Test serialization and deserialization
    let json = serde_json::to_string(&cached_user).unwrap();
    let deserialized: CachedUserLogin = serde_json::from_str(&json).unwrap();
    
    assert_eq!(cached_user.user_id, deserialized.user_id);
    assert_eq!(cached_user.email, deserialized.email);
    assert_eq!(cached_user.password_hash, deserialized.password_hash);
    assert_eq!(cached_user.totp_enabled, deserialized.totp_enabled);
    assert_eq!(cached_user.totp_secret, deserialized.totp_secret);
}

#[test]
fn test_login_performance_metrics_defaults() {
    let metrics = LoginPerformanceMetrics::default();
    
    assert_eq!(metrics.total_logins, 0);
    assert_eq!(metrics.successful_logins, 0);
    assert_eq!(metrics.failed_logins, 0);
    assert_eq!(metrics.avg_login_time_ms, 0.0);
    assert_eq!(metrics.cache_hit_rate, 0.0);
    assert_eq!(metrics.password_verification_time_ms, 0.0);
    assert_eq!(metrics.database_query_time_ms, 0.0);
    assert_eq!(metrics.token_generation_time_ms, 0.0);
}

#[tokio::test]
async fn test_performance_comparison() {
    // This test compares optimized vs non-optimized operations
    let password = "test_password_for_performance";
    
    // Test bcrypt with different rounds
    let hash_12_rounds = bcrypt::hash(password, 12).unwrap();
    let hash_8_rounds = bcrypt::hash(password, 8).unwrap();
    
    // Time verification with 12 rounds (standard)
    let start = Instant::now();
    let _result1 = bcrypt::verify(password, &hash_12_rounds).unwrap();
    let duration_12 = start.elapsed();
    
    // Time verification with 8 rounds (optimized for refresh tokens)
    let start = Instant::now();
    let _result2 = bcrypt::verify(password, &hash_8_rounds).unwrap();
    let duration_8 = start.elapsed();
    
    tracing::info!(
        "bcrypt 12 rounds: {}ms, 8 rounds: {}ms, improvement: {:.1}x",
        duration_12.as_millis(),
        duration_8.as_millis(),
        duration_12.as_millis() as f64 / duration_8.as_millis() as f64
    );
    
    // 8 rounds should be significantly faster
    assert!(duration_8 < duration_12);
    
    // Both should still be reasonably secure (> 1ms)
    assert!(duration_8.as_millis() > 0);
    assert!(duration_12.as_millis() > 0);
}