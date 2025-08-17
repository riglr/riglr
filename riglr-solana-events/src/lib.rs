//! Standalone Solana event parsing library integrated with riglr-events-core.
//!
//! This library provides comprehensive Solana event parsing capabilities with support for
//! multiple protocols including Orca, Meteora, MarginFi, Jupiter, Raydium, and more.
//!
//! ## Migration to riglr-events-core
//!
//! This crate has been enhanced to support both the legacy `UnifiedEvent` interface
//! and the new `riglr-events-core::Event` interface for seamless migration.
//!
//! ### Legacy Usage (Maintained for backward compatibility)
//!
//! ```rust,no_run
//! use riglr_solana_events::prelude::*;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a multi-parser for various protocols
//! let parser = EventParserFactory::with_all_parsers().build();
//!
//! // Parse events using legacy interface
//! let events = parser.parse_events_from_instruction(
//!     &instruction,
//!     &accounts,
//!     "signature",
//!     slot,
//!     Some(block_time),
//!     received_time,
//!     "0".to_string(),
//! );
//!
//! for event in events {
//!     println!("Legacy event: {}", event.id());
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### New riglr-events-core Usage
//!
//! ```rust,no_run
//! use riglr_solana_events::prelude::*;
//! use riglr_events_core::prelude::*;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a Solana event parser that implements riglr-events-core traits
//! let parser = SolanaEventParser::new();
//!
//! let input = SolanaTransactionInput::new(
//!     instruction,
//!     accounts,
//!     "signature".to_string(),
//!     slot,
//!     Some(block_time),
//!     0,
//! );
//!
//! // Parse events using new interface - returns Vec<Box<dyn Event>>
//! let events = parser.parse(input).await?;
//!
//! for event in events {
//!     // Events implement both UnifiedEvent and Event traits
//!     println!("New event: {}", event.id());
//!     println!("Kind: {:?}", event.kind());
//!     println!("Source: {}", event.source());
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Hybrid Usage - Best of Both Worlds
//!
//! ```rust,no_run
//! use riglr_solana_events::prelude::*;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create SolanaEvent that implements both trait interfaces
//! let solana_event = SolanaEvent::swap(
//!     "event-1".to_string(),
//!     "signature123".to_string(),
//!     12345,
//!     1234567890,
//!     ProtocolType::Jupiter,
//!     program_id,
//!     input_mint,
//!     output_mint,
//!     1_000_000,
//!     950_000,
//! );
//!
//! // Use legacy interface
//! println!("Slot: {}", solana_event.slot());
//! println!("Protocol: {}", solana_event.protocol_type());
//!
//! // Use new interface
//! println!("Kind: {:?}", solana_event.kind());
//! let json = solana_event.to_json()?;
//! println!("JSON: {}", json);
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//!
//! - **Protocol Support**: Orca, Meteora, MarginFi, Jupiter, Raydium AMM/CLMM/CPMM, PumpSwap, Bonk
//! - **Zero-copy Parsing**: Efficient parsing with minimal allocations where possible
//! - **Dual Interface**: Both legacy `UnifiedEvent` and new `Event` trait support
//! - **Type Safety**: Strongly typed events with rich metadata
//! - **Async Ready**: Full async/await support with proper Send/Sync bounds
//! - **Error Rich**: Comprehensive error handling with context preservation
//!
//! ## Migration Guide
//!
//! 1. **Immediate**: Use `SolanaEvent` for new event types - it implements both interfaces
//! 2. **Gradual**: Replace direct `UnifiedEvent` usage with `Event` trait where possible
//! 3. **Parser Migration**: Move from `MultiEventParser` to `SolanaEventParser` for new code
//! 4. **Error Handling**: Adopt `EventError` and `EventResult` types for better error context
//!
//! The migration maintains 100% backward compatibility while providing access to enhanced
//! functionality from riglr-events-core.

// UnifiedEvent macro has been removed. Use riglr_events_core::Event trait directly.

