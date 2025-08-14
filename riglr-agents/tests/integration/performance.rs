//! Performance and scalability tests for the riglr-agents system.
//!
//! These tests verify that the agent system performs well under various
//! load conditions, scales appropriately with the number of agents and tasks,
//! and maintains acceptable latency and throughput characteristics.

use riglr_agents::*;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use serde_json::json;
use tokio::sync::Semaphore;

/// High-performance agent for benchmarking.
#[derive(Clone)]
struct BenchmarkAgent {
    id: AgentId,
    processing_time: Duration,
    task_counter: Arc<AtomicU64>,
    total_processing_time: Arc<AtomicU64>,
}

impl BenchmarkAgent {
    fn new(id: impl Into<String>, processing_time: Duration) -> Self {
        Self {
            id: AgentId::new(id),
            processing_time,
            task_counter: Arc::new(AtomicU64::new(0)),
            total_processing_time: Arc::new(AtomicU64::new(0)),
        }
    }

    fn get_task_count(&self) -> u64 {
        self.task_counter.load(Ordering::Relaxed)
    }

    fn get_total_processing_time(&self) -> Duration {
        Duration::from_nanos(self.total_processing_time.load(Ordering::Relaxed))
    }

    fn get_average_processing_time(&self) -> Duration {
        let count = self.get_task_count();
        if count == 0 {
            Duration::ZERO
        } else {
            self.get_total_processing_time() / count as u32
        }
    }
}

#[async_trait::async_trait]
impl Agent for BenchmarkAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        let start = Instant::now();
        
        // Simulate processing work
        if self.processing_time > Duration::ZERO {
            tokio::time::sleep(self.processing_time).await;
        }

        let elapsed = start.elapsed();
        
        // Update metrics
        self.task_counter.fetch_add(1, Ordering::Relaxed);
        self.total_processing_time.fetch_add(elapsed.as_nanos() as u64, Ordering::Relaxed);

        Ok(TaskResult::success(
            json!({
                "agent_id": self.id.as_str(),
                "task_id": task.id,
                "processing_time_ms": elapsed.as_millis(),
                "timestamp": chrono::Utc::now().timestamp_millis()
            }),
            None,
            elapsed,
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["benchmark".to_string(), "performance".to_string()]
    }

    fn load(&self) -> f64 {
        // Simulate load based on recent task count
        let count = self.task_counter.load(Ordering::Relaxed);
        (count % 10) as f64 / 10.0
    }
}

/// Memory-intensive agent for memory usage testing.
#[derive(Clone)]
struct MemoryIntensiveAgent {
    id: AgentId,
    memory_allocation_size: usize,
}

impl MemoryIntensiveAgent {
    fn new(id: impl Into<String>, memory_size_mb: usize) -> Self {
        Self {
            id: AgentId::new(id),
            memory_allocation_size: memory_size_mb * 1024 * 1024, // Convert MB to bytes
        }
    }
}

#[async_trait::async_trait]
impl Agent for MemoryIntensiveAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        let start = Instant::now();
        
        // Allocate and use memory
        let mut data = vec![0u8; self.memory_allocation_size];
        
        // Simulate some work with the memory
        for i in (0..data.len()).step_by(4096) {
            data[i] = (i % 256) as u8;
        }
        
        // Small delay to simulate processing
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let checksum: u64 = data.iter().map(|&x| x as u64).sum();
        let elapsed = start.elapsed();

        Ok(TaskResult::success(
            json!({
                "agent_id": self.id.as_str(),
                "task_id": task.id,
                "memory_used_mb": self.memory_allocation_size / (1024 * 1024),
                "checksum": checksum,
                "processing_time_ms": elapsed.as_millis()
            }),
            None,
            elapsed,
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["memory_intensive".to_string(), "benchmark".to_string()]
    }
}

/// CPU-intensive agent for CPU load testing.
#[derive(Clone)]
struct CpuIntensiveAgent {
    id: AgentId,
    work_iterations: u64,
}

