//! Job data structures and types for task execution.
//!
//! This module defines the core data structures used for representing work units
//! and their execution results in the riglr system. Jobs are queued for execution
//! by the [`ToolWorker`](crate::ToolWorker) and can include retry logic, idempotency
//! keys, and structured result handling.
//!
//! ## Core Types
//!
//! - **[`Job`]** - A unit of work to be executed, with retry and idempotency support
//! - **[`JobResult`]** - The outcome of job execution with success/failure classification
//!
//! ## Job Lifecycle
//!
//! 1. **Creation** - Jobs are created with tool name, parameters, and retry limits
//! 2. **Queueing** - Jobs are enqueued for processing by workers
//! 3. **Execution** - Workers execute the corresponding tool with job parameters
//! 4. **Result** - Execution produces a [`JobResult`] indicating success or failure
//! 5. **Retry** - Failed jobs may be retried based on failure type and retry limits
//!
//! ## Examples
//!
//! ### Creating and Processing Jobs
//!
//! ```rust
//! use riglr_core::{Job, JobResult};
//! use serde_json::json;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a simple job
//! let job = Job::new(
//!     "weather_check",
//!     &json!({"city": "San Francisco"}),
//!     3 // max retries
//! )?;
//!
//! // Create an idempotent job
//! let idempotent_job = Job::new_idempotent(
//!     "account_balance",
//!     &json!({"address": "0x123..."}),
//!     2, // max retries
//!     "balance_check_user_123" // idempotency key
//! )?;
//!
//! // Check retry status
//! assert!(job.can_retry());
//! assert_eq!(job.retry_count, 0);
//! # Ok(())
//! # }
//! ```
//!
//! ### Working with Job Results
//!
//! ```rust
//! use riglr_core::{JobResult, ToolError};
//! use serde_json::json;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Successful result
//! let success = JobResult::success(&json!({
//!     "temperature": 72,
//!     "condition": "sunny"
//! }))?;
//! assert!(success.is_success());
//!
//! // Successful transaction result
//! let tx_result = JobResult::success_with_tx(
//!     &json!({"amount": 100, "recipient": "0xabc..."}),
//!     "0x789def..."
//! )?;
//!
//! // Retriable failure using new ToolError structure
//! let retriable = JobResult::Failure {
//!     error: ToolError::retriable_string("Network timeout")
//! };
//! assert!(retriable.is_retriable());
//! assert!(!retriable.is_success());
//!
//! // Permanent failure using new ToolError structure
//! let permanent = JobResult::Failure {
//!     error: ToolError::permanent_string("Invalid address format")
//! };
//! assert!(!permanent.is_retriable());
//! assert!(!permanent.is_success());
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling Strategy
//!
//! The system distinguishes between two types of failures:
//!
//! ### Retriable Failures
//! These represent temporary issues that may resolve on retry:
//! - Network connectivity problems
//! - Rate limiting from external APIs
//! - Temporary service unavailability
//! - Resource contention
//!
//! ### Permanent Failures
//! These represent issues that won't be resolved by retrying:
//! - Invalid parameters or malformed requests
//! - Authentication or authorization failures
//! - Insufficient funds or resources
//! - Business logic violations
//!
//! ## Idempotency
//!
//! Jobs can include idempotency keys to ensure safe retry behavior:
//!
//! ```rust
//! use riglr_core::Job;
//! use serde_json::json;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let job = Job::new_idempotent(
//!     "transfer",
//!     &json!({
//!         "from": "user123",
//!         "to": "user456",
//!         "amount": 100
//!     }),
//!     3, // max retries
//!     "transfer_user123_to_user456_100_20241201" // unique key
//! )?;
//!
//! // Subsequent executions with the same key return cached results
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A job represents a unit of work to be executed by a [`ToolWorker`](crate::ToolWorker).
///
/// Jobs encapsulate all the information needed to execute a tool, including
/// the tool name, parameters, retry configuration, and optional idempotency key.
/// They are the primary unit of work in the riglr job processing system.
///
/// ## Job Lifecycle
///
/// 1. **Creation** - Job is created with tool name and parameters
/// 2. **Queuing** - Job is submitted to a job queue for processing
/// 3. **Execution** - Worker picks up job and executes the corresponding tool
/// 4. **Retry** - If execution fails with a retriable error, job may be retried
/// 5. **Completion** - Job succeeds or fails permanently after exhausting retries
///
/// ## Examples
///
/// ### Basic Job Creation
///
/// ```rust
/// use riglr_core::Job;
/// use serde_json::json;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let job = Job::new(
///     "price_checker",
///     &json!({
///         "symbol": "BTC",
///         "currency": "USD"
///     }),
///     3 // max retries
/// )?;
///
/// println!("Job ID: {}", job.job_id);
/// println!("Tool: {}", job.tool_name);
/// println!("Can retry: {}", job.can_retry());
/// # Ok(())
/// # }
/// ```
///
/// ### Idempotent Job Creation
///
/// ```rust
/// use riglr_core::Job;
/// use serde_json::json;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let job = Job::new_idempotent(
///     "bank_transfer",
///     &json!({
///         "from_account": "123",
///         "to_account": "456",
///         "amount": 100.00,
///         "currency": "USD"
///     }),
///     2, // max retries
///     "transfer_123_456_100_20241201" // idempotency key
/// )?;
///
/// assert!(job.idempotency_key.is_some());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique identifier for this job instance.
    ///
    /// Generated automatically when the job is created. This ID is used
    /// for tracking and logging purposes throughout the job's lifecycle.
    pub job_id: Uuid,

    /// Name of the tool to execute.
    ///
    /// This must match the name of a tool registered with the [`ToolWorker`](crate::ToolWorker).
    /// Tool names also influence resource pool assignment (e.g., "solana_*" tools
    /// use the "solana_rpc" resource pool).
    pub tool_name: String,

    /// Parameters to pass to the tool, serialized as JSON.
    ///
    /// These parameters will be passed to the tool's [`execute`](crate::Tool::execute)
    /// method. Tools are responsible for validating and extracting the required
    /// parameters from this JSON value.
    pub params: serde_json::Value,

    /// Optional idempotency key to prevent duplicate execution.
    ///
    /// When an idempotency key is provided and an idempotency store is configured,
    /// the worker will cache successful results. Subsequent jobs with the same
    /// idempotency key will return the cached result instead of re-executing.
    ///
    /// Idempotency keys should be unique per logical operation and include
    /// relevant parameters to ensure uniqueness.
    pub idempotency_key: Option<String>,

    /// Maximum number of retry attempts allowed for this job.
    ///
    /// If a tool execution fails with a retriable error, the job will be
    /// retried up to this many times. The total execution attempts will
    /// be `max_retries + 1`.
    ///
    /// Only retriable failures trigger retries - permanent failures will
    /// not be retried regardless of this setting.
    pub max_retries: u32,

    /// Current retry count for this job.
    ///
    /// This tracks how many retry attempts have been made. It starts at 0
    /// for new jobs and is incremented after each failed execution attempt.
    /// The job stops retrying when `retry_count >= max_retries`.
    #[serde(default)]
    pub retry_count: u32,
}

