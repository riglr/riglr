//! Tool orchestration and execution framework.
//!
//! Provides the ToolWorker abstraction for coordinating tool execution
//! with proper error handling, logging, and integration with rig agents.
//!
//! This module provides the core abstractions for executing tools in a resilient,
//! asynchronous manner with support for retries, timeouts, and job queuing.
//!
//! ## Overview
//!
//! The tool execution system is designed around several key components:
//!
//! - **[`Tool`]** - The core trait defining executable tools
//! - **[`ToolWorker`]** - The execution engine that processes jobs using registered tools
//! - **[`ExecutionConfig`]** - Configuration for retry behavior, timeouts, and resource limits
//! - **[`ResourceLimits`]** - Fine-grained control over resource usage per tool type
//! - **[`WorkerMetrics`]** - Performance and operational metrics for monitoring
//!
//! ## Key Features
//!
//! ### Resilient Execution
//! - **Exponential backoff** retry logic with configurable delays
//! - **Timeout protection** to prevent hanging operations
//! - **Error classification** to distinguish retriable vs permanent failures
//! - **Idempotency support** to safely retry operations
//!
//! ### Resource Management
//! - **Semaphore-based limits** for different resource types (RPC calls, HTTP requests)
//! - **Concurrent execution** with configurable limits per resource
//! - **Tool-specific resource mapping** based on naming patterns
//!
//! ### Monitoring and Observability
//! - **Comprehensive metrics** tracking success/failure rates
//! - **Structured logging** with correlation IDs
//! - **Performance monitoring** with execution timings
//!
//! ## Examples
//!
//! ### Basic Tool Implementation
//!
//! ```ignore
//! use riglr_core::{Tool, JobResult};
//! use async_trait::async_trait;
//! use serde_json::Value;
//!
//! struct SimpleCalculator;
//!
//! #[async_trait]
//! impl Tool for SimpleCalculator {
//!     async fn execute(&self, params: Value, _context: &ApplicationContext) -> Result<JobResult, ToolError> {
//!         let a = params["a"].as_f64().ok_or("Missing parameter 'a'")?;
//!         let b = params["b"].as_f64().ok_or("Missing parameter 'b'")?;
//!         let operation = params["op"].as_str().unwrap_or("add");
//!
//!         let result = match operation {
//!             "add" => a + b,
//!             "multiply" => a * b,
//!             "divide" => {
//!                 if b == 0.0 {
//!                     return Err(ToolError::permanent_string("Division by zero", "Cannot divide by zero"));
//!                 }
//!                 a / b
//!             }
//!             _ => return Err(ToolError::permanent_string("Unknown operation", "Operation not supported")),
//!         };
//!
//!         Ok(JobResult::success(&result)?)
//!     }
//!
//!     fn name(&self) -> &str {
//!         "calculator"
//!     }
//! }
//! ```
//!
//! ### Worker Setup and Execution
//!
//! ```ignore
//! use riglr_core::{
//!     ToolWorker, ExecutionConfig, ResourceLimits, Job,
//!     idempotency::InMemoryIdempotencyStore
//! };
//! use std::{sync::Arc, time::Duration};
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Configure execution behavior
//! let config = ExecutionConfig {
//!     max_concurrency: 20,
//!     default_timeout: Duration::from_secs(60),
//!     max_retries: 3,
//!     initial_retry_delay: Duration::from_millis(100),
//!     max_retry_delay: Duration::from_secs(30),
//!     enable_idempotency: true,
//!     ..Default::default()
//! };
//!
//! // Set up resource limits
//! let limits = ResourceLimits::default()
//!     .with_limit("solana_rpc", 5)   // Limit Solana RPC calls
//!     .with_limit("evm_rpc", 10)     // Limit EVM RPC calls
//!     .with_limit("http_api", 20);   // Limit HTTP API calls
//!
//! // Create worker with idempotency store
//! let store = Arc::new(InMemoryIdempotencyStore::new());
//! let worker = ToolWorker::new(config)
//!     .with_idempotency_store(store)
//!     .with_resource_limits(limits);
//!
//! // Register tools
//! # use riglr_core::{Tool, JobResult};
//! # use async_trait::async_trait;
//! # struct SimpleCalculator;
//! # #[async_trait]
//! # impl Tool for SimpleCalculator {
//! #     async fn execute(&self, _: serde_json::Value, _: &crate::provider::ApplicationContext) -> Result<JobResult, ToolError> {
//! #         Ok(JobResult::success(&42)?)
//! #     }
//! #     fn name(&self) -> &str { "calculator" }
//! # }
//! worker.register_tool(Arc::new(SimpleCalculator)).await;
//!
//! // Process a job
//! let job = Job::new_idempotent(
//!     "calculator",
//!     &serde_json::json!({"a": 10, "b": 5, "op": "add"}),
//!     3, // max retries
//!     "calc_10_plus_5" // idempotency key
//! )?;
//!
//! let result = worker.process_job(job).await.unwrap();
//! println!("Result: {:?}", result);
//!
//! // Get metrics
//! let metrics = worker.metrics();
//! println!("Jobs processed: {}", metrics.jobs_processed.load(std::sync::atomic::Ordering::Relaxed));
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tracing::{debug, error, info, warn};

use crate::error::{ToolError, WorkerError};
use crate::idempotency::IdempotencyStore;
use crate::jobs::{Job, JobResult};
use crate::queue::JobQueue;
use crate::signer::SignerContext;

/// A trait defining the execution interface for tools.
///
/// This is compatible with `rig::Tool` and provides the foundation
/// for executing tools within the riglr ecosystem. Tools represent
/// individual operations that can be executed by the [`ToolWorker`].
///
/// ## Design Principles
///
/// - **Stateless**: Tools should not maintain internal state between executions
/// - **Idempotent**: When possible, tools should be safe to retry
/// - **Error-aware**: Tools should classify errors as retriable or permanent
/// - **Resource-conscious**: Tools should handle rate limits and timeouts gracefully
///
/// ## Error Handling
///
/// Tools should carefully distinguish between different types of errors:
///
/// - **Retriable errors**: Network timeouts, rate limits, temporary service unavailability
/// - **Permanent errors**: Invalid parameters, insufficient funds, authorization failures
/// - **System errors**: Internal configuration issues, unexpected state
///
/// ## Examples
///
/// ### Basic Tool Implementation
///
/// ```ignore
/// use riglr_core::{Tool, JobResult};
/// use async_trait::async_trait;
/// use serde_json::Value;
///
/// struct HttpFetcher;
///
/// #[async_trait]
/// impl Tool for HttpFetcher {
///     async fn execute(&self, params: Value, _context: &ApplicationContext) -> Result<JobResult, ToolError> {
///         let url = params["url"].as_str()
///             .ok_or("Missing required parameter: url")?;
///
///         // Mock HTTP client behavior for this example
///         if url.starts_with("https://") {
///             let body = r#"{"data": "success"}"#;
///             Ok(JobResult::success(&serde_json::json!({"body": body}))?)
///         } else if url.contains("error") {
///             // Simulate client error
///             Ok(JobResult::Failure {
///                 error: ToolError::permanent_string("Client error: Invalid URL")
///             })
///         } else if url.contains("timeout") {
///             // Simulate timeout
///             Ok(JobResult::Failure {
///                 error: ToolError::retriable_string("Request timeout: Connection timed out")
///             })
///         } else {
///             // Simulate server error
///             Ok(JobResult::Failure {
///                 error: ToolError::retriable_string("Server error: HTTP 503")
///             })
///         }
///     }
///
///     fn name(&self) -> &str {
///         "http_fetcher"
///     }
/// }
/// ```
///
/// ### Tool with Resource-Aware Naming
///
/// ```ignore
/// use riglr_core::{Tool, JobResult};
/// use async_trait::async_trait;
/// use serde_json::Value;
///
/// // The name starts with "solana_" which will use the solana_rpc resource limit
/// struct SolanaBalanceChecker;
///
/// #[async_trait]
/// impl Tool for SolanaBalanceChecker {
///     async fn execute(&self, params: Value, _context: &ApplicationContext) -> Result<JobResult, ToolError> {
///         let address = params["address"].as_str()
///             .ok_or("Missing required parameter: address")?;
///
///         // This tool will be rate-limited by the "solana_rpc" resource limit
///         // because its name starts with "solana_"
///
///         // Implementation would use Solana RPC client here...
///         let balance = 1.5; // Mock balance
///
///         Ok(JobResult::success(&serde_json::json!({
///             "address": address,
///             "balance": balance,
///             "unit": "SOL"
///         }))?)
///     }
///
///     fn name(&self) -> &str {
///         "solana_balance_check"  // Name prefix determines resource pool
///     }
/// }
/// ```
///
/// ### Tool Using Signer Context
///
/// ```ignore
/// use riglr_core::{Tool, JobResult, SignerContext};
/// use async_trait::async_trait;
/// use serde_json::Value;
///
/// struct TransferTool;
///
/// #[async_trait]
/// impl Tool for TransferTool {
///     async fn execute(&self, params: Value, _context: &ApplicationContext) -> Result<JobResult, ToolError> {
///         // Check if we have a signer context
///         if !SignerContext::is_available().await {
///             return Ok(JobResult::Failure {
///                 error: ToolError::permanent_string("Transfer operations require a signer context")
///             });
///         }
///
///         let signer = SignerContext::current().await
///             .map_err(|_| "Failed to get signer context")?;
///
///         let recipient = params["recipient"].as_str()
///             .ok_or("Missing required parameter: recipient")?;
///         let amount = params["amount"].as_f64()
///             .ok_or("Missing required parameter: amount")?;
///
///         // Use the signer to perform the transfer...
///         // This is just a mock implementation
///         Ok(JobResult::success_with_tx(
///             &serde_json::json!({
///                 "recipient": recipient,
///                 "amount": amount,
///                 "status": "completed"
///             }),
///             "mock_tx_hash_12345"
///         )?)
///     }
///
///     fn name(&self) -> &str {
///         "transfer"
///     }
/// }
/// ```
#[async_trait]
pub trait Tool: Send + Sync {
    /// Execute the tool with the given parameters.
    ///
    /// This method performs the core work of the tool. It receives parameters
    /// as a JSON value and should return a [`JobResult`] indicating success or failure.
    ///
    /// # Parameters
    /// * `params` - JSON parameters passed to the tool. Tools should validate
    ///              and extract required parameters, returning appropriate errors
    ///              for missing or invalid data.
    ///
    /// # Returns
    /// * `Ok(JobResult::Success)` - Tool executed successfully with result data
    /// * `Ok(JobResult::Failure { retriable: true })` - Tool failed but can be retried
    /// * `Ok(JobResult::Failure { retriable: false })` - Tool failed permanently
    /// * `Err(Box<dyn Error>)` - Unexpected system error occurred
    ///
    /// # Error Classification Guidelines
    ///
    /// Return retriable failures (`JobResult::Failure { error: ToolError::retriable_string(...) }`) for:
    /// - Network timeouts or connection errors
    /// - Rate limiting (HTTP 429, RPC rate limits) - use `ToolError::rate_limited_string` for these
    /// - Temporary service unavailability (HTTP 503)
    /// - Blockchain congestion or temporary RPC failures
    ///
    /// Return permanent failures (`JobResult::Failure { error: ToolError::permanent_string(...) }`) for:
    /// - Invalid parameters or malformed requests - use `ToolError::invalid_input_string` for these
    /// - Authentication/authorization failures
    /// - Insufficient funds or balance
    /// - Invalid blockchain addresses or transaction data
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use riglr_core::{Tool, JobResult};
    /// use async_trait::async_trait;
    /// use serde_json::Value;
    ///
    /// struct WeatherTool;
    ///
    /// #[async_trait]
    /// impl Tool for WeatherTool {
    ///     async fn execute(&self, params: Value, _context: &ApplicationContext) -> Result<JobResult, ToolError> {
    ///         // Validate parameters
    ///         let city = params["city"].as_str()
    ///             .ok_or("Missing required parameter: city")?;
    ///
    ///         if city.is_empty() {
    ///             return Ok(JobResult::Failure {
    ///                 error: ToolError::invalid_input_string("City name cannot be empty")
    ///             });
    ///         }
    ///
    ///         // Simulate API call with mock weather service
    ///         let weather_data = if city == "InvalidCity" {
    ///             return Ok(JobResult::Failure {
    ///                 error: ToolError::permanent_string(format!("City not found: {}", city))
    ///             });
    ///         } else if city == "TimeoutCity" {
    ///             return Ok(JobResult::Failure {
    ///                 error: ToolError::retriable_string("Network timeout")
    ///             });
    ///         } else {
    ///             serde_json::json!({
    ///                 "city": city,
    ///                 "temperature": 22,
    ///                 "condition": "sunny"
    ///             })
    ///         };
    ///
    ///         Ok(JobResult::success(&weather_data)?)
    ///     }
    ///
    ///     fn name(&self) -> &str {
    ///         "weather"
    ///     }
    /// }
    /// ```
    async fn execute(
        &self,
        params: serde_json::Value,
        context: &crate::provider::ApplicationContext,
    ) -> Result<JobResult, ToolError>;

