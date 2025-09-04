/// Raydium CPMM event definitions and structures.
pub mod events;
/// Raydium CPMM transaction and log parsing functionality.
pub mod parser;

pub use events::*;
pub use parser::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::parser_types::LegacyEventParser;
    use riglr_events_core::{Event, EventKind};
    use solana_sdk::pubkey::Pubkey;
    use std::collections::HashMap;

    #[test]
    fn test_module_re_exports_events() {
        // Test that events module items are re-exported correctly
        let _swap_event = RaydiumCpmmSwapEvent::default();
        let _deposit_event = RaydiumCpmmDepositEvent::default();
    }

    #[test]
    fn test_module_re_exports_parser() {
        // Test that parser module items are re-exported correctly
        let _parser = RaydiumCpmmEventParser::new();
        let _program_id = RAYDIUM_CPMM_PROGRAM_ID;
    }

    #[test]
    fn test_module_re_exports_discriminators() {
        // Test that discriminator constants are re-exported correctly
        let _swap_event_str = discriminators::SWAP_EVENT;
        let _deposit_event_str = discriminators::DEPOSIT_EVENT;
        let _swap_event_bytes = discriminators::SWAP_EVENT_BYTES;
        let _deposit_event_bytes = discriminators::DEPOSIT_EVENT_BYTES;
        let _swap_base_input_ix = discriminators::SWAP_BASE_INPUT_IX;
        let _swap_base_output_ix = discriminators::SWAP_BASE_OUTPUT_IX;
        let _deposit_ix = discriminators::DEPOSIT_IX;
        let _initialize_ix = discriminators::INITIALIZE_IX;
        let _withdraw_ix = discriminators::WITHDRAW_IX;

        // Verify discriminator values
        assert_eq!(_swap_event_str, "raydium_cpmm_swap_event");
        assert_eq!(_deposit_event_str, "raydium_cpmm_deposit_event");
        assert_eq!(_swap_event_bytes.len(), 16);
        assert_eq!(_deposit_event_bytes.len(), 16);
        assert_eq!(_swap_base_input_ix.len(), 8);
        assert_eq!(_swap_base_output_ix.len(), 8);
        assert_eq!(_deposit_ix.len(), 8);
        assert_eq!(_initialize_ix.len(), 8);
        assert_eq!(_withdraw_ix.len(), 8);
    }

    #[test]
    fn test_raydium_cpmm_swap_event_default() {
        // Test RaydiumCpmmSwapEvent default implementation
        let event = RaydiumCpmmSwapEvent::default();
        assert_eq!(event.pool_state, Pubkey::default());
        assert_eq!(event.payer, Pubkey::default());
        assert_eq!(event.input_token_account, Pubkey::default());
        assert_eq!(event.output_token_account, Pubkey::default());
        assert_eq!(event.input_vault, Pubkey::default());
        assert_eq!(event.output_vault, Pubkey::default());
        assert_eq!(event.input_token_mint, Pubkey::default());
        assert_eq!(event.output_token_mint, Pubkey::default());
        assert_eq!(event.amount_in, 0);
        assert_eq!(event.amount_out, 0);
        assert_eq!(event.trade_fee, 0);
        assert_eq!(event.transfer_fee, 0);
    }

    #[test]
    fn test_raydium_cpmm_swap_event_trait_implementations() {
        // Test Event trait implementation for RaydiumCpmmSwapEvent
        let event = RaydiumCpmmSwapEvent::default();

        // Test id method
        assert_eq!(event.id(), "");

        // Test kind method
        assert_eq!(*event.kind(), EventKind::Swap);

        // Test metadata method
        let metadata = event.metadata();
        assert_eq!(metadata.id, "");
        assert_eq!(metadata.kind, EventKind::Swap);
        assert_eq!(metadata.source, "solana");
        assert!(metadata.chain_data.is_none());
        assert_eq!(metadata.custom, HashMap::default());

        // Test as_any method
        let any_ref = event.as_any();
        assert!(any_ref.is::<RaydiumCpmmSwapEvent>());

        // Test clone_boxed method
        let boxed_event = event.clone_boxed();
        assert!(boxed_event.as_any().is::<RaydiumCpmmSwapEvent>());

        // Test clone trait
        let cloned_event = event.clone();
        assert_eq!(cloned_event, event);
    }

    #[test]
    fn test_raydium_cpmm_swap_event_metadata_mut_should_work() {
        // Test that metadata_mut works correctly
        let mut event = RaydiumCpmmSwapEvent::default();
        let metadata = event.metadata_mut();
        metadata.id = "test-cpmm-swap-id".to_string();
        assert_eq!(event.metadata().id, "test-cpmm-swap-id");
    }

    #[test]
    fn test_raydium_cpmm_swap_event_as_any_mut() {
        // Test as_any_mut method
        let mut event = RaydiumCpmmSwapEvent::default();
        let any_mut_ref = event.as_any_mut();
        assert!(any_mut_ref.is::<RaydiumCpmmSwapEvent>());
    }

    #[test]
    fn test_raydium_cpmm_deposit_event_default() {
        // Test RaydiumCpmmDepositEvent default implementation
        let event = RaydiumCpmmDepositEvent::default();
        assert_eq!(event.pool_state, Pubkey::default());
        assert_eq!(event.user, Pubkey::default());
        assert_eq!(event.lp_token_amount, 0);
        assert_eq!(event.token_0_amount, 0);
        assert_eq!(event.token_1_amount, 0);
    }

    #[test]
    fn test_raydium_cpmm_deposit_event_trait_implementations() {
        // Test Event trait implementation for RaydiumCpmmDepositEvent
        let event = RaydiumCpmmDepositEvent::default();

        // Test id method
        assert_eq!(event.id(), "");

        // Test kind method
        assert_eq!(*event.kind(), EventKind::Liquidity);

        // Test metadata method
        let metadata = event.metadata();
        assert_eq!(metadata.id, "");
        assert_eq!(metadata.kind, EventKind::Liquidity);
        assert_eq!(metadata.source, "solana");
        assert!(metadata.chain_data.is_none());
        assert_eq!(metadata.custom, HashMap::default());

        // Test as_any method
        let any_ref = event.as_any();
        assert!(any_ref.is::<RaydiumCpmmDepositEvent>());

        // Test clone_boxed method
        let boxed_event = event.clone_boxed();
        assert!(boxed_event.as_any().is::<RaydiumCpmmDepositEvent>());

        // Test clone trait
        let cloned_event = event.clone();
        assert_eq!(cloned_event, event);
    }

    #[test]
    fn test_raydium_cpmm_deposit_event_metadata_mut_should_work() {
        // Test that metadata_mut works correctly
        let mut event = RaydiumCpmmDepositEvent::default();
        let metadata = event.metadata_mut();
        metadata.id = "test-cpmm-deposit-id".to_string();
        assert_eq!(event.metadata().id, "test-cpmm-deposit-id");
    }

    #[test]
    fn test_raydium_cpmm_deposit_event_as_any_mut() {
        // Test as_any_mut method
        let mut event = RaydiumCpmmDepositEvent::default();
        let any_mut_ref = event.as_any_mut();
        assert!(any_mut_ref.is::<RaydiumCpmmDepositEvent>());
    }

    #[test]
    fn test_raydium_cpmm_event_parser_new() {
        // Test RaydiumCpmmEventParser::new()
        let parser = RaydiumCpmmEventParser::new();

        // Test should_handle method
        assert!(parser.should_handle(&RAYDIUM_CPMM_PROGRAM_ID));
        assert!(!parser.should_handle(&Pubkey::default()));

        // Test supported_program_ids method
        let supported_ids = parser.supported_program_ids();
        assert!(supported_ids.contains(&RAYDIUM_CPMM_PROGRAM_ID));
        assert_eq!(supported_ids.len(), 1);
    }

    #[test]
    fn test_raydium_cpmm_event_parser_default() {
        // Test RaydiumCpmmEventParser::default()
        let parser = RaydiumCpmmEventParser::default();

        // Test that default and new produce equivalent results
        let new_parser = RaydiumCpmmEventParser::new();
        assert_eq!(
            parser.supported_program_ids(),
            new_parser.supported_program_ids()
        );
    }

    #[test]
    fn test_raydium_cpmm_event_parser_configs() {
        // Test parser configuration methods
        let parser = RaydiumCpmmEventParser::new();

        // Test inner_instruction_configs
        let inner_configs = parser.inner_instruction_configs();
        assert!(inner_configs.contains_key(discriminators::SWAP_EVENT));
        assert!(inner_configs.contains_key(discriminators::DEPOSIT_EVENT));

        // Test instruction_configs
        let instruction_configs = parser.instruction_configs();
        assert!(instruction_configs.contains_key(&discriminators::SWAP_BASE_INPUT_IX.to_vec()));
        assert!(instruction_configs.contains_key(&discriminators::SWAP_BASE_OUTPUT_IX.to_vec()));
        assert!(instruction_configs.contains_key(&discriminators::DEPOSIT_IX.to_vec()));
    }

    #[test]
    fn test_program_id_constant() {
        // Test that the RAYDIUM_CPMM_PROGRAM_ID constant is correct
        let expected_program_id =
            solana_sdk::pubkey!("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C");
        assert_eq!(RAYDIUM_CPMM_PROGRAM_ID, expected_program_id);
    }

    #[test]
    fn test_swap_event_with_custom_values() {
        // Test RaydiumCpmmSwapEvent with custom values
        let pool_state = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let input_token_account = Pubkey::new_unique();
        let output_token_account = Pubkey::new_unique();
        let input_vault = Pubkey::new_unique();
        let output_vault = Pubkey::new_unique();
        let input_token_mint = Pubkey::new_unique();
        let output_token_mint = Pubkey::new_unique();

        let event = RaydiumCpmmSwapEvent {
            pool_state,
            payer,
            input_token_account,
            output_token_account,
            input_vault,
            output_vault,
            input_token_mint,
            output_token_mint,
            amount_in: 1000,
            amount_out: 900,
            trade_fee: 10,
            transfer_fee: 5,
            ..Default::default()
        };

        assert_eq!(event.pool_state, pool_state);
        assert_eq!(event.payer, payer);
        assert_eq!(event.input_token_account, input_token_account);
        assert_eq!(event.output_token_account, output_token_account);
        assert_eq!(event.input_vault, input_vault);
        assert_eq!(event.output_vault, output_vault);
        assert_eq!(event.input_token_mint, input_token_mint);
        assert_eq!(event.output_token_mint, output_token_mint);
        assert_eq!(event.amount_in, 1000);
        assert_eq!(event.amount_out, 900);
        assert_eq!(event.trade_fee, 10);
        assert_eq!(event.transfer_fee, 5);
    }

    #[test]
    fn test_deposit_event_with_custom_values() {
        // Test RaydiumCpmmDepositEvent with custom values
        let pool_state = Pubkey::new_unique();
        let user = Pubkey::new_unique();

        let event = RaydiumCpmmDepositEvent {
            pool_state,
            user,
            lp_token_amount: 1000,
            token_0_amount: 500,
            token_1_amount: 600,
            ..Default::default()
        };

        assert_eq!(event.pool_state, pool_state);
        assert_eq!(event.user, user);
        assert_eq!(event.lp_token_amount, 1000);
        assert_eq!(event.token_0_amount, 500);
        assert_eq!(event.token_1_amount, 600);
    }

    #[test]
    fn test_event_partial_eq() {
        // Test PartialEq implementation for events
        let event1 = RaydiumCpmmSwapEvent::default();
        let event2 = RaydiumCpmmSwapEvent::default();
        assert_eq!(event1, event2);

        let mut event3 = RaydiumCpmmSwapEvent::default();
        event3.amount_in = 100;
        assert_ne!(event1, event3);

        let deposit_event1 = RaydiumCpmmDepositEvent::default();
        let deposit_event2 = RaydiumCpmmDepositEvent::default();
        assert_eq!(deposit_event1, deposit_event2);

        let mut deposit_event3 = RaydiumCpmmDepositEvent::default();
        deposit_event3.lp_token_amount = 100;
        assert_ne!(deposit_event1, deposit_event3);
    }

    #[test]
    fn test_event_debug_format() {
        // Test Debug implementation for events
        let swap_event = RaydiumCpmmSwapEvent::default();
        let debug_str = format!("{:?}", swap_event);
        assert!(debug_str.contains("RaydiumCpmmSwapEvent"));

        let deposit_event = RaydiumCpmmDepositEvent::default();
        let debug_str = format!("{:?}", deposit_event);
        assert!(debug_str.contains("RaydiumCpmmDepositEvent"));
    }
}
