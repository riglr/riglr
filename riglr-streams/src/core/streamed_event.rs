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
}
