# riglr Architecture Overview

## Vision Statement

riglr is a comprehensive blockchain toolkit designed to seamlessly extend the `rig` AI framework with powerful, modular tools for on-chain operations. Our guiding principle is simple: **build a library for `rig`, not an application**.

## Core Principles

### 1. **Modular Design**

- **Blockchain-agnostic core** with specific tool implementations
- Each crate serves a specific purpose and can be used independently
- Minimal dependencies - only pull in what you need

### 2. **SignerContext Pattern**

- **Unified interface** for transaction signing across all supported blockchains
- **Thread-safe** and **secure** private key management
- **Extensible** - implement `TransactionSigner` for custom KMS solutions

### 3. **Stateless Tools**

- Pure functions that can be easily **tested** and **composed**
- No global state or client parameters in tool signatures
- Tools "just work" by pulling signers from context via `SignerContext::current().await`

### 4. **rig-First Integration**

- Tools generate full `impl rig::tool::Tool` boilerplate via `#[tool]` macro
- Seamless integration with `AgentBuilder` pattern
- Leverage `rig`'s native features (multi-turn, reasoning loops) instead of replacing them

## Architecture Diagram

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   rig-core      │    │   riglr-core    │    │  riglr-macros   │
│                 │    │                 │    │                 │
│ • AgentBuilder  │◄───┤ • SignerContext │    │ • #[tool] macro │
│ • Tool trait    │    │ • TransactionSigner    │ • Tool impl     │
│ • Multi-turn    │    │ • Error handling│    │   generation    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                ┌───────────────┼───────────────┐
                │               │               │
        ┌───────▼─────┐ ┌───────▼─────┐ ┌───────▼─────┐
        │riglr-solana-│ │ riglr-evm-  │ │ riglr-web-  │
        │tools        │ │ tools       │ │ tools       │
        │             │ │             │ │             │
        │• Pump.fun   │ │• Uniswap    │ │• DexScreener│
        │• Jupiter    │ │• 1inch      │ │• CoinGecko  │
        │• Risk tools │ │• ENS        │ │• Twitter    │
        └─────────────┘ └─────────────┘ └─────────────┘

    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
    │riglr-cross-     │    │riglr-hyperliquid│    │riglr-graph-     │
    │chain-tools      │    │tools            │    │memory           │
    │                 │    │                 │    │                 │
    │• LiFi bridge    │    │• Perp trading   │    │• Neo4j backend  │
    │• Cross-chain    │    │• Portfolio      │    │• Graph RAG      │
    │  swaps          │    │  management     │    │• Entity extract │
    └─────────────────┘    └─────────────────┘    └─────────────────┘

                    ┌─────────────────┐    ┌─────────────────┐
                    │  riglr-server   │    │ riglr-showcase  │
                    │                 │    │                 │
                    │• HTTP server    │    │• Example agents │
                    │• SSE streaming  │    │• Best practices │
                    │• API endpoints  │    │• Multi-turn     │
                    └─────────────────┘    │  patterns       │
                                          └─────────────────┘
```

## Component Overview

### Core Components

#### riglr-core

The foundational crate providing shared abstractions and patterns:

- **SignerContext**: Thread-local signer management
- **TransactionSigner**: Trait for blockchain signing implementations
- **Error handling**: Structured error types for reliable transaction processing

#### riglr-macros

Procedural macros for developer productivity:

- **#[tool] macro**: Generates complete `rig::tool::Tool` implementations
- **Reduces boilerplate**: From ~30 lines to a single attribute
- **Documentation extraction**: Automatically generates tool schemas from docstrings

### Blockchain-Specific Tools

#### riglr-solana-tools

Comprehensive Solana ecosystem integration:

- **DeFi protocols**: Jupiter swaps, Pump.fun trading
- **Risk analysis**: RugCheck.xyz integration
- **Advanced orders**: Conditional and limit orders
- **Account management**: Balance queries, token operations

#### riglr-evm-tools

Ethereum and EVM-compatible chain support:

- **DeFi protocols**: Uniswap, 1inch integration
- **ENS support**: Domain resolution and management
- **Multi-chain**: Polygon, BSC, Arbitrum support
- **Token operations**: ERC-20, ERC-721 handling

#### riglr-web-tools

External data source integrations:

- **Price feeds**: CoinGecko, DexScreener
- **Social data**: Twitter, LunarCrush analytics
- **Market data**: Real-time quotes and historical data

### Specialized Components

#### riglr-cross-chain-tools

Cross-blockchain operations:

- **Bridge integrations**: LiFi, Synapse protocol support
- **Cross-chain swaps**: Atomic swaps and bridge operations
- **Multi-chain portfolio**: Unified balance and position tracking

#### riglr-hyperliquid-tools

Derivatives trading platform integration:

- **Perpetual futures**: Position management and trading
- **Portfolio analytics**: PnL tracking and risk metrics
- **Advanced orders**: Stop-loss, take-profit automation

#### riglr-graph-memory

Graph-based RAG (Retrieval-Augmented Generation):

- **Neo4j backend**: Scalable graph database storage
- **Entity extraction**: Automatic knowledge graph construction
- **Semantic search**: Advanced query capabilities for AI agents

#### riglr-server

Production-ready HTTP server:

- **Generic agent serving**: Works with any `rig::Agent`
- **SSE streaming**: Real-time response streaming
- **OpenAPI documentation**: Auto-generated API specs

#### riglr-showcase

Example implementations and best practices:

- **Agent composition**: Real-world agent examples
- **Integration patterns**: How to combine multiple tools
- **Best practices**: Error handling, testing, deployment

## Quick Start Guide

### 1. Basic Setup

```rust
use riglr_core::signer::SignerContext;
use riglr_solana_tools::LocalSolanaSigner;
use rig::{
    providers::openai,
    agent::AgentBuilder,
    completion::Prompt,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize your AI provider
    let openai_client = openai::Client::from_env();

    // Set up blockchain signer
    let signer = LocalSolanaSigner::from_env()?;
    
    // Build your agent with riglr tools
    let agent = AgentBuilder::new(openai_client)
        .preamble("You are a helpful DeFi assistant")
        .tool(riglr_solana_tools::GetSolBalance)
        .tool(riglr_solana_tools::SwapTokens)
        .build();

    // Execute within signer context
    SignerContext::new(&signer).execute(async {
        let response = agent
            .prompt("What's my SOL balance and can you swap 0.1 SOL to USDC?")
            .await?;
        
        println!("{}", response);
        Ok(())
    }).await
}
```

### 2. Creating Custom Tools

```rust
use riglr_macros::tool;
use riglr_core::signer::SignerContext;

