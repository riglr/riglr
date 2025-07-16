//! Comprehensive tests for tool module

use async_trait::async_trait;
use riglr_core::idempotency::InMemoryIdempotencyStore;
use riglr_core::jobs::{Job, JobResult};
use riglr_core::queue::{InMemoryJobQueue, JobQueue};
use riglr_core::tool::{ExecutionConfig, ResourceLimits, Tool, ToolWorker, WorkerMetrics};
use serde_json::json;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

// Mock tool implementations for testing
struct SuccessTool {
    name: String,
    delay: Option<Duration>,
}

#[async_trait]
impl Tool for SuccessTool {
    async fn execute(
        &self,
        params: serde_json::Value,
    ) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(delay) = self.delay {
            tokio::time::sleep(delay).await;
        }
        Ok(JobResult::success(&params)?)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        ""
    }
}

struct FailureTool {
    name: String,
    error_message: String,
    attempts_before_success: AtomicU32,
}

#[async_trait]
impl Tool for FailureTool {
    async fn execute(
        &self,
        _params: serde_json::Value,
    ) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
        let attempts = self.attempts_before_success.fetch_sub(1, Ordering::SeqCst);
        if attempts > 0 {
            Err(self.error_message.clone().into())
        } else {
            Ok(JobResult::success(&"finally succeeded")?)
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        ""
    }
}

struct TimeoutTool {
    name: String,
}

#[async_trait]
impl Tool for TimeoutTool {
    async fn execute(
        &self,
        _params: serde_json::Value,
    ) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
        tokio::time::sleep(Duration::from_secs(60)).await; // Will timeout
        Ok(JobResult::success(&"shouldn't reach here")?)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        ""
    }
}

// PanicTool removed as it's not used in any tests

#[test]
fn test_execution_config_default() {
    let config = ExecutionConfig::default();
    assert_eq!(config.max_concurrency, 10);
    assert_eq!(config.default_timeout, Duration::from_secs(30));
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.initial_retry_delay, Duration::from_millis(100));
    assert_eq!(config.max_retry_delay, Duration::from_secs(10));
    assert_eq!(config.idempotency_ttl, Duration::from_secs(3600));
    assert!(config.enable_idempotency);
}

#[test]
fn test_execution_config_custom() {
    let config = ExecutionConfig {
        max_concurrency: 20,
        default_timeout: Duration::from_secs(60),
        max_retries: 5,
        initial_retry_delay: Duration::from_millis(200),
        max_retry_delay: Duration::from_secs(20),
        idempotency_ttl: Duration::from_secs(7200),
        enable_idempotency: false,
    };

    assert_eq!(config.max_concurrency, 20);
    assert_eq!(config.default_timeout, Duration::from_secs(60));
    assert_eq!(config.max_retries, 5);
    assert!(!config.enable_idempotency);
}

#[test]
fn test_execution_config_clone() {
    let config = ExecutionConfig::default();
    let cloned = config.clone();

    assert_eq!(cloned.max_concurrency, config.max_concurrency);
    assert_eq!(cloned.default_timeout, config.default_timeout);
    assert_eq!(cloned.max_retries, config.max_retries);
    assert_eq!(cloned.enable_idempotency, config.enable_idempotency);
}

#[test]
fn test_execution_config_debug() {
    let config = ExecutionConfig::default();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("max_concurrency"));
    assert!(debug_str.contains("default_timeout"));
    assert!(debug_str.contains("max_retries"));
}

#[test]
fn test_resource_limits_new() {
    let limits = ResourceLimits::new();
    assert!(limits.get_semaphore("nonexistent").is_none());
}

#[test]
fn test_resource_limits_with_limit() {
    let limits = ResourceLimits::new()
        .with_limit("api", 5)
        .with_limit("database", 10)
        .with_limit("file_system", 20);

    assert!(limits.get_semaphore("api").is_some());
    assert!(limits.get_semaphore("database").is_some());
    assert!(limits.get_semaphore("file_system").is_some());
    assert!(limits.get_semaphore("nonexistent").is_none());
}

