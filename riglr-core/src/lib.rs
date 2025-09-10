//! # riglr-core
//!
//! Chain-agnostic foundation for the riglr ecosystem, providing core abstractions
//! for multi-blockchain tool orchestration without SDK dependencies.
//!
//! ## Design Philosophy
//!
//! riglr-core maintains strict chain-agnosticism through:
//! - **Type Erasure**: Blockchain clients stored as `Arc<dyn Any>`
//! - **Serialization Boundaries**: Transactions passed as bytes/JSON
//! - **Dependency Injection**: ApplicationContext pattern for runtime injection
//! - **Unidirectional Flow**: Tools depend on core, never the reverse
//!
//! ## Architecture Overview
//!
//! The riglr-core crate provides three main architectural patterns:
//!
//! ### 1. Unified Tool Architecture with ApplicationContext
//!
//! All tools now use a single `ApplicationContext` parameter that provides access to RPC clients,
//! configuration, and other shared resources. Tools are defined with the `#[tool]` macro:
//!
//! ```ignore
//! use riglr_core::{Tool, ToolError, provider::ApplicationContext};
//! use riglr_macros::tool;
//!
//! #[tool]
//! async fn my_tool(
//!     param1: String,
//!     param2: u64,
//!     context: &ApplicationContext,
//! ) -> Result<serde_json::Value, ToolError> {
//!     // Access RPC client from context
//!     // In practice, use concrete types from blockchain SDKs:
//!     // let rpc_client = context.get_extension::<Arc<MyRpcClient>>()
//!     //     .ok_or_else(|| ToolError::permanent_string("RPC client not available"))?;
//!
//!     // Access configuration
//!     let config = &context.config;
//!
//!     // Perform operations...
//!     Ok(serde_json::json!({ "result": "success" }))
//! }
//! ```
//!
//! ### 2. `ToolWorker` Lifecycle
//!
//! Orchestrates tool execution with proper error handling and ApplicationContext:
//!
//! ```ignore
//! use riglr_core::{ToolWorker, ExecutionConfig, provider::ApplicationContext};
//! use riglr_core::idempotency::InMemoryIdempotencyStore;
//! use riglr_config::Config;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::from_env();
//! let context = ApplicationContext::from_config(&config);
//!
//! let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
//!     ExecutionConfig::default(),
//!     context
//! );
//!
//! // Execute tools with automatic retry logic
//! // let result = worker.process_job(job).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### 3. `ToolError` Structure and Classification
//!
//! The new ToolError structure provides rich error classification with context:
//! - `Permanent { source, source_message, context }`: Non-retriable errors
//! - `Retriable { source, source_message, context }`: Temporary issues that can be retried
//! - `RateLimited { source, source_message, context, retry_after }`: Rate limiting with optional backoff
//! - `InvalidInput { source, source_message, context }`: Input validation failures
//! - `SignerContext(String)`: Signer-related errors
//!
//! ## Integration with rig
//!
//! riglr-core extends rig's agent capabilities with blockchain-specific tooling
//! while maintaining compatibility with rig's execution model.
//!
//! ### Key Components
//!
//! - **[`SignerContext`]** - Thread-safe signer management for transactional operations
//! - **[`UnifiedSigner`]** - Trait for blockchain transaction signing across chains
//! - **[`ApplicationContext`]** - Dependency injection container with RPC client extensions
//! - **[`ToolWorker`]** - Resilient tool execution engine with retry logic and timeouts
//! - **[`JobQueue`]** - Distributed job processing with Redis backend
//! - **[`Tool`]** - Core trait for defining executable tools with error handling
//! - **[`Job`]** - Work unit representation with retry and idempotency support
//! - **[`JobResult`]** - Structured results distinguishing success, retriable, and permanent failures
//! - **[`retry_async`]** - Centralized retry logic with exponential backoff
//!
//! ### Quick Start Example
//!
//! ```ignore
//! use riglr_core::{ToolWorker, ExecutionConfig, Job, JobResult, ToolError};
//! use riglr_core::{idempotency::InMemoryIdempotencyStore, provider::ApplicationContext};
//! use riglr_macros::tool;
//! use riglr_config::Config;
//! use std::sync::Arc;
//!
//! // Define a simple tool using the #[tool] macro
//! #[tool]
//! async fn greeting_tool(
//!     name: Option<String>,
//!     context: &ApplicationContext,
//! ) -> Result<serde_json::Value, ToolError> {
//!     let name = name.unwrap_or_else(|| "World".to_string());
//!     Ok(serde_json::json!({
//!         "message": format!("Hello, {}!", name),
//!         "timestamp": chrono::Utc::now()
//!     }))
//! }
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Set up ApplicationContext
//! let config = Config::from_env();
//! let context = ApplicationContext::from_config(&config);
//!
//! // Set up worker with context
//! let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
//!     ExecutionConfig::default(),
//!     context
//! );
//!
//! // Register your tool
//! worker.register_tool(Arc::new(greeting_tool)).await;
//!
//! // Create and process a job
//! let job = Job::new(
//!     "greeting_tool",
//!     &serde_json::json!({"name": "riglr"}),
//!     3 // max retries
//! )?;
//!
//! let result = worker.process_job(job).await.unwrap();
//! println!("Result: {:?}", result);
//! # Ok(())
//! # }
//! ```
//!
//! ### Architecture Patterns
//!
//! #### 1. Unified Tool Architecture
//!
//! All tools now use ApplicationContext for consistent access to resources:
//!
//! ```ignore
//! use riglr_core::{ToolError, provider::ApplicationContext};
//! use riglr_macros::tool;
//! use std::sync::Arc;
//!
//! // READ-ONLY OPERATIONS: Access RPC client from context
//! #[tool]
//! async fn get_sol_balance(
//!     address: String,
//!     context: &ApplicationContext,
//! ) -> Result<serde_json::Value, ToolError> {
//!     // In practice, use concrete RPC client types from blockchain SDKs:
//!     // let rpc_client = context.get_extension::<Arc<MyRpcClient>>()
//!     //     .ok_or_else(|| ToolError::permanent_string("RPC client not available"))?;
//!     
//!     // Query balance using RPC client
//!     // ...
//!     Ok(serde_json::json!({ "balance_sol": 1.5 }))
//! }
//!
//! // TRANSACTIONAL OPERATIONS: Use SignerContext within tools
//! #[tool]
//! async fn transfer_sol(
//!     recipient: String,
//!     amount: f64,
//!     context: &ApplicationContext,
//! ) -> Result<serde_json::Value, ToolError> {
//!     // Access signer through SignerContext::current()
//!     // Perform transaction signing and submission
//!     // ...
//!     Ok(serde_json::json!({
//!         "transaction_hash": "ABC123",
//!         "amount": amount,
//!         "recipient": recipient
//!     }))
//! }
//! ```
//!
//! #### 2. Resilient Tool Execution
//!
//! The [`ToolWorker`] provides automatic retry logic, timeouts, idempotency checking,
//! and resource management:
//!
//! - **Exponential backoff** for failed operations
//! - **Configurable timeouts** to prevent hanging operations
//! - **Idempotency store** integration for safe retries
//! - **Resource limits** to prevent system overload
//! - **Comprehensive metrics** for monitoring
//!
//! #### 3. Error Classification
//!
//! The system distinguishes between different types of errors to enable intelligent retry logic:
//!
//! - **Retriable errors**: Network timeouts, rate limits, temporary service unavailability
//! - **Permanent errors**: Invalid parameters, insufficient funds, authorization failures
//! - **System errors**: Configuration issues, internal failures
//!
//! ### Features
//!
//! - `redis` - Enable Redis-backed job queue and idempotency store (default)
//! - `tokio` - Async runtime support (required)
//! - `tracing` - Structured logging and observability
//!
//! ### Production Considerations
//!
//! For production deployments, consider:
//!
//! - Setting appropriate resource limits based on your infrastructure
//! - Configuring Redis with persistence and clustering for reliability
//! - Implementing proper monitoring and alerting on worker metrics
//! - Using structured logging with correlation IDs for debugging
//! - Setting up dead letter queues for failed job analysis

