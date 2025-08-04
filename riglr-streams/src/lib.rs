//! RIGLR Streams - Event-driven streaming capabilities for blockchain agents
//!
//! This crate provides streaming components that enable developers to build
//! proactive, event-driven blockchain agents while maintaining RIGLR's
//! library-first philosophy.

pub mod core;
pub mod evm;
pub mod external;
pub mod production;
pub mod solana;
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
