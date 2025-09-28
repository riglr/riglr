//! Orca protocol event definitions and constants.

use core::any::Any;
#[cfg(test)]
use core::array;
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};
use solana_sdk::pubkey::Pubkey;
use std::{sync::OnceLock, time::SystemTime};

use crate::error::{Error as ParseError, ParseResult};
use crate::events::core::EventParameters;
use crate::events::factory::{InnerInstructionParseParams, InstructionParseParams};
use crate::metadata_helpers::{get_block_time, get_signature, get_slot};
use crate::solana_metadata::SolanaEventMetadata;
use crate::types::{EventType, ProtocolType, TransferData};
use riglr_events_core::error::{EventError, EventResult};
use riglr_events_core::{Event, EventFilter, EventKind, EventMetadata as CoreEventMetadata};

// Parser imports for additional parsing functionality
use crate::{
    events::{
        common::{
            has_discriminator, parse_u128_le, parse_u32_le, parse_u64_le, safe_get_account,
            validate_account_count, validate_data_length,
        },
        factory::SolanaTransactionInput,
        parser_types::{GenericEventParseConfig, ProtocolParser},
    },
    solana_metadata::create_metadata,
};
use riglr_events_core::traits::{EventParser as EventParserTrait, ParserInfo};
use std::collections::HashMap;

/// Orca Whirlpool program ID
pub const ORCA_WHIRLPOOL_PROGRAM_ID: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";

/// Orca Whirlpool instruction discriminators (calculated from Anchor's "global:<`instruction_name`>")
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

/// Orca Whirlpool account layout
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhirlpoolAccount {
    /// Global fee growth accumulator for token A as Q64.64 fixed-point
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub fee_growth_global_a: u128,
    /// Global fee growth accumulator for token B as Q64.64 fixed-point
    pub fee_growth_global_b: u128,
    /// Fee rate charged for swaps in basis points (e.g., 300 = 0.3%)
    pub fee_rate: u16,
    /// Total liquidity currently available in the pool
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub liquidity: u128,
    /// Protocol fees accumulated for token A awaiting collection
    pub protocol_fee_owed_a: u64,
    /// Protocol fees accumulated for token B awaiting collection
    pub protocol_fee_owed_b: u64,
    /// Protocol fee rate in basis points taken from the swap fee
    pub protocol_fee_rate: u16,
    /// Array of up to 3 reward token configurations for this pool
    pub reward_infos: [WhirlpoolRewardInfo; 3],
    /// Timestamp when reward calculations were last updated
    pub reward_last_updated_timestamp: u64,
    /// Square root of the current price as a Q64.64 fixed-point number
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub sqrt_price: u128,
    /// Current tick index representing the active price range
    pub tick_current_index: i32,
    /// The tick spacing for this whirlpool, determining price granularity
    pub tick_spacing: u16,
    /// Seed bytes used to derive the whirlpool account from tick spacing
    pub tick_spacing_seed: [u8; 2],
    /// Mint address of the first token in the trading pair
    pub token_mint_a: Pubkey,
    /// Mint address of the second token in the trading pair
    pub token_mint_b: Pubkey,
    /// Vault account holding the pool's token A reserves
    pub token_vault_a: Pubkey,
    /// Vault account holding the pool's token B reserves
    pub token_vault_b: Pubkey,
    /// Program-derived address bump seed for this whirlpool account
    pub whirlpool_bump: [u8; 1],
    /// Configuration account that governs this whirlpool's parameters
    pub whirlpools_config: Pubkey,
}

/// Whirlpool reward information
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WhirlpoolRewardInfo {
    /// Authority account that can control reward distribution parameters
    pub authority: Pubkey,
    /// Rate of reward token emissions per second as Q64.64 fixed-point
    pub emissions_per_second_x64: u128,
    /// Global growth accumulator for this reward token as Q64.64 fixed-point
    pub growth_global_x64: u128,
    /// Mint address of the reward token being distributed
    pub mint: Pubkey,
    /// Vault account holding the reward tokens for distribution
    pub vault: Pubkey,
}

/// Serialize u128 as string for JSON compatibility
#[inline]
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
    let string = String::deserialize(deserializer)?;
    string.parse().map_err(DeError::custom)
}

/// Orca swap event data
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwapData {
    /// Direction of the swap: true for A→B, false for B→A
    pub a_to_b: bool,
    /// The amount specified by the user for the swap operation
    pub amount: u64,
    /// Actual amount of tokens consumed in the swap
    pub amount_in: u64,
    /// Actual amount of tokens received from the swap
    pub amount_out: u64,
    /// Whether the specified amount represents input (true) or output (false)
    pub amount_specified_is_input: bool,
    /// Total fee amount charged for this swap transaction
    pub fee_amount: u64,
    /// Current liquidity in the pool after the swap
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub liquidity: u128,
    /// Current square root price after the swap as Q64.64 fixed-point
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub sqrt_price: u128,
    /// Price limit for the swap as square root price Q64.64 fixed-point
    #[serde(
        serialize_with = "serialize_u128_as_string",
        deserialize_with = "deserialize_u128_from_string"
    )]
    pub sqrt_price_limit: u128,
    /// Current tick index after the swap execution
    pub tick_current_index: i32,
    /// Mint address of the first token in the trading pair
    pub token_mint_a: Pubkey,
    /// Mint address of the second token in the trading pair
    pub token_mint_b: Pubkey,
    /// Vault account holding token A reserves for the pool
    pub token_vault_a: Pubkey,
    /// Vault account holding token B reserves for the pool
    pub token_vault_b: Pubkey,
    /// The user account that initiated the swap transaction
    pub user: Pubkey,
    /// The whirlpool account where the swap occurred
    pub whirlpool: Pubkey,
}

/// Orca position data
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PositionData {
    /// Fee growth checkpoint for token A when position was last updated
    pub fee_growth_checkpoint_a: u128,
    /// Fee growth checkpoint for token B when position was last updated
    pub fee_growth_checkpoint_b: u128,
    /// Accumulated fees owed to this position in token A
    pub fee_owed_a: u64,
    /// Accumulated fees owed to this position in token B
    pub fee_owed_b: u64,
    /// Amount of liquidity provided by this position
    pub liquidity: u128,
    /// Account address storing the position's state and parameters
    pub position: Pubkey,
    /// Authority account that can modify this position
    pub position_authority: Pubkey,
    /// Mint address for the NFT representing this liquidity position
    pub position_mint: Pubkey,
    /// Token account holding the position NFT
    pub position_token_account: Pubkey,
    /// Array of reward information for up to 3 reward tokens
    pub reward_infos: [PositionRewardInfo; 3],
    /// Lower tick boundary of the position's price range
    pub tick_lower_index: i32,
    /// Upper tick boundary of the position's price range
    pub tick_upper_index: i32,
    /// The whirlpool account this position provides liquidity to
    pub whirlpool: Pubkey,
}

/// Position reward information
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PositionRewardInfo {
    /// Amount of reward tokens owed to this position
    pub amount_owed: u64,
    /// Reward growth checkpoint inside the position's tick range
    pub growth_inside_checkpoint: u128,
}

/// Orca liquidity change data
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LiquidityData {
    /// Whether this is a liquidity increase (true) or decrease (false)
    pub is_increase: bool,
    /// Amount of liquidity being added or removed
    pub liquidity_amount: u128,
    /// The position account being modified
    pub position: Pubkey,
    /// Authority account that can modify the position
    pub position_authority: Pubkey,
    /// Lower tick boundary of the position's price range
    pub tick_lower_index: i32,
    /// Upper tick boundary of the position's price range
    pub tick_upper_index: i32,
    /// Actual amount of token A deposited/withdrawn
    pub token_actual_a: u64,
    /// Actual amount of token B deposited/withdrawn
    pub token_actual_b: u64,
    /// Maximum amount of token A willing to deposit/withdraw
    pub token_max_a: u64,
    /// Maximum amount of token B willing to deposit/withdraw
    pub token_max_b: u64,
    /// Mint address of the first token in the trading pair
    pub token_mint_a: Pubkey,
    /// Mint address of the second token in the trading pair
    pub token_mint_b: Pubkey,
    /// Vault account holding token A reserves for the pool
    pub token_vault_a: Pubkey,
    /// Vault account holding token B reserves for the pool
    pub token_vault_b: Pubkey,
    /// The whirlpool account where liquidity is being modified
    pub whirlpool: Pubkey,
}

/// Orca swap direction
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SwapDirection {
    /// Swap from token A to token B
    AtoB,
    /// Swap from token B to token A
    BtoA,
}

/// Extract Orca Whirlpool program ID as Pubkey
///
/// This uses a static lazy-evaluated Pubkey to avoid repeated parsing.
static ORCA_WHIRLPOOL_PUBKEY: OnceLock<Pubkey> = OnceLock::new();

#[must_use]
#[inline]
/// Returns the Orca Whirlpool program ID.
///
/// # Panics
/// Panics if the Orca Whirlpool program ID constant is invalid,
/// which should never happen as it's a hardcoded valid constant.
pub fn whirlpool_program_id() -> Pubkey {
    *ORCA_WHIRLPOOL_PUBKEY.get_or_init(|| {
        Pubkey::try_from(ORCA_WHIRLPOOL_PROGRAM_ID).unwrap_or_else(|_| Pubkey::default())
    })
}

/// Check if the given pubkey is Orca Whirlpool program
#[must_use]
#[inline]
pub fn is_orca_whirlpool_program(program_id: &Pubkey) -> bool {
    *program_id == whirlpool_program_id()
}

/// Convert tick index to price
#[must_use]
#[inline]
pub fn tick_index_to_price(tick_index: i32) -> f64 {
    // Using a safer method to calculate the power
    let base = 1.0001f64;
    if tick_index >= 0 {
        base.powi(tick_index)
    } else {
        1.0 / base.powi(tick_index.wrapping_neg())
    }
}

/// Orca Whirlpool swap event
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwapEvent {
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    pub metadata: SolanaEventMetadata,
    /// Orca-specific swap data
    pub swap_data: SwapData,
    /// Associated token transfer data
    pub transfer_data: Vec<TransferData>,
}

impl SwapEvent {
    /// Creates a new `SwapEvent` with the provided parameters and swap data
    #[must_use]
    #[inline]
    pub fn new(params: EventParameters, swap_data: SwapData) -> Self {
        let metadata = riglr_events_core::EventMetadata::new(
            params.id.clone(),
            EventKind::Swap,
            "solana-orca".to_owned(),
        );

        let solana_metadata = SolanaEventMetadata::new(
            params.signature,
            params.slot,
            EventType::Swap,
            ProtocolType::OrcaWhirlpool,
            params.index,
            params.program_received_time_ms,
            metadata,
        );

        Self {
            metadata: solana_metadata,
            swap_data,
            transfer_data: Vec::default(),
        }
    }

