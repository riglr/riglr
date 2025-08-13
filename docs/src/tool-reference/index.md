# Tool Reference

This section provides a detailed, auto-generated reference for every tool available across the official `riglr` crates. Each entry includes the tool's description, parameters, and usage examples.

## How This Reference is Generated

The documentation for each tool is generated directly from the Rust source code. The `#[tool]` macro extracts information from a function's signature and its doc comments to build a structured schema.

- The function's main doc comment becomes the **tool's description**
- Doc comments on each parameter become the **parameter descriptions**
- Parameter types and names are extracted from the function signature

This ensures that the documentation is always in sync with the implementation.

## Available Tool Sets

### Core Tools

#### [`riglr-core`](riglr-core.md)
Foundation utilities and abstractions used across all riglr crates.
- Signer context management
- Error handling utilities
- Job processing tools

### Blockchain Tools

#### [`riglr-solana-tools`](riglr-solana-tools.md)  
Comprehensive Solana blockchain integration.
- **Token Operations**: Balance queries, transfers, token creation
- **DeFi Protocols**: Jupiter swaps, Raydium pools, Pump.fun trading
- **Risk Analysis**: RugCheck integration, holder analysis
- **Event Monitoring**: Real-time transaction and event parsing

#### [`riglr-evm-tools`](riglr-evm-tools.md)
Ethereum and EVM-compatible chain support.
- **DeFi Protocols**: Uniswap V2/V3, 1inch aggregator
- **Token Standards**: ERC-20, ERC-721, ERC-1155
- **ENS Integration**: Domain resolution and reverse lookups
- **Multi-chain**: Support for any EVM chain via RPC_URL_{CHAIN_ID}

### Data & Analytics Tools

#### [`riglr-web-tools`](riglr-web-tools.md)
External data source integrations.
- **Price Data**: CoinGecko, DexScreener APIs
- **Social Analytics**: Twitter sentiment, social metrics
- **News & Events**: Crypto news aggregation
- **Market Data**: Real-time and historical price feeds

### Specialized Tools

#### [`riglr-cross-chain-tools`](riglr-cross-chain-tools.md)
Cross-blockchain operations and bridging.
- **Bridge Protocols**: Li.Fi integration
- **Cross-chain Swaps**: Atomic swaps between chains
- **Portfolio Tracking**: Multi-chain balance aggregation

#### [`riglr-hyperliquid-tools`](riglr-hyperliquid-tools.md)
Perpetual futures and derivatives trading.
- **Trading Operations**: Market/limit orders, position management
- **Risk Management**: Stop-loss, take-profit automation
- **Analytics**: PnL tracking, funding rates

#### [`riglr-graph-memory`](riglr-graph-memory.md)
Knowledge graph and RAG capabilities.
- **Graph Operations**: Entity extraction and linking
- **Semantic Search**: Vector similarity search
- **Memory Management**: Long-term context storage

## Using the Tools

All tools follow a consistent pattern and can be used with the `rig` framework:

```rust
use rig::agent::Agent;
use riglr_solana_tools::{get_sol_balance, swap_tokens};
use riglr_core::signer::SignerContext;

// Add tools to your agent
let agent = Agent::builder()
    .tools(vec![get_sol_balance, swap_tokens])
    .model(model)
    .build();

// Execute within signer context
SignerContext::with_signer(signer, async {
    let response = agent.prompt("What's my SOL balance?").await?;
    Ok(response)
}).await?;
```

## Tool Schemas

Each tool exposes a JSON schema that describes its parameters:

```rust
let schema = get_sol_balance.schema();
println!("{}", serde_json::to_string_pretty(&schema)?);
```

Output:
```json
{
  "name": "get_sol_balance",
  "description": "Get SOL balance for a wallet address",
  "parameters": {
    "type": "object",
    "properties": {
      "address": {
        "type": "string",
        "description": "The Solana wallet address"
      }
    },
    "required": ["address"]
  }
}
```

## Error Handling

All tools return `Result<T, ToolError>` where errors are classified as:
- **Retriable**: Temporary failures (network issues, rate limits)
- **Permanent**: Unrecoverable errors (invalid inputs, insufficient funds)

See the [Error Handling Philosophy](../concepts/error-handling.md) for detailed patterns.

## Auto-Generation Process

The tool reference is automatically generated during the documentation build:

1. `rustdoc` generates JSON output for all crates
2. A custom parser extracts functions with the `#[tool]` attribute
3. Documentation is generated from doc comments and signatures
4. Markdown files are created for each crate

This process runs in CI/CD to ensure documentation stays current.

## Contributing New Tools

To add a new tool:

1. Create a function with the `#[tool]` attribute
2. Add comprehensive doc comments
3. Use `SignerContext::current()` for blockchain operations
4. Return `Result<T, ToolError>` with proper error classification
5. Add tests including error cases

The documentation will be automatically generated on the next build.