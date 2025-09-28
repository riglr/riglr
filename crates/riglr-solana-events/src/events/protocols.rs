//! Protocol-specific event parsers and types for various Solana DeFi protocols.
//!
//! This module contains implementations for parsing events from different DeFi protocols
//! on Solana including DEXs, lending protocols, and other DeFi applications.

/// BONK token protocol events and parsers
pub mod bonk;
/// Jupiter aggregator protocol events and parsers
pub mod jupiter;
/// `MarginFi` lending protocol events and parsers
pub mod marginfi;
/// Meteora protocol events and parsers
pub mod meteora;
/// Orca DEX protocol events and parsers
pub mod orca;
/// `PumpSwap` DEX protocol events and parsers
pub mod pumpswap;
/// Raydium AMM v4 protocol events and parsers
pub mod raydium_amm_v4;
/// Raydium Concentrated Liquidity Market Maker events and parsers
pub mod raydium_clmm;
/// Raydium Constant Product Market Maker events and parsers
pub mod raydium_cpmm;

// Create module aliases for backward compatibility - these must come before pub use statements
pub mod jupiter_events {
    pub use super::jupiter::{
        LiquidityEvent as JupiterLiquidityEvent, SwapBorshEvent as JupiterSwapBorshEvent,
        SwapEvent as JupiterSwapEvent,
    };
}

pub mod jupiter_parser {
    pub use super::jupiter::EventParser as JupiterEventParser;
}

pub mod marginfi_events {
    pub use super::marginfi::{
        MarginFiBorrowEvent, MarginFiDepositEvent, MarginFiLiquidationEvent, MarginFiRepayEvent,
        MarginFiWithdrawEvent,
    };
}

pub mod marginfi_parser {
    pub use super::marginfi::MarginFiEventParser;
}

pub mod marginfi_types {
    pub use super::marginfi::{
        MarginFiAccount, MarginFiAccountType, MarginFiBalance, MarginFiBankConfig,
        MarginFiBankState, MarginFiBorrowData, MarginFiDepositData, MarginFiLiquidationData,
        MarginFiRepayData, MarginFiWithdrawData,
    };
}

pub mod raydium_v4_discriminators {
    pub use super::raydium_amm_v4::{
        DEPOSIT, INITIALIZE2, SWAP_BASE_IN, SWAP_BASE_OUT, WITHDRAW, WITHDRAW_PNL,
    };
}

pub mod raydium_cpmm_events {
    pub use super::raydium_cpmm::{
        DepositEvent as RaydiumCpmmDepositEvent, SwapEvent as RaydiumCpmmSwapEvent,
    };
}

pub mod raydium_cpmm_parser {
    pub use super::raydium_cpmm::EventParser as RaydiumCpmmEventParser;
}

// Missing alias modules to fix test imports
pub mod bonk_events {
    pub use super::bonk::events::{
        PoolCreateEvent as BonkPoolCreateEvent, TradeEvent as BonkTradeEvent,
    };
}

pub mod bonk_parser {
    pub use super::bonk::parser::Parser as BonkEventParser;
}

pub mod meteora_events {
    pub use super::meteora::events::{
        LiquidityEvent as MeteoraLiquidityEvent, SwapEvent as MeteoraSwapEvent,
    };
}

pub mod meteora_parser {
    pub use super::meteora::parser::Parser as MeteoraEventParser;
}

pub mod orca_events {
    pub use super::orca::events::{PositionEvent as OrcaPositionEvent, SwapEvent as OrcaSwapEvent};
}

pub mod orca_parser {
    pub use super::orca::parser::Parser as OrcaEventParser;
}

pub mod raydium_v4_events {
    pub use super::raydium_amm_v4::{
        DepositEvent as RaydiumAmmV4DepositEvent, SwapEvent as RaydiumAmmV4SwapEvent,
    };
}

pub mod raydium_v4_parser {
    pub use super::raydium_amm_v4::Parser as RaydiumAmmV4EventParser;
}

pub mod raydium_clmm_events {
    pub use super::raydium_clmm::events::{
        CreatePoolEvent as RaydiumClmmCreatePoolEvent, SwapEvent as RaydiumClmmSwapEvent,
    };
}

pub mod raydium_clmm_parser {
    pub use super::raydium_clmm::Parser as RaydiumClmmEventParser;
}

