use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Meteora DLMM program ID
pub const METEORA_DLMM_PROGRAM_ID: &str = "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo";

/// Meteora Dynamic program ID
pub const METEORA_DYNAMIC_PROGRAM_ID: &str = "Dooar9JkhdZ7J3LHN3A7YCuoGRUggXhQaG4kijfLGU2j";

/// Meteora instruction discriminators
/// Discriminator for DLMM swap instruction
pub const DLMM_SWAP_DISCRIMINATOR: [u8; 8] = [0x14, 0x65, 0x32, 0x1f, 0x7a, 0x43, 0x2a, 0x9f];
/// Discriminator for DLMM add liquidity instruction
pub const DLMM_ADD_LIQUIDITY_DISCRIMINATOR: [u8; 8] =
    [0x4c, 0x1c, 0x9b, 0x2d, 0xe3, 0x7a, 0x8b, 0x12];
/// Discriminator for DLMM remove liquidity instruction
pub const DLMM_REMOVE_LIQUIDITY_DISCRIMINATOR: [u8; 8] =
    [0xa2, 0xfd, 0x67, 0xe3, 0x45, 0x1b, 0x8c, 0x9a];
/// Discriminator for Dynamic AMM add liquidity instruction
pub const DYNAMIC_ADD_LIQUIDITY_DISCRIMINATOR: [u8; 8] =
    [0x85, 0x72, 0x1a, 0x5f, 0x9d, 0x4e, 0x23, 0x7c];
/// Discriminator for Dynamic AMM remove liquidity instruction
pub const DYNAMIC_REMOVE_LIQUIDITY_DISCRIMINATOR: [u8; 8] =
    [0x6a, 0x8b, 0x47, 0x2e, 0x1c, 0x93, 0x5f, 0x4d];

/// Meteora DLMM bin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlmmBin {
    /// Unique identifier for this price bin
    pub bin_id: u32,
    /// Amount of token X reserves in this bin
    pub reserve_x: u64,
    /// Amount of token Y reserves in this bin
    pub reserve_y: u64,
    /// Current price for this bin
    pub price: f64,
    /// Total liquidity token supply for this bin
    pub liquidity_supply: u128,
}

/// Meteora DLMM pair configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlmmPairConfig {
    /// Public key of the DLMM pair account
    pub pair: Pubkey,
    /// Mint address of token X
    pub token_mint_x: Pubkey,
    /// Mint address of token Y
    pub token_mint_y: Pubkey,
    /// Step size between bins in basis points
    pub bin_step: u16,
    /// Base fee percentage charged for swaps
    pub base_fee_percentage: u64,
    /// Maximum fee percentage that can be charged
    pub max_fee_percentage: u64,
    /// Protocol fee percentage taken from trades
    pub protocol_fee_percentage: u64,
    /// Liquidity provider fee percentage
    pub liquidity_fee_percentage: u64,
    /// Current volatility accumulator value
    pub volatility_accumulator: u32,
    /// Volatility reference point
    pub volatility_reference: u32,
    /// ID reference for bin tracking
    pub id_reference: u32,
    /// Timestamp of last pair update
    pub time_of_last_update: u64,
    /// Currently active bin ID
    pub active_id: u32,
    /// Base key for the pair
    pub base_key: Pubkey,
}

/// Meteora DLMM swap data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MeteoraSwapData {
    /// Public key of the DLMM pair being swapped on
    pub pair: Pubkey,
    /// Public key of the user performing the swap
    pub user: Pubkey,
    /// Mint address of token X
    pub token_mint_x: Pubkey,
    /// Mint address of token Y
    pub token_mint_y: Pubkey,
    /// Reserve account for token X
    pub reserve_x: Pubkey,
    /// Reserve account for token Y
    pub reserve_y: Pubkey,
    /// Amount of tokens being swapped in
    pub amount_in: u64,
    /// Minimum expected amount of tokens out
    pub min_amount_out: u64,
    /// Actual amount of tokens received
    pub actual_amount_out: u64,
    /// Whether swapping X for Y (true) or Y for X (false)
    pub swap_for_y: bool,
    /// Active bin ID before the swap
    pub active_id_before: u32,
    /// Active bin ID after the swap
    pub active_id_after: u32,
    /// Total fee amount charged for the swap
    pub fee_amount: u64,
    /// Protocol fee portion of the total fee
    pub protocol_fee: u64,
    /// List of bin IDs traversed during the swap
    pub bins_traversed: Vec<u32>,
}

