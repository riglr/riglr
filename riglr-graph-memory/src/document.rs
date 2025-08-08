//! Document types for graph memory.

use serde::{Deserialize, Serialize};

/// A raw text document that can be added to the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTextDocument {
    pub content: String,
    pub metadata: Option<serde_json::Value>,
}

impl RawTextDocument {
    /// Create a new raw text document.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            metadata: None,
        }
    }
    
    /// Create a document with metadata.
    pub fn with_metadata(content: impl Into<String>, metadata: serde_json::Value) -> Self {
        Self {
            content: content.into(),
            metadata: Some(metadata),
        }
    }
}