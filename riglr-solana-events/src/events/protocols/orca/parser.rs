use super::{
    events::{OrcaLiquidityEvent, OrcaPositionEvent, OrcaSwapEvent},
    types::{
        orca_whirlpool_program_id, OrcaLiquidityData, OrcaPositionData, OrcaSwapData,
        PositionRewardInfo, CLOSE_POSITION_DISCRIMINATOR, DECREASE_LIQUIDITY_DISCRIMINATOR,
        INCREASE_LIQUIDITY_DISCRIMINATOR, OPEN_POSITION_DISCRIMINATOR, SWAP_DISCRIMINATOR,
    },
};
use crate::{
    error::ParseResult,
    events::{
        common::utils::{
            has_discriminator, parse_u128_le, parse_u32_le, parse_u64_le, safe_get_account,
            validate_account_count, validate_data_length,
        },
        core::EventParameters,
        factory::SolanaTransactionInput,
        parser_types::{GenericEventParseConfig, ProtocolParser},
    },
    solana_metadata::{create_metadata, SolanaEventMetadata},
    types::{EventType, ProtocolType},
};

use riglr_events_core::{
    error::EventResult,
    traits::{EventParser, ParserInfo},
};

use riglr_events_core::Event;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

/// Orca Whirlpool event parser
pub struct OrcaEventParser {
    program_ids: Vec<Pubkey>,
    inner_instruction_configs: HashMap<&'static str, Vec<GenericEventParseConfig>>,
    instruction_configs: HashMap<Vec<u8>, Vec<GenericEventParseConfig>>,
}

#[async_trait::async_trait]
impl EventParser for OrcaEventParser {
    type Input = SolanaTransactionInput;

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

                            if let Ok(event) = (config.inner_instruction_parser)(&data, metadata) {
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
                                metadata,
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

    fn can_parse(&self, input: &Self::Input) -> bool {
        match input {
            SolanaTransactionInput::InnerInstruction(_) => {
                // Can always attempt to parse inner instructions for configured protocols
                !self.inner_instruction_configs.is_empty()
            }
            SolanaTransactionInput::Instruction(params) => {
                // Check if any discriminator matches
                self.instruction_configs
                    .keys()
                    .any(|discriminator| has_discriminator(&params.instruction_data, discriminator))
            }
        }
    }

    fn info(&self) -> ParserInfo {
        use riglr_events_core::EventKind;

        ParserInfo::new("orca_parser".to_string(), "1.0.0".to_string())
            .with_kind(EventKind::Swap)
            .with_kind(EventKind::Custom("position".to_string()))
            .with_kind(EventKind::Custom("liquidity".to_string()))
            .with_format("solana_instruction".to_string())
            .with_format("solana_inner_instruction".to_string())
    }
}

// Keep legacy trait implementation for backward compatibility during transition
#[async_trait::async_trait]
impl ProtocolParser for OrcaEventParser {
    fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner_instruction_configs.clone()
    }

    fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.instruction_configs.clone()
    }

    fn parse_events_from_inner_instruction(
        &self,
        params: &crate::events::factory::InnerInstructionParseParams,
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

                    if let Ok(event) = (config.inner_instruction_parser)(&data, metadata) {
                        events.push(event);
                    }
                }
            }
        }

        events
    }

    fn parse_events_from_instruction(
        &self,
        params: &crate::events::factory::InstructionParseParams,
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
                        metadata,
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

impl Default for OrcaEventParser {
    fn default() -> Self {
        let program_ids = vec![orca_whirlpool_program_id()];

        let configs = vec![
            GenericEventParseConfig {
                program_id: orca_whirlpool_program_id(),
                protocol_type: ProtocolType::OrcaWhirlpool,
                inner_instruction_discriminator: "swap",
                instruction_discriminator: &SWAP_DISCRIMINATOR,
                event_type: EventType::Swap,
                inner_instruction_parser: parse_orca_swap_inner_instruction,
                instruction_parser: parse_orca_swap_instruction,
            },
            GenericEventParseConfig {
                program_id: orca_whirlpool_program_id(),
                protocol_type: ProtocolType::OrcaWhirlpool,
                inner_instruction_discriminator: "openPosition",
                instruction_discriminator: &OPEN_POSITION_DISCRIMINATOR,
                event_type: EventType::CreatePool,
                inner_instruction_parser: parse_orca_open_position_inner_instruction,
                instruction_parser: parse_orca_open_position_instruction,
            },
            GenericEventParseConfig {
                program_id: orca_whirlpool_program_id(),
                protocol_type: ProtocolType::OrcaWhirlpool,
                inner_instruction_discriminator: "closePosition",
                instruction_discriminator: &CLOSE_POSITION_DISCRIMINATOR,
                event_type: EventType::Unknown,
                inner_instruction_parser: parse_orca_close_position_inner_instruction,
                instruction_parser: parse_orca_close_position_instruction,
            },
            GenericEventParseConfig {
                program_id: orca_whirlpool_program_id(),
                protocol_type: ProtocolType::OrcaWhirlpool,
                inner_instruction_discriminator: "increaseLiquidity",
                instruction_discriminator: &INCREASE_LIQUIDITY_DISCRIMINATOR,
                event_type: EventType::AddLiquidity,
                inner_instruction_parser: parse_orca_increase_liquidity_inner_instruction,
                instruction_parser: parse_orca_increase_liquidity_instruction,
            },
            GenericEventParseConfig {
                program_id: orca_whirlpool_program_id(),
                protocol_type: ProtocolType::OrcaWhirlpool,
                inner_instruction_discriminator: "decreaseLiquidity",
                instruction_discriminator: &DECREASE_LIQUIDITY_DISCRIMINATOR,
                event_type: EventType::RemoveLiquidity,
                inner_instruction_parser: parse_orca_decrease_liquidity_inner_instruction,
                instruction_parser: parse_orca_decrease_liquidity_instruction,
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

        Self {
            program_ids,
            inner_instruction_configs,
            instruction_configs,
        }
    }
}

// Parser functions for different Orca instruction types

fn parse_orca_swap_inner_instruction(
    data: &[u8],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let swap_data = parse_orca_swap_data(data).ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat("Failed to parse Orca swap data".to_string())
    })?;

    let params = EventParameters {
        id: metadata.id().to_string(),
        signature: crate::metadata_helpers::get_signature(&metadata.core)
            .unwrap_or("")
            .to_string(),
        slot: crate::metadata_helpers::get_slot(&metadata.core).unwrap_or(0),
        block_time: crate::metadata_helpers::get_block_time(&metadata.core).unwrap_or(0),
        block_time_ms: crate::metadata_helpers::get_block_time(&metadata.core)
            .map(|t| t * 1000)
            .unwrap_or(0),
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };

    Ok(Box::new(OrcaSwapEvent::new(params, swap_data)) as Box<dyn Event>)
}

fn parse_orca_swap_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let swap_data = parse_orca_swap_data_from_instruction(data, accounts).ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat("Failed to parse Orca swap data".to_string())
    })?;

    let params = EventParameters {
        id: metadata.id().to_string(),
        signature: crate::metadata_helpers::get_signature(&metadata.core)
            .unwrap_or("")
            .to_string(),
        slot: crate::metadata_helpers::get_slot(&metadata.core).unwrap_or(0),
        block_time: crate::metadata_helpers::get_block_time(&metadata.core).unwrap_or(0),
        block_time_ms: crate::metadata_helpers::get_block_time(&metadata.core)
            .map(|t| t * 1000)
            .unwrap_or(0),
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };

    Ok(Box::new(OrcaSwapEvent::new(params, swap_data)) as Box<dyn Event>)
}

fn parse_orca_open_position_inner_instruction(
    data: &[u8],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let position_data = parse_orca_position_data(data).ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat(
            "Failed to parse Orca position data".to_string(),
        )
    })?;

    let params = EventParameters {
        id: metadata.id().to_string(),
        signature: crate::metadata_helpers::get_signature(&metadata.core)
            .unwrap_or("")
            .to_string(),
        slot: crate::metadata_helpers::get_slot(&metadata.core).unwrap_or(0),
        block_time: crate::metadata_helpers::get_block_time(&metadata.core).unwrap_or(0),
        block_time_ms: crate::metadata_helpers::get_block_time(&metadata.core)
            .map(|t| t * 1000)
            .unwrap_or(0),
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };

    Ok(Box::new(OrcaPositionEvent::new(params, position_data, true)) as Box<dyn Event>)
}

fn parse_orca_open_position_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let position_data =
        parse_orca_position_data_from_instruction(data, accounts).ok_or_else(|| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to parse Orca position data".to_string(),
            )
        })?;

    let params = EventParameters {
        id: metadata.id().to_string(),
        signature: crate::metadata_helpers::get_signature(&metadata.core)
            .unwrap_or("")
            .to_string(),
        slot: crate::metadata_helpers::get_slot(&metadata.core).unwrap_or(0),
        block_time: crate::metadata_helpers::get_block_time(&metadata.core).unwrap_or(0),
        block_time_ms: crate::metadata_helpers::get_block_time(&metadata.core)
            .map(|t| t * 1000)
            .unwrap_or(0),
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };

    Ok(Box::new(OrcaPositionEvent::new(params, position_data, true)) as Box<dyn Event>)
}

