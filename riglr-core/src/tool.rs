//! Tool execution and worker infrastructure for riglr.
//!
//! This module provides the core abstractions for executing tools in a resilient,
//! asynchronous manner with support for retries, timeouts, and job queuing.

use async_trait::async_trait;
use backoff::{backoff::Backoff, ExponentialBackoffBuilder};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{OwnedSemaphorePermit, RwLock, Semaphore};
use tracing::{debug, error, info, warn};

use crate::idempotency::IdempotencyStore;
use crate::jobs::{Job, JobResult};
use crate::queue::JobQueue;

/// A trait defining the execution interface for tools.
///
/// This is compatible with `rig::Tool` and provides the foundation
/// for executing tools within the riglr ecosystem.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Execute the tool with the given parameters.
    ///
    /// Returns a `JobResult` indicating success or failure.
    async fn execute(
        &self,
        params: serde_json::Value,
    ) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>>;

    /// Get the name of this tool.
    fn name(&self) -> &str;
}

/// Configuration for tool execution behavior.
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Maximum number of concurrent executions per resource type
    pub max_concurrency: usize,
    /// Default timeout for tool execution
    pub default_timeout: Duration,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial retry delay for exponential backoff
    pub initial_retry_delay: Duration,
    /// Maximum retry delay for exponential backoff
    pub max_retry_delay: Duration,
    /// TTL for idempotency cache entries
    pub idempotency_ttl: Duration,
    /// Whether to enable idempotency checking
    pub enable_idempotency: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 10,
            default_timeout: Duration::from_secs(30),
            max_retries: 3,
            initial_retry_delay: Duration::from_millis(100),
            max_retry_delay: Duration::from_secs(10),
            idempotency_ttl: Duration::from_secs(3600), // 1 hour
            enable_idempotency: true,
        }
    }
}

/// Resource limits configuration
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Resource name to semaphore mapping
    semaphores: Arc<HashMap<String, Arc<Semaphore>>>,
}

impl ResourceLimits {
    /// Create new resource limits
    pub fn new() -> Self {
        Self {
            semaphores: Arc::new(HashMap::new()),
        }
    }

    /// Add a resource limit
    pub fn with_limit(mut self, resource: impl Into<String>, limit: usize) -> Self {
        let semaphores = Arc::make_mut(&mut self.semaphores);
        semaphores.insert(resource.into(), Arc::new(Semaphore::new(limit)));
        self
    }

    /// Get semaphore for a resource
    pub fn get_semaphore(&self, resource: &str) -> Option<Arc<Semaphore>> {
        self.semaphores.get(resource).cloned()
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self::new()
            .with_limit("solana_rpc", 5)
            .with_limit("evm_rpc", 10)
            .with_limit("http_api", 20)
    }
}

/// A worker that processes jobs from a queue using registered tools.
pub struct ToolWorker<I: IdempotencyStore + 'static> {
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
    default_semaphore: Arc<Semaphore>,
    resource_limits: ResourceLimits,
    config: ExecutionConfig,
    idempotency_store: Option<Arc<I>>,
    metrics: Arc<WorkerMetrics>,
}

/// Metrics for worker performance
#[derive(Debug, Default)]
pub struct WorkerMetrics {
    pub jobs_processed: std::sync::atomic::AtomicU64,
    pub jobs_succeeded: std::sync::atomic::AtomicU64,
    pub jobs_failed: std::sync::atomic::AtomicU64,
    pub jobs_retried: std::sync::atomic::AtomicU64,
}

