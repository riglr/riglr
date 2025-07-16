# Introduction to riglr

**riglr** (pronounced "riggler") is a production-ready Rust ecosystem for building enterprise-grade on-chain AI agents. It extends the powerful `rig` AI framework with a comprehensive toolkit for blockchain interaction, data analysis, and secure transaction management.

Whether you're building a sophisticated DeFi trading bot, a market analysis agent, or a cross-chain portfolio manager, riglr provides the secure, performant, and developer-friendly components you need.

### Key Features

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