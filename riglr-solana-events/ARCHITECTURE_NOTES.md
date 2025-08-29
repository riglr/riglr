# Solana Events Architecture Notes

## EventMetadata Design Pattern

### The Problem
The `riglr-events-core` crate provides a generic `EventMetadata` type with 3 constructor parameters, but Solana events need additional fields (signature, slot, etc.) and Borsh serialization support.

### The Solution: Explicit Wrapper with Deref

Instead of using a confusing type alias, we use an explicit wrapper pattern:

```rust
// BAD: Confusing type alias
pub use SolanaEventMetadata as EventMetadata; // DON'T DO THIS

// GOOD: Explicit types
pub struct SolanaEventMetadata {
    // Solana-specific fields
    pub signature: String,
    pub slot: u64,
    pub event_type: EventType,
    pub protocol_type: ProtocolType,
    pub index: String,
    pub program_received_time_ms: i64,
    
    // Wrapped core metadata
    #[borsh(skip)]
    #[serde(skip)]
    pub core: EventMetadata,
}

// Implement Deref for ergonomic field access
impl Deref for SolanaEventMetadata {
    type Target = EventMetadata;
    fn deref(&self) -> &Self::Target {
        &self.core
    }
}
```

### Benefits

1. **No Ambiguity**: It's always clear whether you're using `SolanaEventMetadata` or core `EventMetadata`
2. **Ergonomic Access**: With `Deref`, you can access core fields directly: `solana_meta.id`
3. **Single Source of Truth**: Solana-specific data lives only on the wrapper
4. **Type Safety**: The compiler enforces correct usage

### Usage Examples

```rust
// Creating Solana metadata
let solana_meta = SolanaEventMetadata::new(
    signature,
    slot,
    event_type,
    protocol_type,
    index,
    received_time_ms,
    core_metadata,
);

// Accessing fields
let slot = solana_meta.slot;           // Solana-specific field
let id = solana_meta.id;               // Core field via Deref
let kind = &solana_meta.kind;          // Core field via Deref
let signature = &solana_meta.signature; // Solana-specific field

// For Event trait implementation
impl Event for SomeEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata.core  // Return the core metadata
    }
}
```

### Implementation Status

✅ **Migration Complete (2025-01-03)**

The entire codebase now uses the explicit wrapper pattern:

1. All imports use `SolanaEventMetadata` explicitly - no type alias confusion
2. All struct definitions use `SolanaEventMetadata` for their metadata field
3. All Event trait implementations correctly return `&self.metadata.core`
4. Field access leverages Deref for seamless access to core metadata fields

✅ **Duplication Elimination Complete (2025-01-03)**

Eliminated data duplication between `SolanaEventMetadata` fields and `core.chain_data`:

1. Introduced `create_core_metadata()` function that creates base `EventMetadata` without Solana-specific chain data
2. Refactored all `create_solana_metadata()` usage to use the new non-duplicating pattern
3. Updated all protocol event constructors to avoid putting Solana fields in both wrapper and chain_data
4. Maintained backward compatibility by deprecating the old duplication-prone functions

### Data Flow

```
SolanaEventMetadata (wrapper)
├── signature: String           ← Solana-specific
├── slot: u64                   ← Solana-specific  
├── event_type: EventType       ← Solana-specific
├── protocol_type: ProtocolType ← Solana-specific
├── index: String               ← Solana-specific
├── program_received_time_ms: i64 ← Solana-specific
└── core: EventMetadata         ← Generic metadata
    ├── id: String
    ├── kind: EventKind
    ├── timestamp: DateTime<Utc>
    ├── received_at: DateTime<Utc>
    ├── source: String
    ├── chain_data: Option<ChainData>  ← Should NOT duplicate Solana fields
    └── custom: HashMap<String, Value>
```

### Important: Avoid Data Duplication

The `chain_data` field in core `EventMetadata` should NOT contain the same information as the wrapper's fields. This prevents:
- Inconsistencies between the two sources
- Unnecessary memory usage
- Confusion about which is authoritative

### Best Practices

1. **Be Explicit**: Always use `SolanaEventMetadata` in Solana-specific code
2. **Use Non-Duplicating Constructors**: Use `create_core_metadata()` + `SolanaEventMetadata::new()` instead of deprecated functions that duplicate data
3. **Avoid Chain Data Duplication**: Don't put Solana-specific fields in both the wrapper and `core.chain_data`
4. **Document Intent**: Make it clear in comments when you're working with Solana-specific vs core metadata
5. **Test Both Layers**: Ensure tests verify both Solana fields and core fields work correctly

#### Recommended Creation Pattern

```rust
// GOOD: Non-duplicating pattern
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

// AVOID: Deprecated functions that duplicate data
let metadata = create_solana_metadata(/* many params that duplicate between wrapper and chain_data */);
```