impl CpuIntensiveAgent {
    fn new(id: impl Into<String>, work_iterations: u64) -> Self {
        Self {
            id: AgentId::new(id),
            work_iterations,
        }
    }
}

#[async_trait::async_trait]
impl Agent for CpuIntensiveAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        let start = Instant::now();
        
        // CPU-intensive work
        let mut result = 0u64;
        for i in 0..self.work_iterations {
            result = result.wrapping_mul(31).wrapping_add(i);
            // Yield occasionally to prevent blocking
            if i % 10000 == 0 {
                tokio::task::yield_now().await;
            }
        }
        
        let elapsed = start.elapsed();

        Ok(TaskResult::success(
            json!({
                "agent_id": self.id.as_str(),
                "task_id": task.id,
                "work_result": result,
                "iterations": self.work_iterations,
                "processing_time_ms": elapsed.as_millis()
            }),
            None,
            elapsed,
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["cpu_intensive".to_string(), "benchmark".to_string()]
    }
}

/// Test basic throughput with lightweight tasks.
#[tokio::test]
async fn test_basic_throughput() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    // Create multiple fast agents
    let num_agents = 4;
    let mut agents = Vec::new();
    
    for i in 0..num_agents {
        let agent = Arc::new(BenchmarkAgent::new(
            format!("throughput-agent-{}", i),
            Duration::from_millis(1),
        ));
        agents.push(agent.clone());
        registry.register_agent(agent).await.unwrap();
    }

    let num_tasks = 1000;
    let tasks: Vec<Task> = (0..num_tasks)
        .map(|i| Task::new(
            TaskType::Custom("benchmark".to_string()),
            json!({"task_index": i}),
        ))
        .collect();

    let start_time = Instant::now();
    let results = dispatcher.dispatch_tasks(tasks).await;
    let elapsed = start_time.elapsed();

    // Verify results
    assert_eq!(results.len(), num_tasks);
    let success_count = results.iter().filter(|r| r.is_ok()).count();
    
    // Calculate throughput
    let throughput = success_count as f64 / elapsed.as_secs_f64();
    
    println!("Throughput: {:.2} tasks/second", throughput);
    println!("Total time: {:?}", elapsed);
    println!("Success rate: {:.2}%", (success_count as f64 / num_tasks as f64) * 100.0);
    
    // Should achieve reasonable throughput
    assert!(throughput > 100.0); // At least 100 tasks per second
    assert!(success_count >= num_tasks * 95 / 100); // At least 95% success rate
    
    // Verify load distribution across agents
    let total_tasks_processed: u64 = agents.iter().map(|a| a.get_task_count()).sum();
    assert!(total_tasks_processed >= success_count as u64);
}

/// Test scalability with increasing number of agents.
#[tokio::test]
async fn test_agent_scalability() {
    let agent_counts = vec![1, 2, 4, 8];
    let tasks_per_test = 200;
    
    for &num_agents in &agent_counts {
        let registry = Arc::new(LocalAgentRegistry::new());
        let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

        // Create agents
        for i in 0..num_agents {
            let agent = Arc::new(BenchmarkAgent::new(
                format!("scale-agent-{}", i),
                Duration::from_millis(5),
            ));
            registry.register_agent(agent).await.unwrap();
        }

        // Create tasks
        let tasks: Vec<Task> = (0..tasks_per_test)
            .map(|i| Task::new(
                TaskType::Custom("benchmark".to_string()),
                json!({"task_index": i, "agent_count": num_agents}),
            ))
            .collect();

        let start_time = Instant::now();
        let results = dispatcher.dispatch_tasks(tasks).await;
        let elapsed = start_time.elapsed();

        let success_count = results.iter().filter(|r| r.is_ok()).count();
        let throughput = success_count as f64 / elapsed.as_secs_f64();

        println!("Agents: {}, Throughput: {:.2} tasks/sec, Time: {:?}", 
                 num_agents, throughput, elapsed);
        
        assert!(success_count >= tasks_per_test * 90 / 100); // 90% success rate
        
        // With more agents, we expect better throughput (up to a point)
        if num_agents > 1 {
            assert!(throughput > 10.0); // Should scale reasonably
        }
    }
}

