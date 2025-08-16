use async_trait::async_trait;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use riglr_core::{
    ExecutionConfig, IdempotencyStore, InMemoryIdempotencyStore, InMemoryJobQueue, Job, JobQueue,
    JobResult, Tool, ToolWorker,
};
use serde::{Deserialize, Serialize};
use std::hint::black_box;
use std::sync::Arc;
use std::time::Duration;

const REDIS_URL: &str = "REDIS_URL";
use tokio::runtime::Runtime;

#[cfg(feature = "redis")]
use riglr_core::RedisJobQueue;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestParams {
    value: i32,
    message: String,
}

struct TestTool;

#[async_trait]
impl Tool for TestTool {
    async fn execute(
        &self,
        params: serde_json::Value,
    ) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
        let _params: TestParams = serde_json::from_value(params)?;
        tokio::time::sleep(Duration::from_micros(10)).await;
        Ok(JobResult::success(&"completed")?)
    }

    fn name(&self) -> &str {
        "test_tool"
    }

    fn description(&self) -> &str {
        ""
    }
}

fn job_creation_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("job_creation");

    let params = TestParams {
        value: 42,
        message: "test message".to_string(),
    };

    group.bench_function("new_job", |b| {
        b.iter(|| Job::new(black_box("test_tool"), black_box(&params), black_box(3)))
    });

    group.bench_function("new_idempotent_job", |b| {
        b.iter(|| {
            Job::new_idempotent(
                black_box("test_tool"),
                black_box(&params),
                black_box(3),
                black_box("idempotency_key_123"),
            )
        })
    });

    group.bench_function("can_retry", |b| {
        let job = Job::new("test_tool", &params, 3).unwrap();
        b.iter(|| black_box(job.can_retry()))
    });

    group.bench_function("increment_retry", |b| {
        b.iter(|| {
            let mut job = Job::new("test_tool", &params, 3).unwrap();
            job.increment_retry();
            black_box(job.retry_count)
        })
    });

    group.finish();
}

fn job_result_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("job_result");

    group.bench_function("success", |b| b.iter(|| JobResult::success(&"test_value")));

    group.bench_function("success_with_tx", |b| {
        b.iter(|| JobResult::success_with_tx(&"test_value", "tx_hash_123"))
    });

    group.bench_function("retriable_failure", |b| {
        b.iter(|| JobResult::retriable_failure("test error"))
    });

    group.bench_function("permanent_failure", |b| {
        b.iter(|| JobResult::permanent_failure("test error"))
    });

    group.bench_function("is_success", |b| {
        let result = JobResult::success(&"test").unwrap();
        b.iter(|| black_box(result.is_success()))
    });

    group.bench_function("is_retriable", |b| {
        let result = JobResult::retriable_failure("error");
        b.iter(|| black_box(result.is_retriable()))
    });

    group.finish();
}

fn queue_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("queue_operations");
    let rt = Runtime::new().unwrap();

    let params = TestParams {
        value: 42,
        message: "test message".to_string(),
    };

    group.bench_function("inmemory_enqueue", |b| {
        let queue = Arc::new(InMemoryJobQueue::new());
        b.iter(|| {
            rt.block_on(async {
                let job = Job::new("test_tool", &params, 3).unwrap();
                queue.enqueue(black_box(job)).await.unwrap()
            })
        })
    });

    group.bench_function("inmemory_dequeue", |b| {
        let queue = Arc::new(InMemoryJobQueue::new());
        rt.block_on(async {
            for _ in 0..1000 {
                let job = Job::new("test_tool", &params, 3).unwrap();
                queue.enqueue(job).await.unwrap();
            }
        });

        b.iter(|| {
            rt.block_on(async {
                queue
                    .dequeue_with_timeout(Duration::from_millis(10))
                    .await
                    .unwrap()
            })
        })
    });

    group.bench_function("inmemory_queue_len", |b| {
        let queue = Arc::new(InMemoryJobQueue::new());
        rt.block_on(async {
            for _ in 0..100 {
                let job = Job::new("test_tool", &params, 3).unwrap();
                queue.enqueue(job).await.unwrap();
            }
        });

        b.iter(|| rt.block_on(async { black_box(queue.len().await.unwrap()) }))
    });

    group.bench_function("inmemory_is_empty", |b| {
        let queue = Arc::new(InMemoryJobQueue::new());
        b.iter(|| rt.block_on(async { black_box(queue.is_empty().await.unwrap()) }))
    });

    group.finish();
}

