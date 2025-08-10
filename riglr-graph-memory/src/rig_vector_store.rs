// Implementation of rig::VectorStore trait for GraphVectorStore
// This module provides the bridge between riglr-graph-memory and the rig framework

use crate::{
    client::Neo4jClient,
    document::RawTextDocument,
    error::{GraphMemoryError, Result},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, sync::Arc};
use tracing::{debug, info, warn};

/// Document type that bridges between rig and our graph memory system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigDocument {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: HashMap<String, Value>,
}

/// Graph-based vector store implementation for rig
/// 
/// This implementation leverages Neo4j's vector capabilities combined with graph traversal
/// to provide enhanced contextual search for rig agents.
pub struct GraphVectorStore {
    client: Arc<Neo4jClient>,
    index_name: String,
    embedding_dimension: usize,
}

impl GraphVectorStore {
    /// Create a new GraphVectorStore with Neo4j client
    pub fn new(client: Arc<Neo4jClient>, index_name: String) -> Self {
        Self { 
            client, 
            index_name,
            embedding_dimension: 1536, // Default OpenAI embedding dimension
        }
    }
    
    /// Create GraphVectorStore with custom embedding dimension
    pub fn with_dimension(client: Arc<Neo4jClient>, index_name: String, dimension: usize) -> Self {
        Self {
            client,
            index_name,
            embedding_dimension: dimension,
        }
    }
    
    /// Initialize vector index in Neo4j
    async fn ensure_vector_index(&self) -> Result<()> {
        let query = format!(
            "CREATE VECTOR INDEX IF NOT EXISTS {} FOR (d:Document) ON (d.embedding) 
             OPTIONS {{indexConfig: {{`vector.dimensions`: {}, `vector.similarity_function`: 'cosine'}}}}",
            self.index_name,
            self.embedding_dimension
        );
        
        self.client.execute_query(&query, None).await
            .map_err(|e| GraphMemoryError::Database(format!("Failed to create vector index: {}", e)))?;
        
        Ok(())
    }
}

impl GraphVectorStore {
    /// Add documents to the graph vector store (direct implementation)
    pub async fn add_documents(&self, documents: Vec<RigDocument>) -> Result<Vec<String>> {
        debug!("Adding {} documents to GraphVectorStore", documents.len());
        
        // Ensure vector index exists
        self.ensure_vector_index().await?;
        
        let mut document_ids = Vec::new();
        
        for doc in documents {
            let query = r#"
                CREATE (d:Document {
                    id: $id,
                    content: $content,
                    embedding: $embedding,
                    metadata: $metadata,
                    created_at: datetime()
                })
                RETURN d.id as id
            "#;
            
            let params = HashMap::from([
                ("id".to_string(), json!(doc.id)),
                ("content".to_string(), json!(doc.content)),
                ("embedding".to_string(), json!(doc.embedding)),
                ("metadata".to_string(), json!(doc.metadata)),
            ]);
            
            match self.client.execute_query(query, Some(params)).await {
                Ok(_) => {
                    document_ids.push(doc.id.clone());
                    debug!("Successfully added document {}", doc.id);
                }
                Err(e) => {
                    warn!("Failed to add document {}: {}", doc.id, e);
                    return Err(GraphMemoryError::Database(format!("Failed to add document: {}", e)));
                }
            }
        }
        
        info!("Successfully added {} documents", document_ids.len());
        Ok(document_ids)
    }

