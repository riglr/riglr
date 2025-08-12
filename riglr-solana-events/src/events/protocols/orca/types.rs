use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Orca Whirlpool program ID
pub const ORCA_WHIRLPOOL_PROGRAM_ID: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";

/// Orca Whirlpool instruction discriminators (calculated from Anchor's "global:<instruction_name>")
/// Instruction discriminator for swap operations
pub const SWAP_DISCRIMINATOR: [u8; 8] = [0xf8, 0xc6, 0x9e, 0x91, 0xe1, 0x75, 0x87, 0xc8]; // swap
/// Instruction discriminator for opening liquidity positions
pub const OPEN_POSITION_DISCRIMINATOR: [u8; 8] = [0x87, 0x80, 0x2f, 0x4d, 0x0f, 0x98, 0xf0, 0x31]; // open_position
/// Instruction discriminator for closing liquidity positions
pub const CLOSE_POSITION_DISCRIMINATOR: [u8; 8] = [0x7b, 0x86, 0x51, 0x00, 0x31, 0x44, 0x62, 0x62]; // close_position
/// Instruction discriminator for increasing liquidity in positions
pub const INCREASE_LIQUIDITY_DISCRIMINATOR: [u8; 8] =
    [0x2e, 0x9c, 0xf3, 0x76, 0x0d, 0xcd, 0xfb, 0xb2]; // increase_liquidity
/// Instruction discriminator for decreasing liquidity in positions
pub const DECREASE_LIQUIDITY_DISCRIMINATOR: [u8; 8] =
    [0xa0, 0x26, 0xd0, 0x6f, 0x68, 0x5b, 0x2c, 0x01]; // decrease_liquidity

/// Orca swap direction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SwapDirection {
    /// Swap from token A to token B
    AtoB,
    /// Swap from token B to token A
    BtoA,
}

/// Orca Whirlpool account layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhirlpoolAccount {
    /// Configuration account that governs this whirlpool's parameters
    pub whirlpools_config: Pubkey,
    /// Program-derived address bump seed for this whirlpool account
    pub whirlpool_bump: [u8; 1],
    /// The tick spacing for this whirlpool, determining price granularity
    pub tick_spacing: u16,
    /// Seed bytes used to derive the whirlpool account from tick spacing
    pub tick_spacing_seed: [u8; 2],
    /// Fee rate charged for swaps in basis points (e.g., 300 = 0.3%)
    pub fee_rate: u16,
    /// Protocol fee rate in basis points taken from the swap fee
    pub protocol_fee_rate: u16,
    /// Total liquidity currently available in the pool
    pub liquidity: u128,
    /// Square root of the current price as a Q64.64 fixed-point number
    pub sqrt_price: u128,
    /// Current tick index representing the active price range
    pub tick_current_index: i32,
    /// Protocol fees accumulated for token A awaiting collection
    pub protocol_fee_owed_a: u64,
    /// Protocol fees accumulated for token B awaiting collection
    pub protocol_fee_owed_b: u64,
    /// Mint address of the first token in the trading pair
    pub token_mint_a: Pubkey,
    /// Vault account holding the pool's token A reserves
    pub token_vault_a: Pubkey,
    /// Global fee growth accumulator for token A as Q64.64 fixed-point
    pub fee_growth_global_a: u128,
    /// Mint address of the second token in the trading pair
    pub token_mint_b: Pubkey,
    /// Vault account holding the pool's token B reserves
    pub token_vault_b: Pubkey,
    /// Global fee growth accumulator for token B as Q64.64 fixed-point
    pub fee_growth_global_b: u128,
    /// Timestamp when reward calculations were last updated
    pub reward_last_updated_timestamp: u64,
    /// Array of up to 3 reward token configurations for this pool
    pub reward_infos: [WhirlpoolRewardInfo; 3],
}

/// Whirlpool reward information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhirlpoolRewardInfo {
    /// Mint address of the reward token being distributed
    pub mint: Pubkey,
    /// Vault account holding the reward tokens for distribution
    pub vault: Pubkey,
    /// Authority account that can control reward distribution parameters
    pub authority: Pubkey,
    /// Rate of reward token emissions per second as Q64.64 fixed-point
    pub emissions_per_second_x64: u128,
    /// Global growth accumulator for this reward token as Q64.64 fixed-point
    pub growth_global_x64: u128,
}