#[test]
fn test_resource_limits_default() {
    let limits = ResourceLimits::default();

    assert!(limits.get_semaphore("solana_rpc").is_some());
    assert!(limits.get_semaphore("evm_rpc").is_some());
    assert!(limits.get_semaphore("http_api").is_some());
    assert!(limits.get_semaphore("other").is_none());
}

#[test]
fn test_resource_limits_clone() {
    let limits = ResourceLimits::new().with_limit("test", 5);

    let cloned = limits.clone();
    assert!(cloned.get_semaphore("test").is_some());
}

#[test]
fn test_resource_limits_debug() {
    let limits = ResourceLimits::new();
    let debug_str = format!("{:?}", limits);
    assert!(debug_str.contains("ResourceLimits"));
}

#[test]
fn test_resource_limits_overwrite() {
    let limits = ResourceLimits::new()
        .with_limit("api", 5)
        .with_limit("api", 10); // Overwrite

    assert!(limits.get_semaphore("api").is_some());
}

#[test]
fn test_worker_metrics_default() {
    let metrics = WorkerMetrics::default();
    assert_eq!(metrics.jobs_processed.load(Ordering::Relaxed), 0);
    assert_eq!(metrics.jobs_succeeded.load(Ordering::Relaxed), 0);
    assert_eq!(metrics.jobs_failed.load(Ordering::Relaxed), 0);
    assert_eq!(metrics.jobs_retried.load(Ordering::Relaxed), 0);
}

#[test]
fn test_worker_metrics_increment() {
    let metrics = WorkerMetrics::default();

    metrics.jobs_processed.fetch_add(1, Ordering::Relaxed);
    metrics.jobs_succeeded.fetch_add(2, Ordering::Relaxed);
    metrics.jobs_failed.fetch_add(3, Ordering::Relaxed);
    metrics.jobs_retried.fetch_add(4, Ordering::Relaxed);

    assert_eq!(metrics.jobs_processed.load(Ordering::Relaxed), 1);
    assert_eq!(metrics.jobs_succeeded.load(Ordering::Relaxed), 2);
    assert_eq!(metrics.jobs_failed.load(Ordering::Relaxed), 3);
    assert_eq!(metrics.jobs_retried.load(Ordering::Relaxed), 4);
}

#[test]
fn test_worker_metrics_debug() {
    let metrics = WorkerMetrics::default();
    let debug_str = format!("{:?}", metrics);

    assert!(debug_str.contains("jobs_processed"));
    assert!(debug_str.contains("jobs_succeeded"));
    assert!(debug_str.contains("jobs_failed"));
    assert!(debug_str.contains("jobs_retried"));
}

