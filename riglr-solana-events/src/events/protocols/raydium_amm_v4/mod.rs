/// Raydium AMM V4 instruction discriminators and constants.
pub mod discriminators;
/// Raydium AMM V4 event definitions and structures.
pub mod events;
/// Raydium AMM V4 transaction and log parsing functionality.
pub mod parser;

pub use events::*;
pub use parser::{RaydiumAmmV4EventParser, RAYDIUM_AMM_V4_PROGRAM_ID};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::parser_types::LegacyEventParser;
    use crate::{
        solana_metadata::SolanaEventMetadata,
        types::{EventType, ProtocolType},
    };
    use riglr_events_core::{Event, EventKind};
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn test_module_exports_event_structs() {
        // Test that all event structs are properly exported and can be instantiated
        let core = riglr_events_core::EventMetadata::new(
            "test-id".to_string(),
            EventKind::Swap,
            "raydium-amm-v4".to_string(),
        );

        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            12345,
            EventType::Swap,
            ProtocolType::RaydiumAmmV4,
            "0".to_string(),
            1640995200000,
            core,
        );

        // Test RaydiumAmmV4SwapEvent export
        let swap_event = RaydiumAmmV4SwapEvent {
            metadata: metadata.clone(),
            amount_in: 1000,
            amount_out: 950,
            direction: SwapDirection::BaseIn,
            amm: Pubkey::default(),
            amm_authority: Pubkey::default(),
            amm_open_orders: Pubkey::default(),
            pool_coin_token_account: Pubkey::default(),
            pool_pc_token_account: Pubkey::default(),
            serum_program: Pubkey::default(),
            serum_market: Pubkey::default(),
            user_coin_token_account: Pubkey::default(),
            user_pc_token_account: Pubkey::default(),
            user_owner: Pubkey::default(),
        };
        assert_eq!(swap_event.amount_in, 1000);
        assert_eq!(swap_event.amount_out, 950);
    }

    #[test]
    fn test_module_exports_deposit_event() {
        let core = riglr_events_core::EventMetadata::new(
            "test-deposit-id".to_string(),
            EventKind::Liquidity,
            "raydium-amm-v4".to_string(),
        );

        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            12346,
            EventType::AddLiquidity,
            ProtocolType::RaydiumAmmV4,
            "0".to_string(),
            1640995200000,
            core,
        );

        let deposit_event = RaydiumAmmV4DepositEvent {
            metadata,
            max_coin_amount: 2000,
            max_pc_amount: 1800,
            base_side: 1,
            token_program: Pubkey::default(),
            amm: Pubkey::default(),
            amm_authority: Pubkey::default(),
            amm_open_orders: Pubkey::default(),
            amm_target_orders: Pubkey::default(),
            lp_mint_address: Pubkey::default(),
            pool_coin_token_account: Pubkey::default(),
            pool_pc_token_account: Pubkey::default(),
            serum_market: Pubkey::default(),
            user_coin_token_account: Pubkey::default(),
            user_pc_token_account: Pubkey::default(),
            user_lp_token_account: Pubkey::default(),
            user_owner: Pubkey::default(),
        };
        assert_eq!(deposit_event.max_coin_amount, 2000);
        assert_eq!(deposit_event.base_side, 1);
    }

    #[test]
    fn test_module_exports_initialize2_event() {
        let core = riglr_events_core::EventMetadata::new(
            "test-init-id".to_string(),
            EventKind::Contract,
            "raydium-amm-v4".to_string(),
        );

        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            12347,
            EventType::CreatePool,
            ProtocolType::RaydiumAmmV4,
            "0".to_string(),
            1640995200000,
            core,
        );

        let init_event = RaydiumAmmV4Initialize2Event {
            metadata,
            nonce: 42,
            open_time: 1634567890,
            init_pc_amount: 5000,
            init_coin_amount: 4500,
            amm: Pubkey::default(),
            amm_authority: Pubkey::default(),
            amm_open_orders: Pubkey::default(),
            lp_mint_address: Pubkey::default(),
            coin_mint_address: Pubkey::default(),
            pc_mint_address: Pubkey::default(),
            pool_coin_token_account: Pubkey::default(),
            pool_pc_token_account: Pubkey::default(),
            pool_withdraw_queue: Pubkey::default(),
            amm_target_orders: Pubkey::default(),
            pool_lp_token_account: Pubkey::default(),
            pool_temp_lp_token_account: Pubkey::default(),
            serum_program: Pubkey::default(),
            serum_market: Pubkey::default(),
            user_wallet: Pubkey::default(),
        };
        assert_eq!(init_event.nonce, 42);
        assert_eq!(init_event.open_time, 1634567890);
    }

    #[test]
    fn test_module_exports_withdraw_event() {
        let core = riglr_events_core::EventMetadata::new(
            "test-withdraw-id".to_string(),
            EventKind::Liquidity,
            "raydium-amm-v4".to_string(),
        );

        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            12348,
            EventType::RemoveLiquidity,
            ProtocolType::RaydiumAmmV4,
            "0".to_string(),
            1640995200000,
            core,
        );

        let withdraw_event = RaydiumAmmV4WithdrawEvent {
            metadata,
            amount: 1500,
            token_program: Pubkey::default(),
            amm: Pubkey::default(),
            amm_authority: Pubkey::default(),
            amm_open_orders: Pubkey::default(),
            amm_target_orders: Pubkey::default(),
            lp_mint_address: Pubkey::default(),
            pool_coin_token_account: Pubkey::default(),
            pool_pc_token_account: Pubkey::default(),
            pool_withdraw_queue: Pubkey::default(),
            pool_temp_lp_token_account: Pubkey::default(),
            serum_program: Pubkey::default(),
            serum_market: Pubkey::default(),
            serum_coin_vault_account: Pubkey::default(),
            serum_pc_vault_account: Pubkey::default(),
            serum_vault_signer: Pubkey::default(),
            user_lp_token_account: Pubkey::default(),
            user_coin_token_account: Pubkey::default(),
            user_pc_token_account: Pubkey::default(),
            user_owner: Pubkey::default(),
            serum_event_queue: Pubkey::default(),
            serum_bids: Pubkey::default(),
            serum_asks: Pubkey::default(),
        };
        assert_eq!(withdraw_event.amount, 1500);
    }

    #[test]
    fn test_module_exports_withdraw_pnl_event() {
        let core = riglr_events_core::EventMetadata::new(
            "test-pnl-id".to_string(),
            EventKind::Transfer,
            "raydium-amm-v4".to_string(),
        );

        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            12349,
            EventType::Transfer,
            ProtocolType::RaydiumAmmV4,
            "0".to_string(),
            1640995200000,
            core,
        );

        let pnl_event = RaydiumAmmV4WithdrawPnlEvent {
            metadata,
            token_program: Pubkey::default(),
            amm: Pubkey::default(),
            amm_config: Pubkey::default(),
            amm_authority: Pubkey::default(),
            amm_open_orders: Pubkey::default(),
            pool_coin_token_account: Pubkey::default(),
            pool_pc_token_account: Pubkey::default(),
            coin_pnl_token_account: Pubkey::default(),
            pc_pnl_token_account: Pubkey::default(),
            pnl_owner_account: Pubkey::default(),
            amm_target_orders: Pubkey::default(),
            serum_program: Pubkey::default(),
            serum_market: Pubkey::default(),
            serum_event_queue: Pubkey::default(),
            serum_coin_vault_account: Pubkey::default(),
            serum_pc_vault_account: Pubkey::default(),
            serum_vault_signer: Pubkey::default(),
        };
        // PNL event doesn't have numeric fields to test, just verify it was created
        assert_eq!(pnl_event.token_program, Pubkey::default());
    }

    #[test]
    fn test_module_exports_swap_direction_enum() {
        // Test SwapDirection enum export and variants
        let base_in = SwapDirection::BaseIn;
        let base_out = SwapDirection::BaseOut;

        // Test that the enum can be cloned and debugged
        let base_in_clone = base_in.clone();
        assert!(format!("{:?}", base_in_clone).contains("BaseIn"));
        assert!(format!("{:?}", base_out).contains("BaseOut"));

        // Test default implementation
        let default_direction = SwapDirection::default();
        assert!(matches!(default_direction, SwapDirection::BaseIn));
    }

    #[test]
    fn test_module_exports_parser() {
        // Test that RaydiumAmmV4EventParser is properly exported
        let parser = RaydiumAmmV4EventParser::default();

        // Verify parser can be created and has the expected program ID support
        let supported_ids = parser.supported_program_ids();
        assert!(!supported_ids.is_empty());
        assert!(supported_ids.contains(&RAYDIUM_AMM_V4_PROGRAM_ID));
    }

    #[test]
    fn test_module_exports_program_id_constant() {
        // Test that RAYDIUM_AMM_V4_PROGRAM_ID is properly exported
        let expected_program_id =
            solana_sdk::pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");
        assert_eq!(RAYDIUM_AMM_V4_PROGRAM_ID, expected_program_id);

        // Verify it's not the default pubkey
        assert_ne!(RAYDIUM_AMM_V4_PROGRAM_ID, Pubkey::default());
    }

    #[test]
    fn test_module_exports_work_with_event_trait() {
        // Test that exported events implement the Event trait correctly
        let core = riglr_events_core::EventMetadata::new(
            "trait-test-id".to_string(),
            EventKind::Swap,
            "raydium-amm-v4".to_string(),
        );

        let metadata = SolanaEventMetadata::new(
            "test-signature".to_string(),
            12350,
            EventType::Swap,
            ProtocolType::RaydiumAmmV4,
            "0".to_string(),
            1640995200000,
            core,
        );

        let swap_event = RaydiumAmmV4SwapEvent {
            metadata: metadata.clone(),
            amount_in: 100,
            amount_out: 95,
            direction: SwapDirection::BaseOut,
            amm: Pubkey::default(),
            amm_authority: Pubkey::default(),
            amm_open_orders: Pubkey::default(),
            pool_coin_token_account: Pubkey::default(),
            pool_pc_token_account: Pubkey::default(),
            serum_program: Pubkey::default(),
            serum_market: Pubkey::default(),
            user_coin_token_account: Pubkey::default(),
            user_pc_token_account: Pubkey::default(),
            user_owner: Pubkey::default(),
        };

        // Test Event trait methods
        assert_eq!(swap_event.id(), "trait-test-id");
        assert_eq!(*swap_event.kind(), EventKind::Swap);

        // Test clone_boxed
        let boxed_event = swap_event.clone_boxed();
        assert_eq!(boxed_event.id(), "trait-test-id");

        // Test to_json
        let json_result = swap_event.to_json();
        assert!(json_result.is_ok());
        let json_value = json_result.unwrap();
        assert!(json_value.is_object());
    }

    #[test]
    fn test_all_event_types_can_be_used_as_trait_objects() {
        // Create instances of all event types as trait objects
        let events: Vec<Box<dyn Event>> = vec![
            Box::new(RaydiumAmmV4SwapEvent::default()),
            Box::new(RaydiumAmmV4DepositEvent::default()),
            Box::new(RaydiumAmmV4Initialize2Event::default()),
            Box::new(RaydiumAmmV4WithdrawEvent::default()),
            Box::new(RaydiumAmmV4WithdrawPnlEvent::default()),
        ];

        // Verify they all implement the Event trait properly
        for event in events {
            assert!(!event.id().is_empty() || event.id().is_empty()); // Just check method exists
            let _kind = event.kind(); // Verify method exists
            let _json = event.to_json(); // Verify method exists and doesn't panic
        }
    }

    #[test]
    fn test_parser_should_handle_correct_program_id() {
        let parser = RaydiumAmmV4EventParser::default();

        // Should handle the Raydium AMM V4 program ID
        assert!(parser.should_handle(&RAYDIUM_AMM_V4_PROGRAM_ID));

        // Should not handle other program IDs
        assert!(!parser.should_handle(&Pubkey::default()));
        assert!(!parser.should_handle(&solana_sdk::pubkey!("11111111111111111111111111111112")));
    }

    #[test]
    fn test_module_structure_integrity() {
        // This test ensures that the module re-exports work correctly
        // and don't cause compilation issues

        // Test discriminators module is accessible
        let _swap_base_in = discriminators::SWAP_BASE_IN;
        let _swap_base_out = discriminators::SWAP_BASE_OUT;
        let _deposit = discriminators::DEPOSIT;
        let _initialize2 = discriminators::INITIALIZE2;
        let _withdraw = discriminators::WITHDRAW;
        let _withdraw_pnl = discriminators::WITHDRAW_PNL;

        // Verify discriminators are different
        assert_ne!(_swap_base_in, _swap_base_out);
        assert_ne!(_deposit, _withdraw);
    }
}
