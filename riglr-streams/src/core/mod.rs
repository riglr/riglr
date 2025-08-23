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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[test]
    fn test_stream_metadata_creation_with_all_fields_should_succeed() {
        // Happy Path: Create StreamMetadata with all fields
        let custom_data = serde_json::json!({"key": "value", "number": 42});
        let metadata = StreamMetadata {
            stream_source: "test_stream".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(123),
            custom_data: Some(custom_data.clone()),
        };

        assert_eq!(metadata.stream_source, "test_stream");
        assert!(metadata.received_at <= SystemTime::now());
        assert_eq!(metadata.sequence_number, Some(123));
        assert_eq!(metadata.custom_data, Some(custom_data));
    }

    #[test]
    fn test_stream_metadata_creation_with_minimal_fields_should_succeed() {
        // Edge Case: Create StreamMetadata with only required fields
        let metadata = StreamMetadata {
            stream_source: String::default(),
            received_at: UNIX_EPOCH,
            sequence_number: None,
            custom_data: None,
        };

        assert_eq!(metadata.stream_source, "");
        assert_eq!(metadata.received_at, UNIX_EPOCH);
        assert_eq!(metadata.sequence_number, None);
        assert_eq!(metadata.custom_data, None);
    }

    #[test]
    fn test_stream_metadata_clone_should_create_identical_copy() {
        // Test Clone trait
        let original = StreamMetadata {
            stream_source: "original_stream".to_string(),
            received_at: UNIX_EPOCH + Duration::from_secs(1000),
            sequence_number: Some(456),
            custom_data: Some(serde_json::json!({"test": true})),
        };

        let cloned = original.clone();

        assert_eq!(original.stream_source, cloned.stream_source);
        assert_eq!(original.received_at, cloned.received_at);
        assert_eq!(original.sequence_number, cloned.sequence_number);
        assert_eq!(original.custom_data, cloned.custom_data);
    }

    #[test]
    fn test_stream_metadata_debug_format_should_contain_all_fields() {
        // Test Debug trait
        let metadata = StreamMetadata {
            stream_source: "debug_stream".to_string(),
            received_at: UNIX_EPOCH,
            sequence_number: Some(789),
            custom_data: Some(serde_json::json!({"debug": "test"})),
        };

        let debug_string = format!("{:?}", metadata);
        assert!(debug_string.contains("debug_stream"));
        assert!(debug_string.contains("789"));
    }

    #[test]
    fn test_stream_metadata_serialization_with_unix_epoch_should_serialize_to_zero() {
        // Test serialization with UNIX_EPOCH (edge case)
        let metadata = StreamMetadata {
            stream_source: "epoch_stream".to_string(),
            received_at: UNIX_EPOCH,
            sequence_number: Some(0),
            custom_data: None,
        };

        let serialized = serde_json::to_string(&metadata).expect("Serialization should succeed");
        let parsed: serde_json::Value =
            serde_json::from_str(&serialized).expect("Should parse as JSON");

        assert_eq!(parsed["stream_source"], "epoch_stream");
        assert_eq!(parsed["received_at"], 0);
        assert_eq!(parsed["sequence_number"], 0);
        assert_eq!(parsed["custom_data"], serde_json::Value::Null);
    }

    #[test]
    fn test_stream_metadata_serialization_with_future_time_should_serialize_correctly() {
        // Test serialization with a known future time
        let future_time = UNIX_EPOCH + Duration::from_millis(1609459200000); // 2021-01-01 00:00:00 UTC
        let metadata = StreamMetadata {
            stream_source: "future_stream".to_string(),
            received_at: future_time,
            sequence_number: Some(u64::MAX),
            custom_data: Some(serde_json::json!({"array": [1, 2, 3], "nested": {"key": "value"}})),
        };

        let serialized = serde_json::to_string(&metadata).expect("Serialization should succeed");
        let parsed: serde_json::Value =
            serde_json::from_str(&serialized).expect("Should parse as JSON");

        assert_eq!(parsed["stream_source"], "future_stream");
        assert_eq!(parsed["received_at"], 1609459200000u64);
        assert_eq!(parsed["sequence_number"], u64::MAX);
        assert_eq!(parsed["custom_data"]["array"], serde_json::json!([1, 2, 3]));
        assert_eq!(parsed["custom_data"]["nested"]["key"], "value");
    }

    #[test]
    fn test_stream_metadata_serialization_with_very_large_sequence_number_should_succeed() {
        // Edge Case: Test with maximum sequence number
        let metadata = StreamMetadata {
            stream_source: "max_sequence_stream".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(u64::MAX),
            custom_data: None,
        };

        let serialized = serde_json::to_string(&metadata).expect("Serialization should succeed");
        assert!(serialized.contains(&u64::MAX.to_string()));
    }

    #[test]
    fn test_stream_metadata_serialization_with_complex_custom_data_should_preserve_structure() {
        // Test with complex nested JSON
        let complex_data = serde_json::json!({
            "numbers": [1, 2, 3, 4, 5],
            "strings": ["hello", "world", ""],
            "nested": {
                "bool": true,
                "null": null,
                "float": std::f64::consts::PI,
                "empty_object": {},
                "empty_array": []
            },
            "unicode": "こんにちは世界",
            "special_chars": "!@#$%^&*()_+-=[]{}|;':\",./<>?"
        });

        let metadata = StreamMetadata {
            stream_source: "complex_stream".to_string(),
            received_at: UNIX_EPOCH + Duration::from_secs(1000),
            sequence_number: None,
            custom_data: Some(complex_data.clone()),
        };

        let serialized = serde_json::to_string(&metadata).expect("Serialization should succeed");
        let parsed: serde_json::Value =
            serde_json::from_str(&serialized).expect("Should parse as JSON");

        assert_eq!(parsed["custom_data"], complex_data);
    }

    #[test]
    fn test_systemtime_as_millis_serialize_with_unix_epoch_should_return_zero() {
        // Test the custom serialization module directly
        let time = UNIX_EPOCH;
        let mut serializer = serde_json::Serializer::new(Vec::with_capacity(64));

        systemtime_as_millis::serialize(&time, &mut serializer)
            .expect("Serialization should succeed");
        let output = String::from_utf8(serializer.into_inner()).expect("Should be valid UTF-8");

        assert_eq!(output, "0");
    }

    #[test]
    fn test_systemtime_as_millis_serialize_with_known_time_should_return_correct_millis() {
        // Test with a known time: 1000 seconds after UNIX_EPOCH
        let time = UNIX_EPOCH + Duration::from_secs(1000);
        let mut serializer = serde_json::Serializer::new(Vec::with_capacity(64));

        systemtime_as_millis::serialize(&time, &mut serializer)
            .expect("Serialization should succeed");
        let output = String::from_utf8(serializer.into_inner()).expect("Should be valid UTF-8");

        assert_eq!(output, "1000000"); // 1000 seconds = 1,000,000 milliseconds
    }

    #[test]
    fn test_systemtime_as_millis_serialize_with_fractional_seconds_should_include_millis() {
        // Test with fractional seconds
        let time = UNIX_EPOCH + Duration::from_millis(1500); // 1.5 seconds
        let mut serializer = serde_json::Serializer::new(Vec::with_capacity(64));

        systemtime_as_millis::serialize(&time, &mut serializer)
            .expect("Serialization should succeed");
        let output = String::from_utf8(serializer.into_inner()).expect("Should be valid UTF-8");

        assert_eq!(output, "1500");
    }

    #[test]
    fn test_systemtime_as_millis_serialize_with_time_before_epoch_should_use_default() {
        // Edge Case: Test with time before UNIX_EPOCH (should use default duration)
        // This tests the .unwrap_or_default() branch in the serialization function
        let time = UNIX_EPOCH - Duration::from_secs(1000);
        let mut serializer = serde_json::Serializer::new(Vec::with_capacity(64));

        systemtime_as_millis::serialize(&time, &mut serializer)
            .expect("Serialization should succeed");
        let output = String::from_utf8(serializer.into_inner()).expect("Should be valid UTF-8");

        assert_eq!(output, "0"); // Should default to 0 when duration_since fails
    }

    #[test]
    fn test_stream_metadata_with_empty_string_source_should_serialize_correctly() {
        // Edge Case: Empty string source
        let metadata = StreamMetadata {
            stream_source: String::default(),
            received_at: UNIX_EPOCH + Duration::from_millis(123456),
            sequence_number: Some(0),
            custom_data: Some(serde_json::Value::Null),
        };

        let serialized = serde_json::to_string(&metadata).expect("Serialization should succeed");
        let parsed: serde_json::Value =
            serde_json::from_str(&serialized).expect("Should parse as JSON");

        assert_eq!(parsed["stream_source"], "");
        assert_eq!(parsed["received_at"], 123456);
    }

    #[test]
    fn test_stream_metadata_with_unicode_source_should_preserve_encoding() {
        // Edge Case: Unicode characters in source
        let metadata = StreamMetadata {
            stream_source: "🚀💎🔥 Unicode Stream ñoël".to_string(),
            received_at: UNIX_EPOCH,
            sequence_number: None,
            custom_data: None,
        };

        let serialized = serde_json::to_string(&metadata).expect("Serialization should succeed");
        let parsed: serde_json::Value =
            serde_json::from_str(&serialized).expect("Should parse as JSON");

        assert_eq!(parsed["stream_source"], "🚀💎🔥 Unicode Stream ñoël");
    }

    #[test]
    fn test_stream_metadata_with_very_long_source_name_should_handle_correctly() {
        // Edge Case: Very long source name
        let long_source = "a".repeat(10000);
        let metadata = StreamMetadata {
            stream_source: long_source.clone(),
            received_at: UNIX_EPOCH,
            sequence_number: Some(1),
            custom_data: None,
        };

        let serialized = serde_json::to_string(&metadata).expect("Serialization should succeed");
        let parsed: serde_json::Value =
            serde_json::from_str(&serialized).expect("Should parse as JSON");

        assert_eq!(parsed["stream_source"], long_source);
    }
}
