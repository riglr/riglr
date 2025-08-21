# riglr Architecture Overview

## Vision Statement

riglr is a comprehensive, modular toolkit for building production-grade, AI-powered blockchain agents. It extends the `rig` AI framework with a robust ecosystem of libraries for on-chain operations, real-time data streaming, multi-agent coordination, and secure transaction management. Our guiding principle has evolved: **build a framework of libraries for creating powerful, autonomous on-chain applications.**

## Core Principles

### 1. **Modular & Composable Design**

- **Layered Architecture**: Crates are organized into logical layers (Core, Tools, Services, Applications), allowing developers to adopt only what they need.
- **Blockchain-agnostic Core**: `riglr-core` provides shared abstractions, with specific implementations in crates like `riglr-solana-tools` and `riglr-evm-tools`.
- **Minimal Dependencies**: Each crate is self-contained and serves a specific purpose, ensuring you only pull in what's necessary.

### 2. **Context-Driven Architecture**

- **Unified Configuration**: `riglr-config` acts as the single source of truth for all settings, from RPC endpoints to feature flags.
- **Dependency Injection**: The `ApplicationContext` pattern provides a clean, type-safe way to inject dependencies like RPC clients and API keys into tools, eliminating circular dependencies.
- **Secure Signer Management**: The `SignerContext` pattern ensures secure, isolated, and thread-safe management of cryptographic keys for transactions.

### 3. **Stateless & Testable Tools**

- **Pure Functions**: Tools are designed as stateless functions that receive all necessary context and parameters, making them easy to test, compose, and reason about.
- **Type-Safe Interfaces**: All tool inputs and outputs are strongly typed, reducing runtime errors.
- **Mocking & Integration Testing**: The architecture is designed to be easily testable, from unit tests with mock signers to full end-to-end tests against real blockchains.

### 4. **rig-First Integration**

- **Seamless Integration**: The `#[tool]` macro in `riglr-macros` automatically generates the boilerplate required to integrate any function with `rig`'s `AgentBuilder`.
- **Agent-Native**: The framework is designed to empower `rig` agents, not replace their core reasoning capabilities. `riglr-agents` provides the coordination layer for `rig`-powered agents to collaborate.

## Architecture Diagram

```
                                 ┌─────────────────┐
                                 │    rig-core     │
                                 │ (The AI Brain)  │
                                 └───────┬─────────┘
                                         │
                                         ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  riglr-config   │    │   riglr-core    │    │  riglr-macros   │
│ (Configuration) │───►│ (Abstractions)  │◄───│  (Code Gen)     │
└─────────────────┘    └───────┬─────────┘    └─────────────────┘
                               │
           ┌───────────────────┼───────────────────┐
           │                   │                   │
   ┌───────▼───────┐   ┌───────▼───────┐   ┌───────▼───────┐
   │ riglr-evm-    │   │ riglr-solana- │   │ riglr-web-    │
   │ tools         │   │ tools         │   │ tools         │
   │ (Uniswap, etc)│   │(Jupiter, Pump)│   │(APIs, Search) │
   └───────┬───────┘   └───────┬───────┘   └───────┬───────┘
           │                   │                   │
   ┌───────▼───────┐   ┌───────▼───────┐   ┌───────▼───────┐
   │riglr-cross-   │   │riglr-hyperliq.│   │riglr-graph-   │
   │chain-tools    │   │tools (Perps)  │   │memory (RAG)   │
   └───────┬───────┘   └───────┬───────┘   └───────┬───────┘
           │                   │                   │
           └────────┬──────────┼──────────┬────────┘
                    │          │          │
           ┌────────▼────────┐ │ ┌────────▼────────┐
           │ riglr-streams   ├─┘ │ riglr-events-   │
           │ (Real-time Data)│   │ core & solana   │
           └────────┬────────┘   └────────┬────────┘
                    │                      │
           ┌────────▼────────┐   ┌─────────▼───────┐
           │  riglr-agents   │   │   riglr-auth    │
           │ (Coordination)  │   │ (Privy, Web3Auth) │
           └────────┬────────┘   └────────┬────────┘
                    │                      │
           ┌────────▼────────┐   ┌─────────▼───────┐
           │ riglr-indexer   │   │  riglr-server   │
           │ (Application)   │   │ (Application)   │
           └─────────────────┘   └─────────────────┘

```

## Component Overview

### Core Layer

*   **riglr-config**: Unified, hierarchical configuration from environment variables (`.env`) and `chains.toml`. Provides type-safe settings for the entire framework.
*   **riglr-core**: The foundational crate with shared abstractions like `UnifiedSigner`, `ApplicationContext`, `SignerContext`, and structured error handling.
*   **riglr-macros**: The `#[tool]` procedural macro that automatically generates all boilerplate for integrating functions with `rig` and `riglr-core`.

### Tools Layer