    /// Sets the transfer data for this swap event
    #[must_use]
    #[inline]
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

/// Convert sqrt price to price
#[must_use]
#[inline]
pub fn sqrt_price_to_price(sqrt_price: u128, decimals_a: u8, decimals_b: u8) -> f64 {
    // Convert u128 to f64 using a safer method
    #[expect(clippy::cast_precision_loss)]
    let sqrt_price_f64 = sqrt_price as f64;
    #[expect(clippy::cast_precision_loss)]
    let divisor_f64 = (1u128 << 64) as f64;

    let price_x64 = (sqrt_price_f64 / divisor_f64).powi(2);

    let decimal_difference = i32::from(decimals_a)
        .checked_sub(i32::from(decimals_b))
        .unwrap_or({
            // Log an error or handle the case where subtraction overflows
            0
        });

    let decimal_adjustment = 10f64.powi(decimal_difference);
    price_x64 * decimal_adjustment
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod types_tests {
    use super::*;
    use core::str::FromStr;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_whirlpool_program_id_should_return_valid_pubkey() {
        let program_id = whirlpool_program_id();
        let expected = Pubkey::from_str(ORCA_WHIRLPOOL_PROGRAM_ID).unwrap();
        assert_eq!(program_id, expected);
    }

    #[test]
    fn test_is_orca_whirlpool_program_when_correct_program_id_should_return_true() {
        let program_id = whirlpool_program_id();
        assert!(is_orca_whirlpool_program(&program_id));
    }

    #[test]
    fn test_is_orca_whirlpool_program_when_different_program_id_should_return_false() {
        let different_program_id = Pubkey::new_unique();
        assert!(!is_orca_whirlpool_program(&different_program_id));
    }

    #[test]
    fn test_tick_index_to_price_when_zero_should_return_one() {
        assert!((tick_index_to_price(0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tick_index_to_price_when_positive_should_return_greater_than_one() {
        let price = tick_index_to_price(1000);
        assert!(price > 1.0);
        assert!((price - 1.105_165_392_603_197).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tick_index_to_price_when_negative_should_return_less_than_one() {
        let price = tick_index_to_price(-1000);
        assert!(price < 1.0);
        assert!((price - 0.904_841_941_932_798_1).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tick_index_to_price_when_max_value_should_not_panic() {
        let price = tick_index_to_price(i32::MAX);
        assert!(price.is_infinite() && price.is_sign_positive());
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
        assert!(price.abs() < f64::EPSILON);
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
        let debug_str = format!("{direction:?}");
        assert_eq!(debug_str, "AtoB");

        let direction = SwapDirection::BtoA;
        let debug_str = format!("{direction:?}");
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
            reward_last_updated_timestamp: 1_640_995_200,
            reward_infos: array::from_fn(|_| reward_info),
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
        let swap_data = SwapData::default();
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

        let swap_data = SwapData {
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
        let position_data = PositionData::default();
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
        let liquidity_data = LiquidityData::default();
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

        let liquidity_data = LiquidityData {
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
            reward_last_updated_timestamp: 1_640_995_200,
            reward_infos: array::from_fn(|_| WhirlpoolRewardInfo {
                mint: Pubkey::new_unique(),
                vault: Pubkey::new_unique(),
                authority: Pubkey::new_unique(),
                emissions_per_second_x64: 0,
                growth_global_x64: 0,
            }),
        };

        let debug_str = format!("{whirlpool:?}");
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
            reward_last_updated_timestamp: 1_640_995_200,
            reward_infos: array::from_fn(|_| WhirlpoolRewardInfo {
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
        let _serialized = format!("{swap_direction:?}"); // Basic test that Debug works with Serialize

        let position_reward = PositionRewardInfo {
            growth_inside_checkpoint: 100,
            amount_owed: 50,
        };
        let _serialized = format!("{position_reward:?}");
    }
}

// ================================
// Events Module
// ================================

// Types are defined above in this module

/// Orca position event (open/close)
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PositionEvent {
    /// Whether the position is being opened (true) or closed (false)
    pub is_open: bool,
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    pub metadata: SolanaEventMetadata,
    /// Orca-specific position data
    pub position_data: PositionData,
    /// Associated token transfer data
    pub transfer_data: Vec<TransferData>,
}

impl PositionEvent {
    /// Creates a new `PositionEvent` with the provided parameters and position data
    #[must_use]
    #[inline]
    pub fn new(params: EventParameters, position_data: PositionData, is_open: bool) -> Self {
        let event_type = if is_open {
            EventType::OpenPosition
        } else {
            EventType::ClosePosition
        };

        let metadata = riglr_events_core::EventMetadata::new(
            params.id.clone(),
            EventKind::Custom("position".to_owned()),
            "solana-orca".to_owned(),
        );

        let solana_metadata = SolanaEventMetadata::new(
            params.signature,
            params.slot,
            event_type,
            ProtocolType::OrcaWhirlpool,
            params.index,
            params.program_received_time_ms,
            metadata,
        );

        Self {
            is_open,
            metadata: solana_metadata,
            position_data,
            transfer_data: Vec::default(),
        }
    }

    /// Sets the transfer data for this position event
    #[must_use]
    #[inline]
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

/// Orca liquidity event (increase/decrease)
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LiquidityEvent {
    /// Orca-specific liquidity data
    pub liquidity_data: LiquidityData,
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    pub metadata: SolanaEventMetadata,
    /// Associated token transfer data
    pub transfer_data: Vec<TransferData>,
}

impl LiquidityEvent {
    /// Creates a new `LiquidityEvent` with the provided parameters and liquidity data
    #[must_use]
    #[inline]
    pub fn new(params: EventParameters, liquidity_data: LiquidityData) -> Self {
        let metadata = riglr_events_core::EventMetadata::new(
            params.id.clone(),
            EventKind::Custom("liquidity".to_owned()),
            "solana-orca".to_owned(),
        );

        let solana_metadata = SolanaEventMetadata::new(
            params.signature,
            params.slot,
            EventType::AddLiquidity,
            ProtocolType::OrcaWhirlpool,
            params.index,
            params.program_received_time_ms,
            metadata,
        );

        Self {
            liquidity_data,
            metadata: solana_metadata,
            transfer_data: Vec::default(),
        }
    }

    /// Sets the transfer data for this liquidity event
    #[must_use]
    #[inline]
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

// Event trait implementation for SwapEvent
impl Event for SwapEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn id(&self) -> &str {
        self.metadata.id()
    }

    fn kind(&self) -> &EventKind {
        self.metadata.kind()
    }

    fn matches_filter(&self, filter: &dyn EventFilter) -> bool {
        filter.matches(self)
    }

    fn metadata(&self) -> &CoreEventMetadata {
        self.metadata.core()
    }

    fn metadata_mut(&mut self) -> EventResult<&mut CoreEventMetadata> {
        Ok(self.metadata.core_mut())
    }

    fn source(&self) -> &str {
        &self.metadata.source
    }

    fn timestamp(&self) -> SystemTime {
        self.metadata.timestamp.into()
    }

    fn to_json(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self)
            .map_err(|e| EventError::generic(format!("Serialization failed: {e}")))
    }
}

// Event trait implementation for PositionEvent
impl Event for PositionEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn id(&self) -> &str {
        self.metadata.id()
    }

    fn kind(&self) -> &EventKind {
        self.metadata.kind()
    }

    fn matches_filter(&self, filter: &dyn EventFilter) -> bool {
        filter.matches(self)
    }

    fn metadata(&self) -> &CoreEventMetadata {
        self.metadata.core()
    }

    fn metadata_mut(&mut self) -> EventResult<&mut CoreEventMetadata> {
        Ok(self.metadata.core_mut())
    }

    fn source(&self) -> &str {
        &self.metadata.source
    }

    fn timestamp(&self) -> SystemTime {
        self.metadata.timestamp.into()
    }

    fn to_json(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self)
            .map_err(|e| EventError::generic(format!("Serialization failed: {e}")))
    }
}

// Event trait implementation for LiquidityEvent
impl Event for LiquidityEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn id(&self) -> &str {
        self.metadata.id()
    }

    fn kind(&self) -> &EventKind {
        self.metadata.kind()
    }

    fn matches_filter(&self, filter: &dyn EventFilter) -> bool {
        filter.matches(self)
    }

    fn metadata(&self) -> &CoreEventMetadata {
        self.metadata.core()
    }

    fn metadata_mut(&mut self) -> EventResult<&mut CoreEventMetadata> {
        Ok(self.metadata.core_mut())
    }

    fn source(&self) -> &str {
        &self.metadata.source
    }

    fn timestamp(&self) -> SystemTime {
        self.metadata.timestamp.into()
    }

    fn to_json(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self)
            .map_err(|e| EventError::generic(format!("Serialization failed: {e}")))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod events_tests {
    use super::*;
    // PositionRewardInfo available through super::*
    use core::str::FromStr;
    use solana_sdk::pubkey::Pubkey;

    // Helper function to create test EventParameters
    fn create_test_event_parameters() -> EventParameters {
        EventParameters::new(
            "test-id".to_owned(),
            "test-signature".to_owned(),
            12345,
            164_099,
            164_099_000,
            164_099_001,
            "0".to_owned(),
        )
    }

    // Helper function to create test SwapData
    fn create_test_swap_data() -> SwapData {
        SwapData {
            whirlpool: Pubkey::from_str("11111111111111111111111111111112")
                .expect("Valid test pubkey"),
            user: Pubkey::from_str("11111111111111111111111111111113").expect("Valid test pubkey"),
            token_mint_a: Pubkey::from_str("11111111111111111111111111111114")
                .expect("Valid test pubkey"),
            token_mint_b: Pubkey::from_str("11111111111111111111111111111115")
                .expect("Valid test pubkey"),
            token_vault_a: Pubkey::from_str("11111111111111111111111111111116")
                .expect("Valid test pubkey"),
            token_vault_b: Pubkey::from_str("11111111111111111111111111111117")
                .expect("Valid test pubkey"),
            amount: 1000,
            amount_specified_is_input: true,
            a_to_b: true,
            sqrt_price_limit: 123_456_789,
            amount_in: 1000,
            amount_out: 950,
            fee_amount: 3,
            tick_current_index: 100,
            sqrt_price: 98_765_432_109_876_543_210,
            liquidity: 50000,
        }
    }

    // Helper function to create test PositionData
    fn create_test_position_data() -> PositionData {
        PositionData {
            whirlpool: Pubkey::from_str("11111111111111111111111111111112")
                .expect("Valid test pubkey"),
            position_mint: Pubkey::from_str("11111111111111111111111111111113")
                .expect("Valid test pubkey"),
            position: Pubkey::from_str("11111111111111111111111111111114")
                .expect("Valid test pubkey"),
            position_token_account: Pubkey::from_str("11111111111111111111111111111115")
                .expect("Valid test pubkey"),
            position_authority: Pubkey::from_str("11111111111111111111111111111116")
                .expect("Valid test pubkey"),
            tick_lower_index: -1000,
            tick_upper_index: 1000,
            liquidity: 25000,
            fee_growth_checkpoint_a: 123_456_789,
            fee_growth_checkpoint_b: 987_654_321,
            fee_owed_a: 100,
            fee_owed_b: 200,
            reward_infos: [PositionRewardInfo::default(); 3],
        }
    }

    // Helper function to create test LiquidityData
    fn create_test_liquidity_data() -> LiquidityData {
        LiquidityData {
            whirlpool: Pubkey::from_str("11111111111111111111111111111112")
                .expect("Valid test pubkey"),
            position: Pubkey::from_str("11111111111111111111111111111113")
                .expect("Valid test pubkey"),
            position_authority: Pubkey::from_str("11111111111111111111111111111114")
                .expect("Valid test pubkey"),
            token_mint_a: Pubkey::from_str("11111111111111111111111111111115")
                .expect("Valid test pubkey"),
            token_mint_b: Pubkey::from_str("11111111111111111111111111111116")
                .expect("Valid test pubkey"),
            token_vault_a: Pubkey::from_str("11111111111111111111111111111117")
                .expect("Valid test pubkey"),
            token_vault_b: Pubkey::from_str("11111111111111111111111111111118")
                .expect("Valid test pubkey"),
            tick_lower_index: -500,
            tick_upper_index: 500,
            liquidity_amount: 10000,
            token_max_a: 1000,
            token_max_b: 2000,
            token_actual_a: 950,
            token_actual_b: 1900,
            is_increase: true,
        }
    }

    // Helper function to create test TransferData
    fn create_test_transfer_data() -> Vec<TransferData> {
        vec![
            TransferData {
                source: Pubkey::from_str("11111111111111111111111111111112")
                    .expect("Valid test pubkey"),
                destination: Pubkey::from_str("11111111111111111111111111111113")
                    .expect("Valid test pubkey"),
                mint: Some(
                    Pubkey::from_str("11111111111111111111111111111114")
                        .expect("Valid test pubkey"),
                ),
                amount: 1000,
            },
            TransferData {
                source: Pubkey::from_str("11111111111111111111111111111115")
                    .expect("Valid test pubkey"),
                destination: Pubkey::from_str("11111111111111111111111111111116")
                    .expect("Valid test pubkey"),
                mint: None,
                amount: 500,
            },
        ]
    }

    // Tests for EventParameters
    #[test]
    fn test_event_parameters_new() {
        let params = EventParameters::new(
            "id1".to_string(),
            "sig1".to_string(),
            100,
            1000,
            1_000_000,
            1_000_001,
            "1".to_string(),
        );

        assert_eq!(params.id, "id1");
        assert_eq!(params.signature, "sig1");
        assert_eq!(params.slot, 100);
        assert_eq!(params.block_time, 1000);
        assert_eq!(params.block_time_ms, 1_000_000);
        assert_eq!(params.program_received_time_ms, 1_000_001);
        assert_eq!(params.index, "1");
    }

    #[test]
    fn test_event_parameters_default() {
        let params = EventParameters::default();

        assert_eq!(params.id, "");
        assert_eq!(params.signature, "");
        assert_eq!(params.slot, 0);
        assert_eq!(params.block_time, 0);
        assert_eq!(params.block_time_ms, 0);
        assert_eq!(params.program_received_time_ms, 0);
        assert_eq!(params.index, "");
    }

    #[test]
    fn test_event_parameters_debug() {
        let params = create_test_event_parameters();
        let debug_string = format!("{params:?}");
        assert!(debug_string.contains("EventParameters"));
    }

    #[test]
    fn test_event_parameters_clone() {
        let params = create_test_event_parameters();
        let cloned = params.clone();

        assert_eq!(params.id, cloned.id);
        assert_eq!(params.signature, cloned.signature);
        assert_eq!(params.slot, cloned.slot);
        assert_eq!(params.block_time, cloned.block_time);
        assert_eq!(params.block_time_ms, cloned.block_time_ms);
        assert_eq!(
            params.program_received_time_ms,
            cloned.program_received_time_ms
        );
        assert_eq!(params.index, cloned.index);
    }

    // Tests for SwapEvent
    #[test]
    fn test_orca_swap_event_new() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let event = SwapEvent::new(params.clone(), swap_data.clone());

        assert_eq!(event.metadata.id(), &params.id);
        assert_eq!(event.swap_data.whirlpool, swap_data.whirlpool);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_orca_swap_event_with_transfer_data() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let transfer_data = create_test_transfer_data();

        let event = SwapEvent::new(params, swap_data).with_transfer_data(transfer_data);

        assert_eq!(event.transfer_data.len(), 2);
        assert_eq!(event.transfer_data.first().map(|t| t.amount), Some(1000));
        assert_eq!(event.transfer_data.get(1).map(|t| t.amount), Some(500));
    }

    #[test]
    fn test_orca_swap_event_with_empty_transfer_data() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();

        let event = SwapEvent::new(params, swap_data).with_transfer_data(vec![]);

        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_orca_swap_event_default() {
        let event = SwapEvent::default();

        assert_eq!(event.metadata.id(), "");
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_orca_swap_event_clone() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let transfer_data = create_test_transfer_data();

        let event = SwapEvent::new(params, swap_data).with_transfer_data(transfer_data);
        let cloned = event.clone();

        assert_eq!(event.metadata.id(), cloned.metadata.id());
        assert_eq!(event.swap_data.whirlpool, cloned.swap_data.whirlpool);
        assert_eq!(event.transfer_data.len(), cloned.transfer_data.len());
    }

    #[test]
    fn test_orca_swap_event_serialization() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let event = SwapEvent::new(params, swap_data);

        let serialized = serde_json::to_string(&event).expect("Test serialization should succeed");
        let deserialized: SwapEvent =
            serde_json::from_str(&serialized).expect("Test deserialization should succeed");

        // Note: metadata field is skipped in serialization, so we only compare other fields
        assert_eq!(event.swap_data.whirlpool, deserialized.swap_data.whirlpool);
    }

    // Tests for PositionEvent
    #[test]
    fn test_orca_position_event_new_open() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let event = PositionEvent::new(params.clone(), position_data.clone(), true);

        assert_eq!(event.metadata.core.id, params.id);
        assert_eq!(event.position_data.whirlpool, position_data.whirlpool);
        assert!(event.is_open);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_orca_position_event_new_close() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let event = PositionEvent::new(params, position_data, false);

        assert!(!event.is_open);
    }

    #[test]
    fn test_orca_position_event_with_transfer_data() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let transfer_data = create_test_transfer_data();

        let event =
            PositionEvent::new(params, position_data, true).with_transfer_data(transfer_data);

        assert_eq!(event.transfer_data.len(), 2);
        assert_eq!(event.transfer_data.first().map(|t| t.amount), Some(1000));
        assert_eq!(event.transfer_data.get(1).map(|t| t.amount), Some(500));
    }

    #[test]
    fn test_orca_position_event_with_empty_transfer_data() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();

        let event = PositionEvent::new(params, position_data, true).with_transfer_data(vec![]);

        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_orca_position_event_default() {
        let event = PositionEvent::default();

        assert_eq!(event.id(), "");
        assert_eq!(event.metadata.signature, "");
        assert_eq!(event.metadata.slot, 0);
        assert_eq!(get_block_time(&event.metadata.core).unwrap_or(0), 0);
        assert_eq!(get_block_time(&event.metadata.core).unwrap_or(0) * 1000, 0);
        assert_eq!(event.metadata.program_received_time_ms, 0);
        assert_eq!(event.metadata.index, "");
        assert!(!event.is_open);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_orca_position_event_clone() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let transfer_data = create_test_transfer_data();

        let event =
            PositionEvent::new(params, position_data, true).with_transfer_data(transfer_data);
        let cloned = event.clone();

        assert_eq!(event.id(), cloned.id());
        assert_eq!(event.is_open, cloned.is_open);
        assert_eq!(
            event.position_data.whirlpool,
            cloned.position_data.whirlpool
        );
        assert_eq!(event.transfer_data.len(), cloned.transfer_data.len());
    }

    #[test]
    fn test_orca_position_event_serialization() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let event = PositionEvent::new(params, position_data, true);

        let serialized = serde_json::to_string(&event).expect("Test serialization should succeed");
        let deserialized: PositionEvent =
            serde_json::from_str(&serialized).expect("Test deserialization should succeed");

        // Note: metadata field is skipped in serialization, so we only compare other fields
        assert_eq!(event.is_open, deserialized.is_open);
        assert_eq!(
            event.position_data.whirlpool,
            deserialized.position_data.whirlpool
        );
    }

    // Tests for LiquidityEvent
    #[test]
    fn test_orca_liquidity_event_new() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let event = LiquidityEvent::new(params.clone(), liquidity_data.clone());

        assert_eq!(event.id(), &params.id);
        assert_eq!(event.metadata.signature, params.signature);
        assert_eq!(event.metadata.slot, params.slot);
        assert_eq!(
            get_block_time(&event.metadata.core).unwrap_or(0),
            params.block_time
        );
        assert_eq!(
            get_block_time(&event.metadata.core).unwrap_or(0) * 1000,
            params.block_time_ms
        );
        assert_eq!(
            event.metadata.program_received_time_ms,
            params.program_received_time_ms
        );
        assert_eq!(event.metadata.index, params.index);
        assert_eq!(event.liquidity_data.whirlpool, liquidity_data.whirlpool);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_orca_liquidity_event_with_transfer_data() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let transfer_data = create_test_transfer_data();

        let event = LiquidityEvent::new(params, liquidity_data).with_transfer_data(transfer_data);

        assert_eq!(event.transfer_data.len(), 2);
        assert_eq!(event.transfer_data.first().map(|t| t.amount), Some(1000));
        assert_eq!(event.transfer_data.get(1).map(|t| t.amount), Some(500));
    }

    #[test]
    fn test_orca_liquidity_event_with_empty_transfer_data() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();

        let event = LiquidityEvent::new(params, liquidity_data).with_transfer_data(vec![]);

        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_orca_liquidity_event_default() {
        let event = LiquidityEvent::default();

        assert_eq!(event.id(), "");
        assert_eq!(event.metadata.signature, "");
        assert_eq!(event.metadata.slot, 0);
        assert_eq!(get_block_time(&event.metadata.core).unwrap_or(0), 0);
        assert_eq!(get_block_time(&event.metadata.core).unwrap_or(0) * 1000, 0);
        assert_eq!(event.metadata.program_received_time_ms, 0);
        assert_eq!(event.metadata.index, "");
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_orca_liquidity_event_clone() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let transfer_data = create_test_transfer_data();

        let event = LiquidityEvent::new(params, liquidity_data).with_transfer_data(transfer_data);
        let cloned = event.clone();

        assert_eq!(event.id(), cloned.id());
        assert_eq!(
            event.liquidity_data.whirlpool,
            cloned.liquidity_data.whirlpool
        );
        assert_eq!(event.transfer_data.len(), cloned.transfer_data.len());
    }

    #[test]
    fn test_orca_liquidity_event_serialization() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let event = LiquidityEvent::new(params, liquidity_data);

        let serialized = serde_json::to_string(&event).expect("Test serialization should succeed");
        let deserialized: LiquidityEvent =
            serde_json::from_str(&serialized).expect("Test deserialization should succeed");

        // Note: metadata field is skipped in serialization, so we only compare other fields
        assert_eq!(
            event.liquidity_data.whirlpool,
            deserialized.liquidity_data.whirlpool
        );
    }

    // Tests for Event trait implementation on SwapEvent
    #[test]
    fn test_orca_swap_event_trait_id() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let event = SwapEvent::new(params, swap_data);

        assert_eq!(event.id(), event.metadata.id());
    }

    #[test]
    fn test_orca_swap_event_trait_kind() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let event = SwapEvent::new(params, swap_data);

        assert_eq!(event.kind(), event.metadata.kind());
    }

    #[test]
    fn test_orca_swap_event_trait_metadata() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let event = SwapEvent::new(params, swap_data);

        let metadata = event.metadata();
        assert_eq!(metadata, &event.metadata.core);
    }

    #[test]
    fn test_orca_swap_event_trait_metadata_mut() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let mut event = SwapEvent::new(params, swap_data);

        let metadata_mut = event.metadata_mut().expect("Metadata should be accessible");
        metadata_mut.id = "updated-swap-id".to_string();

        assert_eq!(event.metadata.id(), "updated-swap-id");
    }