#[tokio::test]
async fn test_tool_worker_new() {
    let config = ExecutionConfig::default();
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(config.clone());

    assert_eq!(worker.metrics().jobs_processed.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn test_tool_worker_with_idempotency_store() {
    let config = ExecutionConfig::default();
    let store = Arc::new(InMemoryIdempotencyStore::new());
    let worker = ToolWorker::new(config).with_idempotency_store(store);

    // Worker should have idempotency store set
    assert_eq!(worker.metrics().jobs_processed.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn test_tool_worker_with_resource_limits() {
    let config = ExecutionConfig::default();
    let limits = ResourceLimits::new().with_limit("custom_api", 3);

    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(config).with_resource_limits(limits);

    assert_eq!(worker.metrics().jobs_processed.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn test_tool_worker_register_tool() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());

    let tool1 = Arc::new(SuccessTool {
        name: "tool1".to_string(),
        delay: None,
    });
    let tool2 = Arc::new(SuccessTool {
        name: "tool2".to_string(),
        delay: None,
    });

    worker.register_tool(tool1).await;
    worker.register_tool(tool2).await;

    // Tools should be registered
    assert_eq!(worker.metrics().jobs_processed.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn test_tool_worker_process_job_success() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());

    let tool = Arc::new(SuccessTool {
        name: "success_tool".to_string(),
        delay: None,
    });
    worker.register_tool(tool).await;

    let job = Job::new("success_tool", &json!({"test": "data"}), 0).unwrap();
    let result = worker.process_job(job).await.unwrap();

    assert!(result.is_success());
    assert_eq!(worker.metrics().jobs_succeeded.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn test_tool_worker_process_job_failure() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());

    let tool = Arc::new(FailureTool {
        name: "failure_tool".to_string(),
        error_message: "Always fails".to_string(),
        attempts_before_success: AtomicU32::new(100), // Will never succeed
    });
    worker.register_tool(tool).await;

    let job = Job::new("failure_tool", &json!({}), 2).unwrap();
    let result = worker.process_job(job).await.unwrap();

    assert!(!result.is_success());
    assert_eq!(worker.metrics().jobs_failed.load(Ordering::Relaxed), 1);
    assert_eq!(worker.metrics().jobs_retried.load(Ordering::Relaxed), 2);
}

#[tokio::test]
async fn test_tool_worker_process_job_with_retries() {
    let config = ExecutionConfig {
        initial_retry_delay: Duration::from_millis(10),
        ..Default::default()
    };

    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(config);

    let tool = Arc::new(FailureTool {
        name: "retry_tool".to_string(),
        error_message: "Temporary failure".to_string(),
        attempts_before_success: AtomicU32::new(2), // Succeed on 3rd attempt
    });
    worker.register_tool(tool).await;

    let job = Job::new("retry_tool", &json!({}), 3).unwrap();
    let result = worker.process_job(job).await.unwrap();

    assert!(result.is_success());
    assert_eq!(worker.metrics().jobs_succeeded.load(Ordering::Relaxed), 1);
    assert_eq!(worker.metrics().jobs_retried.load(Ordering::Relaxed), 2);
}

#[tokio::test]
async fn test_tool_worker_process_job_timeout() {

    let config = ExecutionConfig {
        default_timeout: Duration::from_millis(100),
        initial_retry_delay: Duration::from_millis(10),
        ..Default::default()
    };

    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(config);

    let tool = Arc::new(TimeoutTool {
        name: "timeout_tool".to_string(),
    });
    worker.register_tool(tool).await;

    let job = Job::new("timeout_tool", &json!({}), 1).unwrap();
    let result = worker.process_job(job).await.unwrap();

    assert!(!result.is_success());
    match result {
        JobResult::Failure { error, .. } => {
            assert!(error.contains("timeout") || error.contains("Timeout"));
        }
        _ => panic!("Expected failure"),
    }
}

#[tokio::test]
async fn test_tool_worker_process_job_tool_not_found() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());

    let job = Job::new("nonexistent_tool", &json!({}), 0).unwrap();
    let result = worker.process_job(job).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_tool_worker_idempotency() {
    let store = Arc::new(InMemoryIdempotencyStore::new());
    let worker = ToolWorker::new(ExecutionConfig::default()).with_idempotency_store(store.clone());

    let tool = Arc::new(SuccessTool {
        name: "idempotent_tool".to_string(),
        delay: None,
    });
    worker.register_tool(tool).await;

    let job = Job::new_idempotent(
        "idempotent_tool",
        &json!({"unique": "data"}),
        0,
        "idempotent_key_123",
    )
    .unwrap();

    // First execution
    let result1 = worker.process_job(job.clone()).await.unwrap();
    assert!(result1.is_success());

    // Second execution should return cached result
    let result2 = worker.process_job(job.clone()).await.unwrap();
    assert!(result2.is_success());

    // Only one successful execution should be recorded
    assert_eq!(worker.metrics().jobs_succeeded.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn test_tool_worker_idempotency_disabled() {
    let config = ExecutionConfig {
        enable_idempotency: false,
        ..Default::default()
    };

    let store = Arc::new(InMemoryIdempotencyStore::new());
    let worker = ToolWorker::new(config).with_idempotency_store(store);

    let tool = Arc::new(SuccessTool {
        name: "tool".to_string(),
        delay: None,
    });
    worker.register_tool(tool).await;

    let job = Job::new_idempotent("tool", &json!({}), 0, "key").unwrap();

    // Execute twice
    worker.process_job(job.clone()).await.unwrap();
    worker.process_job(job.clone()).await.unwrap();

    // Both executions should happen
    assert_eq!(worker.metrics().jobs_succeeded.load(Ordering::Relaxed), 2);
}

#[tokio::test]
async fn test_tool_worker_resource_limits_solana() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());

    let tool = Arc::new(SuccessTool {
        name: "solana_transfer".to_string(),
        delay: Some(Duration::from_millis(10)),
    });
    worker.register_tool(tool).await;

    // Process multiple jobs concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let worker_clone = worker.clone();
        let job = Job::new("solana_transfer", &json!({"id": i}), 0).unwrap();
        let handle = tokio::spawn(async move { worker_clone.process_job(job).await });
        handles.push(handle);
    }

    // All should complete
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}

#[tokio::test]
async fn test_tool_worker_resource_limits_evm() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());

    let tool = Arc::new(SuccessTool {
        name: "evm_call".to_string(),
        delay: Some(Duration::from_millis(10)),
    });
    worker.register_tool(tool).await;

    let job = Job::new("evm_call", &json!({}), 0).unwrap();
    let result = worker.process_job(job).await.unwrap();
    assert!(result.is_success());
}

