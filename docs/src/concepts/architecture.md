# Core Architecture

riglr is a comprehensive blockchain toolkit designed to seamlessly extend the `rig` AI framework with powerful, modular tools for on-chain operations. This guide explores the architecture and design decisions that make riglr production-ready.

## Guiding Principle

riglr follows a simple philosophy: **build a library for `rig`, not an application**. This ensures maximum flexibility and composability for developers building AI-powered blockchain agents. 

Each crate in the riglr ecosystem is a composable building block that can be used independently or combined to create powerful applications. Whether you need just the Solana tools, the full agent coordination system, or anything in between, you only pull in what you need. This library-first approach means riglr enhances and extends rig's capabilities without replacing or wrapping them.

## Architecture Layers

```
┌──────────────────────────────────────────────────────────────────┐
│                        Application Layer                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │riglr-agents │  │riglr-streams│  │riglr-indexer│              │
│  │             │  │             │  │             │              │
│  │• Multi-agent│  │• Real-time  │  │• Data       │              │
│  │  coordination  │  streaming  │  │  pipelines  │              │
│  │• Task routing  │• Operators  │  │• Storage    │              │
│  └─────────────┘  └─────────────┘  └─────────────┘              │
└──────────────────────────────────────────────────────────────────┘
                                │
┌──────────────────────────────────────────────────────────────────┐
│                         Core Framework                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌──────────┐│
│  │  rig-core   │  │ riglr-core  │  │riglr-macros │  │riglr-    ││
│  │             │◄─┤             │  │             │  │config    ││
│  │• AgentBuilder  │• SignerContext │• #[tool]    │  │          ││
│  │• Tool trait │  │• Transaction│  │  macro      │  │• Unified ││
│  │• Multi-turn │  │  Signer     │  │• Tool impl  │  │  config  ││
│  └─────────────┘  └─────────────┘  └─────────────┘  └──────────┘│
└──────────────────────────────────────────────────────────────────┘
                                │
┌──────────────────────────────────────────────────────────────────┐
│                         Tool Libraries                           │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐             │
│  │riglr-solana- │ │ riglr-evm-   │ │ riglr-web-   │             │
│  │tools         │ │ tools        │ │ tools        │             │
│  │• Pump.fun    │ │• Uniswap     │ │• DexScreener │             │
│  │• Jupiter     │ │• 1inch       │ │• CoinGecko   │             │
│  │• Risk tools  │ │• ENS         │ │• Twitter     │             │
│  └──────────────┘ └──────────────┘ └──────────────┘             │
│                                                                   │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐             │
│  │riglr-cross-  │ │riglr-hyper-  │ │ riglr-graph- │             │
│  │chain-tools   │ │liquid-tools  │ │ memory       │             │
│  │• LiFi bridge │ │• Perpetuals  │ │• Neo4j       │             │
│  │• Multi-chain │ │• Portfolio   │ │• Knowledge   │             │
│  └──────────────┘ └──────────────┘ └──────────────┘             │
└──────────────────────────────────────────────────────────────────┘
                                │
┌──────────────────────────────────────────────────────────────────┐
│                      Foundation Components                       │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐             │
│  │riglr-events- │ │riglr-solana- │ │riglr-auth    │             │
│  │core          │ │events        │ │              │             │
│  │• Event base  │ │• Solana      │ │• Multi-      │             │
│  │• Streaming   │ │  parsers     │ │  provider    │             │
│  └──────────────┘ └──────────────┘ └──────────────┘             │
└──────────────────────────────────────────────────────────────────┘
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

### Application Layer

**riglr-agents**
- Agent coordination: Build distributed networks of specialized agents
- Task routing: Intelligent routing strategies for multi-agent systems
- Inter-agent communication: Message passing and coordination protocols
- Registry pattern: Dynamic agent discovery and registration

**riglr-streams**
- StreamManager: Centralized management of real-time data streams
- Stream operators: Map, filter, batch, and transform stream data
- Multi-source support: Connect to Geyser, WebSockets, exchanges
- Backpressure handling: Graceful handling of high-throughput data

**riglr-indexer**
- Pipeline architecture: Ingester → Processor → Storage
- Custom processors: Build domain-specific indexing logic
- Storage backends: PostgreSQL, Redis, and custom adapters
- Real-time indexing: Process blockchain events as they happen

### Core Framework

**riglr-core**
- SignerContext: Thread-local signer management
- TransactionSigner: Trait for blockchain signing implementations
- Error handling: Structured error types for reliable transaction processing

**riglr-macros**
- #[tool] macro: Generates complete `rig::tool::Tool` implementations
- Reduces boilerplate from ~30 lines to a single attribute
- Automatically extracts documentation for tool schemas

**riglr-config**
- Unified configuration: Single source of truth for all settings
- Fail-fast validation: Catch configuration errors at startup
- Chain management: Standardized `RPC_URL_{CHAIN_ID}` convention
- Environment-based: Seamless development to production transition

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

### Foundation Components

**riglr-events-core**
- Event base types: Common interfaces for all event sources
- Stream primitives: Core streaming abstractions
- Protocol registry: Dynamic protocol parser registration

**riglr-solana-events**
- Protocol parsers: Raydium, Pump.fun, Jupiter, and more
- Zero-copy parsing: High-performance event deserialization
- Pipeline support: Integration with riglr-indexer pipelines

**riglr-auth**
- Multi-provider support: Privy, Web3Auth, Magic.link
- Session management: Secure token handling and refresh
- Wallet connection: Seamless wallet integration for web apps

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