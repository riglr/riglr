use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Orca Whirlpool program ID
pub const ORCA_WHIRLPOOL_PROGRAM_ID: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";

/// Orca Whirlpool instruction discriminators (calculated from Anchor's "global:<instruction_name>")
pub const SWAP_DISCRIMINATOR: [u8; 8] = [0xf8, 0xc6, 0x9e, 0x91, 0xe1, 0x75, 0x87, 0xc8]; // swap
pub const OPEN_POSITION_DISCRIMINATOR: [u8; 8] = [0x87, 0x80, 0x2f, 0x4d, 0x0f, 0x98, 0xf0, 0x31]; // open_position
pub const CLOSE_POSITION_DISCRIMINATOR: [u8; 8] = [0x7b, 0x86, 0x51, 0x00, 0x31, 0x44, 0x62, 0x62]; // close_position
pub const INCREASE_LIQUIDITY_DISCRIMINATOR: [u8; 8] = [0x2e, 0x9c, 0xf3, 0x76, 0x0d, 0xcd, 0xfb, 0xb2]; // increase_liquidity
pub const DECREASE_LIQUIDITY_DISCRIMINATOR: [u8; 8] = [0xa0, 0x26, 0xd0, 0x6f, 0x68, 0x5b, 0x2c, 0x01]; // decrease_liquidity

/// Orca swap direction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SwapDirection {
    AtoB,
    BtoA,
}

/// Orca Whirlpool account layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhirlpoolAccount {
    pub whirlpools_config: Pubkey,
    pub whirlpool_bump: [u8; 1],
    pub tick_spacing: u16,
    pub tick_spacing_seed: [u8; 2],
    pub fee_rate: u16,
    pub protocol_fee_rate: u16,
    pub liquidity: u128,
    pub sqrt_price: u128,
    pub tick_current_index: i32,
    pub protocol_fee_owed_a: u64,
    pub protocol_fee_owed_b: u64,
    pub token_mint_a: Pubkey,
    pub token_vault_a: Pubkey,
    pub fee_growth_global_a: u128,
    pub token_mint_b: Pubkey,
    pub token_vault_b: Pubkey,
    pub fee_growth_global_b: u128,
    pub reward_last_updated_timestamp: u64,
    pub reward_infos: [WhirlpoolRewardInfo; 3],
}

/// Whirlpool reward information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhirlpoolRewardInfo {
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub authority: Pubkey,
    pub emissions_per_second_x64: u128,
    pub growth_global_x64: u128,
}

/// Orca swap event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcaSwapData {
    pub whirlpool: Pubkey,
    pub user: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub token_vault_a: Pubkey,
    pub token_vault_b: Pubkey,
    pub amount: u64,
    pub amount_specified_is_input: bool,
    pub a_to_b: bool,
    pub sqrt_price_limit: u128,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee_amount: u64,
    pub tick_current_index: i32,
    pub sqrt_price: u128,
    pub liquidity: u128,
}

/// Orca position data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcaPositionData {
    pub whirlpool: Pubkey,
    pub position_mint: Pubkey,
    pub position: Pubkey,
    pub position_token_account: Pubkey,
    pub position_authority: Pubkey,
    pub tick_lower_index: i32,
    pub tick_upper_index: i32,
    pub liquidity: u128,
    pub fee_growth_checkpoint_a: u128,
    pub fee_growth_checkpoint_b: u128,
    pub fee_owed_a: u64,
    pub fee_owed_b: u64,
    pub reward_infos: [PositionRewardInfo; 3],
}

/// Position reward information
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PositionRewardInfo {
    pub growth_inside_checkpoint: u128,
    pub amount_owed: u64,
}

/// Orca liquidity change data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcaLiquidityData {
    pub whirlpool: Pubkey,
    pub position: Pubkey,
    pub position_authority: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub token_vault_a: Pubkey,
    pub token_vault_b: Pubkey,
    pub tick_lower_index: i32,
    pub tick_upper_index: i32,
    pub liquidity_amount: u128,
    pub token_max_a: u64,
    pub token_max_b: u64,
    pub token_actual_a: u64,
    pub token_actual_b: u64,
    pub is_increase: bool,
}

/// Extract Orca Whirlpool program ID as Pubkey
pub fn orca_whirlpool_program_id() -> Pubkey {
    Pubkey::try_from(ORCA_WHIRLPOOL_PROGRAM_ID).expect("Invalid Orca Whirlpool program ID")
}

/// Check if the given pubkey is Orca Whirlpool program
pub fn is_orca_whirlpool_program(program_id: &Pubkey) -> bool {
    *program_id == orca_whirlpool_program_id()
}

/// Convert tick index to price
pub fn tick_index_to_price(tick_index: i32) -> f64 {
    1.0001_f64.powi(tick_index)
}

/// Convert sqrt price to price
pub fn sqrt_price_to_price(sqrt_price: u128, decimals_a: u8, decimals_b: u8) -> f64 {
    let price_x64 = (sqrt_price as f64 / (1u128 << 64) as f64).powi(2);
    let decimal_adjustment = 10_f64.powi(decimals_a as i32 - decimals_b as i32);
    price_x64 * decimal_adjustment
}