*   **riglr-solana-tools**: A comprehensive toolkit for the Solana ecosystem, including Jupiter swaps, Pump.fun trading, balance queries, and transaction management.
*   **riglr-evm-tools**: Full support for Ethereum and EVM-compatible chains, including Uniswap integration, token operations (ERC-20, ERC-721), and ENS support.
*   **riglr-web-tools**: Integrations for off-chain data sources like DexScreener, CoinGecko, Twitter, and Exa for web search.
*   **riglr-cross-chain-tools**: Tools for cross-blockchain operations, featuring bridge integrations with Li.Fi for swaps and asset transfers.
*   **riglr-hyperliquid-tools**: Specialized tools for interacting with the Hyperliquid perpetuals DEX, including trading and portfolio management.
*   **riglr-graph-memory**: An advanced Retrieval-Augmented Generation (RAG) system using a Neo4j backend for creating and querying knowledge graphs.

### Services Layer

*   **riglr-events-core & riglr-solana-events**: High-performance libraries for parsing and processing real-time on-chain events.
*   **riglr-streams**: A powerful, composable framework for creating event-driven data pipelines from on-chain (Solana Geyser, EVM WebSockets) and off-chain (Binance, Mempool) sources.
*   **riglr-agents**: A multi-agent coordination system with a dispatcher and registry, enabling specialized `rig`-powered agents to collaborate on complex tasks.
*   **riglr-auth**: Pluggable authentication and signer factories for services like Privy, Web3Auth, and Magic.link.

### Applications Layer

*   **riglr-indexer**: A production-grade, configurable service for indexing blockchain data into a PostgreSQL database for fast querying.
*   **riglr-server**: A turnkey HTTP server (using Axum or Actix) for deploying agents as production-ready services with SSE streaming and API endpoints.
*   **riglr-showcase**: A collection of example agents and best practices demonstrating how to build sophisticated applications with the riglr framework.
*   **create-riglr-app**: An interactive CLI tool for scaffolding new riglr projects with pre-configured templates.

## Quick Start Guide

### 1. Generate a Project with `create-riglr-app`

The fastest way to start is by using the official project generator.

```bash
# Install the generator
cargo install create-riglr-app

# Create a new project (e.g., a trading bot)
create-riglr-app my-trading-bot
```

This will guide you through an interactive setup to create a new, fully configured riglr project.

### 2. A Modern `riglr` Agent (End-to-End Example)

This example, inspired by `riglr-agents/examples/live_fire_demo.rs`, shows the complete, modern workflow from a user prompt to a blockchain transaction.

```rust
use riglr_agents::agents::tool_calling::ToolCallingAgentBuilder;
use riglr_agents::dispatcher::AgentDispatcher;
use riglr_agents::registry::LocalAgentRegistry;
use riglr_agents::toolset::Toolset;
use riglr_agents::types::{Task, TaskType};
use riglr_core::provider::ApplicationContext;
use riglr_core::signer::SignerContext;
use riglr_solana_tools::signer::LocalSolanaSigner;
use rig::providers::openai;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup: Load config, create RPC clients and signers
    let openai_client = openai::Client::from_env();
    let solana_signer = Arc::new(LocalSolanaSigner::from_env()?);
    let solana_rpc_client = Arc::new(solana_client::rpc_client::RpcClient::new(solana_signer.rpc_url().to_string()));

    // 2. Create ApplicationContext and inject dependencies
    let app_context = ApplicationContext::from_env();
    app_context.set_extension(solana_rpc_client); // Inject RPC client for read-only tools

    // 3. Define a Toolset and build a Tool-Calling Agent
    let toolset = Toolset::new().with_solana_tools();
    let agent = ToolCallingAgentBuilder::new(toolset, app_context)
        .build(openai_client.agent("gpt-4o"))
        .await?;

    // 4. Register the agent in a multi-agent system
    let registry = Arc::new(LocalAgentRegistry::new());
    registry.register_agent(agent).await?;

    // 5. Create a dispatcher to route tasks
    let dispatcher = AgentDispatcher::new(registry);

    // 6. Create and dispatch a task
    let prompt = "Send 0.001 SOL to 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM";
    let task = Task::new(
        TaskType::Custom("tool_calling".to_string()),
        serde_json::json!({ "prompt": prompt })
    );

    // 7. Execute within a SignerContext for the transaction
    let result = SignerContext::with_signer(solana_signer, async {
        dispatcher.dispatch_task(task).await
    }).await?;

    println!("Task Result: {:?}", result);
    Ok(())
}
```

## Key Patterns

### ApplicationContext for Dependency Injection

The `ApplicationContext` is the cornerstone of riglr's clean architecture. It holds shared, read-only resources like RPC clients and API keys, which are injected into tools at runtime.

```rust
// In main.rs or application setup
let app_context = ApplicationContext::from_env();
app_context.set_extension(Arc::new(solana_client::rpc_client::RpcClient::new(rpc_url)));

// In a tool function
#[tool]
async fn get_sol_balance(context: &ApplicationContext, address: String) -> Result<f64, ToolError> {
    // Retrieve the client via type-based dependency injection
    let client = context.get_extension::<Arc<solana_client::rpc_client::RpcClient>>()?;
    let balance = client.get_balance(&pubkey).await?;
    Ok(balance as f64 / 1_000_000_000.0)
}
```

