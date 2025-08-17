# riglr-core API Reference

Comprehensive API documentation for the `riglr-core` crate.

## Table of Contents

### Structs

- [`EvmNetworkConfig`](#evmnetworkconfig)
- [`EvmTransactionProcessor`](#evmtransactionprocessor)
- [`ExecutionConfig`](#executionconfig)
- [`GasConfig`](#gasconfig)
- [`GenericTransactionProcessor`](#generictransactionprocessor)
- [`InMemoryIdempotencyStore`](#inmemoryidempotencystore)
- [`InMemoryJobQueue`](#inmemoryjobqueue)
- [`Job`](#job)
- [`LegacySignerAdapter`](#legacysigneradapter)
- [`LocalEvmSigner`](#localevmsigner)
- [`LocalSolanaSigner`](#localsolanasigner)
- [`PriorityFeeConfig`](#priorityfeeconfig)
- [`RedisIdempotencyStore`](#redisidempotencystore)
- [`RedisJobQueue`](#redisjobqueue)
- [`ResourceLimits`](#resourcelimits)
- [`RetryConfig`](#retryconfig)
- [`RpcConfig`](#rpcconfig)
- [`SignerContext`](#signercontext)
- [`SolanaNetworkConfig`](#solananetworkconfig)
- [`SolanaTransactionProcessor`](#solanatransactionprocessor)
- [`ToolWorker`](#toolworker)
- [`WorkerMetrics`](#workermetrics)

### Functions

- [`add_evm_network`](#add_evm_network)
- [`add_priority_fee_instructions`](#add_priority_fee_instructions)
- [`caip2`](#caip2)
- [`can_retry`](#can_retry)
- [`current`](#current)
- [`current_as_evm`](#current_as_evm)
- [`current_as_solana`](#current_as_solana)
- [`estimate_gas_limit`](#estimate_gas_limit)
- [`estimate_gas_price`](#estimate_gas_price)
- [`evm_caip2_for`](#evm_caip2_for)
- [`from_config`](#from_config)
- [`from_config`](#from_config)
- [`from_env`](#from_env)
- [`from_keypair`](#from_keypair)
- [`get_address`](#get_address)
- [`get_env_or_default`](#get_env_or_default)
- [`get_env_vars`](#get_env_vars)
- [`get_pubkey`](#get_pubkey)
- [`get_recent_prioritization_fees`](#get_recent_prioritization_fees)
- [`get_required_env`](#get_required_env)
- [`get_semaphore`](#get_semaphore)
- [`increment_retry`](#increment_retry)
- [`init_env_from_file`](#init_env_from_file)
- [`invalid_input_string`](#invalid_input_string)
- [`invalid_input_with_source`](#invalid_input_with_source)
- [`is_available`](#is_available)
- [`is_retriable`](#is_retriable)
- [`is_success`](#is_success)
- [`metrics`](#metrics)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new_idempotent`](#new_idempotent)
- [`optimize_transaction`](#optimize_transaction)
- [`permanent_failure`](#permanent_failure)
- [`permanent_string`](#permanent_string)
- [`permanent_with_source`](#permanent_with_source)
- [`prepare_transaction`](#prepare_transaction)
- [`process_job`](#process_job)
- [`rate_limited_string`](#rate_limited_string)
- [`rate_limited_with_source`](#rate_limited_with_source)
- [`register_tool`](#register_tool)
- [`retriable_failure`](#retriable_failure)
- [`retriable_string`](#retriable_string)
- [`retriable_with_source`](#retriable_with_source)
- [`run`](#run)
- [`send_transaction_with_retry`](#send_transaction_with_retry)
- [`simulate_transaction`](#simulate_transaction)
- [`success`](#success)
- [`success_with_tx`](#success_with_tx)
- [`validate_required_env`](#validate_required_env)
- [`with_env_overrides`](#with_env_overrides)
- [`with_idempotency_store`](#with_idempotency_store)
- [`with_limit`](#with_limit)
- [`with_resource_limits`](#with_resource_limits)
- [`with_signer`](#with_signer)
- [`with_timeout`](#with_timeout)
- [`with_unified_signer`](#with_unified_signer)

### Enums

- [`Chain`](#chain)
- [`CoreError`](#coreerror)
- [`EnvError`](#enverror)
- [`JobResult`](#jobresult)
- [`SignerError`](#signererror)
- [`ToolError`](#toolerror)
- [`TransactionStatus`](#transactionstatus)
- [`WorkerError`](#workererror)

### Traits

- [`EvmClient`](#evmclient)
- [`EvmSigner`](#evmsigner)
- [`IdempotencyStore`](#idempotencystore)
- [`JobQueue`](#jobqueue)
- [`MultiChainSigner`](#multichainsigner)
- [`SignerBase`](#signerbase)
- [`SolanaClient`](#solanaclient)
- [`SolanaSigner`](#solanasigner)
- [`Tool`](#tool)
- [`TransactionProcessor`](#transactionprocessor)
- [`TransactionSigner`](#transactionsigner)
- [`UnifiedSigner`](#unifiedsigner)

## Structs

### EvmNetworkConfig

**Source**: `src/config.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct EvmNetworkConfig { /// Human-readable display name for the network (e.g., "Ethereum Mainnet", "Polygon")
```

EVM network configuration

---

### EvmTransactionProcessor

**Source**: `transactions/evm.rs`

```rust
pub struct EvmTransactionProcessor<P: Provider> { /// The blockchain provider for interacting with the EVM network provider: Arc<P>, /// Gas configuration settings for transaction optimization gas_config: GasConfig, }
```

EVM transaction processor with gas optimization

---

### ExecutionConfig

**Source**: `src/tool.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct ExecutionConfig { /// Maximum number of concurrent executions per resource type. /// /// This controls the default concurrency limit when no specific resource /// limit is configured. Individual resource types can have their own /// limits configured via [`ResourceLimits`]. /// /// **Default**: 10 pub max_concurrency: usize, /// Default timeout for tool execution. /// /// Individual tool executions will be cancelled if they exceed this duration. /// Tools should be designed to complete within reasonable time bounds. /// /// **Default**: 30 seconds pub default_timeout: Duration, /// Maximum number of retry attempts for failed operations. /// /// This applies only to retriable failures. Permanent failures are never retried. /// The total number of execution attempts will be `max_retries + 1`. /// /// **Default**: 3 retries (4 total attempts)
```

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

### GasConfig

**Source**: `transactions/evm.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct GasConfig { /// Use EIP-1559 (base fee + priority fee)
```

EVM gas configuration

---

### GenericTransactionProcessor

**Source**: `transactions/mod.rs`

```rust
pub struct GenericTransactionProcessor;
```

Generic transaction processor implementation

---

### InMemoryIdempotencyStore

**Source**: `src/idempotency.rs`

```rust
pub struct InMemoryIdempotencyStore { store: Arc<DashMap<String, IdempotencyEntry>>, }
```

In-memory idempotency store for testing and development

---

### InMemoryJobQueue

**Source**: `src/queue.rs`

```rust
pub struct InMemoryJobQueue { queue: tokio::sync::Mutex<std::collections::VecDeque<Job>>, notify: tokio::sync::Notify, }
```

In-memory job queue implementation for testing and development

---

### Job

**Source**: `src/jobs.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct Job { /// Unique identifier for this job instance. /// /// Generated automatically when the job is created. This ID is used /// for tracking and logging purposes throughout the job's lifecycle. pub job_id: Uuid, /// Name of the tool to execute. /// /// This must match the name of a tool registered with the [`ToolWorker`](crate::ToolWorker).
```

A job represents a unit of work to be executed by a [`ToolWorker`](crate::ToolWorker).

Jobs encapsulate all the information needed to execute a tool, including
the tool name, parameters, retry configuration, and optional idempotency key.
They are the primary unit of work in the riglr job processing system.

## Job Lifecycle

1. **Creation** - Job is created with tool name and parameters
2. **Queuing** - Job is submitted to a job queue for processing
3. **Execution** - Worker picks up job and executes the corresponding tool
4. **Retry** - If execution fails with a retriable error, job may be retried
5. **Completion** - Job succeeds or fails permanently after exhausting retries

## Examples

### Basic Job Creation

```rust
use riglr_core::Job;
use serde_json::json;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
let job = Job::new(
"price_checker",
&json!({
"symbol": "BTC",
"currency": "USD"
}),
3 // max retries
)?;

println!("Job ID: {}", job.job_id);
println!("Tool: {}", job.tool_name);
println!("Can retry: {}", job.can_retry());
# Ok(())
# }
```

### Idempotent Job Creation

```rust
use riglr_core::Job;
use serde_json::json;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
let job = Job::new_idempotent(
"bank_transfer",
&json!({
"from_account": "123",
"to_account": "456",
"amount": 100.00,
"currency": "USD"
}),
2, // max retries
"transfer_123_456_100_20241201" // idempotency key
)?;

assert!(job.idempotency_key.is_some());
# Ok(())
# }
```

---

### LegacySignerAdapter

**Source**: `signer/granular_traits.rs`

```rust
pub struct LegacySignerAdapter<T: super::traits::TransactionSigner + 'static> { /// The wrapped legacy transaction signer inner: Arc<T>, }
```

Adapter to wrap TransactionSigner for use with granular traits

---

### LocalEvmSigner

**Source**: `signer/evm.rs`

```rust
pub struct LocalEvmSigner { wallet: EthereumWallet, provider_url: String, chain_id: u64, _config: EvmNetworkConfig, }
```

Local EVM signer with private key management

---

### LocalSolanaSigner

**Source**: `signer/solana.rs`

```rust
pub struct LocalSolanaSigner { keypair: Arc<Keypair>, client: Arc<RpcClient>, blockhash_cache: Arc<RwLock<BlockhashCache>>, _config: SolanaNetworkConfig, }
```

Local Solana signer with keypair management

---

### PriorityFeeConfig

**Source**: `transactions/solana.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct PriorityFeeConfig { /// Enable priority fees pub enabled: bool, /// Priority fee in microlamports per compute unit pub microlamports_per_cu: u64, /// Additional compute units to request pub additional_compute_units: Option<u32>, }
```

Solana priority fee configuration

---

### RedisIdempotencyStore

**Source**: `src/idempotency.rs`

**Attributes**:
```rust
#[cfg(feature = "redis")]
```

```rust
pub struct RedisIdempotencyStore { client: redis::Client, key_prefix: String, }
```

Redis-based idempotency store for production use

---

### RedisJobQueue

**Source**: `src/queue.rs`

**Attributes**:
```rust
#[cfg(feature = "redis")]
```

```rust
pub struct RedisJobQueue { client: redis::Client, queue_key: String, timeout_seconds: u64, }
```

Redis-based job queue implementation for production use

---

### ResourceLimits

**Source**: `src/tool.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct ResourceLimits { /// Resource name to semaphore mapping. /// /// Each semaphore controls the maximum number of concurrent operations /// for its associated resource type. semaphores: Arc<HashMap<String, Arc<Semaphore>>>, }
```

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
let limits = ResourceLimits::new()
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

### RetryConfig

**Source**: `transactions/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct RetryConfig { /// Maximum number of retry attempts pub max_attempts: u32, /// Initial delay between retries pub initial_delay: Duration, /// Maximum delay between retries pub max_delay: Duration, /// Exponential backoff multiplier pub backoff_multiplier: f64, }
```

Transaction retry configuration

---

### RpcConfig

**Source**: `src/config.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct RpcConfig { /// Collection of EVM-compatible blockchain network configurations indexed by lowercase network name pub evm_networks: HashMap<String, EvmNetworkConfig>, /// Collection of Solana network configurations indexed by network name pub solana_networks: HashMap<String, SolanaNetworkConfig>, }
```

Type-safe RPC configuration for blockchain networks

---

### SignerContext

**Source**: `src/signer.rs`

```rust
pub struct SignerContext;
```

The SignerContext provides thread-local signer management for secure multi-tenant operation.

This enables stateless tools that can access the appropriate signer without explicit passing,
while maintaining strict isolation between different async tasks and users.

## Security Features

- **Thread isolation**: Each async task has its own isolated signer context
- **No signer leakage**: Contexts cannot access signers from other tasks
- **Safe concurrent access**: Multiple tasks can run concurrently with different signers
- **Automatic cleanup**: Contexts are automatically cleaned up when tasks complete

## Usage Patterns

### Basic Usage

```ignore
use riglr_core::signer::SignerContext;
use riglr_solana_tools::LocalSolanaSigner;
use std::sync::Arc;
# use solana_sdk::signer::keypair::Keypair;

# async fn example() -> Result<(), riglr_core::signer::SignerError> {
let keypair = Keypair::new();
let signer = Arc::new(LocalSolanaSigner::new(
keypair,
"https://api.devnet.solana.com".to_string()
));

// Execute code with signer context
let result = SignerContext::with_signer(signer, async {
// Inside this scope, tools can access the signer
let current = SignerContext::current().await?;
let user_id = current.user_id();
Ok(format!("Processing for user: {:?}", user_id))
}).await?;

println!("Result: {}", result);
# Ok(())
# }
```

### Multi-Tenant Service Example

```ignore
use riglr_core::signer::{SignerContext, TransactionSigner};
use std::sync::Arc;

async fn handle_user_request(
user_signer: Arc<dyn TransactionSigner>,
operation: &str
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
SignerContext::with_signer(user_signer, async {
// All operations in this scope use the user's signer
match operation {
"balance" => check_balance().await,
"transfer" => perform_transfer().await,
_ => Err(riglr_core::signer::SignerError::NoSignerContext)
}
}).await.map_err(Into::into)
}

async fn check_balance() -> Result<String, riglr_core::signer::SignerError> {
let signer = SignerContext::current().await?;
Ok(format!("Balance for user: {:?}", signer.user_id()))
}

async fn perform_transfer() -> Result<String, riglr_core::signer::SignerError> {
let signer = SignerContext::current().await?;
// Perform actual transfer using signer...
Ok("Transfer completed".to_string())
}
```

### Error Handling

Tools should always check for signer availability:

```rust
use riglr_core::signer::SignerContext;

async fn safe_operation() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
if !SignerContext::is_available().await {
return Err("This operation requires a signer context".into());
}

let signer = SignerContext::current().await?;
// Proceed with operation...
Ok("Operation completed".to_string())
}
```

## Security Considerations

- **Never store signers globally**: Always use the context pattern
- **Validate user permissions**: Check that users own the addresses they're operating on
- **Audit all operations**: Log all signer usage for security auditing
- **Use environment-specific endpoints**: Different signers for mainnet/testnet

---

### SolanaNetworkConfig

**Source**: `src/config.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct SolanaNetworkConfig { /// Human-readable display name for the Solana network (e.g., "Solana Mainnet", "Solana Devnet")
```

Solana network configuration

---

### SolanaTransactionProcessor

**Source**: `transactions/solana.rs`

```rust
pub struct SolanaTransactionProcessor { /// The RPC client for interacting with the Solana network client: Arc<RpcClient>, /// Priority fee configuration for transaction optimization priority_config: PriorityFeeConfig, }
```

Solana transaction processor with priority fee support

---

### ToolWorker

**Source**: `src/tool.rs`

```rust
pub struct ToolWorker<I: IdempotencyStore + 'static> { /// Registered tools, indexed by name for fast lookup tools: Arc<DashMap<String, Arc<dyn Tool>>>, /// Default semaphore for general concurrency control default_semaphore: Arc<Semaphore>, /// Resource-specific limits for fine-grained control resource_limits: ResourceLimits, /// Configuration for retry behavior, timeouts, etc. config: ExecutionConfig, /// Optional idempotency store for caching results idempotency_store: Option<Arc<I>>, /// Performance and operational metrics metrics: Arc<WorkerMetrics>, }
```

A worker that processes jobs from a queue using registered tools.

The `ToolWorker` is the core execution engine of the riglr system. It manages
registered tools, processes jobs with resilience features, and provides
comprehensive monitoring and observability.

## Key Features

- **Resilient execution**: Automatic retries with exponential backoff
- **Resource management**: Configurable limits per resource type
- **Idempotency support**: Cached results for safe retries
- **Comprehensive monitoring**: Built-in metrics and structured logging
- **Concurrent processing**: Efficient parallel job execution

## Architecture

The worker operates on a job-based model where:
1. Jobs are dequeued from a [`JobQueue`]
2. Tools are looked up by name and executed
3. Results are processed with retry logic for failures
4. Successful results are optionally cached for idempotency
5. Metrics are updated for monitoring

## Examples

### Basic Setup

```ignore
use riglr_core::{
ToolWorker, ExecutionConfig, ResourceLimits,
idempotency::InMemoryIdempotencyStore
};
use std::sync::Arc;

# async fn example() -> anyhow::Result<()> {
// Create worker with default configuration
let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
ExecutionConfig::default()
);

// Add idempotency store for safe retries
let store = Arc::new(InMemoryIdempotencyStore::new());
let worker = worker.with_idempotency_store(store);

// Configure resource limits
let limits = ResourceLimits::new()
.with_limit("solana_rpc", 3)
.with_limit("evm_rpc", 5);
let worker = worker.with_resource_limits(limits);
# Ok(())
# }
```

### Processing Jobs

```ignore
use riglr_core::{ToolWorker, Job, Tool, JobResult, ExecutionConfig};
use riglr_core::idempotency::InMemoryIdempotencyStore;
use async_trait::async_trait;
use std::sync::Arc;

# struct ExampleTool;
# #[async_trait]
# impl Tool for ExampleTool {
#     async fn execute(&self, _: serde_json::Value) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
#         Ok(JobResult::success(&"example result")?)
#     }
#     fn name(&self) -> &str { "example" }
# }
# async fn example() -> anyhow::Result<()> {
let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
ExecutionConfig::default()
);

// Register tools
worker.register_tool(Arc::new(ExampleTool)).await;

// Process a job
let job = Job::new("example", &serde_json::json!({}), 3)?;
let result = worker.process_job(job).await.unwrap();

println!("Job result: {:?}", result);

// Check metrics
let metrics = worker.metrics();
println!("Jobs processed: {}",
metrics.jobs_processed.load(std::sync::atomic::Ordering::Relaxed));
# Ok(())
# }
```

---

### WorkerMetrics

**Source**: `src/tool.rs`

**Attributes**:
```rust
#[derive(Debug, Default)]
```

```rust
pub struct WorkerMetrics { /// Total number of jobs processed (dequeued and executed)
```

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

## Functions

### add_evm_network

**Source**: `src/config.rs`

```rust
pub fn add_evm_network( &mut self, name: impl Into<String>, chain_id: u64, rpc_url: impl Into<String>, explorer_url: Option<String>, ) -> &mut Self
```

Add or update an EVM network configuration dynamically.

---

### add_priority_fee_instructions

**Source**: `transactions/solana.rs`

```rust
pub fn add_priority_fee_instructions(&self, instructions: &mut Vec<Instruction>)
```

Add priority fee instructions to transaction

---

### caip2

**Source**: `src/config.rs`

**Attributes**:
```rust
#[must_use]
```

```rust
pub fn caip2(&self) -> String
```

Return the CAIP-2 identifier for this EVM network, e.g. "eip155:1".

---

### can_retry

**Source**: `src/jobs.rs`

```rust
pub fn can_retry(&self) -> bool
```

Check if this job has retries remaining.

Returns `true` if the job can be retried after a failure,
`false` if all retry attempts have been exhausted.

# Returns
* `true` - Job can be retried (`retry_count < max_retries`)
* `false` - No retries remaining (`retry_count >= max_retries`)

# Examples

```rust
use riglr_core::Job;
use serde_json::json;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
let mut job = Job::new("test_tool", &json!({}), 2)?;

assert!(job.can_retry());  // 0 < 2, can retry
job.increment_retry();
assert!(job.can_retry());  // 1 < 2, can retry
job.increment_retry();
assert!(!job.can_retry()); // 2 >= 2, cannot retry
# Ok(())
# }
```

---

### current

**Source**: `src/signer.rs`

```rust
pub async fn current() -> Result<Arc<dyn TransactionSigner>, SignerError>
```

Get the current signer from thread-local context.

This function retrieves the signer that was set by [`SignerContext::with_signer()`].
It must be called within a signer context scope, otherwise it will return
[`SignerError::NoSignerContext`].

# Returns
* `Ok(Arc<dyn TransactionSigner>)` - The current signer if available
* `Err(SignerError::NoSignerContext)` - If called outside a signer context

# Examples

```ignore
use riglr_core::signer::SignerContext;
use riglr_solana_tools::LocalSolanaSigner;
use std::sync::Arc;
# use solana_sdk::signer::keypair::Keypair;

async fn tool_that_needs_signer() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
// Get the current signer from context
let signer = SignerContext::current().await
.map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

// Use the signer for operations
match signer.user_id() {
Some(user) => Ok(format!("Operating for user: {}", user)),
None => Ok("Operating for anonymous user".to_string()),
}
}

# async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
let keypair = Keypair::new();
let signer = Arc::new(LocalSolanaSigner::new(
keypair,
"https://api.devnet.solana.com".to_string()
));

SignerContext::with_signer(signer, async {
let result = tool_that_needs_signer().await.unwrap();
println!("{}", result);
Ok(())
}).await.unwrap();
# Ok(())
# }
```

# Error Handling

This function will return an error if called outside a signer context:

```ignore
use riglr_core::signer::{SignerContext, SignerError};

# async fn error_example() {
// This will fail because we're not in a signer context
let result = SignerContext::current().await;
assert!(matches!(result, Err(SignerError::NoSignerContext)));
# }
```

---

### current_as_evm

**Source**: `src/signer.rs`

```rust
pub async fn current_as_evm() -> Result<Arc<dyn EvmSigner>, SignerError>
```

Get the current signer as an EVM signer

---

### current_as_solana

**Source**: `src/signer.rs`

```rust
pub async fn current_as_solana() -> Result<Arc<dyn SolanaSigner>, SignerError>
```

Get the current signer as a specific type.

This method allows type-safe access to chain-specific signer capabilities.
Tools can require specific signer types and get compile-time guarantees.

# Type Parameters
* `T` - The specific signer trait to cast to (e.g., `dyn SolanaSigner`, `dyn EvmSigner`)

# Returns
* `Ok(&T)` - Reference to the signer with the requested capabilities
* `Err(SignerError::UnsupportedOperation)` - If the current signer doesn't support the requested type
* `Err(SignerError::NoSignerContext)` - If no signer context is available

# Examples
```ignore
use riglr_core::signer::{SignerContext, SolanaSigner, EvmSigner};

// Require Solana signer
async fn solana_operation() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
let signer = SignerContext::current_as::<dyn SolanaSigner>().await?;
Ok(format!("Solana pubkey: {}", signer.pubkey()))
}

// Require EVM signer
async fn evm_operation() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
let signer = SignerContext::current_as::<dyn EvmSigner>().await?;
Ok(format!("EVM chain: {}", signer.chain_id()))
}
```

---

### estimate_gas_limit

**Source**: `transactions/evm.rs`

```rust
pub async fn estimate_gas_limit(&self, tx: &TransactionRequest) -> Result<u64, ToolError>
```

Estimate gas limit for a transaction

---

### estimate_gas_price

**Source**: `transactions/evm.rs`

```rust
pub async fn estimate_gas_price(&self) -> Result<u128, ToolError>
```

Estimate optimal gas price

---

### evm_caip2_for

**Source**: `src/config.rs`

**Attributes**:
```rust
#[must_use]
```

```rust
pub fn evm_caip2_for(&self, name: &str) -> Option<String>
```

Get CAIP-2 for an EVM network by name (case-insensitive key lookup).

---

### from_config

**Source**: `signer/evm.rs`

```rust
pub fn from_config( private_key: String, config: &RpcConfig, network: &str, ) -> Result<Self, SignerError>
```

Create a new EVM signer from RPC configuration and network name

---

### from_config

**Source**: `signer/solana.rs`

```rust
pub fn from_config( private_key: String, config: &RpcConfig, network: &str, ) -> Result<Self, SignerError>
```

Create a new Solana signer from RPC configuration and network name

---

### from_env

**Source**: `src/config.rs`

```rust
pub fn from_env() -> Result<Self, Box<dyn std::error::Error>>
```

Create `RpcConfig` from environment variables

# Errors

Returns an error if environment variables are invalid or cannot be parsed.

---

### from_keypair

**Source**: `signer/solana.rs`

```rust
pub fn from_keypair(keypair: Keypair, network_config: SolanaNetworkConfig) -> Self
```

Create a new Solana signer from a Keypair (for testing)

---

### get_address

**Source**: `signer/evm.rs`

```rust
pub fn get_address(&self) -> Address
```

Get the address of this signer

---

### get_env_or_default

**Source**: `src/util.rs`

```rust
pub fn get_env_or_default(key: &str, default: &str) -> String
```

Gets an optional environment variable with a default value.

# Examples

```rust
use riglr_core::util::get_env_or_default;

# unsafe { std::env::remove_var("OPTIONAL_SETTING"); }
let setting = get_env_or_default("OPTIONAL_SETTING", "default_value");
assert_eq!(setting, "default_value");
```

---

### get_env_vars

**Source**: `src/util.rs`

```rust
pub fn get_env_vars(keys: &[&str]) -> std::collections::HashMap<String, String>
```

Gets multiple environment variables at once, returning a map.

# Examples

```rust
use riglr_core::util::get_env_vars;
use std::collections::HashMap;

# unsafe { std::env::set_var("VAR1", "value1"); }
# unsafe { std::env::set_var("VAR2", "value2"); }
let vars = get_env_vars(&["VAR1", "VAR2", "VAR3"]);
assert_eq!(vars.get("VAR1"), Some(&"value1".to_string()));
assert_eq!(vars.get("VAR3"), None);
# unsafe { std::env::remove_var("VAR1"); }
# unsafe { std::env::remove_var("VAR2"); }
```

---

### get_pubkey

**Source**: `signer/solana.rs`

```rust
pub fn get_pubkey(&self) -> solana_sdk::pubkey::Pubkey
```

Get the public key of this signer

---

### get_recent_prioritization_fees

**Source**: `transactions/solana.rs`

```rust
pub async fn get_recent_prioritization_fees(&self) -> Result<u64, ToolError>
```

Get recent prioritization fees from the network

---

### get_required_env

**Source**: `src/util.rs`

```rust
pub fn get_required_env(key: &str) -> EnvResult<String>
```

Gets a required environment variable, returning an error if not set.

This is the recommended approach for libraries, allowing the application
to decide how to handle missing configuration.

# Examples

```rust
use riglr_core::util::get_required_env;

# unsafe { std::env::set_var("MY_API_KEY", "secret123"); }
let api_key = get_required_env("MY_API_KEY").expect("MY_API_KEY must be set");
assert_eq!(api_key, "secret123");
# unsafe { std::env::remove_var("MY_API_KEY"); }
```

# Errors

Returns [`EnvError::MissingRequired`] if the environment variable is not set.

---

### get_semaphore

**Source**: `src/tool.rs`

```rust
pub fn get_semaphore(&self, resource: &str) -> Option<Arc<Semaphore>>
```

Get the semaphore for a specific resource type.

This is used internally by the [`ToolWorker`] to acquire permits
before executing tools. Returns `None` if no limit is configured
for the specified resource.

# Parameters
* `resource` - The resource identifier to look up

# Returns
* `Some(Arc<Semaphore>)` - If a limit is configured for this resource
* `None` - If no limit is configured (will use default limit)

# Examples

```rust
use riglr_core::ResourceLimits;

let limits = ResourceLimits::new()
.with_limit("test_resource", 5);

assert!(limits.get_semaphore("test_resource").is_some());
assert!(limits.get_semaphore("unknown_resource").is_none());
```

---

### increment_retry

**Source**: `src/jobs.rs`

```rust
pub fn increment_retry(&mut self)
```

Increment the retry count for this job.

This should be called by the job processing system after each
failed execution attempt. The retry count is used to determine
whether the job can be retried again.

# Examples

```rust
use riglr_core::Job;
use serde_json::json;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
let mut job = Job::new("test_tool", &json!({}), 3)?;

assert_eq!(job.retry_count, 0);
job.increment_retry();
assert_eq!(job.retry_count, 1);
job.increment_retry();
assert_eq!(job.retry_count, 2);
# Ok(())
# }
```

---

### init_env_from_file

**Source**: `src/util.rs`

```rust
pub fn init_env_from_file(path: &str) -> std::io::Result<()>
```

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

### invalid_input_string

**Source**: `src/error.rs`

```rust
pub fn invalid_input_string<S: Into<String>>(msg: S) -> Self
```

Creates an invalid input error from a string message

---

### invalid_input_with_source

**Source**: `src/error.rs`

```rust
pub fn invalid_input_with_source<E: std::error::Error + Send + Sync + 'static>( source: E, context: impl Into<String>, ) -> Self
```

Creates an invalid input error

---

### is_available

**Source**: `src/signer.rs`

```rust
pub async fn is_available() -> bool
```

Check if there is currently a signer context available.

This function returns `true` if the current async task is running within
a [`SignerContext::with_signer()`] scope, and `false` otherwise.

This is useful for tools that want to provide different behavior when called
with or without a signer context, such as read-only vs. transactional operations.

# Returns
* `true` - If a signer context is available
* `false` - If no signer context is available

# Examples

```ignore
use riglr_core::signer::SignerContext;
use riglr_solana_tools::LocalSolanaSigner;
use std::sync::Arc;
# use solana_sdk::signer::keypair::Keypair;

async fn flexible_tool() -> Result<String, riglr_core::signer::SignerError> {
if SignerContext::is_available().await {
// We have a signer, can perform transactions
let signer = SignerContext::current().await?;
Ok("Performing transaction with signer".to_string())
} else {
// No signer available, provide read-only functionality
Ok("Read-only mode: no signer available".to_string())
}
}

# async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
// Test without signer context
let result1 = flexible_tool().await.unwrap();
assert_eq!(result1, "Read-only mode: no signer available");

// Test with signer context
let keypair = Keypair::new();
let signer = Arc::new(LocalSolanaSigner::new(
keypair,
"https://api.devnet.solana.com".to_string()
));

let result2 = SignerContext::with_signer(signer, async {
flexible_tool().await
}).await.unwrap();
assert_eq!(result2, "Performing transaction with signer");

# Ok(())
# }
```

# Performance

This function is very lightweight and can be called frequently without
performance concerns. It simply checks if the thread-local storage
contains a signer reference.

---

### is_retriable

**Source**: `src/jobs.rs`

```rust
pub fn is_retriable(&self) -> bool
```

Check if this result represents a retriable failure.

This method returns `true` only for `JobResult::Failure` variants
where `retriable` is `true`. Success results always return `false`.

# Returns
* `true` - If this is a retriable failure
* `false` - If this is a success or permanent failure

# Examples

```rust
use riglr_core::JobResult;
use serde_json::json;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
let success = JobResult::success(&json!({"data": "ok"}))?;
let retriable = JobResult::retriable_failure("Network timeout");
let permanent = JobResult::permanent_failure("Invalid input");

assert!(!success.is_retriable());     // Success is not retriable
assert!(retriable.is_retriable());    // Retriable failure
assert!(!permanent.is_retriable());   // Permanent failure is not retriable
# Ok(())
# }
```

---

### is_success

**Source**: `src/jobs.rs`

```rust
pub fn is_success(&self) -> bool
```

Check if this result represents a successful execution.

# Returns
* `true` - If this is a `JobResult::Success`
* `false` - If this is a `JobResult::Failure`

# Examples

```rust
use riglr_core::JobResult;
use serde_json::json;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
let success = JobResult::success(&json!({"data": "ok"}))?;
let failure = JobResult::permanent_failure("Error occurred");

assert!(success.is_success());
assert!(!failure.is_success());
# Ok(())
# }
```

---

### metrics

**Source**: `src/tool.rs`

```rust
pub fn metrics(&self) -> &WorkerMetrics
```

Get access to worker metrics.

The returned metrics can be used for monitoring worker performance
and health. All metrics are thread-safe and can be read at any time.

# Returns
A reference to the worker's metrics

# Examples

```ignore
use riglr_core::{ToolWorker, ExecutionConfig, idempotency::InMemoryIdempotencyStore};
use std::sync::atomic::Ordering;

let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
let metrics = worker.metrics();

println!("Jobs processed: {}",
metrics.jobs_processed.load(Ordering::Relaxed));
println!("Success rate: {:.2}%", {
let processed = metrics.jobs_processed.load(Ordering::Relaxed);
let succeeded = metrics.jobs_succeeded.load(Ordering::Relaxed);
if processed > 0 { (succeeded as f64 / processed as f64) * 100.0 } else { 0.0 }
});
```

---

### new

**Source**: `src/jobs.rs`

```rust
pub fn new<T: Serialize>( tool_name: impl Into<String>, params: &T, max_retries: u32, ) -> Result<Self, serde_json::Error>
```

Create a new job without an idempotency key.

This creates a standard job that will be executed normally without
result caching. If the job fails with a retriable error, it may be
retried up to `max_retries` times.

# Parameters
* `tool_name` - Name of the tool to execute (must match a registered tool)
* `params` - Parameters to pass to the tool (will be JSON-serialized)
* `max_retries` - Maximum number of retry attempts (0 = no retries)

# Returns
* `Ok(Job)` - Successfully created job
* `Err(serde_json::Error)` - If parameters cannot be serialized to JSON

# Examples

```rust
use riglr_core::Job;
use serde_json::json;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
// Simple job with basic parameters
let job = Job::new(
"weather_check",
&json!({
"city": "San Francisco",
"units": "metric"
}),
3 // allow up to 3 retries
)?;

assert_eq!(job.tool_name, "weather_check");
assert_eq!(job.max_retries, 3);
assert_eq!(job.retry_count, 0);
assert!(job.idempotency_key.is_none());
# Ok(())
# }
```

---

### new

**Source**: `src/queue.rs`

```rust
pub fn new() -> Self
```

Create a new in-memory job queue

---

### new

**Source**: `src/queue.rs`

```rust
pub fn new(redis_url: &str, queue_name: &str) -> Result<Self>
```

Create a new Redis job queue

# Arguments
* `redis_url` - Redis connection URL (e.g., "redis://127.0.0.1:6379")
* `queue_name` - Name of the queue (will be prefixed with "riglr:queue:")

---

### new

**Source**: `src/idempotency.rs`

**Attributes**:
```rust
#[must_use]
```

```rust
pub fn new() -> Self
```

Create a new in-memory idempotency store

---

### new

**Source**: `src/idempotency.rs`

```rust
pub fn new(redis_url: &str, key_prefix: Option<&str>) -> anyhow::Result<Self>
```

Create a new Redis idempotency store

# Arguments
* `redis_url` - Redis connection URL (e.g., "redis://127.0.0.1:6379")
* `key_prefix` - Prefix for idempotency keys (default: "riglr:idempotency:")

---

### new

**Source**: `src/tool.rs`

```rust
pub fn new() -> Self
```

Create new empty resource limits configuration.

Use [`ResourceLimits::with_limit()`] to add resource limits,
or use [`ResourceLimits::default()`] for pre-configured defaults.

# Examples

```ignore
use riglr_core::ResourceLimits;

let limits = ResourceLimits::new()
.with_limit("custom_resource", 5);
```

---

### new

**Source**: `src/tool.rs`

```rust
pub fn new(config: ExecutionConfig) -> Self
```

Create a new tool worker with the given configuration.

This creates a worker ready to process jobs, but no tools are registered yet.
Use [`register_tool()`](Self::register_tool) to add tools before processing jobs.

# Parameters
* `config` - Execution configuration controlling retry behavior, timeouts, etc.

# Examples

```ignore
use riglr_core::{ToolWorker, ExecutionConfig, idempotency::InMemoryIdempotencyStore};
use std::time::Duration;

let config = ExecutionConfig {
max_concurrency: 20,
default_timeout: Duration::from_secs(60),
max_retries: 5,
..Default::default()
};

let worker = ToolWorker::<InMemoryIdempotencyStore>::new(config);
```

---

### new

**Source**: `signer/evm.rs`

```rust
pub fn new(private_key: String, network_config: EvmNetworkConfig) -> Result<Self, SignerError>
```

Create a new EVM signer from a private key and network configuration

---

### new

**Source**: `signer/solana.rs`

```rust
pub fn new( private_key: String, network_config: SolanaNetworkConfig, ) -> Result<Self, SignerError>
```

Create a new Solana signer from a base58-encoded private key and network configuration

---

### new

**Source**: `signer/granular_traits.rs`

```rust
pub fn new(inner: Arc<T>) -> Self
```

Create a new adapter wrapping the given legacy signer

---

### new

**Source**: `transactions/evm.rs`

```rust
pub fn new(provider: Arc<P>, gas_config: GasConfig) -> Self
```

Create a new EVM transaction processor

---

### new

**Source**: `transactions/solana.rs`

```rust
pub fn new(client: Arc<RpcClient>, priority_config: PriorityFeeConfig) -> Self
```

Create a new Solana transaction processor

---

### new_idempotent

**Source**: `src/jobs.rs`

```rust
pub fn new_idempotent<T: Serialize>( tool_name: impl Into<String>, params: &T, max_retries: u32, idempotency_key: impl Into<String>, ) -> Result<Self, serde_json::Error>
```

Create a new job with an idempotency key for safe retry behavior.

When an idempotency store is configured, successful results for jobs
with idempotency keys are cached. Subsequent jobs with the same key
will return the cached result instead of re-executing the tool.

# Parameters
* `tool_name` - Name of the tool to execute (must match a registered tool)
* `params` - Parameters to pass to the tool (will be JSON-serialized)
* `max_retries` - Maximum number of retry attempts (0 = no retries)
* `idempotency_key` - Unique key for this operation (should include relevant parameters)

# Returns
* `Ok(Job)` - Successfully created job
* `Err(serde_json::Error)` - If parameters cannot be serialized to JSON

# Examples

```rust
use riglr_core::Job;
use serde_json::json;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
// Idempotent job for a financial transaction
let job = Job::new_idempotent(
"transfer_funds",
&json!({
"from": "account_123",
"to": "account_456",
"amount": 100.50,
"currency": "USD"
}),
2, // allow up to 2 retries
"transfer_123_456_100.50_USD_20241201_001" // unique operation ID
)?;

assert_eq!(job.tool_name, "transfer_funds");
assert_eq!(job.max_retries, 2);
assert!(job.idempotency_key.is_some());
# Ok(())
# }
```

# Idempotency Key Best Practices

Idempotency keys should:
- Be unique per logical operation
- Include relevant parameters that affect the result
- Include timestamp or sequence numbers for time-sensitive operations
- Be deterministic for the same logical operation

Example patterns:
- `"transfer_{from}_{to}_{amount}_{currency}_{date}_{sequence}"`
- `"price_check_{symbol}_{currency}_{timestamp}"`
- `"user_action_{user_id}_{action}_{target}_{date}"`

---

### optimize_transaction

**Source**: `transactions/solana.rs`

```rust
pub fn optimize_transaction(&self, tx: &mut Transaction) -> Result<(), ToolError>
```

Optimize transaction for size and compute units

---

### permanent_failure

**Source**: `src/jobs.rs`

```rust
pub fn permanent_failure(error: impl Into<String>) -> Self
```

Create a permanent (non-retriable) failure result.

Use this for failures that won't be resolved by retrying, such as
invalid parameters, authorization failures, or business logic violations.
The worker will not retry jobs that fail with permanent errors.

# Parameters
* `error` - Human-readable error message describing the failure

# Examples

```rust
use riglr_core::JobResult;

// Invalid input parameters
let validation_error = JobResult::permanent_failure(
"Invalid email address format"
);

// Authorization issues
let auth_error = JobResult::permanent_failure(
"API key is invalid or expired"
);

// Business logic violations
let business_error = JobResult::permanent_failure(
"Insufficient funds for transfer"
);

// Invalid blockchain addresses
let address_error = JobResult::permanent_failure(
"Invalid Solana address format"
);

assert!(!validation_error.is_retriable());
assert!(!auth_error.is_retriable());
assert!(!business_error.is_retriable());
assert!(!address_error.is_retriable());
```

# When to Use Permanent Failures

- Invalid input parameters or malformed requests
- Authentication or authorization failures
- HTTP 4xx client errors (except 429 Rate Limited)
- Insufficient funds or resources
- Invalid addresses or identifiers
- Business rule violations
- Unsupported operations (HTTP 501 Not Implemented)

---

### permanent_string

**Source**: `src/error.rs`

```rust
pub fn permanent_string<S: Into<String>>(msg: S) -> Self
```

Creates a permanent error from a string message

---

### permanent_with_source

**Source**: `src/error.rs`

```rust
pub fn permanent_with_source<E: std::error::Error + Send + Sync + 'static>( source: E, context: impl Into<String>, ) -> Self
```

Creates a permanent error with context and source preservation

---

### prepare_transaction

**Source**: `transactions/evm.rs`

```rust
pub async fn prepare_transaction( &self, mut tx: TransactionRequest, ) -> Result<TransactionRequest, ToolError>
```

Prepare transaction with optimal gas settings

---

### process_job

**Source**: `src/tool.rs`

```rust
pub async fn process_job(&self, mut job: Job) -> Result<JobResult, WorkerError>
```

Process a single job with all resilience features.

Returns:
- `Ok(JobResult)` - The job was processed (successfully or with business logic failure)
- `Err(WorkerError)` - System-level worker failure (tool not found, semaphore issues, etc.)

---

### rate_limited_string

**Source**: `src/error.rs`

```rust
pub fn rate_limited_string<S: Into<String>>(msg: S) -> Self
```

Creates a rate limited error from a string message

---

### rate_limited_with_source

**Source**: `src/error.rs`

```rust
pub fn rate_limited_with_source<E: std::error::Error + Send + Sync + 'static>( source: E, context: impl Into<String>, retry_after: Option<std::time::Duration>, ) -> Self
```

Creates a rate limited error with optional retry duration

---

### register_tool

**Source**: `src/tool.rs`

```rust
pub async fn register_tool(&self, tool: Arc<dyn Tool>)
```

Register a tool with this worker.

Tools must be registered before they can be executed by jobs.
Each tool is indexed by its name, so tool names must be unique
within a single worker.

# Parameters
* `tool` - The tool implementation to register

# Panics
This method does not panic, but if a tool with the same name
is already registered, it will be replaced.

# Examples

```ignore
use riglr_core::{ToolWorker, ExecutionConfig, Tool, JobResult, idempotency::InMemoryIdempotencyStore};
use async_trait::async_trait;
use std::sync::Arc;

struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
async fn execute(&self, params: serde_json::Value) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
let a = params["a"].as_f64().unwrap_or(0.0);
let b = params["b"].as_f64().unwrap_or(0.0);
Ok(JobResult::success(&(a + b))?)
}

fn name(&self) -> &str {
"calculator"
}
}

# async fn example() {
let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
worker.register_tool(Arc::new(CalculatorTool)).await;
# }
```

---

### retriable_failure

**Source**: `src/jobs.rs`

```rust
pub fn retriable_failure(error: impl Into<String>) -> Self
```

Create a retriable failure result.

Use this for temporary failures that may resolve on retry, such as
network timeouts, rate limits, or temporary service unavailability.
The worker will attempt to retry jobs that fail with retriable errors.

# Parameters
* `error` - Human-readable error message describing the failure

# Examples

```rust
use riglr_core::JobResult;

// Network connectivity issues
let network_error = JobResult::retriable_failure(
"Connection timeout after 30 seconds"
);

// Rate limiting
let rate_limit = JobResult::retriable_failure(
"API rate limit exceeded, retry in 60 seconds"
);

// Service unavailability
let service_down = JobResult::retriable_failure(
"Service temporarily unavailable (HTTP 503)"
);

assert!(network_error.is_retriable());
assert!(rate_limit.is_retriable());
assert!(service_down.is_retriable());
```

# When to Use Retriable Failures

- Network connectivity problems (timeouts, DNS failures)
- HTTP 5xx server errors (except 501 Not Implemented)
- Rate limiting responses (HTTP 429)
- Temporary resource unavailability
- Blockchain network congestion
- Database connection issues

---

### retriable_string

**Source**: `src/error.rs`

```rust
pub fn retriable_string<S: Into<String>>(msg: S) -> Self
```

Creates a retriable error from a string message

---

### retriable_with_source

**Source**: `src/error.rs`

```rust
pub fn retriable_with_source<E: std::error::Error + Send + Sync + 'static>( source: E, context: impl Into<String>, ) -> Self
```

Creates a retriable error with context and source preservation

---

### run

**Source**: `src/tool.rs`

```rust
pub async fn run<Q: JobQueue>( &self, queue: Arc<Q>, cancellation_token: tokio_util::sync::CancellationToken, ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
```

Start the worker loop, processing jobs from the given queue.

The worker will continue processing jobs until the provided cancellation token
is cancelled. This allows for graceful shutdown where in-flight jobs can complete
before the worker stops.

# Parameters
* `queue` - The job queue to process jobs from
* `cancellation_token` - Token to signal when the worker should stop

# Examples

```rust
use riglr_core::{ToolWorker, ExecutionConfig, idempotency::InMemoryIdempotencyStore};
use riglr_core::queue::InMemoryJobQueue;
use tokio_util::sync::CancellationToken;
use std::sync::Arc;

# async fn example() -> anyhow::Result<()> {
let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
let queue = Arc::new(InMemoryJobQueue::new());
let cancellation_token = CancellationToken::new();

// Start worker in background
let token_clone = cancellation_token.clone();
let worker_handle = tokio::spawn(async move {
worker.run(queue, token_clone).await
});

// Later, signal shutdown
cancellation_token.cancel();
// Await the task and ignore the inner result for simplicity in docs
let _ = worker_handle.await;
# Ok(())
# }
```

---

### send_transaction_with_retry

**Source**: `transactions/solana.rs`

```rust
pub async fn send_transaction_with_retry( &self, tx: &Transaction, ) -> Result<Signature, ToolError>
```

Send transaction with automatic retry on blockhash expiry

---

### simulate_transaction

**Source**: `transactions/evm.rs`

```rust
pub async fn simulate_transaction(&self, tx: &TransactionRequest) -> Result<(), ToolError>
```

Simulate transaction before sending

---

### success

**Source**: `src/jobs.rs`

```rust
pub fn success<T: Serialize>(value: &T) -> Result<Self, serde_json::Error>
```

Create a successful result without a transaction hash.

This is the standard way to create success results for operations
that don't involve blockchain transactions.

# Parameters
* `value` - The result data to serialize as JSON

# Returns
* `Ok(JobResult::Success)` - Successfully created result
* `Err(serde_json::Error)` - If value cannot be serialized to JSON

# Examples

```rust
use riglr_core::JobResult;
use serde_json::json;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
// API response result
let api_result = JobResult::success(&json!({
"data": {
"price": 45000.50,
"currency": "USD",
"timestamp": 1638360000
},
"status": "success"
}))?;

assert!(api_result.is_success());
# Ok(())
# }
```

---

### success_with_tx

**Source**: `src/jobs.rs`

```rust
pub fn success_with_tx<T: Serialize>( value: &T, tx_hash: impl Into<String>, ) -> Result<Self, serde_json::Error>
```

Create a successful result with a blockchain transaction hash.

Use this for operations that involve blockchain transactions,
such as transfers, swaps, or contract interactions. The transaction
hash enables tracking and verification of the operation.

# Parameters
* `value` - The result data to serialize as JSON
* `tx_hash` - The blockchain transaction hash

# Returns
* `Ok(JobResult::Success)` - Successfully created result
* `Err(serde_json::Error)` - If value cannot be serialized to JSON

# Examples

```rust
use riglr_core::JobResult;
use serde_json::json;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
// Solana transfer result
let transfer_result = JobResult::success_with_tx(
&json!({
"from": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
"to": "7XjBz3VSM8Y6kXoHdXFRvLV8Fn6dQJgj9AJhCFJZ9L2x",
"amount": 1.5,
"currency": "SOL",
"status": "confirmed"
}),
"2Z8h8K6wF5L9X3mE4u1nV7yQ9H5pG3rD1M2cB6x8Wq7N"
)?;

// Ethereum swap result
let swap_result = JobResult::success_with_tx(
&json!({
"input_token": "USDC",
"output_token": "WETH",
"input_amount": "1000.00",
"output_amount": "0.42",
"slippage": "0.5%"
}),
"0x1234567890abcdef1234567890abcdef12345678"
)?;

assert!(transfer_result.is_success());
assert!(swap_result.is_success());
# Ok(())
# }
```

---

### validate_required_env

**Source**: `src/util.rs`

```rust
pub fn validate_required_env(keys: &[&str]) -> EnvResult<()>
```

Validates that all required environment variables are set.

This is useful during application initialization to fail fast if
configuration is incomplete.

# Examples

```rust
use riglr_core::util::validate_required_env;

# unsafe { std::env::set_var("API_KEY", "value1"); }
# unsafe { std::env::set_var("DATABASE_URL", "value2"); }
let required = vec!["API_KEY", "DATABASE_URL"];
validate_required_env(&required).expect("Missing required environment variables");
# unsafe { std::env::remove_var("API_KEY"); }
# unsafe { std::env::remove_var("DATABASE_URL"); }
```

# Errors

Returns the first [`EnvError::MissingRequired`] encountered.

---

### with_env_overrides

**Source**: `src/config.rs`

**Attributes**:
```rust
#[must_use]
```

```rust
pub fn with_env_overrides(mut self) -> Self
```

Override or extend EVM networks from env like `RPC_URL`_{`CHAIN_ID`}.
If `chain_id` exists, updates `rpc_url`; otherwise adds as "chain_{id}".

---

### with_idempotency_store

**Source**: `src/tool.rs`

```rust
pub fn with_idempotency_store(mut self, store: Arc<I>) -> Self
```

Configure an idempotency store for result caching.

When an idempotency store is configured, jobs with idempotency keys
will have their results cached. Subsequent executions with the same
idempotency key will return the cached result instead of re-executing.

# Parameters
* `store` - The idempotency store implementation to use

# Returns
Self, for method chaining

# Examples

```ignore
use riglr_core::{ToolWorker, ExecutionConfig, idempotency::InMemoryIdempotencyStore};
use std::sync::Arc;

let store = Arc::new(InMemoryIdempotencyStore::new());
let worker = ToolWorker::new(ExecutionConfig::default())
.with_idempotency_store(store);
```

---

### with_limit

**Source**: `src/tool.rs`

```rust
pub fn with_limit(mut self, resource: impl Into<String>, limit: usize) -> Self
```

Add a resource limit for the specified resource type.

This creates a semaphore that will limit concurrent access to the
specified resource. Tools with names matching the resource mapping
will be subject to this limit.

# Parameters
* `resource` - The resource identifier (e.g., "solana_rpc", "evm_rpc")
* `limit` - Maximum number of concurrent operations for this resource

# Returns
Self, for method chaining

# Examples

```ignore
use riglr_core::ResourceLimits;

let limits = ResourceLimits::new()
.with_limit("solana_rpc", 3)     // Limit Solana RPC calls
.with_limit("database", 10)      // Limit database connections
.with_limit("external_api", 5);  // Limit external API calls
```

---

### with_resource_limits

**Source**: `src/tool.rs`

```rust
pub fn with_resource_limits(mut self, limits: ResourceLimits) -> Self
```

Configure custom resource limits.

Resource limits control how many concurrent operations can run
for each resource type. This prevents overwhelming external
services and provides fine-grained control over resource usage.

# Parameters
* `limits` - The resource limits configuration to use

# Returns
Self, for method chaining

# Examples

```ignore
use riglr_core::{ToolWorker, ExecutionConfig, ResourceLimits, idempotency::InMemoryIdempotencyStore};

let limits = ResourceLimits::new()
.with_limit("solana_rpc", 3)
.with_limit("evm_rpc", 8)
.with_limit("http_api", 15);

let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default())
.with_resource_limits(limits);
```

---

### with_signer

**Source**: `src/signer.rs`

```rust
pub async fn with_signer<T, F>( signer: Arc<dyn TransactionSigner>, future: F, ) -> Result<T, SignerError> where F: std::future::Future<Output = Result<T, SignerError>> + Send,
```

Execute a future with a specific signer context.

This creates an isolated context where the provided signer is available
to all code running within the future via [`SignerContext::current()`].
The context is automatically cleaned up when the future completes.

# Arguments
* `signer` - The signer to make available in the context. This signer will be
accessible to all code executed within the future scope.
* `future` - The async code to execute with the signer context. All async
operations within this future can access the signer.

# Returns
The result of executing the future. If the future succeeds, returns its result.
If the future fails with a [`SignerError`], that error is propagated.

# Examples

```ignore
use riglr_core::signer::SignerContext;
use riglr_solana_tools::LocalSolanaSigner;
use std::sync::Arc;
# use solana_sdk::signer::keypair::Keypair;

# async fn example() -> Result<(), riglr_core::signer::SignerError> {
let keypair = Keypair::new();
let signer = Arc::new(LocalSolanaSigner::new(
keypair,
"https://api.devnet.solana.com".to_string()
));

let result = SignerContext::with_signer(signer, async {
// Multiple operations can access the same signer
let current = SignerContext::current().await?;
let user_id = current.user_id();

// Call other async functions that need the signer
perform_blockchain_operation().await?;

Ok(format!("Completed operations for user: {:?}", user_id))
}).await?;

println!("{}", result);
# Ok(())
# }

async fn perform_blockchain_operation() -> Result<(), riglr_core::signer::SignerError> {
// This function can access the signer context
let signer = SignerContext::current().await?;
// ... perform blockchain operations
Ok(())
}
```

# Thread Safety

Multiple tasks can run concurrently with different signer contexts:

```ignore
use riglr_core::signer::SignerContext;
use riglr_solana_tools::LocalSolanaSigner;
use std::sync::Arc;
# use solana_sdk::signer::keypair::Keypair;

# async fn concurrent_example() -> Result<(), riglr_core::signer::SignerError> {
let signer1 = Arc::new(LocalSolanaSigner::new(
Keypair::new(),
"https://api.devnet.solana.com".to_string()
));
let signer2 = Arc::new(LocalSolanaSigner::new(
Keypair::new(),
"https://api.devnet.solana.com".to_string()
));

let (result1, result2) = tokio::join!(
SignerContext::with_signer(signer1, async {
// This task has access to signer1
Ok("Task 1 completed")
}),
SignerContext::with_signer(signer2, async {
// This task has access to signer2 (isolated from signer1)
Ok("Task 2 completed")
})
);

println!("{}, {}", result1?, result2?);
# Ok(())
# }
```

---

### with_timeout

**Source**: `src/queue.rs`

```rust
pub fn with_timeout(mut self, timeout_seconds: u64) -> Self
```

Set the blocking timeout for dequeue operations

---

### with_unified_signer

**Source**: `src/signer.rs`

```rust
pub async fn with_unified_signer<T, F>( signer: Arc<dyn UnifiedSigner>, future: F, ) -> Result<T, SignerError> where F: std::future::Future<Output = Result<T, SignerError>> + Send,
```

Execute a future with a unified signer context.

Similar to `with_signer` but uses the new granular trait system.
This is the preferred method for new code.

# Examples
```ignore
use riglr_core::signer::{SignerContext, SolanaSigner};

let result = SignerContext::with_unified_signer(signer, async {
// Access as specific signer type
let solana_signer = SignerContext::current_as::<dyn SolanaSigner>().await?;
// Use Solana-specific methods
let pubkey = solana_signer.pubkey();
Ok(())
}).await?;
```

---

## Enums

### Chain

**Source**: `signer/granular_traits.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
```

```rust
pub enum Chain { /// Solana blockchain Solana, /// EVM-compatible blockchain with the specified chain ID Evm { /// The numeric chain identifier for this EVM network chain_id: u64 }, }
```

Enum representing supported blockchain types

**Variants**:

- `Solana`
- `Evm`
- `chain_id`

---

### CoreError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum CoreError { /// Queue operation failed #[error("Queue error: {0}")] Queue(String), /// Job execution failed #[error("Job execution error: {0}")] JobExecution(String), /// Serialization/deserialization failed #[error("Serialization error: {0}")] Serialization(#[from] serde_json::Error), /// Redis connection error (only available with redis feature) #[cfg(feature = "redis")] #[error("Redis error: {0}")] Redis(#[from] redis::RedisError), /// Generic error #[error("Core error: {0}")] Generic(String), }
```

Main error type for riglr-core operations.

**Variants**:

- `Queue(String)`
- `JobExecution(String)`
- `Serialization(#[from] serde_json::Error)`
- `Redis(#[from] redis::RedisError)`
- `Generic(String)`

---

### EnvError

**Source**: `src/util.rs`

**Attributes**:
```rust
#[derive(Debug, thiserror::Error)]
```

```rust
pub enum EnvError { /// Required environment variable is not set #[error("Environment variable '{0}' is required but not set")] MissingRequired(String), /// Environment variable contains invalid UTF-8 #[error("Environment variable '{0}' contains invalid UTF-8")] InvalidUtf8(String), }
```

Error type for environment variable operations

**Variants**:

- `MissingRequired(String)`
- `InvalidUtf8(String)`

---

### JobResult

**Source**: `src/jobs.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub enum JobResult { /// Job executed successfully and produced a result. /// /// This variant indicates that the tool completed its work successfully /// and returned data. The result may optionally include a transaction /// hash for blockchain operations. Success { /// The result data produced by the tool execution. /// /// This contains the actual output of the tool, serialized as JSON. /// The structure of this value depends on the specific tool that /// was executed. value: serde_json::Value, /// Optional transaction hash for blockchain operations. /// /// When the tool performs a blockchain transaction (transfer, swap, etc.), /// this field should contain the transaction hash for tracking and /// verification purposes. For non-blockchain operations, this is typically `None`. tx_hash: Option<String>, }, /// Job execution failed with error details and retry classification. /// /// This variant indicates that the tool execution failed. The failure /// is classified as either retriable (temporary issue) or permanent /// (won't be resolved by retrying). Failure { /// Human-readable error message describing what went wrong. /// /// This should provide enough context for debugging and user feedback. /// It should be descriptive but not include sensitive information. error: String, /// Whether this failure can be resolved by retrying the operation. /// /// - `true`: Retriable failure - temporary issue that may resolve /// - `false`: Permanent failure - retrying won't help /// /// The worker uses this flag to determine whether to attempt retries. retriable: bool, }, }
```

Result of executing a job, indicating success or failure with classification.

The `JobResult` enum provides structured representation of job execution outcomes,
enabling the system to make intelligent decisions about error handling and retry logic.

## Success vs Failure

- **Success**: The tool executed successfully and produced a result
- **Failure**: The tool execution failed, with classification for retry behavior

## Failure Classification

Failed executions are classified as either retriable or permanent:

- **Retriable failures**: Temporary issues that may resolve on retry
- Network timeouts, connection errors
- Rate limiting from external APIs
- Temporary service unavailability
- Resource contention

- **Permanent failures**: Issues that won't be resolved by retrying
- Invalid parameters or malformed requests
- Authentication/authorization failures
- Insufficient funds or resources
- Business logic violations

## Examples

### Creating Success Results

```rust
use riglr_core::JobResult;
use serde_json::json;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
// Simple success result
let result = JobResult::success(&json!({
"temperature": 72,
"humidity": 65,
"condition": "sunny"
}))?;
assert!(result.is_success());

// Success result with transaction hash
let tx_result = JobResult::success_with_tx(
&json!({
"recipient": "0x123...",
"amount": "1.5",
"status": "confirmed"
}),
"0xabc123def456..."
)?;
assert!(tx_result.is_success());
# Ok(())
# }
```

### Creating Failure Results

```rust
use riglr_core::JobResult;

// Retriable failure - network issues
let network_error = JobResult::retriable_failure(
"Connection timeout after 30 seconds"
);
assert!(network_error.is_retriable());
assert!(!network_error.is_success());

// Permanent failure - bad input
let validation_error = JobResult::permanent_failure(
"Invalid email address format"
);
assert!(!validation_error.is_retriable());
assert!(!validation_error.is_success());
```

### Pattern Matching

```rust
use riglr_core::JobResult;

fn handle_result(result: JobResult) {
match result {
JobResult::Success { value, tx_hash } => {
println!("Success: {:?}", value);
if let Some(hash) = tx_hash {
println!("Transaction: {}", hash);
}
}
JobResult::Failure { error, retriable } => {
if retriable {
println!("Temporary failure (will retry): {}", error);
} else {
println!("Permanent failure (won't retry): {}", error);
}
}
}
}
```

**Variants**:

- `Success`
- `value`
- `tx_hash`
- `Failure`
- `error`
- `retriable`

---

### SignerError

**Source**: `signer/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum SignerError { /// No signer context is currently available #[error("No signer context available - must be called within SignerContext::with_signer")] NoSignerContext, /// Solana blockchain transaction error #[error("Solana transaction error: {0}")] SolanaTransaction(#[from] Box<solana_client::client_error::ClientError>), /// EVM blockchain transaction error #[error("EVM transaction error: {0}")] EvmTransaction(String), /// Invalid signer configuration #[error("Invalid configuration: {0}")] Configuration(String), /// Transaction signing operation failed #[error("Signing error: {0}")] Signing(String), /// Blockchain client creation failed #[error("Client creation error: {0}")] ClientCreation(String), /// Private key format or content is invalid #[error("Invalid private key: {0}")] InvalidPrivateKey(String), /// Blockchain provider error #[error("Provider error: {0}")] ProviderError(String), /// Transaction execution failed on blockchain #[error("Transaction failed: {0}")] TransactionFailed(String), /// Operation not supported by current signer #[error("Unsupported operation: {0}")] UnsupportedOperation(String), /// Blockchain hash retrieval or validation error #[error("Blockhash error: {0}")] BlockhashError(String), /// Network not supported by signer #[error("Unsupported network: {0}")] UnsupportedNetwork(String), /// RPC URL format is invalid or unreachable #[error("Invalid RPC URL: {0}")] InvalidRpcUrl(String), }
```

Error types for signer operations

**Variants**:

- `NoSignerContext`
- `SolanaTransaction(#[from] Box<solana_client::client_error::ClientError>)`
- `EvmTransaction(String)`
- `Configuration(String)`
- `Signing(String)`
- `ClientCreation(String)`
- `InvalidPrivateKey(String)`
- `ProviderError(String)`
- `TransactionFailed(String)`
- `UnsupportedOperation(String)`
- `BlockhashError(String)`
- `UnsupportedNetwork(String)`
- `InvalidRpcUrl(String)`

---

### ToolError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum ToolError { /// Operation can be retried #[error("Operation can be retried")] Retriable { /// The underlying error that occurred #[source] source: Box<dyn std::error::Error + Send + Sync>, /// Additional context about the error context: String, }, /// Rate limited, retry after delay #[error("Rate limited, retry after delay")] RateLimited { /// The underlying error that occurred #[source] source: Box<dyn std::error::Error + Send + Sync>, /// Additional context about the rate limiting context: String, /// Optional duration to wait before retrying retry_after: Option<std::time::Duration>, }, /// Permanent error, do not retry #[error("Permanent error, do not retry")] Permanent { /// The underlying error that occurred #[source] source: Box<dyn std::error::Error + Send + Sync>, /// Additional context about the permanent error context: String, }, /// Invalid input provided #[error("Invalid input provided: {context}")] InvalidInput { /// The underlying error that occurred due to invalid input #[source] source: Box<dyn std::error::Error + Send + Sync>, /// Description of what input was invalid context: String, }, /// Signer context error #[error("Signer context error")] SignerContext(#[from] crate::signer::SignerError), }
```

Tool-specific error type for distinguishing retriable vs permanent failures.

**Variants**:

- `Retriable`
- `source`
- `context`
- `RateLimited`
- `source`
- `context`
- `retry_after`
- `Permanent`
- `source`
- `context`
- `InvalidInput`
- `source`
- `context`
- `SignerContext(#[from] crate::signer::SignerError)`

---

### TransactionStatus

**Source**: `transactions/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub enum TransactionStatus { /// Transaction is pending submission Pending, /// Transaction has been submitted to the network Submitted { /// Transaction hash from the network hash: String }, /// Transaction is being confirmed Confirming { /// Transaction hash from the network hash: String, /// Current number of confirmations received confirmations: u64 }, /// Transaction has been confirmed Confirmed { /// Transaction hash from the network hash: String, /// Block number where the transaction was included block: u64 }, /// Transaction failed Failed { /// Reason why the transaction failed reason: String }, }
```

Transaction status tracking

**Variants**:

- `Pending`
- `Submitted`
- `hash`
- `Confirming`
- `hash`
- `confirmations`
- `Confirmed`
- `hash`
- `block`
- `Failed`
- `reason`

---

### WorkerError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum WorkerError { /// Tool not found in the worker's registry #[error("Tool '{tool_name}' not found in worker registry")] ToolNotFound { /// Name of the tool that was not found tool_name: String }, /// Failed to acquire semaphore for concurrency control #[error("Failed to acquire semaphore for tool '{tool_name}': {source}")] SemaphoreAcquisition { /// Name of the tool for which semaphore acquisition failed tool_name: String, /// The underlying error that caused the semaphore acquisition failure #[source] source: Box<dyn std::error::Error + Send + Sync>, }, /// Idempotency store operation failed #[error("Idempotency store operation failed: {source}")] IdempotencyStore { /// The underlying error that caused the idempotency store operation to fail #[source] source: Box<dyn std::error::Error + Send + Sync>, }, /// Job serialization/deserialization error #[error("Job serialization error: {source}")] JobSerialization { /// The underlying JSON serialization error #[source] source: serde_json::Error, }, /// Tool execution exceeded configured timeout #[error("Tool execution timed out after {timeout:?}")] ExecutionTimeout { /// The duration after which the execution timed out timeout: std::time::Duration }, /// Internal worker system error #[error("Internal worker error: {message}")] Internal { /// Human-readable description of the internal error message: String }, }
```

Worker-specific error type for distinguishing system-level worker failures
from tool execution failures.

**Variants**:

- `ToolNotFound`
- `tool_name`
- `SemaphoreAcquisition`
- `tool_name`
- `source`
- `IdempotencyStore`
- `source`
- `JobSerialization`
- `source`
- `ExecutionTimeout`
- `timeout`
- `Internal`
- `message`

---

## Traits

### EvmClient

**Source**: `signer/traits.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait EvmClient: Send + Sync { ... }
```

Type-safe EVM client interface

**Methods**:

#### `get_balance`

```rust
async fn get_balance(&self, address: &str) -> Result<U256, SignerError>;
```

#### `send_transaction`

```rust
async fn send_transaction(&self, tx: &TransactionRequest) -> Result<TxHash, SignerError>;
```

#### `call`

```rust
async fn call(&self, tx: &TransactionRequest) -> Result<Bytes, SignerError>;
```

---

### EvmSigner

**Source**: `signer/granular_traits.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait EvmSigner: SignerBase { ... }
```

Trait for EVM-specific signing capabilities

**Methods**:

#### `chain_id`

```rust
fn chain_id(&self) -> u64;
```

#### `address`

```rust
fn address(&self) -> String;
```

#### `sign_and_send_transaction`

```rust
async fn sign_and_send_transaction( &self, tx: TransactionRequest, ) -> Result<String, SignerError>;
```

#### `sign_and_send_with_retry`

```rust
async fn sign_and_send_with_retry( &self, tx: TransactionRequest, ) -> Result<String, SignerError> {;
```

#### `client`

```rust
fn client(&self) -> Result<Arc<dyn super::traits::EvmClient>, SignerError>;
```

---

### IdempotencyStore

**Source**: `src/idempotency.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait IdempotencyStore: Send + Sync { ... }
```

Trait for idempotency store implementations

**Methods**:

#### `get`

```rust
async fn get(&self, key: &str) -> anyhow::Result<Option<JobResult>>;
```

#### `set`

```rust
async fn set(&self, key: &str, result: &JobResult, ttl: Duration) -> anyhow::Result<()>;
```

#### `remove`

```rust
async fn remove(&self, key: &str) -> anyhow::Result<()>;
```

---

### JobQueue

**Source**: `src/queue.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait JobQueue: Send + Sync { ... }
```

Trait for job queue implementations.

Provides a common interface for different queue backends, enabling
both local development with in-memory queues and production deployment
with distributed Redis queues.

**Methods**:

#### `enqueue`

```rust
async fn enqueue(&self, job: Job) -> Result<()>;
```

#### `dequeue`

```rust
async fn dequeue(&self) -> Result<Option<Job>>;
```

#### `dequeue_with_timeout`

```rust
async fn dequeue_with_timeout(&self, timeout: Duration) -> Result<Option<Job>>;
```

#### `len`

```rust
async fn len(&self) -> Result<usize>;
```

#### `is_empty`

```rust
async fn is_empty(&self) -> Result<bool> {
```

---

### MultiChainSigner

**Source**: `signer/granular_traits.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait MultiChainSigner: SignerBase { ... }
```

Trait for signers that support multiple chains
This combines the capabilities of both Solana and EVM signers

**Methods**:

#### `primary_chain`

```rust
fn primary_chain(&self) -> Chain;
```

#### `supports_chain`

```rust
fn supports_chain(&self, chain: &Chain) -> bool;
```

#### `as_solana`

```rust
fn as_solana(&self) -> Option<&dyn SolanaSigner>;
```

#### `as_evm`

```rust
fn as_evm(&self) -> Option<&dyn EvmSigner>;
```

---

### SignerBase

**Source**: `signer/granular_traits.rs`

```rust
pub trait SignerBase: Send + Sync + std::fmt::Debug + Any { ... }
```

Base trait for all signers providing common metadata and type erasure

**Methods**:

#### `locale`

```rust
fn locale(&self) -> String {
```

#### `user_id`

```rust
fn user_id(&self) -> Option<String> {
```

#### `as_any`

```rust
fn as_any(&self) -> &dyn Any;
```

---

### SolanaClient

**Source**: `signer/traits.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait SolanaClient: Send + Sync { ... }
```

Type-safe Solana client interface

**Methods**:

#### `get_balance`

```rust
async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64, SignerError>;
```

#### `send_transaction`

```rust
async fn send_transaction(&self, tx: &Transaction) -> Result<Signature, SignerError>;
```

---

### SolanaSigner

**Source**: `signer/granular_traits.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait SolanaSigner: SignerBase { ... }
```

Trait for Solana-specific signing capabilities

**Methods**:

#### `address`

```rust
fn address(&self) -> String;
```

#### `pubkey`

```rust
fn pubkey(&self) -> String;
```

#### `sign_and_send_transaction`

```rust
async fn sign_and_send_transaction(&self, tx: &mut Transaction) -> Result<String, SignerError>;
```

#### `sign_and_send_with_retry`

```rust
async fn sign_and_send_with_retry(&self, tx: &mut Transaction) -> Result<String, SignerError> {
```

#### `client`

```rust
fn client(&self) -> Arc<solana_client::rpc_client::RpcClient>;
```

---

### Tool

**Source**: `src/tool.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait Tool: Send + Sync { ... }
```

A trait defining the execution interface for tools.

This is compatible with `rig::Tool` and provides the foundation
for executing tools within the riglr ecosystem. Tools represent
individual operations that can be executed by the [`ToolWorker`].

## Design Principles

- **Stateless**: Tools should not maintain internal state between executions
- **Idempotent**: When possible, tools should be safe to retry
- **Error-aware**: Tools should classify errors as retriable or permanent
- **Resource-conscious**: Tools should handle rate limits and timeouts gracefully

## Error Handling

Tools should carefully distinguish between different types of errors:

- **Retriable errors**: Network timeouts, rate limits, temporary service unavailability
- **Permanent errors**: Invalid parameters, insufficient funds, authorization failures
- **System errors**: Internal configuration issues, unexpected state

## Examples

### Basic Tool Implementation

```ignore
use riglr_core::{Tool, JobResult};
use async_trait::async_trait;
use serde_json::Value;

struct HttpFetcher;

#[async_trait]
impl Tool for HttpFetcher {
async fn execute(&self, params: Value) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
let url = params["url"].as_str()
.ok_or("Missing required parameter: url")?;

// Mock HTTP client behavior for this example
if url.starts_with("https://") {
let body = r#"{"data": "success"}"#;
Ok(JobResult::success(&serde_json::json!({"body": body}))?)
} else if url.contains("error") {
// Simulate client error
Ok(JobResult::permanent_failure("Client error: Invalid URL"))
} else if url.contains("timeout") {
// Simulate timeout
Ok(JobResult::retriable_failure("Request timeout: Connection timed out"))
} else {
// Simulate server error
Ok(JobResult::retriable_failure("Server error: HTTP 503"))
}
}

fn name(&self) -> &str {
"http_fetcher"
}
}
```

### Tool with Resource-Aware Naming

```ignore
use riglr_core::{Tool, JobResult};
use async_trait::async_trait;
use serde_json::Value;

// The name starts with "solana_" which will use the solana_rpc resource limit
struct SolanaBalanceChecker;

#[async_trait]
impl Tool for SolanaBalanceChecker {
async fn execute(&self, params: Value) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
let address = params["address"].as_str()
.ok_or("Missing required parameter: address")?;

// This tool will be rate-limited by the "solana_rpc" resource limit
// because its name starts with "solana_"

// Implementation would use Solana RPC client here...
let balance = 1.5; // Mock balance

Ok(JobResult::success(&serde_json::json!({
"address": address,
"balance": balance,
"unit": "SOL"
}))?)
}

fn name(&self) -> &str {
"solana_balance_check"  // Name prefix determines resource pool
}
}
```

### Tool Using Signer Context

```ignore
use riglr_core::{Tool, JobResult, SignerContext};
use async_trait::async_trait;
use serde_json::Value;

struct TransferTool;

#[async_trait]
impl Tool for TransferTool {
async fn execute(&self, params: Value) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
// Check if we have a signer context
if !SignerContext::is_available().await {
return Ok(JobResult::permanent_failure(
"Transfer operations require a signer context"
));
}

let signer = SignerContext::current().await
.map_err(|_| "Failed to get signer context")?;

let recipient = params["recipient"].as_str()
.ok_or("Missing required parameter: recipient")?;
let amount = params["amount"].as_f64()
.ok_or("Missing required parameter: amount")?;

// Use the signer to perform the transfer...
// This is just a mock implementation
Ok(JobResult::success_with_tx(
&serde_json::json!({
"recipient": recipient,
"amount": amount,
"status": "completed"
}),
"mock_tx_hash_12345"
)?)
}

fn name(&self) -> &str {
"transfer"
}
}
```

**Methods**:

#### `execute`

```rust
///     async fn execute(&self, params: Value) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
```

#### `name`

```rust
///     fn name(&self) -> &str {
```

#### `execute`

```rust
async fn execute( &self, params: serde_json::Value, ) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>>;
```

#### `execute`

```rust
/// #     async fn execute(&self, _: serde_json::Value) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
```

#### `name`

```rust
/// fn name(&self) -> &str {
```

#### `name`

```rust
fn name(&self) -> &str;
```

#### `description`

```rust
fn description(&self) -> &str;
```

---

### TransactionProcessor

**Source**: `transactions/mod.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait TransactionProcessor: Send + Sync { ... }
```

Trait for transaction processors

**Methods**:

#### `process_with_retry`

```rust
async fn process_with_retry<T, F, Fut>( &self, operation: F, config: RetryConfig, ) -> Result<T, crate::error::ToolError> where T: Send, F: Fn() -> Fut + Send, Fut: std::future::Future<Output = Result<T, crate::error::ToolError>> + Send;
```

#### `get_status`

```rust
async fn get_status(&self, tx_hash: &str) -> Result<TransactionStatus, crate::error::ToolError>;
```

#### `wait_for_confirmation`

```rust
async fn wait_for_confirmation( &self, tx_hash: &str, required_confirmations: u64, ) -> Result<TransactionStatus, crate::error::ToolError>;
```

---

### TransactionSigner

**Source**: `signer/traits.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait TransactionSigner: Send + Sync + std::fmt::Debug { ... }
```

A trait for transaction signing across multiple blockchain networks.
This trait provides a unified interface for signing transactions on different chains
while maintaining secure context isolation.

**Methods**:

#### `locale`

```rust
fn locale(&self) -> String {
```

#### `user_id`

```rust
fn user_id(&self) -> Option<String> {
```

#### `chain_id`

```rust
fn chain_id(&self) -> Option<u64> {
```

#### `address`

```rust
fn address(&self) -> Option<String> {
```

#### `pubkey`

```rust
fn pubkey(&self) -> Option<String> {
```

#### `sign_and_send_solana_transaction`

```rust
async fn sign_and_send_solana_transaction( &self, tx: &mut solana_sdk::transaction::Transaction, ) -> Result<String, SignerError>;
```

#### `sign_and_send_evm_transaction`

```rust
async fn sign_and_send_evm_transaction( &self, tx: alloy::rpc::types::TransactionRequest, ) -> Result<String, SignerError>;
```

#### `sign_and_send_solana_with_retry`

```rust
async fn sign_and_send_solana_with_retry( &self, tx: &mut solana_sdk::transaction::Transaction, ) -> Result<String, SignerError> {;
```

#### `sign_and_send_evm_with_retry`

```rust
async fn sign_and_send_evm_with_retry( &self, tx: alloy::rpc::types::TransactionRequest, ) -> Result<String, SignerError> {;
```

#### `solana_client`

```rust
fn solana_client(&self) -> Option<Arc<solana_client::rpc_client::RpcClient>>;
```

#### `evm_client`

```rust
fn evm_client(&self) -> Result<Arc<dyn EvmClient>, SignerError>;
```

---

### UnifiedSigner

**Source**: `signer/granular_traits.rs`

```rust
pub trait UnifiedSigner: SignerBase { ... }
```

Unified signer trait that can represent any type of signer
This is used as the primary trait for SignerContext

**Methods**:

#### `supports_solana`

```rust
fn supports_solana(&self) -> bool;
```

#### `supports_evm`

```rust
fn supports_evm(&self) -> bool;
```

#### `as_solana`

```rust
fn as_solana(&self) -> Option<&dyn SolanaSigner>;
```

#### `as_evm`

```rust
fn as_evm(&self) -> Option<&dyn EvmSigner>;
```

#### `as_multi_chain`

```rust
fn as_multi_chain(&self) -> Option<&dyn MultiChainSigner>;
```

---


---

*This documentation was automatically generated from the source code.*