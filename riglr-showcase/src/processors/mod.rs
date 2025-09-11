//! Output processing patterns for riglr applications
//!
//! This module demonstrates composable patterns for processing tool outputs.
//! These are APPLICATION patterns, not library features - they show developers
//! how to create their own output processing pipelines.
//!
//! # Key Patterns Demonstrated
//!
//! 1. **Output Distillation** - Summarize complex tool outputs using LLMs
//! 2. **Format Transformation** - Convert outputs to different formats (JSON, Markdown, etc.)
//! 3. **Notification Routing** - Send outputs to different channels (Discord, Telegram, etc.)
//! 4. **Error Formatting** - Clean error handling and user-friendly messages
//! 5. **Composable Processors** - Show how processors can be chained
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use riglr_showcase::processors::{
//!     DistillationProcessor, MarkdownFormatter, NotificationRouter
//! };
//!
//! // Create a processing pipeline
//! let pipeline = ProcessorPipeline::new()
//!     .add_processor(DistillationProcessor::new("gemini-1.5-flash"))
//!     .add_processor(MarkdownFormatter::new())
//!     .add_processor(NotificationRouter::new("discord"));
//!
//! // Process tool output
//! let processed = pipeline.process(tool_output).await?;
//! ```
//!
//! # Design Philosophy
//!
//! These patterns are intentionally designed as APPLICATION code that developers
//! can copy, modify, and extend. They are NOT rigid library abstractions but
//! flexible examples showing how to build custom output processing.

pub mod distiller;
pub mod formatter;
pub mod notifier;

// ProcessorPipeline is defined below

// Re-export notification types as they're named differently in the sub-module
pub use notifier::{
    ConsoleChannel, DiscordChannel, NotificationRouter, RoutingCondition, RoutingRule,
    TelegramChannel,
};

// Re-export formatter types
pub use formatter::{HtmlFormatter, JsonFormatter, MarkdownFormatter, MultiFormatProcessor};

// Re-export distiller types
pub use distiller::{DistillationProcessor, SmartDistiller};

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the output from a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    /// Name of the tool that produced this output
    pub tool_name: String,
    /// Whether the tool execution was successful
    pub success: bool,
    /// The actual result data from the tool execution
    pub result: serde_json::Value,
    /// Error message if the tool execution failed
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Additional metadata about the tool execution
    pub metadata: HashMap<String, String>,
}

/// Processed output after going through a processor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedOutput {
    /// The original tool output before processing
    pub original: ToolOutput,
    /// The result after processing (transformed, formatted, etc.)
    pub processed_result: serde_json::Value,
    /// The format of the processed result
    pub format: OutputFormat,
    /// Optional summary or description of the processed output
    pub summary: Option<String>,
    /// Optional routing information for notifications
    pub routing_info: Option<RoutingInfo>,
}

/// Output format types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputFormat {
    /// JSON format output
    Json,
    /// Markdown format output
    Markdown,
    /// Plain text format output
    PlainText,
    /// HTML format output
    Html,
    /// Custom format with specified type name
    Custom(String),
}

/// Routing information for notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingInfo {
    /// The target channel for notification routing
    pub channel: String,
    /// List of recipients to notify
    pub recipients: Vec<String>,
    /// Priority level for the notification
    pub priority: NotificationPriority,
}

/// Notification priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NotificationPriority {
    /// Low priority notification
    Low,
    /// Normal priority notification
    Normal,
    /// High priority notification
    High,
    /// Critical priority notification
    Critical,
}

/// Core trait for output processors
///
/// This trait defines the interface for all output processors.
/// Implementations can transform, summarize, format, or route outputs.
#[async_trait]
pub trait OutputProcessor: Send + Sync {
    /// Process a tool output and return the processed result
    async fn process(&self, input: ToolOutput) -> Result<ProcessedOutput>;

    /// Get the processor name for debugging and logging
    fn name(&self) -> &str;

    /// Check if this processor can handle the given output type
    fn can_process(&self, _output: &ToolOutput) -> bool {
        // Default implementation accepts all outputs
        true
    }

