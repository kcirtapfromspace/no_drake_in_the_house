use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use deadpool_redis::{Config, Pool, Runtime};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Instant};
use uuid::Uuid;

// crate::models imported but unused batch types removed for cleaner code
use crate::services::RateLimitingService;

/// Job types for the queue system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum JobType {
    EnforcementExecution,
    BatchRollback,
    TokenRefresh,
    LibraryScan,
    CommunityListUpdate,
    HealthCheck,
}

/// Job status in the queue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Retrying,
    DeadLetter,
}

/// Job priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum JobPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Job definition for the queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub job_type: JobType,
    pub priority: JobPriority,
    pub payload: serde_json::Value,
    pub user_id: Option<Uuid>,
    pub provider: Option<String>,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub scheduled_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub error_message: Option<String>,
    pub progress: Option<JobProgress>,
}

/// Job progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobProgress {
    pub current_step: String,
    pub total_steps: u32,
    pub completed_steps: u32,
    pub percentage: f64,
    pub estimated_remaining_ms: u64,
    pub details: serde_json::Value,
}

/// Job execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub retry_count: u32,
}

/// Worker configuration
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub worker_id: String,
    pub concurrency: usize,
    pub job_types: Vec<JobType>,
    pub poll_interval_ms: u64,
    pub max_execution_time_ms: u64,
    pub heartbeat_interval_ms: u64,
}

/// Configuration for Redis SCAN operations
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// Batch size for SCAN operations (default: 100)
    pub batch_size: usize,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self { batch_size: 100 }
    }
}

/// Job queue service with Redis backend
pub struct JobQueueService {
    redis_pool: Pool,
    rate_limiter: Arc<RateLimitingService>,
    workers: Arc<RwLock<HashMap<String, WorkerHandle>>>,
    job_handlers: Arc<RwLock<HashMap<JobType, Box<dyn JobHandler + Send + Sync>>>>,
    scan_config: ScanConfig,
}

/// Worker handle for managing worker processes
#[derive(Debug)]
pub struct WorkerHandle {
    pub worker_id: String,
    pub config: WorkerConfig,
    pub status: WorkerStatus,
    pub last_heartbeat: DateTime<Utc>,
    pub jobs_processed: u64,
    pub current_job: Option<Uuid>,
    pub shutdown_tx: Option<mpsc::Sender<()>>,
}

/// Worker status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkerStatus {
    Starting,
    Running,
    Idle,
    Busy,
    Stopping,
    Stopped,
    Error,
}

/// Trait for job handlers
#[async_trait::async_trait]
pub trait JobHandler {
    async fn handle(&self, job: &Job) -> Result<serde_json::Value>;
    fn job_type(&self) -> JobType;
    fn max_execution_time(&self) -> Duration;
}

impl JobQueueService {
    pub fn new(redis_url: &str, rate_limiter: Arc<RateLimitingService>) -> Result<Self> {
        Self::with_scan_config(redis_url, rate_limiter, ScanConfig::default())
    }

    pub fn with_scan_config(
        redis_url: &str,
        rate_limiter: Arc<RateLimitingService>,
        scan_config: ScanConfig,
    ) -> Result<Self> {
        let config = Config::from_url(redis_url);
        let pool = config.create_pool(Some(Runtime::Tokio1))?;

        Ok(Self {
            redis_pool: pool,
            rate_limiter,
            workers: Arc::new(RwLock::new(HashMap::new())),
            job_handlers: Arc::new(RwLock::new(HashMap::new())),
            scan_config,
        })
    }

    /// Register a job handler for a specific job type
    pub async fn register_handler<H>(&self, handler: H) -> Result<()>
    where
        H: JobHandler + Send + Sync + 'static,
    {
        let mut handlers = self.job_handlers.write().await;
        handlers.insert(handler.job_type(), Box::new(handler));
        Ok(())
    }

    /// Enqueue a new job
    pub async fn enqueue_job(
        &self,
        job_type: JobType,
        payload: serde_json::Value,
        priority: JobPriority,
        user_id: Option<Uuid>,
        provider: Option<String>,
        scheduled_at: Option<DateTime<Utc>>,
    ) -> Result<Uuid> {
        let job = Job {
            id: Uuid::new_v4(),
            job_type,
            priority,
            payload,
            user_id,
            provider,
            status: JobStatus::Pending,
            created_at: Utc::now(),
            scheduled_at: scheduled_at.unwrap_or_else(Utc::now),
            started_at: None,
            completed_at: None,
            retry_count: 0,
            max_retries: 3,
            error_message: None,
            progress: None,
        };

        self.save_job(&job).await?;
        self.add_to_queue(&job).await?;

        tracing::info!(
            "Enqueued job {} of type {:?} with priority {:?}",
            job.id,
            job.job_type,
            job.priority
        );

        Ok(job.id)
    }

