# Data Indexing

The `riglr-indexer` crate provides a production-grade indexing service for processing, storing, and querying blockchain data. Built with a flexible pipeline architecture, it can handle high-throughput data ingestion while maintaining queryable, indexed data stores.

## Pipeline Architecture

The indexer follows a three-stage pipeline architecture:

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Ingester  │────▶│  Processor  │────▶│   Storage   │
└─────────────┘     └─────────────┘     └─────────────┘
       │                   │                    │
   Raw Events         Transformed           Indexed
   from Source          Data               Database
```

### Ingester
Connects to data sources and normalizes incoming events into a common format.

### Processor
Applies business logic, enrichment, and transformations to the raw data.

### Storage
Persists processed data in optimized formats for different query patterns.

## Configuration

Configure the indexer using YAML or programmatically:

```yaml
# indexer.yaml
indexer:
  name: "pump_indexer"
  
  ingester:
    source: geyser
    endpoint: "${GEYSER_URL}"
    filters:
      - program: "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P"
    
  processor:
    type: pump_events
    enrichments:
      - token_metadata
      - price_oracle
    
  storage:
    primary:
      type: postgres
      connection: "${DATABASE_URL}"
      schema: pump_events
    cache:
      type: redis
      connection: "${REDIS_URL}"
      ttl: 300
```

## Building an Indexer

### Basic Setup

```rust
use riglr_indexer::{Indexer, Pipeline, Config};

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::from_file("indexer.yaml")?;
    
    // Build the indexer
    let indexer = Indexer::builder()
        .config(config)
        .build()
        .await?;
    
    // Start indexing
    indexer.start().await?;
    
    Ok(())
}
```

### Custom Pipeline

Create a custom pipeline with specific processing logic:

```rust
use riglr_indexer::{Pipeline, Ingester, Processor, Storage};

// Custom ingester for specific data source
struct CustomIngester;

impl Ingester for CustomIngester {
    async fn ingest(&mut self) -> Result<RawEvent> {
        // Connect to data source and fetch events
        let event = fetch_next_event().await?;
        Ok(RawEvent::from(event))
    }
}

// Custom processor with business logic
struct TokenProcessor;

impl Processor for TokenProcessor {
    async fn process(&self, event: RawEvent) -> Result<ProcessedData> {
        // Parse event
        let parsed = parse_token_event(event)?;
        
        // Enrich with metadata
        let metadata = fetch_token_metadata(&parsed.token).await?;
        
        // Calculate metrics
        let metrics = calculate_metrics(&parsed, &metadata);
        
        Ok(ProcessedData {
            parsed,
            metadata,
            metrics,
        })
    }
}

// Build the pipeline
let pipeline = Pipeline::builder()
    .ingester(CustomIngester)
    .processor(TokenProcessor)
    .storage(PostgresStorage::new(db_url))
    .build();

let indexer = Indexer::new(pipeline);
indexer.start().await?;
```

## Data Sources

### Blockchain Sources

#### Solana Geyser
High-performance streaming directly from validators:

```rust
let ingester = GeyserIngester::builder()
    .endpoint(geyser_url)
    .programs(vec![pump_program, raydium_program])
    .commitment(Commitment::Confirmed)
    .build();
```

#### RPC Polling
Fallback option for providers without streaming:

```rust
let ingester = RpcIngester::builder()
    .endpoint(rpc_url)
    .poll_interval(Duration::from_millis(500))
    .batch_size(100)
    .build();
```

#### Historical Backfill
Index historical data from specific points:

```rust
let ingester = HistoricalIngester::builder()
    .source(bigtable_client)
    .start_slot(123456789)
    .end_slot(123556789)
    .parallelism(10)
    .build();
```

### External Sources

#### Exchange APIs
Index trading data from exchanges:

```rust
let ingester = ExchangeIngester::builder()
    .exchange(Exchange::Binance)
    .symbols(vec!["SOLUSDT", "BTCUSDT"])
    .data_types(vec![DataType::Trades, DataType::OrderBook])
    .build();
```

#### Social Media
Index social sentiment data:

```rust
let ingester = SocialIngester::builder()
    .platform(Platform::Twitter)
    .keywords(vec!["#solana", "$SOL"])
    .build();
