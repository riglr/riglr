/// Pumpswap protocol event definitions and constants.
pub mod events;
/// Pumpswap protocol transaction and log parsing functionality.
pub mod parser;

pub use events::*;
pub use parser::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::parser_types::LegacyEventParser;
    use riglr_events_core::Event;
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    #[test]
    fn test_module_exports_events() {
        // Test that we can access all event types through the re-exports
        let _buy_event = PumpSwapBuyEvent::default();
        let _sell_event = PumpSwapSellEvent::default();
        let _create_pool_event = PumpSwapCreatePoolEvent::default();
        let _deposit_event = PumpSwapDepositEvent::default();
        let _withdraw_event = PumpSwapWithdrawEvent::default();
    }

    #[test]
    fn test_module_exports_parser() {
        // Test that we can access the parser through the re-exports
        let _parser = PumpSwapEventParser::new();
        let _default_parser = PumpSwapEventParser::default();
    }

    #[test]
    fn test_module_exports_discriminators() {
        // Test that discriminator constants are accessible
        assert_eq!(discriminators::BUY_EVENT, "pumpswap_buy_event");
        assert_eq!(discriminators::SELL_EVENT, "pumpswap_sell_event");
        assert_eq!(
            discriminators::CREATE_POOL_EVENT,
            "pumpswap_create_pool_event"
        );
        assert_eq!(discriminators::DEPOSIT_EVENT, "pumpswap_deposit_event");
        assert_eq!(discriminators::WITHDRAW_EVENT, "pumpswap_withdraw_event");
    }

    #[test]
    fn test_module_exports_discriminator_bytes() {
        // Test that byte discriminators are accessible and have expected length
        assert_eq!(discriminators::BUY_EVENT_BYTES.len(), 16);
        assert_eq!(discriminators::SELL_EVENT_BYTES.len(), 16);
        assert_eq!(discriminators::CREATE_POOL_EVENT_BYTES.len(), 16);
        assert_eq!(discriminators::DEPOSIT_EVENT_BYTES.len(), 16);
        assert_eq!(discriminators::WITHDRAW_EVENT_BYTES.len(), 16);
    }

    #[test]
    fn test_module_exports_instruction_discriminators() {
        // Test that instruction discriminators are accessible and have expected length
        assert_eq!(discriminators::BUY_IX.len(), 8);
        assert_eq!(discriminators::SELL_IX.len(), 8);
        assert_eq!(discriminators::CREATE_POOL_IX.len(), 8);
        assert_eq!(discriminators::DEPOSIT_IX.len(), 8);
        assert_eq!(discriminators::WITHDRAW_IX.len(), 8);
    }

    #[test]
    fn test_module_exports_program_id() {
        // Test that PUMPSWAP_PROGRAM_ID is accessible
        let program_id = PUMPSWAP_PROGRAM_ID;
        assert_ne!(program_id, Pubkey::default());

        // Verify it's the expected program ID
        let expected = Pubkey::from_str("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA").unwrap();
        assert_eq!(program_id, expected);
    }

    #[test]
    fn test_buy_event_implements_event_trait() {
        let mut event = PumpSwapBuyEvent::default();

        // Test Event trait methods
        let _id = event.id();
        let _kind = event.kind();
        let _metadata = event.metadata();
        let _metadata_mut = event.metadata_mut();
        let _any = event.as_any();
        let _any_mut = event.as_any_mut();
        let _cloned = event.clone_boxed();

        // Test to_json method - should not panic
        let _json_result = event.to_json();
    }

    #[test]
    fn test_sell_event_implements_event_trait() {
        let mut event = PumpSwapSellEvent::default();

        // Test Event trait methods
        let _id = event.id();
        let _kind = event.kind();
        let _metadata = event.metadata();
        let _metadata_mut = event.metadata_mut();
        let _any = event.as_any();
        let _any_mut = event.as_any_mut();
        let _cloned = event.clone_boxed();

        // Test to_json method - should not panic
        let _json_result = event.to_json();
    }

    #[test]
    fn test_create_pool_event_implements_event_trait() {
        let mut event = PumpSwapCreatePoolEvent::default();

        // Test Event trait methods
        let _id = event.id();
        let _kind = event.kind();
        let _metadata = event.metadata();
        let _metadata_mut = event.metadata_mut();
        let _any = event.as_any();
        let _any_mut = event.as_any_mut();
        let _cloned = event.clone_boxed();

        // Test to_json method - should not panic
        let _json_result = event.to_json();
    }

    #[test]
    fn test_deposit_event_implements_event_trait() {
        let mut event = PumpSwapDepositEvent::default();

        // Test Event trait methods
        let _id = event.id();
        let _kind = event.kind();
        let _metadata = event.metadata();
        let _metadata_mut = event.metadata_mut();
        let _any = event.as_any();
        let _any_mut = event.as_any_mut();
        let _cloned = event.clone_boxed();

        // Test to_json method - should not panic
        let _json_result = event.to_json();
    }

    #[test]
    fn test_withdraw_event_implements_event_trait() {
        let mut event = PumpSwapWithdrawEvent::default();

        // Test Event trait methods
        let _id = event.id();
        let _kind = event.kind();
        let _metadata = event.metadata();
        let _metadata_mut = event.metadata_mut();
        let _any = event.as_any();
        let _any_mut = event.as_any_mut();
        let _cloned = event.clone_boxed();

        // Test to_json method - should not panic
        let _json_result = event.to_json();
    }

    #[test]
    fn test_parser_default_vs_new() {
        let parser1 = PumpSwapEventParser::default();
        let parser2 = PumpSwapEventParser::new();

        // Both should have the same program IDs
        assert_eq!(
            parser1.supported_program_ids(),
            parser2.supported_program_ids()
        );

        // Both should handle the same program ID
        assert!(parser1.should_handle(&PUMPSWAP_PROGRAM_ID));
        assert!(parser2.should_handle(&PUMPSWAP_PROGRAM_ID));

        // Both should not handle random program IDs
        let random_pubkey = Pubkey::new_unique();
        assert!(!parser1.should_handle(&random_pubkey));
        assert!(!parser2.should_handle(&random_pubkey));
    }

    #[test]
    fn test_event_structs_clone_and_debug() {
        // Test that all event structs implement Clone and Debug
        let buy_event = PumpSwapBuyEvent::default();
        let _cloned_buy = buy_event.clone();
        let _debug_buy = format!("{:?}", buy_event);

        let sell_event = PumpSwapSellEvent::default();
        let _cloned_sell = sell_event.clone();
        let _debug_sell = format!("{:?}", sell_event);

        let create_pool_event = PumpSwapCreatePoolEvent::default();
        let _cloned_create_pool = create_pool_event.clone();
        let _debug_create_pool = format!("{:?}", create_pool_event);

        let deposit_event = PumpSwapDepositEvent::default();
        let _cloned_deposit = deposit_event.clone();
        let _debug_deposit = format!("{:?}", deposit_event);

        let withdraw_event = PumpSwapWithdrawEvent::default();
        let _cloned_withdraw = withdraw_event.clone();
        let _debug_withdraw = format!("{:?}", withdraw_event);
    }

    #[test]
    fn test_event_structs_partial_eq() {
        // Test that all event structs implement PartialEq
        let buy_event1 = PumpSwapBuyEvent::default();
        let buy_event2 = PumpSwapBuyEvent::default();
        assert_eq!(buy_event1, buy_event2);

        let sell_event1 = PumpSwapSellEvent::default();
        let sell_event2 = PumpSwapSellEvent::default();
        assert_eq!(sell_event1, sell_event2);

        let create_pool_event1 = PumpSwapCreatePoolEvent::default();
        let create_pool_event2 = PumpSwapCreatePoolEvent::default();
        assert_eq!(create_pool_event1, create_pool_event2);

        let deposit_event1 = PumpSwapDepositEvent::default();
        let deposit_event2 = PumpSwapDepositEvent::default();
        assert_eq!(deposit_event1, deposit_event2);

        let withdraw_event1 = PumpSwapWithdrawEvent::default();
        let withdraw_event2 = PumpSwapWithdrawEvent::default();
        assert_eq!(withdraw_event1, withdraw_event2);
    }

    #[test]
    fn test_discriminators_are_unique() {
        // Test that all string discriminators are unique
        let discriminators = vec![
            discriminators::BUY_EVENT,
            discriminators::SELL_EVENT,
            discriminators::CREATE_POOL_EVENT,
            discriminators::DEPOSIT_EVENT,
            discriminators::WITHDRAW_EVENT,
        ];

        for (i, disc1) in discriminators.iter().enumerate() {
            for (j, disc2) in discriminators.iter().enumerate() {
                if i != j {
                    assert_ne!(disc1, disc2, "Discriminators should be unique");
                }
            }
        }
    }

    #[test]
    fn test_byte_discriminators_are_unique() {
        // Test that all byte discriminators are unique
        let byte_discriminators = vec![
            discriminators::BUY_EVENT_BYTES,
            discriminators::SELL_EVENT_BYTES,
            discriminators::CREATE_POOL_EVENT_BYTES,
            discriminators::DEPOSIT_EVENT_BYTES,
            discriminators::WITHDRAW_EVENT_BYTES,
        ];

        for (i, disc1) in byte_discriminators.iter().enumerate() {
            for (j, disc2) in byte_discriminators.iter().enumerate() {
                if i != j {
                    assert_ne!(disc1, disc2, "Byte discriminators should be unique");
                }
            }
        }
    }

    #[test]
    fn test_instruction_discriminators_are_unique() {
        // Test that all instruction discriminators are unique
        let instruction_discriminators = vec![
            discriminators::BUY_IX,
            discriminators::SELL_IX,
            discriminators::CREATE_POOL_IX,
            discriminators::DEPOSIT_IX,
            discriminators::WITHDRAW_IX,
        ];

        for (i, disc1) in instruction_discriminators.iter().enumerate() {
            for (j, disc2) in instruction_discriminators.iter().enumerate() {
                if i != j {
                    assert_ne!(disc1, disc2, "Instruction discriminators should be unique");
                }
            }
        }
    }

    #[test]
    fn test_parser_configs_accessible() {
        let parser = PumpSwapEventParser::new();

        // Test that inner instruction configs are accessible
        let inner_configs = parser.inner_instruction_configs();
        assert!(
            !inner_configs.is_empty(),
            "Should have inner instruction configs"
        );

        // Test that instruction configs are accessible
        let instruction_configs = parser.instruction_configs();
        assert!(
            !instruction_configs.is_empty(),
            "Should have instruction configs"
        );
    }

    #[test]
    fn test_supported_program_ids_contains_pumpswap() {
        let parser = PumpSwapEventParser::new();
        let supported = parser.supported_program_ids();

        assert!(
            supported.contains(&PUMPSWAP_PROGRAM_ID),
            "Should support PUMPSWAP_PROGRAM_ID"
        );
        assert!(
            !supported.is_empty(),
            "Should have at least one supported program ID"
        );
    }
}
