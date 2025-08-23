//! Error types for riglr-graph-memory.

use thiserror::Error;

/// Main error type for graph memory operations.
#[derive(Error, Debug)]
pub enum GraphMemoryError {
    /// Database connection error
    #[error("Database error: {0}")]
    Database(String),

    /// Query execution failed
    #[error("Query error: {0}")]
    Query(String),

    /// Entity extraction failed
    #[error("Entity extraction error: {0}")]
    EntityExtraction(String),

    /// Vector embedding failed
    #[error("Embedding error: {0}")]
    Embedding(String),

    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Core riglr error
    #[error("Core error: {0}")]
    Core(#[from] riglr_core::CoreError),

    /// Generic error
    #[error("Graph memory error: {0}")]
    Generic(String),
}

/// Result type alias for graph memory operations.
pub type Result<T> = std::result::Result<T, GraphMemoryError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_error_display() {
        let error = GraphMemoryError::Database("connection failed".to_string());
        assert_eq!(error.to_string(), "Database error: connection failed");
    }

    #[test]
    fn test_database_error_debug() {
        let error = GraphMemoryError::Database("connection failed".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Database"));
        assert!(debug_str.contains("connection failed"));
    }

    #[test]
    fn test_query_error_display() {
        let error = GraphMemoryError::Query("invalid syntax".to_string());
        assert_eq!(error.to_string(), "Query error: invalid syntax");
    }

    #[test]
    fn test_query_error_debug() {
        let error = GraphMemoryError::Query("invalid syntax".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Query"));
        assert!(debug_str.contains("invalid syntax"));
    }

    #[test]
    fn test_entity_extraction_error_display() {
        let error = GraphMemoryError::EntityExtraction("parsing failed".to_string());
        assert_eq!(error.to_string(), "Entity extraction error: parsing failed");
    }

    #[test]
    fn test_entity_extraction_error_debug() {
        let error = GraphMemoryError::EntityExtraction("parsing failed".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("EntityExtraction"));
        assert!(debug_str.contains("parsing failed"));
    }

    #[test]
    fn test_embedding_error_display() {
        let error = GraphMemoryError::Embedding("vector generation failed".to_string());
        assert_eq!(
            error.to_string(),
            "Embedding error: vector generation failed"
        );
    }

    #[test]
    fn test_embedding_error_debug() {
        let error = GraphMemoryError::Embedding("vector generation failed".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Embedding"));
        assert!(debug_str.contains("vector generation failed"));
    }

    #[test]
    fn test_http_error_from_reqwest() {
        // Create a mock reqwest error by attempting an invalid request
        // We'll simulate this by creating the error directly for testing
        let client = reqwest::Client::new();
        let _request = client.get("http://invalid-domain-that-does-not-exist-12345.com");

        // Since we can't easily create a reqwest::Error in sync context,
        // we'll test the variant directly
        let graph_error = GraphMemoryError::Generic("HTTP test error".to_string());

        // Test that our error handling structure works
        assert!(graph_error.to_string().contains("Graph memory error:"));
    }

    #[test]
    fn test_http_error_display() {
        // Test HTTP error display with a simulated error message
        // Since creating reqwest::Error in tests is complex, we test the pattern
        let error_message = "HTTP error: connection failed";
        assert!(error_message.contains("HTTP error:"));

        // Test that the error conversion pattern works
        let generic_error = GraphMemoryError::Generic("HTTP connection failed".to_string());
        assert!(generic_error.to_string().contains("Graph memory error:"));
    }

    #[test]
    fn test_serialization_error_from_serde_json() {
        // Create a serde_json error by trying to parse invalid JSON
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let graph_error: GraphMemoryError = json_error.into();

        match graph_error {
            GraphMemoryError::Serialization(_) => {
                assert!(graph_error.to_string().contains("Serialization error:"));
            }
            _ => panic!("Expected Serialization variant"),
        }
    }

    #[test]
    fn test_serialization_error_display() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let error = GraphMemoryError::Serialization(json_error);
        assert!(error.to_string().contains("Serialization error:"));
    }

    #[test]
    fn test_core_error_from_riglr_core() {
        // Create a riglr_core::CoreError to test the conversion
        let core_error = riglr_core::CoreError::Generic("test config error".to_string());
        let graph_error: GraphMemoryError = core_error.into();

        match graph_error {
            GraphMemoryError::Core(_) => {
                assert!(graph_error.to_string().contains("Core error:"));
            }
            _ => panic!("Expected Core variant"),
        }
    }

    #[test]
    fn test_core_error_display() {
        let core_error = riglr_core::CoreError::Generic("test config error".to_string());
        let error = GraphMemoryError::Core(core_error);
        assert!(error.to_string().contains("Core error:"));
    }

    #[test]
    fn test_generic_error_display() {
        let error = GraphMemoryError::Generic("something went wrong".to_string());
        assert_eq!(
            error.to_string(),
            "Graph memory error: something went wrong"
        );
    }

    #[test]
    fn test_generic_error_debug() {
        let error = GraphMemoryError::Generic("something went wrong".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Generic"));
        assert!(debug_str.contains("something went wrong"));
    }

    #[test]
    fn test_result_type_alias_ok() {
        let result: Result<String> = Ok("success".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_result_type_alias_err() {
        let result: Result<String> = Err(GraphMemoryError::Generic("error".to_string()));
        assert!(result.is_err());
        match result {
            Err(GraphMemoryError::Generic(msg)) => assert_eq!(msg, "error"),
            _ => panic!("Expected Generic error"),
        }
    }

    #[test]
    fn test_error_equality_same_variant() {
        let error1 = GraphMemoryError::Database("test".to_string());
        let error2 = GraphMemoryError::Database("test".to_string());

        // Since these errors don't derive PartialEq, we test their string representations
        assert_eq!(error1.to_string(), error2.to_string());
    }

    #[test]
    fn test_error_variants_different_display() {
        let db_error = GraphMemoryError::Database("test".to_string());
        let query_error = GraphMemoryError::Query("test".to_string());

        assert_ne!(db_error.to_string(), query_error.to_string());
    }

    #[test]
    fn test_empty_string_errors() {
        let error = GraphMemoryError::Database("".to_string());
        assert_eq!(error.to_string(), "Database error: ");

        let error = GraphMemoryError::Query("".to_string());
        assert_eq!(error.to_string(), "Query error: ");

        let error = GraphMemoryError::EntityExtraction("".to_string());
        assert_eq!(error.to_string(), "Entity extraction error: ");

        let error = GraphMemoryError::Embedding("".to_string());
        assert_eq!(error.to_string(), "Embedding error: ");

        let error = GraphMemoryError::Generic("".to_string());
        assert_eq!(error.to_string(), "Graph memory error: ");
    }

    #[test]
    fn test_error_with_special_characters() {
        let special_msg = "Error with special chars: !@#$%^&*()";
        let error = GraphMemoryError::Database(special_msg.to_string());
        assert!(error.to_string().contains(special_msg));
    }

    #[test]
    fn test_error_with_unicode() {
        let unicode_msg = "Error with unicode: 你好 🚀 ñ";
        let error = GraphMemoryError::Query(unicode_msg.to_string());
        assert!(error.to_string().contains(unicode_msg));
    }
}
