# Signer Context & Application Context

riglr uses two distinct context patterns to ensure clean separation of concerns and maintain security boundaries in multi-tenant environments.

> **Related Documentation:**
> - [Core Patterns](./architecture/core-patterns.md) - Comprehensive guide to riglr's architectural patterns
> - [Configuration](./configuration.md) - How ApplicationContext integrates with configuration
> - [Security Best Practices](./security-best-practices.md) - Security implications of context patterns

## Overview

The riglr architecture strictly separates:
- **ApplicationContext**: Read-only, shared dependencies (RPC clients, API keys)
- **SignerContext**: Secure, thread-local signer management for write operations

This separation ensures that:
1. Read operations can share resources efficiently
2. Write operations are securely isolated per user/tenant
3. Private keys are never passed as parameters
4. Multi-tenant safety is guaranteed

## ApplicationContext: Read-Only Dependencies

The `ApplicationContext` provides centralized dependency injection for shared, read-only resources.

### Purpose

- **Holds**: RPC clients, API keys, database connections, configuration
- **Scope**: Application-wide, shared across all requests
- **Access**: Passed explicitly to tools as a parameter
- **Thread-Safety**: All resources are Arc-wrapped and immutable

### Usage

```rust
use riglr_core::provider::ApplicationContext;

// Create application context at startup
let app_context = ApplicationContext::from_env();

// Add shared dependencies
app_context.set_extension(Arc::new(solana_client));
app_context.set_extension(Arc::new(evm_provider));
app_context.set_extension(Arc::new(database_pool));

// Tools receive context as parameter
#[tool]
async fn get_sol_balance(
    address: String, 
    context: &ApplicationContext  // Explicitly passed
) -> Result<f64, ToolError> {
    // Retrieve dependencies via type-based injection
    let client = context.get_extension::<Arc<RpcClient>>()?;
    let balance = client.get_balance(&address).await?;
    Ok(balance)
}
```

### Key Points

- **No Signers**: ApplicationContext should NEVER contain signers
- **Immutable**: Once created, dependencies cannot be modified
- **Type-Safe**: Extensions are retrieved by type
- **Testable**: Easy to inject mock dependencies

## SignerContext: Secure Transaction Signing

The `SignerContext` provides secure, thread-local access to cryptographic signers.

### Purpose

- **Holds**: Cryptographic signers for transaction signing
- **Scope**: Thread-local, isolated per async task
- **Access**: Retrieved via thread-local storage
- **Security**: Keys never passed as parameters

### Usage

```rust
use riglr_core::signer_context::SignerContext;

// Each user request gets isolated signer context
SignerContext::with_signer(user_signer, async {
    // All operations in this scope use user's signer
    // No way to access other users' signers
    let result = transfer_sol(recipient, amount).await?;
    Ok(result)
}).await

// Tools access signer via thread-local storage
#[tool]
async fn transfer_sol(
    recipient: String,
    amount: f64,
    context: &ApplicationContext,  // RPC client passed as parameter
) -> Result<TransactionResult, ToolError> {
    // Get RPC client from ApplicationContext
    let client = context.get_extension::<Arc<RpcClient>>()?;
    
    // Securely retrieve signer for current context
    let signer = SignerContext::current_as_solana().await?;
    
    // Use both for transaction
    let tx = create_transfer_tx(&recipient, amount);
    let signature = signer.sign(tx).await?;
    let result = client.send_transaction(&signature).await?;
    
    Ok(TransactionResult { signature: result })
}
```

### Key Points

- **Thread-Local**: Each async task has isolated context
- **No Parameters**: Signers never passed as function parameters
- **Automatic Cleanup**: Context cleared when scope exits
- **Multi-Tenant Safe**: Complete isolation between requests

## The Correct Pattern

### ✅ Correct: Clean Separation

```rust
// Application startup
let app_context = ApplicationContext::from_env();
app_context.set_extension(Arc::new(rpc_client));  // ✅ RPC client in AppContext

// Per-request handler
async fn handle_user_request(user_id: UserId) {
    let user_signer = load_user_signer(user_id).await?;
    
    // Wrap user operations in SignerContext
    SignerContext::with_signer(user_signer, async {
        // Tools get AppContext as parameter, SignerContext via TLS
        execute_tool(tool_input, app_context).await
    }).await
}

// Tool implementation
#[tool]
async fn swap_tokens(
    input: SwapInput,
    context: &ApplicationContext,  // ✅ Read dependencies as parameter
) -> Result<SwapOutput, ToolError> {
    // Get RPC client from ApplicationContext
    let client = context.get_extension::<Arc<RpcClient>>()?;
    
    // Get signer from SignerContext (thread-local)
    let signer = SignerContext::current_as_evm().await?;  // ✅ Signer via TLS
    
    // Perform swap...
}
```

### ❌ Incorrect: Mixed Responsibilities