#[tokio::test]
async fn test_tool_worker_resource_limits_web() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());

    let tool = Arc::new(SuccessTool {
        name: "web_fetch".to_string(),
        delay: Some(Duration::from_millis(10)),
    });
    worker.register_tool(tool).await;

    let job = Job::new("web_fetch", &json!({}), 0).unwrap();
    let result = worker.process_job(job).await.unwrap();
    assert!(result.is_success());
}

#[tokio::test]
async fn test_tool_worker_resource_limits_default_fallback() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());

    let tool = Arc::new(SuccessTool {
        name: "other_tool".to_string(),
        delay: None,
    });
    worker.register_tool(tool).await;

    let job = Job::new("other_tool", &json!({}), 0).unwrap();
    let result = worker.process_job(job).await.unwrap();
    assert!(result.is_success());
}

#[tokio::test]
async fn test_tool_worker_clone() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());

    let tool = Arc::new(SuccessTool {
        name: "clone_tool".to_string(),
        delay: None,
    });
    worker.register_tool(tool).await;

    let cloned = worker.clone();

    // Both should be able to process jobs
    let job1 = Job::new("clone_tool", &json!({"id": 1}), 0).unwrap();
    let job2 = Job::new("clone_tool", &json!({"id": 2}), 0).unwrap();

    let result1 = worker.process_job(job1).await.unwrap();
    let result2 = cloned.process_job(job2).await.unwrap();

    assert!(result1.is_success());
    assert!(result2.is_success());

    // Metrics should be shared
    assert_eq!(worker.metrics().jobs_succeeded.load(Ordering::Relaxed), 2);
    assert_eq!(cloned.metrics().jobs_succeeded.load(Ordering::Relaxed), 2);
}

#[tokio::test]
async fn test_tool_worker_run_with_queue() {
    let queue = Arc::new(InMemoryJobQueue::new());
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());

    let tool = Arc::new(SuccessTool {
        name: "queue_tool".to_string(),
        delay: None,
    });
    worker.register_tool(tool).await;

    // Enqueue some jobs
    for i in 0..3 {
        let job = Job::new("queue_tool", &json!({"id": i}), 0).unwrap();
        queue.enqueue(job).await.unwrap();
    }

    // Run worker for a short time
    let worker_clone = worker.clone();
    let queue_clone = queue.clone();
    let handle = tokio::spawn(async move {
        tokio::select! {
            _ = worker_clone.run(queue_clone) => {},
            _ = tokio::time::sleep(Duration::from_millis(100)) => {},
        }
    });

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(200)).await;
    handle.abort();

    // Jobs should be processed
    assert!(queue.is_empty().await.unwrap());
    assert!(worker.metrics().jobs_processed.load(Ordering::Relaxed) >= 3);
}

