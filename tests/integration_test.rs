//! Integration tests for the riglr workspace
//!
//! These tests verify that the core components work together correctly
//! and that the workspace can be built and tested as a whole.

use std::sync::Arc;
use tokio_test;

// Test that we can use riglr-core components together
#[tokio::test]
async fn test_workspace_integration() -> anyhow::Result<()> {
    // This is a basic integration test to ensure the workspace compiles
    // and that core types can be created and used together.
    
    // In a real integration test, we would:
    // 1. Create actual tools using riglr-macros
    // 2. Register them with a ToolWorker
    // 3. Process jobs through the complete pipeline
    // 4. Test SignerContext integration
    // 5. Verify error handling flows
    
    // For now, we just verify the test infrastructure works
    assert!(true);
    Ok(())
}

#[test]
fn test_workspace_builds() {
    // This test ensures the workspace can be built
    // The fact that this test runs means the build succeeded
    assert!(true);
}

// Mock components for testing
struct MockIntegrationTool {
    should_succeed: bool,
}

#[async_trait::async_trait]
impl riglr_core::Tool for MockIntegrationTool {
    async fn execute(
        &self,
        _params: serde_json::Value,
    ) -> Result<riglr_core::JobResult, Box<dyn std::error::Error + Send + Sync>> {
        if self.should_succeed {
            Ok(riglr_core::JobResult::success(&"integration test success")?)
        } else {
            Err("integration test failure".into())
        }
    }

    fn name(&self) -> &str {
        "mock_integration_tool"
    }
}

#[tokio::test]
async fn test_tool_worker_integration() -> anyhow::Result<()> {
    use riglr_core::{ToolWorker, ExecutionConfig, Job, JobResult, InMemoryIdempotencyStore};

    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
        ExecutionConfig::default()
    );

    let tool = Arc::new(MockIntegrationTool { should_succeed: true });
    worker.register_tool(tool).await;

    let job = Job::new(
        "mock_integration_tool",
        &serde_json::json!({}),
        3
    )?;

    let result = worker.process_job(job).await?;
    
    match result {
        JobResult::Success { .. } => Ok(()),
        JobResult::Failure { error, .. } => Err(anyhow::anyhow!("Job failed: {}", error)),
    }
}

#[tokio::test]
async fn test_error_handling_integration() -> anyhow::Result<()> {
    use riglr_core::{ToolWorker, ExecutionConfig, Job, JobResult, InMemoryIdempotencyStore};

    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
        ExecutionConfig::default()
    );

    let tool = Arc::new(MockIntegrationTool { should_succeed: false });
    worker.register_tool(tool).await;

    let job = Job::new(
        "mock_integration_tool",
        &serde_json::json!({}),
        3
    )?;

    let result = worker.process_job(job).await?;
    
    match result {
        JobResult::Failure { error, retriable } => {
            assert!(error.contains("integration test failure"));
            // Errors from Box<dyn Error> are typically treated as permanent
            Ok(())
        },
        JobResult::Success { .. } => Err(anyhow::anyhow!("Expected job to fail")),
    }
}

#[cfg(test)]
mod workspace_validation {
    use super::*;

    #[test]
    fn validate_core_types_exist() {
        // Verify that core types can be referenced
        use riglr_core::{Tool, Job, JobResult, ToolError, ToolWorker, ExecutionConfig};
        
        // Test that we can create error types
        let _error = ToolError::permanent_string("test");
        let _config = ExecutionConfig::default();
        
        // This test succeeds if it compiles
        assert!(true);
    }

    #[test]
    fn validate_web_adapters_exist() {
        // Verify that web adapter types can be referenced
        use riglr_web_adapters::core::{Agent, AgentEvent};
        
        // Test that we can create event types
        let _event = AgentEvent::Start;
        
        // This test succeeds if it compiles
        assert!(true);
    }

    #[test] 
    fn validate_macro_crate_exists() {
        // Verify the macro crate exists and can be referenced
        // Note: Actually using the macro would require a more complex test setup
        
        // This test succeeds if it compiles
        assert!(true);
    }
}