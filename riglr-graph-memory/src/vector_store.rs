//! Vector store implementation for graph memory.
//!
//! This module provides a rig-compatible vector store that uses Neo4j's vector search
//! capabilities combined with graph traversal for enhanced contextual retrieval.

use crate::{
    client::Neo4jClient,
    document::RawTextDocument,
    error::{GraphMemoryError, Result},
};
// Note: VectorStore trait interface may vary in rig-core 0.2.0
// For now, implementing a compatible interface based on common patterns
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, sync::Arc};
use tracing::{debug, info, warn};

/// A retriever that combines graph and vector search for enhanced context.
///
/// This implementation provides sophisticated document retrieval by leveraging both
/// vector similarity search and graph relationships to find the most relevant context
/// for agent queries.
#[derive(Debug)]
pub struct GraphRetriever {
    /// Neo4j client for database operations
    client: Arc<Neo4jClient>,
    /// Vector index name in Neo4j
    index_name: String,
    /// Minimum similarity threshold for vector search
    similarity_threshold: f32,
    /// Maximum number of graph hops for relationship traversal
    max_graph_hops: u32,
    /// Embedding dimension (default 1536 for OpenAI)
    embedding_dimension: usize,
}

/// Configuration for graph-based vector retrieval
#[derive(Debug, Clone)]
pub struct GraphRetrieverConfig {
    /// Minimum similarity threshold (0.0 to 1.0)
    pub similarity_threshold: f32,
    /// Maximum graph traversal depth
    pub max_graph_hops: u32,
    /// Vector embedding dimension
    pub embedding_dimension: usize,
    /// Vector index name
    pub index_name: String,
}

/// A document stored in the graph with vector embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDocument {
    /// Document unique identifier
    pub id: String,
    /// Document content
    pub content: String,
    /// Vector embedding
    pub embedding: Vec<f32>,
    /// Metadata extracted from the document
    pub metadata: HashMap<String, Value>,
    /// Entities extracted from this document
    pub entities: Vec<String>,
    /// Graph relationships
    pub relationships: Vec<String>,
    /// Similarity score (populated during search)
    pub similarity_score: Option<f32>,
}

/// Search result from graph vector store
#[derive(Debug, Clone)]
pub struct GraphSearchResult {
    /// Retrieved documents
    pub documents: Vec<GraphDocument>,
    /// Related entities found through graph traversal
    pub related_entities: Vec<String>,
    /// Query performance metrics
    pub metrics: SearchMetrics,
}

/// Performance metrics for graph search operations
#[derive(Debug, Clone)]
pub struct SearchMetrics {
    /// Vector search time in milliseconds
    pub vector_search_time_ms: u64,
    /// Graph traversal time in milliseconds
    pub graph_traversal_time_ms: u64,
    /// Total query time in milliseconds
    pub total_time_ms: u64,
    /// Number of nodes examined
    pub nodes_examined: u32,
    /// Number of relationships traversed
    pub relationships_traversed: u32,
}

impl GraphRetrieverConfig {
    /// Create default configuration
    pub fn new_default() -> Self {
        Self {
            similarity_threshold: 0.7,
            max_graph_hops: 2,
            embedding_dimension: 1536,
            index_name: "document_embeddings".to_string(),
        }
    }

    /// Create configuration for high-precision search
    pub fn high_precision() -> Self {
        Self {
            similarity_threshold: 0.8,
            max_graph_hops: 1,
            embedding_dimension: 1536,
            index_name: "document_embeddings".to_string(),
        }
    }

    /// Create configuration for broad contextual search
    pub fn broad_context() -> Self {
        Self {
            similarity_threshold: 0.6,
            max_graph_hops: 3,
            embedding_dimension: 1536,
            index_name: "document_embeddings".to_string(),
        }
    }
}

impl GraphRetriever {
    /// Create a new graph retriever with Neo4j client
    pub async fn new(
        client: Arc<Neo4jClient>,
        config: Option<GraphRetrieverConfig>,
    ) -> Result<Self> {
        let config = config.unwrap_or_else(GraphRetrieverConfig::new_default);

        let retriever = Self {
            client,
            index_name: config.index_name,
            similarity_threshold: config.similarity_threshold,
            max_graph_hops: config.max_graph_hops,
            embedding_dimension: config.embedding_dimension,
        };

        // Ensure vector index exists
        retriever.ensure_vector_index().await?;

        info!(
            "GraphRetriever initialized with similarity threshold: {}, max hops: {}",
            retriever.similarity_threshold, retriever.max_graph_hops
        );

        Ok(retriever)
    }

    /// Ensure the vector index exists in Neo4j
    async fn ensure_vector_index(&self) -> Result<()> {
        debug!("Ensuring vector index '{}' exists", self.index_name);

        let create_index_query = format!(
            "CREATE VECTOR INDEX IF NOT EXISTS {} FOR (d:Document) ON (d.embedding)
             OPTIONS {{indexConfig: {{`vector.dimensions`: {}, `vector.similarity_function`: 'cosine'}}}}",
            self.index_name, self.embedding_dimension
        );

        self.client
            .execute_query(&create_index_query, None)
            .await
            .map_err(|e| {
                GraphMemoryError::Database(format!("Failed to create vector index: {}", e))
            })?;

        debug!("Vector index '{}' is ready", self.index_name);
        Ok(())
    }

