# RIGLR 🦀⚡ - Production-Ready AI Agent Framework for Blockchain

[![CI](https://github.com/riglr-project/riglr/workflows/CI/badge.svg)](https://github.com/riglr-project/riglr/actions)
[![Crates.io](https://img.shields.io/crates/v/riglr-core.svg)](https://crates.io/crates/riglr-core)
[![Documentation](https://docs.rs/riglr-core/badge.svg)](https://docs.rs/riglr-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> ⚠️ **UNDER HEAVY DEVELOPMENT**: This project is being actively developed with frequent updates. APIs may change and things will probably break. Use with caution in production environments.

> **Production-ready Rust ecosystem for building enterprise-grade on-chain AI agents**

RIGLR (pronounced "riggler") is a modular Rust framework for building sophisticated, high-performance AI agents that interact with blockchains. It provides a comprehensive toolkit for everything from real-time event streaming and data indexing to secure transaction execution and multi-agent coordination.

## 🗺️ Quick Navigation

> **New to RIGLR?** Check out our comprehensive [Architecture Guide](./ARCHITECTURE.md) for a detailed map of all crates and how they fit together.

### What is RIGLR?
RIGLR is a modular framework organized into specialized crates:
- **Core Layer**: Foundation (`riglr-core`), code generation (`riglr-macros`), and unified configuration (`riglr-config`).
- **Blockchain Layer**: Tools for Solana (`riglr-solana-tools`), EVM chains (`riglr-evm-tools`), and cross-chain operations (`riglr-cross-chain-tools`).
- **Data & Coordination Layer**: Real-time event streaming (`riglr-streams`), data indexing (`riglr-indexer`), multi-agent systems (`riglr-agents`), and external web APIs (`riglr-web-tools`).
- **Application Layer**: Production server (`riglr-server`), pre-built agents (`riglr-showcase`), and authentication (`riglr-auth`).

See the [Architecture Guide](./ARCHITECTURE.md) for the complete crate dependency graph and detailed explanations.

## 🚀 Key Features

### Enterprise-Grade Reliability
- **Fail-Fast Configuration**: Centralized environment variable management (`riglr-config`) with startup validation.
- **Extensible Chain Support**: Add new EVM chains without code changes using the `RPC_URL_{CHAIN_ID}` convention.
- **Rich Error Handling**: Structured error types with source preservation for better debugging.
- **Production-Ready**: No mock implementations—all operations use real blockchain APIs and production patterns.

### Core Infrastructure
- **🔧 Declarative Tool System**: Define complex blockchain operations with a simple `#[tool]` macro.
- ** COORDINATION**: Build complex systems with multiple, specialized agents using `riglr-agents`.
- **⚡ REAL-TIME**: Process high-throughput, low-latency event streams with `riglr-streams`.
- **💾 INDEXING**: Create custom, high-performance data indexers with `riglr-indexer`.
- **🔐 SECURITY**: The `SignerContext` pattern ensures cryptographic keys are handled safely and isolated between requests.

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

---

<p align="center">
  Built with ❤️ by the riglr community
</p>