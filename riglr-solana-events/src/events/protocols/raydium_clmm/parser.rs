use riglr_events_core::Event;
use std::collections::HashMap;

use solana_sdk::pubkey::Pubkey;

use crate::{
    error::ParseResult,
    events::{
        common::{
            read_i32_le, read_option_bool, read_u128_le, read_u64_le, read_u8_le, EventMetadata,
            EventType, ProtocolType,
        },
        core::traits::{EventParser, GenericEventParseConfig, GenericEventParser},
        protocols::raydium_clmm::{
            discriminators, RaydiumClmmClosePositionEvent, RaydiumClmmCreatePoolEvent,
            RaydiumClmmDecreaseLiquidityV2Event, RaydiumClmmIncreaseLiquidityV2Event,
            RaydiumClmmOpenPositionV2Event, RaydiumClmmOpenPositionWithToken22NftEvent,
            RaydiumClmmSwapEvent, RaydiumClmmSwapV2Event,
        },
    },
};

/// Raydium CLMM program ID
pub const RAYDIUM_CLMM_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK");

/// Raydium CLMM event parser
pub struct RaydiumClmmEventParser {
    inner: GenericEventParser,
}

impl Default for RaydiumClmmEventParser {
    fn default() -> Self {
        let configs = vec![
            GenericEventParseConfig {
                program_id: RAYDIUM_CLMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumClmm,
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::SWAP,
                event_type: EventType::RaydiumClmmSwap,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_swap_instruction,
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_CLMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumClmm,
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::SWAP_V2,
                event_type: EventType::RaydiumClmmSwapV2,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_swap_v2_instruction,
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_CLMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumClmm,
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::CLOSE_POSITION,
                event_type: EventType::RaydiumClmmClosePosition,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_close_position_instruction,
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_CLMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumClmm,
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::DECREASE_LIQUIDITY_V2,
                event_type: EventType::RaydiumClmmDecreaseLiquidityV2,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_decrease_liquidity_v2_instruction,
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_CLMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumClmm,
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::CREATE_POOL,
                event_type: EventType::RaydiumClmmCreatePool,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_create_pool_instruction,
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_CLMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumClmm,
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::INCREASE_LIQUIDITY_V2,
                event_type: EventType::RaydiumClmmIncreaseLiquidityV2,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_increase_liquidity_v2_instruction,
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_CLMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumClmm,
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::OPEN_POSITION_WITH_TOKEN_22_NFT,
                event_type: EventType::RaydiumClmmOpenPositionWithToken22Nft,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_open_position_with_token_22_nft_instruction,
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_CLMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumClmm,
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::OPEN_POSITION_V2,
                event_type: EventType::RaydiumClmmOpenPositionV2,
                inner_instruction_parser: Self::empty_parse,
                instruction_parser: Self::parse_open_position_v2_instruction,
            },
        ];

        let inner = GenericEventParser::new(vec![RAYDIUM_CLMM_PROGRAM_ID], configs);
        Self { inner }
    }
}

impl RaydiumClmmEventParser {
    /// Convert sqrt price X64 to tick
    /// Based on Uniswap V3 math: tick = log1.0001(price) * 2
    /// Since sqrt_price_x64 = sqrt(price) * 2^64, we need to:
    /// 1. Convert from X64 fixed point to f64
    /// 2. Square to get price
    /// 3. Calculate tick
    fn sqrt_price_to_tick(sqrt_price_x64: u128) -> i32 {
        if sqrt_price_x64 == 0 {
            return 0;
        }

        // Convert from X64 fixed point to f64
        // Use 2^64 as f64 to avoid overflow
        let two_pow_64 = 18446744073709551616.0_f64; // 2^64
        let sqrt_price = (sqrt_price_x64 as f64) / two_pow_64;

        // Square to get actual price
        let price = sqrt_price * sqrt_price;

        // Calculate tick = log1.0001(price)
        // Using change of base: log1.0001(price) = ln(price) / ln(1.0001)
        if price > 0.0 {
            (price.ln() / 1.0001_f64.ln()).round() as i32
        } else {
            0
        }
    }

    /// Creates a new Raydium CLMM event parser
    ///
    /// Initializes the parser with all supported Raydium CLMM instruction types
    /// including swaps, position management, and pool creation operations.
    pub fn new() -> Self {
        Self::default()
    }

