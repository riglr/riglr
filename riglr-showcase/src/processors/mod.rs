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
//! ```rust
//! use riglr_showcase::processors::{
//!     DistillationProcessor, MarkdownFormatter, NotificationRouter
//! };
//!
//! // Create a processing pipeline
//! let pipeline = ProcessorPipeline::new()
//!     .add(DistillationProcessor::new("gemini-1.5-flash"))
//!     .add(MarkdownFormatter::new())
//!     .add(NotificationRouter::new("discord"));
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

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the output from a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    pub tool_name: String,
    pub success: bool,
    pub result: serde_json::Value,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub metadata: HashMap<String, String>,
}

/// Processed output after going through a processor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedOutput {
    pub original: ToolOutput,
    pub processed_result: serde_json::Value,
    pub format: OutputFormat,
    pub summary: Option<String>,
    pub routing_info: Option<RoutingInfo>,
}

/// Output format types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Json,
    Markdown,
    PlainText,
    Html,
    Custom(String),
}

/// Routing information for notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingInfo {
    pub channel: String,
    pub recipients: Vec<String>,
    pub priority: NotificationPriority,
}

/// Notification priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
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
pub struct ProcessorPipeline {
    processors: Vec<Box<dyn OutputProcessor>>,
}

impl ProcessorPipeline {
    /// Create a new empty pipeline
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }
    
    /// Add a processor to the pipeline
    pub fn add<P: OutputProcessor + 'static>(mut self, processor: P) -> Self {
        self.processors.push(Box::new(processor));
        self
    }
    
    /// Process output through the entire pipeline
    pub async fn process(&self, mut output: ToolOutput) -> Result<ProcessedOutput> {
        let mut current = ProcessedOutput {
            original: output.clone(),
            processed_result: output.result.clone(),
            format: OutputFormat::Json,
            summary: None,
            routing_info: None,
        };
        
        for processor in &self.processors {
            if processor.can_process(&output) {
                // Update the tool output with the processed result for the next processor
                output.result = current.processed_result.clone();
                current = processor.process(output.clone()).await?;
            }
        }
        
        Ok(current)
    }
    
    /// Get information about all processors in the pipeline
    pub fn info(&self) -> Vec<serde_json::Value> {
        self.processors.iter().map(|p| p.config()).collect()
    }
}

impl Default for ProcessorPipeline {
    fn default() -> Self {
        Self::new()
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
    
    #[tokio::test]
    async fn test_pipeline_composition() {
        let pipeline = ProcessorPipeline::new()
            .add(MockDistiller::new())
            .add(MarkdownFormatter::new());
        
        let output = utils::success_output(
            "test_tool",
            serde_json::json!({"balance": "1.5 SOL", "address": "11111111111111111111111111111112"})
        );
        
        let processed = pipeline.process(output).await.unwrap();
        
        assert!(!processed.summary.is_none());
        assert!(matches!(processed.format, OutputFormat::Markdown));
    }
    
    #[tokio::test]
    async fn test_error_handling() {
        let output = utils::error_output("test_tool", "Connection timeout");
        let friendly_error = utils::user_friendly_error(&output);
        
        assert!(friendly_error.contains("timed out"));
        assert!(!friendly_error.contains("Connection timeout")); // Should be cleaned up
    }
}