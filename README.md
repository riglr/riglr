<div align="center">
  <img src="logo.png" alt="riglr Logo" width="200" />
  
  # riglr - Production-Ready AI Agent Framework for Blockchain

  [![CI](https://github.com/riglr/riglr/workflows/CI/badge.svg)](https://github.com/riglr/riglr/actions)
  [![Crates.io](https://img.shields.io/crates/v/riglr-core.svg)](https://crates.io/crates/riglr-core)
  [![Documentation](https://docs.rs/riglr-core/badge.svg)](https://docs.rs/riglr-core)
  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
</div>

> ⚠️ **UNDER HEAVY DEVELOPMENT**: This project is being actively developed with frequent updates. APIs may change and things will probably break. Use with caution in production environments.

## 🎯 What Makes riglr Different?

**riglr** transforms the powerful `rig` LLM framework into a **production-grade platform** for blockchain AI agents. While `rig` provides the "brain" (LLM-to-tool connections), riglr adds the complete "body and nervous system" with battle-tested patterns from real deployments:

### 🚀 Production Performance
- **10,000+ events/second** throughput with parallel processing
- **Multi-tenant isolation** with zero cross-contamination risk
- **Horizontal scaling** with distributed job queues
- **Circuit breakers & retry logic** for resilient operations

### 🔐 Enterprise Security
- **`SignerContext` pattern**: Thread-local transaction signing that never exposes keys to the LLM
- **Type-safe chain handles**: Compile-time guarantees for blockchain operations
- **Pluggable authentication**: Official integrations with Privy, Web3Auth, Magic.link
- **Idempotency store**: Safe retries in distributed systems

### ⚡ Developer Experience
- **`#[tool]` macro**: Turn any async function into a tool with **one line** (30+ lines of boilerplate eliminated)
- **Zero-code chain additions**: Add new EVM chains via environment variables
- **Mock-friendly architecture**: Test without real blockchain connections
- **Convention over configuration**: Smart defaults with override capability

### 🏗️ Clean Architecture
- **`ApplicationContext` pattern**: Dependency injection for modular, testable code
- **Two-level error handling**: Behavioral classification + chain-specific details
- **Chain-agnostic core**: Zero blockchain SDK dependencies in core
- **Unidirectional dependencies**: Clean compilation boundaries

## 📊 riglr vs. Other Frameworks

| Feature | riglr | Other Frameworks |
|---------|-------|------------------|
| **Transaction Security** | Thread-local `SignerContext` isolation | Keys passed as parameters |
| **Multi-tenancy** | Built-in tenant isolation | Manual implementation required |
| **Error Handling** | Two-level with auto-retry | Simple error returns |
| **Tool Creation** | 1-line `#[tool]` macro | 30+ lines boilerplate |
| **Event Processing** | 10k+ events/sec with backpressure | Basic WebSocket connections |
| **Production Patterns** | Circuit breakers, idempotency, retry logic | DIY implementation |
| **Testing** | Mock-friendly, mainnet forking | Requires real connections |
| **Multi-agent** | Built-in coordination & dispatch | Single agent only |
| **Chain Support** | Dynamic via env vars | Hardcoded chains |

## 🗺️ Quick Navigation

> **New to riglr?** Check out our comprehensive [Documentation](https://riglr.com/docs) for detailed guides, tutorials, and API references.

## 🏆 Key Production Patterns

These battle-tested patterns differentiate riglr from other frameworks:

### 1. **`SignerContext`: Secure Multi-Tenant Transaction Signing**
```rust
// Each user request gets completely isolated signer context
SignerContext::with_signer(user_signer, async {
    // Keys NEVER exposed to LLM reasoning loop
    // Automatic cleanup when scope exits
    // Zero cross-tenant contamination risk
    agent.execute("Send 0.1 SOL to alice.sol").await
}).await
```
**Why It Matters**: Keys are never passed as parameters, preventing exposure to the LLM. Complete isolation between concurrent users in production.

### 2. **`ApplicationContext`: Clean Dependency Injection**
```rust
// Application injects dependencies once
context.set_extension(Arc::new(solana_client));
context.set_extension(Arc::new(evm_provider));

// Tools retrieve what they need - no hardcoding
#[tool]
async fn get_balance(address: String) -> Result<f64, ToolError> {
    let client = context.get_extension::<SolanaClient>()?;
    // Use client...
}
```
**Why It Matters**: Decouples tools from implementations, enables mock testing, supports multiple chains without circular dependencies.

### 3. **`#[tool]` Macro: 30 Lines → 1 Line**
```rust
// Before: 30+ lines of boilerplate for Tool trait impl
// After: Just one line
#[tool]
async fn swap_tokens(from: String, to: String, amount: u64) -> Result<String, ToolError> {
    // Your logic only - macro handles all boilerplate
}
```
**Generated Automatically**: Parameter struct, JSON schema, Tool trait impl, error conversion, documentation extraction.

### 4. **Two-Level Error Handling with Auto-Retry**
```rust
// High-level behavioral classification
match error {
    ToolError::Retriable(_) => // Automatic exponential backoff
    ToolError::RateLimited(_) => // Respect rate limits
    ToolError::Permanent(_) => // Don't retry
}

// Low-level chain-specific details preserved
if let Some(solana_error) = error.downcast::<SolanaToolError>() {
    match solana_error {
        SolanaToolError::BlockhashExpired => // Handle specifically
    }
}
```

### 5. **Multi-Agent Coordination System**
```rust
// Specialized agents work together
let research_result = dispatcher.dispatch_task(
    Task::new(TaskType::Research, market_data)
).await?;

let risk_result = dispatcher.dispatch_task(
    Task::new(TaskType::RiskAnalysis, research_result)
).await?;

if risk_result.approved() {
    dispatcher.dispatch_task(Task::new(TaskType::Trading, ...)).await?;
}
```

### 6. **Real-Time Event Processing (10k+ events/sec)**
```rust
// Composable stream pipeline with backpressure handling
let processed_stream = solana_stream
    .filter(|e| matches!(e.kind(), EventKind::Swap))
    .map(extract_metrics)
    .throttle(Duration::from_secs(10))
    .batch(50, Duration::from_secs(60));
```

### 7. **Production Resilience Patterns**
- **Circuit Breakers**: Prevent cascading failures
- **Idempotency Store**: Safe retries in distributed systems
- **Resource Limits**: Configurable timeouts and memory limits
- **Health Checks & Metrics**: Prometheus-compatible monitoring
- **Graceful Shutdown**: Clean resource cleanup

## 🏗️ What is riglr?

riglr is a **modular, production-ready framework** for building blockchain AI agents that scale from simple bots to sophisticated multi-agent systems:

### Core Foundation (Zero Blockchain Dependencies)
- **`riglr-core`**: SignerContext, ApplicationContext, ToolWorker patterns
- **`riglr-macros`**: Zero-boilerplate tool generation with `#[tool]`
- **`riglr-config`**: Unified configuration with fail-fast validation

### Blockchain Integration (50+ Pre-built Tools)
- **`riglr-solana-tools`**: Balance queries, swaps, Pump.fun, Jupiter
- **`riglr-evm-tools`**: Uniswap, contract interactions, multi-chain support
- **`riglr-cross-chain-tools`**: Li.Fi bridge integration

### Production Systems
- **`riglr-agents`**: Multi-agent coordination with dispatch patterns
- **`riglr-streams`**: 10k+ events/sec processing with backpressure
- **`riglr-indexer`**: High-throughput blockchain data indexing
- **`riglr-server`**: Turnkey HTTP server with auth & metrics
- **`riglr-auth`**: Privy, Web3Auth, Magic.link integrations

## 🚀 Quick Example

```rust
// 1. Define a tool with ZERO boilerplate
#[tool]
async fn transfer_sol(to: String, amount: f64) -> Result<String, ToolError> {
    // SignerContext automatically provides the signer
    let signer = SignerContext::get_signer()?;
    // Your logic here
}

// 2. Build an agent with automatic tool discovery
let agent = ToolCallingAgentBuilder::new()
    .with_tools(riglr_solana_tools::all_tools())
    .build(openai_client.agent("gpt-4o"))
    .await?;

// 3. Execute with secure, isolated signing
SignerContext::with_signer(user_signer, async {
    // Signer is ONLY available within this scope
    // Automatic cleanup when scope exits
    agent.chat("Send 0.1 SOL to alice.sol").await?
}).await?;
```

## 🏗️ Architecture

riglr uses a multi-crate architecture with clear separation of concerns:

### Core Foundation
- **`riglr-config`**: Unified configuration management for all crates
- **`riglr-core`**: Core abstractions, ToolWorker, SignerContext, and ApplicationContext patterns
- **`riglr-macros`**: Code generation with the `#[tool]` macro

### Blockchain Integration
- **`riglr-solana-tools`**: Solana-specific tools (balance queries, swaps, Pump.fun)
- **`riglr-evm-tools`**: EVM-specific tools (balance queries, Uniswap, contract interactions)
- **`riglr-cross-chain-tools`**: Cross-chain bridging and multi-chain operations

### Application Layer
- **`riglr-agents`**: Multi-agent coordination and communication
- **`riglr-streams`**: Real-time event processing and data pipelines
- **`riglr-indexer`**: Production blockchain data indexing
- **`riglr-web-tools`**: External API integrations (price feeds, news, social)
- **`riglr-auth`**: Authentication providers (Privy, Web3Auth, Magic)

### Dual-Pattern Architecture

riglr implements two complementary patterns:

**Client Injection Pattern** (Read-only operations):
```rust
// Application creates and injects all clients
let config = Config::from_env();
let app_context = ApplicationContext::from_config(&config);

// Inject blockchain clients
let solana_client = Arc::new(RpcClient::new(config.network.solana_rpc_url));
app_context.set_extension(solana_client);

let evm_client = Arc::new(EvmClient::new("https://eth.llamarpc.com").await?);
app_context.set_extension(evm_client);

// Tools retrieve clients from context's extensions
#[tool]
async fn get_balance(address: String) -> Result<Balance, ToolError> {
    let app_context = ApplicationContext::from_env();
    let client = app_context.get_extension::<Arc<RpcClient>>()?;
    // Use client...
}
```

**SignerContext Pattern** (Transactional operations):
```rust
// Create signer with network configuration
let signer = Arc::new(LocalSolanaSigner::from_keypair(keypair, network_config));

// Execute transactions within signer context
SignerContext::with_signer(signer, async {
    transfer_sol(recipient, amount).await
}).await?;
```

## 🔄 Two-Level Error Handling Pattern

riglr uses a sophisticated two-level error handling pattern for robust blockchain interactions:

### High-Level: Behavior-Based Retry Logic

At the framework level, errors are classified by behavior (retriable, permanent, rate-limited) not by chain:

```rust
use riglr_core::ToolError;
use riglr_core::retry::retry_async;

// Framework automatically retries based on error classification
let result = retry_async(
    || async { perform_blockchain_operation().await },
    |error| match error {
        ToolError::Retriable(_) => ErrorClass::Retriable,
        ToolError::RateLimited(_) => ErrorClass::RateLimited,
        _ => ErrorClass::Permanent,
    },
    RetryConfig::default()
).await?;
```

### Low-Level: Chain-Specific Error Details

For advanced users who need chain-specific error handling:

```rust
use riglr_core::ToolError;
use riglr_solana_tools::SolanaToolError;

match result {
    Err(tool_error) => {
        // Try to downcast to get chain-specific error details
        if let Some(solana_error) = tool_error.source()
            .and_then(|e| e.downcast_ref::<SolanaToolError>()) {
            // Access Solana-specific error information
            match solana_error {
                SolanaToolError::InsufficientFunds => { /* handle */ },
                SolanaToolError::BlockhashExpired => { /* retry */ },
                _ => { /* other handling */ }
            }
        }
    }
    Ok(value) => { /* success */ }
}
```

This pattern provides both simplicity for common cases and power for advanced scenarios.

## 📚 Documentation

The riglr documentation is available at [riglr.com/docs](https://riglr.com/docs) and includes:

- **[Getting Started Guide](https://riglr.com/docs/getting-started/quick-start)** - Quick introduction to riglr
- **[Architecture Overview](https://riglr.com/docs/concepts/architecture-overview)** - Comprehensive system design
- **[Under the Hood](https://riglr.com/docs/concepts/under-the-hood)** - From brain to blockchain flow
- **[Dependency Graph](https://riglr.com/docs/concepts/dependency-graph)** - Visual crate relationships
- **[API Reference](https://riglr.com/docs/api-reference)** - Complete API documentation
- **[Tutorials](https://riglr.com/docs/tutorials)** - Step-by-step guides for common use cases

### Building the Documentation Locally

```bash
# Install mdBook
cargo install mdbook

# Build the documentation
cd docs
mdbook build

# Serve locally
mdbook serve --open
```

## 🔧 Quick Start with `create-riglr-app`

The easiest way to start is with our official project generator.

### 1. Install the Scaffolding Tool
```bash
cargo install create-riglr-app
```

### 2. Generate a New Project
Create a new trading bot project. The interactive CLI will guide you through selecting a template, blockchains, and features.
```bash
create-riglr-app my-trading-bot
```

### 3. Configure and Run
```bash
cd my-trading-bot
cp .env.example .env
# Edit .env with your API keys and RPC URLs
cargo run
```

## 📦 Crates Overview

| Crate | Description | Version |
|-------|-------------|---------|
| [riglr-core](./riglr-core) | Core framework, job processing, idempotency, signer traits. | 0.1.0 |
| [riglr-config](./riglr-config) | Unified, hierarchical configuration management. | 0.1.0 |
| [riglr-macros](./riglr-macros) | Procedural macros (`#[tool]`) for rapid tool generation. | 0.1.0 |
| [riglr-agents](./riglr-agents) | Multi-agent coordination system with dispatch and registry. | 0.1.0 |
| [riglr-streams](./riglr-streams) | Real-time event streaming from multiple on-chain and off-chain sources. | 0.1.0 |
| [riglr-indexer](./riglr-indexer) | Production-grade blockchain indexing service. | 0.1.0 |
| [riglr-events-core](./riglr-events-core) | Core event processing abstractions and traits. | 0.1.0 |
| [riglr-solana-events](./riglr-solana-events) | High-performance Solana event parsing for multiple protocols. | 0.1.0 |
| [riglr-solana-tools](./riglr-solana-tools) | Tools for interacting with the Solana blockchain. | 0.1.0 |
| [riglr-evm-tools](./riglr-evm-tools) | Tools for EVM-compatible chains (Ethereum, Polygon, etc.). | 0.1.0 |
| [riglr-web-tools](./riglr-web-tools) | Tools for web APIs (DexScreener, Twitter, News). | 0.1.0 |
| [riglr-auth](./riglr-auth) | Authentication and signer factories (Privy, Web3Auth). | 0.1.0 |
| [riglr-graph-memory](./riglr-graph-memory) | Graph-based memory system with Neo4j. | 0.1.0 |
| [riglr-cross-chain-tools](./riglr-cross-chain-tools) | Cross-chain bridge integration via Li.Fi. | 0.1.0 |
| [riglr-hyperliquid-tools](./riglr-hyperliquid-tools) | Tools for the Hyperliquid perpetuals DEX. | 0.1.0 |
| [riglr-server](./riglr-server) | Turnkey, production-ready HTTP server for agents. | 0.1.0 |
| [riglr-showcase](./riglr-showcase) | Example agents and demonstrations. | 0.1.0 |

## 🗺️ Project Roadmap: Powering the Next Generation of Blockchain AI Agents

This roadmap outlines the strategic vision for `riglr`, a professional-grade Rust framework for building the entire spectrum of AI agents that interact with blockchains. From simple, reactive bots to sophisticated, proactive multi-agent systems, `riglr` will provide a modular, high-performance, and secure foundation that scales with developer ambition.

### **Pillar 1: Radically Improve Developer Experience & Onboarding**

A seamless and intuitive developer journey is paramount. Our goal is to not only match but exceed the ease of use and quality of documentation of existing solutions.

**1. Comprehensive Documentation Hub (`mdbook`):** To centralize and streamline access to information, we will create an official documentation website. This hub, built with `mdbook`, will serve as the single source of truth for developers. It will feature:
- **Tutorials:** Step-by-step guides for building common agent types, such as "Building a Solana Arbitrage Bot" and "Creating a Cross-Chain Portfolio Manager."
- **Conceptual Guides:** In-depth explanations of core concepts like `SignerContext`, our error handling philosophy, and the event parsing architecture.
- **Tool Reference:** Auto-generated, searchable documentation for every tool across all official crates, complete with practical examples.
- **Deployment Guides:** Actionable blueprints for deploying `riglr`-based agents to platforms like Fly.io, Docker, and Kubernetes.

**2. Enhanced Scaffolding with `create-riglr-app`:** The initial project setup will be made more powerful and flexible through significant enhancements to our scaffolding tool. These improvements will include:
- **Expanded Template Library:** A wider range of templates for diverse use cases, including "API Service Backend," "Data Analytics Bot," and "Event-Driven Trading Engine."
- **Pre-configured Server Integration:** An option to scaffold projects with a ready-to-run Actix or Axum server, providing an immediate, robust backend.
- **Interactive Command-Line Interface:** A more engaging and guided CLI experience that helps users select the specific tools and protocols they need for their project.

**3. Interactive Learning Resources:** To foster a more effective and active learning environment, we will develop a `riglr-by-example` repository. This will feature interactive, in-browser tutorials, allowing developers to write and run `riglr` code without the need for any local environment setup.

### **Pillar 2: Provide Turnkey Production-Ready Services**

We will equip developers with the components necessary to deploy production-grade services with minimal friction.

**1. The `riglr-server` Crate:** To simplify the process of deploying a web-facing service, we will introduce a dedicated `riglr-server` crate. This will offer a pre-built, configurable, and production-ready server using either Actix or Axum. Key features will include:
- Pre-configured endpoints for streaming and completion.
- Built-in middleware for authentication, logging, and metrics.
- Seamless integration with the `SignerFactory` pattern.

**2. First-Class Authentication with `riglr-auth`:** Recognizing the critical importance of secure and flexible authentication, we will create a `riglr-auth` crate. This crate will provide official, maintained `SignerFactory` implementations for leading authentication services, including:
- **Privy:** A simple library for beautiful authentication flows and embedded wallets.
- **Web3Auth:** An infrastructure for Web3 apps and wallets that provides seamless user logins.
- **Magic.link:** A passwordless authentication method that uses unique, time-sensitive URLs.

**3. Official Deployment Blueprints:** To provide a clear and reliable path to production, we will establish a `riglr-deploy` repository. This will contain official, production-grade templates for a variety of deployment targets:
- **Docker Compose:** A multi-container setup for managing dependencies like Redis, Neo4j, and the `riglr-server`.
- **Fly.io:** Simplified `fly.toml` configurations for effortless deployments.
- **Kubernetes:** Foundational Helm charts for deploying to any Kubernetes cluster.

### **Pillar 3: Double Down on Unique Strengths & Advanced Capabilities**

We will continue to invest in and expand upon the features that set `riglr` apart as a high-performance, professional-grade framework.

**1. Expanded Solana Event Parsing System:** Our unique and powerful Solana event parsing capabilities will be enhanced by:
- Adding parsers for a broader range of protocols.
- Creating tools that enable agents to subscribe to real-time streams of parsed events, facilitating proactive, event-driven strategies.
- Publishing the parser as a standalone, high-performance library to establish it as an industry standard.

**2. Evolved `riglr-graph-memory`:** The Neo4j-based knowledge graph, a key differentiator, will be improved with:
- More sophisticated graph analytics tools for pathfinding, community detection, and fraud analysis.
- A "Graph RAG" agent template within `create-riglr-app`, pre-configured to build and query a knowledge graph from blockchain data.

**3. Proactive, Event-Driven Tooling:** A new `riglr-streams` or `riglr-ingest` crate will provide the components for developers to build their own proactive agents. This will include:
- A Solana Geyser plugin connector for low-latency data access.
- EVM WebSocket subscription helpers for events like `pendingTransactions`.
- Connectors for real-time data sources such as Binance and Mempool.space streams.

**4. Formalized Advanced Agentic Patterns:** To support the development of more complex systems, we will introduce a `riglr-agents` module or crate. This will provide helpers and examples for building multi-agent systems, such as a `DispatcherAgent` that routes tasks to specialized agents.

### **Pillar 4: Foster a Thriving Ecosystem**

Building a vibrant and collaborative community is essential for long-term success.

**1. Community Tool Registry:** We will create a platform for the community to publish and share their own `riglr`-compatible tools. This will foster a network effect, making `riglr` the framework of choice due to its extensive library of community-vetted tools.

**2. Sharpened Project Positioning:** The project's vision will be clearly and concisely articulated in the main `README.md`:

> "riglr is the professional-grade Rust framework for building any blockchain AI agent imaginable—from simple, reactive bots to complex, proactive, multi-agent systems. It provides a modular, high-performance, and secure foundation that scales with your ambition."

### **Summary of the Winning Strategy**

By executing on these pillars, `riglr` will establish itself as the premier framework for blockchain AI agent development by offering:

- **The Best Onboarding:** A developer experience and documentation that sets a new industry standard.
- **The Easiest Path to Production:** Turnkey server and authentication components that simplify deployment.
- **The Most Powerful Capabilities:** Unmatched, specialized tools for event parsing and graph memory, alongside components for building proactive, event-driven systems.
- **The Strongest Ecosystem:** A flourishing community built around a shared registry of tools and a collaborative spirit.

## 🧪 Testing Strategy

A comprehensive unit and integration testing strategy is crucial for ensuring the reliability and robustness of the `riglr` codebase. Our approach will focus on creating an isolated, deterministic, and realistic testing environment.

### Test Environment and Infrastructure

Building upon our existing use of `.env.test` and a `Dockerfile.test`, we will enhance our testing infrastructure with:

- **Containerized Services:** Docker Compose will be used to define and manage all external services required for testing, including a Solana test validator, an Anvil node for EVM, Redis, and PostgreSQL. This will guarantee a consistent and clean environment for every test run.
- **Test Runner Configuration:** We will leverage `cargo test` with test workspaces and feature flags to segregate different test types, such as `unit`, `integration`, and `e2e`.
- **CI/CD Pipeline Integration:** Our CI/CD pipeline will be expanded to execute different test suites based on the context. Quick unit tests will run on every commit, while full integration tests that require forked data will be reserved for pull requests to the main branch and nightly builds.

### Test Data Strategy: Downloading and Simulating Blockchain Data

To test against realistic and complex blockchain states, we will implement mainnet forking and transaction replay.

**Implementation:**

1. **Mainnet Forking:** We will create local test environments that mirror the state of a public network at a specific block, enabling fast and deterministic execution against real-world data.
2. **Transaction Replay:** Historical transactions will be fetched and replayed against a local test validator to thoroughly test indexers, event parsers, and streaming systems.
3. **State Seeding:** Scripts will be developed to programmatically set up specific blockchain states on a local validator before tests are executed.

**Available Resources:**

- **For EVM Chains:** We will utilize **Foundry Anvil** for its built-in mainnet forking capabilities.
- **For Solana:** The **Solana Test Validator** will be used with the `--clone` flag to pull down specific accounts and protocols. We will also explore using the **Geyser plugin** interface for replaying transactions.

### Multi-Layered Testing Approach

We will formalize our testing into the following distinct layers:

- **Unit Tests:** Fast, isolated tests for individual functions and components, utilizing mocks for external dependencies.
- **Integration Tests:** Verification that different components of the system work together correctly, leveraging a forked blockchain environment.
- **End-to-End (E2E) Tests:** Simulation of full user workflows to validate the entire system from start to finish.
- **Property-Based Testing:** Expansion of our existing property-based tests to cover a wider range of inputs and ensure the logical correctness of our code.

### Actionable Plan to Get Ready

1. **Develop a Test Data Management Service:**
   - Create a new internal crate, `riglr-test-utils`, to house utilities for programmatically starting and stopping forked test environments with Anvil and `solana-test-validator`.
   - Add functions to fetch and replay a specific number of historical blocks or transactions.

2. **Standardize Environment Orchestration:**
   - Create a `docker-compose.test.yml` file to define all necessary services for a fully provisioned, isolated testing environment.
   - Integrate this with our CI pipeline for automated integration and E2E testing.

3. **Enhance Mocking Capabilities:**
   - Introduce a mock HTTP server like `wiremock-rs` to simulate responses from external APIs, allowing for comprehensive testing of various scenarios, including API errors and rate limits.

4. **Expand Test Scenario Coverage:**
   - Develop a "scenario" testing framework to define specific blockchain states, agent actions, and expected outcomes.
   - Add tests for complex DeFi interactions and expand security tests to cover scenarios like transaction front-running and oracle manipulation on our local forked testnet.

---

<p align="center">
  Built with ❤️ by the riglr community
</p>