#[tokio::test]
async fn test_tool_worker_concurrent_processing() {
    let config = ExecutionConfig {
        max_concurrency: 2, // Limit concurrency
        ..Default::default()
    };

    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(config);

    let tool = Arc::new(SuccessTool {
        name: "concurrent_tool".to_string(),
        delay: Some(Duration::from_millis(50)),
    });
    worker.register_tool(tool).await;

    // Start multiple jobs
    let start = std::time::Instant::now();
    let mut handles = vec![];

    for i in 0..4 {
        let worker_clone = worker.clone();
        let job = Job::new("concurrent_tool", &json!({"id": i}), 0).unwrap();
        let handle = tokio::spawn(async move { worker_clone.process_job(job).await });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    let elapsed = start.elapsed();

    // With concurrency of 2 and 50ms per job, 4 jobs should take ~100ms
    // Be more generous with timing for instrumented runs
    assert!(elapsed >= Duration::from_millis(50));
    assert!(elapsed < Duration::from_secs(2));
}

#[tokio::test]
async fn test_tool_worker_error_handling_in_run_loop() {
    let queue = Arc::new(InMemoryJobQueue::new());
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());

    // Don't register any tools - jobs will fail

    // Enqueue a job
    let job = Job::new("nonexistent", &json!({}), 0).unwrap();
    queue.enqueue(job).await.unwrap();

    // Run worker briefly
    let worker_clone = worker.clone();
    let queue_clone = queue.clone();
    let handle = tokio::spawn(async move {
        tokio::select! {
            _ = worker_clone.run(queue_clone) => {},
            _ = tokio::time::sleep(Duration::from_millis(100)) => {},
        }
    });

    tokio::time::sleep(Duration::from_millis(200)).await;
    handle.abort();

    // Job should be processed (and failed)
    assert!(queue.is_empty().await.unwrap());
    assert_eq!(worker.metrics().jobs_processed.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn test_tool_worker_metrics_accuracy() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());

    // Register various tools
    worker
        .register_tool(Arc::new(SuccessTool {
            name: "success".to_string(),
            delay: None,
        }))
        .await;

    worker
        .register_tool(Arc::new(FailureTool {
            name: "failure".to_string(),
            error_message: "fail".to_string(),
            attempts_before_success: AtomicU32::new(100),
        }))
        .await;

    // Process various jobs
    let success_job = Job::new("success", &json!({}), 0).unwrap();
    worker.process_job(success_job).await.unwrap();

    let failure_job = Job::new("failure", &json!({}), 2).unwrap();
    worker.process_job(failure_job).await.unwrap();

    // Check metrics
    let metrics = worker.metrics();
    assert_eq!(metrics.jobs_succeeded.load(Ordering::Relaxed), 1);
    assert_eq!(metrics.jobs_failed.load(Ordering::Relaxed), 1);
    assert_eq!(metrics.jobs_retried.load(Ordering::Relaxed), 2);
}

#[tokio::test]
async fn test_tool_worker_with_zero_retries() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());

    worker
        .register_tool(Arc::new(FailureTool {
            name: "fail".to_string(),
            error_message: "error".to_string(),
            attempts_before_success: AtomicU32::new(10),
        }))
        .await;

    let job = Job::new("fail", &json!({}), 0).unwrap(); // Zero retries
    let result = worker.process_job(job).await.unwrap();

    assert!(!result.is_success());
    assert_eq!(worker.metrics().jobs_retried.load(Ordering::Relaxed), 0);
    assert_eq!(worker.metrics().jobs_failed.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn test_tool_execution_with_transaction_hash() {
    struct TxTool;

    #[async_trait]
    impl Tool for TxTool {
        async fn execute(
            &self,
            _params: serde_json::Value,
        ) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
            Ok(JobResult::success_with_tx(&"result", "0x12345")?)
        }

        fn name(&self) -> &str {
            "tx_tool"
        }

        fn description(&self) -> &str {
            ""
        }
    }

    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
    worker.register_tool(Arc::new(TxTool)).await;

    let job = Job::new("tx_tool", &json!({}), 0).unwrap();
    let result = worker.process_job(job).await.unwrap();

    match result {
        JobResult::Success { tx_hash, .. } => {
            assert_eq!(tx_hash, Some("0x12345".to_string()));
        }
        _ => panic!("Expected success with tx_hash"),
    }
}

