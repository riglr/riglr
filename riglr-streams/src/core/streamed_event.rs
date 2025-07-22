//! Composition-based streaming event wrapper
//!
//! This module provides a `StreamedEvent<T>` wrapper that composes streaming metadata
//! with any Event from riglr-events-core. This follows the architectural
//! principle of separation of concerns - parsing logic is in protocol-specific crates,
//! while streaming metadata is added here through composition.

use std::any::Any;
use std::time::SystemTime;
use std::fmt;
use riglr_events_core::traits::Event;
use riglr_events_core::types::{EventKind, EventMetadata};
use riglr_solana_events::{UnifiedEvent, EventType, ProtocolType, TransferData, SwapData};
use crate::core::{StreamEvent, StreamMetadata};

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
impl<T: Event + Clone + 'static> Event for StreamedEvent<T> {
    fn id(&self) -> &str {
        self.inner.id()
    }

    fn kind(&self) -> &EventKind {
        self.inner.kind()
    }

    fn metadata(&self) -> &EventMetadata {
        self.inner.metadata()
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
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

    fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
        self.inner.to_json()
    }
}

// Legacy UnifiedEvent implementation for backward compatibility with Solana events
// This allows StreamedEvent to wrap legacy Solana events that still use UnifiedEvent
impl<T> UnifiedEvent for StreamedEvent<T> 
where 
    T: Event + Clone + UnifiedEvent + 'static
{
    fn id(&self) -> &str {
        <T as UnifiedEvent>::id(&self.inner)
    }

    fn event_type(&self) -> EventType {
        self.inner.event_type()
    }

    fn signature(&self) -> &str {
        self.inner.signature()
    }

    fn slot(&self) -> u64 {
        self.inner.slot()
    }

    fn program_received_time_ms(&self) -> i64 {
        self.inner.program_received_time_ms()
    }

    fn program_handle_time_consuming_ms(&self) -> i64 {
        self.inner.program_handle_time_consuming_ms()
    }

    fn set_program_handle_time_consuming_ms(&mut self, time: i64) {
        self.inner.set_program_handle_time_consuming_ms(time)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn UnifiedEvent> {
        Box::new(self.clone())
    }

    fn set_transfer_data(
        &mut self,
        transfer_data: Vec<TransferData>,
        swap_data: Option<SwapData>,
    ) {
        self.inner.set_transfer_data(transfer_data, swap_data)
    }

    fn index(&self) -> String {
        self.inner.index()
    }

    fn protocol_type(&self) -> ProtocolType {
        self.inner.protocol_type()
    }

    fn timestamp(&self) -> SystemTime {
        <T as UnifiedEvent>::timestamp(&self.inner)
    }

    fn transaction_hash(&self) -> Option<String> {
        self.inner.transaction_hash()
    }

    fn block_number(&self) -> Option<u64> {
        self.inner.block_number()
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
    pub fn from_event(
        event: Box<dyn Event>,
        stream_metadata: StreamMetadata,
    ) -> Self {
        Self {
            inner: event,
            stream_metadata,
        }
    }

    /// Create from any UnifiedEvent (backward compatibility)
    pub fn from_unified(
        event: Box<dyn UnifiedEvent>,
        stream_metadata: StreamMetadata,
    ) -> Self {
        use crate::core::event_adapter::{UnifiedEventAdapter, EventConversion};
        let adapted = event.to_event();
        Self {
            inner: adapted,
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

    fn kind(&self) -> &EventKind {
        self.inner.kind()
    }

    fn metadata(&self) -> &EventMetadata {
        self.inner.metadata()
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
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

    fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
        self.inner.to_json()
    }
}

// Implement UnifiedEvent by using defaults for non-Solana events
// Note: This is a bridge implementation for backward compatibility
impl UnifiedEvent for DynamicStreamedEvent {
    fn id(&self) -> &str {
        self.inner.id()
    }

    fn event_type(&self) -> EventType {
        match self.inner.kind() {
            EventKind::Swap => EventType::Swap,
            EventKind::Transfer => EventType::Transfer,
            EventKind::Liquidity => EventType::AddLiquidity,
            EventKind::Price => EventType::PriceUpdate,
            _ => EventType::Unknown,
        }
    }

    fn signature(&self) -> &str {
        // For non-Solana events, use the event ID as signature
        self.inner.id()
    }

    fn slot(&self) -> u64 {
        // Default value for non-Solana events
        0
    }

    fn program_received_time_ms(&self) -> i64 {
        self.inner.timestamp()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64
    }

    fn program_handle_time_consuming_ms(&self) -> i64 {
        0 // Default value for non-Solana events
    }

    fn set_program_handle_time_consuming_ms(&mut self, _time: i64) {
        // No-op for non-Solana events
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn UnifiedEvent> {
        // Can't clone because Box<dyn Event> doesn't have clone
        panic!("Cannot clone DynamicStreamedEvent as UnifiedEvent")
    }

    fn set_transfer_data(
        &mut self,
        _transfer_data: Vec<TransferData>,
        _swap_data: Option<SwapData>,
    ) {
        // No-op for non-Solana events
    }

    fn index(&self) -> String {
        "0".to_string() // Default value
    }

    fn protocol_type(&self) -> ProtocolType {
        // Default to Other for non-Solana events
        ProtocolType::Other(self.inner.source().to_string())
    }

    fn timestamp(&self) -> SystemTime {
        self.inner.timestamp()
    }

    fn transaction_hash(&self) -> Option<String> {
        None // Default for non-Solana events
    }

    fn block_number(&self) -> Option<u64> {
        None // Default for non-Solana events
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

/// Helper trait for creating DynamicStreamedEvent from Box<dyn UnifiedEvent>
pub trait IntoDynamicStreamedEvent {
    /// Wrap this boxed event with streaming metadata
    fn with_stream_metadata(self, metadata: StreamMetadata) -> DynamicStreamedEvent;
    
    /// Wrap this boxed event with default streaming metadata
    fn with_default_stream_metadata(self, source: impl Into<String>) -> DynamicStreamedEvent;
}

impl IntoDynamicStreamedEvent for Box<dyn UnifiedEvent> {
    fn with_stream_metadata(self, metadata: StreamMetadata) -> DynamicStreamedEvent {
        use crate::core::event_adapter::{UnifiedEventAdapter, EventConversion};
        let adapted = self.to_event();
        DynamicStreamedEvent::from_event(adapted, metadata)
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
    use riglr_solana_events::{ProtocolType, EventType};
    
    // Mock event for testing
    #[derive(Debug, Clone)]
    struct MockEvent {
        id: String,
    }
    
    impl UnifiedEvent for MockEvent {
        fn id(&self) -> &str { &self.id }
        fn event_type(&self) -> EventType { EventType::Swap }
        fn signature(&self) -> &str { "mock_sig" }
        fn slot(&self) -> u64 { 0 }
        fn program_received_time_ms(&self) -> i64 { 0 }
        fn program_handle_time_consuming_ms(&self) -> i64 { 0 }
        fn set_program_handle_time_consuming_ms(&mut self, _: i64) {}
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
        fn clone_boxed(&self) -> Box<dyn UnifiedEvent> { Box::new(self.clone()) }
        fn set_transfer_data(&mut self, _: Vec<TransferData>, _: Option<SwapData>) {}
        fn index(&self) -> String { "0".to_string() }
        fn protocol_type(&self) -> ProtocolType { ProtocolType::Jupiter }
    }
    
    #[test]
    fn test_streamed_event_composition() {
        let event = MockEvent { id: "test".to_string() };
        let metadata = StreamMetadata {
            stream_source: "test-stream".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(1),
            custom_data: None,
        };
        
        let streamed = event.with_stream_metadata(metadata);
        
        // Test UnifiedEvent delegation
        assert_eq!(streamed.id(), "test");
        assert_eq!(streamed.event_type(), EventType::Swap);
        
        // Test StreamEvent functionality
        assert!(streamed.stream_metadata().is_some());
        assert_eq!(streamed.stream_source(), Some("test-stream"));
        assert_eq!(streamed.sequence_number(), Some(1));
    }
}