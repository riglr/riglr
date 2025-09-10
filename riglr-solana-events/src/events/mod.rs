//! Event parsing modules for Solana blockchain transactions.
//!
//! This module provides comprehensive event parsing capabilities for various Solana protocols
//! including DEXs, lending protocols, and other DeFi applications. It contains parsers for
//! extracting structured events from transaction data.

/// Common utilities and types shared across protocol parsers
pub mod common;

/// Core event parsing traits and utilities
pub mod core;

/// Parser types and configurations for Solana events
pub mod parser_types;

/// Event parser factory and registry for managing protocol-specific parsers
pub mod factory;

/// Protocol-specific event parsers for various Solana DeFi protocols
pub mod protocols;

pub use crate::types::{EventMetadata, EventType, ProtocolType, SwapData, TransferData};
pub use factory::{EventParserRegistry, Protocol};
pub use parser_types::{GenericEventParseConfig, GenericEventParser, ProtocolParser};

/// Pattern matching macro for event types with type downcasting.
///
/// This macro provides a convenient way to match against different event types
/// by attempting to downcast the event to each specified type. It uses the
/// `as_any()` method to access the underlying `Any` trait for downcasting.
///
/// # Example
///
/// ```rust,ignore
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
        {
            let mut matched = false;
            $(
                if !matched {
                    if let Some($var) = $event.as_any().downcast_ref::<$event_type>() {
                        $body
                        matched = true;
                    }
                }
            )*
            matched
        }
    };
}

// Re-export common utilities
pub use crate::utils::*;

#[cfg(test)]
#[allow(unreachable_code, unused_mut)]
mod tests {
    use std::any::Any;

    // Mock event types for testing the macro
    #[derive(Debug, PartialEq)]
    struct MockSwapEvent {
        pub amount: u64,
    }

    #[derive(Debug, PartialEq)]
    struct MockTransferEvent {
        pub destination: String,
    }

    #[derive(Debug, PartialEq)]
    struct MockOtherEvent {
        pub data: String,
    }

    // Trait to make events downcasting-compatible for macro testing
    trait TestEvent {
        fn as_any(&self) -> &dyn Any;
    }

    impl TestEvent for MockSwapEvent {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    impl TestEvent for MockTransferEvent {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    impl TestEvent for MockOtherEvent {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_match_event_macro_when_swap_event_should_execute_swap_branch() {
        let swap_event = MockSwapEvent { amount: 1000 };

        let matched = match_event!(swap_event, {
            MockSwapEvent => |swap: MockSwapEvent| {
                assert_eq!(swap.amount, 1000);
            },
            MockTransferEvent => |_transfer: MockTransferEvent| {
                panic!("Should not match transfer event");
            },
        });

        assert!(matched);
    }

    #[test]
    fn test_match_event_macro_when_transfer_event_should_execute_transfer_branch() {
        let transfer_event = MockTransferEvent {
            destination: "test_address".to_string(),
        };

        let matched = match_event!(transfer_event, {
            MockSwapEvent => |_swap: MockSwapEvent| {
                panic!("Should not match swap event");
            },
            MockTransferEvent => |transfer: MockTransferEvent| {
                assert_eq!(transfer.destination, "test_address");
            },
        });

        assert!(matched);
    }

    #[test]
    fn test_match_event_macro_when_no_match_should_not_execute_any_branch() {
        let other_event = MockOtherEvent {
            data: "test_data".to_string(),
        };

        let matched = match_event!(other_event, {
            MockSwapEvent => |_swap: MockSwapEvent| {
                panic!("Should not match swap event");
            },
            MockTransferEvent => |_transfer: MockTransferEvent| {
                panic!("Should not match transfer event");
            },
        });

        // No branch should have matched
        assert!(!matched);
    }

    #[test]
    fn test_match_event_macro_when_multiple_types_should_match_first_applicable() {
        let swap_event = MockSwapEvent { amount: 500 };

        let matched = match_event!(swap_event, {
            MockSwapEvent => |swap: MockSwapEvent| {
                // Only the first matching type should execute, verify it's the right match
                assert_eq!(swap.amount, 500);
            },
            MockTransferEvent => |_transfer: MockTransferEvent| {
                panic!("Should not match transfer event - first match should win");
            },
        });

        assert!(matched);
    }

    #[test]
    fn test_match_event_macro_when_empty_event_list_should_not_panic() {
        let swap_event = MockSwapEvent { amount: 100 };

        // This should compile and not panic even with no match arms
        // Note: This tests the edge case of an empty match but the macro
        // requires at least one arm, so we test with a non-matching arm
        let matched = match_event!(swap_event, {
            MockTransferEvent => |_transfer: MockTransferEvent| {
                panic!("Should not match transfer event");
            },
        });

        // No branch should have matched
        assert!(!matched);
    }

    #[test]
    fn test_match_event_macro_when_same_type_multiple_times_should_match_first() {
        let swap_event = MockSwapEvent { amount: 750 };

        let matched = match_event!(swap_event, {
            MockSwapEvent => |swap: MockSwapEvent| {
                // Only the first branch should match due to matched flag check
                assert_eq!(swap.amount, 750);
            },
            MockSwapEvent => |_swap: MockSwapEvent| {
                panic!("Second branch should not execute due to matched flag check in first branch");
            },
        });

        assert!(matched);
    }

    #[test]
    fn test_match_event_macro_with_trailing_comma_should_work() {
        let transfer_event = MockTransferEvent {
            destination: "trailing_comma_test".to_string(),
        };

        let matched = match_event!(transfer_event, {
            MockTransferEvent => |transfer: MockTransferEvent| {
                assert_eq!(transfer.destination, "trailing_comma_test");
            },
        });

        assert!(matched);
    }

    #[test]
    fn test_re_exports_are_accessible() {
        // Test that re-exported items from other modules are accessible
        // This ensures the pub use statements work correctly

        // These should compile without errors if the re-exports work
        let _parser_registry = crate::events::EventParserRegistry::new();
        let _protocol = crate::events::Protocol::Jupiter;
        let _event_type = crate::events::EventType::Swap;
        let _protocol_type = crate::events::ProtocolType::Jupiter;

        // Basic assertion to ensure the test runs
    }
}
