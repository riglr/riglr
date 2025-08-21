//! Composition-based streaming event wrapper
//!
//! This module provides a `StreamedEvent<T>` wrapper that composes streaming metadata
//! with any Event from riglr-events-core. This follows the architectural
//! principle of separation of concerns - parsing logic is in protocol-specific crates,
//! while streaming metadata is added here through composition.

use crate::core::{StreamEvent, StreamMetadata};
use riglr_events_core::prelude::Event;
use std::any::Any;
use std::fmt;
use std::time::SystemTime;

/// Wrapper that adds streaming metadata to any Event through composition
#[derive(Debug, Clone)]
pub struct StreamedEvent<T: Event + Clone> {
    /// The original event from any source
    pub inner: T,
    /// Streaming-specific metadata
    pub stream_metadata: StreamMetadata,
}

impl<T: Event + Clone> StreamedEvent<T> {
    /// Create a new streamed event by wrapping an existing event
    pub fn new(inner: T, stream_metadata: StreamMetadata) -> Self {
        Self {
            inner,
            stream_metadata,
        }
    }

    /// Get a reference to the inner event
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Get a mutable reference to the inner event
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Unwrap to get the inner event
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Get the stream metadata
    pub fn stream_meta(&self) -> &StreamMetadata {
        &self.stream_metadata
    }
}

// Implement Event by delegating to the inner event
impl<T> Event for StreamedEvent<T>
where
    T: Event + Clone + 'static,
{
    fn id(&self) -> &str {
        self.inner.id()
    }

    fn kind(&self) -> &riglr_events_core::EventKind {
        self.inner.kind()
    }

    fn metadata(&self) -> &riglr_events_core::EventMetadata {
        self.inner.metadata()
    }

    fn metadata_mut(&mut self) -> &mut riglr_events_core::EventMetadata {
        self.inner.metadata_mut()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn to_json(&self) -> riglr_events_core::EventResult<serde_json::Value> {
        self.inner.to_json()
    }
}

// Implement StreamEvent to provide streaming-specific functionality
impl<T: Event + Clone + 'static> StreamEvent for StreamedEvent<T> {
    fn stream_metadata(&self) -> Option<&StreamMetadata> {
        Some(&self.stream_metadata)
    }
}

/// Dynamic event wrapper for type-erased Events with streaming metadata
/// This is a separate implementation that doesn't rely on the generic StreamedEvent<T>
#[derive(Clone)]
pub struct DynamicStreamedEvent {
    /// The type-erased event
    pub inner: Box<dyn Event>,
    /// Streaming-specific metadata
    pub stream_metadata: StreamMetadata,
}

impl fmt::Debug for DynamicStreamedEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynamicStreamedEvent")
            .field("inner_id", &self.inner.id())
            .field("inner_kind", &self.inner.kind())
            .field("stream_metadata", &self.stream_metadata)
            .finish()
    }
}

impl DynamicStreamedEvent {
    /// Create from any Event
    pub fn from_event(event: Box<dyn Event>, stream_metadata: StreamMetadata) -> Self {
        Self {
            inner: event,
            stream_metadata,
        }
    }

    /// Get a reference to the inner event
    pub fn inner(&self) -> &dyn Event {
        self.inner.as_ref()
    }

    /// Get the stream metadata
    pub fn stream_meta(&self) -> &StreamMetadata {
        &self.stream_metadata
    }
}

// Implement Event by delegating to the inner event
impl Event for DynamicStreamedEvent {
    fn id(&self) -> &str {
        self.inner.id()
    }

    fn kind(&self) -> &riglr_events_core::EventKind {
        self.inner.kind()
    }

    fn metadata(&self) -> &riglr_events_core::EventMetadata {
        self.inner.metadata()
    }

    fn metadata_mut(&mut self) -> &mut riglr_events_core::EventMetadata {
        self.inner.metadata_mut()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn to_json(&self) -> riglr_events_core::EventResult<serde_json::Value> {
        self.inner.to_json()
    }
}

// Implement StreamEvent to provide streaming-specific functionality
impl StreamEvent for DynamicStreamedEvent {
    fn stream_metadata(&self) -> Option<&StreamMetadata> {
        Some(&self.stream_metadata)
    }
}

/// Helper trait to easily wrap events with streaming metadata
pub trait IntoStreamedEvent: Event + Clone + Sized {
    /// Wrap this event with streaming metadata
    fn with_stream_metadata(self, metadata: StreamMetadata) -> StreamedEvent<Self> {
        StreamedEvent::new(self, metadata)
    }

