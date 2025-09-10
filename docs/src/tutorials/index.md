# Tutorials

Learn how to build real-world blockchain agents with riglr through hands-on tutorials. Each tutorial walks you through building a complete, functional agent from scratch.

## Available Tutorials

### [Building a Solana Arbitrage Bot](solana-arbitrage-bot.md)
Learn how to build an automated arbitrage bot that monitors Solana DEXs for price discrepancies and executes profitable trades.

**You'll learn:**
- Setting up real-time event monitoring
- Calculating arbitrage opportunities
- Executing atomic swap sequences
- Managing slippage and MEV protection

### [Creating a Cross-Chain Portfolio Manager](cross-chain-portfolio-manager.md)
Build an intelligent agent that manages assets across multiple blockchains, automatically rebalancing and optimizing yields.

**You'll learn:**
- Multi-chain signer management
- Cross-chain bridge integration
- Portfolio rebalancing strategies
- Yield optimization techniques

### [Building a Pump.fun Trading Agent](pump-fun-trader.md)
Create an agent that trades on Pump.fun's bonding curves, implementing strategies like sniping new launches and riding momentum.

**You'll learn:**
- Bonding curve mechanics
- Event-driven trading strategies
- Risk management for volatile assets
- Social sentiment integration

## Prerequisites

Before starting these tutorials, you should:

1. Have Rust installed (1.75 or later)
2. Basic understanding of async Rust
3. Familiarity with blockchain concepts
4. Have completed the [Quick Start](../getting-started/quick-start.md) guide

## Tutorial Structure

Each tutorial follows a consistent structure:

1. **Overview**: What you'll build and why it's useful
2. **Architecture**: System design and component overview
3. **Implementation**: Step-by-step code walkthrough
4. **Testing**: How to test your agent safely
5. **Deployment**: Taking your agent to production
6. **Extensions**: Ideas for enhancing your agent

## Development Environment

All tutorials use the same development setup:

```bash
# Navigate to the showcase directory within the riglr repo
cd riglr-showcase

# Install dependencies for the examples
cargo build

# Set up environment
cp .env.example .env
# Edit .env with your API keys

# Start Redis for job processing
docker run -d --name redis -p 6379:6379 redis:alpine
```

## Safety First

⚠️ **Important**: These tutorials involve real blockchain transactions. Always:

- Start with devnet/testnet tokens
- Use small amounts when testing on mainnet
- Implement proper error handling
- Add monitoring and alerts
- Never expose private keys

## Getting Help

If you get stuck:

- Check the tutorial's troubleshooting section
- Review the [Conceptual Guides](../concepts/index.md)
- Ask in our Discord community
- Open an issue on GitHub

## Contributing Tutorials

Have an idea for a tutorial? We welcome contributions! See our [Contributing Guide](../contributing.md) for details on submitting new tutorials.