fn parse_orca_close_position_inner_instruction(
    data: &[u8],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let position_data = parse_orca_position_data(data).ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat(
            "Failed to parse Orca position data".to_string(),
        )
    })?;

    let params = EventParameters {
        id: metadata.id().to_string(),
        signature: crate::metadata_helpers::get_signature(&metadata.core)
            .unwrap_or("")
            .to_string(),
        slot: crate::metadata_helpers::get_slot(&metadata.core).unwrap_or(0),
        block_time: crate::metadata_helpers::get_block_time(&metadata.core).unwrap_or(0),
        block_time_ms: crate::metadata_helpers::get_block_time(&metadata.core)
            .map(|t| t * 1000)
            .unwrap_or(0),
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };

    Ok(Box::new(OrcaPositionEvent::new(params, position_data, false)) as Box<dyn Event>)
}

fn parse_orca_close_position_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let position_data =
        parse_orca_position_data_from_instruction(data, accounts).ok_or_else(|| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to parse Orca position data".to_string(),
            )
        })?;

    let params = EventParameters {
        id: metadata.id().to_string(),
        signature: crate::metadata_helpers::get_signature(&metadata.core)
            .unwrap_or("")
            .to_string(),
        slot: crate::metadata_helpers::get_slot(&metadata.core).unwrap_or(0),
        block_time: crate::metadata_helpers::get_block_time(&metadata.core).unwrap_or(0),
        block_time_ms: crate::metadata_helpers::get_block_time(&metadata.core)
            .map(|t| t * 1000)
            .unwrap_or(0),
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };

    Ok(Box::new(OrcaPositionEvent::new(params, position_data, false)) as Box<dyn Event>)
}

fn parse_orca_increase_liquidity_inner_instruction(
    data: &[u8],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let liquidity_data = parse_orca_liquidity_data(data, true).ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat(
            "Failed to parse Orca liquidity data".to_string(),
        )
    })?;

    let params = EventParameters {
        id: metadata.id().to_string(),
        signature: crate::metadata_helpers::get_signature(&metadata.core)
            .unwrap_or("")
            .to_string(),
        slot: crate::metadata_helpers::get_slot(&metadata.core).unwrap_or(0),
        block_time: crate::metadata_helpers::get_block_time(&metadata.core).unwrap_or(0),
        block_time_ms: crate::metadata_helpers::get_block_time(&metadata.core)
            .map(|t| t * 1000)
            .unwrap_or(0),
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };
    Ok(Box::new(OrcaLiquidityEvent::new(params, liquidity_data)) as Box<dyn Event>)
}

fn parse_orca_increase_liquidity_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let liquidity_data = parse_orca_liquidity_data_from_instruction(data, accounts, true)
        .ok_or_else(|| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to parse Orca liquidity data".to_string(),
            )
        })?;

    let params = EventParameters {
        id: metadata.id().to_string(),
        signature: crate::metadata_helpers::get_signature(&metadata.core)
            .unwrap_or("")
            .to_string(),
        slot: crate::metadata_helpers::get_slot(&metadata.core).unwrap_or(0),
        block_time: crate::metadata_helpers::get_block_time(&metadata.core).unwrap_or(0),
        block_time_ms: crate::metadata_helpers::get_block_time(&metadata.core)
            .map(|t| t * 1000)
            .unwrap_or(0),
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };
    Ok(Box::new(OrcaLiquidityEvent::new(params, liquidity_data)) as Box<dyn Event>)
}

fn parse_orca_decrease_liquidity_inner_instruction(
    data: &[u8],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let liquidity_data = parse_orca_liquidity_data(data, false).ok_or_else(|| {
        crate::error::ParseError::InvalidDataFormat(
            "Failed to parse Orca liquidity data".to_string(),
        )
    })?;

    let params = EventParameters {
        id: metadata.id().to_string(),
        signature: crate::metadata_helpers::get_signature(&metadata.core)
            .unwrap_or("")
            .to_string(),
        slot: crate::metadata_helpers::get_slot(&metadata.core).unwrap_or(0),
        block_time: crate::metadata_helpers::get_block_time(&metadata.core).unwrap_or(0),
        block_time_ms: crate::metadata_helpers::get_block_time(&metadata.core)
            .map(|t| t * 1000)
            .unwrap_or(0),
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };
    Ok(Box::new(OrcaLiquidityEvent::new(params, liquidity_data)) as Box<dyn Event>)
}