// Re-export specific types to avoid conflicts - these provide the public API
pub use bonk::types as bonk_types;

// Jupiter module exports - no need for submodule prefixes since it's now a single file
pub use jupiter::{
    is_jupiter_v6_program, v6_program_id as jupiter_v6_program_id,
    AccountLayout as JupiterAccountLayout, EventParser as JupiterEventParser, JupiterSwapData,
    LiquidityEvent as JupiterLiquidityEventType, RoutePlan,
    SwapBorshEvent as JupiterSwapBorshEvent, SwapEvent as JupiterSwapEventType,
    EXACT_OUT_ROUTE_DISCRIMINATOR, JUPITER_V6_PROGRAM_ID, LEGACY_EXACT_OUT_DISCRIMINATOR,
    LEGACY_ROUTE_DISCRIMINATOR, ROUTE_DISCRIMINATOR, ROUTE_WITH_TOKEN_LEDGER_DISCRIMINATOR,
    SWAP_DISCRIMINATOR,
};

// MarginFi module exports - using consolidated single file structure
pub use marginfi::{
    bank_program_id, is_marginfi_program, program_id, MarginFiAccount, MarginFiAccountType,
    MarginFiBalance, MarginFiBankConfig, MarginFiBankState, MarginFiBorrowData,
    MarginFiBorrowEvent, MarginFiDepositData, MarginFiDepositEvent, MarginFiEventParser,
    MarginFiLiquidationData, MarginFiLiquidationEvent, MarginFiRepayData, MarginFiRepayEvent,
    MarginFiWithdrawData, MarginFiWithdrawEvent, MARGINFI_BANK_PROGRAM_ID,
    MARGINFI_BORROW_DISCRIMINATOR, MARGINFI_DEPOSIT_DISCRIMINATOR,
    MARGINFI_LIQUIDATE_DISCRIMINATOR, MARGINFI_PROGRAM_ID, MARGINFI_REPAY_DISCRIMINATOR,
    MARGINFI_WITHDRAW_DISCRIMINATOR,
};

pub use meteora::{
    types as meteora_types, DynamicLiquidityEvent, LiquidityEvent, Parser as MeteoraParser,
    SwapEvent as MeteoraSwapEvent,
};

// Orca re-exports handled by alias modules above

pub use pumpswap::{
    discriminators as pumpswap_discriminators, PumpSwapBuyEvent, PumpSwapCreatePoolEvent,
    PumpSwapDepositEvent, PumpSwapEventParser, PumpSwapSellEvent, PumpSwapWithdrawEvent,
    PUMPSWAP_PROGRAM_ID,
};

pub use raydium_amm_v4::{
    DepositEvent, Initialize2Event, Parser, SwapDirection, SwapEvent, WithdrawEvent,
    WithdrawPnlEvent, DEPOSIT, INITIALIZE2, RAYDIUM_AMM_V4_PROGRAM_ID, SWAP_BASE_IN, SWAP_BASE_OUT,
    WITHDRAW, WITHDRAW_PNL,
};

pub use raydium_clmm::{discriminators as raydium_clmm_discriminators, RAYDIUM_CLMM_PROGRAM_ID};