/// Test load balancing effectiveness.
#[tokio::test]
async fn test_load_balancing_performance() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let config = DispatchConfig {
        enable_load_balancing: true,
        routing_strategy: RoutingStrategy::LeastLoaded,
        ..Default::default()
    };
    let dispatcher = Arc::new(AgentDispatcher::with_config(registry.clone(), config));

    let num_agents = 6;
    let mut agents = Vec::new();
    
    // Create agents with different processing speeds
    for i in 0..num_agents {
        let processing_time = Duration::from_millis(10 + (i * 5) as u64);
        let agent = Arc::new(BenchmarkAgent::new(
            format!("load-balance-agent-{}", i),
            processing_time,
        ));
        agents.push(agent.clone());
        registry.register_agent(agent).await.unwrap();
    }

    let num_tasks = 100;
    let tasks: Vec<Task> = (0..num_tasks)
        .map(|i| Task::new(
            TaskType::Custom("benchmark".to_string()),
            json!({"task_index": i}),
        ))
        .collect();

    let start_time = Instant::now();
    let results = dispatcher.dispatch_tasks(tasks).await;
    let elapsed = start_time.elapsed();

    let success_count = results.iter().filter(|r| r.is_ok()).count();
    
    println!("Load balancing test - Time: {:?}, Success: {}/{}", 
             elapsed, success_count, num_tasks);

    // Verify task distribution
    let task_counts: Vec<u64> = agents.iter().map(|a| a.get_task_count()).collect();
    println!("Task distribution: {:?}", task_counts);

    // Check that tasks were distributed (not all on one agent)
    let max_tasks = task_counts.iter().max().unwrap_or(&0);
    let min_tasks = task_counts.iter().min().unwrap_or(&0);
    let distribution_ratio = if *max_tasks > 0 { 
        *min_tasks as f64 / *max_tasks as f64 
    } else { 
        0.0 
    };

    println!("Distribution ratio (min/max): {:.2}", distribution_ratio);
    
    // Should have reasonable distribution (not perfect due to load balancing heuristics)
    assert!(distribution_ratio > 0.3); // At least some distribution
    assert!(success_count >= num_tasks * 95 / 100);
}

/// Test memory usage under high load.
#[tokio::test]
#[ignore] // Ignore for regular test runs due to memory requirements
async fn test_memory_usage() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    // Create memory-intensive agents
    let num_agents = 3;
    let memory_per_agent_mb = 10; // 10MB per agent
    
    for i in 0..num_agents {
        let agent = Arc::new(MemoryIntensiveAgent::new(
            format!("memory-agent-{}", i),
            memory_per_agent_mb,
        ));
        registry.register_agent(agent).await.unwrap();
    }

    let num_tasks = 20;
    let tasks: Vec<Task> = (0..num_tasks)
        .map(|i| Task::new(
            TaskType::Custom("memory_intensive".to_string()),
            json!({"task_index": i}),
        ))
        .collect();

    let start_time = Instant::now();
    
    // Monitor memory usage (simplified)
    let initial_memory = get_process_memory_mb().unwrap_or(0);
    
    let results = dispatcher.dispatch_tasks(tasks).await;
    
    let final_memory = get_process_memory_mb().unwrap_or(0);
    let elapsed = start_time.elapsed();

    let success_count = results.iter().filter(|r| r.is_ok()).count();
    let memory_increase = final_memory.saturating_sub(initial_memory);

    println!("Memory test - Time: {:?}, Success: {}/{}", 
             elapsed, success_count, num_tasks);
    println!("Memory usage: {} MB -> {} MB (+{} MB)", 
             initial_memory, final_memory, memory_increase);
    
    assert!(success_count >= num_tasks * 90 / 100);
    // Memory usage should be reasonable (not excessive leaks)
    assert!(memory_increase < 1000); // Less than 1GB increase
}

