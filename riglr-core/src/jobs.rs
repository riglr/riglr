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
//! use riglr_core::JobResult;
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
//! // Retriable failure (network timeout, rate limit, etc.)
//! let retriable = JobResult::retriable_failure("Network timeout");
//! assert!(retriable.is_retriable());
//! assert!(!retriable.is_success());
//!
//! // Permanent failure (invalid input, insufficient funds, etc.)
//! let permanent = JobResult::permanent_failure("Invalid address format");
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
/// // Retriable failure - network issues
/// let network_error = JobResult::retriable_failure(
///     "Connection timeout after 30 seconds"
/// );
/// assert!(network_error.is_retriable());
/// assert!(!network_error.is_success());
///
/// // Permanent failure - bad input
/// let validation_error = JobResult::permanent_failure(
///     "Invalid email address format"
/// );
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
///         JobResult::Failure { error, retriable } => {
///             if retriable {
///                 println!("Temporary failure (will retry): {}", error);
///             } else {
///                 println!("Permanent failure (won't retry): {}", error);
///             }
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// This variant indicates that the tool execution failed. The failure
    /// is classified as either retriable (temporary issue) or permanent
    /// (won't be resolved by retrying).
    Failure {
        /// Human-readable error message describing what went wrong.
        ///
        /// This should provide enough context for debugging and user feedback.
        /// It should be descriptive but not include sensitive information.
        error: String,

        /// Whether this failure can be resolved by retrying the operation.
        ///
        /// - `true`: Retriable failure - temporary issue that may resolve
        /// - `false`: Permanent failure - retrying won't help
        ///
        /// The worker uses this flag to determine whether to attempt retries.
        retriable: bool,
    },
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
    /// use riglr_core::JobResult;
    ///
    /// // Network connectivity issues
    /// let network_error = JobResult::retriable_failure(
    ///     "Connection timeout after 30 seconds"
    /// );
    ///
    /// // Rate limiting
    /// let rate_limit = JobResult::retriable_failure(
    ///     "API rate limit exceeded, retry in 60 seconds"
    /// );
    ///
    /// // Service unavailability
    /// let service_down = JobResult::retriable_failure(
    ///     "Service temporarily unavailable (HTTP 503)"
    /// );
    ///
    /// assert!(network_error.is_retriable());
    /// assert!(rate_limit.is_retriable());
    /// assert!(service_down.is_retriable());
    /// ```
    ///
    /// # When to Use Retriable Failures
    ///
    /// - Network connectivity problems (timeouts, DNS failures)
    /// - HTTP 5xx server errors (except 501 Not Implemented)
    /// - Rate limiting responses (HTTP 429)
    /// - Temporary resource unavailability
    /// - Blockchain network congestion
    /// - Database connection issues
    pub fn retriable_failure(error: impl Into<String>) -> Self {
        JobResult::Failure {
            error: error.into(),
            retriable: true,
        }
    }

    /// Create a permanent (non-retriable) failure result.
    ///
    /// Use this for failures that won't be resolved by retrying, such as
    /// invalid parameters, authorization failures, or business logic violations.
    /// The worker will not retry jobs that fail with permanent errors.
    ///
    /// # Parameters
    /// * `error` - Human-readable error message describing the failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// use riglr_core::JobResult;
    ///
    /// // Invalid input parameters
    /// let validation_error = JobResult::permanent_failure(
    ///     "Invalid email address format"
    /// );
    ///
    /// // Authorization issues
    /// let auth_error = JobResult::permanent_failure(
    ///     "API key is invalid or expired"
    /// );
    ///
    /// // Business logic violations
    /// let business_error = JobResult::permanent_failure(
    ///     "Insufficient funds for transfer"
    /// );
    ///
    /// // Invalid blockchain addresses
    /// let address_error = JobResult::permanent_failure(
    ///     "Invalid Solana address format"
    /// );
    ///
    /// assert!(!validation_error.is_retriable());
    /// assert!(!auth_error.is_retriable());
    /// assert!(!business_error.is_retriable());
    /// assert!(!address_error.is_retriable());
    /// ```
    ///
    /// # When to Use Permanent Failures
    ///
    /// - Invalid input parameters or malformed requests
    /// - Authentication or authorization failures
    /// - HTTP 4xx client errors (except 429 Rate Limited)
    /// - Insufficient funds or resources
    /// - Invalid addresses or identifiers
    /// - Business rule violations
    /// - Unsupported operations (HTTP 501 Not Implemented)
    pub fn permanent_failure(error: impl Into<String>) -> Self {
        JobResult::Failure {
            error: error.into(),
            retriable: false,
        }
    }

    /// Check if this result represents a successful execution.
    ///
    /// # Returns
    /// * `true` - If this is a `JobResult::Success`
    /// * `false` - If this is a `JobResult::Failure`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use riglr_core::JobResult;
    /// use serde_json::json;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let success = JobResult::success(&json!({"data": "ok"}))?;
    /// let failure = JobResult::permanent_failure("Error occurred");
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
    /// use riglr_core::JobResult;
    /// use serde_json::json;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let success = JobResult::success(&json!({"data": "ok"}))?;
    /// let retriable = JobResult::retriable_failure("Network timeout");
    /// let permanent = JobResult::permanent_failure("Invalid input");
    ///
    /// assert!(!success.is_retriable());     // Success is not retriable
    /// assert!(retriable.is_retriable());    // Retriable failure
    /// assert!(!permanent.is_retriable());   // Permanent failure is not retriable
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            JobResult::Failure {
                retriable: true,
                ..
            }
        )
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
    fn test_job_with_idempotency() {
        let params = serde_json::json!({"key": "value"});
        let job = Job::new_idempotent("test_tool", &params, 3, "test_key").unwrap();

        assert_eq!(job.idempotency_key, Some("test_key".to_string()));
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
    fn test_job_result_creation() {
        let success = JobResult::success(&"test_value").unwrap();
        assert!(success.is_success());

        let success_with_tx = JobResult::success_with_tx(&"test_value", "tx_hash").unwrap();
        assert!(success_with_tx.is_success());

        let retriable_failure = JobResult::retriable_failure("test error");
        assert!(retriable_failure.is_retriable());
        assert!(!retriable_failure.is_success());

        let permanent_failure = JobResult::permanent_failure("test error");
        assert!(!permanent_failure.is_retriable());
        assert!(!permanent_failure.is_success());
    }
}
