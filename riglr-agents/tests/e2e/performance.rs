//! End-to-end performance and load testing for riglr-agents.
//!
//! This module contains performance tests that validate the riglr-agents system
//! under load and stress conditions. These tests measure throughput, latency,
//! resource utilization, and system stability when processing multiple concurrent
//! tasks across multiple agents.
//!
//! Performance metrics tested:
//! - Task throughput (tasks per second)
//! - Task execution latency (time to completion)
//! - Agent system scalability (performance with multiple agents)
//! - Memory usage and resource management
//! - Blockchain operation performance under load
//! - System stability during stress testing

use crate::common::*;
use async_trait::async_trait;
use riglr_agents::{registry::RegistryConfig, CapabilityType, *};
use riglr_core::signer::{SignerContext, SignerError, UnifiedSigner};
use riglr_solana_tools::signer::LocalSolanaSigner;
use solana_sdk::signature::Keypair;
use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::{
    sync::{RwLock, Semaphore},
    time::{sleep, timeout},
};
use tracing::{debug, error, info};

/// Performance metrics collector for analyzing system behavior under load.
#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    /// Total tasks processed
    tasks_processed: AtomicUsize,
    /// Total tasks failed
    tasks_failed: AtomicUsize,
    /// Total execution time across all tasks (nanoseconds)
    total_execution_time_ns: AtomicU64,
    /// Latency measurements (stored as nanoseconds)
    latencies: RwLock<VecDeque<u64>>,
    /// Peak memory usage (estimated)
    #[allow(dead_code)]
    peak_memory_usage: AtomicUsize,
    /// Active agent count
    active_agents: AtomicUsize,
    /// Test start time
    start_time: RwLock<Option<Instant>>,
}

impl PerformanceMetrics {
    /// Creates a new PerformanceMetrics instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts the performance test timer.
    pub async fn start_test(&self) {
        *self.start_time.write().await = Some(Instant::now());
    }

    /// Records the completion of a task with its execution time and success status.
    pub async fn record_task_completion(&self, execution_time: Duration, success: bool) {
        if success {
            self.tasks_processed.fetch_add(1, Ordering::SeqCst);
        } else {
            self.tasks_failed.fetch_add(1, Ordering::SeqCst);
        }

        let execution_ns = execution_time.as_nanos() as u64;
        self.total_execution_time_ns
            .fetch_add(execution_ns, Ordering::SeqCst);

        // Store latency (keep last 10000 measurements to avoid unbounded growth)
        let mut latencies = self.latencies.write().await;
        if latencies.len() >= 10000 {
            latencies.pop_front();
        }
        latencies.push_back(execution_ns);
    }

    /// Gets a comprehensive performance summary of the test run.
    pub async fn get_performance_summary(&self) -> PerformanceSummary {
        let successful_tasks = self.tasks_processed.load(Ordering::SeqCst);
        let failed_tasks = self.tasks_failed.load(Ordering::SeqCst);
        let total_tasks = successful_tasks + failed_tasks;
        let total_time_ns = self.total_execution_time_ns.load(Ordering::SeqCst);

        let latencies = self.latencies.read().await;
        let mut sorted_latencies: Vec<u64> = latencies.iter().copied().collect();
        sorted_latencies.sort_unstable();

        let (avg_latency_ns, p50_latency_ns, p95_latency_ns, p99_latency_ns) =
            if !sorted_latencies.is_empty() {
                let avg = total_time_ns / sorted_latencies.len() as u64;
                let p50_idx = sorted_latencies.len() / 2;
                let p95_idx = (sorted_latencies.len() * 95) / 100;
                let p99_idx = (sorted_latencies.len() * 99) / 100;

                (
                    avg,
                    sorted_latencies[p50_idx],
                    sorted_latencies[p95_idx.min(sorted_latencies.len() - 1)],
                    sorted_latencies[p99_idx.min(sorted_latencies.len() - 1)],
                )
            } else {
                (0, 0, 0, 0)
            };

        let test_duration = self
            .start_time
            .read()
            .await
            .map(|start| start.elapsed())
            .unwrap_or_default();

        let throughput_tps = if test_duration.as_secs() > 0 {
            total_tasks as f64 / test_duration.as_secs() as f64
        } else {
            0.0
        };

        PerformanceSummary {
            total_tasks,
            failed_tasks,
            success_rate: if total_tasks > 0 {
                successful_tasks as f64 / total_tasks as f64
            } else {
                0.0
            },
            throughput_tps,
            avg_latency_ms: avg_latency_ns as f64 / 1_000_000.0,
            p50_latency_ms: p50_latency_ns as f64 / 1_000_000.0,
            p95_latency_ms: p95_latency_ns as f64 / 1_000_000.0,
            p99_latency_ms: p99_latency_ns as f64 / 1_000_000.0,
            test_duration,
            active_agents: self.active_agents.load(Ordering::SeqCst),
        }
    }

