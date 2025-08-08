//! Entity extraction for graph memory.

use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Extracted entity from text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub entity_type: String,
    pub value: String,
    pub confidence: f32,
}

/// Entity extractor for processing text.
pub struct EntityExtractor {
    // Placeholder
}

impl EntityExtractor {
    /// Create a new entity extractor.
    pub fn new() -> Self {
        Self {}
    }
    
    /// Extract entities from text.
    pub async fn extract(&self, _text: &str) -> Result<Vec<Entity>> {
        // Placeholder implementation
        Ok(Vec::new())
    }
}

impl Default for EntityExtractor {
    fn default() -> Self {
        Self::new()
    }
}