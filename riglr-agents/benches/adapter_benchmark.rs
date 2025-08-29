//! Benchmarks for RigToolAdapter performance optimization

#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};
use rig::tool::Tool as RigTool;
use riglr_agents::adapter::RigToolAdapter;
use riglr_config::Config;
use riglr_core::{provider::ApplicationContext, JobResult, Tool};
use serde_json::json;
use std::{hint::black_box, sync::Arc};
use tokio::runtime::Runtime;

/// A simple mock tool for benchmarking
#[derive(Clone)]
struct MockTool;

#[async_trait::async_trait]
impl Tool for MockTool {
    fn name(&self) -> &'static str {
        "mock_tool"
    }

    fn description(&self) -> &'static str {
        "A mock tool for benchmarking"
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "input": { "type": "string" }
            }
        })
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _context: &ApplicationContext,
    ) -> Result<JobResult, riglr_core::ToolError> {
        // Simulate some minimal work
        tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
        Ok(JobResult::Success {
            value: json!({ "result": "success", "input": args.get("input") }),
            tx_hash: None,
        })
    }
}

/// Benchmarks the performance of RigToolAdapter call operations.
///
/// Tests adapter call performance with different payload sizes:
/// - Without signer context (most common case)
/// - Small, medium, and large JSON payloads
fn benchmark_adapter_call(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();

    // Create the adapter with a mock tool
    let mock_tool = Arc::new(MockTool);
    let config = Config::from_env();
    let context = ApplicationContext::from_config(&config);
    let adapter = RigToolAdapter::new(mock_tool, context);

    // Benchmark without SignerContext (most common case)
    c.bench_function("adapter_call_without_signer", |b| {
        b.to_async(&runtime).iter(|| async {
            let args = json!({ "input": "test" });
            let result = RigTool::call(&adapter, black_box(args)).await;
            black_box(result)
        });
    });

    // Benchmark with varying payload sizes
    let small_payload = json!({ "input": "a" });
    let medium_payload = json!({ "input": "a".repeat(100) });
    let large_payload = json!({ "input": "a".repeat(1000) });

    c.bench_function("adapter_call_small_payload", |b| {
        b.to_async(&runtime).iter(|| async {
            let result = RigTool::call(&adapter, black_box(small_payload.clone())).await;
            black_box(result)
        });
    });

    c.bench_function("adapter_call_medium_payload", |b| {
        b.to_async(&runtime).iter(|| async {
            let result = RigTool::call(&adapter, black_box(medium_payload.clone())).await;
            black_box(result)
        });
    });

    c.bench_function("adapter_call_large_payload", |b| {
        b.to_async(&runtime).iter(|| async {
            let result = RigTool::call(&adapter, black_box(large_payload.clone())).await;
            black_box(result)
        });
    });
}

/// Benchmarks the performance of RigToolAdapter definition operations.
///
/// Tests the speed of generating tool definitions for the rig framework.
fn benchmark_adapter_definition(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();

    let mock_tool = Arc::new(MockTool);
    let config = Config::from_env();
    let context = ApplicationContext::from_config(&config);
    let adapter = RigToolAdapter::new(mock_tool, context);

    c.bench_function("adapter_definition", |b| {
        b.to_async(&runtime).iter(|| async {
            let definition =
                RigTool::definition(&adapter, black_box("test prompt".to_string())).await;
            black_box(definition)
        });
    });
}

criterion_group!(
    benches,
    benchmark_adapter_call,
    benchmark_adapter_definition
);
criterion_main!(benches);
