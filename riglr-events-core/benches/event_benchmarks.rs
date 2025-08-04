use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use riglr_events_core::prelude::*;
use serde_json::json;
use std::hint::black_box;
use std::time::Duration;
use tokio::runtime::Runtime;

fn bench_event_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_creation");

    for size in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("generic_event", size), size, |b, &size| {
            b.iter(|| {
                let mut events = Vec::with_capacity(size);
                for i in 0..size {
                    let event = GenericEvent::new(
                        format!("event_{}", i),
                        EventKind::Transaction,
                        json!({"amount": i, "token": "USDC"}),
                    );
                    events.push(black_box(event));
                }
                events
            });
        });
    }

    group.finish();
}

fn bench_event_filtering(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_filtering");

    // Create test events
    let events: Vec<Box<dyn Event>> = (0..1000)
        .map(|i| {
            Box::new(GenericEvent::new(
                format!("event_{}", i),
                if i % 2 == 0 {
                    EventKind::Transaction
                } else {
                    EventKind::Block
                },
                json!({"index": i}),
            )) as Box<dyn Event>
        })
        .collect();

    let filter = KindFilter::single(EventKind::Transaction);

    group.throughput(Throughput::Elements(events.len() as u64));
    group.bench_function("kind_filter", |b| {
        b.iter(|| {
            let matched: Vec<_> = events
                .iter()
                .filter(|event| filter.matches(event.as_ref()))
                .collect();
            black_box(matched)
        });
    });

    group.finish();
}

fn bench_event_batching(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_batching");

    for batch_size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("batcher", batch_size),
            batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    let mut batcher = EventBatcher::new(batch_size, Duration::from_secs(10));
                    let mut batches = Vec::new();

                    for i in 0..(batch_size * 3) {
                        let event = Box::new(GenericEvent::new(
                            format!("event_{}", i),
                            EventKind::Transaction,
                            json!({"index": i}),
                        )) as Box<dyn Event>;

                        if let Some(batch) = batcher.add(event) {
                            batches.push(batch);
                        }
                    }

                    // Flush remaining
                    if let Some(batch) = batcher.flush() {
                        batches.push(batch);
                    }

                    black_box(batches)
                });
            },
        );
    }

    group.finish();
}

fn bench_parsing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("parsing");

    // Create a JSON parser configuration
    let config = ParsingConfig::new(
        "benchmark-parser".to_string(),
        "1.0.0".to_string(),
        "json".to_string(),
    )
    .with_mapping("amount".to_string(), "transaction.amount".to_string())
    .with_mapping("token".to_string(), "transaction.token".to_string())
    .with_default("fee".to_string(), 0.01);

    let parser = JsonEventParser::new(config, EventKind::Transaction, "benchmark".to_string());

    let test_data = json!({
        "transaction": {
            "amount": 1000,
            "token": "USDC"
        },
        "timestamp": "2023-01-01T00:00:00Z"
    });

    group.bench_function("json_parser", |b| {
        b.to_async(&rt).iter(|| async {
            let events = parser.parse(black_box(test_data.clone())).await.unwrap();
            black_box(events)
        });
    });

    group.finish();
}

fn bench_deduplication(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("deduplication");

    let deduplicator = EventDeduplicator::new(Duration::from_secs(300), Duration::from_secs(60));

    let events: Vec<Box<dyn Event>> = (0..1000)
        .map(|i| {
            Box::new(GenericEvent::new(
                format!("event_{}", i % 100), // Create duplicates
                EventKind::Transaction,
                json!({"index": i}),
            )) as Box<dyn Event>
        })
        .collect();

    group.throughput(Throughput::Elements(events.len() as u64));
    group.bench_function("check_duplicates", |b| {
        b.to_async(&rt).iter(|| async {
            let mut unique_count = 0;
            for event in &events {
                if !deduplicator.is_duplicate(event.as_ref()).await {
                    deduplicator.mark_seen(event.as_ref()).await;
                    unique_count += 1;
                }
            }
            black_box(unique_count)
        });
    });

    group.finish();
}

fn bench_rate_limiting(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("rate_limiting");

    for rate in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("rate_limiter", rate), rate, |b, &rate| {
            let rate_limiter = RateLimiter::new(rate, Duration::from_secs(1));

            b.to_async(&rt).iter(|| async {
                let mut processed = 0;
                for _ in 0..rate {
                    if rate_limiter.can_process().await {
                        rate_limiter.record_event().await;
                        processed += 1;
                    }
                }
                black_box(processed)
            });
        });
    }

    group.finish();
}

fn bench_metrics_collection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("metrics_collection");

    let metrics = EventPerformanceMetrics::new();

    group.bench_function("record_metrics", |b| {
        b.to_async(&rt).iter(|| async {
            for i in 0..1000 {
                let duration = Duration::from_micros(i % 10000);
                metrics.record_processing_time(duration).await;

                if i % 10 == 0 {
                    metrics.record_error();
                }
            }

            let summary = metrics.summary().await;
            black_box(summary)
        });
    });

    group.finish();
}

fn bench_id_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("id_generation");

    let generator = EventIdGenerator::new("benchmark".to_string());

    group.bench_function("generate_ids", |b| {
        b.iter(|| {
            let mut ids = Vec::with_capacity(1000);
            for _ in 0..1000 {
                ids.push(generator.next());
            }
            black_box(ids)
        });
    });

    group.bench_function("generate_ids_with_context", |b| {
        b.iter(|| {
            let mut ids = Vec::with_capacity(1000);
            for i in 0..1000 {
                ids.push(generator.next_with_context(&format!("context_{}", i)));
            }
            black_box(ids)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_event_creation,
    bench_event_filtering,
    bench_event_batching,
    bench_parsing,
    bench_deduplication,
    bench_rate_limiting,
    bench_metrics_collection,
    bench_id_generation
);
criterion_main!(benches);
