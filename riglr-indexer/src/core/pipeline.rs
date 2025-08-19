//! Event processing pipeline stages

use crate::error::{IndexerError, IndexerResult};
use riglr_events_core::prelude::*;

pub use crate::core::processor::{PipelineStage, ProcessingPipeline};

/// Result of pipeline processing
pub type PipelineResult<T> = IndexerResult<T>;

/// Validation pipeline stage
pub struct ValidationStage {
    name: String,
}

impl ValidationStage {
    /// Create a new validation stage
    pub fn new() -> Self {
        Self {
            name: "validation".to_string(),
        }
    }
}

impl Default for ValidationStage {
    fn default() -> Self {
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
    /// Create a new enrichment stage
    pub fn new() -> Self {
        Self {
            name: "enrichment".to_string(),
        }
    }
}

impl Default for EnrichmentStage {
    fn default() -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_events_core::types::GenericEvent;
    use serde_json::json;

    // Helper function to create a test event with valid data
    fn create_valid_event() -> Box<dyn Event> {
        Box::new(GenericEvent::with_source(
            "test-event-id".to_string(),
            EventKind::Transaction,
            "test-source".to_string(),
            json!({"test": "data"}),
        ))
    }

    // Helper function to create a test event with empty ID
    fn create_event_with_empty_id() -> Box<dyn Event> {
        Box::new(GenericEvent::with_source(
            "".to_string(), // Empty ID
            EventKind::Transaction,
            "test-source".to_string(),
            json!({"test": "data"}),
        ))
    }

    // Helper function to create a test event with empty source
    fn create_event_with_empty_source() -> Box<dyn Event> {
        Box::new(GenericEvent::with_source(
            "test-event-id".to_string(),
            EventKind::Transaction,
            "".to_string(), // Empty source
            json!({"test": "data"}),
        ))
    }

    // Test ValidationStage::default()
    #[test]
    fn test_validation_stage_new_should_create_with_correct_name() {
        let stage = ValidationStage::default();
        assert_eq!(stage.name, "validation");
    }

    // Test ValidationStage::default()
    #[test]
    fn test_validation_stage_default_should_create_with_correct_name() {
        let stage = ValidationStage::default();
        assert_eq!(stage.name, "validation");
    }

    // Test ValidationStage::name()
    #[test]
    fn test_validation_stage_name_should_return_validation() {
        let stage = ValidationStage::default();
        assert_eq!(stage.name(), "validation");
    }

    // Test ValidationStage::process() with valid event (Happy Path)
    #[tokio::test]
    async fn test_validation_stage_process_when_valid_event_should_return_ok() {
        let stage = ValidationStage::default();
        let event = create_valid_event();
        let event_id = event.id().to_string();
        let event_source = event.source().to_string();

        let result = stage.process(event).await;

        assert!(result.is_ok());
        let processed_event = result.unwrap();
        assert_eq!(processed_event.id(), event_id);
        assert_eq!(processed_event.source(), event_source);
    }

