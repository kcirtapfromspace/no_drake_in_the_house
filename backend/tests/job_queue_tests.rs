use music_streaming_blocklist_backend::*;
use std::sync::Arc;
use tokio_test;
use uuid::Uuid;
use chrono::Utc;

#[tokio::test]
async fn test_job_creation_and_status() {
    let job = Job {
        id: Uuid::new_v4(),
        job_type: JobType::EnforcementExecution,
        priority: JobPriority::Normal,
        payload: serde_json::json!({"test": "data"}),
        user_id: Some(Uuid::new_v4()),
        provider: Some("spotify".to_string()),
        status: JobStatus::Pending,
        created_at: Utc::now(),
        scheduled_at: Utc::now(),
        started_at: None,
        completed_at: None,
        retry_count: 0,
        max_retries: 3,
        error_message: None,
        progress: None,
    };

    assert_eq!(job.status, JobStatus::Pending);
    assert_eq!(job.retry_count, 0);
    assert!(job.started_at.is_none());
    assert_eq!(job.job_type, JobType::EnforcementExecution);
    assert_eq!(job.priority, JobPriority::Normal);
}

#[tokio::test]
async fn test_job_progress_tracking() {
    let progress = JobProgress {
        current_step: "Processing batch 2 of 5".to_string(),
        total_steps: 5,
        completed_steps: 2,
        percentage: 40.0,
        estimated_remaining_ms: 180000,
        details: serde_json::json!({"current_batch": 2}),
    };

    assert_eq!(progress.percentage, 40.0);
    assert_eq!(progress.completed_steps, 2);
    assert_eq!(progress.total_steps, 5);
    assert_eq!(progress.current_step, "Processing batch 2 of 5");
}

#[tokio::test]
async fn test_worker_config_defaults() {
    let config = WorkerConfig::default();
    
    assert!(!config.worker_id.is_empty());
    assert_eq!(config.concurrency, 2);
    assert_eq!(config.job_types, vec![JobType::EnforcementExecution]);
    assert_eq!(config.poll_interval_ms, 1000);
    assert_eq!(config.max_execution_time_ms, 300000);
    assert_eq!(config.heartbeat_interval_ms, 30000);
}

#[tokio::test]
async fn test_job_queue_service_creation() {
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    // Create a mock rate limiter
    let rate_limiter = match RateLimitingService::new(&redis_url) {
        Ok(service) => Arc::new(service),
        Err(_) => {
            println!("Skipping job queue test - Redis not available");
            return;
        }
    };

    let job_queue = JobQueueService::new(&redis_url, rate_limiter);
    assert!(job_queue.is_ok(), "Should be able to create job queue service");
}

#[tokio::test]
async fn test_job_priority_ordering() {
    let mut jobs = vec![
        Job {
            id: Uuid::new_v4(),
            job_type: JobType::EnforcementExecution,
            priority: JobPriority::Low,
            payload: serde_json::json!({}),
            user_id: Some(Uuid::new_v4()),
            provider: Some("spotify".to_string()),
            status: JobStatus::Pending,
            created_at: Utc::now(),
            scheduled_at: Utc::now(),
            started_at: None,
            completed_at: None,
            retry_count: 0,
            max_retries: 3,
            error_message: None,
            progress: None,
        },
        Job {
            id: Uuid::new_v4(),
            job_type: JobType::TokenRefresh,
            priority: JobPriority::Critical,
            payload: serde_json::json!({}),
            user_id: Some(Uuid::new_v4()),
            provider: Some("spotify".to_string()),
            status: JobStatus::Pending,
            created_at: Utc::now(),
            scheduled_at: Utc::now(),
            started_at: None,
            completed_at: None,
            retry_count: 0,
            max_retries: 3,
            error_message: None,
            progress: None,
        },
        Job {
            id: Uuid::new_v4(),
            job_type: JobType::LibraryScan,
            priority: JobPriority::High,
            payload: serde_json::json!({}),
            user_id: Some(Uuid::new_v4()),
            provider: Some("spotify".to_string()),
            status: JobStatus::Pending,
            created_at: Utc::now(),
            scheduled_at: Utc::now(),
            started_at: None,
            completed_at: None,
            retry_count: 0,
            max_retries: 3,
            error_message: None,
            progress: None,
        },
    ];

    // Sort by priority (highest first) then by created_at
    jobs.sort_by(|a, b| {
        b.priority.cmp(&a.priority)
            .then_with(|| a.created_at.cmp(&b.created_at))
    });

    assert_eq!(jobs[0].priority, JobPriority::Critical);
    assert_eq!(jobs[1].priority, JobPriority::High);
    assert_eq!(jobs[2].priority, JobPriority::Low);
}