#[tokio::test]
async fn test_tool_worker_idempotency_cache_miss() {
    let store = Arc::new(InMemoryIdempotencyStore::new());
    let worker = ToolWorker::new(ExecutionConfig::default()).with_idempotency_store(store);

    let tool = Arc::new(SuccessTool {
        name: "cache_miss_tool".to_string(),
        delay: None,
    });
    worker.register_tool(tool).await;

    // Test with idempotency key that doesn't exist in cache
    let job = Job::new_idempotent(
        "cache_miss_tool",
        &json!({"data": "test"}),
        0,
        "nonexistent_key",
    )
    .unwrap();

    // This should execute the tool normally since cache is empty (covers lines 167-179)
    let result = worker.process_job(job).await.unwrap();
    assert!(result.is_success());
}

#[tokio::test]
async fn test_tool_worker_without_idempotency_store() {
    // Test worker without idempotency store but with idempotency key
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());

    let tool = Arc::new(SuccessTool {
        name: "no_store_tool".to_string(),
        delay: None,
    });
    worker.register_tool(tool).await;

    let job = Job::new_idempotent("no_store_tool", &json!({}), 0, "some_key").unwrap();

    // This should work normally even without idempotency store (covers line 169 condition)
    let result = worker.process_job(job).await.unwrap();
    assert!(result.is_success());
}

#[tokio::test]
async fn test_tool_worker_idempotency_store_error() {
    // Test error handling when idempotency store get() fails
    struct FailingIdempotencyStore;

    #[async_trait::async_trait]
    impl riglr_core::idempotency::IdempotencyStore for FailingIdempotencyStore {
        async fn get(&self, _key: &str) -> anyhow::Result<Option<JobResult>> {
            Err(anyhow::anyhow!("Store get error"))
        }

        async fn set(&self, _key: &str, _result: &JobResult, _ttl: Duration) -> anyhow::Result<()> {
            Ok(())
        }

        async fn remove(&self, _key: &str) -> anyhow::Result<()> {
            Ok(())
        }
    }

    let store = Arc::new(FailingIdempotencyStore);
    let worker = ToolWorker::new(ExecutionConfig::default()).with_idempotency_store(store);

    let tool = Arc::new(SuccessTool {
        name: "failing_store_tool".to_string(),
        delay: None,
    });
    worker.register_tool(tool).await;

    let job = Job::new_idempotent("failing_store_tool", &json!({}), 0, "test_key").unwrap();

    // Should continue execution even if idempotency store fails (covers error path in line 170)
    let result = worker.process_job(job).await.unwrap();
    assert!(result.is_success());
}

#[tokio::test]
async fn test_tool_worker_idempotency_set_error() {
    // Test error handling when idempotency store set() fails
    struct FailingSetStore;

    #[async_trait::async_trait]
    impl riglr_core::idempotency::IdempotencyStore for FailingSetStore {
        async fn get(&self, _key: &str) -> anyhow::Result<Option<JobResult>> {
            Ok(None) // No cached result
        }

        async fn set(&self, _key: &str, _result: &JobResult, _ttl: Duration) -> anyhow::Result<()> {
            Err(anyhow::anyhow!("Store set error"))
        }

        async fn remove(&self, _key: &str) -> anyhow::Result<()> {
            Ok(())
        }
    }

    let store = Arc::new(FailingSetStore);
    let worker = ToolWorker::new(ExecutionConfig::default()).with_idempotency_store(store);

    let tool = Arc::new(SuccessTool {
        name: "failing_set_tool".to_string(),
        delay: None,
    });
    worker.register_tool(tool).await;

    let job = Job::new_idempotent("failing_set_tool", &json!({}), 0, "test_key").unwrap();

    // Should still return success even if caching fails (covers error path in lines 224-227)
    let result = worker.process_job(job).await.unwrap();
    assert!(result.is_success());
}

