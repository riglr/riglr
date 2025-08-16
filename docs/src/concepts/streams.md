# Real-Time Streaming

The `riglr-streams` crate provides a powerful stream processing framework for handling real-time blockchain events, market data, and other high-throughput data sources.

## StreamManager

The StreamManager is the central hub for all streaming operations, providing unified access to multiple data sources and powerful stream transformation capabilities.

```rust
use riglr_streams::{StreamManager, StreamConfig};

// Initialize the StreamManager
let manager = StreamManager::builder()
    .max_connections(100)
    .buffer_size(10_000)
    .build()
    .await?;

// Connect to a data source
let stream = manager.connect(StreamConfig {
    source: StreamSource::SolanaGeyser,
    endpoint: "wss://geyser.example.com",
    filters: vec![
        Filter::Program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
    ],
}).await?;
```

## Stream Sources

### Solana Geyser
High-performance streaming of Solana blockchain events directly from validators.

```rust
let geyser_stream = manager.connect(StreamConfig {
    source: StreamSource::SolanaGeyser,
    endpoint: std::env::var("GEYSER_URL")?,
    filters: vec![
        Filter::Program(pump_program_id),
        Filter::AccountType("TokenAccount"),
    ],
}).await?;

// Subscribe to events
geyser_stream.subscribe(|event| async move {
    match event {
        GeyserEvent::AccountUpdate(account) => {
            println!("Account updated: {}", account.pubkey);
        }
        GeyserEvent::Transaction(tx) => {
            println!("Transaction: {}", tx.signature);
        }
        _ => {}
    }
}).await;
```

### WebSocket Connections
Generic WebSocket support for various data sources.

```rust
let ws_stream = manager.connect(StreamConfig {
    source: StreamSource::WebSocket,
    endpoint: "wss://stream.binance.com:9443/ws",
    subscription: json!({
        "method": "SUBSCRIBE",
        "params": ["btcusdt@trade", "ethusdt@trade"],
        "id": 1
    }),
}).await?;
```

### Exchange Streams
Pre-configured connections to major exchanges.

```rust
// Binance stream
let binance_stream = manager.connect_binance(BinanceConfig {
    symbols: vec!["BTCUSDT", "ETHUSDT", "SOLUSDT"],
    stream_types: vec![StreamType::Trade, StreamType::OrderBook],
}).await?;

// Coinbase stream
let coinbase_stream = manager.connect_coinbase(CoinbaseConfig {
    product_ids: vec!["BTC-USD", "ETH-USD"],
    channels: vec!["ticker", "trades"],
}).await?;
```

## Stream Operators

Transform and process streams with powerful operators that can be chained together.

### Map
Transform each element in the stream.

```rust
let prices = stream
    .map(|event| {
        match event {
            TradeEvent { price, .. } => price,
            _ => 0.0,
        }
    })
    .collect();
```

### Filter
Keep only elements that match a predicate.

```rust
let large_trades = stream
    .filter(|trade| trade.volume > 1000.0)
    .collect();
```

### Batch
Group elements into batches for efficient processing.

```rust
let batched = stream
    .batch(100)  // Batch 100 events
    .batch_timeout(Duration::from_secs(5))  // Or timeout after 5 seconds
    .for_each(|batch| async move {
        // Process batch of events
        process_batch(batch).await?;
    })
    .await;
```

### Window
Apply sliding or tumbling windows for time-based aggregation.

```rust
// 1-minute tumbling window for VWAP calculation
let vwap = stream
    .window(WindowType::Tumbling(Duration::from_secs(60)))
    .aggregate(|window| {
        let total_value: f64 = window.iter()
            .map(|t| t.price * t.volume)
            .sum();
        let total_volume: f64 = window.iter()
            .map(|t| t.volume)
            .sum();
        total_value / total_volume
    })
    .collect();
```

### Merge
Combine multiple streams into one.

```rust
let combined = StreamManager::merge(vec![
    binance_stream,
    coinbase_stream,
    kraken_stream,
])
.map(|event| normalize_exchange_event(event))
.collect();
```

### Rate Limit
Control the rate of events flowing through the stream.

```rust
let throttled = stream
    .rate_limit(100)  // Max 100 events per second
    .collect();
```

## Advanced Stream Processing

### Stateful Processing
Maintain state across stream events.

```rust
use riglr_streams::StatefulProcessor;

let processor = StatefulProcessor::new(
    InitialState { count: 0, sum: 0.0 },
    |state, event| {
        state.count += 1;
        state.sum += event.price;
        let avg = state.sum / state.count as f64;
        
        if avg > threshold {
            Some(Alert::PriceHigh(avg))
        } else {
            None
        }
    }
);

let alerts = stream
    .process_stateful(processor)
    .collect();
```

### Fork and Join
Split streams for parallel processing and rejoin results.

