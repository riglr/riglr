pub mod builder;
pub mod config;
pub mod connection;
pub mod enhanced_operators;
pub mod error;
pub mod event_adapter;
pub mod financial_operators;
pub mod manager;
pub mod metrics;
pub mod mock_stream;
pub mod operators;
pub mod processor;
pub mod stream;
pub mod streamed_event;
#[macro_use]
pub mod resilience_macro;
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
