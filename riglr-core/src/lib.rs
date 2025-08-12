//! # riglr-core
//!
//! The foundational crate for the riglr ecosystem, providing core abstractions for
//! multi-chain tool orchestration and execution within the rig framework.
//!
//! This crate provides the foundational components for building resilient AI agents,
//! including job queues, execution engines, and core data structures.
//!
//! ## Architecture Overview
//!
//! The riglr-core crate provides three main components:
//!
//! ### 1. SignerContext Pattern
//!
//! Thread-local storage for multi-tenant blockchain client management:
//!
//! ```rust
//! use riglr_core::signer::SignerContext;
//! 
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Set context for current operation
//! // Create your signer implementation
//! // SignerContext::with_signer(signer, async_code).await;
//!
//! // Tools automatically use the context
//! // No need to pass clients as parameters
//! # Ok(())
//! # }
//! ```
//!
//! ### 2. ToolWorker Lifecycle
//!
//! Orchestrates tool execution with proper error handling:
//!
//! ```rust
//! use riglr_core::{ToolWorker, ExecutionConfig};
//! use riglr_core::idempotency::InMemoryIdempotencyStore;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
//!     ExecutionConfig::default()
//! );
//! 
//! // Execute tools with automatic retry logic
//! // let result = worker.process_job(job).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### 3. ToolError Classification Philosophy
//!
//! Errors are classified for intelligent retry logic:
//! - `Permanent`: Requires human intervention (invalid parameters, auth failures)
//! - `Retriable`: Temporary issues (network timeouts, service unavailable)
//! - `RateLimited`: Specific rate limiting with backoff strategies
//!
//! ## Integration with rig
//!
//! riglr-core extends rig's agent capabilities with blockchain-specific tooling
//! while maintaining compatibility with rig's execution model.
//!
//! ### Key Components
//!
//! - **[`SignerContext`]** - Thread-safe signer management for multi-tenant operations
//! - **[`TransactionSigner`]** - Trait for blockchain transaction signing across chains
//! - **[`ToolWorker`]** - Resilient tool execution engine with retry logic and timeouts
//! - **[`JobQueue`]** - Distributed job processing with Redis backend
//! - **[`Tool`]** - Core trait for defining executable tools with error handling
//! - **[`Job`]** - Work unit representation with retry and idempotency support
//! - **[`JobResult`]** - Structured results distinguishing success, retriable, and permanent failures
//!
//! ### Quick Start Example
//!
//! ```rust
//! use riglr_core::{
//!     ToolWorker, ExecutionConfig, Tool, Job, JobResult,
//!     idempotency::InMemoryIdempotencyStore
//! };
//! use async_trait::async_trait;
//! use std::sync::Arc;
//!
//! // Define a simple tool
//! struct GreetingTool;
//!
//! #[async_trait]
//! impl Tool for GreetingTool {
//!     async fn execute(
//!         &self,
//!         params: serde_json::Value,
//!     ) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
//!         let name = params["name"].as_str().unwrap_or("World");
//!         Ok(JobResult::success(&format!("Hello, {}!", name))?)
//!     }
//!
//!     fn name(&self) -> &str {
//!         "greeting"
//!     }
//! }
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Set up worker with default configuration
//! let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
//!     ExecutionConfig::default()
//! );
//!
//! // Register your tool
//! worker.register_tool(Arc::new(GreetingTool)).await;
//!
//! // Create and process a job
//! let job = Job::new(
//!     "greeting",
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
//! #### 1. Signer Context Pattern
//!
//! The [`SignerContext`] provides secure, thread-local access to cryptographic signers,
//! enabling tools to perform blockchain operations without directly handling private keys:
//!
//! ```rust
//! use riglr_core::SignerContext;
//! use riglr_solana_tools::LocalSolanaSigner;
//! use std::sync::Arc;
//! # use solana_sdk::signer::keypair::Keypair;
//!
//! # async fn signer_example() -> anyhow::Result<()> {
//! let keypair = Keypair::new();
//! let signer = Arc::new(LocalSolanaSigner::new(
//!     keypair,
//!     "https://api.devnet.solana.com".to_string()
//! ));
//!
//! // Execute code with signer context
//! SignerContext::with_signer(signer, async {
//!     // Tools can now access the signer via SignerContext::current()
//!     let current_signer = SignerContext::current().await?;
//!     Ok(())
//! }).await?;
//! # Ok(())
//! # }
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

pub mod config;
pub mod error;
pub mod idempotency;
pub mod jobs;
pub mod queue;
pub mod signer;
pub mod tool;
pub mod transactions;
pub mod util;

pub use config::*;
pub use error::{ToolError, CoreError, SignerError};
pub use idempotency::*;
pub use jobs::*;
pub use queue::*;
pub use signer::{SignerContext, TransactionSigner};
pub use tool::*;
pub use util::*;

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
        ) -> std::result::Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
            if self.should_fail {
                return Err("Mock tool failure".into());
            }
            
            let message = params["message"].as_str().unwrap_or("Hello");
            Ok(JobResult::success(&format!("{}: {}", self.name, message))?)
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_job_creation() -> anyhow::Result<()> {
        let job = Job::new(
            "test_tool",
            &serde_json::json!({"message": "test"}),
            3
        )?;

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
        let result = JobResult::retriable_failure("test error");
        
        match result {
            JobResult::Failure { error, retriable } => {
                assert_eq!(error, "test error");
                assert!(retriable);
            }
            _ => panic!("Expected failure result"),
        }
    }

    #[tokio::test]
    async fn test_tool_worker_creation() {
        let _worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default()
        );
        
        // Verify worker was created successfully - creation itself is the test
    }

    #[tokio::test]
    async fn test_tool_registration_and_execution() -> anyhow::Result<()> {
        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default()
        );

        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            should_fail: false,
        });

        worker.register_tool(tool).await;

        let job = Job::new(
            "test_tool",
            &serde_json::json!({"message": "Hello World"}),
            3
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
            ExecutionConfig::default()
        );

        let tool = Arc::new(MockTool {
            name: "failing_tool".to_string(),
            should_fail: true,
        });

        worker.register_tool(tool).await;

        let job = Job::new(
            "failing_tool",
            &serde_json::json!({"message": "test"}),
            3
        )?;

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
        let retriable = ToolError::retriable("Network timeout");
        assert!(retriable.is_retriable());
        assert!(!retriable.is_rate_limited());

        let rate_limited = ToolError::rate_limited("API rate limit exceeded");
        assert!(rate_limited.is_retriable());
        assert!(rate_limited.is_rate_limited());

        let permanent = ToolError::permanent("Invalid parameters");
        assert!(!permanent.is_retriable());
        assert!(!permanent.is_rate_limited());
    }

    #[test]
    fn test_error_conversions() {
        let anyhow_error = anyhow::anyhow!("Test error");
        let tool_error: ToolError = anyhow_error.into();
        assert!(!tool_error.is_retriable());

        let string_error = "Test string error".to_string();
        let tool_error: ToolError = string_error.into();
        assert!(!tool_error.is_retriable());

        let str_error = "Test str error";
        let tool_error: ToolError = str_error.into();
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
        store.set(key, &job_result, std::time::Duration::from_secs(60)).await?;

        // Retrieve the value
        let retrieved = store.get(key).await?;
        assert!(retrieved.is_some());
        // Verify the stored result matches what we expect
        if let Some(JobResult::Success { value, .. }) = retrieved {
            assert_eq!(value, serde_json::json!({"test": "value"}));
        } else {
            panic!("Expected Success variant");
        }

        Ok(())
    }
}