    #[test]
    fn test_orca_swap_event_trait_as_any() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let event = SwapEvent::new(params, swap_data);

        let any = event.as_any();
        assert!(any.downcast_ref::<SwapEvent>().is_some());
    }

    #[test]
    fn test_orca_swap_event_trait_as_any_mut() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let mut event = SwapEvent::new(params, swap_data);

        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<SwapEvent>().is_some());
    }

    #[test]
    fn test_orca_swap_event_trait_clone_boxed() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let event = SwapEvent::new(params, swap_data);

        let boxed = event.clone_boxed();
        let downcast = boxed
            .as_any()
            .downcast_ref::<SwapEvent>()
            .expect("Downcast should succeed");
        assert_eq!(event.id(), downcast.id());
    }

    #[test]
    fn test_orca_swap_event_trait_to_json() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let event = SwapEvent::new(params, swap_data);

        let json_result = event.to_json();
        assert!(json_result.is_ok());
        let json_value = json_result.expect("JSON conversion should succeed");
        assert!(json_value.is_object());
    }

    // Tests for Event trait implementation on PositionEvent
    #[test]
    fn test_orca_position_event_trait_id() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let event = PositionEvent::new(params, position_data, true);

        assert_eq!(event.id(), event.metadata.id());
    }

    #[test]
    fn test_orca_position_event_trait_kind() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let event = PositionEvent::new(params, position_data, true);

        assert_eq!(event.kind(), event.metadata.kind());
    }

    #[test]
    fn test_orca_position_event_trait_metadata() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let event = PositionEvent::new(params, position_data, true);

        let metadata = event.metadata();
        assert_eq!(metadata, &event.metadata.core);
    }

    #[test]
    fn test_orca_position_event_trait_metadata_mut() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let mut event = PositionEvent::new(params, position_data, true);

        let metadata_mut = event.metadata_mut().expect("Metadata should be accessible");
        metadata_mut.id = "updated-position-id".to_string();

        assert_eq!(event.metadata.id(), "updated-position-id");
    }

    #[test]
    fn test_orca_position_event_trait_as_any() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let event = PositionEvent::new(params, position_data, true);

        let any = event.as_any();
        assert!(any.downcast_ref::<PositionEvent>().is_some());
    }

    #[test]
    fn test_orca_position_event_trait_as_any_mut() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let mut event = PositionEvent::new(params, position_data, true);

        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<PositionEvent>().is_some());
    }

    #[test]
    fn test_orca_position_event_trait_clone_boxed() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let event = PositionEvent::new(params, position_data, true);

        let boxed = event.clone_boxed();
        let downcast = boxed
            .as_any()
            .downcast_ref::<PositionEvent>()
            .expect("Downcast should succeed");
        assert_eq!(event.id(), downcast.id());
    }

    #[test]
    fn test_orca_position_event_trait_to_json() {
        let params = create_test_event_parameters();
        let position_data = create_test_position_data();
        let event = PositionEvent::new(params, position_data, true);

        let json_result = event.to_json();
        assert!(json_result.is_ok());
        let json_value = json_result.expect("JSON conversion should succeed");
        assert!(json_value.is_object());
    }

    // Tests for Event trait implementation on LiquidityEvent
    #[test]
    fn test_orca_liquidity_event_trait_id() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let event = LiquidityEvent::new(params, liquidity_data);

        assert_eq!(event.id(), event.metadata.id());
    }

    #[test]
    fn test_orca_liquidity_event_trait_kind() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let event = LiquidityEvent::new(params, liquidity_data);

        assert_eq!(event.kind(), event.metadata.kind());
    }

    #[test]
    fn test_orca_liquidity_event_trait_metadata() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let event = LiquidityEvent::new(params, liquidity_data);

        let metadata = event.metadata();
        assert_eq!(metadata, &event.metadata.core);
    }

    #[test]
    fn test_orca_liquidity_event_trait_metadata_mut() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let mut event = LiquidityEvent::new(params, liquidity_data);

        let metadata_mut = event.metadata_mut().expect("Metadata should be accessible");
        metadata_mut.id = "updated-liquidity-id".to_string();

        assert_eq!(event.metadata.id(), "updated-liquidity-id");
    }

    #[test]
    fn test_orca_liquidity_event_trait_as_any() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let event = LiquidityEvent::new(params, liquidity_data);

        let any = event.as_any();
        assert!(any.downcast_ref::<LiquidityEvent>().is_some());
    }

    #[test]
    fn test_orca_liquidity_event_trait_as_any_mut() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let mut event = LiquidityEvent::new(params, liquidity_data);

        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<LiquidityEvent>().is_some());
    }

    #[test]
    fn test_orca_liquidity_event_trait_clone_boxed() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let event = LiquidityEvent::new(params, liquidity_data);

        let boxed = event.clone_boxed();
        let downcast = boxed
            .as_any()
            .downcast_ref::<LiquidityEvent>()
            .expect("Downcast should succeed");
        assert_eq!(event.id(), downcast.id());
    }

    #[test]
    fn test_orca_liquidity_event_trait_to_json() {
        let params = create_test_event_parameters();
        let liquidity_data = create_test_liquidity_data();
        let event = LiquidityEvent::new(params, liquidity_data);

        let json_result = event.to_json();
        assert!(json_result.is_ok());
        let json_value = json_result.expect("JSON conversion should succeed");
        assert!(json_value.is_object());
    }

    // Edge case tests
    #[test]
    fn test_event_parameters_with_empty_strings() {
        let params = EventParameters::new(String::new(), String::new(), 0, 0, 0, 0, String::new());

        assert_eq!(params.id, "");
        assert_eq!(params.signature, "");
        assert_eq!(params.index, "");
    }

    #[test]
    fn test_event_parameters_with_max_values() {
        let params = EventParameters::new(
            "max-id".to_string(),
            "max-signature".to_string(),
            u64::MAX,
            i64::MAX,
            i64::MAX,
            i64::MAX,
            "max-index".to_string(),
        );

        assert_eq!(params.slot, u64::MAX);
        assert_eq!(params.block_time, i64::MAX);
        assert_eq!(params.block_time_ms, i64::MAX);
        assert_eq!(params.program_received_time_ms, i64::MAX);
    }

    #[test]
    fn test_event_parameters_with_negative_values() {
        let params = EventParameters::new(
            "neg-id".to_string(),
            "neg-signature".to_string(),
            0,
            i64::MIN,
            i64::MIN,
            i64::MIN,
            "neg-index".to_string(),
        );

        assert_eq!(params.block_time, i64::MIN);
        assert_eq!(params.block_time_ms, i64::MIN);
        assert_eq!(params.program_received_time_ms, i64::MIN);
    }

    #[test]
    fn test_orca_swap_event_with_multiple_transfers() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let mut transfer_data = create_test_transfer_data();

        // Add more transfers
        for i in 3..10 {
            transfer_data.push(TransferData {
                source: Pubkey::from_str("11111111111111111111111111111112")
                    .expect("Valid test pubkey"),
                destination: Pubkey::from_str("11111111111111111111111111111113")
                    .expect("Valid test pubkey"),
                mint: None,
                amount: i * 100,
            });
        }

        let event = SwapEvent::new(params, swap_data).with_transfer_data(transfer_data.clone());

        assert_eq!(event.transfer_data.len(), transfer_data.len());
    }

    #[test]
    fn test_debug_format_implementations() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let position_data = create_test_position_data();
        let liquidity_data = create_test_liquidity_data();

        let swap_event = SwapEvent::new(params.clone(), swap_data);
        let position_event = PositionEvent::new(params.clone(), position_data, true);
        let liquidity_event = LiquidityEvent::new(params, liquidity_data);

        // Test that debug formatting works
        let swap_debug = format!("{swap_event:?}");
        let position_debug = format!("{position_event:?}");
        let liquidity_debug = format!("{liquidity_event:?}");

        assert!(swap_debug.contains("SwapEvent"));
        assert!(position_debug.contains("PositionEvent"));
        assert!(liquidity_debug.contains("LiquidityEvent"));
    }

    // Test chaining methods
    #[test]
    fn test_method_chaining() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let transfer_data = create_test_transfer_data();

        let event = SwapEvent::new(params, swap_data)
            .with_transfer_data(transfer_data)
            .with_transfer_data(vec![]); // Chain multiple calls

        assert!(event.transfer_data.is_empty()); // Last call should override
    }

    // Test that metadata field is excluded from serialization
    #[test]
    fn test_metadata_field_excluded_from_serialization() {
        let params = create_test_event_parameters();
        let swap_data = create_test_swap_data();
        let event = SwapEvent::new(params, swap_data);

        let serialized = serde_json::to_string(&event).expect("Test serialization should succeed");
        assert!(!serialized.contains("metadata"));
    }
}