fn worker_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("worker_operations");
    let rt = Runtime::new().unwrap();

    let params = TestParams {
        value: 42,
        message: "test message".to_string(),
    };

    let config = ExecutionConfig::default();

    group.bench_function("worker_creation", |b| {
        b.iter(|| ToolWorker::<InMemoryIdempotencyStore>::new(black_box(config.clone())))
    });

    group.bench_function("register_tool", |b| {
        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(config.clone());
        let tool = Arc::new(TestTool);
        b.iter(|| rt.block_on(async { worker.register_tool(tool.clone()).await }))
    });

    group.bench_function("process_job", |b| {
        let worker = ToolWorker::new(config.clone())
            .with_idempotency_store(Arc::new(InMemoryIdempotencyStore::new()));

        let tool = Arc::new(TestTool);
        rt.block_on(async {
            worker.register_tool(tool).await;
        });

        b.iter(|| {
            rt.block_on(async {
                let job = Job::new("test_tool", &params, 3).unwrap();
                worker.process_job(job).await
            })
        })
    });

    group.bench_function("process_job_with_idempotency", |b| {
        let worker = ToolWorker::new(config.clone())
            .with_idempotency_store(Arc::new(InMemoryIdempotencyStore::new()));

        let tool = Arc::new(TestTool);
        rt.block_on(async {
            worker.register_tool(tool).await;
        });

        b.iter(|| {
            rt.block_on(async {
                let job = Job::new_idempotent(
                    "test_tool",
                    &params,
                    3,
                    format!("key_{}", uuid::Uuid::new_v4()),
                )
                .unwrap();
                worker.process_job(job).await
            })
        })
    });

    group.finish();
}

fn idempotency_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("idempotency");
    let rt = Runtime::new().unwrap();

    group.bench_function("inmemory_get", |b| {
        let store = Arc::new(InMemoryIdempotencyStore::new());
        b.iter(|| rt.block_on(async { store.get(black_box("test_key")).await.unwrap() }))
    });

    group.bench_function("inmemory_set", |b| {
        let store = Arc::new(InMemoryIdempotencyStore::new());
        let result = JobResult::success(&"test").unwrap();
        b.iter(|| {
            rt.block_on(async {
                let key = format!("key_{}", uuid::Uuid::new_v4());
                store
                    .set(black_box(&key), &result, Duration::from_secs(60))
                    .await
                    .unwrap()
            })
        })
    });

    group.bench_function("inmemory_remove", |b| {
        let store = Arc::new(InMemoryIdempotencyStore::new());
        rt.block_on(async {
            for i in 0..100 {
                let result = JobResult::success(&"test").unwrap();
                store
                    .set(&format!("key_{}", i), &result, Duration::from_secs(60))
                    .await
                    .unwrap();
            }
        });

        b.iter(|| rt.block_on(async { store.remove(black_box("key_50")).await.unwrap() }))
    });

    group.finish();
}

