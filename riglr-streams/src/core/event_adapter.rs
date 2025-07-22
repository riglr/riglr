//! Adapters for bridging between UnifiedEvent (legacy) and Event (new) traits
//!
//! This module provides compatibility layers to allow seamless migration from
//! the Solana-specific UnifiedEvent to the chain-agnostic Event trait.

use std::any::Any;
use std::time::SystemTime;
use riglr_events_core::traits::Event;
use riglr_events_core::types::{EventKind, EventMetadata as CoreMetadata};
use riglr_events_core::error::EventResult;
use riglr_solana_events::{UnifiedEvent, EventType, ProtocolType};
use chrono::Utc;

/// Adapter that makes any UnifiedEvent implement the Event trait
#[derive(Clone, Debug)]
pub struct UnifiedEventAdapter {
    inner: Box<dyn UnifiedEvent>,
    core_metadata: CoreMetadata,
}

impl UnifiedEventAdapter {
    /// Create a new adapter from a UnifiedEvent
    pub fn new(event: Box<dyn UnifiedEvent>) -> Self {
        // Convert legacy metadata to core metadata
        let core_metadata = CoreMetadata::new(
            event.id().to_string(),
            Self::convert_event_type(&event.event_type()),
            format!("solana-{}", event.protocol_type()),
        );
        
        Self {
            inner: event,
            core_metadata,
        }
    }
    
    /// Convert EventType to EventKind
    fn convert_event_type(event_type: &EventType) -> EventKind {
        match event_type {
            EventType::Swap => EventKind::Swap,
            EventType::Transfer => EventKind::Transfer,
            EventType::AddLiquidity | EventType::RemoveLiquidity => EventKind::Liquidity,
            EventType::PriceUpdate => EventKind::Price,
            _ => EventKind::Custom(format!("{:?}", event_type)),
        }
    }
}

impl Event for UnifiedEventAdapter {
    fn id(&self) -> &str {
        self.inner.id()
    }
    
    fn kind(&self) -> &EventKind {
        &self.core_metadata.kind
    }
    
    fn metadata(&self) -> &CoreMetadata {
        &self.core_metadata
    }
    
    fn metadata_mut(&mut self) -> &mut CoreMetadata {
        &mut self.core_metadata
    }
    
    fn timestamp(&self) -> SystemTime {
        self.inner.timestamp()
    }
    
    fn source(&self) -> &str {
        &self.core_metadata.source
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
    
    fn to_json(&self) -> EventResult<serde_json::Value> {
        // Create a JSON representation combining both metadata styles
        Ok(serde_json::json!({
            "id": self.inner.id(),
            "kind": format!("{:?}", self.kind()),
            "event_type": format!("{:?}", self.inner.event_type()),
            "signature": self.inner.signature(),
            "slot": self.inner.slot(),
            "protocol_type": format!("{:?}", self.inner.protocol_type()),
            "timestamp": self.inner.timestamp().duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
        }))
    }
}

/// Adapter that makes any Event implement the UnifiedEvent trait
/// This is used for new events that need to work with legacy code
#[derive(Debug)]
pub struct EventToUnifiedAdapter<E: Event> {
    inner: E,
}

impl<E: Event + Clone + 'static> EventToUnifiedAdapter<E> {
    /// Create a new adapter from an Event
    pub fn new(event: E) -> Self {
        Self { inner: event }
    }
}

impl<E: Event + Clone + 'static> UnifiedEvent for EventToUnifiedAdapter<E> {
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
        // For non-Solana events, use a default value
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
        Box::new(self.clone())
    }
    
    fn set_transfer_data(
        &mut self,
        _transfer_data: Vec<riglr_solana_events::TransferData>,
        _swap_data: Option<riglr_solana_events::SwapData>,
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
}

impl<E: Event + Clone> Clone for EventToUnifiedAdapter<E> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Extension trait for easy conversion between event types
pub trait EventConversion {
    /// Convert to a boxed Event trait object
    fn to_event(self: Box<Self>) -> Box<dyn Event>;
}

impl EventConversion for dyn UnifiedEvent {
    fn to_event(self: Box<Self>) -> Box<dyn Event> {
        Box::new(UnifiedEventAdapter::new(self))
    }
}

/// Extension trait for converting Event to UnifiedEvent
pub trait ToUnifiedEvent {
    /// Convert to a boxed UnifiedEvent trait object
    fn to_unified(self) -> Box<dyn UnifiedEvent>;
}

impl<E: Event + Clone + 'static> ToUnifiedEvent for E {
    fn to_unified(self) -> Box<dyn UnifiedEvent> {
        Box::new(EventToUnifiedAdapter::new(self))
    }
}