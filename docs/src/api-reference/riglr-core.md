# riglr-core

{{#include ../../../riglr-core/README.md}}

## API Reference

## Key Components

> The most important types and functions in this crate.

### `ApplicationContext`

Chain-agnostic context for dependency injection and resource management.

[→ Full documentation](#structs)

### `SignerContext`

The SignerContext provides thread-local signer management for secure multi-tenant operation.

[→ Full documentation](#structs)

### `ToolWorker`

A worker that processes jobs from a queue using registered tools.

[→ Full documentation](#structs)

### `Tool`

A trait defining the execution interface for tools.

[→ Full documentation](#traits)

### `Job`

A job represents a unit of work to be executed by a [`ToolWorker`](crate::ToolWorker).

[→ Full documentation](#structs)

### `JobResult`

Result of executing a job, indicating success or failure with classification.

[→ Full documentation](#enums)

### `ToolError`

Tool-specific error type for distinguishing retriable vs permanent failures.

[→ Full documentation](#enums)

---

### Contents

- [Structs](#structs)
- [Enums](#enums)
- [Traits](#traits)
- [Functions](#functions)
- [Type Aliases](#type-aliases)

### Structs

> Core data structures and types.

#### `ClientRateInfo`

Information about a client's rate limit status

---

#### `EvmSignerHandle`

A handle to the current EVM signer, providing type-safe access.

This handle guarantees that the underlying signer supports EVM operations.
It implements `Deref` to `dyn EvmSigner`, allowing direct access to all
EVM-specific methods.

# Thread Safety

The handle is `Send + Sync` and can be passed between threads safely.
The underlying signer is reference-counted and immutable.

# Examples

```ignore
use riglr_core::signer::SignerContext;

async fn evm_transfer() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get a type-safe handle to the EVM signer
    let signer = SignerContext::current_as_evm().await?;
     
    // Access EVM-specific methods directly
    let address = signer.address();
    let chain_id = signer.chain_id();
     
    // Sign and send a transaction
    let tx = TransactionRequest::default();
    let tx_hash = signer.sign_and_send_transaction(tx).await?;
     
    Ok(())
}
```

---

#### `ExecutionConfig`

Configuration for tool execution behavior.

This struct controls all aspects of how tools are executed by the [`ToolWorker`],
including retry behavior, timeouts, concurrency limits, and idempotency settings.

# Examples

```ignore
use riglr_core::ExecutionConfig;
use std::time::Duration;

// Default configuration
let config = ExecutionConfig::default();

// Custom configuration for high-throughput scenario
let high_throughput_config = ExecutionConfig {
    max_concurrency: 50,
    default_timeout: Duration::from_secs(120),
    max_retries: 5,
    initial_retry_delay: Duration::from_millis(200),
    max_retry_delay: Duration::from_secs(60),
    enable_idempotency: true,
    ..Default::default()
};

// Conservative configuration for sensitive operations
let conservative_config = ExecutionConfig {
    max_concurrency: 5,
    default_timeout: Duration::from_secs(300),
    max_retries: 1,
    initial_retry_delay: Duration::from_secs(5),
    max_retry_delay: Duration::from_secs(30),
    enable_idempotency: true,
    ..Default::default()
};
```

---

#### `FixedWindowClientInfo`

Information about a client's fixed window rate limit state

---

#### `FixedWindowStrategy`

Fixed window rate limiting strategy

This strategy divides time into fixed windows and allows a fixed number
of requests per window. When a window expires, the count resets.

---

#### `InMemoryIdempotencyStore`

In-memory idempotency store for testing and development

---

#### `InMemoryJobQueue`

In-memory job queue implementation for testing and development

---

#### `RateLimiter`

A configurable rate limiter for controlling request rates.

This rate limiter supports multiple strategies:
- Token bucket: Continuous token replenishment with burst capacity
- Fixed window: Fixed request count per time window
- Custom strategies via the RateLimitStrategy trait

# Example

```rust
use riglr_core::util::{RateLimiter, RateLimitStrategyType};
use std::time::Duration;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Default token bucket strategy
let rate_limiter = RateLimiter::new(10, Duration::from_secs(60));

// Check rate limit for a user
rate_limiter.check_rate_limit("user123")?;

// Use fixed window strategy
let fixed_limiter = RateLimiter::builder()
    .strategy(RateLimitStrategyType::FixedWindow)
    .max_requests(100)
    .time_window(Duration::from_secs(60))
    .build();
# Ok(())
# }
```

---

#### `RateLimiterBuilder`

Builder for creating customized RateLimiter instances

---

#### `RedisIdempotencyStore`

Redis-based idempotency store for production use

---

#### `RedisJobQueue`

Redis-based job queue implementation for production use

---

#### `ResourceLimits`

Resource limits configuration for fine-grained concurrency control.

This struct allows you to set different concurrency limits for different
types of operations, preventing any single resource type from overwhelming
external services or consuming too many system resources.

Resource limits are applied based on tool name prefixes:
- Tools starting with `solana_` use the "solana_rpc" resource pool
- Tools starting with `evm_` use the "evm_rpc" resource pool
- Tools starting with `web_` use the "http_api" resource pool
- All other tools use the default concurrency limit

# Examples

```rust
use riglr_core::ResourceLimits;

// Create limits for different resource types
let limits = ResourceLimits::default()
    .with_limit("solana_rpc", 3)     // Max 3 concurrent Solana RPC calls
    .with_limit("evm_rpc", 8)        // Max 8 concurrent EVM RPC calls
    .with_limit("http_api", 15)      // Max 15 concurrent HTTP requests
    .with_limit("database", 5);      // Max 5 concurrent database operations

// Use default limits (solana_rpc: 5, evm_rpc: 10, http_api: 20)
let default_limits = ResourceLimits::default();
```

# Resource Pool Mapping

The system automatically maps tool names to resource pools:

```text
Tool Name              → Resource Pool    → Limit
"solana_balance"       → "solana_rpc"     → configured limit
"evm_swap"             → "evm_rpc"        → configured limit
"web_fetch"            → "http_api"       → configured limit
"custom_tool"          → default          → ExecutionConfig.max_concurrency
```

---

#### `RetryConfig`

Configuration for retry behavior

---

#### `SolanaSignerHandle`

A handle to the current Solana signer, providing type-safe access.

This handle guarantees that the underlying signer supports Solana operations.
It implements `Deref` to `dyn SolanaSigner`, allowing direct access to all
Solana-specific methods.

# Thread Safety

The handle is `Send + Sync` and can be passed between threads safely.
The underlying signer is reference-counted and immutable.

# Examples

```ignore
use riglr_core::signer::SignerContext;

async fn solana_transfer() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get a type-safe handle to the Solana signer
    let signer = SignerContext::current_as_solana().await?;
     
    // Access Solana-specific methods directly
    let pubkey = signer.pubkey();
    let signature = signer.sign_message(b"Hello").await?;
     
    Ok(())
}
```

---

#### `TokenBucketStrategy`

Token bucket rate limiting strategy

This strategy implements a time-based token bucket algorithm where:
- Tokens are replenished continuously based on elapsed time
- The replenishment rate is `max_requests` per `time_window`
- Burst capacity allows temporary spikes up to `burst_size` tokens

---

#### `WorkerMetrics`

Performance and operational metrics for monitoring worker health.

These metrics provide insight into worker performance and can be used
for monitoring, alerting, and performance optimization.

All metrics use atomic integers for thread-safe updates and reads.

## Metrics Tracked

- **jobs_processed**: Total number of jobs dequeued and processed
- **jobs_succeeded**: Number of jobs that completed successfully
- **jobs_failed**: Number of jobs that failed permanently (after all retries)
- **jobs_retried**: Total number of retry attempts across all jobs

## Examples

```ignore
use riglr_core::{ToolWorker, ExecutionConfig, idempotency::InMemoryIdempotencyStore};
use std::sync::atomic::Ordering;

# async fn example() {
let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
    ExecutionConfig::default()
);

let metrics = worker.metrics();

// Read current metrics
let processed = metrics.jobs_processed.load(Ordering::Relaxed);
let succeeded = metrics.jobs_succeeded.load(Ordering::Relaxed);
let failed = metrics.jobs_failed.load(Ordering::Relaxed);
let retried = metrics.jobs_retried.load(Ordering::Relaxed);

println!("Worker Stats:");
println!("  Processed: {}", processed);
println!("  Succeeded: {}", succeeded);
println!("  Failed: {}", failed);
println!("  Retried: {}", retried);
println!("  Success Rate: {:.2}%",
    if processed > 0 { (succeeded as f64 / processed as f64) * 100.0 } else { 0.0 });
# }
```

---

### Enums

> Enumeration types for representing variants.

#### `Chain`

Enum representing supported blockchain types

**Variants:**

- `Solana`
  - Solana blockchain
- `Evm`
  - EVM-compatible blockchain with the specified chain ID

---

#### `CoreError`

Main error type for riglr-core operations.

**Variants:**

- `Queue`
  - Queue operation failed
- `JobExecution`
  - Job execution failed
- `Serialization`
  - Serialization/deserialization failed
- `Redis`
  - Redis connection error (only available with redis feature)
- `Generic`
  - Generic error
- `InvalidInput`
  - Invalid input provided

---

#### `EnvError`

Error type for environment variable operations

**Variants:**

- `MissingRequired`
  - Required environment variable is not set
- `InvalidUtf8`
  - Environment variable contains invalid UTF-8

---

#### `ErrorClass`

Error classification for retry logic

**Variants:**

- `Permanent`
  - Error is permanent and should not be retried
- `Retryable`
  - Error is temporary and should be retried
- `RateLimited`
  - Error indicates rate limiting, use longer backoff

---

#### `RateLimitStrategyType`

Rate limiting strategy type

**Variants:**

- `TokenBucket`
  - Token bucket algorithm (default)
- `FixedWindow`
  - Fixed window algorithm

---

#### `SignerError`

Error types for signer operations

**Variants:**

- `NoSignerContext`
  - No signer context is currently available
- `BlockchainTransaction`
  - Blockchain transaction error (generic, preserves error as string)
- `EvmTransaction`
  - EVM blockchain transaction error
- `Configuration`
  - Invalid signer configuration
- `Signing`
  - Transaction signing operation failed
- `ClientCreation`
  - Blockchain client creation failed
- `InvalidPrivateKey`
  - Private key format or content is invalid
- `ProviderError`
  - Blockchain provider error
- `TransactionFailed`
  - Transaction execution failed on blockchain
- `UnsupportedOperation`
  - Operation not supported by current signer
- `BlockhashError`
  - Blockchain hash retrieval or validation error
- `UnsupportedNetwork`
  - Network not supported by signer
- `InvalidRpcUrl`
  - RPC URL format is invalid or unreachable

---

#### `TransactionStatus`

Transaction status tracking for job lifecycle

This enum represents the various states a transaction can be in during
its execution lifecycle, from initial submission through final confirmation.

**Variants:**

- `Pending`
  - Transaction is pending submission
- `Submitted`
  - Transaction has been submitted to the network
- `Confirming`
  - Transaction is being confirmed
- `Confirmed`
  - Transaction has been confirmed
- `Failed`
  - Transaction failed

---

#### `WorkerError`

Worker-specific error type for distinguishing system-level worker failures
from tool execution failures.

**Variants:**

- `ToolNotFound`
  - Tool not found in the worker's registry
- `SemaphoreAcquisition`
  - Failed to acquire semaphore for concurrency control
- `IdempotencyStore`
  - Idempotency store operation failed
- `JobSerialization`
  - Job serialization/deserialization error
- `ExecutionTimeout`
  - Tool execution exceeded configured timeout
- `Internal`
  - Internal worker system error

---

### Traits

> Trait definitions for implementing common behaviors.

#### `AppContextExtension`

Generic extension trait for custom providers

This can be used for any custom client type that needs to be accessed
from the ApplicationContext.

# Example:
```rust,ignore
use riglr_core::provider::ApplicationContext;
use riglr_core::provider_extensions::AppContextExtension;
use riglr_core::ToolError;
use std::sync::Arc;

struct MyCustomClient;

impl AppContextExtension<MyCustomClient> for ApplicationContext {
    fn get_client(&self) -> Result<Arc<MyCustomClient>, ToolError> {
        self.get_extension::<Arc<MyCustomClient>>()
            .ok_or_else(|| ToolError::permanent_string(
                "MyCustomClient not configured in ApplicationContext"
            ))
    }
}
```

**Methods:**

- `get_client()`
  - Get a client of type T from the context

---

#### `EvmAppContextProvider`

Example trait that should be implemented in riglr-evm-tools

# Example Implementation (in riglr-evm-tools/src/lib.rs):
```rust,ignore
use riglr_core::provider::ApplicationContext;
use riglr_core::provider_extensions::EvmAppContextProvider;
use riglr_core::ToolError;
use ethers::providers::Provider;
use std::sync::Arc;

impl EvmAppContextProvider for ApplicationContext {
    fn evm_client(&self) -> Result<Arc<dyn Provider>, ToolError> {
        self.get_extension::<Arc<dyn Provider>>()
            .ok_or_else(|| ToolError::permanent_string(
                "EVM provider not configured in ApplicationContext"
            ))
    }
}
```

**Methods:**

- `evm_client()`
  - Get the EVM provider from the context

---

#### `EvmClient`

Chain-agnostic interface for EVM-compatible blockchain clients.

This trait abstracts EVM network operations without exposing
implementation details from alloy, ethers-rs, or other EVM libraries.

# Implementation Notes

Concrete implementations should wrap actual EVM clients (e.g., alloy Provider)
and handle JSON serialization/deserialization internally.

**Methods:**

- `get_balance()`
  - Returns the balance of an address in wei as a string.
- `send_transaction()`
  - Sends a transaction to the network and returns the transaction hash.
- `call()`
  - Executes a call against the network without creating a transaction.

---

#### `EvmSigner`

Trait for EVM-compatible blockchain signing capabilities.

Implementations of this trait handle operations for Ethereum
and EVM-compatible chains (Polygon, Arbitrum, Optimism, etc.)
without exposing alloy or ethers types to riglr-core.

# Chain-Agnostic Design

Transactions are passed as JSON values to avoid dependency on
specific Ethereum libraries (alloy, ethers-rs, etc.).

**Methods:**

- `chain_id()`
  - Returns the EVM chain ID this signer is configured for.
- `address()`
  - Returns the EVM wallet address in checksummed hex format.
- `sign_and_send_transaction()`
  - Signs and sends an EVM transaction to the network.
- `sign_and_send_with_retry()`
  - Signs and sends a transaction with automatic retry on transient failures.
- `client()`
  - Returns the underlying EVM RPC client.

---

#### `IdempotencyStore`

Trait for idempotency store implementations

**Methods:**

- `get()`
  - Check if a result exists for the given idempotency key
- `set()`
  - Store a result with the given idempotency key and TTL
- `remove()`
  - Remove an entry by key

---

#### `JobQueue`

Trait for job queue implementations.

Provides a common interface for different queue backends, enabling
both local development with in-memory queues and production deployment
with distributed Redis queues.

**Methods:**

- `enqueue()`
  - Add a job to the queue
- `dequeue()`
  - Get the next job from the queue, blocks until a job is available or timeout
- `dequeue_with_timeout()`
  - Get the next job from the queue with timeout
- `len()`
  - Get queue length
- `is_empty()`
  - Check if queue is empty

---

#### `MultiChainSigner`

Trait for signers that support multiple chains
This combines the capabilities of both Solana and EVM signers

**Methods:**

- `primary_chain()`
  - Get the primary chain this signer operates on
- `supports_chain()`
  - Check if a specific chain is supported
- `as_solana()`
  - Get as Solana signer if supported
- `as_evm()`
  - Get as EVM signer if supported

---

#### `RateLimitStrategy`

Trait defining the interface for rate limiting strategies

Different strategies can implement this trait to provide various
rate limiting algorithms such as token bucket, fixed window, sliding window, etc.

**Methods:**

- `check_rate_limit()`
  - Check if a request should be allowed for the given client
- `reset_client()`
  - Reset rate limit state for a specific client
- `clear_all()`
  - Clear all rate limit data
- `get_request_count()`
  - Get current request count for a client
- `strategy_name()`
  - Get strategy name for debugging/logging

---

#### `SentimentAnalyzerMarker`

Marker trait for sentiment analyzers

This trait serves as a type marker for sentiment analysis implementations
that can be stored in ApplicationContext's extension system.

The actual sentiment analysis logic is defined in implementing crates
(e.g., riglr-web-tools) to avoid circular dependencies.

# Example

```rust,no_run
use riglr_core::provider::ApplicationContext;
use std::sync::Arc;

// In application setup:
let context = ApplicationContext::default();

// Store a sentiment analyzer (from riglr-web-tools or custom implementation)
// let analyzer = Arc::new(MyConcreteAnalyzer::new());
// context.set_extension(analyzer);

// Later, retrieve it:
// let analyzer = context.get_extension::<Arc<dyn SentimentAnalyzerMarker>>()
//     .expect("Sentiment analyzer not configured");
```

---

#### `SignerBase`

Base trait for all signers providing common metadata and type erasure.

This trait provides the foundation for all signer implementations,
enabling multi-tenant support and runtime type identification.

**Methods:**

- `locale()`
  - Returns the user's locale for localized error messages and responses.
- `user_id()`
  - Returns an optional user identifier for multi-tenant scenarios.
- `as_any()`
  - Returns a reference to self as `Any` for downcasting to concrete types.

---

#### `SolanaAppContextProvider`

Example trait that should be implemented in riglr-solana-tools

# Example Implementation (in riglr-solana-tools/src/lib.rs):
```rust,ignore
use riglr_core::provider::ApplicationContext;
use riglr_core::provider_extensions::SolanaAppContextProvider;
use riglr_core::ToolError;
use solana_client::rpc_client::RpcClient as SolanaRpcClient;
use std::sync::Arc;

impl SolanaAppContextProvider for ApplicationContext {
    fn solana_client(&self) -> Result<Arc<SolanaRpcClient>, ToolError> {
        self.get_extension::<Arc<SolanaRpcClient>>()
            .ok_or_else(|| ToolError::permanent_string(
                "Solana RPC client not configured in ApplicationContext"
            ))
    }
}
```

**Methods:**

- `solana_client()`
  - Get the Solana RPC client from the context

---

#### `SolanaClient`

Chain-agnostic interface for Solana blockchain clients.

This trait abstracts Solana network operations without exposing
types from solana-sdk or solana-client.

# Implementation Notes

Concrete implementations should wrap actual Solana RPC clients
and handle transaction serialization with bincode internally.

**Methods:**

- `get_balance()`
  - Returns the balance of a Solana account in lamports.
- `send_transaction()`
  - Sends a transaction to the Solana network and returns the signature.

---

#### `SolanaSigner`

Trait for Solana blockchain signing capabilities.

Implementations of this trait handle Solana-specific operations
without exposing Solana SDK types to riglr-core.

# Chain-Agnostic Design

Transactions are passed as serialized byte arrays using bincode,
avoiding direct dependency on solana-sdk types.

**Methods:**

- `address()`
  - Returns the Solana wallet address in base58 encoding.
- `pubkey()`
  - Returns the Solana public key in base58 encoding.
- `sign_and_send_transaction()`
  - Signs and sends a Solana transaction to the network.
- `sign_and_send_with_retry()`
  - Signs and sends a transaction with automatic retry on transient failures.
- `client()`
  - Returns the underlying Solana RPC client as a type-erased trait object.

---

#### `UnifiedSigner`

Unified signer trait that can represent any type of signer
This is used as the primary trait for SignerContext

**Methods:**

- `supports_solana()`
  - Check if this signer supports Solana
- `supports_evm()`
  - Check if this signer supports EVM
- `as_solana()`
  - Try to get as a Solana signer
- `as_evm()`
  - Try to get as an EVM signer
- `as_multi_chain()`
  - Try to get as a MultiChain signer

---

### Functions

> Standalone functions and utilities.

#### `ensure_key_directory`

Create the default key directory with appropriate permissions

---

#### `get_default_key_directory`

Get the default key directory for riglr

Returns ~/.riglr/keys on Unix-like systems
Returns %APPDATA%\riglr\keys on Windows

---

#### `get_env_or_default`

Gets an optional environment variable with a default value.

# Examples

```rust
use riglr_core::util::{get_env_or_default, DOCTEST_OPTIONAL_SETTING};

# std::env::remove_var(DOCTEST_OPTIONAL_SETTING);
let setting = get_env_or_default(DOCTEST_OPTIONAL_SETTING, "default_value");
assert_eq!(setting, "default_value");
```

---

#### `get_env_vars`

Gets multiple environment variables at once, returning a map.

# Examples

```rust
use riglr_core::util::{get_env_vars, DOCTEST_VAR1, DOCTEST_VAR2, DOCTEST_VAR3};
use std::collections::HashMap;

# std::env::set_var(DOCTEST_VAR1, "value1");
# std::env::set_var(DOCTEST_VAR2, "value2");
let vars = get_env_vars(&[DOCTEST_VAR1, DOCTEST_VAR2, DOCTEST_VAR3]);
assert_eq!(vars.get(DOCTEST_VAR1), Some(&"value1".to_string()));
assert_eq!(vars.get(DOCTEST_VAR3), None);
# std::env::remove_var(DOCTEST_VAR1);
# std::env::remove_var(DOCTEST_VAR2);
```

---

#### `get_required_env`

Gets a required environment variable, returning an error if not set.

This is the recommended approach for libraries, allowing the application
to decide how to handle missing configuration.

# Examples

```rust
use riglr_core::util::{get_required_env, DOCTEST_API_KEY};

# std::env::set_var(DOCTEST_API_KEY, "secret123");
let api_key = get_required_env(DOCTEST_API_KEY).expect("MY_API_KEY must be set");
assert_eq!(api_key, "secret123");
# std::env::remove_var(DOCTEST_API_KEY);
```

# Errors

Returns [`EnvError::MissingRequired`] if the environment variable is not set.

---

#### `init_env_from_file`

Application-level helper that initializes environment from a `.env` file if present.

This is a convenience function for applications (not libraries) that want to
support `.env` files for local development.

# Examples

```rust
use riglr_core::util::init_env_from_file;

// Load .env file if it exists (usually at application startup)
init_env_from_file(".env").ok(); // Ignore if file doesn't exist
```

---

#### `load_private_key_from_file`

Load a private key from a file with appropriate security checks

# Security Notes
- The key file should have restricted permissions (e.g., 0600 on Unix)
- Never commit key files to version control
- Consider using system keyrings or HSMs in production

# Example
```no_run
use riglr_core::util::load_private_key_from_file;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Load from default location
let key = load_private_key_from_file("~/.riglr/keys/solana.key")?;

// Load from custom location
let key = load_private_key_from_file("/secure/keys/my-key.pem")?;
# Ok(())
# }
```

---

#### `load_private_key_with_fallback`

Load a private key with fallback to environment variable

This function first attempts to load from a file, then falls back to
an environment variable if the file doesn't exist.

# Example
```no_run
use riglr_core::util::load_private_key_with_fallback;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Try file first, then env var
let key = load_private_key_with_fallback(
    "~/.riglr/keys/solana.key",
    "SOLANA_PRIVATE_KEY"
)?;
# Ok(())
# }
```

---

#### `retry_async`

Execute an async operation with retry logic

This function provides a generic retry mechanism for any async operation.
It automatically applies exponential backoff with optional jitter.

# Arguments

* `operation` - Async closure that performs the operation
* `classifier` - Function to classify errors as permanent or retryable
* `config` - Retry configuration
* `operation_name` - Human-readable name for logging

# Type Parameters

* `F` - The async operation closure type
* `Fut` - The future type returned by the operation
* `T` - The success type
* `E` - The error type
* `C` - The error classifier function type

# Returns

Returns the successful result or the last error after all retries are exhausted

# Examples

```rust,ignore
use riglr_core::retry::{retry_async, RetryConfig, ErrorClass};

async fn example() -> Result<String, MyError> {
    retry_async(
        || async {
            // Your async operation here
            fetch_data().await
        },
        |error| {
            // Classify error
            match error {
                MyError::NetworkTimeout => ErrorClass::Retryable,
                MyError::InvalidInput => ErrorClass::Permanent,
                MyError::RateLimited => ErrorClass::RateLimited,
            }
        },
        &RetryConfig::default(),
        "fetch_data"
    ).await
}
```

---

#### `retry_with_backoff`

Simplified retry for operations that return std::result::Result

---

#### `spawn_with_context`

Spawns a new task while preserving SignerContext if available.

# Why This Is Necessary

The `SignerContext` is stored in `tokio::task_local!` storage, which means it is
**NOT** automatically propagated to new tasks spawned with `tokio::spawn`. This is
a fundamental limitation of task-local storage - it's only accessible within the
same task where it was set.

Without this function, the following would fail:

```rust,ignore
// WRONG - This will fail with "No signer context"
SignerContext::with_signer(signer, async {
    let handle = tokio::spawn(async {
        // This will fail - SignerContext is not available here!
        let current = SignerContext::current().await?; // ERROR
        transfer_sol("recipient", 1.0).await
    });
    handle.await?
}).await
```

The correct approach using `spawn_with_context`:

```rust,ignore
// CORRECT - This properly propagates the SignerContext
SignerContext::with_signer(signer, async {
    let handle = spawn_with_context(async {
        // SignerContext is available here!
        transfer_sol("recipient", 1.0).await
    }).await;
    handle.await?
}).await
```

# How It Works

This function:
1. Checks if a SignerContext exists in the current task
2. If yes, captures it and wraps the spawned future with `SignerContext::with_signer`
3. If no, spawns the task normally without context

This ensures that tools requiring signing operations work correctly when
executed through agent frameworks that use task spawning for parallelism.

Note: This function is async because it needs to check for the current signer.
The future passed in should return a Result<T, SignerError>.

---

#### `validate_required_env`

Validates that all required environment variables are set.

This is useful during application initialization to fail fast if
configuration is incomplete.

# Examples

```rust
use riglr_core::util::{validate_required_env, DOCTEST_API_KEY_VALIDATE, DOCTEST_DATABASE_URL};

# std::env::set_var(DOCTEST_API_KEY_VALIDATE, "value1");
# std::env::set_var(DOCTEST_DATABASE_URL, "value2");
let required = vec![DOCTEST_API_KEY_VALIDATE, DOCTEST_DATABASE_URL];
validate_required_env(&required).expect("Missing required environment variables");
# std::env::remove_var(DOCTEST_API_KEY_VALIDATE);
# std::env::remove_var(DOCTEST_DATABASE_URL);
```

# Errors

Returns the first [`EnvError::MissingRequired`] encountered.

---

### Type Aliases

#### `EnvResult`

Result type alias for environment operations

**Type:** `<T, >`

---

#### `Result`

Result type alias for riglr-core operations.

**Type:** `<T, >`

---