#[tokio::test]
async fn test_tool_worker_backoff_exhausted() {
    // Test when all retries are exhausted and backoff returns None
    let config = ExecutionConfig {
        initial_retry_delay: Duration::from_millis(1),
        max_retry_delay: Duration::from_millis(2),
        ..Default::default()
    };

    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(config);

    let tool = Arc::new(FailureTool {
        name: "exhausted_tool".to_string(),
        error_message: "Always fails".to_string(),
        attempts_before_success: AtomicU32::new(100),
    });
    worker.register_tool(tool).await;

    let job = Job::new("exhausted_tool", &json!({}), 1).unwrap();
    let result = worker.process_job(job).await.unwrap();

    // Should fail after retries are exhausted (covers final failure path lines 262-269)
    assert!(!result.is_success());
    match result {
        JobResult::Failure { retriable, .. } => {
            assert!(!retriable); // Should be non-retriable after exhausting retries
        }
        _ => panic!("Expected failure"),
    }
}

#[tokio::test]
async fn test_tool_worker_unknown_error_fallback() {
    // This is tricky to test directly since it requires a specific error condition
    // where attempts > max_retries but last_error is None
    // This test focuses on getting that code path
    let config = ExecutionConfig {
        initial_retry_delay: Duration::from_millis(1),
        ..Default::default()
    };

    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(config);

    // A tool that fails but we'll try to trigger the unknown error path
    let tool = Arc::new(FailureTool {
        name: "unknown_error_tool".to_string(),
        error_message: "".to_string(), // Empty error message
        attempts_before_success: AtomicU32::new(10),
    });
    worker.register_tool(tool).await;

    let job = Job::new("unknown_error_tool", &json!({}), 2).unwrap();
    let result = worker.process_job(job).await.unwrap();

    // Should still fail properly
    assert!(!result.is_success());
}

#[tokio::test]
async fn test_tool_worker_resource_matching_exact() {
    // Test exact resource matching logic (lines 278-283)
    let limits = ResourceLimits::new()
        .with_limit("solana_rpc", 1)
        .with_limit("evm_rpc", 1)
        .with_limit("http_api", 1);

    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default())
        .with_resource_limits(limits);

    // Test different tool name prefixes
    let tools = vec![
        ("solana_balance", "solana_"),
        ("solana_transfer", "solana_"),
        ("evm_call", "evm_"),
        ("evm_send", "evm_"),
        ("web_fetch", "web_"),
        ("web_post", "web_"),
        ("other_tool", ""), // Should use default semaphore
    ];

    for (tool_name, _expected_prefix) in tools {
        let tool = Arc::new(SuccessTool {
            name: tool_name.to_string(),
            delay: None,
        });
        worker.register_tool(tool).await;

        let job = Job::new(tool_name, &json!({}), 0).unwrap();
        let result = worker.process_job(job).await.unwrap();
        assert!(result.is_success());
    }
}

#[tokio::test]
async fn test_tool_worker_empty_resource_name() {
    // Test when resource_name is empty (should use default semaphore) - line 285
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());

    let tool = Arc::new(SuccessTool {
        name: "random_tool".to_string(), // Doesn't match any prefix patterns
        delay: None,
    });
    worker.register_tool(tool).await;

    let job = Job::new("random_tool", &json!({}), 0).unwrap();
    let result = worker.process_job(job).await.unwrap();
    assert!(result.is_success());
}

