# Core Architecture

riglr is a comprehensive blockchain toolkit designed to seamlessly extend the `rig` AI framework with powerful, modular tools for on-chain operations. This guide explores the architecture and design decisions that make riglr production-ready.

## Guiding Principle

riglr follows a simple philosophy: **build a library for `rig`, not an application**. This ensures maximum flexibility and composability for developers building AI-powered blockchain agents.

## Architecture Layers

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
```

## Core Design Patterns

### 1. Modular Design

- **Blockchain-agnostic core** with specific tool implementations
- Each crate serves a specific purpose and can be used independently
- Minimal dependencies - only pull in what you need

### 2. SignerContext Pattern

The SignerContext provides thread-safe, secure access to cryptographic signers without passing them through every function call. [Learn more →](signer-context.md)

### 3. Stateless Tools

- Pure functions that can be easily tested and composed
- No global state or client parameters in tool signatures
- Tools "just work" by pulling signers from context

### 4. rig-First Integration

- Tools generate full `impl rig::tool::Tool` boilerplate via `#[tool]` macro
- Seamless integration with `AgentBuilder` pattern
- Leverage `rig`'s native features instead of replacing them

## Component Overview

### Core Components

**riglr-core**
- SignerContext: Thread-local signer management
- TransactionSigner: Trait for blockchain signing implementations
- Error handling: Structured error types for reliable transaction processing

**riglr-macros**
- #[tool] macro: Generates complete `rig::tool::Tool` implementations
- Reduces boilerplate from ~30 lines to a single attribute
- Automatically extracts documentation for tool schemas

### Blockchain Tools

**riglr-solana-tools**
- DeFi protocols: Jupiter swaps, Pump.fun trading
- Risk analysis: RugCheck.xyz integration
- Advanced orders: Conditional and limit orders

**riglr-evm-tools**
- DeFi protocols: Uniswap, 1inch integration
- ENS support: Domain resolution and management
- Multi-chain: Any EVM-compatible chain

**riglr-web-tools**
- Price feeds: CoinGecko, DexScreener
- Social data: Twitter analytics
- Market data: Real-time quotes and historical data

### Specialized Components

**riglr-cross-chain-tools**
- Bridge integrations: LiFi protocol support
- Cross-chain swaps: Atomic swaps and bridge operations
- Multi-chain portfolio: Unified balance tracking

**riglr-hyperliquid-tools**
- Perpetual futures: Position management and trading
- Portfolio analytics: PnL tracking and risk metrics
- Advanced orders: Stop-loss, take-profit automation

**riglr-graph-memory**
- Neo4j backend: Scalable graph database storage
- Entity extraction: Automatic knowledge graph construction
- Semantic search: Advanced query capabilities for AI agents

## Security Architecture

### Private Key Management

riglr implements multiple layers of security for handling cryptographic keys:

1. **Isolation**: Keys are confined to SignerContext scope
2. **No Logging**: Private keys are never logged or exposed
3. **Automatic Cleanup**: Keys are cleared when context exits
4. **Thread Safety**: Each async task has its own isolated context

### Transaction Safety

Every transaction follows these safety checks:

1. **Input Validation**: All parameters are validated before signing
2. **Slippage Protection**: Configurable slippage tolerance for swaps
3. **Deadline Parameters**: Transactions expire to prevent stale execution
4. **Error Recovery**: Automatic retry with exponential backoff for transient failures

## Performance Considerations

### Caching Strategy

- **RPC calls**: Intelligent caching with appropriate TTL
- **Price data**: Short-lived cache for market data (10-30 seconds)
- **Static data**: Long-lived cache for token metadata

### Concurrency Control

- **Rate limiting**: Respect API quotas with built-in limiters
- **Connection pooling**: Reuse RPC connections efficiently
- **Semaphores**: Limit parallel operations to prevent overload

## Extensibility

### Adding New Blockchains

Support for new chains can be added without modifying core code:

1. Implement the `TransactionSigner` trait
2. Create chain-specific tools using `#[tool]` macro
3. Configure RPC endpoints via environment variables

### Custom Tool Development

Creating new tools follows a simple pattern:

```rust
use riglr_macros::tool;
use riglr_core::ToolError;

#[tool]
/// Your tool description becomes the schema
async fn my_custom_tool(
    /// Parameter descriptions are extracted
    param: String,
) -> Result<String, ToolError> {
    // Implementation here
}
```

## Production Deployment

### Development → Production Path

1. **Development**: Local keys and test networks
2. **Staging**: Environment variables and devnets
3. **Production**: KMS integration and mainnets

### Monitoring and Observability

- Structured logging with appropriate levels
- Metrics collection for performance monitoring
- Error tracking and alerting for critical issues

## Next Steps

- Explore [The SignerContext Pattern](signer-context.md) in detail
- Learn about [Error Handling Philosophy](error-handling.md)
- Understand the [Event Parsing System](event-parsing.md)
- Check out the [Tool Reference](../tool-reference/index.md) for available tools