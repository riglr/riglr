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

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    #[test]
    fn test_meteora_dlmm_program_id_when_called_should_return_valid_pubkey() {
        let program_id = meteora_dlmm_program_id();
        assert_eq!(
            program_id.to_string(),
            "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo"
        );
    }

    #[test]
    fn test_meteora_dynamic_program_id_when_called_should_return_valid_pubkey() {
        let program_id = meteora_dynamic_program_id();
        assert_eq!(
            program_id.to_string(),
            "Dooar9JkhdZ7J3LHN3A7YCuoGRUggXhQaG4kijfLGU2j"
        );
    }

    #[test]
    fn test_is_meteora_dlmm_program_when_correct_pubkey_should_return_true() {
        let correct_pubkey = meteora_dlmm_program_id();
        assert!(is_meteora_dlmm_program(&correct_pubkey));
    }

    #[test]
    fn test_is_meteora_dlmm_program_when_incorrect_pubkey_should_return_false() {
        let incorrect_pubkey = Pubkey::from_str("11111111111111111111111111111112").unwrap();
        assert!(!is_meteora_dlmm_program(&incorrect_pubkey));
    }

    #[test]
    fn test_is_meteora_dynamic_program_when_correct_pubkey_should_return_true() {
        let correct_pubkey = meteora_dynamic_program_id();
        assert!(is_meteora_dynamic_program(&correct_pubkey));
    }

    #[test]
    fn test_is_meteora_dynamic_program_when_incorrect_pubkey_should_return_false() {
        let incorrect_pubkey = Pubkey::from_str("11111111111111111111111111111112").unwrap();
        assert!(!is_meteora_dynamic_program(&incorrect_pubkey));
    }

    #[test]
    fn test_bin_id_to_price_when_center_bin_should_return_one() {
        let price = bin_id_to_price(8388608, 100); // Center bin with 1% step
        assert!((price - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_bin_id_to_price_when_higher_bin_should_return_higher_price() {
        let price = bin_id_to_price(8388609, 100); // One bin above center
        assert!(price > 1.0);
        let expected = (1.0 + 0.01_f64).powi(1);
        assert!((price - expected).abs() < 0.0001);
    }

    #[test]
    fn test_bin_id_to_price_when_lower_bin_should_return_lower_price() {
        let price = bin_id_to_price(8388607, 100); // One bin below center
        assert!(price < 1.0);
        let expected = (1.0 + 0.01_f64).powi(-1);
        assert!((price - expected).abs() < 0.0001);
    }

    #[test]
    fn test_bin_id_to_price_when_zero_bin_step_should_return_one() {
        let price = bin_id_to_price(8388608, 0);
        assert!((price - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_bin_id_to_price_when_max_bin_step_should_work() {
        let price = bin_id_to_price(8388608, u16::MAX);
        assert!(price.is_finite());
    }

    #[test]
    fn test_calculate_active_bin_price_when_called_should_match_bin_id_to_price() {
        let active_id = 8388610;
        let bin_step = 50;
        let price1 = calculate_active_bin_price(active_id, bin_step);
        let price2 = bin_id_to_price(active_id, bin_step);
        assert!((price1 - price2).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_liquidity_distribution_when_equal_range_should_distribute_evenly() {
        let distribution = calculate_liquidity_distribution(1000, 2000, 100, 102, 101);
        assert_eq!(distribution.len(), 3);

        // Check bin 100 (below active): only Y tokens
        assert_eq!(distribution[0], (100, 333, 0));
        // Check bin 101 (active): both X and Y
        assert_eq!(distribution[1], (101, 333, 666));
        // Check bin 102 (above active): only X tokens
        assert_eq!(distribution[2], (102, 0, 666));
    }

    #[test]
    fn test_calculate_liquidity_distribution_when_single_bin_should_contain_all() {
        let distribution = calculate_liquidity_distribution(1000, 2000, 100, 100, 100);
        assert_eq!(distribution.len(), 1);
        assert_eq!(distribution[0], (100, 1000, 2000));
    }

    #[test]
    fn test_calculate_liquidity_distribution_when_active_below_range_should_only_have_y() {
        let distribution = calculate_liquidity_distribution(1000, 2000, 100, 102, 99);
        assert_eq!(distribution.len(), 3);

        for (_, x_amount, y_amount) in distribution {
            assert_eq!(x_amount, 0);
            assert_eq!(y_amount, 666);
        }
    }

    #[test]
    fn test_calculate_liquidity_distribution_when_active_above_range_should_only_have_x() {
        let distribution = calculate_liquidity_distribution(1000, 2000, 100, 102, 103);
        assert_eq!(distribution.len(), 3);

        for (_, x_amount, y_amount) in distribution {
            assert_eq!(x_amount, 333);
            assert_eq!(y_amount, 0);
        }
    }

    #[test]
    fn test_calculate_liquidity_distribution_when_invalid_range_should_return_empty() {
        let distribution = calculate_liquidity_distribution(1000, 2000, 102, 100, 101);
        assert!(distribution.is_empty());
    }

    #[test]
    fn test_calculate_liquidity_distribution_when_zero_amounts_should_work() {
        let distribution = calculate_liquidity_distribution(0, 0, 100, 102, 101);
        assert_eq!(distribution.len(), 3);

        for (_, x_amount, y_amount) in distribution {
            assert_eq!(x_amount, 0);
            assert_eq!(y_amount, 0);
        }
    }

    #[test]
    fn test_dlmm_bin_creation_and_serialization() {
        let bin = DlmmBin {
            bin_id: 123,
            reserve_x: 1000,
            reserve_y: 2000,
            price: 1.5,
            liquidity_supply: 50000,
        };

        // Test that the struct can be serialized and deserialized
        let serialized = serde_json::to_string(&bin).unwrap();
        let deserialized: DlmmBin = serde_json::from_str(&serialized).unwrap();

        assert_eq!(bin.bin_id, deserialized.bin_id);
        assert_eq!(bin.reserve_x, deserialized.reserve_x);
        assert_eq!(bin.reserve_y, deserialized.reserve_y);
        assert!((bin.price - deserialized.price).abs() < f64::EPSILON);
        assert_eq!(bin.liquidity_supply, deserialized.liquidity_supply);
    }

    #[test]
    fn test_dlmm_pair_config_creation_and_serialization() {
        let pair_config = DlmmPairConfig {
            pair: Pubkey::from_str("11111111111111111111111111111112").unwrap(),
            token_mint_x: Pubkey::from_str("11111111111111111111111111111113").unwrap(),
            token_mint_y: Pubkey::from_str("11111111111111111111111111111114").unwrap(),
            bin_step: 100,
            base_fee_percentage: 50,
            max_fee_percentage: 1000,
            protocol_fee_percentage: 25,
            liquidity_fee_percentage: 25,
            volatility_accumulator: 10000,
            volatility_reference: 5000,
            id_reference: 8388608,
            time_of_last_update: 1234567890,
            active_id: 8388610,
            base_key: Pubkey::from_str("11111111111111111111111111111115").unwrap(),
        };

        let serialized = serde_json::to_string(&pair_config).unwrap();
        let deserialized: DlmmPairConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(pair_config.pair, deserialized.pair);
        assert_eq!(pair_config.bin_step, deserialized.bin_step);
        assert_eq!(pair_config.active_id, deserialized.active_id);
    }

    #[test]
    fn test_meteora_swap_data_default() {
        let swap_data = MeteoraSwapData::default();
        assert_eq!(swap_data.amount_in, 0);
        assert_eq!(swap_data.min_amount_out, 0);
        assert_eq!(swap_data.actual_amount_out, 0);
        assert!(!swap_data.swap_for_y);
        assert_eq!(swap_data.active_id_before, 0);
        assert_eq!(swap_data.active_id_after, 0);
        assert_eq!(swap_data.fee_amount, 0);
        assert_eq!(swap_data.protocol_fee, 0);
        assert!(swap_data.bins_traversed.is_empty());
    }

    #[test]
    fn test_meteora_swap_data_serialization() {
        let mut swap_data = MeteoraSwapData::default();
        swap_data.amount_in = 1000;
        swap_data.swap_for_y = true;
        swap_data.bins_traversed = vec![100, 101, 102];

        let serialized = serde_json::to_string(&swap_data).unwrap();
        let deserialized: MeteoraSwapData = serde_json::from_str(&serialized).unwrap();

        assert_eq!(swap_data.amount_in, deserialized.amount_in);
        assert_eq!(swap_data.swap_for_y, deserialized.swap_for_y);
        assert_eq!(swap_data.bins_traversed, deserialized.bins_traversed);
    }

    #[test]
    fn test_meteora_liquidity_data_default() {
        let liquidity_data = MeteoraLiquidityData::default();
        assert_eq!(liquidity_data.bin_id_from, 0);
        assert_eq!(liquidity_data.bin_id_to, 0);
        assert_eq!(liquidity_data.amount_x, 0);
        assert_eq!(liquidity_data.amount_y, 0);
        assert_eq!(liquidity_data.liquidity_minted, 0);
        assert_eq!(liquidity_data.active_id, 0);
        assert!(!liquidity_data.is_add);
        assert!(liquidity_data.bins_affected.is_empty());
    }

    #[test]
    fn test_meteora_liquidity_data_serialization() {
        let mut liquidity_data = MeteoraLiquidityData::default();
        liquidity_data.is_add = true;
        liquidity_data.amount_x = 1000;
        liquidity_data.amount_y = 2000;
        liquidity_data.bins_affected = vec![DlmmBin {
            bin_id: 100,
            reserve_x: 500,
            reserve_y: 1000,
            price: 1.0,
            liquidity_supply: 25000,
        }];

        let serialized = serde_json::to_string(&liquidity_data).unwrap();
        let deserialized: MeteoraLiquidityData = serde_json::from_str(&serialized).unwrap();

        assert_eq!(liquidity_data.is_add, deserialized.is_add);
        assert_eq!(liquidity_data.amount_x, deserialized.amount_x);
        assert_eq!(liquidity_data.amount_y, deserialized.amount_y);
        assert_eq!(
            liquidity_data.bins_affected.len(),
            deserialized.bins_affected.len()
        );
        assert_eq!(
            liquidity_data.bins_affected[0].bin_id,
            deserialized.bins_affected[0].bin_id
        );
    }

    #[test]
    fn test_meteora_dynamic_pool_data_serialization() {
        let pool_data = MeteoraDynamicPoolData {
            pool: Pubkey::from_str("11111111111111111111111111111112").unwrap(),
            token_mint_a: Pubkey::from_str("11111111111111111111111111111113").unwrap(),
            token_mint_b: Pubkey::from_str("11111111111111111111111111111114").unwrap(),
            vault_a: Pubkey::from_str("11111111111111111111111111111115").unwrap(),
            vault_b: Pubkey::from_str("11111111111111111111111111111116").unwrap(),
            lp_mint: Pubkey::from_str("11111111111111111111111111111117").unwrap(),
            fee_rate: 300,
            admin_fee_rate: 50,
            trade_fee_numerator: 25,
            trade_fee_denominator: 10000,
            owner_trade_fee_numerator: 5,
            owner_trade_fee_denominator: 10000,
            owner_withdraw_fee_numerator: 0,
            owner_withdraw_fee_denominator: 10000,
            host_fee_numerator: 20,
            host_fee_denominator: 10000,
        };

        let serialized = serde_json::to_string(&pool_data).unwrap();
        let deserialized: MeteoraDynamicPoolData = serde_json::from_str(&serialized).unwrap();

        assert_eq!(pool_data.pool, deserialized.pool);
        assert_eq!(pool_data.fee_rate, deserialized.fee_rate);
        assert_eq!(
            pool_data.trade_fee_numerator,
            deserialized.trade_fee_numerator
        );
        assert_eq!(
            pool_data.trade_fee_denominator,
            deserialized.trade_fee_denominator
        );
    }

    #[test]
    fn test_meteora_dynamic_liquidity_data_default() {
        let liquidity_data = MeteoraDynamicLiquidityData::default();
        assert_eq!(liquidity_data.pool_token_amount, 0);
        assert_eq!(liquidity_data.token_a_amount, 0);
        assert_eq!(liquidity_data.token_b_amount, 0);
        assert_eq!(liquidity_data.minimum_pool_token_amount, 0);
        assert_eq!(liquidity_data.maximum_token_a_amount, 0);
        assert_eq!(liquidity_data.maximum_token_b_amount, 0);
        assert!(!liquidity_data.is_deposit);
    }

    #[test]
    fn test_meteora_dynamic_liquidity_data_serialization() {
        let mut liquidity_data = MeteoraDynamicLiquidityData::default();
        liquidity_data.is_deposit = true;
        liquidity_data.pool_token_amount = 1000;
        liquidity_data.token_a_amount = 500;
        liquidity_data.token_b_amount = 750;

        let serialized = serde_json::to_string(&liquidity_data).unwrap();
        let deserialized: MeteoraDynamicLiquidityData = serde_json::from_str(&serialized).unwrap();

        assert_eq!(liquidity_data.is_deposit, deserialized.is_deposit);
        assert_eq!(
            liquidity_data.pool_token_amount,
            deserialized.pool_token_amount
        );
        assert_eq!(liquidity_data.token_a_amount, deserialized.token_a_amount);
        assert_eq!(liquidity_data.token_b_amount, deserialized.token_b_amount);
    }

    #[test]
    fn test_constants_values() {
        assert_eq!(
            METEORA_DLMM_PROGRAM_ID,
            "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo"
        );
        assert_eq!(
            METEORA_DYNAMIC_PROGRAM_ID,
            "Dooar9JkhdZ7J3LHN3A7YCuoGRUggXhQaG4kijfLGU2j"
        );

        assert_eq!(
            DLMM_SWAP_DISCRIMINATOR,
            [0x14, 0x65, 0x32, 0x1f, 0x7a, 0x43, 0x2a, 0x9f]
        );
        assert_eq!(
            DLMM_ADD_LIQUIDITY_DISCRIMINATOR,
            [0x4c, 0x1c, 0x9b, 0x2d, 0xe3, 0x7a, 0x8b, 0x12]
        );
        assert_eq!(
            DLMM_REMOVE_LIQUIDITY_DISCRIMINATOR,
            [0xa2, 0xfd, 0x67, 0xe3, 0x45, 0x1b, 0x8c, 0x9a]
        );
        assert_eq!(
            DYNAMIC_ADD_LIQUIDITY_DISCRIMINATOR,
            [0x85, 0x72, 0x1a, 0x5f, 0x9d, 0x4e, 0x23, 0x7c]
        );
        assert_eq!(
            DYNAMIC_REMOVE_LIQUIDITY_DISCRIMINATOR,
            [0x6a, 0x8b, 0x47, 0x2e, 0x1c, 0x93, 0x5f, 0x4d]
        );
    }
}