/// Test CPU-intensive task performance.
#[tokio::test]
async fn test_cpu_intensive_performance() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    let num_agents = 2;
    let work_iterations = 100_000;
    
    for i in 0..num_agents {
        let agent = Arc::new(CpuIntensiveAgent::new(
            format!("cpu-agent-{}", i),
            work_iterations,
        ));
        registry.register_agent(agent).await.unwrap();
    }

    let num_tasks = 10;
    let tasks: Vec<Task> = (0..num_tasks)
        .map(|i| Task::new(
            TaskType::Custom("cpu_intensive".to_string()),
            json!({"task_index": i}),
        ))
        .collect();

    let start_time = Instant::now();
    let results = dispatcher.dispatch_tasks(tasks).await;
    let elapsed = start_time.elapsed();

    let success_count = results.iter().filter(|r| r.is_ok()).count();
    
    println!("CPU intensive test - Time: {:?}, Success: {}/{}", 
             elapsed, success_count, num_tasks);
    
    // Calculate average processing time
    let mut total_processing_time = Duration::ZERO;
    let mut processed_results = 0;
    
    for result in &results {
        if let Ok(task_result) = result {
            if let Some(data) = task_result.data() {
                if let Some(time_ms) = data.get("processing_time_ms").and_then(|v| v.as_u64()) {
                    total_processing_time += Duration::from_millis(time_ms);
                    processed_results += 1;
                }
            }
        }
    }
    
    let avg_processing_time = if processed_results > 0 {
        total_processing_time / processed_results
    } else {
        Duration::ZERO
    };
    
    println!("Average processing time: {:?}", avg_processing_time);
    
    assert!(success_count >= num_tasks * 90 / 100);
    assert!(avg_processing_time < Duration::from_secs(5)); // Should complete within reasonable time
}

/// Test concurrent task execution limits.
#[tokio::test]
async fn test_concurrency_limits() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let config = DispatchConfig {
        max_concurrent_tasks_per_agent: 5,
        ..Default::default()
    };
    let dispatcher = Arc::new(AgentDispatcher::with_config(registry.clone(), config));

    // Single agent to test concurrency limits
    let agent = Arc::new(BenchmarkAgent::new(
        "concurrency-agent",
        Duration::from_millis(100), // Longer processing time
    ));
    registry.register_agent(agent.clone()).await.unwrap();

    let num_tasks = 20;
    let tasks: Vec<Task> = (0..num_tasks)
        .map(|i| Task::new(
            TaskType::Custom("benchmark".to_string()),
            json!({"task_index": i}),
        ))
        .collect();

    let start_time = Instant::now();
    let results = dispatcher.dispatch_tasks(tasks).await;
    let elapsed = start_time.elapsed();

    let success_count = results.iter().filter(|r| r.is_ok()).count();
    
    println!("Concurrency test - Time: {:?}, Success: {}/{}", 
             elapsed, success_count, num_tasks);
    
    // With concurrency limits, tasks should be processed in batches
    // The total time should reflect this batching
    let expected_min_time = Duration::from_millis(100 * 4); // At least 4 batches
    assert!(elapsed >= expected_min_time);
    assert!(success_count >= num_tasks * 90 / 100);
}

/// Test task queue performance under burst load.
#[tokio::test]
async fn test_burst_load_performance() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    // Multiple agents for handling burst
    let num_agents = 8;
    for i in 0..num_agents {
        let agent = Arc::new(BenchmarkAgent::new(
            format!("burst-agent-{}", i),
            Duration::from_millis(10),
        ));
        registry.register_agent(agent).await.unwrap();
    }

    // Create burst of tasks
    let burst_size = 500;
    let tasks: Vec<Task> = (0..burst_size)
        .map(|i| Task::new(
            TaskType::Custom("benchmark".to_string()),
            json!({"burst_index": i}),
        ))
        .collect();

    let start_time = Instant::now();
    let results = dispatcher.dispatch_tasks(tasks).await;
    let elapsed = start_time.elapsed();

    let success_count = results.iter().filter(|r| r.is_ok()).count();
    let throughput = success_count as f64 / elapsed.as_secs_f64();

    println!("Burst load test - Time: {:?}, Throughput: {:.2} tasks/sec", 
             elapsed, throughput);
    
    assert!(success_count >= burst_size * 95 / 100);
    assert!(throughput > 50.0); // Should handle burst efficiently
}

