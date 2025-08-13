# Solana Event Parsing System

This document provides a comprehensive guide for adding new protocol parsers to the RIGLR Solana event parsing system.

## Table of Contents

1. [System Architecture](#system-architecture)
2. [Quick Start Guide](#quick-start-guide)
3. [Detailed Implementation Guide](#detailed-implementation-guide)
4. [Common Patterns and Best Practices](#common-patterns-and-best-practices)
5. [Testing Your Parser](#testing-your-parser)
6. [Advanced Topics](#advanced-topics)
7. [Troubleshooting](#troubleshooting)

## System Architecture

The event parsing system is organized around several key components:

### Core Components

- **`core/traits.rs`**: Defines the `EventParser` and `UnifiedEvent` traits that all parsers must implement
- **`common/`**: Shared utilities for parsing instruction data, logs, and generating event IDs
- **`protocols/`**: Individual protocol implementations
- **`factory.rs`**: Manages parser registration and routing

### Event Types

The system supports two primary event emission patterns:

1. **Instruction-Based**: Events encoded directly in instruction parameters (Bonk, PumpSwap, Raydium AMM V4, Raydium CLMM)
2. **Log-Based**: Events emitted through program logs in inner instructions (Raydium CPMM)

## Quick Start Guide

### Adding a New Protocol Parser

1. **Create the protocol module directory**:
   ```bash
   mkdir -p src/events/protocols/your_protocol
   ```

2. **Create the required files**:
   ```
   src/events/protocols/your_protocol/
   ├── mod.rs           # Module exports
   ├── events.rs        # Event struct definitions
   ├── parser.rs        # Parser implementation
   ├── discriminators.rs # Instruction/log discriminators
   └── types.rs         # Protocol-specific types (optional)
   ```

3. **Register your protocol in the factory**:
   Add your parser to `factory.rs` and update the main module exports.

### Example: Minimal Instruction-Based Parser

```rust
// events.rs
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use crate::events::common::{EventMetadata, ProtocolType, EventType};
use crate::impl_unified_event;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YourProtocolSwapEvent {
    pub metadata: EventMetadata,
    pub amount_in: u64,
    pub amount_out: u64,
    pub user: Pubkey,
    pub pool: Pubkey,
}

impl_unified_event!(YourProtocolSwapEvent);

// parser.rs
use std::collections::HashMap;
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use solana_transaction_status::UiCompiledInstruction;

use crate::events::{
    common::{EventMetadata, EventType, ProtocolType, SwapInstructionParser, EventIdGenerator, InstructionValidator},
    core::traits::{EventParser, GenericEventParseConfig, GenericEventParser, UnifiedEvent},
    protocols::your_protocol::{discriminators, YourProtocolSwapEvent},
};

pub const YOUR_PROTOCOL_PROGRAM_ID: Pubkey = 
    solana_sdk::pubkey!("YourProgramIdHere11111111111111111111111111");

pub struct YourProtocolEventParser {
    inner: GenericEventParser,
}

impl YourProtocolEventParser {
    pub fn new() -> Self {
        let configs = vec![
            GenericEventParseConfig {
                program_id: YOUR_PROTOCOL_PROGRAM_ID,
                protocol_type: ProtocolType::YourProtocol,
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::SWAP,
                event_type: EventType::YourProtocolSwap,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_swap_instruction,
            },
        ];

        let inner = GenericEventParser::new(vec![YOUR_PROTOCOL_PROGRAM_ID], configs);
        Self { inner }
    }

    fn empty_parse(_data: &[u8], _metadata: EventMetadata) -> Option<Box<dyn UnifiedEvent>> {
        None
    }

    fn parse_swap_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        // Validate input
        if !InstructionValidator::validate_instruction(data, accounts, 16, 5) {
            return None;
        }

        // Parse parameters using common utilities
        let (amount_in, amount_out) = SwapInstructionParser::parse_exact_in_params(data)?;

        // Generate event ID
        let mut metadata = metadata;
        let id = EventIdGenerator::swap_id(&metadata.signature, &accounts[1], amount_in, "swap");
        metadata.set_id(id);

        Some(Box::new(YourProtocolSwapEvent {
            metadata,
            amount_in,
            amount_out,
            user: accounts[0],
            pool: accounts[1],
        }))
    }
}

// Implement EventParser trait by delegating to GenericEventParser
#[async_trait::async_trait]
impl EventParser for YourProtocolEventParser {
    fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner.inner_instruction_configs()
    }
    
    fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.inner.instruction_configs()
    }
    
    fn parse_events_from_inner_instruction(
        &self,
        inner_instruction: &UiCompiledInstruction,
        signature: &str,
        slot: u64,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn UnifiedEvent>> {
        self.inner.parse_events_from_inner_instruction(
            inner_instruction, signature, slot, block_time, program_received_time_ms, index
        )
    }

    fn parse_events_from_instruction(
        &self,
        instruction: &CompiledInstruction,
        accounts: &[Pubkey],
        signature: &str,
        slot: u64,
        block_time: Option<i64>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn UnifiedEvent>> {
        self.inner.parse_events_from_instruction(
            instruction, accounts, signature, slot, block_time, program_received_time_ms, index
        )
    }

    fn should_handle(&self, program_id: &Pubkey) -> bool {
        self.inner.should_handle(program_id)
    }

    fn supported_program_ids(&self) -> Vec<Pubkey> {
        self.inner.supported_program_ids()
    }
}
```

## Detailed Implementation Guide

### Step 1: Define Event Types

Each protocol needs to define its event structures. Events must:

1. Implement `Debug`, `Clone`, `Serialize`, `Deserialize`
2. Include a `metadata: EventMetadata` field
3. Use the `impl_unified_event!` macro to implement the `UnifiedEvent` trait

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MyProtocolEvent {
    pub metadata: EventMetadata,
    // Protocol-specific fields
    pub amount: u64,
    pub user: Pubkey,
    // ...
}

impl_unified_event!(MyProtocolEvent);
```

### Step 2: Define Discriminators

Discriminators identify specific instructions or log events:

```rust
// discriminators.rs
pub const SWAP: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8];
pub const DEPOSIT: &[u8] = &[9, 10, 11, 12, 13, 14, 15, 16];

// For log-based events
pub const SWAP_LOG: &str = "0x0102030405060708";
pub const DEPOSIT_LOG: &str = "0x090a0b0c0d0e0f10";
```

### Step 3: Implement the Parser

Choose the appropriate base pattern:

#### For Instruction-Based Protocols

Use `GenericEventParser` as the foundation and implement custom parsing functions:

```rust
impl MyProtocolEventParser {
    fn parse_swap_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        // 1. Validate input
        if !InstructionValidator::validate_instruction(data, accounts, min_data_len, min_accounts) {
            return None;
        }

        // 2. Parse instruction parameters
        let (param1, param2) = SwapInstructionParser::parse_exact_in_params(data)?;
        
        // 3. Extract accounts
        let swap_accounts = AccountMapper::extract_swap_accounts(accounts)?;

        // 4. Generate event ID
        let mut metadata = metadata;
        let id = EventIdGenerator::swap_id(&metadata.signature, &swap_accounts.pool, param1, "swap");
        metadata.set_id(id);

        // 5. Create event
        Some(Box::new(MyProtocolSwapEvent {
            metadata,
            amount_in: param1,
            amount_out: param2,
            user: swap_accounts.user,
            pool: swap_accounts.pool,
            // ...
        }))
    }
}
```

#### For Log-Based Protocols

Use `LogEventParser` utilities for log parsing:

```rust
impl MyProtocolEventParser {
    fn parse_swap_log(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        // For actual implementation, you would parse the log data manually
        // based on your protocol's log format, similar to instruction parsing
        
        // Example manual parsing (adjust to your protocol's format):
        if data.len() < 16 {
            return None;
        }
        
        let amount_in = read_u64_le(data, 0)?;
        let amount_out = read_u64_le(data, 8)?;
        
        // Generate ID
        let mut metadata = metadata;
        let id = LogEventIdGenerator::log_swap_id(&metadata.signature, "pool_from_data", "buy");
        metadata.set_id(id);

        Some(Box::new(MyProtocolSwapEvent { 
            metadata, 
            amount_in,
            amount_out,
            // ... other fields
        }))
    }
}
```

### Step 4: Register with the Factory

Add your parser to the factory system:

```rust
// In factory.rs
use crate::events::protocols::my_protocol::MyProtocolEventParser;

impl EventParserFactory {
    pub fn create_all_parsers() -> Vec<Box<dyn EventParser>> {
        vec![
            // Existing parsers...
            Box::new(MyProtocolEventParser::new()),
        ]
    }
}
```

### Step 5: Update Module Exports

```rust
// In protocols/mod.rs
pub mod my_protocol;

// In events/mod.rs - add to re-exports if needed
pub use protocols::my_protocol::*;
```

## Common Patterns and Best Practices

### Error Handling

Always use `Option<T>` return types for parsing functions and handle errors gracefully:

```rust
fn parse_instruction(data: &[u8], accounts: &[Pubkey]) -> Option<MyEvent> {
    // Validate inputs first
    if data.len() < MINIMUM_DATA_LENGTH {
        return None;
    }
    
    // Use safe parsing utilities
    let amount = read_u64_le(data, 0)?;
    let user = accounts.get(0).copied()?;
    
    Some(MyEvent { amount, user, .. })
}
```

### Input Validation

Use the provided validation utilities:

```rust
// Validate before parsing
if !InstructionValidator::validate_instruction(data, accounts, 16, 5) {
    return None;
}

// Validate account indices
if !validate_account_indices(&instruction.accounts, accounts.len()) {
    return None;
}
```

### Parameter Parsing

Leverage common parsing utilities:

```rust
// For swap operations
let (amount_in, min_out) = SwapInstructionParser::parse_exact_in_params(data)?;
let (max_in, amount_out) = SwapInstructionParser::parse_exact_out_params(data)?;

// For liquidity operations  
let (max_base, max_quote, side) = LiquidityInstructionParser::parse_deposit_params(data)?;
let (liquidity, tick_lower, tick_upper, amount0, amount1) = 
    LiquidityInstructionParser::parse_liquidity_with_bounds(data)?;

// For raw data parsing
let value = read_u64_le(data, offset)?;
let price = read_u128_le(data, offset)?;
let tick = read_i32_le(data, offset)?;
```

### ID Generation

Use consistent ID patterns:

```rust
// For swaps
let id = EventIdGenerator::swap_id(signature, &pool, amount, "buy");

// For pool operations
let id = EventIdGenerator::pool_creation_id(signature, &pool, &creator);

// For liquidity operations
let id = EventIdGenerator::liquidity_id(signature, &pool, &user, amount, "add");

// For position operations (CLMM-style)
let id = EventIdGenerator::position_id(signature, &position_mint, "open");
```

### Account Mapping

Use structured account extraction:

```rust
// For standard swap accounts
let swap_accounts = AccountMapper::extract_swap_accounts(accounts)?;
// Now use: swap_accounts.user, swap_accounts.pool, etc.

// For pool creation accounts
let pool_accounts = AccountMapper::extract_pool_accounts(accounts)?;
// Now use: pool_accounts.creator, pool_accounts.base_mint, etc.
```

### Borsh Deserialization

For protocols that use borsh serialization (mainly in logs):

```rust
// Simple borsh parsing (for log-based events)
let event = LogBorshParser::parse_borsh_log::<MyEvent>(data)?;

// Combined log parsing with discriminator validation
let event = LogBorshParser::parse_borsh_with_metadata::<MyEvent>(
    log_data,
    expected_discriminator,
    skip_bytes,
)?;
```

Note: Most instruction-based protocols parse parameters manually rather than using borsh deserialization.

### Math Calculations

For CLMM protocols requiring price/tick conversions:

```rust
// Convert sqrt price to tick
let tick = PriceCalculator::sqrt_price_to_tick(sqrt_price_x64);

// Convert tick to price
let price = PriceCalculator::tick_to_price(tick);
```

### Configuration Patterns

Define constants for your protocol:

```rust
pub const MY_PROTOCOL_PROGRAM_ID: Pubkey = 
    solana_sdk::pubkey!("YourProgramIdHere11111111111111111111111111");

// Version-specific program IDs if needed
pub const MY_PROTOCOL_V2_PROGRAM_ID: Pubkey = 
    solana_sdk::pubkey!("YourV2ProgramId111111111111111111111111111");
```

## Testing Your Parser

### Unit Tests

Create comprehensive unit tests for your parsing functions:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn test_parse_swap_instruction() {
        // Create test data
        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&1000u64.to_le_bytes());
        data[8..16].copy_from_slice(&500u64.to_le_bytes());
        
        let accounts = vec![Pubkey::new_unique(); 10];
        let metadata = EventMetadata::new(
            "test_signature".to_string(),
            "test_signature".to_string(),
            12345,
            1234567890,
            1234567890000,
            ProtocolType::MyProtocol,
            EventType::MyProtocolSwap,
            MY_PROTOCOL_PROGRAM_ID,
            "0".to_string(),
            chrono::Utc::now().timestamp_millis(),
        );

        let result = MyProtocolEventParser::parse_swap_instruction(&data, &accounts, metadata);
        assert!(result.is_some());
        
        let event = result.unwrap();
        // Verify event fields...
    }

    #[test]
    fn test_invalid_data_handling() {
        // Test with insufficient data
        let data = vec![0u8; 8]; // Too short
        let accounts = vec![Pubkey::new_unique(); 5];
        let metadata = /* ... */;

        let result = MyProtocolEventParser::parse_swap_instruction(&data, &accounts, metadata);
        assert!(result.is_none());
    }
}
```

### Integration Tests

Test with real transaction data:

```rust
#[test]
#[ignore] // Mark as integration test
fn test_parse_real_transaction() {
    // Use real transaction data from your protocol
    let parser = MyProtocolEventParser::new();
    // Test parsing with actual instruction data...
}
```

### Property-Based Tests

Use property-based testing for robust validation:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_parse_swap_never_panics(
        amount_in in 0u64..u64::MAX,
        amount_out in 0u64..u64::MAX,
    ) {
        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&amount_in.to_le_bytes());
        data[8..16].copy_from_slice(&amount_out.to_le_bytes());
        
        let accounts = vec![Pubkey::new_unique(); 10];
        let metadata = /* create test metadata */;
        
        // Should never panic, only return None for invalid input
        let _ = MyProtocolEventParser::parse_swap_instruction(&data, &accounts, metadata);
    }
}
```

## Advanced Topics

### Event Merging

For protocols that emit both instruction and log events:

```rust
impl UnifiedEvent for MyProtocolEvent {
    fn merge(&mut self, other: Box<dyn UnifiedEvent>) {
        if let Some(other_event) = other.as_any().downcast_ref::<MyProtocolEvent>() {
            // Merge fields from log event into instruction event
            self.actual_amount_out = other_event.actual_amount_out;
            self.execution_price = other_event.execution_price;
        }
    }
}
```

### Custom Discriminator Patterns

For protocols with complex discriminator schemes:

```rust
impl MyProtocolEventParser {
    fn get_discriminator_for_version(version: u8) -> &'static [u8] {
        match version {
            1 => discriminators::SWAP_V1,
            2 => discriminators::SWAP_V2,
            _ => discriminators::SWAP_DEFAULT,
        }
    }
}
```

### Multi-Program Support

For protocols spanning multiple programs:

```rust
impl MyProtocolEventParser {
    pub fn new() -> Self {
        let program_ids = vec![
            MY_PROTOCOL_MAIN_PROGRAM_ID,
            MY_PROTOCOL_ROUTER_PROGRAM_ID,
            MY_PROTOCOL_HELPER_PROGRAM_ID,
        ];
        
        // Configure events for each program...
        let inner = GenericEventParser::new(program_ids, configs);
        Self { inner }
    }
}
```

### Performance Optimization

#### Pre-allocate Collections

```rust
fn parse_multiple_events(&self, instructions: &[Instruction]) -> Vec<Box<dyn UnifiedEvent>> {
    let mut events = Vec::with_capacity(instructions.len()); // Pre-allocate
    
    for instruction in instructions {
        if let Some(event) = self.parse_instruction(instruction) {
            events.push(event);
        }
    }
    
    events
}
```

#### Cache Expensive Computations

```rust
lazy_static! {
    static ref TICK_CACHE: HashMap<u128, i32> = HashMap::new();
}

fn cached_sqrt_price_to_tick(sqrt_price: u128) -> i32 {
    *TICK_CACHE.entry(sqrt_price)
        .or_insert_with(|| PriceCalculator::sqrt_price_to_tick(sqrt_price))
}
```

## Troubleshooting

### Common Issues

#### Parser Not Triggered

**Problem**: Your parser's functions are never called.

**Solutions**:
1. Verify the program ID is correct
2. Check discriminator values match the actual instruction data
3. Ensure the parser is registered in the factory
4. Verify the instruction data format matches expectations

#### Events Not Parsing Correctly

**Problem**: Parsing function returns `None` for valid data.

**Solutions**:
1. Add debug logging to see where parsing fails:
   ```rust
   fn parse_instruction(data: &[u8], accounts: &[Pubkey]) -> Option<Event> {
       if data.len() < 16 {
           eprintln!("Data too short: {} bytes", data.len());
           return None;
       }
       // ...
   }
   ```
2. Verify byte order (little-endian vs big-endian)
3. Check account ordering matches protocol specification
4. Validate discriminator extraction logic

#### Missing Events

**Problem**: Some events are not being captured.

**Solutions**:
1. Check if events are emitted as logs vs instruction data
2. Verify all instruction variants have configurations
3. Check if the protocol uses nested instructions
4. Ensure discriminators are complete and accurate

#### Performance Issues

**Problem**: Parser is slow with large transaction volumes.

**Solutions**:
1. Profile parsing functions to identify bottlenecks
2. Pre-allocate collections where possible
3. Cache expensive computations
4. Consider async processing for I/O-bound operations

### Debugging Techniques

#### Enable Debug Logging

```rust
use log::{debug, warn, error};

fn parse_instruction(data: &[u8], accounts: &[Pubkey]) -> Option<Event> {
    debug!("Parsing instruction with {} bytes data, {} accounts", data.len(), accounts.len());
    
    if data.len() < MINIMUM_LENGTH {
        warn!("Insufficient data length: expected >= {}, got {}", MINIMUM_LENGTH, data.len());
        return None;
    }
    
    // ...
}
```

#### Hex Dump Utility

```rust
fn hex_dump(data: &[u8], label: &str) {
    eprintln!("{}: {}", label, hex::encode(data));
}

// Usage in parsing function
hex_dump(data, "Instruction data");
hex_dump(&accounts[0].to_bytes(), "First account");
```

#### Transaction Replay

Create utilities to replay problematic transactions:

```rust
#[cfg(test)]
mod debug_tests {
    #[test]
    #[ignore]
    fn debug_specific_transaction() {
        let signature = "YourProblematicTransactionSignature";
        // Load transaction data and replay parsing...
    }
}
```

## Performance Benchmarks

To ensure your parser meets performance requirements:

```rust
#[cfg(test)]
mod benchmarks {
    use test::Bencher;

    #[bench]
    fn bench_parse_swap_instruction(b: &mut Bencher) {
        let data = create_test_swap_data();
        let accounts = create_test_accounts();
        let metadata = create_test_metadata();

        b.iter(|| {
            MyProtocolEventParser::parse_swap_instruction(&data, &accounts, metadata.clone())
        });
    }
}
```

Run benchmarks with:
```bash
cargo bench --package riglr-solana-tools
```

## Conclusion

This guide provides the foundation for implementing robust, efficient parsers for new Solana protocols. Remember to:

1. **Start simple**: Implement basic functionality first, then add complexity
2. **Test thoroughly**: Unit tests, integration tests, and property-based tests
3. **Follow patterns**: Use the established utilities and patterns for consistency
4. **Document well**: Add clear documentation for future maintainers
5. **Performance matters**: Profile and optimize critical parsing paths

For questions or issues, refer to existing protocol implementations as examples:
- **Instruction-based**: `protocols/bonk/parser.rs`
- **Log-based**: `protocols/raydium_cpmm/parser.rs`
- **Complex CLMM**: `protocols/raydium_clmm/parser.rs`