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
pub use riglr_solana_events::StreamMetadata;
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