```rust
let (fast_stream, slow_stream) = stream.fork();

// Fast processing path
let fast_results = fast_stream
    .filter(|e| e.is_urgent())
    .map(|e| process_urgent(e))
    .collect();

// Slow processing path
let slow_results = slow_stream
    .batch(1000)
    .map(|batch| process_batch(batch))
    .collect();

// Join results
let all_results = StreamManager::join(
    fast_results,
    slow_results,
    JoinStrategy::Merge,
).collect();
```

### Error Handling
Robust error handling with retry and circuit breaker patterns.

```rust
let resilient_stream = stream
    .retry(RetryPolicy {
        max_attempts: 3,
        backoff: ExponentialBackoff::default(),
    })
    .circuit_breaker(CircuitBreakerConfig {
        failure_threshold: 5,
        recovery_timeout: Duration::from_secs(60),
    })
    .on_error(|error| {
        // Log or handle errors
        log::error!("Stream error: {}", error);
    })
    .collect();
```

## Backpressure Handling

Manage high-throughput streams with intelligent backpressure strategies.

```rust
let managed_stream = stream
    .buffer(BufferConfig {
        capacity: 10_000,
        overflow_strategy: OverflowStrategy::DropOldest,
    })
    .backpressure(BackpressureConfig {
        strategy: BackpressureStrategy::Adaptive,
        high_watermark: 8_000,
        low_watermark: 2_000,
    })
    .collect();
```

## Stream Persistence

Save stream data for replay and analysis.

```rust
// Write to persistent storage
let persisted = stream
    .tee(|event| async move {
        // Write to database
        db.insert_event(event).await?;
        Ok(())
    })
    .collect();

// Replay from storage
let historical = StreamManager::replay(
    ReplayConfig {
        source: db,
        start_time: yesterday,
        end_time: now,
        speed: ReplaySpeed::Realtime,
    }
).await?;
```

## Integration Example

Complete example integrating streams with trading logic:

```rust
use riglr_streams::{StreamManager, StreamConfig};
use riglr_solana_tools::pump;
use riglr_agents::Agent;

async fn run_trading_stream() -> Result<()> {
    // Initialize stream manager
    let manager = StreamManager::new().await?;
    
    // Connect to Pump.fun events
    let pump_stream = manager.connect(StreamConfig {
        source: StreamSource::SolanaGeyser,
        endpoint: std::env::var("GEYSER_URL")?,
        filters: vec![
            Filter::Program(PUMP_PROGRAM_ID),
        ],
    }).await?;
    
    // Process events with operators
    pump_stream
        // Parse Pump.fun events
        .map(|event| parse_pump_event(event))
        // Filter for new token launches
        .filter(|event| matches!(event, PumpEvent::TokenLaunch(_)))
        // Batch for efficiency
        .batch(10)
        .batch_timeout(Duration::from_secs(1))
        // Process each batch
        .for_each(|batch| async move {
            for event in batch {
                if let PumpEvent::TokenLaunch(token) = event {
                    // Analyze token
                    let score = analyze_token(&token).await?;
                    
                    // Execute trade if promising
                    if score > 0.8 {
                        pump::buy_token(
                            &token.address,
                            Amount::Sol(0.1),
                        ).await?;
                    }
                }
            }
            Ok(())
        })
        .await?;
    
    Ok(())
}
```

## Monitoring and Metrics

Track stream health and performance:

```rust
let monitored = stream
    .metrics(MetricsConfig {
        export_interval: Duration::from_secs(10),
        exporter: PrometheusExporter::new(),
    })
    .health_check(|stats| {
        if stats.error_rate > 0.1 {
            HealthStatus::Unhealthy
        } else {
            HealthStatus::Healthy
        }
    })
    .collect();

// Access stream statistics
let stats = monitored.stats().await;
println!("Events processed: {}", stats.total_events);
println!("Error rate: {:.2}%", stats.error_rate * 100.0);
println!("Throughput: {} events/sec", stats.throughput);
```

## Best Practices

1. **Buffer Appropriately**: Set buffer sizes based on expected throughput and processing capacity
2. **Handle Disconnections**: Implement automatic reconnection with exponential backoff
3. **Monitor Lag**: Track processing lag to detect performance issues early
4. **Use Batching**: Process events in batches for better efficiency
5. **Implement Checkpointing**: Save progress for recovery from failures

## Performance Tuning

### Memory Management
```rust
let optimized = stream
    .buffer(BufferConfig {
        capacity: 100_000,
        overflow_strategy: OverflowStrategy::Backpressure,
        memory_limit: ByteSize::mb(100),
    })
    .collect();
```

### CPU Optimization
```rust
let parallel = stream
    .parallel(ParallelConfig {
        workers: num_cpus::get(),
        distribution: Distribution::RoundRobin,
    })
    .collect();
```

### Network Optimization
```rust
let compressed = stream
    .compression(CompressionType::Zstd)
    .tcp_nodelay(true)
    .keep_alive(Duration::from_secs(30))
    .collect();
```

## Next Steps

- Explore the [Indexer](indexer.md) for storing and querying stream data
- Learn about [Agent Coordination](agents.md) for stream-driven agents
- Understand [Configuration](configuration.md) for production deployments