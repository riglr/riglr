/// Orca protocol event definitions and constants.
pub mod events;
/// Orca protocol transaction and log parsing functionality.
pub mod parser;
/// Orca protocol data types and structures.
pub mod types;

pub use events::*;
pub use parser::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::core::EventParser;
    use riglr_events_core::Event;
    use serde_json;
    use solana_sdk::pubkey::Pubkey;

    // Test that all public items are re-exported correctly
    #[test]
    fn test_events_module_reexports_event_parameters() {
        let params = EventParameters::default();
        assert_eq!(params.id, String::new());
        assert_eq!(params.signature, String::new());
        assert_eq!(params.slot, 0);
        assert_eq!(params.block_time, 0);
        assert_eq!(params.block_time_ms, 0);
        assert_eq!(params.program_received_time_ms, 0);
        assert_eq!(params.index, String::new());
    }

    #[test]
    fn test_events_module_reexports_orca_swap_event() {
        let params = EventParameters::new(
            "test_id".to_string(),
            "test_sig".to_string(),
            12345,
            1000,
            1000000,
            1000001,
            "0".to_string(),
        );
        let swap_data = OrcaSwapData::default();
        let event = OrcaSwapEvent::new(params, swap_data);

        assert_eq!(event.id(), "test_id");
        assert_eq!(event.metadata.signature, "test_sig");
        assert_eq!(event.metadata.slot, 12345);
        assert_eq!(event.metadata.program_received_time_ms, 1000001);
        assert_eq!(event.metadata.index, "0");
    }

    #[test]
    fn test_events_module_reexports_orca_position_event() {
        let params = EventParameters::new(
            "pos_id".to_string(),
            "pos_sig".to_string(),
            54321,
            2000,
            2000000,
            2000001,
            "1".to_string(),
        );
        let position_data = OrcaPositionData::default();
        let event = OrcaPositionEvent::new(params, position_data, true);

        assert_eq!(event.id(), "pos_id");
        assert_eq!(event.metadata.signature, "pos_sig");
        assert_eq!(event.metadata.slot, 54321);
        assert!(event.is_open);
    }

    #[test]
    fn test_events_module_reexports_orca_liquidity_event() {
        let params = EventParameters::new(
            "liq_id".to_string(),
            "liq_sig".to_string(),
            98765,
            3000,
            3000000,
            3000001,
            "2".to_string(),
        );
        let liquidity_data = OrcaLiquidityData::default();
        let event = OrcaLiquidityEvent::new(params, liquidity_data);

        assert_eq!(event.id(), "liq_id");
        assert_eq!(event.metadata.signature, "liq_sig");
        assert_eq!(event.metadata.slot, 98765);
    }

    #[test]
    fn test_parser_module_reexports_orca_event_parser() {
        let parser = OrcaEventParser::default();
        let program_ids = parser.supported_program_ids();
        assert!(!program_ids.is_empty());
        assert!(parser.should_handle(&orca_whirlpool_program_id()));
    }

    #[test]
    fn test_types_module_reexports_orca_whirlpool_program_id() {
        let program_id = orca_whirlpool_program_id();
        assert!(is_orca_whirlpool_program(&program_id));
    }

    #[test]
    fn test_types_module_reexports_discriminators() {
        // Test that discriminators are accessible
        assert_eq!(SWAP_DISCRIMINATOR.len(), 8);
        assert_eq!(OPEN_POSITION_DISCRIMINATOR.len(), 8);
        assert_eq!(CLOSE_POSITION_DISCRIMINATOR.len(), 8);
        assert_eq!(INCREASE_LIQUIDITY_DISCRIMINATOR.len(), 8);
        assert_eq!(DECREASE_LIQUIDITY_DISCRIMINATOR.len(), 8);
    }

    #[test]
    fn test_types_module_reexports_swap_direction() {
        let direction_a_to_b = SwapDirection::AtoB;
        let direction_b_to_a = SwapDirection::BtoA;

        assert_eq!(direction_a_to_b, SwapDirection::AtoB);
        assert_eq!(direction_b_to_a, SwapDirection::BtoA);
        assert_ne!(direction_a_to_b, direction_b_to_a);
    }

    #[test]
    fn test_types_module_reexports_orca_swap_data() {
        let swap_data = OrcaSwapData::default();
        assert_eq!(swap_data.whirlpool, Pubkey::default());
        assert_eq!(swap_data.amount, 0);
        assert!(!swap_data.amount_specified_is_input);
        assert!(!swap_data.a_to_b);
    }

    #[test]
    fn test_types_module_reexports_orca_position_data() {
        let position_data = OrcaPositionData::default();
        assert_eq!(position_data.whirlpool, Pubkey::default());
        assert_eq!(position_data.tick_lower_index, 0);
        assert_eq!(position_data.tick_upper_index, 0);
        assert_eq!(position_data.liquidity, 0);
    }

    #[test]
    fn test_types_module_reexports_orca_liquidity_data() {
        let liquidity_data = OrcaLiquidityData::default();
        assert_eq!(liquidity_data.whirlpool, Pubkey::default());
        assert_eq!(liquidity_data.liquidity_amount, 0);
        assert!(!liquidity_data.is_increase);
    }

    #[test]
    fn test_types_module_reexports_position_reward_info() {
        let reward_info = PositionRewardInfo::default();
        assert_eq!(reward_info.growth_inside_checkpoint, 0);
        assert_eq!(reward_info.amount_owed, 0);
    }

    #[test]
    fn test_types_module_reexports_whirlpool_account() {
        // Test that WhirlpoolAccount can be created (struct is public)
        let account = WhirlpoolAccount {
            whirlpools_config: Pubkey::default(),
            whirlpool_bump: [0],
            tick_spacing: 64,
            tick_spacing_seed: [0, 64],
            fee_rate: 300,
            protocol_fee_rate: 300,
            liquidity: 1000000,
            sqrt_price: 1000000000,
            tick_current_index: 0,
            protocol_fee_owed_a: 0,
            protocol_fee_owed_b: 0,
            token_mint_a: Pubkey::default(),
            token_vault_a: Pubkey::default(),
            fee_growth_global_a: 0,
            token_mint_b: Pubkey::default(),
            token_vault_b: Pubkey::default(),
            fee_growth_global_b: 0,
            reward_last_updated_timestamp: 0,
            reward_infos: [WhirlpoolRewardInfo {
                mint: Pubkey::default(),
                vault: Pubkey::default(),
                authority: Pubkey::default(),
                emissions_per_second_x64: 0,
                growth_global_x64: 0,
            }; 3],
        };

        assert_eq!(account.tick_spacing, 64);
        assert_eq!(account.fee_rate, 300);
    }

    #[test]
    fn test_types_module_reexports_whirlpool_reward_info() {
        let reward_info = WhirlpoolRewardInfo {
            mint: Pubkey::default(),
            vault: Pubkey::default(),
            authority: Pubkey::default(),
            emissions_per_second_x64: 1000,
            growth_global_x64: 2000,
        };

        assert_eq!(reward_info.emissions_per_second_x64, 1000);
        assert_eq!(reward_info.growth_global_x64, 2000);
    }

    #[test]
    fn test_types_module_reexports_utility_functions() {
        // Test tick_index_to_price function
        let price = tick_index_to_price(0);
        assert_eq!(price, 1.0);

        let price_positive = tick_index_to_price(100);
        assert!(price_positive > 1.0);

        let price_negative = tick_index_to_price(-100);
        assert!(price_negative < 1.0);
    }

    #[test]
    fn test_types_module_reexports_sqrt_price_to_price() {
        // Test sqrt_price_to_price function with standard values
        let sqrt_price = 1u128 << 64; // Square root of 1
        let price = sqrt_price_to_price(sqrt_price, 6, 6);
        assert_eq!(price, 1.0);

        // Test with different decimal values
        let price_different_decimals = sqrt_price_to_price(sqrt_price, 9, 6);
        assert_eq!(price_different_decimals, 1000.0);

        let price_reverse_decimals = sqrt_price_to_price(sqrt_price, 6, 9);
        assert_eq!(price_reverse_decimals, 0.001);
    }

    #[test]
    fn test_all_event_types_implement_event_trait() {
        // Test OrcaSwapEvent implements Event trait
        let params = EventParameters::default();
        let swap_data = OrcaSwapData::default();
        let mut swap_event = OrcaSwapEvent::new(params.clone(), swap_data);

        // Test trait methods
        let _id = swap_event.id();
        let _kind = swap_event.kind();
        let _metadata = swap_event.metadata();
        let _metadata_mut = swap_event.metadata_mut();
        let _as_any = swap_event.as_any();
        let _as_any_mut = swap_event.as_any_mut();
        let _cloned = swap_event.clone_boxed();
        let _json_result = swap_event.to_json();

        // Test OrcaPositionEvent implements Event trait
        let position_data = OrcaPositionData::default();
        let mut position_event = OrcaPositionEvent::new(params.clone(), position_data, true);

        let _id = position_event.id();
        let _kind = position_event.kind();
        let _metadata = position_event.metadata();
        let _metadata_mut = position_event.metadata_mut();
        let _as_any = position_event.as_any();
        let _as_any_mut = position_event.as_any_mut();
        let _cloned = position_event.clone_boxed();
        let _json_result = position_event.to_json();

        // Test OrcaLiquidityEvent implements Event trait
        let liquidity_data = OrcaLiquidityData::default();
        let mut liquidity_event = OrcaLiquidityEvent::new(params, liquidity_data);

        let _id = liquidity_event.id();
        let _kind = liquidity_event.kind();
        let _metadata = liquidity_event.metadata();
        let _metadata_mut = liquidity_event.metadata_mut();
        let _as_any = liquidity_event.as_any();
        let _as_any_mut = liquidity_event.as_any_mut();
        let _cloned = liquidity_event.clone_boxed();
        let _json_result = liquidity_event.to_json();
    }

    #[test]
    fn test_event_serialization_works() {
        let params = EventParameters::new(
            "test_id".to_string(),
            "test_signature".to_string(),
            12345,
            1000,
            1000000,
            1000001,
            "0".to_string(),
        );

        // Test OrcaSwapEvent serialization
        let swap_data = OrcaSwapData::default();
        let swap_event = OrcaSwapEvent::new(params.clone(), swap_data);
        let swap_json = serde_json::to_string(&swap_event);
        assert!(swap_json.is_ok());

        // Test OrcaPositionEvent serialization
        let position_data = OrcaPositionData::default();
        let position_event = OrcaPositionEvent::new(params.clone(), position_data, true);
        let position_json = serde_json::to_string(&position_event);
        assert!(position_json.is_ok());

        // Test OrcaLiquidityEvent serialization
        let liquidity_data = OrcaLiquidityData::default();
        let liquidity_event = OrcaLiquidityEvent::new(params, liquidity_data);
        let liquidity_json = serde_json::to_string(&liquidity_event);
        assert!(liquidity_json.is_ok());
    }

    #[test]
    fn test_event_deserialization_works() {
        // Test that we can deserialize events from JSON
        let swap_json = r#"{"id":"test","signature":"sig","slot":1,"block_time":1000,"block_time_ms":1000000,"program_received_time_ms":1000001,"program_handle_time_consuming_ms":0,"index":"0","swap_data":{"whirlpool":"11111111111111111111111111111112","user":"11111111111111111111111111111112","token_mint_a":"11111111111111111111111111111112","token_mint_b":"11111111111111111111111111111112","token_vault_a":"11111111111111111111111111111112","token_vault_b":"11111111111111111111111111111112","amount":0,"amount_specified_is_input":false,"a_to_b":false,"sqrt_price_limit":0,"amount_in":0,"amount_out":0,"fee_amount":0,"tick_current_index":0,"sqrt_price":0,"liquidity":0},"transfer_data":[]}"#;
        let swap_event: Result<OrcaSwapEvent, _> = serde_json::from_str(swap_json);
        assert!(swap_event.is_ok());

        let position_json = r#"{"id":"test","signature":"sig","slot":1,"block_time":1000,"block_time_ms":1000000,"program_received_time_ms":1000001,"program_handle_time_consuming_ms":0,"index":"0","position_data":{"whirlpool":"11111111111111111111111111111112","position_mint":"11111111111111111111111111111112","position":"11111111111111111111111111111112","position_token_account":"11111111111111111111111111111112","position_authority":"11111111111111111111111111111112","tick_lower_index":0,"tick_upper_index":0,"liquidity":0,"fee_growth_checkpoint_a":0,"fee_growth_checkpoint_b":0,"fee_owed_a":0,"fee_owed_b":0,"reward_infos":[{"growth_inside_checkpoint":0,"amount_owed":0},{"growth_inside_checkpoint":0,"amount_owed":0},{"growth_inside_checkpoint":0,"amount_owed":0}]},"is_open":true,"transfer_data":[]}"#;
        let position_event: Result<OrcaPositionEvent, _> = serde_json::from_str(position_json);
        assert!(position_event.is_ok());

        let liquidity_json = r#"{"id":"test","signature":"sig","slot":1,"block_time":1000,"block_time_ms":1000000,"program_received_time_ms":1000001,"program_handle_time_consuming_ms":0,"index":"0","liquidity_data":{"whirlpool":"11111111111111111111111111111112","position":"11111111111111111111111111111112","position_authority":"11111111111111111111111111111112","token_mint_a":"11111111111111111111111111111112","token_mint_b":"11111111111111111111111111111112","token_vault_a":"11111111111111111111111111111112","token_vault_b":"11111111111111111111111111111112","tick_lower_index":0,"tick_upper_index":0,"liquidity_amount":0,"token_max_a":0,"token_max_b":0,"token_actual_a":0,"token_actual_b":0,"is_increase":false},"transfer_data":[]}"#;
        let liquidity_event: Result<OrcaLiquidityEvent, _> = serde_json::from_str(liquidity_json);
        assert!(liquidity_event.is_ok());
    }

    #[test]
    fn test_module_constants_are_accessible() {
        // Test that all constants from types module are accessible
        assert_eq!(
            ORCA_WHIRLPOOL_PROGRAM_ID,
            "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc"
        );

        // Test discriminators have correct values and lengths
        assert_eq!(
            SWAP_DISCRIMINATOR,
            [0xf8, 0xc6, 0x9e, 0x91, 0xe1, 0x75, 0x87, 0xc8]
        );
        assert_eq!(
            OPEN_POSITION_DISCRIMINATOR,
            [0x87, 0x80, 0x2f, 0x4d, 0x0f, 0x98, 0xf0, 0x31]
        );
        assert_eq!(
            CLOSE_POSITION_DISCRIMINATOR,
            [0x7b, 0x86, 0x51, 0x00, 0x31, 0x44, 0x62, 0x62]
        );
        assert_eq!(
            INCREASE_LIQUIDITY_DISCRIMINATOR,
            [0x2e, 0x9c, 0xf3, 0x76, 0x0d, 0xcd, 0xfb, 0xb2]
        );
        assert_eq!(
            DECREASE_LIQUIDITY_DISCRIMINATOR,
            [0xa0, 0x26, 0xd0, 0x6f, 0x68, 0x5b, 0x2c, 0x01]
        );
    }

    #[test]
    fn test_edge_cases_for_utility_functions() {
        // Test edge cases for tick_index_to_price
        let price_max = tick_index_to_price(i32::MAX);
        assert!(price_max.is_finite());

        let price_min = tick_index_to_price(i32::MIN);
        assert!(price_min.is_finite());
        assert!(price_min > 0.0);

        // Test edge cases for sqrt_price_to_price
        let zero_sqrt_price = sqrt_price_to_price(0, 6, 6);
        assert_eq!(zero_sqrt_price, 0.0);

        let max_sqrt_price = sqrt_price_to_price(u128::MAX, 0, 0);
        assert!(max_sqrt_price.is_finite());
    }

    #[test]
    fn test_all_structs_implement_required_traits() {
        // Test that all data structs implement Debug, Clone, Serialize, Deserialize
        let swap_data = OrcaSwapData::default();
        let _cloned_swap = swap_data.clone();
        let _debug_swap = format!("{:?}", swap_data);

        let position_data = OrcaPositionData::default();
        let _cloned_position = position_data.clone();
        let _debug_position = format!("{:?}", position_data);

        let liquidity_data = OrcaLiquidityData::default();
        let _cloned_liquidity = liquidity_data.clone();
        let _debug_liquidity = format!("{:?}", liquidity_data);

        let reward_info = PositionRewardInfo::default();
        let _cloned_reward = reward_info.clone();
        let _debug_reward = format!("{:?}", reward_info);

        let swap_direction = SwapDirection::AtoB;
        let _cloned_direction = swap_direction.clone();
        let _debug_direction = format!("{:?}", swap_direction);
    }
}
