# riglr-solana-events

[![Crates.io](https://img.shields.io/crates/v/riglr-solana-events.svg)](https://crates.io/crates/riglr-solana-events)
[![Documentation](https://docs.rs/riglr-solana-events/badge.svg)](https://docs.rs/riglr-solana-events)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

High-performance Solana event parsing library providing specialized parsers for major protocols including Raydium, Jupiter, Pump.fun, and more.

## Overview

riglr-solana-events is a standalone, high-performance library for parsing Solana blockchain events. Originally built for riglr, it's designed to be used by any project needing reliable Solana event parsing. It provides battle-tested parsers for major DeFi protocols and can process thousands of transactions per second.

## Key Features

- **Protocol Coverage**: Parsers for Raydium, Jupiter, Pump.fun, Meteora, and more
- **High Performance**: Memory-mapped parsing with zero-copy where possible
- **Standalone**: Can be used independently of the riglr ecosystem
- **Type-Safe**: Strongly typed event definitions with Borsh serialization
- **Battle-Tested**: Used in production for high-frequency trading systems
- **Extensible**: Easy to add parsers for new protocols
- **Async Support**: Full async/await support with Tokio

## Quick Start

Add riglr-solana-events to your `Cargo.toml`:

```toml
[dependencies]
riglr-solana-events = "0.3.0"
solana-sdk = "1.18"
tokio = { version = "1", features = ["full"] }
```

## Supported Protocols

- **Raydium**: AMM swaps, liquidity events
- **Jupiter**: Aggregator swaps, route parsing
- **Pump.fun**: Token launches, bonding curve events
- **Meteora**: Dynamic pools, DLMM events
- **Orca**: Whirlpool swaps
- **And more**: Extensible architecture for custom protocols

## Documentation

For comprehensive documentation, parser implementations, and performance guides, see: [docs/src/concepts/event-parsing.md](../docs/src/concepts/event-parsing.md)

The documentation covers:
- Parser architecture and implementation
- Performance optimization techniques
- Adding custom protocol parsers
- Integration patterns
- Benchmarking and testing
- Real-world usage examples

## License

Licensed under MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)