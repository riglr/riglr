# riglr-streams

Event-driven streaming capabilities for the RIGLR blockchain agent framework.

## Overview

`riglr-streams` provides composable streaming components that enable developers to build proactive, event-driven blockchain agents. It extends RIGLR's reactive tool-based architecture with real-time data streams from multiple sources.

## Features

- 🚀 **High-Performance Streaming**: Designed for >10,000 events/second throughput
- 🔌 **Multi-Source Support**: Solana, EVM chains, Binance, Mempool.space
- 🎯 **Event-Triggered Tools**: Flexible condition system for automated tool execution
- 💪 **Production Ready**: Health monitoring, circuit breakers, and metrics
- 📦 **Library-First Design**: Composable components, not a monolithic pipeline

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      StreamManager                           │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Solana     │  │     EVM      │  │   External   │      │
│  │   Streams    │  │   Streams    │  │   Streams    │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│         │                 │                  │               │
│         └─────────────────┼──────────────────┘               │
│                           │                                   │
│                    ┌──────────────┐                          │
│                    │   Events     │                          │
│                    └──────────────┘                          │
│                           │                                   │
│         ┌─────────────────┼──────────────────┐              │
│         │                 │                  │               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │Event Handler │  │Event Handler │  │Event Handler │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

### Basic Setup

```rust
use riglr_streams::prelude::*;
use riglr_streams::solana::{SolanaGeyserStream, GeyserConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create stream manager
    let stream_manager = StreamManager::new();
    
    // Create and configure Solana stream
    let mut solana_stream = SolanaGeyserStream::new("solana-mainnet");
    let config = GeyserConfig {
        ws_url: "wss://api.mainnet-beta.solana.com".to_string(),
        auth_token: None,
        program_ids: vec!["JUP...".to_string()],
        buffer_size: 10000,
    };
    
    // Start the stream
    solana_stream.start(config).await?;
    
    // Add running stream to manager
    stream_manager.add_stream("solana".to_string(), solana_stream).await?;
    
    // Process events
    stream_manager.process_events().await?;
    
    Ok(())
}
```

### Event-Triggered Tools

```rust
use riglr_streams::tools::{EventTriggeredTool, StreamingTool};
use async_trait::async_trait;

struct MyTool;

#[async_trait]
impl StreamingTool for MyTool {
    async fn execute(&self, event: &dyn UnifiedEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Process event
        Ok(())
    }
    
    fn name(&self) -> &str {
        "MyTool"
    }
}

// Create event-triggered version
let triggered_tool = EventTriggeredTool::new(MyTool, "my-tool")
    .with_condition(/* condition */)
    .build();
```

## Supported Streams

### Solana
- WebSocket RPC subscriptions
- Program-specific filtering
- Transaction and account monitoring

### EVM Chains
- Multi-chain support via RPC_URL_{CHAIN_ID}
- Pending transactions
- New blocks
- Contract events

### External Data
- **Binance**: Real-time market data, order books, trades
- **Mempool.space**: Bitcoin mempool, blocks, fee estimates

## Event Conditions

Build complex event matching logic:

```rust
use riglr_streams::tools::{EventMatcher, ConditionCombinator};

// Match specific event types
let swap_condition = EventMatcher::event_type(EventType::Swap);

// Match protocol
let jupiter_condition = EventMatcher::protocol(ProtocolType::Jupiter);

// Combine conditions
let complex_condition = EventMatcher::all(vec![
    swap_condition,
    jupiter_condition,
    EventMatcher::block_range(Some(100000), None),
]);
```

## Production Features

### Health Monitoring

```rust
use riglr_streams::production::{HealthMonitor, HealthThresholds};

let health_monitor = HealthMonitor::new(stream_manager.clone())
    .with_thresholds(HealthThresholds {
        max_event_age: Duration::from_secs(300),
        max_error_rate: 10.0,
        min_event_rate: 0.1,
        max_consecutive_errors: 5,
    });

health_monitor.start().await;
```

### Circuit Breaker

```rust
use riglr_streams::production::CircuitBreaker;

let circuit_breaker = CircuitBreaker::new("solana-stream")
    .with_failure_threshold(5)
    .with_timeout(Duration::from_secs(60));
```

### Metrics

```rust
use riglr_streams::production::MetricsCollector;

let metrics = MetricsCollector::new();
let stream_metrics = metrics.get_stream_metrics("solana").await;
stream_metrics.record_event("swap", 12.5).await;
```

## Configuration

### Environment Variables

```bash
# Solana
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com

# EVM Chains (use chain ID)
RPC_URL_1=https://eth-mainnet.alchemyapi.io/v2/your-key
RPC_URL_137=https://polygon-mainnet.alchemyapi.io/v2/your-key
RPC_URL_42161=https://arb-mainnet.g.alchemy.com/v2/your-key

# External APIs (optional)
BINANCE_API_KEY=your-key
```

## Examples

See the `examples/` directory for complete examples:
- `streaming_arbitrage.rs` - Cross-chain arbitrage bot
- More examples coming soon

## Performance

Benchmarks on standard hardware:
- **Throughput**: >10,000 events/second
- **Latency**: <10ms event processing
- **Memory**: ~100MB baseline, scales with buffer sizes

## Contributing

Contributions are welcome! Please read our contributing guidelines.

## License

MIT OR Apache-2.0