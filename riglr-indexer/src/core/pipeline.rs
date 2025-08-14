//! Event processing pipeline stages

use riglr_events_core::prelude::*;
use crate::error::{IndexerError, IndexerResult};

pub use crate::core::processor::{ProcessingPipeline, PipelineStage};

/// Result of pipeline processing
pub type PipelineResult<T> = IndexerResult<T>;

/// Validation pipeline stage
pub struct ValidationStage {
    name: String,
}

impl ValidationStage {
    pub fn new() -> Self {
        Self {
            name: "validation".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl PipelineStage for ValidationStage {
    async fn process(&self, event: Box<dyn Event>) -> IndexerResult<Box<dyn Event>> {
        // Basic validation - ensure event has required fields
        if event.id().is_empty() {
            return Err(IndexerError::validation("Event ID cannot be empty"));
        }
        
        if event.source().is_empty() {
            return Err(IndexerError::validation("Event source cannot be empty"));
        }

        Ok(event)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Enrichment pipeline stage
pub struct EnrichmentStage {
    name: String,
}

impl EnrichmentStage {
    pub fn new() -> Self {
        Self {
            name: "enrichment".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl PipelineStage for EnrichmentStage {
    async fn process(&self, event: Box<dyn Event>) -> IndexerResult<Box<dyn Event>> {
        // Add any enrichment logic here
        // For now, just pass through
        Ok(event)
    }

    fn name(&self) -> &str {
        &self.name
    }
}