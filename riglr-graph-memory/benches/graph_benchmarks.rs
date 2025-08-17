//! Performance benchmarks for riglr-graph-memory functionality.
//!
//! This module contains comprehensive benchmarks for testing the performance
//! of graph memory operations including document creation, entity extraction,
//! error handling, serialization, and throughput scenarios.

#![allow(missing_docs)]

use chrono::Utc;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use riglr_graph_memory::{
    document::{DocumentMetadata, DocumentSource, RawTextDocument},
    error::GraphMemoryError,
    extractor::EntityExtractor,
};
use serde_json::json;
use std::hint::black_box;
use uuid::Uuid;

fn document_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("document_operations");

    group.bench_function("document_creation", |b| {
        b.iter(|| RawTextDocument {
            id: Uuid::new_v4().to_string(),
            content: "This is a sample document for benchmarking purposes.".to_string(),
            metadata: Some(DocumentMetadata {
                title: Some("benchmark".to_string()),
                tags: vec!["test".to_string(), "benchmark".to_string()],
                chain: Some("ethereum".to_string()),
                block_number: Some(12345),
                transaction_hash: Some("0x123".to_string()),
                wallet_addresses: vec!["0xabc".to_string()],
                token_addresses: vec!["0xdef".to_string()],
                protocols: vec!["uniswap".to_string()],
                extraction_confidence: Some(0.9),
                custom_fields: std::collections::HashMap::new(),
            }),
            embedding: None,
            created_at: Utc::now(),
            source: DocumentSource::UserInput,
        })
    });

    group.bench_function("metadata_creation", |b| {
        b.iter(|| {
            let mut metadata = DocumentMetadata {
                title: Some("test_source".to_string()),
                tags: vec!["tag1".to_string(), "tag2".to_string()],
                chain: Some("ethereum".to_string()),
                block_number: Some(12345),
                transaction_hash: Some("0x123".to_string()),
                wallet_addresses: vec!["0xabc".to_string()],
                token_addresses: vec!["0xdef".to_string()],
                protocols: vec!["uniswap".to_string()],
                extraction_confidence: Some(0.9),
                custom_fields: std::collections::HashMap::new(),
            };
            metadata
                .custom_fields
                .insert("key1".to_string(), json!("value1"));
            metadata.custom_fields.insert("key2".to_string(), json!(42));
            black_box(metadata)
        })
    });

    group.finish();
}

fn entity_extractor_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_extraction");

    group.bench_function("extractor_creation", |b| {
        b.iter(|| {
            let _extractor = EntityExtractor::default();
            black_box(_extractor)
        })
    });

    group.bench_function("pattern_matching", |b| {
        let text = "0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B swapped 100 USDC for ETH on Uniswap";

        b.iter(|| {
            let patterns = vec![
                r"0x[a-fA-F0-9]{40}", // Ethereum addresses
                r"\b\d+\.\d+\b",      // Decimal numbers
                r"\b[A-Z]{2,5}\b",    // Token symbols
            ];

            for pattern in &patterns {
                let regex = regex::Regex::new(pattern).unwrap();
                let matches: Vec<_> = regex.find_iter(black_box(text)).collect();
                black_box(matches);
            }
        })
    });

    group.finish();
}

fn error_handling_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    group.bench_function("error_database_error", |b| {
        b.iter(|| GraphMemoryError::Database(black_box("Connection failed".to_string())))
    });

    group.bench_function("error_embedding_error", |b| {
        b.iter(|| {
            GraphMemoryError::Embedding(black_box("Failed to generate embeddings".to_string()))
        })
    });

    group.bench_function("error_generic", |b| {
        b.iter(|| GraphMemoryError::Generic(black_box("Document not found".to_string())))
    });

    group.bench_function("error_display", |b| {
        let error = GraphMemoryError::Database("test".to_string());
        b.iter(|| format!("{}", black_box(&error)))
    });

    group.finish();
}

fn serialization_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    let test_data = vec![
        json!({"id": "123", "type": "document"}),
        json!({"nodes": [1, 2, 3], "edges": [[1, 2], [2, 3]]}),
        json!({"embeddings": vec![0.1; 100]}),
        json!({"metadata": {"key": "value", "count": 42}}),
    ];

    group.bench_function("json_serialize", |b| {
        b.iter(|| {
            for data in &test_data {
                let _serialized = serde_json::to_string(black_box(data));
            }
        })
    });

    group.bench_function("json_deserialize", |b| {
        let json_strings: Vec<String> = test_data
            .iter()
            .map(|d| serde_json::to_string(d).unwrap())
            .collect();

        b.iter(|| {
            for json_str in &json_strings {
                let _deserialized: serde_json::Value =
                    serde_json::from_str(black_box(json_str)).unwrap();
            }
        })
    });

    group.finish();
}

fn throughput_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(
            BenchmarkId::new("create_documents", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let docs: Vec<RawTextDocument> = (0..size)
                        .map(|i| RawTextDocument {
                            id: format!("doc_{}", i),
                            content: format!("Document content {}", i),
                            metadata: Some(DocumentMetadata {
                                title: Some("benchmark".to_string()),
                                tags: vec![],
                                chain: Some("ethereum".to_string()),
                                block_number: Some(12345),
                                transaction_hash: Some("0x123".to_string()),
                                wallet_addresses: vec!["0xabc".to_string()],
                                token_addresses: vec!["0xdef".to_string()],
                                protocols: vec!["uniswap".to_string()],
                                extraction_confidence: Some(0.9),
                                custom_fields: std::collections::HashMap::new(),
                            }),
                            embedding: None,
                            created_at: Utc::now(),
                            source: DocumentSource::UserInput,
                        })
                        .collect();
                    black_box(docs)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("similarity_search", size),
            size,
            |b, &size| {
                let vectors: Vec<Vec<f32>> = (0..size)
                    .map(|i| {
                        (0..100)
                            .map(|j| (i * j) as f32 / (size * 100) as f32)
                            .collect()
                    })
                    .collect();
                let query = vec![0.5; 100];

                b.iter(|| {
                    let mut similarities: Vec<(usize, f32)> = vectors
                        .iter()
                        .enumerate()
                        .map(|(idx, vec)| {
                            let dot: f32 = vec.iter().zip(&query).map(|(a, b)| a * b).sum();
                            let norm1: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
                            let norm2: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
                            (idx, dot / (norm1 * norm2))
                        })
                        .collect();
                    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                    let top_k: Vec<_> = similarities.into_iter().take(10).collect();
                    black_box(top_k)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    document_benchmarks,
    entity_extractor_benchmarks,
    error_handling_benchmarks,
    serialization_benchmarks,
    throughput_benchmarks,
);

criterion_main!(benches);