```

## Processing Strategies

### Event Enrichment
Add context and metadata to raw events:

```rust
struct EnrichmentProcessor;

impl Processor for EnrichmentProcessor {
    async fn process(&self, event: RawEvent) -> Result<ProcessedData> {
        let mut data = ProcessedData::from(event);
        
        // Add token metadata
        if let Some(token) = data.token_address {
            data.metadata = fetch_metadata(&token).await?;
        }
        
        // Add price data
        if let Some(symbol) = data.symbol {
            data.price = fetch_price(&symbol).await?;
        }
        
        // Add risk scores
        data.risk_score = calculate_risk(&data).await?;
        
        Ok(data)
    }
}
```

### Aggregation
Compute aggregates and statistics:

```rust
struct AggregationProcessor {
    window: Duration,
}

impl Processor for AggregationProcessor {
    async fn process(&self, events: Vec<RawEvent>) -> Result<AggregatedData> {
        let stats = AggregatedData {
            volume_24h: events.iter().map(|e| e.volume).sum(),
            trade_count: events.len(),
            unique_traders: events.iter()
                .map(|e| &e.trader)
                .collect::<HashSet<_>>()
                .len(),
            vwap: calculate_vwap(&events),
        };
        
        Ok(stats)
    }
}
```

### Deduplication
Handle duplicate events from multiple sources:

```rust
struct DeduplicationProcessor {
    cache: Arc<DashMap<String, Instant>>,
}

impl Processor for DeduplicationProcessor {
    async fn process(&self, event: RawEvent) -> Result<Option<ProcessedData>> {
        let key = event.signature.clone();
        
        // Check if we've seen this event recently
        if self.cache.contains_key(&key) {
            return Ok(None); // Duplicate, skip
        }
        
        // Mark as seen
        self.cache.insert(key, Instant::now());
        
        // Process normally
        Ok(Some(ProcessedData::from(event)))
    }
}
```

## Storage Backends

### PostgreSQL
Relational storage for complex queries:

```rust
let storage = PostgresStorage::builder()
    .connection_pool(pool)
    .schema("indexed_data")
    .batch_size(1000)
    .build();

// Custom table mapping
storage.map_table(EventType::Trade, "trades");
storage.map_table(EventType::Swap, "swaps");
```

### Redis
High-performance cache layer:

```rust
let cache = RedisStorage::builder()
    .connection(redis_client)
    .ttl(Duration::from_secs(300))
    .compression(true)
    .build();
```

### ClickHouse
Analytics-optimized columnar storage:

```rust
let analytics = ClickHouseStorage::builder()
    .connection(clickhouse_url)
    .database("blockchain")
    .table("events")
    .partition_by("toYYYYMM(timestamp)")
    .build();
```

### S3/Object Storage
Long-term archival:

```rust
let archive = S3Storage::builder()
    .bucket("blockchain-archive")
    .prefix("pump-events")
    .compression(CompressionType::Zstd)
    .partitioning(Partitioning::Daily)
    .build();