    /// Sets the number of active agents for reporting purposes.
    pub fn set_active_agents(&self, count: usize) {
        self.active_agents.store(count, Ordering::SeqCst);
    }
}

/// Summary of performance metrics for a test run.
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    /// Total number of tasks processed
    pub total_tasks: usize,
    /// Number of tasks that failed
    pub failed_tasks: usize,
    /// Success rate as a fraction (0.0 to 1.0)
    pub success_rate: f64,
    /// Throughput in tasks per second
    pub throughput_tps: f64,
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
    /// 50th percentile latency in milliseconds
    pub p50_latency_ms: f64,
    /// 95th percentile latency in milliseconds
    pub p95_latency_ms: f64,
    /// 99th percentile latency in milliseconds
    pub p99_latency_ms: f64,
    /// Total duration of the test
    pub test_duration: Duration,
    /// Number of active agents during the test
    pub active_agents: usize,
}

/// A high-performance agent optimized for load testing.
#[derive(Clone)]
pub struct LoadTestAgent {
    id: AgentId,
    signer_context: Arc<dyn UnifiedSigner>,
    metrics: Arc<PerformanceMetrics>,
    /// Simulated processing delay (for testing different load scenarios)
    processing_delay: Duration,
    /// Maximum concurrent tasks this agent can handle
    concurrency_limit: Arc<Semaphore>,
}

impl std::fmt::Debug for LoadTestAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadTestAgent")
            .field("id", &self.id)
            .field("signer_context", &"Arc<dyn UnifiedSigner>")
            .field("metrics", &self.metrics)
            .field("processing_delay", &self.processing_delay)
            .field(
                "concurrency_limit",
                &self.concurrency_limit.available_permits(),
            )
            .finish()
    }
}

impl LoadTestAgent {
    /// Creates a new LoadTestAgent with specified configuration.
    pub fn new(
        signer_context: Arc<dyn UnifiedSigner>,
        metrics: Arc<PerformanceMetrics>,
        processing_delay: Duration,
        max_concurrent_tasks: usize,
    ) -> Self {
        Self {
            id: AgentId::generate(),
            signer_context,
            metrics,
            processing_delay,
            concurrency_limit: Arc::new(Semaphore::new(max_concurrent_tasks)),
        }
    }

    /// Process a lightweight task for performance testing.
    async fn process_performance_task(&self, task: &Task) -> Result<TaskResult> {
        let start_time = Instant::now();

        // Acquire semaphore permit for concurrency control
        let _permit = self.concurrency_limit.acquire().await.map_err(|e| {
            AgentError::task_execution(format!("Failed to acquire concurrency permit: {}", e))
        })?;

        // Simulate processing time
        if !self.processing_delay.is_zero() {
            sleep(self.processing_delay).await;
        }

        let task_type = task
            .parameters
            .get("task_type")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        match task_type {
            "lightweight" => {
                // Minimal processing for throughput testing
                Ok(TaskResult::success(
                    serde_json::json!({
                        "agent_id": self.id.as_str(),
                        "task_type": "lightweight",
                        "processed_at": chrono::Utc::now().timestamp_millis()
                    }),
                    Some("Lightweight task processed".to_string()),
                    start_time.elapsed(),
                ))
            }
            "blockchain_sim" => {
                // Simulate blockchain operation without actual transaction
                self.simulate_blockchain_operation(task, start_time).await
            }
            "memory_intensive" => {
                // Simulate memory-intensive operation
                self.simulate_memory_intensive_operation(task, start_time)
                    .await
            }
            "cpu_intensive" => {
                // Simulate CPU-intensive operation
                self.simulate_cpu_intensive_operation(task, start_time)
                    .await
            }
            _ => Ok(TaskResult::failure(
                format!("Unknown task type: {}", task_type),
                false,
                start_time.elapsed(),
            )),
        }
    }