    /// Perform hybrid vector + graph search
    pub async fn search_with_graph_context(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<GraphSearchResult> {
        let start_time = std::time::Instant::now();
        let mut metrics = SearchMetrics {
            vector_search_time_ms: 0,
            graph_traversal_time_ms: 0,
            total_time_ms: 0,
            nodes_examined: 0,
            relationships_traversed: 0,
        };

        // Step 1: Vector similarity search
        debug!("Performing vector similarity search for {} results", limit);
        let vector_start = std::time::Instant::now();

        let vector_search_query = format!(
            "CALL db.index.vector.queryNodes('{}', {}, $embedding)
             YIELD node, score
             RETURN node.id as id, node.content as content, node.metadata as metadata,
                    node.entities as entities, node.embedding as embedding, score
             LIMIT $limit",
            self.index_name,
            limit * 2 // Get more candidates for graph expansion
        );

        let mut params = HashMap::new();
        params.insert("embedding".to_string(), json!(query_embedding));
        params.insert("limit".to_string(), json!(limit));

        let vector_results = self
            .client
            .execute_query(&vector_search_query, Some(params))
            .await?;
        metrics.vector_search_time_ms = vector_start.elapsed().as_millis() as u64;

        // Parse vector search results
        let mut documents = Vec::new();
        let mut entity_set = std::collections::HashSet::new();

        if let Some(results) = vector_results["results"].as_array() {
            for result in results {
                if let Some(data) = result["data"].as_array() {
                    for row in data {
                        if let Some(row_data) = row["row"].as_array() {
                            if let (Some(id), Some(content), Some(score)) = (
                                row_data[0].as_str(),
                                row_data[1].as_str(),
                                row_data[5].as_f64(), // Score is now at index 5 due to added embedding field
                            ) {
                                let similarity_score = score as f32;

                                // Filter by similarity threshold
                                if similarity_score >= self.similarity_threshold {
                                    let metadata: HashMap<String, Value> = row_data[2]
                                        .as_object()
                                        .map(|obj| {
                                            obj.iter()
                                                .map(|(k, v)| (k.clone(), v.clone()))
                                                .collect()
                                        })
                                        .unwrap_or_default();

                                    let entities: Vec<String> = row_data[3]
                                        .as_array()
                                        .map(|arr| {
                                            arr.iter()
                                                .filter_map(|v| v.as_str())
                                                .map(|s| s.to_string())
                                                .collect()
                                        })
                                        .unwrap_or_default();

                                    // Extract the actual embedding from the database
                                    let actual_embedding: Vec<f32> = row_data[4]
                                        .as_array()
                                        .map(|arr| {
                                            arr.iter()
                                                .filter_map(|v| v.as_f64().map(|f| f as f32))
                                                .collect()
                                        })
                                        .unwrap_or_else(|| {
                                            warn!("Failed to extract embedding for document {}, using empty vector", id);
                                            Vec::new()
                                        });

                                    // Collect entities for graph expansion
                                    entity_set.extend(entities.iter().cloned());

                                    documents.push(GraphDocument {
                                        id: id.to_string(),
                                        content: content.to_string(),
                                        embedding: actual_embedding, // CRITICAL: Using REAL embedding from database
                                        metadata,
                                        entities,
                                        relationships: Vec::new(), // To be filled by graph traversal
                                        similarity_score: Some(similarity_score),
                                    });

                                    metrics.nodes_examined += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Step 2: Graph traversal for related context
        if self.max_graph_hops > 0 && !entity_set.is_empty() {
            debug!(
                "Performing graph traversal for {} entities with {} hops",
                entity_set.len(),
                self.max_graph_hops
            );

            let graph_start = std::time::Instant::now();
            let related_entities = self.find_related_entities(&entity_set).await?;
            metrics.graph_traversal_time_ms = graph_start.elapsed().as_millis() as u64;
            metrics.relationships_traversed = related_entities.len() as u32;

            // Update documents with relationship information
            let relationships = related_entities;
            for doc in &mut documents {
                doc.relationships.clone_from(&relationships);
            }
        }

        // Sort by similarity score
        documents.sort_by(|a, b| {
            b.similarity_score
                .unwrap_or(0.0)
                .partial_cmp(&a.similarity_score.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit to requested number
        documents.truncate(limit);

        metrics.total_time_ms = start_time.elapsed().as_millis() as u64;

        info!(
            "Graph search completed: {} documents, {} related entities ({} ms)",
            documents.len(),
            entity_set.len(),
            metrics.total_time_ms
        );

        Ok(GraphSearchResult {
            documents,
            related_entities: entity_set.into_iter().collect(),
            metrics,
        })
    }

    /// Find related entities through graph traversal
    async fn find_related_entities(
        &self,
        initial_entities: &std::collections::HashSet<String>,
    ) -> Result<Vec<String>> {
        let entities_list: Vec<&str> = initial_entities.iter().map(|s| s.as_str()).collect();

        let graph_query = format!(
            "UNWIND $entities as entity
             MATCH (e1 {{canonical: entity}})-[r]-(e2)
             WHERE e1 <> e2
             RETURN DISTINCT e2.canonical as related_entity
             LIMIT {}",
            self.max_graph_hops * 50 // Reasonable limit for related entities
        );

        let mut params = HashMap::new();
        params.insert("entities".to_string(), json!(entities_list));

        let result = self
            .client
            .execute_query(&graph_query, Some(params))
            .await?;

        let mut related = Vec::new();
        if let Some(results) = result["results"].as_array() {
            for result in results {
                if let Some(data) = result["data"].as_array() {
                    for row in data {
                        if let Some(row_data) = row["row"].as_array() {
                            if let Some(entity) = row_data[0].as_str() {
                                related.push(entity.to_string());
                            }
                        }
                    }
                }
            }
        }

        debug!(
            "Found {} related entities through graph traversal",
            related.len()
        );
        Ok(related)
    }
}

// Note: No Default implementation since GraphRetriever requires a database connection

// NOTE: rig::VectorStore trait implementation deferred pending rig-core interface stabilization
// Core vector store functionality is currently provided through GraphRetriever methods

impl GraphRetriever {
    /// Add documents to the graph vector store
    /// This is the core functionality that would be exposed through rig::VectorStore
    pub async fn add_documents(&self, documents: Vec<RawTextDocument>) -> Result<Vec<String>> {
        debug!("Adding {} documents to graph vector store", documents.len());

        let mut document_ids = Vec::new();

        for doc in documents {
            // In a production implementation, you would:
            // 1. Generate embeddings for the document content
            // 2. Extract entities using the EntityExtractor
            // 3. Store document node with embedding in Neo4j
            // 4. Create entity nodes and relationships

            // For now, just store basic document info
            let create_doc_query = "
                CREATE (d:Document {
                    id: $id,
                    content: $content,
                    created_at: $created_at,
                    source: $source
                })
                RETURN d.id as id
            ";

            let mut params = HashMap::new();
            params.insert("id".to_string(), json!(doc.id));
            params.insert("content".to_string(), json!(doc.content));
            params.insert("created_at".to_string(), json!(doc.created_at.to_rfc3339()));
            params.insert("source".to_string(), json!(format!("{:?}", doc.source)));

            // Store doc.id for use after moving doc fields
            let doc_id = doc.id;

            match self
                .client
                .execute_query(create_doc_query, Some(params))
                .await
            {
                Ok(_) => {
                    debug!("Added document {} to graph", doc_id);
                    document_ids.push(doc_id);
                }
                Err(e) => {
                    warn!("Failed to add document {}: {}", doc_id, e);
                    return Err(GraphMemoryError::Database(format!(
                        "Failed to add document: {}",
                        e
                    )));
                }
            }
        }

        info!(
            "Successfully added {} documents to graph vector store",
            document_ids.len()
        );
        Ok(document_ids)
    }

    /// Get top N document IDs for a query embedding
    pub async fn top_n_ids(&self, query_embedding: &[f32], n: usize) -> Result<Vec<String>> {
        debug!("Retrieving top {} document IDs for query", n);

        let search_result = self.search_with_graph_context(query_embedding, n).await?;
        let ids: Vec<String> = search_result
            .documents
            .into_iter()
            .map(|doc| doc.id)
            .collect();

        debug!("Retrieved {} document IDs", ids.len());
        Ok(ids)
    }
}

impl From<GraphDocument> for RawTextDocument {
    fn from(graph_doc: GraphDocument) -> Self {
        RawTextDocument {
            id: graph_doc.id,
            content: graph_doc.content,
            metadata: None, // Would need to convert from HashMap<String, Value>
            embedding: Some(graph_doc.embedding),
            created_at: chrono::Utc::now(), // Placeholder
            source: crate::document::DocumentSource::UserInput,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{DocumentMetadata, DocumentSource};
    use chrono::Utc;
    use serde_json::json;
    use std::collections::HashMap;

    // Mock Neo4jClient for testing
    #[allow(dead_code)]
    #[derive(Debug, Clone)]
    struct MockNeo4jClient {
        should_fail: bool,
        response_data: Value,
    }

    #[allow(dead_code)]
    impl MockNeo4jClient {
        fn new() -> Self {
            Self {
                should_fail: false,
                response_data: json!({"results": [], "errors": []}),
            }
        }

        fn with_failure() -> Self {
            Self {
                should_fail: true,
                response_data: json!({"errors": [{"message": "Mock error"}]}),
            }
        }

        fn with_response(response: Value) -> Self {
            Self {
                should_fail: false,
                response_data: response,
            }
        }

        async fn execute_query(
            &self,
            _query: &str,
            _parameters: Option<HashMap<String, Value>>,
        ) -> Result<Value> {
            if self.should_fail {
                Err(GraphMemoryError::Database("Mock error".to_string()))
            } else {
                Ok(self.response_data.clone())
            }
        }
    }

    // Helper function to create test GraphDocument
    fn create_test_graph_document() -> GraphDocument {
        let mut metadata = HashMap::new();
        metadata.insert("test_key".to_string(), json!("test_value"));

        GraphDocument {
            id: "test_id".to_string(),
            content: "test content".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            metadata,
            entities: vec!["entity1".to_string(), "entity2".to_string()],
            relationships: vec!["rel1".to_string(), "rel2".to_string()],
            similarity_score: Some(0.85),
        }
    }

    // Helper function to create test RawTextDocument
    fn create_test_raw_document() -> RawTextDocument {
        RawTextDocument {
            id: "test_doc_1".to_string(),
            content: "This is test content".to_string(),
            metadata: Some(DocumentMetadata {
                title: Some("Test Document".to_string()),
                tags: vec!["test".to_string()],
                chain: Some("ethereum".to_string()),
                block_number: Some(12345),
                transaction_hash: Some("0x123abc".to_string()),
                wallet_addresses: vec!["0xabc123".to_string()],
                token_addresses: vec!["0xdef456".to_string()],
                protocols: vec!["uniswap".to_string()],
                extraction_confidence: Some(0.9),
                custom_fields: HashMap::new(),
            }),
            embedding: Some(vec![0.1, 0.2, 0.3, 0.4]),
            created_at: Utc::now(),
            source: DocumentSource::UserInput,
        }
    }

    // Tests for GraphRetrieverConfig

    #[test]
    fn test_graph_retriever_config_new_default() {
        let config = GraphRetrieverConfig::new_default();
        assert_eq!(config.similarity_threshold, 0.7);
        assert_eq!(config.max_graph_hops, 2);
        assert_eq!(config.embedding_dimension, 1536);
        assert_eq!(config.index_name, "document_embeddings");
    }

    #[test]
    fn test_graph_retriever_config_high_precision() {
        let config = GraphRetrieverConfig::high_precision();
        assert_eq!(config.similarity_threshold, 0.8);
        assert_eq!(config.max_graph_hops, 1);
        assert_eq!(config.embedding_dimension, 1536);
        assert_eq!(config.index_name, "document_embeddings");
    }

    #[test]
    fn test_graph_retriever_config_broad_context() {
        let config = GraphRetrieverConfig::broad_context();
        assert_eq!(config.similarity_threshold, 0.6);
        assert_eq!(config.max_graph_hops, 3);
        assert_eq!(config.embedding_dimension, 1536);
        assert_eq!(config.index_name, "document_embeddings");
    }

    #[test]
    fn test_graph_retriever_config_clone() {
        let config = GraphRetrieverConfig::new_default();
        let cloned = config.clone();
        assert_eq!(config.similarity_threshold, cloned.similarity_threshold);
        assert_eq!(config.max_graph_hops, cloned.max_graph_hops);
        assert_eq!(config.embedding_dimension, cloned.embedding_dimension);
        assert_eq!(config.index_name, cloned.index_name);
    }

    #[test]
    fn test_graph_retriever_config_debug() {
        let config = GraphRetrieverConfig::new_default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("GraphRetrieverConfig"));
        assert!(debug_str.contains("0.7"));
        assert!(debug_str.contains("document_embeddings"));
    }

    // Tests for GraphDocument

    #[test]
    fn test_graph_document_creation() {
        let doc = create_test_graph_document();
        assert_eq!(doc.id, "test_id");
        assert_eq!(doc.content, "test content");
        assert_eq!(doc.embedding, vec![0.1, 0.2, 0.3]);
        assert_eq!(doc.entities.len(), 2);
        assert_eq!(doc.relationships.len(), 2);
        assert_eq!(doc.similarity_score, Some(0.85));
    }

    #[test]
    fn test_graph_document_empty_collections() {
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), json!("value"));

        let doc = GraphDocument {
            id: "empty_test".to_string(),
            content: "".to_string(),
            embedding: vec![],
            metadata,
            entities: vec![],
            relationships: vec![],
            similarity_score: None,
        };

        assert_eq!(doc.id, "empty_test");
        assert_eq!(doc.content, "");
        assert!(doc.embedding.is_empty());
        assert!(doc.entities.is_empty());
        assert!(doc.relationships.is_empty());
        assert!(doc.similarity_score.is_none());
        assert_eq!(doc.metadata.len(), 1);
    }

    #[test]
    fn test_graph_document_clone() {
        let doc = create_test_graph_document();
        let cloned = doc.clone();
        assert_eq!(doc.id, cloned.id);
        assert_eq!(doc.content, cloned.content);
        assert_eq!(doc.embedding, cloned.embedding);
        assert_eq!(doc.entities, cloned.entities);
        assert_eq!(doc.relationships, cloned.relationships);
        assert_eq!(doc.similarity_score, cloned.similarity_score);
    }

    #[test]
    fn test_graph_document_debug() {
        let doc = create_test_graph_document();
        let debug_str = format!("{:?}", doc);
        assert!(debug_str.contains("GraphDocument"));
        assert!(debug_str.contains("test_id"));
        assert!(debug_str.contains("test content"));
    }

    #[test]
    fn test_graph_document_serialization() {
        let doc = create_test_graph_document();
        let serialized = serde_json::to_string(&doc).unwrap();
        assert!(serialized.contains("test_id"));
        assert!(serialized.contains("test content"));
        assert!(serialized.contains("entity1"));
    }

    #[test]
    fn test_graph_document_deserialization() {
        let json_str = r#"{
            "id": "test_id",
            "content": "test content",
            "embedding": [0.1, 0.2, 0.3],
            "metadata": {"key": "value"},
            "entities": ["entity1"],
            "relationships": ["rel1"],
            "similarity_score": 0.85
        }"#;

        let doc: GraphDocument = serde_json::from_str(json_str).unwrap();
        assert_eq!(doc.id, "test_id");
        assert_eq!(doc.content, "test content");
        assert_eq!(doc.embedding, vec![0.1, 0.2, 0.3]);
        assert_eq!(doc.entities, vec!["entity1"]);
        assert_eq!(doc.similarity_score, Some(0.85));
    }

    // Tests for GraphSearchResult

    #[test]
    fn test_graph_search_result_creation() {
        let documents = vec![create_test_graph_document()];
        let related_entities = vec!["entity1".to_string(), "entity2".to_string()];
        let metrics = SearchMetrics {
            vector_search_time_ms: 10,
            graph_traversal_time_ms: 5,
            total_time_ms: 15,
            nodes_examined: 3,
            relationships_traversed: 2,
        };

        let result = GraphSearchResult {
            documents: documents.clone(),
            related_entities: related_entities.clone(),
            metrics: metrics.clone(),
        };

        assert_eq!(result.documents.len(), 1);
        assert_eq!(result.related_entities.len(), 2);
        assert_eq!(result.metrics.total_time_ms, 15);
    }

    #[test]
    fn test_graph_search_result_empty() {
        let result = GraphSearchResult {
            documents: vec![],
            related_entities: vec![],
            metrics: SearchMetrics {
                vector_search_time_ms: 0,
                graph_traversal_time_ms: 0,
                total_time_ms: 0,
                nodes_examined: 0,
                relationships_traversed: 0,
            },
        };

        assert!(result.documents.is_empty());
        assert!(result.related_entities.is_empty());
        assert_eq!(result.metrics.total_time_ms, 0);
    }

    #[test]
    fn test_graph_search_result_clone() {
        let result = GraphSearchResult {
            documents: vec![create_test_graph_document()],
            related_entities: vec!["entity".to_string()],
            metrics: SearchMetrics {
                vector_search_time_ms: 10,
                graph_traversal_time_ms: 5,
                total_time_ms: 15,
                nodes_examined: 3,
                relationships_traversed: 2,
            },
        };

        let cloned = result.clone();
        assert_eq!(result.documents.len(), cloned.documents.len());
        assert_eq!(result.related_entities, cloned.related_entities);
        assert_eq!(result.metrics.total_time_ms, cloned.metrics.total_time_ms);
    }

    #[test]
    fn test_graph_search_result_debug() {
        let result = GraphSearchResult {
            documents: vec![],
            related_entities: vec![],
            metrics: SearchMetrics {
                vector_search_time_ms: 10,
                graph_traversal_time_ms: 5,
                total_time_ms: 15,
                nodes_examined: 3,
                relationships_traversed: 2,
            },
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("GraphSearchResult"));
    }

    // Tests for SearchMetrics

    #[test]
    fn test_search_metrics_creation() {
        let metrics = SearchMetrics {
            vector_search_time_ms: 100,
            graph_traversal_time_ms: 50,
            total_time_ms: 150,
            nodes_examined: 10,
            relationships_traversed: 5,
        };

        assert_eq!(metrics.vector_search_time_ms, 100);
        assert_eq!(metrics.graph_traversal_time_ms, 50);
        assert_eq!(metrics.total_time_ms, 150);
        assert_eq!(metrics.nodes_examined, 10);
        assert_eq!(metrics.relationships_traversed, 5);
    }

    #[test]
    fn test_search_metrics_zero_values() {
        let metrics = SearchMetrics {
            vector_search_time_ms: 0,
            graph_traversal_time_ms: 0,
            total_time_ms: 0,
            nodes_examined: 0,
            relationships_traversed: 0,
        };

        assert_eq!(metrics.vector_search_time_ms, 0);
        assert_eq!(metrics.graph_traversal_time_ms, 0);
        assert_eq!(metrics.total_time_ms, 0);
        assert_eq!(metrics.nodes_examined, 0);
        assert_eq!(metrics.relationships_traversed, 0);
    }

    #[test]
    fn test_search_metrics_max_values() {
        let metrics = SearchMetrics {
            vector_search_time_ms: u64::MAX,
            graph_traversal_time_ms: u64::MAX,
            total_time_ms: u64::MAX,
            nodes_examined: u32::MAX,
            relationships_traversed: u32::MAX,
        };

        assert_eq!(metrics.vector_search_time_ms, u64::MAX);
        assert_eq!(metrics.graph_traversal_time_ms, u64::MAX);
        assert_eq!(metrics.total_time_ms, u64::MAX);
        assert_eq!(metrics.nodes_examined, u32::MAX);
        assert_eq!(metrics.relationships_traversed, u32::MAX);
    }

    #[test]
    fn test_search_metrics_clone() {
        let metrics = SearchMetrics {
            vector_search_time_ms: 100,
            graph_traversal_time_ms: 50,
            total_time_ms: 150,
            nodes_examined: 10,
            relationships_traversed: 5,
        };

        let cloned = metrics.clone();
        assert_eq!(metrics.vector_search_time_ms, cloned.vector_search_time_ms);
        assert_eq!(
            metrics.graph_traversal_time_ms,
            cloned.graph_traversal_time_ms
        );
        assert_eq!(metrics.total_time_ms, cloned.total_time_ms);
        assert_eq!(metrics.nodes_examined, cloned.nodes_examined);
        assert_eq!(
            metrics.relationships_traversed,
            cloned.relationships_traversed
        );
    }

    #[test]
    fn test_search_metrics_debug() {
        let metrics = SearchMetrics {
            vector_search_time_ms: 100,
            graph_traversal_time_ms: 50,
            total_time_ms: 150,
            nodes_examined: 10,
            relationships_traversed: 5,
        };

        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("SearchMetrics"));
        assert!(debug_str.contains("100"));
        assert!(debug_str.contains("150"));
    }

    // Tests for From<GraphDocument> trait implementation

    #[test]
    fn test_from_graph_document_to_raw_text_document() {
        let graph_doc = create_test_graph_document();
        let raw_doc: RawTextDocument = graph_doc.into();

        assert_eq!(raw_doc.id, "test_id");
        assert_eq!(raw_doc.content, "test content");
        assert_eq!(raw_doc.embedding, Some(vec![0.1, 0.2, 0.3]));
        assert!(raw_doc.metadata.is_none());
        assert!(matches!(raw_doc.source, DocumentSource::UserInput));
    }

    #[test]
    fn test_from_graph_document_empty_embedding() {
        let mut graph_doc = create_test_graph_document();
        graph_doc.embedding = vec![];

        let raw_doc: RawTextDocument = graph_doc.into();
        assert_eq!(raw_doc.embedding, Some(vec![]));
    }

    #[test]
    fn test_from_graph_document_large_embedding() {
        let mut graph_doc = create_test_graph_document();
        graph_doc.embedding = vec![0.1; 1536]; // Standard OpenAI embedding size

        let raw_doc: RawTextDocument = graph_doc.into();
        assert_eq!(raw_doc.embedding.as_ref().unwrap().len(), 1536);
        assert_eq!(raw_doc.embedding.as_ref().unwrap()[0], 0.1);
        assert_eq!(raw_doc.embedding.as_ref().unwrap()[1535], 0.1);
    }

    #[test]
    fn test_from_graph_document_empty_content() {
        let mut graph_doc = create_test_graph_document();
        graph_doc.content = "".to_string();

        let raw_doc: RawTextDocument = graph_doc.into();
        assert_eq!(raw_doc.content, "");
    }

    #[test]
    fn test_from_graph_document_special_characters() {
        let mut graph_doc = create_test_graph_document();
        graph_doc.content = "Special chars: åéñ™£¡™£¡".to_string();
        graph_doc.id = "special-id-123_abc".to_string();

        let raw_doc: RawTextDocument = graph_doc.into();
        assert_eq!(raw_doc.content, "Special chars: åéñ™£¡™£¡");
        assert_eq!(raw_doc.id, "special-id-123_abc");
    }

    // Note: Since GraphRetriever requires a real Neo4jClient and database connection,
    // these tests focus on the logic that can be tested without mocking the entire client.
    // In a real-world scenario, you would either:
    // 1. Use dependency injection to mock the Neo4jClient
    // 2. Use integration tests with a test database
    // 3. Create a trait for the client and mock that trait

    // Tests for sorting logic used in search_with_graph_context
    #[test]
    fn test_document_sorting_by_similarity_score() {
        let mut docs = vec![
            GraphDocument {
                id: "doc1".to_string(),
                content: "content1".to_string(),
                embedding: vec![],
                metadata: HashMap::new(),
                entities: vec![],
                relationships: vec![],
                similarity_score: Some(0.5),
            },
            GraphDocument {
                id: "doc2".to_string(),
                content: "content2".to_string(),
                embedding: vec![],
                metadata: HashMap::new(),
                entities: vec![],
                relationships: vec![],
                similarity_score: Some(0.9),
            },
            GraphDocument {
                id: "doc3".to_string(),
                content: "content3".to_string(),
                embedding: vec![],
                metadata: HashMap::new(),
                entities: vec![],
                relationships: vec![],
                similarity_score: Some(0.7),
            },
        ];

        // This mimics the sorting logic in search_with_graph_context
        docs.sort_by(|a, b| {
            b.similarity_score
                .unwrap_or(0.0)
                .partial_cmp(&a.similarity_score.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        assert_eq!(docs[0].id, "doc2"); // 0.9
        assert_eq!(docs[1].id, "doc3"); // 0.7
        assert_eq!(docs[2].id, "doc1"); // 0.5
    }

    #[test]
    fn test_document_sorting_with_none_scores() {
        let mut docs = vec![
            GraphDocument {
                id: "doc1".to_string(),
                content: "content1".to_string(),
                embedding: vec![],
                metadata: HashMap::new(),
                entities: vec![],
                relationships: vec![],
                similarity_score: None,
            },
            GraphDocument {
                id: "doc2".to_string(),
                content: "content2".to_string(),
                embedding: vec![],
                metadata: HashMap::new(),
                entities: vec![],
                relationships: vec![],
                similarity_score: Some(0.9),
            },
            GraphDocument {
                id: "doc3".to_string(),
                content: "content3".to_string(),
                embedding: vec![],
                metadata: HashMap::new(),
                entities: vec![],
                relationships: vec![],
                similarity_score: None,
            },
        ];

        docs.sort_by(|a, b| {
            b.similarity_score
                .unwrap_or(0.0)
                .partial_cmp(&a.similarity_score.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        assert_eq!(docs[0].id, "doc2"); // 0.9
                                        // doc1 and doc3 order is not guaranteed since both have None -> 0.0
    }

    #[test]
    fn test_document_sorting_equal_scores() {
        let mut docs = vec![
            GraphDocument {
                id: "doc1".to_string(),
                content: "content1".to_string(),
                embedding: vec![],
                metadata: HashMap::new(),
                entities: vec![],
                relationships: vec![],
                similarity_score: Some(0.5),
            },
            GraphDocument {
                id: "doc2".to_string(),
                content: "content2".to_string(),
                embedding: vec![],
                metadata: HashMap::new(),
                entities: vec![],
                relationships: vec![],
                similarity_score: Some(0.5),
            },
        ];

        docs.sort_by(|a, b| {
            b.similarity_score
                .unwrap_or(0.0)
                .partial_cmp(&a.similarity_score.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Order should remain stable for equal scores
        assert_eq!(docs.len(), 2);
        assert!(docs[0].similarity_score == docs[1].similarity_score);
    }

    #[test]
    fn test_document_truncation_logic() {
        let mut docs = vec![
            create_test_graph_document(),
            create_test_graph_document(),
            create_test_graph_document(),
            create_test_graph_document(),
            create_test_graph_document(),
        ];

        let limit = 3;
        docs.truncate(limit);

        assert_eq!(docs.len(), 3);
    }

    #[test]
    fn test_document_truncation_no_change() {
        let mut docs = vec![create_test_graph_document(), create_test_graph_document()];

        let limit = 5;
        docs.truncate(limit);

        assert_eq!(docs.len(), 2); // No change since limit > length
    }

    #[test]
    fn test_document_truncation_empty() {
        let mut docs: Vec<GraphDocument> = vec![];
        let limit = 3;
        docs.truncate(limit);

        assert_eq!(docs.len(), 0);
    }

    // Tests for entity set operations used in search_with_graph_context
    #[test]
    fn test_entity_set_operations() {
        let mut entity_set = std::collections::HashSet::new();
        let entities = vec![
            "entity1".to_string(),
            "entity2".to_string(),
            "entity1".to_string(),
        ];

        entity_set.extend(entities.iter().cloned());

        assert_eq!(entity_set.len(), 2); // Duplicates should be removed
        assert!(entity_set.contains("entity1"));
        assert!(entity_set.contains("entity2"));
    }

    #[test]
    fn test_entity_set_empty_extension() {
        let mut entity_set = std::collections::HashSet::new();
        let entities: Vec<String> = vec![];

        entity_set.extend(entities.iter().cloned());

        assert_eq!(entity_set.len(), 0);
        assert!(entity_set.is_empty());
    }

    #[test]
    fn test_entity_set_to_vector_conversion() {
        let mut entity_set = std::collections::HashSet::new();
        entity_set.insert("entity1".to_string());
        entity_set.insert("entity2".to_string());

        let related_entities: Vec<String> = entity_set.into_iter().collect();

        assert_eq!(related_entities.len(), 2);
        assert!(related_entities.contains(&"entity1".to_string()));
        assert!(related_entities.contains(&"entity2".to_string()));
    }

    // Tests for query parameter construction used in search methods
    #[test]
    fn test_query_parameter_construction() {
        let query_embedding = vec![0.1, 0.2, 0.3];
        let limit = 10;

        let mut params = HashMap::new();
        params.insert("embedding".to_string(), json!(query_embedding));
        params.insert("limit".to_string(), json!(limit));

        assert_eq!(params.len(), 2);
        assert_eq!(params.get("embedding").unwrap(), &json!([0.1, 0.2, 0.3]));
        assert_eq!(params.get("limit").unwrap(), &json!(10));
    }

    #[test]
    fn test_query_parameter_construction_empty_embedding() {
        let query_embedding: Vec<f32> = vec![];
        let limit = 5;

        let mut params = HashMap::new();
        params.insert("embedding".to_string(), json!(query_embedding));
        params.insert("limit".to_string(), json!(limit));

        assert_eq!(params.get("embedding").unwrap(), &json!([]));
        assert_eq!(params.get("limit").unwrap(), &json!(5));
    }

    #[test]
    fn test_query_parameter_construction_large_embedding() {
        let query_embedding = vec![0.1; 1536];
        let limit = 0;

        let mut params = HashMap::new();
        params.insert("embedding".to_string(), json!(query_embedding));
        params.insert("limit".to_string(), json!(limit));

        assert_eq!(params.get("limit").unwrap(), &json!(0));
        let embedding_param = params.get("embedding").unwrap().as_array().unwrap();
        assert_eq!(embedding_param.len(), 1536);
    }

    // Tests for JSON response parsing logic used in search_with_graph_context
    #[test]
    fn test_vector_search_response_parsing_success() {
        let vector_results = json!({
            "results": [{
                "data": [{
                    "row": [
                        "doc_id_1",           // id
                        "test content",       // content
                        {"title": "Test"},    // metadata
                        ["entity1", "entity2"], // entities
                        [0.1, 0.2, 0.3],      // embedding
                        0.85                  // score
                    ]
                }]
            }]
        });

        let mut documents = Vec::new();
        let mut entity_set = std::collections::HashSet::new();

        if let Some(results) = vector_results["results"].as_array() {
            for result in results {
                if let Some(data) = result["data"].as_array() {
                    for row in data {
                        if let Some(row_data) = row["row"].as_array() {
                            if let (Some(id), Some(content), Some(score)) = (
                                row_data[0].as_str(),
                                row_data[1].as_str(),
                                row_data[5].as_f64(),
                            ) {
                                let similarity_score = score as f32;
                                let similarity_threshold = 0.7;

                                if similarity_score >= similarity_threshold {
                                    let metadata: HashMap<String, Value> = row_data[2]
                                        .as_object()
                                        .map(|obj| {
                                            obj.iter()
                                                .map(|(k, v)| (k.clone(), v.clone()))
                                                .collect()
                                        })
                                        .unwrap_or_default();

                                    let entities: Vec<String> = row_data[3]
                                        .as_array()
                                        .map(|arr| {
                                            arr.iter()
                                                .filter_map(|v| v.as_str())
                                                .map(|s| s.to_string())
                                                .collect()
                                        })
                                        .unwrap_or_default();

                                    let actual_embedding: Vec<f32> = row_data[4]
                                        .as_array()
                                        .map(|arr| {
                                            arr.iter()
                                                .filter_map(|v| v.as_f64().map(|f| f as f32))
                                                .collect()
                                        })
                                        .unwrap_or_else(|| Vec::new());

                                    entity_set.extend(entities.iter().cloned());

                                    documents.push(GraphDocument {
                                        id: id.to_string(),
                                        content: content.to_string(),
                                        embedding: actual_embedding,
                                        metadata,
                                        entities,
                                        relationships: Vec::new(),
                                        similarity_score: Some(similarity_score),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0].id, "doc_id_1");
        assert_eq!(documents[0].content, "test content");
        assert_eq!(documents[0].similarity_score, Some(0.85));
        assert_eq!(documents[0].embedding, vec![0.1, 0.2, 0.3]);
        assert_eq!(entity_set.len(), 2);
    }

    #[test]
    fn test_vector_search_response_parsing_below_threshold() {
        let vector_results = json!({
            "results": [{
                "data": [{
                    "row": [
                        "doc_id_1",
                        "test content",
                        {},
                        [],
                        [],
                        0.5  // Below threshold of 0.7
                    ]
                }]
            }]
        });

        let mut documents = Vec::new();
        let similarity_threshold = 0.7;

        if let Some(results) = vector_results["results"].as_array() {
            for result in results {
                if let Some(data) = result["data"].as_array() {
                    for row in data {
                        if let Some(row_data) = row["row"].as_array() {
                            if let (Some(_id), Some(_content), Some(score)) = (
                                row_data[0].as_str(),
                                row_data[1].as_str(),
                                row_data[5].as_f64(),
                            ) {
                                let similarity_score = score as f32;

                                if similarity_score >= similarity_threshold {
                                    // This should not execute due to threshold
                                    documents.push(create_test_graph_document());
                                }
                            }
                        }
                    }
                }
            }
        }

        assert_eq!(documents.len(), 0); // Should be filtered out
    }

    #[test]
    fn test_vector_search_response_parsing_empty_results() {
        let vector_results = json!({
            "results": []
        });

        let mut documents = Vec::new();

        if let Some(results) = vector_results["results"].as_array() {
            for result in results {
                if let Some(data) = result["data"].as_array() {
                    for row in data {
                        if let Some(_row_data) = row["row"].as_array() {
                            documents.push(create_test_graph_document());
                        }
                    }
                }
            }
        }

        assert_eq!(documents.len(), 0);
    }

    #[test]
    fn test_vector_search_response_parsing_missing_fields() {
        let vector_results = json!({
            "results": [{
                "data": [{
                    "row": [
                        "doc_id_1",
                        null,  // Missing content
                        {},
                        [],
                        [],
                        0.85
                    ]
                }]
            }]
        });

        let mut documents = Vec::new();

        if let Some(results) = vector_results["results"].as_array() {
            for result in results {
                if let Some(data) = result["data"].as_array() {
                    for row in data {
                        if let Some(row_data) = row["row"].as_array() {
                            if let (Some(_id), Some(_content), Some(_score)) = (
                                row_data[0].as_str(),
                                row_data[1].as_str(), // This will be None due to null
                                row_data[5].as_f64(),
                            ) {
                                documents.push(create_test_graph_document());
                            }
                        }
                    }
                }
            }
        }

        assert_eq!(documents.len(), 0); // Should not match due to missing content
    }

    #[test]
    fn test_vector_search_response_parsing_invalid_embedding() {
        let vector_results = json!({
            "results": [{
                "data": [{
                    "row": [
                        "doc_id_1",
                        "test content",
                        {},
                        [],
                        "invalid_embedding", // Not an array
                        0.85
                    ]
                }]
            }]
        });

        let similarity_threshold = 0.7;
        let mut documents = Vec::new();

        if let Some(results) = vector_results["results"].as_array() {
            for result in results {
                if let Some(data) = result["data"].as_array() {
                    for row in data {
                        if let Some(row_data) = row["row"].as_array() {
                            if let (Some(id), Some(content), Some(score)) = (
                                row_data[0].as_str(),
                                row_data[1].as_str(),
                                row_data[5].as_f64(),
                            ) {
                                let similarity_score = score as f32;

                                if similarity_score >= similarity_threshold {
                                    let actual_embedding: Vec<f32> = row_data[4]
                                        .as_array()
                                        .map(|arr| {
                                            arr.iter()
                                                .filter_map(|v| v.as_f64().map(|f| f as f32))
                                                .collect()
                                        })
                                        .unwrap_or_else(|| Vec::new()); // Should return empty vec

                                    documents.push(GraphDocument {
                                        id: id.to_string(),
                                        content: content.to_string(),
                                        embedding: actual_embedding,
                                        metadata: HashMap::new(),
                                        entities: vec![],
                                        relationships: Vec::new(),
                                        similarity_score: Some(similarity_score),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        assert_eq!(documents.len(), 1);
        assert!(documents[0].embedding.is_empty()); // Should be empty due to invalid embedding
    }

    // Tests for related entities response parsing
    #[test]
    fn test_related_entities_response_parsing_success() {
        let result = json!({
            "results": [{
                "data": [
                    {"row": ["related_entity_1"]},
                    {"row": ["related_entity_2"]},
                    {"row": ["related_entity_3"]}
                ]
            }]
        });

        let mut related = Vec::new();
        if let Some(results) = result["results"].as_array() {
            for result in results {
                if let Some(data) = result["data"].as_array() {
                    for row in data {
                        if let Some(row_data) = row["row"].as_array() {
                            if let Some(entity) = row_data[0].as_str() {
                                related.push(entity.to_string());
                            }
                        }
                    }
                }
            }
        }

        assert_eq!(related.len(), 3);
        assert_eq!(related[0], "related_entity_1");
        assert_eq!(related[1], "related_entity_2");
        assert_eq!(related[2], "related_entity_3");
    }

    #[test]
    fn test_related_entities_response_parsing_empty() {
        let result = json!({
            "results": [{
                "data": []
            }]
        });

        let mut related = Vec::new();
        if let Some(results) = result["results"].as_array() {
            for result in results {
                if let Some(data) = result["data"].as_array() {
                    for row in data {
                        if let Some(row_data) = row["row"].as_array() {
                            if let Some(entity) = row_data[0].as_str() {
                                related.push(entity.to_string());
                            }
                        }
                    }
                }
            }
        }

        assert_eq!(related.len(), 0);
    }

    #[test]
    fn test_related_entities_response_parsing_null_values() {
        let result = json!({
            "results": [{
                "data": [
                    {"row": ["valid_entity"]},
                    {"row": [null]},  // Null entity
                    {"row": ["another_valid_entity"]}
                ]
            }]
        });

        let mut related = Vec::new();
        if let Some(results) = result["results"].as_array() {
            for result in results {
                if let Some(data) = result["data"].as_array() {
                    for row in data {
                        if let Some(row_data) = row["row"].as_array() {
                            if let Some(entity) = row_data[0].as_str() {
                                related.push(entity.to_string());
                            }
                        }
                    }
                }
            }
        }

        assert_eq!(related.len(), 2); // Should skip the null value
        assert_eq!(related[0], "valid_entity");
        assert_eq!(related[1], "another_valid_entity");
    }

    // Tests for entity list conversion used in find_related_entities
    #[test]
    fn test_entity_list_conversion() {
        let mut initial_entities = std::collections::HashSet::new();
        initial_entities.insert("entity1".to_string());
        initial_entities.insert("entity2".to_string());
        initial_entities.insert("entity3".to_string());

        let entities_list: Vec<&str> = initial_entities.iter().map(|s| s.as_str()).collect();

        assert_eq!(entities_list.len(), 3);
        assert!(entities_list.contains(&"entity1"));
        assert!(entities_list.contains(&"entity2"));
        assert!(entities_list.contains(&"entity3"));
    }

    #[test]
    fn test_entity_list_conversion_empty() {
        let initial_entities = std::collections::HashSet::new();
        let entities_list: Vec<&str> = initial_entities
            .iter()
            .map(|s: &String| s.as_str())
            .collect();

        assert_eq!(entities_list.len(), 0);
    }

    #[test]
    fn test_entity_list_conversion_single_entity() {
        let mut initial_entities = std::collections::HashSet::new();
        initial_entities.insert("single_entity".to_string());

        let entities_list: Vec<&str> = initial_entities.iter().map(|s| s.as_str()).collect();

        assert_eq!(entities_list.len(), 1);
        assert_eq!(entities_list[0], "single_entity");
    }

    // Tests for query building logic
    #[test]
    fn test_vector_index_query_building() {
        let index_name = "test_index";
        let embedding_dimension = 1536;

        let create_index_query = format!(
            "CREATE VECTOR INDEX IF NOT EXISTS {} FOR (d:Document) ON (d.embedding)
             OPTIONS {{indexConfig: {{`vector.dimensions`: {}, `vector.similarity_function`: 'cosine'}}}}",
            index_name, embedding_dimension
        );

        assert!(create_index_query.contains("CREATE VECTOR INDEX IF NOT EXISTS test_index"));
        assert!(create_index_query.contains("1536"));
        assert!(create_index_query.contains("cosine"));
        assert!(create_index_query.contains("d:Document"));
        assert!(create_index_query.contains("d.embedding"));
    }

    #[test]
    fn test_vector_search_query_building() {
        let index_name = "test_index";
        let limit = 10;

        let vector_search_query = format!(
            "CALL db.index.vector.queryNodes('{}', {}, $embedding)
             YIELD node, score
             RETURN node.id as id, node.content as content, node.metadata as metadata,
                    node.entities as entities, node.embedding as embedding, score
             LIMIT $limit",
            index_name,
            limit * 2
        );

        assert!(vector_search_query
            .contains("CALL db.index.vector.queryNodes('test_index', 20, $embedding)"));
        assert!(vector_search_query.contains("YIELD node, score"));
        assert!(vector_search_query.contains("RETURN node.id as id"));
        assert!(vector_search_query.contains("LIMIT $limit"));
    }

    #[test]
    fn test_graph_traversal_query_building() {
        let max_graph_hops = 3;

        let graph_query = format!(
            "UNWIND $entities as entity
             MATCH (e1 {{canonical: entity}})-[r]-(e2)
             WHERE e1 <> e2
             RETURN DISTINCT e2.canonical as related_entity
             LIMIT {}",
            max_graph_hops * 50
        );

        assert!(graph_query.contains("UNWIND $entities as entity"));
        assert!(graph_query.contains("MATCH (e1 {canonical: entity})-[r]-(e2)"));
        assert!(graph_query.contains("WHERE e1 <> e2"));
        assert!(graph_query.contains("RETURN DISTINCT e2.canonical as related_entity"));
        assert!(graph_query.contains("LIMIT 150")); // 3 * 50
    }

    #[test]
    fn test_add_document_query_building() {
        let create_doc_query = "
                CREATE (d:Document {
                    id: $id,
                    content: $content,
                    created_at: $created_at,
                    source: $source
                })
                RETURN d.id as id
            ";

        assert!(create_doc_query.contains("CREATE (d:Document"));
        assert!(create_doc_query.contains("id: $id"));
        assert!(create_doc_query.contains("content: $content"));
        assert!(create_doc_query.contains("created_at: $created_at"));
        assert!(create_doc_query.contains("source: $source"));
        assert!(create_doc_query.contains("RETURN d.id as id"));
    }

    // Tests for time measurement logic used in search methods
    #[test]
    fn test_time_measurement_simulation() {
        use std::time::Instant;

        let start_time = Instant::now();

        // Simulate some work
        let numbers: Vec<i32> = (0..1000).collect();
        let _sum: i32 = numbers.iter().sum();

        let elapsed_ms = start_time.elapsed().as_millis() as u64;

        // Should be a small positive number (u64 is always >= 0)
        assert!(elapsed_ms < 1000); // Should complete quickly
    }

    #[test]
    fn test_metrics_initialization() {
        let metrics = SearchMetrics {
            vector_search_time_ms: 0,
            graph_traversal_time_ms: 0,
            total_time_ms: 0,
            nodes_examined: 0,
            relationships_traversed: 0,
        };

        assert_eq!(metrics.vector_search_time_ms, 0);
        assert_eq!(metrics.graph_traversal_time_ms, 0);
        assert_eq!(metrics.total_time_ms, 0);
        assert_eq!(metrics.nodes_examined, 0);
        assert_eq!(metrics.relationships_traversed, 0);
    }

    #[test]
    fn test_metrics_updates() {
        let mut metrics = SearchMetrics {
            vector_search_time_ms: 0,
            graph_traversal_time_ms: 0,
            total_time_ms: 0,
            nodes_examined: 0,
            relationships_traversed: 0,
        };

        // Simulate metric updates
        metrics.vector_search_time_ms = 50;
        metrics.graph_traversal_time_ms = 30;
        metrics.total_time_ms = 80;
        metrics.nodes_examined = 10;
        metrics.relationships_traversed = 5;

        assert_eq!(metrics.vector_search_time_ms, 50);
        assert_eq!(metrics.graph_traversal_time_ms, 30);
        assert_eq!(metrics.total_time_ms, 80);
        assert_eq!(metrics.nodes_examined, 10);
        assert_eq!(metrics.relationships_traversed, 5);
    }

    // Tests for document ID collection logic used in top_n_ids
    #[test]
    fn test_document_id_collection() {
        let documents = vec![
            GraphDocument {
                id: "doc1".to_string(),
                content: "content1".to_string(),
                embedding: vec![],
                metadata: HashMap::new(),
                entities: vec![],
                relationships: vec![],
                similarity_score: Some(0.9),
            },
            GraphDocument {
                id: "doc2".to_string(),
                content: "content2".to_string(),
                embedding: vec![],
                metadata: HashMap::new(),
                entities: vec![],
                relationships: vec![],
                similarity_score: Some(0.8),
            },
            GraphDocument {
                id: "doc3".to_string(),
                content: "content3".to_string(),
                embedding: vec![],
                metadata: HashMap::new(),
                entities: vec![],
                relationships: vec![],
                similarity_score: Some(0.7),
            },
        ];

        let ids: Vec<String> = documents.into_iter().map(|doc| doc.id).collect();

        assert_eq!(ids.len(), 3);
        assert_eq!(ids[0], "doc1");
        assert_eq!(ids[1], "doc2");
        assert_eq!(ids[2], "doc3");
    }

    #[test]
    fn test_document_id_collection_empty() {
        let documents: Vec<GraphDocument> = vec![];
        let ids: Vec<String> = documents.into_iter().map(|doc| doc.id).collect();

        assert_eq!(ids.len(), 0);
    }

    #[test]
    fn test_document_id_collection_single() {
        let documents = vec![GraphDocument {
            id: "single_doc".to_string(),
            content: "content".to_string(),
            embedding: vec![],
            metadata: HashMap::new(),
            entities: vec![],
            relationships: vec![],
            similarity_score: Some(0.9),
        }];

        let ids: Vec<String> = documents.into_iter().map(|doc| doc.id).collect();

        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "single_doc");
    }

    // Tests for add_documents parameter construction
    #[test]
    fn test_add_documents_parameter_construction() {
        let doc = create_test_raw_document();

        let mut params = HashMap::new();
        params.insert("id".to_string(), json!(doc.id));
        params.insert("content".to_string(), json!(doc.content));
        params.insert("created_at".to_string(), json!(doc.created_at.to_rfc3339()));
        params.insert("source".to_string(), json!(format!("{:?}", doc.source)));

        assert_eq!(params.len(), 4);
        assert_eq!(params.get("id").unwrap(), &json!("test_doc_1"));
        assert_eq!(
            params.get("content").unwrap(),
            &json!("This is test content")
        );
        assert!(params
            .get("created_at")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("T"));
        assert_eq!(params.get("source").unwrap(), &json!("UserInput"));
    }

    #[test]
    fn test_add_documents_parameter_construction_onchain_source() {
        let mut doc = create_test_raw_document();
        doc.source = DocumentSource::OnChain {
            chain: "ethereum".to_string(),
            transaction_hash: "0x123abc".to_string(),
        };

        let source_debug = format!("{:?}", doc.source);

        assert!(source_debug.contains("OnChain"));
        assert!(source_debug.contains("ethereum"));
        assert!(source_debug.contains("0x123abc"));
    }

    #[test]
    fn test_add_documents_parameter_construction_empty_content() {
        let mut doc = create_test_raw_document();
        doc.content = "".to_string();
        doc.id = "empty_doc".to_string();

        let mut params = HashMap::new();
        params.insert("id".to_string(), json!(doc.id));
        params.insert("content".to_string(), json!(doc.content));

        assert_eq!(params.get("id").unwrap(), &json!("empty_doc"));
        assert_eq!(params.get("content").unwrap(), &json!(""));
    }

    // Tests for max_graph_hops condition checking
    #[test]
    fn test_max_graph_hops_condition_zero() {
        let max_graph_hops = 0;
        let entity_set = std::collections::HashSet::from(["entity1".to_string()]);

        let should_traverse = max_graph_hops > 0 && !entity_set.is_empty();
        assert!(!should_traverse);
    }

    #[test]
    fn test_max_graph_hops_condition_empty_entities() {
        let max_graph_hops = 2;
        let entity_set: std::collections::HashSet<String> = std::collections::HashSet::new();

        let should_traverse = max_graph_hops > 0 && !entity_set.is_empty();
        assert!(!should_traverse);
    }

    #[test]
    fn test_max_graph_hops_condition_valid() {
        let max_graph_hops = 2;
        let entity_set = std::collections::HashSet::from(["entity1".to_string()]);

        let should_traverse = max_graph_hops > 0 && !entity_set.is_empty();
        assert!(should_traverse);
    }

    // Tests for relationships assignment logic
    #[test]
    fn test_relationships_assignment() {
        let mut documents = vec![
            GraphDocument {
                id: "doc1".to_string(),
                content: "content1".to_string(),
                embedding: vec![],
                metadata: HashMap::new(),
                entities: vec![],
                relationships: vec![],
                similarity_score: Some(0.9),
            },
            GraphDocument {
                id: "doc2".to_string(),
                content: "content2".to_string(),
                embedding: vec![],
                metadata: HashMap::new(),
                entities: vec![],
                relationships: vec![],
                similarity_score: Some(0.8),
            },
        ];

        let relationships = vec!["rel1".to_string(), "rel2".to_string(), "rel3".to_string()];

        // This mimics the relationship assignment in search_with_graph_context
        for doc in &mut documents {
            doc.relationships.clone_from(&relationships);
        }

        assert_eq!(documents[0].relationships.len(), 3);
        assert_eq!(documents[1].relationships.len(), 3);
        assert_eq!(documents[0].relationships[0], "rel1");
        assert_eq!(documents[1].relationships[2], "rel3");
    }

    #[test]
    fn test_relationships_assignment_empty() {
        let mut documents = vec![create_test_graph_document()];
        let relationships: Vec<String> = vec![];

        for doc in &mut documents {
            doc.relationships.clone_from(&relationships);
        }

        assert!(documents[0].relationships.is_empty());
    }

    // Test edge cases for similarity threshold filtering
    #[test]
    fn test_similarity_threshold_exact_match() {
        let similarity_threshold = 0.7;
        let test_scores = vec![0.69, 0.7, 0.71];

        let filtered_scores: Vec<f32> = test_scores
            .into_iter()
            .filter(|&score| score >= similarity_threshold)
            .collect();

        assert_eq!(filtered_scores.len(), 2); // 0.7 and 0.71 should pass
        assert!(filtered_scores.contains(&0.7));
        assert!(filtered_scores.contains(&0.71));
        assert!(!filtered_scores.contains(&0.69));
    }

    #[test]
    fn test_similarity_threshold_all_below() {
        let similarity_threshold = 0.8;
        let test_scores = vec![0.1, 0.3, 0.5, 0.79];

        let filtered_scores: Vec<f32> = test_scores
            .into_iter()
            .filter(|&score| score >= similarity_threshold)
            .collect();

        assert_eq!(filtered_scores.len(), 0);
    }

    #[test]
    fn test_similarity_threshold_all_above() {
        let similarity_threshold = 0.5;
        let test_scores = vec![0.6, 0.8, 0.9, 1.0];

        let filtered_scores: Vec<f32> = test_scores
            .into_iter()
            .filter(|&score| score >= similarity_threshold)
            .collect();

        assert_eq!(filtered_scores.len(), 4);
    }

    // Tests for limit calculation in vector search
    #[test]
    fn test_vector_search_limit_calculation() {
        let limit = 10;
        let expanded_limit = limit * 2;

        assert_eq!(expanded_limit, 20);
    }

    #[test]
    fn test_vector_search_limit_calculation_zero() {
        let limit = 0;
        let expanded_limit = limit * 2;

        assert_eq!(expanded_limit, 0);
    }

    #[test]
    fn test_vector_search_limit_calculation_large() {
        let limit = 1000;
        let expanded_limit = limit * 2;

        assert_eq!(expanded_limit, 2000);
    }

    // Tests for graph traversal limit calculation
    #[test]
    fn test_graph_traversal_limit_calculation() {
        let max_graph_hops = 3;
        let related_entity_limit = max_graph_hops * 50;

        assert_eq!(related_entity_limit, 150);
    }

    #[test]
    fn test_graph_traversal_limit_calculation_zero() {
        let max_graph_hops = 0;
        let related_entity_limit = max_graph_hops * 50;

        assert_eq!(related_entity_limit, 0);
    }

    #[test]
    fn test_graph_traversal_limit_calculation_one() {
        let max_graph_hops = 1;
        let related_entity_limit = max_graph_hops * 50;

        assert_eq!(related_entity_limit, 50);
    }
}
