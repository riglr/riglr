use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use riglr_graph_memory::{
    document::{Document, DocumentChunk, DocumentMetadata},
    error::GraphMemoryError,
    extractor::{Entity, EntityExtractor, EntityType},
    graph::{Edge, GraphMemory, Node, RelationType},
    vector_store::{InMemoryVectorStore, SimilarityMetric},
};
use serde_json::json;
use std::sync::Arc;
use tokio::runtime::Runtime;
use uuid::Uuid;

fn document_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("document_operations");

    group.bench_function("document_creation", |b| {
        b.iter(|| Document {
            id: Uuid::new_v4().to_string(),
            content: "This is a sample document for benchmarking purposes.".to_string(),
            metadata: DocumentMetadata {
                source: Some("benchmark".to_string()),
                timestamp: Utc::now(),
                tags: vec!["test".to_string(), "benchmark".to_string()],
                custom: std::collections::HashMap::new(),
            },
            embeddings: None,
            chunks: vec![],
        })
    });

    group.bench_function("document_chunking", |b| {
        let content = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(100);
        b.iter(|| {
            let chunks: Vec<String> = content.split(". ").map(|s| s.to_string()).collect();
            black_box(chunks)
        })
    });

    group.bench_function("chunk_creation", |b| {
        b.iter(|| DocumentChunk {
            id: Uuid::new_v4().to_string(),
            document_id: Uuid::new_v4().to_string(),
            content: "This is a chunk of text from a larger document.".to_string(),
            position: 0,
            embeddings: None,
        })
    });

    group.bench_function("metadata_creation", |b| {
        b.iter(|| {
            let mut metadata = DocumentMetadata {
                source: Some("test_source".to_string()),
                timestamp: Utc::now(),
                tags: vec!["tag1".to_string(), "tag2".to_string()],
                custom: std::collections::HashMap::new(),
            };
            metadata.custom.insert("key1".to_string(), json!("value1"));
            metadata.custom.insert("key2".to_string(), json!(42));
            black_box(metadata)
        })
    });

    group.finish();
}

fn graph_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_operations");
    let rt = Runtime::new().unwrap();

    group.bench_function("node_creation", |b| {
        b.iter(|| Node {
            id: Uuid::new_v4().to_string(),
            node_type: "Document".to_string(),
            properties: std::collections::HashMap::new(),
            embeddings: None,
        })
    });

    group.bench_function("edge_creation", |b| {
        b.iter(|| Edge {
            id: Uuid::new_v4().to_string(),
            source_id: Uuid::new_v4().to_string(),
            target_id: Uuid::new_v4().to_string(),
            relation_type: RelationType::References,
            properties: std::collections::HashMap::new(),
            weight: Some(1.0),
        })
    });

    group.bench_function("graph_memory_creation", |b| {
        b.iter(|| rt.block_on(async { GraphMemory::new() }))
    });

    group.bench_function("relation_type_parsing", |b| {
        let relations = vec![
            "references",
            "mentions",
            "contains",
            "relates_to",
            "derived_from",
        ];

        b.iter(|| {
            for rel in &relations {
                let _parsed = match *rel {
                    "references" => RelationType::References,
                    "mentions" => RelationType::Mentions,
                    "contains" => RelationType::Contains,
                    "relates_to" => RelationType::RelatesTo,
                    "derived_from" => RelationType::DerivedFrom,
                    _ => RelationType::Custom(rel.to_string()),
                };
                black_box(_parsed);
            }
        })
    });

    group.finish();
}

fn vector_store_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_store");
    let rt = Runtime::new().unwrap();

    group.bench_function("store_creation", |b| {
        b.iter(|| InMemoryVectorStore::new(black_box(384)))
    });

    group.bench_function("similarity_cosine", |b| {
        let vec1: Vec<f32> = (0..100).map(|i| i as f32 / 100.0).collect();
        let vec2: Vec<f32> = (0..100).map(|i| (i as f32 / 100.0) * 0.9).collect();

        b.iter(|| {
            let dot: f32 = vec1.iter().zip(&vec2).map(|(a, b)| a * b).sum();
            let norm1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
            let norm2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();
            let similarity = dot / (norm1 * norm2);
            black_box(similarity)
        })
    });

    group.bench_function("similarity_euclidean", |b| {
        let vec1: Vec<f32> = (0..100).map(|i| i as f32 / 100.0).collect();
        let vec2: Vec<f32> = (0..100).map(|i| (i as f32 / 100.0) * 0.9).collect();

        b.iter(|| {
            let distance: f32 = vec1
                .iter()
                .zip(&vec2)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f32>()
                .sqrt();
            black_box(distance)
        })
    });

    group.bench_function("add_document", |b| {
        let store = Arc::new(InMemoryVectorStore::new(384));
        let rt_clone = Runtime::new().unwrap();

        b.iter(|| {
            rt_clone.block_on(async {
                let doc = Document {
                    id: Uuid::new_v4().to_string(),
                    content: "Test document".to_string(),
                    metadata: DocumentMetadata {
                        source: None,
                        timestamp: Utc::now(),
                        tags: vec![],
                        custom: std::collections::HashMap::new(),
                    },
                    embeddings: Some(vec![0.1; 384]),
                    chunks: vec![],
                };
                store.add_document(doc).await
            })
        })
    });

    group.finish();
}

