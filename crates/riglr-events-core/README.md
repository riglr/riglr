# riglr-events-core

[![Crates.io](https://img.shields.io/crates/v/riglr-events-core.svg)](https://crates.io/crates/riglr-events-core)
[![Documentation](https://docs.rs/riglr-events-core/badge.svg)](https://docs.rs/riglr-events-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Core event processing abstractions and traits for the riglr ecosystem, providing a unified framework for blockchain event handling across multiple chains.

## Overview

riglr-events-core defines the foundational event processing infrastructure used throughout riglr. It provides chain-agnostic abstractions for parsing, processing, and reacting to blockchain events with high performance and reliability.

## Key Features

- **Chain-Agnostic Design**: Core abstractions work across Solana, EVM, and future chains
- **Event Parser Traits**: Standardized interface for implementing event parsers
- **Event Processing Pipeline**: Stream-based processing with backpressure
- **Pattern Matching**: Flexible event filtering and routing system
- **Performance Optimized**: Zero-copy parsing where possible
- **Type Safety**: Strongly typed event definitions with compile-time guarantees
- **Extensible Architecture**: Easy to add new event types and chains

## Quick Start

Add riglr-events-core to your `Cargo.toml`:

```toml
[dependencies]
riglr-events-core = "0.3.0"
riglr-core = "0.1.0"

# Optional: Enable chain-specific features
riglr-events-core = { version = "0.3.0", features = ["solana", "evm"] }
```

## Documentation

For comprehensive documentation, architecture details, and implementation guides, see: [docs/src/concepts/event-parsing.md](../docs/src/concepts/event-parsing.md)

The documentation covers:
- Event parser trait system
- Processing pipeline architecture
- Performance optimization techniques
- Chain-specific implementations
- Event pattern matching
- Integration with riglr tools
- Testing and benchmarking

## License

Licensed under MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)