pub use raydium_cpmm::{
    discriminators as raydium_cpmm_discriminators, DepositEvent as RaydiumCpmmDepositEvent,
    EventParser as RaydiumCpmmEventParser, SwapEvent as RaydiumCpmmSwapEvent,
    RAYDIUM_CPMM_PROGRAM_ID,
};

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use core::any::type_name;

    #[test]
    fn bonk_module_exports_when_accessed_should_be_available() {
        // Test that bonk module exports are accessible
        // This ensures the re-exports work correctly
        let _trade_event = type_name::<bonk_events::BonkTradeEvent>();
        let _pool_event = type_name::<bonk_events::BonkPoolCreateEvent>();
        let _parser = type_name::<bonk_parser::BonkEventParser>();
        let _direction = type_name::<bonk_types::TradeDirection>();
    }

    #[test]
    fn jupiter_module_exports_when_accessed_should_be_available() {
        // Test that jupiter module exports are accessible
        let _swap_event = type_name::<jupiter_events::JupiterSwapEvent>();
        let _liquidity_event = type_name::<jupiter_events::JupiterLiquidityEvent>();
        let _parser = type_name::<jupiter_parser::JupiterEventParser>();
    }

    #[test]
    fn marginfi_module_exports_when_accessed_should_be_available() {
        // Test that marginfi module exports are accessible
        let _deposit_event = type_name::<marginfi_events::MarginFiDepositEvent>();
        let _withdraw_event = type_name::<marginfi_events::MarginFiWithdrawEvent>();
        let _parser = type_name::<marginfi_parser::MarginFiEventParser>();
        let _account = type_name::<marginfi_types::MarginFiAccount>();
    }

    #[test]
    fn meteora_module_exports_when_accessed_should_be_available() {
        // Test that meteora module exports are accessible
        let _swap_event = type_name::<meteora_events::MeteoraSwapEvent>();
        let _liquidity_event = type_name::<meteora_events::MeteoraLiquidityEvent>();
        let _parser = type_name::<meteora_parser::MeteoraEventParser>();
    }

    #[test]
    fn orca_module_exports_when_accessed_should_be_available() {
        // Test that orca module exports are accessible
        let _swap_event = type_name::<orca_events::OrcaSwapEvent>();
        let _position_event = type_name::<orca_events::OrcaPositionEvent>();
        let _parser = type_name::<orca_parser::OrcaEventParser>();
    }

    #[test]
    fn pumpswap_module_exports_when_accessed_should_be_available() {
        // Test that pumpswap module exports are accessible
        let _buy_event = type_name::<PumpSwapBuyEvent>();
        let _sell_event = type_name::<PumpSwapSellEvent>();
        let _parser = type_name::<PumpSwapEventParser>();
    }

    #[test]
    fn raydium_v4_module_exports_when_accessed_should_be_available() {
        // Test that raydium AMM v4 module exports are accessible
        let _swap_event = type_name::<raydium_v4_events::RaydiumAmmV4SwapEvent>();
        let _deposit_event = type_name::<raydium_v4_events::RaydiumAmmV4DepositEvent>();
        let _parser = type_name::<raydium_v4_parser::RaydiumAmmV4EventParser>();
    }

    #[test]
    fn raydium_clmm_module_exports_when_accessed_should_be_available() {
        // Test that raydium CLMM module exports are accessible
        let _swap_event = type_name::<raydium_clmm_events::RaydiumClmmSwapEvent>();
        let _create_pool_event = type_name::<raydium_clmm_events::RaydiumClmmCreatePoolEvent>();
        let _parser = type_name::<raydium_clmm_parser::RaydiumClmmEventParser>();
        let program_id = RAYDIUM_CLMM_PROGRAM_ID;
        let _ = program_id; // Explicitly use the variable
    }

    #[test]
    fn raydium_cpmm_module_exports_when_accessed_should_be_available() {
        // Test that raydium CPMM module exports are accessible
        let _swap_event = type_name::<raydium_cpmm_events::RaydiumCpmmSwapEvent>();
        let _deposit_event = type_name::<raydium_cpmm_events::RaydiumCpmmDepositEvent>();
        let _parser = type_name::<raydium_cpmm_parser::RaydiumCpmmEventParser>();
    }

    #[test]
    fn module_compilation_when_imported_should_succeed() {
        // This test verifies that all module declarations are valid
        // and can be compiled without errors. The fact that this test
        // compiles means all the module paths are correct.
        // Module compilation check passed - if this test compiles, all modules are correctly declared
    }

    #[test]
    fn re_exports_when_used_should_not_conflict() {
        // Test that re-exports don't create naming conflicts
        // by accessing types with their aliased names
        let _bonk_trade_event = type_name::<bonk_events::BonkTradeEvent>();
        let _jupiter_swap_event = type_name::<jupiter_events::JupiterSwapEvent>();
        let _marginfi_deposit_event = type_name::<marginfi_events::MarginFiDepositEvent>();
        let _meteora_swap_event = type_name::<meteora_events::MeteoraSwapEvent>();
        let _orca_swap_event = type_name::<orca_events::OrcaSwapEvent>();
        let _pumpswap_buy_event = type_name::<PumpSwapBuyEvent>();

        // Test that we can access multiple raydium variants without conflict
        let _v4_swap_event = type_name::<raydium_v4_events::RaydiumAmmV4SwapEvent>();
        let _clmm_swap_event = type_name::<raydium_clmm_events::RaydiumClmmSwapEvent>();
        let _cpmm_swap_event = type_name::<raydium_cpmm_events::RaydiumCpmmSwapEvent>();
    }
}
