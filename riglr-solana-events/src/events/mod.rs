//! Event parsing modules for Solana blockchain transactions.
//!
//! This module provides comprehensive event parsing capabilities for various Solana protocols
//! including DEXs, lending protocols, and other DeFi applications. It contains parsers for
//! extracting structured events from transaction data.

/// Common utilities and types shared across protocol parsers
pub mod common;

/// Core event parsing traits and utilities
pub mod core;

/// Event parser factory and registry for managing protocol-specific parsers
pub mod factory;

/// Protocol-specific event parsers for various Solana DeFi protocols
pub mod protocols;

pub use crate::types::{EventMetadata, EventType, ProtocolType, SwapData, TransferData};
pub use core::{EventParser, GenericEventParser};
pub use factory::{EventParserRegistry, Protocol};

/// Pattern matching macro for event types with type downcasting.
///
/// This macro provides a convenient way to match against different event types
/// by attempting to downcast the event to each specified type. It uses the
/// `as_any()` method to access the underlying `Any` trait for downcasting.
///
/// # Example
///
/// ```rust
/// use riglr_solana_events::match_event;
///
/// match_event!(event, {
///     SwapEvent => |swap: SwapEvent| {
///         println!("Swap amount: {}", swap.amount);
///     },
///     TransferEvent => |transfer: TransferEvent| {
///         println!("Transfer to: {}", transfer.destination);
///     },
/// });
/// ```
#[macro_export]
macro_rules! match_event {
    ($event:expr, { $($event_type:ty => |$var:ident: $event_type_full:ty| $body:block),* $(,)? }) => {
        $(
            if let Some($var) = $event.as_any().downcast_ref::<$event_type>() {
                $body
                return;
            }
        )*
    };
}

// Re-export common utilities
pub use crate::utils::*;
