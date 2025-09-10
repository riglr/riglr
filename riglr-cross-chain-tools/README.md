# riglr-cross-chain-tools

[![Crates.io](https://img.shields.io/crates/v/riglr-cross-chain-tools.svg)](https://crates.io/crates/riglr-cross-chain-tools)
[![Documentation](https://docs.rs/riglr-cross-chain-tools/badge.svg)](https://docs.rs/riglr-cross-chain-tools)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Cross-chain bridging and interoperability tools for the riglr ecosystem, enabling seamless token transfers and multi-chain operations through LiFi integration.

## Overview

riglr-cross-chain-tools provides powerful cross-chain functionality for blockchain agents, allowing them to bridge tokens across different networks including Ethereum, Polygon, Arbitrum, Solana, and more. Built on the riglr-core foundation with full rig-core integration for intelligent routing decisions.

## Key Features

- **LiFi Integration**: Comprehensive bridge aggregation for optimal routing
- **Multi-Chain Support**: Bridge between EVM chains, Solana, and more
- **Intelligent Routing**: AI-powered route selection with rig-core
- **Transaction Management**: Automated tracking and retry mechanisms
- **Security First**: Maintains riglr's SignerContext security model
- **Type-Safe**: Strongly typed cross-chain transaction builders

## Quick Start

Add riglr-cross-chain-tools to your `Cargo.toml`:

```toml
[dependencies]
riglr-cross-chain-tools = "0.3.0"
riglr-core = "0.3.0"
riglr-config = "0.3.0"
```

## Documentation

For comprehensive documentation, examples, and integration guides, see: [docs/src/tutorials/cross-chain-portfolio-manager.md](../docs/src/tutorials/cross-chain-portfolio-manager.md)

The documentation covers:
- LiFi integration and configuration
- Cross-chain bridging patterns
- Route optimization strategies
- Transaction tracking and monitoring
- Error handling and recovery
- Security considerations
- Integration with riglr agents

## License

Licensed under MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)