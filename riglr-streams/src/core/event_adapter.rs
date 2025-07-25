//! Event adapter module - Legacy adapter functionality removed
//!
//! All events now implement the Event trait from riglr-events-core directly.
//! The UnifiedEvent trait has been deprecated and removed.

/// Placeholder trait for backward compatibility
pub trait EventConversion {
    /// Legacy conversion functionality removed
    fn to_event(self: Box<Self>) -> Box<dyn riglr_events_core::Event>;
}

// All events now implement the Event trait directly