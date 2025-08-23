/// BONK protocol event definitions and constants.
pub mod events;
/// BONK protocol transaction and log parsing functionality.
pub mod parser;
/// BONK protocol data types and structures.
pub mod types;

pub use events::*;
pub use parser::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::core::traits::EventParser;

    #[test]
    fn test_module_exports_events() {
        // Test that events module exports are accessible
        // This tests the pub use events::* re-export
        use crate::events::protocols::bonk::events::{BonkPoolCreateEvent, BonkTradeEvent};

        // Verify we can create default instances of exported event types
        let trade_event = BonkTradeEvent::default();
        let pool_create_event = BonkPoolCreateEvent::default();

        // Basic structural validation
        assert_eq!(trade_event.amount_in, 0);
        assert_eq!(
            pool_create_event.pool_state,
            solana_sdk::pubkey::Pubkey::default()
        );
    }

    #[test]
    fn test_module_exports_parser() {
        // Test that parser module exports are accessible
        // This tests the pub use parser::* re-export
        use crate::events::protocols::bonk::parser::BonkEventParser;

        // Verify we can create parser instance
        let parser = BonkEventParser::new();
        let default_parser = BonkEventParser::default();

        // Verify parser has expected program IDs
        let supported_ids = parser.supported_program_ids();
        assert!(!supported_ids.is_empty());
        assert_eq!(supported_ids.len(), 1);

        let default_supported_ids = default_parser.supported_program_ids();
        assert_eq!(supported_ids, default_supported_ids);
    }

    #[test]
    fn test_module_exports_types() {
        // Test that types module exports are accessible
        // This tests the pub use types::* re-export
        use crate::events::protocols::bonk::types::{
            ConstantCurve, CurveParams, FixedCurve, LinearCurve, MintParams, PoolStatus,
            TradeDirection, VestingParams,
        };

        // Test TradeDirection enum
        let buy_direction = TradeDirection::Buy;
        let sell_direction = TradeDirection::Sell;
        let default_direction = TradeDirection::default();

        assert_eq!(buy_direction, TradeDirection::Buy);
        assert_eq!(sell_direction, TradeDirection::Sell);
        assert_eq!(default_direction, TradeDirection::Buy);
        assert_ne!(buy_direction, sell_direction);

        // Test PoolStatus enum
        let fund_status = PoolStatus::Fund;
        let migrate_status = PoolStatus::Migrate;
        let trade_status = PoolStatus::Trade;
        let default_status = PoolStatus::default();

        assert_eq!(fund_status, PoolStatus::Fund);
        assert_eq!(migrate_status, PoolStatus::Migrate);
        assert_eq!(trade_status, PoolStatus::Trade);
        assert_eq!(default_status, PoolStatus::Fund);
        assert_ne!(fund_status, migrate_status);
        assert_ne!(migrate_status, trade_status);

        // Test struct creation
        let mint_params = MintParams::default();
        let vesting_params = VestingParams::default();
        let constant_curve = ConstantCurve::default();
        let fixed_curve = FixedCurve::default();
        let linear_curve = LinearCurve::default();
        let curve_params = CurveParams::default();

        // Verify default values
        assert_eq!(mint_params.decimals, 0);
        assert_eq!(mint_params.name, "");
        assert_eq!(mint_params.symbol, "");
        assert_eq!(mint_params.uri, "");

        assert_eq!(vesting_params.total_locked_amount, 0);
        assert_eq!(vesting_params.cliff_period, 0);
        assert_eq!(vesting_params.unlock_period, 0);

        assert_eq!(constant_curve.supply, 0);
        assert_eq!(constant_curve.total_base_sell, 0);
        assert_eq!(constant_curve.total_quote_fund_raising, 0);
        assert_eq!(constant_curve.migrate_type, 0);

        assert_eq!(fixed_curve.supply, 0);
        assert_eq!(fixed_curve.total_quote_fund_raising, 0);
        assert_eq!(fixed_curve.migrate_type, 0);

        assert_eq!(linear_curve.supply, 0);
        assert_eq!(linear_curve.total_quote_fund_raising, 0);
        assert_eq!(linear_curve.migrate_type, 0);

        assert!(curve_params.constant_curve.is_none());
        assert!(curve_params.fixed_curve.is_none());
        assert!(curve_params.linear_curve.is_none());
    }

    #[test]
    fn test_module_exports_discriminators() {
        // Test that discriminators are accessible through events module
        use crate::events::protocols::bonk::events::discriminators;

        // Test string discriminators
        assert_eq!(discriminators::TRADE_EVENT, "bonk_trade_event");
        assert_eq!(discriminators::POOL_CREATE_EVENT, "bonk_pool_create_event");

        // Test byte discriminators
        assert_eq!(discriminators::TRADE_EVENT_BYTES.len(), 16);
        assert_eq!(discriminators::POOL_CREATE_EVENT_BYTES.len(), 16);

        // Test instruction discriminators
        assert_eq!(discriminators::BUY_EXACT_IN_IX.len(), 8);
        assert_eq!(discriminators::BUY_EXACT_OUT_IX.len(), 8);
        assert_eq!(discriminators::SELL_EXACT_IN_IX.len(), 8);
        assert_eq!(discriminators::SELL_EXACT_OUT_IX.len(), 8);
        assert_eq!(discriminators::INITIALIZE_IX.len(), 8);
        assert_eq!(discriminators::MIGRATE_TO_AMM_IX.len(), 8);
        assert_eq!(discriminators::MIGRATE_TO_CPSWAP_IX.len(), 8);

        // Verify discriminators are unique
        let trade_bytes = discriminators::TRADE_EVENT_BYTES;
        let pool_create_bytes = discriminators::POOL_CREATE_EVENT_BYTES;
        assert_ne!(trade_bytes, pool_create_bytes);

        // Verify instruction discriminators are unique
        let buy_exact_in = discriminators::BUY_EXACT_IN_IX;
        let buy_exact_out = discriminators::BUY_EXACT_OUT_IX;
        let sell_exact_in = discriminators::SELL_EXACT_IN_IX;
        let sell_exact_out = discriminators::SELL_EXACT_OUT_IX;
        let initialize = discriminators::INITIALIZE_IX;
        let migrate_amm = discriminators::MIGRATE_TO_AMM_IX;
        let migrate_cpswap = discriminators::MIGRATE_TO_CPSWAP_IX;

        assert_ne!(buy_exact_in, buy_exact_out);
        assert_ne!(buy_exact_in, sell_exact_in);
        assert_ne!(buy_exact_in, sell_exact_out);
        assert_ne!(buy_exact_in, initialize);
        assert_ne!(buy_exact_in, migrate_amm);
        assert_ne!(buy_exact_in, migrate_cpswap);

        assert_ne!(buy_exact_out, sell_exact_in);
        assert_ne!(buy_exact_out, sell_exact_out);
        assert_ne!(buy_exact_out, initialize);
        assert_ne!(buy_exact_out, migrate_amm);
        assert_ne!(buy_exact_out, migrate_cpswap);

        assert_ne!(sell_exact_in, sell_exact_out);
        assert_ne!(sell_exact_in, initialize);
        assert_ne!(sell_exact_in, migrate_amm);
        assert_ne!(sell_exact_in, migrate_cpswap);

        assert_ne!(sell_exact_out, initialize);
        assert_ne!(sell_exact_out, migrate_amm);
        assert_ne!(sell_exact_out, migrate_cpswap);

        assert_ne!(initialize, migrate_amm);
        assert_ne!(initialize, migrate_cpswap);

        assert_ne!(migrate_amm, migrate_cpswap);
    }

    #[test]
    fn test_module_exports_program_id() {
        // Test that BONK_PROGRAM_ID is accessible through parser module
        use crate::events::protocols::bonk::parser::BONK_PROGRAM_ID;

        // Verify it's a valid Pubkey
        assert_eq!(
            BONK_PROGRAM_ID,
            solana_sdk::pubkey!("bonksoHKfNJJ8Wo8ZJjpw7dHGePNxS2z2WE5GxUPdSo")
        );

        // Verify it's not the default/zero pubkey
        assert_ne!(BONK_PROGRAM_ID, solana_sdk::pubkey::Pubkey::default());
    }

    #[test]
    fn test_all_re_exports_accessible() {
        // Comprehensive test to ensure all re-exports work correctly
        // This tests that we can use types directly from the bonk module
        // without having to reference the submodules

        // From events module
        let _trade_event = BonkTradeEvent::default();
        let _pool_create_event = BonkPoolCreateEvent::default();

        // From parser module
        let _parser = BonkEventParser::new();
        let _program_id = BONK_PROGRAM_ID;

        // From types module
        let _trade_direction = TradeDirection::Buy;
        let _pool_status = PoolStatus::Fund;
        let _mint_params = MintParams::default();
        let _vesting_params = VestingParams::default();
        let _constant_curve = ConstantCurve::default();
        let _fixed_curve = FixedCurve::default();
        let _linear_curve = LinearCurve::default();
        let _curve_params = CurveParams::default();

        // If this test compiles and runs, all re-exports are working correctly
    }

    #[test]
    fn test_module_structure_completeness() {
        // Test that the module has the expected structure

        // Verify we can access nested modules
        use crate::events::protocols::bonk::{events, parser, types};

        // Verify module-level imports work
        use events::BonkTradeEvent;
        use parser::BonkEventParser;
        use types::TradeDirection;

        // Create instances to verify functionality
        let _event = BonkTradeEvent::default();
        let _parser = BonkEventParser::new();
        let _direction = TradeDirection::Buy;

        // This test passing indicates the module structure is correct
    }
}