    /// Start a worker with the given configuration
    pub async fn start_worker(&self, config: WorkerConfig) -> Result<()> {
        let worker_id = config.worker_id.clone();
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

        let handle = WorkerHandle {
            worker_id: worker_id.clone(),
            config: config.clone(),
            status: WorkerStatus::Starting,
            last_heartbeat: Utc::now(),
            jobs_processed: 0,
            current_job: None,
            shutdown_tx: Some(shutdown_tx),
        };

        // Store worker handle
        {
            let mut workers = self.workers.write().await;
            workers.insert(worker_id.clone(), handle);
        }

        // Spawn worker task
        let service = Arc::new(self.clone());
        tokio::spawn(async move {
            if let Err(e) = service.run_worker(config, shutdown_rx).await {
                tracing::error!("Worker {} failed: {}", worker_id, e);

                // Update worker status to error
                let mut workers = service.workers.write().await;
                if let Some(worker) = workers.get_mut(&worker_id) {
                    worker.status = WorkerStatus::Error;
                }
            }
        });

        Ok(())
    }

    /// Stop a worker
    pub async fn stop_worker(&self, worker_id: &str) -> Result<()> {
        let mut workers = self.workers.write().await;
        if let Some(worker) = workers.get_mut(worker_id) {
            worker.status = WorkerStatus::Stopping;
            if let Some(tx) = worker.shutdown_tx.take() {
                let _ = tx.send(()).await;
            }
        }
        Ok(())
    }

    /// Get job status and progress
    pub async fn get_job_status(&self, job_id: &Uuid) -> Result<Option<Job>> {
        let mut conn = self.redis_pool.get().await?;
        let key = format!("job:{}", job_id);

        let job_json: Option<String> = conn.get(&key).await?;
        if let Some(json) = job_json {
            let job: Job = serde_json::from_str(&json)?;
            Ok(Some(job))
        } else {
            Ok(None)
        }
    }

    /// Update job progress
    pub async fn update_job_progress(&self, job_id: &Uuid, progress: JobProgress) -> Result<()> {
        let mut conn = self.redis_pool.get().await?;
        let key = format!("job:{}", job_id);

        // Get current job
        let job_json: Option<String> = conn.get(&key).await?;
        if let Some(json) = job_json {
            let mut job: Job = serde_json::from_str(&json)?;
            job.progress = Some(progress);

            let updated_json = serde_json::to_string(&job)?;
            let _: () = conn.set_ex(&key, updated_json, 86400).await?; // 24 hours TTL
        }

        Ok(())
    }

    /// Get jobs by user ID using the user index for efficient lookup
    pub async fn get_user_jobs(
        &self,
        user_id: &Uuid,
        status_filter: Option<JobStatus>,
        limit: Option<usize>,
    ) -> Result<Vec<Job>> {
        let mut conn = self.redis_pool.get().await?;
        let user_index_key = format!("user:{}:jobs", user_id);

        // Get job IDs from user index, sorted by score (created_at) descending
        // We fetch more than limit to account for status filtering
        let fetch_limit = limit.map(|l| l * 3).unwrap_or(1000);
        let job_ids: Vec<String> = redis::cmd("ZREVRANGE")
            .arg(&user_index_key)
            .arg(0)
            .arg(fetch_limit as isize - 1)
            .query_async(&mut *conn)
            .await?;

        let mut jobs = Vec::new();
        let target_count = limit.unwrap_or(usize::MAX);

        for job_id in job_ids {
            if jobs.len() >= target_count {
                break;
            }

            if let Ok(job_uuid) = Uuid::parse_str(&job_id) {
                if let Some(job) = self.get_job_status(&job_uuid).await? {
                    // Apply status filter if provided
                    if let Some(ref status) = status_filter {
                        if job.status == *status {
                            jobs.push(job);
                        }
                    } else {
                        jobs.push(job);
                    }
                }
            }
        }

        Ok(jobs)
    }