/// Get the current price of a token from CoinGecko
/// 
/// # Parameters
/// - token_id: The CoinGecko ID of the token (e.g., "bitcoin", "ethereum")
/// - vs_currency: The currency to quote the price in (default: "usd")
#[tool]
async fn get_token_price(
    token_id: String,
    vs_currency: Option<String>,
) -> Result<f64, TokenPriceError> {
    let currency = vs_currency.unwrap_or_else(|| "usd".to_string());
    
    // Implementation here - the macro handles all the Tool trait boilerplate
    let price = fetch_price_from_coingecko(&token_id, &currency).await?;
    Ok(price)
}
```

### 3. Multi-Chain Agent

```rust
use riglr_core::signer::{SignerContext, MultiChainSigner};
use rig::agent::AgentBuilder;

let multi_chain_signer = MultiChainSigner::new()
    .with_solana(solana_signer)
    .with_ethereum(ethereum_signer);

let agent = AgentBuilder::new(openai_client)
    .preamble("You are a multi-chain DeFi agent")
    .tool(riglr_solana_tools::SwapTokens)       // Solana DEX
    .tool(riglr_evm_tools::UniswapV3Swap)       // Ethereum DEX  
    .tool(riglr_cross_chain_tools::BridgeTokens) // Cross-chain
    .build();

SignerContext::new(&multi_chain_signer).execute(async {
    let response = agent
        .prompt("Bridge 100 USDC from Ethereum to Solana and swap to SOL")
        .await?;
    
    println!("{}", response);
    Ok(())
}).await
```

## Key Patterns

### SignerContext Usage

The `SignerContext` provides a thread-safe way to manage blockchain signers:

```rust
// Setting context for a block of operations
SignerContext::new(&signer).execute(async {
    // All tool calls within this block will use this signer
    let balance = get_sol_balance().await?;
    let tx_hash = swap_tokens("SOL", "USDC", 1.0).await?;
    Ok((balance, tx_hash))
}).await

// Tools automatically access the current signer
#[tool]
async fn swap_tokens(from: String, to: String, amount: f64) -> Result<String, SwapError> {
    let signer = SignerContext::current().await?;  // <- Automatic signer access
    // ... implementation
}
```

### Error Handling

riglr uses structured error types for reliable transaction processing:

```rust
use riglr_solana_tools::error::SolanaError;

match swap_result {
    Ok(tx_hash) => println!("Swap successful: {}", tx_hash),
    Err(SolanaError::InsufficientBalance { required, available }) => {
        println!("Insufficient balance: need {}, have {}", required, available);
    },
    Err(SolanaError::NetworkError { retry_after }) => {
        println!("Network error, retry after: {:?}", retry_after);
    },
    Err(SolanaError::SlippageTooHigh { expected, actual }) => {
        println!("Slippage too high: expected {}, got {}", expected, actual);
    },
}
```

### Testing Patterns

riglr tools are designed to be easily testable:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use riglr_core::signer::MockSigner;

    #[tokio::test]
    async fn test_token_swap() {
        let mock_signer = MockSigner::new()
            .with_balance("SOL", 10.0)
            .expect_transaction("swap", vec!["SOL", "USDC", "1.0"]);

        let result = SignerContext::new(&mock_signer).execute(async {
            swap_tokens("SOL".to_string(), "USDC".to_string(), 1.0).await
        }).await;

        assert!(result.is_ok());
        mock_signer.verify_expectations();
    }
}
```

## Development Workflow

### Adding a New Tool

