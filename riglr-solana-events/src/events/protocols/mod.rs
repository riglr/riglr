/// BONK token protocol events and parsers
pub mod bonk;
/// PumpSwap DEX protocol events and parsers
pub mod pumpswap;
/// Raydium AMM v4 protocol events and parsers
pub mod raydium_amm_v4;
/// Raydium Concentrated Liquidity Market Maker events and parsers
pub mod raydium_clmm;
/// Raydium Constant Product Market Maker events and parsers
pub mod raydium_cpmm;

// New protocol modules we'll add
/// Jupiter aggregator protocol events and parsers
pub mod jupiter;
/// MarginFi lending protocol events and parsers
pub mod marginfi;
/// Meteora protocol events and parsers
pub mod meteora;
/// Orca DEX protocol events and parsers
pub mod orca;

// Re-export specific types to avoid conflicts
pub use bonk::{events as bonk_events, parser as bonk_parser, types as bonk_types};
pub use jupiter::{events as jupiter_events, parser as jupiter_parser};
pub use marginfi::{events as marginfi_events, parser as marginfi_parser, types as marginfi_types};
pub use meteora::{events as meteora_events, parser as meteora_parser};
pub use orca::{events as orca_events, parser as orca_parser};
pub use pumpswap::{events as pumpswap_events, parser as pumpswap_parser};
pub use raydium_amm_v4::{
    discriminators as raydium_v4_discriminators, events as raydium_v4_events,
    parser as raydium_v4_parser,
};
pub use raydium_clmm::{
    discriminators as raydium_clmm_discriminators, events as raydium_clmm_events,
    parser as raydium_clmm_parser,
};
pub use raydium_cpmm::{
    discriminators as raydium_cpmm_discriminators, events as raydium_cpmm_events,
    parser as raydium_cpmm_parser,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bonk_module_exports_when_accessed_should_be_available() {
        // Test that bonk module exports are accessible
        // This ensures the re-exports work correctly
        let _trade_event = std::any::type_name::<bonk_events::BonkTradeEvent>();
        let _pool_event = std::any::type_name::<bonk_events::BonkPoolCreateEvent>();
        let _parser = std::any::type_name::<bonk_parser::BonkEventParser>();
        let _direction = std::any::type_name::<bonk_types::TradeDirection>();
    }

    #[test]
    fn test_jupiter_module_exports_when_accessed_should_be_available() {
        // Test that jupiter module exports are accessible
        let _swap_event = std::any::type_name::<jupiter_events::JupiterSwapEvent>();
        let _liquidity_event = std::any::type_name::<jupiter_events::JupiterLiquidityEvent>();
        let _parser = std::any::type_name::<jupiter_parser::JupiterEventParser>();
    }

    #[test]
    fn test_marginfi_module_exports_when_accessed_should_be_available() {
        // Test that marginfi module exports are accessible
        let _deposit_event = std::any::type_name::<marginfi_events::MarginFiDepositEvent>();
        let _withdraw_event = std::any::type_name::<marginfi_events::MarginFiWithdrawEvent>();
        let _parser = std::any::type_name::<marginfi_parser::MarginFiEventParser>();
        let _account = std::any::type_name::<marginfi_types::MarginFiAccount>();
    }

    #[test]
    fn test_meteora_module_exports_when_accessed_should_be_available() {
        // Test that meteora module exports are accessible
        let _swap_event = std::any::type_name::<meteora_events::MeteoraSwapEvent>();
        let _liquidity_event = std::any::type_name::<meteora_events::MeteoraLiquidityEvent>();
        let _parser = std::any::type_name::<meteora_parser::MeteoraEventParser>();
    }

    #[test]
    fn test_orca_module_exports_when_accessed_should_be_available() {
        // Test that orca module exports are accessible
        let _swap_event = std::any::type_name::<orca_events::OrcaSwapEvent>();
        let _position_event = std::any::type_name::<orca_events::OrcaPositionEvent>();
        let _parser = std::any::type_name::<orca_parser::OrcaEventParser>();
    }

    #[test]
    fn test_pumpswap_module_exports_when_accessed_should_be_available() {
        // Test that pumpswap module exports are accessible
        let _buy_event = std::any::type_name::<pumpswap_events::PumpSwapBuyEvent>();
        let _sell_event = std::any::type_name::<pumpswap_events::PumpSwapSellEvent>();
        let _parser = std::any::type_name::<pumpswap_parser::PumpSwapEventParser>();
    }

    #[test]
    fn test_raydium_v4_module_exports_when_accessed_should_be_available() {
        // Test that raydium AMM v4 module exports are accessible
        let _swap_event = std::any::type_name::<raydium_v4_events::RaydiumAmmV4SwapEvent>();
        let _deposit_event = std::any::type_name::<raydium_v4_events::RaydiumAmmV4DepositEvent>();
        let _parser = std::any::type_name::<raydium_v4_parser::RaydiumAmmV4EventParser>();
    }

    #[test]
    fn test_raydium_clmm_module_exports_when_accessed_should_be_available() {
        // Test that raydium CLMM module exports are accessible
        let _swap_event = std::any::type_name::<raydium_clmm_events::RaydiumClmmSwapEvent>();
        let _create_pool_event =
            std::any::type_name::<raydium_clmm_events::RaydiumClmmCreatePoolEvent>();
        let _parser = std::any::type_name::<raydium_clmm_parser::RaydiumClmmEventParser>();
    }

    #[test]
    fn test_raydium_cpmm_module_exports_when_accessed_should_be_available() {
        // Test that raydium CPMM module exports are accessible
        let _swap_event = std::any::type_name::<raydium_cpmm_events::RaydiumCpmmSwapEvent>();
        let _deposit_event = std::any::type_name::<raydium_cpmm_events::RaydiumCpmmDepositEvent>();
        let _parser = std::any::type_name::<raydium_cpmm_parser::RaydiumCpmmEventParser>();
    }

    #[test]
    fn test_module_compilation_when_imported_should_succeed() {
        // This test verifies that all module declarations are valid
        // and can be compiled without errors. The fact that this test
        // compiles means all the module paths are correct.
    }

    #[test]
    fn test_re_exports_when_used_should_not_conflict() {
        // Test that re-exports don't create naming conflicts
        // by accessing types with their aliased names
        let _bonk_trade_event = std::any::type_name::<bonk_events::BonkTradeEvent>();
        let _jupiter_swap_event = std::any::type_name::<jupiter_events::JupiterSwapEvent>();
        let _marginfi_deposit_event =
            std::any::type_name::<marginfi_events::MarginFiDepositEvent>();
        let _meteora_swap_event = std::any::type_name::<meteora_events::MeteoraSwapEvent>();
        let _orca_swap_event = std::any::type_name::<orca_events::OrcaSwapEvent>();
        let _pumpswap_buy_event = std::any::type_name::<pumpswap_events::PumpSwapBuyEvent>();

        // Test that we can access multiple raydium variants without conflict
        let _v4_swap_event = std::any::type_name::<raydium_v4_events::RaydiumAmmV4SwapEvent>();
        let _clmm_swap_event = std::any::type_name::<raydium_clmm_events::RaydiumClmmSwapEvent>();
        let _cpmm_swap_event = std::any::type_name::<raydium_cpmm_events::RaydiumCpmmSwapEvent>();
    }
}