    /// Get processor configuration as JSON for debugging
    fn config(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name(),
            "type": "generic"
        })
    }
}

/// A pipeline that chains multiple processors together
///
/// This demonstrates the composability pattern - processors can be chained
/// to create complex processing workflows.
#[derive(Default)]
pub struct ProcessorPipeline {
    processors: Vec<Box<dyn OutputProcessor>>,
}

impl std::fmt::Debug for ProcessorPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessorPipeline")
            .field("processor_count", &self.processors.len())
            .field(
                "processor_names",
                &self.processors.iter().map(|p| p.name()).collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl ProcessorPipeline {
    /// Create a new empty pipeline
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    /// Add a processor to the pipeline
    pub fn add_processor<P: OutputProcessor + 'static>(mut self, processor: P) -> Self {
        self.processors.push(Box::new(processor));
        self
    }

    /// Process output through the entire pipeline
    pub async fn process(&self, mut output: ToolOutput) -> Result<ProcessedOutput> {
        let original = output.clone();
        let mut current = ProcessedOutput {
            original,
            processed_result: output.result.clone(),
            format: OutputFormat::Json,
            summary: None,
            routing_info: None,
        };

        for processor in &self.processors {
            if processor.can_process(&output) {
                // Update the tool output with the processed result for the next processor
                output.result = std::mem::take(&mut current.processed_result);
                let processed = processor.process(output.clone()).await?;

                // Preserve summary from previous processors if current processor doesn't provide one
                let summary = processed.summary.or_else(|| current.summary.take());

                current = ProcessedOutput {
                    summary,
                    ..processed
                };
            }
        }

        Ok(current)
    }

    /// Get information about all processors in the pipeline
    pub fn info(&self) -> Vec<serde_json::Value> {
        self.processors.iter().map(|p| p.config()).collect()
    }
}

/// Utility functions for working with tool outputs
pub mod utils {
    use super::*;
    use std::time::SystemTime;

    /// Create a ToolOutput from a successful operation
    pub fn success_output(tool_name: &str, result: serde_json::Value) -> ToolOutput {
        ToolOutput {
            tool_name: tool_name.to_string(),
            success: true,
            result,
            error: None,
            execution_time_ms: 0,
            metadata: HashMap::new(),
        }
    }

    /// Create a ToolOutput from a failed operation
    pub fn error_output(tool_name: &str, error: &str) -> ToolOutput {
        ToolOutput {
            tool_name: tool_name.to_string(),
            success: false,
            result: serde_json::Value::Null,
            error: Some(error.to_string()),
            execution_time_ms: 0,
            metadata: HashMap::new(),
        }
    }

    /// Add timing information to a ToolOutput
    pub fn with_timing(mut output: ToolOutput, start_time: SystemTime) -> ToolOutput {
        if let Ok(duration) = start_time.elapsed() {
            output.execution_time_ms = duration.as_millis() as u64;
        }
        output
    }

    /// Add metadata to a ToolOutput
    pub fn with_metadata(mut output: ToolOutput, key: &str, value: &str) -> ToolOutput {
        output.metadata.insert(key.to_string(), value.to_string());
        output
    }

    /// Extract error message from a ToolOutput in a user-friendly way
    pub fn user_friendly_error(output: &ToolOutput) -> String {
        if output.success {
            return "Operation completed successfully".to_string();
        }

        match &output.error {
            Some(error) => {
                // Clean up technical error messages for users
                if error.contains("connection") {
                    "Network connection error. Please check your internet connection.".to_string()
                } else if error.contains("timeout") {
                    "The operation timed out. Please try again.".to_string()
                } else if error.contains("unauthorized") || error.contains("403") {
                    "Access denied. Please check your authentication.".to_string()
                } else if error.contains("not found") || error.contains("404") {
                    "The requested resource was not found.".to_string()
                } else {
                    format!("An error occurred: {}", error)
                }
            }
            None => "An unknown error occurred".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processors::distiller::MockDistiller;
    use crate::processors::formatter::MarkdownFormatter;
    use std::time::{Duration, SystemTime};

    // Mock processor for testing
    struct TestProcessor {
        name: String,
        can_process_result: bool,
        should_error: bool,
    }

    impl TestProcessor {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                can_process_result: true,
                should_error: false,
            }
        }

        fn with_can_process(mut self, can_process: bool) -> Self {
            self.can_process_result = can_process;
            self
        }

        fn with_error(mut self, should_error: bool) -> Self {
            self.should_error = should_error;
            self
        }
    }

