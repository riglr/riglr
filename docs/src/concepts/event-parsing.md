# Event Parsing System

riglr's event parsing system provides a powerful, extensible framework for extracting and interpreting blockchain events from transaction data. This enables real-time monitoring, analytics, and automated reactions to on-chain activity.

## Overview

The event parsing system transforms raw blockchain transaction data into structured, actionable events. It supports multiple emission patterns and protocols, providing a unified interface for consuming diverse on-chain data.

## Architecture

### Core Components

**Event Parser Trait**
The foundation that all parsers implement:
```rust
pub trait EventParser {
    fn parse_transaction(&self, tx: &TransactionInfo) 
        -> Result<Vec<Box<dyn UnifiedEvent>>, EventParseError>;
    
    fn protocol_type(&self) -> ProtocolType;
    fn program_id(&self) -> Pubkey;
}
```

**Unified Event Interface**
All events implement a common interface for consistency:
```rust
pub trait UnifiedEvent {
    fn event_type(&self) -> EventType;
    fn protocol(&self) -> ProtocolType;
    fn metadata(&self) -> &EventMetadata;
    fn as_any(&self) -> &dyn Any;
}
```

**Parser Factory**
Manages parser registration and routing:
```rust
pub struct EventParserFactory {
    parsers: HashMap<Pubkey, Box<dyn EventParser>>,
}
```

## Event Emission Patterns

### 1. Instruction-Based Events

Events encoded directly in instruction parameters. Most efficient for parsing.

```rust
// Example: Parsing a swap instruction
pub fn parse_swap_instruction(
    instruction: &CompiledInstruction,
    accounts: &[Pubkey],
) -> Result<SwapEvent, ParseError> {
    let data = &instruction.data;
    
    // Decode instruction parameters
    let amount_in = u64::from_le_bytes(data[0..8].try_into()?);
    let amount_out = u64::from_le_bytes(data[8..16].try_into()?);
    
    Ok(SwapEvent {
        user: accounts[instruction.accounts[0] as usize],
        pool: accounts[instruction.accounts[1] as usize],
        amount_in,
        amount_out,
    })
}
```

### 2. Log-Based Events

Events emitted through program logs, often in inner instructions.

```rust
// Example: Parsing events from logs
pub fn parse_log_event(
    logs: &[String],
    discriminator: &str,
) -> Result<LogEvent, ParseError> {
    for log in logs {
        if log.starts_with(discriminator) {
            // Extract and parse event data
            let event_data = extract_event_data(log)?;
            return Ok(parse_event_from_data(event_data)?);
        }
    }
    Err(ParseError::EventNotFound)
}
```

## Supported Protocols

riglr includes parsers for major Solana protocols:

### DEX Protocols
- **Raydium AMM V4**: Classic AMM swaps
- **Raydium CLMM**: Concentrated liquidity swaps
- **Raydium CPMM**: Constant product pools
- **Jupiter**: Aggregator swaps
- **Pump.fun**: Bonding curve trades

### Token Protocols
- **SPL Token**: Transfers and mints
- **Token 2022**: Extended token features

## Creating Custom Parsers

### Step 1: Define Event Structures

```rust
use serde::{Deserialize, Serialize};
use crate::events::common::{EventMetadata, impl_unified_event};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSwapEvent {
    pub metadata: EventMetadata,
    pub token_in: Pubkey,
    pub token_out: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub user: Pubkey,
}

// Automatically implement UnifiedEvent trait
impl_unified_event!(CustomSwapEvent);
```

### Step 2: Implement Parser Logic

```rust
pub struct CustomProtocolParser {
    program_id: Pubkey,
}

impl EventParser for CustomProtocolParser {
    fn parse_transaction(
        &self,
        tx: &TransactionInfo,
    ) -> Result<Vec<Box<dyn UnifiedEvent>>, EventParseError> {
        let mut events = Vec::new();
        
        for instruction in &tx.instructions {
            if instruction.program_id == self.program_id {
                if let Some(event) = self.parse_instruction(instruction, tx)? {
                    events.push(Box::new(event));
                }
            }
        }
        
        Ok(events)
    }
    
    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::Custom
    }
    
    fn program_id(&self) -> Pubkey {
        self.program_id
    }
}
```

