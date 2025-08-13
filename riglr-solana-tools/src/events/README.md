# Solana Event Parsing Architecture

## Overview

This module provides a comprehensive event parsing system for various Solana protocols. The architecture supports two distinct patterns for event emission and parsing:

## Event Emission Patterns

### 1. Instruction Data Events
Some protocols encode all event data directly in the instruction data and accounts. These protocols do NOT emit events through inner instructions or program logs.

**Protocols using this pattern:**
- **Raydium CLMM** - All swap, liquidity, and position events are parsed from instruction data
- **Raydium AMM V4** - All swap, deposit, and withdraw events are parsed from instruction data
- **Bonk** - Trade and pool events are parsed from instruction data
- **PumpSwap** - Buy, sell, and pool events are parsed from instruction data

For these protocols:
- The `inner_instruction_discriminator` is an empty string (`""`)
- The `inner_instruction_parser` uses `empty_parse` which returns `None`
- All event data is extracted via the `instruction_parser` functions

### 2. Log-Based Events
Some protocols emit events through program logs that appear as inner instructions. These require parsing both the instruction data AND the inner instruction logs.

**Protocols using this pattern:**
- **Raydium CPMM** - Emits `SWAP_EVENT` and `DEPOSIT_EVENT` through logs

For these protocols:
- The `inner_instruction_discriminator` contains the event signature (e.g., `"0xe445a52e51cb9a1d0a11a9d2be8b72b1"`)
- The `inner_instruction_parser` deserializes the log data (often using borsh)
- Both instruction and inner instruction parsers may be needed for complete event data

## Implementation Details

### Parser Structure

Each protocol parser implements the `EventParser` trait and contains:
1. A `GenericEventParser` instance with event configurations
2. Instruction parsing functions for direct data extraction
3. Inner instruction parsing functions (which may be `empty_parse`)

### Event Configuration

Each event type is configured with:
```rust
GenericEventParseConfig {
    program_id: PROGRAM_ID,
    protocol_type: ProtocolType,
    inner_instruction_discriminator: "", // or event signature for log-based
    instruction_discriminator: &[u8],    // instruction discriminator bytes
    event_type: EventType,
    inner_instruction_parser: fn,        // empty_parse or actual parser
    instruction_parser: fn,               // always implemented
}
```

### Why Some Parsers Use `empty_parse`

The `empty_parse` function returning `None` is **intentional and correct** for protocols that don't emit log events. This is not a placeholder or incomplete implementation - it accurately reflects how these protocols work on-chain.

## Adding New Protocol Support

When adding a new protocol parser:

1. **Determine the event emission pattern:**
   - Check if the protocol emits events through logs (look for `msg!` or `sol_log_data` in the program)
   - Check if all data is available in instruction parameters

2. **Implement the appropriate parsers:**
   - For instruction data events: Implement only `instruction_parser` functions
   - For log-based events: Implement both `instruction_parser` and `inner_instruction_parser`

3. **Register in the factory:**
   - Add the parser to `EventParserFactory`
   - Export the program ID constant
   - Add to the `Protocol` enum

## Testing

Test coverage includes:
- Instruction data parsing validation
- Account extraction and validation
- Event ID generation
- Edge cases (insufficient data, missing accounts)

For protocols with `empty_parse`, no inner instruction tests are needed as this is the expected behavior.