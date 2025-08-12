use std::collections::HashMap;

use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use solana_transaction_status::UiCompiledInstruction;

use crate::events::{
    common::{EventMetadata, EventType, ProtocolType, read_i32_le, read_u64_le, read_u128_le, read_u8_le, read_u16_le, read_option_bool},
    core::traits::{EventParser, GenericEventParseConfig, GenericEventParser, UnifiedEvent},
    protocols::raydium_clmm::{
        discriminators, RaydiumClmmSwapEvent, RaydiumClmmSwapV2Event, RaydiumClmmCreatePoolEvent,
        RaydiumClmmOpenPositionV2Event, RaydiumClmmClosePositionEvent, RaydiumClmmIncreaseLiquidityV2Event,
        RaydiumClmmDecreaseLiquidityV2Event, RaydiumClmmOpenPositionWithToken22NftEvent,
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
        Self::new()
    }
}

impl RaydiumClmmEventParser {
    pub fn new() -> Self {
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

    fn empty_parse(_data: &[u8], _metadata: EventMetadata) -> Option<Box<dyn UnifiedEvent>> {
        None
    }

    /// Parse swap instruction event
    fn parse_swap_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 41 || accounts.len() < 17 {
            return None;
        }

        let amount = read_u64_le(data, 0)?;
        let other_amount_threshold = read_u64_le(data, 8)?;
        let sqrt_price_limit_x64 = read_u128_le(data, 16)?;
        let is_base_input = read_u8_le(data, 32)? == 1;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-swap-{}", metadata.signature, accounts[1], amount));

        Some(Box::new(RaydiumClmmSwapEvent {
            metadata,
            amount0: if is_base_input { amount } else { 0 },
            amount1: if !is_base_input { amount } else { 0 },
            sqrt_price_x64: sqrt_price_limit_x64,
            liquidity: 0, // Will be filled by log parsing
            tick_current: 0, // Will be filled by log parsing
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
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 42 || accounts.len() < 17 {
            return None;
        }

        let amount = read_u64_le(data, 0)?;
        let other_amount_threshold = read_u64_le(data, 8)?;
        let sqrt_price_limit_x64 = read_u128_le(data, 16)?;
        let is_base_input = read_u8_le(data, 32)? == 1;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-swap-v2-{}", metadata.signature, accounts[1], amount));

        Some(Box::new(RaydiumClmmSwapV2Event {
            metadata,
            amount0: if is_base_input { amount } else { 0 },
            amount1: if !is_base_input { amount } else { 0 },
            sqrt_price_x64: sqrt_price_limit_x64,
            liquidity: 0, // Will be filled by log parsing
            tick_current: 0, // Will be filled by log parsing
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
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 22 || accounts.len() < 10 {
            return None;
        }

        let sqrt_price_x64 = read_u128_le(data, 0)?;
        let open_time = read_u64_le(data, 16)?; // Not used in event

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-create-pool", metadata.signature, accounts[0]));

        Some(Box::new(RaydiumClmmCreatePoolEvent {
            metadata,
            sqrt_price_x64,
            tick_current: 0, // Will be calculated from sqrt_price
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
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 51 || accounts.len() < 22 {
            return None;
        }

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-{}", metadata.signature, accounts[0], accounts[1]));

        Some(Box::new(RaydiumClmmOpenPositionV2Event {
            metadata,
            tick_lower_index: read_i32_le(data, 0)?,
            tick_upper_index: read_i32_le(data, 4)?,
            tick_array_lower_start_index: read_i32_le(data, 8)?,
            tick_array_upper_start_index: read_i32_le(data, 12)?,
            liquidity: read_u128_le(data, 16)?,
            amount0_max: read_u64_le(data, 32)?,
            amount1_max: read_u64_le(data, 40)?,
            with_metadata: read_u8_le(data, 48)? == 1,
            base_flag: read_option_bool(data, &mut 49)?,
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
    ) -> Option<Box<dyn UnifiedEvent>> {
        if accounts.len() < 9 {
            return None;
        }

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-close", metadata.signature, accounts[1]));

        Some(Box::new(RaydiumClmmClosePositionEvent {
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
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 33 || accounts.len() < 15 {
            return None;
        }

        let liquidity = read_u128_le(data, 0)?;
        let amount0_max = read_u64_le(data, 16)?;
        let amount1_max = read_u64_le(data, 24)?;
        let base_flag = read_option_bool(data, &mut 32)?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-increase-{}", metadata.signature, accounts[1], liquidity));

        Some(Box::new(RaydiumClmmIncreaseLiquidityV2Event {
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
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 32 || accounts.len() < 15 {
            return None;
        }

        let liquidity = read_u128_le(data, 0)?;
        let amount0_min = read_u64_le(data, 16)?;
        let amount1_min = read_u64_le(data, 24)?;

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-decrease-{}", metadata.signature, accounts[1], liquidity));

        Some(Box::new(RaydiumClmmDecreaseLiquidityV2Event {
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
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 51 || accounts.len() < 20 {
            return None;
        }

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-{}-token22", metadata.signature, accounts[0], accounts[1]));

        Some(Box::new(RaydiumClmmOpenPositionWithToken22NftEvent {
            metadata,
            tick_lower_index: read_i32_le(data, 0)?,
            tick_upper_index: read_i32_le(data, 4)?,
            tick_array_lower_start_index: read_i32_le(data, 8)?,
            tick_array_upper_start_index: read_i32_le(data, 12)?,
            liquidity: read_u128_le(data, 16)?,
            amount0_max: read_u64_le(data, 32)?,
            amount1_max: read_u64_le(data, 40)?,
            with_metadata: read_u8_le(data, 48)? == 1,
            base_flag: read_option_bool(data, &mut 49)?,
            payer: accounts[0],
            position_nft_owner: accounts[1],
            position_nft_mint: accounts[2],
            position_nft_account: accounts[3],
            pool_state: accounts[5],
        }))
    }
}

impl EventParser for RaydiumClmmEventParser {
    fn parse_instruction(
        &self,
        instruction: &CompiledInstruction,
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        self.inner.parse_instruction(instruction, accounts, metadata)
    }

    fn parse_ui_instruction(
        &self,
        instruction: &UiCompiledInstruction,
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        self.inner.parse_ui_instruction(instruction, accounts, metadata)
    }

    fn parse_logs(&self, logs: &[String]) -> Vec<Box<dyn UnifiedEvent>> {
        self.inner.parse_logs(logs)
    }
}