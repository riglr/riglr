# Error Handling Philosophy

riglr implements a sophisticated error handling system designed for production blockchain applications. This guide explains the philosophy, patterns, and best practices for handling errors in riglr-based agents.

## Core Principle: Retriable vs Permanent Failures

riglr distinguishes between two fundamental types of errors:

- **Retriable Errors**: Temporary failures that might succeed if retried (network issues, rate limits, temporary unavailability)
- **Permanent Errors**: Failures that will never succeed no matter how many times retried (invalid inputs, insufficient funds, malformed addresses)

This distinction is critical for building resilient agents that can recover from transient issues while failing fast on unrecoverable errors.

## The ToolError Type

All riglr tools return `Result<T, ToolError>`, where ToolError provides rich context about what went wrong:

```rust
use riglr_core::ToolError;

#[tool]
async fn swap_tokens(
    from: String,
    to: String,
    amount: f64,
) -> Result<String, ToolError> {
    match perform_swap(&from, &to, amount).await {
        Ok(tx_hash) => Ok(tx_hash),
        Err(e) if e.is_network_error() => {
            // Network errors are retriable
            Err(ToolError::retriable(format!("Network error: {}", e)))
        }
        Err(e) if e.is_insufficient_funds() => {
            // Insufficient funds is permanent for this request
            Err(ToolError::permanent(format!("Insufficient funds: {}", e)))
        }
        Err(e) => {
            // Default to permanent for unknown errors
            Err(ToolError::permanent(e.to_string()))
        }
    }
}
```

## Error Classification Guide

### Retriable Errors

These errors should use `ToolError::retriable()`:

- Network timeouts or connection failures
- Rate limiting from APIs or RPC providers
- Temporary service unavailability (503 errors)
- Transaction submission failures due to network congestion
- Nonce conflicts that can be resolved with retry
- Transient blockchain state issues

Example:
```rust
let response = client.get(&url).send().await
    .map_err(|e| ToolError::retriable(format!("HTTP request failed: {}", e)))?;
```

### Permanent Errors

These errors should use `ToolError::permanent()`:

- Invalid input parameters (malformed addresses, negative amounts)
- Insufficient balance or funds
- Invalid signatures or authentication failures
- Business logic violations (trying to swap to the same token)
- Configuration errors (missing API keys)
- Slippage tolerance exceeded (user needs to adjust parameters)

Example:
```rust
if amount <= 0.0 {
    return Err(ToolError::permanent("Amount must be positive"));
}

if !is_valid_address(&address) {
    return Err(ToolError::permanent(format!("Invalid address: {}", address)));
}
```

## Error Context and Chaining

riglr preserves error context through the entire call chain:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SwapError {
    #[error("Insufficient balance: need {required}, have {available}")]
    InsufficientBalance { required: f64, available: f64 },
    
    #[error("Slippage too high: expected {expected}%, got {actual}%")]
    SlippageTooHigh { expected: f64, actual: f64 },
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("RPC error: {0}")]
    Rpc(#[from] JsonRpcError),
}

// Convert domain errors to ToolError
impl From<SwapError> for ToolError {
    fn from(err: SwapError) -> Self {
        match err {
            SwapError::Network(_) => ToolError::retriable(err.to_string()),
            _ => ToolError::permanent(err.to_string()),
        }
    }
}
```

## Retry Strategies

riglr implements intelligent retry strategies based on error types:

### Exponential Backoff

For retriable errors, riglr uses exponential backoff with jitter:

```rust
use tokio::time::{sleep, Duration};

async fn with_retry<T, F, Fut>(
    operation: F,
    max_retries: u32,
) -> Result<T, ToolError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, ToolError>>,
{
    let mut attempt = 0;
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if !e.is_retriable() => return Err(e),
            Err(e) if attempt >= max_retries => return Err(e),
            Err(_) => {
                attempt += 1;
                let delay = Duration::from_millis(100 * 2_u64.pow(attempt));
                sleep(delay).await;
            }
        }
    }
}
```

### Circuit Breaker Pattern

For frequently failing services, implement circuit breakers:

```rust
struct CircuitBreaker {
    failure_count: AtomicU32,
    last_failure: RwLock<Option<Instant>>,
    threshold: u32,
    timeout: Duration,
}

