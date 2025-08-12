pub mod common;
pub mod core;
pub mod factory;
pub mod protocols;

pub use core::{EventParser, UnifiedEvent, GenericEventParser};
pub use common::types::{ProtocolType, EventType, EventMetadata, TransferData, SwapData};
pub use factory::{EventParserFactory, Protocol, MutilEventParser};

// match_event! macro for pattern matching on events
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