/// Meteora DLMM liquidity data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MeteoraLiquidityData {
    /// Public key of the DLMM pair
    pub pair: Pubkey,
    /// Public key of the user adding/removing liquidity
    pub user: Pubkey,
    /// Public key of the liquidity position
    pub position: Pubkey,
    /// Mint address of token X
    pub token_mint_x: Pubkey,
    /// Mint address of token Y
    pub token_mint_y: Pubkey,
    /// Reserve account for token X
    pub reserve_x: Pubkey,
    /// Reserve account for token Y
    pub reserve_y: Pubkey,
    /// Starting bin ID for liquidity range
    pub bin_id_from: u32,
    /// Ending bin ID for liquidity range
    pub bin_id_to: u32,
    /// Amount of token X added or removed
    pub amount_x: u64,
    /// Amount of token Y added or removed
    pub amount_y: u64,
    /// Amount of liquidity tokens minted or burned
    pub liquidity_minted: u128,
    /// Currently active bin ID
    pub active_id: u32,
    /// Whether this is an add (true) or remove (false) operation
    pub is_add: bool,
    /// List of bins affected by this liquidity operation
    pub bins_affected: Vec<DlmmBin>,
}

/// Meteora Dynamic AMM pool data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteoraDynamicPoolData {
    /// Public key of the Dynamic AMM pool
    pub pool: Pubkey,
    /// Mint address of token A
    pub token_mint_a: Pubkey,
    /// Mint address of token B
    pub token_mint_b: Pubkey,
    /// Vault account holding token A reserves
    pub vault_a: Pubkey,
    /// Vault account holding token B reserves
    pub vault_b: Pubkey,
    /// Mint address of the LP tokens
    pub lp_mint: Pubkey,
    /// Base fee rate for the pool
    pub fee_rate: u64,
    /// Administrative fee rate
    pub admin_fee_rate: u64,
    /// Numerator for trade fee calculation
    pub trade_fee_numerator: u64,
    /// Denominator for trade fee calculation
    pub trade_fee_denominator: u64,
    /// Numerator for owner trade fee calculation
    pub owner_trade_fee_numerator: u64,
    /// Denominator for owner trade fee calculation
    pub owner_trade_fee_denominator: u64,
    /// Numerator for owner withdraw fee calculation
    pub owner_withdraw_fee_numerator: u64,
    /// Denominator for owner withdraw fee calculation
    pub owner_withdraw_fee_denominator: u64,
    /// Numerator for host fee calculation
    pub host_fee_numerator: u64,
    /// Denominator for host fee calculation
    pub host_fee_denominator: u64,
}

/// Meteora Dynamic liquidity data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MeteoraDynamicLiquidityData {
    /// Public key of the Dynamic AMM pool
    pub pool: Pubkey,
    /// Public key of the user adding/removing liquidity
    pub user: Pubkey,
    /// Mint address of token A
    pub token_mint_a: Pubkey,
    /// Mint address of token B
    pub token_mint_b: Pubkey,
    /// Vault account holding token A reserves
    pub vault_a: Pubkey,
    /// Vault account holding token B reserves
    pub vault_b: Pubkey,
    /// Mint address of the LP tokens
    pub lp_mint: Pubkey,
    /// Amount of pool tokens being minted or burned
    pub pool_token_amount: u64,
    /// Amount of token A being deposited or withdrawn
    pub token_a_amount: u64,
    /// Amount of token B being deposited or withdrawn
    pub token_b_amount: u64,
    /// Minimum acceptable pool token amount for slippage protection
    pub minimum_pool_token_amount: u64,
    /// Maximum token A amount willing to deposit
    pub maximum_token_a_amount: u64,
    /// Maximum token B amount willing to deposit
    pub maximum_token_b_amount: u64,
    /// Whether this is a deposit (true) or withdrawal (false) operation
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