    /// Retry a failed job
    pub async fn retry_job(&self, job_id: &Uuid) -> Result<()> {
        let mut conn = self.redis_pool.get().await?;
        let key = format!("job:{}", job_id);

        let job_json: Option<String> = conn.get(&key).await?;
        if let Some(json) = job_json {
            let mut job: Job = serde_json::from_str(&json)?;

            if job.status == JobStatus::Failed && job.retry_count < job.max_retries {
                job.status = JobStatus::Pending;
                job.retry_count += 1;
                job.error_message = None;
                job.started_at = None;
                job.completed_at = None;
                job.scheduled_at = Utc::now();

                self.save_job(&job).await?;
                self.add_to_queue(&job).await?;

                tracing::info!("Retrying job {} (attempt {})", job_id, job.retry_count + 1);
            } else {
                return Err(anyhow!("Job cannot be retried"));
            }
        } else {
            return Err(anyhow!("Job not found"));
        }

        Ok(())
    }

    /// Get worker statistics
    pub async fn get_worker_stats(&self) -> Result<HashMap<String, WorkerStats>> {
        let workers = self.workers.read().await;
        let mut stats = HashMap::new();

        for (worker_id, worker) in workers.iter() {
            let worker_stats = WorkerStats {
                worker_id: worker_id.clone(),
                status: worker.status.clone(),
                jobs_processed: worker.jobs_processed,
                current_job: worker.current_job,
                last_heartbeat: worker.last_heartbeat,
                uptime_seconds: Utc::now()
                    .signed_duration_since(worker.last_heartbeat)
                    .num_seconds() as u64,
            };
            stats.insert(worker_id.clone(), worker_stats);
        }

        Ok(stats)
    }

    /// Clean up completed and old jobs using SCAN for efficient iteration
    pub async fn cleanup_jobs(&self, older_than_hours: u64) -> Result<u64> {
        let mut conn = self.redis_pool.get().await?;
        let pattern = "job:*";

        let cutoff_time = Utc::now() - chrono::Duration::hours(older_than_hours as i64);
        let mut cleaned_count = 0;
        let mut cursor: u64 = 0;

        loop {
            // Use SCAN with COUNT hint for batch size
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(pattern)
                .arg("COUNT")
                .arg(self.scan_config.batch_size)
                .query_async(&mut *conn)
                .await?;

            for key in keys {
                let job_json: Option<String> = conn.get(&key).await?;
                if let Some(json) = job_json {
                    if let Ok(job) = serde_json::from_str::<Job>(&json) {
                        let should_clean = match job.status {
                            JobStatus::Completed => {
                                job.completed_at.map(|t| t < cutoff_time).unwrap_or(false)
                            }
                            JobStatus::Failed => {
                                job.completed_at.map(|t| t < cutoff_time).unwrap_or(false)
                            }
                            JobStatus::DeadLetter => job.created_at < cutoff_time,
                            _ => false,
                        };

                        if should_clean {
                            // Build pipeline for atomic deletion
                            let mut pipe = redis::pipe();
                            pipe.atomic();

                            // Delete the job key
                            pipe.del(&key);

                            // Remove from user index if user_id is present
                            if let Some(user_id) = job.user_id {
                                let user_index_key = format!("user:{}:jobs", user_id);
                                pipe.zrem(&user_index_key, job.id.to_string());
                            }

                            let _: () = pipe.query_async(&mut *conn).await?;
                            cleaned_count += 1;
                        }
                    }
                }
            }

            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }

        tracing::info!("Cleaned up {} old jobs", cleaned_count);
        Ok(cleaned_count)
    }

    // Private methods

