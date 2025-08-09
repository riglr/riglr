//! Comprehensive tests for error module

use riglr_graph_memory::error::{GraphMemoryError, Result};
use riglr_core::CoreError;

#[test]
fn test_database_error() {
    let error = GraphMemoryError::Database("Connection refused".to_string());
    assert_eq!(error.to_string(), "Database error: Connection refused");
    
    let error2 = GraphMemoryError::Database("Authentication failed".to_string());
    assert_eq!(error2.to_string(), "Database error: Authentication failed");
}

#[test]
fn test_query_error() {
    let error = GraphMemoryError::Query("Invalid Cypher syntax".to_string());
    assert_eq!(error.to_string(), "Query error: Invalid Cypher syntax");
    
    let error2 = GraphMemoryError::Query("Node not found".to_string());
    assert_eq!(error2.to_string(), "Query error: Node not found");
}

#[test]
fn test_entity_extraction_error() {
    let error = GraphMemoryError::EntityExtraction("Failed to parse text".to_string());
    assert_eq!(error.to_string(), "Entity extraction error: Failed to parse text");
    
    let error2 = GraphMemoryError::EntityExtraction("No entities found".to_string());
    assert_eq!(error2.to_string(), "Entity extraction error: No entities found");
}

#[test]
fn test_embedding_error() {
    let error = GraphMemoryError::Embedding("Model not available".to_string());
    assert_eq!(error.to_string(), "Embedding error: Model not available");
    
    let error2 = GraphMemoryError::Embedding("Text too long".to_string());
    assert_eq!(error2.to_string(), "Embedding error: Text too long");
}

#[test]
fn test_generic_error() {
    let error = GraphMemoryError::Generic("Unexpected failure".to_string());
    assert_eq!(error.to_string(), "Graph memory error: Unexpected failure");
    
    let error2 = GraphMemoryError::Generic("Operation cancelled".to_string());
    assert_eq!(error2.to_string(), "Graph memory error: Operation cancelled");
}

#[test]
fn test_serialization_error_from_json() {
    let invalid_json = "{ broken json";
    let json_err = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();
    let graph_error = GraphMemoryError::from(json_err);
    assert!(graph_error.to_string().contains("Serialization error"));
}

#[test]
fn test_core_error_conversion() {
    let core_error = CoreError::Generic("Core failure".to_string());
    let graph_error = GraphMemoryError::from(core_error);
    assert!(graph_error.to_string().contains("Core error"));
}

#[test]
fn test_http_error_conversion() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let result = runtime.block_on(async {
        reqwest::get("http://invalid-domain-graph-test-12345.com").await
    });
    
    if let Err(req_err) = result {
        let graph_error = GraphMemoryError::from(req_err);
        assert!(graph_error.to_string().contains("HTTP error"));
    }
}

#[test]
fn test_result_type_alias() {
    fn returns_ok() -> Result<String> {
        Ok("success".to_string())
    }
    
    fn returns_err() -> Result<String> {
        Err(GraphMemoryError::Generic("test error".to_string()))
    }
    
    assert_eq!(returns_ok().unwrap(), "success");
    assert!(returns_err().is_err());
}

#[test]
fn test_error_debug_format() {
    let error = GraphMemoryError::Query("Debug test".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("Query"));
    assert!(debug_str.contains("Debug test"));
}

#[test]
fn test_error_chain() {
    fn inner_operation() -> Result<()> {
        Err(GraphMemoryError::Database("Connection lost".to_string()))
    }
    
    fn outer_operation() -> Result<()> {
        inner_operation().map_err(|e| {
            GraphMemoryError::Generic(format!("Operation failed: {}", e))
        })
    }
    
    let result = outer_operation();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Operation failed"));
}

#[test]
fn test_all_error_variants() {
    let errors = vec![
        GraphMemoryError::Database("db".to_string()),
        GraphMemoryError::Query("query".to_string()),
        GraphMemoryError::EntityExtraction("extract".to_string()),
        GraphMemoryError::Embedding("embed".to_string()),
        GraphMemoryError::Generic("generic".to_string()),
    ];
    
    for error in errors {
        // Test string conversion
        let _ = error.to_string();
        // Test debug format
        let _ = format!("{:?}", error);
    }
}

#[test]
fn test_error_with_empty_messages() {
    let errors = vec![
        GraphMemoryError::Database("".to_string()),
        GraphMemoryError::Query("".to_string()),
        GraphMemoryError::EntityExtraction("".to_string()),
        GraphMemoryError::Embedding("".to_string()),
        GraphMemoryError::Generic("".to_string()),
    ];
    
    for error in errors {
        let error_str = error.to_string();
        assert!(!error_str.is_empty());
    }
}

#[test]
fn test_error_with_long_messages() {
    let long_msg = "x".repeat(10000);
    let errors = vec![
        GraphMemoryError::Database(long_msg.clone()),
        GraphMemoryError::Query(long_msg.clone()),
        GraphMemoryError::EntityExtraction(long_msg.clone()),
        GraphMemoryError::Embedding(long_msg.clone()),
        GraphMemoryError::Generic(long_msg.clone()),
    ];
    
    for error in errors {
        let error_str = error.to_string();
        assert!(error_str.len() > 10000);
    }
}

#[test]
fn test_error_variants_display() {
    let db_err = GraphMemoryError::Database("test".to_string());
    assert!(db_err.to_string().starts_with("Database error:"));
    
    let query_err = GraphMemoryError::Query("test".to_string());
    assert!(query_err.to_string().starts_with("Query error:"));
    
    let entity_err = GraphMemoryError::EntityExtraction("test".to_string());
    assert!(entity_err.to_string().starts_with("Entity extraction error:"));
    
    let embed_err = GraphMemoryError::Embedding("test".to_string());
    assert!(embed_err.to_string().starts_with("Embedding error:"));
    
    let gen_err = GraphMemoryError::Generic("test".to_string());
    assert!(gen_err.to_string().starts_with("Graph memory error:"));
}

#[test]
fn test_complex_error_scenarios() {
    // Test database connection error scenario
    let db_error = GraphMemoryError::Database("Connection pool exhausted".to_string());
    assert!(db_error.to_string().contains("Connection pool"));
    
    // Test query timeout scenario
    let query_error = GraphMemoryError::Query("Query timeout after 30s".to_string());
    assert!(query_error.to_string().contains("timeout"));
    
    // Test entity extraction with special characters
    let entity_error = GraphMemoryError::EntityExtraction("Failed to parse: @#$%^&*()".to_string());
    assert!(entity_error.to_string().contains("@#$%^&*()"));
    
    // Test embedding dimension mismatch
    let embed_error = GraphMemoryError::Embedding("Expected 768 dimensions, got 512".to_string());
    assert!(embed_error.to_string().contains("768"));
    assert!(embed_error.to_string().contains("512"));
}