//! Basic example showing how to set up and use a `ToolWorker` with the new architecture
//!
//! This demonstrates:
//! - Creating a `ToolWorker` with default configuration
//! - Registering tools
//! - Processing jobs with `UnifiedSigner` context
//! - Enhanced error handling with retry classification

use async_trait::async_trait;
use core::{error::Error, time::Duration};
use riglr_config::Config;
use riglr_core::{
    idempotency::InMemoryIdempotencyStore,
    provider::ApplicationContext,
    signer::{
        error::Standard, Chain, EvmSigner, SignerBase, SignerContext, SignerError, SolanaSigner,
        UnifiedSigner,
    },
    tool::Worker,
    ExecutionConfig, Job, JobResult, Tool, ToolError,
};
use std::{
    io::{Error as IoError, ErrorKind},
    sync::Arc,
};

/// A simple example tool that works with any signer type
#[derive(Clone)]
struct GreetingTool;

#[async_trait]
impl Tool for GreetingTool {
    type Args = serde_json::Value;
    type Output = JobResult;
    type Error = ToolError;

    fn name(&self) -> &'static str {
        "greeting"
    }

    fn description(&self) -> &'static str {
        "Greets a user by name, with enhanced greeting if signer context is available"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            }
        })
    }

    async fn call(&self, params: Self::Args) -> Result<Self::Output, Self::Error> {
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("World");

        // Check if we have a signer context for enhanced greeting
        if SignerContext::is_available() {
            let signer =
                SignerContext::current().map_err(|e| ToolError::permanent_string(e.to_string()))?;

            let user_id = signer.user_id();
            let greeting = format!("Hello, {name}! (User: {user_id})");
            return JobResult::success(&greeting)
                .map_err(|e| ToolError::permanent_string(e.to_string()));
        }

        // Fallback greeting without signer context
        Ok(JobResult::success(&format!("Hello, {name}!"))
            .map_err(|e| ToolError::permanent_string(e.to_string()))?)
    }
}

/// A tool that demonstrates error classification for retry logic
#[derive(Clone)]
struct NetworkTool;

#[async_trait]
impl Tool for NetworkTool {
    type Args = serde_json::Value;
    type Output = JobResult;
    type Error = ToolError;

    fn name(&self) -> &'static str {
        "network"
    }

    fn description(&self) -> &'static str {
        "Demonstrates network operations with different error types for retry logic"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"}
            }
        })
    }

    async fn call(&self, params: Self::Args) -> Result<Self::Output, Self::Error> {
        let operation = params
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("ping");

        match operation {
            "ping" => {
                // Simulate a successful network operation
                Ok(JobResult::success(&"pong")
                    .map_err(|e| ToolError::permanent_string(e.to_string()))?)
            }
            "timeout" => {
                // Simulate a network timeout (retriable error)
                Err(ToolError::retriable_with_source(
                    IoError::new(ErrorKind::TimedOut, "Connection timed out"),
                    "Network request timed out, should retry",
                ))
            }
            "rate_limit" => {
                // Simulate rate limiting (retriable with delay)
                Err(ToolError::rate_limited_with_source(
                    IoError::other("Too many requests"),
                    "API rate limit exceeded",
                    Some(Duration::from_secs(30)),
                ))
            }
            "invalid" => {
                // Simulate invalid input (permanent error, don't retry)
                Err(ToolError::invalid_input_with_source(
                    IoError::new(ErrorKind::InvalidInput, "Bad request"),
                    "Invalid operation parameter",
                ))
            }
            _ => Err(ToolError::permanent_string(format!(
                "Unknown operation: {operation}"
            ))),
        }
    }
}

/// Mock signer for demonstration purposes
#[derive(Debug)]
struct MockSigner {
    user_id: String,
}

impl SignerBase for MockSigner {
    fn supported_chains(&self) -> &[Chain] {
        &[] // Mock signer supports no actual chains
    }

    fn user_id(&self) -> String {
        self.user_id.clone()
    }
}

impl UnifiedSigner for MockSigner {
    fn as_solana(&self) -> Option<&dyn SolanaSigner> {
        None
    }

    fn as_evm(&self) -> Option<&dyn EvmSigner> {
        None
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("=== riglr-core Basic Worker Example ===\n");

    // Create a worker with default configuration
    let exec_config = ExecutionConfig::default();
    let config = Config::from_env();
    let app_context = ApplicationContext::from_config(&config);
    let worker = Worker::<InMemoryIdempotencyStore>::new(exec_config, app_context);

    // Register our tools
    worker.register_tool(Arc::new(GreetingTool));
    worker.register_tool(Arc::new(NetworkTool));

    println!("✅ Created worker and registered tools\n");

    // Example 1: Process a job without signer context
    println!("📝 Example 1: Job without signer context");
    let job1 = Job::new("greeting", &serde_json::json!({"name": "riglr"}), 3)?;

    let result1 = worker.process_job(job1).await?;
    match result1 {
        JobResult::Success { value, .. } => {
            println!("✅ Success: {value}");
        }
        JobResult::Failure { error, .. } => {
            println!("❌ Failed: {error}");
        }
        _ => {
            println!("🔄 Unexpected result type");
        }
    }

    // Example 2: Process a job with signer context
    println!("\n📝 Example 2: Job with signer context");
    let mock_signer = Arc::new(MockSigner {
        user_id: "alice_123".to_string(),
    });

    let result2 = SignerContext::with_signer(mock_signer, async {
        let job2 = Job::new("greeting", &serde_json::json!({"name": "Bob"}), 3).map_err(|e| {
            Box::new(Standard::Configuration(e.to_string())) as Box<dyn SignerError>
        })?;
        worker
            .process_job(job2)
            .await
            .map_err(|e| Box::new(Standard::Generic(e.to_string())) as Box<dyn SignerError>)
    })
    .await
    .map_err(|e| anyhow::anyhow!("Signer error: {}", e))?;

    match result2 {
        JobResult::Success { value, .. } => {
            println!("✅ Success with signer: {value}");
        }
        JobResult::Failure { error, .. } => {
            println!("❌ Failed: {error}");
        }
        _ => {
            println!("🔄 Unexpected result type");
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
                println!("✅ {operation}: {expected} -> Success: {value}");
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
            _ => {
                println!("🔄 {operation}: {expected} -> Unexpected result type");
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

    println!("🔄 First execution result: {result_a:?}");
    println!("🔄 Second execution result: {result_b:?}");
    println!("✅ Both results should be identical due to idempotency");

    println!("\n🎉 Basic worker example completed!");
    println!("\n🔧 Key takeaways:");
    println!("   • ToolWorker provides automatic retry logic based on error classification");
    println!("   • SignerContext enables secure multi-tenant operation");
    println!("   • Idempotency prevents duplicate processing of the same operation");
    println!("   • Enhanced error types provide structured failure information");

    Ok(())
}
