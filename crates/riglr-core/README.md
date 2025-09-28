# riglr-core

[![Crates.io](https://img.shields.io/crates/v/riglr-core.svg)](https://crates.io/crates/riglr-core)
[![Documentation](https://docs.rs/riglr-core/badge.svg)](https://docs.rs/riglr-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

The foundational crate for the riglr ecosystem, providing core abstractions for multi-chain tool orchestration and execution within the rig framework.

## Overview

riglr-core is designed as a **chain-agnostic foundation layer** that provides pure abstractions for multi-chain tool orchestration. This crate contains no concrete blockchain implementations or SDK dependencies, ensuring clean separation of concerns.

## Key Features

- **ApplicationContext**: Dependency injection and RPC provider management
- **SignerContext**: Thread-safe signer management for transactions
- **Extension System**: Type-safe dependency injection without circular references
- **Tool Execution**: Resilient execution with retries and error classification
- **Idempotency**: Built-in duplicate operation prevention
- **Metrics**: Comprehensive monitoring and observability

## Quick Start

Add riglr-core to your `Cargo.toml`:

```toml
[dependencies]
riglr-core = "0.1.0"
riglr-config = "0.1.0"
```

## Documentation

For comprehensive documentation, architecture patterns, and security guidance, see: [docs/src/concepts/architecture/core-patterns.md](../docs/src/concepts/architecture/core-patterns.md)

The documentation covers:
- Production security best practices
- Core architectural patterns (SignerContext, ApplicationContext)
- Tool system design and implementation
- Error handling and resilience patterns
- Developer experience with #[tool] macro
- Testing and quality patterns
- Integration with blockchain-specific crates

## Where to Find Implementations

- **Solana Signers**: See `riglr-solana-tools` for `LocalSolanaSigner` and Solana-specific tools
- **EVM Signers**: See `riglr-evm-tools` for `LocalEvmSigner` and EVM-specific tools
- **Concrete Examples**: Check the examples in each tools crate for usage patterns

## License

Licensed under MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)