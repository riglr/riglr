# The SignerContext Pattern

The `SignerContext` is a core architectural pattern in riglr that provides secure, thread-local access to cryptographic signers. It enables tools to perform blockchain operations without directly handling private keys, ensuring security and isolation.

## The Problem: Managing Keys in Concurrent Systems

In a multi-tenant or concurrent application (like a web server handling multiple user requests), managing who is allowed to sign which transaction is critical. Passing signer objects through every function call is cumbersome and error-prone. Global signers are a security nightmare.

## The Solution: Thread-Local Context

The `SignerContext` solves this by using `tokio::task_local!` to store the current signer for a specific asynchronous task.

- **Isolation:** Each async task (e.g., an API request) gets its own isolated context. One task cannot access another's signer.
- **Stateless Tools:** Tools don't need a `signer` parameter. They can simply request the current signer from the context.
- **Security:** Private keys are confined to the context scope and are automatically cleaned up when the task completes.

## How It Works

### Setting the Context

You wrap your operation in a `SignerContext::with_signer` block. This is typically done at the entry point of a request or a job.

```rust
use riglr_core::signer::SignerContext;
use riglr_solana_tools::LocalSolanaSigner;
use std::sync::Arc;

let keypair = Keypair::new();
let signer = Arc::new(LocalSolanaSigner::new(
    keypair,
    "https://api.devnet.solana.com".to_string()
));

// All code within this block has access to `signer`
SignerContext::with_signer(signer, async {
    // ... call your tools here ...
    let balance = my_balance_tool().await?;
    Ok(())
}).await?;
```

### Accessing the Signer in a Tool

Inside a tool, you can retrieve the current signer with one line.

```rust
use riglr_core::signer::SignerContext;
use riglr_core::ToolError;

#[tool]
async fn my_balance_tool() -> Result<u64, ToolError> {
    // Retrieve the signer for the current task
    let signer = SignerContext::current().await?;
    
    // Use the signer to interact with the blockchain
    let client = signer.solana_client();
    let pubkey_str = signer.pubkey()
        .ok_or_else(|| ToolError::permanent("Signer has no pubkey"))?;
    let pubkey = pubkey_str.parse().unwrap();
    
    let balance = client.get_balance(&pubkey)
        .map_err(|e| ToolError::retriable(e.to_string()))?;
        
    Ok(balance)
}
```

## Real-World Scenarios

### Web Server Handling Multiple Users

In a web server, each HTTP request might belong to a different user with different signing keys:

```rust
async fn handle_request(req: Request) -> Response {
    // Extract user ID from request
    let user_id = extract_user_id(&req);
    
    // Load user's signer
    let signer = load_user_signer(user_id).await?;
    
    // Process request with user's signer
    SignerContext::with_signer(signer, async {
        // All tools called here will use this user's signer
        let result = process_user_action(req.body).await?;
        Ok(Response::new(result))
    }).await
}
```

### Job Queue Processing

When processing background jobs, each job gets its own signer:

```rust
async fn process_job(job: Job) {
    let signer = load_job_signer(&job).await?;
    
    SignerContext::with_signer(signer, async {
        match job.task {
            Task::SwapTokens { from, to, amount } => {
                swap_tokens(from, to, amount).await?;
            },
            Task::CheckBalance { token } => {
                let balance = get_token_balance(token).await?;
                log::info!("Balance: {}", balance);
            }
        }
        Ok(())
    }).await?;
}
```

## Multi-Chain Support

The SignerContext pattern extends naturally to multi-chain scenarios:

```rust
use riglr_core::signer::MultiChainSigner;

let multi_signer = MultiChainSigner::new()
    .with_solana(solana_signer)
    .with_ethereum(ethereum_signer)
    .with_arbitrum(arbitrum_signer);

SignerContext::with_signer(multi_signer, async {
    // Tools automatically use the appropriate chain's signer
    let sol_balance = get_sol_balance().await?;
    let eth_balance = get_eth_balance().await?;
    
    // Cross-chain operations work seamlessly
    bridge_tokens("USDC", Chain::Ethereum, Chain::Solana, 100.0).await?;
    
    Ok(())
}).await?;
```

## Security Benefits

### 1. No Key Leakage

Keys never leave the context scope:

```rust
async fn outer_function() {
    let signer = load_signer();
    
    SignerContext::with_signer(signer.clone(), async {
        inner_function().await; // Has access
    }).await;
    
    // After context exits, signer is no longer accessible
    inner_function().await; // ERROR: No signer in context
}
```

### 2. Request Isolation

Different requests can't access each other's signers:

```rust
// These run concurrently but are completely isolated
tokio::join!(
    handle_user_request(user1_signer, request1),
    handle_user_request(user2_signer, request2),
);
```

### 3. Automatic Cleanup

When the context exits (normally or via error), the signer is automatically removed:

```rust
SignerContext::with_signer(signer, async {
    risky_operation().await?; // If this fails...
    Ok(())
}).await; // Signer is still cleaned up
```

## Testing with SignerContext

The pattern makes testing straightforward:

```rust
#[tokio::test]
async fn test_swap_tokens() {
    let mock_signer = create_mock_signer();
    
    let result = SignerContext::with_signer(mock_signer, async {
        swap_tokens("SOL", "USDC", 1.0).await
    }).await;
    
    assert!(result.is_ok());
}
```

## Best Practices

### DO

- ✅ Set context at the entry point (API handler, job processor)
- ✅ Use `SignerContext::current()` in tools
- ✅ Handle the `NoSignerInContext` error gracefully
- ✅ Use different signers for different users/tasks

### DON'T

- ❌ Pass signers as function parameters when using riglr tools
- ❌ Store signers in global variables
- ❌ Try to access signers outside their context
- ❌ Share signers between unrelated tasks

## Advanced Patterns

### Nested Contexts

You can temporarily override the signer for specific operations:

```rust
SignerContext::with_signer(main_signer, async {
    // Use main signer
    let balance = get_balance().await?;
    
    // Temporarily use a different signer
    SignerContext::with_signer(special_signer, async {
        perform_special_operation().await?;
        Ok(())
    }).await?;
    
    // Back to main signer
    swap_tokens().await?;
    Ok(())
}).await?;
```

### Context Propagation

The context automatically propagates through async calls:

```rust
async fn complex_operation() {
    step_one().await?;   // Has access to signer
    step_two().await?;   // Also has access
    step_three().await?; // Still has access
}

SignerContext::with_signer(signer, complex_operation()).await?;
```

## Troubleshooting

### Common Error: No Signer in Context

If you see this error, ensure:
1. You've wrapped your operation in `SignerContext::with_signer`
2. You're not trying to access the signer outside the context
3. The async task hasn't been spawned without the context

### Performance Considerations

The SignerContext has minimal overhead:
- Thread-local access is O(1)
- No locks or synchronization needed
- Automatic cleanup prevents memory leaks

## Summary

The SignerContext pattern is fundamental to riglr's security and developer experience. It provides:

- **Security**: Isolated, scoped access to signing keys
- **Simplicity**: Tools don't need signer parameters
- **Flexibility**: Easy multi-chain and multi-tenant support
- **Safety**: Automatic cleanup and error handling

This pattern enables you to build secure, production-ready blockchain agents without the complexity of manual key management.