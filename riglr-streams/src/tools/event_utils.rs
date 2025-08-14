use std::any::Any;
use riglr_solana_events::UnifiedEvent;

/// Helper to convert Any to UnifiedEvent by trying all known event types
pub fn as_unified_event(event: &(dyn Any + Send + Sync)) -> Option<&dyn UnifiedEvent> {
    // Try to downcast to various event types that implement UnifiedEvent
    if let Some(solana_event) = event.downcast_ref::<crate::solana::SolanaStreamEvent>() {
        return Some(solana_event);
    }
    if let Some(evm_event) = event.downcast_ref::<crate::evm::EvmStreamEvent>() {
        return Some(evm_event);
    }
    if let Some(binance_event) = event.downcast_ref::<crate::external::BinanceStreamEvent>() {
        return Some(binance_event);
    }
    if let Some(mempool_event) = event.downcast_ref::<crate::external::MempoolStreamEvent>() {
        return Some(mempool_event);
    }
    None
}

/// Macro to simplify adding new event types
/// Usage: register_event_types!(NewEventType1, NewEventType2);
#[macro_export]
macro_rules! register_event_types {
    ($($event_type:ty),*) => {
        pub fn as_unified_event_extended(event: &(dyn Any + Send + Sync)) -> Option<&dyn UnifiedEvent> {
            // First try the built-in types
            if let Some(unified) = as_unified_event(event) {
                return Some(unified);
            }
            
            // Then try the extended types
            $(
                if let Some(typed_event) = event.downcast_ref::<$event_type>() {
                    return Some(typed_event);
                }
            )*
            
            None
        }
    };
}