//! Standalone Solana event parsing library extracted from riglr-solana-tools.
//! This crate provides the shared traits, common types/utilities, protocol parsers,
//! and a factory to construct parsers. It can be used independently of riglr.

pub mod events {
    pub mod common {
        pub mod types;
        pub mod utils;
        pub mod instruction_parser;
        pub mod log_parser;
        pub use types::*;
        pub use utils::*;
        pub use instruction_parser::*;
        pub use log_parser::*;
    }

    pub mod core {
        pub mod traits;
        pub use traits::{
            EventParser,
            UnifiedEvent,
            GenericEventParser,
            GenericEventParseConfig,
            InnerInstructionEventParser,
            InstructionEventParser,
        };
    }

    pub mod protocols;
    pub mod factory;

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
}