```

## Query Interface

Access indexed data through various query methods:

### SQL Queries
Direct SQL access for complex analytics:

```rust
let results = indexer.query_sql(r#"
    SELECT 
        token_address,
        SUM(volume) as total_volume,
        COUNT(*) as trade_count
    FROM trades
    WHERE timestamp > NOW() - INTERVAL '24 hours'
    GROUP BY token_address
    ORDER BY total_volume DESC
    LIMIT 10
"#).await?;
```

### GraphQL API
Flexible API for frontend applications:

```rust
let schema = indexer.graphql_schema();

let query = r#"
    query TopTokens($limit: Int!) {
        tokens(
            orderBy: VOLUME_24H_DESC,
            first: $limit
        ) {
            address
            symbol
            volume24h
            priceUsd
        }
    }
"#;

let result = schema.execute(query).await?;
```

### REST API
Simple HTTP endpoints:

```rust
indexer.serve_rest(RestConfig {
    port: 8080,
    endpoints: vec![
        Endpoint::get("/tokens/:address", get_token_handler),
        Endpoint::get("/trades", list_trades_handler),
        Endpoint::post("/search", search_handler),
    ],
}).await?;
```

## Performance Optimization

### Parallel Processing
Scale processing across multiple cores:

```rust
let processor = ParallelProcessor::builder()
    .workers(num_cpus::get())
    .queue_size(10_000)
    .build();
```

### Batch Operations
Optimize database writes:

```rust
let storage = BatchedStorage::builder()
    .backend(postgres)
    .batch_size(5_000)
    .flush_interval(Duration::from_secs(1))
    .build();
```

### Indexing Strategies
Optimize query performance:

```rust
// Create indexes for common queries
storage.create_index("trades", vec!["token_address", "timestamp"]);
storage.create_index("trades", vec!["trader", "timestamp"]);

// Partition large tables
storage.partition_table("trades", PartitionStrategy::TimeRange {
    interval: Duration::from_days(1),
});
```

## Monitoring and Maintenance

### Health Checks
Monitor indexer health:

```rust
let health = indexer.health_check().await?;
println!("Status: {:?}", health.status);
println!("Lag: {} seconds", health.indexing_lag.as_secs());
println!("Events/sec: {}", health.throughput);
```

### Metrics
Export metrics for monitoring:

```rust
indexer.metrics(MetricsConfig {
    exporter: PrometheusExporter::new(),
    port: 9090,
    interval: Duration::from_secs(10),
}).await?;
```

### Maintenance Tasks
Schedule regular maintenance:

```rust
indexer.schedule(
    Schedule::daily_at(3, 0),  // 3 AM
    |indexer| async move {
        // Vacuum database
        indexer.storage().vacuum().await?;
        
        // Archive old data
        indexer.archive_before(
            Utc::now() - Duration::from_days(30)
        ).await?;
        
        Ok(())
    }
);
```

## Disaster Recovery

### Checkpointing
Save progress for recovery:

```rust
let indexer = Indexer::builder()
    .checkpoint(CheckpointConfig {
        storage: checkpoint_store,
        interval: Duration::from_secs(10),
    })
    .build();

// Resume from checkpoint after crash
indexer.resume_from_checkpoint().await?;
```

### Reindexing
Rebuild indexes from source data:

```rust
// Reindex specific time range
indexer.reindex(ReindexConfig {
    start: datetime!(2024-01-01 00:00:00),
    end: datetime!(2024-01-31 23:59:59),
    parallel_workers: 10,
}).await?;
```

## Integration Example

Complete example of a Pump.fun trade indexer:

```rust
use riglr_indexer::{Indexer, Pipeline};
use riglr_solana_events::PumpEventParser;

async fn run_pump_indexer() -> Result<()> {
    // Configure pipeline
    let pipeline = Pipeline::builder()
        // Ingest from Geyser
        .ingester(
            GeyserIngester::builder()
                .endpoint(std::env::var("GEYSER_URL")?)
                .program(PUMP_PROGRAM_ID)
                .build()
        )
        // Process Pump events
        .processor(
            PumpProcessor::builder()
                .parser(PumpEventParser::new())
                .enrichments(vec![
                    Enrichment::TokenMetadata,
                    Enrichment::PriceOracle,
                    Enrichment::RiskScore,
                ])
                .build()
        )
        // Store in PostgreSQL and cache in Redis
        .storage(
            MultiStorage::builder()
                .primary(PostgresStorage::new(db_url))
                .cache(RedisStorage::new(redis_url))
                .build()
        )
        .build();
    
    // Create and start indexer
    let indexer = Indexer::new(pipeline);
    
    // Serve query API
    indexer.serve_graphql(8080).await?;
    
    // Start indexing
    indexer.start().await?;
    
    Ok(())
}
```

## Best Practices

1. **Schema Design**: Design storage schema based on query patterns
2. **Idempotency**: Ensure processors are idempotent for reprocessing
3. **Error Handling**: Implement dead letter queues for failed events
4. **Monitoring**: Set up comprehensive monitoring and alerting
5. **Backups**: Regular backups of indexed data and checkpoints

## Next Steps

- Learn about [Configuration](configuration.md) for production deployments
- Explore [Real-Time Streaming](streams.md) for data ingestion
- Understand [Agent Coordination](agents.md) for query-driven agents