fn concurrent_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");
    let rt = Runtime::new().unwrap();

    let params = TestParams {
        value: 42,
        message: "test message".to_string(),
    };

    for num_workers in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_workers),
            num_workers,
            |b, &num_workers| {
                let queue: Arc<dyn JobQueue> = Arc::new(InMemoryJobQueue::new());

                b.iter(|| {
                    rt.block_on(async {
                        let mut handles = vec![];

                        for _ in 0..100 {
                            let job = Job::new("test_tool", &params, 3).unwrap();
                            queue.enqueue(job).await.unwrap();
                        }

                        for _ in 0..num_workers {
                            let queue_clone = queue.clone();
                            let handle = tokio::spawn(async move {
                                for _ in 0..100 / num_workers {
                                    queue_clone
                                        .dequeue_with_timeout(Duration::from_millis(10))
                                        .await
                                        .ok();
                                }
                            });
                            handles.push(handle);
                        }

                        for handle in handles {
                            handle.await.unwrap();
                        }
                    })
                })
            },
        );
    }

    group.finish();
}

fn throughput_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    let rt = Runtime::new().unwrap();

    let params = TestParams {
        value: 42,
        message: "test message".to_string(),
    };

    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let config = ExecutionConfig::default();
            let worker = ToolWorker::new(config)
                .with_idempotency_store(Arc::new(InMemoryIdempotencyStore::new()));

            let tool = Arc::new(TestTool);
            rt.block_on(async {
                worker.register_tool(tool).await;
            });

            b.iter(|| {
                rt.block_on(async {
                    let mut tasks = vec![];
                    for i in 0..size {
                        let job =
                            Job::new_idempotent("test_tool", &params, 3, format!("key_{}", i))
                                .unwrap();
                        tasks.push(worker.process_job(job));
                    }

                    for task in tasks {
                        task.await.ok();
                    }
                })
            })
        });
    }

    group.finish();
}

#[cfg(feature = "redis")]
fn redis_queue_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("redis_queue");
    let rt = Runtime::new().unwrap();

    let redis_url = std::env::var(REDIS_URL).unwrap_or_else(|_| "redis://127.0.0.1/".to_string());

    // Check if Redis is available before running benchmarks
    let redis_available =
        rt.block_on(async { RedisJobQueue::new(&redis_url, "test_connection").is_ok() });

    if !redis_available {
        println!(
            "Redis not available at {}, skipping Redis benchmarks",
            redis_url
        );
        group.finish();
        return;
    }

    let params = TestParams {
        value: 42,
        message: "test message".to_string(),
    };

    group.bench_function("redis_enqueue", |b| {
        let queue = RedisJobQueue::new(&redis_url, "benchmark_queue").ok();

        if let Some(queue) = queue {
            let queue = Arc::new(queue);
            b.iter(|| {
                rt.block_on(async {
                    let job = Job::new("test_tool", &params, 3).unwrap();
                    // Handle Redis enqueue gracefully
                    if let Err(e) = queue.enqueue(black_box(job)).await {
                        eprintln!("Redis enqueue failed: {}", e);
                    }
                })
            })
        }
    });

    group.bench_function("redis_dequeue", |b| {
        let queue = RedisJobQueue::new(&redis_url, "benchmark_queue_dequeue").ok();

        if let Some(queue) = queue {
            let queue = Arc::new(queue);

            // Pre-populate queue with error handling
            rt.block_on(async {
                for _ in 0..1000 {
                    let job = Job::new("test_tool", &params, 3).unwrap();
                    if let Err(e) = queue.enqueue(job).await {
                        eprintln!("Redis enqueue failed during setup: {}", e);
                        return;
                    }
                }
            });

            b.iter(|| {
                rt.block_on(async {
                    // Handle Redis dequeue gracefully
                    match queue.dequeue_with_timeout(Duration::from_millis(100)).await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Redis dequeue failed: {}", e);
                        }
                    }
                })
            })
        }
    });

    group.finish();
}

criterion_group!(
    benches,
    job_creation_benchmarks,
    job_result_benchmarks,
    queue_benchmarks,
    worker_benchmarks,
    idempotency_benchmarks,
    concurrent_benchmarks,
    throughput_benchmarks,
);

#[cfg(feature = "redis")]
criterion_group!(redis_benches, redis_queue_benchmarks);

#[cfg(not(feature = "redis"))]
criterion_main!(benches);

#[cfg(feature = "redis")]
criterion_main!(benches, redis_benches);