fn parse_orca_decrease_liquidity_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let liquidity_data = parse_orca_liquidity_data_from_instruction(data, accounts, false)
        .ok_or_else(|| {
            crate::error::ParseError::InvalidDataFormat(
                "Failed to parse Orca liquidity data".to_string(),
            )
        })?;

    let params = EventParameters {
        id: metadata.id().to_string(),
        signature: crate::metadata_helpers::get_signature(&metadata.core)
            .unwrap_or("")
            .to_string(),
        slot: crate::metadata_helpers::get_slot(&metadata.core).unwrap_or(0),
        block_time: crate::metadata_helpers::get_block_time(&metadata.core).unwrap_or(0),
        block_time_ms: crate::metadata_helpers::get_block_time(&metadata.core)
            .map(|t| t * 1000)
            .unwrap_or(0),
        program_received_time_ms: metadata.program_received_time_ms,
        index: metadata.index.clone(),
    };
    Ok(Box::new(OrcaLiquidityEvent::new(params, liquidity_data)) as Box<dyn Event>)
}

// Data parsing helpers

fn parse_orca_swap_data(data: &[u8]) -> Option<OrcaSwapData> {
    validate_data_length(data, 64, "Orca swap data").ok()?;

    let mut offset = 8; // Skip discriminator

    let amount = parse_u64_le(&data[offset..offset + 8]).ok()?;
    offset += 8;

    let other_amount_threshold = parse_u64_le(&data[offset..offset + 8]).ok()?;
    offset += 8;

    let sqrt_price_limit = parse_u128_le(&data[offset..offset + 16]).ok()?;
    offset += 16;

    let amount_specified_is_input = data.get(offset)? != &0;
    offset += 1;

    let a_to_b = data.get(offset)? != &0;

    Some(OrcaSwapData {
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

fn parse_orca_swap_data_from_instruction(data: &[u8], accounts: &[Pubkey]) -> Option<OrcaSwapData> {
    let mut swap_data = parse_orca_swap_data(data)?;

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

fn parse_orca_position_data(data: &[u8]) -> Option<OrcaPositionData> {
    validate_data_length(data, 32, "Orca position data").ok()?;

    let mut offset = 8; // Skip discriminator

    let tick_lower_index = parse_u32_le(&data[offset..offset + 4]).ok()? as i32;
    offset += 4;

    let tick_upper_index = parse_u32_le(&data[offset..offset + 4]).ok()? as i32;

    Some(OrcaPositionData {
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

fn parse_orca_position_data_from_instruction(
    data: &[u8],
    accounts: &[Pubkey],
) -> Option<OrcaPositionData> {
    let mut position_data = parse_orca_position_data(data)?;

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

fn parse_orca_liquidity_data(data: &[u8], is_increase: bool) -> Option<OrcaLiquidityData> {
    validate_data_length(data, 40, "Orca liquidity data").ok()?;

    let mut offset = 8; // Skip discriminator

    let liquidity_amount = parse_u128_le(&data[offset..offset + 16]).ok()?;
    offset += 16;

    let token_max_a = parse_u64_le(&data[offset..offset + 8]).ok()?;
    offset += 8;

    let token_max_b = parse_u64_le(&data[offset..offset + 8]).ok()?;

    Some(OrcaLiquidityData {
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

fn parse_orca_liquidity_data_from_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    is_increase: bool,
) -> Option<OrcaLiquidityData> {
    let mut liquidity_data = parse_orca_liquidity_data(data, is_increase)?;

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
mod tests {
    use super::*;
    use crate::solana_metadata::SolanaEventMetadata;
    use crate::types::{EventType, ProtocolType};
    use riglr_events_core::{EventKind, EventMetadata as CoreMetadata};
    use solana_message::compiled_instruction::CompiledInstruction;
    use solana_transaction_status::UiCompiledInstruction;
    use std::str::FromStr;

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
            "0".to_string(),
            1000,
            core,
        )
    }

    fn create_test_accounts() -> Vec<Pubkey> {
        (0..15)
            .map(|i| {
                // Create valid test pubkeys by using Pubkey::new_unique() for reproducible tests
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                let hash = hasher.finish();
                let bytes = hash.to_le_bytes();
                let mut full_bytes = [0u8; 32];
                full_bytes[..8].copy_from_slice(&bytes);
                full_bytes[8] = i as u8; // Make each one unique
                Pubkey::new_from_array(full_bytes)
            })
            .collect()
    }

    #[test]
    fn test_orca_event_parser_default() {
        let parser = OrcaEventParser::default();

        assert_eq!(parser.program_ids.len(), 1);
        assert_eq!(parser.program_ids[0], orca_whirlpool_program_id());
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
        let parser = OrcaEventParser::default();
        let orca_program_id = orca_whirlpool_program_id();

        assert!(parser.should_handle(&orca_program_id));
    }

    #[test]
    fn test_should_handle_when_program_id_not_matches_should_return_false() {
        let parser = OrcaEventParser::default();
        let other_program_id = Pubkey::from_str("11111111111111111111111111111111").unwrap();

        assert!(!parser.should_handle(&other_program_id));
    }

    #[test]
    fn test_supported_program_ids() {
        let parser = OrcaEventParser::default();
        let supported_ids = parser.supported_program_ids();

        assert_eq!(supported_ids.len(), 1);
        assert_eq!(supported_ids[0], orca_whirlpool_program_id());
    }

    #[test]
    fn test_inner_instruction_configs() {
        let parser = OrcaEventParser::default();
        let configs = parser.inner_instruction_configs();

        assert_eq!(configs.len(), 5);
        assert!(configs.contains_key("swap"));
    }

    #[test]
    fn test_instruction_configs() {
        let parser = OrcaEventParser::default();
        let configs = parser.instruction_configs();

        assert_eq!(configs.len(), 5);
        assert!(configs.contains_key(&SWAP_DISCRIMINATOR.to_vec()));
    }

    #[test]
    fn test_parse_events_from_inner_instruction_when_valid_data_should_parse() {
        let parser = OrcaEventParser::default();
        let mut data = vec![0u8; 64];

        // Create valid swap data
        data[0..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]); // discriminator
        data[8..16].copy_from_slice(&1000u64.to_le_bytes()); // amount
        data[16..24].copy_from_slice(&2000u64.to_le_bytes()); // other_amount_threshold
        data[24..40].copy_from_slice(&123456789u128.to_le_bytes()); // sqrt_price_limit
        data[40] = 1; // amount_specified_is_input
        data[41] = 1; // a_to_b

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
        let parser = OrcaEventParser::default();

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
        let parser = OrcaEventParser::default();
        let accounts = create_test_accounts();

        let mut data = vec![0u8; 64];
        data[0..8].copy_from_slice(&SWAP_DISCRIMINATOR);
        data[8..16].copy_from_slice(&1000u64.to_le_bytes());
        data[16..24].copy_from_slice(&2000u64.to_le_bytes());
        data[24..40].copy_from_slice(&123456789u128.to_le_bytes());
        data[40] = 1;
        data[41] = 1;

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
        let parser = OrcaEventParser::default();
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
        data[8..16].copy_from_slice(&1000u64.to_le_bytes()); // amount
        data[16..24].copy_from_slice(&2000u64.to_le_bytes()); // other_amount_threshold
        data[24..40].copy_from_slice(&123456789u128.to_le_bytes()); // sqrt_price_limit
        data[40] = 1; // amount_specified_is_input
        data[41] = 1; // a_to_b

        let swap_data = parse_orca_swap_data(&data);

        assert!(swap_data.is_some());
        let swap_data = swap_data.unwrap();
        assert_eq!(swap_data.amount, 1000);
        assert_eq!(swap_data.sqrt_price_limit, 123456789);
        assert!(swap_data.amount_specified_is_input);
        assert!(swap_data.a_to_b);
        assert_eq!(swap_data.amount_in, 1000);
        assert_eq!(swap_data.amount_out, 2000);
    }

    #[test]
    fn test_parse_orca_swap_data_when_amount_not_input_should_swap_amounts() {
        let mut data = vec![0u8; 64];
        data[8..16].copy_from_slice(&1000u64.to_le_bytes()); // amount
        data[16..24].copy_from_slice(&2000u64.to_le_bytes()); // other_amount_threshold
        data[24..40].copy_from_slice(&123456789u128.to_le_bytes()); // sqrt_price_limit
        data[40] = 0; // amount_specified_is_input = false
        data[41] = 0; // a_to_b = false

        let swap_data = parse_orca_swap_data(&data);

        assert!(swap_data.is_some());
        let swap_data = swap_data.unwrap();
        assert!(!swap_data.amount_specified_is_input);
        assert!(!swap_data.a_to_b);
        assert_eq!(swap_data.amount_in, 2000); // swapped
        assert_eq!(swap_data.amount_out, 1000); // swapped
    }

    #[test]
    fn test_parse_orca_swap_data_when_insufficient_data_should_return_none() {
        let data = vec![0u8; 32]; // Too short

        let result = parse_orca_swap_data(&data);

        assert!(result.is_none());
    }

    #[test]
    fn test_parse_orca_swap_data_from_instruction_when_sufficient_accounts_should_extract() {
        let mut data = vec![0u8; 64];
        data[8..16].copy_from_slice(&1000u64.to_le_bytes());
        data[16..24].copy_from_slice(&2000u64.to_le_bytes());
        data[24..40].copy_from_slice(&123456789u128.to_le_bytes());
        data[40] = 1;
        data[41] = 1;

        let accounts = create_test_accounts();

        let swap_data = parse_orca_swap_data_from_instruction(&data, &accounts);

        assert!(swap_data.is_some());
        let swap_data = swap_data.unwrap();
        assert_eq!(swap_data.user, accounts[0]);
        assert_eq!(swap_data.whirlpool, accounts[1]);
        assert_eq!(swap_data.token_vault_a, accounts[3]);
        assert_eq!(swap_data.token_vault_b, accounts[4]);
    }

    #[test]
    fn test_parse_orca_swap_data_from_instruction_when_insufficient_accounts_should_use_defaults() {
        let mut data = vec![0u8; 64];
        data[8..16].copy_from_slice(&1000u64.to_le_bytes());
        data[16..24].copy_from_slice(&2000u64.to_le_bytes());
        data[24..40].copy_from_slice(&123456789u128.to_le_bytes());
        data[40] = 1;
        data[41] = 1;

        let accounts = vec![create_test_accounts()[0]]; // Only one account

        let swap_data = parse_orca_swap_data_from_instruction(&data, &accounts);

        assert!(swap_data.is_some());
        let swap_data = swap_data.unwrap();
        assert_eq!(swap_data.whirlpool, Pubkey::default());
        assert_eq!(swap_data.user, Pubkey::default());
    }

    #[test]
    fn test_parse_orca_position_data_when_valid_data_should_return_some() {
        let mut data = vec![0u8; 32];
        data[8..12].copy_from_slice(&(-100i32 as u32).to_le_bytes()); // tick_lower_index
        data[12..16].copy_from_slice(&(100i32 as u32).to_le_bytes()); // tick_upper_index

        let position_data = parse_orca_position_data(&data);

        assert!(position_data.is_some());
        let position_data = position_data.unwrap();
        assert_eq!(position_data.tick_lower_index, -100);
        assert_eq!(position_data.tick_upper_index, 100);
    }

    #[test]
    fn test_parse_orca_position_data_when_insufficient_data_should_return_none() {
        let data = vec![0u8; 16]; // Too short

        let result = parse_orca_position_data(&data);

        assert!(result.is_none());
    }

    #[test]
    fn test_parse_orca_position_data_from_instruction_when_sufficient_accounts_should_extract() {
        let mut data = vec![0u8; 32];
        data[8..12].copy_from_slice(&(-100i32 as u32).to_le_bytes());
        data[12..16].copy_from_slice(&(100i32 as u32).to_le_bytes());

        let accounts = create_test_accounts();

        let position_data = parse_orca_position_data_from_instruction(&data, &accounts);

        assert!(position_data.is_some());
        let position_data = position_data.unwrap();
        assert_eq!(position_data.position_authority, accounts[0]);
        assert_eq!(position_data.whirlpool, accounts[1]);
        assert_eq!(position_data.position, accounts[2]);
        assert_eq!(position_data.position_mint, accounts[3]);
        assert_eq!(position_data.position_token_account, accounts[4]);
    }

    #[test]
    fn test_parse_orca_position_data_from_instruction_when_insufficient_accounts_should_use_defaults(
    ) {
        let mut data = vec![0u8; 32];
        data[8..12].copy_from_slice(&(-100i32 as u32).to_le_bytes());
        data[12..16].copy_from_slice(&(100i32 as u32).to_le_bytes());

        let accounts = vec![create_test_accounts()[0]]; // Only one account

        let position_data = parse_orca_position_data_from_instruction(&data, &accounts);

        assert!(position_data.is_some());
        let position_data = position_data.unwrap();
        assert_eq!(position_data.whirlpool, Pubkey::default());
        assert_eq!(position_data.position_authority, Pubkey::default());
    }

    #[test]
    fn test_parse_orca_liquidity_data_when_valid_data_should_return_some() {
        let mut data = vec![0u8; 40];
        data[8..24].copy_from_slice(&123456789u128.to_le_bytes()); // liquidity_amount
        data[24..32].copy_from_slice(&1000u64.to_le_bytes()); // token_max_a
        data[32..40].copy_from_slice(&2000u64.to_le_bytes()); // token_max_b

        let liquidity_data = parse_orca_liquidity_data(&data, true);

        assert!(liquidity_data.is_some());
        let liquidity_data = liquidity_data.unwrap();
        assert_eq!(liquidity_data.liquidity_amount, 123456789);
        assert_eq!(liquidity_data.token_max_a, 1000);
        assert_eq!(liquidity_data.token_max_b, 2000);
        assert!(liquidity_data.is_increase);
        assert_eq!(liquidity_data.token_actual_a, 1000);
        assert_eq!(liquidity_data.token_actual_b, 2000);
    }

    #[test]
    fn test_parse_orca_liquidity_data_when_decrease_should_set_flag() {
        let mut data = vec![0u8; 40];
        data[8..24].copy_from_slice(&123456789u128.to_le_bytes());
        data[24..32].copy_from_slice(&1000u64.to_le_bytes());
        data[32..40].copy_from_slice(&2000u64.to_le_bytes());

        let liquidity_data = parse_orca_liquidity_data(&data, false);

        assert!(liquidity_data.is_some());
        let liquidity_data = liquidity_data.unwrap();
        assert!(!liquidity_data.is_increase);
    }

    #[test]
    fn test_parse_orca_liquidity_data_when_insufficient_data_should_return_none() {
        let data = vec![0u8; 20]; // Too short

        let result = parse_orca_liquidity_data(&data, true);

        assert!(result.is_none());
    }

    #[test]
    fn test_parse_orca_liquidity_data_from_instruction_when_sufficient_accounts_should_extract() {
        let mut data = vec![0u8; 40];
        data[8..24].copy_from_slice(&123456789u128.to_le_bytes());
        data[24..32].copy_from_slice(&1000u64.to_le_bytes());
        data[32..40].copy_from_slice(&2000u64.to_le_bytes());

        let accounts = create_test_accounts();

        let liquidity_data = parse_orca_liquidity_data_from_instruction(&data, &accounts, true);

        assert!(liquidity_data.is_some());
        let liquidity_data = liquidity_data.unwrap();
        assert_eq!(liquidity_data.position_authority, accounts[0]);
        assert_eq!(liquidity_data.whirlpool, accounts[1]);
        assert_eq!(liquidity_data.position, accounts[2]);
        assert_eq!(liquidity_data.token_vault_a, accounts[5]);
        assert_eq!(liquidity_data.token_vault_b, accounts[6]);
    }

    #[test]
    fn test_parse_orca_liquidity_data_from_instruction_when_insufficient_accounts_should_use_defaults(
    ) {
        let mut data = vec![0u8; 40];
        data[8..24].copy_from_slice(&123456789u128.to_le_bytes());
        data[24..32].copy_from_slice(&1000u64.to_le_bytes());
        data[32..40].copy_from_slice(&2000u64.to_le_bytes());

        let accounts = vec![create_test_accounts()[0]]; // Only one account

        let liquidity_data = parse_orca_liquidity_data_from_instruction(&data, &accounts, true);

        assert!(liquidity_data.is_some());
        let liquidity_data = liquidity_data.unwrap();
        assert_eq!(liquidity_data.whirlpool, Pubkey::default());
        assert_eq!(liquidity_data.position_authority, Pubkey::default());
    }

    #[test]
    fn test_parse_orca_swap_inner_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 64];
        data[8..16].copy_from_slice(&1000u64.to_le_bytes());
        data[16..24].copy_from_slice(&2000u64.to_le_bytes());
        data[24..40].copy_from_slice(&123456789u128.to_le_bytes());
        data[40] = 1;
        data[41] = 1;

        let metadata = create_test_metadata();
        let event = parse_orca_swap_inner_instruction(&data, metadata);

        assert!(event.is_ok());
    }

    #[test]
    fn test_parse_orca_swap_inner_instruction_when_invalid_data_should_return_none() {
        let data = vec![0u8; 10]; // Too short

        let metadata = create_test_metadata();
        let event = parse_orca_swap_inner_instruction(&data, metadata);

        assert!(event.is_err());
    }

    #[test]
    fn test_parse_orca_swap_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 64];
        data[8..16].copy_from_slice(&1000u64.to_le_bytes());
        data[16..24].copy_from_slice(&2000u64.to_le_bytes());
        data[24..40].copy_from_slice(&123456789u128.to_le_bytes());
        data[40] = 1;
        data[41] = 1;

        let accounts = create_test_accounts();
        let metadata = create_test_metadata();
        let event = parse_orca_swap_instruction(&data, &accounts, metadata);

        assert!(event.is_ok());
    }

    #[test]
    fn test_parse_orca_open_position_inner_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 32];
        data[8..12].copy_from_slice(&(-100i32 as u32).to_le_bytes());
        data[12..16].copy_from_slice(&(100i32 as u32).to_le_bytes());

        let metadata = create_test_metadata();
        let event = parse_orca_open_position_inner_instruction(&data, metadata);

        assert!(event.is_ok());
    }

    #[test]
    fn test_parse_orca_open_position_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 32];
        data[8..12].copy_from_slice(&(-100i32 as u32).to_le_bytes());
        data[12..16].copy_from_slice(&(100i32 as u32).to_le_bytes());

        let accounts = create_test_accounts();
        let metadata = create_test_metadata();
        let event = parse_orca_open_position_instruction(&data, &accounts, metadata);

        assert!(event.is_ok());
    }

    #[test]
    fn test_parse_orca_close_position_inner_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 32];
        data[8..12].copy_from_slice(&(-100i32 as u32).to_le_bytes());
        data[12..16].copy_from_slice(&(100i32 as u32).to_le_bytes());

        let metadata = create_test_metadata();
        let event = parse_orca_close_position_inner_instruction(&data, metadata);

        assert!(event.is_ok());
    }

    #[test]
    fn test_parse_orca_close_position_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 32];
        data[8..12].copy_from_slice(&(-100i32 as u32).to_le_bytes());
        data[12..16].copy_from_slice(&(100i32 as u32).to_le_bytes());

        let accounts = create_test_accounts();
        let metadata = create_test_metadata();
        let event = parse_orca_close_position_instruction(&data, &accounts, metadata);

        assert!(event.is_ok());
    }

    #[test]
    fn test_parse_orca_increase_liquidity_inner_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 40];
        data[8..24].copy_from_slice(&123456789u128.to_le_bytes());
        data[24..32].copy_from_slice(&1000u64.to_le_bytes());
        data[32..40].copy_from_slice(&2000u64.to_le_bytes());

        let metadata = create_test_metadata();
        let event = parse_orca_increase_liquidity_inner_instruction(&data, metadata);

        assert!(event.is_ok());
    }

    #[test]
    fn test_parse_orca_increase_liquidity_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 40];
        data[8..24].copy_from_slice(&123456789u128.to_le_bytes());
        data[24..32].copy_from_slice(&1000u64.to_le_bytes());
        data[32..40].copy_from_slice(&2000u64.to_le_bytes());

        let accounts = create_test_accounts();
        let metadata = create_test_metadata();
        let event = parse_orca_increase_liquidity_instruction(&data, &accounts, metadata);

        assert!(event.is_ok());
    }

    #[test]
    fn test_parse_orca_decrease_liquidity_inner_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 40];
        data[8..24].copy_from_slice(&123456789u128.to_le_bytes());
        data[24..32].copy_from_slice(&1000u64.to_le_bytes());
        data[32..40].copy_from_slice(&2000u64.to_le_bytes());

        let metadata = create_test_metadata();
        let event = parse_orca_decrease_liquidity_inner_instruction(&data, metadata);

        assert!(event.is_ok());
    }

    #[test]
    fn test_parse_orca_decrease_liquidity_instruction_when_valid_data_should_return_event() {
        let mut data = vec![0u8; 40];
        data[8..24].copy_from_slice(&123456789u128.to_le_bytes());
        data[24..32].copy_from_slice(&1000u64.to_le_bytes());
        data[32..40].copy_from_slice(&2000u64.to_le_bytes());

        let accounts = create_test_accounts();
        let metadata = create_test_metadata();
        let event = parse_orca_decrease_liquidity_instruction(&data, &accounts, metadata);

        assert!(event.is_ok());
    }

    #[test]
    fn test_parse_events_from_inner_instruction_when_none_block_time_should_use_zero() {
        let parser = OrcaEventParser::default();
        let mut data = vec![0u8; 64];
        data[8..16].copy_from_slice(&1000u64.to_le_bytes());
        data[16..24].copy_from_slice(&2000u64.to_le_bytes());
        data[24..40].copy_from_slice(&123456789u128.to_le_bytes());
        data[40] = 1;
        data[41] = 1;

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
        let parser = OrcaEventParser::default();
        let accounts = create_test_accounts();

        let mut data = vec![0u8; 64];
        data[0..8].copy_from_slice(&SWAP_DISCRIMINATOR);
        data[8..16].copy_from_slice(&1000u64.to_le_bytes());
        data[16..24].copy_from_slice(&2000u64.to_le_bytes());
        data[24..40].copy_from_slice(&123456789u128.to_le_bytes());
        data[40] = 1;
        data[41] = 1;

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