impl<I: IdempotencyStore + 'static> ToolWorker<I> {
    /// Create a new tool worker with the given configuration.
    pub fn new(config: ExecutionConfig) -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            default_semaphore: Arc::new(Semaphore::new(config.max_concurrency)),
            resource_limits: ResourceLimits::default(),
            config,
            idempotency_store: None,
            metrics: Arc::new(WorkerMetrics::default()),
        }
    }

    /// Set the idempotency store
    pub fn with_idempotency_store(mut self, store: Arc<I>) -> Self {
        self.idempotency_store = Some(store);
        self
    }

    /// Set custom resource limits
    pub fn with_resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.resource_limits = limits;
        self
    }

    /// Register a tool with this worker.
    pub async fn register_tool(&self, tool: Arc<dyn Tool>) {
        let mut tools = self.tools.write().await;
        tools.insert(tool.name().to_string(), tool);
    }

    /// Get metrics
    pub fn metrics(&self) -> &WorkerMetrics {
        &self.metrics
    }

    /// Process a single job with all resilience features.
    pub async fn process_job(
        &self,
        mut job: Job,
    ) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
        // Check idempotency first
        if let Some(ref idempotency_key) = job.idempotency_key {
            if self.config.enable_idempotency {
                if let Some(ref store) = self.idempotency_store {
                    if let Ok(Some(cached_result)) = store.get(idempotency_key).await {
                        info!(
                            "Returning cached result for idempotency key: {}",
                            idempotency_key
                        );
                        return Ok(cached_result);
                    }
                }
            }
        }

        // Acquire appropriate semaphore
        let _permit = self.acquire_semaphore(&job.tool_name).await?;

        let tools = self.tools.read().await;
        let tool = tools
            .get(&job.tool_name)
            .ok_or_else(|| format!("Tool '{}' not found", job.tool_name))?
            .clone();
        drop(tools); // Release read lock early

        // Set up exponential backoff
        let backoff = ExponentialBackoffBuilder::new()
            .with_initial_interval(self.config.initial_retry_delay)
            .with_max_interval(self.config.max_retry_delay)
            .with_max_elapsed_time(Some(Duration::from_secs(300)))
            .build();

        let mut last_error = None;
        let mut attempts = 0;

        // Retry loop with exponential backoff
        while attempts <= job.max_retries {
            attempts += 1;
            debug!(
                "Attempting job {} (attempt {}/{})",
                job.job_id,
                attempts,
                job.max_retries + 1
            );

            // Execute with timeout
            let result = tokio::time::timeout(
                self.config.default_timeout,
                tool.execute(job.params.clone()),
            )
            .await;

            match result {
                Ok(Ok(job_result)) => {
                    // Success - cache if idempotent
                    if let Some(ref idempotency_key) = job.idempotency_key {
                        if self.config.enable_idempotency {
                            if let Some(ref store) = self.idempotency_store {
                                let _ = store
                                    .set(idempotency_key, &job_result, self.config.idempotency_ttl)
                                    .await;
                            }
                        }
                    }

                    self.metrics
                        .jobs_succeeded
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    return Ok(job_result);
                }
                Ok(Err(e)) => {
                    last_error = Some(e.to_string());
                    warn!("Job {} failed: {}", job.job_id, e);
                }
                Err(_) => {
                    last_error = Some("Tool execution timeout".to_string());
                    warn!("Job {} timed out", job.job_id);
                }
            }

            // Check if we should retry
            if attempts <= job.max_retries {
                job.increment_retry();
                self.metrics
                    .jobs_retried
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                // Wait with exponential backoff
                let mut backoff = backoff.clone();
                if let Some(delay) = backoff.next_backoff() {
                    info!("Retrying job {} after {:?}", job.job_id, delay);
                    tokio::time::sleep(delay).await;
                }
            }
        }

        // All retries exhausted
        self.metrics
            .jobs_failed
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(JobResult::Failure {
            error: last_error.unwrap_or_else(|| "Unknown error".to_string()),
            retriable: false,
        })
    }

    /// Acquire the appropriate semaphore for a tool
    async fn acquire_semaphore(
        &self,
        tool_name: &str,
    ) -> Result<OwnedSemaphorePermit, Box<dyn std::error::Error + Send + Sync>> {
        // Check if there's a specific resource limit for this tool
        let resource_name = match tool_name {
            name if name.starts_with("solana_") => "solana_rpc",
            name if name.starts_with("evm_") => "evm_rpc",
            name if name.starts_with("web_") => "http_api",
            _ => "",
        };

        if !resource_name.is_empty() {
            if let Some(semaphore) = self.resource_limits.get_semaphore(resource_name) {
                return Ok(semaphore.acquire_owned().await?);
            }
        }

        // Fall back to default semaphore
        Ok(self.default_semaphore.clone().acquire_owned().await?)
    }

    /// Start the worker loop, processing jobs from the given queue.
    pub async fn run<Q: JobQueue>(
        &self,
        queue: Arc<Q>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Starting ToolWorker with {} tools registered",
            self.tools.read().await.len()
        );

        loop {
            match queue.dequeue_with_timeout(Duration::from_secs(5)).await {
                Ok(Some(job)) => {
                    let job_id = job.job_id;
                    let tool_name = job.tool_name.clone();

                    self.metrics
                        .jobs_processed
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    // Spawn task to process job asynchronously
                    let worker = self.clone();
                    tokio::spawn(async move {
                        match worker.process_job(job).await {
                            Ok(job_result) => {
                                if job_result.is_success() {
                                    info!("Job {} ({}) completed successfully", job_id, tool_name);
                                } else {
                                    warn!(
                                        "Job {} ({}) failed: {:?}",
                                        job_id, tool_name, job_result
                                    );
                                }
                            }
                            Err(e) => {
                                error!("Job {} ({}) processing error: {}", job_id, tool_name, e);
                            }
                        }
                    });
                }
                Ok(None) => {
                    // No jobs available, continue
                    debug!("No jobs available in queue");
                }
                Err(e) => {
                    error!("Failed to dequeue job: {}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }
}

// Implement Clone for ToolWorker to enable spawning tasks
impl<I: IdempotencyStore + 'static> Clone for ToolWorker<I> {
    fn clone(&self) -> Self {
        Self {
            tools: self.tools.clone(),
            default_semaphore: self.default_semaphore.clone(),
            resource_limits: self.resource_limits.clone(),
            config: self.config.clone(),
            idempotency_store: self.idempotency_store.clone(),
            metrics: self.metrics.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::idempotency::InMemoryIdempotencyStore;
    use crate::jobs::Job;
    use uuid::Uuid;

    struct MockTool {
        name: String,
        should_fail: bool,
    }

    #[async_trait]
    impl Tool for MockTool {
        async fn execute(
            &self,
            _params: serde_json::Value,
        ) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
            if self.should_fail {
                Err("Mock failure".into())
            } else {
                Ok(JobResult::Success {
                    value: serde_json::json!({"result": "success"}),
                    tx_hash: None,
                })
            }
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_tool_worker_process_job() {
        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            should_fail: false,
        });
        worker.register_tool(tool).await;

        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "test_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 3,
            retry_count: 0,
        };

        let result = worker.process_job(job).await.unwrap();
        match result {
            JobResult::Success { .. } => (),
            _ => panic!("Expected success"),
        }
    }

    #[tokio::test]
    async fn test_tool_worker_with_idempotency() {
        let store = Arc::new(InMemoryIdempotencyStore::new());
        let worker =
            ToolWorker::new(ExecutionConfig::default()).with_idempotency_store(store.clone());

        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            should_fail: false,
        });
        worker.register_tool(tool).await;

        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "test_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: Some("test_key".to_string()),
            max_retries: 3,
            retry_count: 0,
        };

        // First execution
        let result1 = worker.process_job(job.clone()).await.unwrap();
        assert!(result1.is_success());

        // Second execution should return cached result
        let result2 = worker.process_job(job).await.unwrap();
        assert!(result2.is_success());
    }

    #[tokio::test]
    async fn test_tool_worker_with_retries() {
        let mut config = ExecutionConfig::default();
        config.initial_retry_delay = Duration::from_millis(10);

        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(config);
        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            should_fail: true,
        });
        worker.register_tool(tool).await;

        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "test_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 2,
            retry_count: 0,
        };

        let result = worker.process_job(job).await.unwrap();
        match result {
            JobResult::Failure { retriable, .. } => {
                assert!(!retriable); // Should not be retriable after exhausting retries
            }
            _ => panic!("Expected failure"),
        }
    }
}