impl CircuitBreaker {
    async fn call<T>(&self, operation: impl Future<Output = Result<T, ToolError>>) 
        -> Result<T, ToolError> {
        // Check if circuit is open
        if self.is_open().await {
            return Err(ToolError::retriable("Circuit breaker is open"));
        }
        
        // Try the operation
        match operation.await {
            Ok(result) => {
                self.reset().await;
                Ok(result)
            }
            Err(e) if e.is_retriable() => {
                self.record_failure().await;
                Err(e)
            }
            Err(e) => Err(e),
        }
    }
}
```

## Error Handling in Practice

### Input Validation

Always validate inputs before expensive operations:

```rust
#[tool]
async fn transfer_tokens(
    to: String,
    amount: f64,
    token: String,
) -> Result<String, ToolError> {
    // Validate inputs first (cheap, fails fast)
    if amount <= 0.0 {
        return Err(ToolError::permanent("Amount must be positive"));
    }
    
    let to_address = to.parse::<Pubkey>()
        .map_err(|_| ToolError::permanent(format!("Invalid address: {}", to)))?;
    
    // Only after validation, get expensive resources
    let signer = SignerContext::current().await?;
    
    // Perform the operation
    // ...
}
```

### Resource Cleanup

Use RAII patterns to ensure cleanup even on errors:

```rust
struct Transaction {
    handle: TransactionHandle,
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if !self.handle.is_confirmed() {
            log::warn!("Transaction dropped without confirmation");
            // Clean up any resources
        }
    }
}
```

### Error Aggregation

When performing multiple operations, aggregate errors intelligently:

```rust
async fn batch_operation(items: Vec<Item>) -> Result<BatchResult, ToolError> {
    let mut successes = Vec::new();
    let mut failures = Vec::new();
    
    for item in items {
        match process_item(item).await {
            Ok(result) => successes.push(result),
            Err(e) if e.is_retriable() => {
                // Retry retriable errors
                match process_item_with_retry(item).await {
                    Ok(result) => successes.push(result),
                    Err(e) => failures.push((item, e)),
                }
            }
            Err(e) => failures.push((item, e)),
        }
    }
    
    if failures.is_empty() {
        Ok(BatchResult::Success(successes))
    } else if successes.is_empty() {
        Err(ToolError::permanent(format!("All {} operations failed", failures.len())))
    } else {
        Ok(BatchResult::Partial { successes, failures })
    }
}
```

## Monitoring and Alerting

### Structured Logging

Log errors with appropriate context:

```rust
use tracing::{error, warn, info};

match operation().await {
    Ok(result) => {
        info!(
            operation = "swap_tokens",
            from = %from_token,
            to = %to_token,
            amount = %amount,
            tx_hash = %result,
            "Swap successful"
        );
    }
    Err(e) if e.is_retriable() => {
        warn!(
            operation = "swap_tokens",
            error = %e,
            "Retriable error occurred"
        );
    }
    Err(e) => {
        error!(
            operation = "swap_tokens",
            error = %e,
            "Permanent error occurred"
        );
    }
}
```

### Metrics Collection

Track error rates and types:

```rust
static OPERATION_ERRORS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "riglr_operation_errors_total",
        "Total number of operation errors",
        &["operation", "error_type"]
    ).unwrap()
});

fn record_error(operation: &str, error: &ToolError) {
    let error_type = if error.is_retriable() { "retriable" } else { "permanent" };
    OPERATION_ERRORS.with_label_values(&[operation, error_type]).inc();
}
```

## Testing Error Scenarios

### Unit Tests

Test both success and failure paths:

```rust
#[tokio::test]
async fn test_swap_insufficient_funds() {
    let mock_signer = create_mock_signer()
        .with_balance("SOL", 0.5);
    
    let result = SignerContext::with_signer(mock_signer, async {
        swap_tokens("SOL", "USDC", 1.0).await
    }).await;
    
    assert!(matches!(
        result,
        Err(ToolError::Permanent(msg)) if msg.contains("Insufficient")
    ));
}

#[tokio::test]
async fn test_swap_network_retry() {
    let mock_signer = create_mock_signer()
        .fail_n_times(2)  // Fail first 2 attempts
        .then_succeed();
    
    let result = SignerContext::with_signer(mock_signer, async {
        swap_tokens_with_retry("SOL", "USDC", 1.0).await
    }).await;
    
    assert!(result.is_ok());
}
```

### Integration Tests

Test against real services with error injection:

```rust
#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_real_rpc_failures() {
    // Use a flaky RPC endpoint to test retry logic
    let signer = create_signer_with_flaky_rpc();
    
    let result = SignerContext::with_signer(signer, async {
        get_balance().await
    }).await;
    
    // Should succeed despite flaky RPC
    assert!(result.is_ok());
}
```

## Best Practices Summary

1. **Classify errors correctly**: Use retriable for transient issues, permanent for unrecoverable ones
2. **Fail fast on validation**: Check inputs before expensive operations
3. **Preserve context**: Use error chaining to maintain full error information
4. **Implement retry logic**: Use exponential backoff for retriable errors
5. **Clean up resources**: Use RAII patterns for automatic cleanup
6. **Log appropriately**: Use structured logging with proper severity levels
7. **Monitor error rates**: Track metrics to identify systemic issues
8. **Test error paths**: Ensure your error handling works as expected

By following these patterns, you'll build resilient agents that gracefully handle the inherent unreliability of blockchain networks while failing fast on actual problems.