    /// Get documents by their IDs
    pub async fn get_documents(&self, ids: Vec<String>) -> Result<Vec<RigDocument>> {
        debug!("Getting {} documents by ID", ids.len());
        
        let query = r#"
            UNWIND $ids as doc_id
            MATCH (d:Document {id: doc_id})
            RETURN d.id as id, d.content as content, d.embedding as embedding, d.metadata as metadata
        "#;
        
        let params = HashMap::from([
            ("ids".to_string(), json!(ids)),
        ]);
        
        let result = self.client.execute_query(query, Some(params)).await
            .map_err(|e| GraphMemoryError::Database(format!("Failed to get documents: {}", e)))?;
        
        let mut documents = Vec::new();
        
        // Parse Neo4j response (this would need to be adjusted based on actual Neo4j client response format)
        if let Some(results) = result.get("results").and_then(|r| r.as_array()) {
            for result in results {
                if let Some(data) = result.get("data").and_then(|d| d.as_array()) {
                    for row in data {
                        if let Some(row_data) = row.get("row").and_then(|r| r.as_array()) {
                            if row_data.len() >= 4 {
                                if let (Some(id), Some(content), Some(embedding)) = (
                                    row_data[0].as_str(),
                                    row_data[1].as_str(),
                                    row_data[2].as_array(),
                                ) {
                                    let metadata = &row_data[3];
                                    let embedding_vec: Vec<f32> = embedding
                                        .iter()
                                        .filter_map(|v| v.as_f64().map(|f| f as f32))
                                        .collect();
                                    
                                    let metadata_map: HashMap<String, Value> = if metadata.is_object() {
                                        metadata.as_object().unwrap().clone().into_iter().collect()
                                    } else {
                                        HashMap::new()
                                    };
                                    
                                    documents.push(RigDocument {
                                        id: id.to_string(),
                                        content: content.to_string(),
                                        embedding: embedding_vec,
                                        metadata: metadata_map,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        
        debug!("Retrieved {} documents", documents.len());
        Ok(documents)
    }

    /// Search for similar documents using vector similarity
    pub async fn search(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<(RigDocument, f32)>> {
        debug!("Performing vector search with limit {}", limit);
        
        let search_query = format!(
            "CALL db.index.vector.queryNodes('{}', $limit, $query_embedding) 
             YIELD node, score
             RETURN node.id as id, node.content as content, node.embedding as embedding, 
                    node.metadata as metadata, score
             ORDER BY score DESC",
            self.index_name
        );
        
        let params = HashMap::from([
            ("query_embedding".to_string(), json!(query_embedding)),
            ("limit".to_string(), json!(limit)),
        ]);
        
        let result = self.client.execute_query(&search_query, Some(params)).await
            .map_err(|e| GraphMemoryError::Database(format!("Failed to perform vector search: {}", e)))?;
        
        let mut search_results = Vec::new();
        
        // Parse search results (adjusted based on Neo4j response format)
        if let Some(results) = result.get("results").and_then(|r| r.as_array()) {
            for result in results {
                if let Some(data) = result.get("data").and_then(|d| d.as_array()) {
                    for row in data {
                        if let Some(row_data) = row.get("row").and_then(|r| r.as_array()) {
                            if row_data.len() >= 5 {
                                if let (Some(id), Some(content), Some(embedding), Some(score)) = (
                                    row_data[0].as_str(),
                                    row_data[1].as_str(),
                                    row_data[2].as_array(),
                                    row_data[4].as_f64(),
                                ) {
                                    let embedding_vec: Vec<f32> = embedding
                                        .iter()
                                        .filter_map(|v| v.as_f64().map(|f| f as f32))
                                        .collect();
                                    
                                    let metadata_map: HashMap<String, Value> = if let Some(metadata) = row_data[3].as_object() {
                                        metadata.clone().into_iter().collect()
                                    } else {
                                        HashMap::new()
                                    };
                                    
                                    let document = RigDocument {
                                        id: id.to_string(),
                                        content: content.to_string(),
                                        embedding: embedding_vec,
                                        metadata: metadata_map,
                                    };
                                    
                                    search_results.push((document, score as f32));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        info!("Vector search returned {} results", search_results.len());
        Ok(search_results)
    }

    /// Delete documents by their IDs
    pub async fn delete_documents(&self, ids: Vec<String>) -> Result<Vec<String>> {
        debug!("Deleting {} documents", ids.len());
        
        let query = r#"
            UNWIND $ids as doc_id
            MATCH (d:Document {id: doc_id})
            DELETE d
            RETURN doc_id
        "#;
        
        let params = HashMap::from([
            ("ids".to_string(), json!(ids.clone())),
        ]);
        
        self.client.execute_query(query, Some(params)).await
            .map_err(|e| GraphMemoryError::Database(format!("Failed to delete documents: {}", e)))?;
        
        info!("Successfully deleted {} documents", ids.len());
        Ok(ids)
    }
}

// Implementation of VectorStore trait for integration with rig framework
// This provides a bridge between riglr-graph-memory and rig agents

/// Trait for vector stores that can be used with rig agents
/// This trait defines the interface for storing and retrieving documents with embeddings
#[cfg(feature = "rig")]
#[async_trait]
pub trait VectorStore: Send + Sync {
    type Document;
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Add documents to the vector store
    async fn add_documents(&self, documents: Vec<Self::Document>) -> std::result::Result<Vec<String>, Self::Error>;
    
    /// Search for similar documents using vector similarity
    async fn search(&self, query_embedding: Vec<f32>, limit: usize) -> std::result::Result<Vec<(Self::Document, f32)>, Self::Error>;
    
    /// Get documents by their IDs
    async fn get_documents(&self, ids: Vec<String>) -> std::result::Result<Vec<Self::Document>, Self::Error>;
    
    /// Delete documents by their IDs
    async fn delete_documents(&self, ids: Vec<String>) -> std::result::Result<Vec<String>, Self::Error>;
}

#[cfg(feature = "rig")]
#[async_trait]
impl VectorStore for GraphVectorStore {
    type Document = RigDocument;
    type Error = GraphMemoryError;
    
    async fn add_documents(&self, documents: Vec<Self::Document>) -> std::result::Result<Vec<String>, Self::Error> {
        self.add_documents(documents).await
    }
    
    async fn search(&self, query_embedding: Vec<f32>, limit: usize) -> std::result::Result<Vec<(Self::Document, f32)>, Self::Error> {
        self.search(query_embedding, limit).await
    }
    
    async fn get_documents(&self, ids: Vec<String>) -> std::result::Result<Vec<Self::Document>, Self::Error> {
        self.get_documents(ids).await
    }
    
    async fn delete_documents(&self, ids: Vec<String>) -> std::result::Result<Vec<String>, Self::Error> {
        self.delete_documents(ids).await
    }
}

// Conversion between our internal document types and the rig document type
impl From<RawTextDocument> for RigDocument {
    fn from(raw_doc: RawTextDocument) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), json!(format!("{:?}", raw_doc.source)));
        metadata.insert("created_at".to_string(), json!(raw_doc.created_at.to_rfc3339()));
        
        if let Some(meta) = raw_doc.metadata {
            // Add custom metadata if present
            if let Some(title) = meta.title {
                metadata.insert("title".to_string(), json!(title));
            }
            if !meta.tags.is_empty() {
                metadata.insert("tags".to_string(), json!(meta.tags));
            }
            if let Some(chain) = meta.chain {
                metadata.insert("chain".to_string(), json!(chain));
            }
            // Add other metadata fields as needed
            for (key, value) in meta.custom_fields {
                metadata.insert(key, value);
            }
        }
        
        Self {
            id: raw_doc.id,
            content: raw_doc.content,
            embedding: raw_doc.embedding.unwrap_or_default(),
            metadata,
        }
    }
}

impl From<RigDocument> for RawTextDocument {
    fn from(rig_doc: RigDocument) -> Self {
        let created_at = rig_doc.metadata
            .get("created_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);
        
        // Convert metadata back to DocumentMetadata format
        let metadata = if rig_doc.metadata.is_empty() {
            None
        } else {
            use crate::document::DocumentMetadata;
            let mut doc_meta = DocumentMetadata::default();
            
            if let Some(title) = rig_doc.metadata.get("title").and_then(|v| v.as_str()) {
                doc_meta.title = Some(title.to_string());
            }
            if let Some(tags) = rig_doc.metadata.get("tags").and_then(|v| v.as_array()) {
                doc_meta.tags = tags.iter().filter_map(|t| t.as_str()).map(|s| s.to_string()).collect();
            }
            if let Some(chain) = rig_doc.metadata.get("chain").and_then(|v| v.as_str()) {
                doc_meta.chain = Some(chain.to_string());
            }
            
            // Add other fields to custom_fields
            for (key, value) in &rig_doc.metadata {
                if !["title", "tags", "chain", "source", "created_at"].contains(&key.as_str()) {
                    doc_meta.custom_fields.insert(key.clone(), value.clone());
                }
            }
            
            Some(doc_meta)
        };
        
        Self {
            id: rig_doc.id,
            content: rig_doc.content,
            embedding: if rig_doc.embedding.is_empty() { None } else { Some(rig_doc.embedding) },
            metadata,
            created_at,
            source: crate::document::DocumentSource::UserInput, // Default
        }
    }
}