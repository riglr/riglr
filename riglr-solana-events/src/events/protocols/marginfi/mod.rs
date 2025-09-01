/// MarginFi protocol event definitions and constants.
pub mod events;
/// MarginFi protocol transaction and log parsing functionality.
pub mod parser;
/// MarginFi protocol data types and structures.
pub mod types;

pub use events::*;
pub use parser::*;
pub use types::*;

// Re-export core types
pub use crate::events::core::EventParameters;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::parser_types::ProtocolParser;
    use crate::types::TransferData;
    use riglr_events_core::{Event, EventKind};
    use solana_sdk::pubkey::Pubkey;

    // Test module structure and re-exports
    #[test]
    fn test_module_reexports_all_public_types() {
        // Test that all major types are accessible through re-exports
        let _ = EventParameters::default();
        let _ = MarginFiDepositEvent::default();
        let _ = MarginFiWithdrawEvent::default();
        let _ = MarginFiBorrowEvent::default();
        let _ = MarginFiRepayEvent::default();
        let _ = MarginFiLiquidationEvent::default();

        // Test data types
        let _ = MarginFiDepositData::default();
        let _ = MarginFiWithdrawData::default();
        let _ = MarginFiBorrowData::default();
        let _ = MarginFiRepayData::default();
        let _ = MarginFiLiquidationData::default();

        // Test parser
        let _ = MarginFiEventParser::default();

        // Test utility functions
        let _ = marginfi_program_id();
        let _ = marginfi_bank_program_id();
    }

    #[test]
    fn test_event_parameters_creation_and_usage() {
        let params = EventParameters::new(
            "test_id".to_string(),
            "test_signature".to_string(),
            12345,
            1640995200,
            1640995200000,
            1640995200500,
            "0".to_string(),
        );

        // Test that parameters are stored correctly
        assert_eq!(params.id, "test_id");
        assert_eq!(params.signature, "test_signature");
        assert_eq!(params.slot, 12345);
        assert_eq!(params.block_time, 1640995200);
        assert_eq!(params.block_time_ms, 1640995200000);
        assert_eq!(params.program_received_time_ms, 1640995200500);
        assert_eq!(params.index, "0");

        // Test event creation with parameters
        let deposit_data = MarginFiDepositData::default();
        let event = MarginFiDepositEvent::new(params, deposit_data);

        assert_eq!(event.id(), "test_id");
        assert_eq!(event.metadata.signature, "test_signature");
        assert_eq!(event.metadata.slot, 12345);
    }

    #[test]
    fn test_event_parameters_default() {
        let params = EventParameters::default();

        assert_eq!(params.id, "");
        assert_eq!(params.signature, "");
        assert_eq!(params.slot, 0);
        assert_eq!(params.block_time, 0);
        assert_eq!(params.block_time_ms, 0);
        assert_eq!(params.program_received_time_ms, 0);
        assert_eq!(params.index, "");
    }

    #[test]
    fn test_marginfi_deposit_event_creation() {
        let params = EventParameters::new(
            "deposit_test".to_string(),
            "sig123".to_string(),
            100,
            1640995200,
            1640995200000,
            1640995200100,
            "1".to_string(),
        );

        let deposit_data = MarginFiDepositData {
            marginfi_group: Pubkey::new_unique(),
            marginfi_account: Pubkey::new_unique(),
            signer: Pubkey::new_unique(),
            bank: Pubkey::new_unique(),
            token_account: Pubkey::new_unique(),
            bank_liquidity_vault: Pubkey::new_unique(),
            token_program: Pubkey::new_unique(),
            amount: 1000000,
        };

        let event = MarginFiDepositEvent::new(params, deposit_data.clone());

        assert_eq!(event.id(), "deposit_test");
        assert_eq!(event.metadata.signature, "sig123");
        assert_eq!(event.metadata.slot, 100);
        assert_eq!(event.deposit_data.amount, 1000000);
        assert!(event.transfer_data.is_empty());
        assert_eq!(event.metadata.program_received_time_ms, 0);
    }

    #[test]
    fn test_marginfi_deposit_event_with_transfer_data() {
        let params = EventParameters::default();
        let deposit_data = MarginFiDepositData::default();

        let transfer_data = vec![
            TransferData {
                source: Pubkey::new_unique(),
                destination: Pubkey::new_unique(),
                amount: 500000,
                mint: Some(Pubkey::new_unique()),
            },
            TransferData {
                source: Pubkey::new_unique(),
                destination: Pubkey::new_unique(),
                amount: 250000,
                mint: Some(Pubkey::new_unique()),
            },
        ];

        let event = MarginFiDepositEvent::new(params, deposit_data)
            .with_transfer_data(transfer_data.clone());

        assert_eq!(event.transfer_data.len(), 2);
        assert_eq!(event.transfer_data[0].amount, 500000);
        assert_eq!(event.transfer_data[1].amount, 250000);
    }

    #[test]
    fn test_marginfi_deposit_event_default() {
        let event = MarginFiDepositEvent::default();

        assert_eq!(event.id(), "");
        assert_eq!(event.metadata.signature, "");
        assert_eq!(event.metadata.slot, 0);
        assert_eq!(
            crate::metadata_helpers::get_block_time(&event.metadata.core).unwrap_or(0),
            0
        );
        assert_eq!(
            crate::metadata_helpers::get_block_time(&event.metadata.core).unwrap_or(0) * 1000,
            0
        );
        assert_eq!(event.metadata.program_received_time_ms, 0);
        assert_eq!(event.metadata.program_received_time_ms, 0);
        assert_eq!(event.metadata.index, "");
        assert_eq!(event.deposit_data.amount, 0);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_marginfi_withdraw_event_creation() {
        let params = EventParameters::new(
            "withdraw_test".to_string(),
            "sig456".to_string(),
            200,
            1640995300,
            1640995300000,
            1640995300200,
            "2".to_string(),
        );

        let withdraw_data = MarginFiWithdrawData {
            marginfi_group: Pubkey::new_unique(),
            marginfi_account: Pubkey::new_unique(),
            signer: Pubkey::new_unique(),
            bank: Pubkey::new_unique(),
            token_account: Pubkey::new_unique(),
            bank_liquidity_vault: Pubkey::new_unique(),
            bank_liquidity_vault_authority: Pubkey::new_unique(),
            token_program: Pubkey::new_unique(),
            amount: 2000000,
            withdraw_all: true,
        };

        let event = MarginFiWithdrawEvent::new(params, withdraw_data.clone());

        assert_eq!(event.id(), "withdraw_test");
        assert_eq!(event.metadata.signature, "sig456");
        assert_eq!(event.metadata.slot, 200);
        assert_eq!(event.withdraw_data.amount, 2000000);
        assert!(event.withdraw_data.withdraw_all);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_marginfi_withdraw_event_with_transfer_data() {
        let params = EventParameters::default();
        let withdraw_data = MarginFiWithdrawData::default();

        let transfer_data = vec![TransferData {
            source: Pubkey::new_unique(),
            destination: Pubkey::new_unique(),
            amount: 750000,
            mint: Some(Pubkey::new_unique()),
        }];

        let event = MarginFiWithdrawEvent::new(params, withdraw_data)
            .with_transfer_data(transfer_data.clone());

        assert_eq!(event.transfer_data.len(), 1);
        assert_eq!(event.transfer_data[0].amount, 750000);
    }

    #[test]
    fn test_marginfi_borrow_event_creation() {
        let params = EventParameters::new(
            "borrow_test".to_string(),
            "sig789".to_string(),
            300,
            1640995400,
            1640995400000,
            1640995400300,
            "3".to_string(),
        );

        let borrow_data = MarginFiBorrowData {
            marginfi_group: Pubkey::new_unique(),
            marginfi_account: Pubkey::new_unique(),
            signer: Pubkey::new_unique(),
            bank: Pubkey::new_unique(),
            token_account: Pubkey::new_unique(),
            bank_liquidity_vault: Pubkey::new_unique(),
            bank_liquidity_vault_authority: Pubkey::new_unique(),
            token_program: Pubkey::new_unique(),
            amount: 3000000,
        };

        let event = MarginFiBorrowEvent::new(params, borrow_data.clone());

        assert_eq!(event.id(), "borrow_test");
        assert_eq!(event.metadata.signature, "sig789");
        assert_eq!(event.metadata.slot, 300);
        assert_eq!(event.borrow_data.amount, 3000000);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_marginfi_borrow_event_with_transfer_data() {
        let params = EventParameters::default();
        let borrow_data = MarginFiBorrowData::default();

        let transfer_data = vec![TransferData {
            source: Pubkey::new_unique(),
            destination: Pubkey::new_unique(),
            amount: 1500000,
            mint: Some(Pubkey::new_unique()),
        }];

        let event =
            MarginFiBorrowEvent::new(params, borrow_data).with_transfer_data(transfer_data.clone());

        assert_eq!(event.transfer_data.len(), 1);
        assert_eq!(event.transfer_data[0].amount, 1500000);
    }

    #[test]
    fn test_marginfi_repay_event_creation() {
        let params = EventParameters::new(
            "repay_test".to_string(),
            "sig101112".to_string(),
            400,
            1640995500,
            1640995500000,
            1640995500400,
            "4".to_string(),
        );

        let repay_data = MarginFiRepayData {
            marginfi_group: Pubkey::new_unique(),
            marginfi_account: Pubkey::new_unique(),
            signer: Pubkey::new_unique(),
            bank: Pubkey::new_unique(),
            token_account: Pubkey::new_unique(),
            bank_liquidity_vault: Pubkey::new_unique(),
            token_program: Pubkey::new_unique(),
            amount: 4000000,
            repay_all: false,
        };

        let event = MarginFiRepayEvent::new(params, repay_data.clone());

        assert_eq!(event.id(), "repay_test");
        assert_eq!(event.metadata.signature, "sig101112");
        assert_eq!(event.metadata.slot, 400);
        assert_eq!(event.repay_data.amount, 4000000);
        assert!(!event.repay_data.repay_all);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_marginfi_repay_event_with_transfer_data() {
        let params = EventParameters::default();
        let repay_data = MarginFiRepayData::default();

        let transfer_data = vec![TransferData {
            source: Pubkey::new_unique(),
            destination: Pubkey::new_unique(),
            amount: 2250000,
            mint: Some(Pubkey::new_unique()),
        }];

        let event =
            MarginFiRepayEvent::new(params, repay_data).with_transfer_data(transfer_data.clone());

        assert_eq!(event.transfer_data.len(), 1);
        assert_eq!(event.transfer_data[0].amount, 2250000);
    }

    #[test]
    fn test_marginfi_liquidation_event_creation() {
        let params = EventParameters::new(
            "liquidation_test".to_string(),
            "sig131415".to_string(),
            500,
            1640995600,
            1640995600000,
            1640995600500,
            "5".to_string(),
        );

        let liquidation_data = MarginFiLiquidationData {
            marginfi_group: Pubkey::new_unique(),
            asset_bank: Pubkey::new_unique(),
            liab_bank: Pubkey::new_unique(),
            liquidatee_marginfi_account: Pubkey::new_unique(),
            liquidator_marginfi_account: Pubkey::new_unique(),
            liquidator: Pubkey::new_unique(),
            asset_bank_liquidity_vault: Pubkey::new_unique(),
            liab_bank_liquidity_vault: Pubkey::new_unique(),
            liquidator_token_account: Pubkey::new_unique(),
            token_program: Pubkey::new_unique(),
            asset_amount: 5000000,
            liab_amount: 4500000,
        };

        let event = MarginFiLiquidationEvent::new(params, liquidation_data.clone());

        assert_eq!(event.id(), "liquidation_test");
        assert_eq!(event.metadata.signature, "sig131415");
        assert_eq!(event.metadata.slot, 500);
        assert_eq!(event.liquidation_data.asset_amount, 5000000);
        assert_eq!(event.liquidation_data.liab_amount, 4500000);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_marginfi_liquidation_event_with_transfer_data() {
        let params = EventParameters::default();
        let liquidation_data = MarginFiLiquidationData::default();

        let transfer_data = vec![
            TransferData {
                source: Pubkey::new_unique(),
                destination: Pubkey::new_unique(),
                amount: 3000000,
                mint: Some(Pubkey::new_unique()),
            },
            TransferData {
                source: Pubkey::new_unique(),
                destination: Pubkey::new_unique(),
                amount: 2700000,
                mint: Some(Pubkey::new_unique()),
            },
        ];

        let event = MarginFiLiquidationEvent::new(params, liquidation_data)
            .with_transfer_data(transfer_data.clone());

        assert_eq!(event.transfer_data.len(), 2);
        assert_eq!(event.transfer_data[0].amount, 3000000);
        assert_eq!(event.transfer_data[1].amount, 2700000);
    }

    #[test]
    fn test_marginfi_event_parser_default() {
        let parser = MarginFiEventParser::default();

        // Test that parser contains expected program IDs
        let program_ids = parser.supported_program_ids();
        assert_eq!(program_ids.len(), 2);
        assert!(program_ids.contains(&marginfi_program_id()));
        assert!(program_ids.contains(&marginfi_bank_program_id()));

        // Test should_handle method
        assert!(parser.should_handle(&marginfi_program_id()));
        assert!(parser.should_handle(&marginfi_bank_program_id()));
        assert!(!parser.should_handle(&Pubkey::new_unique()));

        // Test configs are populated
        let inner_configs = parser.inner_instruction_configs();
        let instruction_configs = parser.instruction_configs();

        assert!(!inner_configs.is_empty());
        assert!(!instruction_configs.is_empty());

        // Test specific discriminators are present
        assert!(inner_configs.contains_key("lendingAccountDeposit"));
        assert!(inner_configs.contains_key("lendingAccountWithdraw"));
        assert!(inner_configs.contains_key("lendingAccountBorrow"));
        assert!(inner_configs.contains_key("lendingAccountRepay"));
        assert!(inner_configs.contains_key("lendingAccountLiquidate"));

        assert!(instruction_configs.contains_key(&MARGINFI_DEPOSIT_DISCRIMINATOR.to_vec()));
        assert!(instruction_configs.contains_key(&MARGINFI_WITHDRAW_DISCRIMINATOR.to_vec()));
        assert!(instruction_configs.contains_key(&MARGINFI_BORROW_DISCRIMINATOR.to_vec()));
        assert!(instruction_configs.contains_key(&MARGINFI_REPAY_DISCRIMINATOR.to_vec()));
        assert!(instruction_configs.contains_key(&MARGINFI_LIQUIDATE_DISCRIMINATOR.to_vec()));
    }

    #[test]
    fn test_program_id_functions() {
        let main_program_id = marginfi_program_id();
        let bank_program_id = marginfi_bank_program_id();

        // Test that program IDs are different
        assert_ne!(main_program_id, bank_program_id);

        // Test is_marginfi_program function
        assert!(is_marginfi_program(&main_program_id));
        assert!(is_marginfi_program(&bank_program_id));
        assert!(!is_marginfi_program(&Pubkey::new_unique()));
    }

    #[test]
    fn test_event_trait_implementations() {
        // Test deposit event
        let mut deposit_event = MarginFiDepositEvent::default();
        let test_id = "test_deposit_id";
        let test_kind = EventKind::Transfer;

        // Note: EventMetadata doesn't have set_id/set_kind methods
        // These would need to be set during metadata creation

        // Create new metadata with proper values
        deposit_event.metadata.core = riglr_events_core::EventMetadata::new(
            test_id.to_string(),
            test_kind.clone(),
            "marginfi-test".to_string(),
        );

        assert_eq!(deposit_event.id(), test_id);
        assert_eq!(deposit_event.kind(), &test_kind);
        assert!(!deposit_event.metadata().id.is_empty());

        // Test clone_boxed
        let boxed_event = deposit_event.clone_boxed();
        assert_eq!(boxed_event.id(), test_id);

        // Test as_any
        let any_event = deposit_event.as_any();
        assert!(any_event.downcast_ref::<MarginFiDepositEvent>().is_some());

        // Test withdraw event
        let mut withdraw_event = MarginFiWithdrawEvent::default();
        withdraw_event.metadata.core = riglr_events_core::EventMetadata::new(
            "test_withdraw_id".to_string(),
            EventKind::Transfer,
            "marginfi-test".to_string(),
        );
        assert_eq!(withdraw_event.id(), "test_withdraw_id");

        // Test borrow event
        let mut borrow_event = MarginFiBorrowEvent::default();
        borrow_event.metadata.core = riglr_events_core::EventMetadata::new(
            "test_borrow_id".to_string(),
            EventKind::Transfer,
            "marginfi-test".to_string(),
        );
        assert_eq!(borrow_event.id(), "test_borrow_id");

        // Test repay event
        let mut repay_event = MarginFiRepayEvent::default();
        repay_event.metadata.core = riglr_events_core::EventMetadata::new(
            "test_repay_id".to_string(),
            EventKind::Transfer,
            "marginfi-test".to_string(),
        );
        assert_eq!(repay_event.id(), "test_repay_id");

        // Test liquidation event
        let mut liquidation_event = MarginFiLiquidationEvent::default();
        liquidation_event.metadata.core = riglr_events_core::EventMetadata::new(
            "test_liquidation_id".to_string(),
            EventKind::Transfer,
            "marginfi-test".to_string(),
        );
        assert_eq!(liquidation_event.id(), "test_liquidation_id");
    }

    #[test]
    fn test_event_serialization() {
        let params = EventParameters::new(
            "serialize_test".to_string(),
            "sig_serialize".to_string(),
            999,
            1640995999,
            1640995999000,
            1640995999999,
            "serialize".to_string(),
        );

        let deposit_data = MarginFiDepositData {
            amount: 1000000,
            ..Default::default()
        };

        let event = MarginFiDepositEvent::new(params, deposit_data);

        // Test to_json method
        let json_result = event.to_json();
        assert!(json_result.is_ok());

        let json_value = json_result.unwrap();
        assert!(json_value.is_object());

        // Verify some key fields are present in JSON
        let obj = json_value.as_object().unwrap();
        assert!(obj.contains_key("id"));
        assert!(obj.contains_key("signature"));
        assert!(obj.contains_key("slot"));
        assert!(obj.contains_key("deposit_data"));

        // Test that metadata field is skipped (not serialized)
        assert!(!obj.contains_key("metadata"));
    }

    #[test]
    fn test_discriminators_consistency() {
        // Test that discriminator constants are correct length
        assert_eq!(MARGINFI_DEPOSIT_DISCRIMINATOR.len(), 8);
        assert_eq!(MARGINFI_WITHDRAW_DISCRIMINATOR.len(), 8);
        assert_eq!(MARGINFI_BORROW_DISCRIMINATOR.len(), 8);
        assert_eq!(MARGINFI_REPAY_DISCRIMINATOR.len(), 8);
        assert_eq!(MARGINFI_LIQUIDATE_DISCRIMINATOR.len(), 8);

        // Test that discriminators are unique
        let discriminators = vec![
            MARGINFI_DEPOSIT_DISCRIMINATOR,
            MARGINFI_WITHDRAW_DISCRIMINATOR,
            MARGINFI_BORROW_DISCRIMINATOR,
            MARGINFI_REPAY_DISCRIMINATOR,
            MARGINFI_LIQUIDATE_DISCRIMINATOR,
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
    fn test_utility_functions_with_edge_cases() {
        // Test health ratio calculations
        assert_eq!(calculate_health_ratio(0, 0, 5000), f64::INFINITY);
        assert_eq!(calculate_health_ratio(1000, 0, 5000), f64::INFINITY);

        let ratio = calculate_health_ratio(2000, 1000, 5000);
        assert!((ratio - 1.0).abs() < f64::EPSILON);

        // Test liquidation threshold
        assert_eq!(calculate_liquidation_threshold(0, 8000), 0);
        assert_eq!(calculate_liquidation_threshold(10000, 8000), 8000);

        // Test shares conversion
        assert_eq!(shares_to_amount(0, 1000), 0);
        assert_eq!(amount_to_shares(0, 1000), 0);
        assert_eq!(amount_to_shares(100, 0), 100);

        // Test interest rate calculation
        let rate = calculate_interest_rate(5000, 200, 800, 2000, 8000);
        assert!(rate >= 200); // Should be at least base rate

        let rate_high = calculate_interest_rate(9000, 200, 800, 2000, 8000);
        assert!(rate_high > rate); // Higher utilization should mean higher rate
    }

    #[test]
    fn test_marginfi_account_type_enum() {
        // Test enum variants
        assert_eq!(
            MarginFiAccountType::MarginfiGroup,
            MarginFiAccountType::MarginfiGroup
        );
        assert_ne!(
            MarginFiAccountType::MarginfiGroup,
            MarginFiAccountType::MarginfiAccount
        );
        assert_ne!(MarginFiAccountType::Bank, MarginFiAccountType::Unknown);

        // Test serialization/deserialization
        let account_type = MarginFiAccountType::Bank;
        let serialized = serde_json::to_string(&account_type).unwrap();
        let deserialized: MarginFiAccountType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(account_type, deserialized);
    }

    #[test]
    fn test_complex_data_structures() {
        // Test MarginFiBankConfig with all fields
        let bank_config = MarginFiBankConfig {
            bank: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            vault: Pubkey::new_unique(),
            oracle: Pubkey::new_unique(),
            bank_authority: Pubkey::new_unique(),
            collected_insurance_fees_outstanding: 1000,
            fee_rate: 100,
            insurance_fee_rate: 50,
            insurance_vault: Pubkey::new_unique(),
            deposit_limit: 1000000000,
            borrow_limit: 800000000,
            operational_state: 1,
            oracle_setup: 2,
            oracle_keys: [Pubkey::new_unique(); 5],
        };

        assert_eq!(bank_config.fee_rate, 100);
        assert_eq!(bank_config.oracle_keys.len(), 5);

        // Test MarginFiBalance
        let balance = MarginFiBalance {
            active: true,
            bank_pk: Pubkey::new_unique(),
            asset_shares: 1000000,
            liability_shares: 500000,
            emissions_outstanding: 100,
            last_update: 1640995200,
            padding: [0; 1],
        };

        assert!(balance.active);
        assert_eq!(balance.asset_shares, 1000000);
        assert_eq!(balance.liability_shares, 500000);
    }

    #[test]
    fn test_empty_transfer_data_operations() {
        let params = EventParameters::default();
        let deposit_data = MarginFiDepositData::default();

        // Test with empty transfer data
        let event = MarginFiDepositEvent::new(params, deposit_data).with_transfer_data(vec![]);

        assert!(event.transfer_data.is_empty());

        // Test multiple with_transfer_data calls (should replace, not append)
        let transfer1 = vec![TransferData {
            source: Pubkey::new_unique(),
            destination: Pubkey::new_unique(),
            amount: 100,
            mint: Some(Pubkey::new_unique()),
        }];

        let transfer2 = vec![TransferData {
            source: Pubkey::new_unique(),
            destination: Pubkey::new_unique(),
            amount: 200,
            mint: Some(Pubkey::new_unique()),
        }];

        let params2 = EventParameters::default();
        let deposit_data2 = MarginFiDepositData::default();

        let event = MarginFiDepositEvent::new(params2, deposit_data2)
            .with_transfer_data(transfer1)
            .with_transfer_data(transfer2);

        assert_eq!(event.transfer_data.len(), 1);
        assert_eq!(event.transfer_data[0].amount, 200);
    }
}
