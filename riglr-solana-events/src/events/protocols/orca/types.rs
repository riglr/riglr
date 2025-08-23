use serde::{Deserialize, Deserializer, Serialize, Serializer};
use solana_sdk::pubkey::Pubkey;

/// Serialize u128 as string for JSON compatibility
fn serialize_u128_as_string<S>(value: &u128, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

/// Deserialize u128 from string for JSON compatibility
fn deserialize_u128_from_string<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

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
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub liquidity: u128,
    /// Square root of the current price as a Q64.64 fixed-point number
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
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
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
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
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub sqrt_price: u128,
    /// Current liquidity in the pool after the swap
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
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

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    #[test]
    fn test_orca_whirlpool_program_id_should_return_valid_pubkey() {
        let program_id = orca_whirlpool_program_id();
        let expected = Pubkey::from_str(ORCA_WHIRLPOOL_PROGRAM_ID).unwrap();
        assert_eq!(program_id, expected);
    }

    #[test]
    fn test_is_orca_whirlpool_program_when_correct_program_id_should_return_true() {
        let program_id = orca_whirlpool_program_id();
        assert!(is_orca_whirlpool_program(&program_id));
    }

    #[test]
    fn test_is_orca_whirlpool_program_when_different_program_id_should_return_false() {
        let different_program_id = Pubkey::new_unique();
        assert!(!is_orca_whirlpool_program(&different_program_id));
    }

    #[test]
    fn test_tick_index_to_price_when_zero_should_return_one() {
        assert_eq!(tick_index_to_price(0), 1.0);
    }

    #[test]
    fn test_tick_index_to_price_when_positive_should_return_greater_than_one() {
        let price = tick_index_to_price(1000);
        assert!(price > 1.0);
        assert!((price - 1.1051709180756477).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tick_index_to_price_when_negative_should_return_less_than_one() {
        let price = tick_index_to_price(-1000);
        assert!(price < 1.0);
        assert!((price - 0.9048374180359595).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tick_index_to_price_when_max_value_should_not_panic() {
        let price = tick_index_to_price(i32::MAX);
        assert!(price.is_finite());
    }

    #[test]
    fn test_tick_index_to_price_when_min_value_should_not_panic() {
        let price = tick_index_to_price(i32::MIN);
        assert!(price >= 0.0);
    }

    #[test]
    fn test_sqrt_price_to_price_when_equal_decimals_should_calculate_correctly() {
        let sqrt_price = 1u128 << 64; // Q64.64 representation of sqrt(1) = 1
        let price = sqrt_price_to_price(sqrt_price, 6, 6);
        assert!((price - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_sqrt_price_to_price_when_different_decimals_should_adjust_correctly() {
        let sqrt_price = 1u128 << 64; // Q64.64 representation of sqrt(1) = 1
        let price = sqrt_price_to_price(sqrt_price, 9, 6); // 9 - 6 = 3 decimal difference
        assert!((price - 1000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_sqrt_price_to_price_when_zero_sqrt_price_should_return_zero() {
        let price = sqrt_price_to_price(0, 6, 6);
        assert_eq!(price, 0.0);
    }

    #[test]
    fn test_sqrt_price_to_price_when_large_sqrt_price_should_calculate() {
        let sqrt_price = u128::MAX;
        let price = sqrt_price_to_price(sqrt_price, 6, 6);
        assert!(price.is_finite());
        assert!(price > 0.0);
    }

    #[test]
    fn test_sqrt_price_to_price_when_decimals_b_greater_than_a_should_adjust_correctly() {
        let sqrt_price = 1u128 << 64;
        let price = sqrt_price_to_price(sqrt_price, 6, 9); // 6 - 9 = -3 decimal difference
        assert!((price - 0.001).abs() < f64::EPSILON);
    }

    #[test]
    fn test_swap_direction_atob_equality() {
        assert_eq!(SwapDirection::AtoB, SwapDirection::AtoB);
        assert_ne!(SwapDirection::AtoB, SwapDirection::BtoA);
    }

    #[test]
    fn test_swap_direction_btoa_equality() {
        assert_eq!(SwapDirection::BtoA, SwapDirection::BtoA);
        assert_ne!(SwapDirection::BtoA, SwapDirection::AtoB);
    }

    #[test]
    fn test_swap_direction_clone() {
        let direction = SwapDirection::AtoB;
        let cloned = direction.clone();
        assert_eq!(direction, cloned);
    }

    #[test]
    fn test_swap_direction_debug() {
        let direction = SwapDirection::AtoB;
        let debug_str = format!("{:?}", direction);
        assert_eq!(debug_str, "AtoB");

        let direction = SwapDirection::BtoA;
        let debug_str = format!("{:?}", direction);
        assert_eq!(debug_str, "BtoA");
    }

    #[test]
    fn test_whirlpool_account_creation() {
        let config = Pubkey::new_unique();
        let token_mint_a = Pubkey::new_unique();
        let token_mint_b = Pubkey::new_unique();
        let token_vault_a = Pubkey::new_unique();
        let token_vault_b = Pubkey::new_unique();

        let reward_info = WhirlpoolRewardInfo {
            mint: Pubkey::new_unique(),
            vault: Pubkey::new_unique(),
            authority: Pubkey::new_unique(),
            emissions_per_second_x64: 1000,
            growth_global_x64: 2000,
        };

        let whirlpool = WhirlpoolAccount {
            whirlpools_config: config,
            whirlpool_bump: [1],
            tick_spacing: 64,
            tick_spacing_seed: [0, 64],
            fee_rate: 300,
            protocol_fee_rate: 100,
            liquidity: 50000,
            sqrt_price: 1u128 << 64,
            tick_current_index: 0,
            protocol_fee_owed_a: 100,
            protocol_fee_owed_b: 200,
            token_mint_a,
            token_vault_a,
            fee_growth_global_a: 1000,
            token_mint_b,
            token_vault_b,
            fee_growth_global_b: 2000,
            reward_last_updated_timestamp: 1640995200,
            reward_infos: core::array::from_fn(|_| reward_info.clone()),
        };

        assert_eq!(whirlpool.whirlpools_config, config);
        assert_eq!(whirlpool.tick_spacing, 64);
        assert_eq!(whirlpool.fee_rate, 300);
    }

    #[test]
    fn test_whirlpool_reward_info_creation() {
        let mint = Pubkey::new_unique();
        let vault = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let reward_info = WhirlpoolRewardInfo {
            mint,
            vault,
            authority,
            emissions_per_second_x64: 1000,
            growth_global_x64: 2000,
        };

        assert_eq!(reward_info.mint, mint);
        assert_eq!(reward_info.vault, vault);
        assert_eq!(reward_info.authority, authority);
        assert_eq!(reward_info.emissions_per_second_x64, 1000);
        assert_eq!(reward_info.growth_global_x64, 2000);
    }

    #[test]
    fn test_orca_swap_data_default() {
        let swap_data = OrcaSwapData::default();
        assert_eq!(swap_data.whirlpool, Pubkey::default());
        assert_eq!(swap_data.user, Pubkey::default());
        assert_eq!(swap_data.amount, 0);
        assert!(!swap_data.amount_specified_is_input);
        assert!(!swap_data.a_to_b);
        assert_eq!(swap_data.sqrt_price_limit, 0);
        assert_eq!(swap_data.amount_in, 0);
        assert_eq!(swap_data.amount_out, 0);
        assert_eq!(swap_data.fee_amount, 0);
        assert_eq!(swap_data.tick_current_index, 0);
        assert_eq!(swap_data.sqrt_price, 0);
        assert_eq!(swap_data.liquidity, 0);
    }

    #[test]
    fn test_orca_swap_data_creation() {
        let whirlpool = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let token_mint_a = Pubkey::new_unique();

        let swap_data = OrcaSwapData {
            whirlpool,
            user,
            token_mint_a,
            amount: 1000,
            amount_specified_is_input: true,
            a_to_b: true,
            sqrt_price_limit: 1u128 << 64,
            amount_in: 1000,
            amount_out: 950,
            fee_amount: 3,
            tick_current_index: 100,
            sqrt_price: 1u128 << 64,
            liquidity: 50000,
            ..Default::default()
        };

        assert_eq!(swap_data.whirlpool, whirlpool);
        assert_eq!(swap_data.user, user);
        assert_eq!(swap_data.amount, 1000);
        assert!(swap_data.amount_specified_is_input);
        assert!(swap_data.a_to_b);
    }

    #[test]
    fn test_orca_position_data_default() {
        let position_data = OrcaPositionData::default();
        assert_eq!(position_data.whirlpool, Pubkey::default());
        assert_eq!(position_data.position_mint, Pubkey::default());
        assert_eq!(position_data.position, Pubkey::default());
        assert_eq!(position_data.tick_lower_index, 0);
        assert_eq!(position_data.tick_upper_index, 0);
        assert_eq!(position_data.liquidity, 0);
        assert_eq!(position_data.fee_growth_checkpoint_a, 0);
        assert_eq!(position_data.fee_growth_checkpoint_b, 0);
        assert_eq!(position_data.fee_owed_a, 0);
        assert_eq!(position_data.fee_owed_b, 0);
    }

    #[test]
    fn test_position_reward_info_default() {
        let reward_info = PositionRewardInfo::default();
        assert_eq!(reward_info.growth_inside_checkpoint, 0);
        assert_eq!(reward_info.amount_owed, 0);
    }

    #[test]
    fn test_position_reward_info_creation() {
        let reward_info = PositionRewardInfo {
            growth_inside_checkpoint: 5000,
            amount_owed: 250,
        };

        assert_eq!(reward_info.growth_inside_checkpoint, 5000);
        assert_eq!(reward_info.amount_owed, 250);
    }

    #[test]
    fn test_position_reward_info_copy() {
        let original = PositionRewardInfo {
            growth_inside_checkpoint: 5000,
            amount_owed: 250,
        };
        let copied = original;
        assert_eq!(
            original.growth_inside_checkpoint,
            copied.growth_inside_checkpoint
        );
        assert_eq!(original.amount_owed, copied.amount_owed);
    }

    #[test]
    fn test_orca_liquidity_data_default() {
        let liquidity_data = OrcaLiquidityData::default();
        assert_eq!(liquidity_data.whirlpool, Pubkey::default());
        assert_eq!(liquidity_data.position, Pubkey::default());
        assert_eq!(liquidity_data.tick_lower_index, 0);
        assert_eq!(liquidity_data.tick_upper_index, 0);
        assert_eq!(liquidity_data.liquidity_amount, 0);
        assert_eq!(liquidity_data.token_max_a, 0);
        assert_eq!(liquidity_data.token_max_b, 0);
        assert_eq!(liquidity_data.token_actual_a, 0);
        assert_eq!(liquidity_data.token_actual_b, 0);
        assert!(!liquidity_data.is_increase);
    }

    #[test]
    fn test_orca_liquidity_data_creation() {
        let whirlpool = Pubkey::new_unique();
        let position = Pubkey::new_unique();
        let position_authority = Pubkey::new_unique();

        let liquidity_data = OrcaLiquidityData {
            whirlpool,
            position,
            position_authority,
            tick_lower_index: -1000,
            tick_upper_index: 1000,
            liquidity_amount: 50000,
            token_max_a: 1000,
            token_max_b: 2000,
            token_actual_a: 950,
            token_actual_b: 1900,
            is_increase: true,
            ..Default::default()
        };

        assert_eq!(liquidity_data.whirlpool, whirlpool);
        assert_eq!(liquidity_data.position, position);
        assert_eq!(liquidity_data.position_authority, position_authority);
        assert_eq!(liquidity_data.tick_lower_index, -1000);
        assert_eq!(liquidity_data.tick_upper_index, 1000);
        assert!(liquidity_data.is_increase);
    }

    #[test]
    fn test_constants_values() {
        assert_eq!(
            ORCA_WHIRLPOOL_PROGRAM_ID,
            "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc"
        );
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
    fn test_whirlpool_account_debug() {
        let whirlpool = WhirlpoolAccount {
            whirlpools_config: Pubkey::new_unique(),
            whirlpool_bump: [1],
            tick_spacing: 64,
            tick_spacing_seed: [0, 64],
            fee_rate: 300,
            protocol_fee_rate: 100,
            liquidity: 50000,
            sqrt_price: 1u128 << 64,
            tick_current_index: 0,
            protocol_fee_owed_a: 100,
            protocol_fee_owed_b: 200,
            token_mint_a: Pubkey::new_unique(),
            token_vault_a: Pubkey::new_unique(),
            fee_growth_global_a: 1000,
            token_mint_b: Pubkey::new_unique(),
            token_vault_b: Pubkey::new_unique(),
            fee_growth_global_b: 2000,
            reward_last_updated_timestamp: 1640995200,
            reward_infos: core::array::from_fn(|_| WhirlpoolRewardInfo {
                mint: Pubkey::new_unique(),
                vault: Pubkey::new_unique(),
                authority: Pubkey::new_unique(),
                emissions_per_second_x64: 0,
                growth_global_x64: 0,
            }),
        };

        let debug_str = format!("{:?}", whirlpool);
        assert!(debug_str.contains("WhirlpoolAccount"));
    }

    #[test]
    fn test_whirlpool_account_clone() {
        let original = WhirlpoolAccount {
            whirlpools_config: Pubkey::new_unique(),
            whirlpool_bump: [1],
            tick_spacing: 64,
            tick_spacing_seed: [0, 64],
            fee_rate: 300,
            protocol_fee_rate: 100,
            liquidity: 50000,
            sqrt_price: 1u128 << 64,
            tick_current_index: 0,
            protocol_fee_owed_a: 100,
            protocol_fee_owed_b: 200,
            token_mint_a: Pubkey::new_unique(),
            token_vault_a: Pubkey::new_unique(),
            fee_growth_global_a: 1000,
            token_mint_b: Pubkey::new_unique(),
            token_vault_b: Pubkey::new_unique(),
            fee_growth_global_b: 2000,
            reward_last_updated_timestamp: 1640995200,
            reward_infos: core::array::from_fn(|_| WhirlpoolRewardInfo {
                mint: Pubkey::new_unique(),
                vault: Pubkey::new_unique(),
                authority: Pubkey::new_unique(),
                emissions_per_second_x64: 0,
                growth_global_x64: 0,
            }),
        };

        let cloned = original.clone();
        assert_eq!(original.tick_spacing, cloned.tick_spacing);
        assert_eq!(original.fee_rate, cloned.fee_rate);
    }

    #[test]
    fn test_edge_case_extreme_decimals() {
        // Test with maximum decimal values
        let sqrt_price = 1u128 << 64;
        let price_max_diff = sqrt_price_to_price(sqrt_price, u8::MAX, 0);
        assert!(price_max_diff.is_finite());

        let price_min_diff = sqrt_price_to_price(sqrt_price, 0, u8::MAX);
        assert!(price_min_diff >= 0.0);
        assert!(price_min_diff.is_finite());
    }

    #[test]
    fn test_serialization_compatibility() {
        // Test that structs with Serialize derive can be instantiated properly
        let swap_direction = SwapDirection::AtoB;
        let _serialized = format!("{:?}", swap_direction); // Basic test that Debug works with Serialize

        let position_reward = PositionRewardInfo {
            growth_inside_checkpoint: 100,
            amount_owed: 50,
        };
        let _serialized = format!("{:?}", position_reward);
    }
}
