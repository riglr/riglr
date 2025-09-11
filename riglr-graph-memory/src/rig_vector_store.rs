// Implementation of rig::VectorStore trait for GraphVectorStore
// This module provides the bridge between riglr-graph-memory and the rig framework

use crate::{
    client::Neo4jClient,
    document::RawTextDocument,
    error::{GraphMemoryError, Result},
};
#[cfg(feature = "rig")]
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, sync::Arc};
use tracing::{debug, info, warn};

/// Document type that bridges between rig and our graph memory system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigDocument {
    /// Unique document identifier
    pub id: String,
    /// Main text content of the document
    pub content: String,
    /// Vector embedding representation of the document
    pub embedding: Vec<f32>,
    /// Additional metadata key-value pairs for the document
    pub metadata: HashMap<String, Value>,
}

/// Neo4j query response structure for proper deserialization
#[derive(Debug, Deserialize)]
struct Neo4jQueryResponse {
    results: Vec<Neo4jResult>,
    #[allow(dead_code)]
    errors: Vec<Neo4jError>,
}

#[derive(Debug, Deserialize)]
struct Neo4jResult {
    columns: Vec<String>,
    data: Vec<Neo4jDataRow>,
}

#[derive(Debug, Deserialize)]
struct Neo4jDataRow {
    row: Vec<Value>,
    #[allow(dead_code)]
    meta: Option<Vec<Value>>,
}

#[derive(Debug, Deserialize)]
struct Neo4jError {
    #[allow(dead_code)]
    code: String,
    #[allow(dead_code)]
    message: String,
}

impl Neo4jQueryResponse {
    /// Extract documents from query response
    fn extract_documents(&self) -> Vec<RigDocument> {
        let mut documents = Vec::new();

        for result in &self.results {
            // Find column indices
            let id_idx = result.columns.iter().position(|c| c == "id");
            let content_idx = result.columns.iter().position(|c| c == "content");
            let embedding_idx = result.columns.iter().position(|c| c == "embedding");
            let metadata_idx = result.columns.iter().position(|c| c == "metadata");

            for data_row in &result.data {
                if let Some(doc) = Self::parse_document_row(
                    &data_row.row,
                    id_idx,
                    content_idx,
                    embedding_idx,
                    metadata_idx,
                    None,
                ) {
                    documents.push(doc);
                }
            }
        }

        documents
    }

    /// Extract documents with scores from search results
    fn extract_documents_with_scores(&self) -> Vec<(RigDocument, f32)> {
        let mut results = Vec::new();

        for result in &self.results {
            // Find column indices
            let id_idx = result.columns.iter().position(|c| c == "id");
            let content_idx = result.columns.iter().position(|c| c == "content");
            let embedding_idx = result.columns.iter().position(|c| c == "embedding");
            let metadata_idx = result.columns.iter().position(|c| c == "metadata");
            let score_idx = result.columns.iter().position(|c| c == "score");

            for data_row in &result.data {
                if let Some(doc) = Self::parse_document_row(
                    &data_row.row,
                    id_idx,
                    content_idx,
                    embedding_idx,
                    metadata_idx,
                    score_idx,
                ) {
                    // Extract score if available
                    let score = score_idx
                        .and_then(|idx| data_row.row.get(idx))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as f32;

                    results.push((doc, score));
                }
            }
        }

        results
    }

    /// Parse a single document row
    fn parse_document_row(
        row: &[Value],
        id_idx: Option<usize>,
        content_idx: Option<usize>,
        embedding_idx: Option<usize>,
        metadata_idx: Option<usize>,
        _score_idx: Option<usize>,
    ) -> Option<RigDocument> {
        // Extract required fields
        let id = id_idx
            .and_then(|idx| row.get(idx))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())?;

