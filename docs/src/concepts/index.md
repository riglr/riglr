# Conceptual Guides

These guides dive deep into the core concepts and design patterns that power riglr. Understanding these concepts will help you build more robust and secure blockchain agents.

## Core Concepts

### [Core Architecture](architecture.md)
Learn about riglr's modular architecture, how it extends the `rig` framework, and the design decisions that make it production-ready.

### [The SignerContext Pattern](signer-context.md)
Understand how riglr securely manages cryptographic signers in concurrent environments without passing them through every function call.

### [Error Handling Philosophy](error-handling.md)
Explore riglr's approach to error classification, retriable vs permanent failures, and how to build resilient agents.

### [Event Parsing System](event-parsing.md)
Discover how riglr efficiently parses and interprets blockchain events, enabling real-time monitoring and reaction to on-chain activity.

## Design Principles

riglr follows several key design principles:

1. **Security First**: All cryptographic operations are isolated and secure by default
2. **Fail Fast**: Configuration errors are caught at startup, not runtime
3. **No Mocks**: All tools use real blockchain/API interactions
4. **Extensibility**: New chains and tools can be added without modifying core code
5. **Developer Experience**: Simple macros and patterns reduce boilerplate

## Architecture Layers

riglr is organized into distinct layers:

- **Core Layer**: Fundamental abstractions and traits
- **Tool Layer**: Blockchain and web interaction tools
- **Framework Layer**: Web adapters and job processing
- **Application Layer**: Your custom agents and business logic

Each layer builds upon the previous one, creating a cohesive and powerful ecosystem for blockchain AI agents.