```rust
// DON'T DO THIS - Never put signers in ApplicationContext
let app_context = ApplicationContext::from_env();
app_context.set_extension(Arc::new(signer));  // ❌ WRONG!

// DON'T DO THIS - Never pass signers as parameters
#[tool]
async fn swap_tokens(
    input: SwapInput,
    signer: &Signer,  // ❌ WRONG! Breaks isolation
    context: &ApplicationContext,
) -> Result<SwapOutput, ToolError> {
    // This breaks multi-tenant isolation!
}

// DON'T DO THIS - Never put RPC clients in SignerContext
SignerContext::with_signer(signer, async {
    let ctx = SignerContext::current().await?;
    let client = ctx.get_rpc_client()?;  // ❌ WRONG! Not its responsibility
}).await
```

## Multi-Tenant Server Example

Here's how to implement a multi-tenant server with proper context separation:

```rust
use axum::{Router, Extension};
use riglr_core::{ApplicationContext, SignerContext};

// Shared application context (created once at startup)
let app_context = ApplicationContext::from_env();
app_context.set_extension(Arc::new(rpc_client));
app_context.set_extension(Arc::new(database_pool));

// Web server setup
let app = Router::new()
    .route("/execute", post(execute_handler))
    .layer(Extension(app_context));

async fn execute_handler(
    Extension(app_context): Extension<Arc<ApplicationContext>>,
    Json(request): Json<ExecuteRequest>,
    auth: AuthenticatedUser,  // From auth middleware
) -> Result<Json<ExecuteResponse>, Error> {
    // Load user's signer (from secure storage)
    let user_signer = load_user_signer(auth.user_id).await?;
    
    // Execute in isolated signer context
    let result = SignerContext::with_signer(user_signer, async {
        // This async block has exclusive access to user's signer
        // Other concurrent requests cannot access it
        execute_tool(request.tool, request.input, app_context).await
    }).await?;
    
    Ok(Json(result))
}
```

## Complete Example: Token Swap

This example shows proper usage of both contexts:

```rust
#[tool]
async fn swap_tokens(
    context: &ApplicationContext,  // For clients and config
    input_token: String,
    output_token: String,
    amount: f64,
) -> Result<String, ToolError> {
    // READ: Get quote from DEX (uses ApplicationContext)
    let jupiter_client = context.get_extension::<Arc<JupiterClient>>()?;
    let quote = jupiter_client.get_quote(
        &input_token,
        &output_token,
        amount
    ).await?;
    
    // READ: Check current prices (uses ApplicationContext)
    let price_client = context.get_extension::<Arc<PriceClient>>()?;
    let price = price_client.get_price(&output_token).await?;
    
    // WRITE: Get signer for transaction (uses SignerContext)
    let signer = SignerContext::current_as_solana().await?;
    
    // BUILD: Create swap transaction
    let swap_tx = jupiter_client.build_swap_transaction(
        quote,
        signer.pubkey()?
    ).await?;
    
    // SIGN: Sign the transaction (uses SignerContext)
    let signed_tx = signer.sign_transaction(swap_tx).await?;
    
    // SEND: Submit via RPC (uses ApplicationContext)
    let solana_client = context.get_extension::<Arc<SolanaClient>>()?;
    let signature = solana_client.send_transaction(&signed_tx).await?;
    
    Ok(signature.to_string())
}
```

## Testing

The separation of contexts makes testing straightforward:

### Testing with Mock Dependencies

```rust
#[cfg(test)]
mod tests {
    use riglr_core::{ApplicationContext, SignerContext};
    
    #[tokio::test]
    async fn test_read_only_operation() {
        // Read-only operations only need ApplicationContext
        let mut context = ApplicationContext::default();
        context.set_extension(Arc::new(MockRpcClient::new()));
        
        // No SignerContext needed
        let result = get_balance("address", &context).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_write_operation() {
        // Write operations need both contexts
        let mut context = ApplicationContext::default();
        context.set_extension(Arc::new(MockRpcClient::new()));
        
        let test_signer = TestSigner::new();
        
        // Execute in SignerContext
        let result = SignerContext::with_signer(test_signer, async {
            transfer_sol("recipient", 1.0, &context).await
        }).await;
        
        assert!(result.is_ok());
    }
}
```

### Testing Multi-Tenant Isolation

```rust
#[tokio::test]
async fn test_multi_tenant_isolation() {
    let context = ApplicationContext::default();
    context.set_extension(Arc::new(MockRpcClient::new()));
    
    let signer1 = TestSigner::new("user1");
    let signer2 = TestSigner::new("user2");
    
    // Run concurrent operations with different signers
    let (result1, result2) = tokio::join!(
        SignerContext::with_signer(signer1, async {
            transfer_sol("recipient1", 1.0, &context).await
        }),
        SignerContext::with_signer(signer2, async {
            transfer_sol("recipient2", 2.0, &context).await
        })
    );
    
    // Each operation used its own signer
    assert_eq!(result1?.signer_id, "user1");
    assert_eq!(result2?.signer_id, "user2");
}
```

