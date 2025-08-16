# Introduction to riglr

**riglr** (pronounced "riggler") is a production-ready Rust ecosystem for building enterprise-grade on-chain AI agents. More than just a blockchain toolkit, riglr is a complete ecosystem that extends the powerful `rig` AI framework with comprehensive tools for blockchain interaction, real-time data streaming, intelligent coordination, and secure transaction management.

Whether you're building a sophisticated DeFi trading bot, a market analysis agent, a cross-chain portfolio manager, or a distributed network of autonomous agents, riglr provides the secure, performant, and developer-friendly components you need.

### Key Features

- **Agent Coordination** (`riglr-agents`): Build distributed networks of specialized agents with intelligent task routing and inter-agent communication.
- **Real-Time Streaming** (`riglr-streams`): Process high-throughput blockchain events and market data with powerful stream operators.
- **Data Indexing** (`riglr-indexer`): Production-grade indexing service with customizable pipelines for ingesting, processing, and storing on-chain data.
- **Unified Configuration** (`riglr-config`): Fail-fast configuration system with standardized chain management and environment-based settings.
- **Declarative Tool System**: Define complex blockchain operations with the simple `#[tool]` macro.
- **Secure by Design**: The `SignerContext` pattern ensures cryptographic keys are handled safely and isolated between requests.
- **Production-Ready**: With fail-fast configuration, robust error handling, and no mock implementations, riglr is built for real-world use.
- **Multi-Chain Native**: Built-in support for Solana, Ethereum, and any EVM-compatible chain.
- **Rich Data Integration**: Tools for web APIs, social media, and market data sources are included out of the box.

### Who is this for?

- **Developers** building AI-powered applications that interact with blockchains.
- **DeFi Teams** creating automated trading, monitoring, or portfolio management systems.
- **Researchers** exploring on-chain agent behavior and autonomous systems.

This documentation will guide you through setting up your first agent, understanding core concepts, and deploying your application to production.