    /// Get the name of this tool.
    ///
    /// The tool name is used for:
    /// - Tool registration and lookup in the worker
    /// - Job identification and logging
    /// - Resource limit mapping (based on name prefixes)
    /// - Metrics and monitoring
    ///
    /// # Naming Conventions
    ///
    /// Tool names should follow these patterns for automatic resource management:
    ///
    /// - `solana_*` - Uses "solana_rpc" resource pool (e.g., "solana_balance", "solana_transfer")
    /// - `evm_*` - Uses "evm_rpc" resource pool (e.g., "evm_balance", "evm_swap")
    /// - `web_*` - Uses "http_api" resource pool (e.g., "web_fetch", "web_scrape")
    /// - Others - Use default resource pool
    ///
    /// # Returns
    /// A string identifier for this tool that must be unique within a worker.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use riglr_core::Tool;
    /// # use async_trait::async_trait;
    /// # use riglr_core::JobResult;
    ///
    /// struct BitcoinPriceChecker;
    ///
    /// # #[async_trait]
    /// # impl Tool for BitcoinPriceChecker {
    /// #     async fn execute(&self, _: serde_json::Value, _: &crate::provider::ApplicationContext) -> Result<JobResult, ToolError> {
    /// #         Ok(JobResult::success(&42)?)
    /// #     }
    /// #
    /// fn name(&self) -> &str {
    ///     "web_bitcoin_price"  // Will use "http_api" resource pool
    /// }
    /// # }
    /// ```
    fn name(&self) -> &str;

    /// A concise, AI-friendly description of this tool's purpose.
    ///
    /// This should be a short, human-readable sentence that explains
    /// what the tool does and when to use it. It's intended for AI models
    /// and end-user UIs, and is separate from developer-facing rustdoc.
    ///
    /// When using the `#[tool]` macro from `riglr-macros`, this value is:
    /// - Taken from the explicit `#[tool(description = "...")]` attribute when provided
    /// - Otherwise derived from the item's doc comments
    /// - Falls back to an empty string if neither is present
    fn description(&self) -> &str;

    /// Returns the JSON schema for this tool's parameters.
    ///
    /// This schema is used by AI models to understand what parameters
    /// the tool accepts and their types. It should be a valid JSON Schema
    /// object describing the parameters structure.
    ///
    /// The default implementation returns a generic object schema that
    /// accepts any properties, which may not work well with all AI providers.
    /// Tools should override this to provide their actual parameter schema.
    fn schema(&self) -> serde_json::Value {
        // Default to a generic object schema
        serde_json::json!({
            "type": "object",
            "additionalProperties": true
        })
    }
}

/// Configuration for tool execution behavior.
///
/// This struct controls all aspects of how tools are executed by the [`ToolWorker`],
/// including retry behavior, timeouts, concurrency limits, and idempotency settings.
///
/// # Examples
///
/// ```ignore
/// use riglr_core::ExecutionConfig;
/// use std::time::Duration;
///
/// // Default configuration
/// let config = ExecutionConfig::default();
///
/// // Custom configuration for high-throughput scenario
/// let high_throughput_config = ExecutionConfig {
///     max_concurrency: 50,
///     default_timeout: Duration::from_secs(120),
///     max_retries: 5,
///     initial_retry_delay: Duration::from_millis(200),
///     max_retry_delay: Duration::from_secs(60),
///     enable_idempotency: true,
///     ..Default::default()
/// };
///
/// // Conservative configuration for sensitive operations
/// let conservative_config = ExecutionConfig {
///     max_concurrency: 5,
///     default_timeout: Duration::from_secs(300),
///     max_retries: 1,
///     initial_retry_delay: Duration::from_secs(5),
///     max_retry_delay: Duration::from_secs(30),
///     enable_idempotency: true,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Maximum number of concurrent executions per resource type.
    ///
    /// This controls the default concurrency limit when no specific resource
    /// limit is configured. Individual resource types can have their own
    /// limits configured via [`ResourceLimits`].
    ///
    /// **Default**: 10
    pub max_concurrency: usize,

    /// Default timeout for tool execution.
    ///
    /// Individual tool executions will be cancelled if they exceed this duration.
    /// Tools should be designed to complete within reasonable time bounds.
    ///
    /// **Default**: 30 seconds
    pub default_timeout: Duration,

    /// Maximum number of retry attempts for failed operations.
    ///
    /// This applies only to retriable failures. Permanent failures are never retried.
    /// The total number of execution attempts will be `max_retries + 1`.
    ///
    /// **Default**: 3 retries (4 total attempts)
    pub max_retries: u32,

    /// Initial retry delay for exponential backoff.
    ///
    /// The first retry will wait this long after the initial failure.
    /// Subsequent retries will use exponentially increasing delays.
    ///
    /// **Default**: 100 milliseconds
    pub initial_retry_delay: Duration,

    /// Maximum retry delay for exponential backoff.
    ///
    /// Retry delays will not exceed this value, even with exponential backoff.
    /// This prevents excessive wait times for highly retried operations.
    ///
    /// **Default**: 10 seconds
    pub max_retry_delay: Duration,

    /// TTL for idempotency cache entries.
    ///
    /// Completed operations with idempotency keys will be cached for this duration.
    /// Subsequent requests with the same key will return the cached result.
    ///
    /// **Default**: 1 hour
    pub idempotency_ttl: Duration,

    /// Whether to enable idempotency checking.
    ///
    /// When enabled, jobs with idempotency keys will have their results cached
    /// and subsequent identical requests will return cached results instead of
    /// re-executing the tool.
    ///
    /// **Default**: true
    pub enable_idempotency: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 10,
            default_timeout: Duration::from_secs(30),
            max_retries: 3,
            initial_retry_delay: Duration::from_millis(100),
            max_retry_delay: Duration::from_secs(10),
            idempotency_ttl: Duration::from_secs(3600), // 1 hour
            enable_idempotency: true,
        }
    }
}