#[tokio::test]
async fn test_job_type_variants() {
    let job_types = vec![
        JobType::EnforcementExecution,
        JobType::BatchRollback,
        JobType::TokenRefresh,
        JobType::LibraryScan,
        JobType::CommunityListUpdate,
        JobType::HealthCheck,
    ];

    // Test that all job types can be serialized and deserialized
    for job_type in job_types {
        let serialized = serde_json::to_string(&job_type).unwrap();
        let deserialized: JobType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(job_type, deserialized);
    }
}

#[tokio::test]
async fn test_job_status_transitions() {
    let statuses = vec![
        JobStatus::Pending,
        JobStatus::Processing,
        JobStatus::Completed,
        JobStatus::Failed,
        JobStatus::Retrying,
        JobStatus::DeadLetter,
    ];

    // Test that all statuses can be serialized and deserialized
    for status in statuses {
        let serialized = serde_json::to_string(&status).unwrap();
        let deserialized: JobStatus = serde_json::from_str(&serialized).unwrap();
        assert_eq!(status, deserialized);
    }
}

// Mock job handler for testing
struct MockJobHandler {
    job_type: JobType,
    should_fail: bool,
}

impl MockJobHandler {
    fn new(job_type: JobType, should_fail: bool) -> Self {
        Self { job_type, should_fail }
    }
}

#[async_trait::async_trait]
impl JobHandler for MockJobHandler {
    async fn handle(&self, job: &Job) -> anyhow::Result<serde_json::Value> {
        if self.should_fail {
            anyhow::bail!("Mock job handler failure");
        }
        
        Ok(serde_json::json!({
            "job_id": job.id,
            "processed_at": Utc::now(),
            "result": "success"
        }))
    }

    fn job_type(&self) -> JobType {
        self.job_type.clone()
    }

    fn max_execution_time(&self) -> std::time::Duration {
        std::time::Duration::from_secs(60)
    }
}

#[tokio::test]
async fn test_mock_job_handler() {
    let handler = MockJobHandler::new(JobType::HealthCheck, false);
    
    let job = Job {
        id: Uuid::new_v4(),
        job_type: JobType::HealthCheck,
        priority: JobPriority::Normal,
        payload: serde_json::json!({}),
        user_id: Some(Uuid::new_v4()),
        provider: None,
        status: JobStatus::Pending,
        created_at: Utc::now(),
        scheduled_at: Utc::now(),
        started_at: None,
        completed_at: None,
        retry_count: 0,
        max_retries: 3,
        error_message: None,
        progress: None,
    };

    let result = handler.handle(&job).await;
    assert!(result.is_ok());
    
    let result_value = result.unwrap();
    assert!(result_value.get("job_id").is_some());
    assert_eq!(result_value.get("result").unwrap(), "success");
}

#[tokio::test]
async fn test_mock_job_handler_failure() {
    let handler = MockJobHandler::new(JobType::HealthCheck, true);
    
    let job = Job {
        id: Uuid::new_v4(),
        job_type: JobType::HealthCheck,
        priority: JobPriority::Normal,
        payload: serde_json::json!({}),
        user_id: Some(Uuid::new_v4()),
        provider: None,
        status: JobStatus::Pending,
        created_at: Utc::now(),
        scheduled_at: Utc::now(),
        started_at: None,
        completed_at: None,
        retry_count: 0,
        max_retries: 3,
        error_message: None,
        progress: None,
    };

    let result = handler.handle(&job).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Mock job handler failure"));
}