    /// Wrap this event with default streaming metadata
    fn with_default_stream_metadata(self, source: impl Into<String>) -> StreamedEvent<Self> {
        let metadata = StreamMetadata {
            stream_source: source.into(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };
        self.with_stream_metadata(metadata)
    }
}

// Implement for all Event types
impl<T: Event + Clone + 'static> IntoStreamedEvent for T {}

/// Helper trait for creating DynamicStreamedEvent from Box<dyn Event>
pub trait IntoDynamicStreamedEvent {
    /// Wrap this boxed event with streaming metadata
    fn with_stream_metadata(self, metadata: StreamMetadata) -> DynamicStreamedEvent;

    /// Wrap this boxed event with default streaming metadata
    fn with_default_stream_metadata(self, source: impl Into<String>) -> DynamicStreamedEvent;
}

impl IntoDynamicStreamedEvent for Box<dyn Event> {
    fn with_stream_metadata(self, metadata: StreamMetadata) -> DynamicStreamedEvent {
        DynamicStreamedEvent::from_event(self, metadata)
    }

    fn with_default_stream_metadata(self, source: impl Into<String>) -> DynamicStreamedEvent {
        let metadata = StreamMetadata {
            stream_source: source.into(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };
        self.with_stream_metadata(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_events_core::prelude::{EventKind, EventMetadata};

    // Mock event for testing
    #[derive(Debug, Clone)]
    struct MockEvent {
        metadata: EventMetadata,
    }

    impl MockEvent {
        fn new() -> Self {
            Self {
                metadata: EventMetadata::new(
                    "test".to_string(),
                    EventKind::Swap,
                    "mock".to_string(),
                ),
            }
        }
    }

    impl Event for MockEvent {
        fn id(&self) -> &str {
            &self.metadata.id
        }
        fn kind(&self) -> &EventKind {
            &self.metadata.kind
        }
        fn metadata(&self) -> &EventMetadata {
            &self.metadata
        }
        fn metadata_mut(&mut self) -> &mut EventMetadata {
            &mut self.metadata
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
        fn clone_boxed(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }

        fn to_json(&self) -> riglr_events_core::EventResult<serde_json::Value> {
            Ok(serde_json::json!({
                "id": self.id(),
                "kind": format!("{:?}", self.kind()),
                "source": &self.metadata.source
            }))
        }
    }

    #[test]
    fn test_streamed_event_composition() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "test-stream".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(1),
            custom_data: None,
        };

        let streamed = event.with_stream_metadata(metadata);

        // Test Event delegation
        assert_eq!(streamed.id(), "test");
        assert_eq!(streamed.kind(), &EventKind::Swap);

        // Test StreamEvent functionality
        assert!(streamed.stream_metadata().is_some());
        assert_eq!(streamed.stream_source(), Some("test-stream"));
        assert_eq!(streamed.sequence_number(), Some(1));
    }

    #[test]
    fn test_streamed_event_new() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "test-source".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(42),
            custom_data: Some(serde_json::json!({"key": "value"})),
        };

        let streamed = StreamedEvent::new(event.clone(), metadata.clone());

        assert_eq!(streamed.inner.id(), event.id());
        assert_eq!(
            streamed.stream_metadata.stream_source,
            metadata.stream_source
        );
        assert_eq!(
            streamed.stream_metadata.sequence_number,
            metadata.sequence_number
        );
    }

    #[test]
    fn test_streamed_event_inner() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let streamed = StreamedEvent::new(event.clone(), metadata);
        let inner_ref = streamed.inner();

        assert_eq!(inner_ref.id(), event.id());
        assert_eq!(inner_ref.kind(), event.kind());
    }

    #[test]
    fn test_streamed_event_inner_mut() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let mut streamed = StreamedEvent::new(event, metadata);
        let inner_mut = streamed.inner_mut();

        // Modify the inner event
        inner_mut.metadata.id = "modified".to_string();

        assert_eq!(streamed.inner().id(), "modified");
    }

    #[test]
    fn test_streamed_event_into_inner() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let streamed = StreamedEvent::new(event.clone(), metadata);
        let inner = streamed.into_inner();