// ================================
// Parser Module
// ================================

/// Orca Whirlpool event parser
#[non_exhaustive]
#[derive(Debug)]
pub struct Parser {
    info: ParserInfo,
    inner_instruction_configs: HashMap<&'static str, Vec<GenericEventParseConfig>>,
    instruction_configs: HashMap<Vec<u8>, Vec<GenericEventParseConfig>>,
    program_ids: Vec<Pubkey>,
}

#[async_trait::async_trait]
impl EventParserTrait for Parser {
    type Input = SolanaTransactionInput;
    fn can_parse(&self, input: &Self::Input) -> bool {
        match *input {
            SolanaTransactionInput::InnerInstruction(_) => {
                // Can always attempt to parse inner instructions for configured protocols
                !self.inner_instruction_configs.is_empty()
            }
            SolanaTransactionInput::Instruction(ref params) => {
                // Check if any discriminator matches
                self.instruction_configs
                    .keys()
                    .any(|discriminator| has_discriminator(&params.instruction_data, discriminator))
            }
        }
    }
    fn info(&self) -> &ParserInfo {
        &self.info
    }

    async fn parse(&self, input: Self::Input) -> EventResult<Vec<Box<dyn Event>>> {
        let mut events = Vec::new();

        match input {
            SolanaTransactionInput::InnerInstruction(params) => {
                // For inner instructions, we'll use the data to identify the instruction type
                if let Ok(data) = bs58::decode(&params.inner_instruction_data).into_vec() {
                    for configs in self.inner_instruction_configs.values() {
                        for config in configs {
                            let metadata = create_metadata(
                                format!("{}_{}", params.signature, params.index),
                                params.signature.clone(),
                                params.slot,
                                params.block_time,
                                params.program_received_time_ms,
                                params.index.clone(),
                                config.event_type.clone(),
                                config.protocol_type.clone(),
                            );

                            if let Ok(event) =
                                (config.inner_instruction_parser)(&data, metadata.clone())
                            {
                                events.push(event);
                            }
                        }
                    }
                }
            }
            SolanaTransactionInput::Instruction(params) => {
                // Check each discriminator
                for (discriminator, configs) in &self.instruction_configs {
                    if has_discriminator(&params.instruction_data, discriminator) {
                        for config in configs {
                            let metadata = create_metadata(
                                format!("{}_{}", params.signature, params.index),
                                params.signature.clone(),
                                params.slot,
                                params.block_time,
                                params.program_received_time_ms,
                                params.index.clone(),
                                config.event_type.clone(),
                                config.protocol_type.clone(),
                            );

                            if let Ok(event) = (config.instruction_parser)(
                                &params.instruction_data,
                                &params.accounts,
                                metadata.clone(),
                            ) {
                                events.push(event);
                            }
                        }
                    }
                }
            }
        }

        Ok(events)
    }
}

// Keep legacy trait implementation for backward compatibility during transition
#[async_trait::async_trait]
impl ProtocolParser for Parser {
    fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner_instruction_configs.clone()
    }

    fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.instruction_configs.clone()
    }

    fn parse_events_from_inner_instruction(
        &self,
        params: &InnerInstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        let mut events = Vec::new();

        // For inner instructions, we'll use the data to identify the instruction type
        if let Ok(data) = bs58::decode(&params.inner_instruction.data).into_vec() {
            for configs in self.inner_instruction_configs.values() {
                for config in configs {
                    let metadata = create_metadata(
                        format!("{}_{}", params.signature, params.index),
                        params.signature.to_string(),
                        params.slot,
                        params.block_time,
                        params.program_received_time_ms,
                        params.index.clone(),
                        config.event_type.clone(),
                        config.protocol_type.clone(),
                    );

                    if let Ok(event) = (config.inner_instruction_parser)(&data, metadata.clone()) {
                        events.push(event);
                    }
                }
            }
        }

        events
    }

    fn parse_events_from_instruction(
        &self,
        params: &InstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        let mut events = Vec::new();

        // Check each discriminator
        for (discriminator, configs) in &self.instruction_configs {
            if has_discriminator(&params.instruction.data, discriminator) {
                for config in configs {
                    let metadata = create_metadata(
                        format!("{}_{}", params.signature, params.index),
                        params.signature.to_string(),
                        params.slot,
                        params.block_time,
                        params.program_received_time_ms,
                        params.index.clone(),
                        config.event_type.clone(),
                        config.protocol_type.clone(),
                    );

                    if let Ok(event) = (config.instruction_parser)(
                        &params.instruction.data,
                        params.accounts,
                        metadata.clone(),
                    ) {
                        events.push(event);
                    }
                }
            }
        }

        events
    }

    fn should_handle(&self, program_id: &Pubkey) -> bool {
        self.program_ids.contains(program_id)
    }

    fn supported_program_ids(&self) -> Vec<Pubkey> {
        self.program_ids.clone()
    }
}

impl Default for Parser {
    fn default() -> Self {
        let program_ids = vec![whirlpool_program_id()];

        let configs = vec![
            GenericEventParseConfig {
                program_id: whirlpool_program_id(),
                protocol_type: ProtocolType::OrcaWhirlpool,
                inner_instruction_discriminator: "swap",
                instruction_discriminator: &SWAP_DISCRIMINATOR,
                event_type: EventType::Swap,
                inner_instruction_parser: parse_swap_inner_instruction,
                instruction_parser: parse_swap_instruction,
            },
            GenericEventParseConfig {
                program_id: whirlpool_program_id(),
                protocol_type: ProtocolType::OrcaWhirlpool,
                inner_instruction_discriminator: "openPosition",
                instruction_discriminator: &OPEN_POSITION_DISCRIMINATOR,
                event_type: EventType::CreatePool,
                inner_instruction_parser: parse_open_position_inner_instruction,
                instruction_parser: parse_open_position_instruction,
            },
            GenericEventParseConfig {
                program_id: whirlpool_program_id(),
                protocol_type: ProtocolType::OrcaWhirlpool,
                inner_instruction_discriminator: "closePosition",
                instruction_discriminator: &CLOSE_POSITION_DISCRIMINATOR,
                event_type: EventType::Unknown,
                inner_instruction_parser: parse_close_position_inner_instruction,
                instruction_parser: parse_close_position_instruction,
            },
            GenericEventParseConfig {
                program_id: whirlpool_program_id(),
                protocol_type: ProtocolType::OrcaWhirlpool,
                inner_instruction_discriminator: "increaseLiquidity",
                instruction_discriminator: &INCREASE_LIQUIDITY_DISCRIMINATOR,
                event_type: EventType::AddLiquidity,
                inner_instruction_parser: parse_increase_liquidity_inner_instruction,
                instruction_parser: parse_increase_liquidity_instruction,
            },
            GenericEventParseConfig {
                program_id: whirlpool_program_id(),
                protocol_type: ProtocolType::OrcaWhirlpool,
                inner_instruction_discriminator: "decreaseLiquidity",
                instruction_discriminator: &DECREASE_LIQUIDITY_DISCRIMINATOR,
                event_type: EventType::RemoveLiquidity,
                inner_instruction_parser: parse_decrease_liquidity_inner_instruction,
                instruction_parser: parse_decrease_liquidity_instruction,
            },
        ];

        let mut inner_instruction_configs = HashMap::new();
        let mut instruction_configs = HashMap::new();

        for config in configs {
            inner_instruction_configs
                .entry(config.inner_instruction_discriminator)
                .or_insert_with(Vec::new)
                .push(config.clone());
            instruction_configs
                .entry(config.instruction_discriminator.to_vec())
                .or_insert_with(Vec::new)
                .push(config);
        }

        let info = ParserInfo::new("orca_parser".to_owned(), "1.0.0".to_owned())
            .with_kind(riglr_events_core::EventKind::Swap)
            .with_kind(riglr_events_core::EventKind::Custom("position".to_owned()))
            .with_kind(riglr_events_core::EventKind::Custom("liquidity".to_owned()))
            .with_format("solana_instruction".to_owned());
        Self {
            info,
            inner_instruction_configs,
            instruction_configs,
            program_ids,
        }
    }
}

// Parser functions for different Orca instruction types

#[expect(clippy::needless_pass_by_value)]
fn parse_swap_inner_instruction(
    data: &[u8],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let swap_data = parse_swap_data(data).ok_or_else(|| {
        ParseError::InvalidDataFormat("Failed to parse Orca swap data".to_string())
    })?;

    let params = EventParameters {
        id: metadata.id().to_string(),
        signature: get_signature(&metadata.core).unwrap_or("").to_string(),
        slot: get_slot(&metadata.core).unwrap_or(0),
        block_time: get_block_time(&metadata.core).unwrap_or(0),
        block_time_ms: get_block_time(&metadata.core).map_or(0, |t| t.saturating_mul(1000)),
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };

    Ok(Box::new(SwapEvent::new(params, swap_data)) as Box<dyn Event>)
}

#[expect(clippy::needless_pass_by_value)]
fn parse_swap_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let swap_data = parse_swap_data_from_instruction(data, accounts).ok_or_else(|| {
        ParseError::InvalidDataFormat("Failed to parse Orca swap data".to_string())
    })?;

    let params = EventParameters {
        id: metadata.id().to_string(),
        signature: get_signature(&metadata.core).unwrap_or("").to_string(),
        slot: get_slot(&metadata.core).unwrap_or(0),
        block_time: get_block_time(&metadata.core).unwrap_or(0),
        block_time_ms: get_block_time(&metadata.core).map_or(0, |t| t.saturating_mul(1000)),
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };

    Ok(Box::new(SwapEvent::new(params, swap_data)) as Box<dyn Event>)
}

#[expect(clippy::needless_pass_by_value)]
fn parse_open_position_inner_instruction(
    data: &[u8],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let position_data = parse_position_data(data).ok_or_else(|| {
        ParseError::InvalidDataFormat("Failed to parse Orca position data".to_string())
    })?;

    let params = EventParameters {
        id: metadata.id().to_string(),
        signature: get_signature(&metadata.core).unwrap_or("").to_string(),
        slot: get_slot(&metadata.core).unwrap_or(0),
        block_time: get_block_time(&metadata.core).unwrap_or(0),
        block_time_ms: get_block_time(&metadata.core).map_or(0, |t| t.saturating_mul(1000)),
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };

    Ok(Box::new(PositionEvent::new(params, position_data, true)) as Box<dyn Event>)
}

#[expect(clippy::needless_pass_by_value)]
fn parse_open_position_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let position_data = parse_position_data_from_instruction(data, accounts).ok_or_else(|| {
        ParseError::InvalidDataFormat("Failed to parse Orca position data".to_string())
    })?;

    let params = EventParameters {
        id: metadata.id().to_string(),
        signature: get_signature(&metadata.core).unwrap_or("").to_string(),
        slot: get_slot(&metadata.core).unwrap_or(0),
        block_time: get_block_time(&metadata.core).unwrap_or(0),
        block_time_ms: get_block_time(&metadata.core).map_or(0, |t| t.saturating_mul(1000)),
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };

    Ok(Box::new(PositionEvent::new(params, position_data, true)) as Box<dyn Event>)
}

