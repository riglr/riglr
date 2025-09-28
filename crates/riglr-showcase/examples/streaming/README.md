# RIGLR Streaming Examples

This directory contains runnable examples demonstrating the RIGLR streaming ecosystem in action.

## 📚 Documentation

For complete streaming documentation including architecture, operators, configuration, and best practices, see:
**[Real-Time Streaming Documentation](../../../docs/src/concepts/streams.md)**

## 🚀 Quick Start

### Prerequisites
```bash
# Install Rust and dependencies
cargo build --workspace

# Set up environment variables
cp .env.example .env
# Edit .env with your RPC endpoints and API keys
```

### Running Examples

#### Basic Examples
```bash
# Simple event streaming
cargo run --example simple_event_stream

# Advanced event filtering
cargo run --example filtered_events
```

#### Advanced Examples
```bash
# High-frequency trading bot (paper trading mode)
cargo run --example hft_bot

# DeFi yield farming monitor
cargo run --example yield_monitor
```

## 📁 Directory Structure

### `basic/`
- `simple_event_stream.rs` - Basic streaming setup and event processing
- `filtered_events.rs` - Event filtering, routing, and pattern matching

### `advanced/`
- `hft_bot.rs` - High-frequency trading with microsecond latency
- `yield_monitor.rs` - DeFi yield monitoring across protocols

<!-- Removed `integrations/` section (all entries were placeholders) -->

<!-- Removed `infrastructure/` section (all entries were placeholders) -->

## ⚙️ Environment Variables

```bash
# Solana Configuration
SOLANA_RPC_URL=wss://api.mainnet-beta.solana.com
SOLANA_RPC_HTTP_URL=https://api.mainnet-beta.solana.com

# Streaming Configuration
RIGLR_CONNECT_TIMEOUT_SECS=10
RIGLR_CHANNEL_SIZE=10000
RIGLR_BACKPRESSURE_STRATEGY=adaptive
RIGLR_METRICS_ENABLED=true

# Alert Configuration (for yield monitor)
DISCORD_WEBHOOK_URL=https://discord.com/api/webhooks/...
SLACK_WEBHOOK_URL=https://hooks.slack.com/services/...
```

## 🐛 Troubleshooting

For debugging tips and common issues, see the [Troubleshooting section](../../../docs/src/concepts/streams.md#troubleshooting) in the main documentation.

## 🤝 Contributing

To add new streaming examples:

1. Follow the existing directory structure (`basic/`, `advanced/`, `integrations/`, `infrastructure/`)
2. Include comprehensive documentation and comments
3. Add unit tests for core functionality
4. Update this README with example descriptions
5. Ensure examples work with simulated data for testing

## 📊 Performance

For detailed performance benchmarks and optimization tips, see the [Performance section](../../../docs/src/concepts/streams.md#performance-benchmarks) in the main documentation.