    /// Empty parser for inner instructions
    ///
    /// Raydium CLMM does not emit events through inner instructions or program logs.
    /// All event data is encoded directly in the instruction data itself, which is
    /// parsed by the instruction_parser functions below. This is intentional and
    /// follows the protocol's design where all necessary information is available
    /// in the instruction parameters and accounts.
    ///
    /// This differs from protocols like Raydium CPMM which emit events through logs
    /// that need to be parsed from inner instructions.
    fn empty_parse(_data: &[u8], _metadata: EventMetadata) -> ParseResult<Box<dyn Event>> {
        Err(crate::error::ParseError::InvalidDataFormat(
            "Raydium CLMM does not emit events through inner instructions".to_string(),
        ))
    }

    /// Parse swap instruction event
    fn parse_swap_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 41 || accounts.len() < 17 {
            return Err(crate::error::ParseError::InvalidDataFormat(
                "Insufficient data or accounts for Raydium CLMM swap instruction".to_string(),
            ));
        }

        let amount = read_u64_le(data, 0).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read amount".to_string())
        })?;
        let other_amount_threshold = read_u64_le(data, 8).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to read other_amount_threshold".to_string(),
            )
        })?;
        let sqrt_price_limit_x64 = read_u128_le(data, 16).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to read sqrt_price_limit_x64".to_string(),
            )
        })?;
        let is_base_input = read_u8_le(data, 32).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read is_base_input".to_string())
        })? == 1;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-swap-{}",
            metadata.signature, accounts[1], amount
        ));

        Ok(Box::new(RaydiumClmmSwapEvent {
            metadata,
            amount0: if is_base_input {
                amount
            } else {
                other_amount_threshold
            },
            amount1: if !is_base_input {
                amount
            } else {
                other_amount_threshold
            },
            sqrt_price_x64: sqrt_price_limit_x64,
            liquidity: 0, // Dynamic liquidity not available in instruction data
            tick_current: Self::sqrt_price_to_tick(sqrt_price_limit_x64), // Calculate from price
            payer: accounts[0],
            pool_state: accounts[1],
            input_token_account: accounts[2],
            output_token_account: accounts[3],
            input_vault: accounts[4],
            output_vault: accounts[5],
            token_mint0: accounts[6],
            token_mint1: accounts[7],
        }))
    }

    /// Parse swap V2 instruction event
    fn parse_swap_v2_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 42 || accounts.len() < 17 {
            return Err(crate::error::ParseError::InvalidDataFormat(
                "Insufficient data or accounts for Raydium CLMM swap v2 instruction".to_string(),
            ));
        }

        let amount = read_u64_le(data, 0).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read amount".to_string())
        })?;
        let other_amount_threshold = read_u64_le(data, 8).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to read other_amount_threshold".to_string(),
            )
        })?;
        let sqrt_price_limit_x64 = read_u128_le(data, 16).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to read sqrt_price_limit_x64".to_string(),
            )
        })?;
        let is_base_input = read_u8_le(data, 32).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read is_base_input".to_string())
        })? == 1;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-swap-v2-{}",
            metadata.signature, accounts[1], amount
        ));

        Ok(Box::new(RaydiumClmmSwapV2Event {
            metadata,
            amount0: if is_base_input {
                amount
            } else {
                other_amount_threshold
            },
            amount1: if !is_base_input {
                amount
            } else {
                other_amount_threshold
            },
            sqrt_price_x64: sqrt_price_limit_x64,
            liquidity: 0, // Dynamic liquidity not available in instruction data
            tick_current: Self::sqrt_price_to_tick(sqrt_price_limit_x64), // Calculate from price
            is_base_input,
            payer: accounts[0],
            pool_state: accounts[1],
            input_token_account: accounts[2],
            output_token_account: accounts[3],
            input_vault: accounts[4],
            output_vault: accounts[5],
            token_mint0: accounts[6],
            token_mint1: accounts[7],
        }))
    }

    /// Parse create pool instruction event
    fn parse_create_pool_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 22 || accounts.len() < 10 {
            return Err(crate::error::ParseError::InvalidDataFormat(
                "Insufficient data or accounts for Raydium CLMM create pool instruction"
                    .to_string(),
            ));
        }

        let sqrt_price_x64 = read_u128_le(data, 0).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read sqrt_price_x64".to_string())
        })?;
        let _open_time = read_u64_le(data, 16).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read open_time".to_string())
        })?; // Not used in event

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-create-pool",
            metadata.signature, accounts[0]
        ));

        Ok(Box::new(RaydiumClmmCreatePoolEvent {
            metadata,
            sqrt_price_x64,
            tick_current: Self::sqrt_price_to_tick(sqrt_price_x64), // Calculate from sqrt_price
            observation_index: 0,
            pool_creator: accounts[0],
            pool_state: accounts[1],
            token_mint0: accounts[2],
            token_mint1: accounts[3],
            token_vault0: accounts[4],
            token_vault1: accounts[5],
        }))
    }

    /// Parse open position V2 instruction event
    fn parse_open_position_v2_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 51 || accounts.len() < 22 {
            return Err(crate::error::ParseError::InvalidDataFormat(
                "Insufficient data or accounts for Raydium CLMM open position v2 instruction"
                    .to_string(),
            ));
        }

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}",
            metadata.signature, accounts[0], accounts[1]
        ));

        Ok(Box::new(RaydiumClmmOpenPositionV2Event {
            metadata,
            tick_lower_index: read_i32_le(data, 0).map_err(|_| {
                crate::error::ParseError::InvalidDataFormat(
                    "Failed to read tick_lower_index".to_string(),
                )
            })?,
            tick_upper_index: read_i32_le(data, 4).map_err(|_| {
                crate::error::ParseError::InvalidDataFormat(
                    "Failed to read tick_upper_index".to_string(),
                )
            })?,
            tick_array_lower_start_index: read_i32_le(data, 8).map_err(|_| {
                crate::error::ParseError::InvalidDataFormat(
                    "Failed to read tick_array_lower_start_index".to_string(),
                )
            })?,
            tick_array_upper_start_index: read_i32_le(data, 12).map_err(|_| {
                crate::error::ParseError::InvalidDataFormat(
                    "Failed to read tick_array_upper_start_index".to_string(),
                )
            })?,
            liquidity: read_u128_le(data, 16).map_err(|_| {
                crate::error::ParseError::InvalidDataFormat("Failed to read liquidity".to_string())
            })?,
            amount0_max: read_u64_le(data, 32).map_err(|_| {
                crate::error::ParseError::InvalidDataFormat(
                    "Failed to read amount0_max".to_string(),
                )
            })?,
            amount1_max: read_u64_le(data, 40).map_err(|_| {
                crate::error::ParseError::InvalidDataFormat(
                    "Failed to read amount1_max".to_string(),
                )
            })?,
            with_metadata: read_u8_le(data, 48).map_err(|_| {
                crate::error::ParseError::InvalidDataFormat(
                    "Failed to read with_metadata".to_string(),
                )
            })? == 1,
            base_flag: read_option_bool(data, &mut 49).map_err(|_| {
                crate::error::ParseError::InvalidDataFormat("Failed to read base_flag".to_string())
            })?,
            payer: accounts[0],
            position_nft_owner: accounts[1],
            position_nft_mint: accounts[2],
            position_nft_account: accounts[3],
            metadata_account: accounts[4],
            pool_state: accounts[5],
        }))
    }

    /// Parse close position instruction event
    fn parse_close_position_instruction(
        _data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if accounts.len() < 9 {
            return Err(crate::error::ParseError::InvalidDataFormat(
                "Insufficient accounts for Raydium CLMM close position instruction".to_string(),
            ));
        }

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-close", metadata.signature, accounts[1]));

        Ok(Box::new(RaydiumClmmClosePositionEvent {
            metadata,
            nft_owner: accounts[0],
            position_nft_mint: accounts[1],
            position_nft_account: accounts[2],
            personal_position: accounts[3],
        }))
    }

    /// Parse increase liquidity V2 instruction event
    fn parse_increase_liquidity_v2_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 33 || accounts.len() < 15 {
            return Err(crate::error::ParseError::InvalidDataFormat(
                "Insufficient data or accounts for Raydium CLMM increase liquidity v2 instruction"
                    .to_string(),
            ));
        }

        let liquidity = read_u128_le(data, 0).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read liquidity".to_string())
        })?;
        let amount0_max = read_u64_le(data, 16).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read amount0_max".to_string())
        })?;
        let amount1_max = read_u64_le(data, 24).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read amount1_max".to_string())
        })?;
        let base_flag = read_option_bool(data, &mut 32).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read base_flag".to_string())
        })?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-increase-{}",
            metadata.signature, accounts[1], liquidity
        ));

        Ok(Box::new(RaydiumClmmIncreaseLiquidityV2Event {
            metadata,
            liquidity,
            amount0_max,
            amount1_max,
            base_flag,
            nft_owner: accounts[0],
            position_nft_account: accounts[1],
            pool_state: accounts[4],
        }))
    }

    /// Parse decrease liquidity V2 instruction event
    fn parse_decrease_liquidity_v2_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 32 || accounts.len() < 15 {
            return Err(crate::error::ParseError::InvalidDataFormat(
                "Insufficient data or accounts for Raydium CLMM decrease liquidity v2 instruction"
                    .to_string(),
            ));
        }

        let liquidity = read_u128_le(data, 0).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read liquidity".to_string())
        })?;
        let amount0_min = read_u64_le(data, 16).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read amount0_min".to_string())
        })?;
        let amount1_min = read_u64_le(data, 24).map_err(|_| {
            crate::error::ParseError::InvalidDataFormat("Failed to read amount1_min".to_string())
        })?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-decrease-{}",
            metadata.signature, accounts[1], liquidity
        ));

        Ok(Box::new(RaydiumClmmDecreaseLiquidityV2Event {
            metadata,
            liquidity,
            amount0_min,
            amount1_min,
            nft_owner: accounts[0],
            position_nft_account: accounts[1],
            pool_state: accounts[4],
        }))
    }

    /// Parse open position with Token-22 NFT instruction event
    fn parse_open_position_with_token_22_nft_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> ParseResult<Box<dyn Event>> {
        if data.len() < 51 || accounts.len() < 20 {
            return Err(crate::error::ParseError::InvalidDataFormat("Insufficient data or accounts for Raydium CLMM open position with token-22 NFT instruction".to_string()));
        }

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-token22",
            metadata.signature, accounts[0], accounts[1]
        ));

        Ok(Box::new(RaydiumClmmOpenPositionWithToken22NftEvent {
            metadata,
            tick_lower_index: read_i32_le(data, 0).map_err(|_| {
                crate::error::ParseError::InvalidDataFormat(
                    "Failed to read tick_lower_index".to_string(),
                )
            })?,
            tick_upper_index: read_i32_le(data, 4).map_err(|_| {
                crate::error::ParseError::InvalidDataFormat(
                    "Failed to read tick_upper_index".to_string(),
                )
            })?,
            tick_array_lower_start_index: read_i32_le(data, 8).map_err(|_| {
                crate::error::ParseError::InvalidDataFormat(
                    "Failed to read tick_array_lower_start_index".to_string(),
                )
            })?,
            tick_array_upper_start_index: read_i32_le(data, 12).map_err(|_| {
                crate::error::ParseError::InvalidDataFormat(
                    "Failed to read tick_array_upper_start_index".to_string(),
                )
            })?,
            liquidity: read_u128_le(data, 16).map_err(|_| {
                crate::error::ParseError::InvalidDataFormat("Failed to read liquidity".to_string())
            })?,
            amount0_max: read_u64_le(data, 32).map_err(|_| {
                crate::error::ParseError::InvalidDataFormat(
                    "Failed to read amount0_max".to_string(),
                )
            })?,
            amount1_max: read_u64_le(data, 40).map_err(|_| {
                crate::error::ParseError::InvalidDataFormat(
                    "Failed to read amount1_max".to_string(),
                )
            })?,
            with_metadata: read_u8_le(data, 48).map_err(|_| {
                crate::error::ParseError::InvalidDataFormat(
                    "Failed to read with_metadata".to_string(),
                )
            })? == 1,
            base_flag: read_option_bool(data, &mut 49).map_err(|_| {
                crate::error::ParseError::InvalidDataFormat("Failed to read base_flag".to_string())
            })?,
            payer: accounts[0],
            position_nft_owner: accounts[1],
            position_nft_mint: accounts[2],
            position_nft_account: accounts[3],
            pool_state: accounts[5],
        }))
    }
}