### Step 3: Register with Factory

```rust
let mut factory = EventParserFactory::new();
factory.register_parser(
    CUSTOM_PROGRAM_ID,
    Box::new(CustomProtocolParser::new())
);
```

## Event Metadata

Every event includes rich metadata for context. The metadata system uses a wrapper pattern to separate chain-specific data from core metadata:

### Core EventMetadata

The base metadata structure from `riglr-events-core`:

```rust
pub struct EventMetadata {
    pub id: String,                    // Unique identifier
    pub kind: EventKind,                // Event classification
    pub timestamp: DateTime<Utc>,      // Event timestamp
    pub received_at: DateTime<Utc>,    // When received by system
    pub source: String,                 // Data source
    pub chain_data: Option<ChainData>, // Optional chain-specific data
    pub custom: HashMap<String, Value>, // Custom fields
}
```

### Solana-Specific Metadata

Solana events use an explicit wrapper pattern to add chain-specific fields:

```rust
pub struct SolanaEventMetadata {
    // Solana-specific fields
    pub signature: String,
    pub slot: u64,
    pub event_type: EventType,
    pub protocol_type: ProtocolType,
    pub index: String,
    pub program_received_time_ms: i64,
    
    // Wrapped core metadata
    pub core: EventMetadata,
}

// Deref implementation for ergonomic field access
impl Deref for SolanaEventMetadata {
    type Target = EventMetadata;
    fn deref(&self) -> &Self::Target {
        &self.core
    }
}
```

### Design Rationale

This wrapper pattern was chosen after careful architectural consideration:

1. **No Ambiguity**: Clear distinction between `SolanaEventMetadata` and core `EventMetadata`
2. **Ergonomic Access**: With `Deref`, you can access core fields directly: `solana_meta.id`
3. **Single Source of Truth**: Solana-specific data lives only on the wrapper
4. **Type Safety**: The compiler enforces correct usage
5. **No Data Duplication**: Avoids storing the same data in both wrapper and `core.chain_data`

### Usage Example

```rust
// Creating Solana event metadata
let core = create_core_metadata(
    id,
    kind,
    source,
    block_time,
);

let solana_metadata = SolanaEventMetadata::new(
    signature,
    slot,
    event_type,
    protocol_type,
    index,
    received_time_ms,
    core,
);

// Accessing fields
let slot = solana_meta.slot;           // Solana-specific field
let id = solana_meta.id;               // Core field via Deref
let signature = &solana_meta.signature; // Solana-specific field

// For Event trait implementation
impl Event for CustomEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata.core  // Return the core metadata
    }
}
```

## Real-Time Event Streaming

### WebSocket Subscription

```rust
use riglr_solana_tools::events::EventStream;

let stream = EventStream::new(rpc_url);

// Subscribe to specific protocols
stream.subscribe(vec![
    ProtocolType::Raydium,
    ProtocolType::Jupiter,
]).await?;

// Process events as they arrive
while let Some(event) = stream.next().await {
    match event.event_type() {
        EventType::Swap => handle_swap(event),
        EventType::AddLiquidity => handle_liquidity(event),
        _ => {}
    }
}
```

### Historical Event Queries

```rust
use riglr_solana_tools::events::EventQuery;

let query = EventQuery::new()
    .protocol(ProtocolType::PumpFun)
    .event_type(EventType::Trade)
    .from_slot(150_000_000)
    .to_slot(150_100_000);

let events = query.execute().await?;
```

## Performance Optimization

### Batch Processing

Process multiple transactions efficiently:

