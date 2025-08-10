//! # riglr-core
//!
//! Core abstractions and job execution engine for riglr.
//!
//! This crate provides the foundational components for building resilient AI agents,
//! including job queues, execution engines, and core data structures.
//!
//! ## Architecture Overview
//!
//! The `riglr-core` crate is built around several key abstractions that work together
//! to provide a robust foundation for AI agent systems:
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
//! let result = worker.process_job(job).await?;
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
//! use riglr_core::{SignerContext, signer::LocalSolanaSigner};
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

pub mod error;
pub mod idempotency;
pub mod jobs;
pub mod queue;
pub mod signer;
pub mod tool;

pub use error::*;
pub use idempotency::*;
pub use jobs::*;
pub use queue::*;
pub use signer::*;
pub use tool::*;
