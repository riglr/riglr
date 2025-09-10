# riglr-showcase

[![Crates.io](https://img.shields.io/crates/v/riglr-showcase.svg)](https://crates.io/crates/riglr-showcase)
[![Documentation](https://docs.rs/riglr-showcase/badge.svg)](https://docs.rs/riglr-showcase)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Interactive showcase application demonstrating the full capabilities of the riglr ecosystem through practical examples and ready-to-use agents.

## Overview

riglr-showcase is a comprehensive demonstration application that showcases how to build sophisticated blockchain automation systems using riglr. It includes working examples of trading bots, portfolio managers, arbitrage systems, and multi-agent coordination patterns.

### Architecture Patterns

The showcase demonstrates modern architectural patterns:
- **ApplicationContext**: Centralized dependency injection for managing shared resources
- **ToolCallingAgentBuilder**: Builder pattern for creating agents with integrated tools
- **StreamManagerBuilder**: Builder pattern for stream processing setup
- **SignerContext**: Secure, multi-tenant blockchain operations
- **Task Objects**: Structured task representation for agent operations

## Key Features

- **Interactive CLI**: User-friendly command-line interface for exploring riglr capabilities
- **Pre-built Agents**: Ready-to-use trading, research, and automation agents
- **Live Examples**: Real-world examples including DEX trading, yield farming, and arbitrage
- **Multi-Chain Demos**: Examples spanning Solana, Ethereum, and cross-chain operations
- **Stream Processing**: Advanced examples using riglr-streams for real-time data
- **Authentication**: Web3 authentication integration examples
- **Educational**: Well-documented code designed for learning

## Quick Start

Add riglr-showcase to your `Cargo.toml`:

```toml
[dependencies]
riglr-showcase = "0.3.0"

# Optional features
riglr-showcase = { version = "0.3.0", features = ["web-server", "cross-chain", "hyperliquid"] }
```

Run the showcase:

```bash
# Set up environment variables
cp .env.example .env
# Edit .env with your configuration

# Run the interactive showcase
cargo run --bin riglr-showcase
```

## Examples

The showcase includes numerous examples in the `examples/` directory:

### Agent Examples
- `trading_agent.rs` - Trading agent using ToolCallingAgentBuilder and Task objects
- Multi-agent coordination patterns with AgentDispatcher

### Streaming Examples
- `streaming/basic/simple_event_stream.rs` - Basic event streaming with StreamManagerBuilder
- `streaming/basic/filtered_events.rs` - Advanced event filtering and routing
- `streaming/advanced/hft_bot.rs` - High-frequency trading bot implementation
- `streaming/advanced/yield_monitor.rs` - DeFi yield farming monitor

### Command-Line Demos
The CLI provides interactive demos for:
- Solana operations (balance checking, token transfers)
- EVM operations (multi-chain support)
- Web tools integration (search, analysis)
- Cross-chain operations
- Graph memory systems

## Documentation

For comprehensive documentation and tutorials, see: [docs/src/tutorials/](../docs/src/tutorials/)

The documentation covers:
- Getting started with riglr-showcase
- Building custom agents
- Integrating with exchanges and protocols
- Stream processing patterns
- Authentication and security
- Deployment strategies

## License

Licensed under MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)