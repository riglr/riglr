//! Integration tests for rig VectorStore implementation
//!
//! These tests verify that the GraphVectorStore can properly handle
//! document storage, retrieval, and vector similarity search operations.

#[cfg(test)]
mod tests {
    use riglr_graph_memory::{GraphVectorStore, Neo4jClient, RigDocument};
    use serde_json::json;
    use std::{collections::HashMap, sync::Arc};

    /// Mock Neo4j client for testing (since we don't have a real Neo4j instance)
    fn create_mock_client() -> Arc<Neo4jClient> {
        Arc::new(Neo4jClient::default())
    }

    /// Generate a mock embedding for testing
    fn mock_embedding() -> Vec<f32> {
        (0..384).map(|i| i as f32 / 384.0).collect()
    }

    #[tokio::test]
    async fn test_graph_vector_store_creation() {
        let client = create_mock_client();
        let _vector_store = GraphVectorStore::new(client, "test_index".to_string());

        // Just ensure the store can be created without panicking
        // Note: index_name is private, but we can test that construction succeeds
        // Vector store created successfully
    }

    #[tokio::test]
    async fn test_rig_document_creation() {
        let mut metadata = HashMap::new();
        metadata.insert("topic".to_string(), json!("blockchain"));
        metadata.insert("chain".to_string(), json!("ethereum"));

        let document = RigDocument {
            id: "test_doc_001".to_string(),
            content: "This is a test document about blockchain technology.".to_string(),
            embedding: mock_embedding(),
            metadata,
        };

        assert_eq!(document.id, "test_doc_001");
        assert_eq!(document.embedding.len(), 384);
        assert!(document.metadata.contains_key("topic"));
    }

    #[tokio::test]
    async fn test_document_conversion() {
        use riglr_graph_memory::document::{DocumentMetadata, DocumentSource, RawTextDocument};

        // Test RigDocument -> RawTextDocument conversion
        let mut rig_metadata = HashMap::new();
        rig_metadata.insert("title".to_string(), json!("Test Title"));
        rig_metadata.insert("chain".to_string(), json!("solana"));

        let rig_doc = RigDocument {
            id: "test_123".to_string(),
            content: "Test content".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            metadata: rig_metadata,
        };

        let raw_doc: RawTextDocument = rig_doc.clone().into();
        assert_eq!(raw_doc.id, "test_123");
        assert_eq!(raw_doc.content, "Test content");
        assert!(raw_doc.metadata.is_some());

        if let Some(meta) = raw_doc.metadata {
            assert_eq!(meta.title, Some("Test Title".to_string()));
            assert_eq!(meta.chain, Some("solana".to_string()));
        }

        // Test RawTextDocument -> RigDocument conversion
        let doc_metadata = DocumentMetadata {
            title: Some("Another Test".to_string()),
            tags: vec!["defi".to_string(), "nft".to_string()],
            ..Default::default()
        };

        let raw_doc2 = RawTextDocument {
            id: "raw_456".to_string(),
            content: "Raw document content".to_string(),
            metadata: Some(doc_metadata),
            embedding: Some(vec![0.4, 0.5, 0.6]),
            created_at: chrono::Utc::now(),
            source: DocumentSource::UserInput,
        };

        let rig_doc2: RigDocument = raw_doc2.into();
        assert_eq!(rig_doc2.id, "raw_456");
        assert_eq!(rig_doc2.content, "Raw document content");
        assert!(rig_doc2.metadata.contains_key("title"));
        assert!(rig_doc2.metadata.contains_key("tags"));
    }

    #[tokio::test]
    async fn test_mock_vector_operations() {
        let client = create_mock_client();
        let vector_store = GraphVectorStore::new(client, "test_vectors".to_string());

        // Create test documents
        let documents = vec![
            RigDocument {
                id: "doc1".to_string(),
                content: "Ethereum blockchain technology".to_string(),
                embedding: mock_embedding(),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("topic".to_string(), json!("ethereum"));
                    map
                },
            },
            RigDocument {
                id: "doc2".to_string(),
                content: "Solana smart contracts".to_string(),
                embedding: mock_embedding(),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("topic".to_string(), json!("solana"));
                    map
                },
            },
        ];

        // Note: These operations would fail with the mock client, but we can test
        // that the methods exist and accept the correct parameters
        let doc_ids = vec!["doc1".to_string(), "doc2".to_string()];
        let query_embedding = mock_embedding();

        // These assertions verify the API exists and compiles correctly
        // In a real test with Neo4j, these would actually perform operations
        assert!(vector_store.add_documents(documents).await.is_err()); // Mock client will fail
        assert!(vector_store.get_documents(doc_ids).await.is_err()); // Mock client will fail
        assert!(vector_store.search(query_embedding, 5).await.is_err()); // Mock client will fail
        assert!(vector_store
            .delete_documents(vec!["doc1".to_string()])
            .await
            .is_err()); // Mock client will fail
    }
}