#[expect(clippy::needless_pass_by_value)]
fn parse_close_position_inner_instruction(
    data: &[u8],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let position_data = parse_position_data(data).ok_or_else(|| {
        ParseError::InvalidDataFormat("Failed to parse Orca position data".to_string())
    })?;

    let params = EventParameters {
        id: metadata.id().to_string(),
        signature: get_signature(&metadata.core).unwrap_or("").to_string(),
        slot: get_slot(&metadata.core).unwrap_or(0),
        block_time: get_block_time(&metadata.core).unwrap_or(0),
        block_time_ms: get_block_time(&metadata.core).map_or(0, |t| t.saturating_mul(1000)),
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };

    Ok(Box::new(PositionEvent::new(params, position_data, false)) as Box<dyn Event>)
}

#[expect(clippy::needless_pass_by_value)]
fn parse_close_position_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let position_data = parse_position_data_from_instruction(data, accounts).ok_or_else(|| {
        ParseError::InvalidDataFormat("Failed to parse Orca position data".to_string())
    })?;

    let params = EventParameters {
        id: metadata.id().to_string(),
        signature: get_signature(&metadata.core).unwrap_or("").to_string(),
        slot: get_slot(&metadata.core).unwrap_or(0),
        block_time: get_block_time(&metadata.core).unwrap_or(0),
        block_time_ms: get_block_time(&metadata.core).map_or(0, |t| t.saturating_mul(1000)),
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };

    Ok(Box::new(PositionEvent::new(params, position_data, false)) as Box<dyn Event>)
}

#[expect(clippy::needless_pass_by_value)]
fn parse_increase_liquidity_inner_instruction(
    data: &[u8],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let liquidity_data = parse_liquidity_data(data, true).ok_or_else(|| {
        ParseError::InvalidDataFormat("Failed to parse Orca liquidity data".to_string())
    })?;

    let params = EventParameters {
        id: metadata.id().to_string(),
        signature: get_signature(&metadata.core).unwrap_or("").to_string(),
        slot: get_slot(&metadata.core).unwrap_or(0),
        block_time: get_block_time(&metadata.core).unwrap_or(0),
        block_time_ms: get_block_time(&metadata.core).map_or(0, |t| t.saturating_mul(1000)),
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };
    Ok(Box::new(LiquidityEvent::new(params, liquidity_data)) as Box<dyn Event>)
}

#[expect(clippy::needless_pass_by_value)]
fn parse_increase_liquidity_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let liquidity_data =
        parse_liquidity_data_from_instruction(data, accounts, true).ok_or_else(|| {
            ParseError::InvalidDataFormat("Failed to parse Orca liquidity data".to_string())
        })?;

    let params = EventParameters {
        id: metadata.id().to_string(),
        signature: get_signature(&metadata.core).unwrap_or("").to_string(),
        slot: get_slot(&metadata.core).unwrap_or(0),
        block_time: get_block_time(&metadata.core).unwrap_or(0),
        block_time_ms: get_block_time(&metadata.core).map_or(0, |t| t.saturating_mul(1000)),
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };
    Ok(Box::new(LiquidityEvent::new(params, liquidity_data)) as Box<dyn Event>)
}

#[expect(clippy::needless_pass_by_value)]
fn parse_decrease_liquidity_inner_instruction(
    data: &[u8],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let liquidity_data = parse_liquidity_data(data, false).ok_or_else(|| {
        ParseError::InvalidDataFormat("Failed to parse Orca liquidity data".to_string())
    })?;

    let params = EventParameters {
        id: metadata.id().to_string(),
        signature: get_signature(&metadata.core).unwrap_or("").to_string(),
        slot: get_slot(&metadata.core).unwrap_or(0),
        block_time: get_block_time(&metadata.core).unwrap_or(0),
        block_time_ms: get_block_time(&metadata.core).map_or(0, |t| t.saturating_mul(1000)),
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };
    Ok(Box::new(LiquidityEvent::new(params, liquidity_data)) as Box<dyn Event>)
}

#[expect(clippy::needless_pass_by_value)]
fn parse_decrease_liquidity_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let liquidity_data =
        parse_liquidity_data_from_instruction(data, accounts, false).ok_or_else(|| {
            ParseError::InvalidDataFormat("Failed to parse Orca liquidity data".to_string())
        })?;

    let params = EventParameters {
        id: metadata.id().to_string(),
        signature: get_signature(&metadata.core).unwrap_or("").to_string(),
        slot: get_slot(&metadata.core).unwrap_or(0),
        block_time: get_block_time(&metadata.core).unwrap_or(0),
        block_time_ms: get_block_time(&metadata.core).map_or(0, |t| t.saturating_mul(1000)),
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };
    Ok(Box::new(LiquidityEvent::new(params, liquidity_data)) as Box<dyn Event>)
}

// Data parsing helpers

fn parse_swap_data(data: &[u8]) -> Option<SwapData> {
    validate_data_length(data, 64, "Orca swap data").ok()?;

    let mut offset: usize = 8; // Skip discriminator

    let amount = parse_u64_le(data.get(offset..offset.saturating_add(8))?).ok()?;
    offset = offset.saturating_add(8);

    let other_amount_threshold = parse_u64_le(data.get(offset..offset.saturating_add(8))?).ok()?;
    offset = offset.saturating_add(8);

    let sqrt_price_limit = parse_u128_le(data.get(offset..offset.saturating_add(16))?).ok()?;
    offset = offset.saturating_add(16);

    let amount_specified_is_input = data.get(offset)? != &0;
    offset = offset.saturating_add(1);

    let a_to_b = data.get(offset)? != &0;

    Some(SwapData {
        whirlpool: Pubkey::default(),     // Would need to extract from accounts
        user: Pubkey::default(),          // Would need to extract from accounts
        token_mint_a: Pubkey::default(),  // Would need to extract from accounts
        token_mint_b: Pubkey::default(),  // Would need to extract from accounts
        token_vault_a: Pubkey::default(), // Would need to extract from accounts
        token_vault_b: Pubkey::default(), // Would need to extract from accounts
        amount,
        amount_specified_is_input,
        a_to_b,
        sqrt_price_limit,
        amount_in: if amount_specified_is_input {
            amount
        } else {
            other_amount_threshold
        },
        amount_out: if amount_specified_is_input {
            other_amount_threshold
        } else {
            amount
        },
        fee_amount: 0,         // Would need to calculate from pool state
        tick_current_index: 0, // Would need to extract from pool state
        sqrt_price: 0,         // Would need to extract from pool state
        liquidity: 0,          // Would need to extract from pool state
    })
}
fn parse_swap_data_from_instruction(data: &[u8], accounts: &[Pubkey]) -> Option<SwapData> {
    let mut swap_data = parse_swap_data(data)?;

    // Extract accounts (typical Orca swap instruction layout)
    if validate_account_count(accounts, 11, "Orca swap instruction").is_ok() {
        swap_data.whirlpool = safe_get_account(accounts, 1).unwrap_or_default();
        swap_data.user = safe_get_account(accounts, 0).unwrap_or_default();
        swap_data.token_vault_a = safe_get_account(accounts, 3).unwrap_or_default();
        swap_data.token_vault_b = safe_get_account(accounts, 4).unwrap_or_default();
        // Note: Would need more sophisticated parsing for all fields
    }

    Some(swap_data)
}

fn parse_position_data(data: &[u8]) -> Option<PositionData> {
    validate_data_length(data, 32, "Orca position data").ok()?;

    let mut offset: usize = 8; // Skip discriminator

    let tick_lower_index = i32::from_ne_bytes(
        parse_u32_le(data.get(offset..offset.saturating_add(4))?)
            .ok()?
            .to_ne_bytes(),
    );
    offset = offset.saturating_add(4);

    let tick_upper_index = i32::from_ne_bytes(
        parse_u32_le(data.get(offset..offset.saturating_add(4))?)
            .ok()?
            .to_ne_bytes(),
    );

    Some(PositionData {
        whirlpool: Pubkey::default(),
        position_mint: Pubkey::default(),
        position: Pubkey::default(),
        position_token_account: Pubkey::default(),
        position_authority: Pubkey::default(),
        tick_lower_index,
        tick_upper_index,
        liquidity: 0,
        fee_growth_checkpoint_a: 0,
        fee_growth_checkpoint_b: 0,
        fee_owed_a: 0,
        fee_owed_b: 0,
        reward_infos: [PositionRewardInfo::default(); 3],
    })
}
fn parse_position_data_from_instruction(data: &[u8], accounts: &[Pubkey]) -> Option<PositionData> {
    let mut position_data = parse_position_data(data)?;

    // Extract accounts from instruction
    if validate_account_count(accounts, 7, "Orca position instruction").is_ok() {
        position_data.whirlpool = safe_get_account(accounts, 1).unwrap_or_default();
        position_data.position_authority = safe_get_account(accounts, 0).unwrap_or_default();
        position_data.position = safe_get_account(accounts, 2).unwrap_or_default();
        position_data.position_mint = safe_get_account(accounts, 3).unwrap_or_default();
        position_data.position_token_account = safe_get_account(accounts, 4).unwrap_or_default();
    }

    Some(position_data)
}

// Module structure for backward compatibility with the previous directory structure
/// Orca protocol event definitions and constants.
pub mod events {
    pub use super::{LiquidityEvent, PositionEvent, SwapEvent};
}

/// Orca protocol transaction and log parsing functionality.
pub mod parser {
    pub use super::Parser;
}

/// Orca protocol data types and structures.
pub mod types {
    pub use super::{
        is_orca_whirlpool_program, sqrt_price_to_price, tick_index_to_price, whirlpool_program_id,
        LiquidityData, PositionData, PositionRewardInfo, SwapData, SwapDirection, WhirlpoolAccount,
        WhirlpoolRewardInfo, CLOSE_POSITION_DISCRIMINATOR, DECREASE_LIQUIDITY_DISCRIMINATOR,
        INCREASE_LIQUIDITY_DISCRIMINATOR, OPEN_POSITION_DISCRIMINATOR, ORCA_WHIRLPOOL_PROGRAM_ID,
        SWAP_DISCRIMINATOR,
    };
}