### SignerContext for Secure Transactions

The `SignerContext` provides a thread-safe, isolated environment for all operations that require cryptographic signatures, preventing key leakage and ensuring multi-tenant safety.

```rust
// Set the context for a block of operations
SignerContext::with_signer(signer, async {
    // All transactional tools called within this block will automatically use the signer
    let tx_hash = transfer_sol("recipient_address", 0.1, None, None, &app_context).await?;
    Ok(tx_hash)
}).await?;

// Tools automatically access the current signer when needed
#[tool]
async fn transfer_sol(...) -> Result<TransactionResult, ToolError> {
    // This call securely retrieves the signer for the current async context
    let signer = SignerContext::current_as_solana().await?;
    // ... implementation uses the retrieved signer
}
```

### Agent Coordination

The `riglr-agents` crate provides a high-level framework for building systems of collaborating agents.

```rust
// 1. Create specialized agents
let research_agent = Arc::new(ResearchAgent::new(...));
let risk_agent = Arc::new(RiskManagementAgent::new(...));
let execution_agent = Arc::new(TradeExecutionAgent::new(...));

// 2. Register them in the system
let registry = Arc::new(LocalAgentRegistry::new());
registry.register_agent(research_agent).await?;
registry.register_agent(risk_agent).await?;
registry.register_agent(execution_agent).await?;

// 3. The dispatcher routes tasks based on agent capabilities
let dispatcher = AgentDispatcher::new(registry);

// A research task will automatically go to the ResearchAgent
let analysis = dispatcher.dispatch_task(research_task).await?;

// A trading task will go to the TradeExecutionAgent
let trade_result = dispatcher.dispatch_task(trade_task).await?;
```

## Development Workflow

### Adding a New Tool

1.  **Define the function** with a clear signature, including `&ApplicationContext` as a parameter if it needs external resources.
2.  **Add doc comments** to describe the tool's purpose and parameters. This documentation is automatically used to generate the tool's schema for the AI.
3.  **Apply the `#[tool]` macro** to the function.
4.  **Implement the logic**, using `context.get_extension()` for read-only clients and `SignerContext::current_as_*()` for transactions.
5.  **Use structured `ToolError` types** for robust error handling (e.g., `ToolError::retriable_string(...)` vs. `ToolError::permanent_string(...)`).
6.  **Add the tool to a `Toolset`** in `riglr-agents` to make it available to your agents.

## Security Best Practices

### Private Key Management

-   **Never** hardcode private keys or seed phrases.
-   Use secure storage for production keys (e.g., AWS KMS, HashiCorp Vault). `riglr-auth` provides patterns for this.
-   Load keys from environment variables **only** in development (`.env` files should be in `.gitignore`).
-   The `SignerContext` pattern is designed to prevent keys from being cloned or passed around inadvertently.

### Transaction Safety

-   **Validate all inputs** before creating and signing transactions.
-   Implement **slippage protection** for all swaps.
-   Use **deadline parameters** in transactions to prevent them from being executed unexpectedly late.
-   For production systems, consider monitoring for **MEV** (Maximal Extractable Value) attacks.

## Future Roadmap

### Short Term (Next 3 months)

-   **Enhanced EVM Tools**: Add support for more DeFi protocols and complex contract interactions.
-   **Improved `riglr-streams`**: Add more data sources (e.g., other exchanges, on-chain event providers) and advanced operators.
-   **Comprehensive Documentation**: Create interactive tutorials, video guides, and a dedicated documentation website.

### Medium Term (3-6 months)

-   **Additional Blockchain Support**: Begin integration with the Cosmos and Polkadot ecosystems.
-   **Advanced Agent Features**: Introduce more sophisticated routing strategies in `riglr-agents` and built-in agent memory solutions.
-   **Enterprise Features**: Add multi-tenant signer management, detailed audit logging, and enhanced metrics.

### Long Term (6+ months)

-   **AI-Driven Strategies**: Develop tools and agents that leverage machine learning for predictive trading and portfolio optimization.
-   **Governance Integration**: Build tools for interacting with DAOs, including voting and proposal submission.
-   **Institutional Grade Features**: Add compliance reporting, advanced risk management frameworks, and support for institutional custody solutions.

## Getting Help

-   **Documentation**: Check this architecture guide and the documentation on each crate's `README.md`.
-   **Examples**: The `riglr-showcase` crate is the best place for real-world patterns and complete examples.
-   **Issues**: Report bugs and request features on our [GitHub Issues](https://github.com/riglr/riglr/issues).
-   **Discussions**: Join the community for questions and ideas on [GitHub Discussions](https://github.com/riglr/riglr/discussions).

---

*This architecture overview is designed to help you understand riglr's design principles and get started quickly. For detailed API documentation, see the individual crate docs. For examples and best practices, explore the `riglr-showcase` crate.*