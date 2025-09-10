# API Reference

Complete API documentation for all riglr crates.

## Core Crates

- **[riglr-core](./riglr-core.md)** - Core framework, job processing, idempotency, signer traits
- **[riglr-config](./riglr-config.md)** - Unified, hierarchical configuration management
- **[riglr-macros](./riglr-macros.md)** - Procedural macros (`#[tool]`) for rapid tool generation

## Blockchain Integration

- **[riglr-solana-tools](./riglr-solana-tools.md)** - Tools for interacting with the Solana blockchain
- **[riglr-solana-events](./riglr-solana-events.md)** - High-performance Solana event parsing
- **[riglr-evm-tools](./riglr-evm-tools.md)** - Tools for EVM-compatible chains
- **[riglr-evm-common](./riglr-evm-common.md)** - Common EVM utilities
- **[riglr-cross-chain-tools](./riglr-cross-chain-tools.md)** - Cross-chain bridge integration
- **[riglr-hyperliquid-tools](./riglr-hyperliquid-tools.md)** - Tools for the Hyperliquid perpetuals DEX

## Data & Coordination

- **[riglr-agents](./riglr-agents.md)** - Multi-agent coordination system
- **[riglr-streams](./riglr-streams.md)** - Real-time event streaming
- **[riglr-indexer](./riglr-indexer.md)** - Production-grade blockchain indexing
- **[riglr-events-core](./riglr-events-core.md)** - Core event processing abstractions
- **[riglr-graph-memory](./riglr-graph-memory.md)** - Graph-based memory system with Neo4j

## Application Layer

- **[riglr-server](./riglr-server.md)** - Turnkey, production-ready HTTP server
- **[riglr-auth](./riglr-auth.md)** - Authentication and signer factories
- **[riglr-web-tools](./riglr-web-tools.md)** - Tools for web APIs
- **[riglr-web-adapters](./riglr-web-adapters.md)** - Web service adapters
- **[riglr-showcase](./riglr-showcase.md)** - Example agents and demonstrations

## Generating Full API Documentation

To generate complete API documentation with all types and functions:

```bash
cargo doc --workspace --no-deps --open
```

This will build and open the full rustdoc documentation in your browser.