impl Job {
    /// Create a new job without an idempotency key.
    ///
    /// This creates a standard job that will be executed normally without
    /// result caching. If the job fails with a retriable error, it may be
    /// retried up to `max_retries` times.
    ///
    /// # Parameters
    /// * `tool_name` - Name of the tool to execute (must match a registered tool)
    /// * `params` - Parameters to pass to the tool (will be JSON-serialized)
    /// * `max_retries` - Maximum number of retry attempts (0 = no retries)
    ///
    /// # Returns
    /// * `Ok(Job)` - Successfully created job
    /// * `Err(serde_json::Error)` - If parameters cannot be serialized to JSON
    ///
    /// # Examples
    ///
    /// ```rust
    /// use riglr_core::Job;
    /// use serde_json::json;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Simple job with basic parameters
    /// let job = Job::new(
    ///     "weather_check",
    ///     &json!({
    ///         "city": "San Francisco",
    ///         "units": "metric"
    ///     }),
    ///     3 // allow up to 3 retries
    /// )?;
    ///
    /// assert_eq!(job.tool_name, "weather_check");
    /// assert_eq!(job.max_retries, 3);
    /// assert_eq!(job.retry_count, 0);
    /// assert!(job.idempotency_key.is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new<T: Serialize>(
        tool_name: impl Into<String>,
        params: &T,
        max_retries: u32,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            job_id: Uuid::new_v4(),
            tool_name: tool_name.into(),
            params: serde_json::to_value(params)?,
            idempotency_key: None,
            max_retries,
            retry_count: 0,
        })
    }

    /// Create a new job with an idempotency key for safe retry behavior.
    ///
    /// When an idempotency store is configured, successful results for jobs
    /// with idempotency keys are cached. Subsequent jobs with the same key
    /// will return the cached result instead of re-executing the tool.
    ///
    /// # Parameters
    /// * `tool_name` - Name of the tool to execute (must match a registered tool)
    /// * `params` - Parameters to pass to the tool (will be JSON-serialized)
    /// * `max_retries` - Maximum number of retry attempts (0 = no retries)
    /// * `idempotency_key` - Unique key for this operation (should include relevant parameters)
    ///
    /// # Returns
    /// * `Ok(Job)` - Successfully created job
    /// * `Err(serde_json::Error)` - If parameters cannot be serialized to JSON
    ///
    /// # Examples
    ///
    /// ```rust
    /// use riglr_core::Job;
    /// use serde_json::json;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Idempotent job for a financial transaction
    /// let job = Job::new_idempotent(
    ///     "transfer_funds",
    ///     &json!({
    ///         "from": "account_123",
    ///         "to": "account_456",
    ///         "amount": 100.50,
    ///         "currency": "USD"
    ///     }),
    ///     2, // allow up to 2 retries
    ///     "transfer_123_456_100.50_USD_20241201_001" // unique operation ID
    /// )?;
    ///
    /// assert_eq!(job.tool_name, "transfer_funds");
    /// assert_eq!(job.max_retries, 2);
    /// assert!(job.idempotency_key.is_some());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Idempotency Key Best Practices
    ///
    /// Idempotency keys should:
    /// - Be unique per logical operation
    /// - Include relevant parameters that affect the result
    /// - Include timestamp or sequence numbers for time-sensitive operations
    /// - Be deterministic for the same logical operation
    ///
    /// Example patterns:
    /// - `"transfer_{from}_{to}_{amount}_{currency}_{date}_{sequence}"`
    /// - `"price_check_{symbol}_{currency}_{timestamp}"`
    /// - `"user_action_{user_id}_{action}_{target}_{date}"`
    pub fn new_idempotent<T: Serialize>(
        tool_name: impl Into<String>,
        params: &T,
        max_retries: u32,
        idempotency_key: impl Into<String>,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            job_id: Uuid::new_v4(),
            tool_name: tool_name.into(),
            params: serde_json::to_value(params)?,
            idempotency_key: Some(idempotency_key.into()),
            max_retries,
            retry_count: 0,
        })
    }

    /// Check if this job has retries remaining.
    ///
    /// Returns `true` if the job can be retried after a failure,
    /// `false` if all retry attempts have been exhausted.
    ///
    /// # Returns
    /// * `true` - Job can be retried (`retry_count < max_retries`)
    /// * `false` - No retries remaining (`retry_count >= max_retries`)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use riglr_core::Job;
    /// use serde_json::json;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut job = Job::new("test_tool", &json!({}), 2)?;
    ///
    /// assert!(job.can_retry());  // 0 < 2, can retry
    /// job.increment_retry();
    /// assert!(job.can_retry());  // 1 < 2, can retry
    /// job.increment_retry();
    /// assert!(!job.can_retry()); // 2 >= 2, cannot retry
    /// # Ok(())
    /// # }
    /// ```
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    /// Increment the retry count for this job.
    ///
    /// This should be called by the job processing system after each
    /// failed execution attempt. The retry count is used to determine
    /// whether the job can be retried again.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use riglr_core::Job;
    /// use serde_json::json;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut job = Job::new("test_tool", &json!({}), 3)?;
    ///
    /// assert_eq!(job.retry_count, 0);
    /// job.increment_retry();
    /// assert_eq!(job.retry_count, 1);
    /// job.increment_retry();
    /// assert_eq!(job.retry_count, 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}

