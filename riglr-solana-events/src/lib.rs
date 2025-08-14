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

/// Macro to implement UnifiedEvent trait for event types
#[macro_export]
macro_rules! impl_unified_event {
    ($event_type:ty) => {
        impl $crate::core::UnifiedEvent for $event_type {
            fn id(&self) -> &str {
                &self.metadata.id
            }

            fn event_type(&self) -> $crate::types::EventType {
                self.metadata.event_type.clone()
            }

            fn signature(&self) -> &str {
                &self.metadata.signature
            }

            fn slot(&self) -> u64 {
                self.metadata.slot
            }

            fn program_received_time_ms(&self) -> i64 {
                self.metadata.program_received_time_ms
            }

            fn program_handle_time_consuming_ms(&self) -> i64 {
                self.metadata.program_handle_time_consuming_ms
            }

            fn set_program_handle_time_consuming_ms(&mut self, time: i64) {
                self.metadata.program_handle_time_consuming_ms = time;
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn clone_boxed(&self) -> Box<dyn $crate::core::UnifiedEvent> {
                Box::new(self.clone())
            }

            fn set_transfer_data(
                &mut self,
                _transfer_data: Vec<$crate::types::TransferData>,
                _swap_data: Option<$crate::types::SwapData>,
            ) {
                // Default implementation - events can override if needed
            }

            fn index(&self) -> String {
                self.metadata.index.clone()
            }

            fn protocol_type(&self) -> $crate::types::ProtocolType {
                self.metadata.protocol_type.clone()
            }
        }
    };
}

pub mod prelude {
    pub use crate::events::core::traits::{EventParser, UnifiedEvent, GenericEventParser, GenericEventParseConfig};
    pub use crate::types::*;
    pub use crate::events::factory::{EventParserFactory, Protocol, MultiEventParser};
    pub use crate::events::core::traits::*;
    pub use crate::solana_events::{SolanaEvent, ToSolanaEvent};
    pub use crate::solana_parser::{SolanaEventParser, SolanaInnerInstructionParser, SolanaTransactionInput, SolanaInnerInstructionInput};
    // Re-export new high-performance parsing components
    pub use crate::zero_copy::{ZeroCopyEvent, ZeroCopySwapEvent, ZeroCopyLiquidityEvent, ByteSliceEventParser, 
                               CustomDeserializer, MemoryMappedParser, SIMDPatternMatcher, BatchEventParser};
    pub use crate::parsers::{RaydiumV4Parser, RaydiumV4ParserFactory, PumpFunParser, PumpFunParserFactory,
                             JupiterParser, JupiterParserFactory, MetaplexParser, MetaplexParserFactory};
    pub use crate::pipelines::{ParsingPipeline, ParsingPipelineBuilder, EventEnricher, ValidationPipeline};
    // Re-export riglr-events-core types for convenience
    pub use riglr_events_core::prelude::*;
}

pub mod constants;
pub mod error;
pub mod types;
pub mod utils;
pub mod events;
pub mod solana_events;
pub mod solana_parser;
pub mod zero_copy;
pub mod parsers;
pub mod pipelines;

// Create core module that re-exports from events::core::traits for backward compatibility
pub mod core {
    pub use crate::events::core::traits::*;
}

// Re-export key types at crate root for backward compatibility
pub use events::core::traits::{UnifiedEvent, EventParser, GenericEventParser, GenericEventParseConfig};
pub use types::{EventType, ProtocolType, EventMetadata, StreamMetadata, TransferData, SwapData};
pub use events::factory::{EventParserFactory, Protocol, MultiEventParser};

// New riglr-events-core integration
pub use solana_events::{SolanaEvent, ToSolanaEvent};
pub use solana_parser::{SolanaEventParser, SolanaInnerInstructionParser, SolanaTransactionInput, SolanaInnerInstructionInput};

// Re-export core error types for convenience
pub use error::{ParseError, ParseResult, EventError, EventResult};