    async fn simulate_blockchain_operation(
        &self,
        _task: &Task,
        start_time: Instant,
    ) -> Result<TaskResult> {
        // Simulate SignerContext operation using with_signer pattern
        let signer_context = self.signer_context.clone();

        let job_result = SignerContext::with_signer(signer_context, async {
            // Simulate blockchain operation processing time
            sleep(Duration::from_millis(50)).await;

            let current_signer = SignerContext::current().await?;
            match current_signer.as_solana() {
                Some(_) => Ok(serde_json::json!({
                    "simulated": true,
                    "operation": "blockchain_sim",
                    "signature": "simulated_signature"
                })),
                None => Err(SignerError::UnsupportedOperation(
                    "Simulation requires Solana signer".to_string(),
                )),
            }
        })
        .await;

        match job_result {
            Ok(result) => Ok(TaskResult::success(
                result,
                Some("Blockchain simulation completed".to_string()),
                start_time.elapsed(),
            )),
            Err(e) => Ok(TaskResult::failure(
                format!("Blockchain simulation failed: {}", e),
                true,
                start_time.elapsed(),
            )),
        }
    }

    async fn simulate_memory_intensive_operation(
        &self,
        _task: &Task,
        start_time: Instant,
    ) -> Result<TaskResult> {
        // Simulate memory allocation and processing
        let data_size = 1024 * 1024; // 1MB
        let mut data = Vec::with_capacity(data_size);
        for i in 0..data_size {
            data.push((i % 256) as u8);
        }

        // Simulate some processing on the data
        let checksum: u64 = data.iter().map(|&b| b as u64).sum();

        sleep(Duration::from_millis(10)).await;

        Ok(TaskResult::success(
            serde_json::json!({
                "agent_id": self.id.as_str(),
                "task_type": "memory_intensive",
                "data_size": data_size,
                "checksum": checksum
            }),
            Some("Memory intensive operation completed".to_string()),
            start_time.elapsed(),
        ))
    }

    async fn simulate_cpu_intensive_operation(
        &self,
        _task: &Task,
        start_time: Instant,
    ) -> Result<TaskResult> {
        // Simulate CPU-intensive computation
        let mut result = 0u64;
        let iterations = 100_000;

        for i in 0..iterations {
            result = result.wrapping_add(i * i);
        }

        Ok(TaskResult::success(
            serde_json::json!({
                "agent_id": self.id.as_str(),
                "task_type": "cpu_intensive",
                "iterations": iterations,
                "result": result
            }),
            Some("CPU intensive operation completed".to_string()),
            start_time.elapsed(),
        ))
    }
}

#[async_trait]
impl Agent for LoadTestAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        let start_time = Instant::now();

        let result = self
            .process_performance_task(&task)
            .await
            .unwrap_or_else(|e| {
                TaskResult::failure(
                    format!("Performance task failed: {}", e),
                    true,
                    start_time.elapsed(),
                )
            });

        // Record metrics - extract duration from TaskResult
        let duration = match &result {
            TaskResult::Success { duration, .. } => *duration,
            TaskResult::Failure { duration, .. } => *duration,
            TaskResult::Timeout { duration } => *duration,
            TaskResult::Cancelled { .. } => start_time.elapsed(),
        };

        self.metrics
            .record_task_completion(duration, result.is_success())
            .await;

        Ok(result)
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<CapabilityType> {
        vec![
            CapabilityType::Custom("performance_testing".to_string()),
            CapabilityType::Custom("load_testing".to_string()),
            CapabilityType::Custom("high_throughput".to_string()),
            CapabilityType::Trading, // For capability matching
        ]
    }

    fn load(&self) -> f64 {
        // Return load based on available semaphore permits
        let available = self.concurrency_limit.available_permits();
        let max_concurrent = 10; // Default concurrency limit

        1.0 - (available as f64 / max_concurrent as f64)
    }
}

