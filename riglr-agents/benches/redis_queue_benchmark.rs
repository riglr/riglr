//! Benchmarks for Redis-based distributed task queue scalability
//!
//! This benchmark suite evaluates the performance of the current Redis-based
//! distributed queue implementation under various load conditions, measuring:
//! - Throughput (tasks per second)
//! - Latency (time per task)
//! - Key proliferation (number of Redis keys created)

#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use riglr_agents::{
    dispatcher::queue::RedisTaskQueue,
    dispatcher::queue_trait::DistributedTaskQueue,
    types::{AgentId, Task, TaskResult, TaskType},
};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

const REDIS_URL_ENV_VAR: &str = "REDIS_URL";

/// Count the number of keys in Redis (for monitoring key proliferation)
async fn count_redis_keys(redis_url: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let client = redis::Client::open(redis_url)?;
    let mut conn = client.get_multiplexed_async_connection().await?;
    let keys: Vec<String> = redis::cmd("KEYS").arg("*").query_async(&mut conn).await?;
    Ok(keys.len())
}

/// Clean up all test keys from Redis
async fn cleanup_redis(redis_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = redis::Client::open(redis_url)?;
    let mut conn = client.get_multiplexed_async_connection().await?;

    // Delete all keys with our test prefixes
    let keys: Vec<String> = redis::cmd("KEYS")
        .arg("response:*")
        .query_async(&mut conn)
        .await?;

    if !keys.is_empty() {
        redis::cmd("DEL")
            .arg(&keys)
            .query_async::<()>(&mut conn)
            .await?;
    }

    Ok(())
}

/// Simulate a remote agent that processes tasks
async fn simulate_remote_agent(
    redis_url: String,
    agent_id: String,
    task_counter: Arc<AtomicUsize>,
) {
    let client = redis::Client::open(redis_url.as_str()).unwrap();
    let mut conn = client.get_multiplexed_async_connection().await.unwrap();

    loop {
        // Listen for tasks on the agent's queue
        let task_queue_key = format!("agent:{}:tasks", agent_id);

        // Use BRPOP with 1 second timeout
        let result: Option<(String, String)> = redis::cmd("BRPOP")
            .arg(&task_queue_key)
            .arg(1)
            .query_async(&mut conn)
            .await
            .unwrap_or(None);

        if let Some((_, task_json)) = result {
            // Deserialize task
            if let Ok(task) = serde_json::from_str::<Task>(&task_json) {
                // Increment counter
                task_counter.fetch_add(1, Ordering::Relaxed);

                // Create a mock result
                let result = TaskResult::success(
                    serde_json::json!({
                        "status": "completed",
                        "agent": agent_id,
                        "task_id": task.id,
                    }),
                    None,
                    Duration::from_millis(10),
                );

                // Send response back
                let response_key = format!("response:{}", task.id);
                let result_json = serde_json::to_string(&result).unwrap();

                redis::cmd("LPUSH")
                    .arg(&response_key)
                    .arg(&result_json)
                    .query_async::<()>(&mut conn)
                    .await
                    .unwrap();
            }
        }

        // Check if we should stop
        if task_counter.load(Ordering::Relaxed) >= 1000 {
            break;
        }
    }
}

fn benchmark_single_task_dispatch(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();
    let redis_url =
        std::env::var(REDIS_URL_ENV_VAR).unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // Clean up before starting
    runtime.block_on(cleanup_redis(&redis_url)).unwrap();

    c.bench_function("single_task_dispatch", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let client = redis::Client::open(redis_url.as_str()).unwrap();
                let conn = client.get_multiplexed_async_connection().await.unwrap();
                let queue = RedisTaskQueue::new(conn);
                let agent_id = AgentId::generate();

                let task = Task::new(
                    TaskType::Trading,
                    serde_json::json!({
                        "action": "test",
                        "value": 42,
                    }),
                );

                // Simulate the remote agent in background
                let agent_id_str = agent_id.to_string();
                let redis_url_clone = redis_url.clone();
                let task_counter = Arc::new(AtomicUsize::new(0));
                let counter_clone = task_counter.clone();

                let agent_handle = tokio::spawn(async move {
                    simulate_remote_agent(redis_url_clone, agent_id_str, counter_clone).await
                });

                // Dispatch the task
                let result = queue
                    .dispatch_remote_task(&agent_id, task, Duration::from_secs(5))
                    .await;

                // Clean up
                agent_handle.abort();

                black_box(result)
            })
        });
    });

    // Clean up after test
    runtime.block_on(cleanup_redis(&redis_url)).unwrap();
}

