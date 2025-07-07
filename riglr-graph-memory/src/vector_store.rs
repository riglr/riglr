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
    pub fn default() -> Self {
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
        let config = config.unwrap_or_else(GraphRetrieverConfig::default);

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
                    node.entities as entities, score
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
                                row_data[4].as_f64(),
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

                                    // Collect entities for graph expansion
                                    for entity in &entities {
                                        entity_set.insert(entity.clone());
                                    }

                                    documents.push(GraphDocument {
                                        id: id.to_string(),
                                        content: content.to_string(),
                                        embedding: query_embedding.to_vec(), // Placeholder
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
            for doc in &mut documents {
                doc.relationships = related_entities.clone();
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

// TODO: Implement rig::VectorStore trait once rig-core interface is clarified
// For now, providing the core vector store functionality through GraphRetriever methods

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

            match self
                .client
                .execute_query(create_doc_query, Some(params))
                .await
            {
                Ok(_) => {
                    document_ids.push(doc.id.clone());
                    debug!("Added document {} to graph", doc.id);
                }
                Err(e) => {
                    warn!("Failed to add document {}: {}", doc.id, e);
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