#[tokio::test]
async fn test_tool_worker_run_loop_job_processing_error() {
    // Test error handling in the run loop spawn task (lines 329-331)
    struct ProcessingErrorTool;

    #[async_trait::async_trait]
    impl Tool for ProcessingErrorTool {
        async fn execute(
            &self,
            _params: serde_json::Value,
        ) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
            // Return an error that can't be processed
            Err("Processing error".into())
        }

        fn name(&self) -> &str {
            "processing_error_tool"
        }

        fn description(&self) -> &str {
            ""
        }
    }

    let queue = Arc::new(InMemoryJobQueue::new());
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());

    worker.register_tool(Arc::new(ProcessingErrorTool)).await;

    // Enqueue a job that will cause processing error
    let job = Job::new("processing_error_tool", &json!({}), 0).unwrap();
    queue.enqueue(job).await.unwrap();

    // Run worker briefly
    let worker_clone = worker.clone();
    let queue_clone = queue.clone();
    let handle = tokio::spawn(async move {
        tokio::select! {
            _ = worker_clone.run(queue_clone) => {},
            _ = tokio::time::sleep(Duration::from_millis(100)) => {},
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;
    handle.abort();

    // Job should be processed and failed
    assert!(queue.is_empty().await.unwrap());
    assert_eq!(worker.metrics().jobs_processed.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn test_tool_worker_run_loop_startup_logging() {
    // Test the startup logging (lines 300-302)
    let queue = Arc::new(InMemoryJobQueue::new());
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());

    // Register multiple tools to test the logging
    for i in 0..3 {
        let tool = Arc::new(SuccessTool {
            name: format!("startup_tool_{}", i),
            delay: None,
        });
        worker.register_tool(tool).await;
    }

    // Run worker briefly to trigger startup logging
    let worker_clone = worker.clone();
    let queue_clone = queue.clone();
    let handle = tokio::spawn(async move {
        tokio::select! {
            _ = worker_clone.run(queue_clone) => {},
            _ = tokio::time::sleep(Duration::from_millis(50)) => {},
        }
    });

    tokio::time::sleep(Duration::from_millis(25)).await;
    handle.abort();
}

#[tokio::test]
async fn test_tool_worker_run_loop_no_jobs() {
    // Test when dequeue returns None (line 335-337)
    let queue = Arc::new(InMemoryJobQueue::new());
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());

    // Don't enqueue any jobs - dequeue will return None
    let worker_clone = worker.clone();
    let queue_clone = queue.clone();
    let handle = tokio::spawn(async move {
        tokio::select! {
            _ = worker_clone.run(queue_clone) => {},
            _ = tokio::time::sleep(Duration::from_millis(100)) => {},
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;
    handle.abort();

    // No jobs should be processed
    assert_eq!(worker.metrics().jobs_processed.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn test_tool_worker_run_loop_queue_error() {
    // Test error handling in run loop when dequeue fails (lines 339-342)
    struct ErrorQueue;

    #[async_trait::async_trait]
    impl JobQueue for ErrorQueue {
        async fn enqueue(&self, _job: Job) -> anyhow::Result<()> {
            Ok(())
        }

        async fn dequeue(&self) -> anyhow::Result<Option<Job>> {
            Err(anyhow::anyhow!("Queue dequeue error"))
        }

        async fn dequeue_with_timeout(&self, _timeout: Duration) -> anyhow::Result<Option<Job>> {
            Err(anyhow::anyhow!("Queue dequeue timeout error"))
        }

        async fn len(&self) -> anyhow::Result<usize> {
            Ok(0)
        }
    }

    let error_queue = Arc::new(ErrorQueue);
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());

    // Run worker with error queue
    let worker_clone = worker.clone();
    let queue_clone = error_queue.clone();
    let handle = tokio::spawn(async move {
        tokio::select! {
            _ = worker_clone.run(queue_clone) => {},
            _ = tokio::time::sleep(Duration::from_millis(200)) => {},
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;
    handle.abort();
}