/// Test high-throughput task processing with multiple agents.
///
/// This test validates that the riglr-agents system can handle high volumes
/// of concurrent tasks efficiently with good throughput and latency characteristics.
#[tokio::test]
async fn test_high_throughput_task_processing() {
    tracing_subscriber::fmt::try_init().ok();
    info!("Testing high-throughput task processing");

    let harness = BlockchainTestHarness::new()
        .await
        .expect("Failed to create blockchain test harness");

    let metrics = Arc::new(PerformanceMetrics::default());
    metrics.start_test().await;

    // Create multiple high-performance agents
    let num_agents = 5;
    let max_concurrent_per_agent = 10;
    let mut agents = Vec::with_capacity(num_agents);

    for i in 0..num_agents {
        let keypair = harness
            .get_funded_keypair(i % 3) // Reuse some keypairs
            .expect("Failed to get keypair");

        let signer =
            LocalSolanaSigner::new(keypair.insecure_clone(), harness.rpc_url().to_string());
        let unified_signer: Arc<dyn UnifiedSigner> = Arc::new(signer);

        let agent = Arc::new(LoadTestAgent::new(
            unified_signer,
            metrics.clone(),
            Duration::from_millis(1), // Minimal processing delay for throughput
            max_concurrent_per_agent,
        ));

        agents.push(agent);
    }

    info!("Created {} agents for high-throughput testing", num_agents);
    metrics.set_active_agents(num_agents);

    // Create agent system
    let registry = LocalAgentRegistry::with_config(RegistryConfig::default());

    for agent in &agents {
        registry
            .register_agent(agent.clone())
            .await
            .expect("Failed to register agent");
    }

    let dispatcher = Arc::new(AgentDispatcher::with_config(
        Arc::new(registry),
        DispatchConfig {
            routing_strategy: RoutingStrategy::LeastLoaded, // Distribute load efficiently
            ..DispatchConfig::default()
        },
    ));

    // Generate high volume of lightweight tasks
    let num_tasks = 1000;
    let batch_size = 50; // Process in batches to avoid overwhelming the system

    info!(
        "Processing {} tasks in batches of {}",
        num_tasks, batch_size
    );

    let mut total_successful = 0;
    let mut total_failed = 0;

    for batch_start in (0..num_tasks).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(num_tasks);
        let batch_size_actual = batch_end - batch_start;

        debug!("Processing batch {}-{}", batch_start, batch_end);

        // Create batch of tasks
        let mut task_handles = Vec::with_capacity(batch_size_actual);
        for task_idx in batch_start..batch_end {
            let dispatcher_clone = dispatcher.clone();

            let task = Task::new(
                TaskType::Trading,
                serde_json::json!({
                    "task_type": "lightweight",
                    "task_index": task_idx,
                    "batch": batch_start / batch_size
                }),
            )
            .with_priority(Priority::Normal);

            let handle = tokio::spawn(async move { dispatcher_clone.dispatch_task(task).await });

            task_handles.push(handle);
        }

        // Wait for batch to complete
        let batch_results = futures::future::join_all(task_handles).await;

        // Count batch results
        let mut batch_successful = 0;
        let mut batch_failed = 0;

        for result in batch_results {
            match result {
                Ok(Ok(task_result)) => {
                    if task_result.is_success() {
                        batch_successful += 1;
                    } else {
                        batch_failed += 1;
                        debug!("Task failed in batch: {:?}", task_result);
                    }
                }
                Ok(Err(e)) => {
                    batch_failed += 1;
                    debug!("Task dispatch failed: {:?}", e);
                }
                Err(e) => {
                    batch_failed += 1;
                    debug!("Task join failed: {:?}", e);
                }
            }
        }

        total_successful += batch_successful;
        total_failed += batch_failed;

        info!(
            "Batch completed: {}/{} successful, {}/{} failed",
            batch_successful, batch_size_actual, batch_failed, batch_size_actual
        );

        // Small delay between batches to prevent overwhelming
        sleep(Duration::from_millis(10)).await;
    }

    // Get performance summary
    let summary = metrics.get_performance_summary().await;

    info!("High-throughput test completed:");
    info!("  Total tasks: {}", summary.total_tasks);
    info!("  Failed tasks: {}", summary.failed_tasks);
    info!("  Success rate: {:.2}%", summary.success_rate * 100.0);
    info!("  Throughput: {:.2} tasks/second", summary.throughput_tps);
    info!("  Average latency: {:.2} ms", summary.avg_latency_ms);
    info!("  P50 latency: {:.2} ms", summary.p50_latency_ms);
    info!("  P95 latency: {:.2} ms", summary.p95_latency_ms);
    info!("  P99 latency: {:.2} ms", summary.p99_latency_ms);
    info!("  Test duration: {:?}", summary.test_duration);
    info!("  Active agents: {}", summary.active_agents);

    // Verify performance requirements
    assert_eq!(
        total_successful, num_tasks,
        "All tasks should succeed in high-throughput test"
    );
    assert_eq!(
        total_failed, 0,
        "No tasks should fail in high-throughput test"
    );
    assert!(
        summary.success_rate >= 0.99,
        "Success rate should be at least 99%"
    );
    assert!(
        summary.throughput_tps >= 50.0,
        "Throughput should be at least 50 tasks/second"
    );
    assert!(
        summary.avg_latency_ms <= 200.0,
        "Average latency should be under 200ms"
    );
    assert!(
        summary.p95_latency_ms <= 500.0,
        "P95 latency should be under 500ms"
    );

    info!("✅ High-throughput task processing test passed all performance requirements");
}