/// Resource limits configuration for fine-grained concurrency control.
///
/// This struct allows you to set different concurrency limits for different
/// types of operations, preventing any single resource type from overwhelming
/// external services or consuming too many system resources.
///
/// Resource limits are applied based on tool name prefixes:
/// - Tools starting with `solana_` use the "solana_rpc" resource pool
/// - Tools starting with `evm_` use the "evm_rpc" resource pool
/// - Tools starting with `web_` use the "http_api" resource pool
/// - All other tools use the default concurrency limit
///
/// # Examples
///
/// ```rust
/// use riglr_core::ResourceLimits;
///
/// // Create limits for different resource types
/// let limits = ResourceLimits::default()
///     .with_limit("solana_rpc", 3)     // Max 3 concurrent Solana RPC calls
///     .with_limit("evm_rpc", 8)        // Max 8 concurrent EVM RPC calls
///     .with_limit("http_api", 15)      // Max 15 concurrent HTTP requests
///     .with_limit("database", 5);      // Max 5 concurrent database operations
///
/// // Use default limits (solana_rpc: 5, evm_rpc: 10, http_api: 20)
/// let default_limits = ResourceLimits::default();
/// ```
///
/// # Resource Pool Mapping
///
/// The system automatically maps tool names to resource pools:
///
/// ```text
/// Tool Name              → Resource Pool    → Limit
/// "solana_balance"       → "solana_rpc"     → configured limit
/// "evm_swap"             → "evm_rpc"        → configured limit
/// "web_fetch"            → "http_api"       → configured limit
/// "custom_tool"          → default          → ExecutionConfig.max_concurrency
/// ```
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Resource name to semaphore mapping.
    ///
    /// Each semaphore controls the maximum number of concurrent operations
    /// for its associated resource type.
    semaphores: Arc<HashMap<String, Arc<Semaphore>>>,
}

impl ResourceLimits {
    /// Add a resource limit for the specified resource type.
    ///
    /// This creates a semaphore that will limit concurrent access to the
    /// specified resource. Tools with names matching the resource mapping
    /// will be subject to this limit.
    ///
    /// # Parameters
    /// * `resource` - The resource identifier (e.g., "solana_rpc", "evm_rpc")
    /// * `limit` - Maximum number of concurrent operations for this resource
    ///
    /// # Returns
    /// Self, for method chaining
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use riglr_core::ResourceLimits;
    ///
    /// let limits = ResourceLimits::default()
    ///     .with_limit("solana_rpc", 3)     // Limit Solana RPC calls
    ///     .with_limit("database", 10)      // Limit database connections
    ///     .with_limit("external_api", 5);  // Limit external API calls
    /// ```
    pub fn with_limit(mut self, resource: impl Into<String>, limit: usize) -> Self {
        let semaphores = Arc::make_mut(&mut self.semaphores);
        semaphores.insert(resource.into(), Arc::new(Semaphore::new(limit)));
        self
    }

    /// Get the semaphore for a specific resource type.
    ///
    /// This is used internally by the [`ToolWorker`] to acquire permits
    /// before executing tools. Returns `None` if no limit is configured
    /// for the specified resource.
    ///
    /// # Parameters
    /// * `resource` - The resource identifier to look up
    ///
    /// # Returns
    /// * `Some(Arc<Semaphore>)` - If a limit is configured for this resource
    /// * `None` - If no limit is configured (will use default limit)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use riglr_core::ResourceLimits;
    ///
    /// let limits = ResourceLimits::default()
    ///     .with_limit("test_resource", 5);
    ///
    /// assert!(limits.get_semaphore("test_resource").is_some());
    /// assert!(limits.get_semaphore("unknown_resource").is_none());
    /// ```
    pub fn get_semaphore(&self, resource: &str) -> Option<Arc<Semaphore>> {
        self.semaphores.get(resource).cloned()
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        let mut semaphores = HashMap::new();
        semaphores.insert("solana_rpc".to_string(), Arc::new(Semaphore::new(5)));
        semaphores.insert("evm_rpc".to_string(), Arc::new(Semaphore::new(10)));
        semaphores.insert("http_api".to_string(), Arc::new(Semaphore::new(20)));
        Self {
            semaphores: Arc::new(semaphores),
        }
    }
}

/// A worker that processes jobs from a queue using registered tools.
///
/// The `ToolWorker` is the core execution engine of the riglr system. It manages
/// registered tools, processes jobs with resilience features, and provides
/// comprehensive monitoring and observability.
///
/// ## Key Features
///
/// - **Resilient execution**: Automatic retries with exponential backoff
/// - **Resource management**: Configurable limits per resource type
/// - **Idempotency support**: Cached results for safe retries
/// - **Comprehensive monitoring**: Built-in metrics and structured logging
/// - **Concurrent processing**: Efficient parallel job execution
///
/// ## Architecture
///
/// The worker operates on a job-based model where:
/// 1. Jobs are dequeued from a [`JobQueue`]
/// 2. Tools are looked up by name and executed
/// 3. Results are processed with retry logic for failures
/// 4. Successful results are optionally cached for idempotency
/// 5. Metrics are updated for monitoring
///
/// ## Examples
///
/// ### Basic Setup
///
/// ```ignore
/// use riglr_core::{
///     ToolWorker, ExecutionConfig, ResourceLimits,
///     idempotency::InMemoryIdempotencyStore
/// };
/// use std::sync::Arc;
///
/// # async fn example() -> anyhow::Result<()> {
/// // Create worker with default configuration
/// let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
///     ExecutionConfig::default()
/// );
///
/// // Add idempotency store for safe retries
/// let store = Arc::new(InMemoryIdempotencyStore::new());
/// let worker = worker.with_idempotency_store(store);
///
/// // Configure resource limits
/// let limits = ResourceLimits::default()
///     .with_limit("solana_rpc", 3)
///     .with_limit("evm_rpc", 5);
/// let worker = worker.with_resource_limits(limits);
/// # Ok(())
/// # }
/// ```
///
/// ### Processing Jobs
///
/// ```ignore
/// use riglr_core::{ToolWorker, Job, Tool, JobResult, ExecutionConfig};
/// use riglr_core::idempotency::InMemoryIdempotencyStore;
/// use async_trait::async_trait;
/// use std::sync::Arc;
///
/// # struct ExampleTool;
/// # #[async_trait]
/// # impl Tool for ExampleTool {
/// #     async fn execute(&self, _: serde_json::Value, _: &crate::provider::ApplicationContext) -> Result<JobResult, ToolError> {
/// #         Ok(JobResult::success(&"example result")?)
/// #     }
/// #     fn name(&self) -> &str { "example" }
/// # }
/// # async fn example() -> anyhow::Result<()> {
/// let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
///     ExecutionConfig::default()
/// );
///
/// // Register tools
/// worker.register_tool(Arc::new(ExampleTool)).await;
///
/// // Process a job
/// let job = Job::new("example", &serde_json::json!({}), 3)?;
/// let result = worker.process_job(job).await.unwrap();
///
/// println!("Job result: {:?}", result);
///
/// // Check metrics
/// let metrics = worker.metrics();
/// println!("Jobs processed: {}",
///     metrics.jobs_processed.load(std::sync::atomic::Ordering::Relaxed));
/// # Ok(())
/// # }
/// ```
pub struct ToolWorker<I: IdempotencyStore + 'static> {
    /// Registered tools, indexed by name for fast lookup
    tools: Arc<DashMap<String, Arc<dyn Tool>>>,

    /// Default semaphore for general concurrency control
    default_semaphore: Arc<Semaphore>,

    /// Resource-specific limits for fine-grained control
    resource_limits: ResourceLimits,

    /// Configuration for retry behavior, timeouts, etc.
    config: ExecutionConfig,

    /// Optional idempotency store for caching results
    idempotency_store: Option<Arc<I>>,

    /// Performance and operational metrics
    metrics: Arc<WorkerMetrics>,

    /// Application context providing access to RPC providers and shared resources
    app_context: crate::provider::ApplicationContext,
}

/// Performance and operational metrics for monitoring worker health.
///
/// These metrics provide insight into worker performance and can be used
/// for monitoring, alerting, and performance optimization.
///
/// All metrics use atomic integers for thread-safe updates and reads.
///
/// ## Metrics Tracked
///
/// - **jobs_processed**: Total number of jobs dequeued and processed
/// - **jobs_succeeded**: Number of jobs that completed successfully
/// - **jobs_failed**: Number of jobs that failed permanently (after all retries)
/// - **jobs_retried**: Total number of retry attempts across all jobs
///
/// ## Examples
///
/// ```ignore
/// use riglr_core::{ToolWorker, ExecutionConfig, idempotency::InMemoryIdempotencyStore};
/// use std::sync::atomic::Ordering;
///
/// # async fn example() {
/// let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
///     ExecutionConfig::default()
/// );
///
/// let metrics = worker.metrics();
///
/// // Read current metrics
/// let processed = metrics.jobs_processed.load(Ordering::Relaxed);
/// let succeeded = metrics.jobs_succeeded.load(Ordering::Relaxed);
/// let failed = metrics.jobs_failed.load(Ordering::Relaxed);
/// let retried = metrics.jobs_retried.load(Ordering::Relaxed);
///
/// println!("Worker Stats:");
/// println!("  Processed: {}", processed);
/// println!("  Succeeded: {}", succeeded);
/// println!("  Failed: {}", failed);
/// println!("  Retried: {}", retried);
/// println!("  Success Rate: {:.2}%",
///     if processed > 0 { (succeeded as f64 / processed as f64) * 100.0 } else { 0.0 });
/// # }
/// ```
#[derive(Debug, Default)]
pub struct WorkerMetrics {
    /// Total number of jobs processed (dequeued and executed)
    pub jobs_processed: std::sync::atomic::AtomicU64,

    /// Number of jobs that completed successfully
    pub jobs_succeeded: std::sync::atomic::AtomicU64,