fn benchmark_concurrent_dispatch_throughput(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();
    let redis_url =
        std::env::var(REDIS_URL_ENV_VAR).unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    let mut group = c.benchmark_group("concurrent_dispatch");

    for num_tasks in &[10, 50, 100, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_tasks),
            num_tasks,
            |b, &num_tasks| {
                b.iter(|| {
                    runtime.block_on(async {
                        // Clean up before test
                        cleanup_redis(&redis_url).await.unwrap();

                        let client = redis::Client::open(redis_url.as_str()).unwrap();
                        let conn = client.get_multiplexed_async_connection().await.unwrap();
                        let queue = Arc::new(RedisTaskQueue::new(conn));
                        let agent_id = AgentId::generate();

                        // Start monitoring keys before dispatch
                        let keys_before = count_redis_keys(&redis_url).await.unwrap();

                        // Simulate remote agent
                        let agent_id_str = agent_id.to_string();
                        let redis_url_clone = redis_url.clone();
                        let task_counter = Arc::new(AtomicUsize::new(0));
                        let counter_clone = task_counter.clone();

                        let agent_handle = tokio::spawn(async move {
                            simulate_remote_agent(redis_url_clone, agent_id_str, counter_clone)
                                .await
                        });

                        // Dispatch tasks concurrently
                        let mut handles = vec![];
                        for i in 0..num_tasks {
                            let queue = queue.clone();
                            let agent_id = agent_id.clone();

                            let handle = tokio::spawn(async move {
                                let task = Task::new(
                                    TaskType::Trading,
                                    serde_json::json!({
                                        "action": "test",
                                        "index": i,
                                    }),
                                );

                                queue
                                    .dispatch_remote_task(&agent_id, task, Duration::from_secs(5))
                                    .await
                            });

                            handles.push(handle);
                        }

                        // Wait for all tasks to complete
                        for handle in handles {
                            let _ = handle.await;
                        }

                        // Count keys after dispatch
                        let keys_after = count_redis_keys(&redis_url).await.unwrap();
                        let keys_created = keys_after - keys_before;

                        // Clean up
                        agent_handle.abort();
                        cleanup_redis(&redis_url).await.unwrap();

                        black_box((num_tasks, keys_created))
                    })
                });
            },
        );
    }

    group.finish();
}

fn benchmark_key_proliferation(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();
    let redis_url =
        std::env::var(REDIS_URL_ENV_VAR).unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    c.bench_function("key_proliferation_1000_tasks", |b| {
        b.iter(|| {
            runtime.block_on(async {
                // Clean up before test
                cleanup_redis(&redis_url).await.unwrap();

                let initial_keys = count_redis_keys(&redis_url).await.unwrap();

                // Create many response keys (simulating incomplete task cleanup)
                let client = redis::Client::open(redis_url.as_str()).unwrap();
                let mut conn = client.get_multiplexed_async_connection().await.unwrap();

                for i in 0..1000 {
                    let key = format!("response:task_{}", i);
                    redis::cmd("SET")
                        .arg(&key)
                        .arg("test_value")
                        .arg("EX")
                        .arg(60) // Expire after 60 seconds
                        .query_async::<()>(&mut conn)
                        .await
                        .unwrap();
                }

                let final_keys = count_redis_keys(&redis_url).await.unwrap();
                let keys_created = final_keys - initial_keys;

                // Clean up
                cleanup_redis(&redis_url).await.unwrap();

                black_box(keys_created)
            })
        });
    });
}

criterion_group!(
    benches,
    benchmark_single_task_dispatch,
    benchmark_concurrent_dispatch_throughput,
    benchmark_key_proliferation
);
criterion_main!(benches);
