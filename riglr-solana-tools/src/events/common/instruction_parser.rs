//! Common instruction data parsing utilities
//!
//! This module provides shared parsing logic for instruction-based protocols
//! including Bonk, PumpSwap, Raydium AMM V4, and Raydium CLMM.
//! 
//! These utilities reduce code duplication and provide consistent patterns
//! for parsing Solana instruction data across different protocols.

use borsh::BorshDeserialize;
use solana_sdk::pubkey::Pubkey;

use crate::events::common::{EventMetadata, read_u64_le, read_u8_le, read_u128_le, read_i32_le};
use crate::events::core::traits::UnifiedEvent;

/// Common validation for instruction data and accounts
pub struct InstructionValidator;

impl InstructionValidator {
    /// Validate minimum data length
    pub fn validate_data_length(data: &[u8], min_length: usize) -> bool {
        data.len() >= min_length
    }
    
    /// Validate minimum account count
    pub fn validate_account_count(accounts: &[Pubkey], min_count: usize) -> bool {
        accounts.len() >= min_count
    }
    
    /// Combined validation for data and accounts
    pub fn validate_instruction(data: &[u8], accounts: &[Pubkey], min_data: usize, min_accounts: usize) -> bool {
        Self::validate_data_length(data, min_data) && Self::validate_account_count(accounts, min_accounts)
    }
}

/// Common patterns for parsing swap instruction parameters
pub struct SwapInstructionParser;

impl SwapInstructionParser {
    /// Parse exact input swap parameters (amount_in, min_amount_out)
    pub fn parse_exact_in_params(data: &[u8]) -> Option<(u64, u64)> {
        if data.len() < 16 {
            return None;
        }
        let amount_in = read_u64_le(data, 0)?;
        let min_amount_out = read_u64_le(data, 8)?;
        Some((amount_in, min_amount_out))
    }
    
    /// Parse exact output swap parameters (max_amount_in, amount_out)
    pub fn parse_exact_out_params(data: &[u8]) -> Option<(u64, u64)> {
        if data.len() < 16 {
            return None;
        }
        let max_amount_in = read_u64_le(data, 0)?;
        let amount_out = read_u64_le(data, 8)?;
        Some((max_amount_in, amount_out))
    }
    
    /// Parse swap direction flag
    pub fn parse_swap_direction(data: &[u8], offset: usize) -> Option<bool> {
        let direction = read_u8_le(data, offset)?;
        Some(direction == 1)
    }
}

/// Common patterns for parsing liquidity instruction parameters
pub struct LiquidityInstructionParser;

impl LiquidityInstructionParser {
    /// Parse deposit parameters (max_base, max_quote, base_side)
    pub fn parse_deposit_params(data: &[u8]) -> Option<(u64, u64, u64)> {
        if data.len() < 24 {
            return None;
        }
        let max_base = read_u64_le(data, 0)?;
        let max_quote = read_u64_le(data, 8)?;
        let base_side = read_u64_le(data, 16)?;
        Some((max_base, max_quote, base_side))
    }
    
    /// Parse withdraw parameters (amount, min_base_out, min_quote_out)
    pub fn parse_withdraw_params(data: &[u8]) -> Option<(u64, Option<u64>, Option<u64>)> {
        if data.len() < 8 {
            return None;
        }
        let amount = read_u64_le(data, 0)?;
        
        // Optional min amounts for complex withdraw operations
        let min_base_out = if data.len() >= 16 {
            read_u64_le(data, 8)
        } else {
            None
        };
        let min_quote_out = if data.len() >= 24 {
            read_u64_le(data, 16)
        } else {
            None
        };
        
        Some((amount, min_base_out, min_quote_out))
    }
    
    /// Parse liquidity amount with position bounds for CLMM-style operations
    pub fn parse_liquidity_with_bounds(data: &[u8]) -> Option<(u128, i32, i32, u64, u64)> {
        if data.len() < 40 {
            return None;
        }
        let tick_lower = read_i32_le(data, 0)?;
        let tick_upper = read_i32_le(data, 4)?;
        let liquidity = read_u128_le(data, 8)?;
        let amount0_max = read_u64_le(data, 24)?;
        let amount1_max = read_u64_le(data, 32)?;
        Some((liquidity, tick_lower, tick_upper, amount0_max, amount1_max))
    }
}

/// Common patterns for ID generation across protocols
pub struct EventIdGenerator;

impl EventIdGenerator {
    /// Generate ID for swap events
    pub fn swap_id(signature: &str, pool: &Pubkey, amount: u64, suffix: &str) -> String {
        format!("{}-{}-{}-{}", signature, pool, suffix, amount)
    }
    
    /// Generate ID for pool creation events
    pub fn pool_creation_id(signature: &str, pool: &Pubkey, creator: &Pubkey) -> String {
        format!("{}-{}-{}", signature, pool, creator)
    }
    
    /// Generate ID for liquidity events
    pub fn liquidity_id(signature: &str, pool: &Pubkey, user: &Pubkey, amount: u64, operation: &str) -> String {
        format!("{}-{}-{}-{}-{}", signature, pool, user, operation, amount)
    }
    