/// Test latency characteristics under different loads.
#[tokio::test]
async fn test_latency_characteristics() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    let agent = Arc::new(BenchmarkAgent::new(
        "latency-agent",
        Duration::from_millis(5),
    ));
    registry.register_agent(agent).await.unwrap();

    let mut latencies = Vec::new();
    
    // Measure individual task latencies
    for i in 0..50 {
        let task = Task::new(
            TaskType::Custom("benchmark".to_string()),
            json!({"latency_test": i}),
        );

        let start = Instant::now();
        let result = dispatcher.dispatch_task(task).await.unwrap();
        let latency = start.elapsed();

        latencies.push(latency);
        assert!(result.is_success());
        
        // Small delay between tasks to avoid overwhelming
        tokio::time::sleep(Duration::from_millis(1)).await;
    }

    // Calculate latency statistics
    latencies.sort();
    let min_latency = latencies[0];
    let max_latency = latencies[latencies.len() - 1];
    let median_latency = latencies[latencies.len() / 2];
    let p95_latency = latencies[(latencies.len() as f64 * 0.95) as usize];
    let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;

    println!("Latency statistics:");
    println!("  Min: {:?}", min_latency);
    println!("  Avg: {:?}", avg_latency);
    println!("  Median: {:?}", median_latency);
    println!("  P95: {:?}", p95_latency);
    println!("  Max: {:?}", max_latency);

    // Latency assertions
    assert!(median_latency < Duration::from_millis(100));
    assert!(p95_latency < Duration::from_millis(200));
    assert!(max_latency < Duration::from_secs(1));
}

/// Test system performance under sustained load.
#[tokio::test]
#[ignore] // Ignore for regular test runs due to duration
async fn test_sustained_load_performance() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    // Create moderate number of agents
    let num_agents = 4;
    let mut agents = Vec::new();
    
    for i in 0..num_agents {
        let agent = Arc::new(BenchmarkAgent::new(
            format!("sustained-agent-{}", i),
            Duration::from_millis(20),
        ));
        agents.push(agent.clone());
        registry.register_agent(agent).await.unwrap();
    }

    let test_duration = Duration::from_secs(30); // 30 seconds of sustained load
    let task_interval = Duration::from_millis(100); // New task every 100ms
    
    let start_time = Instant::now();
    let mut task_count = 0;
    let mut results = Vec::new();

    while start_time.elapsed() < test_duration {
        let task = Task::new(
            TaskType::Custom("benchmark".to_string()),
            json!({"sustained_test": task_count}),
        );

        let task_start = Instant::now();
        let result = dispatcher.dispatch_task(task).await;
        let task_latency = task_start.elapsed();

        results.push((result.is_ok(), task_latency));
        task_count += 1;

        tokio::time::sleep(task_interval).await;
    }

    let total_elapsed = start_time.elapsed();
    let success_count = results.iter().filter(|(success, _)| *success).count();
    let avg_throughput = task_count as f64 / total_elapsed.as_secs_f64();

    // Calculate latency statistics
    let mut latencies: Vec<Duration> = results.iter()
        .filter(|(success, _)| *success)
        .map(|(_, latency)| *latency)
        .collect();
    latencies.sort();

    let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
    let p95_latency = latencies[(latencies.len() as f64 * 0.95) as usize];

    println!("Sustained load test results:");
    println!("  Duration: {:?}", total_elapsed);
    println!("  Tasks: {}", task_count);
    println!("  Success rate: {:.2}%", (success_count as f64 / task_count as f64) * 100.0);
    println!("  Throughput: {:.2} tasks/sec", avg_throughput);
    println!("  Avg latency: {:?}", avg_latency);
    println!("  P95 latency: {:?}", p95_latency);

    // Verify sustained performance
    assert!(success_count >= task_count * 95 / 100); // 95% success rate
    assert!(avg_throughput > 5.0); // Maintain reasonable throughput
    assert!(p95_latency < Duration::from_secs(2)); // Reasonable P95 latency
    
    // Verify agents handled tasks
    let total_agent_tasks: u64 = agents.iter().map(|a| a.get_task_count()).sum();
    assert!(total_agent_tasks >= success_count as u64);
}