/// Result of executing a job, indicating success or failure with classification.
///
/// The `JobResult` enum provides structured representation of job execution outcomes,
/// enabling the system to make intelligent decisions about error handling and retry logic.
///
/// ## Success vs Failure
///
/// - **Success**: The tool executed successfully and produced a result
/// - **Failure**: The tool execution failed, with classification for retry behavior
///
/// ## Failure Classification
///
/// Failed executions are classified as either retriable or permanent:
///
/// - **Retriable failures**: Temporary issues that may resolve on retry
///   - Network timeouts, connection errors
///   - Rate limiting from external APIs
///   - Temporary service unavailability
///   - Resource contention
///
/// - **Permanent failures**: Issues that won't be resolved by retrying
///   - Invalid parameters or malformed requests
///   - Authentication/authorization failures
///   - Insufficient funds or resources
///   - Business logic violations
///
/// ## Examples
///
/// ### Creating Success Results
///
/// ```rust
/// use riglr_core::JobResult;
/// use serde_json::json;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Simple success result
/// let result = JobResult::success(&json!({
///     "temperature": 72,
///     "humidity": 65,
///     "condition": "sunny"
/// }))?;
/// assert!(result.is_success());
///
/// // Success result with transaction hash
/// let tx_result = JobResult::success_with_tx(
///     &json!({
///         "recipient": "0x123...",
///         "amount": "1.5",
///         "status": "confirmed"
///     }),
///     "0xabc123def456..."
/// )?;
/// assert!(tx_result.is_success());
/// # Ok(())
/// # }
/// ```
///
/// ### Creating Failure Results
///
/// ```rust
/// use riglr_core::JobResult;
///
/// // Retriable failure using new ToolError structure
/// let network_error = JobResult::Failure {
///     error: ToolError::retriable_string("Connection timeout after 30 seconds")
/// };
/// assert!(network_error.is_retriable());
/// assert!(!network_error.is_success());
///
/// // Permanent failure using new ToolError structure
/// let validation_error = JobResult::Failure {
///     error: ToolError::permanent_string("Invalid email address format")
/// };
/// assert!(!validation_error.is_retriable());
/// assert!(!validation_error.is_success());
/// ```
///
/// ### Pattern Matching
///
/// ```rust
/// use riglr_core::JobResult;
///
/// fn handle_result(result: JobResult) {
///     match result {
///         JobResult::Success { value, tx_hash } => {
///             println!("Success: {:?}", value);
///             if let Some(hash) = tx_hash {
///                 println!("Transaction: {}", hash);
///             }
///         }
///         JobResult::Failure { error } => {
///             if error.is_retriable() {
///                 println!("Temporary failure (will retry): {}", error);
///                 if let Some(delay) = error.retry_after() {
///                     println!("Retry after: {:?}", delay);
///                 }
///             } else {
///                 println!("Permanent failure (won't retry): {}", error);
///             }
///         }
///     }
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub enum JobResult {
    /// Job executed successfully and produced a result.
    ///
    /// This variant indicates that the tool completed its work successfully
    /// and returned data. The result may optionally include a transaction
    /// hash for blockchain operations.
    Success {
        /// The result data produced by the tool execution.
        ///
        /// This contains the actual output of the tool, serialized as JSON.
        /// The structure of this value depends on the specific tool that
        /// was executed.
        value: serde_json::Value,

        /// Optional transaction hash for blockchain operations.
        ///
        /// When the tool performs a blockchain transaction (transfer, swap, etc.),
        /// this field should contain the transaction hash for tracking and
        /// verification purposes. For non-blockchain operations, this is typically `None`.
        tx_hash: Option<String>,
    },

    /// Job execution failed with error details and retry classification.
    ///
    /// This variant indicates that the tool execution failed. The ToolError
    /// contains the error details and determines whether it's retriable.
    Failure {
        /// The structured error from tool execution.
        ///
        /// ToolError provides rich error classification including retriability,
        /// rate limiting, and error context.
        error: crate::error::ToolError,
    },
}

