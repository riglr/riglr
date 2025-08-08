//! Main graph memory implementation.

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// The main graph memory system that provides a rig::VectorStore implementation.
#[derive(Debug, Clone)]
pub struct GraphMemory {
    connection_string: String,
}

impl GraphMemory {
    /// Create a new graph memory instance.
    pub async fn new(connection_string: impl Into<String>) -> Result<Self> {
        Ok(Self {
            connection_string: connection_string.into(),
        })
    }
    
    /// Add documents to the graph.
    pub async fn add_documents(&self, _documents: Vec<crate::document::RawTextDocument>) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }
}