    async fn run_worker(
        &self,
        config: WorkerConfig,
        mut shutdown_rx: mpsc::Receiver<()>,
    ) -> Result<()> {
        let worker_id = config.worker_id.clone();
        tracing::info!("Starting worker {}", worker_id);

        // Update worker status
        {
            let mut workers = self.workers.write().await;
            if let Some(worker) = workers.get_mut(&worker_id) {
                worker.status = WorkerStatus::Running;
                worker.last_heartbeat = Utc::now();
            }
        }

        let mut poll_interval = interval(Duration::from_millis(config.poll_interval_ms));
        let mut heartbeat_interval = interval(Duration::from_millis(config.heartbeat_interval_ms));

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    tracing::info!("Worker {} received shutdown signal", worker_id);
                    break;
                }
                _ = poll_interval.tick() => {
                    if let Err(e) = self.process_jobs(&config).await {
                        tracing::error!("Worker {} job processing error: {}", worker_id, e);
                    }
                }
                _ = heartbeat_interval.tick() => {
                    self.update_worker_heartbeat(&worker_id).await?;
                }
            }
        }

        // Update worker status to stopped
        {
            let mut workers = self.workers.write().await;
            if let Some(worker) = workers.get_mut(&worker_id) {
                worker.status = WorkerStatus::Stopped;
            }
        }

        tracing::info!("Worker {} stopped", worker_id);
        Ok(())
    }

    async fn process_jobs(&self, config: &WorkerConfig) -> Result<()> {
        let jobs = self
            .get_pending_jobs(&config.job_types, config.concurrency)
            .await?;

        if jobs.is_empty() {
            // Update worker status to idle
            let mut workers = self.workers.write().await;
            if let Some(worker) = workers.get_mut(&config.worker_id) {
                worker.status = WorkerStatus::Idle;
            }
            return Ok(());
        }

        // Update worker status to busy
        {
            let mut workers = self.workers.write().await;
            if let Some(worker) = workers.get_mut(&config.worker_id) {
                worker.status = WorkerStatus::Busy;
            }
        }

        // Process jobs concurrently
        let mut handles = Vec::new();
        for job in jobs {
            let service = Arc::new(self.clone());
            let config = config.clone();
            let handle = tokio::spawn(async move { service.execute_job(job, &config).await });
            handles.push(handle);
        }

        // Wait for all jobs to complete
        for handle in handles {
            if let Err(e) = handle.await? {
                tracing::error!("Job execution failed: {}", e);
            }
        }

        Ok(())
    }

    async fn execute_job(&self, mut job: Job, config: &WorkerConfig) -> Result<()> {
        let start_time = Instant::now();

        // Update job status to processing
        job.status = JobStatus::Processing;
        job.started_at = Some(Utc::now());
        self.save_job(&job).await?;

        // Update worker current job
        {
            let mut workers = self.workers.write().await;
            if let Some(worker) = workers.get_mut(&config.worker_id) {
                worker.current_job = Some(job.id);
            }
        }

        let result = {
            let handlers = self.job_handlers.read().await;
            if let Some(handler) = handlers.get(&job.job_type) {
                // Execute with timeout
                let execution_future = handler.handle(&job);
                let timeout_duration = std::cmp::min(
                    handler.max_execution_time(),
                    Duration::from_millis(config.max_execution_time_ms),
                );

                match tokio::time::timeout(timeout_duration, execution_future).await {
                    Ok(result) => result,
                    Err(_) => Err(anyhow!(
                        "Job execution timed out after {}ms",
                        timeout_duration.as_millis()
                    )),
                }
            } else {
                Err(anyhow!(
                    "No handler registered for job type: {:?}",
                    job.job_type
                ))
            }
        };

        let execution_time = start_time.elapsed();

        // Update job with result
        match result {
            Ok(_result_data) => {
                job.status = JobStatus::Completed;
                job.completed_at = Some(Utc::now());
                job.error_message = None;

                tracing::info!(
                    "Job {} completed successfully in {}ms",
                    job.id,
                    execution_time.as_millis()
                );
            }
            Err(e) => {
                job.error_message = Some(e.to_string());

                if job.retry_count < job.max_retries {
                    job.status = JobStatus::Retrying;
                    job.retry_count += 1;
                    job.scheduled_at = Utc::now()
                        + chrono::Duration::seconds(
                            (2_u64.pow(job.retry_count.min(5)) * 30) as i64, // Exponential backoff
                        );

                    // Re-queue for retry
                    self.add_to_queue(&job).await?;

                    tracing::warn!(
                        "Job {} failed, scheduling retry {} in {}s: {}",
                        job.id,
                        job.retry_count,
                        (job.scheduled_at - Utc::now()).num_seconds(),
                        e
                    );
                } else {
                    job.status = JobStatus::DeadLetter;
                    job.completed_at = Some(Utc::now());

                    tracing::error!(
                        "Job {} moved to dead letter queue after {} retries: {}",
                        job.id,
                        job.retry_count,
                        e
                    );
                }
            }
        }

        self.save_job(&job).await?;

        // Update worker stats
        {
            let mut workers = self.workers.write().await;
            if let Some(worker) = workers.get_mut(&config.worker_id) {
                worker.jobs_processed += 1;
                worker.current_job = None;
            }
        }

        Ok(())
    }

    async fn get_pending_jobs(&self, job_types: &[JobType], limit: usize) -> Result<Vec<Job>> {
        let mut conn = self.redis_pool.get().await?;
        let mut jobs = Vec::new();

        for job_type in job_types {
            let queue_key = format!("queue:{:?}", job_type);
            let job_ids: Vec<String> = conn.zrange(&queue_key, 0, limit as isize - 1).await?;

            for job_id in job_ids {
                if let Ok(job_uuid) = Uuid::parse_str(&job_id) {
                    if let Some(job) = self.get_job_status(&job_uuid).await? {
                        if job.status == JobStatus::Pending && job.scheduled_at <= Utc::now() {
                            jobs.push(job);
                            // Remove from queue
                            let _: i32 = conn.zrem(&queue_key, &job_id).await?;
                        }
                    }
                }
            }

            if jobs.len() >= limit {
                break;
            }
        }

        // Sort by priority (highest first) then by created_at
        jobs.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });

        jobs.truncate(limit);
        Ok(jobs)
    }

    async fn save_job(&self, job: &Job) -> Result<()> {
        let mut conn = self.redis_pool.get().await?;
        let key = format!("job:{}", job.id);
        let job_json = serde_json::to_string(job)?;

        // Store job for 24 hours
        let _: () = conn.set_ex(&key, job_json, 86400).await?;
        Ok(())
    }

    async fn add_to_queue(&self, job: &Job) -> Result<()> {
        let mut conn = self.redis_pool.get().await?;
        let queue_key = format!("queue:{:?}", job.job_type);

        // Use scheduled_at timestamp as score for priority queue
        let score = job.scheduled_at.timestamp() as f64;

        // Build pipeline for atomic operations
        let mut pipe = redis::pipe();
        pipe.atomic();

        // Add to job type queue
        pipe.zadd(&queue_key, job.id.to_string(), score);

        // Add to user index if user_id is present
        if let Some(user_id) = job.user_id {
            let user_index_key = format!("user:{}:jobs", user_id);
            // Use created_at as score for user index (for descending sort by recency)
            let user_score = job.created_at.timestamp() as f64;
            pipe.zadd(&user_index_key, job.id.to_string(), user_score);
        }

        let _: () = pipe.query_async(&mut *conn).await?;

        Ok(())
    }

    async fn update_worker_heartbeat(&self, worker_id: &str) -> Result<()> {
        let mut workers = self.workers.write().await;
        if let Some(worker) = workers.get_mut(worker_id) {
            worker.last_heartbeat = Utc::now();
        }
        Ok(())
    }

    /// Get the depth (number of pending jobs) for each job type (US-022)
    ///
    /// Returns a HashMap with job type names as keys and pending job counts as values
    pub async fn get_queue_depths(&self) -> Result<HashMap<String, u64>> {
        let mut conn = self.redis_pool.get().await?;
        let mut depths = HashMap::new();

        // All job types to check
        let job_types = [
            JobType::EnforcementExecution,
            JobType::BatchRollback,
            JobType::TokenRefresh,
            JobType::LibraryScan,
            JobType::CommunityListUpdate,
            JobType::HealthCheck,
        ];

        for job_type in job_types {
            let queue_key = format!("queue:{:?}", job_type);
            let count: u64 = conn.zcard(&queue_key).await.unwrap_or(0);
            depths.insert(format!("{:?}", job_type), count);
        }

        Ok(depths)
    }

    /// Get queue depth for a specific job type (US-022)
    pub async fn get_queue_depth(&self, job_type: &JobType) -> Result<u64> {
        let mut conn = self.redis_pool.get().await?;
        let queue_key = format!("queue:{:?}", job_type);
        let count: u64 = conn.zcard(&queue_key).await?;
        Ok(count)
    }
}