```rust
pub async fn parse_batch(
    transactions: Vec<TransactionInfo>,
    parser: &dyn EventParser,
) -> Vec<Result<Vec<Box<dyn UnifiedEvent>>, EventParseError>> {
    transactions
        .into_par_iter()
        .map(|tx| parser.parse_transaction(&tx))
        .collect()
}
```

### Caching Strategies

Cache parsed events to avoid redundant processing:

```rust
use lru::LruCache;

pub struct CachedEventParser {
    parser: Box<dyn EventParser>,
    cache: RwLock<LruCache<String, Vec<Box<dyn UnifiedEvent>>>>,
}

impl CachedEventParser {
    pub async fn parse_with_cache(
        &self,
        tx: &TransactionInfo,
    ) -> Result<Vec<Box<dyn UnifiedEvent>>, EventParseError> {
        let signature = &tx.signature;
        
        // Check cache first
        if let Some(events) = self.cache.read().await.get(signature) {
            return Ok(events.clone());
        }
        
        // Parse and cache
        let events = self.parser.parse_transaction(tx)?;
        self.cache.write().await.put(signature.clone(), events.clone());
        
        Ok(events)
    }
}
```

## Event-Driven Agents

Build reactive agents that respond to events:

```rust
use riglr_core::signer::SignerContext;

async fn arbitrage_bot(event_stream: EventStream) {
    while let Some(event) = event_stream.next().await {
        if let Some(swap) = event.as_any().downcast_ref::<SwapEvent>() {
            // Check for arbitrage opportunity
            if let Some(opportunity) = find_arbitrage(swap).await {
                // Execute arbitrage within signer context
                SignerContext::current().await?
                    .execute_arbitrage(opportunity).await?;
            }
        }
    }
}
```

## Testing Event Parsers

### Unit Tests

Test individual parsing functions:

```rust
#[test]
fn test_parse_swap_instruction() {
    let instruction = create_mock_instruction();
    let accounts = vec![user_pubkey, pool_pubkey];
    
    let event = parse_swap_instruction(&instruction, &accounts).unwrap();
    
    assert_eq!(event.amount_in, 1000);
    assert_eq!(event.amount_out, 950);
}
```

### Integration Tests

Test against real transaction data:

```rust
#[tokio::test]
async fn test_parse_real_transaction() {
    let tx = fetch_transaction("5xKYx...").await?;
    let parser = RaydiumParser::new();
    
    let events = parser.parse_transaction(&tx)?;
    
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type(), EventType::Swap);
}
```

## Best Practices

### 1. Discriminator Management

Keep discriminators organized and documented:

```rust
pub mod discriminators {
    pub const SWAP: &[u8] = &[0x01, 0x02, 0x03, 0x04];
    pub const ADD_LIQUIDITY: &[u8] = &[0x05, 0x06, 0x07, 0x08];
}
```

### 2. Error Handling

Provide detailed error context:

```rust
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid instruction data: expected {expected} bytes, got {actual}")]
    InvalidDataLength { expected: usize, actual: usize },
    
    #[error("Unknown instruction discriminator: {0:?}")]
    UnknownDiscriminator(Vec<u8>),
}
```

### 3. Event Validation

Validate events before returning:

```rust
fn validate_swap_event(event: &SwapEvent) -> Result<(), ValidationError> {
    if event.amount_in == 0 {
        return Err(ValidationError::ZeroAmount);
    }
    if event.token_in == event.token_out {
        return Err(ValidationError::SameToken);
    }
    Ok(())
}
```

## Summary

The event parsing system is a cornerstone of riglr's blockchain monitoring capabilities. It provides:

- **Extensibility**: Easy to add new protocol parsers
- **Performance**: Efficient batch processing and caching
- **Reliability**: Robust error handling and validation
- **Flexibility**: Support for multiple event patterns
- **Integration**: Seamless connection with riglr's agent framework

By leveraging this system, you can build sophisticated agents that react to on-chain events in real-time, enabling use cases like arbitrage bots, liquidation monitors, and analytics dashboards.