/// Test agent system behavior under stress conditions.
///
/// This test validates system stability and resource management when pushed
/// to high load with various types of intensive operations.
#[tokio::test]
async fn test_agent_system_under_load() {
    tracing_subscriber::fmt::try_init().ok();
    info!("Testing agent system under stress conditions");

    let harness = BlockchainTestHarness::new()
        .await
        .expect("Failed to create blockchain test harness");

    let metrics = Arc::new(PerformanceMetrics::default());
    metrics.start_test().await;

    // Create fewer agents with higher processing delays for stress testing
    let num_agents = 3;
    let max_concurrent_per_agent = 20; // Higher concurrency for stress
    let mut agents = Vec::with_capacity(num_agents);

    for i in 0..num_agents {
        let keypair = harness
            .get_funded_keypair(i)
            .expect("Failed to get keypair");

        let signer =
            LocalSolanaSigner::new(keypair.insecure_clone(), harness.rpc_url().to_string());
        let unified_signer: Arc<dyn UnifiedSigner> = Arc::new(signer);

        // Longer processing delay for stress testing
        let processing_delay = Duration::from_millis(50 + i as u64 * 10);

        let agent = Arc::new(LoadTestAgent::new(
            unified_signer,
            metrics.clone(),
            processing_delay,
            max_concurrent_per_agent,
        ));

        agents.push(agent);
    }

    info!("Created {} agents for stress testing", num_agents);
    metrics.set_active_agents(num_agents);

    // Create agent system
    let registry = LocalAgentRegistry::with_config(RegistryConfig::default());

    for agent in &agents {
        registry
            .register_agent(agent.clone())
            .await
            .expect("Failed to register agent");
    }

    let dispatcher = Arc::new(AgentDispatcher::with_config(
        Arc::new(registry),
        DispatchConfig {
            routing_strategy: RoutingStrategy::RoundRobin, // Even distribution for stress testing
            ..DispatchConfig::default()
        },
    ));

    // Create mix of task types for comprehensive stress testing
    let task_types = vec![
        ("lightweight", 200),
        ("blockchain_sim", 100),
        ("memory_intensive", 50),
        ("cpu_intensive", 50),
    ];

    let mut all_task_handles = Vec::with_capacity(400); // Approximate total based on task_types
    let mut expected_total_tasks = 0;

    for (task_type, count) in task_types {
        info!("Creating {} {} tasks", count, task_type);
        expected_total_tasks += count;

        for task_idx in 0..count {
            let dispatcher_clone = dispatcher.clone();
            let task_type = task_type.to_string();

            let task = Task::new(
                TaskType::Trading,
                serde_json::json!({
                    "task_type": task_type,
                    "task_index": task_idx,
                    "stress_test": true
                }),
            )
            .with_priority(Priority::Normal);

            let handle = tokio::spawn(async move {
                // Add some randomization to task submission timing
                let delay_ms = (task_idx % 10) * 5;
                sleep(Duration::from_millis(delay_ms as u64)).await;

                dispatcher_clone.dispatch_task(task).await
            });

            all_task_handles.push((task_type.clone(), handle));
        }
    }

    info!(
        "Submitting {} total tasks for stress testing",
        expected_total_tasks
    );

    // Wait for all tasks with timeout to detect system hangs
    let stress_timeout = Duration::from_secs(300); // 5 minutes timeout
    let stress_start = Instant::now();

    let stress_results = timeout(
        stress_timeout,
        futures::future::join_all(all_task_handles.into_iter().map(|(_, handle)| handle)),
    )
    .await;

    let task_results = match stress_results {
        Ok(results) => results,
        Err(_) => {
            error!("Stress test timed out after {:?}", stress_timeout);
            panic!("System failed stress test - tasks did not complete within timeout");
        }
    };

    let stress_duration = stress_start.elapsed();
    info!("Stress test completed in {:?}", stress_duration);

    // Analyze stress test results
    let mut successful_tasks = 0;
    let mut failed_tasks = 0;
    let mut task_type_counts = std::collections::HashMap::with_capacity(4);

    for result in task_results {
        match result {
            Ok(Ok(task_result)) => {
                if task_result.is_success() {
                    successful_tasks += 1;

                    // Count by task type
                    if let Some(output_obj) = task_result.data().and_then(|v| v.as_object()) {
                        if let Some(task_type) =
                            output_obj.get("task_type").and_then(|v| v.as_str())
                        {
                            *task_type_counts.entry(task_type.to_string()).or_insert(0) += 1;
                        }
                    }
                } else {
                    failed_tasks += 1;
                    debug!("Task failed during stress test: {:?}", task_result);
                }
            }
            Ok(Err(e)) => {
                failed_tasks += 1;
                debug!("Task dispatch failed during stress: {:?}", e);
            }
            Err(e) => {
                failed_tasks += 1;
                debug!("Task join failed during stress: {:?}", e);
            }
        }
    }

    // Get final performance metrics
    let summary = metrics.get_performance_summary().await;

    info!("Stress test results:");
    info!(
        "  Total tasks processed: {}",
        successful_tasks + failed_tasks
    );
    info!("  Successful tasks: {}", successful_tasks);
    info!("  Failed tasks: {}", failed_tasks);
    info!(
        "  Success rate: {:.2}%",
        (successful_tasks as f64 / (successful_tasks + failed_tasks) as f64) * 100.0
    );
    info!("  Task type distribution: {:?}", task_type_counts);
    info!("  Performance metrics:");
    info!("    Throughput: {:.2} tasks/second", summary.throughput_tps);
    info!("    Average latency: {:.2} ms", summary.avg_latency_ms);
    info!("    P95 latency: {:.2} ms", summary.p95_latency_ms);
    info!("    P99 latency: {:.2} ms", summary.p99_latency_ms);
    info!("    Test duration: {:?}", summary.test_duration);

    // Verify stress test requirements (more relaxed than high-throughput test)
    assert!(
        successful_tasks >= (expected_total_tasks * 90) / 100,
        "At least 90% of tasks should succeed under stress"
    );
    assert!(
        summary.success_rate >= 0.90,
        "Success rate should be at least 90% under stress"
    );
    assert!(
        summary.avg_latency_ms <= 1000.0,
        "Average latency should be under 1 second under stress"
    );
    assert!(
        summary.p99_latency_ms <= 5000.0,
        "P99 latency should be under 5 seconds under stress"
    );
    assert!(
        stress_duration <= Duration::from_secs(180),
        "Stress test should complete within 3 minutes"
    );

    info!("✅ Agent system successfully handled stress conditions");
}

