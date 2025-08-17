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
