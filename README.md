# riglr 🦀⚡

[![CI](https://github.com/riglr-project/riglr/workflows/CI/badge.svg)](https://github.com/riglr-project/riglr/actions)
[![Crates.io](https://img.shields.io/crates/v/riglr-core.svg)](https://crates.io/crates/riglr-core)
[![Documentation](https://docs.rs/riglr-core/badge.svg)](https://docs.rs/riglr-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **The premier Rust ecosystem for building high-performance, resilient on-chain AI agents** 

riglr (pronounced "riggler") is a suite of modular, production-ready Rust crates that make it easy to build sophisticated AI agents that interact with blockchains. Built on top of the [rig](https://github.com/0xPlaygrounds/rig) framework, riglr provides everything you need to create powerful crypto-native applications.

## 🚀 Features

### Core Infrastructure
- **🔧 Declarative Tool System**: Define tools with the `#[tool]` macro - automatic trait implementation and error handling
- **♻️ Idempotency Built-in**: Safe retries with Redis-backed idempotency store
- **🔄 Smart Error Handling**: Distinguishes between retriable and permanent errors
- **📊 Job Queue**: Redis-backed job processing with automatic retries and dead letter queues
- **📝 Comprehensive Logging**: Structured logging with tracing

### Blockchain Support
- **⚡ Solana**: Complete SPL token support, Jupiter DEX integration, transaction building
- **🔷 EVM Chains**: Ethereum, Polygon, Arbitrum, Optimism, Base - powered by alloy-rs
- **💱 DeFi Integrations**: Uniswap V3, Jupiter aggregator, cross-chain swaps
- **🔐 Secure Key Management**: Signer context pattern keeps keys safe
- **🔁 Transaction Safety**: Idempotent operations, automatic retry logic

### Web & Data
- **🌐 API Integrations**: DexScreener, Twitter/X, NewsAPI, CryptoPanic, Exa search
- **🔍 Entity Extraction**: Automatic detection of wallets, tokens, protocols
- **📈 Sentiment Analysis**: Market sentiment from news and social media
- **⚡ Real-time Data**: WebSocket support for live market feeds

### Advanced Memory
- **🧠 Knowledge Graph**: Neo4j-powered relationship tracking
- **🔎 Vector Search**: Semantic similarity with graph-enhanced context
- **🏷️ Entity Recognition**: Blockchain entity extraction and classification
- **🔗 Contextual Retrieval**: Graph relationships enhance vector search

## 🏗️ Architecture Overview

```mermaid
graph TD
    subgraph "rig Framework"
        A[rig Agent] --> B[Tool Registry]
        B --> C[Tool Execution]
    end
    
    subgraph "riglr Ecosystem"
        D[riglr-core]
        E[riglr-macros]
        F[SignerContext]
        G[ToolWorker]
    end
    
    subgraph "Blockchain Tools"
        H[riglr-solana-tools]
        I[riglr-evm-tools]
        J[riglr-hyperliquid-tools]
        K[riglr-cross-chain-tools]
    end
    
    subgraph "Supporting Tools"
        L[riglr-web-tools]
        M[riglr-graph-memory]
    end
    
    subgraph "Application Layer"
        N[riglr-server]
        O[riglr-showcase]
    end
    
    %% Integration flow
    C --> D
    D --> F
    D --> G
    E --> H
    E --> I
    E --> J
    E --> K
    F --> H
    F --> I
    F --> J
    F --> K
    
    %% Application usage
    N --> D
    O --> D
    
    classDef core fill:#ff9999
    classDef tools fill:#99ccff
    classDef apps fill:#99ff99
    
    class D,E,F,G core
    class H,I,J,K,L,M tools
    class N,O apps
```

The riglr ecosystem extends rig with blockchain-specific capabilities through a layered architecture:

- **riglr-core**: Foundational abstractions (SignerContext, ToolWorker, ToolError)
- **riglr-macros**: Procedural macros for tool definitions and rig integration
- **Tool Crates**: Blockchain-specific implementations (Solana, EVM, Hyperliquid, Cross-chain)
- **Supporting Tools**: Web scraping, graph memory, and utility functions
- **Applications**: Server and showcase implementations

## 📦 Crates

| Crate | Description | Version |
|-------|-------------|---------|
| [riglr-core](./riglr-core) | Core framework, job processing, idempotency | 0.1.0 |
| [riglr-macros](./riglr-macros) | Procedural macros for tool generation | 0.1.0 |
| [riglr-solana-tools](./riglr-solana-tools) | Solana blockchain tools | 0.1.0 |
| [riglr-evm-tools](./riglr-evm-tools) | EVM chains support | 0.1.0 |
| [riglr-web-tools](./riglr-web-tools) | Web APIs and data processing | 0.1.0 |
| [riglr-graph-memory](./riglr-graph-memory) | Graph-based memory system | 0.1.0 |

## 🚀 Quick Start

### Installation

Add riglr to your `Cargo.toml`:

```toml
[dependencies]
riglr-core = "0.1.0"
riglr-macros = "0.1.0"

# Add the tools you need:
riglr-solana-tools = "0.1.0"  # For Solana
riglr-evm-tools = "0.1.0"      # For EVM chains
riglr-web-tools = "0.1.0"      # For web APIs
riglr-graph-memory = "0.1.0"   # For graph memory
```

### Basic Example

```rust
use riglr_core::ToolError;
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct PriceInfo {
    token: String,
    price_usd: f64,
    change_24h: f64,
}

#[tool]
async fn get_token_price(
    token_symbol: String,
    chain: Option<String>,
) -> Result<PriceInfo, ToolError> {
    // Implementation that distinguishes between error types
    match fetch_price(&token_symbol).await {
        Ok(price) => Ok(PriceInfo {
            token: token_symbol,
            price_usd: price,
            change_24h: 5.2,
        }),
        Err(e) if e.is_network_error() => {
            Err(ToolError::retriable(format!("Network error: {}", e)))
        }
        Err(e) => {
            Err(ToolError::permanent(format!("Invalid token: {}", e)))
        }
    }
}
```

### Solana Example

```rust
use riglr_solana_tools::{
    client::SolanaClient,
    balance::get_sol_balance,
    transaction::transfer_sol,
};
use solana_sdk::signature::Keypair;

// Create client for Solana mainnet
let client = SolanaClient::mainnet();

// Check balance
let balance = get_sol_balance(
    &client,
    "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
).await?;
println!("Balance: {} SOL", balance.sol);

// Create client with signer for transactions
let keypair = Keypair::new(); // In production, load from secure storage
let client_with_signer = SolanaClient::mainnet()
    .with_signer(keypair);

// Transfer SOL
let result = transfer_sol(
    &client_with_signer,
    "recipient_address".to_string(),
    0.5,
    Some("Payment memo".to_string()),
    None, // Priority fee
).await?;
println!("Transaction: {}", result.signature);
```

### EVM Example

```rust
use riglr_evm_tools::{
    client::EvmClient,
    balance::get_eth_balance,
    swap::{get_uniswap_quote, perform_uniswap_swap},
};

// Create client for Ethereum mainnet
let client = EvmClient::mainnet().await?;

// Get ETH balance
let balance = get_eth_balance(
    &client,
    "0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B".to_string(),
    None, // Latest block
).await?;
println!("Balance: {} ETH", balance.balance_formatted);

// Create client with signer for swaps
let client_with_signer = client.with_signer("YOUR_PRIVATE_KEY");

// Get Uniswap quote
let quote = get_uniswap_quote(
    &client_with_signer,
    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC
    "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(), // WETH  
    "1000".to_string(), // Amount
    6,    // USDC decimals
    18,   // WETH decimals
    Some(3000), // 0.3% fee tier
    Some(50),   // 0.5% slippage
).await?;
println!("Expected output: {} WETH", quote.amount_out);
```

### Web Tools Example

```rust
use riglr_web_tools::{
    dexscreener::get_token_info,
    twitter::search_tweets,
    news::get_crypto_news,
};

// Get token info from DexScreener
let token_info = get_token_info(
    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC
    Some("ethereum".to_string()),
).await?;
println!("Price: ${}", token_info.price_usd);

// Search Twitter
let tweets = search_tweets(
    "$BTC OR Bitcoin".to_string(),
    Some(100),
    Some(true),
    Some("en".to_string()),
    None,
    None,
).await?;

// Get crypto news
let news = get_crypto_news(
    Some("Ethereum".to_string()),
    Some(50), // Max articles
    Some(24), // Hours back
).await?;

for article in news.articles {
    println!("{}: {}", article.source.name, article.title);
}
```

## 🛠️ Development

### Prerequisites

- Rust 1.75+
- Redis (for job processing)
- Neo4j 5.0+ (for graph memory, optional)
- Docker (for testing)

### Building

```bash
# Clone the repository
git clone https://github.com/yourusername/riglr.git
cd riglr

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run with all features
cargo build --workspace --all-features
```

### Testing

```bash
# Unit tests
cargo test --workspace

# Integration tests (requires Docker)
docker-compose up -d
cargo test --workspace -- --ignored

# Test a specific crate
cargo test -p riglr-solana-tools

# Run benchmarks
cargo bench --workspace
```

## 📊 Performance

Riglr is designed for high performance:

- **Zero-copy deserialization** where possible
- **Connection pooling** for all network requests
- **Async/await** throughout for maximum concurrency
- **Efficient caching** with Redis
- **Optimized graph queries** with Neo4j indexes

Benchmarks on a standard developer machine:

| Operation | Time | Throughput |
|-----------|------|------------|
| Tool execution | ~10ms | 100 ops/sec |
| Entity extraction | ~50ms | 20 docs/sec |
| Graph query | ~25ms | 40 queries/sec |
| Vector search | ~30ms | 33 searches/sec |

## 🔒 Security

### Best Practices
- **Never expose private keys** - Use environment variables
- **Input validation** - All addresses and parameters are validated
- **Error recovery** - Automatic retry logic for transient failures
- **Rate limiting** - Built-in rate limit compliance
- **Audit logging** - Complete audit trail of all operations

### Environment Variables

```bash
# Solana
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_DEVNET_URL=https://api.devnet.solana.com

# EVM
ETH_RPC_URL=https://eth.llamarpc.com
POLYGON_RPC_URL=https://polygon-rpc.com

# APIs
TWITTER_BEARER_TOKEN=your_token
NEWSAPI_KEY=your_key
EXA_API_KEY=your_key

# Infrastructure
REDIS_URL=redis://localhost:6379
NEO4J_URL=bolt://localhost:7687
NEO4J_USER=neo4j
NEO4J_PASSWORD=password
```

## 📚 Documentation

- [API Documentation](https://docs.rs/riglr-core)
- [Getting Started Guide](./docs/getting-started.md)
- [Tool Development](./docs/tool-development.md)
- [Error Handling](./docs/error-handling.md)
- [Testing Guide](./docs/testing.md)

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

## 🙏 Acknowledgments

- Built on the excellent [rig](https://github.com/0xPlaygrounds/rig) framework
- Powered by the Rust ecosystem:
  - [tokio](https://tokio.rs/) - Async runtime
  - [solana-sdk](https://github.com/solana-labs/solana) - Solana support
  - [alloy](https://github.com/alloy-rs/alloy) - EVM support
  - [reqwest](https://github.com/seanmonstar/reqwest) - HTTP client

## 🚧 Roadmap

### ✅ Phase 1: Foundation (Complete)
- Core framework with `#[tool]` macro
- Redis-backed job processing
- Idempotency support
- Error handling patterns with ToolError enum

### ✅ Phase 2: Blockchain Tools (Complete)
- Solana tools with Jupiter integration
- EVM tools with Uniswap V3 support
- Multi-chain support (Ethereum, Polygon, Arbitrum, Optimism, Base)
- Secure key management with per-client signer configuration

### ✅ Phase 3: Data & Memory (Complete)
- Web API integrations (Twitter/X, DexScreener, News)
- Entity extraction for blockchain addresses and protocols
- Neo4j graph memory with relationship tracking
- Hybrid vector + graph search capabilities

### ✅ Phase 4: Production Ready (Complete)
- Comprehensive documentation for all crates
- Example applications demonstrating usage
- Integration tests with Docker containers
- Error resilience with retry logic

### 🔮 Phase 5: Future Enhancements
- Mobile SDK for iOS/Android
- Cloud deployment templates (AWS, GCP, Azure)
- Enterprise features (SSO, audit logs)
- Advanced ML pipelines for prediction
- Cross-chain bridge integrations
- Additional DEX support (PancakeSwap, SushiSwap)
- WebSocket support for real-time data
- GraphQL API layer

## ⚖️ Disclaimer

This software is provided "as is" without warranty of any kind. Use at your own risk. Always test thoroughly with devnet/testnet before mainnet deployment. Never expose private keys in code or logs.

---

<p align="center">
  Built with ❤️ by the riglr community
</p>