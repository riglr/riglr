# riglr-core

The foundational crate for the riglr ecosystem, providing core abstractions for multi-chain tool orchestration and execution within the rig framework.

[![Crates.io](https://img.shields.io/crates/v/riglr-core.svg)](https://crates.io/crates/riglr-core)
[![Documentation](https://docs.rs/riglr-core/badge.svg)](https://docs.rs/riglr-core)

## 🔒 Production Security Notice

**⚠️ CRITICAL: The file-based key management utilities in riglr-core are for DEVELOPMENT ONLY.**

### For Production Applications, Use Secure Key Management:

#### Option 1: Operating System Keychains (Recommended for desktop apps)
```toml
[dependencies]
keyring = "2.0"
```

```rust
use keyring::Entry;

// One-time key storage
let entry = Entry::new("riglr", "solana_private_key")?;
entry.set_password("your_private_key_here")?;

// Runtime key retrieval
let private_key = entry.get_password()?;
```

#### Option 2: Cloud Secret Managers (Recommended for cloud deployments)
- **AWS Secrets Manager**: Managed secrets with automatic rotation
- **Google Cloud Secret Manager**: Secure secret storage with IAM integration  
- **Azure Key Vault**: Enterprise key management with audit trails
- **HashiCorp Vault**: Open-source secret management platform

#### Option 3: Hardware Security Modules (Enterprise)
- **AWS CloudHSM**: FIPS 140-2 Level 3 validated HSMs
- **YubiHSM**: Compact hardware security modules
- **Ledger/Trezor**: Hardware wallet integration

### Security Principles:
- 🔑 **Never store private keys in code or configuration files**
- 🔒 **Use encrypted storage with access controls**
- 📋 **Implement audit logging for key access**
- 🔄 **Enable automatic key rotation where possible**
- 🌐 **Isolate production and development key storage**

See [`riglr-core::util::secure_keys`] for detailed guidance and examples.

---

## Architecture Overview

riglr-core provides the foundation for building resilient AI agents with clean dependency injection and no circular dependencies. The architecture enforces a unidirectional flow: `tools` -> `core`.

### 1. Client Injection Pattern (ApplicationContext)

For read-only operations with dependency injection:

```rust
use riglr_config::Config;
use riglr_core::provider::ApplicationContext;
use std::sync::Arc;

// Application is responsible for creating and injecting clients
let config = Config::from_env();
let app_context = ApplicationContext::from_config(&config);

// Inject Solana RPC client
let solana_client = Arc::new(
    solana_client::rpc_client::RpcClient::new(config.network.solana_rpc_url)
);
app_context.set_extension(solana_client);

// Inject EVM Provider (using alloy)
use alloy::providers::{Provider, ProviderBuilder};
let provider = ProviderBuilder::new()
    .on_http("https://eth.llamarpc.com".parse()?)
    .boxed();
app_context.add_extension(Arc::new(provider) as Arc<dyn Provider>);

// Tools retrieve clients from extensions
#[tool]
async fn get_balance(address: String) -> Result<Balance, ToolError> {
    let app_context = ApplicationContext::from_env();
    
    // Get injected client from context
    let rpc_client = app_context
        .get_extension::<Arc<solana_client::rpc_client::RpcClient>>()
        .ok_or_else(|| ToolError::permanent_string("Solana RpcClient not found"))?;
    
    // Use client for operations...
    Ok(balance)
}
```

### 2. SignerContext Pattern

For transactional operations requiring signatures:

```rust
use riglr_core::{SignerContext, signer::LocalSolanaSigner};

// For operations that need signing
let signer = Arc::new(LocalSolanaSigner::from_keypair(keypair, network_config));

SignerContext::with_signer(signer, async {
    // All transactional tools automatically use this signer
    swap_tokens(input_mint, output_mint, amount).await
}).await?;
```

## Extension System

riglr-core provides a generic extension system that allows higher-level crates to inject their dependencies without creating circular references:

```rust
use riglr_solana_tools::clients::ApiClients;

// In your main function
let config = Config::from_env();
let app_context = ApplicationContext::from_config(&config);

// Inject crate-specific dependencies
let api_clients = ApiClients::new(&config.providers);
app_context.set_extension(api_clients);

// Tools can now access these extensions
let api_clients = context.get_extension::<ApiClients>()?;
```

## Tool Worker

The ToolWorker executes tools with automatic context injection:

```rust
use riglr_core::{ToolWorker, ExecutionConfig};
use riglr_core::idempotency::InMemoryIdempotencyStore;

let worker = ToolWorker::<InMemoryIdempotencyStore>::new(
    ExecutionConfig::default(),
    app_context
);

// Register tools
worker.register_tool(Arc::new(GetBalanceTool::new())).await;

// Process jobs
let job = Job::new("get_balance", &json!({"address": "..."}), 3)?;
let result = worker.process_job(job).await?;
```

## Tool Trait

The Tool trait has been updated for type-safe error handling:

```rust
use async_trait::async_trait;
use riglr_core::{Tool, ToolError, JobResult};
use riglr_core::provider::ApplicationContext;

#[async_trait]
impl Tool for MyTool {
    async fn execute(
        &self,
        params: serde_json::Value,
        context: &ApplicationContext,
    ) -> Result<JobResult, ToolError> {
        // Type-safe error handling with ToolError
        let address = params["address"].as_str()
            .ok_or_else(|| ToolError::invalid_input_string("Missing address"))?;
        
        // Access injected dependencies from context
        let client = context.get_extension::<MyClient>()
            .ok_or_else(|| ToolError::permanent_string("Client not configured"))?;
        
        // Perform operation
        let result = client.fetch(address).await
            .map_err(|e| ToolError::retriable_string(e.to_string()))?;
        
        JobResult::success(&result)
            .map_err(|e| ToolError::permanent_string(e.to_string()))
    }
    
    fn name(&self) -> &str {
        "my_tool"
    }
    
    fn description(&self) -> &str {
        "Description of what this tool does"
    }
}
```

## Error Handling

riglr-core provides structured error handling with retry classification:

```rust
use riglr_core::ToolError;

// Permanent errors (don't retry)
return Err(ToolError::permanent_string("Invalid address format"));

// Retriable errors (safe to retry)
return Err(ToolError::retriable_string("Network timeout"));

// Invalid input (user error)
return Err(ToolError::invalid_input_string("Missing required parameter"));
```

## Features

- **ApplicationContext**: Dependency injection and RPC provider management
- **SignerContext**: Thread-safe signer management for transactions
- **Extension System**: Type-safe dependency injection without circular references
- **Tool Execution**: Resilient execution with retries and error classification
- **Idempotency**: Built-in duplicate operation prevention
- **Metrics**: Comprehensive monitoring and observability

## Integration with Higher-Level Crates

riglr-core is designed to be extended by blockchain-specific crates:

- `riglr-solana-tools`: Solana blockchain operations
- `riglr-evm-tools`: EVM blockchain operations
- `riglr-config`: Unified configuration management

Each crate provides its own tools and can inject dependencies into the ApplicationContext without creating circular dependencies.