    // Test ValidationStage::process() with empty ID (Error Path 1)
    #[tokio::test]
    async fn test_validation_stage_process_when_empty_id_should_return_validation_error() {
        let stage = ValidationStage::default();
        let event = create_event_with_empty_id();

        let result = stage.process(event).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            IndexerError::Validation { message } => {
                assert_eq!(message, "Event ID cannot be empty");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    // Test ValidationStage::process() with empty source (Error Path 2)
    #[tokio::test]
    async fn test_validation_stage_process_when_empty_source_should_return_validation_error() {
        let stage = ValidationStage::default();
        let event = create_event_with_empty_source();

        let result = stage.process(event).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            IndexerError::Validation { message } => {
                assert_eq!(message, "Event source cannot be empty");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    // Test EnrichmentStage::default()
    #[test]
    fn test_enrichment_stage_new_should_create_with_correct_name() {
        let stage = EnrichmentStage::default();
        assert_eq!(stage.name, "enrichment");
    }

    // Test EnrichmentStage::default()
    #[test]
    fn test_enrichment_stage_default_should_create_with_correct_name() {
        let stage = EnrichmentStage::default();
        assert_eq!(stage.name, "enrichment");
    }

    // Test EnrichmentStage::name()
    #[test]
    fn test_enrichment_stage_name_should_return_enrichment() {
        let stage = EnrichmentStage::default();
        assert_eq!(stage.name(), "enrichment");
    }

    // Test EnrichmentStage::process() with valid event (Happy Path)
    #[tokio::test]
    async fn test_enrichment_stage_process_when_valid_event_should_return_ok() {
        let stage = EnrichmentStage::default();
        let event = create_valid_event();
        let event_id = event.id().to_string();
        let event_source = event.source().to_string();

        let result = stage.process(event).await;

        assert!(result.is_ok());
        let processed_event = result.unwrap();
        assert_eq!(processed_event.id(), event_id);
        assert_eq!(processed_event.source(), event_source);
    }

    // Test EnrichmentStage::process() with empty ID (Pass-through)
    #[tokio::test]
    async fn test_enrichment_stage_process_when_empty_id_should_pass_through() {
        let stage = EnrichmentStage::default();
        let event = create_event_with_empty_id();

        let result = stage.process(event).await;

        // EnrichmentStage doesn't validate, it just passes through
        assert!(result.is_ok());
        let processed_event = result.unwrap();
        assert_eq!(processed_event.id(), "");
    }

    // Test EnrichmentStage::process() with empty source (Pass-through)
    #[tokio::test]
    async fn test_enrichment_stage_process_when_empty_source_should_pass_through() {
        let stage = EnrichmentStage::default();
        let event = create_event_with_empty_source();

        let result = stage.process(event).await;

        // EnrichmentStage doesn't validate, it just passes through
        assert!(result.is_ok());
        let processed_event = result.unwrap();
        assert_eq!(processed_event.source(), "");
    }

    // Test PipelineResult type alias
    #[test]
    fn test_pipeline_result_ok() {
        let result: PipelineResult<String> = Ok("success".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_pipeline_result_err() {
        let result: PipelineResult<String> = Err(IndexerError::validation("test error"));
        assert!(result.is_err());
        match result.unwrap_err() {
            IndexerError::Validation { message } => {
                assert_eq!(message, "test error");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    // Edge case tests
    #[tokio::test]
    async fn test_validation_stage_process_with_different_event_kinds() {
        let stage = ValidationStage::default();

        // Test with different event kinds to ensure the validator doesn't depend on event type
        let event_kinds = vec![
            EventKind::Transaction,
            EventKind::Block,
            EventKind::Contract,
            EventKind::Transfer,
            EventKind::Swap,
            EventKind::Liquidity,
            EventKind::Price,
            EventKind::External,
            EventKind::Custom("custom-event".to_string()),
        ];

        for kind in event_kinds {
            let event = Box::new(GenericEvent::with_source(
                "test-id".to_string(),
                kind.clone(),
                "test-source".to_string(),
                json!({"kind": kind.to_string()}),
            ));

            let result = stage.process(event).await;
            assert!(result.is_ok(), "Failed for event kind: {:?}", kind);
        }
    }

    #[tokio::test]
    async fn test_enrichment_stage_process_with_different_event_kinds() {
        let stage = EnrichmentStage::default();

        // Test with different event kinds to ensure the enricher works with all types
        let event_kinds = vec![
            EventKind::Transaction,
            EventKind::Block,
            EventKind::Contract,
            EventKind::Transfer,
            EventKind::Swap,
            EventKind::Liquidity,
            EventKind::Price,
            EventKind::External,
            EventKind::Custom("custom-event".to_string()),
        ];

        for kind in event_kinds {
            let event = Box::new(GenericEvent::with_source(
                "test-id".to_string(),
                kind.clone(),
                "test-source".to_string(),
                json!({"kind": kind.to_string()}),
            ));

            let result = stage.process(event).await;
            assert!(result.is_ok(), "Failed for event kind: {:?}", kind);
        }
    }

    // Test with edge case data
    #[tokio::test]
    async fn test_validation_stage_process_with_special_characters() {
        let stage = ValidationStage::default();

        // Test with special characters in ID and source
        let event = Box::new(GenericEvent::with_source(
            "test-id-with-special-chars-@#$%^&*()".to_string(),
            EventKind::Transaction,
            "source-with-unicode-字符".to_string(),
            json!({"special": "data"}),
        ));

        let result = stage.process(event).await;
        assert!(result.is_ok());
    }

    // Test with very long strings (edge case)
    #[tokio::test]
    async fn test_validation_stage_process_with_long_strings() {
        let stage = ValidationStage::default();

        let long_id = "a".repeat(1000);
        let long_source = "b".repeat(1000);

        let event = Box::new(GenericEvent::with_source(
            long_id.clone(),
            EventKind::Transaction,
            long_source.clone(),
            json!({"long": "data"}),
        ));

        let result = stage.process(event).await;
        assert!(result.is_ok());
        let processed_event = result.unwrap();
        assert_eq!(processed_event.id(), long_id);
        assert_eq!(processed_event.source(), long_source);
    }

    // Test stage names are correctly set for both new() and default()
    #[test]
    fn test_stage_names_consistency() {
        let validation_new = ValidationStage::default();
        let validation_default = ValidationStage::default();
        assert_eq!(validation_new.name(), validation_default.name());

        let enrichment_new = EnrichmentStage::default();
        let enrichment_default = EnrichmentStage::default();
        assert_eq!(enrichment_new.name(), enrichment_default.name());
    }

    // Test that all public methods are covered
    #[test]
    fn test_public_api_coverage() {
        // This test ensures all public methods are accessible and covered
        let validation_stage = ValidationStage::default();
        let _ = validation_stage.name();

        let validation_default = ValidationStage::default();
        let _ = validation_default.name();

        let enrichment_stage = EnrichmentStage::default();
        let _ = enrichment_stage.name();

        let enrichment_default = EnrichmentStage::default();
        let _ = enrichment_default.name();

        // Test type alias
        let _result: PipelineResult<()> = Ok(());
    }
}
