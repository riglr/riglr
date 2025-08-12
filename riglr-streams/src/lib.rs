//! RIGLR Streams - Event-driven streaming capabilities for blockchain agents
//!
//! This crate provides streaming components that enable developers to build
//! proactive, event-driven blockchain agents while maintaining RIGLR's
//! library-first philosophy.

/// Core streaming abstractions, operators, and management utilities
pub mod core;
/// EVM blockchain streaming capabilities and multi-chain management
pub mod evm;
/// External data source streaming for exchanges and blockchain services
pub mod external;
/// Production-ready utilities for stream management and monitoring
pub mod production;
/// Solana blockchain streaming capabilities via Geyser
pub mod solana;
/// Event-triggered tools and condition matching utilities
pub mod tools;

/// Prelude module for convenient imports
pub mod prelude {
    // Core streaming abstractions
    pub use crate::core::{
        combinators, BufferStrategy, ComposableStream, DynamicStreamedEvent, FinancialStreamExt,
        HandlerExecutionMode, IntoStreamedEvent, MetricsCollector, Stream, StreamError,
        StreamEvent, StreamHealth, StreamManager, StreamManagerBuilder, StreamedEvent,
    };

    // Enhanced operators with events-core integration
    pub use crate::core::enhanced_operators::{
        BatchEvent, BatchedStream, EnhancedStream, EnhancedStreamExt,
    };

    // Event-triggered tools
    pub use crate::tools::{ConditionCombinator, EventTriggerBuilder, EventTriggeredTool};

    // Re-export events-core utilities for enhanced streaming capabilities
    pub use riglr_events_core::prelude::*;
}