    #[async_trait]
    impl OutputProcessor for TestProcessor {
        async fn process(&self, input: ToolOutput) -> Result<ProcessedOutput> {
            if self.should_error {
                return Err(anyhow::anyhow!("Test processor error"));
            }

            Ok(ProcessedOutput {
                original: input.clone(),
                processed_result: serde_json::json!({
                    "processed_by": self.name,
                    "original_result": input.result
                }),
                format: OutputFormat::Json,
                summary: Some(format!("Processed by {}", self.name)),
                routing_info: None,
            })
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn can_process(&self, _output: &ToolOutput) -> bool {
            self.can_process_result
        }

        fn config(&self) -> serde_json::Value {
            serde_json::json!({
                "name": self.name,
                "type": "test_processor",
                "custom_field": "test_value"
            })
        }
    }

    // Test ToolOutput creation and fields
    #[test]
    fn test_tool_output_creation() {
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), "value1".to_string());

        let output = ToolOutput {
            tool_name: "test_tool".to_string(),
            success: true,
            result: serde_json::json!({"data": "test"}),
            error: None,
            execution_time_ms: 100,
            metadata,
        };

        assert_eq!(output.tool_name, "test_tool");
        assert!(output.success);
        assert_eq!(output.result, serde_json::json!({"data": "test"}));
        assert!(output.error.is_none());
        assert_eq!(output.execution_time_ms, 100);
        assert_eq!(output.metadata.get("key1"), Some(&"value1".to_string()));
    }

    // Test ProcessedOutput creation and fields
    #[test]
    fn test_processed_output_creation() {
        let original = utils::success_output("test", serde_json::json!({}));
        let routing_info = RoutingInfo {
            channel: "discord".to_string(),
            recipients: vec!["user1".to_string(), "user2".to_string()],
            priority: NotificationPriority::High,
        };

        let processed = ProcessedOutput {
            original: original.clone(),
            processed_result: serde_json::json!({"processed": true}),
            format: OutputFormat::Markdown,
            summary: Some("Test summary".to_string()),
            routing_info: Some(routing_info.clone()),
        };

        assert_eq!(processed.original.tool_name, original.tool_name);
        assert_eq!(
            processed.processed_result,
            serde_json::json!({"processed": true})
        );
        assert!(matches!(processed.format, OutputFormat::Markdown));
        assert_eq!(processed.summary, Some("Test summary".to_string()));
        assert!(processed.routing_info.is_some());

        let routing = processed.routing_info.unwrap();
        assert_eq!(routing.channel, "discord");
        assert_eq!(routing.recipients.len(), 2);
        assert!(matches!(routing.priority, NotificationPriority::High));
    }

    // Test OutputFormat variants
    #[test]
    fn test_output_format_variants() {
        let json_format = OutputFormat::Json;
        let markdown_format = OutputFormat::Markdown;
        let text_format = OutputFormat::PlainText;
        let html_format = OutputFormat::Html;
        let custom_format = OutputFormat::Custom("xml".to_string());

        assert!(matches!(json_format, OutputFormat::Json));
        assert!(matches!(markdown_format, OutputFormat::Markdown));
        assert!(matches!(text_format, OutputFormat::PlainText));
        assert!(matches!(html_format, OutputFormat::Html));

        if let OutputFormat::Custom(ref format_type) = custom_format {
            assert_eq!(format_type, "xml");
        } else {
            panic!("Expected Custom format");
        }
    }

    // Test NotificationPriority variants
    #[test]
    fn test_notification_priority_variants() {
        let priorities = vec![
            NotificationPriority::Low,
            NotificationPriority::Normal,
            NotificationPriority::High,
            NotificationPriority::Critical,
        ];

        assert_eq!(priorities.len(), 4);
        assert!(matches!(priorities[0], NotificationPriority::Low));
        assert!(matches!(priorities[1], NotificationPriority::Normal));
        assert!(matches!(priorities[2], NotificationPriority::High));
        assert!(matches!(priorities[3], NotificationPriority::Critical));
    }

    // Test ProcessorPipeline creation
    #[test]
    fn test_pipeline_new() {
        let pipeline = ProcessorPipeline::default();
        assert_eq!(pipeline.processors.len(), 0);
    }

    // Test ProcessorPipeline add_processor
    #[test]
    fn test_pipeline_add_processor() {
        let pipeline = ProcessorPipeline::default()
            .add_processor(TestProcessor::new("processor1"))
            .add_processor(TestProcessor::new("processor2"));

        assert_eq!(pipeline.processors.len(), 2);
    }

    // Test ProcessorPipeline info
    #[test]
    fn test_pipeline_info() {
        let pipeline = ProcessorPipeline::default()
            .add_processor(TestProcessor::new("processor1"))
            .add_processor(TestProcessor::new("processor2"));

        let info = pipeline.info();
        assert_eq!(info.len(), 2);
        assert_eq!(info[0]["name"], "processor1");
        assert_eq!(info[0]["type"], "test_processor");
        assert_eq!(info[1]["name"], "processor2");
        assert_eq!(info[1]["type"], "test_processor");
    }

    // Test empty pipeline processing
    #[tokio::test]
    async fn test_pipeline_process_empty() {
        let pipeline = ProcessorPipeline::default();
        let output = utils::success_output("test", serde_json::json!({"data": "test"}));

        let processed = pipeline.process(output.clone()).await.unwrap();

        assert_eq!(processed.original.tool_name, output.tool_name);
        assert_eq!(processed.processed_result, output.result);
        assert!(matches!(processed.format, OutputFormat::Json));
        assert!(processed.summary.is_none());
        assert!(processed.routing_info.is_none());
    }

    // Test pipeline with processors that can't process
    #[tokio::test]
    async fn test_pipeline_process_with_skip() {
        let pipeline = ProcessorPipeline::default()
            .add_processor(TestProcessor::new("skip_processor").with_can_process(false))
            .add_processor(TestProcessor::new("process_processor").with_can_process(true));

        let output = utils::success_output("test", serde_json::json!({"data": "test"}));
        let processed = pipeline.process(output).await.unwrap();

        // Only the second processor should have processed it
        assert_eq!(
            processed.processed_result["processed_by"],
            "process_processor"
        );
        assert_eq!(
            processed.summary,
            Some("Processed by process_processor".to_string())
        );
    }

    // Test pipeline error handling
    #[tokio::test]
    async fn test_pipeline_process_error() {
        let pipeline = ProcessorPipeline::default()
            .add_processor(TestProcessor::new("error_processor").with_error(true));

        let output = utils::success_output("test", serde_json::json!({"data": "test"}));
        let result = pipeline.process(output).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Test processor error"));
    }

    // Test pipeline summary preservation
    #[tokio::test]
    async fn test_pipeline_summary_preservation() {
        struct SummaryProcessor {
            name: String,
            provide_summary: bool,
        }

        impl SummaryProcessor {
            fn new(name: &str, provide_summary: bool) -> Self {
                Self {
                    name: name.to_string(),
                    provide_summary,
                }
            }
        }

        #[async_trait]
        impl OutputProcessor for SummaryProcessor {
            async fn process(&self, input: ToolOutput) -> Result<ProcessedOutput> {
                Ok(ProcessedOutput {
                    original: input.clone(),
                    processed_result: input.result,
                    format: OutputFormat::Json,
                    summary: if self.provide_summary {
                        Some(format!("Summary from {}", self.name))
                    } else {
                        None
                    },
                    routing_info: None,
                })
            }

            fn name(&self) -> &str {
                &self.name
            }
        }

        let pipeline = ProcessorPipeline::default()
            .add_processor(SummaryProcessor::new("first", true))
            .add_processor(SummaryProcessor::new("second", false));

        let output = utils::success_output("test", serde_json::json!({}));
        let processed = pipeline.process(output).await.unwrap();

        // Should preserve the first processor's summary since second doesn't provide one
        assert_eq!(processed.summary, Some("Summary from first".to_string()));
    }

    #[tokio::test]
    async fn test_pipeline_composition() {
        let pipeline = ProcessorPipeline::default()
            .add_processor(MockDistiller::default())
            .add_processor(MarkdownFormatter::default());

        let output = utils::success_output(
            "test_tool",
            serde_json::json!({"balance": "1.5 SOL", "address": "11111111111111111111111111111112"}),
        );

        let processed = pipeline.process(output).await.unwrap();

        assert!(processed.summary.is_some());
        assert!(matches!(processed.format, OutputFormat::Markdown));
    }

    #[tokio::test]
    async fn test_error_handling() {
        let output = utils::error_output("test_tool", "Connection timeout");
        let friendly_error = utils::user_friendly_error(&output);

        assert!(friendly_error.contains("timed out"));
        assert!(!friendly_error.contains("Connection timeout")); // Should be cleaned up
    }

    // Test utils::success_output
    #[test]
    fn test_utils_success_output() {
        let result = serde_json::json!({"key": "value"});
        let output = utils::success_output("test_tool", result.clone());

        assert_eq!(output.tool_name, "test_tool");
        assert!(output.success);
        assert_eq!(output.result, result);
        assert!(output.error.is_none());
        assert_eq!(output.execution_time_ms, 0);
        assert!(output.metadata.is_empty());
    }

    // Test utils::error_output
    #[test]
    fn test_utils_error_output() {
        let output = utils::error_output("test_tool", "Test error message");

        assert_eq!(output.tool_name, "test_tool");
        assert!(!output.success);
        assert_eq!(output.result, serde_json::Value::Null);
        assert_eq!(output.error, Some("Test error message".to_string()));
        assert_eq!(output.execution_time_ms, 0);
        assert!(output.metadata.is_empty());
    }

    // Test utils::with_timing - successful elapsed time
    #[test]
    fn test_utils_with_timing_success() {
        let start_time = SystemTime::now() - Duration::from_millis(100);
        let output = utils::success_output("test", serde_json::json!({}));
        let timed_output = utils::with_timing(output, start_time);

        // Should have some execution time (at least 100ms)
        assert!(timed_output.execution_time_ms >= 100);
    }

    // Test utils::with_timing - failed elapsed time (future time)
    #[test]
    fn test_utils_with_timing_failure() {
        let future_time = SystemTime::now() + Duration::from_secs(1);
        let output = utils::success_output("test", serde_json::json!({}));
        let timed_output = utils::with_timing(output, future_time);

        // Should remain 0 if elapsed() fails
        assert_eq!(timed_output.execution_time_ms, 0);
    }

    // Test utils::with_metadata
    #[test]
    fn test_utils_with_metadata() {
        let output = utils::success_output("test", serde_json::json!({}));
        let output_with_metadata = utils::with_metadata(output, "environment", "production");

        assert_eq!(
            output_with_metadata.metadata.get("environment"),
            Some(&"production".to_string())
        );
        assert_eq!(output_with_metadata.metadata.len(), 1);
    }

    // Test utils::with_metadata - multiple metadata
    #[test]
    fn test_utils_with_metadata_multiple() {
        let output = utils::success_output("test", serde_json::json!({}));
        let output = utils::with_metadata(output, "key1", "value1");
        let output = utils::with_metadata(output, "key2", "value2");

        assert_eq!(output.metadata.len(), 2);
        assert_eq!(output.metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(output.metadata.get("key2"), Some(&"value2".to_string()));
    }

    // Test utils::user_friendly_error - success case
    #[test]
    fn test_utils_user_friendly_error_success() {
        let output = utils::success_output("test", serde_json::json!({}));
        let error_msg = utils::user_friendly_error(&output);

        assert_eq!(error_msg, "Operation completed successfully");
    }

    // Test utils::user_friendly_error - connection error
    #[test]
    fn test_utils_user_friendly_error_connection() {
        let output = utils::error_output("test", "connection refused");
        let error_msg = utils::user_friendly_error(&output);

        assert_eq!(
            error_msg,
            "Network connection error. Please check your internet connection."
        );
    }

    // Test utils::user_friendly_error - timeout error
    #[test]
    fn test_utils_user_friendly_error_timeout() {
        let output = utils::error_output("test", "operation timeout exceeded");
        let error_msg = utils::user_friendly_error(&output);

        assert_eq!(error_msg, "The operation timed out. Please try again.");
    }

    // Test utils::user_friendly_error - unauthorized error
    #[test]
    fn test_utils_user_friendly_error_unauthorized() {
        let output = utils::error_output("test", "unauthorized access");
        let error_msg = utils::user_friendly_error(&output);

        assert_eq!(
            error_msg,
            "Access denied. Please check your authentication."
        );
    }

    // Test utils::user_friendly_error - 403 error
    #[test]
    fn test_utils_user_friendly_error_403() {
        let output = utils::error_output("test", "HTTP 403 Forbidden");
        let error_msg = utils::user_friendly_error(&output);

        assert_eq!(
            error_msg,
            "Access denied. Please check your authentication."
        );
    }

    // Test utils::user_friendly_error - not found error
    #[test]
    fn test_utils_user_friendly_error_not_found() {
        let output = utils::error_output("test", "resource not found");
        let error_msg = utils::user_friendly_error(&output);

        assert_eq!(error_msg, "The requested resource was not found.");
    }

    // Test utils::user_friendly_error - 404 error
    #[test]
    fn test_utils_user_friendly_error_404() {
        let output = utils::error_output("test", "HTTP 404 Not Found");
        let error_msg = utils::user_friendly_error(&output);

        assert_eq!(error_msg, "The requested resource was not found.");
    }

    // Test utils::user_friendly_error - generic error
    #[test]
    fn test_utils_user_friendly_error_generic() {
        let output = utils::error_output("test", "some other error");
        let error_msg = utils::user_friendly_error(&output);

        assert_eq!(error_msg, "An error occurred: some other error");
    }

    // Test utils::user_friendly_error - no error message
    #[test]
    fn test_utils_user_friendly_error_no_message() {
        let mut output = utils::error_output("test", "");
        output.error = None; // Remove error message
        let error_msg = utils::user_friendly_error(&output);

        assert_eq!(error_msg, "An unknown error occurred");
    }

    // Test default OutputProcessor trait methods
    #[tokio::test]
    async fn test_output_processor_defaults() {
        let processor = TestProcessor::new("test");
        let output = utils::success_output("test", serde_json::json!({}));

        // Test default can_process (should return true)
        assert!(processor.can_process(&output));

        // Test custom config override
        let config = processor.config();
        assert_eq!(config["name"], "test");
        assert_eq!(config["type"], "test_processor");
        assert_eq!(config["custom_field"], "test_value");
    }

    // Test serialization/deserialization of structs
    #[test]
    fn test_serialization_tool_output() {
        let output = utils::success_output("test", serde_json::json!({"data": "test"}));
        let serialized = serde_json::to_string(&output).unwrap();
        let deserialized: ToolOutput = serde_json::from_str(&serialized).unwrap();

        assert_eq!(output.tool_name, deserialized.tool_name);
        assert_eq!(output.success, deserialized.success);
        assert_eq!(output.result, deserialized.result);
    }

    #[test]
    fn test_serialization_processed_output() {
        let original = utils::success_output("test", serde_json::json!({}));
        let processed = ProcessedOutput {
            original,
            processed_result: serde_json::json!({"processed": true}),
            format: OutputFormat::Markdown,
            summary: Some("test".to_string()),
            routing_info: None,
        };

        let serialized = serde_json::to_string(&processed).unwrap();
        let deserialized: ProcessedOutput = serde_json::from_str(&serialized).unwrap();

        assert!(matches!(deserialized.format, OutputFormat::Markdown));
        assert_eq!(deserialized.summary, Some("test".to_string()));
    }

    #[test]
    fn test_serialization_output_format() {
        let formats = vec![
            OutputFormat::Json,
            OutputFormat::Markdown,
            OutputFormat::PlainText,
            OutputFormat::Html,
            OutputFormat::Custom("xml".to_string()),
        ];

        for format in formats {
            let serialized = serde_json::to_string(&format).unwrap();
            let deserialized: OutputFormat = serde_json::from_str(&serialized).unwrap();

            match (&format, &deserialized) {
                (OutputFormat::Json, OutputFormat::Json) => {}
                (OutputFormat::Markdown, OutputFormat::Markdown) => {}
                (OutputFormat::PlainText, OutputFormat::PlainText) => {}
                (OutputFormat::Html, OutputFormat::Html) => {}
                (OutputFormat::Custom(a), OutputFormat::Custom(b)) => assert_eq!(a, b),
                _ => panic!("Serialization mismatch"),
            }
        }
    }

    #[test]
    fn test_serialization_notification_priority() {
        let priorities = vec![
            NotificationPriority::Low,
            NotificationPriority::Normal,
            NotificationPriority::High,
            NotificationPriority::Critical,
        ];

        for priority in priorities {
            let serialized = serde_json::to_string(&priority).unwrap();
            let deserialized: NotificationPriority = serde_json::from_str(&serialized).unwrap();

            match (&priority, &deserialized) {
                (NotificationPriority::Low, NotificationPriority::Low) => {}
                (NotificationPriority::Normal, NotificationPriority::Normal) => {}
                (NotificationPriority::High, NotificationPriority::High) => {}
                (NotificationPriority::Critical, NotificationPriority::Critical) => {}
                _ => panic!("Serialization mismatch"),
            }
        }
    }

    #[test]
    fn test_serialization_routing_info() {
        let routing_info = RoutingInfo {
            channel: "discord".to_string(),
            recipients: vec!["user1".to_string(), "user2".to_string()],
            priority: NotificationPriority::High,
        };

        let serialized = serde_json::to_string(&routing_info).unwrap();
        let deserialized: RoutingInfo = serde_json::from_str(&serialized).unwrap();

        assert_eq!(routing_info.channel, deserialized.channel);
        assert_eq!(routing_info.recipients, deserialized.recipients);
        assert!(matches!(deserialized.priority, NotificationPriority::High));
    }

    // Test Clone implementations
    #[test]
    fn test_clone_tool_output() {
        let output = utils::success_output("test", serde_json::json!({"data": "test"}));
        let cloned = output.clone();

        assert_eq!(output.tool_name, cloned.tool_name);
        assert_eq!(output.success, cloned.success);
        assert_eq!(output.result, cloned.result);
    }

    #[test]
    fn test_clone_processed_output() {
        let original = utils::success_output("test", serde_json::json!({}));
        let processed = ProcessedOutput {
            original,
            processed_result: serde_json::json!({"processed": true}),
            format: OutputFormat::Markdown,
            summary: Some("test".to_string()),
            routing_info: None,
        };

        let cloned = processed.clone();
        assert!(matches!(cloned.format, OutputFormat::Markdown));
        assert_eq!(cloned.summary, Some("test".to_string()));
    }

    // Test Debug implementations
    #[test]
    fn test_debug_implementations() {
        let output = utils::success_output("test", serde_json::json!({}));
        let debug_str = format!("{:?}", output);
        assert!(debug_str.contains("ToolOutput"));
        assert!(debug_str.contains("test"));

        let format = OutputFormat::Json;
        let debug_str = format!("{:?}", format);
        assert!(debug_str.contains("Json"));

        let priority = NotificationPriority::High;
        let debug_str = format!("{:?}", priority);
        assert!(debug_str.contains("High"));

        let routing_info = RoutingInfo {
            channel: "test".to_string(),
            recipients: vec![],
            priority: NotificationPriority::Low,
        };
        let debug_str = format!("{:?}", routing_info);
        assert!(debug_str.contains("RoutingInfo"));
    }
}