impl Clone for JobResult {
    fn clone(&self) -> Self {
        match self {
            Self::Success { value, tx_hash } => Self::Success {
                value: value.clone(),
                tx_hash: tx_hash.clone(),
            },
            Self::Failure { error } => {
                // Clone the ToolError by reconstructing it without the source
                let cloned_error = match error {
                    crate::error::ToolError::Retriable {
                        source_message,
                        context,
                        ..
                    } => {
                        crate::error::ToolError::Retriable {
                            source: None, // Drop the source for cloning
                            source_message: source_message.clone(),
                            context: context.clone(),
                        }
                    }
                    crate::error::ToolError::RateLimited {
                        source_message,
                        context,
                        retry_after,
                        ..
                    } => {
                        crate::error::ToolError::RateLimited {
                            source: None, // Drop the source for cloning
                            source_message: source_message.clone(),
                            context: context.clone(),
                            retry_after: *retry_after,
                        }
                    }
                    crate::error::ToolError::Permanent {
                        source_message,
                        context,
                        ..
                    } => {
                        crate::error::ToolError::Permanent {
                            source: None, // Drop the source for cloning
                            source_message: source_message.clone(),
                            context: context.clone(),
                        }
                    }
                    crate::error::ToolError::InvalidInput {
                        source_message,
                        context,
                        ..
                    } => {
                        crate::error::ToolError::InvalidInput {
                            source: None, // Drop the source for cloning
                            source_message: source_message.clone(),
                            context: context.clone(),
                        }
                    }
                    crate::error::ToolError::SignerContext(msg) => {
                        crate::error::ToolError::SignerContext(msg.clone())
                    }
                };
                Self::Failure {
                    error: cloned_error,
                }
            }
        }
    }
}

impl JobResult {
    /// Create a successful result without a transaction hash.
    ///
    /// This is the standard way to create success results for operations
    /// that don't involve blockchain transactions.
    ///
    /// # Parameters
    /// * `value` - The result data to serialize as JSON
    ///
    /// # Returns
    /// * `Ok(JobResult::Success)` - Successfully created result
    /// * `Err(serde_json::Error)` - If value cannot be serialized to JSON
    ///
    /// # Examples
    ///
    /// ```rust
    /// use riglr_core::JobResult;
    /// use serde_json::json;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // API response result
    /// let api_result = JobResult::success(&json!({
    ///     "data": {
    ///         "price": 45000.50,
    ///         "currency": "USD",
    ///         "timestamp": 1638360000
    ///     },
    ///     "status": "success"
    /// }))?;
    ///
    /// assert!(api_result.is_success());
    /// # Ok(())
    /// # }
    /// ```
    pub fn success<T: Serialize>(value: &T) -> Result<Self, serde_json::Error> {
        Ok(JobResult::Success {
            value: serde_json::to_value(value)?,
            tx_hash: None,
        })
    }

    /// Create a successful result with a blockchain transaction hash.
    ///
    /// Use this for operations that involve blockchain transactions,
    /// such as transfers, swaps, or contract interactions. The transaction
    /// hash enables tracking and verification of the operation.
    ///
    /// # Parameters
    /// * `value` - The result data to serialize as JSON
    /// * `tx_hash` - The blockchain transaction hash
    ///
    /// # Returns
    /// * `Ok(JobResult::Success)` - Successfully created result
    /// * `Err(serde_json::Error)` - If value cannot be serialized to JSON
    ///
    /// # Examples
    ///
    /// ```rust
    /// use riglr_core::JobResult;
    /// use serde_json::json;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Solana transfer result
    /// let transfer_result = JobResult::success_with_tx(
    ///     &json!({
    ///         "from": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
    ///         "to": "7XjBz3VSM8Y6kXoHdXFRvLV8Fn6dQJgj9AJhCFJZ9L2x",
    ///         "amount": 1.5,
    ///         "currency": "SOL",
    ///         "status": "confirmed"
    ///     }),
    ///     "2Z8h8K6wF5L9X3mE4u1nV7yQ9H5pG3rD1M2cB6x8Wq7N"
    /// )?;
    ///
    /// // Ethereum swap result
    /// let swap_result = JobResult::success_with_tx(
    ///     &json!({
    ///         "input_token": "USDC",
    ///         "output_token": "WETH",
    ///         "input_amount": "1000.00",
    ///         "output_amount": "0.42",
    ///         "slippage": "0.5%"
    ///     }),
    ///     "0x1234567890abcdef1234567890abcdef12345678"
    /// )?;
    ///
    /// assert!(transfer_result.is_success());
    /// assert!(swap_result.is_success());
    /// # Ok(())
    /// # }
    /// ```
    pub fn success_with_tx<T: Serialize>(
        value: &T,
        tx_hash: impl Into<String>,
    ) -> Result<Self, serde_json::Error> {
        Ok(JobResult::Success {
            value: serde_json::to_value(value)?,
            tx_hash: Some(tx_hash.into()),
        })
    }