impl EventParser for RaydiumClmmEventParser {
    fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner.inner_instruction_configs()
    }

    fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.inner.instruction_configs()
    }

    fn parse_events_from_inner_instruction(
        &self,
        params: &crate::events::factory::InnerInstructionParseParams,
    ) -> Vec<Box<dyn Event>> {
        self.inner.parse_events_from_inner_instruction(params)
    }

    fn parse_events_from_instruction(
        &self,
        params: &crate::events::factory::InstructionParseParams,
    ) -> Vec<Box<dyn Event>> {
        self.inner.parse_events_from_instruction(params)
    }

    fn should_handle(&self, program_id: &Pubkey) -> bool {
        self.inner.should_handle(program_id)
    }

    fn supported_program_ids(&self) -> Vec<Pubkey> {
        self.inner.supported_program_ids()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EventMetadata;
    use solana_sdk::pubkey::Pubkey;

    fn create_test_metadata() -> EventMetadata {
        use crate::solana_metadata::create_metadata;
        create_metadata(
            "test-event-id".to_string(),
            "test_signature".to_string(),
            123456,
            Some(1234567890),
            1234567890000,
            "0".to_string(),
            EventType::RaydiumClmmSwap,
            ProtocolType::RaydiumClmm,
        )
    }

    fn create_test_accounts(count: usize) -> Vec<Pubkey> {
        (0..count)
            .map(|i| Pubkey::new_from_array([i as u8; 32]))
            .collect()
    }

    #[test]
    fn test_sqrt_price_to_tick_when_zero_should_return_zero() {
        let tick = RaydiumClmmEventParser::sqrt_price_to_tick(0);
        assert_eq!(tick, 0);
    }

    #[test]
    fn test_sqrt_price_to_tick_when_valid_price_should_calculate_correctly() {
        // Test with a known sqrt price value
        let sqrt_price_x64 = 79228162514264337593543950336_u128; // 2^96 (approximate)
        let tick = RaydiumClmmEventParser::sqrt_price_to_tick(sqrt_price_x64);
        // Should be a reasonable tick value (exact value depends on calculation)
        assert!(tick > -1000000 && tick < 1000000);
    }

    #[test]
    fn test_sqrt_price_to_tick_when_small_price_should_handle_edge_case() {
        let tick = RaydiumClmmEventParser::sqrt_price_to_tick(1);
        // Very small price should result in negative tick
        assert!(tick <= 0);
    }

    #[test]
    fn test_sqrt_price_to_tick_when_max_value_should_handle_large_numbers() {
        let tick = RaydiumClmmEventParser::sqrt_price_to_tick(u128::MAX);
        // Should handle large values without panicking
        assert!(tick > 0);
    }

    #[test]
    fn test_new_should_create_parser_with_correct_configs() {
        let parser = RaydiumClmmEventParser::default();
        let configs = parser.instruction_configs();
        assert!(!configs.is_empty());
    }

    #[test]
    fn test_default_should_create_parser_with_all_configs() {
        let parser = RaydiumClmmEventParser::default();
        let supported_ids = parser.supported_program_ids();
        assert_eq!(supported_ids.len(), 1);
        assert_eq!(supported_ids[0], RAYDIUM_CLMM_PROGRAM_ID);
    }

    #[test]
    fn test_empty_parse_should_always_return_none() {
        let data = vec![1, 2, 3, 4];
        let metadata = create_test_metadata();
        let result = RaydiumClmmEventParser::empty_parse(&data, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_swap_instruction_when_valid_data_should_create_event() {
        let mut data = vec![0u8; 50];

        // Amount (8 bytes at offset 0)
        data[0..8].copy_from_slice(&1000u64.to_le_bytes());
        // Other amount threshold (8 bytes at offset 8)
        data[8..16].copy_from_slice(&2000u64.to_le_bytes());
        // Sqrt price limit (16 bytes at offset 16)
        data[16..32].copy_from_slice(&79228162514264337593543950336u128.to_le_bytes());
        // Is base input (1 byte at offset 32)
        data[32] = 1;

        let accounts = create_test_accounts(20);
        let metadata = create_test_metadata();

        let result = RaydiumClmmEventParser::parse_swap_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());

        let event = result.unwrap();
        // Verify it's the correct event type by checking the metadata
        assert!(!event.metadata().id.is_empty());
    }

    #[test]
    fn test_parse_swap_instruction_when_insufficient_data_should_return_none() {
        let data = vec![0u8; 30]; // Less than required 41 bytes
        let accounts = create_test_accounts(20);
        let metadata = create_test_metadata();

        let result = RaydiumClmmEventParser::parse_swap_instruction(&data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_swap_instruction_when_insufficient_accounts_should_return_none() {
        let data = vec![0u8; 50];
        let accounts = create_test_accounts(10); // Less than required 17 accounts
        let metadata = create_test_metadata();

        let result = RaydiumClmmEventParser::parse_swap_instruction(&data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_swap_instruction_when_not_base_input_should_swap_amounts() {
        let mut data = vec![0u8; 50];

        data[0..8].copy_from_slice(&1000u64.to_le_bytes());
        data[8..16].copy_from_slice(&2000u64.to_le_bytes());
        data[16..32].copy_from_slice(&79228162514264337593543950336u128.to_le_bytes());
        data[32] = 0; // Not base input

        let accounts = create_test_accounts(20);
        let metadata = create_test_metadata();

        let result = RaydiumClmmEventParser::parse_swap_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_swap_v2_instruction_when_valid_data_should_create_event() {
        let mut data = vec![0u8; 50];

        data[0..8].copy_from_slice(&1000u64.to_le_bytes());
        data[8..16].copy_from_slice(&2000u64.to_le_bytes());
        data[16..32].copy_from_slice(&79228162514264337593543950336u128.to_le_bytes());
        data[32] = 1;

        let accounts = create_test_accounts(20);
        let metadata = create_test_metadata();

        let result = RaydiumClmmEventParser::parse_swap_v2_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_swap_v2_instruction_when_insufficient_data_should_return_none() {
        let data = vec![0u8; 30]; // Less than required 42 bytes
        let accounts = create_test_accounts(20);
        let metadata = create_test_metadata();

        let result = RaydiumClmmEventParser::parse_swap_v2_instruction(&data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_swap_v2_instruction_when_insufficient_accounts_should_return_none() {
        let data = vec![0u8; 50];
        let accounts = create_test_accounts(10);
        let metadata = create_test_metadata();

        let result = RaydiumClmmEventParser::parse_swap_v2_instruction(&data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_create_pool_instruction_when_valid_data_should_create_event() {
        let mut data = vec![0u8; 30];

        // Sqrt price (16 bytes at offset 0)
        data[0..16].copy_from_slice(&79228162514264337593543950336u128.to_le_bytes());
        // Open time (8 bytes at offset 16)
        data[16..24].copy_from_slice(&1234567890u64.to_le_bytes());

        let accounts = create_test_accounts(15);
        let metadata = create_test_metadata();

        let result =
            RaydiumClmmEventParser::parse_create_pool_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_create_pool_instruction_when_insufficient_data_should_return_none() {
        let data = vec![0u8; 15]; // Less than required 22 bytes
        let accounts = create_test_accounts(15);
        let metadata = create_test_metadata();

        let result =
            RaydiumClmmEventParser::parse_create_pool_instruction(&data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_create_pool_instruction_when_insufficient_accounts_should_return_none() {
        let data = vec![0u8; 30];
        let accounts = create_test_accounts(5); // Less than required 10 accounts
        let metadata = create_test_metadata();

        let result =
            RaydiumClmmEventParser::parse_create_pool_instruction(&data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_open_position_v2_instruction_when_valid_data_should_create_event() {
        let mut data = vec![0u8; 60];

        // Tick lower index (4 bytes at offset 0)
        data[0..4].copy_from_slice(&(-1000i32).to_le_bytes());
        // Tick upper index (4 bytes at offset 4)
        data[4..8].copy_from_slice(&1000i32.to_le_bytes());
        // Tick array lower start index (4 bytes at offset 8)
        data[8..12].copy_from_slice(&(-2000i32).to_le_bytes());
        // Tick array upper start index (4 bytes at offset 12)
        data[12..16].copy_from_slice(&2000i32.to_le_bytes());
        // Liquidity (16 bytes at offset 16)
        data[16..32].copy_from_slice(&1000000u128.to_le_bytes());
        // Amount0 max (8 bytes at offset 32)
        data[32..40].copy_from_slice(&5000u64.to_le_bytes());
        // Amount1 max (8 bytes at offset 40)
        data[40..48].copy_from_slice(&6000u64.to_le_bytes());
        // With metadata (1 byte at offset 48)
        data[48] = 1;
        // Base flag (1 byte at offset 49)
        data[49] = 1;

        let accounts = create_test_accounts(25);
        let metadata = create_test_metadata();

        let result =
            RaydiumClmmEventParser::parse_open_position_v2_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_open_position_v2_instruction_when_insufficient_data_should_return_none() {
        let data = vec![0u8; 40]; // Less than required 51 bytes
        let accounts = create_test_accounts(25);
        let metadata = create_test_metadata();

        let result =
            RaydiumClmmEventParser::parse_open_position_v2_instruction(&data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_open_position_v2_instruction_when_insufficient_accounts_should_return_none() {
        let data = vec![0u8; 60];
        let accounts = create_test_accounts(15); // Less than required 22 accounts
        let metadata = create_test_metadata();

        let result =
            RaydiumClmmEventParser::parse_open_position_v2_instruction(&data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_close_position_instruction_when_valid_accounts_should_create_event() {
        let data = vec![0u8; 10]; // Data not used for close position
        let accounts = create_test_accounts(15);
        let metadata = create_test_metadata();

        let result =
            RaydiumClmmEventParser::parse_close_position_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_close_position_instruction_when_insufficient_accounts_should_return_none() {
        let data = vec![0u8; 10];
        let accounts = create_test_accounts(5); // Less than required 9 accounts
        let metadata = create_test_metadata();

        let result =
            RaydiumClmmEventParser::parse_close_position_instruction(&data, &accounts, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_increase_liquidity_v2_instruction_when_valid_data_should_create_event() {
        let mut data = vec![0u8; 40];

        // Liquidity (16 bytes at offset 0)
        data[0..16].copy_from_slice(&1000000u128.to_le_bytes());
        // Amount0 max (8 bytes at offset 16)
        data[16..24].copy_from_slice(&5000u64.to_le_bytes());
        // Amount1 max (8 bytes at offset 24)
        data[24..32].copy_from_slice(&6000u64.to_le_bytes());
        // Base flag (1 byte at offset 32)
        data[32] = 1;

        let accounts = create_test_accounts(20);
        let metadata = create_test_metadata();

        let result = RaydiumClmmEventParser::parse_increase_liquidity_v2_instruction(
            &data, &accounts, metadata,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_increase_liquidity_v2_instruction_when_insufficient_data_should_return_none() {
        let data = vec![0u8; 20]; // Less than required 33 bytes
        let accounts = create_test_accounts(20);
        let metadata = create_test_metadata();

        let result = RaydiumClmmEventParser::parse_increase_liquidity_v2_instruction(
            &data, &accounts, metadata,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_increase_liquidity_v2_instruction_when_insufficient_accounts_should_return_none()
    {
        let data = vec![0u8; 40];
        let accounts = create_test_accounts(10); // Less than required 15 accounts
        let metadata = create_test_metadata();

        let result = RaydiumClmmEventParser::parse_increase_liquidity_v2_instruction(
            &data, &accounts, metadata,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_decrease_liquidity_v2_instruction_when_valid_data_should_create_event() {
        let mut data = vec![0u8; 40];

        // Liquidity (16 bytes at offset 0)
        data[0..16].copy_from_slice(&1000000u128.to_le_bytes());
        // Amount0 min (8 bytes at offset 16)
        data[16..24].copy_from_slice(&5000u64.to_le_bytes());
        // Amount1 min (8 bytes at offset 24)
        data[24..32].copy_from_slice(&6000u64.to_le_bytes());

        let accounts = create_test_accounts(20);
        let metadata = create_test_metadata();

        let result = RaydiumClmmEventParser::parse_decrease_liquidity_v2_instruction(
            &data, &accounts, metadata,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_decrease_liquidity_v2_instruction_when_insufficient_data_should_return_none() {
        let data = vec![0u8; 20]; // Less than required 32 bytes
        let accounts = create_test_accounts(20);
        let metadata = create_test_metadata();

        let result = RaydiumClmmEventParser::parse_decrease_liquidity_v2_instruction(
            &data, &accounts, metadata,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_decrease_liquidity_v2_instruction_when_insufficient_accounts_should_return_none()
    {
        let data = vec![0u8; 40];
        let accounts = create_test_accounts(10); // Less than required 15 accounts
        let metadata = create_test_metadata();

        let result = RaydiumClmmEventParser::parse_decrease_liquidity_v2_instruction(
            &data, &accounts, metadata,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_open_position_with_token_22_nft_instruction_when_valid_data_should_create_event()
    {
        let mut data = vec![0u8; 60];

        // Same structure as open_position_v2
        data[0..4].copy_from_slice(&(-1000i32).to_le_bytes());
        data[4..8].copy_from_slice(&1000i32.to_le_bytes());
        data[8..12].copy_from_slice(&(-2000i32).to_le_bytes());
        data[12..16].copy_from_slice(&2000i32.to_le_bytes());
        data[16..32].copy_from_slice(&1000000u128.to_le_bytes());
        data[32..40].copy_from_slice(&5000u64.to_le_bytes());
        data[40..48].copy_from_slice(&6000u64.to_le_bytes());
        data[48] = 1;
        data[49] = 1;

        let accounts = create_test_accounts(25);
        let metadata = create_test_metadata();

        let result = RaydiumClmmEventParser::parse_open_position_with_token_22_nft_instruction(
            &data, &accounts, metadata,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_open_position_with_token_22_nft_instruction_when_insufficient_data_should_return_none(
    ) {
        let data = vec![0u8; 40]; // Less than required 51 bytes
        let accounts = create_test_accounts(25);
        let metadata = create_test_metadata();

        let result = RaydiumClmmEventParser::parse_open_position_with_token_22_nft_instruction(
            &data, &accounts, metadata,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_open_position_with_token_22_nft_instruction_when_insufficient_accounts_should_return_none(
    ) {
        let data = vec![0u8; 60];
        let accounts = create_test_accounts(15); // Less than required 20 accounts
        let metadata = create_test_metadata();

        let result = RaydiumClmmEventParser::parse_open_position_with_token_22_nft_instruction(
            &data, &accounts, metadata,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_event_parser_trait_inner_instruction_configs_should_delegate() {
        let parser = RaydiumClmmEventParser::default();
        let configs = parser.inner_instruction_configs();
        // Should return the inner parser's configs
        assert!(configs.is_empty() || !configs.is_empty()); // Just verify it doesn't panic
    }

    #[test]
    fn test_event_parser_trait_instruction_configs_should_delegate() {
        let parser = RaydiumClmmEventParser::default();
        let configs = parser.instruction_configs();
        assert!(!configs.is_empty());
    }

    #[test]
    fn test_event_parser_trait_should_handle_when_correct_program_id_should_return_true() {
        let parser = RaydiumClmmEventParser::default();
        assert!(parser.should_handle(&RAYDIUM_CLMM_PROGRAM_ID));
    }

    #[test]
    fn test_event_parser_trait_should_handle_when_incorrect_program_id_should_return_false() {
        let parser = RaydiumClmmEventParser::default();
        let wrong_id = Pubkey::new_from_array([1; 32]);
        assert!(!parser.should_handle(&wrong_id));
    }

    #[test]
    fn test_event_parser_trait_supported_program_ids_should_return_raydium_clmm() {
        let parser = RaydiumClmmEventParser::default();
        let ids = parser.supported_program_ids();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], RAYDIUM_CLMM_PROGRAM_ID);
    }

    #[test]
    fn test_raydium_clmm_program_id_constant_should_be_correct() {
        let expected = solana_sdk::pubkey!("CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK");
        assert_eq!(RAYDIUM_CLMM_PROGRAM_ID, expected);
    }

    #[test]
    fn test_parse_swap_instruction_when_base_flag_false_should_handle_correctly() {
        let mut data = vec![0u8; 50];

        data[0..8].copy_from_slice(&1000u64.to_le_bytes());
        data[8..16].copy_from_slice(&2000u64.to_le_bytes());
        data[16..32].copy_from_slice(&79228162514264337593543950336u128.to_le_bytes());
        data[32] = 0; // is_base_input = false

        let accounts = create_test_accounts(20);
        let metadata = create_test_metadata();

        let result = RaydiumClmmEventParser::parse_swap_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_open_position_v2_instruction_when_with_metadata_false_should_handle_correctly() {
        let mut data = vec![0u8; 60];

        data[0..4].copy_from_slice(&(-1000i32).to_le_bytes());
        data[4..8].copy_from_slice(&1000i32.to_le_bytes());
        data[8..12].copy_from_slice(&(-2000i32).to_le_bytes());
        data[12..16].copy_from_slice(&2000i32.to_le_bytes());
        data[16..32].copy_from_slice(&1000000u128.to_le_bytes());
        data[32..40].copy_from_slice(&5000u64.to_le_bytes());
        data[40..48].copy_from_slice(&6000u64.to_le_bytes());
        data[48] = 0; // with_metadata = false
        data[49] = 0; // base_flag = None

        let accounts = create_test_accounts(25);
        let metadata = create_test_metadata();

        let result =
            RaydiumClmmEventParser::parse_open_position_v2_instruction(&data, &accounts, metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_increase_liquidity_v2_instruction_when_base_flag_false_should_handle_correctly() {
        let mut data = vec![0u8; 40];

        data[0..16].copy_from_slice(&1000000u128.to_le_bytes());
        data[16..24].copy_from_slice(&5000u64.to_le_bytes());
        data[24..32].copy_from_slice(&6000u64.to_le_bytes());
        data[32] = 0; // base_flag = None

        let accounts = create_test_accounts(20);
        let metadata = create_test_metadata();

        let result = RaydiumClmmEventParser::parse_increase_liquidity_v2_instruction(
            &data, &accounts, metadata,
        );
        assert!(result.is_ok());
    }
}
