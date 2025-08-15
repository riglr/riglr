pub mod stream;
pub mod streamed_event;
pub mod event_adapter;
pub mod mock_stream;
pub mod error;
pub mod manager;
pub mod metrics;
pub mod builder;
pub mod operators;
pub mod financial_operators;
pub mod enhanced_operators;
pub mod config;
pub mod connection;
pub mod processor;
#[macro_use]
pub mod resilience_macro;
#[cfg(test)]
mod test_asnumeric;

pub use stream::{Stream, StreamEvent, StreamHealth, DynamicStream, DynamicStreamWrapper};
pub use streamed_event::{StreamedEvent, DynamicStreamedEvent, IntoStreamedEvent, IntoDynamicStreamedEvent};
pub use mock_stream::{MockStream, MockConfig};
// pub use riglr_solana_events::StreamMetadata; // Temporarily disabled

/// Temporary StreamMetadata for migration phase
#[derive(Debug, Clone, serde::Serialize)]
pub struct StreamMetadata {
    pub stream_source: String,
    #[serde(with = "systemtime_as_millis")]
    pub received_at: std::time::SystemTime,
    pub sequence_number: Option<u64>,
    pub custom_data: Option<serde_json::Value>,
}

mod systemtime_as_millis {
    use serde::{Serialize, Serializer};
    use std::time::SystemTime;
    
    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let millis = time.duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        millis.serialize(serializer)
    }
}
pub use error::{StreamError, StreamResult};
pub use manager::{StreamManager, EventHandler, LoggingEventHandler, HandlerExecutionMode};
pub use metrics::{MetricsCollector, StreamMetrics, HandlerMetrics, GlobalMetrics, MetricsTimer};
pub use builder::{StreamManagerBuilder, StreamConfig, from_config_file, from_env};
pub use operators::{
    ComposableStream, BufferStrategy, combinators, 
    PerformanceStreamExt, ResilienceStreamExt, TypeErasedEvent,
    GuaranteedDeliveryStream
};
pub use financial_operators::{FinancialStreamExt, AsNumeric};
pub use config::{StreamClientConfig, BackpressureStrategy, BatchConfig, BackpressureConfig, ConnectionConfig, MetricsConfig};
pub use connection::{ConnectionManager, ConnectionPool, ConnectionHealth, ConnectionState, CircuitBreaker};
pub use processor::{WindowManager, WindowType, Window, StatefulProcessor, FlowController, BatchProcessor, PatternMatcher, EventPattern};