    /// Create a retriable failure result.
    ///
    /// Use this for temporary failures that may resolve on retry, such as
    /// network timeouts, rate limits, or temporary service unavailability.
    /// The worker will attempt to retry jobs that fail with retriable errors.
    ///
    /// # Parameters
    /// * `error` - Human-readable error message describing the failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// use riglr_core::{JobResult, ToolError};
    ///
    /// // Network connectivity issues
    /// let network_error = JobResult::Failure {
    ///     error: ToolError::retriable_string("Connection timeout after 30 seconds")
    /// };
    ///
    /// // Rate limiting with retry delay
    /// let rate_limit = JobResult::Failure {
    ///     error: ToolError::rate_limited_string("API rate limit exceeded")
    /// };
    ///
    /// // Service unavailability
    /// let service_down = JobResult::Failure {
    ///     error: ToolError::retriable_string("Service temporarily unavailable (HTTP 503)")
    /// };
    ///
    /// assert!(network_error.is_retriable());
    /// assert!(rate_limit.is_retriable());
    /// assert!(service_down.is_retriable());
    /// ```
    /// Check if this result represents a successful execution.
    ///
    /// # Returns
    /// * `true` - If this is a `JobResult::Success`
    /// * `false` - If this is a `JobResult::Failure`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use riglr_core::{JobResult, ToolError};
    /// use serde_json::json;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let success = JobResult::success(&json!({"data": "ok"}))?;
    /// let failure = JobResult::Failure {
    ///     error: ToolError::permanent_string("Error occurred")
    /// };
    ///
    /// assert!(success.is_success());
    /// assert!(!failure.is_success());
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_success(&self) -> bool {
        matches!(self, JobResult::Success { .. })
    }

