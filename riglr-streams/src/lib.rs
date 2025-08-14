//! RIGLR Streams - Event-driven streaming capabilities for blockchain agents
//!
//! This crate provides streaming components that enable developers to build
//! proactive, event-driven blockchain agents while maintaining RIGLR's
//! library-first philosophy.

pub mod core;
pub mod solana;
pub mod evm;
pub mod external;
pub mod tools;
pub mod production;

/// Prelude module for convenient imports
pub mod prelude {
    // Core streaming abstractions
    pub use crate::core::{
        Stream, StreamEvent, StreamHealth, StreamError, StreamManager, HandlerExecutionMode,
        StreamedEvent, DynamicStreamedEvent, IntoStreamedEvent,
        ComposableStream, FinancialStreamExt, BufferStrategy, combinators,
        MetricsCollector, StreamManagerBuilder,
    };
    
    // Enhanced operators with events-core integration
    pub use crate::core::enhanced_operators::{
        BatchEvent, BatchedStream, EnhancedStream, EnhancedStreamExt,
    };
    
    // Event-triggered tools
    pub use crate::tools::{EventTriggeredTool, EventTriggerBuilder, ConditionCombinator};
    
    // Re-export events-core utilities for enhanced streaming capabilities
    pub use riglr_events_core::prelude::*;
}