/// Orca swap event data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrcaSwapData {
    /// The whirlpool account where the swap occurred
    pub whirlpool: Pubkey,
    /// The user account that initiated the swap transaction
    pub user: Pubkey,
    /// Mint address of the first token in the trading pair
    pub token_mint_a: Pubkey,
    /// Mint address of the second token in the trading pair
    pub token_mint_b: Pubkey,
    /// Vault account holding token A reserves for the pool
    pub token_vault_a: Pubkey,
    /// Vault account holding token B reserves for the pool
    pub token_vault_b: Pubkey,
    /// The amount specified by the user for the swap operation
    pub amount: u64,
    /// Whether the specified amount represents input (true) or output (false)
    pub amount_specified_is_input: bool,
    /// Direction of the swap: true for A→B, false for B→A
    pub a_to_b: bool,
    /// Price limit for the swap as square root price Q64.64 fixed-point
    pub sqrt_price_limit: u128,
    /// Actual amount of tokens consumed in the swap
    pub amount_in: u64,
    /// Actual amount of tokens received from the swap
    pub amount_out: u64,
    /// Total fee amount charged for this swap transaction
    pub fee_amount: u64,
    /// Current tick index after the swap execution
    pub tick_current_index: i32,
    /// Current square root price after the swap as Q64.64 fixed-point
    pub sqrt_price: u128,
    /// Current liquidity in the pool after the swap
    pub liquidity: u128,
}

/// Orca position data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrcaPositionData {
    /// The whirlpool account this position provides liquidity to
    pub whirlpool: Pubkey,
    /// Mint address for the NFT representing this liquidity position
    pub position_mint: Pubkey,
    /// Account address storing the position's state and parameters
    pub position: Pubkey,
    /// Token account holding the position NFT
    pub position_token_account: Pubkey,
    /// Authority account that can modify this position
    pub position_authority: Pubkey,
    /// Lower tick boundary of the position's price range
    pub tick_lower_index: i32,
    /// Upper tick boundary of the position's price range
    pub tick_upper_index: i32,
    /// Amount of liquidity provided by this position
    pub liquidity: u128,
    /// Fee growth checkpoint for token A when position was last updated
    pub fee_growth_checkpoint_a: u128,
    /// Fee growth checkpoint for token B when position was last updated
    pub fee_growth_checkpoint_b: u128,
    /// Accumulated fees owed to this position in token A
    pub fee_owed_a: u64,
    /// Accumulated fees owed to this position in token B
    pub fee_owed_b: u64,
    /// Array of reward information for up to 3 reward tokens
    pub reward_infos: [PositionRewardInfo; 3],
}

/// Position reward information
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PositionRewardInfo {
    /// Reward growth checkpoint inside the position's tick range
    pub growth_inside_checkpoint: u128,
    /// Amount of reward tokens owed to this position
    pub amount_owed: u64,
}

/// Orca liquidity change data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrcaLiquidityData {
    /// The whirlpool account where liquidity is being modified
    pub whirlpool: Pubkey,
    /// The position account being modified
    pub position: Pubkey,
    /// Authority account that can modify the position
    pub position_authority: Pubkey,
    /// Mint address of the first token in the trading pair
    pub token_mint_a: Pubkey,
    /// Mint address of the second token in the trading pair
    pub token_mint_b: Pubkey,
    /// Vault account holding token A reserves for the pool
    pub token_vault_a: Pubkey,
    /// Vault account holding token B reserves for the pool
    pub token_vault_b: Pubkey,
    /// Lower tick boundary of the position's price range
    pub tick_lower_index: i32,
    /// Upper tick boundary of the position's price range
    pub tick_upper_index: i32,
    /// Amount of liquidity being added or removed
    pub liquidity_amount: u128,
    /// Maximum amount of token A willing to deposit/withdraw
    pub token_max_a: u64,
    /// Maximum amount of token B willing to deposit/withdraw
    pub token_max_b: u64,
    /// Actual amount of token A deposited/withdrawn
    pub token_actual_a: u64,
    /// Actual amount of token B deposited/withdrawn
    pub token_actual_b: u64,
    /// Whether this is a liquidity increase (true) or decrease (false)
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
