/// Stream configuration and builder utilities
pub mod builder;
/// Configuration structures for stream management
pub mod config;
/// Connection management and pooling functionality
pub mod connection;
/// Advanced stream operators for complex data processing
pub mod enhanced_operators;
/// Error types and result definitions for streaming operations
pub mod error;
/// Adapters for converting between different event formats
pub mod event_adapter;
/// Financial analysis operators for market data streams
pub mod financial_operators;
/// Stream lifecycle management and coordination
pub mod manager;
/// Performance monitoring and metrics collection
pub mod metrics;
/// Mock streaming utilities for testing and development
pub mod mock_stream;
/// Core stream transformation and composition operators
pub mod operators;
/// Event processing and pattern matching utilities
pub mod processor;
/// Core streaming abstractions and implementations
pub mod stream;
/// Event wrapper types for streaming data
pub mod streamed_event;
/// Macros for resilience patterns in stream processing
#[macro_use]
pub mod resilience_macro;
/// Tests for numeric conversion functionality
#[cfg(test)]
mod test_asnumeric;

pub use mock_stream::{MockConfig, MockStream};
pub use stream::{DynamicStream, DynamicStreamWrapper, Stream, StreamEvent, StreamHealth};
pub use streamed_event::{
    DynamicStreamedEvent, IntoDynamicStreamedEvent, IntoStreamedEvent, StreamedEvent,
};
// pub use riglr_solana_events::StreamMetadata; // Temporarily disabled

/// Temporary StreamMetadata for migration phase
#[derive(Debug, Clone, serde::Serialize)]
pub struct StreamMetadata {
    /// Source identifier for the stream that generated this event
    pub stream_source: String,
    /// Timestamp when this event was received by the stream processor
    #[serde(with = "systemtime_as_millis")]
    pub received_at: std::time::SystemTime,
    /// Optional sequence number for ordering events within a stream
    pub sequence_number: Option<u64>,
    /// Optional custom metadata specific to the event or stream source
    pub custom_data: Option<serde_json::Value>,
}

mod systemtime_as_millis {
    use serde::{Serialize, Serializer};
    use std::time::SystemTime;

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let millis = time
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        millis.serialize(serializer)
    }
}
pub use builder::{from_config_file, from_env, StreamConfig, StreamManagerBuilder};
pub use config::{
    BackpressureConfig, BackpressureStrategy, BatchConfig, ConnectionConfig, MetricsConfig,
    StreamClientConfig,
};
pub use connection::{
    CircuitBreaker, ConnectionHealth, ConnectionManager, ConnectionPool, ConnectionState,
};
pub use error::{StreamError, StreamResult};
pub use financial_operators::{AsNumeric, FinancialStreamExt};
pub use manager::{EventHandler, HandlerExecutionMode, LoggingEventHandler, StreamManager};
pub use metrics::{GlobalMetrics, HandlerMetrics, MetricsCollector, MetricsTimer, StreamMetrics};
pub use operators::{
    combinators, BufferStrategy, ComposableStream, GuaranteedDeliveryStream, PerformanceStreamExt,
    ResilienceStreamExt, TypeErasedEvent,
};
pub use processor::{
    BatchProcessor, EventPattern, FlowController, PatternMatcher, StatefulProcessor, Window,
    WindowManager, WindowType,
};