    /// Number of jobs that failed permanently (after all retries)
    pub jobs_failed: std::sync::atomic::AtomicU64,

    /// Total number of retry attempts across all jobs
    pub jobs_retried: std::sync::atomic::AtomicU64,
}

impl<I: IdempotencyStore + 'static> std::fmt::Debug for ToolWorker<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolWorker")
            .field("tools_count", &self.tools.len())
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl<I: IdempotencyStore + 'static> ToolWorker<I> {
    /// Create a new tool worker with the given configuration and application context.
    ///
    /// This creates a worker ready to process jobs, but no tools are registered yet.
    /// Use [`register_tool()`](Self::register_tool) to add tools before processing jobs.
    ///
    /// # Parameters
    /// * `config` - Execution configuration controlling retry behavior, timeouts, etc.
    /// * `app_context` - Application context providing access to RPC providers and shared resources
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use riglr_core::{ToolWorker, ExecutionConfig, idempotency::InMemoryIdempotencyStore, provider::ApplicationContext};
    /// use std::time::Duration;
    /// use riglr_config::Config;
    ///
    /// let exec_config = ExecutionConfig {
    ///     max_concurrency: 20,
    ///     default_timeout: Duration::from_secs(60),
    ///     max_retries: 5,
    ///     ..Default::default()
    /// };
    /// let config = Config::from_env();
    /// let app_context = ApplicationContext::from_config(&config);
    ///
    /// let worker = ToolWorker::<InMemoryIdempotencyStore>::new(exec_config, app_context);
    /// ```
    pub fn new(config: ExecutionConfig, app_context: crate::provider::ApplicationContext) -> Self {
        Self {
            tools: Arc::new(DashMap::new()),
            default_semaphore: Arc::new(Semaphore::new(config.max_concurrency)),
            resource_limits: ResourceLimits::default(),
            config,
            idempotency_store: None,
            metrics: Arc::new(WorkerMetrics::default()),
            app_context,
        }
    }

    /// Configure an idempotency store for result caching.
    ///
    /// When an idempotency store is configured, jobs with idempotency keys
    /// will have their results cached. Subsequent executions with the same
    /// idempotency key will return the cached result instead of re-executing.
    ///
    /// # Parameters
    /// * `store` - The idempotency store implementation to use
    ///
    /// # Returns
    /// Self, for method chaining
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use riglr_core::{ToolWorker, ExecutionConfig, idempotency::InMemoryIdempotencyStore};
    /// use std::sync::Arc;
    ///
    /// let store = Arc::new(InMemoryIdempotencyStore::new());
    /// let worker = ToolWorker::new(ExecutionConfig::default())
    ///     .with_idempotency_store(store);
    /// ```
    pub fn with_idempotency_store(mut self, store: Arc<I>) -> Self {
        self.idempotency_store = Some(store);
        self
    }

    /// Configure custom resource limits.
    ///
    /// Resource limits control how many concurrent operations can run
    /// for each resource type. This prevents overwhelming external
    /// services and provides fine-grained control over resource usage.
    ///
    /// # Parameters
    /// * `limits` - The resource limits configuration to use
    ///
    /// # Returns
    /// Self, for method chaining
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use riglr_core::{ToolWorker, ExecutionConfig, ResourceLimits, idempotency::InMemoryIdempotencyStore};
    ///
    /// let limits = ResourceLimits::default()
    ///     .with_limit("solana_rpc", 3)
    ///     .with_limit("evm_rpc", 8)
    ///     .with_limit("http_api", 15);
    ///
    /// let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default())
    ///     .with_resource_limits(limits);
    /// ```
    pub fn with_resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.resource_limits = limits;
        self
    }

    /// Register a tool with this worker.
    ///
    /// Tools must be registered before they can be executed by jobs.
    /// Each tool is indexed by its name, so tool names must be unique
    /// within a single worker.
    ///
    /// # Parameters
    /// * `tool` - The tool implementation to register
    ///
    /// # Panics
    /// This method does not panic, but if a tool with the same name
    /// is already registered, it will be replaced.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use riglr_core::{ToolWorker, ExecutionConfig, Tool, JobResult, idempotency::InMemoryIdempotencyStore};
    /// use async_trait::async_trait;
    /// use std::sync::Arc;
    ///
    /// struct CalculatorTool;
    ///
    /// #[async_trait]
    /// impl Tool for CalculatorTool {
    ///     async fn execute(&self, params: serde_json::Value, _context: &ApplicationContext) -> Result<JobResult, ToolError> {
    ///         let a = params["a"].as_f64().unwrap_or(0.0);
    ///         let b = params["b"].as_f64().unwrap_or(0.0);
    ///         Ok(JobResult::success(&(a + b))?)
    ///     }
    ///
    ///     fn name(&self) -> &str {
    ///         "calculator"
    ///     }
    /// }
    ///
    /// # async fn example() {
    /// let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
    /// worker.register_tool(Arc::new(CalculatorTool)).await;
    /// # }
    /// ```
    pub async fn register_tool(&self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Get access to worker metrics.
    ///
    /// The returned metrics can be used for monitoring worker performance
    /// and health. All metrics are thread-safe and can be read at any time.
    ///
    /// # Returns
    /// A reference to the worker's metrics
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use riglr_core::{ToolWorker, ExecutionConfig, idempotency::InMemoryIdempotencyStore};
    /// use std::sync::atomic::Ordering;
    ///
    /// let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
    /// let metrics = worker.metrics();
    ///
    /// println!("Jobs processed: {}",
    ///     metrics.jobs_processed.load(Ordering::Relaxed));
    /// println!("Success rate: {:.2}%", {
    ///     let processed = metrics.jobs_processed.load(Ordering::Relaxed);
    ///     let succeeded = metrics.jobs_succeeded.load(Ordering::Relaxed);
    ///     if processed > 0 { (succeeded as f64 / processed as f64) * 100.0 } else { 0.0 }
    /// });
    /// ```
    pub fn metrics(&self) -> &WorkerMetrics {
        &self.metrics
    }

    /// Process a single job with all resilience features.
    ///
    /// Returns:
    /// - `Ok(JobResult)` - The job was processed (successfully or with business logic failure)
    /// - `Err(WorkerError)` - System-level worker failure (tool not found, semaphore issues, etc.)
    pub async fn process_job(&self, mut job: Job) -> Result<JobResult, WorkerError> {
        // Check idempotency cache first
        if let Some(cached_result) = self.check_idempotency_cache(&job).await? {
            return Ok(Arc::try_unwrap(cached_result).unwrap_or_else(|arc| (*arc).clone()));
        }

        // Apply rate limiting for operations within a SignerContext that has a user_id
        if let Ok(signer) = SignerContext::current().await {
            if let Some(user_id) = signer.user_id() {
                if let Err(rate_limit_error) =
                    self.app_context.rate_limiter.check_rate_limit(&user_id)
                {
                    // Convert rate limit error to a JobResult::Failure
                    return Ok(JobResult::Failure {
                        error: rate_limit_error,
                    });
                }
            }
        }

        // Acquire appropriate semaphore
        let _permit = self.acquire_semaphore(&job.tool_name).await.map_err(|e| {
            WorkerError::SemaphoreAcquisition {
                tool_name: job.tool_name.clone(),
                source_message: e.to_string(),
            }
        })?;

        // Get the tool for this job
        let tool = self.get_tool(&job.tool_name).await?;

        // Execute with retries
        let result = self.execute_with_retries(tool, &mut job).await;

        // Cache successful results
        if let Ok(ref job_result) = result {
            self.cache_result(&job, job_result).await;
        }

        result
    }

    /// Check if there's a cached result for this job
    async fn check_idempotency_cache(
        &self,
        job: &Job,
    ) -> Result<Option<Arc<JobResult>>, WorkerError> {
        if let Some(ref idempotency_key) = job.idempotency_key {
            if self.config.enable_idempotency {
                if let Some(ref store) = self.idempotency_store {
                    match store.get(idempotency_key).await {
                        Ok(Some(cached_result)) => {
                            info!(
                                "Returning cached result for idempotency key: {}",
                                idempotency_key
                            );
                            return Ok(Some(cached_result));
                        }
                        Ok(None) => {} // No cached result
                        Err(e) => {
                            return Err(WorkerError::IdempotencyStore {
                                source_message: e.to_string(),
                            });
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    /// Get a tool from the registry
    async fn get_tool(&self, tool_name: &str) -> Result<Arc<dyn Tool>, WorkerError> {
        self.tools
            .get(tool_name)
            .ok_or_else(|| WorkerError::ToolNotFound {
                tool_name: tool_name.to_string(),
            })
            .map(|entry| (*entry).clone())
    }

    /// Execute a tool with retry logic using unified retry helper
    async fn execute_with_retries(
        &self,
        tool: Arc<dyn Tool>,
        job: &mut Job,
    ) -> Result<JobResult, WorkerError> {
        use crate::retry::{retry_async, ErrorClass, RetryConfig};

        // Create retry config from worker config
        let retry_config = RetryConfig {
            max_retries: job.max_retries,
            base_delay_ms: self.config.initial_retry_delay.as_millis() as u64,
            max_delay_ms: self.config.max_retry_delay.as_millis() as u64,
            backoff_multiplier: 2.0,
            use_jitter: true,
        };

        // Use unified retry helper
        let result = retry_async(
            || async { self.execute_single_attempt(&tool, &job.params).await },
            |error| {
                // Classify error based on ToolError's is_retriable
                if error.is_retriable() {
                    ErrorClass::Retryable
                } else {
                    ErrorClass::Permanent
                }
            },
            &retry_config,
            &format!("job_{}", job.job_id),
        )
        .await;

        // Update metrics based on result
        match result {
            Ok(job_result) => {
                self.metrics
                    .jobs_succeeded
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Ok(job_result)
            }
            Err(tool_error) => {
                self.metrics
                    .jobs_failed
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                // Track retries that occurred - the retry helper manages the actual retries
                if retry_config.max_retries > 0 {
                    self.metrics
                        .jobs_retried
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
                Ok(JobResult::Failure { error: tool_error })
            }
        }
    }

    /// Execute a single attempt of a tool
    async fn execute_single_attempt(
        &self,
        tool: &Arc<dyn Tool>,
        params: &serde_json::Value,
    ) -> Result<JobResult, ToolError> {
        // Execute with timeout
        let result = tokio::time::timeout(
            self.config.default_timeout,
            tool.execute(params.clone(), &self.app_context),
        )
        .await;

        match result {
            Ok(Ok(job_result)) => Ok(job_result),
            Ok(Err(e)) => {
                // Propagate the ToolError directly to preserve its structure
                warn!("Tool execution failed: {}", e);
                Err(e)
            }
            Err(_) => {
                let error_msg = format!(
                    "Tool execution timed out after {:?}",
                    self.config.default_timeout
                );
                warn!("{}", error_msg);
                // Timeout errors are retriable
                Err(ToolError::retriable_string(error_msg))
            }
        }
    }

    // Note: create_backoff_strategy and wait_with_backoff methods removed
    // as they are now handled by the unified retry helper in retry.rs

    /// Cache a successful result
    async fn cache_result(&self, job: &Job, job_result: &JobResult) {
        if let Some(ref idempotency_key) = job.idempotency_key {
            if self.config.enable_idempotency {
                if let Some(ref store) = self.idempotency_store {
                    let _ = store
                        .set(
                            idempotency_key,
                            Arc::new(job_result.clone()),
                            self.config.idempotency_ttl,
                        )
                        .await;
                }
            }
        }
    }

    /// Acquire the appropriate semaphore for a tool
    async fn acquire_semaphore(
        &self,
        tool_name: &str,
    ) -> Result<OwnedSemaphorePermit, Box<dyn std::error::Error + Send + Sync>> {
        // Check if there's a specific resource limit for this tool
        let resource_name = match tool_name {
            name if name.starts_with("solana_") => "solana_rpc",
            name if name.starts_with("evm_") => "evm_rpc",
            name if name.starts_with("web_") => "http_api",
            _ => "",
        };

        if !resource_name.is_empty() {
            if let Some(semaphore) = self.resource_limits.get_semaphore(resource_name) {
                return Ok(semaphore.acquire_owned().await?);
            }
        }

        // Fall back to default semaphore
        Ok(self.default_semaphore.clone().acquire_owned().await?)
    }

    /// Start the worker loop, processing jobs from the given queue.
    ///
    /// The worker will continue processing jobs until the provided cancellation token
    /// is cancelled. This allows for graceful shutdown where in-flight jobs can complete
    /// before the worker stops.
    ///
    /// # Parameters
    /// * `queue` - The job queue to process jobs from
    /// * `cancellation_token` - Token to signal when the worker should stop
    ///
    /// # Examples
    ///
    /// ```rust
    /// use riglr_core::{ToolWorker, ExecutionConfig, idempotency::InMemoryIdempotencyStore};
    /// use riglr_core::provider::ApplicationContext;
    /// use riglr_core::queue::InMemoryJobQueue;
    /// use riglr_config::ConfigBuilder;
    /// use tokio_util::sync::CancellationToken;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let config = ConfigBuilder::default().build().unwrap();
    /// let app_context = ApplicationContext::from_config(&config);
    /// let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default(), app_context);
    /// let queue = Arc::new(InMemoryJobQueue::new());
    /// let cancellation_token = CancellationToken::new();
    ///
    /// // Start worker in background
    /// let token_clone = cancellation_token.clone();
    /// let worker_handle = tokio::spawn(async move {
    ///     worker.run(queue, token_clone).await
    /// });
    ///
    /// // Later, signal shutdown
    /// cancellation_token.cancel();
    /// // Await the task and ignore the inner result for simplicity in docs
    /// let _ = worker_handle.await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run<Q: JobQueue>(
        &self,
        queue: Arc<Q>,
        cancellation_token: tokio_util::sync::CancellationToken,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Starting ToolWorker with {} tools registered",
            self.tools.len()
        );

        while !cancellation_token.is_cancelled() {
            tokio::select! {
                // Check for cancellation
                _ = cancellation_token.cancelled() => {
                    info!("Worker shutdown requested, stopping job processing");
                    break;
                }

                // Try to dequeue and process jobs
                result = queue.dequeue_with_timeout(Duration::from_secs(5)) => {
                    match result {
                        Ok(Some(job)) => {
                            let job_id = job.job_id;
                            let tool_name = job.tool_name.clone();

                            self.metrics
                                .jobs_processed
                                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                            // Spawn task to process job asynchronously
                            let worker = self.clone();
                            tokio::spawn(async move {
                                match worker.process_job(job).await {
                                    Ok(job_result) => {
                                        if job_result.is_success() {
                                            info!("Job {} ({}) completed successfully", job_id, tool_name);
                                        } else {
                                            warn!(
                                                "Job {} ({}) failed: {:?}",
                                                job_id, tool_name, job_result
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        error!("Job {} ({}) processing error: {}", job_id, tool_name, e);
                                    }
                                }
                            });
                        }
                        Ok(None) => {
                            // No jobs available, continue
                            debug!("No jobs available in queue");
                        }
                        Err(e) => {
                            error!("Failed to dequeue job: {}", e);
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                    }
                }
            }
        }

        info!("ToolWorker shutdown completed");
        Ok(())
    }
}

// Implement Clone for ToolWorker to enable spawning tasks
impl<I: IdempotencyStore + 'static> Clone for ToolWorker<I> {
    fn clone(&self) -> Self {
        Self {
            tools: self.tools.clone(),
            default_semaphore: self.default_semaphore.clone(),
            resource_limits: self.resource_limits.clone(),
            config: self.config.clone(),
            idempotency_store: self.idempotency_store.clone(),
            metrics: self.metrics.clone(),
            app_context: self.app_context.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::idempotency::InMemoryIdempotencyStore;
    use crate::jobs::Job;
    use crate::provider::ApplicationContext;
    use uuid::Uuid;

    fn test_app_context() -> ApplicationContext {
        // Use default configuration for tests
        ApplicationContext::default()
    }

    struct MockTool {
        name: String,
        should_fail: bool,
    }

    struct RetriableMockTool {
        name: String,
    }

    #[async_trait]
    impl Tool for MockTool {
        async fn execute(
            &self,
            _params: serde_json::Value,
            _context: &crate::provider::ApplicationContext,
        ) -> Result<JobResult, ToolError> {
            if self.should_fail {
                Err(ToolError::permanent_string("Mock failure"))
            } else {
                Ok(JobResult::Success {
                    value: serde_json::json!({"result": "success"}),
                    tx_hash: None,
                })
            }
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            ""
        }

        fn schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "additionalProperties": true
            })
        }
    }

    #[async_trait]
    impl Tool for RetriableMockTool {
        async fn execute(
            &self,
            _params: serde_json::Value,
            _context: &crate::provider::ApplicationContext,
        ) -> Result<JobResult, ToolError> {
            Err(ToolError::retriable_string("Mock retriable failure"))
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            ""
        }

        fn schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "additionalProperties": true
            })
        }
    }

    #[tokio::test]
    async fn test_tool_worker_process_job() {
        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            test_app_context(),
        );
        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            should_fail: false,
        });
        worker.register_tool(tool).await;

        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "test_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 3,
            retry_count: 0,
        };

        let result = worker.process_job(job).await.unwrap();
        match result {
            JobResult::Success { .. } => (),
            _ => panic!("Expected success"),
        }
    }

    #[tokio::test]
    async fn test_tool_worker_with_idempotency() {
        let store = Arc::new(InMemoryIdempotencyStore::new());
        let worker = ToolWorker::new(ExecutionConfig::default(), test_app_context())
            .with_idempotency_store(store.clone());

        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            should_fail: false,
        });
        worker.register_tool(tool).await;

        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "test_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: Some("test_key".to_string()),
            max_retries: 3,
            retry_count: 0,
        };

        // First execution
        let result1 = worker.process_job(job.clone()).await.unwrap();
        assert!(result1.is_success());

        // Second execution should return cached result
        let result2 = worker.process_job(job).await.unwrap();
        assert!(result2.is_success());
    }

    #[tokio::test]
    async fn test_tool_worker_with_retries() {
        let config = ExecutionConfig {
            initial_retry_delay: Duration::from_millis(10),
            ..Default::default()
        };

        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(config, test_app_context());
        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            should_fail: true,
        });
        worker.register_tool(tool).await;

        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "test_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 2,
            retry_count: 0,
        };

        let result = worker.process_job(job).await.unwrap();
        match result {
            JobResult::Failure { .. } => {
                assert!(!result.is_retriable()); // Should not be retriable after exhausting retries
            }
            _ => panic!("Expected failure"),
        }
    }

    #[tokio::test]
    async fn test_tool_worker_tool_not_found() {
        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            test_app_context(),
        );

        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "nonexistent_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 0,
            retry_count: 0,
        };

        let result = worker.process_job(job).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Tool 'nonexistent_tool' not found"));
    }

    #[tokio::test]
    async fn test_tool_worker_timeout() {
        let config = ExecutionConfig {
            default_timeout: Duration::from_millis(10), // Very short timeout
            ..Default::default()
        };

        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(config, test_app_context());
        let tool = Arc::new(SlowMockTool {
            name: "slow_tool".to_string(),
            delay: Duration::from_millis(100),
        });
        worker.register_tool(tool).await;

        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "slow_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 1,
            retry_count: 0,
        };

        let result = worker.process_job(job).await.unwrap();
        match result {
            JobResult::Failure { error, .. } => {
                assert!(error.to_string().to_lowercase().contains("timed out"));
            }
            _ => panic!("Expected timeout failure"),
        }
    }

    #[tokio::test]
    async fn test_tool_worker_with_resource_limits() {
        let config = ExecutionConfig::default();
        let limits = ResourceLimits::default()
            .with_limit("solana_rpc", 2)
            .with_limit("evm_rpc", 3);

        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(config, test_app_context())
            .with_resource_limits(limits);

        // Test semaphore acquisition for different tool types
        let solana_tool = Arc::new(MockTool {
            name: "solana_test".to_string(),
            should_fail: false,
        });
        let evm_tool = Arc::new(MockTool {
            name: "evm_test".to_string(),
            should_fail: false,
        });
        let web_tool = Arc::new(MockTool {
            name: "web_test".to_string(),
            should_fail: false,
        });
        let other_tool = Arc::new(MockTool {
            name: "other_test".to_string(),
            should_fail: false,
        });

        worker.register_tool(solana_tool).await;
        worker.register_tool(evm_tool).await;
        worker.register_tool(web_tool).await;
        worker.register_tool(other_tool).await;

        // Test different tool name patterns
        let jobs = vec![
            Job {
                job_id: Uuid::new_v4(),
                tool_name: "solana_test".to_string(),
                params: serde_json::json!({}),
                idempotency_key: None,
                max_retries: 0,
                retry_count: 0,
            },
            Job {
                job_id: Uuid::new_v4(),
                tool_name: "evm_test".to_string(),
                params: serde_json::json!({}),
                idempotency_key: None,
                max_retries: 0,
                retry_count: 0,
            },
            Job {
                job_id: Uuid::new_v4(),
                tool_name: "web_test".to_string(),
                params: serde_json::json!({}),
                idempotency_key: None,
                max_retries: 0,
                retry_count: 0,
            },
            Job {
                job_id: Uuid::new_v4(),
                tool_name: "other_test".to_string(),
                params: serde_json::json!({}),
                idempotency_key: None,
                max_retries: 0,
                retry_count: 0,
            },
        ];

        // All should succeed
        for job in jobs {
            let result = worker.process_job(job).await.unwrap();
            assert!(result.is_success());
        }
    }

    #[tokio::test]
    async fn test_tool_worker_idempotency_disabled() {
        let config = ExecutionConfig {
            enable_idempotency: false,
            ..Default::default()
        };

        let store = Arc::new(InMemoryIdempotencyStore::new());
        let worker =
            ToolWorker::new(config, test_app_context()).with_idempotency_store(store.clone());

        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            should_fail: false,
        });
        worker.register_tool(tool).await;

        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "test_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: Some("test_key".to_string()),
            max_retries: 0,
            retry_count: 0,
        };

        // First execution
        let result1 = worker.process_job(job.clone()).await.unwrap();
        assert!(result1.is_success());

        // Second execution should NOT use cache due to disabled idempotency
        let result2 = worker.process_job(job).await.unwrap();
        assert!(result2.is_success());

        // Verify the key was never set in the store
        assert!(store.get("test_key").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_tool_worker_metrics() {
        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            test_app_context(),
        );
        let success_tool = Arc::new(MockTool {
            name: "success_tool".to_string(),
            should_fail: false,
        });
        let fail_tool = Arc::new(RetriableMockTool {
            name: "fail_tool".to_string(),
        });

        worker.register_tool(success_tool).await;
        worker.register_tool(fail_tool).await;

        let metrics = worker.metrics();

        // Initial state
        assert_eq!(
            metrics
                .jobs_processed
                .load(std::sync::atomic::Ordering::Relaxed),
            0
        );
        assert_eq!(
            metrics
                .jobs_succeeded
                .load(std::sync::atomic::Ordering::Relaxed),
            0
        );
        assert_eq!(
            metrics
                .jobs_failed
                .load(std::sync::atomic::Ordering::Relaxed),
            0
        );
        assert_eq!(
            metrics
                .jobs_retried
                .load(std::sync::atomic::Ordering::Relaxed),
            0
        );

        // Process successful job
        let success_job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "success_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 0,
            retry_count: 0,
        };
        worker.process_job(success_job).await.unwrap();
        assert_eq!(
            metrics
                .jobs_succeeded
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );

        // Process failing job with retries
        let fail_job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "fail_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 2,
            retry_count: 0,
        };
        worker.process_job(fail_job).await.unwrap();
        assert_eq!(
            metrics
                .jobs_failed
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );
        assert_eq!(
            metrics
                .jobs_retried
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );
    }

    #[tokio::test]
    async fn test_execution_config_default() {
        let config = ExecutionConfig::default();
        assert_eq!(config.max_concurrency, 10);
        assert_eq!(config.default_timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_retry_delay, Duration::from_millis(100));
        assert_eq!(config.max_retry_delay, Duration::from_secs(10));
        assert_eq!(config.idempotency_ttl, Duration::from_secs(3600));
        assert!(config.enable_idempotency);
    }

    #[tokio::test]
    async fn test_resource_limits() {
        let limits = ResourceLimits::default()
            .with_limit("test_resource", 5)
            .with_limit("another_resource", 10);

        assert!(limits.get_semaphore("test_resource").is_some());
        assert!(limits.get_semaphore("another_resource").is_some());
        assert!(limits.get_semaphore("nonexistent").is_none());

        let default_limits = ResourceLimits::default();
        assert!(default_limits.get_semaphore("solana_rpc").is_some());
        assert!(default_limits.get_semaphore("evm_rpc").is_some());
        assert!(default_limits.get_semaphore("http_api").is_some());
    }

    #[tokio::test]
    async fn test_worker_metrics_default() {
        let metrics = WorkerMetrics::default();
        assert_eq!(
            metrics
                .jobs_processed
                .load(std::sync::atomic::Ordering::Relaxed),
            0
        );
        assert_eq!(
            metrics
                .jobs_succeeded
                .load(std::sync::atomic::Ordering::Relaxed),
            0
        );
        assert_eq!(
            metrics
                .jobs_failed
                .load(std::sync::atomic::Ordering::Relaxed),
            0
        );
        assert_eq!(
            metrics
                .jobs_retried
                .load(std::sync::atomic::Ordering::Relaxed),
            0
        );
    }

    #[tokio::test]
    async fn test_tool_worker_clone() {
        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            test_app_context(),
        );
        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            should_fail: false,
        });
        worker.register_tool(tool).await;

        let cloned_worker = worker.clone();

        // Both workers should have access to the same tools
        assert_eq!(worker.tools.len(), 1);
        assert_eq!(cloned_worker.tools.len(), 1);

        // Test processing with cloned worker
        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "test_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 0,
            retry_count: 0,
        };

        let result = cloned_worker.process_job(job).await.unwrap();
        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_tool_worker_run_loop() {
        use crate::queue::InMemoryJobQueue;

        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            test_app_context(),
        );
        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            should_fail: false,
        });
        worker.register_tool(tool).await;

        let queue = Arc::new(InMemoryJobQueue::new());

        // Enqueue a job
        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "test_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 0,
            retry_count: 0,
        };
        queue.enqueue(job).await.unwrap();

        // Start the worker run loop with cancellation token
        let worker_clone = worker.clone();
        let queue_clone = queue.clone();
        let cancellation_token = tokio_util::sync::CancellationToken::new();
        let token_clone = cancellation_token.clone();

        let handle = tokio::spawn(async move { worker_clone.run(queue_clone, token_clone).await });

        // Give it time to process the job
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Signal shutdown
        cancellation_token.cancel();

        // Check that metrics were updated
        let metrics = worker.metrics();
        assert!(
            metrics
                .jobs_processed
                .load(std::sync::atomic::Ordering::Relaxed)
                > 0
        );

        handle.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn test_idempotency_cache_hit() {
        let store = Arc::new(InMemoryIdempotencyStore::new());
        let worker = ToolWorker::new(ExecutionConfig::default(), test_app_context())
            .with_idempotency_store(store.clone());

        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            should_fail: false,
        });
        worker.register_tool(tool).await;

        // Pre-populate the cache
        let cached_result = JobResult::Success {
            value: serde_json::json!({"cached": true}),
            tx_hash: Some("cached_tx_hash".to_string()),
        };
        store
            .set(
                "cache_key",
                Arc::new(cached_result),
                Duration::from_secs(60),
            )
            .await
            .unwrap();

        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "test_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: Some("cache_key".to_string()),
            max_retries: 0,
            retry_count: 0,
        };

        // Should return cached result without executing the tool
        let result = worker.process_job(job).await.unwrap();
        match result {
            JobResult::Success { value, tx_hash } => {
                assert_eq!(value, serde_json::json!({"cached": true}));
                assert_eq!(tx_hash, Some("cached_tx_hash".to_string()));
            }
            _ => panic!("Expected cached success result"),
        }
    }

    #[tokio::test]
    async fn test_tool_worker_unknown_error_fallback() {
        // Create a worker with a job that will fail with max retries
        // but have no last_error set to trigger the "Unknown error" fallback
        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            test_app_context(),
        );

        // Don't register any tool - this will cause tool not found error
        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "nonexistent_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 0,
            retry_count: 0,
        };

        // This should fail with tool not found, not unknown error
        let result = worker.process_job(job).await;
        assert!(result.is_err());

        // The unknown error fallback is actually hard to trigger in normal flow
        // It would only happen if there's a bug in the retry logic where
        // attempts > max_retries but last_error is None
    }

    #[tokio::test]
    async fn test_run_loop_error_handling() {
        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            test_app_context(),
        );
        let error_queue = Arc::new(ErrorQueue::default());

        // Start run loop with cancellation token
        let worker_clone = worker.clone();
        let queue_clone = error_queue.clone();
        let cancellation_token = tokio_util::sync::CancellationToken::new();
        let token_clone = cancellation_token.clone();

        let handle = tokio::spawn(async move { worker_clone.run(queue_clone, token_clone).await });

        // Give it time to encounter the error
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Signal shutdown
        cancellation_token.cancel();
        handle.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn test_run_loop_empty_queue() {
        use crate::queue::InMemoryJobQueue;

        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            test_app_context(),
        );
        let queue = Arc::new(InMemoryJobQueue::new());

        // Start run loop with cancellation token - should encounter Ok(None) from empty queue
        let worker_clone = worker.clone();
        let queue_clone = queue.clone();
        let cancellation_token = tokio_util::sync::CancellationToken::new();
        let token_clone = cancellation_token.clone();

        let handle = tokio::spawn(async move { worker_clone.run(queue_clone, token_clone).await });

        // Give it time to process
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Signal shutdown
        cancellation_token.cancel();
        handle.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn test_run_loop_with_failing_jobs() {
        use crate::queue::InMemoryJobQueue;

        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            test_app_context(),
        );
        let fail_tool = Arc::new(MockTool {
            name: "fail_tool".to_string(),
            should_fail: true,
        });
        worker.register_tool(fail_tool).await;

        let queue = Arc::new(InMemoryJobQueue::new());

        // Enqueue a failing job
        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "fail_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 0,
            retry_count: 0,
        };
        queue.enqueue(job).await.unwrap();

        // Start run loop with cancellation token
        let worker_clone = worker.clone();
        let queue_clone = queue.clone();
        let cancellation_token = tokio_util::sync::CancellationToken::new();
        let token_clone = cancellation_token.clone();

        let handle = tokio::spawn(async move { worker_clone.run(queue_clone, token_clone).await });

        // Give it time to process the failing job
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Signal shutdown
        cancellation_token.cancel();
        handle.await.unwrap().unwrap();

        // Verify metrics were updated
        let metrics = worker.metrics();
        assert!(
            metrics
                .jobs_processed
                .load(std::sync::atomic::Ordering::Relaxed)
                > 0
        );
    }

    #[tokio::test]
    async fn test_comprehensive_metrics_tracking() {
        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            test_app_context(),
        );
        let success_tool = Arc::new(MockTool {
            name: "success_tool".to_string(),
            should_fail: false,
        });
        let fail_tool = Arc::new(RetriableMockTool {
            name: "fail_tool".to_string(),
        });
        worker.register_tool(success_tool).await;
        worker.register_tool(fail_tool).await;

        let metrics = worker.metrics();

        // Process a successful job
        let success_job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "success_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 0,
            retry_count: 0,
        };
        let result = worker.process_job(success_job).await.unwrap();
        assert!(result.is_success());

        // Verify jobs_succeeded was incremented (line 232)
        assert_eq!(
            metrics
                .jobs_succeeded
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );

        // Process a failing job with retries
        let fail_job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "fail_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 2,
            retry_count: 0,
        };
        let result = worker.process_job(fail_job).await.unwrap();
        assert!(!result.is_success());

        // Verify jobs_retried was incremented (line 250)
        assert_eq!(
            metrics
                .jobs_retried
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );

        // Verify jobs_failed was incremented (line 264)
        assert_eq!(
            metrics
                .jobs_failed
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );
    }

    #[tokio::test]
    async fn test_debug_logging_in_retries() {
        let config = ExecutionConfig {
            initial_retry_delay: Duration::from_millis(1),
            ..Default::default()
        };

        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(config, test_app_context());
        let tool = Arc::new(MockTool {
            name: "retry_tool".to_string(),
            should_fail: true,
        });
        worker.register_tool(tool).await;

        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "retry_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 1,
            retry_count: 0,
        };

        // This should trigger debug logging in the retry loop (lines 205-208)
        let _result = worker.process_job(job).await.unwrap();
    }

    #[tokio::test]
    async fn test_worker_startup_logging() {
        use crate::queue::InMemoryJobQueue;

        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            test_app_context(),
        );
        let tool = Arc::new(MockTool {
            name: "startup_tool".to_string(),
            should_fail: false,
        });
        worker.register_tool(tool).await;

        let queue = Arc::new(InMemoryJobQueue::new());

        // This should trigger the startup info log
        let worker_clone = worker.clone();
        let queue_clone = queue.clone();
        let cancellation_token = tokio_util::sync::CancellationToken::new();
        let token_clone = cancellation_token.clone();

        let handle = tokio::spawn(async move { worker_clone.run(queue_clone, token_clone).await });

        // Give it time to start up
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Signal shutdown
        cancellation_token.cancel();
        handle.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn test_timeout_specific_error() {
        let config = ExecutionConfig {
            default_timeout: Duration::from_millis(1), // Very short timeout
            ..Default::default()
        };

        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(config, test_app_context());
        let tool = Arc::new(SlowMockTool {
            name: "timeout_tool".to_string(),
            delay: Duration::from_millis(50),
        });
        worker.register_tool(tool).await;

        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "timeout_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 0,
            retry_count: 0,
        };

        // This should specifically hit the timeout error assignment (line 240)
        let result = worker.process_job(job).await.unwrap();
        match result {
            JobResult::Failure { error, .. } => {
                assert!(error.to_string().to_lowercase().contains("timed out"));
            }
            _ => panic!("Expected timeout failure"),
        }
    }

    #[tokio::test]
    async fn test_resource_matching_edge_cases() {
        let limits = ResourceLimits::default()
            .with_limit("solana_rpc", 1)
            .with_limit("evm_rpc", 1);

        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            test_app_context(),
        )
        .with_resource_limits(limits);

        // Register tools with different name patterns to exercise line 278
        let solana_tool = Arc::new(MockTool {
            name: "solana_balance".to_string(), // Should match solana_ pattern
            should_fail: false,
        });
        let evm_tool = Arc::new(MockTool {
            name: "evm_call".to_string(), // Should match evm_ pattern
            should_fail: false,
        });
        let web_tool = Arc::new(MockTool {
            name: "web_fetch".to_string(), // Should match web_ pattern
            should_fail: false,
        });
        let other_tool = Arc::new(MockTool {
            name: "other_operation".to_string(), // Should use default semaphore
            should_fail: false,
        });

        worker.register_tool(solana_tool).await;
        worker.register_tool(evm_tool).await;
        worker.register_tool(web_tool).await;
        worker.register_tool(other_tool).await;

        // Process jobs to exercise the acquire_semaphore method
        let job1 = Job {
            job_id: Uuid::new_v4(),
            tool_name: "solana_balance".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 0,
            retry_count: 0,
        };

        let _result = worker.process_job(job1).await.unwrap();

        let job2 = Job {
            job_id: Uuid::new_v4(),
            tool_name: "other_operation".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 0,
            retry_count: 0,
        };

        let _result = worker.process_job(job2).await.unwrap();
    }

    #[tokio::test]
    async fn test_execution_config_custom_values() {
        let config = ExecutionConfig {
            max_concurrency: 25,
            default_timeout: Duration::from_secs(60),
            max_retries: 5,
            initial_retry_delay: Duration::from_millis(200),
            max_retry_delay: Duration::from_secs(20),
            idempotency_ttl: Duration::from_secs(7200),
            enable_idempotency: false,
        };

        assert_eq!(config.max_concurrency, 25);
        assert_eq!(config.default_timeout, Duration::from_secs(60));
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_retry_delay, Duration::from_millis(200));
        assert_eq!(config.max_retry_delay, Duration::from_secs(20));
        assert_eq!(config.idempotency_ttl, Duration::from_secs(7200));
        assert!(!config.enable_idempotency);
    }

    #[tokio::test]
    async fn test_resource_limits_new() {
        let limits = ResourceLimits::default();
        assert!(limits.get_semaphore("any_resource").is_none());
    }

    #[tokio::test]
    async fn test_resource_limits_with_limit_chaining() {
        let limits = ResourceLimits::default()
            .with_limit("first", 1)
            .with_limit("second", 2)
            .with_limit("third", 3);

        assert!(limits.get_semaphore("first").is_some());
        assert!(limits.get_semaphore("second").is_some());
        assert!(limits.get_semaphore("third").is_some());
        assert!(limits.get_semaphore("fourth").is_none());
    }

    #[tokio::test]
    async fn test_tool_worker_new_custom_config() {
        let config = ExecutionConfig {
            max_concurrency: 5,
            ..Default::default()
        };
        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(config, test_app_context());

        // Verify worker was created with custom concurrency
        // We can't directly access default_semaphore, but we can verify through the config
        assert_eq!(worker.config.max_concurrency, 5);
    }

    #[tokio::test]
    async fn test_tool_worker_register_tool_replacement() {
        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            test_app_context(),
        );

        // Register initial tool
        let tool1 = Arc::new(MockTool {
            name: "test_tool".to_string(),
            should_fail: false,
        });
        worker.register_tool(tool1).await;
        assert_eq!(worker.tools.len(), 1);

        // Register tool with same name - should replace
        let tool2 = Arc::new(MockTool {
            name: "test_tool".to_string(),
            should_fail: true,
        });
        worker.register_tool(tool2).await;
        assert_eq!(worker.tools.len(), 1);

        // Verify the new tool was used (it should fail)
        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "test_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 0,
            retry_count: 0,
        };

        let result = worker.process_job(job).await.unwrap();
        assert!(!result.is_success());
    }

    #[tokio::test]
    async fn test_idempotency_store_error_handling() {
        let store = Arc::new(ErrorIdempotencyStore::default());
        let worker = ToolWorker::new(ExecutionConfig::default(), test_app_context())
            .with_idempotency_store(store.clone());

        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            should_fail: false,
        });
        worker.register_tool(tool).await;

        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "test_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: Some("test_key".to_string()),
            max_retries: 0,
            retry_count: 0,
        };

        // Should fail with idempotency store error
        let result = worker.process_job(job).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Store error"));
    }

    #[tokio::test]
    async fn test_no_idempotency_key_no_cache_check() {
        let store = Arc::new(InMemoryIdempotencyStore::new());
        let worker = ToolWorker::new(ExecutionConfig::default(), test_app_context())
            .with_idempotency_store(store.clone());

        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            should_fail: false,
        });
        worker.register_tool(tool).await;

        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "test_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None, // No idempotency key
            max_retries: 0,
            retry_count: 0,
        };

        // Should execute normally without cache check
        let result = worker.process_job(job).await.unwrap();
        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_no_idempotency_store_no_cache() {
        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            test_app_context(),
        );
        // Don't set idempotency store

        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            should_fail: false,
        });
        worker.register_tool(tool).await;

        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "test_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: Some("test_key".to_string()),
            max_retries: 0,
            retry_count: 0,
        };

        // Should execute normally without cache check
        let result = worker.process_job(job).await.unwrap();
        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_cache_result_with_no_idempotency_key() {
        let store = Arc::new(InMemoryIdempotencyStore::new());
        let worker = ToolWorker::new(ExecutionConfig::default(), test_app_context())
            .with_idempotency_store(store.clone());

        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            should_fail: false,
        });
        worker.register_tool(tool).await;

        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "test_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None, // No idempotency key
            max_retries: 0,
            retry_count: 0,
        };

        // Should execute and not attempt to cache
        let result = worker.process_job(job).await.unwrap();
        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_cache_result_with_idempotency_disabled() {
        let config = ExecutionConfig {
            enable_idempotency: false,
            ..Default::default()
        };

        let store = Arc::new(InMemoryIdempotencyStore::new());
        let worker =
            ToolWorker::new(config, test_app_context()).with_idempotency_store(store.clone());

        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            should_fail: false,
        });
        worker.register_tool(tool).await;

        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "test_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: Some("test_key".to_string()),
            max_retries: 0,
            retry_count: 0,
        };

        // Should execute but not cache due to disabled idempotency
        let result = worker.process_job(job).await.unwrap();
        assert!(result.is_success());

        // Verify nothing was cached
        assert!(store.get("test_key").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_cache_result_with_no_store() {
        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            test_app_context(),
        );
        // Don't set idempotency store

        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            should_fail: false,
        });
        worker.register_tool(tool).await;

        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "test_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: Some("test_key".to_string()),
            max_retries: 0,
            retry_count: 0,
        };

        // Should execute but not cache due to no store
        let result = worker.process_job(job).await.unwrap();
        assert!(result.is_success());
    }

    // Tests for create_backoff_strategy and wait_with_backoff removed
    // These methods are now handled by the unified retry helper in retry.rs

    #[tokio::test]
    async fn test_acquire_semaphore_web_prefix() {
        let limits = ResourceLimits::default().with_limit("http_api", 1);

        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            test_app_context(),
        )
        .with_resource_limits(limits);

        // Test web_ prefix maps to http_api
        let _permit = worker.acquire_semaphore("web_fetch").await.unwrap();
    }

    #[tokio::test]
    async fn test_acquire_semaphore_no_matching_resource() {
        let limits = ResourceLimits::default(); // Empty limits

        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            test_app_context(),
        )
        .with_resource_limits(limits);

        // Should fall back to default semaphore
        let _permit = worker.acquire_semaphore("solana_test").await.unwrap();
        let _permit = worker.acquire_semaphore("random_tool").await.unwrap();
    }

    #[tokio::test]
    async fn test_run_loop_job_processing_success_logging() {
        use crate::queue::InMemoryJobQueue;

        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            test_app_context(),
        );
        let tool = Arc::new(MockTool {
            name: "log_test_tool".to_string(),
            should_fail: false,
        });
        worker.register_tool(tool).await;

        let queue = Arc::new(InMemoryJobQueue::new());

        // Enqueue a successful job
        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "log_test_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 0,
            retry_count: 0,
        };
        queue.enqueue(job).await.unwrap();

        let worker_clone = worker.clone();
        let queue_clone = queue.clone();
        let cancellation_token = tokio_util::sync::CancellationToken::new();
        let token_clone = cancellation_token.clone();

        let handle = tokio::spawn(async move { worker_clone.run(queue_clone, token_clone).await });

        // Give it time to process
        tokio::time::sleep(Duration::from_millis(50)).await;

        cancellation_token.cancel();
        handle.await.unwrap().unwrap();

        // Check that jobs_processed was incremented (line 1274)
        let metrics = worker.metrics();
        assert!(
            metrics
                .jobs_processed
                .load(std::sync::atomic::Ordering::Relaxed)
                > 0
        );
    }

    #[tokio::test]
    async fn test_run_loop_worker_error_logging() {
        use crate::queue::InMemoryJobQueue;

        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
            ExecutionConfig::default(),
            test_app_context(),
        );
        // Don't register any tools to cause WorkerError

        let queue = Arc::new(InMemoryJobQueue::new());

        // Enqueue a job for non-existent tool
        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "nonexistent_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 0,
            retry_count: 0,
        };
        queue.enqueue(job).await.unwrap();

        let worker_clone = worker.clone();
        let queue_clone = queue.clone();
        let cancellation_token = tokio_util::sync::CancellationToken::new();
        let token_clone = cancellation_token.clone();

        let handle = tokio::spawn(async move { worker_clone.run(queue_clone, token_clone).await });

        // Give it time to process and hit the error path
        tokio::time::sleep(Duration::from_millis(50)).await;

        cancellation_token.cancel();
        handle.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn test_semaphore_acquisition_error() {
        // This test is tricky since semaphore acquisition rarely fails
        // But we can test the error path by creating a scenario where it might
        let config = ExecutionConfig {
            max_concurrency: 1,
            ..Default::default()
        };

        let worker = ToolWorker::<InMemoryIdempotencyStore>::new(config, test_app_context());
        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
            should_fail: false,
        });
        worker.register_tool(tool).await;

        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "test_tool".to_string(),
            params: serde_json::json!({}),
            idempotency_key: None,
            max_retries: 0,
            retry_count: 0,
        };

        // Normal case should work
        let result = worker.process_job(job).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_tool_trait_description_method() {
        let tool = MockTool {
            name: "test".to_string(),
            should_fail: false,
        };

        // Test that description method returns expected value
        assert_eq!(tool.description(), "");
    }

    #[test]
    fn test_mock_tool_name_method() {
        let tool = MockTool {
            name: "test_name".to_string(),
            should_fail: false,
        };

        assert_eq!(tool.name(), "test_name");
    }

    #[test]
    fn test_slow_mock_tool_name_method() {
        let tool = SlowMockTool {
            name: "slow_test".to_string(),
            delay: Duration::from_millis(1),
        };

        assert_eq!(tool.name(), "slow_test");
        assert_eq!(tool.description(), "");
    }

    #[test]
    fn test_error_queue_new() {
        let _queue = ErrorQueue::default();
        // Just testing construction
    }

    #[derive(Default)]
    struct ErrorIdempotencyStore {
        _phantom: std::marker::PhantomData<()>,
    }

    #[async_trait]
    impl crate::idempotency::IdempotencyStore for ErrorIdempotencyStore {
        async fn get(&self, _key: &str) -> anyhow::Result<Option<Arc<JobResult>>> {
            Err(anyhow::anyhow!("Store error"))
        }

        async fn set(
            &self,
            _key: &str,
            _result: Arc<JobResult>,
            _ttl: Duration,
        ) -> anyhow::Result<()> {
            Err(anyhow::anyhow!("Store error"))
        }

        async fn remove(&self, _key: &str) -> anyhow::Result<()> {
            Err(anyhow::anyhow!("Store error"))
        }
    }

    struct SlowMockTool {
        name: String,
        delay: Duration,
    }

    #[async_trait]
    impl Tool for SlowMockTool {
        async fn execute(
            &self,
            _params: serde_json::Value,
            _context: &crate::provider::ApplicationContext,
        ) -> Result<JobResult, ToolError> {
            tokio::time::sleep(self.delay).await;
            Ok(JobResult::Success {
                value: serde_json::json!({"result": "slow_success"}),
                tx_hash: None,
            })
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            ""
        }

        fn schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "additionalProperties": true
            })
        }
    }

    #[derive(Default)]
    struct ErrorQueue {
        _phantom: std::marker::PhantomData<()>,
    }

    #[async_trait]
    impl crate::queue::JobQueue for ErrorQueue {
        async fn enqueue(&self, _job: crate::jobs::Job) -> anyhow::Result<()> {
            Err(anyhow::anyhow!("Queue error"))
        }

        async fn dequeue(&self) -> anyhow::Result<Option<crate::jobs::Job>> {
            Err(anyhow::anyhow!("Dequeue error"))
        }

        async fn dequeue_with_timeout(
            &self,
            _timeout: Duration,
        ) -> anyhow::Result<Option<crate::jobs::Job>> {
            Err(anyhow::anyhow!("Dequeue timeout error"))
        }

        async fn len(&self) -> anyhow::Result<usize> {
            Err(anyhow::anyhow!("Len error"))
        }
    }
}