/// Error types and handling for riglr-core operations.
pub mod error;
/// Idempotency store implementations for ensuring operation uniqueness.
pub mod idempotency;
/// Job definition and processing structures.
pub mod jobs;
/// Application context and RPC provider infrastructure.
pub mod provider;
/// Extension traits for ergonomic client access.
pub mod provider_extensions;
/// Queue implementations for distributed job processing.
pub mod queue;
/// Retry logic with exponential backoff for resilient operations.
pub mod retry;
/// Sentiment analysis abstraction for news and social media content.
pub mod sentiment;
/// Thread-safe signer context and transaction signing abstractions.
pub mod signer;
/// Task spawning utilities that preserve signer context across async boundaries.
pub mod spawn;
/// Core tool trait and execution infrastructure.
pub mod tool;
/// Internal utility functions and helpers.
pub mod util;

// Re-export configuration types from riglr-config
pub use riglr_config::{
    AppConfig, Config, DatabaseConfig, Environment, FeaturesConfig, NetworkConfig, ProvidersConfig,
};

pub use error::{CoreError, ToolError};
pub use idempotency::*;
pub use jobs::*;
pub use queue::*;
pub use signer::SignerContext;
pub use signer::SignerError;
pub use signer::UnifiedSigner;
pub use tool::*;
// Note: util functions are for internal use only
// Environment variable functions should not be used by library consumers

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[derive(Clone)]
    struct MockTool {
        name: String,
        should_fail: bool,
    }

    #[async_trait::async_trait]
    impl Tool for MockTool {
        async fn execute(
            &self,
            params: serde_json::Value,
            _context: &crate::provider::ApplicationContext,
        ) -> Result<JobResult, ToolError> {
            if self.should_fail {
                return Err(ToolError::permanent_string("Mock tool failure"));
            }

            let message = params["message"].as_str().unwrap_or("Hello");
            Ok(JobResult::success(&format!("{}: {}", self.name, message))?)
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            ""
        }
    }

    #[tokio::test]
    async fn test_job_creation() -> anyhow::Result<()> {
        let job = Job::new("test_tool", &serde_json::json!({"message": "test"}), 3)?;

        assert_eq!(job.tool_name, "test_tool");
        assert_eq!(job.max_retries, 3);
        assert_eq!(job.retry_count, 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_job_result_success() -> anyhow::Result<()> {
        let result = JobResult::success(&"test result")?;

        match result {
            JobResult::Success { value, tx_hash } => {
                assert_eq!(value, serde_json::json!("test result"));
                assert!(tx_hash.is_none());
            }
            _ => panic!("Expected success result"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_job_result_failure() {
        let result = JobResult::Failure {
            error: crate::error::ToolError::retriable_string("test error"),
        };

        match result {
            JobResult::Failure { ref error } => {
                assert!(error.contains("test error"));
                assert!(result.is_retriable());
            }
            _ => panic!("Expected failure result"),
        }
    }

    #[tokio::test]
    async fn test_tool_worker_creation() {
        let _worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            provider::ApplicationContext::default(),
        );

        // Verify worker was created successfully - creation itself is the test
    }

    #[tokio::test]
    async fn test_tool_registration_and_execution() -> anyhow::Result<()> {
        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            provider::ApplicationContext::default(),
        );

        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            should_fail: false,
        });

        worker.register_tool(tool).await;

        let job = Job::new(
            "test_tool",
            &serde_json::json!({"message": "Hello World"}),
            3,
        )?;

        let result = worker.process_job(job).await;
        assert!(result.is_ok());

        match result.unwrap() {
            JobResult::Success { value, .. } => {
                assert!(value.as_str().unwrap().contains("Hello World"));
            }
            _ => panic!("Expected successful job result"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_tool_error_handling() -> anyhow::Result<()> {
        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            provider::ApplicationContext::default(),
        );

        let tool = Arc::new(MockTool {
            name: "failing_tool".to_string(),
            should_fail: true,
        });

        worker.register_tool(tool).await;

        let job = Job::new("failing_tool", &serde_json::json!({"message": "test"}), 3)?;

        let result = worker.process_job(job).await;
        assert!(result.is_ok());

        // The tool should handle errors gracefully and return a failure result
        match result.unwrap() {
            JobResult::Failure { error, .. } => {
                assert!(error.contains("Mock tool failure"));
            }
            _ => panic!("Expected failure job result"),
        }

        Ok(())
    }

    #[test]
    fn test_tool_error_types() {
        let retriable = ToolError::retriable_string("Network timeout");
        assert!(retriable.is_retriable());
        assert!(!retriable.is_rate_limited());

        let rate_limited = ToolError::rate_limited_string("API rate limit exceeded");
        assert!(rate_limited.is_retriable());
        assert!(rate_limited.is_rate_limited());

        let permanent = ToolError::permanent_string("Invalid parameters");
        assert!(!permanent.is_retriable());
        assert!(!permanent.is_rate_limited());
    }

    #[test]
    fn test_error_conversions() {
        let anyhow_error = anyhow::anyhow!("Test error");
        let tool_error: ToolError = ToolError::permanent_string(anyhow_error.to_string());
        assert!(!tool_error.is_retriable());

        let string_error = "Test string error".to_string();
        let tool_error: ToolError = ToolError::permanent_string(string_error);
        assert!(!tool_error.is_retriable());

        let str_error = "Test str error";
        let tool_error: ToolError = ToolError::permanent_string(str_error);
        assert!(!tool_error.is_retriable());
    }

    #[tokio::test]
    async fn test_execution_config() {
        let config = ExecutionConfig::default();

        assert!(config.default_timeout > std::time::Duration::from_millis(0));
        assert!(config.max_concurrency > 0);
        assert!(config.initial_retry_delay > std::time::Duration::from_millis(0));
    }

    #[tokio::test]
    async fn test_idempotency_store() -> anyhow::Result<()> {
        let store = InMemoryIdempotencyStore::new();
        let key = "test_key";
        let value = serde_json::json!({"test": "value"});

        // Check key doesn't exist initially
        assert!(store.get(key).await?.is_none());

        // Store a value
        let job_result = JobResult::success(&value)?;
        store
            .set(
                key,
                Arc::new(job_result),
                std::time::Duration::from_secs(60),
            )
            .await?;

        // Retrieve the value
        let retrieved = store.get(key).await?;
        assert!(retrieved.is_some());
        // Verify the stored result matches what we expect
        if let Some(arc_result) = retrieved {
            if let JobResult::Success { value, .. } = arc_result.as_ref() {
                assert_eq!(*value, serde_json::json!({"test": "value"}));
            } else {
                panic!("Expected Success variant");
            }
        }

        Ok(())
    }

    // Additional comprehensive tests for 100% coverage

    #[test]
    fn test_job_result_success_with_tx_hash() -> anyhow::Result<()> {
        let result = JobResult::success_with_tx(&"test result", "0x123abc")?;

        match result {
            JobResult::Success { value, tx_hash } => {
                assert_eq!(value, serde_json::json!("test result"));
                assert_eq!(tx_hash, Some("0x123abc".to_string()));
            }
            _ => panic!("Expected success result with tx hash"),
        }
        Ok(())
    }

    #[test]
    fn test_job_result_permanent_failure() {
        let result = JobResult::Failure {
            error: crate::error::ToolError::permanent_string("invalid parameters"),
        };

        match result {
            JobResult::Failure { ref error } => {
                assert!(error.contains("invalid parameters"));
                assert!(!result.is_retriable());
            }
            _ => panic!("Expected permanent failure result"),
        }
    }

    #[test]
    fn test_job_retry_logic() -> anyhow::Result<()> {
        let mut job = Job::new("test_tool", &serde_json::json!({"test": "data"}), 3)?;

        assert!(job.can_retry());
        assert_eq!(job.retry_count, 0);

        // Simulate retries
        job.retry_count = 1;
        assert!(job.can_retry());

        job.retry_count = 2;
        assert!(job.can_retry());

        job.retry_count = 3;
        assert!(!job.can_retry());

        job.retry_count = 4;
        assert!(!job.can_retry());

        Ok(())
    }

    #[test]
    fn test_job_new_idempotent() -> anyhow::Result<()> {
        let job = Job::new_idempotent(
            "test_tool",
            &serde_json::json!({"message": "test"}),
            5,
            "unique_key_123",
        )?;

        assert_eq!(job.tool_name, "test_tool");
        assert_eq!(job.max_retries, 5);
        assert_eq!(job.retry_count, 0);
        assert_eq!(job.idempotency_key, Some("unique_key_123".to_string()));
        Ok(())
    }

    #[test]
    fn test_job_new_idempotent_without_key() -> anyhow::Result<()> {
        let job = Job::new("test_tool", &serde_json::json!({"message": "test"}), 2)?;

        assert_eq!(job.tool_name, "test_tool");
        assert_eq!(job.max_retries, 2);
        assert_eq!(job.retry_count, 0);
        assert!(job.idempotency_key.is_none());
        Ok(())
    }

    #[test]
    fn test_core_error_display() {
        let queue_error = CoreError::Queue("Failed to connect".to_string());
        assert!(queue_error
            .to_string()
            .contains("Queue error: Failed to connect"));

        let job_error = CoreError::JobExecution("Timeout occurred".to_string());
        assert!(job_error
            .to_string()
            .contains("Job execution error: Timeout occurred"));

        let generic_error = CoreError::Generic("Something went wrong".to_string());
        assert!(generic_error
            .to_string()
            .contains("Core error: Something went wrong"));
    }

    #[test]
    fn test_core_error_from_serde_json() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let core_error: CoreError = json_error.into();

        match core_error {
            CoreError::Serialization(_) => (),
            _ => panic!("Expected Serialization error"),
        }
    }

    #[test]
    fn test_tool_error_permanent() {
        let error = ToolError::permanent_string("Authorization failed");
        assert!(!error.is_retriable());
        assert!(!error.is_rate_limited());
        assert!(error.to_string().contains("Authorization failed"));
    }

    #[test]
    fn test_tool_error_retriable() {
        let error = ToolError::retriable_string("Connection timeout");
        assert!(error.is_retriable());
        assert!(!error.is_rate_limited());
        assert!(error.to_string().contains("Connection timeout"));
    }

    #[test]
    fn test_tool_error_rate_limited() {
        let error = ToolError::rate_limited_string("Too many requests");
        assert!(error.is_retriable());
        assert!(error.is_rate_limited());
        assert!(error.to_string().contains("Too many requests"));
    }

    #[test]
    fn test_tool_error_from_anyhow() {
        let anyhow_error = anyhow::anyhow!("Test anyhow error");
        let tool_error: ToolError = ToolError::permanent_string(anyhow_error.to_string());
        assert!(!tool_error.is_retriable());
        assert!(!tool_error.is_rate_limited());
    }

    #[test]
    fn test_tool_error_from_str() {
        let str_error = "Test str error";
        let tool_error: ToolError = ToolError::permanent_string(str_error);
        assert!(!tool_error.is_retriable());
        assert!(!tool_error.is_rate_limited());
        assert!(tool_error.to_string().contains("Test str error"));
    }

    #[test]
    fn test_tool_error_from_string() {
        let string_error = "Test string error".to_string();
        let tool_error: ToolError = ToolError::permanent_string(string_error);
        assert!(!tool_error.is_retriable());
        assert!(!tool_error.is_rate_limited());
        assert!(tool_error.to_string().contains("Test string error"));
    }

    #[test]
    fn test_execution_config_customization() {
        let mut config = ExecutionConfig::default();

        // Test default values exist
        assert!(config.default_timeout > std::time::Duration::from_millis(0));
        assert!(config.max_concurrency > 0);
        assert!(config.initial_retry_delay > std::time::Duration::from_millis(0));

        // Test that we can modify the config
        config.max_concurrency = 10;
        config.default_timeout = std::time::Duration::from_secs(30);
        config.initial_retry_delay = std::time::Duration::from_millis(100);

        assert_eq!(config.max_concurrency, 10);
        assert_eq!(config.default_timeout, std::time::Duration::from_secs(30));
        assert_eq!(
            config.initial_retry_delay,
            std::time::Duration::from_millis(100)
        );
    }

    #[tokio::test]
    async fn test_tool_worker_with_non_existent_tool() -> anyhow::Result<()> {
        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            provider::ApplicationContext::default(),
        );

        let job = Job::new(
            "non_existent_tool",
            &serde_json::json!({"message": "test"}),
            1,
        )?;

        let result = worker.process_job(job).await;

        // Should return error for missing tool
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("not found") || error.to_string().contains("Tool"));

        Ok(())
    }

    #[tokio::test]
    async fn test_job_with_empty_parameters() -> anyhow::Result<()> {
        let job = Job::new("test_tool", &serde_json::json!({}), 1)?;

        assert_eq!(job.tool_name, "test_tool");
        assert_eq!(job.params, serde_json::json!({}));
        assert_eq!(job.max_retries, 1);
        assert_eq!(job.retry_count, 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_job_with_null_parameters() -> anyhow::Result<()> {
        let job = Job::new("test_tool", &serde_json::Value::Null, 1)?;

        assert_eq!(job.tool_name, "test_tool");
        assert_eq!(job.params, serde_json::Value::Null);
        assert_eq!(job.max_retries, 1);
        assert_eq!(job.retry_count, 0);
        Ok(())
    }

    #[test]
    fn test_job_retry_boundary_conditions() -> anyhow::Result<()> {
        // Test with max_retries = 0
        let job = Job::new("test_tool", &serde_json::json!({}), 0)?;
        assert!(!job.can_retry());

        // Test with max_retries = 1
        let mut job = Job::new("test_tool", &serde_json::json!({}), 1)?;
        assert!(job.can_retry());
        job.retry_count = 1;
        assert!(!job.can_retry());

        Ok(())
    }

    #[tokio::test]
    async fn test_mock_tool_with_empty_message() -> anyhow::Result<()> {
        let tool = MockTool {
            name: "test_tool".to_string(),
            should_fail: false,
        };

        let context = provider::ApplicationContext::default();
        let result = tool.execute(serde_json::json!({}), &context).await.unwrap();

        match result {
            JobResult::Success { value, .. } => {
                assert!(value.as_str().unwrap().contains("test_tool"));
                assert!(value.as_str().unwrap().contains("Hello")); // Default message
            }
            _ => panic!("Expected success result"),
        }

        Ok(())
    }

    #[test]
    fn test_mock_tool_name_and_description() {
        let tool = MockTool {
            name: "custom_tool".to_string(),
            should_fail: false,
        };

        assert_eq!(tool.name(), "custom_tool");
        assert_eq!(tool.description(), "");
    }

    #[tokio::test]
    async fn test_worker_multiple_tool_registration() -> anyhow::Result<()> {
        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            provider::ApplicationContext::default(),
        );

        let tool1 = Arc::new(MockTool {
            name: "tool1".to_string(),
            should_fail: false,
        });

        let tool2 = Arc::new(MockTool {
            name: "tool2".to_string(),
            should_fail: false,
        });

        worker.register_tool(tool1).await;
        worker.register_tool(tool2).await;

        // Test both tools work
        let job1 = Job::new("tool1", &serde_json::json!({"message": "test1"}), 1)?;
        let result1 = worker.process_job(job1).await?;

        let job2 = Job::new("tool2", &serde_json::json!({"message": "test2"}), 1)?;
        let result2 = worker.process_job(job2).await?;

        match (&result1, &result2) {
            (JobResult::Success { .. }, JobResult::Success { .. }) => (),
            _ => panic!("Both tools should succeed"),
        }

        Ok(())
    }

    #[test]
    fn test_job_result_serialization() -> anyhow::Result<()> {
        let success_result = JobResult::success(&"test data")?;
        let serialized = serde_json::to_string(&success_result)?;
        let deserialized: JobResult = serde_json::from_str(&serialized)?;

        match deserialized {
            JobResult::Success { value, tx_hash } => {
                assert_eq!(value, serde_json::json!("test data"));
                assert!(tx_hash.is_none());
            }
            _ => panic!("Expected success result after deserialization"),
        }

        Ok(())
    }

    #[test]
    fn test_job_result_failure_serialization() -> anyhow::Result<()> {
        let failure_result = JobResult::Failure {
            error: crate::error::ToolError::retriable_string("network error"),
        };
        let serialized = serde_json::to_string(&failure_result)?;
        let deserialized: JobResult = serde_json::from_str(&serialized)?;

        match deserialized {
            JobResult::Failure { ref error } => {
                assert!(error.contains("network error"));
                assert!(deserialized.is_retriable());
            }
            _ => panic!("Expected failure result after deserialization"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_idempotency_store_expiration() -> anyhow::Result<()> {
        let store = InMemoryIdempotencyStore::new();
        let key = "expiring_key";
        let value = serde_json::json!({"test": "expiry"});

        let job_result = JobResult::success(&value)?;

        // Set with very short expiration
        store
            .set(
                key,
                Arc::new(job_result),
                std::time::Duration::from_millis(1),
            )
            .await?;

        // Wait for expiration
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Key should be expired/removed
        let retrieved = store.get(key).await?;
        assert!(retrieved.is_none());

        Ok(())
    }
}
