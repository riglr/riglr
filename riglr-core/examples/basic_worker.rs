//! Basic example showing how to set up and use a ToolWorker with the new architecture
//!
//! This demonstrates:
//! - Creating a ToolWorker with default configuration
//! - Registering tools
//! - Processing jobs with UnifiedSigner context
//! - Enhanced error handling with retry classification

use async_trait::async_trait;
use riglr_core::{
    idempotency::InMemoryIdempotencyStore,
    provider::ApplicationContext,
    signer::{SignerContext, UnifiedSigner},
    ExecutionConfig, Job, JobResult, Tool, ToolError, ToolWorker,
};
use std::sync::Arc;

/// A simple example tool that works with any signer type
#[derive(Clone)]
struct GreetingTool;

#[async_trait]
impl Tool for GreetingTool {
    async fn execute(
        &self,
        params: serde_json::Value,
        _app_context: &ApplicationContext,
    ) -> Result<JobResult, ToolError> {
        let name = params["name"].as_str().unwrap_or("World");

        // Check if we have a signer context for enhanced greeting
        if SignerContext::is_available().await {
            let signer = SignerContext::current().await?;

            if let Some(user_id) = signer.user_id() {
                let greeting = format!("Hello, {}! (User: {})", name, user_id);
                return Ok(JobResult::success(&greeting)
                    .map_err(|e| ToolError::permanent_string(e.to_string()))?);
            }
        }

        // Fallback greeting without signer context
        Ok(JobResult::success(&format!("Hello, {}!", name))
            .map_err(|e| ToolError::permanent_string(e.to_string()))?)
    }

    fn name(&self) -> &str {
        "greeting"
    }

    fn description(&self) -> &str {
        "Greets a user by name, with enhanced greeting if signer context is available"
    }
}

/// A tool that demonstrates error classification for retry logic
#[derive(Clone)]
struct NetworkTool;

#[async_trait]
impl Tool for NetworkTool {
    async fn execute(
        &self,
        params: serde_json::Value,
        _app_context: &ApplicationContext,
    ) -> Result<JobResult, ToolError> {
        let operation = params["operation"].as_str().unwrap_or("ping");

        match operation {
            "ping" => {
                // Simulate a successful network operation
                Ok(JobResult::success(&"pong")
                    .map_err(|e| ToolError::permanent_string(e.to_string()))?)
            }
            "timeout" => {
                // Simulate a network timeout (retriable error)
                Err(ToolError::retriable_with_source(
                    std::io::Error::new(std::io::ErrorKind::TimedOut, "Connection timed out"),
                    "Network request timed out, should retry",
                ))
            }
            "rate_limit" => {
                // Simulate rate limiting (retriable with delay)
                Err(ToolError::rate_limited_with_source(
                    std::io::Error::new(std::io::ErrorKind::Other, "Too many requests"),
                    "API rate limit exceeded",
                    Some(std::time::Duration::from_secs(30)),
                ))
            }
            "invalid" => {
                // Simulate invalid input (permanent error, don't retry)
                Err(ToolError::invalid_input_with_source(
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "Bad request"),
                    "Invalid operation parameter",
                ))
            }
            _ => Err(ToolError::permanent_string(format!(
                "Unknown operation: {}",
                operation
            ))),
        }
    }

    fn name(&self) -> &str {
        "network"
    }

    fn description(&self) -> &str {
        "Demonstrates network operations with different error types for retry logic"
    }
}

/// Mock signer for demonstration purposes
#[derive(Debug)]
struct MockSigner {
    user_id: String,
}