    /// Check if this result represents a retriable failure.
    ///
    /// This method returns `true` only for `JobResult::Failure` variants
    /// where `retriable` is `true`. Success results always return `false`.
    ///
    /// # Returns
    /// * `true` - If this is a retriable failure
    /// * `false` - If this is a success or permanent failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// use riglr_core::{JobResult, ToolError};
    /// use serde_json::json;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let success = JobResult::success(&json!({"data": "ok"}))?;
    /// let retriable = JobResult::Failure {
    ///     error: ToolError::retriable_string("Network timeout")
    /// };
    /// let permanent = JobResult::Failure {
    ///     error: ToolError::permanent_string("Invalid input")
    /// };
    ///
    /// assert!(!success.is_retriable());     // Success is not retriable
    /// assert!(retriable.is_retriable());    // Retriable failure
    /// assert!(!permanent.is_retriable());   // Permanent failure is not retriable
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_retriable(&self) -> bool {
        match self {
            JobResult::Failure { error } => error.is_retriable(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation() {
        let params = serde_json::json!({"key": "value"});
        let job = Job::new("test_tool", &params, 3).unwrap();

        assert_eq!(job.tool_name, "test_tool");
        assert_eq!(job.params, params);
        assert_eq!(job.max_retries, 3);
        assert_eq!(job.retry_count, 0);
        assert!(job.idempotency_key.is_none());
        assert!(job.can_retry());
    }

    #[test]
    fn test_job_new_when_valid_params_should_succeed() {
        // Test with various parameter types
        let string_params = "test string";
        let job = Job::new("string_tool", &string_params, 5).unwrap();
        assert_eq!(job.tool_name, "string_tool");
        assert_eq!(job.max_retries, 5);
        assert_eq!(job.retry_count, 0);
        assert!(job.idempotency_key.is_none());
        assert!(job.job_id != Uuid::nil());

        // Test with complex JSON object
        let complex_params = serde_json::json!({
            "nested": {
                "array": [1, 2, 3],
                "string": "test",
                "bool": true,
                "null": null
            }
        });
        let job = Job::new("complex_tool", &complex_params, 0).unwrap();
        assert_eq!(job.params, complex_params);
        assert_eq!(job.max_retries, 0);
    }

    #[test]
    fn test_job_new_when_zero_retries_should_not_allow_retry() {
        let job = Job::new("test_tool", &serde_json::json!({}), 0).unwrap();
        assert_eq!(job.max_retries, 0);
        assert!(!job.can_retry()); // 0 < 0 is false
    }

    #[test]
    fn test_job_new_when_tool_name_is_string_slice_should_convert() {
        let job = Job::new("slice_tool", &serde_json::json!({}), 1).unwrap();
        assert_eq!(job.tool_name, "slice_tool");
    }

    #[test]
    fn test_job_new_when_tool_name_is_string_should_convert() {
        let tool_name = String::from("owned_tool");
        let job = Job::new(tool_name, &serde_json::json!({}), 1).unwrap();
        assert_eq!(job.tool_name, "owned_tool");
    }

    #[test]
    fn test_job_with_idempotency() {
        let params = serde_json::json!({"key": "value"});
        let job = Job::new_idempotent("test_tool", &params, 3, "test_key").unwrap();

        assert_eq!(job.idempotency_key, Some("test_key".to_string()));
    }

    #[test]
    fn test_job_new_idempotent_when_valid_params_should_succeed() {
        let params = serde_json::json!({"transfer": "data"});
        let job = Job::new_idempotent("idempotent_tool", &params, 2, "unique_key_123").unwrap();

        assert_eq!(job.tool_name, "idempotent_tool");
        assert_eq!(job.params, params);
        assert_eq!(job.max_retries, 2);
        assert_eq!(job.retry_count, 0);
        assert_eq!(job.idempotency_key, Some("unique_key_123".to_string()));
        assert!(job.job_id != Uuid::nil());
        assert!(job.can_retry());
    }

    #[test]
    fn test_job_new_idempotent_when_key_is_string_slice_should_convert() {
        let job = Job::new_idempotent("test_tool", &serde_json::json!({}), 1, "slice_key").unwrap();
        assert_eq!(job.idempotency_key, Some("slice_key".to_string()));
    }

    #[test]
    fn test_job_new_idempotent_when_key_is_string_should_convert() {
        let key = String::from("owned_key");
        let job = Job::new_idempotent("test_tool", &serde_json::json!({}), 1, key).unwrap();
        assert_eq!(job.idempotency_key, Some("owned_key".to_string()));
    }

    #[test]
    fn test_job_retry_logic() {
        let params = serde_json::json!({"key": "value"});
        let mut job = Job::new("test_tool", &params, 2).unwrap();

        assert!(job.can_retry());
        job.increment_retry();
        assert!(job.can_retry());
        job.increment_retry();
        assert!(!job.can_retry());
    }

    #[test]
    fn test_job_can_retry_when_retry_count_equals_max_retries_should_return_false() {
        let mut job = Job::new("test_tool", &serde_json::json!({}), 1).unwrap();
        job.retry_count = 1; // Set equal to max_retries
        assert!(!job.can_retry());
    }

    #[test]
    fn test_job_can_retry_when_retry_count_exceeds_max_retries_should_return_false() {
        let mut job = Job::new("test_tool", &serde_json::json!({}), 1).unwrap();
        job.retry_count = 2; // Set greater than max_retries
        assert!(!job.can_retry());
    }

    #[test]
    fn test_job_increment_retry_when_called_multiple_times_should_increment() {
        let mut job = Job::new("test_tool", &serde_json::json!({}), 5).unwrap();

        assert_eq!(job.retry_count, 0);
        job.increment_retry();
        assert_eq!(job.retry_count, 1);
        job.increment_retry();
        assert_eq!(job.retry_count, 2);
        job.increment_retry();
        assert_eq!(job.retry_count, 3);
    }

    #[test]
    fn test_job_increment_retry_when_already_at_max_should_still_increment() {
        let mut job = Job::new("test_tool", &serde_json::json!({}), 1).unwrap();
        job.retry_count = 1;

        job.increment_retry();
        assert_eq!(job.retry_count, 2); // Should still increment even if past max
    }

    #[test]
    fn test_job_result_creation() {
        let success = JobResult::success(&"test_value").unwrap();
        assert!(success.is_success());

        let success_with_tx = JobResult::success_with_tx(&"test_value", "tx_hash").unwrap();
        assert!(success_with_tx.is_success());

        let retriable_failure = JobResult::Failure {
            error: crate::error::ToolError::retriable_string("test error"),
        };
        assert!(retriable_failure.is_retriable());
        assert!(!retriable_failure.is_success());

        let permanent_failure = JobResult::Failure {
            error: crate::error::ToolError::permanent_string("test error"),
        };
        assert!(!permanent_failure.is_retriable());
        assert!(!permanent_failure.is_success());
    }

    #[test]
    fn test_job_result_success_when_valid_value_should_create_success() {
        // Test with simple value
        let result = JobResult::success(&42).unwrap();
        match result {
            JobResult::Success {
                ref value,
                ref tx_hash,
            } => {
                assert_eq!(*value, serde_json::json!(42));
                assert!(tx_hash.is_none());
            }
            _ => panic!("Expected Success variant"),
        }
        assert!(result.is_success());
        assert!(!result.is_retriable());

        // Test with complex object
        let complex_data = serde_json::json!({
            "status": "completed",
            "data": {
                "items": [1, 2, 3],
                "metadata": {
                    "count": 3,
                    "timestamp": "2024-01-01"
                }
            }
        });
        let result = JobResult::success(&complex_data).unwrap();
        match result {
            JobResult::Success { value, tx_hash } => {
                assert_eq!(value, complex_data);
                assert!(tx_hash.is_none());
            }
            _ => panic!("Expected Success variant"),
        }
    }

    #[test]
    fn test_job_result_success_with_tx_when_valid_params_should_create_success() {
        let data = serde_json::json!({"amount": 100, "recipient": "0xabc"});
        let tx_hash = "0x123456789abcdef";

        let result = JobResult::success_with_tx(&data, tx_hash).unwrap();

        match result {
            JobResult::Success {
                ref value,
                tx_hash: ref hash,
            } => {
                assert_eq!(*value, data);
                assert_eq!(*hash, Some("0x123456789abcdef".to_string()));
            }
            _ => panic!("Expected Success variant"),
        }
        assert!(result.is_success());
        assert!(!result.is_retriable());
    }

    #[test]
    fn test_job_result_success_with_tx_when_tx_hash_is_string_slice_should_convert() {
        let result = JobResult::success_with_tx(&"test", "slice_hash").unwrap();
        match result {
            JobResult::Success { tx_hash, .. } => {
                assert_eq!(tx_hash, Some("slice_hash".to_string()));
            }
            _ => panic!("Expected Success variant"),
        }
    }

    #[test]
    fn test_job_result_success_with_tx_when_tx_hash_is_string_should_convert() {
        let hash = String::from("owned_hash");
        let result = JobResult::success_with_tx(&"test", hash).unwrap();
        match result {
            JobResult::Success { tx_hash, .. } => {
                assert_eq!(tx_hash, Some("owned_hash".to_string()));
            }
            _ => panic!("Expected Success variant"),
        }
    }

    #[test]
    fn test_job_result_retriable_failure_when_error_message_should_create_retriable() {
        let error_msg = "Connection timeout occurred";
        let result = JobResult::Failure {
            error: crate::error::ToolError::retriable_string(error_msg),
        };

        match result {
            JobResult::Failure { ref error } => {
                assert!(error.contains("Connection timeout occurred"));
                assert!(result.is_retriable());
            }
            _ => panic!("Expected Failure variant"),
        }
        assert!(!result.is_success());
        assert!(result.is_retriable());
    }

    #[test]
    fn test_job_result_retriable_failure_when_error_is_string_slice_should_convert() {
        let result = JobResult::Failure {
            error: crate::error::ToolError::retriable_string("slice error"),
        };
        match result {
            JobResult::Failure { error } => {
                assert_eq!(
                    error.to_string(),
                    "Operation can be retried: slice error - slice error"
                );
            }
            _ => panic!("Expected Failure variant"),
        }
    }

    #[test]
    fn test_job_result_retriable_failure_when_error_is_string_should_convert() {
        let error = String::from("owned error");
        let result = JobResult::Failure {
            error: crate::error::ToolError::retriable_string(error),
        };
        match result {
            JobResult::Failure { error } => {
                assert_eq!(
                    error.to_string(),
                    "Operation can be retried: owned error - owned error"
                );
            }
            _ => panic!("Expected Failure variant"),
        }
    }

    #[test]
    fn test_job_result_permanent_failure_when_error_message_should_create_permanent() {
        let error_msg = "Invalid input parameters";
        let result = JobResult::Failure {
            error: crate::error::ToolError::permanent_string(error_msg),
        };

        match result {
            JobResult::Failure { ref error } => {
                assert!(error.contains("Invalid input parameters"));
                assert!(!result.is_retriable());
            }
            _ => panic!("Expected Failure variant"),
        }
        assert!(!result.is_success());
        assert!(!result.is_retriable());
    }

    #[test]
    fn test_job_result_permanent_failure_when_error_is_string_slice_should_convert() {
        let result = JobResult::Failure {
            error: crate::error::ToolError::permanent_string("slice error"),
        };
        match result {
            JobResult::Failure { error } => {
                assert_eq!(
                    error.to_string(),
                    "Permanent error: slice error - slice error"
                );
            }
            _ => panic!("Expected Failure variant"),
        }
    }

    #[test]
    fn test_job_result_permanent_failure_when_error_is_string_should_convert() {
        let error = String::from("owned error");
        let result = JobResult::Failure {
            error: crate::error::ToolError::permanent_string(error),
        };
        match result {
            JobResult::Failure { error } => {
                assert_eq!(
                    error.to_string(),
                    "Permanent error: owned error - owned error"
                );
            }
            _ => panic!("Expected Failure variant"),
        }
    }

    #[test]
    fn test_job_result_is_success_when_success_variant_should_return_true() {
        let success = JobResult::success(&"test").unwrap();
        assert!(success.is_success());

        let success_with_tx = JobResult::success_with_tx(&"test", "hash").unwrap();
        assert!(success_with_tx.is_success());
    }

    #[test]
    fn test_job_result_is_success_when_failure_variant_should_return_false() {
        let retriable_failure = JobResult::Failure {
            error: crate::error::ToolError::retriable_string("error"),
        };
        assert!(!retriable_failure.is_success());

        let permanent_failure = JobResult::Failure {
            error: crate::error::ToolError::permanent_string("error"),
        };
        assert!(!permanent_failure.is_success());
    }

    #[test]
    fn test_job_result_is_retriable_when_retriable_failure_should_return_true() {
        let result = JobResult::Failure {
            error: crate::error::ToolError::retriable_string("Network error"),
        };
        assert!(result.is_retriable());
    }

    #[test]
    fn test_job_result_is_retriable_when_permanent_failure_should_return_false() {
        let result = JobResult::Failure {
            error: crate::error::ToolError::permanent_string("Invalid input"),
        };
        assert!(!result.is_retriable());
    }

    #[test]
    fn test_job_result_is_retriable_when_success_should_return_false() {
        let result = JobResult::success(&"test").unwrap();
        assert!(!result.is_retriable());

        let result_with_tx = JobResult::success_with_tx(&"test", "hash").unwrap();
        assert!(!result_with_tx.is_retriable());
    }

    #[test]
    fn test_job_clone_should_create_identical_copy() {
        let original = Job::new_idempotent(
            "clone_tool",
            &serde_json::json!({"test": true}),
            3,
            "clone_key",
        )
        .unwrap();
        let cloned = original.clone();

        assert_eq!(original.job_id, cloned.job_id);
        assert_eq!(original.tool_name, cloned.tool_name);
        assert_eq!(original.params, cloned.params);
        assert_eq!(original.idempotency_key, cloned.idempotency_key);
        assert_eq!(original.max_retries, cloned.max_retries);
        assert_eq!(original.retry_count, cloned.retry_count);
    }

    #[test]
    fn test_job_result_clone_should_create_identical_copy() {
        let original_success =
            JobResult::success_with_tx(&serde_json::json!({"data": "test"}), "tx123").unwrap();
        let cloned_success = original_success.clone();

        match (&original_success, &cloned_success) {
            (
                JobResult::Success {
                    value: v1,
                    tx_hash: h1,
                },
                JobResult::Success {
                    value: v2,
                    tx_hash: h2,
                },
            ) => {
                assert_eq!(v1, v2);
                assert_eq!(h1, h2);
            }
            _ => panic!("Expected Success variants"),
        }

        let original_failure = JobResult::Failure {
            error: crate::error::ToolError::retriable_string("test error"),
        };
        let cloned_failure = original_failure.clone();

        match (&original_failure, &cloned_failure) {
            (JobResult::Failure { error: e1 }, JobResult::Failure { error: e2 }) => {
                assert_eq!(e1, e2);
            }
            _ => panic!("Expected Failure variants"),
        }
    }

    #[test]
    fn test_job_serialization_and_deserialization_should_preserve_data() {
        let original = Job::new_idempotent(
            "serialize_tool",
            &serde_json::json!({"key": "value"}),
            5,
            "serialize_key",
        )
        .unwrap();

        // Serialize to JSON
        let serialized = serde_json::to_string(&original).unwrap();

        // Deserialize back
        let deserialized: Job = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.job_id, deserialized.job_id);
        assert_eq!(original.tool_name, deserialized.tool_name);
        assert_eq!(original.params, deserialized.params);
        assert_eq!(original.idempotency_key, deserialized.idempotency_key);
        assert_eq!(original.max_retries, deserialized.max_retries);
        assert_eq!(original.retry_count, deserialized.retry_count);
    }

    #[test]
    fn test_job_result_serialization_and_deserialization_should_preserve_data() {
        // Test Success variant
        let original_success =
            JobResult::success_with_tx(&serde_json::json!({"amount": 100}), "hash123").unwrap();
        let serialized = serde_json::to_string(&original_success).unwrap();
        let deserialized: JobResult = serde_json::from_str(&serialized).unwrap();

        match (&original_success, &deserialized) {
            (
                JobResult::Success {
                    value: v1,
                    tx_hash: h1,
                },
                JobResult::Success {
                    value: v2,
                    tx_hash: h2,
                },
            ) => {
                assert_eq!(v1, v2);
                assert_eq!(h1, h2);
            }
            _ => panic!("Expected Success variants"),
        }

        // Test Failure variant
        let original_failure = JobResult::Failure {
            error: crate::error::ToolError::retriable_string("Network timeout"),
        };
        let serialized = serde_json::to_string(&original_failure).unwrap();
        let deserialized: JobResult = serde_json::from_str(&serialized).unwrap();

        match (&original_failure, &deserialized) {
            (JobResult::Failure { error: e1 }, JobResult::Failure { error: e2 }) => {
                assert_eq!(e1, e2);
            }
            _ => panic!("Expected Failure variants"),
        }
    }

    #[test]
    fn test_job_default_retry_count_when_deserializing_without_field_should_be_zero() {
        // Create JSON without retry_count field to test #[serde(default)]
        let json = r#"{
            "job_id": "550e8400-e29b-41d4-a716-446655440000",
            "tool_name": "test_tool",
            "params": {"key": "value"},
            "idempotency_key": null,
            "max_retries": 3
        }"#;

        let job: Job = serde_json::from_str(json).unwrap();
        assert_eq!(job.retry_count, 0); // Should default to 0
    }

    #[test]
    fn test_job_debug_format_should_include_all_fields() {
        let job = Job::new_idempotent(
            "debug_tool",
            &serde_json::json!({"test": "data"}),
            2,
            "debug_key",
        )
        .unwrap();
        let debug_output = format!("{:?}", job);

        // Should contain all major field information
        assert!(debug_output.contains("job_id"));
        assert!(debug_output.contains("debug_tool"));
        assert!(debug_output.contains("test"));
        assert!(debug_output.contains("debug_key"));
        assert!(debug_output.contains("2")); // max_retries
    }

    #[test]
    fn test_job_result_debug_format_should_include_variant_info() {
        let success =
            JobResult::success_with_tx(&serde_json::json!({"data": "test"}), "tx456").unwrap();
        let success_debug = format!("{:?}", success);
        assert!(success_debug.contains("Success"));
        assert!(success_debug.contains("tx456"));

        let failure = JobResult::Failure {
            error: crate::error::ToolError::permanent_string("Debug error message"),
        };
        let failure_debug = format!("{:?}", failure);
        assert!(failure_debug.contains("Failure"));
        assert!(failure_debug.contains("Debug error message"));
        assert!(failure_debug.contains("Permanent")); // Permanent error variant
    }
}