/// Macro to implement Event trait for event types
#[macro_export]
macro_rules! impl_event {
    ($event_type:ty) => {
        impl riglr_events_core::Event for $event_type {
            fn kind(&self) -> riglr_events_core::EventKind {
                match $crate::metadata_helpers::get_event_type(&self.metadata).unwrap_or_default() {
                    $crate::types::EventType::Swap => riglr_events_core::EventKind::Swap,
                    $crate::types::EventType::Transfer => riglr_events_core::EventKind::Transfer,
                    $crate::types::EventType::Liquidation => {
                        riglr_events_core::EventKind::Custom("liquidation".to_string())
                    }
                    $crate::types::EventType::Deposit => {
                        riglr_events_core::EventKind::Custom("deposit".to_string())
                    }
                    $crate::types::EventType::Withdraw => {
                        riglr_events_core::EventKind::Custom("withdraw".to_string())
                    }
                    $crate::types::EventType::Borrow => {
                        riglr_events_core::EventKind::Custom("borrow".to_string())
                    }
                    $crate::types::EventType::Repay => {
                        riglr_events_core::EventKind::Custom("repay".to_string())
                    }
                    $crate::types::EventType::CreatePool => {
                        riglr_events_core::EventKind::Custom("create_pool".to_string())
                    }
                    $crate::types::EventType::AddLiquidity => {
                        riglr_events_core::EventKind::Custom("add_liquidity".to_string())
                    }
                    $crate::types::EventType::RemoveLiquidity => {
                        riglr_events_core::EventKind::Custom("remove_liquidity".to_string())
                    }
                    $crate::types::EventType::Unknown => {
                        riglr_events_core::EventKind::Custom("unknown".to_string())
                    }
                }
            }

            fn metadata(&self) -> &riglr_events_core::EventMetadata {
                // Create compatible metadata if needed
                self.core_metadata
                    .as_ref()
                    .unwrap_or_else(|| panic!("Event must have core_metadata initialized"))
            }

            fn signature(&self) -> &str {
                &self.metadata.signature
            }

            fn slot(&self) -> Option<u64> {
                Some(self.metadata.slot)
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn clone_boxed(&self) -> Box<dyn riglr_events_core::Event> {
                Box::new(self.clone())
            }
        }
    };
}

/// Convenient re-exports for common types and traits used throughout the library
pub mod prelude {
    pub use crate::events::core::traits::{
        EventParser, GenericEventParseConfig, GenericEventParser,
    };
    // Re-export types but exclude EventMetadata as it conflicts with riglr_events_core
    pub use crate::events::core::traits::*;
    pub use crate::events::factory::{EventParserRegistry, Protocol};
    pub use crate::solana_events::{SolanaEvent, ToSolanaEvent};
    pub use crate::solana_parser::{
        SolanaEventParser, SolanaInnerInstructionInput, SolanaInnerInstructionParser,
        SolanaTransactionInput,
    };
    pub use crate::types::{EventType, ProtocolType, SwapData, TransferData};
    // Re-export new high-performance parsing components
    pub use crate::parsers::{
        JupiterParser, JupiterParserFactory, MetaplexParser, MetaplexParserFactory, PumpFunParser,
        PumpFunParserFactory, RaydiumV4Parser, RaydiumV4ParserFactory,
    };
    pub use crate::pipelines::{
        EventEnricher, ParsingPipeline, ParsingPipelineBuilder, ValidationPipeline,
    };
    pub use crate::zero_copy::{
        BatchEventParser, ByteSliceEventParser, CustomDeserializer, MemoryMappedParser,
        SIMDPatternMatcher, ZeroCopyEvent, ZeroCopyLiquidityEvent, ZeroCopySwapEvent,
    };
    // Re-export riglr-events-core types for convenience
    pub use riglr_events_core::prelude::*;
}

/// Constants used throughout the Solana events library
pub mod constants;
/// Error types and utilities for Solana event parsing
pub mod error;
/// Core event structures and parsing logic
pub mod events;
/// Helper functions for working with Solana-specific metadata
pub mod metadata_helpers;
/// Solana-specific metadata wrapper
pub mod solana_metadata;
/// High-performance parsers for specific protocols
pub mod parsers;
/// Event processing pipelines for validation and enrichment
pub mod pipelines;
/// Solana-specific event types that implement both legacy and new interfaces
pub mod solana_events;
/// Parser implementation for the riglr-events-core Event trait
pub mod solana_parser;
/// Common types and data structures used across the library
pub mod types;
/// Utility functions for Solana event processing
pub mod utils;
/// Zero-copy parsing implementations for high-performance scenarios
pub mod zero_copy;

/// Backward compatibility module that re-exports core traits from events::core::traits
pub mod core {
    pub use crate::events::core::traits::*;
}

// Re-export key types at crate root
pub use events::core::traits::{EventParser, GenericEventParseConfig, GenericEventParser};
pub use events::factory::{EventParserRegistry, Protocol};
pub use types::{EventMetadata, EventType, ProtocolType, StreamMetadata, SwapData, TransferData};

// New riglr-events-core integration
pub use solana_events::{SolanaEvent, ToSolanaEvent};
pub use solana_parser::{
    SolanaEventParser, SolanaInnerInstructionInput, SolanaInnerInstructionParser,
    SolanaTransactionInput,
};

// Re-export core error types for convenience
pub use error::{EventError, EventResult, ParseError, ParseResult};