fn parse_liquidity_data(data: &[u8], is_increase: bool) -> Option<LiquidityData> {
    validate_data_length(data, 40, "Orca liquidity data").ok()?;

    let mut offset: usize = 8; // Skip discriminator

    let liquidity_amount = parse_u128_le(data.get(offset..offset.saturating_add(16))?).ok()?;
    offset = offset.saturating_add(16);

    let token_max_a = parse_u64_le(data.get(offset..offset.saturating_add(8))?).ok()?;
    offset = offset.saturating_add(8);

    let token_max_b = parse_u64_le(data.get(offset..offset.saturating_add(8))?).ok()?;

    Some(LiquidityData {
        whirlpool: Pubkey::default(),
        position: Pubkey::default(),
        position_authority: Pubkey::default(),
        token_mint_a: Pubkey::default(),
        token_mint_b: Pubkey::default(),
        token_vault_a: Pubkey::default(),
        token_vault_b: Pubkey::default(),
        tick_lower_index: 0,
        tick_upper_index: 0,
        liquidity_amount,
        token_max_a,
        token_max_b,
        token_actual_a: token_max_a, // Simplified
        token_actual_b: token_max_b, // Simplified
        is_increase,
    })
}
fn parse_liquidity_data_from_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    is_increase: bool,
) -> Option<LiquidityData> {
    let mut liquidity_data = parse_liquidity_data(data, is_increase)?;

    // Extract accounts from instruction
    if validate_account_count(accounts, 12, "Orca liquidity instruction").is_ok() {
        liquidity_data.whirlpool = safe_get_account(accounts, 1).unwrap_or_default();
        liquidity_data.position_authority = safe_get_account(accounts, 0).unwrap_or_default();
        liquidity_data.position = safe_get_account(accounts, 2).unwrap_or_default();
        liquidity_data.token_vault_a = safe_get_account(accounts, 5).unwrap_or_default();
        liquidity_data.token_vault_b = safe_get_account(accounts, 6).unwrap_or_default();
    }

    Some(liquidity_data)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod parser_tests {
    use super::*;
    use crate::solana_metadata::SolanaEventMetadata;
    use crate::types::{EventType, ProtocolType};
    use core::str::FromStr;
    use riglr_events_core::{EventKind, EventMetadata as CoreMetadata};
    use solana_message::compiled_instruction::CompiledInstruction;
    use solana_transaction_status::UiCompiledInstruction;

    fn create_test_metadata() -> SolanaEventMetadata {
        let core = CoreMetadata::new(
            "test_id".to_string(),
            EventKind::Swap,
            "orca_whirlpool".to_string(),
        );

        SolanaEventMetadata::new(
            "test_sig".to_string(),
            12345,
            EventType::Swap,
            ProtocolType::OrcaWhirlpool,
            "0".to_owned(),
            1000,
            core,
        )
    }

    fn create_test_accounts() -> Vec<Pubkey> {
        (0..15)
            .map(|i| {
                // Create valid test pubkeys by using Pubkey::new_unique() for reproducible tests
                use core::hash::{Hash, Hasher};
                use std::collections::hash_map::DefaultHasher;
                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                let hash = hasher.finish();
                let bytes = hash.to_le_bytes();
                let mut full_bytes = [0u8; 32];
                full_bytes[..8].copy_from_slice(&bytes);
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                {
                    full_bytes[8] = i as u8; // Make each one unique
                }
                Pubkey::new_from_array(full_bytes)
            })
            .collect()
    }

    #[test]
    fn test_orca_event_parser_default() {
        let parser = Parser::default();

        assert_eq!(parser.program_ids.len(), 1);
        assert_eq!(
            parser.program_ids.first().copied(),
            Some(whirlpool_program_id())
        );
        assert_eq!(parser.inner_instruction_configs.len(), 5);
        assert_eq!(parser.instruction_configs.len(), 5);

        // Verify all expected discriminators are present
        assert!(parser.inner_instruction_configs.contains_key("swap"));
        assert!(parser
            .inner_instruction_configs
            .contains_key("openPosition"));
        assert!(parser
            .inner_instruction_configs
            .contains_key("closePosition"));
        assert!(parser
            .inner_instruction_configs
            .contains_key("increaseLiquidity"));
        assert!(parser
            .inner_instruction_configs
            .contains_key("decreaseLiquidity"));
    }

    #[test]
    fn test_should_handle_when_program_id_matches_should_return_true() {
        let parser = Parser::default();
        let orca_program_id = whirlpool_program_id();

        assert!(parser.should_handle(&orca_program_id));
    }

    #[test]
    fn test_should_handle_when_program_id_not_matches_should_return_false() {
        let parser = Parser::default();
        let other_program_id = Pubkey::from_str("11111111111111111111111111111111")
            .expect("Valid pubkey string should parse successfully");

        assert!(!parser.should_handle(&other_program_id));
    }

    #[test]
    fn test_supported_program_ids() {
        let parser = Parser::default();
        let supported_ids = parser.supported_program_ids();

        assert_eq!(supported_ids.len(), 1);
        assert_eq!(supported_ids.first().copied(), Some(whirlpool_program_id()));
    }

    #[test]
    fn test_inner_instruction_configs() {
        let parser = Parser::default();
        let configs = parser.inner_instruction_configs();

        assert_eq!(configs.len(), 5);
        assert!(configs.contains_key("swap"));
    }

    #[test]
    fn test_instruction_configs() {
        let parser = Parser::default();
        let configs = parser.instruction_configs();

        assert_eq!(configs.len(), 5);
        assert!(configs.contains_key(SWAP_DISCRIMINATOR.as_slice()));
    }

    #[test]
    fn test_parse_events_from_inner_instruction_when_valid_data_should_parse() {
        let parser = Parser::default();
        let mut data = vec![0u8; 64];

        // Create valid swap data
        data.get_mut(0..8)
            .unwrap()
            .copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]); // discriminator
        data.get_mut(8..16)
            .unwrap()
            .copy_from_slice(&1000u64.to_le_bytes()); // amount
        data.get_mut(16..24)
            .unwrap()
            .copy_from_slice(&2000u64.to_le_bytes()); // other_amount_threshold
        data.get_mut(24..40)
            .unwrap()
            .copy_from_slice(&123_456_789_u128.to_le_bytes()); // sqrt_price_limit
        *data.get_mut(40).unwrap() = 1; // amount_specified_is_input
        *data.get_mut(41).unwrap() = 1; // a_to_b

        let encoded_data = bs58::encode(&data).into_string();

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2],
            data: encoded_data,
            stack_height: Some(1),
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_sig",
            slot: 12345,
            block_time: Some(1000),
            program_received_time_ms: 2000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        // Should have events for all 5 inner instruction configs
        assert_eq!(events.len(), 5);
    }

    #[test]
    fn test_parse_events_from_inner_instruction_when_invalid_data_should_return_empty() {
        let parser = Parser::default();

        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2],
            data: "invalid_base58".to_string(),
            stack_height: Some(1),
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_sig",
            slot: 12345,
            block_time: Some(1000),
            program_received_time_ms: 2000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_parse_events_from_instruction_when_discriminator_matches_should_parse() {
        let parser = Parser::default();
        let accounts = create_test_accounts();

        let mut data = vec![0u8; 64];
        data.get_mut(0..8)
            .unwrap()
            .copy_from_slice(&SWAP_DISCRIMINATOR);
        data.get_mut(8..16)
            .unwrap()
            .copy_from_slice(&1000u64.to_le_bytes());
        data.get_mut(16..24)
            .unwrap()
            .copy_from_slice(&2000u64.to_le_bytes());
        data.get_mut(24..40)
            .unwrap()
            .copy_from_slice(&123_456_789_u128.to_le_bytes());
        *data.get_mut(40).unwrap() = 1;
        *data.get_mut(41).unwrap() = 1;

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            data,
        };

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_sig",
            slot: 12345,
            block_time: Some(1000),
            program_received_time_ms: 2000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_parse_events_from_instruction_when_no_discriminator_match_should_return_empty() {
        let parser = Parser::default();
        let accounts = create_test_accounts();

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2],
            data: vec![99, 99, 99, 99, 99, 99, 99, 99], // Non-matching discriminator
        };

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_sig",
            slot: 12345,
            block_time: Some(1000),
            program_received_time_ms: 2000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_parse_orca_swap_data_when_valid_data_should_return_some() {
        let mut data = vec![0u8; 64];
        data.get_mut(8..16)
            .unwrap()
            .copy_from_slice(&1000u64.to_le_bytes()); // amount
        data.get_mut(16..24)
            .unwrap()
            .copy_from_slice(&2000u64.to_le_bytes()); // other_amount_threshold
        data.get_mut(24..40)
            .unwrap()
            .copy_from_slice(&123_456_789_u128.to_le_bytes()); // sqrt_price_limit
        *data.get_mut(40).unwrap() = 1; // amount_specified_is_input
        *data.get_mut(41).unwrap() = 1; // a_to_b

        let swap_data = parse_swap_data(&data);

        assert!(swap_data.is_some());
        let swap_data = swap_data.expect("Valid swap data should parse successfully");
        assert_eq!(swap_data.amount, 1000);
        assert_eq!(swap_data.sqrt_price_limit, 123_456_789);
        assert!(swap_data.amount_specified_is_input);
        assert!(swap_data.a_to_b);
        assert_eq!(swap_data.amount_in, 1000);
        assert_eq!(swap_data.amount_out, 2000);
    }

    #[test]
    fn test_parse_orca_swap_data_when_amount_not_input_should_swap_amounts() {
        let mut data = vec![0u8; 64];
        data.get_mut(8..16)
            .unwrap()
            .copy_from_slice(&1000u64.to_le_bytes()); // amount
        data.get_mut(16..24)
            .unwrap()
            .copy_from_slice(&2000u64.to_le_bytes()); // other_amount_threshold
        data.get_mut(24..40)
            .unwrap()
            .copy_from_slice(&123_456_789_u128.to_le_bytes()); // sqrt_price_limit
        *data.get_mut(40).unwrap() = 0; // amount_specified_is_input = false
        *data.get_mut(41).unwrap() = 0; // a_to_b = false

        let swap_data = parse_swap_data(&data);

        assert!(swap_data.is_some());
        let swap_data = swap_data.expect("Valid swap data should parse successfully");
        assert!(!swap_data.amount_specified_is_input);
        assert!(!swap_data.a_to_b);
        assert_eq!(swap_data.amount_in, 2000); // swapped
        assert_eq!(swap_data.amount_out, 1000); // swapped
    }

    #[test]
    fn test_parse_orca_swap_data_when_insufficient_data_should_return_none() {
        let data = vec![0u8; 32]; // Too short

        let result = parse_swap_data(&data);

        assert!(result.is_none());
    }

    #[test]
    fn test_parse_orca_swap_data_from_instruction_when_sufficient_accounts_should_extract() {
        let mut data = vec![0u8; 64];
        data.get_mut(8..16)
            .unwrap()
            .copy_from_slice(&1000u64.to_le_bytes());
        data.get_mut(16..24)
            .unwrap()
            .copy_from_slice(&2000u64.to_le_bytes());
        data.get_mut(24..40)
            .unwrap()
            .copy_from_slice(&123_456_789_u128.to_le_bytes());
        *data.get_mut(40).unwrap() = 1;
        *data.get_mut(41).unwrap() = 1;

        let accounts = create_test_accounts();

        let swap_data = parse_swap_data_from_instruction(&data, &accounts);

        assert!(swap_data.is_some());
        let swap_data = swap_data.expect("Valid swap data should parse successfully");
        assert_eq!(swap_data.user, *accounts.first().unwrap());
        assert_eq!(swap_data.whirlpool, *accounts.get(1).unwrap());
        assert_eq!(swap_data.token_vault_a, *accounts.get(3).unwrap());
        assert_eq!(swap_data.token_vault_b, *accounts.get(4).unwrap());
    }

    #[test]
    fn test_parse_orca_swap_data_from_instruction_when_insufficient_accounts_should_use_defaults() {
        let mut data = vec![0u8; 64];
        data.get_mut(8..16)
            .unwrap()
            .copy_from_slice(&1000u64.to_le_bytes());
        data.get_mut(16..24)
            .unwrap()
            .copy_from_slice(&2000u64.to_le_bytes());
        data.get_mut(24..40)
            .unwrap()
            .copy_from_slice(&123_456_789_u128.to_le_bytes());
        *data.get_mut(40).unwrap() = 1;
        *data.get_mut(41).unwrap() = 1;

        let accounts = vec![*create_test_accounts().first().unwrap()]; // Only one account

        let swap_data = parse_swap_data_from_instruction(&data, &accounts);

        assert!(swap_data.is_some());
        let swap_data = swap_data.expect("Valid swap data should parse successfully");
        assert_eq!(swap_data.whirlpool, Pubkey::default());
        assert_eq!(swap_data.user, Pubkey::default());
    }

    #[test]
    fn test_parse_orca_position_data_when_valid_data_should_return_some() {
        let mut data = vec![0u8; 32];
        data.get_mut(8..12).unwrap().copy_from_slice(&{
            // Safe cast: negative tick index to u32 bytes for serialization
            {
                #[allow(clippy::cast_sign_loss)]
                (-100i32 as u32).to_le_bytes()
            }
        }); // tick_lower_index
        data.get_mut(12..16).unwrap().copy_from_slice(&{
            #[allow(clippy::cast_sign_loss)]
            (100i32 as u32).to_le_bytes()
        }); // tick_upper_index

        let position_data = parse_position_data(&data);

        assert!(position_data.is_some());
        let position_data = position_data.expect("Valid position data should parse successfully");
        assert_eq!(position_data.tick_lower_index, -100);
        assert_eq!(position_data.tick_upper_index, 100);
    }

    #[test]
    fn test_parse_orca_position_data_when_insufficient_data_should_return_none() {
        let data = vec![0u8; 16]; // Too short

        let result = parse_position_data(&data);

        assert!(result.is_none());
    }

    #[test]
    fn test_parse_orca_position_data_from_instruction_when_sufficient_accounts_should_extract() {
        let mut data = vec![0u8; 32];
        data.get_mut(8..12).unwrap().copy_from_slice(&{
            // Safe cast: negative tick index to u32 bytes for serialization
            {
                #[allow(clippy::cast_sign_loss)]
                (-100i32 as u32).to_le_bytes()
            }
        });
        data.get_mut(12..16).unwrap().copy_from_slice(&{
            #[allow(clippy::cast_sign_loss)]
            (100i32 as u32).to_le_bytes()
        });

        let accounts = create_test_accounts();

        let position_data = parse_position_data_from_instruction(&data, &accounts);

        assert!(position_data.is_some());
        let position_data = position_data.expect("Valid position data should parse successfully");
        assert_eq!(position_data.position_authority, *accounts.first().unwrap());
        assert_eq!(position_data.whirlpool, *accounts.get(1).unwrap());
        assert_eq!(position_data.position, *accounts.get(2).unwrap());
        assert_eq!(position_data.position_mint, *accounts.get(3).unwrap());
        assert_eq!(
            position_data.position_token_account,
            *accounts.get(4).unwrap()
        );
    }

    #[test]
    fn test_parse_orca_position_data_from_instruction_when_insufficient_accounts_should_use_defaults(
    ) {
        let mut data = vec![0u8; 32];
        data.get_mut(8..12).unwrap().copy_from_slice(&{
            // Safe cast: negative tick index to u32 bytes for serialization
            {
                #[allow(clippy::cast_sign_loss)]
                (-100i32 as u32).to_le_bytes()
            }
        });
        data.get_mut(12..16).unwrap().copy_from_slice(&{
            #[allow(clippy::cast_sign_loss)]
            (100i32 as u32).to_le_bytes()
        });

        let accounts = vec![*create_test_accounts().first().unwrap()]; // Only one account

        let position_data = parse_position_data_from_instruction(&data, &accounts);

        assert!(position_data.is_some());
        let position_data = position_data.expect("Valid position data should parse successfully");
        assert_eq!(position_data.whirlpool, Pubkey::default());
        assert_eq!(position_data.position_authority, Pubkey::default());
    }

    #[test]
    fn test_parse_orca_liquidity_data_when_valid_data_should_return_some() {
        let mut data = vec![0u8; 40];
        data.get_mut(8..24)
            .unwrap()
            .copy_from_slice(&123_456_789_u128.to_le_bytes()); // liquidity_amount
        data.get_mut(24..32)
            .unwrap()
            .copy_from_slice(&1000u64.to_le_bytes()); // token_max_a
        data.get_mut(32..40)
            .unwrap()
            .copy_from_slice(&2000u64.to_le_bytes()); // token_max_b

        let liquidity_data = parse_liquidity_data(&data, true);

        assert!(liquidity_data.is_some());
        let liquidity_data =
            liquidity_data.expect("Valid liquidity data should parse successfully");
        assert_eq!(liquidity_data.liquidity_amount, 123_456_789);
        assert_eq!(liquidity_data.token_max_a, 1000);
        assert_eq!(liquidity_data.token_max_b, 2000);
        assert!(liquidity_data.is_increase);
        assert_eq!(liquidity_data.token_actual_a, 1000);
        assert_eq!(liquidity_data.token_actual_b, 2000);
    }

    #[test]
    fn test_parse_orca_liquidity_data_when_decrease_should_set_flag() {
        let mut data = vec![0u8; 40];
        data.get_mut(8..24)
            .unwrap()
            .copy_from_slice(&123_456_789_u128.to_le_bytes());
        data.get_mut(24..32)
            .unwrap()
            .copy_from_slice(&1000u64.to_le_bytes());
        data.get_mut(32..40)
            .unwrap()
            .copy_from_slice(&2000u64.to_le_bytes());

        let liquidity_data = parse_liquidity_data(&data, false);

        assert!(liquidity_data.is_some());
        let liquidity_data =
            liquidity_data.expect("Valid liquidity data should parse successfully");
        assert!(!liquidity_data.is_increase);
    }

    #[test]
    fn test_parse_orca_liquidity_data_when_insufficient_data_should_return_none() {
        let data = vec![0u8; 20]; // Too short

        let result = parse_liquidity_data(&data, true);

        assert!(result.is_none());
    }

    #[test]
    fn test_parse_orca_liquidity_data_from_instruction_when_sufficient_accounts_should_extract() {
        let mut data = vec![0u8; 40];
        data.get_mut(8..24)
            .unwrap()
            .copy_from_slice(&123_456_789_u128.to_le_bytes());
        data.get_mut(24..32)
            .unwrap()
            .copy_from_slice(&1000u64.to_le_bytes());
        data.get_mut(32..40)
            .unwrap()
            .copy_from_slice(&2000u64.to_le_bytes());

        let accounts = create_test_accounts();

        let liquidity_data = parse_liquidity_data_from_instruction(&data, &accounts, true);

        assert!(liquidity_data.is_some());
        let liquidity_data =
            liquidity_data.expect("Valid liquidity data should parse successfully");
        assert_eq!(
            liquidity_data.position_authority,
            *accounts.first().unwrap()
        );
        assert_eq!(liquidity_data.whirlpool, *accounts.get(1).unwrap());
        assert_eq!(liquidity_data.position, *accounts.get(2).unwrap());
        assert_eq!(liquidity_data.token_vault_a, *accounts.get(5).unwrap());
        assert_eq!(liquidity_data.token_vault_b, *accounts.get(6).unwrap());
    }

    #[test]
    fn test_parse_orca_liquidity_data_from_instruction_when_insufficient_accounts_should_use_defaults(
    ) {
        let mut data = vec![0u8; 40];
        data.get_mut(8..24)
            .unwrap()
            .copy_from_slice(&123_456_789_u128.to_le_bytes());
        data.get_mut(24..32)
            .unwrap()
            .copy_from_slice(&1000u64.to_le_bytes());
        data.get_mut(32..40)
            .unwrap()
            .copy_from_slice(&2000u64.to_le_bytes());

        let accounts = vec![*create_test_accounts().first().unwrap()]; // Only one account

        let liquidity_data = parse_liquidity_data_from_instruction(&data, &accounts, true);

        assert!(liquidity_data.is_some());
        let liquidity_data =
            liquidity_data.expect("Valid liquidity data should parse successfully");
        assert_eq!(liquidity_data.whirlpool, Pubkey::default());
        assert_eq!(liquidity_data.position_authority, Pubkey::default());
    }

    #[test]
    fn test_parse_orca_swap_inner_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 64];
        data.get_mut(8..16)
            .unwrap()
            .copy_from_slice(&1000u64.to_le_bytes());
        data.get_mut(16..24)
            .unwrap()
            .copy_from_slice(&2000u64.to_le_bytes());
        data.get_mut(24..40)
            .unwrap()
            .copy_from_slice(&123_456_789_u128.to_le_bytes());
        *data.get_mut(40).unwrap() = 1;
        *data.get_mut(41).unwrap() = 1;

        let metadata = create_test_metadata();
        let event = parse_swap_inner_instruction(&data, metadata);

        assert!(event.is_ok());
    }

    #[test]
    fn test_parse_orca_swap_inner_instruction_when_invalid_data_should_return_none() {
        let data = vec![0u8; 10]; // Too short

        let metadata = create_test_metadata();
        let event = parse_swap_inner_instruction(&data, metadata);

        assert!(event.is_err());
    }

    #[test]
    fn test_parse_orca_swap_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 64];
        data.get_mut(8..16)
            .unwrap()
            .copy_from_slice(&1000u64.to_le_bytes());
        data.get_mut(16..24)
            .unwrap()
            .copy_from_slice(&2000u64.to_le_bytes());
        data.get_mut(24..40)
            .unwrap()
            .copy_from_slice(&123_456_789_u128.to_le_bytes());
        *data.get_mut(40).unwrap() = 1;
        *data.get_mut(41).unwrap() = 1;

        let accounts = create_test_accounts();
        let metadata = create_test_metadata();
        let event = parse_swap_instruction(&data, &accounts, metadata);

        assert!(event.is_ok());
    }

    #[test]
    fn test_parse_orca_open_position_inner_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 32];
        data.get_mut(8..12).unwrap().copy_from_slice(&{
            // Safe cast: negative tick index to u32 bytes for serialization
            {
                #[allow(clippy::cast_sign_loss)]
                (-100i32 as u32).to_le_bytes()
            }
        });
        data.get_mut(12..16).unwrap().copy_from_slice(&{
            #[allow(clippy::cast_sign_loss)]
            (100i32 as u32).to_le_bytes()
        });

        let metadata = create_test_metadata();
        let event = parse_open_position_inner_instruction(&data, metadata);

        assert!(event.is_ok());
    }

    #[test]
    fn test_parse_orca_open_position_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 32];
        data.get_mut(8..12).unwrap().copy_from_slice(&{
            // Safe cast: negative tick index to u32 bytes for serialization
            {
                #[allow(clippy::cast_sign_loss)]
                (-100i32 as u32).to_le_bytes()
            }
        });
        data.get_mut(12..16).unwrap().copy_from_slice(&{
            #[allow(clippy::cast_sign_loss)]
            (100i32 as u32).to_le_bytes()
        });

        let accounts = create_test_accounts();
        let metadata = create_test_metadata();
        let event = parse_open_position_instruction(&data, &accounts, metadata);

        assert!(event.is_ok());
    }

    #[test]
    fn test_parse_orca_close_position_inner_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 32];
        data.get_mut(8..12).unwrap().copy_from_slice(&{
            // Safe cast: negative tick index to u32 bytes for serialization
            {
                #[allow(clippy::cast_sign_loss)]
                (-100i32 as u32).to_le_bytes()
            }
        });
        data.get_mut(12..16).unwrap().copy_from_slice(&{
            #[allow(clippy::cast_sign_loss)]
            (100i32 as u32).to_le_bytes()
        });

        let metadata = create_test_metadata();
        let event = parse_close_position_inner_instruction(&data, metadata);

        assert!(event.is_ok());
    }

    #[test]
    fn test_parse_orca_close_position_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 32];
        data.get_mut(8..12).unwrap().copy_from_slice(&{
            // Safe cast: negative tick index to u32 bytes for serialization
            {
                #[allow(clippy::cast_sign_loss)]
                (-100i32 as u32).to_le_bytes()
            }
        });
        data.get_mut(12..16).unwrap().copy_from_slice(&{
            #[allow(clippy::cast_sign_loss)]
            (100i32 as u32).to_le_bytes()
        });

        let accounts = create_test_accounts();
        let metadata = create_test_metadata();
        let event = parse_close_position_instruction(&data, &accounts, metadata);

        assert!(event.is_ok());
    }

    #[test]
    fn test_parse_orca_increase_liquidity_inner_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 40];
        data.get_mut(8..24)
            .unwrap()
            .copy_from_slice(&123_456_789_u128.to_le_bytes());
        data.get_mut(24..32)
            .unwrap()
            .copy_from_slice(&1000u64.to_le_bytes());
        data.get_mut(32..40)
            .unwrap()
            .copy_from_slice(&2000u64.to_le_bytes());

        let metadata = create_test_metadata();
        let event = parse_increase_liquidity_inner_instruction(&data, metadata);

        assert!(event.is_ok());
    }

    #[test]
    fn test_parse_orca_increase_liquidity_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 40];
        data.get_mut(8..24)
            .unwrap()
            .copy_from_slice(&123_456_789_u128.to_le_bytes());
        data.get_mut(24..32)
            .unwrap()
            .copy_from_slice(&1000u64.to_le_bytes());
        data.get_mut(32..40)
            .unwrap()
            .copy_from_slice(&2000u64.to_le_bytes());

        let accounts = create_test_accounts();
        let metadata = create_test_metadata();
        let event = parse_increase_liquidity_instruction(&data, &accounts, metadata);

        assert!(event.is_ok());
    }

    #[test]
    fn test_parse_orca_decrease_liquidity_inner_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 40];
        data.get_mut(8..24)
            .unwrap()
            .copy_from_slice(&123_456_789_u128.to_le_bytes());
        data.get_mut(24..32)
            .unwrap()
            .copy_from_slice(&1000u64.to_le_bytes());
        data.get_mut(32..40)
            .unwrap()
            .copy_from_slice(&2000u64.to_le_bytes());

        let metadata = create_test_metadata();
        let event = parse_decrease_liquidity_inner_instruction(&data, metadata);

        assert!(event.is_ok());
    }

    #[test]
    fn test_parse_orca_decrease_liquidity_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 40];
        data.get_mut(8..24)
            .unwrap()
            .copy_from_slice(&123_456_789_u128.to_le_bytes());
        data.get_mut(24..32)
            .unwrap()
            .copy_from_slice(&1000u64.to_le_bytes());
        data.get_mut(32..40)
            .unwrap()
            .copy_from_slice(&2000u64.to_le_bytes());

        let accounts = create_test_accounts();
        let metadata = create_test_metadata();
        let event = parse_decrease_liquidity_instruction(&data, &accounts, metadata);

        assert!(event.is_ok());
    }

    #[test]
    fn test_parse_events_from_inner_instruction_when_none_block_time_should_use_zero() {
        let parser = Parser::default();
        let mut data = vec![0u8; 64];
        data.get_mut(8..16)
            .unwrap()
            .copy_from_slice(&1000u64.to_le_bytes());
        data.get_mut(16..24)
            .unwrap()
            .copy_from_slice(&2000u64.to_le_bytes());
        data.get_mut(24..40)
            .unwrap()
            .copy_from_slice(&123_456_789_u128.to_le_bytes());
        *data.get_mut(40).unwrap() = 1;
        *data.get_mut(41).unwrap() = 1;

        let encoded_data = bs58::encode(&data).into_string();
        let inner_instruction = UiCompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2],
            data: encoded_data,
            stack_height: Some(1),
        };

        let params = crate::events::factory::InnerInstructionParseParams {
            inner_instruction: &inner_instruction,
            signature: "test_sig",
            slot: 12345,
            block_time: None, // None block_time
            program_received_time_ms: 2000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_inner_instruction(&params);

        assert_eq!(events.len(), 5);
    }

    #[test]
    fn test_parse_events_from_instruction_when_none_block_time_should_use_zero() {
        let parser = Parser::default();
        let accounts = create_test_accounts();

        let mut data = vec![0u8; 64];
        data.get_mut(0..8)
            .unwrap()
            .copy_from_slice(&SWAP_DISCRIMINATOR);
        data.get_mut(8..16)
            .unwrap()
            .copy_from_slice(&1000u64.to_le_bytes());
        data.get_mut(16..24)
            .unwrap()
            .copy_from_slice(&2000u64.to_le_bytes());
        data.get_mut(24..40)
            .unwrap()
            .copy_from_slice(&123_456_789_u128.to_le_bytes());
        *data.get_mut(40).unwrap() = 1;
        *data.get_mut(41).unwrap() = 1;

        let instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            data,
        };

        let params = crate::events::factory::InstructionParseParams {
            instruction: &instruction,
            accounts: &accounts,
            signature: "test_sig",
            slot: 12345,
            block_time: None, // None block_time
            program_received_time_ms: 2000,
            index: "0".to_string(),
        };
        let events = parser.parse_events_from_instruction(&params);

        assert_eq!(events.len(), 1);
    }
}

