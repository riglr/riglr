# From rig to riglr: Architectural Enhancements

This document details how riglr transforms the `rig` AI framework into a production-ready platform for blockchain AI agents. While `rig` provides the foundational LLM-to-tool-call capabilities, riglr adds the complete infrastructure needed for real-world deployment.

## The Foundation: What rig Provides

The upstream `rig` crate is a powerful AI framework that:
- Connects LLMs to executable tools
- Manages the tool-calling pipeline
- Handles structured outputs from language models
- Provides the core "reasoning engine" for AI agents

Think of `rig` as the "brain" - it makes decisions and determines what actions to take.

## The Evolution: What riglr Adds

riglr builds upon `rig` to provide the complete "body and nervous system" that agents need to operate in the complex blockchain environment:

## 1. The SignerContext Pattern: Transactional Security

### The Problem
How can an LLM-driven agent, whose logic is inherently non-deterministic, be given the power to sign transactions without exposing private keys directly to the reasoning loop? How can we ensure multi-tenant safety in a server environment?

### The riglr Solution
`SignerContext` uses Tokio's task-local storage to scope a `UnifiedSigner` to a specific asynchronous task:

```rust
// From riglr-core/examples/multi_tenant.rs
// A single worker can process jobs for multiple users securely
let result = SignerContext::with_signer(user_signer, async {
    // Within this block, any tool called by the worker can securely
    // access the signer for this specific user
    worker.process_job(job).await
}).await;
```

**Benefits:**
- **Security**: Keys are never passed as direct parameters, only available within the scope
- **Multi-Tenancy**: Each request gets its own isolated context
- **Chain-Agnosticism**: Works with any blockchain through the `UnifiedSigner` interface

## 2. The ApplicationContext Pattern: Clean Dependency Injection

### The Problem
How can tools access shared resources like RPC clients, API keys, and database connections without hardcoding them or creating circular dependencies?

### The riglr Solution
The `ApplicationContext` holds shared, read-only resources wrapped in `Arc`:

```rust
// From riglr-core/examples/service_worker.rs
// The application creates the context and injects dependencies
let context = ApplicationContext::from_config(&config);
context.set_extension(Arc::new(solana_client));
context.set_extension(Arc::new(evm_provider));

// Tools retrieve dependencies from the context
#[tool]
async fn get_sol_balance(
    address: String, 
    context: &ApplicationContext
) -> Result<f64, ToolError> {
    let client = context.get_extension::<SolanaClient>()?;
    // ... use client
}
```

**Benefits:**
- **Decoupling**: Tools don't depend on concrete implementations
- **Testability**: Easy to inject mock clients for testing
- **Flexibility**: Add any number of clients and resources

## 3. The #[tool] Macro: Superior Developer Experience

### The Problem
Manually implementing the `Tool` trait for every function requires extensive boilerplate for parameter serialization, schema generation, and error handling.

### The riglr Solution
The `#[tool]` macro automates the entire process:

```rust
// Before (with rig alone)
struct BasicTool;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct BasicToolArgs {
    name: String,
    age: u32,
}

impl Tool for BasicTool {
    const NAME: &'static str = "basic_tool";
    type Error = ToolError;
    type Args = BasicToolArgs;
    
    async fn definition(&self, _: &CompletionModel) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "A basic tool that greets a user".to_string(),
            parameters: schemars::schema_for!(Self::Args),
        }
    }
    
    async fn call(&self, args: Self::Args) -> Result<String, Self::Error> {
        Ok(format!("Hello {}, age {}", args.name, args.age))
    }
}

// After (with riglr)
/// A basic tool that greets a user
#[tool]
async fn basic_tool(
    context: &ApplicationContext,  // Automatically injected
    name: String,                  // Becomes a parameter
    age: u32,
) -> Result<String, ToolError> {
    Ok(format!("Hello {}, age {}", name, age))
}
```

**Benefits:**
- **Reduced Boilerplate**: 80% less code to write
- **Automatic Documentation**: Doc comments become tool descriptions
- **Type Safety**: Full compile-time checking
- **Error Conversion**: Automatic conversion to `JobResult`

## 4. Two-Level Error Handling Pattern

### The Problem
Blockchain operations fail for many reasons. Some are temporary (network issues) and should be retried, while others are permanent (invalid address). Simple error systems can't distinguish between these cases.

### The riglr Solution
A sophisticated two-level error system:

1. **High-Level (`ToolError`)**: Behavioral classification
   - `Retriable`: Temporary failures that should be retried
   - `Permanent`: Failures that won't succeed on retry
   - `RateLimited`: Need to wait before retrying
   - `InvalidInput`: Bad user input

2. **Low-Level**: Chain-specific error details wrapped in high-level errors

```rust
// High-level, behavior-based error handling
match simulate_tool_operation("network_fail") {
    Err(e) if e.is_retriable() => {
        // Retry with exponential backoff
    },
    Err(e) => {
        // Don't retry - permanent failure
    },
    _ => {}
}
```

## 5. Multi-Agent Coordination

While `rig` focuses on single agents, riglr enables complex multi-agent systems:

```rust
// From riglr-agents/examples/trading_swarm.rs
// Specialized agents collaborate on complex tasks
let research_task = Task::new(TaskType::Research, ...);
let research_result = dispatcher.dispatch_task(research_task).await?;

let risk_task = Task::new(TaskType::RiskAnalysis, research_result.data());
let risk_result = dispatcher.dispatch_task(risk_task).await?;

if risk_result.is_approved() {
    let execution_task = Task::new(TaskType::Trading, ...);
    dispatcher.dispatch_task(execution_task).await?;
}
```

## 6. Real-Time Event Streaming

Transform agents from reactive to proactive with real-time capabilities:

```rust
// From riglr-streams/examples/stream_composition.rs
let solana_stream = SolanaGeyserStream::new(...);
let processed_stream = solana_stream
    .filter(|event| matches!(event.inner.kind(), EventKind::Swap))
    .map(extract_swap_metrics)
    .throttle(Duration::from_secs(10))
    .batch(50, Duration::from_secs(60));
```

## 7. Production-Ready Infrastructure

riglr provides everything needed for production deployment:

- **Unified Configuration**: Centralized, validated settings
- **Turnkey Server**: Pre-configured HTTP servers with auth and metrics
- **Authentication**: Official integrations with Privy, Web3Auth, Magic.link
- **Deployment Templates**: Docker, Kubernetes, Fly.io configurations

## Summary

riglr takes the powerful `rig` framework and transforms it into a complete, enterprise-grade platform. By adding robust security patterns, clean architecture, superior developer experience, and production infrastructure, riglr enables developers to build the next generation of sophisticated, autonomous blockchain applications.

The relationship is complementary: `rig` provides the AI reasoning capabilities, while riglr provides everything else needed to turn that reasoning into secure, scalable, production-ready blockchain agents.