    /// Generate ID for position events (CLMM-style)
    pub fn position_id(signature: &str, position_mint: &Pubkey, operation: &str) -> String {
        format!("{}-{}-{}", signature, position_mint, operation)
    }
}

/// Common borsh deserialization patterns (optional for protocols that use it)
pub struct BorshInstructionParser;

impl BorshInstructionParser {
    /// Deserialize instruction data using borsh with error handling
    /// This is mainly for protocols that emit borsh-serialized data in logs
    pub fn parse_borsh_instruction<T>(data: &[u8]) -> Option<T>
    where
        T: BorshDeserialize,
    {
        T::try_from_slice(data).ok()
    }
}

/// Account mapping utilities for consistent account extraction
pub struct AccountMapper;

impl AccountMapper {
    /// Extract standard swap accounts (user, pool, tokens)
    pub fn extract_swap_accounts(accounts: &[Pubkey]) -> Option<SwapAccounts> {
        if accounts.len() < 8 {
            return None;
        }
        Some(SwapAccounts {
            user: accounts[0],
            pool: accounts[1],
            user_source: accounts[2],
            user_destination: accounts[3],
            pool_source: accounts[4],
            pool_destination: accounts[5],
            source_mint: accounts[6],
            destination_mint: accounts[7],
        })
    }
    
    /// Extract pool creation accounts
    pub fn extract_pool_accounts(accounts: &[Pubkey]) -> Option<PoolAccounts> {
        if accounts.len() < 6 {
            return None;
        }
        Some(PoolAccounts {
            creator: accounts[0],
            pool: accounts[1],
            base_mint: accounts[2],
            quote_mint: accounts[3],
            base_vault: accounts[4],
            quote_vault: accounts[5],
        })
    }
}

/// Standard account structures
#[derive(Debug, Clone)]
pub struct SwapAccounts {
    pub user: Pubkey,
    pub pool: Pubkey,
    pub user_source: Pubkey,
    pub user_destination: Pubkey,
    pub pool_source: Pubkey,
    pub pool_destination: Pubkey,
    pub source_mint: Pubkey,
    pub destination_mint: Pubkey,
}

#[derive(Debug, Clone)]
pub struct PoolAccounts {
    pub creator: Pubkey,
    pub pool: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
}

/// Math utilities for price and tick calculations
pub struct PriceCalculator;

impl PriceCalculator {
    /// Convert sqrt price X64 to tick (used by CLMM protocols)
    pub fn sqrt_price_to_tick(sqrt_price_x64: u128) -> i32 {
        if sqrt_price_x64 == 0 {
            return 0;
        }
        
        // Convert from X64 fixed point to f64
        let two_pow_64 = 18446744073709551616.0_f64; // 2^64
        let sqrt_price = (sqrt_price_x64 as f64) / two_pow_64;
        
        // Square to get actual price
        let price = sqrt_price * sqrt_price;
        
        // Calculate tick = log1.0001(price)
        if price > 0.0 {
            (price.ln() / 1.0001_f64.ln()).round() as i32
        } else {
            0
        }
    }
    
    /// Calculate price from tick
    pub fn tick_to_price(tick: i32) -> f64 {
        1.0001_f64.powi(tick)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_instruction_validation() {
        let data = vec![0u8; 16];
        let accounts = vec![Pubkey::default(); 5];
        
        assert!(InstructionValidator::validate_instruction(&data, &accounts, 10, 3));
        assert!(!InstructionValidator::validate_instruction(&data, &accounts, 20, 3));
        assert!(!InstructionValidator::validate_instruction(&data, &accounts, 10, 10));
    }
    
    #[test]
    fn test_swap_parameter_parsing() {
        let mut data = vec![0u8; 16];
        // Write test values in little-endian format
        data[0..8].copy_from_slice(&1000u64.to_le_bytes());
        data[8..16].copy_from_slice(&500u64.to_le_bytes());
        
        let (amount_in, min_out) = SwapInstructionParser::parse_exact_in_params(&data).unwrap();
        assert_eq!(amount_in, 1000);
        assert_eq!(min_out, 500);
    }
    
    #[test]
    fn test_id_generation() {
        let sig = "test_signature";
        let pool = Pubkey::default();
        let amount = 1000;
        
        let id = EventIdGenerator::swap_id(sig, &pool, amount, "buy");
        assert!(id.contains(sig));
        assert!(id.contains("buy"));
        assert!(id.contains("1000"));
    }
    
    #[test]
    fn test_price_calculations() {
        // Test basic tick to price conversion
        let tick = 0;
        let price = PriceCalculator::tick_to_price(tick);
        assert!((price - 1.0).abs() < f64::EPSILON);
        
        // Test sqrt price conversion (with a known value)
        let sqrt_price_x64 = 1u128 << 64; // Represents sqrt(1) = 1 in X64 format
        let tick = PriceCalculator::sqrt_price_to_tick(sqrt_price_x64);
        assert_eq!(tick, 0); // log1.0001(1) = 0
    }
}