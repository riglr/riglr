# riglr-indexer

[![Crates.io](https://img.shields.io/crates/v/riglr-indexer.svg)](https://crates.io/crates/riglr-indexer)
[![Documentation](https://docs.rs/riglr-indexer/badge.svg)](https://docs.rs/riglr-indexer)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Production-grade blockchain indexing framework for high-throughput event processing and real-time data access.

## Overview

riglr-indexer provides a scalable, modular indexing service that processes blockchain events in real-time and stores them for efficient querying. Built on riglr-events-core, it supports multiple chains and protocols with parallel processing, persistent storage, and streaming capabilities.

## Key Features

- **High Throughput**: Process 10,000+ events/second with parallel workers
- **Multi-Chain Support**: Index events from Solana, EVM chains, and more
- **Persistent Storage**: PostgreSQL with optional ClickHouse for analytics
- **Real-time Streaming**: WebSocket API for live event updates
- **Horizontal Scaling**: Consistent hashing for distributed deployments
- **Comprehensive Metrics**: Built-in monitoring and health checks
- **Flexible Querying**: Rich query API with complex filtering

## Quick Start

Add riglr-indexer to your `Cargo.toml`:

```toml
[dependencies]
riglr-indexer = "0.1.0"
riglr-events-core = "0.3.0"
```

Run as a service:

```rust
use riglr_indexer::prelude::*;

#[tokio::main]
async fn main() -> Result<(), IndexerError> {
    let config = IndexerConfig::from_env()?;
    let mut indexer = IndexerService::new(config).await?;
    indexer.start().await?;
    Ok(())
}
```

## Documentation

For comprehensive documentation, architecture details, and configuration guides, see: [docs/src/api-reference/riglr-indexer.md](../docs/src/api-reference/riglr-indexer.md)

The documentation covers:
- Architecture overview and components
- Configuration and deployment options
- Storage layer design
- API and streaming endpoints
- Performance tuning
- Integration with riglr-streams
- Common use cases and patterns

## License

Licensed under MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)