impl riglr_core::signer::SignerBase for MockSigner {
    fn user_id(&self) -> Option<String> {
        Some(self.user_id.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl UnifiedSigner for MockSigner {
    fn supports_solana(&self) -> bool {
        false // This mock signer doesn't support any blockchain operations
    }

    fn supports_evm(&self) -> bool {
        false
    }

    fn as_solana(&self) -> Option<&dyn riglr_core::signer::SolanaSigner> {
        None
    }

    fn as_evm(&self) -> Option<&dyn riglr_core::signer::EvmSigner> {
        None
    }

    fn as_multi_chain(&self) -> Option<&dyn riglr_core::signer::MultiChainSigner> {
        None
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== riglr-core Basic Worker Example ===\n");

    // Create a worker with default configuration
    let config = ExecutionConfig::default();
    let app_context = ApplicationContext::from_env();
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(config, app_context);

    // Register our tools
    worker.register_tool(Arc::new(GreetingTool)).await;
    worker.register_tool(Arc::new(NetworkTool)).await;

    println!("✅ Created worker and registered tools\n");

    // Example 1: Process a job without signer context
    println!("📝 Example 1: Job without signer context");
    let job1 = Job::new("greeting", &serde_json::json!({"name": "riglr"}), 3)?;

    let result1 = worker.process_job(job1).await?;
    match result1 {
        JobResult::Success { value, .. } => {
            println!("✅ Success: {}", value);
        }
        JobResult::Failure { error, .. } => {
            println!("❌ Failed: {}", error);
        }
    }

    // Example 2: Process a job with signer context
    println!("\n📝 Example 2: Job with signer context");
    let mock_signer = Arc::new(MockSigner {
        user_id: "alice_123".to_string(),
    });

    let result2 = SignerContext::with_signer(mock_signer, async {
        let job2 = Job::new("greeting", &serde_json::json!({"name": "Bob"}), 3)
            .map_err(|e| riglr_core::signer::SignerError::Configuration(e.to_string()))?;
        worker
            .process_job(job2)
            .await
            .map_err(|e| riglr_core::signer::SignerError::ProviderError(e.to_string()))
    })
    .await?;

    match result2 {
        JobResult::Success { value, .. } => {
            println!("✅ Success with signer: {}", value);
        }
        JobResult::Failure { error, .. } => {
            println!("❌ Failed: {}", error);
        }
    }

    // Example 3: Demonstrate error classification
    println!("\n📝 Example 3: Error classification examples");

    let test_cases = vec![
        ("ping", "Should succeed"),
        ("timeout", "Should be retriable"),
        ("rate_limit", "Should be rate limited"),
        ("invalid", "Should be permanent failure"),
        ("unknown", "Should be permanent failure"),
    ];

    for (operation, expected) in test_cases {
        let job = Job::new("network", &serde_json::json!({"operation": operation}), 3)?;

        let result = worker.process_job(job).await?;
        match result {
            JobResult::Success { value, .. } => {
                println!("✅ {}: {} -> Success: {}", operation, expected, value);
            }
            JobResult::Failure { ref error } => {
                println!(
                    "❌ {}: {} -> Error: {} (retriable: {})",
                    operation,
                    expected,
                    error,
                    result.is_retriable()
                );
                // Error data is now embedded within the ToolError structure
            }
        }
    }

    // Example 4: Idempotent job processing
    println!("\n📝 Example 4: Idempotent job processing");
    let idempotent_job = Job::new_idempotent(
        "greeting",
        &serde_json::json!({"name": "Charlie"}),
        3,
        "greeting_charlie_unique_key",
    )?;

    // Process the job twice - should get the same result from idempotency store
    let result_a = worker.process_job(idempotent_job.clone()).await?;
    let result_b = worker.process_job(idempotent_job).await?;

    println!("🔄 First execution result: {:?}", result_a);
    println!("🔄 Second execution result: {:?}", result_b);
    println!("✅ Both results should be identical due to idempotency");

    println!("\n🎉 Basic worker example completed!");
    println!("\n🔧 Key takeaways:");
    println!("   • ToolWorker provides automatic retry logic based on error classification");
    println!("   • SignerContext enables secure multi-tenant operation");
    println!("   • Idempotency prevents duplicate processing of the same operation");
    println!("   • Enhanced error types provide structured failure information");

    Ok(())
}