/// Worker statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerStats {
    pub worker_id: String,
    pub status: WorkerStatus,
    pub jobs_processed: u64,
    pub current_job: Option<Uuid>,
    pub last_heartbeat: DateTime<Utc>,
    pub uptime_seconds: u64,
}

impl Clone for JobQueueService {
    fn clone(&self) -> Self {
        Self {
            redis_pool: self.redis_pool.clone(),
            rate_limiter: self.rate_limiter.clone(),
            workers: self.workers.clone(),
            job_handlers: self.job_handlers.clone(),
            scan_config: self.scan_config.clone(),
        }
    }
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            worker_id: format!("worker_{}", Uuid::new_v4()),
            concurrency: 2,
            job_types: vec![JobType::EnforcementExecution],
            poll_interval_ms: 1000,
            max_execution_time_ms: 300000, // 5 minutes
            heartbeat_interval_ms: 30000,  // 30 seconds
        }
    }
}

impl Default for JobProgress {
    fn default() -> Self {
        Self {
            current_step: "Starting".to_string(),
            total_steps: 1,
            completed_steps: 0,
            percentage: 0.0,
            estimated_remaining_ms: 0,
            details: serde_json::json!({}),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_job_creation() {
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
    }

    #[tokio::test]
    async fn test_job_progress_calculation() {
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
    }
}
