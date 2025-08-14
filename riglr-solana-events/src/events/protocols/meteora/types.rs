use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Meteora DLMM program ID
pub const METEORA_DLMM_PROGRAM_ID: &str = "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo";

/// Meteora Dynamic program ID  
pub const METEORA_DYNAMIC_PROGRAM_ID: &str = "Dooar9JkhdZ7J3LHN3A7YCuoGRUggXhQaG4kijfLGU2j";

/// Meteora instruction discriminators
pub const DLMM_SWAP_DISCRIMINATOR: [u8; 8] = [0x14, 0x65, 0x32, 0x1f, 0x7a, 0x43, 0x2a, 0x9f];
pub const DLMM_ADD_LIQUIDITY_DISCRIMINATOR: [u8; 8] = [0x4c, 0x1c, 0x9b, 0x2d, 0xe3, 0x7a, 0x8b, 0x12];
pub const DLMM_REMOVE_LIQUIDITY_DISCRIMINATOR: [u8; 8] = [0xa2, 0xfd, 0x67, 0xe3, 0x45, 0x1b, 0x8c, 0x9a];
pub const DYNAMIC_ADD_LIQUIDITY_DISCRIMINATOR: [u8; 8] = [0x85, 0x72, 0x1a, 0x5f, 0x9d, 0x4e, 0x23, 0x7c];
pub const DYNAMIC_REMOVE_LIQUIDITY_DISCRIMINATOR: [u8; 8] = [0x6a, 0x8b, 0x47, 0x2e, 0x1c, 0x93, 0x5f, 0x4d];

/// Meteora DLMM bin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlmmBin {
    pub bin_id: u32,
    pub reserve_x: u64,
    pub reserve_y: u64,
    pub price: f64,
    pub liquidity_supply: u128,
}

/// Meteora DLMM pair configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlmmPairConfig {
    pub pair: Pubkey,
    pub token_mint_x: Pubkey,
    pub token_mint_y: Pubkey,
    pub bin_step: u16,
    pub base_fee_percentage: u64,
    pub max_fee_percentage: u64,
    pub protocol_fee_percentage: u64,
    pub liquidity_fee_percentage: u64,
    pub volatility_accumulator: u32,
    pub volatility_reference: u32,
    pub id_reference: u32,
    pub time_of_last_update: u64,
    pub active_id: u32,
    pub base_key: Pubkey,
}

/// Meteora DLMM swap data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraSwapData {
    pub pair: Pubkey,
    pub user: Pubkey,
    pub token_mint_x: Pubkey,
    pub token_mint_y: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub amount_in: u64,
    pub min_amount_out: u64,
    pub actual_amount_out: u64,
    pub swap_for_y: bool,
    pub active_id_before: u32,
    pub active_id_after: u32,
    pub fee_amount: u64,
    pub protocol_fee: u64,
    pub bins_traversed: Vec<u32>,
}

/// Meteora DLMM liquidity data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraLiquidityData {
    pub pair: Pubkey,
    pub user: Pubkey,
    pub position: Pubkey,
    pub token_mint_x: Pubkey,
    pub token_mint_y: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub bin_id_from: u32,
    pub bin_id_to: u32,
    pub amount_x: u64,
    pub amount_y: u64,
    pub liquidity_minted: u128,
    pub active_id: u32,
    pub is_add: bool,
    pub bins_affected: Vec<DlmmBin>,
}

/// Meteora Dynamic AMM pool data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraDynamicPoolData {
    pub pool: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub lp_mint: Pubkey,
    pub fee_rate: u64,
    pub admin_fee_rate: u64,
    pub trade_fee_numerator: u64,
    pub trade_fee_denominator: u64,
    pub owner_trade_fee_numerator: u64,
    pub owner_trade_fee_denominator: u64,
    pub owner_withdraw_fee_numerator: u64,
    pub owner_withdraw_fee_denominator: u64,
    pub host_fee_numerator: u64,
    pub host_fee_denominator: u64,
}

/// Meteora Dynamic liquidity data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraDynamicLiquidityData {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub lp_mint: Pubkey,
    pub pool_token_amount: u64,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub minimum_pool_token_amount: u64,
    pub maximum_token_a_amount: u64,
    pub maximum_token_b_amount: u64,
    pub is_deposit: bool,
}

/// Extract Meteora DLMM program ID as Pubkey
pub fn meteora_dlmm_program_id() -> Pubkey {
    Pubkey::try_from(METEORA_DLMM_PROGRAM_ID).expect("Invalid Meteora DLMM program ID")
}

/// Extract Meteora Dynamic program ID as Pubkey
pub fn meteora_dynamic_program_id() -> Pubkey {
    Pubkey::try_from(METEORA_DYNAMIC_PROGRAM_ID).expect("Invalid Meteora Dynamic program ID")
}

/// Check if the given pubkey is Meteora DLMM program
pub fn is_meteora_dlmm_program(program_id: &Pubkey) -> bool {
    *program_id == meteora_dlmm_program_id()
}

/// Check if the given pubkey is Meteora Dynamic program
pub fn is_meteora_dynamic_program(program_id: &Pubkey) -> bool {
    *program_id == meteora_dynamic_program_id()
}

/// Convert bin ID to price for DLMM
pub fn bin_id_to_price(bin_id: u32, bin_step: u16) -> f64 {
    let bin_step_decimal = bin_step as f64 / 10000.0;
    (1.0 + bin_step_decimal).powi(bin_id as i32 - 8388608) // 2^23 offset
}

/// Calculate active bin price
pub fn calculate_active_bin_price(active_id: u32, bin_step: u16) -> f64 {
    bin_id_to_price(active_id, bin_step)
}

/// Calculate liquidity distribution across bins
pub fn calculate_liquidity_distribution(
    amount_x: u64,
    amount_y: u64,
    bin_id_from: u32,
    bin_id_to: u32,
    active_id: u32,
) -> Vec<(u32, u64, u64)> {
    let mut distribution = Vec::new();
    let total_bins = (bin_id_to - bin_id_from + 1) as u64;
    
    if total_bins == 0 {
        return distribution;
    }

    for bin_id in bin_id_from..=bin_id_to {
        let x_amount = if bin_id <= active_id {
            amount_x / total_bins
        } else {
            0
        };
        
        let y_amount = if bin_id >= active_id {
            amount_y / total_bins
        } else {
            0
        };
        
        distribution.push((bin_id, x_amount, y_amount));
    }
    
    distribution
}