        let content = content_idx
            .and_then(|idx| row.get(idx))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let embedding = embedding_idx
            .and_then(|idx| row.get(idx))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect()
            })
            .unwrap_or_default();

        let metadata = metadata_idx
            .and_then(|idx| row.get(idx))
            .and_then(|v| v.as_object())
            .map(|obj| obj.clone().into_iter().collect())
            .unwrap_or_default();

        Some(RigDocument {
            id,
            content,
            embedding,
            metadata,
        })
    }
}

/// Graph-based vector store implementation for rig
///
/// This implementation leverages Neo4j's vector capabilities combined with graph traversal
/// to provide enhanced contextual search for rig agents.
#[derive(Debug)]
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

        self.client.execute_query(&query, None).await.map_err(|e| {
            GraphMemoryError::Database(format!("Failed to create vector index: {}", e))
        })?;

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
                    return Err(GraphMemoryError::Database(format!(
                        "Failed to add document: {}",
                        e
                    )));
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

        let params = HashMap::from([("ids".to_string(), json!(ids))]);

        let result = self
            .client
            .execute_query(query, Some(params))
            .await
            .map_err(|e| GraphMemoryError::Database(format!("Failed to get documents: {}", e)))?;

        // Parse response using serde deserialization
        let response: Neo4jQueryResponse = serde_json::from_value(result)?;

        let documents = response.extract_documents();

        debug!("Retrieved {} documents", documents.len());
        Ok(documents)
    }

    /// Search for similar documents using vector similarity
    pub async fn search(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<(RigDocument, f32)>> {
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

        let result = self
            .client
            .execute_query(&search_query, Some(params))
            .await
            .map_err(|e| {
                GraphMemoryError::Database(format!("Failed to perform vector search: {}", e))
            })?;

        // Parse response using serde deserialization
        let response: Neo4jQueryResponse = serde_json::from_value(result)?;

        let search_results = response.extract_documents_with_scores();

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

        let params = HashMap::from([("ids".to_string(), json!(ids.clone()))]);

        self.client
            .execute_query(query, Some(params))
            .await
            .map_err(|e| {
                GraphMemoryError::Database(format!("Failed to delete documents: {}", e))
            })?;

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
    /// The document type that this vector store manages
    type Document;
    /// The error type that vector store operations can return
    type Error: std::error::Error + Send + Sync + 'static;

    /// Add documents to the vector store
    async fn add_documents(
        &self,
        documents: Vec<Self::Document>,
    ) -> std::result::Result<Vec<String>, Self::Error>;

    /// Search for similar documents using vector similarity
    async fn search(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
    ) -> std::result::Result<Vec<(Self::Document, f32)>, Self::Error>;

    /// Get documents by their IDs
    async fn get_documents(
        &self,
        ids: Vec<String>,
    ) -> std::result::Result<Vec<Self::Document>, Self::Error>;

    /// Delete documents by their IDs
    async fn delete_documents(
        &self,
        ids: Vec<String>,
    ) -> std::result::Result<Vec<String>, Self::Error>;
}

#[cfg(feature = "rig")]
#[async_trait]
impl VectorStore for GraphVectorStore {
    type Document = RigDocument;
    type Error = GraphMemoryError;

    async fn add_documents(
        &self,
        documents: Vec<Self::Document>,
    ) -> std::result::Result<Vec<String>, Self::Error> {
        self.add_documents(documents).await
    }

    async fn search(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
    ) -> std::result::Result<Vec<(Self::Document, f32)>, Self::Error> {
        self.search(query_embedding, limit).await
    }

    async fn get_documents(
        &self,
        ids: Vec<String>,
    ) -> std::result::Result<Vec<Self::Document>, Self::Error> {
        self.get_documents(ids).await
    }

    async fn delete_documents(
        &self,
        ids: Vec<String>,
    ) -> std::result::Result<Vec<String>, Self::Error> {
        self.delete_documents(ids).await
    }
}

// Conversion between our internal document types and the rig document type
impl From<RawTextDocument> for RigDocument {
    fn from(raw_doc: RawTextDocument) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), json!(format!("{:?}", raw_doc.source)));
        metadata.insert(
            "created_at".to_string(),
            json!(raw_doc.created_at.to_rfc3339()),
        );

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
        let created_at = rig_doc
            .metadata
            .get("created_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map_or_else(chrono::Utc::now, |dt| dt.with_timezone(&chrono::Utc));

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
                doc_meta.tags = tags
                    .iter()
                    .filter_map(|t| t.as_str())
                    .map(|s| s.to_string())
                    .collect();
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
            embedding: if rig_doc.embedding.is_empty() {
                None
            } else {
                Some(rig_doc.embedding)
            },
            metadata,
            created_at,
            source: crate::document::DocumentSource::UserInput, // Default
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
    use std::sync::Arc;

    fn create_test_rig_document() -> RigDocument {
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), json!("Test Document"));
        metadata.insert("tags".to_string(), json!(["test", "document"]));

        RigDocument {
            id: "test-doc-1".to_string(),
            content: "This is test content".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            metadata,
        }
    }

    fn create_test_raw_document() -> RawTextDocument {
        let mut metadata = DocumentMetadata::default();
        metadata.title = Some("Test Title".to_string());
        metadata.tags = vec!["tag1".to_string(), "tag2".to_string()];
        metadata.chain = Some("ethereum".to_string());
        metadata
            .custom_fields
            .insert("custom".to_string(), json!("value"));

        RawTextDocument {
            id: "raw-doc-1".to_string(),
            content: "Raw document content".to_string(),
            embedding: Some(vec![0.4, 0.5, 0.6]),
            metadata: Some(metadata),
            created_at: Utc::now(),
            source: DocumentSource::UserInput,
        }
    }

    // Tests for RigDocument
    #[test]
    fn test_rig_document_creation() {
        let doc = create_test_rig_document();
        assert_eq!(doc.id, "test-doc-1");
        assert_eq!(doc.content, "This is test content");
        assert_eq!(doc.embedding, vec![0.1, 0.2, 0.3]);
        assert!(doc.metadata.contains_key("title"));
    }

    #[test]
    fn test_rig_document_empty_fields() {
        let doc = RigDocument {
            id: "".to_string(),
            content: "".to_string(),
            embedding: vec![],
            metadata: HashMap::new(),
        };
        assert!(doc.id.is_empty());
        assert!(doc.content.is_empty());
        assert!(doc.embedding.is_empty());
        assert!(doc.metadata.is_empty());
    }

    // Tests for Neo4jQueryResponse
    #[test]
    fn test_neo4j_query_response_extract_documents_empty_results() {
        let response = Neo4jQueryResponse {
            results: vec![],
            errors: vec![],
        };
        let documents = response.extract_documents();
        assert!(documents.is_empty());
    }

    #[test]
    fn test_neo4j_query_response_extract_documents_valid_data() {
        let response = Neo4jQueryResponse {
            results: vec![Neo4jResult {
                columns: vec![
                    "id".to_string(),
                    "content".to_string(),
                    "embedding".to_string(),
                    "metadata".to_string(),
                ],
                data: vec![Neo4jDataRow {
                    row: vec![
                        json!("doc-1"),
                        json!("content text"),
                        json!([0.1, 0.2, 0.3]),
                        json!({"title": "Test"}),
                    ],
                    meta: None,
                }],
            }],
            errors: vec![],
        };

        let documents = response.extract_documents();
        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0].id, "doc-1");
        assert_eq!(documents[0].content, "content text");
        assert_eq!(documents[0].embedding, vec![0.1, 0.2, 0.3]);
    }

    #[test]
    fn test_neo4j_query_response_extract_documents_missing_columns() {
        let response = Neo4jQueryResponse {
            results: vec![Neo4jResult {
                columns: vec!["other".to_string()],
                data: vec![Neo4jDataRow {
                    row: vec![json!("value")],
                    meta: None,
                }],
            }],
            errors: vec![],
        };

        let documents = response.extract_documents();
        assert!(documents.is_empty());
    }

    #[test]
    fn test_neo4j_query_response_extract_documents_with_scores_empty() {
        let response = Neo4jQueryResponse {
            results: vec![],
            errors: vec![],
        };
        let results = response.extract_documents_with_scores();
        assert!(results.is_empty());
    }

    #[test]
    fn test_neo4j_query_response_extract_documents_with_scores_valid() {
        let response = Neo4jQueryResponse {
            results: vec![Neo4jResult {
                columns: vec![
                    "id".to_string(),
                    "content".to_string(),
                    "embedding".to_string(),
                    "metadata".to_string(),
                    "score".to_string(),
                ],
                data: vec![Neo4jDataRow {
                    row: vec![
                        json!("doc-1"),
                        json!("content"),
                        json!([0.1, 0.2]),
                        json!({}),
                        json!(0.85),
                    ],
                    meta: None,
                }],
            }],
            errors: vec![],
        };

        let results = response.extract_documents_with_scores();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, 0.85);
    }

    #[test]
    fn test_neo4j_query_response_extract_documents_with_scores_missing_score() {
        let response = Neo4jQueryResponse {
            results: vec![Neo4jResult {
                columns: vec![
                    "id".to_string(),
                    "content".to_string(),
                    "embedding".to_string(),
                    "metadata".to_string(),
                ],
                data: vec![Neo4jDataRow {
                    row: vec![
                        json!("doc-1"),
                        json!("content"),
                        json!([0.1, 0.2]),
                        json!({}),
                    ],
                    meta: None,
                }],
            }],
            errors: vec![],
        };

        let results = response.extract_documents_with_scores();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, 0.0); // Default score when missing
    }

    #[test]
    fn test_parse_document_row_all_none_indices() {
        let row = vec![json!("value1"), json!("value2")];
        let result = Neo4jQueryResponse::parse_document_row(&row, None, None, None, None, None);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_document_row_valid_id_only() {
        let row = vec![
            json!("doc-id"),
            json!("content"),
            json!([0.1, 0.2]),
            json!({}),
        ];
        let result =
            Neo4jQueryResponse::parse_document_row(&row, Some(0), Some(1), Some(2), Some(3), None);
        assert!(result.is_some());
        let doc = result.unwrap();
        assert_eq!(doc.id, "doc-id");
        assert_eq!(doc.content, "content");
        assert_eq!(doc.embedding, vec![0.1, 0.2]);
    }

    #[test]
    fn test_parse_document_row_missing_content() {
        let row = vec![json!("doc-id")];
        let result =
            Neo4jQueryResponse::parse_document_row(&row, Some(0), Some(1), None, None, None);
        assert!(result.is_some());
        let doc = result.unwrap();
        assert_eq!(doc.content, ""); // Default empty content
    }

    #[test]
    fn test_parse_document_row_invalid_embedding() {
        let row = vec![
            json!("doc-id"),
            json!("content"),
            json!("not-an-array"),
            json!({}),
        ];
        let result =
            Neo4jQueryResponse::parse_document_row(&row, Some(0), Some(1), Some(2), Some(3), None);
        assert!(result.is_some());
        let doc = result.unwrap();
        assert!(doc.embedding.is_empty()); // Default empty embedding
    }

    #[test]
    fn test_parse_document_row_invalid_metadata() {
        let row = vec![
            json!("doc-id"),
            json!("content"),
            json!([0.1]),
            json!("not-an-object"),
        ];
        let result =
            Neo4jQueryResponse::parse_document_row(&row, Some(0), Some(1), Some(2), Some(3), None);
        assert!(result.is_some());
        let doc = result.unwrap();
        assert!(doc.metadata.is_empty()); // Default empty metadata
    }

    // Tests for GraphVectorStore creation
    #[test]
    fn test_graph_vector_store_new() {
        let client = Arc::new(crate::client::Neo4jClient::default());
        let store = GraphVectorStore::new(client, "test_index".to_string());
        assert_eq!(store.index_name, "test_index");
        assert_eq!(store.embedding_dimension, 1536); // Default dimension
    }

    #[test]
    fn test_graph_vector_store_with_dimension() {
        let client = Arc::new(crate::client::Neo4jClient::default());
        let store = GraphVectorStore::with_dimension(client, "test_index".to_string(), 768);
        assert_eq!(store.index_name, "test_index");
        assert_eq!(store.embedding_dimension, 768);
    }

    #[test]
    fn test_graph_vector_store_with_zero_dimension() {
        let client = Arc::new(crate::client::Neo4jClient::default());
        let store = GraphVectorStore::with_dimension(client, "test_index".to_string(), 0);
        assert_eq!(store.embedding_dimension, 0);
    }

    #[test]
    fn test_graph_vector_store_empty_index_name() {
        let client = Arc::new(crate::client::Neo4jClient::default());
        let store = GraphVectorStore::new(client, "".to_string());
        assert_eq!(store.index_name, "");
    }

    // Tests for From implementations
    #[test]
    fn test_from_raw_text_document_to_rig_document_with_metadata() {
        let raw_doc = create_test_raw_document();
        let rig_doc: RigDocument = raw_doc.into();

        assert_eq!(rig_doc.id, "raw-doc-1");
        assert_eq!(rig_doc.content, "Raw document content");
        assert_eq!(rig_doc.embedding, vec![0.4, 0.5, 0.6]);
        assert!(rig_doc.metadata.contains_key("title"));
        assert!(rig_doc.metadata.contains_key("tags"));
        assert!(rig_doc.metadata.contains_key("chain"));
        assert!(rig_doc.metadata.contains_key("custom"));
    }

    #[test]
    fn test_from_raw_text_document_to_rig_document_no_metadata() {
        let raw_doc = RawTextDocument {
            id: "simple-doc".to_string(),
            content: "Simple content".to_string(),
            embedding: None,
            metadata: None,
            created_at: Utc::now(),
            source: DocumentSource::UserInput,
        };

        let rig_doc: RigDocument = raw_doc.into();
        assert_eq!(rig_doc.id, "simple-doc");
        assert_eq!(rig_doc.content, "Simple content");
        assert!(rig_doc.embedding.is_empty());
        assert!(rig_doc.metadata.contains_key("source"));
        assert!(rig_doc.metadata.contains_key("created_at"));
    }

    #[test]
    fn test_from_raw_text_document_to_rig_document_empty_tags() {
        let mut metadata = DocumentMetadata::default();
        metadata.tags = vec![]; // Empty tags

        let raw_doc = RawTextDocument {
            id: "doc-empty-tags".to_string(),
            content: "Content".to_string(),
            embedding: Some(vec![0.1]),
            metadata: Some(metadata),
            created_at: Utc::now(),
            source: DocumentSource::UserInput,
        };

        let rig_doc: RigDocument = raw_doc.into();
        // Empty tags should not be added to metadata
        assert!(!rig_doc.metadata.contains_key("tags"));
    }

    #[test]
    fn test_from_rig_document_to_raw_text_document_with_metadata() {
        let rig_doc = create_test_rig_document();
        let raw_doc: RawTextDocument = rig_doc.into();

        assert_eq!(raw_doc.id, "test-doc-1");
        assert_eq!(raw_doc.content, "This is test content");
        assert_eq!(raw_doc.embedding, Some(vec![0.1, 0.2, 0.3]));
        assert!(raw_doc.metadata.is_some());

        let metadata = raw_doc.metadata.unwrap();
        assert_eq!(metadata.title, Some("Test Document".to_string()));
        assert_eq!(metadata.tags, vec!["test", "document"]);
    }

    #[test]
    fn test_from_rig_document_to_raw_text_document_empty_metadata() {
        let rig_doc = RigDocument {
            id: "simple".to_string(),
            content: "Simple".to_string(),
            embedding: vec![],
            metadata: HashMap::new(),
        };

        let raw_doc: RawTextDocument = rig_doc.into();
        assert_eq!(raw_doc.embedding, None); // Empty embedding becomes None
        assert!(raw_doc.metadata.is_none()); // Empty metadata becomes None
    }

    #[test]
    fn test_from_rig_document_to_raw_text_document_invalid_created_at() {
        let mut metadata = HashMap::new();
        metadata.insert("created_at".to_string(), json!("invalid-date"));

        let rig_doc = RigDocument {
            id: "test".to_string(),
            content: "content".to_string(),
            embedding: vec![0.1],
            metadata,
        };

        let raw_doc: RawTextDocument = rig_doc.into();
        // Should use current time when created_at is invalid
        assert!(raw_doc.created_at <= Utc::now());
    }

    #[test]
    fn test_from_rig_document_to_raw_text_document_custom_fields() {
        let mut metadata = HashMap::new();
        metadata.insert("custom1".to_string(), json!("value1"));
        metadata.insert("custom2".to_string(), json!(42));
        metadata.insert("title".to_string(), json!("Title")); // This should NOT go to custom_fields

        let rig_doc = RigDocument {
            id: "test".to_string(),
            content: "content".to_string(),
            embedding: vec![0.1],
            metadata,
        };

        let raw_doc: RawTextDocument = rig_doc.into();
        let doc_metadata = raw_doc.metadata.unwrap();

        assert_eq!(doc_metadata.title, Some("Title".to_string()));
        assert!(doc_metadata.custom_fields.contains_key("custom1"));
        assert!(doc_metadata.custom_fields.contains_key("custom2"));
        assert!(!doc_metadata.custom_fields.contains_key("title")); // Should not be in custom_fields
    }

    #[test]
    fn test_from_rig_document_to_raw_text_document_invalid_tags() {
        let mut metadata = HashMap::new();
        metadata.insert("tags".to_string(), json!([123, true, null])); // Invalid tag types

        let rig_doc = RigDocument {
            id: "test".to_string(),
            content: "content".to_string(),
            embedding: vec![],
            metadata,
        };

        let raw_doc: RawTextDocument = rig_doc.into();
        let doc_metadata = raw_doc.metadata.unwrap();
        assert!(doc_metadata.tags.is_empty()); // Invalid tags should be filtered out
    }

    // Additional edge case tests
    #[test]
    fn test_rig_document_clone() {
        let doc = create_test_rig_document();
        let cloned = doc.clone();
        assert_eq!(doc.id, cloned.id);
        assert_eq!(doc.content, cloned.content);
        assert_eq!(doc.embedding, cloned.embedding);
    }

    #[test]
    fn test_rig_document_debug() {
        let doc = create_test_rig_document();
        let debug_str = format!("{:?}", doc);
        assert!(debug_str.contains("RigDocument"));
        assert!(debug_str.contains("test-doc-1"));
    }

    #[test]
    fn test_neo4j_error_struct() {
        let error = Neo4jError {
            code: "Neo.ClientError.Statement.SyntaxError".to_string(),
            message: "Invalid syntax".to_string(),
        };
        assert_eq!(error.code, "Neo.ClientError.Statement.SyntaxError");
        assert_eq!(error.message, "Invalid syntax");
    }

    #[test]
    fn test_neo4j_data_row_with_meta() {
        let row = Neo4jDataRow {
            row: vec![json!("value")],
            meta: Some(vec![json!("meta_value")]),
        };
        assert_eq!(row.row.len(), 1);
        assert!(row.meta.is_some());
    }

    #[test]
    fn test_neo4j_result_empty_columns() {
        let result = Neo4jResult {
            columns: vec![],
            data: vec![],
        };
        assert!(result.columns.is_empty());
        assert!(result.data.is_empty());
    }

    #[test]
    fn test_parse_document_row_index_out_of_bounds() {
        let row = vec![json!("value1")];
        // Try to access index 5 when row only has 1 element
        let result = Neo4jQueryResponse::parse_document_row(&row, Some(5), None, None, None, None);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_document_row_non_string_id() {
        let row = vec![json!(123), json!("content")]; // ID is number, not string
        let result =
            Neo4jQueryResponse::parse_document_row(&row, Some(0), Some(1), None, None, None);
        assert!(result.is_none()); // Should return None because ID must be string
    }

    #[test]
    fn test_extract_documents_multiple_results() {
        let response = Neo4jQueryResponse {
            results: vec![
                Neo4jResult {
                    columns: vec!["id".to_string(), "content".to_string()],
                    data: vec![Neo4jDataRow {
                        row: vec![json!("doc-1"), json!("content-1")],
                        meta: None,
                    }],
                },
                Neo4jResult {
                    columns: vec!["id".to_string(), "content".to_string()],
                    data: vec![Neo4jDataRow {
                        row: vec![json!("doc-2"), json!("content-2")],
                        meta: None,
                    }],
                },
            ],
            errors: vec![],
        };

        let documents = response.extract_documents();
        assert_eq!(documents.len(), 2);
        assert_eq!(documents[0].id, "doc-1");
        assert_eq!(documents[1].id, "doc-2");
    }

    #[test]
    fn test_extract_documents_multiple_data_rows() {
        let response = Neo4jQueryResponse {
            results: vec![Neo4jResult {
                columns: vec!["id".to_string(), "content".to_string()],
                data: vec![
                    Neo4jDataRow {
                        row: vec![json!("doc-1"), json!("content-1")],
                        meta: None,
                    },
                    Neo4jDataRow {
                        row: vec![json!("doc-2"), json!("content-2")],
                        meta: None,
                    },
                ],
            }],
            errors: vec![],
        };

        let documents = response.extract_documents();
        assert_eq!(documents.len(), 2);
    }

    #[test]
    fn test_rig_document_serialization() {
        let doc = create_test_rig_document();
        let serialized = serde_json::to_string(&doc);
        assert!(serialized.is_ok());

        let deserialized: std::result::Result<RigDocument, _> =
            serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());
        let deserialized_doc = deserialized.unwrap();
        assert_eq!(doc.id, deserialized_doc.id);
    }

    #[test]
    fn test_embedding_conversion_mixed_types() {
        let row = vec![
            json!("doc-1"),
            json!("content"),
            json!([1, 2.5, "3", null, true]),
            json!({}),
        ];
        let result =
            Neo4jQueryResponse::parse_document_row(&row, Some(0), Some(1), Some(2), Some(3), None);
        assert!(result.is_some());
        let doc = result.unwrap();
        // Should only convert valid f64 values
        assert_eq!(doc.embedding, vec![1.0, 2.5]); // Only valid numbers are converted
    }

    // Additional tests for comprehensive coverage
    #[test]
    fn test_graph_vector_store_large_embedding_dimension() {
        let client = Arc::new(crate::client::Neo4jClient::default());
        let store = GraphVectorStore::with_dimension(client, "test_index".to_string(), usize::MAX);
        assert_eq!(store.embedding_dimension, usize::MAX);
    }

    #[test]
    fn test_graph_vector_store_special_characters_in_index_name() {
        let client = Arc::new(crate::client::Neo4jClient::default());
        let store = GraphVectorStore::new(client, "test_index_with_special-chars.123".to_string());
        assert_eq!(store.index_name, "test_index_with_special-chars.123");
    }
}