fn entity_extractor_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_extraction");

    group.bench_function("entity_creation", |b| {
        b.iter(|| Entity {
            id: Uuid::new_v4().to_string(),
            name: "Test Entity".to_string(),
            entity_type: EntityType::Person,
            confidence: 0.95,
            context: Some("This is the context for the entity".to_string()),
            metadata: std::collections::HashMap::new(),
        })
    });

    group.bench_function("entity_type_parsing", |b| {
        let types = vec![
            "person",
            "organization",
            "location",
            "date",
            "concept",
            "custom_type",
        ];

        b.iter(|| {
            for type_str in &types {
                let _parsed = match *type_str {
                    "person" => EntityType::Person,
                    "organization" => EntityType::Organization,
                    "location" => EntityType::Location,
                    "date" => EntityType::Date,
                    "concept" => EntityType::Concept,
                    _ => EntityType::Other(type_str.to_string()),
                };
                black_box(_parsed);
            }
        })
    });

    group.bench_function("pattern_matching", |b| {
        let text = "John Smith works at OpenAI in San Francisco since 2023.";
        let patterns = vec![
            r"\b[A-Z][a-z]+ [A-Z][a-z]+\b", // Names
            r"\b[A-Z][a-z]+\b",             // Proper nouns
            r"\b\d{4}\b",                   // Years
        ];

        b.iter(|| {
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
        b.iter(|| GraphMemoryError::DatabaseError(black_box("Connection failed".to_string())))
    });

    group.bench_function("error_embedding_error", |b| {
        b.iter(|| {
            GraphMemoryError::EmbeddingError(black_box("Failed to generate embeddings".to_string()))
        })
    });

    group.bench_function("error_not_found", |b| {
        b.iter(|| GraphMemoryError::NotFound(black_box("Document not found".to_string())))
    });

    group.bench_function("error_display", |b| {
        let error = GraphMemoryError::DatabaseError("test".to_string());
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
    let rt = Runtime::new().unwrap();

    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(
            BenchmarkId::new("create_documents", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let docs: Vec<Document> = (0..size)
                        .map(|i| Document {
                            id: format!("doc_{}", i),
                            content: format!("Document content {}", i),
                            metadata: DocumentMetadata {
                                source: Some("benchmark".to_string()),
                                timestamp: Utc::now(),
                                tags: vec![],
                                custom: std::collections::HashMap::new(),
                            },
                            embeddings: None,
                            chunks: vec![],
                        })
                        .collect();
                    black_box(docs)
                })
            },
        );

        group.bench_with_input(BenchmarkId::new("create_nodes", size), size, |b, &size| {
            b.iter(|| {
                let nodes: Vec<Node> = (0..size)
                    .map(|i| Node {
                        id: format!("node_{}", i),
                        node_type: "Document".to_string(),
                        properties: std::collections::HashMap::new(),
                        embeddings: None,
                    })
                    .collect();
                black_box(nodes)
            })
        });

        group.bench_with_input(
            BenchmarkId::new("similarity_search", size),
            size,
            |b, &size| {
                let vectors: Vec<Vec<f32>> = (0..size)
                    .map(|i| {
                        (0..100)
                            .map(|j| ((i * j) as f32 / (size * 100) as f32))
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

fn concurrent_operations_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");
    let rt = Runtime::new().unwrap();

    for num_tasks in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_tasks),
            num_tasks,
            |b, &num_tasks| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut handles = vec![];

                        for _ in 0..num_tasks {
                            let handle = tokio::spawn(async move {
                                for i in 0..100 {
                                    let _doc = Document {
                                        id: format!("doc_{}", i),
                                        content: format!("Content {}", i),
                                        metadata: DocumentMetadata {
                                            source: None,
                                            timestamp: Utc::now(),
                                            tags: vec![],
                                            custom: std::collections::HashMap::new(),
                                        },
                                        embeddings: None,
                                        chunks: vec![],
                                    };
                                    tokio::task::yield_now().await;
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

criterion_group!(
    benches,
    document_benchmarks,
    graph_benchmarks,
    vector_store_benchmarks,
    entity_extractor_benchmarks,
    error_handling_benchmarks,
    serialization_benchmarks,
    throughput_benchmarks,
    concurrent_operations_benchmarks,
);

criterion_main!(benches);