## Security Considerations

### Private Key Management

**Never**:
- Store private keys in ApplicationContext
- Pass signers as function parameters
- Share signers between requests
- Log or serialize signers

**Always**:
- Load signers per-request from secure storage
- Use SignerContext for signer isolation
- Clear context after use (automatic with `with_signer`)
- Audit signer access patterns

### Production Deployment

For production environments:

```rust
// Use secure key management
async fn load_user_signer(user_id: UserId) -> Result<UnifiedSigner, Error> {
    // Option 1: Cloud KMS
    let key = aws_kms::get_key(&format!("user/{}", user_id)).await?;
    
    // Option 2: Hardware Security Module
    let key = hsm::get_key(user_id).await?;
    
    // Option 3: Secure database with encryption at rest
    let encrypted_key = db.get_user_key(user_id).await?;
    let key = decrypt_with_master_key(encrypted_key)?;
    
    Ok(UnifiedSigner::from_key(key))
}
```

## Common Patterns

### Pattern 1: Agent Execution

```rust
async fn execute_agent_task(
    agent: &Agent,
    task: Task,
    app_context: Arc<ApplicationContext>,
    user_id: UserId,
) -> Result<TaskResult, Error> {
    // Load user's signer
    let signer = load_user_signer(user_id).await?;
    
    // Execute agent task with isolated signer
    SignerContext::with_signer(signer, async {
        agent.execute(task, app_context).await
    }).await
}
```

### Pattern 2: Batch Operations

```rust
async fn execute_batch_operations(
    operations: Vec<Operation>,
    app_context: Arc<ApplicationContext>,
    user_id: UserId,
) -> Vec<Result<OpResult, Error>> {
    let signer = load_user_signer(user_id).await?;
    
    SignerContext::with_signer(signer, async {
        // All operations in batch use same signer
        let mut results = Vec::new();
        for op in operations {
            results.push(execute_operation(op, app_context).await);
        }
        results
    }).await
}
```

### Pattern 3: Tool Composition

```rust
#[tool]
async fn complex_operation(
    input: ComplexInput,
    context: &ApplicationContext,
) -> Result<ComplexOutput, ToolError> {
    // Get dependencies from ApplicationContext
    let db = context.get_extension::<Arc<Database>>()?;
    let cache = context.get_extension::<Arc<Cache>>()?;
    
    // Signer automatically available via SignerContext
    let step1 = simple_tool_1(input.part1, context).await?;
    let step2 = simple_tool_2(step1, context).await?;
    
    // All tools share same SignerContext
    Ok(ComplexOutput { result: step2 })
}
```

## Migration Guide

If you're migrating from an older pattern where SignerContext held RPC clients:

### Old Pattern (Incorrect)

```rust
// Old: SignerContext held everything (WRONG!)
let signer_context = SignerContext::new(signer);
signer_context.set_rpc_client(rpc_client);  // ❌ Wrong!

#[tool]
async fn my_tool() -> Result<Output, ToolError> {
    let context = SignerContext::current().await?;
    let client = context.get_rpc_client()?;  // ❌ Wrong!
    let signer = context.get_signer()?;
    // ...
}
```

### New Pattern (Correct)

```rust
// New: Clean separation
let app_context = ApplicationContext::from_env();
app_context.set_extension(Arc::new(rpc_client));  // ✅ Correct!

SignerContext::with_signer(signer, async {
    my_tool(&app_context).await
}).await

#[tool]
async fn my_tool(
    context: &ApplicationContext,  // ✅ RPC client here
) -> Result<Output, ToolError> {
    let client = context.get_extension::<Arc<RpcClient>>()?;
    let signer = SignerContext::current_as_solana().await?;  // ✅ Signer here
    // ...
}
```

## Best Practices Summary

### DO
- ✅ Pass ApplicationContext explicitly as a parameter
- ✅ Use ApplicationContext for all read operations and shared resources
- ✅ Use SignerContext only for signing operations
- ✅ Create ApplicationContext once at startup and reuse it
- ✅ Set SignerContext at request/job boundaries

### DON'T
- ❌ Put signers in ApplicationContext
- ❌ Put RPC clients in SignerContext
- ❌ Pass signers as function parameters
- ❌ Store ApplicationContext in globals (pass it explicitly)
- ❌ Mix read and write concerns in the same context

## Summary

The separation of ApplicationContext and SignerContext is fundamental to riglr's architecture:

- **ApplicationContext**: Shared, read-only dependencies passed as parameters
- **SignerContext**: Secure, thread-local signers accessed via TLS
- **Never Mix**: Keep signers out of ApplicationContext, keep RPC clients out of SignerContext
- **Multi-Tenant**: Each request gets isolated SignerContext
- **Testing**: Easy to mock both contexts independently

This pattern ensures security, testability, and scalability in production environments.