1. **Define the tool function** with proper documentation
2. **Apply `#[tool]` macro** for automatic `rig` integration
3. **Use `SignerContext::current()`** for blockchain operations
4. **Handle errors appropriately** with structured error types
5. **Write comprehensive tests** including edge cases
6. **Add integration tests** with real blockchain interactions

### Contributing Guidelines

- **Keep tools stateless** - no global variables or persistent state
- **Use SignerContext** for all blockchain signing operations
- **Document thoroughly** - docstrings become tool descriptions
- **Test comprehensively** - unit tests, integration tests, edge cases
- **Follow Rust idioms** - prefer `Result<T, E>` over panics
- **Minimize dependencies** - only add what's necessary

## Integration Examples

### Building a Trading Bot

```rust
let trading_agent = AgentBuilder::new(openai_client)
    .preamble(r#"
        You are a professional crypto trading assistant. Always:
        1. Check balances before trading
        2. Analyze risks using on-chain data
        3. Use appropriate slippage tolerance
        4. Confirm transactions before execution
    "#)
    .tool(riglr_solana_tools::GetSolBalance)
    .tool(riglr_solana_tools::GetTokenBalance) 
    .tool(riglr_solana_tools::AnalyzeTokenRisk)
    .tool(riglr_solana_tools::SwapTokens)
    .tool(riglr_web_tools::GetTokenPrice)
    .build();
```

### Cross-Chain Portfolio Manager

```rust
let portfolio_agent = AgentBuilder::new(openai_client)
    .preamble("You manage a multi-chain DeFi portfolio")
    .tool(riglr_solana_tools::GetSolBalance)
    .tool(riglr_evm_tools::GetEthBalance)
    .tool(riglr_cross_chain_tools::BridgeTokens)
    .tool(riglr_hyperliquid_tools::GetPositions)
    .tool(riglr_web_tools::GetPortfolioValue)
    .build();
```

### DeFi Analytics Agent

```rust
let analytics_agent = AgentBuilder::new(openai_client)
    .preamble("You provide deep DeFi market analysis")
    .tool(riglr_solana_tools::AnalyzeTokenRisk)
    .tool(riglr_web_tools::GetSocialSentiment)
    .tool(riglr_web_tools::GetMarketData)
    .tool(riglr_graph_memory::QueryKnowledgeGraph)
    .build();
```

## Performance Considerations

### Caching Strategy

- **RPC calls**: Cache blockchain queries with appropriate TTL
- **Price data**: Cache market data for 10-30 seconds
- **Static data**: Cache token metadata and contract addresses

### Rate Limiting

- **RPC providers**: Implement exponential backoff for rate limits
- **External APIs**: Respect API quotas and implement queuing
- **Concurrent operations**: Use semaphores to limit parallel requests

### Error Recovery

- **Transient failures**: Automatic retry with exponential backoff
- **Network issues**: Failover to alternative RPC endpoints
- **Slippage**: Dynamic slippage adjustment based on market conditions

## Security Best Practices

### Private Key Management

- **Never log** private keys or seed phrases
- **Use secure storage** for production deployments
- **Implement key rotation** for long-running services
- **Consider hardware wallets** for high-value operations

### Transaction Safety

- **Validate inputs** thoroughly before signing
- **Implement slippage protection** for all swaps
- **Use deadline parameters** to prevent stale transactions
- **Monitor for MEV attacks** in sandwich-prone operations

### Error Information

- **Sanitize error messages** to avoid leaking sensitive data
- **Log securely** with structured logging and appropriate levels
- **Implement alerting** for security-relevant events

## Deployment Patterns

### Development Environment

```rust
// Local development with test keys
let signer = LocalSolanaSigner::from_keypair(test_keypair);
```

### Staging Environment

```rust  
// Staging with environment variables
let signer = LocalSolanaSigner::from_env()?;
```

### Production Environment

```rust
// Production with secure key management
let signer = AWSKMSSigner::new(kms_client, key_id).await?;
```

## Future Roadmap

### Short Term (Next 3 months)

- **Enhanced testing**: Property-based testing for all tools
- **Performance optimization**: Benchmark-driven improvements
- **Documentation**: Interactive tutorials and video guides

### Medium Term (3-6 months)

- **Additional blockchains**: Cosmos, Polkadot ecosystem support
- **Advanced features**: MEV protection, flashloan integration
- **Enterprise features**: Multi-tenant signers, audit logging

### Long Term (6+ months)

- **AI-driven optimization**: ML-based trading strategies
- **Governance integration**: DAO voting and proposal tools
- **Institutional features**: Compliance reporting, risk management

## Getting Help

- **Documentation**: Check this architecture guide and crate-specific docs
- **Examples**: Browse `riglr-showcase` for real-world patterns
- **Issues**: Report bugs and feature requests on GitHub
- **Discussions**: Join the community for questions and ideas
- **Discord**: Real-time help from maintainers and community

---

*This architecture overview is designed to help you understand riglr's design principles and get started quickly. For detailed API documentation, see the individual crate docs. For examples and best practices, explore the `riglr-showcase` crate.*
