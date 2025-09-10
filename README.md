<div align="center">
  <img src="https://raw.githubusercontent.com/riglr/riglr/main/logo.png" alt="riglr Logo" width="200" />
  
  # riglr: Production-Ready AI Agents for Blockchain
  
  [![CI](https://github.com/riglr/riglr/workflows/CI/badge.svg)](https://github.com/riglr/riglr/actions)
  [![Crates.io](https://img.shields.io/crates/v/riglr-core.svg)](https://crates.io/crates/riglr-core)
  [![Documentation](https://img.shields.io/badge/docs-mdBook-brightgreen.svg)](https://riglr.com/docs)
  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
</div>

> ⚠️ **Under Heavy Development**: This project is being actively developed. APIs may change. Use with caution in production.

## What is riglr?

**riglr** (pronounced "riggler") transforms the powerful `rig` AI framework from a tool-calling "brain" into a complete "body and nervous system" for production-grade blockchain AI agents.

While `rig` provides the core LLM-to-tool pipeline, `riglr` adds the enterprise-grade infrastructure, security patterns, and blockchain-specific tooling needed to build, deploy, and scale real-world AI agents that interact with blockchains securely and efficiently.

📚 **[Read the full documentation](https://riglr.com/docs)** for detailed guides, tutorials, and API references.

## Key Features

- 🔒 **Secure by Design**: The `SignerContext` pattern ensures cryptographic keys are never exposed to the LLM reasoning loop, with complete multi-tenant isolation.
- ⚡ **High Performance**: Built for >10,000 events/second throughput with parallel workers, backpressure handling, and circuit breakers.
- 🔧 **Zero Boilerplate**: The `#[tool]` macro transforms a simple Rust function into a production-ready tool, eliminating 30+ lines of boilerplate.
- 🏗️ **Production Patterns**: Clean dependency injection (`ApplicationContext`), fail-fast configuration, and intelligent two-level error handling with automatic retries.
- 🌐 **Multi-Chain Native**: First-class support for Solana, Ethereum, and any EVM-compatible chain. Add new chains with a single environment variable.
- 🤖 **Multi-Agent Coordination**: Build "swarms" of specialized agents that collaborate on complex tasks with intelligent, capability-based routing.

## Quick Start

The fastest way to get started is with our official project generator.

```bash
# 1. Install the generator
cargo install create-riglr-app

# 2. Create a new agent project
create-riglr-app my-trading-bot

# 3. Configure and run!
cd my-trading-bot
cp .env.example .env 
# Edit .env with your API keys and RPC URLs
cargo run -- --interactive
```

### A Glimpse of the Code

`riglr` is designed for an exceptional developer experience. Here's how you can define a complex, secure blockchain operation with almost no boilerplate:

```rust
// 1. Define a tool with a single macro.
// Doc comments are automatically used for the AI's understanding.
#[tool]
/// Transfers SOL to a specified address.
async fn transfer_sol(
    context: &ApplicationContext,  // For shared, read-only clients (e.g., RPC)
    to_address: String, 
    amount_sol: f64
) -> Result<String, ToolError> {
    // 2. Securely access the signer for this specific request.
    // The key is never passed as a parameter or exposed to the LLM.
    let signer = SignerContext::current_as_solana().await?;
    
    // 3. Use injected clients and the isolated signer to execute.
    let rpc_client = context.solana_client()?;
    // ... transaction logic using rpc_client and signer ...
    
    Ok("transaction_signature".to_string())
}

// 4. Build an agent that can use this tool.
let agent = AgentBuilder::new(model)
    .tool(transfer_sol_tool()) // The macro creates this helper function!
    .build();

// 5. Execute in a secure, isolated context for a user.
SignerContext::with_signer(user_signer, async {
    agent.prompt("Send 0.1 SOL to alice.sol").await
}).await?;
```

## Documentation

Our comprehensive documentation is the best place to learn everything about `riglr`.

- 📖 **[Getting Started Guide](https://riglr.com/docs/getting-started/quick-start.md)**
- 🏗️ **[Architecture Deep Dive](https://riglr.com/docs/concepts/architecture/index.md)**
- 💡 **[Tutorials & Examples](https://riglr.com/docs/tutorials/index.md)**
- 🔧 **[Full API Reference](https://riglr.com/docs/api-reference/index.md)**

## Contributing

We welcome contributions of all kinds! Please see [**CONTRIBUTING.md**](CONTRIBUTING.md) for guidelines on how to get involved.

## Community

- **GitHub Issues**: [Report bugs or request features](https://github.com/riglr/riglr/issues)
- **Discussions**: [Join the conversation](https://github.com/riglr/riglr/discussions)

## License

This project is licensed under the [MIT License](LICENSE).