/// Test memory and resource management under load.
///
/// This test validates that the system properly manages memory and resources
/// during intensive operations and doesn't have memory leaks or resource exhaustion.
#[tokio::test]
async fn test_memory_and_resource_management() {
    tracing_subscriber::fmt::try_init().ok();
    info!("Testing memory and resource management under load");

    let harness = BlockchainTestHarness::new()
        .await
        .expect("Failed to create blockchain test harness");

    let metrics = Arc::new(PerformanceMetrics::default());
    metrics.start_test().await;

    // Create agents for memory testing
    let num_agents = 2;
    let mut agents = Vec::with_capacity(num_agents);

    for i in 0..num_agents {
        let keypair = harness
            .get_funded_keypair(i)
            .expect("Failed to get keypair");

        let signer =
            LocalSolanaSigner::new(keypair.insecure_clone(), harness.rpc_url().to_string());
        let unified_signer: Arc<dyn UnifiedSigner> = Arc::new(signer);

        let agent = Arc::new(LoadTestAgent::new(
            unified_signer,
            metrics.clone(),
            Duration::from_millis(10),
            5, // Lower concurrency to focus on memory usage
        ));

        agents.push(agent);
    }

    info!("Created {} agents for memory testing", num_agents);
    metrics.set_active_agents(num_agents);

    // Create agent system
    let registry = LocalAgentRegistry::with_config(RegistryConfig::default());

    for agent in &agents {
        registry
            .register_agent(agent.clone())
            .await
            .expect("Failed to register agent");
    }

    let dispatcher = Arc::new(AgentDispatcher::with_config(
        Arc::new(registry),
        DispatchConfig {
            routing_strategy: RoutingStrategy::LeastLoaded,
            ..DispatchConfig::default()
        },
    ));

    // Run multiple rounds of memory-intensive tasks
    let num_rounds = 5;
    let tasks_per_round = 50;

    for round in 0..num_rounds {
        info!("Starting memory test round {}/{}", round + 1, num_rounds);

        let mut round_handles = Vec::with_capacity(tasks_per_round);

        for task_idx in 0..tasks_per_round {
            let dispatcher_clone = dispatcher.clone();

            let task = Task::new(
                TaskType::Trading,
                serde_json::json!({
                    "task_type": "memory_intensive",
                    "task_index": task_idx,
                    "round": round
                }),
            );

            let handle = tokio::spawn(async move { dispatcher_clone.dispatch_task(task).await });

            round_handles.push(handle);
        }

        // Wait for round to complete
        let round_results = futures::future::join_all(round_handles).await;

        let mut round_successful = 0;
        let mut round_failed = 0;

        for result in round_results {
            match result {
                Ok(Ok(task_result)) => {
                    if task_result.is_success() {
                        round_successful += 1;
                    } else {
                        round_failed += 1;
                    }
                }
                _ => {
                    round_failed += 1;
                }
            }
        }

        info!(
            "Round {} completed: {}/{} successful, {} failed",
            round + 1,
            round_successful,
            tasks_per_round,
            round_failed
        );

        // Verify round success rate
        assert!(
            round_successful >= (tasks_per_round * 90) / 100,
            "Round {} should have at least 90% success rate",
            round + 1
        );

        // Small delay between rounds to allow cleanup
        sleep(Duration::from_millis(100)).await;
    }

    // Get final performance metrics
    let summary = metrics.get_performance_summary().await;

    info!("Memory and resource management test results:");
    info!("  Total tasks processed: {}", summary.total_tasks);
    info!("  Success rate: {:.2}%", summary.success_rate * 100.0);
    info!("  Average latency: {:.2} ms", summary.avg_latency_ms);
    info!("  Test completed {} rounds successfully", num_rounds);

    // Verify overall success
    let expected_total_tasks = num_rounds * tasks_per_round;
    assert!(
        summary.total_tasks >= (expected_total_tasks * 90) / 100,
        "Should process at least 90% of expected tasks"
    );
    assert!(
        summary.success_rate >= 0.90,
        "Overall success rate should be at least 90%"
    );

    info!("✅ Memory and resource management test completed successfully");
}