        assert_eq!(inner.id(), event.id());
        assert_eq!(inner.kind(), event.kind());
    }

    #[test]
    fn test_streamed_event_stream_meta() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "test-stream".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(100),
            custom_data: Some(serde_json::json!({"test": true})),
        };

        let streamed = StreamedEvent::new(event, metadata.clone());
        let stream_meta = streamed.stream_meta();

        assert_eq!(stream_meta.stream_source, metadata.stream_source);
        assert_eq!(stream_meta.sequence_number, metadata.sequence_number);
        assert_eq!(stream_meta.custom_data, metadata.custom_data);
    }

    #[test]
    fn test_streamed_event_implements_event_trait() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let streamed = StreamedEvent::new(event.clone(), metadata);

        // Test all Event trait methods
        assert_eq!(streamed.id(), event.id());
        assert_eq!(streamed.kind(), event.kind());
        assert_eq!(streamed.metadata().id, event.metadata().id);
    }

    #[test]
    fn test_streamed_event_metadata_mut() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let mut streamed = StreamedEvent::new(event, metadata);

        // Modify metadata through Event trait
        streamed.metadata_mut().id = "new_id".to_string();

        assert_eq!(streamed.id(), "new_id");
    }

    #[test]
    fn test_streamed_event_as_any() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let streamed = StreamedEvent::new(event, metadata);
        let any_ref = streamed.as_any();

        // Should be able to downcast back to StreamedEvent
        assert!(any_ref.downcast_ref::<StreamedEvent<MockEvent>>().is_some());
    }

    #[test]
    fn test_streamed_event_as_any_mut() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let mut streamed = StreamedEvent::new(event, metadata);
        let any_mut = streamed.as_any_mut();

        // Should be able to downcast back to mutable StreamedEvent
        assert!(any_mut.downcast_mut::<StreamedEvent<MockEvent>>().is_some());
    }

    #[test]
    fn test_streamed_event_clone_boxed() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let streamed = StreamedEvent::new(event, metadata);
        let boxed = streamed.clone_boxed();

        assert_eq!(boxed.id(), streamed.id());
        assert_eq!(boxed.kind(), streamed.kind());
    }

    #[test]
    fn test_streamed_event_to_json() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let streamed = StreamedEvent::new(event, metadata);

        // Should delegate to inner event's to_json
        let result = streamed.to_json();
        assert!(result.is_ok());
    }

    #[test]
    fn test_streamed_event_stream_event_trait() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "test-stream".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(42),
            custom_data: None,
        };

        let streamed = StreamedEvent::new(event, metadata);

        // Test StreamEvent trait methods
        assert!(streamed.stream_metadata().is_some());
        assert_eq!(streamed.stream_source(), Some("test-stream"));
        assert_eq!(streamed.sequence_number(), Some(42));
        assert!(streamed.received_at().is_some());
    }

    #[test]
    fn test_dynamic_streamed_event_from_event() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "dynamic-test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(1),
            custom_data: None,
        };

        let boxed_event: Box<dyn Event> = Box::new(event.clone());
        let dynamic = DynamicStreamedEvent::from_event(boxed_event, metadata.clone());

        assert_eq!(dynamic.inner().id(), event.id());
        assert_eq!(dynamic.stream_meta().stream_source, metadata.stream_source);
    }

    #[test]
    fn test_dynamic_streamed_event_inner() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let boxed_event: Box<dyn Event> = Box::new(event.clone());
        let dynamic = DynamicStreamedEvent::from_event(boxed_event, metadata);

        let inner = dynamic.inner();
        assert_eq!(inner.id(), event.id());
        assert_eq!(inner.kind(), event.kind());
    }

    #[test]
    fn test_dynamic_streamed_event_stream_meta() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "test-dynamic".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(99),
            custom_data: Some(serde_json::json!({"dynamic": true})),
        };

        let boxed_event: Box<dyn Event> = Box::new(event);
        let dynamic = DynamicStreamedEvent::from_event(boxed_event, metadata.clone());

        let stream_meta = dynamic.stream_meta();
        assert_eq!(stream_meta.stream_source, metadata.stream_source);
        assert_eq!(stream_meta.sequence_number, metadata.sequence_number);
        assert_eq!(stream_meta.custom_data, metadata.custom_data);
    }

    #[test]
    fn test_dynamic_streamed_event_debug() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "debug-test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(1),
            custom_data: None,
        };

        let boxed_event: Box<dyn Event> = Box::new(event);
        let dynamic = DynamicStreamedEvent::from_event(boxed_event, metadata);

        let debug_str = format!("{:?}", dynamic);
        assert!(debug_str.contains("DynamicStreamedEvent"));
        assert!(debug_str.contains("inner_id"));
        assert!(debug_str.contains("stream_metadata"));
    }

    #[test]
    fn test_dynamic_streamed_event_implements_event_trait() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let boxed_event: Box<dyn Event> = Box::new(event.clone());
        let dynamic = DynamicStreamedEvent::from_event(boxed_event, metadata);

        // Test all Event trait methods
        assert_eq!(dynamic.id(), event.id());
        assert_eq!(dynamic.kind(), event.kind());
        assert_eq!(dynamic.metadata().id, event.metadata().id);
    }

    #[test]
    fn test_dynamic_streamed_event_metadata_mut() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let boxed_event: Box<dyn Event> = Box::new(event);
        let mut dynamic = DynamicStreamedEvent::from_event(boxed_event, metadata);

        // Modify metadata through Event trait
        dynamic.metadata_mut().id = "modified_dynamic".to_string();

        assert_eq!(dynamic.id(), "modified_dynamic");
    }

    #[test]
    fn test_dynamic_streamed_event_as_any() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let boxed_event: Box<dyn Event> = Box::new(event);
        let dynamic = DynamicStreamedEvent::from_event(boxed_event, metadata);

        let any_ref = dynamic.as_any();

        // Should be able to downcast back to DynamicStreamedEvent
        assert!(any_ref.downcast_ref::<DynamicStreamedEvent>().is_some());
    }

    #[test]
    fn test_dynamic_streamed_event_as_any_mut() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let boxed_event: Box<dyn Event> = Box::new(event);
        let mut dynamic = DynamicStreamedEvent::from_event(boxed_event, metadata);

        let any_mut = dynamic.as_any_mut();

        // Should be able to downcast back to mutable DynamicStreamedEvent
        assert!(any_mut.downcast_mut::<DynamicStreamedEvent>().is_some());
    }

    #[test]
    fn test_dynamic_streamed_event_clone_boxed() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let boxed_event: Box<dyn Event> = Box::new(event);
        let dynamic = DynamicStreamedEvent::from_event(boxed_event, metadata);

        let boxed = dynamic.clone_boxed();

        assert_eq!(boxed.id(), dynamic.id());
        assert_eq!(boxed.kind(), dynamic.kind());
    }

    #[test]
    fn test_dynamic_streamed_event_to_json() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let boxed_event: Box<dyn Event> = Box::new(event);
        let dynamic = DynamicStreamedEvent::from_event(boxed_event, metadata);

        // Should delegate to inner event's to_json
        let result = dynamic.to_json();
        assert!(result.is_ok());
    }

    #[test]
    fn test_dynamic_streamed_event_stream_event_trait() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "dynamic-stream".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(123),
            custom_data: None,
        };

        let boxed_event: Box<dyn Event> = Box::new(event);
        let dynamic = DynamicStreamedEvent::from_event(boxed_event, metadata);

        // Test StreamEvent trait methods
        assert!(dynamic.stream_metadata().is_some());
        assert_eq!(dynamic.stream_source(), Some("dynamic-stream"));
        assert_eq!(dynamic.sequence_number(), Some(123));
        assert!(dynamic.received_at().is_some());
    }

    #[test]
    fn test_into_streamed_event_trait_with_stream_metadata() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "trait-test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(456),
            custom_data: None,
        };

        let streamed = event.with_stream_metadata(metadata.clone());

        assert_eq!(streamed.stream_meta().stream_source, metadata.stream_source);
        assert_eq!(
            streamed.stream_meta().sequence_number,
            metadata.sequence_number
        );
    }

    #[test]
    fn test_into_streamed_event_trait_with_default_stream_metadata() {
        let event = MockEvent::new();

        let streamed = event.with_default_stream_metadata("default-source");

        assert_eq!(streamed.stream_meta().stream_source, "default-source");
        assert_eq!(streamed.stream_meta().sequence_number, None);
        assert_eq!(streamed.stream_meta().custom_data, None);
        assert!(streamed.received_at().is_some());
    }

    #[test]
    fn test_into_streamed_event_trait_with_default_stream_metadata_string() {
        let event = MockEvent::new();

        let streamed = event.with_default_stream_metadata("string-source".to_string());

        assert_eq!(streamed.stream_meta().stream_source, "string-source");
        assert_eq!(streamed.stream_meta().sequence_number, None);
        assert_eq!(streamed.stream_meta().custom_data, None);
        assert!(streamed.received_at().is_some());
    }

    #[test]
    fn test_into_dynamic_streamed_event_trait_with_stream_metadata() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "boxed-trait-test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(789),
            custom_data: Some(serde_json::json!({"boxed": true})),
        };

        let boxed_event: Box<dyn Event> = Box::new(event);
        let dynamic = boxed_event.with_stream_metadata(metadata.clone());

        assert_eq!(dynamic.stream_meta().stream_source, metadata.stream_source);
        assert_eq!(
            dynamic.stream_meta().sequence_number,
            metadata.sequence_number
        );
        assert_eq!(dynamic.stream_meta().custom_data, metadata.custom_data);
    }

    #[test]
    fn test_into_dynamic_streamed_event_trait_with_default_stream_metadata() {
        let event = MockEvent::new();

        let boxed_event: Box<dyn Event> = Box::new(event);
        let dynamic = boxed_event.with_default_stream_metadata("boxed-default");

        assert_eq!(dynamic.stream_meta().stream_source, "boxed-default");
        assert_eq!(dynamic.stream_meta().sequence_number, None);
        assert_eq!(dynamic.stream_meta().custom_data, None);
        assert!(dynamic.received_at().is_some());
    }

    #[test]
    fn test_into_dynamic_streamed_event_trait_with_default_stream_metadata_string() {
        let event = MockEvent::new();

        let boxed_event: Box<dyn Event> = Box::new(event);
        let dynamic = boxed_event.with_default_stream_metadata("boxed-string".to_string());

        assert_eq!(dynamic.stream_meta().stream_source, "boxed-string");
        assert_eq!(dynamic.stream_meta().sequence_number, None);
        assert_eq!(dynamic.stream_meta().custom_data, None);
        assert!(dynamic.received_at().is_some());
    }

    #[test]
    fn test_streamed_event_debug_clone() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "debug-clone-test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(999),
            custom_data: None,
        };

        let streamed = StreamedEvent::new(event, metadata);
        let cloned = streamed.clone();

        // Test Debug trait
        let debug_str = format!("{:?}", streamed);
        assert!(debug_str.contains("StreamedEvent"));

        // Test Clone trait
        assert_eq!(cloned.id(), streamed.id());
        assert_eq!(
            cloned.stream_meta().stream_source,
            streamed.stream_meta().stream_source
        );
    }

    #[test]
    fn test_dynamic_streamed_event_clone() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "clone-test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(111),
            custom_data: None,
        };

        let boxed_event: Box<dyn Event> = Box::new(event);
        let dynamic = DynamicStreamedEvent::from_event(boxed_event, metadata);
        let cloned = dynamic.clone();

        assert_eq!(cloned.id(), dynamic.id());
        assert_eq!(
            cloned.stream_meta().stream_source,
            dynamic.stream_meta().stream_source
        );
    }

    // Edge case tests
    #[test]
    fn test_streamed_event_with_none_sequence_number() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "no-seq".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let streamed = StreamedEvent::new(event, metadata);

        assert_eq!(streamed.sequence_number(), None);
    }

    #[test]
    fn test_streamed_event_with_large_sequence_number() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "large-seq".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(u64::MAX),
            custom_data: None,
        };

        let streamed = StreamedEvent::new(event, metadata);

        assert_eq!(streamed.sequence_number(), Some(u64::MAX));
    }

    #[test]
    fn test_streamed_event_with_empty_source() {
        let event = MockEvent::new();
        let metadata = StreamMetadata {
            stream_source: "".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let streamed = StreamedEvent::new(event, metadata);

        assert_eq!(streamed.stream_source(), Some(""));
    }

    #[test]
    fn test_streamed_event_with_complex_custom_data() {
        let event = MockEvent::new();
        let custom_data = serde_json::json!({
            "nested": {
                "value": 42,
                "array": [1, 2, 3],
                "bool": true
            }
        });
        let metadata = StreamMetadata {
            stream_source: "complex".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(1),
            custom_data: Some(custom_data.clone()),
        };

        let streamed = StreamedEvent::new(event, metadata);

        assert_eq!(streamed.stream_meta().custom_data, Some(custom_data));
    }
}