/// Helper function to get process memory usage (simplified).
fn get_process_memory_mb() -> Option<usize> {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        let contents = fs::read_to_string("/proc/self/status").ok()?;
        for line in contents.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb: usize = parts[1].parse().ok()?;
                    return Some(kb / 1024); // Convert KB to MB
                }
            }
        }
        None
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        // Simplified fallback - return a dummy value
        Some(100) // 100MB as a placeholder
    }
}

/// Test dispatcher statistics accuracy under load.
#[tokio::test]
async fn test_dispatcher_statistics_accuracy() {
    let registry = Arc::new(LocalAgentRegistry::new());
    let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

    let num_agents = 3;
    for i in 0..num_agents {
        let agent = Arc::new(BenchmarkAgent::new(
            format!("stats-agent-{}", i),
            Duration::from_millis(10),
        ));
        registry.register_agent(agent).await.unwrap();
    }

    // Execute some tasks
    let num_tasks = 30;
    let tasks: Vec<Task> = (0..num_tasks)
        .map(|i| Task::new(
            TaskType::Custom("benchmark".to_string()),
            json!({"stats_test": i}),
        ))
        .collect();

    let results = dispatcher.dispatch_tasks(tasks).await;
    let success_count = results.iter().filter(|r| r.is_ok()).count();

    // Get dispatcher statistics
    let stats = dispatcher.stats().await.unwrap();
    
    println!("Dispatcher statistics:");
    println!("  Registered agents: {}", stats.registered_agents);
    println!("  Total active tasks: {}", stats.total_active_tasks);
    println!("  Average agent load: {:.2}", stats.average_agent_load);
    println!("  Routing strategy: {:?}", stats.routing_strategy);

    // Verify statistics accuracy
    assert_eq!(stats.registered_agents, num_agents);
    assert!(success_count >= num_tasks * 90 / 100);
    
    // Load should be reasonable (tasks completed, so load might be low)
    assert!(stats.average_agent_load >= 0.0 && stats.average_agent_load <= 1.0);
}

/// Benchmark different routing strategies.
#[tokio::test]
async fn test_routing_strategy_performance() {
    let strategies = vec![
        RoutingStrategy::Capability,
        RoutingStrategy::RoundRobin,
        RoutingStrategy::LeastLoaded,
        RoutingStrategy::Random,
    ];

    for strategy in strategies {
        let registry = Arc::new(LocalAgentRegistry::new());
        let config = DispatchConfig {
            routing_strategy: strategy,
            enable_load_balancing: true,
            ..Default::default()
        };
        let dispatcher = Arc::new(AgentDispatcher::with_config(registry.clone(), config));

        // Create agents with different processing times
        for i in 0..4 {
            let processing_time = Duration::from_millis(10 + (i * 5) as u64);
            let agent = Arc::new(BenchmarkAgent::new(
                format!("routing-agent-{}", i),
                processing_time,
            ));
            registry.register_agent(agent).await.unwrap();
        }

        let num_tasks = 100;
        let tasks: Vec<Task> = (0..num_tasks)
            .map(|i| Task::new(
                TaskType::Custom("benchmark".to_string()),
                json!({"routing_test": i}),
            ))
            .collect();

        let start_time = Instant::now();
        let results = dispatcher.dispatch_tasks(tasks).await;
        let elapsed = start_time.elapsed();

        let success_count = results.iter().filter(|r| r.is_ok()).count();
        let throughput = success_count as f64 / elapsed.as_secs_f64();

        println!("Strategy {:?}: {:.2} tasks/sec, {:?}", 
                 strategy, throughput, elapsed);

        assert!(success_count >= num_tasks * 90 / 100);
        assert!(throughput > 10.0);
    }
}