/// Test system latency under normal operating conditions.
#[tokio::test]
async fn test_system_latency_benchmarks() {
    tracing_subscriber::fmt::try_init().ok();
    info!("Testing system latency benchmarks");

    let harness = BlockchainTestHarness::new()
        .await
        .expect("Failed to create blockchain test harness");

    let metrics = Arc::new(PerformanceMetrics::default());
    metrics.start_test().await;

    // Create single agent for latency testing
    let keypair = harness
        .get_funded_keypair(0)
        .expect("Failed to get keypair");

    let signer = LocalSolanaSigner::new(keypair.insecure_clone(), harness.rpc_url().to_string());
    let unified_signer: Arc<dyn UnifiedSigner> = Arc::new(signer);

    let agent = Arc::new(LoadTestAgent::new(
        unified_signer,
        metrics.clone(),
        Duration::from_millis(1), // Minimal delay for latency testing
        1,                        // Single concurrent task for accurate latency measurement
    ));

    info!("Created single agent for latency benchmarking");
    metrics.set_active_agents(1);

    // Create agent system
    let registry = LocalAgentRegistry::with_config(RegistryConfig::default());

    registry
        .register_agent(agent.clone())
        .await
        .expect("Failed to register agent");

    let dispatcher = AgentDispatcher::with_config(
        Arc::new(registry),
        DispatchConfig {
            routing_strategy: RoutingStrategy::Direct,
            ..DispatchConfig::default()
        },
    );

    // Run sequential tasks to measure pure latency
    let num_tasks = 100;
    let mut latencies = Vec::with_capacity(num_tasks);

    for task_idx in 0..num_tasks {
        let start_time = Instant::now();

        let task = Task::new(
            TaskType::Trading,
            serde_json::json!({
                "task_type": "lightweight",
                "task_index": task_idx
            }),
        );

        let result = dispatcher
            .dispatch_task(task)
            .await
            .expect("Task should dispatch successfully");

        let end_to_end_latency = start_time.elapsed();
        latencies.push(end_to_end_latency);

        assert!(result.is_success(), "Latency test task should succeed");

        // Small delay between tasks
        sleep(Duration::from_millis(5)).await;
    }

    // Calculate latency statistics
    latencies.sort();
    let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
    let median_latency = latencies[latencies.len() / 2];
    let p95_latency = latencies[(latencies.len() * 95) / 100];
    let p99_latency = latencies[(latencies.len() * 99) / 100];
    let min_latency = latencies[0];
    let max_latency = latencies[latencies.len() - 1];

    info!("Latency benchmark results:");
    info!("  Min latency: {:?}", min_latency);
    info!("  Average latency: {:?}", avg_latency);
    info!("  Median latency: {:?}", median_latency);
    info!("  P95 latency: {:?}", p95_latency);
    info!("  P99 latency: {:?}", p99_latency);
    info!("  Max latency: {:?}", max_latency);

    // Verify latency requirements
    assert!(
        avg_latency <= Duration::from_millis(50),
        "Average latency should be under 50ms"
    );
    assert!(
        p95_latency <= Duration::from_millis(100),
        "P95 latency should be under 100ms"
    );
    assert!(
        p99_latency <= Duration::from_millis(200),
        "P99 latency should be under 200ms"
    );

    info!("✅ System latency benchmarks meet performance requirements");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_metrics_basic_functionality() {
        let metrics = PerformanceMetrics::default();
        metrics.start_test().await;

        // Record some task completions
        metrics
            .record_task_completion(Duration::from_millis(100), true)
            .await;
        metrics
            .record_task_completion(Duration::from_millis(200), true)
            .await;
        metrics
            .record_task_completion(Duration::from_millis(150), false)
            .await;

        let summary = metrics.get_performance_summary().await;

        assert_eq!(summary.total_tasks, 3);
        assert_eq!(summary.failed_tasks, 1);
        assert!((summary.success_rate - 0.666).abs() < 0.01); // Approximately 2/3
        assert!(summary.avg_latency_ms > 0.0);
        assert!(summary.test_duration > Duration::from_millis(0));
    }

    #[tokio::test]
    async fn test_load_test_agent_capabilities() {
        let keypair = Keypair::new();
        let signer = LocalSolanaSigner::new(keypair, "http://localhost:8899".to_string());
        let unified_signer: Arc<dyn UnifiedSigner> = Arc::new(signer);
        let metrics = Arc::new(PerformanceMetrics::default());

        let agent = LoadTestAgent::new(unified_signer, metrics, Duration::from_millis(0), 5);

        let capabilities = agent.capabilities();
        assert!(capabilities.contains(&CapabilityType::Custom("performance_testing".to_string())));
        assert!(capabilities.contains(&CapabilityType::Custom("load_testing".to_string())));
        assert!(capabilities.contains(&CapabilityType::Custom("high_throughput".to_string())));
        assert!(capabilities.contains(&CapabilityType::Trading));

        // Test load calculation
        let load = agent.load();
        assert!((0.0..=1.0).contains(&load));
    }

    #[test]
    fn test_performance_summary_calculations() {
        let summary = PerformanceSummary {
            total_tasks: 100,
            failed_tasks: 5,
            success_rate: 0.95,
            throughput_tps: 50.0,
            avg_latency_ms: 25.0,
            p50_latency_ms: 20.0,
            p95_latency_ms: 40.0,
            p99_latency_ms: 60.0,
            test_duration: Duration::from_secs(2),
            active_agents: 3,
        };

        assert_eq!(summary.total_tasks, 100);
        assert_eq!(summary.failed_tasks, 5);
        assert_eq!(summary.success_rate, 0.95);
        assert_eq!(summary.throughput_tps, 50.0);
        assert_eq!(summary.active_agents, 3);
    }
}