// ================================
// Module Re-export Tests (from mod.rs)
// ================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod integration_tests {
    use super::*;
    use serde_json;
    use solana_sdk::pubkey::Pubkey;

    // Test that all public items are re-exported correctly
    #[test]
    fn events_module_reexports_event_parameters() {
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
    fn events_module_reexports_orca_swap_event() {
        let params = EventParameters::new(
            "test_id".to_owned(),
            "test_sig".to_owned(),
            12345,
            1000,
            1_000_000,
            1_000_001,
            "0".to_owned(),
        );
        let swap_data = SwapData::default();
        let event = SwapEvent::new(params, swap_data);

        assert_eq!(event.id(), "test_id");
        assert_eq!(event.metadata.signature, "test_sig");
        assert_eq!(event.metadata.slot, 12345);
        assert_eq!(event.metadata.program_received_time_ms, 1_000_001);
        assert_eq!(event.metadata.index, "0");
    }

    #[test]
    fn events_module_reexports_orca_position_event() {
        let params = EventParameters::new(
            "pos_id".to_owned(),
            "pos_sig".to_owned(),
            54321,
            2000,
            2_000_000,
            2_000_001,
            "1".to_owned(),
        );
        let position_data = PositionData::default();
        let event = PositionEvent::new(params, position_data, true);

        assert_eq!(event.id(), "pos_id");
        assert_eq!(event.metadata.signature, "pos_sig");
        assert_eq!(event.metadata.slot, 54321);
        assert!(event.is_open);
    }

    #[test]
    fn events_module_reexports_orca_liquidity_event() {
        let params = EventParameters::new(
            "liq_id".to_owned(),
            "liq_sig".to_owned(),
            98765,
            3000,
            3_000_000,
            3_000_001,
            "2".to_owned(),
        );
        let liquidity_data = LiquidityData::default();
        let event = LiquidityEvent::new(params, liquidity_data);

        assert_eq!(event.id(), "liq_id");
        assert_eq!(event.metadata.signature, "liq_sig");
        assert_eq!(event.metadata.slot, 98765);
    }

    #[test]
    fn parser_module_reexports_orca_event_parser() {
        let parser = Parser::default();
        let program_ids = parser.supported_program_ids();
        assert!(!program_ids.is_empty());
        assert!(parser.should_handle(&whirlpool_program_id()));
    }

    #[test]
    fn types_module_reexports_whirlpool_program_id() {
        let program_id = whirlpool_program_id();
        assert!(is_orca_whirlpool_program(&program_id));
    }

    #[test]
    fn types_module_reexports_discriminators() {
        // Test that discriminators are accessible
        assert_eq!(SWAP_DISCRIMINATOR.len(), 8);
        assert_eq!(OPEN_POSITION_DISCRIMINATOR.len(), 8);
        assert_eq!(CLOSE_POSITION_DISCRIMINATOR.len(), 8);
        assert_eq!(INCREASE_LIQUIDITY_DISCRIMINATOR.len(), 8);
        assert_eq!(DECREASE_LIQUIDITY_DISCRIMINATOR.len(), 8);
    }

    #[test]
    fn types_module_reexports_swap_direction() {
        let direction_a_to_b = SwapDirection::AtoB;
        let direction_b_to_a = SwapDirection::BtoA;

        assert_eq!(direction_a_to_b, SwapDirection::AtoB);
        assert_eq!(direction_b_to_a, SwapDirection::BtoA);
        assert_ne!(direction_a_to_b, direction_b_to_a);
    }

    #[test]
    fn types_module_reexports_orca_swap_data() {
        let swap_data = SwapData::default();
        assert_eq!(swap_data.whirlpool, Pubkey::default());
        assert_eq!(swap_data.amount, 0);
        assert!(!swap_data.amount_specified_is_input);
        assert!(!swap_data.a_to_b);
    }

    #[test]
    fn types_module_reexports_orca_position_data() {
        let position_data = PositionData::default();
        assert_eq!(position_data.whirlpool, Pubkey::default());
        assert_eq!(position_data.tick_lower_index, 0_i32);
        assert_eq!(position_data.tick_upper_index, 0_i32);
        assert_eq!(position_data.liquidity, 0);
    }

    #[test]
    fn types_module_reexports_orca_liquidity_data() {
        let liquidity_data = LiquidityData::default();
        assert_eq!(liquidity_data.whirlpool, Pubkey::default());
        assert_eq!(liquidity_data.liquidity_amount, 0);
        assert!(!liquidity_data.is_increase);
    }

    #[test]
    fn types_module_reexports_position_reward_info() {
        let reward_info = PositionRewardInfo::default();
        assert_eq!(reward_info.growth_inside_checkpoint, 0);
        assert_eq!(reward_info.amount_owed, 0);
    }

    #[test]
    fn types_module_reexports_whirlpool_account() {
        // Test that WhirlpoolAccount can be created (struct is public)
        let account = WhirlpoolAccount {
            whirlpools_config: Pubkey::default(),
            whirlpool_bump: [0],
            tick_spacing: 64,
            tick_spacing_seed: [0, 64],
            fee_rate: 300,
            protocol_fee_rate: 300,
            liquidity: 1_000_000,
            sqrt_price: 1_000_000_000,
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
    fn types_module_reexports_whirlpool_reward_info() {
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
    fn types_module_reexports_utility_functions() {
        // Test tick_index_to_price function
        let price = tick_index_to_price(0);
        assert!((price - 1.0_f64).abs() < f64::EPSILON);

        let price_positive = tick_index_to_price(100);
        assert!(price_positive > 1.0_f64);

        let price_negative = tick_index_to_price(-100);
        assert!(price_negative < 1.0_f64);
    }

    #[test]
    fn types_module_reexports_sqrt_price_to_price() {
        // Test sqrt_price_to_price function with standard values
        let sqrt_price = 1_u128 << 64_i32; // Square root of 1
        let price = sqrt_price_to_price(sqrt_price, 6, 6);
        assert!((price - 1.0_f64).abs() < f64::EPSILON);

        // Test with different decimal values
        let price_different_decimals = sqrt_price_to_price(sqrt_price, 9, 6);
        assert!((price_different_decimals - 1_000.0_f64).abs() < f64::EPSILON);

        let price_reverse_decimals = sqrt_price_to_price(sqrt_price, 6, 9);
        assert!((price_reverse_decimals - 0.001_f64).abs() < f64::EPSILON);
    }

    #[test]
    fn all_event_types_implement_event_trait() {
        // Test SwapEvent implements Event trait
        let params = EventParameters::default();
        let swap_data = SwapData::default();
        let mut swap_event = SwapEvent::new(params.clone(), swap_data);

        // Test trait methods
        let _swap_id = swap_event.id();
        let _swap_kind = swap_event.kind();
        let _swap_metadata = swap_event.metadata();
        let _swap_metadata_mut = swap_event.metadata_mut();
        let _swap_as_any = swap_event.as_any();
        let _swap_as_any_mut = swap_event.as_any_mut();
        let _swap_cloned = swap_event.clone_boxed();
        let _swap_json_result = swap_event.to_json();

        // Test PositionEvent implements Event trait
        let position_data = PositionData::default();
        let mut position_event = PositionEvent::new(params.clone(), position_data, true);

        let _position_id = position_event.id();
        let _position_kind = position_event.kind();
        let _position_metadata = position_event.metadata();
        let _position_metadata_mut = position_event.metadata_mut();
        let _position_as_any = position_event.as_any();
        let _position_as_any_mut = position_event.as_any_mut();
        let _position_cloned = position_event.clone_boxed();
        let _position_json_result = position_event.to_json();

        // Test LiquidityEvent implements Event trait
        let liquidity_data = LiquidityData::default();
        let mut liquidity_event = LiquidityEvent::new(params, liquidity_data);

        let _liquidity_id = liquidity_event.id();
        let _liquidity_kind = liquidity_event.kind();
        let _liquidity_metadata = liquidity_event.metadata();
        let _liquidity_metadata_mut = liquidity_event.metadata_mut();
        let _liquidity_as_any = liquidity_event.as_any();
        let _liquidity_as_any_mut = liquidity_event.as_any_mut();
        let _liquidity_cloned = liquidity_event.clone_boxed();
        let _liquidity_json_result = liquidity_event.to_json();
    }

    #[test]
    fn event_serialization_works() {
        let params = EventParameters::new(
            "test_id".to_owned(),
            "test_signature".to_owned(),
            12345,
            1000,
            1_000_000,
            1_000_001,
            "0".to_owned(),
        );

        // Test SwapEvent serialization
        let swap_data = SwapData::default();
        let swap_event = SwapEvent::new(params.clone(), swap_data);
        let swap_json = serde_json::to_string(&swap_event);
        swap_json.unwrap();

        // Test PositionEvent serialization
        let position_data = PositionData::default();
        let position_event = PositionEvent::new(params.clone(), position_data, true);
        let position_json = serde_json::to_string(&position_event);
        position_json.unwrap();

        // Test LiquidityEvent serialization
        let liquidity_data = LiquidityData::default();
        let liquidity_event = LiquidityEvent::new(params, liquidity_data);
        let liquidity_json = serde_json::to_string(&liquidity_event);
        liquidity_json.unwrap();
    }

    #[test]
    fn event_deserialization_works() {
        // Test that we can deserialize events from JSON
        let swap_json = r#"{"id":"test","signature":"sig","slot":1,"block_time":1000,"block_time_ms":1000000,"program_received_time_ms":1000001,"program_handle_time_consuming_ms":0,"index":"0","swap_data":{"whirlpool":"11111111111111111111111111111112","user":"11111111111111111111111111111112","token_mint_a":"11111111111111111111111111111112","token_mint_b":"11111111111111111111111111111112","token_vault_a":"11111111111111111111111111111112","token_vault_b":"11111111111111111111111111111112","amount":0,"amount_specified_is_input":false,"a_to_b":false,"sqrt_price_limit":"0","amount_in":0,"amount_out":0,"fee_amount":0,"tick_current_index":0,"sqrt_price":"0","liquidity":"0"},"transfer_data":[]}"#;
        let swap_event: Result<SwapEvent, _> = serde_json::from_str(swap_json);
        swap_event.unwrap();

        let position_json = r#"{"id":"test","signature":"sig","slot":1,"block_time":1000,"block_time_ms":1000000,"program_received_time_ms":1000001,"program_handle_time_consuming_ms":0,"index":"0","position_data":{"whirlpool":"11111111111111111111111111111112","position_mint":"11111111111111111111111111111112","position":"11111111111111111111111111111112","position_token_account":"11111111111111111111111111111112","position_authority":"11111111111111111111111111111112","tick_lower_index":0,"tick_upper_index":0,"liquidity":0,"fee_growth_checkpoint_a":0,"fee_growth_checkpoint_b":0,"fee_owed_a":0,"fee_owed_b":0,"reward_infos":[{"growth_inside_checkpoint":0,"amount_owed":0},{"growth_inside_checkpoint":0,"amount_owed":0},{"growth_inside_checkpoint":0,"amount_owed":0}]},"is_open":true,"transfer_data":[]}"#;
        let position_event: Result<PositionEvent, _> = serde_json::from_str(position_json);
        position_event.unwrap();

        let liquidity_json = r#"{"id":"test","signature":"sig","slot":1,"block_time":1000,"block_time_ms":1000000,"program_received_time_ms":1000001,"program_handle_time_consuming_ms":0,"index":"0","liquidity_data":{"whirlpool":"11111111111111111111111111111112","position":"11111111111111111111111111111112","position_authority":"11111111111111111111111111111112","token_mint_a":"11111111111111111111111111111112","token_mint_b":"11111111111111111111111111111112","token_vault_a":"11111111111111111111111111111112","token_vault_b":"11111111111111111111111111111112","tick_lower_index":0,"tick_upper_index":0,"liquidity_amount":0,"token_max_a":0,"token_max_b":0,"token_actual_a":0,"token_actual_b":0,"is_increase":false},"transfer_data":[]}"#;
        let liquidity_event: Result<LiquidityEvent, _> = serde_json::from_str(liquidity_json);
        liquidity_event.unwrap();
    }

    #[test]
    fn module_constants_are_accessible() {
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
    fn edge_cases_for_utility_functions() {
        // Test edge cases for tick_index_to_price
        let price_max = tick_index_to_price(i32::MAX);
        assert!(price_max.is_finite());

        let price_min = tick_index_to_price(i32::MIN);
        assert!(price_min.is_finite());
        assert!(price_min > 0.0_f64);

        // Test edge cases for sqrt_price_to_price
        let zero_sqrt_price = sqrt_price_to_price(0, 6, 6);
        assert!(zero_sqrt_price.abs() < f64::EPSILON);

        let max_sqrt_price = sqrt_price_to_price(u128::MAX, 0, 0);
        assert!(max_sqrt_price.is_finite());
    }

    #[test]
    fn all_structs_implement_required_traits() {
        // Test that all data structs implement Debug, Clone, Serialize, Deserialize
        let swap_data = SwapData::default();
        let _cloned_swap = swap_data.clone();
        let _debug_swap = format!("{swap_data:?}");

        let position_data = PositionData::default();
        let _cloned_position = position_data.clone();
        let _debug_position = format!("{position_data:?}");

        let liquidity_data = LiquidityData::default();
        let _cloned_liquidity = liquidity_data.clone();
        let _debug_liquidity = format!("{liquidity_data:?}");

        let reward_info = PositionRewardInfo::default();
        let _: PositionRewardInfo = reward_info;
        let _debug_reward = format!("{reward_info:?}");

        let swap_direction = SwapDirection::AtoB;
        let _cloned_direction = swap_direction.clone();
        let _debug_direction = format!("{swap_direction:?}");
    }
}

// Re-export core types for direct access
// EventParameters already available at module level
